//! # TermiMon 🎮
//!
//! Your AI agents, alive in the terminal.
//! Pixel creature companions for tmux that react to what your AI coding agents are doing.

mod agents;
mod config;
mod creatures;
mod daemon;
mod render;
mod state;
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
    }

    Ok(())
}
