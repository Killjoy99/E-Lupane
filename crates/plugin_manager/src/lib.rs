pub mod registry;
pub mod discovery;
pub mod runtime_registry;
pub mod runtime_plugin;
pub mod launcher;

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::RwLock;

use elupane_core::plugin::Plugin;
use registry::PluginRegistry;


pub struct PluginManager {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    registry: RwLock<PluginRegistry>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            registry: RwLock::new(PluginRegistry::new()),
        }
    }

    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<()> {
        let mut plugins = self.plugins.write().await;

        plugins.insert(plugin.metadata().name.to_string(), plugin);

        Ok(())
    }

    pub async fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;

        plugins.keys().cloned().collect()
    }
}