//! # TermiMon 🎮
//!
//! Your AI agents, alive in the terminal.
//! Pixel creature companions for tmux that react to what your AI coding agents are doing.

#[allow(dead_code)]
mod agents;
mod config;
#[allow(dead_code)]
mod creatures;
mod daemon;
mod plugin;
#[allow(dead_code)]
mod render;
mod sound;
mod state;
mod stats;
mod team;
mod theme;
#[allow(dead_code)]
mod tmux;
mod ui;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "termimon",
    about = "🎮 TermiMon — Your AI agents, alive in the terminal",
    version,
    long_about = "Pixel creature companions for tmux that react to what your AI coding agents are doing.\n\nGotta spawn 'em all. 🔥⚡💧"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the TermiMon daemon
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,
    },

    /// Stop the TermiMon daemon
    Stop,

    /// Show status of all tracked agents and creatures
    Status,

    /// Open the interactive dashboard (tmux popup)
    Dash,

    /// Show your creature collection and stats
    Pokedex,

    /// Assign a creature to a tmux pane
    Assign {
        /// Pane ID (e.g., %0, %1)
        #[arg(short, long)]
        pane: String,

        /// Creature name (e.g., embercli, voltprompt, shelloise)
        #[arg(short, long)]
        creature: String,
    },

    /// View or edit configuration
    Config {
        /// Open config file in $EDITOR
        #[arg(short, long)]
        edit: bool,

        /// Config file path
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Switch to an agent's tmux pane
    Switch {
        /// Agent number (1-based). If omitted, shows interactive list.
        number: Option<usize>,
    },

    /// Add tmux key binding (prefix+P) to toggle the dashboard
    Bind,

    /// Remove tmux key binding
    Unbind,

    /// Show session history and cost breakdown
    History {
        /// Number of days to show (default: 7)
        #[arg(short, long, default_value = "7")]
        days: u32,
    },

    /// Team mode — connect with other TermiMon instances
    Team {
        #[command(subcommand)]
        action: TeamAction,
    },

    /// Manage color themes
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
}

#[derive(Subcommand)]
enum TeamAction {
    /// Host a team session (start listening for peers)
    Host {
        /// Port to listen on (default: from config or 4662)
        #[arg(short, long)]
        port: Option<u16>,
    },

    /// Join an existing team session
    Join {
        /// Host address (ip:port)
        addr: String,
    },

    /// Auto-discover peers on local network via mDNS
    Auto,

    /// Show team connection status
    Status,

    /// Leave the current team session
    Leave,
}

#[derive(Subcommand)]
enum ThemeAction {
    /// List available themes
    List,

    /// Set the active theme
    Set {
        /// Theme name (default, retro, neon, pastel)
        name: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    match cli.command {
        Commands::Start { foreground } => {
            daemon::start(foreground).await?;
        }
        Commands::Stop => {
            daemon::stop().await?;
        }
        Commands::Status => {
            daemon::status().await?;
        }
        Commands::Dash => {
            ui::dashboard::run().await?;
        }
        Commands::Pokedex => {
            ui::pokedex::show().await?;
        }
        Commands::Assign { pane, creature } => {
            state::assign_creature(&pane, &creature).await?;
        }
        Commands::Config { edit, path } => {
            config::handle_config(edit, path).await?;
        }
        Commands::Switch { number } => {
            ui::dashboard::switch_command(number).await?;
        }
        Commands::Bind => {
            tmux::bind::bind_hotkey()?;
        }
        Commands::Unbind => {
            tmux::bind::unbind_hotkey()?;
        }
        Commands::History { days } => {
            stats::show_history(days)?;
        }
        Commands::Team { action } => {
            let cfg = config::load();
            match action {
                TeamAction::Host { port } => {
                    let port = port.unwrap_or(cfg.team.port);
                    let team_state = team::new_shared_team_state(cfg.team.name.clone());
                    team::set_global_team_state(team_state.clone());
                    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                    println!("🎮 Hosting team session on port {port}...");
                    println!("   Others can join with: termimon team join <your-ip>:{port}");
                    team::server::run_team_server(port, team_state, shutdown_rx).await?;
                }
                TeamAction::Join { addr } => {
                    let team_state = team::new_shared_team_state(cfg.team.name.clone());
                    team::set_global_team_state(team_state.clone());
                    let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                    println!("🎮 Joining team at {addr}...");
                    team::client::connect_to_host(&addr, team_state, shutdown_rx).await?;
                }
                TeamAction::Auto => {
                    println!("🔍 Searching for TermiMon peers on local network...");
                    match team::mdns::discover_and_connect().await {
                        Ok(()) => {}
                        Err(e) => {
                            println!("⚠️  mDNS discovery not available: {e}");
                            println!("   Falling back to manual mode.");
                            println!("   Host:  termimon team host");
                            println!("   Join:  termimon team join <ip:port>");
                        }
                    }
                }
                TeamAction::Status => {
                    println!("🎮 Team Status");
                    if let Some(ts) = team::get_global_team_state() {
                        if let Ok(state) = ts.lock() {
                            if state.hosting {
                                println!("  Hosting on port {}", cfg.team.port);
                            } else if state.connected {
                                println!("  Connected to team");
                            } else {
                                println!("  Not connected");
                            }
                            let peers = state.registry.peer_names();
                            if peers.is_empty() {
                                println!("  No peers connected");
                            } else {
                                println!("  Peers: {}", peers.join(", "));
                            }
                        }
                    } else {
                        println!("  Not connected to any team.");
                        println!("  Host:  termimon team host");
                        println!("  Join:  termimon team join <ip:port>");
                    }
                }
                TeamAction::Leave => {
                    println!("👋 Leaving team...");
                    // Signal shutdown to team connections
                    if let Some(ts) = team::get_global_team_state() {
                        if let Ok(mut state) = ts.lock() {
                            state.connected = false;
                            state.hosting = false;
                            state.registry.peers.clear();
                        }
                    }
                    println!("✅ Left team.");
                }
            }
        }
        Commands::Theme { action } => {
            match action {
                ThemeAction::List => {
                    theme::list_themes();
                }
                ThemeAction::Set { name } => {
                    theme::set_theme(&name)?;
                }
            }
        }
    }

    Ok(())
}
