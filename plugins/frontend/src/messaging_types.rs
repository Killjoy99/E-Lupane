use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscussionTopicView {
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscussionReplyView {
    pub id: Uuid,
    pub topic_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
    pub body: String,
    pub is_answer: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateTopicInput {
    pub course_id: Uuid,
    pub module_id: Option<Uuid>,
    pub lesson_id: Option<Uuid>,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateReplyInput {
    pub topic_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub body: String,
}
