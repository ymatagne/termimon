//! Battle system — turn-based auto-resolve battles between creatures.
//!
//! Stats derived from agent metrics:
//! - ATK = lines written per dollar (productivity)
//! - DEF = build success rate
//! - SPD = commits per hour
//! - HP = XP total
//!
//! Element type advantages: Fire > Grass > Water > Fire, Electric > Water,
//! Rust > Electric, Dark > Grass, etc.

use serde::{Deserialize, Serialize};

use crate::creatures::Element;

/// Stats for a battle combatant.
#[derive(Debug, Clone)]
pub struct BattleStats {
    pub name: String,
    pub species: String,
    pub element: Element,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: i32,
    pub def: i32,
    pub spd: i32,
    pub owner: String,
}

impl BattleStats {
    /// Create battle stats from creature sync data + productivity.
    pub fn from_creature(
        name: &str,
        species: &str,
        xp: u64,
        owner: &str,
        lines_per_dollar: f64,
        build_success_rate: f64,
        commits_per_hour: f64,
    ) -> Self {
        let element = crate::creatures::registry::get_creature_def(species)
            .map(|d| d.element)
            .unwrap_or(Element::Dark);

        // Base stats from XP, scaled by real metrics
        let base_hp = (xp as i32).max(50).min(999);
        let atk = (10.0 + lines_per_dollar * 2.0).min(99.0) as i32;
        let def = (10.0 + build_success_rate * 50.0).min(99.0) as i32;
        let spd = (10.0 + commits_per_hour * 5.0).min(99.0) as i32;

        Self {
            name: name.to_string(),
            species: species.to_string(),
            element,
            hp: base_hp,
            max_hp: base_hp,
            atk,
            def,
            spd,
            owner: owner.to_string(),
        }
    }

    /// Create default battle stats when no productivity data is available.
    pub fn from_xp(name: &str, species: &str, xp: u64, owner: &str) -> Self {
        Self::from_creature(name, species, xp, owner, 5.0, 0.5, 2.0)
    }
}

/// A single round of battle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleRound {
    pub round: u32,
    pub attacker: String,
    pub defender: String,
    pub damage: i32,
    pub message: String,
    pub attacker_hp: i32,
    pub defender_hp: i32,
}

/// Result of a complete battle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleResult {
    pub winner: String,
    pub loser: String,
    pub winner_owner: String,
    pub loser_owner: String,
    pub rounds: Vec<BattleRound>,
    pub xp_gained: u64,
}

/// Calculate type effectiveness multiplier.
fn type_multiplier(attacker: Element, defender: Element) -> f64 {
    match (attacker, defender) {
        // Classic triangle
        (Element::Fire, Element::Grass) => 2.0,
        (Element::Grass, Element::Water) => 2.0,
        (Element::Water, Element::Fire) => 2.0,
        // Reverse
        (Element::Grass, Element::Fire) => 0.5,
        (Element::Water, Element::Grass) => 0.5,
        (Element::Fire, Element::Water) => 0.5,
        // Electric advantages
        (Element::Electric, Element::Water) => 2.0,
        (Element::Water, Element::Electric) => 0.5,
        // Rust advantages
        (Element::Rust, Element::Electric) => 2.0,
        (Element::Electric, Element::Rust) => 0.5,
        // Dark advantages
        (Element::Dark, Element::Grass) => 2.0,
        (Element::Grass, Element::Dark) => 0.5,
        // Fire vs Rust
        (Element::Fire, Element::Rust) => 2.0,
        (Element::Rust, Element::Fire) => 0.5,
        // Same type
        _ if attacker == defender => 0.75,
        // Neutral
        _ => 1.0,
    }
}

/// Simple deterministic PRNG for battle (no external deps needed).
struct BattleRng {
    state: u64,
}

impl BattleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state >> 33
    }

    /// Random float 0.0..1.0
    fn float(&mut self) -> f64 {
        (self.next() % 10000) as f64 / 10000.0
    }
}

