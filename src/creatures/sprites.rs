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

// ── Rustacean (Rust) ─────────────────────────────────────────────────────────
//
//  An orange crab with claws — cute face, raised pincers
//

// Rust/crab palette
const R1: Color = Color::new(230, 100, 30);   // bright orange
const R2: Color = Color::new(200, 70, 20);    // dark orange
const R3: Color = Color::new(255, 140, 50);   // light orange
const R4: Color = Color::new(180, 50, 10);    // deep red-orange
const R5: Color = Color::new(255, 180, 80);   // highlight / belly
const RE: Color = Color::new(20, 20, 20);     // eyes

pub const RUSTACEAN_IDLE_1: SpriteFrame = [
    //  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    [  T,  T, R4,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T, R4,  T,  T], // 0  claw tips
    [  T, R4, R2, R4,  T,  T,  T,  T,  T,  T,  T,  T, R4, R2, R4,  T], // 1  claws open
    [  T, R2, R4, R2,  T,  T,  T,  T,  T,  T,  T,  T, R2, R4, R2,  T], // 2  claw arms
    [  T,  T, R2, R1,  T,  T,  T,  T,  T,  T,  T,  T, R1, R2,  T,  T], // 3  claw arms
    [  T,  T,  T, R2, R1,  T,  T,  T,  T,  T,  T, R1, R2,  T,  T,  T], // 4  arms to body
    [  T,  T,  T,  T, R2, R1, R1, R1, R1, R1, R1, R2,  T,  T,  T,  T], // 5  shell top
    [  T,  T,  T, R2, R1, R1, R3, R1, R1, R3, R1, R1, R2,  T,  T,  T], // 6  shell
    [  T,  T,  T, R1, R1, RE, RE, R1, R1, RE, RE, R1, R1,  T,  T,  T], // 7  eyes
    [  T,  T,  T, R1, R1, RE, RE, R1, R1, RE, RE, R1, R1,  T,  T,  T], // 8  eyes
    [  T,  T,  T, R1, R1, R1, R1, R5, R1, R1, R1, R1, R1,  T,  T,  T], // 9  mouth
    [  T,  T, R2, R1, R3, R1, R5, R5, R5, R1, R3, R1, R2,  T,  T,  T], // 10 belly
    [  T,  T, R2, R1, R1, R1, R1, R5, R1, R1, R1, R1, R2,  T,  T,  T], // 11 body
    [  T,  T,  T, R4, R2, R1, R1, R1, R1, R1, R2, R4,  T,  T,  T,  T], // 12 bottom
    [  T,  T, R4, R4, T,  R2, R4,  T, R4, R2,  T, R4, R4,  T,  T,  T], // 13 legs
    [  T, R4, R4,  T,  T, R4, R4,  T, R4, R4,  T,  T, R4, R4,  T,  T], // 14 feet
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 15
];

// ── Pythorn (Grass) ──────────────────────────────────────────────────────────
//
//  A green snake/vine creature coiled with leaf details
//

// Grass/vine palette
const G1: Color = Color::new(60, 180, 80);    // bright green
const G2: Color = Color::new(40, 140, 55);    // medium green
const G3: Color = Color::new(30, 100, 40);    // dark green
const G4: Color = Color::new(100, 210, 100);  // light green / highlight
const G5: Color = Color::new(140, 230, 120);  // leaf bright
const G6: Color = Color::new(80, 160, 60);    // leaf dark
const GE: Color = Color::new(200, 50, 50);    // eyes (red, snake-like)

pub const PYTHORN_IDLE_1: SpriteFrame = [
    //  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    [  T,  T,  T,  T,  T,  T, G5,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 0  leaf tip
    [  T,  T,  T,  T,  T, G5, G6, G5,  T,  T,  T,  T,  T,  T,  T,  T], // 1  leaf
    [  T,  T,  T,  T, G6, G5, G2, G6, G5,  T,  T,  T,  T,  T,  T,  T], // 2  leaf base
    [  T,  T,  T,  T,  T, G3, G2, G3,  T,  T,  T,  T,  T,  T,  T,  T], // 3  stem
    [  T,  T,  T,  T, G2, G1, G1, G1, G2,  T,  T,  T,  T,  T,  T,  T], // 4  head top
    [  T,  T,  T, G2, G1, G1, G1, G1, G1, G2,  T,  T,  T,  T,  T,  T], // 5  head
    [  T,  T,  T, G1, GE, GE, G1, GE, GE, G1,  T,  T,  T,  T,  T,  T], // 6  eyes
    [  T,  T,  T, G2, G1, G1, G4, G1, G1, G2,  T,  T,  T,  T,  T,  T], // 7  mouth
    [  T,  T, G3, G2, G1, G1, G1, G1, G1, G2, G3,  T,  T,  T,  T,  T], // 8  neck
    [  T,  T, G2, G1, G1, G4, G1, G4, G1, G1, G2,  T,  T,  T,  T,  T], // 9  coil top
    [  T, G3, G2, G1, G1, G1, G1, G1, G1, G1, G2, G3,  T,  T,  T,  T], // 10 coil
    [  T, G2, G1, G4, G1, G1, G1, G1, G1, G4, G1, G2,  T, G5,  T,  T], // 11 coil + leaf
    [  T,  T, G2, G1, G1, G1, G1, G1, G1, G1, G2, G5, G6, G5,  T,  T], // 12 coil bottom + leaf
    [  T,  T, G3, G2, G2, G1, G1, G1, G2, G2, G3, G5, G6,  T,  T,  T], // 13 tail
    [  T,  T,  T, G3, G3, G2, G2, G2, G3, G3,  T,  T,  T,  T,  T,  T], // 14 tail tip
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T], // 15
];

