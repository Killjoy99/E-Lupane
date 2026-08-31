//! Database row models.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Faculty {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub name: String,
    pub code: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Department {
    pub id: Uuid,
    pub faculty_id: Uuid,
    pub name: String,
    pub code: String,
    pub description: String,
    pub head_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Program {
    pub id: Uuid,
    pub department_id: Uuid,
    pub name: String,
    pub code: String,
    pub degree_type: String,
    pub duration_years: i32,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct AcademicYear {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Semester {
    pub id: Uuid,
    pub academic_year_id: Uuid,
    pub name: String,
    pub code: String,
    pub session_number: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Class {
    pub id: Uuid,
    pub program_id: Uuid,
    pub academic_year_id: Uuid,
    pub name: String,
    pub code: String,
    pub intake_date: Option<NaiveDate>,
    pub capacity: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct Offering {
    pub id: Uuid,
    pub course_id: Uuid,
    pub semester_id: Uuid,
    pub program_id: Uuid,
    pub instructor_id: Uuid,
    pub section: String,
    pub max_enrolment: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
