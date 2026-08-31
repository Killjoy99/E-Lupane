//! Business logic for the academic structure with relationship validation,
//! duplicate detection, and manager-gated authorization.

use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::authz::{self, Claims};
use crate::dto::{
    AcademicYearList, AcademicYearView, ClassList, ClassView, CreateAcademicYearRequest,
    CreateClassRequest, CreateDepartmentRequest, CreateFacultyRequest, CreateOfferingRequest,
    CreateProgramRequest, CreateSemesterRequest, DepartmentList, DepartmentView, FacultyList,
    FacultyView, OfferingList, OfferingView, ProgramList, ProgramView, SemesterList,
    SemesterView, UpdateAcademicYearRequest, UpdateClassRequest, UpdateDepartmentRequest,
    UpdateFacultyRequest, UpdateOfferingRequest, UpdateProgramRequest, UpdateSemesterRequest,
};
use crate::models::{
    AcademicYear, Class, Department, Faculty, Offering, Program, Semester,
};
use crate::repository;

pub const MAX_CODE_LEN: usize = 20;
pub const MAX_NAME_LEN: usize = 200;

/// Validate a short identifier (code): upper-case alphanumerics plus dashes.
pub fn validate_code(code: &str, what: &str) -> Result<()> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("{what} code is required"));
    }
    if trimmed.len() > MAX_CODE_LEN {
        return Err(anyhow!("{what} code too long (max {MAX_CODE_LEN} characters)"));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(anyhow!("{what} code may only contain letters, digits, dashes and underscores"));
    }
    if trimmed.chars().any(|c| c.is_ascii_alphabetic()) && trimmed != trimmed.to_uppercase() {
        return Err(anyhow!("{what} code must be upper-case"));
    }
    Ok(())
}

/// Validate a human-readable name.
pub fn validate_name(name: &str, what: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("{what} name is required"));
    }
    if trimmed.len() > MAX_NAME_LEN {
        return Err(anyhow!("{what} name too long (max {MAX_NAME_LEN} characters)"));
    }
    Ok(())
}

/// Ensure `start <= end` for a date range.
pub fn validate_range(start: NaiveDate, end: NaiveDate, what: &str) -> Result<()> {
    if start > end {
        return Err(anyhow!("{what} end date must be on or after the start date"));
    }
    Ok(())
}

