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
    let cfg = config::load();
    let mut tracked: HashMap<String, TrackedAgent> = HashMap::new();
    let mut cycle: u64 = 0;
    let mut last_cost_alert_cents: u64 = 0;
    let cost_threshold_cents: u64 = cfg.notifications.cost_alert_threshold_cents;

    // Intelligence layer: cost tracking, activity feed, productivity
    let mut cost_tracker = AgentCostTracker::new();
    let mut activity_feed = ActivityFeed::new();
    let mut productivity_tracker = crate::agents::productivity::ProductivityTracker::new();
    /// How often (in heartbeat cycles) to run the heavier transcript scan.
    /// With 2s heartbeats, 15 cycles ≈ every 30 seconds.
    const SCAN_INTERVAL: u64 = 15;

    tracing::info!("Heartbeat starting ({}ms interval)", HEARTBEAT_INTERVAL_MS);

    loop {
        if *shutdown.borrow() {
            break;
        }

        if let Err(e) = tick(&registry, &cfg, &mut tracked, cycle) {
            tracing::warn!("Heartbeat error: {e:#}");
            if cycle <= 3 {
                eprintln!("TermiMon heartbeat error: {e:#}");
            }
        }
        cycle += 1;

        // Periodically scan transcripts for cost & activity data
        if cycle % SCAN_INTERVAL == 1 {
            // Build encoded_project_dir → agent_id mapping so costs are keyed correctly
            // Claude encodes working dirs like /Users/yan/foo → -Users-yan-foo
            let workdir_to_agent_id: HashMap<String, String> = tracked
                .values()
                .filter(|a| !a.agent_id.is_empty())
                .filter_map(|a| {
                    a.working_dir.as_ref().map(|wd| {
                        let encoded = crate::agents::cost::encode_working_dir(wd);
                        tracing::debug!(wd = %wd, encoded = %encoded, agent_id = %a.agent_id, "Cost mapping");
                        (encoded, a.agent_id.clone())
                    })
                })
                .collect();
            cost_tracker.scan_all_transcripts(&workdir_to_agent_id);
            activity_feed.scan_transcripts();

            // Persist daily stats (best-effort)
            if let Err(e) = crate::stats::update_from_costs(&cost_tracker, "claude") {
                tracing::debug!("Failed to persist daily stats: {e}");
            }

            // Award XP from recent activity events
            award_xp_from_activity(&activity_feed, &tracked, &cfg);

            // Check cost threshold
            if let Some(total) = cost_tracker.total_summary() {
                if total.cost_cents >= cost_threshold_cents && last_cost_alert_cents < cost_threshold_cents {
                    super::notify::send_notification(
                        &super::notify::NotifyEvent::CostThreshold {
                            current_cents: total.cost_cents,
                            threshold_cents: cost_threshold_cents,
                        },
                        &cfg,
                    );
                    last_cost_alert_cents = total.cost_cents;
                }
            }
        }

        // Update productivity stats (every 30s like cost scanning)
        if cycle % SCAN_INTERVAL == 1 {
            for agent in tracked.values() {
                if !agent.agent_id.is_empty() {
                    if let Some(ref wd) = agent.working_dir {
                        let cost = cost_tracker
                            .summary()
                            .iter()
                            .find(|c| c.agent_id == agent.agent_id)
                            .map(|c| c.cost_cents)
                            .unwrap_or(0);
                        productivity_tracker.update(&agent.agent_id, wd, cost);
                    }
                }
            }
        }

        // Push state to IPC
        if let Some(state) = super::server::get_global_state() {
            if let Ok(mut st) = state.lock() {
                st.heartbeat_count = cycle;
                let mut snapshots: Vec<super::server::AgentSnapshot> = tracked
                    .values()
                    .map(super::server::AgentSnapshot::from)
                    .collect();
                // Attach productivity stats to each snapshot
                for snap in &mut snapshots {
                    if let Some(ps) = productivity_tracker.get(&snap.agent_id) {
                        snap.productivity = Some(super::server::ProductivitySnapshot {
                            files_changed: ps.files_changed,
                            lines_added: ps.lines_added,
                            lines_removed: ps.lines_removed,
                            commits: ps.commits_this_session,
                            build_attempts: ps.build_attempts,
                            build_successes: ps.build_successes,
                            lines_per_dollar: ps.lines_per_dollar,
                        });
                    }
                }
                st.agents = snapshots;
                st.costs = cost_tracker.summary();
                st.recent_activity = activity_feed.recent(20);
                if let Some(total) = cost_tracker.total_summary() {
                    st.total_tokens = total.input_tokens + total.output_tokens;
                    st.total_cost_cents = total.cost_cents;
                }
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
                // Try to find which tmux pane owns this process
                let real_pane_id = panes.iter().find(|p| {
                    detector::descendant_processes(p.pane_pid, &procs)
                        .iter()
                        .any(|d| d.pid == proc.pid)
                }).map(|p| format!("{}:{}.{}", p.session, p.window_index, p.pane_index));

                let pane_id_str = real_pane_id.unwrap_or_else(|| format!("pid-{}", proc.pid));
                let agent = tracked
                    .entry(key)
                    .or_insert_with(|| {
                        tracing::info!("New agent (process scan): {} pid={} pane={}", kind, proc.pid, pane_id_str);
                        let mut a = TrackedAgent::new(kind, pane_id_str.clone());
                        a.pid = Some(proc.pid);
                        a
                    });
                agent.kind = kind;
                agent.pid = Some(proc.pid);
                // Update pane_id if we found a real one
                if agent.pane_id.starts_with("pid-") && !pane_id_str.starts_with("pid-") {
                    agent.pane_id = pane_id_str;
                }

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
    // Find active Claude projects (modified in last 6 hours)
    let active_projects = detector::find_active_claude_projects(6 * 3600);
    let mut used_projects: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Collect already-assigned working dirs
    for agent in tracked.values() {
        if let Some(ref wd) = agent.working_dir {
            if wd != "/" {
                used_projects.insert(wd.clone());
            }
        }
    }

    // Sort Claude agents so we assign projects in a stable order
    let mut agent_keys: Vec<String> = tracked.keys().cloned().collect();
    agent_keys.sort();

    // Pre-collect used species from active agents (for rotation)
    let mut active_species: std::collections::HashSet<String> = tracked.values()
        .filter_map(|a| a.creature_species.clone())
        .collect();

    for key in &agent_keys {
        let agent = match tracked.get_mut(key) {
            Some(a) => a,
            None => continue,
        };
        if let Some(pid) = agent.pid {
            // Sum CPU and memory across all descendant processes
            let descendants = detector::descendant_processes(pid, &procs);
            agent.cpu_pct = descendants.iter().map(|p| p.cpu_pct).sum();
            agent.mem_mb = descendants.iter().map(|p| p.mem_mb).sum();

            // Detect working directory
            if agent.working_dir.is_none() || agent.working_dir.as_deref() == Some("/") {
                let wd = detector::get_working_dir(pid);
                if wd.as_deref() != Some("/") && wd.is_some() {
                    agent.working_dir = wd;
                } else if agent.kind == AgentKind::Claude {
                    // Assign from active Claude projects (most recently modified JSONL)
                    for (_encoded, real_path) in &active_projects {
                        if !used_projects.contains(real_path) {
                            agent.working_dir = Some(real_path.clone());
                            used_projects.insert(real_path.clone());
                            break;
                        }
                    }
                }
            }

            // Compute stable agent identity
            if agent.agent_id.is_empty() {
                let agent_kind_str = agent.kind.to_string();
                let agent_id = identity::compute_agent_id_with_pid(
                    &agent_kind_str,
                    agent.working_dir.as_deref(),
                    agent.pid,
                );
                agent.agent_id = agent_id.clone();

                // Check if binding already exists (has a known species)
                let bindings = identity::load_bindings();
                let (species_str, binding, is_new) = if let Some(existing) = bindings.get(&agent_id) {
                    (existing.creature_species.clone(), existing.clone(), false)
                } else {
                    // Find the next unused species by checking ACTIVE agents only
                    drop(bindings);
                    // Try each index until we find an unused species
                    let mut species = crate::creatures::sprites::species_for_agent(&agent_kind_str);
                    for idx in 0..6 {
                        let candidate = crate::creatures::sprites::species_for_agent_idx(&agent_kind_str, idx);
                        if !active_species.contains(candidate as &str) {
                            species = candidate;
                            break;
                        }
                    }
                    let (b, _) = identity::get_or_create_binding(&agent_id, species);
                    (species.to_string(), b, true)
                };

                agent.creature_species = Some(species_str.clone());
                active_species.insert(species_str.clone());

                // Ensure binding exists
                if !is_new {
                    let _ = identity::get_or_create_binding(&agent_id, &species_str);
                }

                if is_new {
                    tracing::info!(
                        agent_id = %agent_id,
                        species = %species_str,
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
    _config: &config::Config,
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

/// Award XP to creatures based on recent activity events.
fn award_xp_from_activity(
    feed: &ActivityFeed,
    tracked: &HashMap<String, TrackedAgent>,
    config: &config::Config,
) {
    use crate::agents::activity::EventType;

    // Get recent events (last 20)
    let events = feed.recent(50);
    if events.is_empty() {
        return;
    }

    // Build maps for matching events to agents
    let name_to_id: HashMap<String, String> = tracked
        .values()
        .filter(|a| !a.agent_id.is_empty())
        .map(|a| (a.kind.to_string().to_lowercase(), a.agent_id.clone()))
        .collect();
    
    // Build project → agent_id map for project-based matching
    let project_to_id: HashMap<String, String> = tracked
        .values()
        .filter(|a| !a.agent_id.is_empty() && a.working_dir.is_some())
        .map(|a| {
            let encoded = crate::agents::cost::encode_working_dir(
                a.working_dir.as_deref().unwrap_or("/")
            );
            (encoded, a.agent_id.clone())
        })
        .collect();

    // XP rewards per event type
    let mut xp_gains: HashMap<String, u64> = HashMap::new();

    for event in &events {
        let xp = match event.event_type {
            EventType::FileWrite => 2,
            EventType::FileRead => 1,
            EventType::Command => 3,
            EventType::Error => 5,       // resilience XP for error/crash recovery
            EventType::TokenUsage => 1,  // thinking/reasoning
            EventType::Thinking => 1,
            EventType::Responding => 1,  // active typing
            EventType::StateChange => 0,
        };

        if xp > 0 {
            // Try to match event to an agent by project dir first
            let mut matched = false;
            if !event.project.is_empty() {
                if let Some(agent_id) = project_to_id.get(&event.project) {
                    *xp_gains.entry(agent_id.clone()).or_insert(0) += xp;
                    matched = true;
                }
            }
            
            if !matched {
                // Fallback: match by name
                let agent_name = event.agent_name.to_lowercase();
                if let Some(agent_id) = name_to_id.get(&agent_name) {
                    *xp_gains.entry(agent_id.clone()).or_insert(0) += xp;
                    matched = true;
                } else {
                    for (name, id) in &name_to_id {
                        if agent_name.contains(name) || name.contains(&agent_name) {
                            *xp_gains.entry(id.clone()).or_insert(0) += xp;
                            matched = true;
                            break;
                        }
                    }
                }
            }
            
            // Last resort: if only one agent, give it all XP
            if !matched && name_to_id.len() == 1 {
                if let Some(id) = name_to_id.values().next() {
                    *xp_gains.entry(id.clone()).or_insert(0) += xp;
                }
            }
        }
    }

    // Apply XP gains
    for (agent_id, xp_gain) in &xp_gains {
        let bindings = identity::load_bindings();
        if let Some(binding) = bindings.get(agent_id) {
            let new_xp = binding.xp + xp_gain;
            let new_stage = if new_xp >= 500 {
                3
            } else if new_xp >= 100 {
                2
            } else {
                1
            };

            if new_xp != binding.xp {
                identity::update_xp(agent_id, new_xp, new_stage);
                if new_stage > binding.stage {
                    let species = binding.creature_species.clone();
                    let creature_def = crate::creatures::registry::get_creature_def(&species);
                    let creature_name = creature_def
                        .map(|d| d.evolution_names[(new_stage as usize).saturating_sub(1).min(2)].to_string())
                        .unwrap_or_else(|| species.clone());

                    tracing::info!(
                        agent_id = %agent_id,
                        old_stage = binding.stage,
                        new_stage = new_stage,
                        xp = new_xp,
                        "🎉 Creature evolved!"
                    );

                    super::notify::send_notification(
                        &super::notify::NotifyEvent::Evolution {
                            creature_name,
                            new_stage,
                            xp: new_xp,
                        },
                        config,
                    );
                }
            }
        }
    }
}
