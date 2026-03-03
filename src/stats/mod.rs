//! Daily stats persistence.
//!
//! Stores daily aggregate data at ~/.termimon/stats/YYYY-MM-DD.json.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};

use crate::agents::cost::AgentCostTracker;

/// Daily stats for all agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub agents: HashMap<String, AgentDailyStats>,
    pub total_cost_cents: u64,
}

/// Stats for a single agent on a given day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDailyStats {
    pub kind: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_cents: u64,
    pub sessions: u32,
    pub active_minutes: u64,
}

impl Default for DailyStats {
    fn default() -> Self {
        Self {
            date: today_str(),
            agents: HashMap::new(),
            total_cost_cents: 0,
        }
    }
}

/// Get the stats directory path.
pub fn stats_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".termimon")
        .join("stats")
}

/// Today's date as YYYY-MM-DD string.
pub fn today_str() -> String {
    let now = Utc::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

/// Path to today's stats file.
pub fn today_stats_path() -> PathBuf {
    stats_dir().join(format!("{}.json", today_str()))
}

/// Load stats for a specific date.
pub fn load_stats(date: &str) -> Result<DailyStats> {
    let path = stats_dir().join(format!("{date}.json"));
    if !path.exists() {
        return Ok(DailyStats {
            date: date.to_string(),
            ..Default::default()
        });
    }
    let content = std::fs::read_to_string(&path)?;
    let stats: DailyStats = serde_json::from_str(&content)?;
    Ok(stats)
}

/// Load today's stats (or create empty).
pub fn load_today() -> DailyStats {
    load_stats(&today_str()).unwrap_or_default()
}

/// Save stats for today.
pub fn save_today(stats: &DailyStats) -> Result<()> {
    let dir = stats_dir();
    std::fs::create_dir_all(&dir)?;
    let path = today_stats_path();
    let content = serde_json::to_string_pretty(stats)?;
    std::fs::write(&path, content)?;
    tracing::debug!("Saved daily stats to {}", path.display());
    Ok(())
}

/// Update today's stats from a cost tracker snapshot.
/// This merges current cost data into the daily aggregate.
pub fn update_from_costs(tracker: &AgentCostTracker, agent_kind: &str) -> Result<()> {
    let mut daily = load_today();

    for (agent_id, agent_cost) in &tracker.agents {
        let entry = daily.agents.entry(agent_id.clone()).or_insert_with(|| {
            AgentDailyStats {
                kind: agent_kind.to_string(),
                input_tokens: 0,
                output_tokens: 0,
                cost_cents: 0,
                sessions: 0,
                active_minutes: 0,
            }
        });

        // Overwrite with latest cumulative values
        // (since cost tracker already accumulates)
        entry.input_tokens = agent_cost.total_input_tokens;
        entry.output_tokens = agent_cost.total_output_tokens;
        entry.cost_cents = agent_cost.total_cost_cents;
        entry.sessions = agent_cost.sessions;
    }

    daily.total_cost_cents = daily.agents.values().map(|a| a.cost_cents).sum();
    daily.date = today_str();

    save_today(&daily)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_today_str_format() {
        let s = today_str();
        // Should be YYYY-MM-DD
        assert_eq!(s.len(), 10);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
    }

    #[test]
    fn test_daily_stats_serde() {
        let mut stats = DailyStats::default();
        stats.agents.insert(
            "test_agent".into(),
            AgentDailyStats {
                kind: "claude".into(),
                input_tokens: 45000,
                output_tokens: 12000,
                cost_cents: 315,
                sessions: 3,
                active_minutes: 45,
            },
        );
        stats.total_cost_cents = 315;

        let json = serde_json::to_string(&stats).unwrap();
        let parsed: DailyStats = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.total_cost_cents, 315);
        assert!(parsed.agents.contains_key("test_agent"));
    }
}
