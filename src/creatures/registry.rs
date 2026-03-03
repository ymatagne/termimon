//! Creature registry — lookup definitions by name.

use super::{CreatureDef, Element};

const CREATURES: &[CreatureDef] = &[
    CreatureDef {
        name: "embercli",
        element: Element::Fire,
        description: "A fiery CLI companion born from Claude Code sessions.",
        default_agent: "claude",
        evolution_names: ["Embercli", "Blazecli", "Infernocli"],
    },
    CreatureDef {
        name: "voltprompt",
        element: Element::Electric,
        description: "An electric creature that thrives on Codex completions.",
        default_agent: "codex",
        evolution_names: ["Voltprompt", "Sparkprompt", "Thunderprompt"],
    },
    CreatureDef {
        name: "shelloise",
        element: Element::Water,
        description: "A water creature that flows with aider's pair-programming.",
        default_agent: "aider",
        evolution_names: ["Shelloise", "Torrentoise", "Tsunamoise"],
    },
    CreatureDef {
        name: "rustacean",
        element: Element::Rust,
        description: "A fearless crab forged in Cursor's rapid-fire completions.",
        default_agent: "cursor",
        evolution_names: ["Rustacean", "Ferrocrab", "Oxidragon"],
    },
    CreatureDef {
        name: "pythorn",
        element: Element::Grass,
        description: "A vine creature that grows alongside Copilot and Continue sessions.",
        default_agent: "copilot",
        evolution_names: ["Pythorn", "Vineconda", "Thornviper"],
    },
    CreatureDef {
        name: "gitbat",
        element: Element::Dark,
        description: "A shadowy bat that emerges from unknown agent processes.",
        default_agent: "unknown",
        evolution_names: ["Gitbat", "Commitwing", "Mergefiend"],
    },
];

pub fn get_creature_def(name: &str) -> Option<&'static CreatureDef> {
    let lower = name.to_lowercase();
    CREATURES.iter().find(|c| c.name.to_lowercase() == lower)
}

pub fn all_creatures() -> &'static [CreatureDef] {
    CREATURES
}

pub fn creature_for_agent(agent: &str) -> Option<&'static CreatureDef> {
    let lower = agent.to_lowercase();
    CREATURES.iter().find(|c| lower.contains(c.default_agent))
}
