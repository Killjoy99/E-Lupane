//! Request/response payloads exchanged over the message envelope.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Faculties
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateFacultyRequest {
    pub institution_id: Uuid,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFacultyRequest {
    pub faculty_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FacultyIdRequest {
    pub faculty_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListFacultiesRequest {
    #[serde(default)]
    pub institution_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct FacultyView {
    pub id: Uuid,
    pub institution_id: Uuid,
    pub name: String,
    pub code: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FacultyList {
    pub faculties: Vec<FacultyView>,
}

// ---------------------------------------------------------------------------
// Departments
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub faculty_id: Uuid,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub head_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub department_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub head_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct DepartmentIdRequest {
    pub department_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListDepartmentsRequest {
    #[serde(default)]
    pub faculty_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DepartmentView {
    pub id: Uuid,
    pub faculty_id: Uuid,
    pub name: String,
    pub code: String,
    pub description: String,
    pub head_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DepartmentList {
    pub departments: Vec<DepartmentView>,
}

// ---------------------------------------------------------------------------
// Programs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateProgramRequest {
    pub department_id: Uuid,
    pub name: String,
    pub code: String,
    #[serde(default = "default_degree_type")]
    pub degree_type: String,
    #[serde(default = "default_duration")]
    pub duration_years: i32,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgramRequest {
    pub program_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub degree_type: Option<String>,
    #[serde(default)]
    pub duration_years: Option<i32>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProgramIdRequest {
    pub program_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListProgramsRequest {
    #[serde(default)]
    pub department_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ProgramView {
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

#[derive(Debug, Serialize)]
pub struct ProgramList {
    pub programs: Vec<ProgramView>,
}

// ---------------------------------------------------------------------------
// Academic years
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateAcademicYearRequest {
    pub name: String,
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAcademicYearRequest {
    pub academic_year_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub status: Option<String>,
}

// ---------------------------------------------------------------------------
// Semesters
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateSemesterRequest {
    pub academic_year_id: Uuid,
    pub name: String,
    pub code: String,
    #[serde(default = "default_session")]
    pub session_number: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSemesterRequest {
    pub semester_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub session_number: Option<i32>,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SemesterIdRequest {
    pub semester_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListSemestersRequest {
    #[serde(default)]
    pub academic_year_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct SemesterView {
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

#[derive(Debug, Serialize)]
pub struct SemesterList {
    pub semesters: Vec<SemesterView>,
}

fn default_session() -> i32 {
    1
}

// ---------------------------------------------------------------------------
// Classes / cohorts
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateClassRequest {
    pub program_id: Uuid,
    pub academic_year_id: Uuid,
    pub name: String,
    pub code: String,
    #[serde(default)]
    pub intake_date: Option<NaiveDate>,
    #[serde(default)]
    pub capacity: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClassRequest {
    pub class_id: Uuid,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub intake_date: Option<NaiveDate>,
    #[serde(default)]
    pub capacity: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ClassIdRequest {
    pub class_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListClassesRequest {
    #[serde(default)]
    pub program_id: Option<Uuid>,
    #[serde(default)]
    pub academic_year_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ClassView {
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

#[derive(Debug, Serialize)]
pub struct ClassList {
    pub classes: Vec<ClassView>,
}

// ---------------------------------------------------------------------------
// Course offerings
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateOfferingRequest {
    pub course_id: Uuid,
    pub semester_id: Uuid,
    pub program_id: Uuid,
    pub instructor_id: Uuid,
    #[serde(default)]
    pub section: String,
    #[serde(default)]
    pub max_enrolment: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOfferingRequest {
    pub offering_id: Uuid,
    #[serde(default)]
    pub instructor_id: Option<Uuid>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub max_enrolment: Option<i32>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OfferingIdRequest {
    pub offering_id: Uuid,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListOfferingsRequest {
    #[serde(default)]
    pub course_id: Option<Uuid>,
    #[serde(default)]
    pub semester_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct OfferingView {
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

#[derive(Debug, Serialize)]
pub struct OfferingList {
    pub offerings: Vec<OfferingView>,
}


#[derive(Debug, Deserialize)]
pub struct AcademicYearIdRequest {
    pub academic_year_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct AcademicYearView {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AcademicYearList {
    pub academic_years: Vec<AcademicYearView>,
}

fn default_degree_type() -> String {
    "degree".to_string()
}
fn default_duration() -> i32 {
    4
}
