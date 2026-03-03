//! Notification system for key events
//!
//! Sends notifications via:
//! - Terminal bell (\x07)
//! - tmux display-message (if available)
//! - System notification (optional)

use crate::config::Config;

/// Notification event types
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum NotifyEvent {
    /// Creature evolved to a new stage
    Evolution {
        creature_name: String,
        new_stage: u8,
        xp: u64,
    },
    /// Agent process disappeared
    AgentDied {
        agent_kind: String,
        agent_id: String,
    },
    /// Daily cost threshold exceeded
    CostThreshold {
        current_cents: u64,
        threshold_cents: u64,
    },
}

impl NotifyEvent {
    fn message(&self) -> String {
        match self {
            NotifyEvent::Evolution { creature_name, new_stage, xp } => {
                format!("🎉 {} evolved to Stage {}! ({} XP)", creature_name, new_stage, xp)
            }
            NotifyEvent::AgentDied { agent_kind, agent_id } => {
                format!("💀 {} ({}) has stopped", agent_kind, &agent_id[..8.min(agent_id.len())])
            }
            NotifyEvent::CostThreshold { current_cents, threshold_cents } => {
                format!(
                    "💰 Cost alert: {} (threshold: {})",
                    crate::agents::cost::format_cost(*current_cents),
                    crate::agents::cost::format_cost(*threshold_cents),
                )
            }
        }
    }
}

/// Send a notification through all configured channels.
pub fn send_notification(event: &NotifyEvent, config: &Config) {
    let msg = event.message();
    tracing::info!("Notification: {}", msg);

    // Terminal bell
    if config.notifications.terminal_bell {
        eprint!("\x07");
    }

    // tmux display-message (best effort)
    let _ = std::process::Command::new(crate::tmux::find_tmux())
        .args(["display-message", &format!("TermiMon: {}", msg)])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // macOS system notification (best effort)
    if config.notifications.system_notify {
        let _ = std::process::Command::new("osascript")
            .args([
                "-e",
                &format!(
                    "display notification \"{}\" with title \"TermiMon\"",
                    msg.replace('"', "\\\"")
                ),
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}
