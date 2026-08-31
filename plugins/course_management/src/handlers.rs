//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    CourseIdRequest, CreateCourseRequest, InstructorRequest, ListCoursesRequest, SetStatusRequest,
    UpdateCourseRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.course_management";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "courses.error", &err.to_string()),
    }
}

/// Extract the caller's JWT from the payload and verify it.
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
        // ---- Course lifecycle ----
        "course.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateCourseRequest = parse_payload(&command.payload)?;
            let view = services::create_course(pool, &claims, req).await?;
            reply(command, &view)
        }
        "course.get" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::get_course(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }
        "course.list" => {
            let claims = authorize(&command.payload)?;
            let req: ListCoursesRequest = parse_payload(&command.payload)?;
            let view = services::list_courses(pool, &claims, req.status, req.mine, req.search).await?;
            reply(command, &view)
        }
        "course.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateCourseRequest = parse_payload(&command.payload)?;
            let view = services::update_course(pool, &claims, req).await?;
            reply(command, &view)
        }
        "course.delete" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            services::delete_course(pool, &claims, req.course_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "course_id": req.course_id }),
            ))
        }
        "course.archive" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::set_status(pool, &claims, req.course_id, "archived").await?;
            reply(command, &view)
        }
        "course.duplicate" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::duplicate_course(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }
        "course.preview" => {
            // Preview shows the full course regardless of status — managers only.
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::preview_course(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }

        // ---- Publishing / status ----
        "course.set_status" => {
            let claims = authorize(&command.payload)?;
            let req: SetStatusRequest = parse_payload(&command.payload)?;
            let view = services::set_status(pool, &claims, req.course_id, &req.status).await?;
            reply(command, &view)
        }
        "course.publish" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::set_status(pool, &claims, req.course_id, "published").await?;
            reply(command, &view)
        }
        "course.unpublish" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::set_status(pool, &claims, req.course_id, "unpublished").await?;
            reply(command, &view)
        }
        "course.submit_review" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::set_status(pool, &claims, req.course_id, "pending_review").await?;
            reply(command, &view)
        }

        // ---- Instructor management ----
        "course.instructor.add" => {
            let claims = authorize(&command.payload)?;
            let req: InstructorRequest = parse_payload(&command.payload)?;
            let view = services::add_instructor(pool, &claims, req.course_id, req.user_id).await?;
            reply(command, &view)
        }
        "course.instructor.remove" => {
            let claims = authorize(&command.payload)?;
            let req: InstructorRequest = parse_payload(&command.payload)?;
            let view = services::remove_instructor(pool, &claims, req.course_id, req.user_id).await?;
            reply(command, &view)
        }

        // ---- Event subscription placeholder ----
        "events.subscribe" => Ok(Message::reply_to(
            command,
            PLUGIN_ID,
            serde_json::json!({ "subscribed": true }),
        )),

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}


