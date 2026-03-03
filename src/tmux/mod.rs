//! tmux integration layer
//!
//! Provides control mode connections, pane discovery, content capture,
//! and status bar management for the TermiMon daemon.

pub mod bind;
pub mod control;
pub mod pane;
pub mod status;

use anyhow::{Context, Result};

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
    let output = std::process::Command::new(find_tmux())
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

/// Find the tmux binary path.
pub fn find_tmux() -> &'static str {
    static TMUX_PATH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    TMUX_PATH.get_or_init(|| {
        // Check common paths
        for path in &[
            "/opt/homebrew/bin/tmux",
            "/usr/local/bin/tmux",
            "/usr/bin/tmux",
            "tmux",
        ] {
            if let Ok(output) = std::process::Command::new(path).arg("-V").output() {
                if output.status.success() {
                    return path.to_string();
                }
            }
        }
        "tmux".to_string()
    })
}

/// Run a single tmux command synchronously and return stdout.
pub fn run_tmux_command(args: &[&str]) -> Result<String> {
    let tmux = find_tmux();
    let output = std::process::Command::new(tmux)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run tmux ({tmux}) with args: {args:?}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        anyhow::bail!("tmux command `{tmux} {args:?}` failed: {stderr}");
    }
}
