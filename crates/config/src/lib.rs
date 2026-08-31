use envy::{from_env, Error};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub environment: String,
    /// Redis connection string used for inter-plugin messaging.
    #[serde(default = "default_redis_url")]
    pub redis_url: String,
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379".to_string()
}

impl AppConfig {
    // Populate the app with env variables
    pub fn load() -> Result<Self, Error> {
        from_env()
    }
}