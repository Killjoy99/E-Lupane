use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: &'static str,
    pub version: &'static str,
    pub authors: Vec<&'static str>,
    pub description: &'static str,
}

#[async_trait]
pub trait Plugin: Send + Sync {
    
    fn metadata(&self) -> PluginMetadata;

    async fn on_load(&self) -> Result<()>;

    async fn on_enable(&self) -> Result<()>;

    async fn on_disable(&self) -> Result<()>;
}