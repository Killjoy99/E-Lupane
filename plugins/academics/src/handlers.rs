//! Command dispatch: maps incoming envelope ops to academics services and
//! builds replies.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use elupane_core::message::Message;

use crate::authz::{self, Claims};
use crate::dto::{
    AcademicYearIdRequest, ClassIdRequest, CreateAcademicYearRequest, CreateClassRequest,
    CreateDepartmentRequest, CreateFacultyRequest, CreateOfferingRequest, CreateProgramRequest,
    CreateSemesterRequest, DepartmentIdRequest, FacultyIdRequest, ListClassesRequest,
    ListDepartmentsRequest, ListFacultiesRequest, ListOfferingsRequest, ListProgramsRequest,
    ListSemestersRequest, OfferingIdRequest, ProgramIdRequest, SemesterIdRequest,
    UpdateAcademicYearRequest, UpdateClassRequest, UpdateDepartmentRequest, UpdateFacultyRequest,
    UpdateOfferingRequest, UpdateProgramRequest, UpdateSemesterRequest,
};
use crate::services;

pub const PLUGIN_ID: &str = "com.elupane.academics";

/// Handle a command and produce a reply message (success or error).
pub async fn handle_command(pool: &PgPool, command: &Message) -> Message {
    match dispatch(pool, command).await {
        Ok(reply) => reply,
        Err(err) => Message::error_reply(command, PLUGIN_ID, "academics.error", &err.to_string()),
    }
}

/// Extract the caller's JWT from the payload and verify it.
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

fn deleted(command: &Message, key: &str, id: Uuid) -> Message {
    Message::reply_to(
        command,
        PLUGIN_ID,
        serde_json::json!({ "deleted": true, key: id }),
    )
}

