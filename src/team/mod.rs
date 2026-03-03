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

/// Shared team state accessible from daemon and UI.
pub type SharedTeamState = Arc<Mutex<TeamState>>;

/// Global team state.
#[derive(Debug)]
pub struct TeamState {
    pub registry: PeerRegistry,
    pub hosting: bool,
    pub connected: bool,
    pub local_name: String,
    pub battle_log: Vec<battle::BattleResult>,
}

impl TeamState {
    pub fn new(name: String) -> Self {
        Self {
            registry: PeerRegistry::new(),
            hosting: false,
            connected: false,
            local_name: name,
            battle_log: Vec::new(),
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
