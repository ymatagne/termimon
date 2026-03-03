//! Sprite data for all TermiMon creatures — 16×16 pixel art
//!
//! Each creature has at least one idle frame. Sprites use the Color type
//! from the parent module, with transparent() for empty pixels.

use super::{Color, SpriteFrame};

// ── Color palette ────────────────────────────────────────────────────────────

const T: Color = Color::transparent();

// Fire palette
const F1: Color = Color::new(255, 80, 20);    // bright orange-red
const F2: Color = Color::new(255, 160, 30);   // orange
const F3: Color = Color::new(255, 220, 60);   // yellow
const F4: Color = Color::new(200, 40, 10);    // dark red
const F5: Color = Color::new(180, 30, 5);     // darker red
const FE: Color = Color::new(40, 40, 40);     // eyes (dark)

// Electric palette
const E1: Color = Color::new(255, 230, 50);   // bright yellow
const E2: Color = Color::new(255, 200, 0);    // gold
const E3: Color = Color::new(200, 160, 0);    // dark gold
const E4: Color = Color::new(100, 200, 255);  // electric blue
const E5: Color = Color::new(60, 150, 230);   // darker blue
const EE: Color = Color::new(30, 30, 30);     // eyes

// Water palette
const W1: Color = Color::new(60, 160, 230);   // bright blue
const W2: Color = Color::new(40, 120, 200);   // medium blue
const W3: Color = Color::new(30, 90, 160);    // dark blue
const W4: Color = Color::new(140, 200, 240);  // light blue
const W5: Color = Color::new(100, 180, 100);  // green (shell)
const W6: Color = Color::new(70, 140, 70);    // dark green
const WE: Color = Color::new(30, 30, 30);     // eyes
const WW: Color = Color::new(230, 230, 240);  // white belly

// ── Embercli (Fire) ──────────────────────────────────────────────────────────
//
//  A flame spirit — round body with flickering fire on top
//
pub const EMBERCLI_IDLE_1: SpriteFrame = [
    //  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    [  T,  T,  T,  T,  T,  T,  T, F3,  T,  T,  T,  T,  T,  T,  T,  T], // 0  flame tip
    [  T,  T,  T,  T,  T,  T, F3, F3, F3,  T,  T,  T,  T,  T,  T,  T], // 1
    [  T,  T,  T,  T,  T, F3, F2, F3, F2, F3,  T,  T,  T,  T,  T,  T], // 2
    [  T,  T,  T,  T, F2, F2, F1, F2, F1, F2, F2,  T,  T,  T,  T,  T], // 3
    [  T,  T,  T,  T, F1, F1, F1, F1, F1, F1, F1,  T,  T,  T,  T,  T], // 4  flame base
    [  T,  T,  T, F4, F4, F1, F1, F1, F1, F1, F4, F4,  T,  T,  T,  T], // 5  head top
    [  T,  T, F4, F1, F1, F1, F1, F1, F1, F1, F1, F1, F4,  T,  T,  T], // 6
    [  T,  T, F1, F1, FE, FE, F1, F1, F1, FE, FE, F1, F1,  T,  T,  T], // 7  eyes
    [  T,  T, F1, F1, FE, FE, F1, F1, F1, FE, FE, F1, F1,  T,  T,  T], // 8
    [  T,  T, F1, F1, F1, F1, F1, F2, F1, F1, F1, F1, F1,  T,  T,  T], // 9  mouth
    [  T,  T, F4, F1, F1, F1, F2, F3, F2, F1, F1, F1, F4,  T,  T,  T], // 10
    [  T,  T,  T, F4, F1, F1, F1, F1, F1, F1, F1, F4,  T,  T,  T,  T], // 11 body
    [  T,  T,  T,  T, F5, F4, F1, F1, F1, F4, F5,  T,  T,  T,  T,  T], // 12
    [  T,  T,  T, F5, F5,  T, F4, F4, F4,  T, F5, F5,  T,  T,  T,  T], // 13 feet
    [  T,  T, F5, F5,  T,  T,  T,  T,  T,  T,  T, F5, F5,  T,  T,  T], // 14
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 15
];

