//! Main polling loop (heartbeat)
//!
//! Every 2 seconds: capture panes → detect agents → update state → refresh status bar.

use tokio::sync::watch;
use std::collections::HashMap;

use crate::agents::{AgentKind, AgentState, DetectorRegistry, TrackedAgent};
use crate::agents::activity::ActivityFeed;
use crate::agents::cost::AgentCostTracker;
use crate::agents::detector;
use crate::agents::identity;
use crate::config;
use crate::tmux::{pane, status};

const SLEEP_TIMEOUT_SECS: u64 = 300; // 5 minutes
const HEARTBEAT_INTERVAL_MS: u64 = 2000;

/// Known agent process names to search for in process trees.
const AGENT_PROCESS_NAMES: &[&str] = &[
    "claude", "codex", "aider", "copilot", "cursor", "continue", "cline", "gpt", "llm",
];

/// Run the heartbeat loop until shutdown.
pub async fn run_heartbeat(mut shutdown: watch::Receiver<bool>) {
    let registry = DetectorRegistry::new();
    let config = config::load();
    let mut tracked: HashMap<String, TrackedAgent> = HashMap::new();
    let mut cycle: u64 = 0;

    // Intelligence layer: cost tracking & activity feed
    let mut cost_tracker = AgentCostTracker::new();
    let mut activity_feed = ActivityFeed::new();
    /// How often (in heartbeat cycles) to run the heavier transcript scan.
    /// With 2s heartbeats, 15 cycles ≈ every 30 seconds.
    const SCAN_INTERVAL: u64 = 15;

    tracing::info!("Heartbeat starting ({}ms interval)", HEARTBEAT_INTERVAL_MS);

    loop {
        if *shutdown.borrow() {
            break;
        }

        if let Err(e) = tick(&registry, &config, &mut tracked, cycle) {
            tracing::warn!("Heartbeat error: {e:#}");
            if cycle <= 3 {
                eprintln!("TermiMon heartbeat error: {e:#}");
            }
        }
        cycle += 1;

        // Periodically scan transcripts for cost & activity data
        if cycle % SCAN_INTERVAL == 1 {
            cost_tracker.scan_all_transcripts();
            activity_feed.scan_transcripts();

            // Persist daily stats (best-effort)
            if let Err(e) = crate::stats::update_from_costs(&cost_tracker, "claude") {
                tracing::debug!("Failed to persist daily stats: {e}");
            }
        }

        // Push state to IPC
        if let Some(state) = super::server::get_global_state() {
            if let Ok(mut st) = state.lock() {
                st.heartbeat_count = cycle;
                st.agents = tracked
                    .values()
                    .map(super::server::AgentSnapshot::from)
                    .collect();
                st.costs = cost_tracker.summary();
                st.recent_activity = activity_feed.recent(20);
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
    cycle: u64,
) -> anyhow::Result<()> {
    let procs = detector::list_processes()?;
    let mut seen: Vec<String> = Vec::new();

    // Strategy 1: Try tmux pane-based detection
    let panes = pane::list_all_panes().unwrap_or_default();
    for info in &panes {
        seen.push(info.pane_id.clone());

        let agent_proc = detector::find_process_in_tree(
            info.pane_pid,
            AGENT_PROCESS_NAMES,
            &procs,
        );

        let detected_kind = agent_proc
            .as_ref()
            .and_then(|p| registry.identify_process(&p.comm));

        if detected_kind.is_none() && !tracked.contains_key(&info.pane_id) {
            continue;
        }

        let content = match pane::capture_pane(&info.pane_id) {
            Ok(c) => c,
            Err(_) => String::new(),
        };

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

        let mut new_state = registry.detect_state(kind, &content);
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

    // Strategy 2: Global process scan (works with cmux, screen, or no multiplexer)
    // Find agent processes running anywhere on the system
    for proc in &procs {
        let comm_lower = proc.comm.to_lowercase();
        let kind = if comm_lower.contains("claude") && !comm_lower.contains("helper") && !comm_lower.contains("crashpad") && !comm_lower.contains("shipit") {
            // Only match the CLI binary, not the Electron app helpers
            if proc.comm.contains(".local/bin/claude") || proc.comm == "claude" {
                Some(AgentKind::Claude)
            } else {
                None
            }
        } else if comm_lower.contains("codex") {
            Some(AgentKind::Codex)
        } else if comm_lower == "aider" || proc.comm.contains("/aider") {
            Some(AgentKind::Aider)
        } else if comm_lower.contains("cursor") && !comm_lower.contains("helper") && !comm_lower.contains("crashpad") {
            Some(AgentKind::Cursor)
        } else if comm_lower.contains("copilot") {
            Some(AgentKind::Copilot)
        } else if comm_lower.contains("continue") || comm_lower.contains("cline") {
            Some(AgentKind::Continue)
        } else {
            None
        };

        if let Some(kind) = kind {
            let key = format!("proc-{}", proc.pid);
            if !seen.contains(&key) {
                seen.push(key.clone());
                let agent = tracked
                    .entry(key)
                    .or_insert_with(|| {
                        tracing::info!("New agent (process scan): {} pid={}", kind, proc.pid);
                        let mut a = TrackedAgent::new(kind, format!("pid-{}", proc.pid));
                        a.pid = Some(proc.pid);
                        a
                    });
                agent.kind = kind;
                agent.pid = Some(proc.pid);

                // Try Claude JSONL transcript for state
                if kind == AgentKind::Claude {
                    let mut state = AgentState::Idle;
                    if let Ok(events) = crate::agents::claude::read_latest_transcript(5) {
                        if let Some(s) = crate::agents::claude::state_from_transcript(&events) {
                            state = s;
                        }
                    }
                    agent.transition(state);
                } else {
                    // Default to idle for detected agents
                    if agent.state == AgentState::Unknown {
                        agent.transition(AgentState::Idle);
                    }
                }
                agent.check_sleep_timeout(SLEEP_TIMEOUT_SECS);
            }
        }
    }

    // Prune: keep pane-based entries if pane still exists, process-based if process alive
    tracked.retain(|id, agent| {
        if id.starts_with("proc-") {
            agent.pid.map(|p| detector::is_process_alive(p)).unwrap_or(false)
        } else {
            seen.contains(id)
        }
    });

    // Post-pass: compute CPU/mem sums, working dir, and agent identity for all tracked agents
    for agent in tracked.values_mut() {
        if let Some(pid) = agent.pid {
            // Sum CPU and memory across all descendant processes
            let descendants = detector::descendant_processes(pid, &procs);
            agent.cpu_pct = descendants.iter().map(|p| p.cpu_pct).sum();
            agent.mem_mb = descendants.iter().map(|p| p.mem_mb).sum();

            // Detect working directory on first discovery (cache it)
            if agent.working_dir.is_none() {
                agent.working_dir = detector::get_working_dir(pid);
            }

            // Compute stable agent identity
            if agent.agent_id.is_empty() {
                let agent_id = identity::compute_agent_id(
                    &agent.kind.to_string(),
                    agent.working_dir.as_deref(),
                );
                agent.agent_id = agent_id.clone();

                // Restore or create creature binding
                let species = crate::creatures::sprites::species_for_agent(&agent.kind.to_string());
                let (binding, is_new) = identity::get_or_create_binding(&agent_id, species);
                if is_new {
                    tracing::info!(
                        agent_id = %agent_id,
                        species = %species,
                        "New creature binding created"
                    );
                } else {
                    tracing::info!(
                        agent_id = %agent_id,
                        xp = binding.xp,
                        stage = binding.stage,
                        sessions = binding.sessions,
                        "Restored creature binding"
                    );
                }
            }
        }
    }

    // Update status bar
    update_status(tracked, config, cycle)?;

    Ok(())
}

fn update_status(
    tracked: &HashMap<String, TrackedAgent>,
    config: &config::Config,
    cycle: u64,
) -> anyhow::Result<()> {
    if tracked.is_empty() {
        status::update_status_right_with_time("🎮")?;
        return Ok(());
    }

    // Build agent snapshots for the animated status bar
    let snapshots: Vec<super::server::AgentSnapshot> = tracked
        .values()
        .filter(|a| a.state != AgentState::Unknown)
        .map(super::server::AgentSnapshot::from)
        .collect();

    if snapshots.is_empty() {
        status::update_status_right_with_time("🎮")?;
        return Ok(());
    }

    // Use animated status bar with cycle as tick counter
    let animated = crate::ui::dashboard::format_status_bar_animated(&snapshots, cycle);
    status::update_status_right_with_time(&animated)?;
    Ok(())
}

// agent_icon moved to ui::dashboard::status_bar_icon for animated display
