use std::collections::HashMap;
use uuid::Uuid;
use sqlx::PgPool;
use crate::authz;
use crate::authz::Claims;
use crate::dto::*;
use crate::models::*;
use crate::repository;

pub async fn list_categories(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<GradeCategory>> {
    repository::list_categories(pool, course_id).await
}

pub async fn create_category(pool: &PgPool, claims: &Claims, req: CreateGradeCategoryRequest) -> anyhow::Result<GradeCategory> {
    authz::require_instructor(claims)?;
    let category = GradeCategory {
        id: uuid::Uuid::new_v4(),
        course_id: req.course_id,
        name: req.name,
        weight: req.weight,
        sort_order: req.sort_order,
        created_by: claims.user_id()?,
        updated_by: claims.user_id()?,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repository::create_category(pool, &category).await
}

pub async fn update_category(pool: &PgPool, claims: &Claims, req: UpdateCategoryRequest) -> anyhow::Result<GradeCategory> {
    authz::require_instructor(claims)?;
    repository::update_category(pool, req.id, req.name, req.weight, req.sort_order, claims.user_id()?).await
}

pub async fn delete_category(pool: &PgPool, claims: &Claims, id: Uuid) -> anyhow::Result<()> {
    authz::require_instructor(claims)?;
    repository::delete_category(pool, id).await
}

pub async fn list_scales(pool: &PgPool, institution_id: Uuid) -> anyhow::Result<Vec<GradeScale>> {
    repository::list_scales(pool, institution_id).await
}

pub async fn create_scale(pool: &PgPool, claims: &Claims, req: CreateGradeScaleRequest) -> anyhow::Result<GradeScale> {
    authz::require_instructor(claims)?;
    let scale = GradeScale {
        id: uuid::Uuid::new_v4(),
        institution_id: req.institution_id,
        name: req.name,
        is_default: false,
        passing_grade: req.passing_grade,
        created_by: claims.user_id()?,
        updated_by: claims.user_id()?,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let scale = repository::create_scale(pool, &scale).await?;
    for band in req.bands {
        let b = GradeScaleBand {
            id: uuid::Uuid::new_v4(),
            scale_id: scale.id,
            letter: band.letter,
            min_pct: band.min_pct,
            max_pct: band.max_pct,
            grade_point: band.grade_point,
            sort_order: band.sort_order,
        };
        repository::create_scale_band(pool, &b).await?;
    }
    Ok(scale)
}

pub async fn create_entry(pool: &PgPool, claims: &Claims, req: CreateGradeEntryRequest) -> anyhow::Result<GradeEntry> {
    authz::require_instructor(claims)?;
    let entry = GradeEntry {
        id: uuid::Uuid::new_v4(),
        student_id: req.student_id,
        course_id: req.course_id,
        category_id: req.category_id,
        source_type: req.source_type.clone(),
        source_id: req.source_id,
        score: req.score,
        max_score: req.max_score,
        letter: None,
        grade_point: None,
        graded_by: claims.user_id()?,
        graded_at: chrono::Utc::now(),
        overridden_by: None,
        overridden_at: None,
        override_reason: None,
        comments: req.comments.unwrap_or_default(),
    };
    let pct = entry.score.map(|s| (s / entry.max_score) * 100.0);
    let (letter, grade_point) = compute_letter_and_point(pool, pct.unwrap_or(0.0), entry.course_id).await?;
    let mut updated = entry;
    updated.letter = letter;
    updated.grade_point = grade_point;
    repository::upsert_entry(pool, &updated).await
}

pub async fn list_entries(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<Vec<GradeEntry>> {
    repository::list_entries(pool, student_id, course_id).await
}

pub async fn override_entry(pool: &PgPool, claims: &Claims, req: OverrideGradeEntryRequest) -> anyhow::Result<GradeEntry> {
    authz::require_instructor(claims)?;
    let existing = repository::get_entry(pool, req.id).await?.ok_or_else(|| anyhow::anyhow!("not found"))?;
    let history = GradeHistory {
        id: uuid::Uuid::new_v4(),
        grade_entry_id: req.id,
        old_score: existing.score,
        new_score: Some(req.score),
        old_letter: existing.letter.clone(),
        new_letter: None,
        old_grade_point: existing.grade_point,
        new_grade_point: None,
        changed_by: claims.user_id()?,
        changed_at: chrono::Utc::now(),
        reason: req.reason.clone(),
    };
    let (letter, grade_point) = compute_letter_and_point(pool, req.score, existing.course_id).await?;
    let entry = repository::override_entry(pool, req.id, Some(req.score), letter, grade_point, claims.user_id()?, Some(req.reason)).await?;
    let mut history = history;
    history.new_letter = entry.letter.clone();
    history.new_grade_point = entry.grade_point;
    repository::insert_history(pool, &history).await?;
    Ok(entry)
}

pub async fn get_history(pool: &PgPool, grade_entry_id: Uuid) -> anyhow::Result<Vec<GradeHistory>> {
    repository::list_history(pool, grade_entry_id).await
}

pub async fn calculate_final_grade(pool: &PgPool, claims: &Claims, student_id: Uuid, course_id: Uuid) -> anyhow::Result<FinalGrade> {
    authz::require_instructor(claims)?;
    let entries = repository::list_entries(pool, student_id, course_id).await?;
    let categories = repository::list_categories(pool, course_id).await?;
    let mut weighted_sum: f64 = 0.0;
    let mut total_weight: f64 = 0.0;
    for cat in &categories {
        let cat_entries: Vec<_> = entries.iter().filter(|e| e.category_id == cat.id && e.score.is_some()).collect();
        if cat_entries.is_empty() {
            continue;
        }
        let avg = cat_entries.iter().map(|e| e.score.unwrap()).sum::<f64>() / cat_entries.len() as f64;
        weighted_sum += avg * (cat.weight / 100.0);
        total_weight += cat.weight;
    }
    let final_score = if total_weight > 0.0 { Some(weighted_sum) } else { None };
    let (final_letter, final_grade_point) = if let Some(score) = final_score {
        compute_letter_and_point(pool, score, course_id).await?
    } else {
        (None, None)
    };
    let grade = FinalGrade {
        student_id,
        course_id,
        final_score,
        final_letter,
        final_grade_point,
        status: "finalized".to_string(),
        calculated_at: chrono::Utc::now(),
        calculated_by: Some(claims.user_id()?),
    };
    repository::upsert_final_grade(pool, &grade).await
}

pub async fn get_final_grade(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<Option<FinalGrade>> {
    repository::get_final_grade(pool, student_id, course_id).await
}

pub async fn list_course_final_grades(pool: &PgPool, course_id: Uuid) -> anyhow::Result<Vec<FinalGrade>> {
    repository::list_course_final_grades(pool, course_id).await
}

pub async fn student_grade_view(pool: &PgPool, student_id: Uuid, course_id: Uuid) -> anyhow::Result<StudentGradeView> {
    let entries = repository::list_entries(pool, student_id, course_id).await?;
    let categories = repository::list_categories(pool, course_id).await?;
    let final_grade = repository::get_final_grade(pool, student_id, course_id).await?;
    let mut category_summaries = Vec::new();
    for cat in &categories {
        let cat_entries: Vec<_> = entries.iter().filter(|e| e.category_id == cat.id && e.score.is_some()).collect();
        let average = if cat_entries.is_empty() {
            None
        } else {
            Some(cat_entries.iter().map(|e| e.score.unwrap()).sum::<f64>() / cat_entries.len() as f64)
        };
        category_summaries.push(CategorySummary {
            category_id: cat.id,
            name: cat.name.clone(),
            weight: cat.weight,
            average,
        });
    }
    Ok(StudentGradeView {
        student_id,
        course_id,
        final_grade: final_grade.map(|fg| FinalGradeResponse {
            student_id: fg.student_id,
            course_id: fg.course_id,
            final_score: fg.final_score,
            final_letter: fg.final_letter,
            final_grade_point: fg.final_grade_point,
            status: fg.status,
            calculated_at: fg.calculated_at,
        }),
        entries: entries.into_iter().map(|e| GradeEntryResponse {
            id: e.id,
            student_id: e.student_id,
            course_id: e.course_id,
            category_id: e.category_id,
            source_type: e.source_type,
            source_id: e.source_id,
            score: e.score,
            max_score: e.max_score,
            letter: e.letter,
            grade_point: e.grade_point,
            graded_by: e.graded_by,
            graded_at: e.graded_at,
            overridden_by: e.overridden_by,
            overridden_at: e.overridden_at,
            override_reason: e.override_reason,
            comments: e.comments,
            is_override: e.overridden_by.is_some(),
        }).collect(),
        categories: category_summaries,
    })
}

async fn compute_letter_and_point(pool: &PgPool, pct: f64, course_id: Uuid) -> anyhow::Result<(Option<String>, Option<f64>)> {
    let institution_id = repository::get_course_institution_id(pool, course_id).await?;
    if let Some(institution_id) = institution_id {
        let bands = repository::get_scale_bands_for_institution(pool, institution_id).await?;
        for band in bands {
            if pct >= band.min_pct && pct <= band.max_pct {
                return Ok((Some(band.letter), Some(band.grade_point)));
            }
        }
    }
    Ok((None, None))
}
