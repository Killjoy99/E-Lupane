//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    AssignmentIdRequest, AttachResourceRequest, CreateAssignmentRequest, CreateRubricRequest,
    GradeHistoryRequest, GradeRequest, ResourceIdRequest, RubricIdRequest, SubmissionFileRequest,
    SubmissionStatusRequest, SubmitRequest, UpdateAssignmentRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.assignments";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(
            command,
            PLUGIN_ID,
            "assignments.error",
            &err.to_string(),
        ),
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
        // ---- Instructor: assignment lifecycle ----
        "assignment.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateAssignmentRequest = parse_payload(&command.payload)?;
            let view = services::create_assignment(pool, &claims, req).await?;
            reply(command, &view)
        }
        "assignment.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateAssignmentRequest = parse_payload(&command.payload)?;
            let view = services::update_assignment(pool, &claims, req).await?;
            reply(command, &view)
        }
        "assignment.delete" => {
            let claims = authorize(&command.payload)?;
            let req: AssignmentIdRequest = parse_payload(&command.payload)?;
            services::delete_assignment(pool, &claims, req.assignment_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "deleted": true, "assignment_id": req.assignment_id }),
            ))
        }
        "assignment.get" => {
            authorize(&command.payload)?;
            let req: AssignmentIdRequest = parse_payload(&command.payload)?;
            let view = services::get_assignment(pool, req.assignment_id).await?;
            reply(command, &view)
        }
        "assignment.list" => {
            authorize(&command.payload)?;
            let view = services::list_assignments(pool).await?;
            reply(command, &view)
        }

        // ---- Instructor: resources ----
        "assignment.resource.attach" => {
            let claims = authorize(&command.payload)?;
            let req: AttachResourceRequest = parse_payload(&command.payload)?;
            let view = services::attach_resource(pool, &claims, req).await?;
            reply(command, &view)
        }
        "assignment.resource.remove" => {
            let claims = authorize(&command.payload)?;
            let req: ResourceIdRequest = parse_payload(&command.payload)?;
            services::remove_resource(pool, &claims, req.resource_id).await?;
            Ok(Message::reply_to(
                command,
                PLUGIN_ID,
                serde_json::json!({ "removed": true, "resource_id": req.resource_id }),
            ))
        }
        "assignment.resource.download" => {
            let claims = authorize(&command.payload)?;
            let req: ResourceIdRequest = parse_payload(&command.payload)?;
            let view = services::download_resource(pool, &claims, req.resource_id).await?;
            reply(command, &view)
        }

        // ---- Student: submissions ----
        "submission.submit" => {
            let claims = authorize(&command.payload)?;
            let req: SubmitRequest = parse_payload(&command.payload)?;
            let view = services::submit(pool, &claims, req).await?;
            reply(command, &view)
        }
        "submission.status" => {
            let claims = authorize(&command.payload)?;
            let req: SubmissionStatusRequest = parse_payload(&command.payload)?;
            let view = services::submission_status(pool, &claims, req.assignment_id).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, view))
        }
        "submission.download" => {
            let claims = authorize(&command.payload)?;
            let req: SubmissionFileRequest = parse_payload(&command.payload)?;
            let view = services::download_submission_file(pool, &claims, req.submission_file_id)
                .await?;
            reply(command, &view)
        }

        // ---- Instructor: grading ----
        "grade.create" => {
            let claims = authorize(&command.payload)?;
            let req: GradeRequest = parse_payload(&command.payload)?;
            let view = services::grade(pool, &claims, req).await?;
            reply(command, &view)
        }
        "grade.get" => {
            let claims = authorize(&command.payload)?;
            let req: GradeHistoryRequest = parse_payload(&command.payload)?;
            let view = services::get_grade(pool, &claims, req.submission_id).await?;
            reply(command, &view)
        }
        "grade.history" => {
            let claims = authorize(&command.payload)?;
            let req: GradeHistoryRequest = parse_payload(&command.payload)?;
            let view = services::grade_history(pool, &claims, req.submission_id).await?;
            reply(command, &view)
        }

        // ---- Rubrics ----
        "rubric.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateRubricRequest = parse_payload(&command.payload)?;
            let view = services::create_rubric(
                pool,
                &claims,
                req.assignment_id,
                &req.name,
                &req.description,
                req.criteria,
            )
            .await?;
            reply(command, &view)
        }
        "rubric.list" => {
            authorize(&command.payload)?;
            let req: AssignmentIdRequest = parse_payload(&command.payload)?;
            let view = services::list_rubrics(pool, req.assignment_id).await?;
            reply(command, &view)
        }
        "rubric.get" => {
            authorize(&command.payload)?;
            let req: RubricIdRequest = parse_payload(&command.payload)?;
            let view = services::get_rubric(pool, req.rubric_id).await?;
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
