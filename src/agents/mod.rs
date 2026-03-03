//! Agent detection and state tracking
//!
//! Detects AI coding agents running in tmux panes and tracks their state
//! through a finite state machine.

pub mod aider;
pub mod claude;
pub mod codex;
pub mod detector;
pub mod generic;

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Instant;

/// The kind of AI agent detected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentKind {
    Claude,
    Codex,
    Aider,
    Generic,
    Unknown,
}

impl fmt::Display for AgentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentKind::Claude => write!(f, "Claude Code"),
            AgentKind::Codex => write!(f, "Codex"),
            AgentKind::Aider => write!(f, "aider"),
            AgentKind::Generic => write!(f, "Agent"),
            AgentKind::Unknown => write!(f, "Unknown"),
        }
    }
}

/// State machine for agent activity.
///
/// Transitions:
///   Unknown → Idle       (agent detected, waiting at prompt)
///   Idle    → Thinking   (agent is processing / generating)
///   Idle    → Typing     (agent is writing to files)
///   Idle    → Reading    (agent is reading files / context)
///   Idle    → Running    (agent is executing a command)
///   *       → Idle       (back to prompt)
///   Idle    → Sleeping   (5 min inactivity timeout)
///   *       → Unknown    (agent process gone)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentState {
    Unknown,
    Idle,
    Thinking,
    Typing,
    Reading,
    Running,
    Sleeping,
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentState::Unknown => write!(f, "unknown"),
            AgentState::Idle => write!(f, "idle"),
            AgentState::Thinking => write!(f, "thinking"),
            AgentState::Typing => write!(f, "typing"),
            AgentState::Reading => write!(f, "reading"),
            AgentState::Running => write!(f, "running"),
            AgentState::Sleeping => write!(f, "sleeping"),
        }
    }
}

impl AgentState {
    /// Map to creature animation state.
    pub fn to_anim_state(&self) -> crate::creatures::AnimState {
        match self {
            AgentState::Unknown => crate::creatures::AnimState::Waiting,
            AgentState::Idle => crate::creatures::AnimState::Idle,
            AgentState::Thinking => crate::creatures::AnimState::Thinking,
            AgentState::Typing => crate::creatures::AnimState::Typing,
            AgentState::Reading => crate::creatures::AnimState::Reading,
            AgentState::Running => crate::creatures::AnimState::Running,
            AgentState::Sleeping => crate::creatures::AnimState::Sleeping,
        }
    }
}

/// Tracked state for an agent running in a specific pane.
#[derive(Debug, Clone)]
pub struct TrackedAgent {
    pub kind: AgentKind,
    pub state: AgentState,
    pub pane_id: String,
    pub pid: Option<u32>,
    pub state_since: Instant,
    pub last_activity: Instant,
}

impl TrackedAgent {
    pub fn new(kind: AgentKind, pane_id: String) -> Self {
        let now = Instant::now();
        Self {
            kind,
            state: AgentState::Unknown,
            pane_id,
            pid: None,
            state_since: now,
            last_activity: now,
        }
    }

    /// Transition to a new state. Returns true if it actually changed.
    pub fn transition(&mut self, new_state: AgentState) -> bool {
        if self.state == new_state {
            return false;
        }
        tracing::debug!(
            agent = %self.kind,
            pane = %self.pane_id,
            from = %self.state,
            to = %new_state,
            "Agent state transition"
        );
        self.state = new_state;
        self.state_since = Instant::now();
        if new_state != AgentState::Sleeping {
            self.last_activity = Instant::now();
        }
        true
    }

    /// Check if agent should go to sleep (5 min idle timeout).
    pub fn check_sleep_timeout(&mut self, timeout_secs: u64) -> bool {
        if self.state == AgentState::Idle
            && self.last_activity.elapsed().as_secs() >= timeout_secs
        {
            self.transition(AgentState::Sleeping)
        } else {
            false
        }
    }

    /// Duration in the current state.
    pub fn state_duration(&self) -> std::time::Duration {
        self.state_since.elapsed()
    }
}

/// Trait that all agent detectors implement.
pub trait AgentDetector: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn matches_process(&self, process_name: &str) -> bool;
    fn detect_state(&self, pane_content: &str) -> Option<AgentState>;
}

/// Registry holding all available detectors.
pub struct DetectorRegistry {
    detectors: Vec<Box<dyn AgentDetector>>,
}

impl DetectorRegistry {
    pub fn new() -> Self {
        Self {
            detectors: vec![
                Box::new(claude::ClaudeDetector),
                Box::new(codex::CodexDetector),
                Box::new(aider::AiderDetector),
                Box::new(generic::GenericDetector),
            ],
        }
    }

    /// Identify which agent kind matches a process name.
    pub fn identify_process(&self, process_name: &str) -> Option<AgentKind> {
        for d in &self.detectors {
            if d.matches_process(process_name) {
                return Some(d.kind());
            }
        }
        None
    }

    /// Detect agent state given kind and pane content.
    pub fn detect_state(&self, kind: AgentKind, pane_content: &str) -> AgentState {
        for d in &self.detectors {
            if d.kind() == kind {
                if let Some(state) = d.detect_state(pane_content) {
                    return state;
                }
            }
        }
        AgentState::Unknown
    }

    /// Try all detectors against pane content (fallback when process didn't match).
    pub fn detect_from_content(&self, pane_content: &str) -> Option<(AgentKind, AgentState)> {
        for d in &self.detectors {
            if let Some(state) = d.detect_state(pane_content) {
                return Some((d.kind(), state));
            }
        }
        None
    }
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
