//! Unix socket IPC server for CLI ↔ daemon communication
//!
//! Protocol: simple line-based text.
//!   Request:  "<command>\n"
//!   Response: "<text>\n"

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::watch;

use std::sync::{Arc, Mutex};
use crate::agents::TrackedAgent;
use crate::agents::activity::ActivityEvent;
use crate::agents::cost::AgentCostSummary;
use crate::team::{self, SharedTeamState};
use crate::team::battle::{BattleStats, resolve_battle};

/// Serializable snapshot of a tracked agent for IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSnapshot {
    pub kind: String,
    pub state: String,
    pub pane_id: String,
    pub pid: Option<u32>,
    pub cpu_pct: f32,
    pub mem_mb: f64,
    pub working_dir: Option<String>,
    pub agent_id: String,
    pub creature_species: String,
    pub creature_name: String,
    pub element_icon: String,
    pub stage: u8,
    pub xp: u64,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub xp_into_level: u64,
    #[serde(default)]
    pub xp_for_next_level: u64,
    #[serde(default)]
    pub badge: String,
    #[serde(default)]
    pub productivity: Option<ProductivitySnapshot>,
}

/// Productivity stats snapshot for IPC.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProductivitySnapshot {
    pub files_changed: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub commits: u32,
    pub build_attempts: u32,
    pub build_successes: u32,
    pub lines_per_dollar: f64,
}

impl From<&TrackedAgent> for AgentSnapshot {
    fn from(a: &TrackedAgent) -> Self {
        let species = a.creature_species.as_deref()
            .unwrap_or_else(|| crate::creatures::sprites::species_for_agent(&a.kind.to_string()));
        let creature_def = crate::creatures::registry::get_creature_def(species);

        // Use identity bindings for XP/stage if available
        let bindings = crate::agents::identity::load_bindings();
        let (xp, stage) = if !a.agent_id.is_empty() {
            if let Some(binding) = bindings.get(&a.agent_id) {
                (binding.xp, binding.stage)
            } else {
                (0, 1)
            }
        } else {
            (0, 1)
        };

        let stage_idx = (stage as usize).saturating_sub(1).min(2);
        let (creature_name, element_icon) = match creature_def {
            Some(def) => (def.evolution_names[stage_idx].to_string(), def.element.icon().to_string()),
            None => ("Unknown".to_string(), "❓".to_string()),
        };
        let (level, xp_into_level, xp_for_next_level) = crate::creatures::evolution::level_from_xp(xp);
        let badge = crate::creatures::evolution::prestige_badge(xp).to_string();
        Self {
            kind: a.kind.to_string(),
            state: a.state.to_string(),
            pane_id: a.pane_id.clone(),
            pid: a.pid,
            cpu_pct: a.cpu_pct,
            mem_mb: a.mem_mb,
            working_dir: a.working_dir.clone(),
            agent_id: a.agent_id.clone(),
            creature_species: species.to_string(),
            creature_name,
            element_icon,
            stage,
            xp,
            level,
            xp_into_level,
            xp_for_next_level,
            badge,
            productivity: None,
        }
    }
}

/// Full status response sent over IPC as JSON.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusResponse {
    pub running: bool,
    pub pid: u32,
    pub started_at: String,
    pub heartbeat_count: u64,
    pub agents: Vec<AgentSnapshot>,
    pub total_xp: u64,
    #[serde(default)]
    pub costs: Vec<AgentCostSummary>,
    #[serde(default)]
    pub recent_activity: Vec<ActivityEvent>,
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub total_cost_cents: u64,
}

/// Shared daemon state exposed to IPC clients.
#[derive(Debug, Clone, Default)]
pub struct DaemonState {
    pub agents: Vec<AgentSnapshot>,
    pub started_at: Option<String>,
    pub heartbeat_count: u64,
    pub costs: Vec<AgentCostSummary>,
    pub recent_activity: Vec<ActivityEvent>,
    pub total_tokens: u64,
    pub total_cost_cents: u64,
}

pub type SharedState = Arc<Mutex<DaemonState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(DaemonState {
        agents: Vec::new(),
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        heartbeat_count: 0,
        costs: Vec::new(),
        recent_activity: Vec::new(),
        total_tokens: 0,
        total_cost_cents: 0,
    }))
}

/// Global state handle so the heartbeat loop can update it.
static GLOBAL_STATE: std::sync::Mutex<Option<SharedState>> = std::sync::Mutex::new(None);

/// Get the global shared state (used by heartbeat).
pub fn get_global_state() -> Option<SharedState> {
    GLOBAL_STATE.lock().ok()?.clone()
}

/// Serializable team status for IPC responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamStatusResponse {
    pub hosting: bool,
    pub connected: bool,
    pub local_name: String,
    pub peers: Vec<String>,
    pub battle_count: usize,
    #[serde(default)]
    pub peer_creatures: std::collections::HashMap<String, Vec<crate::team::protocol::CreatureSync>>,
}

