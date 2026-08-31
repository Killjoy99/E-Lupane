use std::collections::HashMap;
use uuid::Uuid;
use sqlx::PgPool;
use crate::authz::Claims;
use crate::dto::*;
use crate::models::*;
use crate::repository;
use crate::authz;

pub async fn track_event(pool: &PgPool, claims: &Claims, event_type: &str, course_id: Option<Uuid>, resource_type: Option<&str>, resource_id: Option<Uuid>, metadata: serde_json::Value) -> anyhow::Result<AnalyticsEvent> {
    let event = AnalyticsEvent {
        id: uuid::Uuid::new_v4(),
        event_type: event_type.to_string(),
        user_id: claims.user_id()?,
        course_id,
        resource_type: resource_type.map(|s| s.to_string()),
        resource_id,
        metadata,
        occurred_at: chrono::Utc::now(),
    };
    repository::insert_event(pool, &event).await
}

pub async fn get_student_progress(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<StudentProgressResponse> {
    let events = repository::list_events(pool, Some(student_id), Some(course_id), None, None, None).await?;
    let lessons_completed = events.iter().filter(|e| e.event_type == "lesson.completed").count() as i64;
    let time_spent: i64 = events.iter().filter(|e| e.event_type == "time.spent").filter_map(|e| e.metadata.get("minutes").and_then(|v| v.as_i64())).sum();
    let quiz_scores: Vec<f64> = events.iter().filter(|e| e.event_type == "quiz.completed").filter_map(|e| e.metadata.get("score").and_then(|v| v.as_f64())).collect();
    let quiz_average = if quiz_scores.is_empty() { None } else { Some(quiz_scores.iter().sum::<f64>() / quiz_scores.len() as f64) };
    let assignment_scores: Vec<f64> = events.iter().filter(|e| e.event_type == "assignment.submitted").filter_map(|e| e.metadata.get("score").and_then(|v| v.as_f64())).collect();
    let assignment_average = if assignment_scores.is_empty() { None } else { Some(assignment_scores.iter().sum::<f64>() / assignment_scores.len() as f64) };
    let total_events = events.len() as i64;
    let completion_rate = if total_events > 0 { (lessons_completed as f64 / total_events.max(1) as f64) * 100.0 } else { 0.0 };

    Ok(StudentProgressResponse {
        student_id,
        course_id,
        completion_rate,
        lessons_completed,
        total_lessons: total_events,
        time_spent_minutes: time_spent,
        quiz_average,
        assignment_average,
    })
}

pub async fn get_instructor_analytics(pool: &PgPool, instructor_id: Uuid, course_id: Uuid) -> anyhow::Result<InstructorAnalyticsResponse> {
    let events = repository::list_events(pool, None, Some(course_id), None, None, None).await?;
    let unique_students: std::collections::HashSet<Uuid> = events.iter().filter(|e| e.event_type == "student.enrolled").map(|e| e.user_id).collect();
    let enrollment_count = unique_students.len() as i64;
    let completed_events = events.iter().filter(|e| e.event_type == "lesson.completed").count() as i64;
    let total_events = events.len() as i64;
    let avg_completion_rate = if total_events > 0 { (completed_events as f64 / total_events.max(1) as f64) * 100.0 } else { 0.0 };
    let grades: Vec<f64> = events.iter().filter(|e| e.event_type == "grade.assigned").filter_map(|e| e.metadata.get("score").and_then(|v| v.as_f64())).collect();
    let avg_grade = if grades.is_empty() { None } else { Some(grades.iter().sum::<f64>() / grades.len() as f64) };
    let engagement_events = events.iter().filter(|e| e.event_type == "engagement").count() as f64;
    let engagement_score = if enrollment_count > 0 { engagement_events / enrollment_count as f64 } else { 0.0 };

    Ok(InstructorAnalyticsResponse {
        instructor_id,
        course_id,
        enrollment_count,
        avg_completion_rate,
        avg_grade,
        engagement_score,
    })
}

pub async fn get_admin_analytics(pool: &PgPool, institution_id: Uuid, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> anyhow::Result<AdminAnalyticsResponse> {
    let events = repository::list_events(pool, None, None, None, Some(start), Some(end)).await?;
    let active_users: std::collections::HashSet<Uuid> = events.iter().map(|e| e.user_id).collect();
    let new_enrollments = events.iter().filter(|e| e.event_type == "student.enrolled").count() as i64;
    let completions = events.iter().filter(|e| e.event_type == "lesson.completed").count() as i64;
    let total_events = events.len() as i64;
    let completion_rate = if total_events > 0 { (completions as f64 / total_events.max(1) as f64) * 100.0 } else { 0.0 };
    let dropout_count = events.iter().filter(|e| e.event_type == "student.dropped").count() as i64;
    let dropout_rate = if new_enrollments > 0 { (dropout_count as f64 / new_enrollments as f64) * 100.0 } else { 0.0 };
    let course_count = events.iter().filter(|e| e.event_type == "course.created").count() as i64;
    let instructor_count = events.iter().filter(|e| e.event_type == "instructor.created").count() as i64;
    let revenue: f64 = events.iter().filter(|e| e.event_type == "payment.received").filter_map(|e| e.metadata.get("amount").and_then(|v| v.as_f64())).sum();

    Ok(AdminAnalyticsResponse {
        active_users: active_users.len() as i64,
        new_enrollments,
        completion_rate,
        dropout_rate,
        course_count,
        instructor_count,
        revenue: Some(revenue),
    })
}

pub async fn generate_report(pool: &PgPool, claims: &Claims, req: GenerateReportRequest) -> anyhow::Result<AnalyticsReport> {
    authz::require_instructor(claims)?;
    let data = serde_json::json!({ "message": "report generated", "type": req.report_type, "format": req.format });
    let report = AnalyticsReport {
        id: uuid::Uuid::new_v4(),
        report_type: req.report_type,
        format: req.format,
        filters: req.filters,
        data,
        file_path: None,
        generated_by: claims.user_id()?,
        generated_at: chrono::Utc::now(),
    };
    repository::create_report(pool, &report).await
}

pub async fn list_reports(pool: &PgPool, generated_by: Option<Uuid>) -> anyhow::Result<Vec<AnalyticsReport>> {
    repository::list_reports(pool, None, generated_by).await
}
