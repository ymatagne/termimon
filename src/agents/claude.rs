//! Claude Code detection
//!
//! Detects Claude Code by process name, terminal output patterns,
//! and JSONL transcripts in ~/.claude/.

use super::{AgentDetector, AgentKind, AgentState};
use anyhow::Result;
use std::path::PathBuf;

pub struct ClaudeDetector;

impl ClaudeDetector {
    const PROCESS_NAMES: &[&str] = &["claude"];
    const PROMPT_PATTERNS: &[&str] = &["claude ❯", "claude>", "❯"];
    const THINKING_PATTERNS: &[&str] = &[
        "Thinking...", "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
    ];
    const TYPING_PATTERNS: &[&str] = &[
        "Writing to file...", "Editing file...", "Creating file...", "Write(", "Edit(",
    ];
    const READING_PATTERNS: &[&str] = &[
        "Reading file...", "Read(", "Searching...", "Indexing...",
    ];
    const RUNNING_PATTERNS: &[&str] = &["Bash(", "Running...", "Executing..."];
}

impl AgentDetector for ClaudeDetector {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn matches_process(&self, process_name: &str) -> bool {
        let lower = process_name.to_lowercase();
        Self::PROCESS_NAMES.iter().any(|p| lower.contains(p))
    }

    fn detect_state(&self, pane_content: &str) -> Option<AgentState> {
        let tail: String = pane_content
            .lines()
            .rev()
            .take(30)
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

// ─── JSONL transcript support ──────────────────────────────────────────

/// Path to Claude Code's transcript directory.
pub fn claude_transcript_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

/// A parsed event from a Claude Code JSONL transcript.
#[derive(Debug, Clone)]
pub struct TranscriptEvent {
    pub timestamp: Option<String>,
    pub event_type: String,
    pub message: Option<String>,
}

/// Read the most recent JSONL transcript and extract the last few events.
pub fn read_latest_transcript(max_events: usize) -> Result<Vec<TranscriptEvent>> {
    let dir = claude_transcript_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jsonl")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });

    let latest = match entries.last() {
        Some(e) => e.path(),
        None => return Ok(Vec::new()),
    };

    let content = std::fs::read_to_string(&latest)?;
    let events: Vec<TranscriptEvent> = content
        .lines()
        .rev()
        .take(max_events)
        .filter_map(parse_transcript_line)
        .collect();

    Ok(events)
}

fn parse_transcript_line(line: &str) -> Option<TranscriptEvent> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = value.as_object()?;
    Some(TranscriptEvent {
        timestamp: obj.get("timestamp").and_then(|v| v.as_str()).map(String::from),
        event_type: obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        message: obj.get("message").and_then(|v| v.as_str()).map(String::from),
    })
}

/// Infer agent state from the latest transcript events.
pub fn state_from_transcript(events: &[TranscriptEvent]) -> Option<AgentState> {
    for event in events {
        match event.event_type.as_str() {
            "tool_use" | "bash" => return Some(AgentState::Running),
            "write_file" | "edit_file" => return Some(AgentState::Typing),
            "read_file" | "search" => return Some(AgentState::Reading),
            "assistant" | "thinking" => return Some(AgentState::Thinking),
            "human" | "prompt" => return Some(AgentState::Idle),
            _ => continue,
        }
    }
    None
}