/// Normalize a status; empty falls back to the given default.
pub fn normalize_status(s: &str, default: &'static str) -> Result<&'static str> {
    match s {
        "" => Ok(default),
        "active" => Ok("active"),
        "inactive" => Ok("inactive"),
        "planned" => Ok("planned"),
        "ongoing" => Ok("ongoing"),
        "completed" => Ok("completed"),
        "cancelled" => Ok("cancelled"),
        "open" => Ok("open"),
        "closed" => Ok("closed"),
        other => Err(anyhow!("invalid status '{other}'")),
    }
}

fn faculty_view(f: &Faculty) -> FacultyView {
    FacultyView {
        id: f.id,
        institution_id: f.institution_id,
        name: f.name.clone(),
        code: f.code.clone(),
        description: f.description.clone(),
        created_at: f.created_at,
        updated_at: f.updated_at,
    }
}

fn department_view(d: &Department) -> DepartmentView {
    DepartmentView {
        id: d.id,
        faculty_id: d.faculty_id,
        name: d.name.clone(),
        code: d.code.clone(),
        description: d.description.clone(),
        head_id: d.head_id,
        created_at: d.created_at,
        updated_at: d.updated_at,
    }
}

fn program_view(p: &Program) -> ProgramView {
    ProgramView {
        id: p.id,
        department_id: p.department_id,
        name: p.name.clone(),
        code: p.code.clone(),
        degree_type: p.degree_type.clone(),
        duration_years: p.duration_years,
        description: p.description.clone(),
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

fn year_view(y: &AcademicYear) -> AcademicYearView {
    AcademicYearView {
        id: y.id,
        name: y.name.clone(),
        code: y.code.clone(),
        start_date: y.start_date,
        end_date: y.end_date,
        status: y.status.clone(),
        created_at: y.created_at,
        updated_at: y.updated_at,
    }
}

fn semester_view(s: &Semester) -> SemesterView {
    SemesterView {
        id: s.id,
        academic_year_id: s.academic_year_id,
        name: s.name.clone(),
        code: s.code.clone(),
        session_number: s.session_number,
        start_date: s.start_date,
        end_date: s.end_date,
        status: s.status.clone(),
        created_at: s.created_at,
        updated_at: s.updated_at,
    }
}

fn class_view(c: &Class) -> ClassView {
    ClassView {
        id: c.id,
        program_id: c.program_id,
        academic_year_id: c.academic_year_id,
        name: c.name.clone(),
        code: c.code.clone(),
        intake_date: c.intake_date,
        capacity: c.capacity,
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}


// ---------------------------------------------------------------------------
// Faculties
// ---------------------------------------------------------------------------

pub async fn create_faculty(pool: &PgPool, claims: &Claims, req: CreateFacultyRequest) -> Result<FacultyView> {
    authz::require_manager(claims)?;
    if repository::find_faculty_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a faculty with code '{}' already exists", req.code));
    }
    validate_name(&req.name, "faculty")?;
    validate_code(&req.code, "faculty")?;
    let row = repository::insert_faculty(
        pool,
        req.institution_id,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.description.trim(),
    )
    .await?;
    Ok(faculty_view(&row))
}

pub async fn get_faculty(pool: &PgPool, id: Uuid) -> Result<FacultyView> {
    let row = repository::find_faculty_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("faculty not found"))?;
    Ok(faculty_view(&row))
}

pub async fn list_faculties(pool: &PgPool, institution_id: Option<Uuid>) -> Result<FacultyList> {
    let rows = repository::list_faculties(pool, institution_id).await?;
    Ok(FacultyList {
        faculties: rows.iter().map(faculty_view).collect(),
    })
}

pub async fn update_faculty(pool: &PgPool, claims: &Claims, req: UpdateFacultyRequest) -> Result<FacultyView> {
    authz::require_manager(claims)?;
    let existing = repository::find_faculty_by_id(pool, req.faculty_id)
        .await?
        .ok_or_else(|| anyhow!("faculty not found"))?;
    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "faculty")?;
    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "faculty")?;
    if let Some(dup) = repository::find_faculty_by_code(pool, &code).await? {
        if dup.id != req.faculty_id {
            return Err(anyhow!("a faculty with code '{code}' already exists"));
        }
    }
    let row = repository::update_faculty(
        pool,
        req.faculty_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.description.as_deref().map(str::trim),
    )
    .await?;
    Ok(faculty_view(&row))
}

pub async fn delete_faculty(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_faculty(pool, id).await? {
        return Err(anyhow!("faculty not found"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

pub async fn create_department(
    pool: &PgPool,
    claims: &Claims,
    req: CreateDepartmentRequest,
) -> Result<DepartmentView> {
    authz::require_manager(claims)?;
    if repository::find_department_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a department with code '{}' already exists", req.code));
    }
    validate_name(&req.name, "department")?;
    validate_code(&req.code, "department")?;
    let row = repository::insert_department(
        pool,
        req.faculty_id,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.description.trim(),
        req.head_id,
    )
    .await?;
    Ok(department_view(&row))
}

pub async fn get_department(pool: &PgPool, id: Uuid) -> Result<DepartmentView> {
    let row = repository::find_department_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("department not found"))?;
    Ok(department_view(&row))
}

pub async fn list_departments(pool: &PgPool, faculty_id: Option<Uuid>) -> Result<DepartmentList> {
    let rows = repository::list_departments(pool, faculty_id).await?;
    Ok(DepartmentList {
        departments: rows.iter().map(department_view).collect(),
    })
}

pub async fn update_department(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateDepartmentRequest,
) -> Result<DepartmentView> {
    authz::require_manager(claims)?;
    let existing = repository::find_department_by_id(pool, req.department_id)
        .await?
        .ok_or_else(|| anyhow!("department not found"))?;
    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "department")?;
    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "department")?;
    if let Some(dup) = repository::find_department_by_code(pool, &code).await? {
        if dup.id != req.department_id {
            return Err(anyhow!("a department with code '{code}' already exists"));
        }
    }
    let row = repository::update_department(
        pool,
        req.department_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.description.as_deref().map(str::trim),
        req.head_id,
    )
    .await?;
    Ok(department_view(&row))
}

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

