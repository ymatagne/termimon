//! Daemon lifecycle management
//!
//! start/stop/status via PID file, Unix socket IPC, and heartbeat loop.

pub mod heartbeat;
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
        let child = std::process::Command::new(exe)
            .args(["start", "--foreground"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
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
                Ok(response) => println!("{response}"),
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

    let server_shutdown = shutdown_rx.clone();
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server::run_server(server_shutdown).await {
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