async fn dispatch(pool: &PgPool, command: &Message) -> Result<Message> {
    match command.op.as_str() {
        // ---- Faculties ----
        "faculty.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateFacultyRequest = parse_payload(&command.payload)?;
            let view = services::create_faculty(pool, &claims, req).await?;
            reply(command, &view)
        }
        "faculty.get" => {
            let _claims = authorize(&command.payload)?;
            let req: FacultyIdRequest = parse_payload(&command.payload)?;
            let view = services::get_faculty(pool, req.faculty_id).await?;
            reply(command, &view)
        }
        "faculty.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListFacultiesRequest = parse_payload(&command.payload)?;
            let view = services::list_faculties(pool, req.institution_id).await?;
            reply(command, &view)
        }
        "faculty.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateFacultyRequest = parse_payload(&command.payload)?;
            let view = services::update_faculty(pool, &claims, req).await?;
            reply(command, &view)
        }
        "faculty.delete" => {
            let claims = authorize(&command.payload)?;
            let req: FacultyIdRequest = parse_payload(&command.payload)?;
            services::delete_faculty(pool, &claims, req.faculty_id).await?;
            Ok(deleted(command, "faculty_id", req.faculty_id))
        }

        // ---- Departments ----
        "department.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateDepartmentRequest = parse_payload(&command.payload)?;
            let view = services::create_department(pool, &claims, req).await?;
            reply(command, &view)
        }
        "department.get" => {
            let _claims = authorize(&command.payload)?;
            let req: DepartmentIdRequest = parse_payload(&command.payload)?;
            let view = services::get_department(pool, req.department_id).await?;
            reply(command, &view)
        }
        "department.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListDepartmentsRequest = parse_payload(&command.payload)?;
            let view = services::list_departments(pool, req.faculty_id).await?;
            reply(command, &view)
        }
        "department.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateDepartmentRequest = parse_payload(&command.payload)?;
            let view = services::update_department(pool, &claims, req).await?;
            reply(command, &view)
        }
        "department.delete" => {
            let claims = authorize(&command.payload)?;
            let req: DepartmentIdRequest = parse_payload(&command.payload)?;
            services::delete_department(pool, &claims, req.department_id).await?;
            Ok(deleted(command, "department_id", req.department_id))
        }

        // ---- Programs ----
        "program.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateProgramRequest = parse_payload(&command.payload)?;
            let view = services::create_program(pool, &claims, req).await?;
            reply(command, &view)
        }
        "program.get" => {
            let _claims = authorize(&command.payload)?;
            let req: ProgramIdRequest = parse_payload(&command.payload)?;
            let view = services::get_program(pool, req.program_id).await?;
            reply(command, &view)
        }
        "program.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListProgramsRequest = parse_payload(&command.payload)?;
            let view = services::list_programs(pool, req.department_id).await?;
            reply(command, &view)
        }
        "program.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateProgramRequest = parse_payload(&command.payload)?;
            let view = services::update_program(pool, &claims, req).await?;
            reply(command, &view)
        }
        "program.delete" => {
            let claims = authorize(&command.payload)?;
            let req: ProgramIdRequest = parse_payload(&command.payload)?;
            services::delete_program(pool, &claims, req.program_id).await?;
            Ok(deleted(command, "program_id", req.program_id))
        }

        // ---- Academic years ----
        "year.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateAcademicYearRequest = parse_payload(&command.payload)?;
            let view = services::create_academic_year(pool, &claims, req).await?;
            reply(command, &view)
        }
        "year.get" => {
            let _claims = authorize(&command.payload)?;
            let req: AcademicYearIdRequest = parse_payload(&command.payload)?;
            let view = services::get_academic_year(pool, req.academic_year_id).await?;
            reply(command, &view)
        }
        "year.list" => {
            let _claims = authorize(&command.payload)?;
            let view = services::list_academic_years(pool).await?;
            reply(command, &view)
        }
        "year.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateAcademicYearRequest = parse_payload(&command.payload)?;
            let view = services::update_academic_year(pool, &claims, req).await?;
            reply(command, &view)
        }
        "year.delete" => {
            let claims = authorize(&command.payload)?;
            let req: AcademicYearIdRequest = parse_payload(&command.payload)?;
            services::delete_academic_year(pool, &claims, req.academic_year_id).await?;
            Ok(deleted(command, "academic_year_id", req.academic_year_id))
        }

        // ---- Semesters ----
        "semester.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateSemesterRequest = parse_payload(&command.payload)?;
            let view = services::create_semester(pool, &claims, req).await?;
            reply(command, &view)
        }
        "semester.get" => {
            let _claims = authorize(&command.payload)?;
            let req: SemesterIdRequest = parse_payload(&command.payload)?;
            let view = services::get_semester(pool, req.semester_id).await?;
            reply(command, &view)
        }
        "semester.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListSemestersRequest = parse_payload(&command.payload)?;
            let view = services::list_semesters(pool, req.academic_year_id).await?;
            reply(command, &view)
        }
        "semester.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateSemesterRequest = parse_payload(&command.payload)?;
            let view = services::update_semester(pool, &claims, req).await?;
            reply(command, &view)
        }
        "semester.delete" => {
            let claims = authorize(&command.payload)?;
            let req: SemesterIdRequest = parse_payload(&command.payload)?;
            services::delete_semester(pool, &claims, req.semester_id).await?;
            Ok(deleted(command, "semester_id", req.semester_id))
        }

        // ---- Classes / cohorts ----
        "class.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateClassRequest = parse_payload(&command.payload)?;
            let view = services::create_class(pool, &claims, req).await?;
            reply(command, &view)
        }
        "class.get" => {
            let _claims = authorize(&command.payload)?;
            let req: ClassIdRequest = parse_payload(&command.payload)?;
            let view = services::get_class(pool, req.class_id).await?;
            reply(command, &view)
        }
        "class.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListClassesRequest = parse_payload(&command.payload)?;
            let view =
                services::list_classes(pool, req.program_id, req.academic_year_id).await?;
            reply(command, &view)
        }
        "class.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateClassRequest = parse_payload(&command.payload)?;
            let view = services::update_class(pool, &claims, req).await?;
            reply(command, &view)
        }
        "class.delete" => {
            let claims = authorize(&command.payload)?;
            let req: ClassIdRequest = parse_payload(&command.payload)?;
            services::delete_class(pool, &claims, req.class_id).await?;
            Ok(deleted(command, "class_id", req.class_id))
        }

        // ---- Course offerings ----
        "offering.create" => {
            let claims = authorize(&command.payload)?;
            let req: CreateOfferingRequest = parse_payload(&command.payload)?;
            let view = services::create_offering(pool, &claims, req).await?;
            reply(command, &view)
        }
        "offering.get" => {
            let _claims = authorize(&command.payload)?;
            let req: OfferingIdRequest = parse_payload(&command.payload)?;
            let view = services::get_offering(pool, req.offering_id).await?;
            reply(command, &view)
        }
        "offering.list" => {
            let _claims = authorize(&command.payload)?;
            let req: ListOfferingsRequest = parse_payload(&command.payload)?;
            let view =
                services::list_offerings(pool, req.course_id, req.semester_id).await?;
            reply(command, &view)
        }
        "offering.update" => {
            let claims = authorize(&command.payload)?;
            let req: UpdateOfferingRequest = parse_payload(&command.payload)?;
            let view = services::update_offering(pool, &claims, req).await?;
            reply(command, &view)
        }
        "offering.delete" => {
            let claims = authorize(&command.payload)?;
            let req: OfferingIdRequest = parse_payload(&command.payload)?;
            services::delete_offering(pool, &claims, req.offering_id).await?;
            Ok(deleted(command, "offering_id", req.offering_id))
        }

        // ---- Event subscription placeholder ----
        "events.subscribe" => Ok(Message::reply_to(
            command,
            PLUGIN_ID,
            serde_json::json!({ "subscribed": true }),
        )),

        _ => Err(anyhow!("unknown operation: {}", command.op)),
    }
}
