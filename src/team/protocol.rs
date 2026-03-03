//! Team protocol — line-based JSON messages over TCP.

use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: &str = "0.4.0";

/// All message types in the team protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    /// Client → Host: initial handshake
    #[serde(rename = "hello")]
    Hello { name: String, version: String },

    /// Host → Client: handshake response
    #[serde(rename = "welcome")]
    Welcome { name: String, peers: Vec<String> },

    /// Host → All: a new peer joined
    #[serde(rename = "peer_joined")]
    PeerJoined { name: String },

    /// Host → All: a peer left
    #[serde(rename = "peer_left")]
    PeerLeft { name: String },

    /// Any → Any: periodic creature state sync
    #[serde(rename = "sync")]
    Sync { creatures: Vec<CreatureSync> },

    /// Client → Host: battle challenge
    #[serde(rename = "challenge")]
    Challenge { from: String, creature: String },

    /// Client → Host: accept a battle challenge
    #[serde(rename = "accept")]
    Accept { from: String, creature: String },

    /// Client → Host: decline a battle challenge
    #[serde(rename = "decline")]
    Decline { from: String },

    /// Host → All: battle result
    #[serde(rename = "battle_result")]
    BattleResult {
        winner: String,
        loser: String,
        rounds: Vec<super::battle::BattleRound>,
    },

    /// Any → Any: emote/reaction
    #[serde(rename = "emote")]
    Emote { from: String, emoji: String },

    /// Disconnect notification
    #[serde(rename = "goodbye")]
    Goodbye { name: String },
}

/// Creature state sent during sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureSync {
    pub name: String,
    pub species: String,
    pub stage: u8,
    pub xp: u64,
    pub state: String,
    pub cpu: f32,
    pub project: String,
    pub owner: String,
}

impl Message {
    /// Serialize to a single JSON line (no newlines in output) + trailing newline.
    pub fn to_line(&self) -> String {
        let mut s = serde_json::to_string(self).expect("Failed to serialize message");
        s.push('\n');
        s
    }

    /// Parse a JSON line into a Message.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_roundtrip() {
        let msg = Message::Hello {
            name: "yan".into(),
            version: "0.4.0".into(),
        };
        let line = msg.to_line();
        let parsed = Message::from_line(&line).unwrap();
        match parsed {
            Message::Hello { name, version } => {
                assert_eq!(name, "yan");
                assert_eq!(version, "0.4.0");
            }
            _ => panic!("Expected Hello"),
        }
    }

    #[test]
    fn test_welcome_roundtrip() {
        let msg = Message::Welcome {
            name: "host".into(),
            peers: vec!["yan".into(), "alex".into()],
        };
        let line = msg.to_line();
        let parsed = Message::from_line(&line).unwrap();
        match parsed {
            Message::Welcome { name, peers } => {
                assert_eq!(name, "host");
                assert_eq!(peers, vec!["yan", "alex"]);
            }
            _ => panic!("Expected Welcome"),
        }
    }

    #[test]
    fn test_sync_roundtrip() {
        let msg = Message::Sync {
            creatures: vec![CreatureSync {
                name: "Infernocli".into(),
                species: "embercli".into(),
                stage: 3,
                xp: 1500,
                state: "typing".into(),
                cpu: 42.0,
                project: "loop-mobile".into(),
                owner: "yan".into(),
            }],
        };
        let line = msg.to_line();
        let parsed = Message::from_line(&line).unwrap();
        match parsed {
            Message::Sync { creatures } => {
                assert_eq!(creatures.len(), 1);
                assert_eq!(creatures[0].name, "Infernocli");
                assert_eq!(creatures[0].xp, 1500);
            }
            _ => panic!("Expected Sync"),
        }
    }

    #[test]
    fn test_battle_result_roundtrip() {
        let msg = Message::BattleResult {
            winner: "Infernocli".into(),
            loser: "Thunderprompt".into(),
            rounds: vec![super::super::battle::BattleRound {
                round: 1,
                attacker: "Infernocli".into(),
                defender: "Thunderprompt".into(),
                damage: 45,
                message: "Infernocli uses Fire Blast!".into(),
                attacker_hp: 100,
                defender_hp: 55,
            }],
        };
        let line = msg.to_line();
        let parsed = Message::from_line(&line).unwrap();
        match parsed {
            Message::BattleResult { winner, loser, rounds } => {
                assert_eq!(winner, "Infernocli");
                assert_eq!(loser, "Thunderprompt");
                assert_eq!(rounds.len(), 1);
                assert_eq!(rounds[0].damage, 45);
            }
            _ => panic!("Expected BattleResult"),
        }
    }

    #[test]
    fn test_emote_roundtrip() {
        let msg = Message::Emote {
            from: "yan".into(),
            emoji: "⚔️".into(),
        };
        let line = msg.to_line();
        let parsed = Message::from_line(&line).unwrap();
        match parsed {
            Message::Emote { from, emoji } => {
                assert_eq!(from, "yan");
                assert_eq!(emoji, "⚔️");
            }
            _ => panic!("Expected Emote"),
        }
    }

    #[test]
    fn test_invalid_json() {
        let result = Message::from_line("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_type() {
        let result = Message::from_line(r#"{"type":"unknown_msg","foo":"bar"}"#);
        assert!(result.is_err());
    }
}