/// Run a battle between two creatures. Returns the result.
pub fn resolve_battle(mut a: BattleStats, mut b: BattleStats) -> BattleResult {
    let mut rounds = Vec::new();
    let mut round_num = 0u32;

    // Seed from names for deterministic-ish results
    let seed = a.name.bytes().map(|b| b as u64).sum::<u64>()
        + b.name.bytes().map(|b| b as u64).sum::<u64>();
    let mut rng = BattleRng::new(seed);

    // Determine who goes first by SPD
    let a_first = a.spd >= b.spd;

    while a.hp > 0 && b.hp > 0 && round_num < 10 {
        round_num += 1;

        let (attacker, defender) = if (round_num % 2 == 1) == a_first {
            (&a, &mut b)
        } else {
            (&b, &mut a)
        };

        let multiplier = type_multiplier(attacker.element, defender.element);
        let variance = 0.85 + rng.float() * 0.3; // 0.85 to 1.15
        let raw_damage = (attacker.atk as f64 * multiplier * variance) - (defender.def as f64 * 0.3);
        let damage = (raw_damage.max(1.0)) as i32;

        defender.hp = (defender.hp - damage).max(0);

        let effectiveness = if multiplier >= 2.0 {
            " It's super effective!"
        } else if multiplier <= 0.5 {
            " Not very effective..."
        } else {
            ""
        };

        let move_name = match attacker.element {
            Element::Fire => "Fire Blast",
            Element::Electric => "Thunder Shock",
            Element::Water => "Hydro Pump",
            Element::Rust => "Borrow Check",
            Element::Grass => "Vine Whip",
            Element::Dark => "Shadow Strike",
        };

        let message = format!(
            "{} uses {}! {} takes {} damage.{}",
            attacker.name, move_name, defender.name, damage, effectiveness
        );

        rounds.push(BattleRound {
            round: round_num,
            attacker: attacker.name.clone(),
            defender: defender.name.clone(),
            damage,
            message,
            attacker_hp: if (round_num % 2 == 1) == a_first { a.hp } else { b.hp },
            defender_hp: if (round_num % 2 == 1) == a_first { b.hp } else { a.hp },
        });
    }

    let (winner, loser) = if a.hp > b.hp {
        (a, b)
    } else {
        (b, a)
    };

    // Winner gets bonus XP based on loser's level
    let xp_gained = ((loser.max_hp as u64) / 5).max(10);

    BattleResult {
        winner: winner.name.clone(),
        loser: loser.name.clone(),
        winner_owner: winner.owner.clone(),
        loser_owner: loser.owner.clone(),
        rounds,
        xp_gained,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_multiplier_fire_beats_grass() {
        assert_eq!(type_multiplier(Element::Fire, Element::Grass), 2.0);
    }

    #[test]
    fn test_type_multiplier_water_beats_fire() {
        assert_eq!(type_multiplier(Element::Water, Element::Fire), 2.0);
    }

    #[test]
    fn test_type_multiplier_same_type() {
        assert_eq!(type_multiplier(Element::Fire, Element::Fire), 0.75);
    }

    #[test]
    fn test_type_multiplier_neutral() {
        assert_eq!(type_multiplier(Element::Fire, Element::Electric), 1.0);
    }

    #[test]
    fn test_battle_stats_from_xp() {
        let stats = BattleStats::from_xp("Infernocli", "embercli", 500, "yan");
        assert_eq!(stats.name, "Infernocli");
        assert_eq!(stats.element, Element::Fire);
        assert_eq!(stats.hp, 500);
        assert!(stats.atk > 0);
        assert!(stats.def > 0);
        assert!(stats.spd > 0);
    }

    #[test]
    fn test_battle_stats_min_hp() {
        let stats = BattleStats::from_xp("Baby", "embercli", 10, "yan");
        assert_eq!(stats.hp, 50); // min 50
    }

    #[test]
    fn test_battle_stats_max_hp() {
        let stats = BattleStats::from_xp("God", "embercli", 99999, "yan");
        assert_eq!(stats.hp, 999); // max 999
    }

    #[test]
    fn test_resolve_battle_produces_result() {
        let a = BattleStats::from_xp("Infernocli", "embercli", 500, "yan");
        let b = BattleStats::from_xp("Shelloise", "shelloise", 500, "alex");
        let result = resolve_battle(a, b);
        assert!(!result.winner.is_empty());
        assert!(!result.loser.is_empty());
        assert!(!result.rounds.is_empty());
        assert!(result.rounds.len() <= 10);
        assert!(result.xp_gained > 0);
    }

    #[test]
    fn test_resolve_battle_deterministic() {
        let a1 = BattleStats::from_xp("Infernocli", "embercli", 500, "yan");
        let b1 = BattleStats::from_xp("Shelloise", "shelloise", 500, "alex");
        let r1 = resolve_battle(a1, b1);

        let a2 = BattleStats::from_xp("Infernocli", "embercli", 500, "yan");
        let b2 = BattleStats::from_xp("Shelloise", "shelloise", 500, "alex");
        let r2 = resolve_battle(a2, b2);

        assert_eq!(r1.winner, r2.winner);
        assert_eq!(r1.rounds.len(), r2.rounds.len());
    }

    #[test]
    fn test_type_advantage_matters() {
        // Fire vs Grass — fire should win more often
        let fire = BattleStats::from_creature("Fire", "embercli", 200, "a", 5.0, 0.5, 2.0);
        let grass = BattleStats::from_creature("Grass", "pythorn", 200, "b", 5.0, 0.5, 2.0);
        let result = resolve_battle(fire, grass);
        assert_eq!(result.winner, "Fire");
    }
}
