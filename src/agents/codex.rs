//! Codex CLI detection

use super::{AgentDetector, AgentKind, AgentState};

pub struct CodexDetector;

impl CodexDetector {
    const PROCESS_NAMES: &[&str] = &["codex"];
    const PROMPT_PATTERNS: &[&str] = &["codex>", "codex ❯"];
    const THINKING_PATTERNS: &[&str] = &["Thinking...", "Generating...", "Planning..."];
    const TYPING_PATTERNS: &[&str] = &["Writing", "Patching", "Creating"];
    const RUNNING_PATTERNS: &[&str] = &["Running:", "Executing:", "$ "];
}

impl AgentDetector for CodexDetector {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
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
