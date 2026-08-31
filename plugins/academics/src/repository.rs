//! SQLx data access for the academic structure. All filters are bound
//! parameters; column lists are compile-time constants wrapped in
//! `AssertSqlSafe`.

use anyhow::Result;
use chrono::NaiveDate;
use sqlx::{AssertSqlSafe, PgPool};
use uuid::Uuid;

use crate::models::{
    AcademicYear, Class, Department, Faculty, Offering, Program, Semester,
};

pub const FACULTY_COLUMNS: &str =
    "id, institution_id, name, code, description, created_at, updated_at";
pub const DEPARTMENT_COLUMNS: &str =
    "id, faculty_id, name, code, description, head_id, created_at, updated_at";
pub const PROGRAM_COLUMNS: &str =
    "id, department_id, name, code, degree_type, duration_years, description, created_at, updated_at";
pub const YEAR_COLUMNS: &str =
    "id, name, code, start_date, end_date, status, created_at, updated_at";
pub const SEMESTER_COLUMNS: &str = "id, academic_year_id, name, code, session_number, \
    start_date, end_date, status, created_at, updated_at";
pub const CLASS_COLUMNS: &str = "id, program_id, academic_year_id, name, code, intake_date, \
    capacity, created_at, updated_at";
pub const OFFERING_COLUMNS: &str = "id, course_id, semester_id, program_id, instructor_id, \
    section, max_enrolment, status, created_at, updated_at";

// ---------------------------------------------------------------------------
// Faculties
// ---------------------------------------------------------------------------

