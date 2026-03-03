//! Tmux hotkey binding management
//!
//! Adds/removes a tmux key binding (prefix+P) to toggle the TermiMon dashboard.

use anyhow::{Context, Result};

const BIND_COMMENT: &str = "# TermiMon dashboard hotkey";
const BIND_KEY: &str = "P";

/// Get the path to the termimon binary.
fn termimon_bin() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "termimon".to_string())
}

/// Add tmux key binding (prefix+P) to toggle the dashboard.
pub fn bind_hotkey() -> Result<()> {
    let bin = termimon_bin();

    // Try live tmux binding first
    let tmux_available = std::process::Command::new("tmux")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if tmux_available {
        let popup_cmd = format!(
            "display-popup -E -w 80% -h 80% '{} dash'",
            bin
        );
        let status = std::process::Command::new("tmux")
            .args(["bind-key", BIND_KEY, &popup_cmd])
            .status()
            .context("Failed to run tmux bind-key")?;

        if status.success() {
            println!("✅ Bound prefix+{BIND_KEY} to TermiMon dashboard (live)");
        }
    }

    // Also persist to ~/.tmux.conf
    let tmux_conf = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".tmux.conf");

    let existing = std::fs::read_to_string(&tmux_conf).unwrap_or_default();

    // Check if already bound
    if existing.contains("termimon dash") {
        println!("📋 Binding already exists in ~/.tmux.conf");
        return Ok(());
    }

    let bind_line = format!(
        "\n{BIND_COMMENT}\nbind-key {BIND_KEY} display-popup -E -w 80% -h 80% \"{bin} dash\"\n"
    );

    let mut content = existing;
    content.push_str(&bind_line);
    std::fs::write(&tmux_conf, content)
        .context("Failed to write ~/.tmux.conf")?;

    println!("📝 Added binding to ~/.tmux.conf");
    println!("   Use prefix+{BIND_KEY} to open the TermiMon dashboard");

    Ok(())
}

/// Remove tmux key binding.
pub fn unbind_hotkey() -> Result<()> {
    // Unbind live
    let _ = std::process::Command::new("tmux")
        .args(["unbind-key", BIND_KEY])
        .status();

    // Remove from ~/.tmux.conf
    let tmux_conf = dirs::home_dir()
        .context("Could not find home directory")?
        .join(".tmux.conf");

    if tmux_conf.exists() {
        let content = std::fs::read_to_string(&tmux_conf)?;
        let filtered: Vec<&str> = content
            .lines()
            .filter(|line| {
                !line.contains("termimon dash") && line.trim() != BIND_COMMENT
            })
            .collect();
        std::fs::write(&tmux_conf, filtered.join("\n") + "\n")?;
        println!("🗑  Removed TermiMon binding from ~/.tmux.conf");
    }

    println!("✅ Unbound prefix+{BIND_KEY}");
    Ok(())
}