pub async fn create_program(pool: &PgPool, claims: &Claims, req: CreateProgramRequest) -> Result<ProgramView> {
    authz::require_manager(claims)?;
    if repository::find_program_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a program with code '{}' already exists", req.code));
    }
    validate_name(&req.name, "program")?;
    validate_code(&req.code, "program")?;
    let row = repository::insert_program(
        pool,
        req.department_id,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.degree_type.trim(),
        req.duration_years,
        req.description.trim(),
    )
    .await?;
    Ok(program_view(&row))
}

pub async fn get_program(pool: &PgPool, id: Uuid) -> Result<ProgramView> {
    let row = repository::find_program_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("program not found"))?;
    Ok(program_view(&row))
}

pub async fn list_programs(pool: &PgPool, department_id: Option<Uuid>) -> Result<ProgramList> {
    let rows = repository::list_programs(pool, department_id).await?;
    Ok(ProgramList {
        programs: rows.iter().map(program_view).collect(),
    })
}

pub async fn update_program(pool: &PgPool, claims: &Claims, req: UpdateProgramRequest) -> Result<ProgramView> {
    authz::require_manager(claims)?;
    let existing = repository::find_program_by_id(pool, req.program_id)
        .await?
        .ok_or_else(|| anyhow!("program not found"))?;
    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "program")?;
    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "program")?;
    if let Some(dup) = repository::find_program_by_code(pool, &code).await? {
        if dup.id != req.program_id {
            return Err(anyhow!("a program with code '{code}' already exists"));
        }
    }
    let row = repository::update_program(
        pool,
        req.program_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.degree_type.as_deref().map(str::trim),
        req.duration_years,
        req.description.as_deref().map(str::trim),
    )
    .await?;
    Ok(program_view(&row))
}

pub async fn delete_program(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_program(pool, id).await? {
        return Err(anyhow!("program not found"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Academic years
// ---------------------------------------------------------------------------

pub async fn create_academic_year(pool: &PgPool, claims: &Claims, req: CreateAcademicYearRequest) -> Result<AcademicYearView> {
    authz::require_manager(claims)?;
    if repository::find_year_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("an academic year with code '{}' already exists", req.code));
    }
    validate_name(&req.name, "academic year")?;
    validate_code(&req.code, "academic year")?;
    validate_range(req.start_date, req.end_date, "academic year")?;
    let row = repository::insert_year(
        pool,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.start_date,
        req.end_date,
    )
    .await?;
    Ok(year_view(&row))
}

pub async fn get_academic_year(pool: &PgPool, id: Uuid) -> Result<AcademicYearView> {
    let row = repository::find_year_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("academic year not found"))?;
    Ok(year_view(&row))
}

pub async fn list_academic_years(pool: &PgPool) -> Result<AcademicYearList> {
    let rows = repository::list_years(pool).await?;
    Ok(AcademicYearList {
        academic_years: rows.iter().map(year_view).collect(),
    })
}

pub async fn update_academic_year(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateAcademicYearRequest,
) -> Result<AcademicYearView> {
    authz::require_manager(claims)?;
    let existing = repository::find_year_by_id(pool, req.academic_year_id)
        .await?
        .ok_or_else(|| anyhow!("academic year not found"))?;
    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "academic year")?;
    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "academic year")?;
    if let Some(dup) = repository::find_year_by_code(pool, &code).await? {
        if dup.id != req.academic_year_id {
            return Err(anyhow!("an academic year with code '{code}' already exists"));
        }
    }
    if let Some(status) = &req.status {
        normalize_status(status, "active")?;
    }
    let row = repository::update_year(
        pool,
        req.academic_year_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.start_date,
        req.end_date,
        req.status.as_deref(),
    )
    .await?;
    Ok(year_view(&row))
}