// ── Gitbat (Dark) ────────────────────────────────────────────────────────────
//
//  A purple bat with spread code-wings — cute face, pointy ears
//

// Dark/bat palette
const D1: Color = Color::new(140, 80, 200);   // bright purple
const D2: Color = Color::new(100, 50, 160);   // medium purple
const D3: Color = Color::new(70, 30, 120);    // dark purple
const D4: Color = Color::new(180, 120, 240);  // light purple / highlight
const D5: Color = Color::new(200, 160, 255);  // wing membrane
const D6: Color = Color::new(50, 20, 90);     // darkest (wing bones)
const DE: Color = Color::new(255, 220, 50);   // eyes (yellow, glowing)

pub const GITBAT_IDLE_1: SpriteFrame = [
    //  0   1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    [  T,  T,  T,  T, D3,  T,  T,  T,  T,  T,  T, D3,  T,  T,  T,  T], // 0  ear tips
    [  T,  T,  T, D3, D2,  T,  T,  T,  T,  T,  T, D2, D3,  T,  T,  T], // 1  ears
    [  T,  T,  T, D2, D1, D2, D1, D1, D1, D2, D1, D2,  T,  T,  T,  T], // 2  head top
    [  T,  T,  T, D1, D1, D1, D1, D1, D1, D1, D1, D1,  T,  T,  T,  T], // 3  head
    [  T,  T,  T, D1, DE, DE, D1, D1, D1, DE, DE, D1,  T,  T,  T,  T], // 4  eyes
    [  T,  T,  T, D2, D1, D1, D1, D4, D1, D1, D1, D2,  T,  T,  T,  T], // 5  mouth (fangs)
    [  T,  T,  T,  T, D2, D1, D4, D4, D4, D1, D2,  T,  T,  T,  T,  T], // 6  chin
    [  T, D6, D3,  T, D3, D2, D1, D1, D1, D2, D3,  T, D3, D6,  T,  T], // 7  wing start + body
    [ D6, D3, D5, D3, D2, D1, D1, D4, D1, D1, D2, D3, D5, D3, D6,  T], // 8  wings spread
    [D6, D5, D5, D5, D3, D2, D1, D1, D1, D2, D3, D5, D5, D5, D6,  T], // 9  wings full
    [  T, D6, D5, D5, D3, D3, D2, D1, D2, D3, D3, D5, D5, D6,  T,  T], // 10 wings lower
    [  T,  T, D6, D5, D5, D3,  T, D2,  T, D3, D5, D5, D6,  T,  T,  T], // 11 wing tips
    [  T,  T,  T, D6, D6,  T,  T, D3,  T,  T, D6, D6,  T,  T,  T,  T], // 12 wing tips
    [  T,  T,  T,  T,  T,  T, D3, D3, D3,  T,  T,  T,  T,  T,  T,  T], // 13 feet
    [  T,  T,  T,  T,  T,  T, D6,  T, D6,  T,  T,  T,  T,  T,  T,  T], // 14 claws
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
        ("Rustacean",   &RUSTACEAN_IDLE_1),
        ("Pythorn",     &PYTHORN_IDLE_1),
        ("Gitbat",      &GITBAT_IDLE_1),
    ]
}

/// Get the idle sprite for a creature by species name.
pub fn sprite_for_species(name: &str) -> &'static SpriteFrame {
    match name.to_lowercase().as_str() {
        "embercli"    => &EMBERCLI_IDLE_1,
        "voltprompt"  => &VOLTPROMPT_IDLE_1,
        "shelloise"   => &SHELLOISE_IDLE_1,
        "rustacean"   => &RUSTACEAN_IDLE_1,
        "pythorn"     => &PYTHORN_IDLE_1,
        "gitbat"      => &GITBAT_IDLE_1,
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
    } else if lower.contains("cursor") {
        &RUSTACEAN_IDLE_1
    } else if lower.contains("continue") || lower.contains("copilot") {
        &PYTHORN_IDLE_1
    } else {
        &GITBAT_IDLE_1
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
    } else if lower.contains("cursor") {
        "rustacean"
    } else if lower.contains("continue") || lower.contains("copilot") {
        "pythorn"
    } else {
        "gitbat"
    }
}
