//! Capability model used for inter-plugin permission broadcasting.
//!
//! A capability is a dotted string such as `"auth.user.create"`. Wildcards are
//! supported: `"auth.user.*"` matches any `auth.user.<op>`.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// A single capability/permission name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability(pub String);

impl Capability {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this capability grants `required`.
    ///
    /// Exact matches win; otherwise a trailing `*` segment matches any
    /// remaining segments (`"auth.user.*"` grants `"auth.user.create"`).
    pub fn matches(&self, required: &str) -> bool {
        if self.0 == required {
            return true;
        }

        let cap: Vec<&str> = self.0.split('.').collect();
        let req: Vec<&str> = required.split('.').collect();

        // A wildcard only helps if it is the last segment of the capability.
        if cap.last() != Some(&"*") {
            return false;
        }

        cap.len() <= req.len()
            && cap
                .iter()
                .zip(req.iter())
                .all(|(a, b)| *a == "*" || a == b)
    }
}

/// An ordered set of granted capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<String>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, capability: impl Into<String>) {
        self.capabilities.insert(capability.into());
    }

    pub fn revoke(&mut self, capability: &str) {
        self.capabilities.remove(capability);
    }

    /// Whether any granted capability covers `required` (wildcard aware).
    pub fn contains(&self, required: &str) -> bool {
        self.capabilities
            .iter()
            .any(|c| Capability::new(c.clone()).matches(required))
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.capabilities.iter()
    }

    pub fn to_vec(&self) -> Vec<String> {
        self.capabilities.iter().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }
}

impl FromIterator<String> for CapabilitySet {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self {
            capabilities: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let cap = Capability::new("auth.user.create");
        assert!(cap.matches("auth.user.create"));
        assert!(!cap.matches("auth.user.update"));
        assert!(!cap.matches("auth.role.list"));
    }

    #[test]
    fn wildcard_match() {
        let cap = Capability::new("auth.user.*");
        assert!(cap.matches("auth.user.create"));
        assert!(cap.matches("auth.user.delete"));
        assert!(!cap.matches("auth.role.list"));
        assert!(!cap.matches("auth.user"));
    }

    #[test]
    fn capability_set_contains() {
        let mut set = CapabilitySet::new();
        set.grant("auth.user.*");
        assert!(set.contains("auth.user.create"));
        assert!(!set.contains("auth.role.assign"));

        set.grant("auth.role.assign");
        assert!(set.contains("auth.role.assign"));

        set.revoke("auth.role.assign");
        assert!(!set.contains("auth.role.assign"));
    }
}
