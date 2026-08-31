use sqlx::PgPool;
use uuid::Uuid;
use crate::models::*;

pub async fn create_category(pool: &PgPool, category: &GradeCategory) -> anyhow::Result<GradeCategory> {
    let row = sqlx::query_as::<_, GradeCategory>(
        r#"INSERT INTO grade_categories (id, course_id, name, weight, sort_order, created_by, updated_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, course_id, name, weight, sort_order, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(category.id)
    .bind(category.course_id)
    .bind(&category.name)
    .bind(category.weight)
    .bind(category.sort_order)
    .bind(category.created_by)
    .bind(category.updated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_categories(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<GradeCategory>> {
    let rows = sqlx::query_as::<_, GradeCategory>(
        "SELECT id, course_id, name, weight, sort_order, created_by, updated_by, created_at, updated_at FROM grade_categories WHERE course_id = $1 ORDER BY sort_order",
    )
    .bind(course_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn update_category(pool: &PgPool, id: Uuid, name: Option<String>, weight: Option<f64>, sort_order: Option<i32>, updated_by: Uuid) -> anyhow::Result<GradeCategory> {
    let row = sqlx::query_as::<_, GradeCategory>(
        r#"UPDATE grade_categories SET name = COALESCE($2, name), weight = COALESCE($3, weight), sort_order = COALESCE($4, sort_order), updated_by = $5, updated_at = now()
           WHERE id = $1
           RETURNING id, course_id, name, weight, sort_order, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(id)
    .bind(name)
    .bind(weight)
    .bind(sort_order)
    .bind(updated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn delete_category(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM grade_categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_scales(pool: &PgPool, institution_id: Uuid) -> anyhow::Result<Vec<GradeScale>> {
    let rows = sqlx::query_as::<_, GradeScale>("SELECT * FROM grade_scales WHERE institution_id = $1")
        .bind(institution_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_scale(pool: &PgPool, institution_id: Uuid) -> anyhow::Result<Option<GradeScale>> {
    let row = sqlx::query_as::<_, GradeScale>("SELECT * FROM grade_scales WHERE institution_id = $1 LIMIT 1")
        .bind(institution_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn create_scale(pool: &PgPool, scale: &GradeScale) -> anyhow::Result<GradeScale> {
    let row = sqlx::query_as::<_, GradeScale>(
        r#"INSERT INTO grade_scales (id, institution_id, name, is_default, passing_grade, created_by, updated_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, institution_id, name, is_default, passing_grade, created_by, updated_by, created_at, updated_at"#,
    )
    .bind(scale.id)
    .bind(scale.institution_id)
    .bind(&scale.name)
    .bind(scale.is_default)
    .bind(scale.passing_grade)
    .bind(scale.created_by)
    .bind(scale.updated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn create_scale_band(pool: &PgPool, band: &GradeScaleBand) -> anyhow::Result<GradeScaleBand> {
    let row = sqlx::query_as::<_, GradeScaleBand>(
        r#"INSERT INTO grade_scale_bands (id, scale_id, letter, min_pct, max_pct, grade_point, sort_order)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id, scale_id, letter, min_pct, max_pct, grade_point, sort_order"#,
    )
    .bind(band.id)
    .bind(band.scale_id)
    .bind(&band.letter)
    .bind(band.min_pct)
    .bind(band.max_pct)
    .bind(band.grade_point)
    .bind(band.sort_order)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn get_scale_bands(pool: &PgPool, scale_id: Uuid) -> anyhow::Result<Vec<GradeScaleBand>> {
    let rows = sqlx::query_as::<_, GradeScaleBand>("SELECT * FROM grade_scale_bands WHERE scale_id = $1 ORDER BY sort_order")
        .bind(scale_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_scale_bands_for_institution(pool: &PgPool, institution_id: Uuid) -> anyhow::Result<Vec<GradeScaleBand>> {
    let scale = get_scale(pool, institution_id).await?;
    match scale {
        Some(s) => get_scale_bands(pool, s.id).await,
        None => Ok(Vec::new()),
    }
}

pub async fn get_entry(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<GradeEntry>> {
    let row = sqlx::query_as::<_, GradeEntry>("SELECT * FROM grade_entries WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn list_entries(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<Vec<GradeEntry>> {
    let rows = sqlx::query_as::<_, GradeEntry>(
        "SELECT * FROM grade_entries WHERE student_id = $1 AND course_id = $2 ORDER BY graded_at DESC",
    )
    .bind(student_id)
    .bind(course_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn upsert_entry(pool: &PgPool, entry: &GradeEntry) -> anyhow::Result<GradeEntry> {
    let row = sqlx::query_as::<_, GradeEntry>(
        r#"INSERT INTO grade_entries (id, student_id, course_id, category_id, source_type, source_id, score, max_score, letter, grade_point, graded_by, graded_at, overridden_by, overridden_at, override_reason, comments)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
           ON CONFLICT (source_type, source_id) WHERE source_type NOT IN ('manual') DO UPDATE SET
             score = EXCLUDED.score,
             letter = EXCLUDED.letter,
             grade_point = EXCLUDED.grade_point,
             graded_by = EXCLUDED.graded_by,
             graded_at = EXCLUDED.graded_at,
             comments = EXCLUDED.comments
           RETURNING id, student_id, course_id, category_id, source_type, source_id, score, max_score, letter, grade_point, graded_by, graded_at, overridden_by, overridden_at, override_reason, comments"#,
    )
    .bind(entry.id)
    .bind(entry.student_id)
    .bind(entry.course_id)
    .bind(entry.category_id)
    .bind(&entry.source_type)
    .bind(entry.source_id)
    .bind(entry.score)
    .bind(entry.max_score)
    .bind(&entry.letter)
    .bind(entry.grade_point)
    .bind(entry.graded_by)
    .bind(entry.graded_at)
    .bind(entry.overridden_by)
    .bind(entry.overridden_at)
    .bind(&entry.override_reason)
    .bind(&entry.comments)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn override_entry(pool: &PgPool, id: Uuid, score: Option<f64>, letter: Option<String>, grade_point: Option<f64>, overridden_by: Uuid, reason: Option<String>) -> anyhow::Result<GradeEntry> {
    let row = sqlx::query_as::<_, GradeEntry>(
        r#"UPDATE grade_entries SET score = COALESCE($2, score), letter = COALESCE($3, letter), grade_point = COALESCE($4, grade_point), overridden_by = $5, overridden_at = now(), override_reason = COALESCE($6, override_reason)
           WHERE id = $1
           RETURNING id, student_id, course_id, category_id, source_type, source_id, score, max_score, letter, grade_point, graded_by, graded_at, overridden_by, overridden_at, override_reason, comments"#,
    )
    .bind(id)
    .bind(score)
    .bind(letter)
    .bind(grade_point)
    .bind(overridden_by)
    .bind(reason)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn insert_history(pool: &PgPool, history: &GradeHistory) -> anyhow::Result<GradeHistory> {
    let row = sqlx::query_as::<_, GradeHistory>(
        r#"INSERT INTO grade_history (id, grade_entry_id, old_score, new_score, old_letter, new_letter, old_grade_point, new_grade_point, changed_by, changed_at, reason)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING id, grade_entry_id, old_score, new_score, old_letter, new_letter, old_grade_point, new_grade_point, changed_by, changed_at, reason"#,
    )
    .bind(history.id)
    .bind(history.grade_entry_id)
    .bind(history.old_score)
    .bind(history.new_score)
    .bind(&history.old_letter)
    .bind(&history.new_letter)
    .bind(history.old_grade_point)
    .bind(history.new_grade_point)
    .bind(history.changed_by)
    .bind(history.changed_at)
    .bind(&history.reason)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_history(pool: &PgPool, grade_entry_id: Uuid) -> anyhow::Result<Vec<GradeHistory>> {
    let rows = sqlx::query_as::<_, GradeHistory>("SELECT * FROM grade_history WHERE grade_entry_id = $1 ORDER BY changed_at DESC")
        .bind(grade_entry_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn get_final_grade(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<Option<FinalGrade>> {
    let row = sqlx::query_as::<_, FinalGrade>("SELECT * FROM final_grades WHERE student_id = $1 AND course_id = $2")
        .bind(student_id)
        .bind(course_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn upsert_final_grade(pool: &PgPool, grade: &FinalGrade) -> anyhow::Result<FinalGrade> {
    let row = sqlx::query_as::<_, FinalGrade>(
        r#"INSERT INTO final_grades (student_id, course_id, final_score, final_letter, final_grade_point, status, calculated_at, calculated_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           ON CONFLICT (student_id, course_id) DO UPDATE SET
             final_score = EXCLUDED.final_score,
             final_letter = EXCLUDED.final_letter,
             final_grade_point = EXCLUDED.final_grade_point,
             status = EXCLUDED.status,
             calculated_at = EXCLUDED.calculated_at,
             calculated_by = EXCLUDED.calculated_by
           RETURNING student_id, course_id, final_score, final_letter, final_grade_point, status, calculated_at, calculated_by"#,
    )
    .bind(grade.student_id)
    .bind(grade.course_id)
    .bind(grade.final_score)
    .bind(&grade.final_letter)
    .bind(grade.final_grade_point)
    .bind(&grade.status)
    .bind(grade.calculated_at)
    .bind(grade.calculated_by)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn list_course_final_grades(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<FinalGrade>> {
    let rows = sqlx::query_as::<_, FinalGrade>("SELECT * FROM final_grades WHERE course_id = $1")
        .bind(course_id)
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
