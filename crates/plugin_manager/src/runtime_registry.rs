use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::runtime_plugin::RuntimePlugin;

pub struct RuntimeRegistry {
    // Map of plugin name to its runtime instance
    plugins: RwLock<HashMap<String, RuntimePlugin>>,
}

impl RuntimeRegistry {
    pub fn new() -> Self {
        Self { plugins: RwLock::new(HashMap::new()) }
    }

    pub async fn add(
        &self,
        plugin: RuntimePlugin,
    ) {
        // In a real implementation, we would also track the plugin's state and handle errors during startup
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin.manifest.id.clone(), plugin,);
    }

    pub async fn list(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }


}