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

// ── Neuromorph (Psychic) ─────────────────────────────────────────────────────
//
//  A purple/pink brain creature with psionic waves
//

const P1: Color = Color::new(200, 100, 220);  // bright purple-pink
const P2: Color = Color::new(160, 60, 180);   // medium purple
const P3: Color = Color::new(120, 30, 140);   // dark purple
const P4: Color = Color::new(240, 150, 255);  // light pink highlight
const P5: Color = Color::new(255, 180, 230);  // brain fold highlight
const PE: Color = Color::new(255, 255, 100);  // eyes (yellow glow)

pub const NEUROMORPH_IDLE_1: SpriteFrame = [
    [  T,  T,  T,  T,  T, P4,  T, P4,  T, P4,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T, P4,  T, P4,  T, P4,  T, P4,  T,  T,  T,  T,  T],
    [  T,  T,  T, P3, P2, P1, P5, P1, P5, P1, P2, P3,  T,  T,  T,  T],
    [  T,  T, P3, P2, P5, P1, P5, P5, P5, P1, P5, P2, P3,  T,  T,  T],
    [  T,  T, P2, P1, P5, P5, P1, P5, P1, P5, P5, P1, P2,  T,  T,  T],
    [  T, P3, P1, P5, P1, P5, P5, P1, P5, P5, P1, P5, P1, P3,  T,  T],
    [  T, P3, P1, P5, P5, P1, P5, P5, P5, P1, P5, P5, P1, P3,  T,  T],
    [  T, P2, P1, P1, P1, P1, P1, P1, P1, P1, P1, P1, P1, P2,  T,  T],
    [  T, P2, P1, PE, PE, P1, P1, P1, P1, PE, PE, P1, P1, P2,  T,  T],
    [  T, P3, P1, PE, PE, P1, P1, P4, P1, PE, PE, P1, P1, P3,  T,  T],
    [  T, P3, P2, P1, P1, P1, P4, P4, P4, P1, P1, P1, P2, P3,  T,  T],
    [  T,  T, P3, P2, P1, P1, P1, P1, P1, P1, P1, P2, P3,  T,  T,  T],
    [  T,  T,  T, P3, P2, P2, P1, P1, P1, P2, P2, P3,  T,  T,  T,  T],
    [  T,  T,  T,  T, P3, P3,  T, P3,  T, P3, P3,  T,  T,  T,  T,  T],
    [  T,  T,  T, P3, P3,  T,  T,  T,  T,  T, P3, P3,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
];

// ── Dockersaur (Steel) ──────────────────────────────────────────────────────
//
//  A metallic dinosaur with container markings
//

const S1: Color = Color::new(160, 170, 190);  // bright steel
const S2: Color = Color::new(120, 130, 150);  // medium steel
const S3: Color = Color::new(80, 90, 110);    // dark steel
const S4: Color = Color::new(200, 210, 230);  // highlight
const S5: Color = Color::new(60, 140, 220);   // docker blue accent
const SE: Color = Color::new(30, 30, 30);     // eyes

pub const DOCKERSAUR_IDLE_1: SpriteFrame = [
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T, S3, S2, S1, S1, S2, S3,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T, S3, S1, S1, S4, S1, S1, S1, S3,  T,  T,  T,  T],
    [  T,  T,  T,  T, S1, SE, SE, S1, S1, SE, SE, S1,  T,  T,  T,  T],
    [  T,  T,  T,  T, S1, SE, SE, S1, S1, SE, SE, S1,  T,  T,  T,  T],
    [  T,  T,  T,  T, S2, S1, S1, S4, S4, S1, S1, S2,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T, S3, S2, S1, S2, S3,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T, S3, S2, S5, S5, S5, S5, S5, S2, S3,  T,  T,  T,  T],
    [  T,  T, S3, S2, S5, S4, S5, S4, S5, S4, S5, S2, S3,  T,  T,  T],
    [  T,  T, S2, S1, S5, S5, S5, S5, S5, S5, S5, S1, S2,  T,  T,  T],
    [  T,  T, S2, S1, S1, S1, S1, S1, S1, S1, S1, S1, S2,  T,  T,  T],
    [  T,  T, S3, S2, S1, S1, S4, S1, S4, S1, S1, S2, S3,  T,  T,  T],
    [  T,  T,  T, S3, S2, S1, S1, S1, S1, S1, S2, S3,  T,  T,  T,  T],
    [  T,  T, S3, S3,  T, S3, S2, S1, S2, S3,  T, S3, S3,  T,  T,  T],
    [  T, S3, S3,  T,  T,  T, S3,  T, S3,  T,  T,  T, S3, S3,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
];

// ── Termignite (Fire/Dark) ──────────────────────────────────────────────────
//
//  A dark flame creature with terminal symbols
//

const TG1: Color = Color::new(200, 60, 20);   // dark red-orange
const TG2: Color = Color::new(160, 30, 10);   // deep dark red
const TG3: Color = Color::new(100, 15, 5);    // very dark
const TG4: Color = Color::new(255, 100, 30);  // bright flame
const TG5: Color = Color::new(255, 200, 50);  // flame tip
const TGE: Color = Color::new(0, 255, 0);     // green terminal eyes

pub const TERMIGNITE_IDLE_1: SpriteFrame = [
    [  T,  T,  T,  T,  T,  T, TG5,  T,  T, TG5,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T, TG5, TG4, TG5, TG5, TG4, TG5,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T, TG4, TG4, TG1, TG4, TG4, TG1, TG4, TG4,  T,  T,  T,  T],
    [  T,  T,  T, TG3, TG2, TG1, TG1, TG1, TG1, TG1, TG2, TG3,  T,  T,  T,  T],
    [  T,  T, TG3, TG2, TG1, TG1, TG1, TG1, TG1, TG1, TG1, TG2, TG3,  T,  T,  T],
    [  T,  T, TG2, TG1, TGE, TGE, TG1, TG1, TG1, TGE, TGE, TG1, TG2,  T,  T,  T],
    [  T,  T, TG2, TG1, TGE, TGE, TG1, TG1, TG1, TGE, TGE, TG1, TG2,  T,  T,  T],
    [  T,  T, TG3, TG1, TG1, TG1, TG4, TG1, TG4, TG1, TG1, TG1, TG3,  T,  T,  T],
    [  T,  T, TG3, TG2, TG1, TG1, TG1, TG4, TG1, TG1, TG1, TG2, TG3,  T,  T,  T],
    [  T,  T,  T, TG3, TG2, TG1, TG1, TG1, TG1, TG1, TG2, TG3,  T,  T,  T,  T],
    [  T,  T,  T, TG3, TG2, TG2, TG1, TG1, TG1, TG2, TG2, TG3,  T,  T,  T,  T],
    [  T,  T, TG4, TG3, TG3, TG2, TG2, TG2, TG2, TG2, TG3, TG3, TG4,  T,  T,  T],
    [  T, TG4, TG1,  T,  T, TG3, TG3, TG3, TG3, TG3,  T,  T, TG1, TG4,  T,  T],
    [  T,  T,  T,  T, TG3, TG3,  T,  T,  T, TG3, TG3,  T,  T,  T,  T,  T],
    [  T,  T,  T, TG3, TG3,  T,  T,  T,  T,  T, TG3, TG3,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
];

// ── Pixelbyte (Digital) ─────────────────────────────────────────────────────
//
//  A glitchy digital creature with pixel artifacts
//

const PX1: Color = Color::new(0, 220, 255);    // cyan
const PX2: Color = Color::new(0, 160, 200);    // dark cyan
const PX3: Color = Color::new(0, 100, 140);    // darker cyan
const PX4: Color = Color::new(255, 0, 200);    // magenta glitch
const PX5: Color = Color::new(0, 255, 100);    // green glitch
const PXE: Color = Color::new(255, 255, 255);  // white eyes

pub const PIXELBYTE_IDLE_1: SpriteFrame = [
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T, PX4,  T,  T,  T,  T,  T,  T,  T,  T, PX5,  T,  T,  T],
    [  T,  T,  T, PX3, PX2, PX1, PX1, PX1, PX1, PX1, PX2, PX3,  T,  T,  T,  T],
    [  T,  T, PX3, PX2, PX1, PX1, PX1, PX1, PX1, PX1, PX1, PX2, PX3,  T,  T,  T],
    [  T,  T, PX2, PX1, PXE, PXE, PX1, PX1, PX1, PXE, PXE, PX1, PX2,  T,  T,  T],
    [  T,  T, PX2, PX1, PXE, PXE, PX1, PX4, PX1, PXE, PXE, PX1, PX2,  T,  T,  T],
    [  T,  T, PX2, PX1, PX1, PX1, PX1, PX1, PX1, PX1, PX1, PX1, PX2,  T,  T,  T],
    [  T,  T, PX3, PX1, PX1, PX5, PX1, PX1, PX1, PX5, PX1, PX1, PX3,  T,  T,  T],
    [  T,  T, PX3, PX2, PX1, PX1, PX1, PX1, PX1, PX1, PX1, PX2, PX3,  T,  T,  T],
    [  T, PX4, PX3, PX2, PX2, PX1, PX1, PX1, PX1, PX1, PX2, PX2, PX3, PX5,  T,  T],
    [  T,  T,  T, PX3, PX2, PX2, PX1, PX1, PX1, PX2, PX2, PX3,  T,  T,  T,  T],
    [  T,  T,  T,  T, PX3, PX2, PX1, PX1, PX1, PX2, PX3,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T, PX3, PX3, PX2, PX2, PX2, PX3, PX3,  T,  T,  T,  T,  T],
    [  T,  T,  T, PX3, PX3,  T, PX3,  T, PX3,  T, PX3, PX3,  T,  T,  T,  T],
    [  T,  T, PX3, PX3,  T,  T,  T,  T,  T,  T,  T, PX3, PX3,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
];

// ── Cloudwisp (Air) ─────────────────────────────────────────────────────────
//
//  A floating cloud creature
//

const CW1: Color = Color::new(220, 230, 255);  // bright white-blue
const CW2: Color = Color::new(180, 200, 240);  // light blue
const CW3: Color = Color::new(140, 170, 220);  // medium blue
const CW4: Color = Color::new(100, 140, 200);  // darker blue
const CW5: Color = Color::new(240, 245, 255);  // white highlight
const CWE: Color = Color::new(50, 50, 80);     // eyes

pub const CLOUDWISP_IDLE_1: SpriteFrame = [
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T, CW3, CW2, CW1, CW2, CW3,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T, CW3, CW2, CW1, CW5, CW5, CW5, CW1, CW2, CW3,  T,  T,  T,  T],
    [  T,  T, CW3, CW1, CW5, CW5, CW1, CW5, CW1, CW5, CW5, CW1, CW3,  T,  T,  T],
    [  T, CW4, CW2, CW1, CW5, CW1, CW1, CW1, CW1, CW1, CW5, CW1, CW2, CW4,  T,  T],
    [  T, CW3, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW3,  T,  T],
    [  T, CW3, CW1, CWE, CWE, CW1, CW1, CW1, CW1, CWE, CWE, CW1, CW1, CW3,  T,  T],
    [  T, CW3, CW1, CWE, CWE, CW1, CW1, CW5, CW1, CWE, CWE, CW1, CW1, CW3,  T,  T],
    [ CW4, CW2, CW1, CW1, CW1, CW1, CW5, CW5, CW5, CW1, CW1, CW1, CW1, CW2, CW4,  T],
    [ CW3, CW1, CW5, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW5, CW1, CW3,  T],
    [ CW4, CW2, CW1, CW5, CW1, CW1, CW1, CW1, CW1, CW1, CW1, CW5, CW1, CW2, CW4,  T],
    [  T, CW3, CW2, CW1, CW1, CW5, CW1, CW1, CW1, CW5, CW1, CW1, CW2, CW3,  T,  T],
    [  T,  T, CW4, CW3, CW2, CW1, CW1, CW1, CW1, CW1, CW2, CW3, CW4,  T,  T,  T],
    [  T,  T,  T,  T, CW4, CW3, CW2, CW2, CW2, CW3, CW4,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
];

// ── Dataslime (Poison) ──────────────────────────────────────────────────────
//
//  A green slime creature with database symbols
//

const DS1: Color = Color::new(80, 220, 80);    // bright green
const DS2: Color = Color::new(50, 170, 50);    // medium green
const DS3: Color = Color::new(30, 120, 30);    // dark green
const DS4: Color = Color::new(120, 255, 120);  // highlight green
const DS5: Color = Color::new(200, 255, 100);  // yellow-green
const DSE: Color = Color::new(20, 20, 20);     // eyes

pub const DATASLIME_IDLE_1: SpriteFrame = [
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T, DS4,  T,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T, DS4, DS4, DS4,  T,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T, DS3, DS2, DS1, DS2, DS3,  T,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T, DS3, DS1, DS1, DS4, DS1, DS1, DS3,  T,  T,  T,  T,  T,  T],
    [  T,  T, DS3, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS3,  T,  T,  T,  T,  T],
    [  T, DS3, DS1, DS1, DSE, DSE, DS1, DSE, DSE, DS1, DS1, DS3,  T,  T,  T,  T],
    [  T, DS2, DS1, DS1, DSE, DSE, DS1, DSE, DSE, DS1, DS1, DS2,  T,  T,  T,  T],
    [  T, DS2, DS1, DS1, DS1, DS1, DS5, DS1, DS1, DS1, DS1, DS2,  T,  T,  T,  T],
    [ DS3, DS1, DS1, DS4, DS1, DS5, DS5, DS5, DS1, DS4, DS1, DS1, DS3,  T,  T,  T],
    [ DS3, DS1, DS4, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS4, DS1, DS3,  T,  T,  T],
    [ DS2, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS1, DS2,  T,  T,  T],
    [  T, DS2, DS1, DS1, DS4, DS1, DS1, DS1, DS4, DS1, DS1, DS2,  T,  T,  T,  T],
    [  T,  T, DS3, DS2, DS2, DS1, DS1, DS1, DS2, DS2, DS3,  T,  T,  T,  T,  T],
    [  T,  T,  T, DS3, DS3, DS2, DS2, DS2, DS3, DS3,  T,  T,  T,  T,  T,  T],
    [  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T,  T],
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
        ("Neuromorph",  &NEUROMORPH_IDLE_1),
        ("Dockersaur",  &DOCKERSAUR_IDLE_1),
        ("Termignite",  &TERMIGNITE_IDLE_1),
        ("Pixelbyte",   &PIXELBYTE_IDLE_1),
        ("Cloudwisp",   &CLOUDWISP_IDLE_1),
        ("Dataslime",   &DATASLIME_IDLE_1),
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
        "neuromorph"  => &NEUROMORPH_IDLE_1,
        "dockersaur"  => &DOCKERSAUR_IDLE_1,
        "termignite"  => &TERMIGNITE_IDLE_1,
        "pixelbyte"   => &PIXELBYTE_IDLE_1,
        "cloudwisp"   => &CLOUDWISP_IDLE_1,
        "dataslime"   => &DATASLIME_IDLE_1,
        _ => &EMBERCLI_IDLE_1, // default fallback
    }
}

