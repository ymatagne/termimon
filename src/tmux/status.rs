//! tmux status bar formatting and updates

use anyhow::Result;

/// A single creature entry to display in the status bar.
#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub icon: String,
    pub name: String,
    pub state: String,
    pub state_emoji: String,
}

impl StatusEntry {
    /// Format using a template string.
    /// Placeholders: {icon}, {name}, {state}, {state_emoji}
    pub fn format(&self, template: &str) -> String {
        template
            .replace("{icon}", &self.icon)
            .replace("{name}", &self.name)
            .replace("{state}", &self.state)
            .replace("{state_emoji}", &self.state_emoji)
    }
}

/// Format a list of status entries into a single status-right string.
pub fn format_status_bar(entries: &[StatusEntry], template: &str, max_entries: usize) -> String {
    if entries.is_empty() {
        return "🎮 TermiMon".to_string();
    }
    entries
        .iter()
        .take(max_entries)
        .map(|e| e.format(template))
        .collect::<Vec<_>>()
        .join(" │ ")
}

/// Push an update to the tmux status bar (status-right).
pub fn update_status_right(content: &str) -> Result<()> {
    let tagged = format!("#[fg=colour39]{}#[default]", content);
    super::run_tmux_command(&["set", "-g", "status-right", &tagged])?;
    Ok(())
}

/// Update status-right with a clock appended.
pub fn update_status_right_with_time(content: &str) -> Result<()> {
    let tagged = format!(
        "#[fg=colour39]{}#[default] #[fg=colour244]%H:%M#[default]",
        content
    );
    super::run_tmux_command(&["set", "-g", "status-right", &tagged])?;
    Ok(())
}

/// Get current status-right value.
pub fn get_status_right() -> Result<String> {
    let output = super::run_tmux_command(&["display-message", "-p", "#{status-right}"])?;
    Ok(output.trim().to_string())
}

/// Set the status bar refresh interval (seconds).
pub fn set_status_interval(seconds: u32) -> Result<()> {
    let s = seconds.to_string();
    super::run_tmux_command(&["set", "-g", "status-interval", &s])?;
    Ok(())
}

/// Clear TermiMon's status bar contribution.
pub fn clear_status() -> Result<()> {
    super::run_tmux_command(&["set", "-g", "status-right", "%H:%M %d-%b"])?;
    Ok(())
}
