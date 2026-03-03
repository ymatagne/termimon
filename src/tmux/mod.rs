//! tmux integration layer
//!
//! Provides control mode connections, pane discovery, content capture,
//! and status bar management for the TermiMon daemon.

pub mod control;
pub mod pane;
pub mod status;

use anyhow::Result;

/// Check whether we are running inside a tmux session.
pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok()
}

/// Alias for compatibility.
pub fn is_tmux() -> bool {
    is_inside_tmux()
}

/// Return the current tmux session name, or None if not in tmux.
pub fn current_session() -> Option<String> {
    let output = std::process::Command::new("tmux")
        .args(["display-message", "-p", "#{session_name}"])
        .output()
        .ok()?;
    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    } else {
        None
    }
}

/// Run a single tmux command synchronously and return stdout.
pub fn run_tmux_command(args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("tmux")
        .args(args)
        .output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        anyhow::bail!("tmux command failed: {stderr}");
    }
}
