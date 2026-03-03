//! Main polling loop (heartbeat)
//!
//! Every 2 seconds: capture panes → detect agents → update state → refresh status bar.

use tokio::sync::watch;
use std::collections::HashMap;

use crate::agents::{AgentKind, AgentState, DetectorRegistry, TrackedAgent};
use crate::agents::detector;
use crate::config;
use crate::tmux::{pane, status};

const SLEEP_TIMEOUT_SECS: u64 = 300; // 5 minutes
const HEARTBEAT_INTERVAL_MS: u64 = 2000;

/// Known agent process names to search for in process trees.
const AGENT_PROCESS_NAMES: &[&str] = &[
    "claude", "codex", "aider", "copilot", "cursor", "gpt", "llm",
];

/// Run the heartbeat loop until shutdown.
pub async fn run_heartbeat(mut shutdown: watch::Receiver<bool>) {
    let registry = DetectorRegistry::new();
    let config = config::load();
    let mut tracked: HashMap<String, TrackedAgent> = HashMap::new();
    let mut cycle: u64 = 0;

    tracing::info!("Heartbeat starting ({}ms interval)", HEARTBEAT_INTERVAL_MS);

    loop {
        if *shutdown.borrow() {
            break;
        }

        if let Err(e) = tick(&registry, &config, &mut tracked) {
            tracing::warn!("Heartbeat error: {e}");
        }
        cycle += 1;

        // Push state to IPC
        if let Some(state) = super::server::get_global_state() {
            if let Ok(mut st) = state.lock() {
                st.heartbeat_count = cycle;
                st.agents = tracked
                    .values()
                    .map(super::server::AgentSnapshot::from)
                    .collect();
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_millis(HEARTBEAT_INTERVAL_MS)) => {}
            _ = shutdown.changed() => {
                if *shutdown.borrow() { break; }
            }
        }
    }

    tracing::info!("Heartbeat stopped after {cycle} cycles");
}

/// One tick of the heartbeat.
fn tick(
    registry: &DetectorRegistry,
    config: &config::Config,
    tracked: &mut HashMap<String, TrackedAgent>,
) -> anyhow::Result<()> {
    let panes = pane::list_all_panes()?;
    let procs = detector::list_processes()?;
    let mut seen: Vec<String> = Vec::new();

    for info in &panes {
        seen.push(info.pane_id.clone());

        // Walk process tree from pane PID
        let agent_proc = detector::find_process_in_tree(
            info.pane_pid,
            AGENT_PROCESS_NAMES,
            &procs,
        );

        let detected_kind = agent_proc
            .as_ref()
            .and_then(|p| registry.identify_process(&p.comm));

        // Skip panes with no agent (unless already tracked)
        if detected_kind.is_none() && !tracked.contains_key(&info.pane_id) {
            continue;
        }

        // Capture pane content
        let content = match pane::capture_pane(&info.pane_id) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("Capture failed for {}: {e}", info.pane_id);
                continue;
            }
        };

        // Determine kind
        let kind = detected_kind.unwrap_or_else(|| {
            registry
                .detect_from_content(&content)
                .map(|(k, _)| k)
                .unwrap_or(AgentKind::Unknown)
        });

        if kind == AgentKind::Unknown {
            tracked.remove(&info.pane_id);
            continue;
        }

        // Get or create tracking entry
        let agent = tracked
            .entry(info.pane_id.clone())
            .or_insert_with(|| {
                tracing::info!("New agent: {} in pane {}", kind, info.pane_id);
                let mut a = TrackedAgent::new(kind, info.pane_id.clone());
                a.pid = agent_proc.as_ref().map(|p| p.pid);
                a
            });

        agent.kind = kind;
        agent.pid = agent_proc.as_ref().map(|p| p.pid);

        // Detect state from content
        let mut new_state = registry.detect_state(kind, &content);

        // Claude fallback: check JSONL transcripts
        if new_state == AgentState::Unknown && kind == AgentKind::Claude {
            if let Ok(events) = crate::agents::claude::read_latest_transcript(5) {
                if let Some(s) = crate::agents::claude::state_from_transcript(&events) {
                    new_state = s;
                }
            }
        }

        if new_state != AgentState::Unknown {
            agent.transition(new_state);
        }

        agent.check_sleep_timeout(SLEEP_TIMEOUT_SECS);
    }

    // Prune stale entries
    tracked.retain(|id, _| seen.contains(id));

    // Update status bar
    update_status(tracked, config)?;

    Ok(())
}

fn update_status(
    tracked: &HashMap<String, TrackedAgent>,
    config: &config::Config,
) -> anyhow::Result<()> {
    if tracked.is_empty() {
        status::update_status_right_with_time("🎮")?;
        return Ok(());
    }

    let entries: Vec<status::StatusEntry> = tracked
        .values()
        .filter(|a| a.state != AgentState::Unknown)
        .map(|a| {
            let anim = a.state.to_anim_state();
            status::StatusEntry {
                icon: agent_icon(a.kind),
                name: a.kind.to_string(),
                state: a.state.to_string(),
                state_emoji: anim.emoji().to_string(),
            }
        })
        .collect();

    let formatted = status::format_status_bar(
        &entries,
        &config.statusbar.format,
        config.statusbar.max_creatures,
    );
    status::update_status_right_with_time(&formatted)?;
    Ok(())
}

fn agent_icon(kind: AgentKind) -> String {
    match kind {
        AgentKind::Claude => "🔥".to_string(),
        AgentKind::Codex => "⚡".to_string(),
        AgentKind::Aider => "💧".to_string(),
        AgentKind::Generic => "🤖".to_string(),
        AgentKind::Unknown => "❓".to_string(),
    }
}
