//! SQLx data access for assignments, resources, submissions, grades, rubrics.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Assignment, AssignmentResource, Grade, GradeHistoryEntry, Rubric, Submission, SubmissionFile,
    SubmissionLink,
};

// ---------------------------------------------------------------------------
// Assignments
// ---------------------------------------------------------------------------

pub async fn find_assignment_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Assignment>> {
    Ok(sqlx::query_as::<_, Assignment>(
        "SELECT id, title, instructions, course_id, created_by, deadline, max_marks, \
                allowed_attempts, submission_type, late_submission_policy, \
                late_penalty_percent, status, created_at, updated_at \
         FROM assignments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_assignments(pool: &PgPool) -> Result<Vec<Assignment>> {
    Ok(sqlx::query_as::<_, Assignment>(
        "SELECT id, title, instructions, course_id, created_by, deadline, max_marks, \
                allowed_attempts, submission_type, late_submission_policy, \
                late_penalty_percent, status, created_at, updated_at \
         FROM assignments ORDER BY created_at DESC LIMIT 200",
    )
    .fetch_all(pool)
    .await?)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_assignment(
    pool: &PgPool,
    title: &str,
    instructions: &str,
    course_id: Option<Uuid>,
    created_by: Uuid,
    deadline: Option<DateTime<Utc>>,
    max_marks: f64,
    allowed_attempts: i32,
    submission_type: &str,
    late_submission_policy: &str,
    late_penalty_percent: f64,
) -> Result<Assignment> {
    Ok(sqlx::query_as::<_, Assignment>(
        "INSERT INTO assignments \
            (title, instructions, course_id, created_by, deadline, max_marks, \
             allowed_attempts, submission_type, late_submission_policy, late_penalty_percent) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         RETURNING id, title, instructions, course_id, created_by, deadline, max_marks, \
                   allowed_attempts, submission_type, late_submission_policy, \
                   late_penalty_percent, status, created_at, updated_at",
    )
    .bind(title)
    .bind(instructions)
    .bind(course_id)
    .bind(created_by)
    .bind(deadline)
    .bind(max_marks)
    .bind(allowed_attempts)
    .bind(submission_type)
    .bind(late_submission_policy)
    .bind(late_penalty_percent)
    .fetch_one(pool)
    .await?)
}

pub async fn update_assignment_fields(
    pool: &PgPool,
    id: Uuid,
    title: Option<&str>,
    instructions: Option<&str>,
    deadline: Option<DateTime<Utc>>,
    max_marks: Option<f64>,
    allowed_attempts: Option<i32>,
    submission_type: Option<&str>,
    late_submission_policy: Option<&str>,
    late_penalty_percent: Option<f64>,
) -> Result<Assignment> {
    Ok(sqlx::query_as::<_, Assignment>(
        "UPDATE assignments SET \
            title = COALESCE($2, title), \
            instructions = COALESCE($3, instructions), \
            deadline = $4, \
            max_marks = COALESCE($5, max_marks), \
            allowed_attempts = COALESCE($6, allowed_attempts), \
            submission_type = COALESCE($7, submission_type), \
            late_submission_policy = COALESCE($8, late_submission_policy), \
            late_penalty_percent = COALESCE($9, late_penalty_percent), \
            updated_at = now() \
         WHERE id = $1 \
         RETURNING id, title, instructions, course_id, created_by, deadline, max_marks, \
                   allowed_attempts, submission_type, late_submission_policy, \
                   late_penalty_percent, status, created_at, updated_at",
    )
    .bind(id)
    .bind(title)
    .bind(instructions)
    .bind(deadline)
    .bind(max_marks)
    .bind(allowed_attempts)
    .bind(submission_type)
    .bind(late_submission_policy)
    .bind(late_penalty_percent)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_assignment(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM assignments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

pub async fn insert_resource(
    pool: &PgPool,
    assignment_id: Uuid,
    name: &str,
    stored_name: &str,
    mime_type: &str,
    size_bytes: i64,
    content_hash: &str,
    created_by: Uuid,
) -> Result<AssignmentResource> {
    Ok(sqlx::query_as::<_, AssignmentResource>(
        "INSERT INTO assignment_resources \
            (assignment_id, name, stored_name, mime_type, size_bytes, content_hash, created_by) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         RETURNING id, assignment_id, name, stored_name, mime_type, size_bytes, \
                   content_hash, created_by, created_at",
    )
    .bind(assignment_id)
    .bind(name)
    .bind(stored_name)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(content_hash)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

#[allow(dead_code)] // part of the plugin's API surface (assignment view lists resources)
pub async fn list_resources(pool: &PgPool, assignment_id: Uuid) -> Result<Vec<AssignmentResource>> {
    Ok(sqlx::query_as::<_, AssignmentResource>(
        "SELECT id, assignment_id, name, stored_name, mime_type, size_bytes, content_hash, \
                created_by, created_at \
         FROM assignment_resources WHERE assignment_id = $1 ORDER BY created_at",
    )
    .bind(assignment_id)
    .fetch_all(pool)
    .await?)
}

pub async fn find_resource_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AssignmentResource>> {
    Ok(sqlx::query_as::<_, AssignmentResource>(
        "SELECT id, assignment_id, name, stored_name, mime_type, size_bytes, content_hash, \
                created_by, created_at \
         FROM assignment_resources WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_resource(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM assignment_resources WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}


// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

pub async fn count_submissions_for_student(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT count(*) FROM submissions WHERE assignment_id = $1 AND student_id = $2",
    )
    .bind(assignment_id)
    .bind(student_id)
    .fetch_one(pool)
    .await?)
}

pub async fn insert_submission(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
    attempt: i32,
    text_content: Option<&str>,
    is_late: bool,
) -> Result<Submission> {
    Ok(sqlx::query_as::<_, Submission>(
        "INSERT INTO submissions (assignment_id, student_id, attempt, text_content, is_late) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, assignment_id, student_id, attempt, text_content, submitted_at, \
                   is_late, status, graded",
    )
    .bind(assignment_id)
    .bind(student_id)
    .bind(attempt)
    .bind(text_content)
    .bind(is_late)
    .fetch_one(pool)
    .await?)
}

pub async fn find_submission_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Submission>> {
    Ok(sqlx::query_as::<_, Submission>(
        "SELECT id, assignment_id, student_id, attempt, text_content, submitted_at, \
                is_late, status, graded \
         FROM submissions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_submissions_for_student(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<Vec<Submission>> {
    Ok(sqlx::query_as::<_, Submission>(
        "SELECT id, assignment_id, student_id, attempt, text_content, submitted_at, \
                is_late, status, graded \
         FROM submissions WHERE assignment_id = $1 AND student_id = $2 ORDER BY attempt DESC",
    )
    .bind(assignment_id)
    .bind(student_id)
    .fetch_all(pool)
    .await?)
}

pub async fn list_submissions_for_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
) -> Result<Vec<Submission>> {
    Ok(sqlx::query_as::<_, Submission>(
        "SELECT id, assignment_id, student_id, attempt, text_content, submitted_at, \
                is_late, status, graded \
         FROM submissions WHERE assignment_id = $1 ORDER BY submitted_at DESC",
    )
    .bind(assignment_id)
    .fetch_all(pool)
    .await?)
}


pub async fn insert_submission_file(
    pool: &PgPool,
    submission_id: Uuid,
    name: &str,
    stored_name: &str,
    mime_type: &str,
    size_bytes: i64,
    content_hash: &str,
) -> Result<SubmissionFile> {
    Ok(sqlx::query_as::<_, SubmissionFile>(
        "INSERT INTO submission_files \
            (submission_id, name, stored_name, mime_type, size_bytes, content_hash) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, submission_id, name, stored_name, mime_type, size_bytes, \
                   content_hash, created_at",
    )
    .bind(submission_id)
    .bind(name)
    .bind(stored_name)
    .bind(mime_type)
    .bind(size_bytes)
    .bind(content_hash)
    .fetch_one(pool)
    .await?)
}

pub async fn list_submission_files(
    pool: &PgPool,
    submission_id: Uuid,
) -> Result<Vec<SubmissionFile>> {
    Ok(sqlx::query_as::<_, SubmissionFile>(
        "SELECT id, submission_id, name, stored_name, mime_type, size_bytes, content_hash, \
                created_at \
         FROM submission_files WHERE submission_id = $1 ORDER BY created_at",
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await?)
}

pub async fn find_submission_file_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<SubmissionFile>> {
    Ok(sqlx::query_as::<_, SubmissionFile>(
        "SELECT id, submission_id, name, stored_name, mime_type, size_bytes, content_hash, \
                created_at \
         FROM submission_files WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn insert_submission_link(
    pool: &PgPool,
    submission_id: Uuid,
    url: &str,
    title: &str,
) -> Result<SubmissionLink> {
    Ok(sqlx::query_as::<_, SubmissionLink>(
        "INSERT INTO submission_links (submission_id, url, title) \
         VALUES ($1, $2, $3) \
         RETURNING id, submission_id, url, title, created_at",
    )
    .bind(submission_id)
    .bind(url)
    .bind(title)
    .fetch_one(pool)
    .await?)
}

pub async fn list_submission_links(
    pool: &PgPool,
    submission_id: Uuid,
) -> Result<Vec<SubmissionLink>> {
    Ok(sqlx::query_as::<_, SubmissionLink>(
        "SELECT id, submission_id, url, title, created_at \
         FROM submission_links WHERE submission_id = $1 ORDER BY created_at",
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await?)
}


// ---------------------------------------------------------------------------
// Grades
// ---------------------------------------------------------------------------

pub async fn upsert_grade(
    pool: &PgPool,
    submission_id: Uuid,
    assignment_id: Uuid,
    student_id: Uuid,
    grader_id: Uuid,
    marks: f64,
    raw_marks: f64,
    comments: &str,
    feedback: &str,
) -> Result<Grade> {
    Ok(sqlx::query_as::<_, Grade>(
        "INSERT INTO grades \
            (submission_id, assignment_id, student_id, grader_id, marks, raw_marks, \
             comments, feedback) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
         ON CONFLICT (assignment_id, student_id) DO UPDATE SET \
            submission_id = EXCLUDED.submission_id, \
            grader_id = EXCLUDED.grader_id, \
            marks = EXCLUDED.marks, \
            raw_marks = EXCLUDED.raw_marks, \
            comments = EXCLUDED.comments, \
            feedback = EXCLUDED.feedback, \
            graded_at = now() \
         RETURNING id, submission_id, assignment_id, student_id, grader_id, marks, \
                   raw_marks, comments, feedback, graded_at",
    )
    .bind(submission_id)
    .bind(assignment_id)
    .bind(student_id)
    .bind(grader_id)
    .bind(marks)
    .bind(raw_marks)
    .bind(comments)
    .bind(feedback)
    .fetch_one(pool)
    .await?)
}

pub async fn find_grade_by_submission(pool: &PgPool, submission_id: Uuid) -> Result<Option<Grade>> {
    Ok(sqlx::query_as::<_, Grade>(
        "SELECT id, submission_id, assignment_id, student_id, grader_id, marks, raw_marks, \
                comments, feedback, graded_at \
         FROM grades WHERE submission_id = $1",
    )
    .bind(submission_id)
    .fetch_optional(pool)
    .await?)
}

#[allow(dead_code)] // part of the plugin's API surface (student-facing grade lookups)
pub async fn find_grade_for_student(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
) -> Result<Option<Grade>> {
    Ok(sqlx::query_as::<_, Grade>(
        "SELECT id, submission_id, assignment_id, student_id, grader_id, marks, raw_marks, \
                comments, feedback, graded_at \
         FROM grades WHERE assignment_id = $1 AND student_id = $2",
    )
    .bind(assignment_id)
    .bind(student_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn insert_grade_history(
    pool: &PgPool,
    submission_id: Uuid,
    grader_id: Uuid,
    marks: f64,
    raw_marks: f64,
    comments: &str,
    feedback: &str,
) -> Result<GradeHistoryEntry> {
    Ok(sqlx::query_as::<_, GradeHistoryEntry>(
        "INSERT INTO grade_history (submission_id, grader_id, marks, raw_marks, comments, feedback) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         RETURNING id, submission_id, grader_id, marks, raw_marks, comments, feedback, graded_at",
    )
    .bind(submission_id)
    .bind(grader_id)
    .bind(marks)
    .bind(raw_marks)
    .bind(comments)
    .bind(feedback)
    .fetch_one(pool)
    .await?)
}

pub async fn list_grade_history(
    pool: &PgPool,
    submission_id: Uuid,
) -> Result<Vec<GradeHistoryEntry>> {
    Ok(sqlx::query_as::<_, GradeHistoryEntry>(
        "SELECT id, submission_id, grader_id, marks, raw_marks, comments, feedback, graded_at \
         FROM grade_history WHERE submission_id = $1 ORDER BY graded_at DESC",
    )
    .bind(submission_id)
    .fetch_all(pool)
    .await?)
}

pub async fn mark_submission_graded(pool: &PgPool, submission_id: Uuid) -> Result<()> {
    sqlx::query("UPDATE submissions SET graded = TRUE, status = 'graded' WHERE id = $1")
        .bind(submission_id)
        .execute(pool)
        .await?;
    Ok(())
}


// ---------------------------------------------------------------------------
// Rubrics
// ---------------------------------------------------------------------------

pub async fn insert_rubric(
    pool: &PgPool,
    assignment_id: Uuid,
    name: &str,
    description: &str,
    criteria: &Value,
    created_by: Uuid,
) -> Result<Rubric> {
    Ok(sqlx::query_as::<_, Rubric>(
        "INSERT INTO rubrics (assignment_id, name, description, criteria, created_by) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, assignment_id, name, description, criteria, created_by, created_at",
    )
    .bind(assignment_id)
    .bind(name)
    .bind(description)
    .bind(criteria)
    .bind(created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn list_rubrics(pool: &PgPool, assignment_id: Uuid) -> Result<Vec<Rubric>> {
    Ok(sqlx::query_as::<_, Rubric>(
        "SELECT id, assignment_id, name, description, criteria, created_by, created_at \
         FROM rubrics WHERE assignment_id = $1 ORDER BY created_at",
    )
    .bind(assignment_id)
    .fetch_all(pool)
    .await?)
}

pub async fn find_rubric_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Rubric>> {
    Ok(sqlx::query_as::<_, Rubric>(
        "SELECT id, assignment_id, name, description, criteria, created_by, created_at \
         FROM rubrics WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

