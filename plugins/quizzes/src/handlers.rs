use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::*;
use crate::repository;
use crate::services;
use crate::models::*;

pub const PLUGIN_ID: &str = "com.elupane.quizzes";

pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "quizzes.error", &err.to_string()),
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
        "quizzes.list" => {
            let _claims = authorize(&command.payload)?;
            let req: CourseIdRequest = parse_payload(&command.payload)?;
            let view = services::list_course_quizzes(pool, req.course_id).await?;
            reply(command, &view)
        }
        "quizzes.get" => {
            let _claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            let view = services::get_quiz(pool, req.id).await?;
            reply(command, &view)
        }
        "quizzes.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateQuizRequest = parse_payload(&command.payload)?;
            let view = services::create_quiz(pool, &claims, req).await?;
            reply(command, &view)
        }
        "quizzes.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateQuizRequest = parse_payload(&command.payload)?;
            let view = services::update_quiz(pool, &claims, req).await?;
            reply(command, &view)
        }
        "quizzes.delete" => {
            let claims = authorize(&command.payload)?;
            let req: IdRequest = parse_payload(&command.payload)?;
            services::delete_quiz(pool, &claims, req.id).await?;
            Ok(Message::reply_to(command, PLUGIN_ID, serde_json::json!({ "deleted": true, "quiz_id": req.id })))
        }

        "quizzes.questions.list" => {
            let _claims = authorize(&command.payload)?;
            let req: QuizIdRequest = parse_payload(&command.payload)?;
            let view = services::list_questions(pool, req.quiz_id).await?;
            reply(command, &view)
        }
        "quizzes.questions.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateQuestionRequest = parse_payload(&command.payload)?;
            let view = services::create_question(pool, &claims, req).await?;
            reply(command, &view)
        }

        "quizzes.attempts.start" => {
            let claims = authorize(&command.payload)?;
            let req: StartAttemptRequest = parse_payload(&command.payload)?;
            let view = services::start_attempt(pool, &claims, req).await?;
            reply(command, &view)
        }
        "quizzes.attempts.submit_answer" => {
            let claims = authorize(&command.payload)?;
            let req: SubmitAnswerRequest = parse_payload(&command.payload)?;
            let view = services::submit_answer(pool, &claims, req).await?;
            reply(command, &view)
        }
        "quizzes.attempts.submit" => {
            let claims = authorize(&command.payload)?;
            let req: SubmitAttemptRequest = parse_payload(&command.payload)?;
            let view = services::submit_attempt(pool, &claims, req).await?;
            reply(command, &view)
        }
        "quizzes.attempts.grade" => {
            let claims = authorize(&command.payload)?;
            let req: AttemptIdRequest = parse_payload(&command.payload)?;
            let view = services::grade_attempt(pool, &claims, req.attempt_id).await?;
            reply(command, &view)
        }
        "quizzes.attempts.get" => {
            let _claims = authorize(&command.payload)?;
            let req: AttemptIdRequest = parse_payload(&command.payload)?;
            let view = services::get_attempt(pool, req.attempt_id).await?;
            reply(command, &view)
        }
        "quizzes.attempts.answers" => {
            let _claims = authorize(&command.payload)?;
            let req: AttemptIdRequest = parse_payload(&command.payload)?;
            let view = services::list_answers(pool, req.attempt_id).await?;
            reply(command, &view)
        }

        "quizzes.student.view" => {
            let claims = authorize(&command.payload)?;
            let req: QuizIdRequest = parse_payload(&command.payload)?;
            let (quiz, questions) = services::get_quiz_with_questions(pool, req.quiz_id).await?;
            let student_id = claims.user_id()?;
            let attempts = repository::list_student_attempts(pool, req.quiz_id, student_id).await?;
            let attempt = attempts.into_iter().next();
            let attempt_response = attempt.map(|a| QuizAttemptResponse {
                id: a.id,
                quiz_id: a.quiz_id,
                student_id: a.student_id,
                attempt_number: a.attempt_number,
                started_at: a.started_at,
                submitted_at: a.submitted_at,
                time_expired_at: a.time_expired_at,
                score: a.score,
                max_score: a.max_score,
                passed: a.passed,
                graded: a.graded,
                status: a.status,
                answers: Vec::new(),
            });
            reply(command, &(quiz, questions, attempt_response))
        }

        "events.subscribe" => Ok(Message::reply_to(
            command,
            PLUGIN_ID,
            serde_json::json!({ "subscribed": true }),
        )),

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}
