use std::collections::HashMap;
use uuid::Uuid;
use sqlx::PgPool;
use crate::authz::Claims;
use crate::dto::*;
use crate::models::*;
use crate::repository;
use crate::authz;

pub async fn get_or_create_progress(pool: &PgPool, claims: &Claims, course_id: Uuid) -> anyhow::Result<LearningProgress> {
    let student_id = claims.user_id()?;
    if let Some(progress) = repository::get_progress(pool, student_id, course_id, None).await? {
        return Ok(progress);
    }
    let progress = LearningProgress {
        id: uuid::Uuid::new_v4(),
        student_id,
        course_id,
        module_id: None,
        lesson_id: None,
        content_item_id: None,
        status: "not_started".to_string(),
        progress_pct: 0.0,
        completed_at: None,
        last_activity_at: chrono::Utc::now(),
        last_lesson_id: None,
        video_timestamp: 0.0,
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::upsert_progress(pool, &progress).await
}

pub async fn update_progress(pool: &PgPool, claims: &Claims, course_id: Uuid, req: UpdateProgressRequest) -> anyhow::Result<LearningProgress> {
    let student_id = claims.user_id()?;
    let mut progress = repository::get_progress(pool, student_id, course_id, Some(req.lesson_id)).await?
        .unwrap_or_else(|| LearningProgress {
            id: uuid::Uuid::new_v4(),
            student_id,
            course_id,
            module_id: None,
            lesson_id: Some(req.lesson_id),
            content_item_id: None,
            status: "not_started".to_string(),
            progress_pct: 0.0,
            completed_at: None,
            last_activity_at: chrono::Utc::now(),
            last_lesson_id: None,
            video_timestamp: 0.0,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
    if let Some(pct) = req.progress_pct {
        progress.progress_pct = pct.clamp(0.0, 100.0);
        if progress.progress_pct >= 100.0 {
            progress.status = "completed".to_string();
            progress.completed_at = Some(chrono::Utc::now());
        } else if progress.progress_pct > 0.0 {
            progress.status = "in_progress".to_string();
        }
    }
    if let Some(ts) = req.video_timestamp {
        progress.video_timestamp = ts;
    }
    progress.last_activity_at = chrono::Utc::now();
    progress.last_lesson_id = Some(req.lesson_id);
    if let Some(event_type) = req.event_type {
        progress.metadata = serde_json::json!({ "last_event": event_type });
    }
    repository::upsert_progress(pool, &progress).await
}

pub async fn mark_lesson_complete(pool: &PgPool, claims: &Claims, course_id: Uuid, req: MarkCompleteRequest) -> anyhow::Result<LearningProgress> {
    let student_id = claims.user_id()?;
    let mut progress = repository::get_progress(pool, student_id, course_id, Some(req.lesson_id)).await?
        .unwrap_or_else(|| LearningProgress {
            id: uuid::Uuid::new_v4(),
            student_id,
            course_id,
            module_id: None,
            lesson_id: Some(req.lesson_id),
            content_item_id: None,
            status: "not_started".to_string(),
            progress_pct: 0.0,
            completed_at: None,
            last_activity_at: chrono::Utc::now(),
            last_lesson_id: None,
            video_timestamp: 0.0,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
    progress.status = "completed".to_string();
    progress.progress_pct = 100.0;
    progress.completed_at = Some(chrono::Utc::now());
    progress.last_activity_at = chrono::Utc::now();
    progress.last_lesson_id = Some(req.lesson_id);
    repository::upsert_progress(pool, &progress).await
}

pub async fn get_course_progress(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<CourseProgressResponse> {
    let progresses = repository::list_course_progress(pool, student_id, course_id).await?;
    let total_lessons = repository::count_course_lessons(pool, course_id).await?;
    let lessons_completed = progresses.iter().filter(|p| p.status == "completed").count() as i64;
    let modules_completed = progresses.iter().filter(|p| p.status == "completed" && p.module_id.is_some()).count() as i64;
    let overall_progress_pct = if total_lessons > 0 {
        (lessons_completed as f64 / total_lessons as f64) * 100.0
    } else {
        0.0
    };
    let last_progress = progresses.iter().max_by_key(|p| p.last_activity_at);
    let last_activity_at = last_progress.map(|p| p.last_activity_at).unwrap_or_else(|| chrono::Utc::now());
    let last_lesson_id = last_progress.and_then(|p| p.last_lesson_id);
    Ok(CourseProgressResponse {
        course_id,
        student_id,
        overall_progress_pct,
        modules_completed,
        lessons_completed,
        total_lessons,
        last_activity_at,
        last_lesson_id,
    })
}

pub async fn resume_learning(pool: &PgPool, claims: &Claims, course_id: Uuid) -> anyhow::Result<Option<LearningProgress>> {
    let student_id = claims.user_id()?;
    let progresses = repository::list_course_progress(pool, student_id, course_id).await?;
    let in_progress = progresses.iter().find(|p| p.status == "in_progress");
    if let Some(progress) = in_progress {
        return Ok(Some(progress.clone()));
    }
    let not_started = progresses.iter().find(|p| p.status == "not_started");
    if let Some(progress) = not_started {
        return Ok(Some(progress.clone()));
    }
    Ok(None)
}

pub async fn get_progress_events(pool: &PgPool, progress_id: Uuid) -> anyhow::Result<Vec<ProgressEvent>> {
    repository::list_events(pool, progress_id).await
}
