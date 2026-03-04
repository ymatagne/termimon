//! Evolution system — XP tracking, milestone checks, and stage transitions
//!
//! Creatures earn XP from agent activity:
//!   - Task completed:  +10 XP
//!   - Error handled:    +3 XP (resilience!)
//!   - Long session:     +5 XP per 30 min
//!   - First run:       +20 XP
//!
//! Evolution milestones:
//!   - Stage 1 (Base)    →  Stage 2 (Evolved)  at 100 XP
//!   - Stage 2 (Evolved) →  Stage 3 (Final)    at 500 XP

use super::{Creature, Stage};
use serde::{Deserialize, Serialize};

// ── Leveling system ──────────────────────────────────────────────────────────

/// Compute level, XP into current level, and XP needed for the next level from total XP.
///
/// - **Stage 1** (Levels 1–10): 10 XP per level → evolves at Level 10 (100 XP)
/// - **Stage 2** (Levels 11–25): ~27 XP per level (400 XP / 15 levels) → evolves at Level 25 (500 XP)
/// - **Stage 3** (Levels 26+): 50 XP per level, uncapped
pub fn level_from_xp(xp: u64) -> (u8, u64, u64) {
    if xp < 100 {
        // Stage 1: levels 1-10, 10 XP each
        let level = (xp / 10) as u8 + 1; // 0 XP → Lv.1, 10 XP → Lv.2, ...
        let xp_into = xp % 10;
        (level.min(10), xp_into, 10)
    } else if xp < 500 {
        // Stage 2: levels 11-25, ~27 XP each (400 XP / 15 levels ≈ 26.67)
        let xp_in_stage = xp - 100;
        let level_in_stage = (xp_in_stage * 15 / 400) as u8; // 0..14
        let level = 11 + level_in_stage;
        // XP boundaries: level N starts at 100 + (N-11) * 400/15
        let xp_for_this_level = 100 + (level_in_stage as u64) * 400 / 15;
        let xp_for_next_level = 100 + (level_in_stage as u64 + 1) * 400 / 15;
        let xp_into = xp - xp_for_this_level;
        let xp_needed = xp_for_next_level - xp_for_this_level;
        (level.min(25), xp_into, xp_needed)
    } else {
        // Stage 3: levels 26+, 50 XP each, uncapped
        let xp_in_stage = xp - 500;
        let level_in_stage = (xp_in_stage / 50) as u8;
        let level = 26u8.saturating_add(level_in_stage);
        let xp_into = xp_in_stage % 50;
        (level, xp_into, 50)
    }
}

/// Return the prestige badge for lifetime XP milestones, if any.
pub fn prestige_badge(xp: u64) -> &'static str {
    if xp >= 100_000 {
        "🌟"
    } else if xp >= 50_000 {
        "🏆"
    } else if xp >= 10_000 {
        "👑"
    } else if xp >= 5_000 {
        "💎"
    } else if xp >= 1_000 {
        "⭐"
    } else {
        ""
    }
}

// ── XP rewards ───────────────────────────────────────────────────────────────

/// XP awarded for various agent activities.
#[derive(Debug, Clone, Copy)]
pub enum XpReward {
    /// Agent completed a task/prompt cycle
    TaskComplete,
    /// Agent encountered and handled an error
    ErrorHandled,
    /// Sustained session milestone (every 30 min)
    SessionMilestone,
    /// First time this creature was spawned
    FirstRun,
    /// Custom amount
    Custom(u64),
}

impl XpReward {
    /// The XP amount for this reward type.
    pub fn amount(&self) -> u64 {
        match self {
            XpReward::TaskComplete     => 10,
            XpReward::ErrorHandled     =>  3,
            XpReward::SessionMilestone =>  5,
            XpReward::FirstRun         => 20,
            XpReward::Custom(n)        => *n,
        }
    }

    /// Human-readable description of the reward.
    pub fn description(&self) -> &'static str {
        match self {
            XpReward::TaskComplete     => "Task completed",
            XpReward::ErrorHandled     => "Error handled",
            XpReward::SessionMilestone => "Session milestone",
            XpReward::FirstRun         => "First run",
            XpReward::Custom(_)        => "Bonus",
        }
    }
}

// ── Evolution event ──────────────────────────────────────────────────────────

/// Emitted when a creature evolves to a new stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEvent {
    /// Species that evolved
    pub species: String,
    /// The display name *after* evolution
    pub new_name: String,
    /// Which stage it evolved to
    pub new_stage: Stage,
    /// Total XP at time of evolution
    pub xp: u64,
    /// Timestamp
    pub timestamp: String,
}

// ── Evolution engine ─────────────────────────────────────────────────────────

/// Manages XP grants and evolution checks for the creature roster.
pub struct EvolutionEngine;

