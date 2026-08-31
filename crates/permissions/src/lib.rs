//! Permission and capability primitives for the E-Lupane plugin system.

pub mod capability;
pub mod rbac;

pub use capability::{Capability, CapabilitySet};
pub use rbac::{Role, UserPrincipal, any_grants};

