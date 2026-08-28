// Plugin registry and management
// The plugin manager should track metadata separately from runtime instances.
use std::collections::HashMap;

use elupane_core::manifest::{PluginManifest};


pub struct PluginRegistry {
    // Map of plugin name to its manifest
    plugins: HashMap<String, PluginManifest>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    // Register a plugin manifest
    pub fn register(&mut self, manifest: PluginManifest) {
        self.plugins.insert(manifest.name.clone(), manifest);
    }

    // Get a plugin manifest by name
    pub fn get(&self, name: &str) -> Option<&PluginManifest> {
        self.plugins.get(name)
    }

    // List all registered plugins
    pub fn list(&self) -> Vec<&PluginManifest> {
        self.plugins.values().collect()
    }
}