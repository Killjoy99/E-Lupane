use sqlx::PgPool;
use uuid::Uuid;
use crate::models::*;

pub async fn upsert_progress(pool: &PgPool, progress: &LearningProgress) -> anyhow::Result<LearningProgress> {
    let row = sqlx::query_as::<_, LearningProgress>(
        r#"INSERT INTO learning_progress (id, student_id, course_id, module_id, lesson_id, content_item_id, status, progress_pct, completed_at, last_activity_at, last_lesson_id, video_timestamp, metadata)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
           ON CONFLICT (student_id, course_id, lesson_id) WHERE lesson_id IS NOT NULL
           DO UPDATE SET status = EXCLUDED.status, progress_pct = EXCLUDED.progress_pct, completed_at = EXCLUDED.completed_at, last_activity_at = EXCLUDED.last_activity_at, last_lesson_id = EXCLUDED.last_lesson_id, video_timestamp = EXCLUDED.video_timestamp, metadata = EXCLUDED.metadata, updated_at = now()
           RETURNING id, student_id, course_id, module_id, lesson_id, content_item_id, status, progress_pct, completed_at, last_activity_at, last_lesson_id, video_timestamp, metadata, created_at, updated_at"#,
    )
    .bind(progress.id)
    .bind(progress.student_id)
    .bind(progress.course_id)
    .bind(progress.module_id)
    .bind(progress.lesson_id)
    .bind(progress.content_item_id)
    .bind(&progress.status)
    .bind(progress.progress_pct)
    .bind(progress.completed_at)
    .bind(progress.last_activity_at)
    .bind(progress.last_lesson_id)
    .bind(progress.video_timestamp)
    .bind(&progress.metadata)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_progress(pool: &PgPool, student_id: Uuid, course_id: Uuid, lesson_id: Option<Uuid>) -> anyhow::Result<Option<LearningProgress>> {
    if let Some(lid) = lesson_id {
        let row = sqlx::query_as::<_, LearningProgress>("SELECT * FROM learning_progress WHERE student_id = $1 AND course_id = $2 AND lesson_id = $3")
            .bind(student_id)
            .bind(course_id)
            .bind(lid)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    } else {
        let row = sqlx::query_as::<_, LearningProgress>("SELECT * FROM learning_progress WHERE student_id = $1 AND course_id = $2 AND lesson_id IS NULL")
            .bind(student_id)
            .bind(course_id)
            .fetch_optional(pool)
            .await?;
        Ok(row)
    }
}

pub async fn list_course_progress(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<Vec<LearningProgress>> {
    let rows = sqlx::query_as::<_, LearningProgress>("SELECT * FROM learning_progress WHERE student_id = $1 AND course_id = $2 ORDER BY created_at DESC")
        .bind(student_id)
        .bind(course_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn list_course_progress_by_instructor(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<LearningProgress>> {
    let rows = sqlx::query_as::<_, LearningProgress>("SELECT * FROM learning_progress WHERE course_id = $1 ORDER BY student_id, created_at DESC")
        .bind(course_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn insert_event(pool: &PgPool, event: &ProgressEvent) -> anyhow::Result<ProgressEvent> {
    let row = sqlx::query_as::<_, ProgressEvent>(
        r#"INSERT INTO progress_events (id, progress_id, event_type, data, occurred_at)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, progress_id, event_type, data, occurred_at"#,
    )
    .bind(event.id)
    .bind(event.progress_id)
    .bind(&event.event_type)
    .bind(&event.data)
    .bind(event.occurred_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_events(pool: &PgPool, progress_id: Uuid) -> anyhow::Result<Vec<ProgressEvent>> {
    let rows = sqlx::query_as::<_, ProgressEvent>("SELECT * FROM progress_events WHERE progress_id = $1 ORDER BY occurred_at DESC")
        .bind(progress_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn count_course_lessons(pool: &PgPool, course_id: Uuid) -> anyhow::Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM content_lessons WHERE module_id IN (SELECT id FROM content_modules WHERE course_id = $1)")
        .bind(course_id)
        .fetch_one(pool)
        .await?;
    Ok(count)
}