impl EvolutionEngine {
    /// Grant XP to a creature. Returns an `EvolutionEvent` if the creature
    /// evolves as a result.
    pub fn grant_xp(creature: &mut Creature, reward: XpReward) -> Option<EvolutionEvent> {
        let amount = reward.amount();
        creature.total_tasks += match reward {
            XpReward::TaskComplete => 1,
            _ => 0,
        };
        creature.total_errors += match reward {
            XpReward::ErrorHandled => 1,
            _ => 0,
        };

        if let Some(new_stage) = creature.add_xp(amount) {
            Some(EvolutionEvent {
                species: creature.species.clone(),
                new_name: creature.display_name(),
                new_stage,
                xp: creature.xp,
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        } else {
            None
        }
    }

    /// Check if a creature is close to evolving (within 10% of threshold).
    pub fn is_close_to_evolution(creature: &Creature) -> bool {
        match creature.stage {
            Stage::Base    => creature.xp >= 90,
            Stage::Evolved => creature.xp >= 450,
            Stage::Final   => false,
        }
    }

    /// Get the XP thresholds for the next evolution, or None if maxed.
    pub fn next_threshold(creature: &Creature) -> Option<u64> {
        match creature.stage {
            Stage::Base    => Some(100),
            Stage::Evolved => Some(500),
            Stage::Final   => None,
        }
    }

    /// Calculate XP progress as a ratio (0.0 to 1.0) toward next evolution.
    pub fn progress(creature: &Creature) -> f64 {
        creature.evolution_progress()
    }

    /// Generate the evolution celebration message.
    pub fn celebration_message(event: &EvolutionEvent) -> String {
        let stage_text = match event.new_stage {
            Stage::Base    => "hatched",
            Stage::Evolved => "evolved",
            Stage::Final   => "reached final form",
        };

        let sparkle = match event.new_stage {
            Stage::Base    => "🥚→🐣",
            Stage::Evolved => "✨🔥✨",
            Stage::Final   => "🌟⭐🌟",
        };

        format!(
            "{sparkle} {name} {stage_text}! {sparkle}\n  Stage {s} • {xp} XP",
            name = event.new_name,
            s = event.new_stage as u8,
            xp = event.xp,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Creature;

    #[test]
    fn xp_grants_work() {
        let mut c = Creature::new("embercli");
        assert_eq!(c.xp, 0);

        let evt = EvolutionEngine::grant_xp(&mut c, XpReward::TaskComplete);
        assert!(evt.is_none());
        assert_eq!(c.xp, 10);
    }

    #[test]
    fn evolution_at_100() {
        let mut c = Creature::new("embercli");
        c.xp = 95;

        let evt = EvolutionEngine::grant_xp(&mut c, XpReward::TaskComplete);
        assert!(evt.is_some());
        let evt = evt.unwrap();
        assert_eq!(evt.new_stage, Stage::Evolved);
        assert_eq!(c.stage, Stage::Evolved);
    }

    #[test]
    fn evolution_at_500() {
        let mut c = Creature::new("voltprompt");
        c.xp = 495;
        c.stage = Stage::Evolved;

        let evt = EvolutionEngine::grant_xp(&mut c, XpReward::TaskComplete);
        assert!(evt.is_some());
        assert_eq!(evt.unwrap().new_stage, Stage::Final);
    }

    #[test]
    fn no_evolution_past_final() {
        let mut c = Creature::new("shelloise");
        c.xp = 999;
        c.stage = Stage::Final;

        let evt = EvolutionEngine::grant_xp(&mut c, XpReward::TaskComplete);
        assert!(evt.is_none());
        assert_eq!(c.xp, 1009);
    }

    #[test]
    fn close_to_evolution_detection() {
        let mut c = Creature::new("embercli");
        c.xp = 89;
        assert!(!EvolutionEngine::is_close_to_evolution(&c));

        c.xp = 90;
        assert!(EvolutionEngine::is_close_to_evolution(&c));
    }

    #[test]
    fn level_from_xp_stage1() {
        let (level, into, needed) = super::level_from_xp(0);
        assert_eq!(level, 1);
        assert_eq!(into, 0);
        assert_eq!(needed, 10);

        let (level, into, needed) = super::level_from_xp(55);
        assert_eq!(level, 6);
        assert_eq!(into, 5);
        assert_eq!(needed, 10);

        let (level, _, _) = super::level_from_xp(99);
        assert_eq!(level, 10);
    }

    #[test]
    fn level_from_xp_stage2() {
        let (level, _, _) = super::level_from_xp(100);
        assert_eq!(level, 11);

        let (level, _, _) = super::level_from_xp(499);
        assert!(level >= 24 && level <= 25);
    }

    #[test]
    fn level_from_xp_stage3() {
        let (level, into, needed) = super::level_from_xp(500);
        assert_eq!(level, 26);
        assert_eq!(into, 0);
        assert_eq!(needed, 50);

        let (level, _, _) = super::level_from_xp(3967);
        // (3967 - 500) / 50 = 69.34 → level 26 + 69 = 95
        assert_eq!(level, 95);
    }

    #[test]
    fn prestige_badges() {
        assert_eq!(super::prestige_badge(500), "");
        assert_eq!(super::prestige_badge(1000), "⭐");
        assert_eq!(super::prestige_badge(5000), "💎");
        assert_eq!(super::prestige_badge(10000), "👑");
        assert_eq!(super::prestige_badge(50000), "🏆");
        assert_eq!(super::prestige_badge(100000), "🌟");
    }

    #[test]
    fn celebration_messages() {
        let evt = EvolutionEvent {
            species: "embercli".into(),
            new_name: "Blazecli".into(),
            new_stage: Stage::Evolved,
            xp: 105,
            timestamp: "2025-01-01T00:00:00Z".into(),
        };
        let msg = EvolutionEngine::celebration_message(&evt);
        assert!(msg.contains("Blazecli"));
        assert!(msg.contains("evolved"));
    }
}