// ---------------------------------------------------------------------------
// Semesters
// ---------------------------------------------------------------------------

pub async fn create_semester(pool: &PgPool, claims: &Claims, req: CreateSemesterRequest) -> Result<SemesterView> {
    authz::require_manager(claims)?;
    if repository::find_semester_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a semester with code '{}' already exists", req.code));
    }
    validate_name(&req.name, "semester")?;
    validate_code(&req.code, "semester")?;
    validate_range(req.start_date, req.end_date, "semester")?;
    let row = repository::insert_semester(
        pool,
        req.academic_year_id,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.session_number,
        req.start_date,
        req.end_date,
    )
    .await?;
    Ok(semester_view(&row))
}

pub async fn get_semester(pool: &PgPool, id: Uuid) -> Result<SemesterView> {
    let row = repository::find_semester_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("semester not found"))?;
    Ok(semester_view(&row))
}

pub async fn list_semesters(pool: &PgPool, academic_year_id: Option<Uuid>) -> Result<SemesterList> {
    let rows = repository::list_semesters(pool, academic_year_id).await?;
    Ok(SemesterList {
        semesters: rows.iter().map(semester_view).collect(),
    })
}

pub async fn update_semester(pool: &PgPool, claims: &Claims, req: UpdateSemesterRequest) -> Result<SemesterView> {
    authz::require_manager(claims)?;
    let existing = repository::find_semester_by_id(pool, req.semester_id)
        .await?
        .ok_or_else(|| anyhow!("semester not found"))?;
    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "semester")?;
    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "semester")?;
    if let Some(dup) = repository::find_semester_by_code(pool, &code).await? {
        if dup.id != req.semester_id {
            return Err(anyhow!("a semester with code '{code}' already exists"));
        }
    }
    if let Some(status) = &req.status {
        normalize_status(status, "planned")?;
    }
    let row = repository::update_semester(
        pool,
        req.semester_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.session_number,
        req.start_date,
        req.end_date,
        req.status.as_deref(),
    )
    .await?;
    Ok(semester_view(&row))
}

pub async fn delete_semester(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_semester(pool, id).await? {
        return Err(anyhow!("semester not found"));
    }
    Ok(())
}

pub async fn delete_academic_year(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_year(pool, id).await? {
        return Err(anyhow!("academic year not found"));
    }
    Ok(())
}

pub async fn delete_department(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_department(pool, id).await? {
        return Err(anyhow!("department not found"));
    }
    Ok(())
}

fn offering_view(o: &Offering) -> OfferingView {
    OfferingView {
        id: o.id,
        course_id: o.course_id,
        semester_id: o.semester_id,
        program_id: o.program_id,
        instructor_id: o.instructor_id,
        section: o.section.clone(),
        max_enrolment: o.max_enrolment,
        status: o.status.clone(),
        created_at: o.created_at,
        updated_at: o.updated_at,
    }
}

// ---------------------------------------------------------------------------
// Classes
// ---------------------------------------------------------------------------

pub async fn create_class(
    pool: &PgPool,
    claims: &Claims,
    req: CreateClassRequest,
) -> Result<ClassView> {
    authz::require_manager(claims)?;
    if repository::find_class_by_code(pool, &req.code).await?.is_some() {
        return Err(anyhow!("a class with code '{}' already exists", req.code));
    }
    validate_name(&req.name, "class")?;
    validate_code(&req.code, "class")?;
    if req.capacity < 0 {
        return Err(anyhow!("class capacity must be >= 0"));
    }
    let row = repository::insert_class(
        pool,
        req.program_id,
        req.academic_year_id,
        req.name.trim(),
        req.code.trim().to_uppercase().as_str(),
        req.intake_date,
        req.capacity,
    )
    .await?;
    Ok(class_view(&row))
}

