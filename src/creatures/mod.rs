//! Creature definitions, sprites, and animation engine
//!
//! Each creature is a 16×16 pixel art sprite rendered with half-block characters (▀▄).
//! Colors are stored as const arrays for zero-allocation rendering.

pub mod animation;
pub mod assignment;
pub mod evolution;
pub mod registry;
pub mod sprites;

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Color type ───────────────────────────────────────────────────────────────

/// An RGBA pixel color for sprite art.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Opaque color.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Fully transparent pixel.
    pub const fn transparent() -> Self {
        Self { r: 0, g: 0, b: 0, a: 0 }
    }

    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }
}

// ── Sprite types ─────────────────────────────────────────────────────────────

/// A 16×16 sprite frame: 16 rows of 16 Color pixels.
pub type SpriteFrame = [[Color; 16]; 16];

/// Blank/transparent frame constant.
pub const BLANK_FRAME: SpriteFrame = [[Color::transparent(); 16]; 16];

// ── Element ──────────────────────────────────────────────────────────────────

/// Creature element types (à la Pokémon).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Fire,
    Electric,
    Water,
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Element::Fire     => write!(f, "🔥 Fire"),
            Element::Electric => write!(f, "⚡ Electric"),
            Element::Water    => write!(f, "💧 Water"),
        }
    }
}

impl Element {
    pub fn icon(&self) -> &'static str {
        match self {
            Element::Fire     => "🔥",
            Element::Electric => "⚡",
            Element::Water    => "💧",
        }
    }
}

// ── Animation state ──────────────────────────────────────────────────────────

/// All possible animation states a creature can be in.
///
/// Maps 1:1 to what the AI agent is doing in its pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimState {
    Idle,
    Typing,
    Reading,
    Thinking,
    Running,
    Sleeping,
    Celebrating,
    Error,
    Waiting,
}

impl fmt::Display for AnimState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnimState::Idle        => write!(f, "idle"),
            AnimState::Typing      => write!(f, "typing"),
            AnimState::Reading     => write!(f, "reading"),
            AnimState::Thinking    => write!(f, "thinking"),
            AnimState::Running     => write!(f, "running"),
            AnimState::Sleeping    => write!(f, "sleeping"),
            AnimState::Celebrating => write!(f, "celebrating"),
            AnimState::Error       => write!(f, "error"),
            AnimState::Waiting     => write!(f, "waiting"),
        }
    }
}

impl AnimState {
    /// Status emoji for this animation state.
    pub fn emoji(&self) -> &'static str {
        match self {
            AnimState::Idle        => "😊",
            AnimState::Typing      => "⌨️",
            AnimState::Reading     => "📖",
            AnimState::Thinking    => "🤔",
            AnimState::Running     => "🏃",
            AnimState::Sleeping    => "💤",
            AnimState::Celebrating => "🎉",
            AnimState::Error       => "💥",
            AnimState::Waiting     => "⏳",
        }
    }

    /// All variant values, for iteration.
    pub fn all() -> &'static [AnimState] {
        &[
            AnimState::Idle,
            AnimState::Typing,
            AnimState::Reading,
            AnimState::Thinking,
            AnimState::Running,
            AnimState::Sleeping,
            AnimState::Celebrating,
            AnimState::Error,
            AnimState::Waiting,
        ]
    }
}

// ── Evolution stage ──────────────────────────────────────────────────────────

/// Evolution stages. Creatures start at Base and evolve through XP milestones.
///
/// - Base:    0–99 XP
/// - Evolved: 100–499 XP
/// - Final:   500+ XP
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stage {
    Base    = 1,
    Evolved = 2,
    Final   = 3,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stage::Base    => write!(f, "Stage 1"),
            Stage::Evolved => write!(f, "Stage 2"),
            Stage::Final   => write!(f, "Stage 3"),
        }
    }
}

impl Stage {
    /// Stars representation for display: ★☆☆ / ★★☆ / ★★★
    pub fn stars(&self) -> &'static str {
        match self {
            Stage::Base    => "★☆☆",
            Stage::Evolved => "★★☆",
            Stage::Final   => "★★★",
        }
    }
}

// ── Creature definition (static, compile-time) ──────────────────────────────

/// A static creature species definition. Lives in the registry.
#[derive(Debug, Clone)]
pub struct CreatureDef {
    pub name: &'static str,
    pub element: Element,
    pub description: &'static str,
    pub default_agent: &'static str,
    pub evolution_names: [&'static str; 3],
}

// ── Creature instance (runtime, serializable) ────────────────────────────────

/// A living creature instance with XP, state, and agent binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub species: String,
    pub nickname: Option<String>,
    pub xp: u64,
    pub stage: Stage,
    pub state: AnimState,
    pub assigned_agent: Option<String>,
    pub assigned_pane: Option<String>,
    pub total_tasks: u64,
    pub total_errors: u64,
    pub created_at: String,
}

impl Creature {
    pub fn new(species: &str) -> Self {
        Self {
            species: species.to_string(),
            nickname: None,
            xp: 0,
            stage: Stage::Base,
            state: AnimState::Idle,
            assigned_agent: None,
            assigned_pane: None,
            total_tasks: 0,
            total_errors: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Add XP and check for evolution. Returns `Some(new_stage)` on evolution.
    pub fn add_xp(&mut self, amount: u64) -> Option<Stage> {
        self.xp += amount;
        let new_stage = if self.xp >= 500 {
            Stage::Final
        } else if self.xp >= 100 {
            Stage::Evolved
        } else {
            Stage::Base
        };

        if new_stage > self.stage {
            self.stage = new_stage;
            Some(new_stage)
        } else {
            None
        }
    }

    /// Display name: nickname → evolution name → species.
    pub fn display_name(&self) -> String {
        if let Some(ref nick) = self.nickname {
            return nick.clone();
        }
        match registry::get_creature_def(&self.species) {
            Some(d) => d.evolution_names[self.stage as usize - 1].to_string(),
            None => self.species.clone(),
        }
    }

    /// Approximate level from XP (sqrt curve).
    pub fn level(&self) -> u32 {
        (self.xp as f64).sqrt() as u32 + 1
    }

    /// XP progress toward next evolution as 0.0–1.0.
    pub fn evolution_progress(&self) -> f64 {
        match self.stage {
            Stage::Base    => self.xp as f64 / 100.0,
            Stage::Evolved => (self.xp.saturating_sub(100)) as f64 / 400.0,
            Stage::Final   => 1.0,
        }
    }

    /// XP needed for the *next* evolution, or 0 if fully evolved.
    pub fn xp_to_next_evolution(&self) -> u64 {
        match self.stage {
            Stage::Base    => 100u64.saturating_sub(self.xp),
            Stage::Evolved => 500u64.saturating_sub(self.xp),
            Stage::Final   => 0,
        }
    }
}
