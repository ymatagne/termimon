//! Token & cost tracking for AI agents.
//!
//! Parses Claude Code JSONL transcripts from ~/.claude/projects/**/*.jsonl
//! to extract token usage and compute estimated costs.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Per-model token stats snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    /// Total cost in **cents** (avoids floating-point drift).
    pub total_cost_cents: u64,
    pub model: String,
    pub last_updated: DateTime<Utc>,
}

impl Default for TokenStats {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
            total_cost_cents: 0,
            model: String::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Per-agent cost tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCostTracker {
    /// Map of agent_id → per-model stats.
    pub agents: HashMap<String, AgentCost>,
}

/// Costs for a single agent (may span multiple models in one session).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCost {
    pub agent_id: String,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_cents: u64,
    pub per_model: HashMap<String, TokenStats>,
    pub sessions: u32,
    pub last_updated: DateTime<Utc>,
}

impl Default for AgentCost {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost_cents: 0,
            per_model: HashMap::new(),
            sessions: 0,
            last_updated: Utc::now(),
        }
    }
}

impl Default for AgentCostTracker {
    fn default() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }
}

// ─── Pricing ─────────────────────────────────────────────────────────────

/// Pricing per million tokens, stored in micro-dollars (1 cent = 10_000 µ$).
/// We use u64 micro-cents internally to stay integer-only.
struct ModelPricing {
    /// Cost per 1M input tokens in cents (e.g. $3/MTok = 300 cents).
    input_cents_per_mtok: u64,
    /// Cost per 1M output tokens in cents.
    output_cents_per_mtok: u64,
}

fn pricing_for_model(model: &str) -> ModelPricing {
    let m = model.to_lowercase();
    if m.contains("opus") {
        ModelPricing { input_cents_per_mtok: 1500, output_cents_per_mtok: 7500 }
    } else if m.contains("haiku") {
        ModelPricing { input_cents_per_mtok: 100, output_cents_per_mtok: 500 }
    } else if m.contains("sonnet") {
        ModelPricing { input_cents_per_mtok: 300, output_cents_per_mtok: 1500 }
    } else {
        // Default fallback: sonnet pricing
        ModelPricing { input_cents_per_mtok: 300, output_cents_per_mtok: 1500 }
    }
}

/// Compute cost in cents for the given token counts and model.
pub fn compute_cost_cents(input_tokens: u64, output_tokens: u64, model: &str) -> u64 {
    let p = pricing_for_model(model);
    // cost = tokens * cents_per_mtok / 1_000_000
    // Use u128 to avoid overflow on large token counts
    let input_cost = (input_tokens as u128 * p.input_cents_per_mtok as u128) / 1_000_000;
    let output_cost = (output_tokens as u128 * p.output_cents_per_mtok as u128) / 1_000_000;
    (input_cost + output_cost) as u64
}

// ─── JSONL parsing ───────────────────────────────────────────────────────