/// Shared handle for the team shutdown sender so IPC handlers can signal team tasks.
type SharedTeamShutdown = Arc<watch::Sender<bool>>;

/// Run the IPC server until shutdown.
pub async fn run_server(
    mut shutdown: watch::Receiver<bool>,
    team_state: SharedTeamState,
    team_shutdown_tx: watch::Sender<bool>,
) -> Result<()> {
    let socket_path = super::socket_path();
    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;
    tracing::info!("IPC server listening on {}", socket_path.display());

    let state = new_shared_state();
    if let Ok(mut g) = GLOBAL_STATE.lock() {
        *g = Some(state.clone());
    }

    // Store team state globally so dashboard can read it
    team::set_global_team_state(team_state.clone());

    let team_shutdown = Arc::new(team_shutdown_tx);

    loop {
        tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok((stream, _)) => {
                        let st = state.clone();
                        let ts = team_state.clone();
                        let ttx = team_shutdown.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, st, ts, ttx).await {
                                tracing::debug!("Client error: {e}");
                            }
                        });
                    }
                    Err(e) => tracing::warn!("Accept error: {e}"),
                }
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    tracing::info!("IPC server shutting down");
                    break;
                }
            }
        }
    }
    Ok(())
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    state: SharedState,
    team_state: SharedTeamState,
    team_shutdown_tx: SharedTeamShutdown,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let command = line.trim().to_string();

    let response = match command.as_str() {
        "status" | "status_json" => {
            let st = state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
            let resp = StatusResponse {
                running: true,
                pid: std::process::id(),
                started_at: st.started_at.clone().unwrap_or_default(),
                heartbeat_count: st.heartbeat_count,
                agents: st.agents.clone(),
                total_xp: st.agents.iter().map(|a| a.xp).sum(),
                costs: st.costs.clone(),
                recent_activity: st.recent_activity.clone(),
                total_tokens: st.total_tokens,
                total_cost_cents: st.total_cost_cents,
            };
            serde_json::to_string(&resp)?
        }
        "costs" => {
            let st = state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
            serde_json::to_string_pretty(&st.costs)?
        }
        "activity" => {
            let st = state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
            serde_json::to_string_pretty(&st.recent_activity)?
        }
        "agents" => {
            let st = state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
            serde_json::to_string_pretty(&st.agents)?
        }
        "ping" => "pong".to_string(),
        "team_status" => {
            let ts = team_state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
            let mut peer_creatures = std::collections::HashMap::new();
            for (name, peer) in &ts.registry.peers {
                peer_creatures.insert(name.clone(), peer.creatures.clone());
            }
            let resp = TeamStatusResponse {
                hosting: ts.hosting,
                connected: ts.connected,
                local_name: ts.local_name.clone(),
                peers: ts.registry.peer_names(),
                battle_count: ts.battle_log.len(),
                peer_creatures,
            };
            serde_json::to_string(&resp)?
        }
        cmd if cmd.starts_with("team_host") => {
            let port: u16 = cmd.strip_prefix("team_host")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or_else(|| crate::config::load().team.port);

            // Check if already hosting/connected
            let already_active = team_state.lock()
                .map(|ts| ts.hosting || ts.connected)
                .unwrap_or(false);
            if already_active {
                return write_response(&mut writer, "ERROR: Already in a team session. Leave first.").await;
            }

            let ts = team_state.clone();
            let shutdown_rx = team_shutdown_tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = team::server::run_team_server(port, ts, shutdown_rx).await {
                    tracing::error!("Team server error: {e}");
                }
            });

            format!("OK: Hosting team on port {port}")
        }
        cmd if cmd.starts_with("team_join ") => {
            let addr = cmd.strip_prefix("team_join ").unwrap().trim().to_string();

            let already_active = team_state.lock()
                .map(|ts| ts.hosting || ts.connected)
                .unwrap_or(false);
            if already_active {
                return write_response(&mut writer, "ERROR: Already in a team session. Leave first.").await;
            }

            let resp = format!("OK: Joining team at {addr}");
            let ts = team_state.clone();
            let shutdown_rx = team_shutdown_tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = team::client::connect_to_host(&addr, ts, shutdown_rx).await {
                    tracing::error!("Team client error: {e}");
                }
            });

            resp
        }
        "team_leave" => {
            // Signal team shutdown
            let _ = team_shutdown_tx.send(true);

            // Clear team state
            if let Ok(mut ts) = team_state.lock() {
                ts.connected = false;
                ts.hosting = false;
                ts.registry.peers.clear();
            }

            // Reset team shutdown channel for future use (we can't easily,
            // but team server/client will have exited)
            "OK: Left team".to_string()
        }
        "team_auto" => {
            let ts = team_state.clone();
            let shutdown_rx = team_shutdown_tx.subscribe();
            tokio::spawn(async move {
                if let Err(e) = team_auto_discover(ts, shutdown_rx).await {
                    tracing::error!("Team auto-discover error: {e}");
                }
            });
            "OK: Auto-discovery started".to_string()
        }
        cmd if cmd.starts_with("battle ") => {
            handle_battle_command(cmd, &state, &team_state)?
        }
        _ => format!("unknown command: {command}"),
    };

    writer.write_all(response.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.shutdown().await?;
    Ok(())
}