/// Get the idle sprite for an agent kind string (e.g. "Claude Code" → embercli).
pub fn sprite_for_agent(agent_kind: &str) -> &'static SpriteFrame {
    sprite_for_species(species_for_agent(agent_kind))
}

/// Map agent kind to creature species name.
/// Get default species for an agent kind (first instance).
pub fn species_for_agent(agent_kind: &str) -> &'static str {
    species_for_agent_idx(agent_kind, 0)
}

/// All available species, in rotation order.
const ALL_SPECIES: &[&str] = &[
    "embercli", "voltprompt", "shelloise", "rustacean", "pythorn", "gitbat",
    "neuromorph", "dockersaur", "termignite", "pixelbyte", "cloudwisp", "dataslime",
];

/// Get species for an agent kind + instance index.
/// First instance of each kind gets the default species, additional instances
/// rotate through other species so each agent looks unique.
pub fn species_for_agent_idx(agent_kind: &str, instance_idx: usize) -> &'static str {
    let lower = agent_kind.to_lowercase();
    let primary = if lower.contains("claude") {
        0 // embercli
    } else if lower.contains("codex") {
        1 // voltprompt
    } else if lower.contains("aider") {
        2 // shelloise
    } else if lower.contains("cursor") {
        3 // rustacean
    } else if lower.contains("continue") || lower.contains("copilot") {
        4 // pythorn
    } else if lower.contains("gpt") || lower.contains("openai") {
        6 // neuromorph
    } else if lower.contains("docker") {
        7 // dockersaur
    } else if lower.contains("vim") || lower.contains("tmux") {
        8 // termignite
    } else if lower.contains("game") || lower.contains("unity") || lower.contains("godot") {
        9 // pixelbyte
    } else if lower.contains("railway") || lower.contains("fly") || lower.contains("vercel") || lower.contains("deploy") {
        10 // cloudwisp
    } else if lower.contains("database") || lower.contains("postgres") || lower.contains("mysql") || lower.contains("redis") {
        11 // dataslime
    } else {
        5 // gitbat
    };
    let idx = (primary + instance_idx) % ALL_SPECIES.len();
    ALL_SPECIES[idx]
}
