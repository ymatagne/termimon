//! Peer state tracking for team mode.

use std::collections::HashMap;
use std::time::Instant;

use super::protocol::CreatureSync;

/// A connected peer.
#[derive(Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub creatures: Vec<CreatureSync>,
    pub last_sync: Instant,
    pub connected_at: Instant,
}

impl Peer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            creatures: Vec::new(),
            last_sync: Instant::now(),
            connected_at: Instant::now(),
        }
    }

    pub fn update_creatures(&mut self, creatures: Vec<CreatureSync>) {
        self.creatures = creatures;
        self.last_sync = Instant::now();
    }

    /// Returns true if this peer hasn't synced in over 10 seconds.
    pub fn is_stale(&self) -> bool {
        self.last_sync.elapsed().as_secs() > 10
    }
}

/// Registry of all known peers.
#[derive(Debug)]
pub struct PeerRegistry {
    pub peers: HashMap<String, Peer>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    pub fn add_peer(&mut self, name: String) -> bool {
        if self.peers.contains_key(&name) {
            return false;
        }
        self.peers.insert(name.clone(), Peer::new(name));
        true
    }

    pub fn remove_peer(&mut self, name: &str) -> bool {
        self.peers.remove(name).is_some()
    }

    pub fn update_peer_creatures(&mut self, name: &str, creatures: Vec<CreatureSync>) {
        if let Some(peer) = self.peers.get_mut(name) {
            peer.update_creatures(creatures);
        }
    }

    pub fn peer_names(&self) -> Vec<String> {
        self.peers.keys().cloned().collect()
    }

    /// Get all creatures from all peers, flattened.
    pub fn all_creatures(&self) -> Vec<&CreatureSync> {
        self.peers.values().flat_map(|p| &p.creatures).collect()
    }

    /// Prune stale peers. Returns names of removed peers.
    pub fn prune_stale(&mut self) -> Vec<String> {
        let stale: Vec<String> = self
            .peers
            .iter()
            .filter(|(_, p)| p.is_stale())
            .map(|(n, _)| n.clone())
            .collect();
        for name in &stale {
            self.peers.remove(name);
        }
        stale
    }
}
