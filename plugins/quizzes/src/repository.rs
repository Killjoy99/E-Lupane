use sqlx::PgPool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_quiz(pool: &PgPool, quiz: &Quiz) -> anyhow::Result<Quiz> {
    let row = sqlx::query_as::<_, Quiz>(
        r#"INSERT INTO quizzes (id, course_id, title, description, instructions, time_limit_minutes, max_attempts, passing_score, randomize_questions, randomize_answers, negative_marking, negative_mark_value, show_results, allow_review, status, created_by, updated_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
           RETURNING id, course_id, title, description, instructions, time_limit_minutes, max_attempts, passing_score, randomize_questions, randomize_answers, negative_marking, negative_mark_value, show_results, allow_review, status, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(quiz.id)
    .bind(quiz.course_id)
    .bind(&quiz.title)
    .bind(&quiz.description)
    .bind(&quiz.instructions)
    .bind(quiz.time_limit_minutes)
    .bind(quiz.max_attempts)
    .bind(quiz.passing_score)
    .bind(quiz.randomize_questions)
    .bind(quiz.randomize_answers)
    .bind(quiz.negative_marking)
    .bind(quiz.negative_mark_value)
    .bind(&quiz.show_results)
    .bind(quiz.allow_review)
    .bind(&quiz.status)
    .bind(quiz.created_by)
    .bind(quiz.updated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_quiz(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Quiz>> {
    let row = sqlx::query_as::<_, Quiz>("SELECT * FROM quizzes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_course_quizzes(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<Quiz>> {
    let rows = sqlx::query_as::<_, Quiz>("SELECT * FROM quizzes WHERE course_id = $1 ORDER BY created_at DESC")
        .bind(course_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_quiz(pool: &PgPool, id: Uuid, title: Option<String>, description: Option<String>, instructions: Option<String>, time_limit_minutes: Option<i32>, max_attempts: Option<i32>, passing_score: Option<f64>, randomize_questions: Option<bool>, randomize_answers: Option<bool>, negative_marking: Option<bool>, negative_mark_value: Option<f64>, show_results: Option<String>, allow_review: Option<bool>, status: Option<String>, updated_by: Uuid) -> anyhow::Result<Quiz> {
    let row = sqlx::query_as::<_, Quiz>(
        r#"UPDATE quizzes SET title = COALESCE($2, title), description = COALESCE($3, description), instructions = COALESCE($4, instructions), time_limit_minutes = COALESCE($5, time_limit_minutes), max_attempts = COALESCE($6, max_attempts), passing_score = COALESCE($7, passing_score), randomize_questions = COALESCE($8, randomize_questions), randomize_answers = COALESCE($9, randomize_answers), negative_marking = COALESCE($10, negative_marking), negative_mark_value = COALESCE($11, negative_mark_value), show_results = COALESCE($12, show_results), allow_review = COALESCE($13, allow_review), status = COALESCE($14, status), updated_by = $15, updated_at = now()
           WHERE id = $1
           RETURNING id, course_id, title, description, instructions, time_limit_minutes, max_attempts, passing_score, randomize_questions, randomize_answers, negative_marking, negative_mark_value, show_results, allow_review, status, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(instructions)
    .bind(time_limit_minutes)
    .bind(max_attempts)
    .bind(passing_score)
    .bind(randomize_questions)
    .bind(randomize_answers)
    .bind(negative_marking)
    .bind(negative_mark_value)
    .bind(show_results)
    .bind(allow_review)
    .bind(status)
    .bind(updated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_quiz(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM quizzes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_question(pool: &PgPool, question: &QuizQuestion) -> anyhow::Result<QuizQuestion> {
    let row = sqlx::query_as::<_, QuizQuestion>(
        r#"INSERT INTO quiz_questions (id, quiz_id, question_type, title, body, points, sort_order, correct_answer, metadata, created_by, updated_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id, quiz_id, question_type, title, body, points, sort_order, correct_answer, metadata, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(question.id)
    .bind(question.quiz_id)
    .bind(&question.question_type)
    .bind(&question.title)
    .bind(&question.body)
    .bind(question.points)
    .bind(question.sort_order)
    .bind(&question.correct_answer)
    .bind(&question.metadata)
    .bind(question.created_by)
    .bind(question.updated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_questions(pool: &PgPool, quiz_id: Uuid) -> anyhow::Result<Vec<QuizQuestion>> {
    let rows = sqlx::query_as::<_, QuizQuestion>("SELECT * FROM quiz_questions WHERE quiz_id = $1 ORDER BY sort_order")
        .bind(quiz_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_question(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<QuizQuestion>> {
    let row = sqlx::query_as::<_, QuizQuestion>("SELECT * FROM quiz_questions WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn create_question_option(pool: &PgPool, option: &QuizQuestionOption) -> anyhow::Result<QuizQuestionOption> {
    let row = sqlx::query_as::<_, QuizQuestionOption>(
        r#"INSERT INTO quiz_question_options (id, question_id, label, value, is_correct, sort_order)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, question_id, label, value, is_correct, sort_order"#,
    )
    .bind(option.id)
    .bind(option.question_id)
    .bind(&option.label)
    .bind(&option.value)
    .bind(option.is_correct)
    .bind(option.sort_order)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_question_options(pool: &PgPool, question_id: Uuid) -> anyhow::Result<Vec<QuizQuestionOption>> {
    let rows = sqlx::query_as::<_, QuizQuestionOption>("SELECT * FROM quiz_question_options WHERE question_id = $1 ORDER BY sort_order")
        .bind(question_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn create_attempt(pool: &PgPool, attempt: &QuizAttempt) -> anyhow::Result<QuizAttempt> {
    let row = sqlx::query_as::<_, QuizAttempt>(
        r#"INSERT INTO quiz_attempts (id, quiz_id, student_id, attempt_number, started_at, time_expired_at, score, max_score, passed, graded, status)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id, quiz_id, student_id, attempt_number, started_at, submitted_at, time_expired_at, score, max_score, passed, graded, status"#,
    )
    .bind(attempt.id)
    .bind(attempt.quiz_id)
    .bind(attempt.student_id)
    .bind(attempt.attempt_number)
    .bind(attempt.started_at)
    .bind(attempt.time_expired_at)
    .bind(attempt.score)
    .bind(attempt.max_score)
    .bind(attempt.passed)
    .bind(attempt.graded)
    .bind(&attempt.status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_attempt(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<QuizAttempt>> {
    let row = sqlx::query_as::<_, QuizAttempt>("SELECT * FROM quiz_attempts WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_student_attempts(pool: &PgPool, quiz_id: Uuid, student_id: Uuid) -> anyhow::Result<Vec<QuizAttempt>> {
    let rows = sqlx::query_as::<_, QuizAttempt>("SELECT * FROM quiz_attempts WHERE quiz_id = $1 AND student_id = $2 ORDER BY started_at DESC")
        .bind(quiz_id)
        .bind(student_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn count_student_attempts(pool: &PgPool, quiz_id: Uuid, student_id: Uuid) -> anyhow::Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM quiz_attempts WHERE quiz_id = $1 AND student_id = $2")
        .bind(quiz_id)
        .bind(student_id)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn submit_attempt(pool: &PgPool, id: Uuid, submitted_at: chrono::DateTime<chrono::Utc>, status: &str) -> anyhow::Result<QuizAttempt> {
    let row = sqlx::query_as::<_, QuizAttempt>(
        r#"UPDATE quiz_attempts SET submitted_at = $2, status = $3 WHERE id = $1
           RETURNING id, quiz_id, student_id, attempt_number, started_at, submitted_at, time_expired_at, score, max_score, passed, graded, status"#,
    )
    .bind(id)
    .bind(submitted_at)
    .bind(status)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn grade_attempt(pool: &PgPool, id: Uuid, score: f64, max_score: f64, passed: bool) -> anyhow::Result<QuizAttempt> {
    let row = sqlx::query_as::<_, QuizAttempt>(
        r#"UPDATE quiz_attempts SET score = $2, max_score = $3, passed = $4, graded = true, status = 'graded' WHERE id = $1
           RETURNING id, quiz_id, student_id, attempt_number, started_at, submitted_at, time_expired_at, score, max_score, passed, graded, status"#,
    )
    .bind(id)
    .bind(score)
    .bind(max_score)
    .bind(passed)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn create_answer(pool: &PgPool, answer: &AttemptAnswer) -> anyhow::Result<AttemptAnswer> {
    let row = sqlx::query_as::<_, AttemptAnswer>(
        r#"INSERT INTO attempt_answers (id, attempt_id, question_id, answer, is_correct, points_awarded, graded_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, attempt_id, question_id, answer, is_correct, points_awarded, graded_at, created_at, updated_at"#,
    )
    .bind(answer.id)
    .bind(answer.attempt_id)
    .bind(answer.question_id)
    .bind(&answer.answer)
    .bind(answer.is_correct)
    .bind(answer.points_awarded)
    .bind(answer.graded_at)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_answers(pool: &PgPool, attempt_id: Uuid) -> anyhow::Result<Vec<AttemptAnswer>> {
    let rows = sqlx::query_as::<_, AttemptAnswer>("SELECT * FROM attempt_answers WHERE attempt_id = $1 ORDER BY created_at")
        .bind(attempt_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_answer(pool: &PgPool, attempt_id: Uuid, question_id: Uuid) -> anyhow::Result<Option<AttemptAnswer>> {
    let row = sqlx::query_as::<_, AttemptAnswer>("SELECT * FROM attempt_answers WHERE attempt_id = $1 AND question_id = $2")
        .bind(attempt_id)
        .bind(question_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn grade_answer(pool: &PgPool, id: Uuid, is_correct: Option<bool>, points_awarded: Option<f64>) -> anyhow::Result<AttemptAnswer> {
    let row = sqlx::query_as::<_, AttemptAnswer>(
        r#"UPDATE attempt_answers SET is_correct = COALESCE($2, is_correct), points_awarded = COALESCE($3, points_awarded), graded_at = now()
           WHERE id = $1
           RETURNING id, attempt_id, question_id, answer, is_correct, points_awarded, graded_at, created_at, updated_at"#,
    )
    .bind(id)
    .bind(is_correct)
    .bind(points_awarded)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn create_assignment(pool: &PgPool, assignment: &QuizAssignment) -> anyhow::Result<QuizAssignment> {
    let row = sqlx::query_as::<_, QuizAssignment>(
        r#"INSERT INTO quiz_assignments (id, quiz_id, course_id, available_from, available_until)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, quiz_id, course_id, available_from, available_until"#,
    )
    .bind(assignment.id)
    .bind(assignment.quiz_id)
    .bind(assignment.course_id)
    .bind(assignment.available_from)
    .bind(assignment.available_until)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_assignments(pool: &PgPool, quiz_id: Uuid) -> anyhow::Result<Vec<QuizAssignment>> {
    let rows = sqlx::query_as::<_, QuizAssignment>("SELECT * FROM quiz_assignments WHERE quiz_id = $1")
        .bind(quiz_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_course_institution_id(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query_scalar::<_, Uuid>("SELECT institution_id FROM courses WHERE id = $1")
        .bind(course_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
