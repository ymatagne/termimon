//! Creature ↔ Agent assignment mapping
//!
//! Default assignments:
//!   - claude (Claude Code)  → embercli  🔥
//!   - codex (Codex CLI)     → voltprompt ⚡
//!   - aider                 → shelloise  💧
//!
//! Users can override assignments in config or at runtime.

use super::registry;
use std::collections::HashMap;

/// Default creature assignment for known AI agents.
///
/// Returns the creature species name for a given agent identifier.
/// Matches are case-insensitive and substring-based:
///   "claude", "claude-code", "claude_code"  → "embercli"
///   "codex", "openai-codex"                 → "voltprompt"
///   "aider"                                 → "shelloise"
pub fn default_creature_for_agent(agent: &str) -> Option<&'static str> {
    let agent_lower = agent.to_lowercase();

    // Check in priority order (most specific first)
    if agent_lower.contains("claude") {
        Some("embercli")
    } else if agent_lower.contains("codex") {
        Some("voltprompt")
    } else if agent_lower.contains("aider") {
        Some("shelloise")
    } else {
        // Check the registry's default_agent field as fallback
        registry::creature_for_agent(agent).map(|def| def.name)
    }
}

/// Resolve the creature for an agent, checking user overrides first.
///
/// Priority:
///   1. Explicit user override from config (`assignments` map)
///   2. Default assignment from the agent name
///   3. None (unknown agent)
pub fn resolve_creature(
    agent: &str,
    user_overrides: &HashMap<String, String>,
) -> Option<String> {
    // 1. Check user overrides (exact match, then substring)
    let agent_lower = agent.to_lowercase();
    for (key, creature) in user_overrides {
        if agent_lower == key.to_lowercase() || agent_lower.contains(&key.to_lowercase()) {
            // Validate the creature exists
            if registry::get_creature_def(creature).is_some() {
                return Some(creature.clone());
            }
        }
    }

    // 2. Default mapping
    default_creature_for_agent(agent).map(|s| s.to_string())
}

/// All default agent→creature assignments, for display/config generation.
pub fn default_assignments() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("claude", "embercli",   "🔥"),
        ("codex",  "voltprompt", "⚡"),
        ("aider",  "shelloise",  "💧"),
    ]
}

/// Check if a creature species name is valid.
pub fn is_valid_creature(species: &str) -> bool {
    registry::get_creature_def(species).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mappings() {
        assert_eq!(default_creature_for_agent("claude"), Some("embercli"));
        assert_eq!(default_creature_for_agent("Claude Code"), Some("embercli"));
        assert_eq!(default_creature_for_agent("codex"), Some("voltprompt"));
        assert_eq!(default_creature_for_agent("openai-codex"), Some("voltprompt"));
        assert_eq!(default_creature_for_agent("aider"), Some("shelloise"));
        assert_eq!(default_creature_for_agent("unknown_agent"), None);
    }

    #[test]
    fn user_override_wins() {
        let mut overrides = HashMap::new();
        overrides.insert("claude".to_string(), "shelloise".to_string());

        let result = resolve_creature("claude", &overrides);
        assert_eq!(result, Some("shelloise".to_string()));
    }

    #[test]
    fn fallback_to_default() {
        let overrides = HashMap::new();
        let result = resolve_creature("codex", &overrides);
        assert_eq!(result, Some("voltprompt".to_string()));
    }

    #[test]
    fn unknown_returns_none() {
        let overrides = HashMap::new();
        let result = resolve_creature("random_tool", &overrides);
        assert_eq!(result, None);
    }
}
