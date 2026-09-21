//! Business logic for learning content: modules, lessons, and content items.
//!
//! This plugin does **not** own user authentication; it trusts JWTs issued by
//! the auth plugin. Course-level authorization (can this instructor manage this
//! course?) is enforced by calling back into the course_management plugin via
//! Redis RPC when necessary. For simplicity and to avoid circular dependencies,
//! the content plugin performs authorization at the module/lesson/item level
//! based on the caller's roles and the parent course's ownership info embedded
//! in the database records.

use anyhow::{anyhow, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::models::*;
use crate::repository;
use crate::repository::CourseAccess;
use sqlx::PgPool;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn to_module_view(m: &ContentModule) -> ContentModuleView {
    ContentModuleView {
        id: m.id,
        course_id: m.course_id,
        subject_id: m.subject_id,
        title: m.title.clone(),
        description: m.description.clone(),
        sort_order: m.sort_order,
        status: m.status.clone(),
        published_at: m.published_at,
        created_by: m.created_by,
        updated_by: m.updated_by,
        created_at: m.created_at,
        updated_at: m.updated_at,
    }
}

fn to_lesson_view(l: &ContentLesson) -> ContentLessonView {
    ContentLessonView {
        id: l.id,
        module_id: l.module_id,
        title: l.title.clone(),
        description: l.description.clone(),
        sort_order: l.sort_order,
        status: l.status.clone(),
        published_at: l.published_at,
        created_by: l.created_by,
        updated_by: l.updated_by,
        created_at: l.created_at,
        updated_at: l.updated_at,
    }
}

fn to_item_view(i: &ContentItem) -> ContentItemView {
    ContentItemView {
        id: i.id,
        lesson_id: i.lesson_id,
        content_type: i.content_type.clone(),
        title: i.title.clone(),
        body: i.body.clone(),
        url: i.url.clone(),
        file_id: i.file_id,
        mime_type: i.mime_type.clone(),
        file_size_bytes: i.file_size_bytes,
        sort_order: i.sort_order,
        created_by: i.created_by,
        updated_by: i.updated_by,
        created_at: i.created_at,
        updated_at: i.updated_at,
    }
}

fn to_file_view(f: &ContentFile) -> ContentFileView {
    ContentFileView {
        id: f.id,
        course_id: f.course_id,
        filename: f.filename.clone(),
        mime_type: f.mime_type.clone(),
        size_bytes: f.size_bytes,
        created_by: f.created_by,
        created_at: f.created_at,
    }
}

// ---------------------------------------------------------------------------
// Course access gating
// ---------------------------------------------------------------------------

/// Resolve how much access `claims` has to a course's content (admins always
/// manage; everyone else is decided by the shared `courses` / `course_instructors`
/// / `course_enrollments` tables in `repository::course_access`).
async fn course_access_for(pool: &PgPool, claims: &Claims, course_id: Uuid) -> Result<CourseAccess> {
    if claims.is_admin() {
        return Ok(CourseAccess::Manage);
    }
    repository::course_access(pool, claims.user_id()?, course_id).await
}

/// Gate: caller must be at least an active enrollee (or owner/admin) to read.
async fn require_view(pool: &PgPool, claims: &Claims, course_id: Uuid) -> Result<()> {
    let access = course_access_for(pool, claims, course_id).await?;
    if access < CourseAccess::View {
        return Err(anyhow!("forbidden: you are not enrolled in this course"));
    }
    Ok(())
}

/// Gate: caller must be the course owner/instructor (or admin) to manage.
async fn require_manage(pool: &PgPool, claims: &Claims, course_id: Uuid) -> Result<()> {
    let access = course_access_for(pool, claims, course_id).await?;
    if access < CourseAccess::Manage {
        return Err(anyhow!("forbidden: not the course owner or an authorised instructor"));
    }
    Ok(())
}

fn ensure_content_type(ct: &str) -> Result<ContentType> {
    ContentType::from_str(ct).ok_or_else(|| anyhow!("invalid content_type: {}", ct))
}

fn ensure_status(s: &str) -> Result<PublishStatus> {
    PublishStatus::from_str(s).ok_or_else(|| anyhow!("invalid status: {}", s))
}

// ---------------------------------------------------------------------------
// File storage helpers
// ---------------------------------------------------------------------------

const UPLOAD_DIR: &str = "./uploads/content";

fn storage_path_for(file_id: Uuid) -> String {
    format!("{}/{}/{}", UPLOAD_DIR, &file_id.to_string()[..8], file_id)
}

fn ensure_storage_dirs() -> Result<()> {
    Ok(())
}

/// Decode base64 content and write to disk, returning the storage path.
fn save_file(course_id: Uuid, file_id: Uuid, filename: &str, content_base64: &str) -> Result<String> {
    let path = storage_path_for(file_id);
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = base64_decode(content_base64)?;
    let mut f = fs::File::create(&path)?;
    f.write_all(&bytes)?;
    Ok(path)
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    Ok(engine.decode(s)?)
}

// ---------------------------------------------------------------------------
// Module services
// ---------------------------------------------------------------------------

pub async fn list_modules(
    pool: &PgPool,
    claims: &Claims,
    course_id: Uuid,
) -> Result<Vec<ContentModuleView>> {
    require_view(pool, claims, course_id).await?;
    let rows = repository::list_modules_by_course(pool, course_id).await?;
    Ok(rows.iter().map(to_module_view).collect())
}

pub async fn get_module(pool: &PgPool, claims: &Claims, module_id: Uuid) -> Result<ContentModuleView> {
    let m = repository::find_module_by_id(pool, module_id)
        .await?
        .ok_or_else(|| anyhow!("module not found"))?;
    let course_id = m.course_id;
    require_view(pool, claims, course_id).await?;
    Ok(to_module_view(&m))
}

pub async fn create_module(
    pool: &PgPool,
    claims: &Claims,
    req: CreateModuleRequest,
) -> Result<ContentModuleView> {
    authz::require_instructor(claims)?;
    require_manage(pool, claims, req.course_id).await?;
    let me = claims.user_id()?;
    let m = repository::insert_module(
        pool,
        req.course_id,
        &req.title,
        &req.description,
        req.sort_order,
        me,
        me,
    )
    .await?;
    Ok(to_module_view(&m))
}

pub async fn update_module(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateModuleRequest,
) -> Result<ContentModuleView> {
    authz::require_instructor(claims)?;
    let me = claims.user_id()?;
    let m = repository::update_module_fields(
        pool,
        req.module_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.sort_order,
        me,
    )
    .await?;
    Ok(to_module_view(&m))
}

pub async fn delete_module(pool: &PgPool, claims: &Claims, module_id: Uuid) -> Result<()> {
    authz::require_instructor(claims)?;
    repository::delete_module(pool, module_id).await?;
    Ok(())
}

pub async fn reorder_modules(
    pool: &PgPool,
    claims: &Claims,
    req: ReorderModulesRequest,
) -> Result<()> {
    authz::require_instructor(claims)?;
    for (idx, module_id) in req.module_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE content_modules SET sort_order = $1, updated_by = $2, updated_at = now() WHERE id = $3 AND course_id = $4",
        )
        .bind(idx as i32)
        .bind(claims.user_id()?)
        .bind(*module_id)
        .bind(req.course_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn set_module_status(
    pool: &PgPool,
    claims: &Claims,
    req: SetModuleStatusRequest,
) -> Result<ContentModuleView> {
    authz::require_instructor(claims)?;
    let status = ensure_status(&req.status)?;
    let m = repository::set_module_status(pool, req.module_id, status.as_str(), claims.user_id()?).await?;
    Ok(to_module_view(&m))
}

// ---------------------------------------------------------------------------
// Lesson services
// ---------------------------------------------------------------------------

pub async fn list_lessons(pool: &PgPool, claims: &Claims, module_id: Uuid) -> Result<Vec<ContentLessonView>> {
    if let Some(course_id) = repository::module_course_id(pool, module_id).await? {
        require_view(pool, claims, course_id).await?;
    }
    let rows = repository::list_lessons_by_module(pool, module_id).await?;
    Ok(rows.iter().map(to_lesson_view).collect())
}

pub async fn get_lesson(pool: &PgPool, claims: &Claims, lesson_id: Uuid) -> Result<ContentLessonView> {
    let l = repository::find_lesson_by_id(pool, lesson_id)
        .await?
        .ok_or_else(|| anyhow!("lesson not found"))?;
    if let Some(course_id) = repository::lesson_course_id(pool, lesson_id).await? {
        require_view(pool, claims, course_id).await?;
    }
    Ok(to_lesson_view(&l))
}

pub async fn create_lesson(
    pool: &PgPool,
    claims: &Claims,
    req: CreateLessonRequest,
) -> Result<ContentLessonView> {
    authz::require_instructor(claims)?;
    if let Some(course_id) = repository::module_course_id(pool, req.module_id).await? {
        require_manage(pool, claims, course_id).await?;
    }
    let me = claims.user_id()?;
    let l = repository::insert_lesson(
        pool,
        req.module_id,
        &req.title,
        &req.description,
        req.sort_order,
        me,
        me,
    )
    .await?;
    Ok(to_lesson_view(&l))
}

pub async fn update_lesson(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateLessonRequest,
) -> Result<ContentLessonView> {
    authz::require_instructor(claims)?;
    let me = claims.user_id()?;
    let l = repository::update_lesson_fields(
        pool,
        req.lesson_id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.sort_order,
        me,
    )
    .await?;
    Ok(to_lesson_view(&l))
}

pub async fn delete_lesson(pool: &PgPool, claims: &Claims, lesson_id: Uuid) -> Result<()> {
    authz::require_instructor(claims)?;
    repository::delete_lesson(pool, lesson_id).await?;
    Ok(())
}

pub async fn reorder_lessons(
    pool: &PgPool,
    claims: &Claims,
    req: ReorderLessonsRequest,
) -> Result<()> {
    authz::require_instructor(claims)?;
    for (idx, lesson_id) in req.lesson_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE content_lessons SET sort_order = $1, updated_by = $2, updated_at = now() WHERE id = $3 AND module_id = $4",
        )
        .bind(idx as i32)
        .bind(claims.user_id()?)
        .bind(*lesson_id)
        .bind(req.module_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn set_lesson_status(
    pool: &PgPool,
    claims: &Claims,
    req: SetLessonStatusRequest,
) -> Result<ContentLessonView> {
    authz::require_instructor(claims)?;
    let status = ensure_status(&req.status)?;
    let l = repository::set_lesson_status(pool, req.lesson_id, status.as_str(), claims.user_id()?).await?;
    Ok(to_lesson_view(&l))
}

// ---------------------------------------------------------------------------
// Content item services
// ---------------------------------------------------------------------------

pub async fn list_items(pool: &PgPool, claims: &Claims, lesson_id: Uuid) -> Result<Vec<ContentItemView>> {
    if let Some(course_id) = repository::lesson_course_id(pool, lesson_id).await? {
        require_view(pool, claims, course_id).await?;
    }
    let rows = repository::list_items_by_lesson(pool, lesson_id).await?;
    Ok(rows.iter().map(to_item_view).collect())
}

pub async fn get_item(pool: &PgPool, claims: &Claims, item_id: Uuid) -> Result<ContentItemView> {
    let i = repository::find_item_by_id(pool, item_id)
        .await?
        .ok_or_else(|| anyhow!("content item not found"))?;
    if let Some(course_id) = repository::item_course_id(pool, item_id).await? {
        require_view(pool, claims, course_id).await?;
    }
    Ok(to_item_view(&i))
}

pub async fn create_item(
    pool: &PgPool,
    claims: &Claims,
    req: CreateContentItemRequest,
) -> Result<ContentItemView> {
    authz::require_instructor(claims)?;
    let _ = ensure_content_type(&req.content_type)?;
    if let Some(course_id) = repository::lesson_course_id(pool, req.lesson_id).await? {
        require_manage(pool, claims, course_id).await?;
    }
    let me = claims.user_id()?;
    let i = repository::insert_item(
        pool,
        req.lesson_id,
        &req.content_type,
        &req.title,
        &req.body,
        &req.url,
        req.file_id,
        &req.mime_type,
        req.sort_order,
        me,
        me,
    )
    .await?;
    Ok(to_item_view(&i))
}

pub async fn update_item(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateContentItemRequest,
) -> Result<ContentItemView> {
    authz::require_instructor(claims)?;
    let me = claims.user_id()?;
    let i = repository::update_item_fields(
        pool,
        req.item_id,
        req.content_type.as_deref(),
        req.title.as_deref(),
        req.body.as_deref(),
        req.url.as_deref(),
        req.file_id,
        req.mime_type.as_deref(),
        req.sort_order,
        me,
    )
    .await?;
    Ok(to_item_view(&i))
}

pub async fn delete_item(pool: &PgPool, claims: &Claims, item_id: Uuid) -> Result<()> {
    authz::require_instructor(claims)?;
    repository::delete_item(pool, item_id).await?;
    Ok(())
}

pub async fn reorder_items(
    pool: &PgPool,
    claims: &Claims,
    req: ReorderContentItemsRequest,
) -> Result<()> {
    authz::require_instructor(claims)?;
    for (idx, item_id) in req.item_ids.iter().enumerate() {
        sqlx::query(
            "UPDATE content_items SET sort_order = $1, updated_by = $2, updated_at = now() WHERE id = $3 AND lesson_id = $4",
        )
        .bind(idx as i32)
        .bind(claims.user_id()?)
        .bind(*item_id)
        .bind(req.lesson_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// File services
// ---------------------------------------------------------------------------

pub async fn upload_file(
    pool: &PgPool,
    claims: &Claims,
    req: UploadFileRequest,
) -> Result<ContentFileView> {
    authz::require_instructor(claims)?;
    require_manage(pool, claims, req.course_id).await?;
    let file_id = Uuid::new_v4();
    let storage_path = save_file(req.course_id, file_id, &req.filename, &req.content_base64)?;
    let f = repository::insert_file(
        pool,
        req.course_id,
        &req.filename,
        &req.mime_type,
        req.size_bytes,
        &storage_path,
        claims.user_id()?,
    )
    .await?;
    Ok(to_file_view(&f))
}

pub async fn list_files(
    pool: &PgPool,
    claims: &Claims,
    course_id: Uuid,
) -> Result<Vec<ContentFileView>> {
    require_view(pool, claims, course_id).await?;
    let rows = repository::list_files_by_course(pool, course_id).await?;
    Ok(rows.iter().map(to_file_view).collect())
}

pub async fn get_file(pool: &PgPool, claims: &Claims, file_id: Uuid) -> Result<ContentFileView> {
    let f = repository::find_file_by_id(pool, file_id)
        .await?
        .ok_or_else(|| anyhow!("file not found"))?;
    let course_id = f.course_id;
    require_view(pool, claims, course_id).await?;
    Ok(to_file_view(&f))
}

pub async fn download_file(pool: &PgPool, claims: &Claims, file_id: Uuid) -> Result<FileDownloadView> {
    let f = repository::find_file_by_id(pool, file_id)
        .await?
        .ok_or_else(|| anyhow!("file not found"))?;
    let course_id = f.course_id;
    require_view(pool, claims, course_id).await?;
    let content = fs::read(&f.storage_path)?;
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    let encoded = engine.encode(content);
    Ok(FileDownloadView {
        file_id: f.id,
        filename: f.filename,
        mime_type: f.mime_type,
        content_base64: encoded,
    })
}

pub async fn delete_file(pool: &PgPool, claims: &Claims, file_id: Uuid) -> Result<()> {
    authz::require_instructor(claims)?;
    let f = repository::find_file_by_id(pool, file_id)
        .await?
        .ok_or_else(|| anyhow!("file not found"))?;
    let course_id = f.course_id;
    require_manage(pool, claims, course_id).await?;
    let _ = fs::remove_file(&f.storage_path);
    repository::delete_file(pool, file_id).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Convenience: add a document/video to a course
// ---------------------------------------------------------------------------

/// Map an uploaded file's MIME type (plus extension fallback) to one of the
/// allowed `content_items.content_type` values (documents and media only).
fn content_type_for_mime(mime: &str, filename: &str) -> Result<&'static str> {
    let mime = mime.to_ascii_lowercase();
    let name = filename.to_ascii_lowercase();

    if mime.starts_with("video/") {
        return Ok("video");
    }
    if mime.starts_with("audio/") {
        return Ok("audio");
    }
    if mime.starts_with("image/") {
        return Ok("image");
    }
    if mime == "application/pdf" || name.ends_with(".pdf") {
        return Ok("pdf");
    }
    if mime.contains("word") || name.ends_with(".doc") || name.ends_with(".docx") {
        return Ok("word");
    }
    if mime.contains("powerpoint") || name.ends_with(".ppt") || name.ends_with(".pptx") {
        return Ok("powerpoint");
    }
    if mime.starts_with("text/") {
        return Ok("text");
    }
    Err(anyhow!("unsupported document/video MIME type: {mime}"))
}

/// Upload a document/video straight onto a course, auto-creating a
/// "Course Materials" module (and a "General" lesson) if the course has none.
///
/// The caller must be the course owner/instructor (or an admin).
pub async fn add_course_material(
    pool: &PgPool,
    claims: &Claims,
    req: AddCourseMaterialRequest,
) -> Result<CourseMaterialView> {
    authz::require_instructor(claims)?;
    require_manage(pool, claims, req.course_id).await?;
    let me = claims.user_id()?;

    let content_type = content_type_for_mime(&req.mime_type, &req.filename)?;
    let title = req.title.clone().unwrap_or_else(|| req.filename.clone());

    // 1. Ensure a well-known module + lesson exist for this course.
    let module = match repository::find_module_by_course_title(pool, req.course_id, "Course Materials")
        .await?
    {
        Some(m) => m,
        None => {
            repository::insert_module(
                pool,
                req.course_id,
                "Course Materials",
                "Documents and videos added by the course creator.",
                0,
                me,
                me,
            )
            .await?
        }
    };
    let lessons = repository::list_lessons_by_module(pool, module.id).await?;
    let lesson = match lessons.first() {
        Some(l) => l.clone(),
        None => repository::insert_lesson(pool, module.id, "General", "Learning material.", 0, me, me).await?,
    };

    // 2. Store the uploaded file.
    let file_id = Uuid::new_v4();
    let storage_path = save_file(req.course_id, file_id, &req.filename, &req.content_base64)?;
    let file = repository::insert_file(
        pool,
        req.course_id,
        &req.filename,
        &req.mime_type,
        req.size_bytes,
        &storage_path,
        me,
    )
    .await?;

    // 3. Attach it as a content item inside the lesson.
    let sort_order = repository::list_items_by_lesson(pool, lesson.id).await?.len() as i32;
    let item = repository::insert_item(
        pool,
        lesson.id,
        content_type,
        &title,
        "",
        "",
        Some(file.id),
        &req.mime_type,
        sort_order,
        me,
        me,
    )
    .await?;

    Ok(CourseMaterialView {
        module: to_module_view(&module),
        lesson: to_lesson_view(&lesson),
        item: to_item_view(&item),
        file: to_file_view(&file),
    })
}