pub async fn get_class(pool: &PgPool, id: Uuid) -> Result<ClassView> {
    let row = repository::find_class_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("class not found"))?;
    Ok(class_view(&row))
}

pub async fn list_classes(
    pool: &PgPool,
    program_id: Option<Uuid>,
    academic_year_id: Option<Uuid>,
) -> Result<ClassList> {
    let rows = repository::list_classes(pool, program_id, academic_year_id).await?;
    Ok(ClassList {
        classes: rows.iter().map(class_view).collect(),
    })
}

pub async fn update_class(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateClassRequest,
) -> Result<ClassView> {
    authz::require_manager(claims)?;
    let existing = repository::find_class_by_id(pool, req.class_id)
        .await?
        .ok_or_else(|| anyhow!("class not found"))?;
    let name = req.name.unwrap_or(existing.name.clone());
    validate_name(&name, "class")?;
    let code = req.code.unwrap_or(existing.code.clone());
    validate_code(&code, "class")?;
    if let Some(dup) = repository::find_class_by_code(pool, &code).await? {
        if dup.id != req.class_id {
            return Err(anyhow!("a class with code '{code}' already exists"));
        }
    }
    if let Some(c) = req.capacity {
        if c < 0 {
            return Err(anyhow!("class capacity must be >= 0"));
        }
    }
    let row = repository::update_class(
        pool,
        req.class_id,
        Some(name.trim()),
        Some(code.trim().to_uppercase().as_str()),
        req.intake_date,
        req.capacity,
    )
    .await?;
    Ok(class_view(&row))
}

pub async fn delete_class(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_class(pool, id).await? {
        return Err(anyhow!("class not found"));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Course offerings
// ---------------------------------------------------------------------------

pub async fn create_offering(
    pool: &PgPool,
    claims: &Claims,
    req: CreateOfferingRequest,
) -> Result<OfferingView> {
    authz::require_manager(claims)?;
    if req.max_enrolment < 0 {
        return Err(anyhow!("offering max_enrolment must be >= 0"));
    }
    let row = repository::insert_offering(
        pool,
        req.course_id,
        req.semester_id,
        req.program_id,
        req.instructor_id,
        req.section.trim(),
        req.max_enrolment,
    )
    .await?;
    Ok(offering_view(&row))
}

pub async fn get_offering(pool: &PgPool, id: Uuid) -> Result<OfferingView> {
    let row = repository::find_offering_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow!("offering not found"))?;
    Ok(offering_view(&row))
}

pub async fn list_offerings(
    pool: &PgPool,
    course_id: Option<Uuid>,
    semester_id: Option<Uuid>,
) -> Result<OfferingList> {
    let rows = repository::list_offerings(pool, course_id, semester_id).await?;
    Ok(OfferingList {
        offerings: rows.iter().map(offering_view).collect(),
    })
}

pub async fn update_offering(
    pool: &PgPool,
    claims: &Claims,
    req: UpdateOfferingRequest,
) -> Result<OfferingView> {
    authz::require_manager(claims)?;
    repository::find_offering_by_id(pool, req.offering_id)
        .await?
        .ok_or_else(|| anyhow!("offering not found"))?;
    if let Some(c) = req.max_enrolment {
        if c < 0 {
            return Err(anyhow!("offering max_enrolment must be >= 0"));
        }
    }
    if let Some(status) = &req.status {
        normalize_status(status, "open")?;
    }
    let row = repository::update_offering(
        pool,
        req.offering_id,
        req.instructor_id,
        req.section.as_deref().map(str::trim),
        req.max_enrolment,
        req.status.as_deref(),
    )
    .await?;
    Ok(offering_view(&row))
}

pub async fn delete_offering(pool: &PgPool, claims: &Claims, id: Uuid) -> Result<()> {
    authz::require_manager(claims)?;
    if !repository::delete_offering(pool, id).await? {
        return Err(anyhow!("offering not found"));
    }
    Ok(())
}
