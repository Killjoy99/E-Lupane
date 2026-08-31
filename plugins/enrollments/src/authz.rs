//! Authorization: JWT verification and enrollment permission rules.
//!
//! The enrollments plugin trusts tokens issued by the auth plugin
//! (`com.elupane.auth`). Both share the same HS256 secret (`AUTH_JWT_SECRET`),
//! so identity + roles are verified locally.
//!
//! Enrollment permission model:
//! * **Students** may enroll *themselves* in a published course (lands in
//!   `pending` until approved), cancel their own enrollment, and re-enroll.
//! * **Instructors** may enroll others into courses they manage, approve
//!   pending enrollments, suspend/cancel/reinstate them, and create invites.
//! * **Admins** may do all of the above on any course.
//! * **Course content access** (lessons, assignments) requires an *active*
//!   enrollment — or a manager role on the course.

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::EnrollmentState;

/// Roles allowed to manage courses and enroll other users.
pub const INSTRUCTOR_ROLES: &[&str] = &["admin", "lecturer"];

/// The claims carried in a token issued by the auth plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub roles: Vec<String>,
    pub exp: usize,
}

impl Claims {
    /// The authenticated user id (the JWT `sub`).
    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|_| anyhow!("token subject is not a valid user id"))
    }

    /// Whether the caller is an administrator.
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    /// Whether the caller holds an instructor role (admin or lecturer).
    pub fn is_instructor(&self) -> bool {
        self.roles.iter().any(|r| INSTRUCTOR_ROLES.contains(&r.as_str()))
    }

    /// Whether the caller holds the student role.
    pub fn is_student(&self) -> bool {
        self.roles.iter().any(|r| r == "student")
    }
}

