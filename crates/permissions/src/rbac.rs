//! Role-based access control primitives shared by plugins and the gateway.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capability::Capability;

/// A role with its granted permission names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

/// An authenticated principal (the identity making a request).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrincipal {
    pub user_id: Uuid,
    pub username: String,
    pub roles: Vec<String>,
}

impl UserPrincipal {
    /// Whether the principal's roles grant `required`, given the role catalog.
    pub fn has_permission(&self, roles: &[Role], required: &str) -> bool {
        roles
            .iter()
            .filter(|r| self.roles.contains(&r.name))
            .flat_map(|r| r.permissions.iter())
            .any(|p| Capability::new(p.clone()).matches(required))
    }
}

/// Whether any of `granted` covers `required` (wildcard aware).
pub fn any_grants(granted: &[String], required: &str) -> bool {
    granted
        .iter()
        .any(|p| Capability::new(p.clone()).matches(required))
}
