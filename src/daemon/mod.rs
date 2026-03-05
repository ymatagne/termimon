//! Daemon lifecycle management
//!
//! start/stop/status via PID file, Unix socket IPC, and heartbeat loop.

pub mod heartbeat;
pub mod notify;
pub mod server;

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Runtime directory for daemon files.
pub fn runtime_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".termimon")
        .join("run");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn pid_file_path() -> PathBuf {
    runtime_dir().join("termimon.pid")
}

pub fn socket_path() -> PathBuf {
    runtime_dir().join("termimon.sock")
}

/// Read PID from file if the process is still alive.
pub fn read_running_pid() -> Option<u32> {
    let content = std::fs::read_to_string(pid_file_path()).ok()?;
    let pid: u32 = content.trim().parse().ok()?;
    if is_process_alive(pid) {
        Some(pid)
    } else {
        let _ = std::fs::remove_file(pid_file_path());
        None
    }
}

fn write_pid_file() -> Result<()> {
    std::fs::write(pid_file_path(), std::process::id().to_string())
        .context("Failed to write PID file")
}

fn remove_pid_file() {
    let _ = std::fs::remove_file(pid_file_path());
}

fn remove_socket_file() {
    let _ = std::fs::remove_file(socket_path());
}

fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Start the daemon.
pub async fn start(foreground: bool) -> Result<()> {
    if let Some(pid) = read_running_pid() {
        println!("🎮 TermiMon daemon is already running (PID {pid})");
        return Ok(());
    }

    if foreground {
        println!("🎮 TermiMon daemon starting in foreground...");
        run_daemon().await
    } else {
        println!("🎮 Starting TermiMon daemon...");
        let exe = std::env::current_exe().context("Could not determine executable path")?;
        let log_path = runtime_dir().join("termimon.log");
        let log_file = std::fs::File::create(&log_path)
            .context("Failed to create log file")?;
        let log_err = log_file.try_clone().context("Failed to clone log file")?;
        let mut cmd = std::process::Command::new(exe);
        cmd.args(["start", "--foreground"])
            .stdin(std::process::Stdio::null())
            .stdout(log_file)
            .stderr(log_err);
        // Preserve TMUX env var so daemon can talk to tmux server
        if let Ok(tmux_env) = std::env::var("TMUX") {
            cmd.env("TMUX", &tmux_env);
        }
        if let Ok(tmux_tmpdir) = std::env::var("TMUX_TMPDIR") {
            cmd.env("TMUX_TMPDIR", &tmux_tmpdir);
        }
        let child = cmd.spawn()
            .context("Failed to spawn daemon process")?;
        println!("🎮 TermiMon daemon started (PID {})", child.id());
        Ok(())
    }
}

/// Stop the daemon.
pub async fn stop() -> Result<()> {
    match read_running_pid() {
        Some(pid) => {
            println!("🛑 Stopping TermiMon daemon (PID {pid})...");
            unsafe { libc::kill(pid as i32, libc::SIGTERM); }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if is_process_alive(pid) {
                unsafe { libc::kill(pid as i32, libc::SIGKILL); }
            }
            remove_pid_file();
            remove_socket_file();
            println!("✅ TermiMon daemon stopped.");
        }
        None => {
            println!("🎮 TermiMon daemon is not running.");
        }
    }
    Ok(())
}

/// Show daemon status.
pub async fn status() -> Result<()> {
    match read_running_pid() {
        Some(pid) => {
            println!("🎮 TermiMon daemon is running (PID {pid})");
            match server::client_request("status").await {
                Ok(response) => {
                    // Try to parse as JSON StatusResponse
                    match serde_json::from_str::<server::StatusResponse>(&response.trim()) {
                        Ok(status) => {
                            // Calculate uptime
                            let uptime_str = if let Ok(started) = chrono::DateTime::parse_from_rfc3339(&status.started_at) {
                                let elapsed = chrono::Utc::now().signed_duration_since(started);
                                let secs = elapsed.num_seconds();
                                if secs >= 3600 {
                                    format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
                                } else if secs >= 60 {
                                    format!("{}m {}s", secs / 60, secs % 60)
                                } else {
                                    format!("{}s", secs)
                                }
                            } else {
                                "unknown".to_string()
                            };

                            println!("  Uptime: {} | Heartbeats: {}", uptime_str, status.heartbeat_count);
                            println!();

                            if status.agents.is_empty() {
                                println!("  No agents detected yet. Start an AI coding agent!");
                            } else {
                                for agent in &status.agents {
                                    let pid_str = agent.pid.map(|p| format!(" pid:{}", p)).unwrap_or_default();
                                    let dir_str = agent.working_dir.as_deref().unwrap_or("?");
                                    println!(
                                        "  {} {} (Stage {}) — {} [{}]{}",
                                        agent.element_icon,
                                        agent.creature_name,
                                        agent.stage,
                                        agent.kind,
                                        agent.state,
                                        pid_str,
                                    );
                                    println!(
                                        "    ID: {} | CPU: {:.1}% | Mem: {:.1} MB | Dir: {}",
                                        agent.agent_id,
                                        agent.cpu_pct,
                                        agent.mem_mb,
                                        dir_str,
                                    );
                                }
                            }

                            println!();
                            println!(
                                "  {} agents tracked | {} XP earned",
                                status.agents.len(),
                                status.total_xp,
                            );
                        }
                        Err(_) => {
                            // Fallback: print raw response
                            println!("{response}");
                        }
                    }
                }
                Err(_) => println!("  (could not connect to daemon socket)"),
            }
        }
        None => {
            println!("😴 TermiMon daemon is not running.");
            println!("   Start it with: termimon start");
        }
    }
    Ok(())
}

/// The main daemon loop.
async fn run_daemon() -> Result<()> {
    write_pid_file()?;
    remove_socket_file();

    tracing::info!("TermiMon daemon starting (PID {})", std::process::id());

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // Signal handling
    let tx = shutdown_tx.clone();
    tokio::spawn(async move {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to register SIGTERM");
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                .expect("Failed to register SIGINT");
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
        tracing::info!("Shutdown signal received");
        let _ = tx.send(true);
    });

    // Initialize team state in the daemon
    let cfg = crate::config::load();
    let team_state = crate::team::new_shared_team_state(cfg.team.name.clone());
    crate::team::set_global_team_state(team_state.clone());

    let (team_shutdown_tx, _team_shutdown_rx) = tokio::sync::watch::channel(false);

    let server_shutdown = shutdown_rx.clone();
    let ts = team_state.clone();
    let ttx = team_shutdown_tx.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::run_server(server_shutdown, ts, ttx).await {
            tracing::error!("IPC server error: {e}");
        }
    });

    let hb_shutdown = shutdown_rx.clone();
    let heartbeat_handle = tokio::spawn(async move {
        heartbeat::run_heartbeat(hb_shutdown).await;
    });

    // Wait for shutdown
    let mut wait = shutdown_rx.clone();
    let _ = wait.changed().await;

    tracing::info!("Shutting down...");
    let _ = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let _ = server_handle.await;
        let _ = heartbeat_handle.await;
    })
    .await;

    remove_pid_file();
    remove_socket_file();
    let _ = crate::tmux::status::clear_status();

    tracing::info!("TermiMon daemon stopped.");
    Ok(())
}
