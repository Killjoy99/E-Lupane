use sqlx::PgPool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_topic(pool: &PgPool, topic: &DiscussionTopic) -> anyhow::Result<DiscussionTopic> {
    let row = sqlx::query_as::<_, DiscussionTopic>(
        r#"INSERT INTO discussion_topics (id, course_id, module_id, lesson_id, user_id, title, body, pinned, locked, marked_answer_id, status)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id, course_id, module_id, lesson_id, user_id, title, body, pinned, locked, marked_answer_id, status, created_at, updated_at"#,
    )
    .bind(topic.id)
    .bind(topic.course_id)
    .bind(topic.module_id)
    .bind(topic.lesson_id)
    .bind(topic.user_id)
    .bind(&topic.title)
    .bind(&topic.body)
    .bind(topic.pinned)
    .bind(topic.locked)
    .bind(topic.marked_answer_id)
    .bind(&topic.status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_topic(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<DiscussionTopic>> {
    let row = sqlx::query_as::<_, DiscussionTopic>("SELECT * FROM discussion_topics WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_course_topics(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<DiscussionTopic>> {
    let rows = sqlx::query_as::<_, DiscussionTopic>("SELECT * FROM discussion_topics WHERE course_id = $1 ORDER BY pinned DESC, created_at DESC")
        .bind(course_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_topic(pool: &PgPool, id: Uuid, title: Option<String>, body: Option<String>, pinned: Option<bool>, locked: Option<bool>, status: Option<String>) -> anyhow::Result<DiscussionTopic> {
    let row = sqlx::query_as::<_, DiscussionTopic>(
        r#"UPDATE discussion_topics SET title = COALESCE($2, title), body = COALESCE($3, body), pinned = COALESCE($4, pinned), locked = COALESCE($5, locked), status = COALESCE($6, status), updated_at = now()
           WHERE id = $1
           RETURNING id, course_id, module_id, lesson_id, user_id, title, body, pinned, locked, marked_answer_id, status, created_at, updated_at"#,
    )
    .bind(id)
    .bind(title)
    .bind(body)
    .bind(pinned)
    .bind(locked)
    .bind(status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_topic(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM discussion_topics WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_reply(pool: &PgPool, reply: &DiscussionReply) -> anyhow::Result<DiscussionReply> {
    let row = sqlx::query_as::<_, DiscussionReply>(
        r#"INSERT INTO discussion_replies (id, topic_id, parent_id, user_id, body, is_answer, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING id, topic_id, parent_id, user_id, body, is_answer, created_at, updated_at"#,
    )
    .bind(reply.id)
    .bind(reply.topic_id)
    .bind(reply.parent_id)
    .bind(reply.user_id)
    .bind(&reply.body)
    .bind(reply.is_answer)
    .bind(reply.created_at)
    .bind(reply.updated_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_replies(pool: &PgPool, topic_id: Uuid) -> anyhow::Result<Vec<DiscussionReply>> {
    let rows = sqlx::query_as::<_, DiscussionReply>("SELECT * FROM discussion_replies WHERE topic_id = $1 ORDER BY created_at ASC")
        .bind(topic_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn mark_answer(pool: &PgPool, topic_id: Uuid, reply_id: Uuid) -> anyhow::Result<DiscussionTopic> {
    let row = sqlx::query_as::<_, DiscussionTopic>(
        r#"UPDATE discussion_topics SET marked_answer_id = $2, updated_at = now() WHERE id = $1
           RETURNING id, course_id, module_id, lesson_id, user_id, title, body, pinned, locked, marked_answer_id, status, created_at, updated_at"#,
    )
    .bind(topic_id)
    .bind(reply_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn create_conversation(pool: &PgPool, conversation: &Conversation, participant_ids: &[Uuid]) -> anyhow::Result<Conversation> {
    let row = sqlx::query_as::<_, Conversation>(
        r#"INSERT INTO conversations (id, created_at, updated_at) VALUES ($1, $2, $3)
           RETURNING id, created_at, updated_at"#,
    )
    .bind(conversation.id)
    .bind(conversation.created_at)
    .bind(conversation.updated_at)
    .fetch_one(pool)
    .await?;
    for pid in participant_ids {
        sqlx::query("INSERT INTO conversation_participants (id, conversation_id, user_id, last_read_at) VALUES ($1, $2, $3, now())")
            .bind(uuid::Uuid::new_v4())
            .bind(row.id)
            .bind(pid)
            .execute(pool)
            .await?;
    }
    Ok(row)
}

pub async fn list_user_conversations(pool: &PgPool, user_id: Uuid) -> anyhow::Result<Vec<Conversation>> {
    let rows = sqlx::query_as::<_, Conversation>(
        "SELECT DISTINCT c.id, c.created_at, c.updated_at FROM conversations c JOIN conversation_participants cp ON c.id = cp.conversation_id WHERE cp.user_id = $1 ORDER BY c.updated_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn send_message(pool: &PgPool, message: &Message) -> anyhow::Result<Message> {
    let row = sqlx::query_as::<_, Message>(
        r#"INSERT INTO messages (id, conversation_id, sender_id, body, read_at, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, conversation_id, sender_id, body, read_at, created_at, updated_at"#,
    )
    .bind(message.id)
    .bind(message.conversation_id)
    .bind(message.sender_id)
    .bind(&message.body)
    .bind(message.read_at)
    .bind(message.created_at)
    .bind(message.updated_at)
    .fetch_one(pool)
    .await?;
    sqlx::query("UPDATE conversations SET updated_at = now() WHERE id = $1")
        .bind(message.conversation_id)
        .execute(pool)
        .await?;
    Ok(row)
}

pub async fn list_messages(pool: &PgPool, conversation_id: Uuid) -> anyhow::Result<Vec<Message>> {
    let rows = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE conversation_id = $1 ORDER BY created_at ASC")
        .bind(conversation_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn mark_message_read(pool: &PgPool, message_id: Uuid) -> anyhow::Result<Message> {
    let row = sqlx::query_as::<_, Message>(
        r#"UPDATE messages SET read_at = now() WHERE id = $1 AND read_at IS NULL
           RETURNING id, conversation_id, sender_id, body, read_at, created_at, updated_at"#,
    )
    .bind(message_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn create_report(pool: &PgPool, report: &DiscussionReport) -> anyhow::Result<DiscussionReport> {
    let row = sqlx::query_as::<_, DiscussionReport>(
        r#"INSERT INTO discussion_reports (id, topic_id, reply_id, reporter_id, reason, status, resolved_by, resolved_at, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           RETURNING id, topic_id, reply_id, reporter_id, reason, status, resolved_by, resolved_at, created_at"#,
    )
    .bind(report.id)
    .bind(report.topic_id)
    .bind(report.reply_id)
    .bind(report.reporter_id)
    .bind(&report.reason)
    .bind(&report.status)
    .bind(report.resolved_by)
    .bind(report.resolved_at)
    .bind(report.created_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_conversation_participants(pool: &PgPool, conversation_id: Uuid) -> anyhow::Result<Vec<ConversationParticipant>> {
    let rows = sqlx::query_as::<_, ConversationParticipant>("SELECT * FROM conversation_participants WHERE conversation_id = $1")
        .bind(conversation_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}
