use uuid::Uuid;
use sqlx::PgPool;
use crate::authz::Claims;
use crate::dto::*;
use crate::models::*;
use crate::repository;
use crate::authz;

pub async fn create_topic(pool: &PgPool, claims: &Claims, req: CreateTopicRequest) -> anyhow::Result<DiscussionTopic> {
    let topic = DiscussionTopic {
        id: uuid::Uuid::new_v4(),
        course_id: req.course_id,
        module_id: req.module_id,
        lesson_id: req.lesson_id,
        user_id: claims.user_id()?,
        title: req.title,
        body: req.body,
        pinned: false,
        locked: false,
        marked_answer_id: None,
        status: "open".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::create_topic(pool, &topic).await
}

pub async fn update_topic(pool: &PgPool, claims: &Claims, req: UpdateTopicRequest) -> anyhow::Result<DiscussionTopic> {
    let topic = repository::get_topic(pool, req.id).await?.ok_or_else(|| anyhow::anyhow!("topic not found"))?;
    if topic.user_id != claims.user_id()? && !claims.is_admin() {
        anyhow::bail!("forbidden");
    }
    repository::update_topic(pool, req.id, req.title, req.body, req.pinned, req.locked, req.status).await
}

pub async fn delete_topic(pool: &PgPool, claims: &Claims, id: Uuid) -> anyhow::Result<()> {
    let topic = repository::get_topic(pool, id).await?.ok_or_else(|| anyhow::anyhow!("topic not found"))?;
    if topic.user_id != claims.user_id()? && !claims.is_admin() {
        anyhow::bail!("forbidden");
    }
    repository::delete_topic(pool, id).await
}

pub async fn create_reply(pool: &PgPool, claims: &Claims, req: CreateReplyRequest) -> anyhow::Result<DiscussionReply> {
    let topic = repository::get_topic(pool, req.topic_id).await?.ok_or_else(|| anyhow::anyhow!("topic not found"))?;
    if topic.locked {
        anyhow::bail!("topic is locked");
    }
    let reply = DiscussionReply {
        id: uuid::Uuid::new_v4(),
        topic_id: req.topic_id,
        parent_id: req.parent_id,
        user_id: claims.user_id()?,
        body: req.body,
        is_answer: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::create_reply(pool, &reply).await
}

pub async fn list_replies(pool: &PgPool, topic_id: Uuid) -> anyhow::Result<Vec<DiscussionReply>> {
    repository::list_replies(pool, topic_id).await
}

pub async fn mark_answer(pool: &PgPool, claims: &Claims, topic_id: Uuid, reply_id: Uuid) -> anyhow::Result<DiscussionTopic> {
    authz::require_instructor(claims)?;
    repository::mark_answer(pool, topic_id, reply_id).await
}

pub async fn list_topics(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<DiscussionTopic>> {
    repository::list_course_topics(pool, course_id).await
}

pub async fn create_conversation(pool: &PgPool, claims: &Claims, req: CreateConversationRequest) -> anyhow::Result<Conversation> {
    let mut participant_ids = req.participant_ids.clone();
    participant_ids.push(claims.user_id()?);
    let conversation = Conversation {
        id: uuid::Uuid::new_v4(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::create_conversation(pool, &conversation, &participant_ids).await
}

pub async fn list_conversations(pool: &PgPool, claims: &Claims) -> anyhow::Result<Vec<Conversation>> {
    repository::list_user_conversations(pool, claims.user_id()?).await
}

pub async fn send_message(pool: &PgPool, claims: &Claims, req: SendMessageRequest) -> anyhow::Result<Message> {
    let participants = repository::get_conversation_participants(pool, req.conversation_id).await?;
    if !participants.iter().any(|p| p.user_id == claims.user_id()?) {
        anyhow::bail!("not a participant");
    }
    let message = Message {
        id: uuid::Uuid::new_v4(),
        conversation_id: req.conversation_id,
        sender_id: claims.user_id()?,
        body: req.body,
        read_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::send_message(pool, &message).await
}

pub async fn list_messages(pool: &PgPool, conversation_id: Uuid) -> anyhow::Result<Vec<Message>> {
    repository::list_messages(pool, conversation_id).await
}

pub async fn mark_message_read(pool: &PgPool, message_id: Uuid) -> anyhow::Result<Message> {
    repository::mark_message_read(pool, message_id).await
}

pub async fn report_content(pool: &PgPool, claims: &Claims, req: ReportRequest) -> anyhow::Result<DiscussionReport> {
    let report = DiscussionReport {
        id: uuid::Uuid::new_v4(),
        topic_id: req.topic_id,
        reply_id: req.reply_id,
        reporter_id: claims.user_id()?,
        reason: req.reason,
        status: "open".to_string(),
        resolved_by: None,
        resolved_at: None,
        created_at: chrono::Utc::now(),
    };
    repository::create_report(pool, &report).await
}
