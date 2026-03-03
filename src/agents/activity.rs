//! Activity feed — a ring-buffer of recent events parsed from agent transcripts.
//!
//! Parses Claude Code JSONL for tool-use events, token usage, errors,
//! and assistant messages to build a human-readable activity timeline.

use std::collections::VecDeque;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::cost;

/// Maximum number of events kept in memory.
const MAX_EVENTS: usize = 500;

/// The type of activity event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    FileRead,
    FileWrite,
    Command,
    Error,
    TokenUsage,
    StateChange,
    Thinking,
    Responding,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::FileRead => write!(f, "read"),
            EventType::FileWrite => write!(f, "write"),
            EventType::Command => write!(f, "cmd"),
            EventType::Error => write!(f, "error"),
            EventType::TokenUsage => write!(f, "tokens"),
            EventType::StateChange => write!(f, "state"),
            EventType::Thinking => write!(f, "thinking"),
            EventType::Responding => write!(f, "response"),
        }
    }
}

/// A single activity event in the feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub timestamp: DateTime<Utc>,
    pub agent_icon: String,
    pub agent_name: String,
    pub message: String,
    pub event_type: EventType,
    #[serde(default)]
    pub project: String,
}

/// Ring-buffer of activity events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityFeed {
    events: VecDeque<ActivityEvent>,
    max_size: usize,
    /// Track the last-seen byte offset per file so we only parse new lines.
    #[serde(skip)]
    file_offsets: std::collections::HashMap<PathBuf, u64>,
}

impl Default for ActivityFeed {
    fn default() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_EVENTS),
            max_size: MAX_EVENTS,
            file_offsets: std::collections::HashMap::new(),
        }
    }
}

impl ActivityFeed {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push an event, evicting the oldest if at capacity.
    pub fn push(&mut self, event: ActivityEvent) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get the most recent `n` events (newest last).
    pub fn recent(&self, n: usize) -> Vec<ActivityEvent> {
        let skip = self.events.len().saturating_sub(n);
        self.events.iter().skip(skip).cloned().collect()
    }

    /// Total events in feed.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Scan Claude JSONL transcript files for new activity events.
    /// Only reads bytes past the last-seen offset per file.
    pub fn scan_transcripts(&mut self) {
        let files = cost::find_transcript_files();
        for file in files {
            self.scan_file(&file);
        }
    }

    fn scan_file(&mut self, path: &PathBuf) {
        let meta = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return,
        };
        let file_len = meta.len();
        let last_offset = self.file_offsets.get(path).copied().unwrap_or(0);
        if file_len <= last_offset {
            return; // no new data
        }

        // Extract project dir from path (parent directory name)
        let project = path.parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        // Only process lines that start after last_offset
        let new_bytes = &content[last_offset as usize..];
        for line in new_bytes.lines() {
            if let Some(mut ev) = parse_activity_line(line) {
                ev.project = project.clone();
                self.push(ev);
            }
        }

        self.file_offsets.insert(path.clone(), file_len);
    }
}

// ─── JSONL → ActivityEvent parsing ───────────────────────────────────────

