//! TCP client for joining a team host.

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::watch;

use super::protocol::{CreatureSync, Message, PROTOCOL_VERSION};
use super::SharedTeamState;

/// Connect to a team host and run the sync loop.
pub async fn connect_to_host(
    addr: &str,
    team_state: SharedTeamState,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let stream = TcpStream::connect(addr)
        .await
        .context(format!("Failed to connect to {addr}"))?;

    tracing::info!("Connected to team host at {addr}");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send Hello
    let local_name = team_state.lock().map_err(|e| anyhow::anyhow!("{e}"))?.local_name.clone();
    let hello = Message::Hello {
        name: local_name.clone(),
        version: PROTOCOL_VERSION.to_string(),
    };
    writer.write_all(hello.to_line().as_bytes()).await?;

    // Wait for Welcome
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let msg = Message::from_line(&line)?;

    match msg {
        Message::Welcome { name, peers } => {
            tracing::info!("Joined team hosted by '{name}' with peers: {:?}", peers);
            if let Ok(mut ts) = team_state.lock() {
                ts.connected = true;
                for peer in &peers {
                    ts.registry.add_peer(peer.clone());
                }
            }
        }
        _ => {
            anyhow::bail!("Expected Welcome, got: {:?}", msg);
        }
    }

    // Spawn reader task
    let ts_read = team_state.clone();
    let read_handle = tokio::spawn(async move {
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => break,
                Ok(_) => {
                    if let Ok(msg) = Message::from_line(&line) {
                        handle_incoming_message(msg, &ts_read);
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Sync loop: send local creature state every 2 seconds
    let sync_interval = std::time::Duration::from_secs(2);
    loop {
        tokio::select! {
            _ = tokio::time::sleep(sync_interval) => {
                let creatures = gather_local_creatures(&team_state);
                if !creatures.is_empty() {
                    let sync_msg = Message::Sync { creatures };
                    if writer.write_all(sync_msg.to_line().as_bytes()).await.is_err() {
                        break;
                    }
                }
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    // Send goodbye
                    let goodbye = Message::Goodbye { name: local_name.clone() };
                    let _ = writer.write_all(goodbye.to_line().as_bytes()).await;
                    break;
                }
            }
        }
    }

    read_handle.abort();

    if let Ok(mut ts) = team_state.lock() {
        ts.connected = false;
        ts.registry.peers.clear();
    }

    tracing::info!("Disconnected from team host");
    Ok(())
}

fn handle_incoming_message(msg: Message, team_state: &SharedTeamState) {
    match msg {
        Message::Sync { creatures } => {
            if let Some(owner) = creatures.first().map(|c| c.owner.clone()) {
                if let Ok(mut ts) = team_state.lock() {
                    ts.registry.update_peer_creatures(&owner, creatures);
                }
            }
        }
        Message::PeerJoined { name } => {
            if let Ok(mut ts) = team_state.lock() {
                ts.registry.add_peer(name.clone());
            }
            tracing::info!("Peer '{name}' joined");
        }
        Message::PeerLeft { name } => {
            if let Ok(mut ts) = team_state.lock() {
                ts.registry.remove_peer(&name);
            }
            tracing::info!("Peer '{name}' left");
        }
        Message::BattleResult { winner, loser, rounds } => {
            tracing::info!("Battle result: {winner} defeated {loser} in {} rounds", rounds.len());
            if let Ok(mut ts) = team_state.lock() {
                ts.battle_log.push(super::battle::BattleResult {
                    winner: winner.clone(),
                    loser: loser.clone(),
                    winner_owner: String::new(),
                    loser_owner: String::new(),
                    rounds,
                    xp_gained: 0,
                });
            }
        }
        Message::Emote { from, emoji } => {
            tracing::info!("Emote from {from}: {emoji}");
        }
        _ => {}
    }
}

/// Gather local creature state from the daemon's shared state.
fn gather_local_creatures(team_state: &SharedTeamState) -> Vec<CreatureSync> {
    let owner = team_state
        .lock()
        .ok()
        .map(|ts| ts.local_name.clone())
        .unwrap_or_default();

    // Get creatures from the daemon's global state
    if let Some(daemon_state) = crate::daemon::server::get_global_state() {
        if let Ok(st) = daemon_state.lock() {
            return st
                .agents
                .iter()
                .map(|a| CreatureSync {
                    name: a.creature_name.clone(),
                    species: a.creature_species.clone(),
                    stage: a.stage,
                    xp: a.xp,
                    state: a.state.clone(),
                    cpu: a.cpu_pct,
                    project: a.working_dir.as_deref().unwrap_or("").to_string(),
                    owner: owner.clone(),
                })
                .collect();
        }
    }

    Vec::new()
}
