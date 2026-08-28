// Plugin manifests and traits

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub capabilities: Vec<String>,
    pub runtime: PluginRuntime,
    pub language: PluginLanguage,
    pub dependencies: Vec<String>,
    pub entry: String,
    /// Optional working directory for the plugin process. When set, the plugin
    /// is launched with this as its current directory (relative to the
    /// workspace root). This lets plugins locate their own `.env` and assets.
    #[serde(default)]
    pub working_dir: Option<String>,
    pub minimum_core_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
// This enum defines the supported plugin runtimes. This will help us determine how to load and execute the plugin.
pub enum PluginRuntime {
    Process,
    WebAssembly,
    Native,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginLanguage {
    Rust,
    Python,
    Php,
    CSharp,
    JavaScript,
    TypeScript,
}
