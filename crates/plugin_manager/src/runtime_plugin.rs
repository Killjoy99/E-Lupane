use elupane_core::{
    plugin_state::PluginState,
    manifest::PluginManifest,
};
// use chrono::{DateTime, Utc};

pub struct RuntimePlugin {
    pub manifest: PluginManifest,
    pub state: PluginState,

    // process handle or other runtime-specific data can be added here
    pub pid: Option<u32>, // For process-based plugins, store the PID
    // pub started_at: Option<DateTime<Utc>>,
}