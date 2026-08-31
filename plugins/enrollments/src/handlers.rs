//! Command dispatch: maps incoming envelope ops to services and builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    BulkEnrollRequest, ChangeStateRequest, CourseIdRequest, CreateInviteRequest, EnrollRequest,
    EnrollmentIdRequest, EnrollmentState, JoinRequest, ListEnrollmentsRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.enrollments";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "enrollments.error", &err.to_string()),
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
        // ---- Enrollment lifecycle ----
        "enrollment.self" => {
            let claims = authorize(&command.payload)?;
            let req: EnrollRequest = parse_payload(&command.payload)?;
            let user_id = claims.user_id()?;
            let view = services::self_enroll(pool, user_id, req.course_id, req.expires_at).await?;
            reply(command, &view)
        }
        "enrollment.enroll" => {
            let claims = authorize(&command.payload)?;
            let req: EnrollRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::enroll_by_manager(
                pool,
                actor,
                claims.is_admin(),
                req.course_id,
                req.student
                    .as_deref()
                    .ok_or_else(|| anyhow!("missing 'student'"))?,
                req.expires_at,
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.bulk" => {
            let claims = authorize(&command.payload)?;
            let req: BulkEnrollRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::bulk_enroll(
                pool,
                actor,
                claims.is_admin(),
                req.course_id,
                &req.students,
                req.expires_at,
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.approve" => {
            let claims = authorize(&command.payload)?;
            let req: ChangeStateRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::change_state(
                pool,
                req.enrollment_id,
                EnrollmentState::Active,
                actor,
                claims.is_admin(),
                req.reason.as_deref(),
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.suspend" => {
            let claims = authorize(&command.payload)?;
            let req: ChangeStateRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::change_state(
                pool,
                req.enrollment_id,
                EnrollmentState::Suspended,
                actor,
                claims.is_admin(),
                req.reason.as_deref(),
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.cancel" => {
            let claims = authorize(&command.payload)?;
            let req: ChangeStateRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::change_state(
                pool,
                req.enrollment_id,
                EnrollmentState::Cancelled,
                actor,
                claims.is_admin(),
                req.reason.as_deref(),
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.complete" => {
            let claims = authorize(&command.payload)?;
            let req: ChangeStateRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::change_state(
                pool,
                req.enrollment_id,
                EnrollmentState::Completed,
                actor,
                claims.is_admin(),
                req.reason.as_deref(),
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.reenroll" => {
            let claims = authorize(&command.payload)?;
            let req: EnrollmentIdRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::re_enroll(pool, actor, claims.is_admin(), req.enrollment_id).await?;
            reply(command, &view)
        }

        // ---- Roster / queries ----
        "enrollment.roster" => {
            let claims = authorize(&command.payload)?;
            let req: ListEnrollmentsRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::course_roster(
                pool,
                actor,
                claims.is_admin(),
                req.course_id,
                req.state.as_deref(),
            )
            .await?;
            reply(command, &crate::dto::EnrollmentsView { enrollments: view })
        }
        "enrollment.mine" => {
            let claims = authorize(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::my_enrollments(pool, actor).await?;
            reply(command, &crate::dto::EnrollmentsView { enrollments: view })
        }
        "enrollment.get" => {
            let claims = authorize(&command.payload)?;
            let req: EnrollmentIdRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::get_enrollment(pool, actor, claims.is_admin(), req.enrollment_id).await?;
            reply(command, &view)
        }
        "enrollment.history" => {
            let claims = authorize(&command.payload)?;
            let req: EnrollmentIdRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view =
                services::enrollment_history(pool, actor, claims.is_admin(), req.enrollment_id).await?;
            reply(command, &crate::dto::HistoryEntriesView { history: view })
        }
        "enrollment.access" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::check_content_access(pool, actor, claims.is_admin(), req.course_id).await?;
            reply(command, &view)
        }

        // ---- Invitations ----
        "enrollment.invite.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateInviteRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::create_invite(
                pool,
                actor,
                claims.is_admin(),
                req.course_id,
                req.max_uses,
                req.expires_at,
            )
            .await?;
            reply(command, &view)
        }
        "enrollment.invite.list" => {
            let claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::course_invites(pool, actor, claims.is_admin(), req.course_id).await?;
            reply(command, &crate::dto::InvitesView { invites: view })
        }
        "enrollment.invite.revoke" => {
            let claims = authorize(&command.payload)?;
            let req: EnrollmentIdRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::revoke_invite(pool, actor, claims.is_admin(), req.enrollment_id).await?;
            reply(command, &view)
        }
        "enrollment.join" => {
            let claims = authorize(&command.payload)?;
            let req: JoinRequest = parse_payload(&command.payload)?;
            let actor = claims.user_id()?;
            let view = services::join_by_invite(pool, actor, &req.invite_token).await?;
            reply(command, &view)
        }

        // ---- Maintenance ----
        "enrollment.sweep" => {
            authorize(&command.payload)?;
            let expired = services::sweep_expired(pool).await?;
            reply(command, &serde_json::json!({ "expired": expired }))
        }
        _ => Err(anyhow!("unknown op '{}'", command.op)),
    }
}