// ── Voltprompt (Electric) ────────────────────────────────────────────────────
//
//  An electric rodent — spiky ears, lightning bolt tail
//
pub const VOLTPROMPT_IDLE_1: SpriteFrame = [
    //  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    [  T,  T, E2,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T, E2,  T,  T], // 0  ear tips
    [  T,  T, E2, E1,  T,  T,  T,  T,  T,  T,  T,  T, E1, E2,  T,  T], // 1
    [  T,  T, E3, E1, E1,  T,  T,  T,  T,  T,  T, E1, E1, E3,  T,  T], // 2  ears
    [  T,  T,  T, E2, E1, E1, E1, E1, E1, E1, E1, E1, E2,  T,  T,  T], // 3  head top
    [  T,  T,  T, E1, E1, E1, E1, E1, E1, E1, E1, E1, E1,  T,  T,  T], // 4
    [  T,  T,  T, E1, EE, EE, E1, E1, E1, EE, EE, E1, E1,  T,  T,  T], // 5  eyes
    [  T,  T,  T, E1, EE, EE, E1, E1, E1, EE, EE, E1, E1,  T,  T,  T], // 6
    [  T,  T,  T, E1, E1, E1, E4, E1, E4, E1, E1, E1, E1,  T,  T,  T], // 7  cheeks (blue)
    [  T,  T,  T, E2, E1, E1, E1, E2, E1, E1, E1, E1, E2,  T,  T,  T], // 8  mouth
    [  T,  T,  T,  T, E2, E1, E1, E1, E1, E1, E1, E2,  T,  T,  T,  T], // 9
    [  T,  T,  T,  T, E3, E2, E1, E1, E1, E2, E3,  T,  T,  T,  T,  T], // 10 body
    [  T,  T,  T,  T, E2, E2, E1, E1, E1, E2, E2,  T,  T, E1,  T,  T], // 11
    [  T,  T,  T, E3, E3,  T, E2, E2, E2,  T, E3, E3, E1, E1,  T,  T], // 12 feet + tail start
    [  T,  T, E3, E3,  T,  T,  T,  T,  T,  T,  T, E1, E1,  T,  T,  T], // 13 tail bolt
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T, E1, E1,  T,  T,  T,  T], // 14
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 15
];

// ── Shelloise (Water) ────────────────────────────────────────────────────────
//
//  A turtle with a round shell — cute face, green shell top, blue body
//
pub const SHELLOISE_IDLE_1: SpriteFrame = [
    //  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 0
    [  T,  T,  T,  T,  T, W1, W1, W1, W1, W1, W1,  T,  T,  T,  T,  T], // 1  head top
    [  T,  T,  T,  T, W1, W1, W1, W1, W1, W1, W1, W1,  T,  T,  T,  T], // 2
    [  T,  T,  T, W1, W1, WE, WE, W1, W1, WE, WE, W1, W1,  T,  T,  T], // 3  eyes
    [  T,  T,  T, W1, W1, WE, WE, W1, W1, WE, WE, W1, W1,  T,  T,  T], // 4
    [  T,  T,  T, W1, W1, W1, W1, W4, W1, W1, W1, W1, W1,  T,  T,  T], // 5  smile
    [  T,  T,  T,  T, W2, W1, W4, W4, W4, W1, W2,  T,  T,  T,  T,  T], // 6
    [  T,  T, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5,  T,  T], // 7  shell top
    [  T, W5, W6, W5, W6, W5, W6, W5, W6, W5, W6, W5, W6, W5, W5,  T], // 8  shell pattern
    [  T, W5, W5, W6, W5, W6, W5, W6, W5, W6, W5, W6, W5, W6, W5,  T], // 9
    [  T, W6, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5, W6,  T], // 10
    [  T,  T, W6, W5, W5, W5, W5, W5, W5, W5, W5, W5, W5, W6,  T,  T], // 11 shell bottom
    [  T,  T,  T, W6, W6, W6, W6, W6, W6, W6, W6, W6, W6,  T,  T,  T], // 12
    [  T,  T, W2, W2,  T,  T,  T,  T,  T,  T,  T,  T, W2, W2,  T,  T], // 13 feet
    [  T, W3, W3,  T,  T,  T,  T, W2,  T,  T,  T,  T,  T, W3, W3,  T], // 14 tail
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 15
];

// ── Backward compatibility alias ─────────────────────────────────────────────
pub const EMBERCLI_IDLE: SpriteFrame = EMBERCLI_IDLE_1;

// ── Lookup helpers ───────────────────────────────────────────────────────────

/// Return all idle sprites as (name, sprite) pairs.
pub fn all_idle_sprites() -> Vec<(&'static str, &'static SpriteFrame)> {
    vec![
        ("Embercli",    &EMBERCLI_IDLE_1),
        ("Voltprompt",  &VOLTPROMPT_IDLE_1),
        ("Shelloise",   &SHELLOISE_IDLE_1),
    ]
}

/// Get the idle sprite for a creature by species name.
pub fn sprite_for_species(name: &str) -> &'static SpriteFrame {
    match name.to_lowercase().as_str() {
        "embercli"    => &EMBERCLI_IDLE_1,
        "voltprompt"  => &VOLTPROMPT_IDLE_1,
        "shelloise"   => &SHELLOISE_IDLE_1,
        _ => &EMBERCLI_IDLE_1, // default fallback
    }
}

/// Get the idle sprite for an agent kind string (e.g. "Claude Code" → embercli).
pub fn sprite_for_agent(agent_kind: &str) -> &'static SpriteFrame {
    let lower = agent_kind.to_lowercase();
    if lower.contains("claude") {
        &EMBERCLI_IDLE_1
    } else if lower.contains("codex") {
        &VOLTPROMPT_IDLE_1
    } else if lower.contains("aider") {
        &SHELLOISE_IDLE_1
    } else {
        &EMBERCLI_IDLE_1
    }
}

/// Map agent kind to creature species name.
pub fn species_for_agent(agent_kind: &str) -> &'static str {
    let lower = agent_kind.to_lowercase();
    if lower.contains("claude") {
        "embercli"
    } else if lower.contains("codex") {
        "voltprompt"
    } else if lower.contains("aider") {
        "shelloise"
    } else {
        "embercli"
    }
}
