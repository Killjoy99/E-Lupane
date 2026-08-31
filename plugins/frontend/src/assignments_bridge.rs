//! Server-only bridge between the Dioxus fullstack server and the assignments
//! plugin. Performs request/response RPC against `com.elupane.assignments`
//! over the Redis messaging layer — mirroring the auth bridge.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::json;

use messaging_core::rpc::RequestClient;

use crate::assignments_types::{
    AssignmentInput, AssignmentView, DownloadView, FileUploadInput, GradeHistoryView, GradeView,
    LinkInput, ResourceView, SubmissionView,
};

/// The assignments plugin's registered id (matches `plugins/assignments/plugin.toml`).
pub const ASSIGNMENTS_PLUGIN: &str = "com.elupane.assignments";

/// Who we are on the messaging layer.
const SOURCE: &str = "com.elupane.frontend";

fn client() -> RequestClient {
    RequestClient::new(SOURCE).with_timeout(Duration::from_secs(8))
}

async fn request_typed<T: serde::de::DeserializeOwned>(
    op: &str,
    payload: serde_json::Value,
    what: &str,
) -> Result<T> {
    let reply = client().request(ASSIGNMENTS_PLUGIN, op, payload).await?;
    if let Some(err) = &reply.error {
        return Err(anyhow!("{}: {}", what, err.message));
    }
    serde_json::from_value(reply.payload)
        .map_err(|e| anyhow!("{} returned an unexpected payload: {e}", what))
}

fn token_payload(
    token: &str,
    mut extra: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    extra.insert("token".to_string(), json!(token));
    serde_json::Value::Object(extra)
}

/// List all assignments.
pub async fn list_assignments(token: &str) -> Result<Vec<AssignmentView>> {
    let value: serde_json::Value =
        request_typed("assignment.list", token_payload(token, serde_json::Map::new()), "assignments plugin").await?;
    let list = value
        .get("assignments")
        .cloned()
        .ok_or_else(|| anyhow!("assignments plugin reply missing 'assignments'"))?;
    serde_json::from_value(list).map_err(|e| anyhow!("invalid assignments list payload: {e}"))
}

/// Get a single assignment.
pub async fn get_assignment(token: &str, assignment_id: uuid::Uuid) -> Result<AssignmentView> {
    request_typed(
        "assignment.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("assignment_id".to_string(), json!(assignment_id))]),
        ),
        "assignments plugin",
    )
    .await
}

/// Create an assignment.
pub async fn create_assignment(token: &str, input: AssignmentInput) -> Result<AssignmentView> {
    let map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    request_typed("assignment.create", token_payload(token, map), "assignments plugin").await
}

/// Update an assignment.
pub async fn update_assignment(
    token: &str,
    assignment_id: uuid::Uuid,
    input: AssignmentInput,
) -> Result<AssignmentView> {
    let mut map = serde_json::to_value(input)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    map.insert("assignment_id".to_string(), json!(assignment_id));
    request_typed("assignment.update", token_payload(token, map), "assignments plugin").await
}

/// Delete an assignment.
pub async fn delete_assignment(token: &str, assignment_id: uuid::Uuid) -> Result<()> {
    request_typed::<serde_json::Value>(
        "assignment.delete",
        token_payload(
            token,
            serde_json::Map::from_iter([("assignment_id".to_string(), json!(assignment_id))]),
        ),
        "assignments plugin",
    )
    .await?;
    Ok(())
}

/// Attach a resource file to an assignment.
pub async fn attach_resource(
    token: &str,
    assignment_id: uuid::Uuid,
    name: &str,
    mime_type: &str,
    content_base64: &str,
) -> Result<ResourceView> {
    request_typed(
        "assignment.resource.attach",
        token_payload(
            token,
            serde_json::Map::from_iter([
                ("assignment_id".to_string(), json!(assignment_id)),
                ("name".to_string(), json!(name)),
                ("mime_type".to_string(), json!(mime_type)),
                ("content_base64".to_string(), json!(content_base64)),
            ]),
        ),
        "assignments plugin",
    )
    .await
}

/// Download an attached resource (base64 content).
pub async fn download_resource(token: &str, resource_id: uuid::Uuid) -> Result<DownloadView> {
    request_typed(
        "assignment.resource.download",
        token_payload(
            token,
            serde_json::Map::from_iter([("resource_id".to_string(), json!(resource_id))]),
        ),
        "assignments plugin",
    )
    .await
}


/// Submit (or resubmit) work for an assignment.
pub async fn submit(
    token: &str,
    assignment_id: uuid::Uuid,
    text: Option<String>,
    files: Vec<FileUploadInput>,
    links: Vec<LinkInput>,
) -> Result<SubmissionView> {
    request_typed(
        "submission.submit",
        token_payload(
            token,
            serde_json::Map::from_iter([
                ("assignment_id".to_string(), json!(assignment_id)),
                ("text".to_string(), json!(text)),
                ("files".to_string(), json!(files)),
                ("links".to_string(), json!(links)),
            ]),
        ),
        "assignments plugin",
    )
    .await
}

/// The caller's latest submission (student) or all submissions (instructor).
pub async fn submission_status(
    token: &str,
    assignment_id: uuid::Uuid,
) -> Result<serde_json::Value> {
    request_typed(
        "submission.status",
        token_payload(
            token,
            serde_json::Map::from_iter([("assignment_id".to_string(), json!(assignment_id))]),
        ),
        "assignments plugin",
    )
    .await
}

/// Download a submitted file (base64 content).
pub async fn download_submission_file(
    token: &str,
    submission_file_id: uuid::Uuid,
) -> Result<DownloadView> {
    request_typed(
        "submission.download",
        token_payload(
            token,
            serde_json::Map::from_iter([(
                "submission_file_id".to_string(),
                json!(submission_file_id),
            )]),
        ),
        "assignments plugin",
    )
    .await
}

/// Grade a submission.
pub async fn grade(
    token: &str,
    submission_id: uuid::Uuid,
    marks: f64,
    comments: &str,
    feedback: &str,
) -> Result<GradeView> {
    request_typed(
        "grade.create",
        token_payload(
            token,
            serde_json::Map::from_iter([
                ("submission_id".to_string(), json!(submission_id)),
                ("marks".to_string(), json!(marks)),
                ("comments".to_string(), json!(comments)),
                ("feedback".to_string(), json!(feedback)),
            ]),
        ),
        "assignments plugin",
    )
    .await
}

/// Grade history for a submission.
pub async fn grade_history(token: &str, submission_id: uuid::Uuid) -> Result<GradeHistoryView> {
    request_typed(
        "grade.history",
        token_payload(
            token,
            serde_json::Map::from_iter([("submission_id".to_string(), json!(submission_id))]),
        ),
        "assignments plugin",
    )
    .await
}


/// Get a submission's current grade.
pub async fn get_grade(token: &str, submission_id: uuid::Uuid) -> Result<GradeView> {
    request_typed(
        "grade.get",
        token_payload(
            token,
            serde_json::Map::from_iter([("submission_id".to_string(), json!(submission_id))]),
        ),
        "assignments plugin",
    )
    .await
}

