//! Global application state management

use anyhow::Result;

/// Assign a creature to a tmux pane.
pub async fn assign_creature(pane: &str, creature: &str) -> Result<()> {
    tracing::info!("Assigning creature '{creature}' to pane '{pane}'");
    println!("🎮 Assigned {creature} to pane {pane}");
    Ok(())
}