fn parse_activity_line(line: &str) -> Option<ActivityEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let obj = v.as_object()?;

    let timestamp = obj
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let event_type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

    // Determine agent icon/name from the JSONL
    let agent_icon = "🔥".to_string(); // Claude
    let agent_name = "Claude".to_string();

    match event_type_str {
        // Tool use events
        "tool_use" => {
            let tool_name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let input = obj.get("input").and_then(|v| v.as_object());

            match tool_name {
                "Bash" | "bash" => {
                    let cmd = input
                        .and_then(|i| i.get("command"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("...");
                    let short_cmd = truncate_str(cmd, 60);
                    Some(ActivityEvent {
                        timestamp,
                        agent_icon,
                        agent_name,
                        message: format!("ran `{short_cmd}`"),
                        project: String::new(),
                    event_type: EventType::Command,
                    })
                }
                "Write" | "write" | "write_file" => {
                    let file = input
                        .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                        .and_then(|v| v.as_str())
                        .unwrap_or("file");
                    let short = short_path(file);
                    Some(ActivityEvent {
                        timestamp,
                        agent_icon,
                        agent_name,
                        message: format!("wrote {short}"),
                        project: String::new(),
                    event_type: EventType::FileWrite,
                    })
                }
                "Edit" | "edit" | "edit_file" => {
                    let file = input
                        .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                        .and_then(|v| v.as_str())
                        .unwrap_or("file");
                    let short = short_path(file);
                    Some(ActivityEvent {
                        timestamp,
                        agent_icon,
                        agent_name,
                        message: format!("edited {short}"),
                        project: String::new(),
                    event_type: EventType::FileWrite,
                    })
                }
                "Read" | "read" | "read_file" => {
                    let file = input
                        .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                        .and_then(|v| v.as_str())
                        .unwrap_or("file");
                    let short = short_path(file);
                    Some(ActivityEvent {
                        timestamp,
                        agent_icon,
                        agent_name,
                        message: format!("read {short}"),
                        project: String::new(),
                    event_type: EventType::FileRead,
                    })
                }
                _ => {
                    Some(ActivityEvent {
                        timestamp,
                        agent_icon,
                        agent_name,
                        message: format!("used tool: {tool_name}"),
                        project: String::new(),
                    event_type: EventType::Command,
                    })
                }
            }
        }

        // Assistant response messages — may contain tool_use blocks in content
        "assistant" => {
            // First, check for tool_use events inside message.content[]
            let content_blocks = obj.get("message")
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_array());

            let mut tool_events: Vec<ActivityEvent> = Vec::new();
            if let Some(blocks) = content_blocks {
                for block in blocks {
                    let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    if block_type == "tool_use" {
                        let tool_name = block.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let input = block.get("input").and_then(|v| v.as_object());
                        let evt = match tool_name {
                            "Bash" | "bash" => {
                                let cmd = input
                                    .and_then(|i| i.get("command"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("...");
                                let short_cmd = truncate_str(cmd, 60);
                                Some(ActivityEvent {
                                    timestamp,
                                    agent_icon: agent_icon.clone(),
                                    agent_name: agent_name.clone(),
                                    message: format!("ran `{short_cmd}`"),
                                    project: String::new(),
                    event_type: EventType::Command,
                                })
                            }
                            "Write" | "write" | "write_file" => {
                                let file = input
                                    .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("file");
                                let short = short_path(file);
                                Some(ActivityEvent {
                                    timestamp,
                                    agent_icon: agent_icon.clone(),
                                    agent_name: agent_name.clone(),
                                    message: format!("wrote {short}"),
                                    project: String::new(),
                    event_type: EventType::FileWrite,
                                })
                            }
                            "Edit" | "edit" | "edit_file" => {
                                let file = input
                                    .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("file");
                                let short = short_path(file);
                                Some(ActivityEvent {
                                    timestamp,
                                    agent_icon: agent_icon.clone(),
                                    agent_name: agent_name.clone(),
                                    message: format!("edited {short}"),
                                    project: String::new(),
                    event_type: EventType::FileWrite,
                                })
                            }
                            "Read" | "read" | "read_file" => {
                                let file = input
                                    .and_then(|i| i.get("file_path").or_else(|| i.get("path")))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("file");
                                let short = short_path(file);
                                Some(ActivityEvent {
                                    timestamp,
                                    agent_icon: agent_icon.clone(),
                                    agent_name: agent_name.clone(),
                                    message: format!("read {short}"),
                                    project: String::new(),
                    event_type: EventType::FileRead,
                                })
                            }
                            _ if !tool_name.is_empty() => {
                                Some(ActivityEvent {
                                    timestamp,
                                    agent_icon: agent_icon.clone(),
                                    agent_name: agent_name.clone(),
                                    message: format!("used tool: {tool_name}"),
                                    project: String::new(),
                    event_type: EventType::Command,
                                })
                            }
                            _ => None,
                        };
                        if let Some(e) = evt {
                            tool_events.push(e);
                        }
                    }
                }
            }

            // If we found tool_use events, return the last one (most interesting)
            if let Some(last_tool) = tool_events.into_iter().last() {
                return Some(last_tool);
            }

            // Otherwise check for usage/token info
            let usage = obj.get("message")
                .and_then(|m| m.get("usage"))
                .and_then(|u| u.as_object());
            if let Some(usage) = usage {
                let input_tok = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let output_tok = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let total = input_tok + output_tok;
                if total > 0 {
                    let model = obj.get("message")
                        .and_then(|m| m.get("model"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let cost_cents = cost::compute_cost_cents(input_tok, output_tok, model);
                    Some(ActivityEvent {
                        timestamp,
                        agent_icon,
                        agent_name,
                        message: format!(
                            "used {} tokens ({})",
                            cost::format_tokens(total),
                            cost::format_cost(cost_cents),
                        ),
                        project: String::new(),
                    event_type: EventType::TokenUsage,
                    })
                } else {
                    None // Skip zero-token usage events
                }
            } else {
                Some(ActivityEvent {
                    timestamp,
                    agent_icon,
                    agent_name,
                    message: "responding...".to_string(),
                    project: String::new(),
                    event_type: EventType::Responding,
                })
            }
        }

        // Thinking
        "thinking" => {
            Some(ActivityEvent {
                timestamp,
                agent_icon,
                agent_name,
                message: "thinking...".to_string(),
                project: String::new(),
                    event_type: EventType::Thinking,
            })
        }

        // Error events
        "error" => {
            let msg = obj
                .get("error")
                .or_else(|| obj.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            let short = truncate_str(msg, 80);
            Some(ActivityEvent {
                timestamp,
                agent_icon,
                agent_name,
                message: format!("error: {short}"),
                project: String::new(),
                    event_type: EventType::Error,
            })
        }

        // Result events (tool results)
        "tool_result" | "result" => {
            // Check for errors in results
            let is_error = obj.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
            if is_error {
                let content = obj.get("content").and_then(|v| v.as_str()).unwrap_or("error");
                let short = truncate_str(content, 80);
                Some(ActivityEvent {
                    timestamp,
                    agent_icon,
                    agent_name,
                    message: format!("error: {short}"),
                    project: String::new(),
                    event_type: EventType::Error,
                })
            } else {
                None // Skip non-error results to reduce noise
            }
        }

        // Human/prompt events → state change
        "human" | "prompt" => {
            Some(ActivityEvent {
                timestamp,
                agent_icon,
                agent_name,
                message: "waiting for input...".to_string(),
                project: String::new(),
                    event_type: EventType::StateChange,
            })
        }

        _ => None,
    }
}

/// Truncate a string to max length, appending "…" if truncated.
fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        // Find the nearest char boundary at or before `max`
        let mut end = max;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

/// Shorten a file path to just the last 2 components.
fn short_path(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 2 {
        path.to_string()
    } else {
        parts[parts.len() - 2..].join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_eviction() {
        let mut feed = ActivityFeed {
            events: VecDeque::new(),
            max_size: 3,
            file_offsets: std::collections::HashMap::new(),
        };
        for i in 0..5 {
            feed.push(ActivityEvent {
                timestamp: Utc::now(),
                agent_icon: "🔥".into(),
                agent_name: "test".into(),
                message: format!("event {i}"),
                project: String::new(),
                    event_type: EventType::Command,
            });
        }
        assert_eq!(feed.len(), 3);
        assert_eq!(feed.events[0].message, "event 2");
    }

    #[test]
    fn test_recent() {
        let mut feed = ActivityFeed::new();
        for i in 0..10 {
            feed.push(ActivityEvent {
                timestamp: Utc::now(),
                agent_icon: "🔥".into(),
                agent_name: "test".into(),
                message: format!("event {i}"),
                project: String::new(),
                    event_type: EventType::Command,
            });
        }
        let recent = feed.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[2].message, "event 9");
    }

    #[test]
    fn test_parse_tool_use_bash() {
        let line = r#"{"type":"tool_use","name":"Bash","input":{"command":"cargo build"},"timestamp":"2026-03-02T10:00:00Z"}"#;
        let ev = parse_activity_line(line).unwrap();
        assert_eq!(ev.event_type, EventType::Command);
        assert!(ev.message.contains("cargo build"));
    }

    #[test]
    fn test_parse_error() {
        let line = r#"{"type":"error","error":"Rate limit exceeded","timestamp":"2026-03-02T10:00:00Z"}"#;
        let ev = parse_activity_line(line).unwrap();
        assert_eq!(ev.event_type, EventType::Error);
        assert!(ev.message.contains("Rate limit"));
    }

    #[test]
    fn test_short_path() {
        assert_eq!(short_path("src/main.rs"), "src/main.rs");
        assert_eq!(short_path("/home/user/project/src/main.rs"), "src/main.rs");
    }
}
