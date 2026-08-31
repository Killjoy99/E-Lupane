//! Authorization: JWT verification and course ownership/permission rules.
//!
//! The course management plugin trusts tokens issued by the auth plugin
//! (`com.elupane.auth`). Both plugins share the same HS256 secret
//! (`AUTH_JWT_SECRET`), so identity + roles are verified locally.
//!
//! Ownership model:
//! * Admins may do anything.
//! * The course **owner** may manage the course.
//! * Users listed in `course_instructors` may also manage the course.
//! * Everyone may view **published** courses; drafts and other non-public
//!   statuses are only visible to people who can manage them.

use anyhow::{anyhow, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Roles allowed to create and manage courses.
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
// Course ownership rules (pure functions — unit-tested)
// ---------------------------------------------------------------------------

/// Whether the caller can *manage* a course.
///
/// `instructor_ids` is the list of user ids recorded in `course_instructors`
/// for the course (not including the owner).
pub fn can_manage(claims: &Claims, owner_id: Uuid, instructor_ids: &[Uuid]) -> Result<bool> {
    Ok(claims.is_admin()
        || claims.user_id()? == owner_id
        || instructor_ids.contains(&claims.user_id()?))
}

/// Whether the caller can *view* a course in its current state.
///
/// Non-public statuses are only visible to people who manage the course;
/// published courses are visible to any authenticated viewer.
pub fn can_view(
    claims: &Claims,
    status: &str,
    owner_id: Uuid,
    instructor_ids: &[Uuid],
) -> Result<bool> {
    if status == "published" {
        return Ok(true);
    }
    can_manage(claims, owner_id, instructor_ids)
}

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
        let token = make_token(id, "alice", &["student"]);
        let claims = verify_token(&token).unwrap();
        assert_eq!(claims.user_id().unwrap(), id);
        assert_eq!(claims.username, "alice");
        assert_eq!(claims.roles, vec!["student"]);
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
        assert!(claims(Uuid::new_v4(), &["admin"]).is_admin());
        assert!(!claims(Uuid::new_v4(), &["lecturer"]).is_admin());
    }

    #[test]
    fn owner_can_manage_their_course() {
        let owner = Uuid::new_v4();
        let c = claims(owner, &["lecturer"]);
        assert!(can_manage(&c, owner, &[]).unwrap());
    }

    #[test]
    fn other_lecturer_cannot_manage_without_authorization() {
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let c = claims(other, &["lecturer"]);
        assert!(!can_manage(&c, owner, &[]).unwrap());
    }

    #[test]
    fn authorized_instructor_can_manage() {
        let owner = Uuid::new_v4();
        let other = Uuid::new_v4();
        let c = claims(other, &["lecturer"]);
        assert!(can_manage(&c, owner, &[other]).unwrap());
    }

    #[test]
    fn admin_can_manage_any_course() {
        let owner = Uuid::new_v4();
        let admin = claims(Uuid::new_v4(), &["admin"]);
        assert!(can_manage(&admin, owner, &[]).unwrap());
    }

    #[test]
    fn student_cannot_manage() {
        let owner = Uuid::new_v4();
        let student = claims(Uuid::new_v4(), &["student"]);
        assert!(!can_manage(&student, owner, &[]).unwrap());
    }

    #[test]
    fn published_visible_to_everyone_drafts_only_to_managers() {
        let owner = Uuid::new_v4();
        let student = claims(Uuid::new_v4(), &["student"]);
        let other_lecturer = claims(Uuid::new_v4(), &["lecturer"]);
        assert!(can_view(&student, "published", owner, &[]).unwrap());
        assert!(!can_view(&student, "draft", owner, &[]).unwrap());
        assert!(can_view(&other_lecturer, "draft", owner, &[other_lecturer.user_id().unwrap()]).unwrap());
        assert!(!can_view(&other_lecturer, "draft", owner, &[]).unwrap());
    }

    #[test]
    fn instructor_checks_enforced() {
        let lecturer = claims(Uuid::new_v4(), &["lecturer"]);
        let student = claims(Uuid::new_v4(), &["student"]);
        assert!(require_instructor(&lecturer).is_ok());
        assert!(require_instructor(&student).is_err());
    }
}

