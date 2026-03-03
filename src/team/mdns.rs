//! mDNS Auto-Discovery for Team Mode
//!
//! Uses UDP multicast to discover other TermiMon daemons on the local network.
//! Falls back gracefully if mDNS isn't available.

use anyhow::Result;
use std::net::UdpSocket;
use std::time::Duration;

const MDNS_PORT: u16 = 5353;
const TERMIMON_SERVICE: &[u8] = b"_termimon._tcp.local";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);
const ANNOUNCE_PORT: u16 = 4662;

/// Broadcast presence and discover peers on the local network.
/// This is a simplified mDNS-like discovery using UDP multicast.
pub async fn discover_and_connect() -> Result<()> {
    let cfg = crate::config::load();
    let port = cfg.team.port;
    let name = cfg.team.name.clone();

    println!("📡 Broadcasting as '{name}' on port {port}...");

    // Try to bind multicast socket for discovery
    let socket = UdpSocket::bind("0.0.0.0:0")
        .map_err(|e| anyhow::anyhow!("Failed to create discovery socket: {e}"))?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(DISCOVERY_TIMEOUT))?;

    // Send discovery announcement
    let announce = format!("TERMIMON_DISCOVER:{}:{}", name, port);
    let broadcast_addr = "255.255.255.255:14662";
    socket.send_to(announce.as_bytes(), broadcast_addr)?;
    println!("📡 Sent discovery broadcast...");

    // Also start hosting in the background
    let team_state = crate::team::new_shared_team_state(name.clone());
    crate::team::set_global_team_state(team_state.clone());

    // Listen for responses
    let mut buf = [0u8; 1024];
    let mut found_peers: Vec<String> = Vec::new();

    println!("👂 Listening for peers ({} seconds)...", DISCOVERY_TIMEOUT.as_secs());

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, addr)) => {
                let msg = String::from_utf8_lossy(&buf[..len]);
                if msg.starts_with("TERMIMON_DISCOVER:") || msg.starts_with("TERMIMON_REPLY:") {
                    let parts: Vec<&str> = msg.splitn(3, ':').collect();
                    if parts.len() >= 3 {
                        let peer_name = parts[1];
                        let peer_port: u16 = parts[2].parse().unwrap_or(ANNOUNCE_PORT);
                        if peer_name != name {
                            let peer_addr = format!("{}:{}", addr.ip(), peer_port);
                            println!("🎮 Found peer: {} at {}", peer_name, peer_addr);
                            found_peers.push(peer_addr);

                            // Send reply so they know about us too
                            let reply = format!("TERMIMON_REPLY:{}:{}", name, port);
                            let _ = socket.send_to(reply.as_bytes(), addr);
                        }
                    }
                }
            }
            Err(_) => break, // timeout
        }
    }

    if found_peers.is_empty() {
        println!("\n🔍 No peers found. Starting as host...");
        println!("   Others can join with: termimon team join <your-ip>:{port}");
        println!("   Or run: termimon team auto (on their machine)");

        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ts = team_state.clone();
        crate::team::server::run_team_server(port, ts, shutdown_rx).await?;
    } else {
        println!("\n🎮 Connecting to first discovered peer: {}", found_peers[0]);
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        crate::team::client::connect_to_host(&found_peers[0], team_state, shutdown_rx).await?;
    }

    Ok(())
}
