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
}

impl From<&TrackedAgent> for AgentSnapshot {
    fn from(a: &TrackedAgent) -> Self {
        let species = crate::creatures::sprites::species_for_agent(&a.kind.to_string());
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
}

/// Shared daemon state exposed to IPC clients.
#[derive(Debug, Clone, Default)]
pub struct DaemonState {
    pub agents: Vec<AgentSnapshot>,
    pub started_at: Option<String>,
    pub heartbeat_count: u64,
    pub costs: Vec<AgentCostSummary>,
    pub recent_activity: Vec<ActivityEvent>,
}

pub type SharedState = Arc<Mutex<DaemonState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(DaemonState {
        agents: Vec::new(),
        started_at: Some(chrono::Utc::now().to_rfc3339()),
        heartbeat_count: 0,
        costs: Vec::new(),
        recent_activity: Vec::new(),
    }))
}

/// Global state handle so the heartbeat loop can update it.
static GLOBAL_STATE: std::sync::Mutex<Option<SharedState>> = std::sync::Mutex::new(None);

/// Get the global shared state (used by heartbeat).
pub fn get_global_state() -> Option<SharedState> {
    GLOBAL_STATE.lock().ok()?.clone()
}

/// Run the IPC server until shutdown.
pub async fn run_server(mut shutdown: watch::Receiver<bool>) -> Result<()> {
    let socket_path = super::socket_path();
    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;
    tracing::info!("IPC server listening on {}", socket_path.display());

    let state = new_shared_state();
    if let Ok(mut g) = GLOBAL_STATE.lock() {
        *g = Some(state.clone());
    }

    loop {
        tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok((stream, _)) => {
                        let st = state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_client(stream, st).await {
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

async fn handle_client(stream: tokio::net::UnixStream, state: SharedState) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let command = line.trim();

    let response = match command {
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
        _ => format!("unknown command: {command}"),
    };

    writer.write_all(response.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.shutdown().await?;
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
