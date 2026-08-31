use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.grades";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "grades.error", &err.to_string()),
    }
}

fn authorize(payload: &Value) -> Result<Claims> {
    let token = payload
        .get("token")
        .and_then(|t| t.as_str())
        .ok_or_else(|| anyhow!("missing 'token'"))?;
    authz::verify_token(token)
}

fn parse_payload<T: DeserializeOwned>(payload: &Value) -> Result<T> {
    serde_json::from_value(payload.clone()).map_err(|e| anyhow!("invalid payload: {e}"))
}

fn reply<T: serde::Serialize>(command: &Message, value: &T) -> Result<Message> {
    Ok(Message::reply_to(command, PLUGIN_ID, serde_json::to_value(value)?))
}

async fn dispatch(pool: &PgPool, command: &Message) -> Result<Message> {
    match command.op.as_str() {
        "grades.category.list" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::list_categories(pool, req.course_id).await?;
            reply(command, &view)
        }
        "grades.category.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateGradeCategoryRequest = parse_payload(&command.payload)?;
            let view = services::create_category(pool, &claims, req).await?;
            reply(command, &view)
        }
        "grades.category.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateCategoryRequest = parse_payload(&command.payload)?;
            let view = services::update_category(pool, &claims, req).await?;
            reply(command, &view)
        }
        "grades.category.delete" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            services::delete_category(pool, &claims, req.id).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({ "deleted": true, "category_id": req.id })))
        }

        "grades.scale.list" => {
            let claims = authorize(&command.payload)?;
            let req: InstitutionIdRequest = parse_payload(&command.payload)?;
            let view = services::list_scales(pool, req.institution_id).await?;
            reply(command, &view)
        }
        "grades.scale.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateGradeScaleRequest = parse_payload(&command.payload)?;
            let view = services::create_scale(pool, &claims, req).await?;
            reply(command, &view)
        }

        "grades.entry.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateGradeEntryRequest = parse_payload(&command.payload)?;
            let view = services::create_entry(pool, &claims, req).await?;
            reply(command, &view)
        }
        "grades.entry.list" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::list_entries(pool, req.student_id, req.course_id).await?;
            reply(command, &view)
        }
        "grades.entry.override" => {
            let claims = authorize(&command.payload)?;
            let req: OverrideGradeEntryRequest = parse_payload(&command.payload)?;
            let view = services::override_entry(pool, &claims, req).await?;
            reply(command, &view)
        }
        "grades.entry.history" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            let view = services::get_history(pool, req.id).await?;
            reply(command, &view)
        }

        "grades.final.get" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::get_final_grade(pool, req.student_id, req.course_id).await?;
            reply(command, &view)
        }
        "grades.final.calculate" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::calculate_final_grade(pool, &claims, req.student_id, req.course_id).await?;
            reply(command, &view)
        }
        "grades.final.list" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::list_course_final_grades(pool, req.course_id).await?;
            reply(command, &view)
        }

        "grades.student.view" => {
            let claims = authorize(&command.payload)?;
            let req: StudentCourseRequest = parse_payload(&command.payload)?;
            let view = services::student_grade_view(pool, req.student_id, req.course_id).await?;
            reply(command, &view)
        }

        "events.subscribe" => Ok(Message::reply_to(
            command,
            PLUGIN_ID,
            serde_json::json!({ "subscribed": true }),
        )),

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}
