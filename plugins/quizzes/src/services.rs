use std::collections::HashMap;
use uuid::Uuid;
use sqlx::PgPool;
use crate::authz::Claims;
use crate::dto::*;
use crate::models::*;
use crate::repository;
use crate::authz;

pub async fn create_quiz(pool: &PgPool, claims: &Claims, req: CreateQuizRequest) -> anyhow::Result<Quiz> {
    authz::require_instructor(claims)?;
    let quiz = Quiz {
        id: uuid::Uuid::new_v4(),
        course_id: req.course_id,
        title: req.title,
        description: req.description.unwrap_or_default(),
        instructions: req.instructions.unwrap_or_default(),
        time_limit_minutes: req.time_limit_minutes,
        max_attempts: req.max_attempts,
        passing_score: req.passing_score,
        randomize_questions: req.randomize_questions,
        randomize_answers: req.randomize_answers,
        negative_marking: req.negative_marking,
        negative_mark_value: req.negative_mark_value,
        show_results: req.show_results,
        allow_review: req.allow_review,
        status: req.status.unwrap_or_else(|| "draft".to_string()),
        created_by: claims.user_id()?,
        updated_by: claims.user_id()?,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::create_quiz(pool, &quiz).await
}

pub async fn get_quiz(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Quiz>> {
    repository::get_quiz(pool, id).await
}

pub async fn list_course_quizzes(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<Quiz>> {
    repository::list_course_quizzes(pool, course_id).await
}

pub async fn update_quiz(pool: &PgPool, claims: &Claims, req: UpdateQuizRequest) -> anyhow::Result<Quiz> {
    authz::require_instructor(claims)?;
    repository::update_quiz(
        pool,
        req.id,
        req.title,
        req.description,
        req.instructions,
        req.time_limit_minutes,
        req.max_attempts,
        req.passing_score,
        req.randomize_questions,
        req.randomize_answers,
        req.negative_marking,
        req.negative_mark_value,
        req.show_results,
        req.allow_review,
        req.status,
        claims.user_id()?,
    ).await
}

pub async fn delete_quiz(pool: &PgPool, claims: &Claims, id: Uuid) -> anyhow::Result<()> {
    authz::require_instructor(claims)?;
    repository::delete_quiz(pool, id).await
}

pub async fn create_question(pool: &PgPool, claims: &Claims, req: CreateQuestionRequest) -> anyhow::Result<QuizQuestion> {
    authz::require_instructor(claims)?;
    let question = QuizQuestion {
        id: uuid::Uuid::new_v4(),
        quiz_id: req.quiz_id,
        question_type: req.question_type,
        title: req.title,
        body: req.body.unwrap_or_default(),
        points: req.points,
        sort_order: req.sort_order.unwrap_or(0),
        correct_answer: req.correct_answer,
        metadata: req.metadata.unwrap_or_else(|| serde_json::json!({})),
        created_by: claims.user_id()?,
        updated_by: claims.user_id()?,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let question = repository::create_question(pool, &question).await?;
    for opt in req.options {
        let option = QuizQuestionOption {
            id: uuid::Uuid::new_v4(),
            question_id: question.id,
            label: opt.label.unwrap_or_default(),
            value: opt.value,
            is_correct: opt.is_correct,
            sort_order: opt.sort_order.unwrap_or(0),
        };
        repository::create_question_option(pool, &option).await?;
    }
    Ok(question)
}

pub async fn list_questions(pool: &PgPool, quiz_id: Uuid) -> anyhow::Result<Vec<QuizQuestion>> {
    repository::list_questions(pool, quiz_id).await
}

pub async fn start_attempt(pool: &PgPool, claims: &Claims, req: StartAttemptRequest) -> anyhow::Result<QuizAttempt> {
    let quiz = repository::get_quiz(pool, req.quiz_id).await?.ok_or_else(|| anyhow::anyhow!("quiz not found"))?;
    if quiz.status != "published" {
        anyhow::bail!("quiz is not published");
    }
    let student_id = claims.user_id()?;
    let existing_attempts = repository::count_student_attempts(pool, req.quiz_id, student_id).await?;
    if existing_attempts >= quiz.max_attempts as i64 {
        anyhow::bail!("max attempts reached");
    }
    let attempt_number = (existing_attempts + 1) as i32;
    let time_expired_at = quiz.time_limit_minutes.map(|mins| chrono::Utc::now() + chrono::Duration::minutes(mins as i64));
    let attempt = QuizAttempt {
        id: uuid::Uuid::new_v4(),
        quiz_id: req.quiz_id,
        student_id,
        attempt_number,
        started_at: chrono::Utc::now(),
        submitted_at: None,
        time_expired_at,
        score: None,
        max_score: None,
        passed: None,
        graded: false,
        status: "in_progress".to_string(),
    };
    repository::create_attempt(pool, &attempt).await
}

pub async fn submit_answer(pool: &PgPool, claims: &Claims, req: SubmitAnswerRequest) -> anyhow::Result<AttemptAnswer> {
    let attempt = repository::get_attempt(pool, req.attempt_id).await?.ok_or_else(|| anyhow::anyhow!("attempt not found"))?;
    if attempt.status != "in_progress" {
        anyhow::bail!("attempt is not in progress");
    }
    if attempt.student_id != claims.user_id()? {
        anyhow::bail!("not your attempt");
    }
    if let Some(expired_at) = attempt.time_expired_at {
        if chrono::Utc::now() > expired_at {
            anyhow::bail!("attempt timed out");
        }
    }
    let answer = AttemptAnswer {
        id: uuid::Uuid::new_v4(),
        attempt_id: req.attempt_id,
        question_id: req.question_id,
        answer: req.answer,
        is_correct: None,
        points_awarded: None,
        graded_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::create_answer(pool, &answer).await
}

pub async fn submit_attempt(pool: &PgPool, claims: &Claims, req: SubmitAttemptRequest) -> anyhow::Result<QuizAttempt> {
    let attempt = repository::get_attempt(pool, req.attempt_id).await?.ok_or_else(|| anyhow::anyhow!("attempt not found"))?;
    if attempt.status != "in_progress" {
        anyhow::bail!("attempt is not in progress");
    }
    if attempt.student_id != claims.user_id()? {
        anyhow::bail!("not your attempt");
    }
    if let Some(expired_at) = attempt.time_expired_at {
        if chrono::Utc::now() > expired_at {
            anyhow::bail!("attempt timed out");
        }
    }
    let submitted_at = chrono::Utc::now();
    let status = if submitted_at > attempt.time_expired_at.unwrap_or(submitted_at) {
        "timed_out"
    } else {
        "submitted"
    };
    repository::submit_attempt(pool, req.attempt_id, submitted_at, status).await
}

pub async fn grade_attempt(pool: &PgPool, claims: &Claims, attempt_id: Uuid) -> anyhow::Result<QuizAttempt> {
    authz::require_instructor(claims)?;
    let attempt = repository::get_attempt(pool, attempt_id).await?.ok_or_else(|| anyhow::anyhow!("attempt not found"))?;
    let questions = repository::list_questions(pool, attempt.quiz_id).await?;
    let answers = repository::list_answers(pool, attempt_id).await?;
    let mut total_score: f64 = 0.0;
    let mut total_max: f64 = 0.0;
    for question in &questions {
        total_max += question.points;
        let answer = answers.iter().find(|a| a.question_id == question.id);
        if let Some(answer) = answer {
            let (is_correct, points) = evaluate_answer(question, &answer.answer);
            total_score += points;
            repository::grade_answer(pool, answer.id, Some(is_correct), Some(points)).await?;
        }
    }
    let passed = total_score >= attempt.max_score.unwrap_or(total_max) * (attempt.max_score.unwrap_or(total_max) / 100.0);
    repository::grade_attempt(pool, attempt_id, total_score, total_max, passed).await
}

pub async fn get_attempt(pool: &PgPool, attempt_id: Uuid) -> anyhow::Result<Option<QuizAttempt>> {
    repository::get_attempt(pool, attempt_id).await
}

pub async fn list_answers(pool: &PgPool, attempt_id: Uuid) -> anyhow::Result<Vec<AttemptAnswer>> {
    repository::list_answers(pool, attempt_id).await
}

pub async fn get_quiz_with_questions(pool: &PgPool, quiz_id: Uuid) -> anyhow::Result<(Quiz, Vec<QuizQuestion>)> {
    let quiz = repository::get_quiz(pool, quiz_id).await?.ok_or_else(|| anyhow::anyhow!("quiz not found"))?;
    let questions = repository::list_questions(pool, quiz_id).await?;
    Ok((quiz, questions))
}

fn evaluate_answer(question: &QuizQuestion, answer: &serde_json::Value) -> (bool, f64) {
    match question.question_type.as_str() {
        "multiple_choice" => {
            if let Some(selected) = answer.as_str() {
                if let Some(correct) = question.correct_answer.as_str() {
                    return (selected == correct, if selected == correct { question.points } else { 0.0 });
                }
            }
            (false, 0.0)
        }
        "true_false" => {
            if let Some(selected) = answer.as_str() {
                if let Some(correct) = question.correct_answer.as_str() {
                    return (selected == correct, if selected == correct { question.points } else { 0.0 });
                }
            }
            (false, 0.0)
        }
        "short_answer" | "long_answer" => {
            if let Some(text) = answer.as_str() {
                if let Some(correct) = question.correct_answer.as_str() {
                    let similarity = levenshtein_similarity(text, correct);
                    return (similarity > 0.8, if similarity > 0.8 { question.points } else { 0.0 });
                }
            }
            (false, 0.0)
        }
        _ => (false, 0.0),
    }
}

fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();
    if a_lower == b_lower {
        return 1.0;
    }
    let distance = levenshtein_distance(&a_lower, &b_lower);
    let max_len = a_lower.len().max(b_lower.len());
    if max_len == 0 {
        1.0
    } else {
        1.0 - (distance as f64 / max_len as f64)
    }
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];
    for i in 0..=a_chars.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        matrix[0][j] = j;
    }
    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            if a_chars[i - 1] == b_chars[j - 1] {
                matrix[i][j] = matrix[i - 1][j - 1];
            } else {
                matrix[i][j] = 1 + matrix[i - 1][j].min(matrix[i][j - 1]).min(matrix[i - 1][j - 1]);
            }
        }
    }
    matrix[a_chars.len()][b_chars.len()]
}
