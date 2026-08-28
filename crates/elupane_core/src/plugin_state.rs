// Track the state of plugins, including loaded plugins, their capabilities, and their dependencies.
#[derive(Debug, Clone)]
pub enum PluginState {
    Discovered,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String), // include error message
}