/// Shared HS256 secret (must match the auth plugin's `AUTH_JWT_SECRET`).
pub fn jwt_secret() -> String {
    std::env::var("AUTH_JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string())
}

/// Verify a JWT and return its claims.
pub fn verify_token(token: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

/// Require the caller to hold at least one of the given roles.
pub fn require_role(claims: &Claims, allowed: &[&str]) -> Result<()> {
    if claims.roles.iter().any(|r| allowed.contains(&r.as_str())) {
        Ok(())
    } else {
        Err(anyhow!("forbidden: requires one of {allowed:?}"))
    }
}

/// Require an instructor (admin or lecturer).
pub fn require_instructor(claims: &Claims) -> Result<()> {
    require_role(claims, INSTRUCTOR_ROLES)
}

// ---------------------------------------------------------------------------
// Enrollment rules (pure functions — unit-tested)
// ---------------------------------------------------------------------------

/// Whether the caller can *manage the roster* of a course (approve, suspend,
/// cancel, bulk-enroll, invite). Admins always can; otherwise the caller must
/// be the course owner or one of its instructors.
pub fn can_manage_roster(claims: &Claims, owner_id: Uuid, instructor_ids: &[Uuid]) -> Result<bool> {
    Ok(claims.is_admin()
        || claims.user_id()? == owner_id
        || instructor_ids.contains(&claims.user_id()?))
}

/// Whether the caller may enroll *someone else* into the course
/// (`student_id`). The course's manager decides; a plain student may only
/// ever enroll themselves.
pub fn can_enroll_other(
    claims: &Claims,
    student_id: Uuid,
    owner_id: Uuid,
    instructor_ids: &[Uuid],
) -> Result<bool> {
    if claims.user_id()? == student_id {
        return Ok(true); // self-enrollment
    }
    can_manage_roster(claims, owner_id, instructor_ids)
}

/// Whether the caller may *access course content* (lessons, assignments).
///
/// Managers always can; everybody else needs an `active` enrollment.
pub fn can_access_content(
    claims: &Claims,
    enrollment_state: Option<EnrollmentState>,
    owner_id: Uuid,
    instructor_ids: &[Uuid],
) -> Result<bool> {
    if can_manage_roster(claims, owner_id, instructor_ids)? {
        return Ok(true);
    }
    Ok(enrollment_state == Some(EnrollmentState::Active))
}

/// Whether the caller may change the *state* of an enrollment row
/// (`actor` is the caller, `student_id` the enrolled student).
///
/// * Managers: any transition they need (approve/suspend/cancel/reinstate).
/// * The student themself: only their own cancel (withdraw) / re-enroll.
pub fn can_change_enrollment(
    claims: &Claims,
    student_id: Uuid,
    owner_id: Uuid,
    instructor_ids: &[Uuid],
) -> Result<bool> {
    if claims.user_id()? == student_id {
        return Ok(true); // may withdraw / re-enroll own enrollment
    }
    can_manage_roster(claims, owner_id, instructor_ids)
}
/// The authenticated caller, derived from a verified token.
#[derive(Debug, Clone)]
pub struct Caller {
    pub claims: Claims,
}

impl Caller {
    pub fn new(claims: Claims) -> Self {
        Self { claims }
    }

    pub fn user_id(&self) -> Result<Uuid> {
        self.claims.user_id()
    }

    pub fn is_admin(&self) -> bool {
        self.claims.is_admin()
    }

    pub fn is_instructor(&self) -> bool {
        self.claims.is_instructor()
    }
}

#[cfg(test)]

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn make_token(user_id: Uuid, username: &str, roles: &[&str]) -> String {
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(jwt_secret().as_bytes()),
        )
        .unwrap()
    }

    fn claims(user_id: Uuid, roles: &[&str]) -> Claims {
        verify_token(&make_token(user_id, "user", roles)).unwrap()
    }

    #[test]
    fn token_round_trip() {
        let id = Uuid::new_v4();
        let c = verify_token(&make_token(id, "alice", &["student"])).unwrap();
        assert_eq!(c.user_id().unwrap(), id);
        assert_eq!(c.username, "alice");
        assert_eq!(c.roles, vec!["student"]);
    }

    #[test]
    fn rejects_garbage_token() {
        assert!(verify_token("not-a-real-token").is_err());
    }

    #[test]
    fn role_classification() {
        assert!(claims(Uuid::new_v4(), &["lecturer"]).is_instructor());
        assert!(claims(Uuid::new_v4(), &["admin"]).is_instructor());
        assert!(!claims(Uuid::new_v4(), &["student"]).is_instructor());
        assert!(claims(Uuid::new_v4(), &["student"]).is_student());
        assert!(claims(Uuid::new_v4(), &["admin"]).is_admin());
        assert!(!claims(Uuid::new_v4(), &["lecturer"]).is_admin());
    }

    #[test]
    fn only_managers_manage_the_roster() {
        let owner = Uuid::new_v4();
        let co_instructor = Uuid::new_v4();
        assert!(can_manage_roster(&claims(owner, &["lecturer"]), owner, &[]).unwrap());
        assert!(can_manage_roster(&claims(co_instructor, &["lecturer"]), owner, &[co_instructor]).unwrap());
        assert!(can_manage_roster(&claims(Uuid::new_v4(), &["admin"]), owner, &[]).unwrap());
        assert!(!can_manage_roster(&claims(Uuid::new_v4(), &["student"]), owner, &[]).unwrap());
        assert!(!can_manage_roster(&claims(Uuid::new_v4(), &["lecturer"]), owner, &[]).unwrap());
    }

    #[test]
    fn students_only_enroll_themselves() {
        let owner = Uuid::new_v4();
        let student = claims(Uuid::new_v4(), &["student"]);
        let sid = student.user_id().unwrap();
        assert!(can_enroll_other(&student, sid, owner, &[]).unwrap());
        assert!(!can_enroll_other(&student, Uuid::new_v4(), owner, &[]).unwrap());
        let lecturer = claims(Uuid::new_v4(), &["lecturer"]);
        assert!(can_enroll_other(&lecturer, Uuid::new_v4(), owner, &[]).is_ok());
    }

    #[test]
    fn unauthorized_lecturer_cannot_enroll_others() {
        let owner = Uuid::new_v4();
        let outsider = claims(Uuid::new_v4(), &["lecturer"]);
        assert!(!can_enroll_other(&outsider, Uuid::new_v4(), owner, &[]).unwrap());
        assert!(!can_manage_roster(&outsider, owner, &[]).unwrap());
    }

    #[test]
    fn content_access_requires_active_enrollment_or_manager() {
        let owner = Uuid::new_v4();
        let student = claims(Uuid::new_v4(), &["student"]);
        assert!(can_access_content(&student, Some(EnrollmentState::Active), owner, &[]).unwrap());
        for state in [
            None,
            Some(EnrollmentState::Pending),
            Some(EnrollmentState::Suspended),
            Some(EnrollmentState::Expired),
            Some(EnrollmentState::Cancelled),
            Some(EnrollmentState::Completed),
        ] {
            assert!(!can_access_content(&student, state, owner, &[]).unwrap());
        }
        let lecturer = claims(Uuid::new_v4(), &["lecturer"]);
        assert!(can_access_content(&lecturer, None, owner, &[lecturer.user_id().unwrap()]).unwrap());
        assert!(can_access_content(&claims(Uuid::new_v4(), &["admin"]), None, owner, &[]).unwrap());
    }

    #[test]
    fn students_can_only_change_their_own_enrollment() {
        let owner = Uuid::new_v4();
        let student = claims(Uuid::new_v4(), &["student"]);
        let sid = student.user_id().unwrap();
        assert!(can_change_enrollment(&student, sid, owner, &[]).unwrap());
        assert!(!can_change_enrollment(&student, Uuid::new_v4(), owner, &[]).unwrap());
        let lecturer = claims(Uuid::new_v4(), &["lecturer"]);
        assert!(can_change_enrollment(&lecturer, Uuid::new_v4(), owner, &[lecturer.user_id().unwrap()]).unwrap());
        assert!(!can_change_enrollment(&lecturer, Uuid::new_v4(), owner, &[]).unwrap());
    }

    #[test]
    fn instructor_checks_enforced() {
        let lecturer = claims(Uuid::new_v4(), &["lecturer"]);
        let student = claims(Uuid::new_v4(), &["student"]);
        assert!(require_instructor(&lecturer).is_ok());
        assert!(require_instructor(&student).is_err());
    }
}