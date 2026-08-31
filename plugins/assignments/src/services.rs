//! Business logic: assignments, submissions (with deadline enforcement and
//! late handling), resources, grading, rubrics.
//!
//! The pure decision functions at the top (`is_late`, `validate_submission`,
//! `apply_late_penalty`, `validate_content`) are unit-tested without a
//! database; the async service functions wrap them with persistence.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authz::Claims;
use crate::dto::{
    AssignmentList, AssignmentView, AttachResourceRequest, CreateAssignmentRequest, DownloadView,
    FileView, GradeHistoryEntry, GradeHistoryView, GradeRequest, GradeView, LinkView,
    LateSubmissionPolicy, ResourceView, RubricList, RubricView, SubmissionType, SubmissionView,
    SubmitRequest, UpdateAssignmentRequest,
};
use crate::models::{Assignment, Submission, SubmissionFile, SubmissionLink};
use crate::{authz, repository, storage};

// ---------------------------------------------------------------------------
// Pure decision logic (unit-tested)
// ---------------------------------------------------------------------------

/// Whether `now` is after the assignment deadline.
pub fn is_late(now: DateTime<Utc>, deadline: Option<DateTime<Utc>>) -> bool {
    match deadline {
        Some(dl) => now > dl,
        None => false,
    }
}

/// The result of checking whether a submission is permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubmissionDecision {
    pub is_late: bool,
}

/// Validate that a submission is allowed right now.
///
/// * Rejects when the student has used up `allowed_attempts`.
/// * Rejects after the deadline when the late policy is `disallowed`.
/// * Otherwise allows the submission, reporting whether it is late.
pub fn validate_submission(
    now: DateTime<Utc>,
    deadline: Option<DateTime<Utc>>,
    attempts_used: i64,
    allowed_attempts: i32,
    policy: LateSubmissionPolicy,
) -> Result<SubmissionDecision> {
    if attempts_used >= allowed_attempts as i64 {
        return Err(anyhow!(
            "no attempts remaining ({} of {} used)",
            attempts_used,
            allowed_attempts
        ));
    }

    let late = is_late(now, deadline);
    if late && policy == LateSubmissionPolicy::Disallowed {
        return Err(anyhow!("deadline has passed and late submissions are disallowed"));
    }

    Ok(SubmissionDecision { is_late: late })
}

/// Compute the final marks after applying the late penalty.
///
/// Penalty is a percentage of `max_marks` deducted from the raw marks, never
/// going below zero.
pub fn apply_late_penalty(
    raw_marks: f64,
    max_marks: f64,
    is_late: bool,
    policy: LateSubmissionPolicy,
    late_penalty_percent: f64,
) -> f64 {
    if !is_late || policy != LateSubmissionPolicy::AllowedWithPenalty {
        return raw_marks;
    }
    let deduction = max_marks * (late_penalty_percent / 100.0);
    (raw_marks - deduction).max(0.0)
}

/// Validate that the provided content matches the assignment's submission
/// type (file / text / link / any).
pub fn validate_content(
    submission_type: SubmissionType,
    text: &Option<String>,
    file_count: usize,
    link_count: usize,
) -> Result<()> {
    let has_text = text.as_deref().map(|t| !t.trim().is_empty()).unwrap_or(false);
    let has_file = file_count > 0;
    let has_link = link_count > 0;

    match submission_type {
        SubmissionType::File if !has_file => Err(anyhow!("this assignment requires a file submission")),
        SubmissionType::Text if !has_text => Err(anyhow!("this assignment requires a text submission")),
        SubmissionType::Link if !has_link => Err(anyhow!("this assignment requires a link submission")),
        SubmissionType::Any if !(has_text || has_file || has_link) => {
            Err(anyhow!("submission must include text, a file, or a link"))
        }
        _ => Ok(()),
    }
}

// ---------------------------------------------------------------------------
// View helpers
// ---------------------------------------------------------------------------

fn to_assignment_view(a: &Assignment) -> AssignmentView {
    AssignmentView {
        id: a.id,
        title: a.title.clone(),
        instructions: a.instructions.clone(),
        course_id: a.course_id,
        created_by: a.created_by,
        deadline: a.deadline,
        max_marks: a.max_marks,
        allowed_attempts: a.allowed_attempts,
        submission_type: a.submission_type.clone(),
        late_submission_policy: a.late_submission_policy.clone(),
        late_penalty_percent: a.late_penalty_percent,
        status: a.status.clone(),
        created_at: a.created_at,
        updated_at: a.updated_at,
    }
}

