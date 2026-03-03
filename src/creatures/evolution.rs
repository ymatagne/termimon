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
