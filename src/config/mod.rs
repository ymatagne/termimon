//! Configuration management for TermiMon

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Top-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub statusbar: StatusBarConfig,
    #[serde(default)]
    pub popup: PopupConfig,
    #[serde(default)]
    pub creatures: CreatureConfig,
    #[serde(default)]
    pub agents: AgentConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub team: TeamConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_display_mode")]
    pub display_mode: String,
    #[serde(default = "default_animation_fps")]
    pub animation_fps: u32,
    #[serde(default = "default_theme")]
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarConfig {
    #[serde(default = "default_position")]
    pub position: String,
    #[serde(default = "default_max_creatures")]
    pub max_creatures: usize,
    #[serde(default)]
    pub show_xp: bool,
    #[serde(default = "default_true")]
    pub show_state: bool,
    #[serde(default = "default_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopupConfig {
    #[serde(default = "default_popup_width")]
    pub width: u16,
    #[serde(default = "default_popup_height")]
    pub height: u16,
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureConfig {
    #[serde(default)]
    pub assignments: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub custom: Vec<CustomAgent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgent {
    pub name: String,
    pub process: String,
    #[serde(default)]
    pub patterns: Vec<String>,
    #[serde(default)]
    pub creature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_true")]
    pub evolution: bool,
    #[serde(default)]
    pub terminal_bell: bool,
    #[serde(default = "default_true")]
    pub system_notify: bool,
    #[serde(default = "default_cost_alert_threshold")]
    pub cost_alert_threshold_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    #[serde(default = "default_team_name")]
    pub name: String,
    #[serde(default = "default_team_port")]
    pub port: u16,
    #[serde(default)]
    pub auto_host: bool,
}

impl Default for TeamConfig {
    fn default() -> Self {
        Self {
            name: default_team_name(),
            port: default_team_port(),
            auto_host: false,
        }
    }
}

fn default_team_name() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "trainer".into())
}
fn default_team_port() -> u16 { 4662 }

fn default_cost_alert_threshold() -> u64 { 1000 } // $10

// Default value functions
fn default_poll_interval() -> u64 { 2000 }
fn default_display_mode() -> String { "statusbar".into() }
fn default_animation_fps() -> u32 { 4 }
fn default_theme() -> String { "default".into() }
fn default_position() -> String { "right".into() }
fn default_max_creatures() -> usize { 5 }
fn default_true() -> bool { true }
fn default_format() -> String { "{icon} {name}[{state}]".into() }
fn default_popup_width() -> u16 { 60 }
fn default_popup_height() -> u16 { 20 }
fn default_hotkey() -> String { "P".into() }

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            statusbar: StatusBarConfig::default(),
            popup: PopupConfig::default(),
            creatures: CreatureConfig::default(),
            agents: AgentConfig::default(),
            notifications: NotificationConfig::default(),
            team: TeamConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: default_poll_interval(),
            display_mode: default_display_mode(),
            animation_fps: default_animation_fps(),
            theme: default_theme(),
        }
    }
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            max_creatures: default_max_creatures(),
            show_xp: false,
            show_state: true,
            format: default_format(),
        }
    }
}

impl Default for PopupConfig {
    fn default() -> Self {
        Self {
            width: default_popup_width(),
            height: default_popup_height(),
            hotkey: default_hotkey(),
        }
    }
}

impl Default for CreatureConfig {
    fn default() -> Self {
        Self {
            assignments: HashMap::new(),
        }
    }
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self { custom: Vec::new() }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            evolution: true,
            terminal_bell: false,
            system_notify: true,
            cost_alert_threshold_cents: default_cost_alert_threshold(),
        }
    }
}

/// Get the TermiMon config directory path
pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".termimon")
}

/// Get the config file path
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Load configuration from disk, creating defaults if needed
pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => return config,
                Err(e) => {
                    tracing::warn!("Failed to parse config: {e}, using defaults");
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config: {e}, using defaults");
            }
        }
    }
    Config::default()
}

/// Save configuration to disk
#[allow(dead_code)]
pub fn save(config: &Config) -> std::io::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let contents = toml::to_string_pretty(config).expect("Failed to serialize config");
    std::fs::write(config_path(), contents)
}

/// Generate default config file content
pub fn default_config_toml() -> String {
    r#"# TermiMon Configuration 🎮
# Your AI agents, alive in the terminal

[general]
poll_interval_ms = 2000
display_mode = "statusbar"  # "statusbar" | "popup" | "pane"
animation_fps = 4
theme = "default"

[statusbar]
position = "right"
max_creatures = 5
show_xp = false
show_state = true
format = "{icon} {name}[{state}]"

[popup]
width = 60
height = 20
hotkey = "P"

[creatures.assignments]
claude = "embercli"
codex = "voltprompt"
aider = "shelloise"

[notifications]
evolution = true
terminal_bell = false
system_notify = true
cost_alert_threshold_cents = 1000  # $10.00
"#
    .to_string()
}

/// Handle the `config` CLI command
pub async fn handle_config(edit: bool, path: Option<PathBuf>) -> anyhow::Result<()> {
    let config_file = path.unwrap_or_else(config_path);

    if !config_file.exists() {
        std::fs::create_dir_all(config_file.parent().unwrap())?;
        std::fs::write(&config_file, default_config_toml())?;
        println!("📝 Created default config at {}", config_file.display());
    }

    if edit {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
        let status = std::process::Command::new(&editor)
            .arg(&config_file)
            .status()?;
        if !status.success() {
            anyhow::bail!("Editor exited with error");
        }
    } else {
        let contents = std::fs::read_to_string(&config_file)?;
        println!("📋 Config at {}:\n", config_file.display());
        println!("{contents}");
    }

    Ok(())
}
