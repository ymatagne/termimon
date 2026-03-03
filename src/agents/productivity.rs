//! Git-based productivity metrics for tracked agents.
//!
//! Measures agent output by analyzing git changes in the agent's working directory:
//! files changed, lines added/removed, build success rate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Productivity snapshot for one agent session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductivityStats {
    /// Files changed (staged + unstaged) in the working dir
    pub files_changed: u32,
    /// Lines added (from git diff)
    pub lines_added: u32,
    /// Lines removed (from git diff)
    pub lines_removed: u32,
    /// Number of commits made during this session
    pub commits_this_session: u32,
    /// Build attempts (parsed from activity feed)
    pub build_attempts: u32,
    /// Successful builds
    pub build_successes: u32,
    /// Lines per dollar spent (efficiency metric)
    pub lines_per_dollar: f64,
}

impl ProductivityStats {
    /// Build success rate as a percentage.
    pub fn build_success_rate(&self) -> f64 {
        if self.build_attempts == 0 {
            return 0.0;
        }
        (self.build_successes as f64 / self.build_attempts as f64) * 100.0
    }

    /// Net lines (added - removed).
    pub fn net_lines(&self) -> i64 {
        self.lines_added as i64 - self.lines_removed as i64
    }

    /// Format as a compact summary string.
    pub fn summary(&self) -> String {
        let build_str = if self.build_attempts > 0 {
            format!(
                " | Builds: {}/{} ({:.0}%)",
                self.build_successes,
                self.build_attempts,
                self.build_success_rate()
            )
        } else {
            String::new()
        };

        format!(
            "{} files | +{} / -{} lines{}",
            self.files_changed,
            self.lines_added,
            self.lines_removed,
            build_str,
        )
    }
}

/// Get git diff stats for a working directory.
/// Returns (files_changed, insertions, deletions).
pub fn git_diff_stats(working_dir: &str) -> Option<(u32, u32, u32)> {
    let path = Path::new(working_dir);
    if !path.exists() {
        return None;
    }

    // Check if it's a git repo
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // Get diff stats (staged + unstaged)
    let output = std::process::Command::new("git")
        .args(["diff", "--shortstat", "HEAD"])
        .current_dir(path)
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_shortstat(&stdout)
}

/// Parse `git diff --shortstat` output.
/// Example: " 3 files changed, 45 insertions(+), 12 deletions(-)"
fn parse_shortstat(line: &str) -> Option<(u32, u32, u32)> {
    let line = line.trim();
    if line.is_empty() {
        return Some((0, 0, 0)); // clean working tree
    }

    let mut files = 0u32;
    let mut insertions = 0u32;
    let mut deletions = 0u32;

    for part in line.split(',') {
        let part = part.trim();
        let num: u32 = part
            .split_whitespace()
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);

        if part.contains("file") {
            files = num;
        } else if part.contains("insertion") {
            insertions = num;
        } else if part.contains("deletion") {
            deletions = num;
        }
    }

    Some((files, insertions, deletions))
}

/// Count commits made in the last N minutes in a working directory.
pub fn recent_commits(working_dir: &str, minutes: u32) -> u32 {
    let path = Path::new(working_dir);
    let output = std::process::Command::new("git")
        .args([
            "rev-list",
            "--count",
            &format!("--since='{} minutes ago'", minutes),
            "HEAD",
        ])
        .current_dir(path)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .parse()
                .unwrap_or(0)
        }
        _ => 0,
    }
}

/// Global productivity tracker keyed by agent_id.
#[derive(Debug, Default)]
pub struct ProductivityTracker {
    stats: HashMap<String, ProductivityStats>,
}

impl ProductivityTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update productivity stats for an agent with a known working directory.
    pub fn update(&mut self, agent_id: &str, working_dir: &str, cost_cents: u64) {
        let entry = self.stats.entry(agent_id.to_string()).or_default();

        if let Some((files, added, removed)) = git_diff_stats(working_dir) {
            entry.files_changed = files;
            entry.lines_added = added;
            entry.lines_removed = removed;
        }

        entry.commits_this_session = recent_commits(working_dir, 60);

        // Lines per dollar
        if cost_cents > 0 {
            let dollars = cost_cents as f64 / 100.0;
            entry.lines_per_dollar = entry.lines_added as f64 / dollars;
        }
    }

    /// Update build stats from activity events.
    pub fn record_build(&mut self, agent_id: &str, success: bool) {
        let entry = self.stats.entry(agent_id.to_string()).or_default();
        entry.build_attempts += 1;
        if success {
            entry.build_successes += 1;
        }
    }

    /// Get stats for an agent.
    pub fn get(&self, agent_id: &str) -> Option<&ProductivityStats> {
        self.stats.get(agent_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shortstat_full() {
        let input = " 3 files changed, 45 insertions(+), 12 deletions(-)";
        let (f, i, d) = parse_shortstat(input).unwrap();
        assert_eq!(f, 3);
        assert_eq!(i, 45);
        assert_eq!(d, 12);
    }

    #[test]
    fn parse_shortstat_empty() {
        assert_eq!(parse_shortstat(""), Some((0, 0, 0)));
    }

    #[test]
    fn parse_shortstat_no_deletions() {
        let input = " 1 file changed, 10 insertions(+)";
        let (f, i, d) = parse_shortstat(input).unwrap();
        assert_eq!(f, 1);
        assert_eq!(i, 10);
        assert_eq!(d, 0);
    }

    #[test]
    fn productivity_summary() {
        let stats = ProductivityStats {
            files_changed: 5,
            lines_added: 200,
            lines_removed: 50,
            build_attempts: 4,
            build_successes: 3,
            ..Default::default()
        };
        assert!(stats.summary().contains("+200"));
        assert!(stats.summary().contains("-50"));
        assert!(stats.summary().contains("75%"));
    }
}