pub async fn find_faculty_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Faculty>> {
    Ok(sqlx::query_as::<_, Faculty>(AssertSqlSafe(format!(
        "SELECT {FACULTY_COLUMNS} FROM faculties WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_faculty_by_code(pool: &PgPool, code: &str) -> Result<Option<Faculty>> {
    Ok(sqlx::query_as::<_, Faculty>(AssertSqlSafe(format!(
        "SELECT {FACULTY_COLUMNS} FROM faculties WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_faculties(pool: &PgPool, institution_id: Option<Uuid>) -> Result<Vec<Faculty>> {
    Ok(sqlx::query_as::<_, Faculty>(AssertSqlSafe(format!(
        "SELECT {FACULTY_COLUMNS} FROM faculties \
         WHERE ($1::uuid IS NULL OR institution_id = $1) ORDER BY name"
    )))
    .bind(institution_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_faculty(
    pool: &PgPool,
    institution_id: Uuid,
    name: &str,
    code: &str,
    description: &str,
) -> Result<Faculty> {
    Ok(sqlx::query_as::<_, Faculty>(AssertSqlSafe(format!(
        "INSERT INTO faculties (institution_id, name, code, description) \
         VALUES ($1,$2,$3,$4) RETURNING {FACULTY_COLUMNS}"
    )))
    .bind(institution_id)
    .bind(name)
    .bind(code)
    .bind(description)
    .fetch_one(pool)
    .await?)
}

pub async fn update_faculty(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    description: Option<&str>,
) -> Result<Faculty> {
    Ok(sqlx::query_as::<_, Faculty>(AssertSqlSafe(format!(
        "UPDATE faculties SET name = COALESCE($2, name), code = COALESCE($3, code), \
         description = COALESCE($4, description), updated_at = now() \
         WHERE id = $1 RETURNING {FACULTY_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(description)
    .fetch_one(pool)
    .await?)
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

pub async fn find_department_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Department>> {
    Ok(sqlx::query_as::<_, Department>(AssertSqlSafe(format!(
        "SELECT {DEPARTMENT_COLUMNS} FROM departments WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_department_by_code(pool: &PgPool, code: &str) -> Result<Option<Department>> {
    Ok(sqlx::query_as::<_, Department>(AssertSqlSafe(format!(
        "SELECT {DEPARTMENT_COLUMNS} FROM departments WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_departments(pool: &PgPool, faculty_id: Option<Uuid>) -> Result<Vec<Department>> {
    Ok(sqlx::query_as::<_, Department>(AssertSqlSafe(format!(
        "SELECT {DEPARTMENT_COLUMNS} FROM departments \
         WHERE ($1::uuid IS NULL OR faculty_id = $1) ORDER BY name"
    )))
    .bind(faculty_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_department(
    pool: &PgPool,
    faculty_id: Uuid,
    name: &str,
    code: &str,
    description: &str,
    head_id: Option<Uuid>,
) -> Result<Department> {
    Ok(sqlx::query_as::<_, Department>(AssertSqlSafe(format!(
        "INSERT INTO departments (faculty_id, name, code, description, head_id) \
         VALUES ($1,$2,$3,$4,$5) RETURNING {DEPARTMENT_COLUMNS}"
    )))
    .bind(faculty_id)
    .bind(name)
    .bind(code)
    .bind(description)
    .bind(head_id)
    .fetch_one(pool)
    .await?)
}

pub async fn update_department(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    description: Option<&str>,
    head_id: Option<Uuid>,
) -> Result<Department> {
    Ok(sqlx::query_as::<_, Department>(AssertSqlSafe(format!(
        "UPDATE departments SET name = COALESCE($2, name), code = COALESCE($3, code), \
         description = COALESCE($4, description), head_id = COALESCE($5, head_id), \
         updated_at = now() WHERE id = $1 RETURNING {DEPARTMENT_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(description)
    .bind(head_id)
    .fetch_one(pool)
    .await?)
}


// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

pub async fn find_program_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Program>> {
    Ok(sqlx::query_as::<_, Program>(AssertSqlSafe(format!(
        "SELECT {PROGRAM_COLUMNS} FROM programs WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_program_by_code(pool: &PgPool, code: &str) -> Result<Option<Program>> {
    Ok(sqlx::query_as::<_, Program>(AssertSqlSafe(format!(
        "SELECT {PROGRAM_COLUMNS} FROM programs WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_programs(pool: &PgPool, department_id: Option<Uuid>) -> Result<Vec<Program>> {
    Ok(sqlx::query_as::<_, Program>(AssertSqlSafe(format!(
        "SELECT {PROGRAM_COLUMNS} FROM programs \
         WHERE ($1::uuid IS NULL OR department_id = $1) ORDER BY name"
    )))
    .bind(department_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_program(
    pool: &PgPool,
    department_id: Uuid,
    name: &str,
    code: &str,
    degree_type: &str,
    duration_years: i32,
    description: &str,
) -> Result<Program> {
    Ok(sqlx::query_as::<_, Program>(AssertSqlSafe(format!(
        "INSERT INTO programs (department_id, name, code, degree_type, duration_years, description) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {PROGRAM_COLUMNS}"
    )))
    .bind(department_id)
    .bind(name)
    .bind(code)
    .bind(degree_type)
    .bind(duration_years)
    .bind(description)
    .fetch_one(pool)
    .await?)
}

pub async fn update_program(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    degree_type: Option<&str>,
    duration_years: Option<i32>,
    description: Option<&str>,
) -> Result<Program> {
    Ok(sqlx::query_as::<_, Program>(AssertSqlSafe(format!(
        "UPDATE programs SET name = COALESCE($2, name), code = COALESCE($3, code), \
         degree_type = COALESCE($4, degree_type), duration_years = COALESCE($5, duration_years), \
         description = COALESCE($6, description), updated_at = now() \
         WHERE id = $1 RETURNING {PROGRAM_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(degree_type)
    .bind(duration_years)
    .bind(description)
    .fetch_one(pool)
    .await?)
}


// ---------------------------------------------------------------------------
// Academic years
// ---------------------------------------------------------------------------

pub async fn find_year_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AcademicYear>> {
    Ok(sqlx::query_as::<_, AcademicYear>(AssertSqlSafe(format!(
        "SELECT {YEAR_COLUMNS} FROM academic_years WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_year_by_code(pool: &PgPool, code: &str) -> Result<Option<AcademicYear>> {
    Ok(sqlx::query_as::<_, AcademicYear>(AssertSqlSafe(format!(
        "SELECT {YEAR_COLUMNS} FROM academic_years WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_years(pool: &PgPool) -> Result<Vec<AcademicYear>> {
    Ok(sqlx::query_as::<_, AcademicYear>(AssertSqlSafe(format!(
        "SELECT {YEAR_COLUMNS} FROM academic_years ORDER BY start_date DESC"
    )))
    .fetch_all(pool)
    .await?)
}

pub async fn insert_year(
    pool: &PgPool,
    name: &str,
    code: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<AcademicYear> {
    Ok(sqlx::query_as::<_, AcademicYear>(AssertSqlSafe(format!(
        "INSERT INTO academic_years (name, code, start_date, end_date) \
         VALUES ($1,$2,$3,$4) RETURNING {YEAR_COLUMNS}"
    )))
    .bind(name)
    .bind(code)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?)
}

pub async fn update_year(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    status: Option<&str>,
) -> Result<AcademicYear> {
    Ok(sqlx::query_as::<_, AcademicYear>(AssertSqlSafe(format!(
        "UPDATE academic_years SET name = COALESCE($2, name), code = COALESCE($3, code), \
         start_date = COALESCE($4, start_date), end_date = COALESCE($5, end_date), \
         status = COALESCE($6, status), updated_at = now() \
         WHERE id = $1 RETURNING {YEAR_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(start_date)
    .bind(end_date)
    .bind(status)
    .fetch_one(pool)
    .await?)
}


// ---------------------------------------------------------------------------
// Semesters
// ---------------------------------------------------------------------------

pub async fn find_semester_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Semester>> {
    Ok(sqlx::query_as::<_, Semester>(AssertSqlSafe(format!(
        "SELECT {SEMESTER_COLUMNS} FROM semesters WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_semester_by_code(pool: &PgPool, code: &str) -> Result<Option<Semester>> {
    Ok(sqlx::query_as::<_, Semester>(AssertSqlSafe(format!(
        "SELECT {SEMESTER_COLUMNS} FROM semesters WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_semesters(
    pool: &PgPool,
    academic_year_id: Option<Uuid>,
) -> Result<Vec<Semester>> {
    Ok(sqlx::query_as::<_, Semester>(AssertSqlSafe(format!(
        "SELECT {SEMESTER_COLUMNS} FROM semesters \
         WHERE ($1::uuid IS NULL OR academic_year_id = $1) ORDER BY start_date"
    )))
    .bind(academic_year_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_semester(
    pool: &PgPool,
    academic_year_id: Uuid,
    name: &str,
    code: &str,
    session_number: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Semester> {
    Ok(sqlx::query_as::<_, Semester>(AssertSqlSafe(format!(
        "INSERT INTO semesters (academic_year_id, name, code, session_number, start_date, end_date) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {SEMESTER_COLUMNS}"
    )))
    .bind(academic_year_id)
    .bind(name)
    .bind(code)
    .bind(session_number)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?)
}

pub async fn update_semester(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    session_number: Option<i32>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    status: Option<&str>,
) -> Result<Semester> {
    Ok(sqlx::query_as::<_, Semester>(AssertSqlSafe(format!(
        "UPDATE semesters SET name = COALESCE($2, name), code = COALESCE($3, code), \
         session_number = COALESCE($4, session_number), start_date = COALESCE($5, start_date), \
         end_date = COALESCE($6, end_date), status = COALESCE($7, status), updated_at = now() \
         WHERE id = $1 RETURNING {SEMESTER_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(session_number)
    .bind(start_date)
    .bind(end_date)
    .bind(status)
    .fetch_one(pool)
    .await?)
}


// ---------------------------------------------------------------------------
// Classes / cohorts
// ---------------------------------------------------------------------------

pub async fn find_class_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Class>> {
    Ok(sqlx::query_as::<_, Class>(AssertSqlSafe(format!(
        "SELECT {CLASS_COLUMNS} FROM classes WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn find_class_by_code(pool: &PgPool, code: &str) -> Result<Option<Class>> {
    Ok(sqlx::query_as::<_, Class>(AssertSqlSafe(format!(
        "SELECT {CLASS_COLUMNS} FROM classes WHERE lower(code) = lower($1)"
    )))
    .bind(code)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_classes(
    pool: &PgPool,
    program_id: Option<Uuid>,
    academic_year_id: Option<Uuid>,
) -> Result<Vec<Class>> {
    Ok(sqlx::query_as::<_, Class>(AssertSqlSafe(format!(
        "SELECT {CLASS_COLUMNS} FROM classes \
         WHERE ($1::uuid IS NULL OR program_id = $1) \
           AND ($2::uuid IS NULL OR academic_year_id = $2) ORDER BY name"
    )))
    .bind(program_id)
    .bind(academic_year_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_class(
    pool: &PgPool,
    program_id: Uuid,
    academic_year_id: Uuid,
    name: &str,
    code: &str,
    intake_date: Option<NaiveDate>,
    capacity: i32,
) -> Result<Class> {
    Ok(sqlx::query_as::<_, Class>(AssertSqlSafe(format!(
        "INSERT INTO classes (program_id, academic_year_id, name, code, intake_date, capacity) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {CLASS_COLUMNS}"
    )))
    .bind(program_id)
    .bind(academic_year_id)
    .bind(name)
    .bind(code)
    .bind(intake_date)
    .bind(capacity)
    .fetch_one(pool)
    .await?)
}

pub async fn update_class(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    code: Option<&str>,
    intake_date: Option<NaiveDate>,
    capacity: Option<i32>,
) -> Result<Class> {
    Ok(sqlx::query_as::<_, Class>(AssertSqlSafe(format!(
        "UPDATE classes SET name = COALESCE($2, name), code = COALESCE($3, code), \
         intake_date = COALESCE($4, intake_date), capacity = COALESCE($5, capacity), \
         updated_at = now() WHERE id = $1 RETURNING {CLASS_COLUMNS}"
    )))
    .bind(id)
    .bind(name)
    .bind(code)
    .bind(intake_date)
    .bind(capacity)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_class(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM classes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Course offerings
// ---------------------------------------------------------------------------

pub async fn find_offering_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Offering>> {
    Ok(sqlx::query_as::<_, Offering>(AssertSqlSafe(format!(
        "SELECT {OFFERING_COLUMNS} FROM course_offerings WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

pub async fn list_offerings(
    pool: &PgPool,
    course_id: Option<Uuid>,
    semester_id: Option<Uuid>,
) -> Result<Vec<Offering>> {
    Ok(sqlx::query_as::<_, Offering>(AssertSqlSafe(format!(
        "SELECT {OFFERING_COLUMNS} FROM course_offerings \
         WHERE ($1::uuid IS NULL OR course_id = $1) \
           AND ($2::uuid IS NULL OR semester_id = $2) ORDER BY created_at DESC"
    )))
    .bind(course_id)
    .bind(semester_id)
    .fetch_all(pool)
    .await?)
}

pub async fn insert_offering(
    pool: &PgPool,
    course_id: Uuid,
    semester_id: Uuid,
    program_id: Uuid,
    instructor_id: Uuid,
    section: &str,
    max_enrolment: i32,
) -> Result<Offering> {
    Ok(sqlx::query_as::<_, Offering>(AssertSqlSafe(format!(
        "INSERT INTO course_offerings \
            (course_id, semester_id, program_id, instructor_id, section, max_enrolment) \
         VALUES ($1,$2,$3,$4,$5,$6) RETURNING {OFFERING_COLUMNS}"
    )))
    .bind(course_id)
    .bind(semester_id)
    .bind(program_id)
    .bind(instructor_id)
    .bind(section)
    .bind(max_enrolment)
    .fetch_one(pool)
    .await?)
}

pub async fn update_offering(
    pool: &PgPool,
    id: Uuid,
    instructor_id: Option<Uuid>,
    section: Option<&str>,
    max_enrolment: Option<i32>,
    status: Option<&str>,
) -> Result<Offering> {
    Ok(sqlx::query_as::<_, Offering>(AssertSqlSafe(format!(
        "UPDATE course_offerings SET instructor_id = COALESCE($2, instructor_id), \
         section = COALESCE($3, section), max_enrolment = COALESCE($4, max_enrolment), \
         status = COALESCE($5, status), updated_at = now() \
         WHERE id = $1 RETURNING {OFFERING_COLUMNS}"
    )))
    .bind(id)
    .bind(instructor_id)
    .bind(section)
    .bind(max_enrolment)
    .bind(status)
    .fetch_one(pool)
    .await?)
}

pub async fn delete_offering(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM course_offerings WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn delete_semester(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM semesters WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn delete_year(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM academic_years WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn delete_program(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM programs WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn delete_department(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM departments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn delete_faculty(pool: &PgPool, id: Uuid) -> Result<bool> {
    let res = sqlx::query("DELETE FROM faculties WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
}
