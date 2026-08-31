use std::collections::HashMap;
use std::sync::Arc;

use elupane_core::message::CapabilityAnnouncement;
use event_bus::EventBus;
use plugin_manager::PluginManager;
use tokio::process::Child;
use tokio::sync::RwLock;

/// Holds the global state of the application.
///
/// `plugin_manager` and `event_bus` are behind `Arc` so they could be shared
/// across spawned tasks. `plugin_children` owns the live `Child` handles for
/// every launched plugin process so the kernel supervisor can wait for them
/// and terminate them cleanly on shutdown.
///
/// `capabilities` is the live map of plugin id -> capability announcement,
/// fed by the Redis Pub/Sub channel and the durable registry.
pub struct AppState {
    pub plugin_manager: Arc<PluginManager>,
    pub event_bus: Arc<EventBus>,
    pub plugin_children: Vec<Child>,
    pub capabilities: Arc<RwLock<HashMap<String, CapabilityAnnouncement>>>,
}