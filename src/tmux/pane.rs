//! Pane discovery and content capture
//!
//! Uses `tmux list-panes` for discovery and `tmux capture-pane` for reading
//! visible content from panes.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Metadata about a single tmux pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneInfo {
    /// Pane ID (e.g. "%0", "%3")
    pub pane_id: String,
    /// Session name
    pub session: String,
    /// Window index
    pub window_index: u32,
    /// Window name
    pub window_name: String,
    /// Pane index within the window
    pub pane_index: u32,
    /// Width in columns
    pub width: u32,
    /// Height in rows
    pub height: u32,
    /// PID of the shell in this pane
    pub pane_pid: u32,
    /// Whether this pane is currently active
    pub active: bool,
    /// The command running in the pane
    pub current_command: String,
}

/// List all panes across all sessions.
pub fn list_all_panes() -> Result<Vec<PaneInfo>> {
    let format = "#{session_name}\t#{window_index}\t#{window_name}\t#{pane_index}\t#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_pid}\t#{pane_active}\t#{pane_current_command}";
    let output = super::run_tmux_command(&["list-panes", "-a", "-F", format])?;
    parse_pane_list(&output)
}

/// List panes in a specific session.
pub fn list_session_panes(session: &str) -> Result<Vec<PaneInfo>> {
    let format = "#{session_name}\t#{window_index}\t#{window_name}\t#{pane_index}\t#{pane_id}\t#{pane_width}\t#{pane_height}\t#{pane_pid}\t#{pane_active}\t#{pane_current_command}";
    let output = super::run_tmux_command(&[
        "list-panes", "-s", "-t", session, "-F", format,
    ])?;
    parse_pane_list(&output)
}

fn parse_pane_list(raw: &str) -> Result<Vec<PaneInfo>> {
    let mut panes = Vec::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 10 {
            tracing::warn!("Skipping malformed pane line: {line}");
            continue;
        }
        panes.push(PaneInfo {
            session: parts[0].to_string(),
            window_index: parts[1].parse().unwrap_or(0),
            window_name: parts[2].to_string(),
            pane_index: parts[3].parse().unwrap_or(0),
            pane_id: parts[4].to_string(),
            width: parts[5].parse().unwrap_or(0),
            height: parts[6].parse().unwrap_or(0),
            pane_pid: parts[7].parse().unwrap_or(0),
            active: parts[8] == "1",
            current_command: parts[9].to_string(),
        });
    }
    Ok(panes)
}

/// Capture the visible contents of a pane.
pub fn capture_pane(pane_id: &str) -> Result<String> {
    super::run_tmux_command(&["capture-pane", "-p", "-t", pane_id, "-J"])
        .with_context(|| format!("Failed to capture pane {pane_id}"))
}

/// Capture the last N lines of a pane (including scrollback).
pub fn capture_pane_tail(pane_id: &str, lines: u32) -> Result<String> {
    let start = format!("-{lines}");
    super::run_tmux_command(&["capture-pane", "-p", "-t", pane_id, "-J", "-S", &start])
        .with_context(|| format!("Failed to capture pane {pane_id} scrollback"))
}

/// Get the PID of the foreground process in a pane.
pub fn pane_foreground_pid(pane_id: &str) -> Result<u32> {
    let output = super::run_tmux_command(&[
        "display-message", "-t", pane_id, "-p", "#{pane_pid}",
    ])?;
    output.trim().parse().context("Failed to parse pane PID")
}
