//! Generic / fallback agent detector using heuristics.

use super::{AgentDetector, AgentKind, AgentState};

pub struct GenericDetector;

impl GenericDetector {
    const PROCESS_NAMES: &[&str] = &[
        "copilot", "cursor", "continue", "gpt", "llm", "openai", "anthropic",
    ];
    const THINKING_PATTERNS: &[&str] = &[
        "Thinking...", "Generating...", "Processing...",
        "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
    ];
    const TYPING_PATTERNS: &[&str] = &[
        "Writing to", "Saving ", "Creating file", "Editing file",
    ];
    const RUNNING_PATTERNS: &[&str] = &["Running command", "Executing", "$ "];
}

impl AgentDetector for GenericDetector {
    fn kind(&self) -> AgentKind {
        AgentKind::Generic
    }

    fn matches_process(&self, process_name: &str) -> bool {
        let lower = process_name.to_lowercase();
        Self::PROCESS_NAMES.iter().any(|p| lower.contains(p))
    }

    fn detect_state(&self, pane_content: &str) -> Option<AgentState> {
        let tail: String = pane_content
            .lines()
            .rev()
            .take(15)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");

        if tail.is_empty() {
            return None;
        }

        if has_any(&tail, Self::RUNNING_PATTERNS) {
            return Some(AgentState::Running);
        }
        if has_any(&tail, Self::TYPING_PATTERNS) {
            return Some(AgentState::Typing);
        }
        // Only match thinking in the very last 5 lines
        let recent: String = pane_content
            .lines()
            .rev()
            .take(5)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        if has_any(&recent, Self::THINKING_PATTERNS) {
            return Some(AgentState::Thinking);
        }
        None
    }
}

fn has_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}
