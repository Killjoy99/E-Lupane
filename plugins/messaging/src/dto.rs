use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTopicRequest {
    pub course_id: Uuid,
    pub module_id: Option<Uuid>,
    pub lesson_id: Option<Uuid>,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTopicRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub body: Option<String>,
    pub pinned: Option<bool>,
    pub locked: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReplyRequest {
    pub topic_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkAnswerRequest {
    pub reply_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub topic_id: Option<Uuid>,
    pub reply_id: Option<Uuid>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub participant_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Uuid,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionTopicResponse {
    pub id: Uuid,
    pub course_id: Uuid,
    pub module_id: Option<Uuid>,
    pub lesson_id: Option<Uuid>,
    pub user_id: Uuid,
    pub title: String,
    pub body: String,
    pub pinned: bool,
    pub locked: bool,
    pub marked_answer_id: Option<Uuid>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionReplyResponse {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
    pub body: String,
    pub is_answer: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub body: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub participants: Vec<Uuid>,
    pub last_message: Option<MessageResponse>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseIdRequest {
    pub course_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdRequest {
    pub id: Uuid,
}
