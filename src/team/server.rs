//! TCP server for hosting team sessions.

use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::protocol::{Message, PROTOCOL_VERSION};
use super::SharedTeamState;

/// A connected client writer handle.
type ClientWriter = Arc<Mutex<tokio::net::tcp::OwnedWriteHalf>>;

/// Run the team TCP server on the given port.
pub async fn run_team_server(
    port: u16,
    team_state: SharedTeamState,
    mut shutdown: watch::Receiver<bool>,
) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!("Team server listening on port {port}");

    if let Ok(mut ts) = team_state.lock() {
        ts.hosting = true;
        ts.connected = true;
    }

    let (broadcast_tx, _) = broadcast::channel::<String>(100);
    let clients: Arc<Mutex<HashMap<String, ClientWriter>>> = Arc::new(Mutex::new(HashMap::new()));

    loop {
        tokio::select! {
            accept = listener.accept() => {
                match accept {
                    Ok((stream, addr)) => {
                        tracing::info!("Team peer connected from {addr}");
                        let ts = team_state.clone();
                        let tx = broadcast_tx.clone();
                        let rx = broadcast_tx.subscribe();
                        let cls = clients.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_peer(stream, ts, tx, rx, cls).await {
                                tracing::debug!("Peer {addr} error: {e}");
                            }
                        });
                    }
                    Err(e) => tracing::warn!("Accept error: {e}"),
                }
            }
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    tracing::info!("Team server shutting down");
                    break;
                }
            }
        }
    }

    if let Ok(mut ts) = team_state.lock() {
        ts.hosting = false;
        ts.connected = false;
    }

    Ok(())
}

async fn handle_peer(
    stream: tokio::net::TcpStream,
    team_state: SharedTeamState,
    broadcast_tx: broadcast::Sender<String>,
    mut broadcast_rx: broadcast::Receiver<String>,
    clients: Arc<Mutex<HashMap<String, ClientWriter>>>,
) -> Result<()> {
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let writer = Arc::new(Mutex::new(writer));
    let mut peer_name = String::new();

    // Wait for Hello
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let msg = Message::from_line(&line)?;

    match msg {
        Message::Hello { name, version } => {
            if version != PROTOCOL_VERSION {
                tracing::warn!("Version mismatch: {version} vs {PROTOCOL_VERSION}");
            }
            peer_name = name.clone();

            // Register peer
            let peer_names = {
                let mut ts = team_state.lock().map_err(|e| anyhow::anyhow!("{e}"))?;
                ts.registry.add_peer(name.clone());
                ts.registry.peer_names()
            };

            // Send Welcome
            let host_name = team_state.lock().map_err(|e| anyhow::anyhow!("{e}"))?.local_name.clone();
            let welcome = Message::Welcome {
                name: host_name,
                peers: peer_names,
            };
            let mut w = writer.lock().await;
            w.write_all(welcome.to_line().as_bytes()).await?;
            drop(w);

            // Store client writer
            clients.lock().await.insert(name.clone(), writer.clone());

            // Broadcast join
            let join_msg = Message::PeerJoined { name: name.clone() };
            let _ = broadcast_tx.send(join_msg.to_line());

            tracing::info!("Peer '{name}' joined the team");
        }
        _ => {
            anyhow::bail!("Expected Hello, got: {:?}", msg);
        }
    }

    // Main loop: read from peer + forward broadcasts
    let peer_name_clone = peer_name.clone();
    let writer_clone = writer.clone();

    // Spawn broadcast forwarder
    let forward_handle = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            let mut w = writer_clone.lock().await;
            if w.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    // Read messages from this peer
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // disconnected
            Ok(_) => {
                if let Ok(msg) = Message::from_line(&line) {
                    match msg {
                        Message::Sync { ref creatures } => {
                            if let Ok(mut ts) = team_state.lock() {
                                ts.registry.update_peer_creatures(&peer_name, creatures.clone());
                            }
                            // Broadcast sync to other peers
                            let _ = broadcast_tx.send(line.clone());
                        }
                        Message::Challenge { .. } | Message::Accept { .. } | Message::Decline { .. } => {
                            // Forward to all peers
                            let _ = broadcast_tx.send(line.clone());
                        }
                        Message::Emote { .. } => {
                            let _ = broadcast_tx.send(line.clone());
                        }
                        Message::Goodbye { .. } => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => break,
        }
    }

    // Cleanup
    forward_handle.abort();
    clients.lock().await.remove(&peer_name_clone);
    if let Ok(mut ts) = team_state.lock() {
        ts.registry.remove_peer(&peer_name_clone);
    }

    // Broadcast leave
    let leave_msg = Message::PeerLeft { name: peer_name_clone.clone() };
    let _ = broadcast_tx.send(leave_msg.to_line());

    tracing::info!("Peer '{}' left the team", peer_name_clone);
    Ok(())
}