/// Find all Claude JSONL transcript files.
pub fn find_transcript_files() -> Vec<PathBuf> {
    let pattern = dirs::home_dir()
        .map(|h| {
            h.join(".claude")
                .join("projects")
                .join("**")
                .join("*.jsonl")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_default();

    if pattern.is_empty() {
        return Vec::new();
    }

    glob::glob(&pattern)
        .map(|paths| paths.filter_map(|p| p.ok()).collect())
        .unwrap_or_default()
}

/// Token usage extracted from a single JSONL line.
#[derive(Debug, Clone)]
pub struct TokenUsageEvent {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub model: String,
    pub timestamp: Option<DateTime<Utc>>,
    /// The session identifier (usually the file stem).
    pub session_id: String,
}

/// Scan a JSONL file for token usage events.
pub fn parse_transcript_tokens(path: &PathBuf) -> Vec<TokenUsageEvent> {
    let session_id = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".into());

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut events = Vec::new();
    for line in content.lines() {
        if let Some(ev) = parse_token_line(line, &session_id) {
            events.push(ev);
        }
    }
    events
}

fn parse_token_line(line: &str, session_id: &str) -> Option<TokenUsageEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = v.as_object()?;

    // Claude Code JSONL has `usage` objects with input/output tokens
    // Try top-level usage field
    let usage = obj.get("usage").and_then(|u| u.as_object());
    // Also try nested in message.usage
    let msg_usage = obj
        .get("message")
        .and_then(|m| m.as_object())
        .and_then(|m| m.get("usage"))
        .and_then(|u| u.as_object());

    let usage = usage.or(msg_usage)?;

    let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let cache_create = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

    if input == 0 && output == 0 {
        return None;
    }

    let model = obj
        .get("model")
        .or_else(|| obj.get("message").and_then(|m| m.get("model")))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let timestamp = obj
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Some(TokenUsageEvent {
        input_tokens: input,
        output_tokens: output,
        cache_creation_input_tokens: cache_create,
        cache_read_input_tokens: cache_read,
        model,
        timestamp,
        session_id: session_id.to_string(),
    })
}

// ─── Tracker operations ──────────────────────────────────────────────────

impl AgentCostTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingest token usage events for a given agent.
    pub fn ingest(&mut self, agent_id: &str, events: &[TokenUsageEvent]) {
        let cost = self.agents.entry(agent_id.to_string()).or_insert_with(|| {
            AgentCost {
                agent_id: agent_id.to_string(),
                ..Default::default()
            }
        });

        for ev in events {
            cost.total_input_tokens += ev.input_tokens;
            cost.total_output_tokens += ev.output_tokens;

            let ev_cost = compute_cost_cents(ev.input_tokens, ev.output_tokens, &ev.model);
            cost.total_cost_cents += ev_cost;

            let model_stats = cost.per_model.entry(ev.model.clone()).or_insert_with(|| {
                TokenStats {
                    model: ev.model.clone(),
                    ..Default::default()
                }
            });
            model_stats.input_tokens += ev.input_tokens;
            model_stats.output_tokens += ev.output_tokens;
            model_stats.cache_creation_input_tokens += ev.cache_creation_input_tokens;
            model_stats.cache_read_input_tokens += ev.cache_read_input_tokens;
            model_stats.total_cost_cents += ev_cost;
            model_stats.last_updated = Utc::now();
        }

        cost.last_updated = Utc::now();
    }

    /// Scan all Claude transcripts and rebuild cost data.
    /// This is an incremental-friendly interface: for now it does a full scan.
    pub fn scan_all_transcripts(&mut self) {
        let files = find_transcript_files();
        // Group by session
        for file in &files {
            let events = parse_transcript_tokens(file);
            if events.is_empty() {
                continue;
            }
            let session_id = events[0].session_id.clone();
            // Use session_id as agent_id proxy (each JSONL = one session)
            self.ingest(&session_id, &events);
        }
    }

    /// Get a serializable summary for IPC (per-agent costs).
    pub fn summary(&self) -> Vec<AgentCostSummary> {
        self.agents
            .values()
            .map(|c| AgentCostSummary {
                agent_id: c.agent_id.clone(),
                input_tokens: c.total_input_tokens,
                output_tokens: c.total_output_tokens,
                cost_cents: c.total_cost_cents,
                sessions: c.sessions,
                last_updated: c.last_updated,
            })
            .collect()
    }

    /// Total cost across all agents in cents.
    pub fn total_cost_cents(&self) -> u64 {
        self.agents.values().map(|c| c.total_cost_cents).sum()
    }
}

/// Lightweight summary for IPC responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCostSummary {
    pub agent_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_cents: u64,
    pub sessions: u32,
    pub last_updated: DateTime<Utc>,
}

/// Format a cost in cents as a human-readable string like "$1.23".
pub fn format_cost(cents: u64) -> String {
    let dollars = cents / 100;
    let remainder = cents % 100;
    format!("${dollars}.{remainder:02}")
}

/// Format token count with comma separators.
pub fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cost_sonnet() {
        // 1M input tokens at $3/MTok = 300 cents
        assert_eq!(compute_cost_cents(1_000_000, 0, "claude-sonnet-4-20250514"), 300);
        // 1M output tokens at $15/MTok = 1500 cents
        assert_eq!(compute_cost_cents(0, 1_000_000, "claude-sonnet-4-20250514"), 1500);
    }

    #[test]
    fn test_compute_cost_opus() {
        assert_eq!(compute_cost_cents(1_000_000, 0, "claude-opus-4-20250514"), 1500);
        assert_eq!(compute_cost_cents(0, 1_000_000, "claude-opus-4-20250514"), 7500);
    }

    #[test]
    fn test_compute_cost_haiku() {
        assert_eq!(compute_cost_cents(1_000_000, 0, "claude-3-5-haiku"), 100);
        assert_eq!(compute_cost_cents(0, 1_000_000, "claude-3-5-haiku"), 500);
    }

    #[test]
    fn test_format_cost() {
        assert_eq!(format_cost(0), "$0.00");
        assert_eq!(format_cost(315), "$3.15");
        assert_eq!(format_cost(1500), "$15.00");
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(format_tokens(500), "500");
        assert_eq!(format_tokens(1200), "1.2K");
        assert_eq!(format_tokens(1_500_000), "1.5M");
    }
}
