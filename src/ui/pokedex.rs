//! Pokédex — creature collection display
//!
//! Shows all available TermiMon creatures with their pixel art sprites,
//! element info, evolution stages, and descriptions.
//! Optionally shows which ones are currently active via daemon query.

use anyhow::Result;

use crate::creatures::registry;
use crate::creatures::sprites;
use crate::daemon::server;
use crate::render::halfblock;

pub async fn show() -> Result<()> {
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           📖 TermiMon Pokédex — Creature Collection         ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // Try to get active agents from daemon (best effort)
    let active_agents = get_active_agents().await;

    let all = registry::all_creatures();
    for (idx, creature_def) in all.iter().enumerate() {
        let sprite = sprites::sprite_for_species(creature_def.name);
        let sprite_lines = halfblock::render_sprite(sprite);
        let _boxed_lines = halfblock::render_sprite_boxed(sprite, creature_def.evolution_names[0]);

        // Check if this creature type is currently active
        let active_count = active_agents
            .iter()
            .filter(|a| {
                sprites::species_for_agent(&a.kind) == creature_def.name
            })
            .count();

        let active_str = if active_count > 0 {
            format!("  ✅ {} active now!", active_count)
        } else {
            "  ⬜ Not active".to_string()
        };

        // Print number
        println!("  ┌─── #{:03} ───────────────────────────────────────────────┐", idx + 1);
        println!("  │");

        // Print sprite alongside info
        let info_lines = build_info_lines(creature_def, &active_str);

        let max_lines = sprite_lines.len().max(info_lines.len());
        for i in 0..max_lines {
            let sprite_part = if i < sprite_lines.len() {
                format!("    {}", &sprite_lines[i])
            } else {
                "                    ".to_string()
            };
            let info_part = if i < info_lines.len() {
                &info_lines[i]
            } else {
                ""
            };
            // Sprite on left (pad to ~22 visible chars), info on right
            println!("  │ {}   {}", sprite_part, info_part);
        }

        println!("  │");
        println!("  └─────────────────────────────────────────────────────────┘");
        println!();
    }

    // Summary
    let total = all.len();
    let active_species: Vec<String> = active_agents
        .iter()
        .map(|a| sprites::species_for_agent(&a.kind).to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    println!("  📊 Collection: {}/{} species discovered", active_species.len(), total);
    if !active_agents.is_empty() {
        println!("  🔥 {} agents currently running", active_agents.len());
    }
    println!();

    Ok(())
}

fn build_info_lines(def: &crate::creatures::CreatureDef, active_str: &str) -> Vec<String> {
    let elem = def.element;
    vec![
        format!(
            "\x1b[1m{}\x1b[0m  {}",
            def.evolution_names[0],
            elem,
        ),
        String::new(),
        format!("  Type: {}", elem),
        format!("  Default Agent: {}", def.default_agent),
        String::new(),
        "  Evolution:".to_string(),
        format!(
            "    ★☆☆  {} → ★★☆  {} → ★★★  {}",
            def.evolution_names[0],
            def.evolution_names[1],
            def.evolution_names[2],
        ),
        String::new(),
        format!("  \"{}\"", def.description),
        String::new(),
        active_str.to_string(),
    ]
}

async fn get_active_agents() -> Vec<server::AgentSnapshot> {
    match server::client_request("status").await {
        Ok(response) => {
            match serde_json::from_str::<server::StatusResponse>(response.trim()) {
                Ok(status) => status.agents,
                Err(_) => Vec::new(),
            }
        }
        Err(_) => Vec::new(),
    }
}
