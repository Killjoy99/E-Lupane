//! Business logic: course lifecycle, status machine, ownership enforcement,
//! duplication, publishing, and instructor authorization.
//!
//! Pure decision functions (code validation, duplicate title generation, status
//! transitions) are unit-tested without a database; the async service functions
//! wrap them with persistence and authorization.

use anyhow::{anyhow, Result};
use sqlx::PgPool;
use uuid::Uuid;

use crate::authz::{self, Claims};
use crate::dto::{
    can_transition, CourseDifficulty, CourseList, CourseStatus, CourseView, CreateCourseRequest,
    UpdateCourseRequest,
};
use crate::models::Course;
use crate::repository;

/// Maximum length of a course code (validated on create/update).
pub const MAX_CODE_LEN: usize = 20;

/// Whether `code` looks like a valid course code (upper-case letters, digits,
/// dashes/underscores; e.g. `CS101`, `WEB-302`, `MATH_150`).
pub fn validate_course_code(code: &str) -> Result<()> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("course code is required"));
    }
    if trimmed.len() > MAX_CODE_LEN {
        return Err(anyhow!("course code too long (max {MAX_CODE_LEN} characters)"));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!(
            "course code may only contain letters, digits, dashes and underscores"
        ));
    }
    if trimmed.chars().any(|c| c.is_ascii_alphabetic()) && trimmed != trimmed.to_uppercase() {
        return Err(anyhow!("course code must be upper-case"));
    }
    Ok(())
}

/// Title for a duplicated course (kept short enough to stay readable).
pub fn duplicate_title(original: &str) -> String {
    if original.is_empty() {
        "Copy of course".to_string()
    } else if original.to_lowercase().starts_with("copy of ") {
        format!("{original} (copy)")
    } else {
        format!("Copy of {original}")
    }
}

/// Normalize a difficulty string to a valid value (defaults to `beginner`).
pub fn normalize_difficulty(d: &str) -> &'static str {
    CourseDifficulty::from_str(d)
        .map(|x| x.as_str())
        .unwrap_or(CourseDifficulty::Beginner.as_str())
}

/// A course plus its extra instructor ids (owner not included).
pub struct CourseWithInstructors {
    pub course: Course,
    pub instructors: Vec<Uuid>,
}

async fn load_with_instructors(pool: &PgPool, id: Uuid) -> Result<CourseWithInstructors> {
    let course = repository::find_course_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("course not found"))?;
    let instructors = repository::list_instructor_ids(pool, id).await?;
    Ok(CourseWithInstructors { course, instructors })
}

