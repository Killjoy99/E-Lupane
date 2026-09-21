//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::services;
use crate::dto::*;

pub const PLUGIN_ID: &str = "com.elupane.content";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "content.error", &err.to_string()),
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
        // ---- Modules ----
        "content.module.list" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::list_modules(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }
        "content.module.get" => {
            let claims = authorize(&command.payload)?;
            let req: ModuleIdRequest = parse_payload(&command.payload)?;
            let view = services::get_module(pool, &claims, req.module_id).await?;
            reply(command, &view)
        }
        "content.module.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateModuleRequest = parse_payload(&command.payload)?;
            let view = services::create_module(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.module.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateModuleRequest = parse_payload(&command.payload)?;
            let view = services::update_module(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.module.delete" => {
            let claims = authorize(&command.payload)?;
            let req: ModuleIdRequest = parse_payload(&command.payload)?;
            services::delete_module(pool, &claims, req.module_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "module_id": req.module_id }),
            ))
        }
        "content.module.reorder" => {
            let claims = authorize(&command.payload)?;
            let req: ReorderModulesRequest = parse_payload(&command.payload)?;
            services::reorder_modules(pool, &claims, req).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({ "reordered": true })))
        }
        "content.module.set_status" => {
            let claims = authorize(&command.payload)?;
            let req: SetModuleStatusRequest = parse_payload(&command.payload)?;
            let view = services::set_module_status(pool, &claims, req).await?;
            reply(command, &view)
        }

        // ---- Lessons ----
        "content.lesson.list" => {
            let claims = authorize(&command.payload)?;
            let req: ModuleIdRequest = parse_payload(&command.payload)?;
            let view = services::list_lessons(pool, &claims, req.module_id).await?;
            reply(command, &view)
        }
        "content.lesson.get" => {
            let claims = authorize(&command.payload)?;
            let req: LessonIdRequest = parse_payload(&command.payload)?;
            let view = services::get_lesson(pool, &claims, req.lesson_id).await?;
            reply(command, &view)
        }
        "content.lesson.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateLessonRequest = parse_payload(&command.payload)?;
            let view = services::create_lesson(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.lesson.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateLessonRequest = parse_payload(&command.payload)?;
            let view = services::update_lesson(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.lesson.delete" => {
            let claims = authorize(&command.payload)?;
            let req: LessonIdRequest = parse_payload(&command.payload)?;
            services::delete_lesson(pool, &claims, req.lesson_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "lesson_id": req.lesson_id }),
            ))
        }
        "content.lesson.reorder" => {
            let claims = authorize(&command.payload)?;
            let req: ReorderLessonsRequest = parse_payload(&command.payload)?;
            services::reorder_lessons(pool, &claims, req).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({ "reordered": true })))
        }
        "content.lesson.set_status" => {
            let claims = authorize(&command.payload)?;
            let req: SetLessonStatusRequest = parse_payload(&command.payload)?;
            let view = services::set_lesson_status(pool, &claims, req).await?;
            reply(command, &view)
        }

        // ---- Content Items ----
        "content.item.list" => {
            let claims = authorize(&command.payload)?;
            let req: LessonIdRequest = parse_payload(&command.payload)?;
            let view = services::list_items(pool, &claims, req.lesson_id).await?;
            reply(command, &view)
        }
        "content.item.get" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            let view = services::get_item(pool, &claims, req.id).await?;
            reply(command, &view)
        }
        "content.item.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateContentItemRequest = parse_payload(&command.payload)?;
            let view = services::create_item(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.item.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateContentItemRequest = parse_payload(&command.payload)?;
            let view = services::update_item(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.item.delete" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            services::delete_item(pool, &claims, req.id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "item_id": req.id }),
            ))
        }
        "content.item.reorder" => {
            let claims = authorize(&command.payload)?;
            let req: ReorderContentItemsRequest = parse_payload(&command.payload)?;
            services::reorder_items(pool, &claims, req).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({ "reordered": true })))
        }

        // ---- Files ----
        "content.file.upload" => {
            let claims = authorize(&command.payload)?;
            let req: UploadFileRequest = parse_payload(&command.payload)?;
            let view = services::upload_file(pool, &claims, req).await?;
            reply(command, &view)
        }
        "content.file.list" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::list_files(pool, &claims, req.course_id).await?;
            reply(command, &view)
        }
        "content.file.get" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            let view = services::get_file(pool, &claims, req.id).await?;
            reply(command, &view)
        }
        "content.file.download" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            let view = services::download_file(pool, &claims, req.id).await?;
            reply(command, &view)
        }
        "content.file.delete" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            services::delete_file(pool, &claims, req.id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "file_id": req.id }),
            ))
        }

        "content.material.add" => {
            let claims = authorize(&command.payload)?;
            let req: AddCourseMaterialRequest = parse_payload(&command.payload)?;
            let view = services::add_course_material(pool, &claims, req).await?;
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