async fn to_submission_view(
    pool: &PgPool,
    s: &Submission,
) -> Result<SubmissionView> {
    let files = repository::list_submission_files(pool, s.id).await?;
    let links = repository::list_submission_links(pool, s.id).await?;

    Ok(SubmissionView {
        id: s.id,
        assignment_id: s.assignment_id,
        student_id: s.student_id,
        attempt: s.attempt,
        text: s.text_content.clone(),
        files: files.iter().map(to_file_view).collect(),
        links: links.iter().map(to_link_view).collect(),
        submitted_at: s.submitted_at,
        is_late: s.is_late,
        status: s.status.clone(),
        graded: s.graded,
    })
}

fn to_file_view(f: &SubmissionFile) -> FileView {
    FileView {
        id: f.id,
        name: f.name.clone(),
        mime_type: f.mime_type.clone(),
        size_bytes: f.size_bytes,
        content_hash: f.content_hash.clone(),
    }
}

fn to_link_view(l: &SubmissionLink) -> LinkView {
    LinkView {
        id: l.id,
        url: l.url.clone(),
        title: l.title.clone(),
    }
}

// ---------------------------------------------------------------------------
// Assignments (instructor-only mutations)
// ---------------------------------------------------------------------------

/// Validate that the caller may attach an assignment to `course_id`.
///
/// The `course_management` plugin owns the `courses`/`course_instructors`
/// tables in the shared database. This links the two plugins: an assignment
/// may only be created for a course the instructor owns or is authorized to
/// manage. If course_management has not initialized yet (tables missing), the
/// check is deferred so assignment creation still works in a fresh database.
pub async fn validate_course_manage(
    pool: &PgPool,
    claims: &Claims,
    course_id: Option<Uuid>,
) -> Result<()> {
    let Some(course_id) = course_id else { return Ok(()) };

    let table_exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.courses')::text")
            .fetch_optional(pool)
            .await?;
    if table_exists.is_none() {
        return Ok(());
    }

    let owner: Option<(Uuid,)> = sqlx::query_as("SELECT owner_id FROM courses WHERE id = $1")
        .bind(course_id)
        .fetch_optional(pool)
        .await?;
    let owner_id = owner
        .map(|(o,)| o)
        .ok_or_else(|| anyhow!("course not found"))?;

    let me = claims.user_id()?;
    if claims.is_admin() || owner_id == me {
        return Ok(());
    }
    let authorized: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM course_instructors WHERE course_id = $1 AND user_id = $2)",
    )
    .bind(course_id)
    .bind(me)
    .fetch_one(pool)
    .await?;
    if !authorized {
        return Err(anyhow!("forbidden: you do not manage course {course_id}"));
    }
    Ok(())
}

/// Validate that the caller may **access content** of a course-linked
/// assignment: either they manage the course (owner/instructor/admin) or they
/// hold an **active** enrollment in it (the enrollments plugin owns
/// `course_enrollments`). If the enrollments plugin has not initialized yet
/// (table missing), the check is deferred so submissions still work in a
/// fresh database.
pub async fn validate_course_access(
    pool: &PgPool,
    claims: &Claims,
    course_id: Uuid,
) -> Result<()> {
    let me = claims.user_id()?;

    // Managers (course owner, co-instructors, admins) always have access.
    let owner: Option<(Uuid,)> =
        sqlx::query_as("SELECT owner_id FROM courses WHERE id = $1")
            .bind(course_id)
            .fetch_optional(pool)
            .await?;
    if let Some((owner_id,)) = owner {
        if claims.is_admin() || owner_id == me {
            return Ok(());
        }
        let manages: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM course_instructors WHERE course_id = $1 AND user_id = $2)",
        )
        .bind(course_id)
        .bind(me)
        .fetch_one(pool)
        .await?;
        if manages {
            return Ok(());
        }
    }

    // Otherwise an active enrollment is required.
    let table_exists: Option<String> =
        sqlx::query_scalar("SELECT to_regclass('public.course_enrollments')::text")
            .fetch_optional(pool)
            .await?;
    if table_exists.is_none() {
        // enrollments plugin not initialized yet — defer.
        return Ok(());
    }

    let state: Option<(String,)> = sqlx::query_as(
        "SELECT state FROM course_enrollments WHERE course_id = $1 AND student_id = $2",
    )
    .bind(course_id)
    .bind(me)
    .fetch_optional(pool)
    .await?;

    match state {
        Some((state,)) if state == "active" => Ok(()),
        Some((state,)) => Err(anyhow!(
            "forbidden: your enrollment in this course is '{state}' — only active enrollments can access course content"
        )),
        None => Err(anyhow!(
            "forbidden: you are not enrolled in this course"
        )),
    }
}