/// Load a course and require the caller to be able to *manage* it.
async fn require_manage(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<CourseWithInstructors> {
    let loaded = load_with_instructors(pool, id).await?;
    if !authz::can_manage(claims, loaded.course.owner_id, &loaded.instructors)? {
        return Err(anyhow!("forbidden: you do not manage this course"));
    }
    Ok(loaded)
}

fn to_view(loaded: &CourseWithInstructors) -> CourseView {
    let c = &loaded.course;
    CourseView {
        id: c.id,
        code: c.code.clone(),
        title: c.title.clone(),
        short_description: c.short_description.clone(),
        description: c.description.clone(),
        thumbnail: c.thumbnail.clone(),
        banner: c.banner.clone(),
        category: c.category.clone(),
        subject: c.subject.clone(),
        level: c.level.clone(),
        language: c.language.clone(),
        difficulty: c.difficulty.clone(),
        duration: c.duration.clone(),
        learning_objectives: json_strs(&c.learning_objectives),
        target_audience: json_strs(&c.target_audience),
        prerequisites: json_strs(&c.prerequisites),
        tags: json_strs(&c.tags),
        status: c.status.clone(),
        owner_id: c.owner_id,
        instructors: loaded.instructors.clone(),
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

fn json_strs(v: &serde_json::Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}


// ---------------------------------------------------------------------------
// Public service API
// ---------------------------------------------------------------------------

pub async fn create_course(
    pool: &PgPool,
    claims: &Claims,
    req: CreateCourseRequest,
) -> Result<CourseView> {
    authz::require_instructor(claims)?;
    validate_course_code(&req.code)?;
    if req.title.trim().is_empty() {
        return Err(anyhow!("course title is required"));
    }
    if repository::find_course_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a course with code '{}' already exists", req.code));
    }
    let me = claims.user_id()?;
    let difficulty = normalize_difficulty(&req.difficulty);
    let course = repository::insert_course(
        pool,
        req.code.trim(),
        req.title.trim(),
        &req.short_description,
        &req.description,
        &req.thumbnail,
        &req.banner,
        &req.category,
        &req.subject,
        &req.level,
        &req.language,
        difficulty,
        &req.duration,
        &serde_json::to_value(&req.learning_objectives)?,
        &serde_json::to_value(&req.target_audience)?,
        &serde_json::to_value(&req.prerequisites)?,
        &serde_json::to_value(&req.tags)?,
        me,
        me,
    )
    .await?;
    Ok(to_view(&CourseWithInstructors {
        course,
        instructors: Vec::new(),
    }))
}

pub async fn get_course(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<CourseView> {
    let loaded = load_with_instructors(pool, id).await?;
    if !authz::can_view(claims, &loaded.course.status, loaded.course.owner_id, &loaded.instructors)? {
        return Err(anyhow!("forbidden: course is not published"));
    }
    Ok(to_view(&loaded))
}


/// Preview a course in any status. Managers only — used by the editor's
/// "Preview" action and by admins reviewing pending courses.
pub async fn preview_course(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<CourseView> {
    let loaded = require_manage(pool, claims, id).await?;
    Ok(to_view(&loaded))
}

/// List courses the caller is allowed to see.
///
/// * Students/developers: only `published`.
/// * Instructors/admins: published courses plus every course they own or manage.
pub async fn list_courses(
    pool: &PgPool,
    claims: &Claims,
    status: Option<String>,
    mine: bool,
    search: Option<String>,
) -> Result<CourseList> {
    let me = claims.user_id()?;
    let is_instructor = claims.is_instructor();
    let status_filter = if is_instructor {
        status
    } else {
        Some(CourseStatus::Published.as_str().to_string())
    };
    let mine_filter = if mine && is_instructor { Some(me) } else { None };
    let rows = repository::list_courses(
        pool,
        status_filter.as_deref(),
        mine_filter,
        search.as_deref(),
        200,
    )
    .await?;

    let mut views = Vec::new();
    for course in rows {
        let instructors = repository::list_instructor_ids(pool, course.id).await?;
        let loaded = CourseWithInstructors { course, instructors };
        // Belt-and-braces: drop rows the caller may not view.
        if authz::can_view(claims, &loaded.course.status, loaded.course.owner_id, &loaded.instructors)? {
            views.push(to_view(&loaded));
        }
    }
    Ok(CourseList { courses: views })
}

pub async fn update_course(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateCourseRequest,
) -> Result<CourseView> {
    // Permission check: only instructors who manage the course may edit it.
    require_manage(pool, claims, req.course_id).await?;
    if let Some(code) = &req.code {
        validate_course_code(code)?;
        if let Some(existing) = repository::find_course_by_code(pool, code).await? {
            if existing.id != req.course_id {
                return Err(anyhow!("a course with code '{}' already exists", code));
            }
        }
    }
    if let Some(title) = &req.title {
        if title.trim().is_empty() {
            return Err(anyhow!("course title cannot be empty"));
        }
    }
    let obj = |v: &Option<Vec<String>>| v.as_ref().map(|x| serde_json::to_value(x)).transpose();
    let difficulty = req.difficulty.as_deref().map(normalize_difficulty);
    let course = repository::update_course_fields(
        pool,
        req.course_id,
        req.code.as_deref(),
        req.title.as_deref(),
        req.short_description.as_deref(),
        req.description.as_deref(),
        req.thumbnail.as_deref(),
        req.banner.as_deref(),
        req.category.as_deref(),
        req.subject.as_deref(),
        req.level.as_deref(),
        req.language.as_deref(),
        difficulty,
        req.duration.as_deref(),
        obj(&req.learning_objectives)?.as_ref(),
        obj(&req.target_audience)?.as_ref(),
        obj(&req.prerequisites)?.as_ref(),
        obj(&req.tags)?.as_ref(),
        claims.user_id()?,
    )
    .await?;
    let instructors = repository::list_instructor_ids(pool, req.course_id).await?;
    Ok(to_view(&CourseWithInstructors { course, instructors }))
}


/// Set a course's status. Validates the transition and that the caller manages
/// the course. Admins may approve/override any transition.
pub async fn set_status(
    pool: &PgPool,
    claims: &Claims,
    course_id: Uuid,
    target: &str,
) -> Result<CourseView> {
    let loaded = require_manage(pool, claims, course_id).await?;
    let from = CourseStatus::from_str(&loaded.course.status)
        .ok_or_else(|| anyhow!("unknown current status '{}'", loaded.course.status))?;
    let to = CourseStatus::from_str(target)
        .ok_or_else(|| anyhow!("unknown target status '{target}'"))?;

    let allowed = claims.is_admin() || can_transition(from, to);
    if !allowed {
        return Err(anyhow!(
            "invalid status transition: {} → {}",
            from.as_str(),
            to.as_str()
        ));
    }
    let course = repository::set_course_status(pool, course_id, to.as_str(), claims.user_id()?).await?;
    let instructors = repository::list_instructor_ids(pool, course_id).await?;
    Ok(to_view(&CourseWithInstructors { course, instructors }))
}

/// Hard-delete a course. Only the owner or an admin may delete.
pub async fn delete_course(pool: &PgPool, claims: &Claims, course_id: Uuid) -> Result<()> {
    let loaded = load_with_instructors(pool, course_id).await?;
    let is_owner = loaded.course.owner_id == claims.user_id()?;
    if !claims.is_admin() && !is_owner {
        return Err(anyhow!("forbidden: only the course owner or an admin may delete"));
    }
    repository::delete_course(pool, course_id).await?;
    Ok(())
}

/// Duplicate a course: copies all content as a fresh `draft` with a new unique
/// code and a "Copy of ..." title. Only managers may duplicate.
pub async fn duplicate_course(pool: &PgPool, claims: &Claims, course_id: Uuid) -> Result<CourseView> {
    let loaded = require_manage(pool, claims, course_id).await?;
    let src = loaded.course;

    // Build a unique code: CS101 -> CS101-2, CS101-2 -> CS101-3 ...
    let mut candidate = format!("{}-2", src.code);
    let mut n = 2;
    while repository::find_course_by_code(pool, &candidate).await?.is_some() {
        n += 1;
        candidate = format!("{}-{n}", src.code);
    }

    let me = claims.user_id()?;
    let difficulty = normalize_difficulty(&src.difficulty);
    let course = repository::insert_course(
        pool,
        &candidate,
        &duplicate_title(&src.title),
        &src.short_description,
        &src.description,
        &src.thumbnail,
        &src.banner,
        &src.category,
        &src.subject,
        &src.level,
        &src.language,
        difficulty,
        &src.duration,
        &src.learning_objectives,
        &src.target_audience,
        &src.prerequisites,
        &src.tags,
        me,
        me,
    )
    .await?;

    // Co-instructors carry over to the copy.
    for uid in &loaded.instructors {
        if *uid != me {
            let _ = repository::add_instructor(pool, course.id, *uid, me).await;
        }
    }
    Ok(to_view(&CourseWithInstructors {
        course,
        instructors: loaded.instructors.clone(),
    }))
}

/// Add an instructor who may manage the course (owner/managers only).
pub async fn add_instructor(
    pool: &PgPool,
    claims: &Claims,
    course_id: Uuid,
    user_id: Uuid,
) -> Result<CourseView> {
    let loaded = require_manage(pool, claims, course_id).await?;
    repository::add_instructor(pool, course_id, user_id, claims.user_id()?).await?;
    let instructors = repository::list_instructor_ids(pool, course_id).await?;
    Ok(to_view(&CourseWithInstructors {
        course: loaded.course,
        instructors,
    }))
}

/// Remove an instructor from a course. Cannot remove the owner.
pub async fn remove_instructor(
    pool: &PgPool,
    claims: &Claims,
    course_id: Uuid,
    user_id: Uuid,
) -> Result<CourseView> {
    let loaded = require_manage(pool, claims, course_id).await?;
    if user_id == loaded.course.owner_id {
        return Err(anyhow!("cannot remove the course owner"));
    }
    repository::remove_instructor(pool, course_id, user_id).await?;
    let instructors = repository::list_instructor_ids(pool, course_id).await?;
    Ok(to_view(&CourseWithInstructors {
        course: loaded.course,
        instructors,
    }))
}


// ---------------------------------------------------------------------------
// Unit tests (pure logic — no database)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_code_validation() {
        assert!(validate_course_code("CS101").is_ok());
        assert!(validate_course_code("WEB-302").is_ok());
        assert!(validate_course_code("MATH_150").is_ok());
        assert!(validate_course_code("").is_err());
        assert!(validate_course_code("   ").is_err());
        assert!(validate_course_code("cs101").is_err()); // must be upper-case
        assert!(validate_course_code("CS 101").is_err()); // spaces not allowed
        assert!(validate_course_code("THIS-CODE-IS-WAY-TOO-LONG-123").is_err());
    }

    #[test]
    fn duplicate_titles() {
        assert_eq!(duplicate_title("Intro to Python"), "Copy of Intro to Python");
        assert_eq!(duplicate_title("Copy of Intro to Python"), "Copy of Intro to Python (copy)");
        assert_eq!(duplicate_title(""), "Copy of course");
    }

    #[test]
    fn difficulty_normalization() {
        assert_eq!(normalize_difficulty("advanced"), "advanced");
        assert_eq!(normalize_difficulty("expert"), "expert");
        assert_eq!(normalize_difficulty("super-hard"), "beginner"); // unknown -> default
        assert_eq!(normalize_difficulty(""), "beginner");
    }

    #[test]
    fn status_self_transition_is_always_valid() {
        for s in CourseStatus::ALL {
            assert!(can_transition(s, s));
        }
    }

    #[test]
    fn draft_transitions() {
        assert!(can_transition(CourseStatus::Draft, CourseStatus::PendingReview));
        assert!(can_transition(CourseStatus::Draft, CourseStatus::Archived));
        assert!(!can_transition(CourseStatus::Draft, CourseStatus::Published));
        assert!(!can_transition(CourseStatus::Draft, CourseStatus::Unpublished));
    }

    #[test]
    fn pending_review_transitions() {
        assert!(can_transition(CourseStatus::PendingReview, CourseStatus::Published));
        assert!(can_transition(CourseStatus::PendingReview, CourseStatus::Unpublished));
        assert!(can_transition(CourseStatus::PendingReview, CourseStatus::Draft));
        assert!(can_transition(CourseStatus::PendingReview, CourseStatus::Archived));
    }

    #[test]
    fn published_transitions() {
        assert!(can_transition(CourseStatus::Published, CourseStatus::Unpublished));
        assert!(can_transition(CourseStatus::Published, CourseStatus::Archived));
        assert!(!can_transition(CourseStatus::Published, CourseStatus::Draft));
        assert!(!can_transition(CourseStatus::Published, CourseStatus::PendingReview));
    }

    #[test]
    fn unpublished_transitions() {
        assert!(can_transition(CourseStatus::Unpublished, CourseStatus::Published));
        assert!(can_transition(CourseStatus::Unpublished, CourseStatus::Archived));
        assert!(can_transition(CourseStatus::Unpublished, CourseStatus::Draft));
    }

    #[test]
    fn archived_transitions() {
        assert!(can_transition(CourseStatus::Archived, CourseStatus::Draft));
        assert!(can_transition(CourseStatus::Archived, CourseStatus::Unpublished));
        assert!(!can_transition(CourseStatus::Archived, CourseStatus::Published));
        assert!(!can_transition(CourseStatus::Archived, CourseStatus::PendingReview));
    }
}

