//! Stable agent identity — persistent creature bindings across restarts
//!
//! Agent ID = short hex hash of (agent_kind + working_directory).
//! Creature bindings are stored in ~/.termimon/creatures.json.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Persistent creature binding for a specific agent identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureBinding {
    pub creature_species: String,
    pub xp: u64,
    pub stage: u8,
    pub first_seen: String,
    pub sessions: u64,
}

/// Compute a stable agent ID from kind + working directory.
/// If working_dir is "/" or unknown, uses PID as differentiator (not stable across restarts,
/// but at least gives each agent its own identity).
/// Returns a short 8-char hex hash.
pub fn compute_agent_id(agent_kind: &str, working_dir: Option<&str>) -> String {
    let wd = working_dir.unwrap_or("unknown");
    let input = format!("{}:{}", agent_kind, wd);
    // Simple FNV-1a 64-bit hash → truncated to 8 hex chars
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in input.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", hash as u32)
}

/// Compute agent ID with PID fallback when working dir is useless.
/// Uses kind + working_dir for stable identity across restarts.
/// Only falls back to PID when working dir is unknown (not stable, but unique per instance).
pub fn compute_agent_id_with_pid(agent_kind: &str, working_dir: Option<&str>, pid: Option<u32>) -> String {
    let wd = working_dir.unwrap_or("/");
    if wd == "/" || wd == "unknown" || wd.is_empty() {
        // Use PID as differentiator — not stable across restarts but unique per instance
        let input = format!("{}:pid:{}", agent_kind, pid.unwrap_or(0));
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in input.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        format!("{:08x}", hash as u32)
    } else {
        // Stable: kind + working dir (survives restarts)
        compute_agent_id(agent_kind, Some(wd))
    }
}

fn bindings_path() -> PathBuf {
    let dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".termimon");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("creatures.json")
}

/// Load all creature bindings from disk.
pub fn load_bindings() -> HashMap<String, CreatureBinding> {
    let path = bindings_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

/// Save all creature bindings to disk.
pub fn save_bindings(bindings: &HashMap<String, CreatureBinding>) {
    let path = bindings_path();
    if let Ok(json) = serde_json::to_string_pretty(bindings) {
        let _ = std::fs::write(&path, json);
    }
}

/// Look up or create a creature binding for the given agent.
/// Returns (binding, is_new).
pub fn get_or_create_binding(
    agent_id: &str,
    creature_species: &str,
) -> (CreatureBinding, bool) {
    let mut bindings = load_bindings();

    if let Some(existing) = bindings.get_mut(agent_id) {
        existing.sessions += 1;
        let binding = existing.clone();
        save_bindings(&bindings);
        (binding, false)
    } else {
        let binding = CreatureBinding {
            creature_species: creature_species.to_string(),
            xp: 0,
            stage: 1,
            first_seen: chrono::Utc::now().to_rfc3339(),
            sessions: 1,
        };
        bindings.insert(agent_id.to_string(), binding.clone());
        save_bindings(&bindings);
        (binding, true)
    }
}

/// Update XP and stage for a given agent.
pub fn update_xp(agent_id: &str, xp: u64, stage: u8) {
    let mut bindings = load_bindings();
    if let Some(binding) = bindings.get_mut(agent_id) {
        binding.xp = xp;
        binding.stage = stage;
        save_bindings(&bindings);
    }
}