pub async fn create_assignment(
    pool: &PgPool,
    claims: &Claims,
    req: CreateAssignmentRequest,
) -> Result<AssignmentView> {
    authz::require_instructor(claims)?;
    let creator = claims.user_id()?;

    if req.title.trim().is_empty() {
        return Err(anyhow!("assignment title must not be empty"));
    }
    if req.max_marks < 0.0 {
        return Err(anyhow!("max marks must not be negative"));
    }
    if req.allowed_attempts < 1 {
        return Err(anyhow!("allowed attempts must be at least 1"));
    }
    if req.late_penalty_percent < 0.0 || req.late_penalty_percent > 100.0 {
        return Err(anyhow!("late penalty must be between 0 and 100"));
    }
    // Link: only attach assignments to a course the instructor manages.
    validate_course_manage(pool, claims, req.course_id).await?;

    let assignment = repository::create_assignment(
        pool,
        req.title.trim(),
        req.instructions.trim(),
        req.course_id,
        creator,
        req.deadline,
        req.max_marks,
        req.allowed_attempts,
        req.submission_type.as_str(),
        req.late_submission_policy.as_str(),
        req.late_penalty_percent,
    )
    .await?;

    Ok(to_assignment_view(&assignment))
}

pub async fn update_assignment(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateAssignmentRequest,
) -> Result<AssignmentView> {
    authz::require_instructor(claims)?;

    let assignment = repository::find_assignment_by_id(pool, req.assignment_id)
        .await?
        .ok_or_else(|| anyhow!("assignment not found"))?;

    if let Some(max_marks) = req.max_marks {
        if max_marks < 0.0 {
            return Err(anyhow!("max marks must not be negative"));
        }
    }
    if let Some(attempts) = req.allowed_attempts {
        if attempts < 1 {
            return Err(anyhow!("allowed attempts must be at least 1"));
        }
    }
    if let Some(penalty) = req.late_penalty_percent {
        if !(0.0..=100.0).contains(&penalty) {
            return Err(anyhow!("late penalty must be between 0 and 100"));
        }
    }

    let updated = repository::update_assignment_fields(
        pool,
        assignment.id,
        req.title.as_deref(),
        req.instructions.as_deref(),
        req.deadline,
        req.max_marks,
        req.allowed_attempts,
        req.submission_type.map(|t| t.as_str()),
        req.late_submission_policy.map(|p| p.as_str()),
        req.late_penalty_percent,
    )
    .await?;

    Ok(to_assignment_view(&updated))
}

pub async fn delete_assignment(pool: &PgPool, claims: &Claims, assignment_id: Uuid) -> Result<()> {
    authz::require_instructor(claims)?;
    if !repository::delete_assignment(pool, assignment_id).await? {
        return Err(anyhow!("assignment not found"));
    }
    Ok(())
}

pub async fn get_assignment(pool: &PgPool, assignment_id: Uuid) -> Result<AssignmentView> {
    let assignment = repository::find_assignment_by_id(pool, assignment_id)
        .await?
        .ok_or_else(|| anyhow!("assignment not found"))?;
    Ok(to_assignment_view(&assignment))
}

pub async fn list_assignments(pool: &PgPool) -> Result<AssignmentList> {
    let assignments = repository::list_assignments(pool).await?;
    Ok(AssignmentList {
        assignments: assignments.iter().map(to_assignment_view).collect(),
    })
}


// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Decode base64 file content with a sane size pre-check.
fn decode_file_bytes(content_base64: &str) -> Result<Vec<u8>> {
    if content_base64.len() > storage::max_file_bytes() * 4 / 3 + 8 {
        return Err(anyhow!("file exceeds the size limit"));
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(content_base64.trim())
        .map_err(|e| anyhow!("invalid base64 file content: {e}"))
}

fn encode_base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub async fn attach_resource(
    pool: &PgPool,
    claims: &Claims,
    req: AttachResourceRequest,
) -> Result<ResourceView> {
    authz::require_instructor(claims)?;
    let uploader = claims.user_id()?;

    let assignment = repository::find_assignment_by_id(pool, req.assignment_id)
        .await?
        .ok_or_else(|| anyhow!("assignment not found"))?;

    let name = storage::sanitize_filename(&req.name)?;
    let bytes = decode_file_bytes(&req.content_base64)?;
    let stored = storage::store_file(assignment.id, &bytes)?;

    let mime = if req.mime_type.trim().is_empty() {
        "application/octet-stream"
    } else {
        req.mime_type.trim()
    };

    let resource = repository::insert_resource(
        pool,
        assignment.id,
        &name,
        &stored.stored_name,
        mime,
        stored.size_bytes,
        &stored.content_hash,
        uploader,
    )
    .await?;

    Ok(ResourceView {
        id: resource.id,
        assignment_id: resource.assignment_id,
        name: resource.name,
        mime_type: resource.mime_type,
        size_bytes: resource.size_bytes,
        content_hash: resource.content_hash,
        created_by: resource.created_by,
        created_at: resource.created_at,
    })
}

pub async fn remove_resource(pool: &PgPool, claims: &Claims, resource_id: Uuid) -> Result<()> {
    authz::require_instructor(claims)?;
    let resource = repository::find_resource_by_id(pool, resource_id)
        .await?
        .ok_or_else(|| anyhow!("resource not found"))?;
    let _ = storage::delete_file(resource.assignment_id, &resource.stored_name);
    repository::delete_resource(pool, resource.id).await?;
    Ok(())
}

/// Instructors and students may download attached resources.
pub async fn download_resource(
    pool: &PgPool,
    claims: &Claims,
    resource_id: Uuid,
) -> Result<DownloadView> {
    let resource = repository::find_resource_by_id(pool, resource_id)
        .await?
        .ok_or_else(|| anyhow!("resource not found"))?;

    if !claims.is_instructor() {
        authz::require_student(claims)?;
    }

    let bytes = storage::read_file(resource.assignment_id, &resource.stored_name)?;
    Ok(DownloadView {
        name: resource.name.clone(),
        mime_type: resource.mime_type.clone(),
        size_bytes: resource.size_bytes,
        content_hash: resource.content_hash.clone(),
        content_base64: encode_base64(&bytes),
    })
}


// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

/// Submit (or resubmit) an attempt for an assignment. Resubmission is just a
/// new attempt; `validate_submission` enforces the attempt limit and deadline.
pub async fn submit(
    pool: &PgPool,
    claims: &Claims,
    req: SubmitRequest,
) -> Result<SubmissionView> {
    authz::require_student(claims)?;
    let student_id = claims.user_id()?;

    let assignment = repository::find_assignment_by_id(pool, req.assignment_id)
        .await?
        .ok_or_else(|| anyhow!("assignment not found"))?;

    // Course-linked assignments require an active enrollment (or manager
    // rights) — unenrolled/suspended users cannot take them.
    if let Some(course_id) = assignment.course_id {
        validate_course_access(pool, claims, course_id).await?;
    }

    let submission_type = SubmissionType::from_str(&assignment.submission_type)
        .unwrap_or(SubmissionType::Any);
    let policy = LateSubmissionPolicy::from_str(&assignment.late_submission_policy)
        .unwrap_or(LateSubmissionPolicy::AllowedWithPenalty);

    validate_content(submission_type, &req.text, req.files.len(), req.links.len())?;

    let attempts_used = repository::count_submissions_for_student(pool, assignment.id, student_id)
        .await?;
    let decision = validate_submission(
        Utc::now(),
        assignment.deadline,
        attempts_used,
        assignment.allowed_attempts,
        policy,
    )?;

    // Persist the submission attempt.
    let submission = repository::insert_submission(
        pool,
        assignment.id,
        student_id,
        (attempts_used + 1) as i32,
        req.text.as_deref(),
        decision.is_late,
    )
    .await?;

    // Store any uploaded files securely.
    for upload in &req.files {
        let name = storage::sanitize_filename(&upload.name)?;
        let bytes = decode_file_bytes(&upload.content_base64)?;
        let stored = storage::store_file(assignment.id, &bytes)?;
        let mime = if upload.mime_type.trim().is_empty() {
            "application/octet-stream"
        } else {
            upload.mime_type.trim()
        };
        repository::insert_submission_file(
            pool,
            submission.id,
            &name,
            &stored.stored_name,
            mime,
            stored.size_bytes,
            &stored.content_hash,
        )
        .await?;
    }

    for link in &req.links {
        if link.url.trim().is_empty() {
            return Err(anyhow!("submission link url must not be empty"));
        }
        repository::insert_submission_link(pool, submission.id, link.url.trim(), link.title.trim())
            .await?;
    }

    Ok(to_submission_view(pool, &submission).await?)
}

/// The caller's latest submission for an assignment (student) or all
/// submissions (instructor).
pub async fn submission_status(
    pool: &PgPool,
    claims: &Claims,
    assignment_id: Uuid,
) -> Result<serde_json::Value> {
    let me = claims.user_id()?;

    if claims.is_instructor() {
        let submissions = repository::list_submissions_for_assignment(pool, assignment_id).await?;
        let mut views = Vec::new();
        for s in &submissions {
            views.push(to_submission_view(pool, s).await?);
        }
        return Ok(serde_json::to_value(views)?);
    }

    authz::require_student(claims)?;
    let submissions = repository::list_submissions_for_student(pool, assignment_id, me).await?;
    match submissions.first() {
        Some(latest) => Ok(serde_json::to_value(to_submission_view(pool, latest).await?)?),
        None => Err(anyhow!("no submission found for this assignment")),
    }
}

/// Download a submitted file. Students may only download their own files;
/// instructors may download any file.
pub async fn download_submission_file(
    pool: &PgPool,
    claims: &Claims,
    submission_file_id: Uuid,
) -> Result<DownloadView> {
    let file = repository::find_submission_file_by_id(pool, submission_file_id)
        .await?
        .ok_or_else(|| anyhow!("submission file not found"))?;

    let submission = repository::find_submission_by_id(pool, file.submission_id)
        .await?
        .ok_or_else(|| anyhow!("submission not found"))?;

    if !claims.is_instructor() {
        if claims.user_id()? != submission.student_id {
            return Err(anyhow!("forbidden: submission belongs to another student"));
        }
    }

    let assignment_id = submission.assignment_id;
    let bytes = storage::read_file(assignment_id, &file.stored_name)?;
    Ok(DownloadView {
        name: file.name.clone(),
        mime_type: file.mime_type.clone(),
        size_bytes: file.size_bytes,
        content_hash: file.content_hash.clone(),
        content_base64: encode_base64(&bytes),
    })
}


// ---------------------------------------------------------------------------
// Grading
// ---------------------------------------------------------------------------

/// Grade a submission: stores the raw marks, applies the late penalty to get
/// the final marks, records the event in the grade history, and marks the
/// submission as graded.
pub async fn grade(pool: &PgPool, claims: &Claims, req: GradeRequest) -> Result<GradeView> {
    authz::require_instructor(claims)?;
    let grader_id = claims.user_id()?;

    let submission = repository::find_submission_by_id(pool, req.submission_id)
        .await?
        .ok_or_else(|| anyhow!("submission not found"))?;
    let assignment = repository::find_assignment_by_id(pool, submission.assignment_id)
        .await?
        .ok_or_else(|| anyhow!("assignment not found"))?;

    if req.marks < 0.0 || req.marks > assignment.max_marks + f64::EPSILON {
        return Err(anyhow!(
            "marks must be between 0 and {}",
            assignment.max_marks
        ));
    }

    let policy = LateSubmissionPolicy::from_str(&assignment.late_submission_policy)
        .unwrap_or(LateSubmissionPolicy::AllowedWithPenalty);
    let final_marks = apply_late_penalty(
        req.marks,
        assignment.max_marks,
        submission.is_late,
        policy,
        assignment.late_penalty_percent,
    );

    let grade = repository::upsert_grade(
        pool,
        submission.id,
        assignment.id,
        submission.student_id,
        grader_id,
        final_marks,
        req.marks,
        req.comments.trim(),
        req.feedback.trim(),
    )
    .await?;

    repository::insert_grade_history(
        pool,
        submission.id,
        grader_id,
        final_marks,
        req.marks,
        req.comments.trim(),
        req.feedback.trim(),
    )
    .await?;

    repository::mark_submission_graded(pool, submission.id).await?;

    Ok(to_grade_view(&grade))
}

fn to_grade_view(g: &crate::models::Grade) -> GradeView {
    GradeView {
        id: g.id,
        submission_id: g.submission_id,
        assignment_id: g.assignment_id,
        student_id: g.student_id,
        grader_id: g.grader_id,
        marks: g.marks,
        raw_marks: g.raw_marks,
        comments: g.comments.clone(),
        feedback: g.feedback.clone(),
        graded_at: g.graded_at,
    }
}

/// Get a grade. Students may only see their own; instructors see any.
pub async fn get_grade(pool: &PgPool, claims: &Claims, submission_id: Uuid) -> Result<GradeView> {
    let grade = repository::find_grade_by_submission(pool, submission_id)
        .await?
        .ok_or_else(|| anyhow!("grade not found"))?;

    if !claims.is_instructor() {
        if claims.user_id()? != grade.student_id {
            return Err(anyhow!("forbidden: grade belongs to another student"));
        }
    }

    Ok(to_grade_view(&grade))
}

/// Grade history for a submission. Students may only view their own history.
pub async fn grade_history(
    pool: &PgPool,
    claims: &Claims,
    submission_id: Uuid,
) -> Result<GradeHistoryView> {
    let submission = repository::find_submission_by_id(pool, submission_id)
        .await?
        .ok_or_else(|| anyhow!("submission not found"))?;

    if !claims.is_instructor() {
        if claims.user_id()? != submission.student_id {
            return Err(anyhow!("forbidden: submission belongs to another student"));
        }
    }

    let entries = repository::list_grade_history(pool, submission_id).await?;
    Ok(GradeHistoryView {
        entries: entries
            .into_iter()
            .map(|e| GradeHistoryEntry {
                id: e.id,
                submission_id: e.submission_id,
                grader_id: e.grader_id,
                marks: e.marks,
                raw_marks: e.raw_marks,
                comments: e.comments,
                feedback: e.feedback,
                graded_at: e.graded_at,
            })
            .collect(),
    })
}


// ---------------------------------------------------------------------------
// Rubrics
// ---------------------------------------------------------------------------

pub async fn create_rubric(
    pool: &PgPool,
    claims: &Claims,
    assignment_id: Uuid,
    name: &str,
    description: &str,
    criteria: serde_json::Value,
) -> Result<RubricView> {
    authz::require_instructor(claims)?;

    repository::find_assignment_by_id(pool, assignment_id)
        .await?
        .ok_or_else(|| anyhow!("assignment not found"))?;

    if name.trim().is_empty() {
        return Err(anyhow!("rubric name must not be empty"));
    }

    let rubric = repository::insert_rubric(
        pool,
        assignment_id,
        name.trim(),
        description.trim(),
        &criteria,
        claims.user_id()?,
    )
    .await?;

    Ok(RubricView {
        id: rubric.id,
        assignment_id: rubric.assignment_id,
        name: rubric.name,
        description: rubric.description,
        criteria: rubric.criteria,
        created_by: rubric.created_by,
        created_at: rubric.created_at,
    })
}

pub async fn list_rubrics(pool: &PgPool, assignment_id: Uuid) -> Result<RubricList> {
    let rubrics = repository::list_rubrics(pool, assignment_id).await?;
    Ok(RubricList {
        rubrics: rubrics
            .into_iter()
            .map(|r| RubricView {
                id: r.id,
                assignment_id: r.assignment_id,
                name: r.name,
                description: r.description,
                criteria: r.criteria,
                created_by: r.created_by,
                created_at: r.created_at,
            })
            .collect(),
    })
}

pub async fn get_rubric(pool: &PgPool, rubric_id: Uuid) -> Result<RubricView> {
    let rubric = repository::find_rubric_by_id(pool, rubric_id)
        .await?
        .ok_or_else(|| anyhow!("rubric not found"))?;
    Ok(RubricView {
        id: rubric.id,
        assignment_id: rubric.assignment_id,
        name: rubric.name,
        description: rubric.description,
        criteria: rubric.criteria,
        created_by: rubric.created_by,
        created_at: rubric.created_at,
    })
}

// ---------------------------------------------------------------------------
// Unit tests for the pure decision logic
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-08-29T10:00:00Z").unwrap().with_timezone(&Utc)
    }

    fn deadline() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-08-29T12:00:00Z").unwrap().with_timezone(&Utc)
    }

    #[test]
    fn deadline_classification() {
        assert!(!is_late(now(), Some(deadline())));
        assert!(is_late(deadline() + chrono::Duration::seconds(1), Some(deadline())));
        assert!(!is_late(now(), None));
    }

    #[test]
    fn on_time_submission_allowed() {
        let d = validate_submission(now(), Some(deadline()), 0, 1, LateSubmissionPolicy::Disallowed).unwrap();
        assert!(!d.is_late);

        let d = validate_submission(now(), Some(deadline()), 0, 1, LateSubmissionPolicy::AllowedWithPenalty).unwrap();
        assert!(!d.is_late);
    }

    #[test]
    fn late_submission_disallowed_when_policy_blocks() {
        let late = deadline() + chrono::Duration::hours(1);
        assert!(validate_submission(late, Some(deadline()), 0, 1, LateSubmissionPolicy::Disallowed).is_err());
    }

    #[test]
    fn late_submission_allowed_and_flagged() {
        let late = deadline() + chrono::Duration::hours(1);
        let d = validate_submission(late, Some(deadline()), 0, 1, LateSubmissionPolicy::Allowed).unwrap();
        assert!(d.is_late);

        let d = validate_submission(late, Some(deadline()), 0, 1, LateSubmissionPolicy::AllowedWithPenalty).unwrap();
        assert!(d.is_late);
    }

    #[test]
    fn attempt_limit_enforced() {
        // Attempts exhausted: rejected even well before the deadline.
        assert!(validate_submission(now(), Some(deadline()), 2, 2, LateSubmissionPolicy::Allowed).is_err());
        assert!(validate_submission(now(), Some(deadline()), 1, 2, LateSubmissionPolicy::Allowed).is_ok());
    }

    #[test]
    fn no_penalty_when_not_late_or_policy_without_penalty() {
        assert_eq!(apply_late_penalty(80.0, 100.0, false, LateSubmissionPolicy::AllowedWithPenalty, 10.0), 80.0);
        assert_eq!(apply_late_penalty(80.0, 100.0, true, LateSubmissionPolicy::Allowed, 10.0), 80.0);
    }

    #[test]
    fn penalty_applied_to_late_penalised_submissions() {
        // 10% of max (100) deducted from raw 80 -> 70.
        assert_eq!(apply_late_penalty(80.0, 100.0, true, LateSubmissionPolicy::AllowedWithPenalty, 10.0), 70.0);
        // Penalty never drops below zero.
        assert_eq!(apply_late_penalty(5.0, 100.0, true, LateSubmissionPolicy::AllowedWithPenalty, 50.0), 0.0);
        // Zero penalty -> unchanged.
        assert_eq!(apply_late_penalty(80.0, 100.0, true, LateSubmissionPolicy::AllowedWithPenalty, 0.0), 80.0);
    }

    #[test]
    fn content_must_match_submission_type() {
        assert!(validate_content(SubmissionType::File, &None, 1, 0).is_ok());
        assert!(validate_content(SubmissionType::File, &Some("x".into()), 0, 0).is_err());
        assert!(validate_content(SubmissionType::Text, &Some("essay".into()), 0, 0).is_ok());
        assert!(validate_content(SubmissionType::Text, &Some("  ".into()), 0, 0).is_err());
        assert!(validate_content(SubmissionType::Link, &None, 0, 1).is_ok());
        assert!(validate_content(SubmissionType::Link, &None, 1, 0).is_err());
        assert!(validate_content(SubmissionType::Any, &None, 0, 0).is_err());
        assert!(validate_content(SubmissionType::Any, &None, 0, 1).is_ok());
    }
}

