//! Plugin system — load custom creature definitions from TOML files
//!
//! Plugins live in `~/.termimon/plugins/` and each defines a custom creature.

use serde::Deserialize;
use std::path::PathBuf;

/// A custom creature defined by a plugin TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginCreature {
    pub name: String,
    pub element: String,
    pub default_agent: String,
    pub description: String,
    pub evolution_names: Vec<String>,
    #[serde(default)]
    pub detect_process: Vec<String>,
}

/// Get the plugins directory path.
pub fn plugins_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".termimon")
        .join("plugins")
}

/// Load all plugin creature definitions from disk.
pub fn load_plugins() -> Vec<PluginCreature> {
    let dir = plugins_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut plugins = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                match load_plugin(&path) {
                    Ok(p) => {
                        tracing::info!("Loaded plugin creature: {}", p.name);
                        plugins.push(p);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }
    }
    plugins
}

fn load_plugin(path: &std::path::Path) -> anyhow::Result<PluginCreature> {
    let content = std::fs::read_to_string(path)?;
    let plugin: PluginCreature = toml::from_str(&content)?;
    if plugin.evolution_names.len() != 3 {
        anyhow::bail!("Plugin '{}' must have exactly 3 evolution names", plugin.name);
    }
    Ok(plugin)
}

/// Check if a process name matches any plugin's detect_process patterns.
pub fn match_plugin_process<'a>(process_name: &str, plugins: &'a [PluginCreature]) -> Option<&'a PluginCreature> {
    let lower = process_name.to_lowercase();
    plugins.iter().find(|p| {
        p.detect_process.iter().any(|pattern| lower.contains(&pattern.to_lowercase()))
    })
}
