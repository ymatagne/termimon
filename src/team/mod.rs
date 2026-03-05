//! Team Mode — TCP-based peer networking for TermiMon
//!
//! Allows multiple TermiMon instances to connect, sync creature state,
//! and battle each other over the network.

pub mod battle;
pub mod client;
pub mod mdns;
pub mod peer;
pub mod protocol;
pub mod server;

use std::sync::{Arc, Mutex};
use peer::PeerRegistry;
use tokio::sync::broadcast;

/// Shared team state accessible from daemon and UI.
pub type SharedTeamState = Arc<Mutex<TeamState>>;

/// Global team state.
pub struct TeamState {
    pub registry: PeerRegistry,
    pub hosting: bool,
    pub connected: bool,
    pub local_name: String,
    pub battle_log: Vec<battle::BattleResult>,
    /// Broadcast sender for the team server (set when hosting).
    pub broadcast_tx: Option<broadcast::Sender<String>>,
}

impl std::fmt::Debug for TeamState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TeamState")
            .field("registry", &self.registry)
            .field("hosting", &self.hosting)
            .field("connected", &self.connected)
            .field("local_name", &self.local_name)
            .field("battle_log", &self.battle_log)
            .field("broadcast_tx", &self.broadcast_tx.as_ref().map(|_| "Some(...)"))
            .finish()
    }
}

impl TeamState {
    pub fn new(name: String) -> Self {
        Self {
            registry: PeerRegistry::new(),
            hosting: false,
            connected: false,
            local_name: name,
            battle_log: Vec::new(),
            broadcast_tx: None,
        }
    }
}

pub fn new_shared_team_state(name: String) -> SharedTeamState {
    Arc::new(Mutex::new(TeamState::new(name)))
}

/// Global team state handle.
static GLOBAL_TEAM_STATE: std::sync::Mutex<Option<SharedTeamState>> = std::sync::Mutex::new(None);

pub fn set_global_team_state(state: SharedTeamState) {
    if let Ok(mut g) = GLOBAL_TEAM_STATE.lock() {
        *g = Some(state);
    }
}

pub fn get_global_team_state() -> Option<SharedTeamState> {
    GLOBAL_TEAM_STATE.lock().ok()?.clone()
}
