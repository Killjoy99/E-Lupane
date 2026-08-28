// discovery.rs
use tracing::{self, error, info, warn};
use std::fs;
use std::path::{Path};

use anyhow::Result;

use elupane_core::manifest::PluginManifest;

pub fn discover_plugins(
    plugins_dir: &str,
) -> Result<Vec<PluginManifest>> {
    let mut manifests = Vec::new();

    let path = Path::new(plugins_dir);

    info!("Scanning plugins directory: {:?}", path);

    if !path.exists() {
        warn!("Plugins directory does not exist");
        return Ok(manifests);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;

        let plugin_path = entry.path();

        info!("Found: {:?}", plugin_path);

        if plugin_path.is_dir() {
            let manifest_path = plugin_path.join("plugin.toml");

            info!("Checking: {:?}", manifest_path);

            if manifest_path.exists() {
                info!("Found manifest: {:?}", manifest_path);

                let content = fs::read_to_string(&manifest_path)?;

                match toml::from_str::<PluginManifest>(&content) {
                    Ok(manifest) => {
                        println!("Loaded plugin: {}", manifest.name);
                        manifests.push(manifest);
                    }
                    Err(err) => {
                        error!(
                            "Failed to parse {:?}: {}",
                            manifest_path,
                            err
                        );
                    }
                }
            }
        }
    }

    Ok(manifests)
}