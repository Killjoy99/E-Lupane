use envy::{from_env, Error};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub environment: String,
}

impl AppConfig {
    // Populate the app with env variables
    pub fn load() -> Result<Self, Error> {
        from_env()
    }
}