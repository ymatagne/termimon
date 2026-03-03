//! aider detection

use super::{AgentDetector, AgentKind, AgentState};

pub struct AiderDetector;

impl AiderDetector {
    const PROCESS_NAMES: &[&str] = &["aider"];
    const PROMPT_PATTERNS: &[&str] = &["aider>", "aider ❯"];
    const THINKING_PATTERNS: &[&str] = &["Thinking...", "Sending request..."];
    const TYPING_PATTERNS: &[&str] = &["Applied edit to", "Wrote ", "Editing "];
    const RUNNING_PATTERNS: &[&str] = &["Running ", "/run "];
    const READING_PATTERNS: &[&str] = &["Added ", "Scanning repo", "Repo-map:"];
}

impl AgentDetector for AiderDetector {
    fn kind(&self) -> AgentKind {
        AgentKind::Aider
    }

    fn matches_process(&self, process_name: &str) -> bool {
        let lower = process_name.to_lowercase();
        Self::PROCESS_NAMES.iter().any(|p| lower.contains(p))
    }

    fn detect_state(&self, pane_content: &str) -> Option<AgentState> {
        let tail: String = pane_content
            .lines()
            .rev()
            .take(20)
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
        if has_any(&tail, Self::READING_PATTERNS) {
            return Some(AgentState::Reading);
        }
        if has_any(&tail, Self::THINKING_PATTERNS) {
            return Some(AgentState::Thinking);
        }
        if super::detector::is_at_prompt(&tail, Self::PROMPT_PATTERNS) {
            return Some(AgentState::Idle);
        }
        None
    }
}

fn has_any(text: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| text.contains(p))
}