fn handle_battle_command(
    cmd: &str,
    state: &SharedState,
    team_state: &SharedTeamState,
) -> Result<String> {
    // Parse: "battle <local_creature> <peer> <peer_creature>"
    let parts: Vec<&str> = cmd.splitn(4, ' ').collect();
    if parts.len() < 4 {
        return Ok("ERROR: Usage: battle <local_creature> <peer> <peer_creature>".to_string());
    }
    let local_creature_name = parts[1];
    let peer_name = parts[2];
    let peer_creature_name = parts[3];

    // Look up local creature
    let local_stats = {
        let st = state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        let local_name = team_state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?.local_name.clone();
        st.agents.iter()
            .find(|a| a.creature_name == local_creature_name)
            .map(|a| BattleStats::from_xp(&a.creature_name, &a.creature_species, a.xp, &local_name))
    };
    let local_stats = match local_stats {
        Some(s) => s,
        None => return Ok(format!("ERROR: Local creature '{}' not found", local_creature_name)),
    };

    // Look up peer creature
    let peer_stats = {
        let ts = team_state.lock().map_err(|e| anyhow::anyhow!("lock: {e}"))?;
        ts.registry.peers.get(peer_name)
            .and_then(|peer| {
                peer.creatures.iter()
                    .find(|c| c.name == peer_creature_name)
                    .map(|c| BattleStats::from_xp(&c.name, &c.species, c.xp, &peer.name))
            })
    };
    let peer_stats = match peer_stats {
        Some(s) => s,
        None => return Ok(format!("ERROR: Peer creature '{}' from '{}' not found", peer_creature_name, peer_name)),
    };

    // Resolve battle
    let result = resolve_battle(local_stats, peer_stats);

    // Store in battle log
    if let Ok(mut ts) = team_state.lock() {
        ts.battle_log.push(result.clone());

        // Broadcast to peers if hosting
        if let Some(ref tx) = ts.broadcast_tx {
            let msg = crate::team::protocol::Message::BattleResult {
                winner: result.winner.clone(),
                loser: result.loser.clone(),
                rounds: result.rounds.clone(),
            };
            let _ = tx.send(msg.to_line());
        }
    }

    // Return JSON result
    Ok(serde_json::to_string(&result)?)
}

async fn write_response(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    msg: &str,
) -> Result<()> {
    writer.write_all(msg.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.shutdown().await?;
    Ok(())
}

/// Auto-discover and connect to team peers (runs in daemon context).
async fn team_auto_discover(
    team_state: SharedTeamState,
    shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let cfg = crate::config::load();
    let port = cfg.team.port;
    let name = cfg.team.name.clone();

    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;

    let announce = format!("TERMIMON_DISCOVER:{}:{}", name, port);
    socket.send_to(announce.as_bytes(), "255.255.255.255:14662")?;

    let mut buf = [0u8; 1024];
    let mut found_peers: Vec<String> = Vec::new();

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.starts_with("TERMIMON_DISCOVER:") || msg.starts_with("TERMIMON_REPLY:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    if parts.len() >= 3 {
                        let peer_name = parts[1];
                        let peer_port: u16 = parts[2].parse().unwrap_or(4662);
                        if peer_name != name {
                            let peer_addr = format!("{}:{}", addr.ip(), peer_port);
                            found_peers.push(peer_addr);
                            let reply = format!("TERMIMON_REPLY:{}:{}", name, port);
                            let _ = socket.send_to(reply.as_bytes(), addr);
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }

    if found_peers.is_empty() {
        tracing::info!("No peers found, starting as host on port {port}");
        team::server::run_team_server(port, team_state, shutdown_rx).await?;
    } else {
        tracing::info!("Found peer, connecting to {}", found_peers[0]);
        team::client::connect_to_host(&found_peers[0], team_state, shutdown_rx).await?;
    }

    Ok(())
}

/// Client-side: connect to daemon and send a request, return response.
pub async fn client_request(command: &str) -> Result<String> {
    let stream = tokio::net::UnixStream::connect(super::socket_path())
        .await
        .context("Could not connect to daemon socket")?;

    let (reader, mut writer) = stream.into_split();
    writer.write_all(command.as_bytes()).await?;
    writer.write_all(b"\n").await?;

    let mut reader = BufReader::new(reader);
    let mut response = String::new();
    loop {
        let mut buf = String::new();
        if reader.read_line(&mut buf).await? == 0 {
            break;
        }
        response.push_str(&buf);
    }
    Ok(response)
}
