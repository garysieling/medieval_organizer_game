//! AI players: heuristic agents that play villages via the same action interface.
//! Other villages/castles are AI-controlled; they differ by ruler personality.

use crate::engine::ecs::Entity;
use super::player::PlayerAction;

/// Ruler personality affects how they play
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RulerPersonality {
    Diplomat,    // talks, recruits, builds alliances
    Merchant,    // invests, trades, builds economy
    Zealot,      // religious, orthodox, intolerant
    Tyrant,      // military, control, harsh
    Hermit,      // isolationist, minimal interaction
    Builder,     // construction, infrastructure
}

/// An AI-controlled ruler
#[derive(Debug)]
pub struct AiRuler {
    pub personality: RulerPersonality,
    pub village: Entity,
    pub aggression: f32,
    pub sociability: f32,
}

/// Sociability above which even a hermit will reach out
pub const HERMIT_REACHES_OUT: f32 = 0.6;

/// Pick an action for an AI ruler based on personality + state. Returns
/// None when the ruler's preferred action needs someone to act on and
/// there's no one (rather than panicking on an empty target list).
pub fn choose_action(ruler: &AiRuler, targets: &[Entity]) -> Option<PlayerAction> {
    let first = targets.first().copied();
    match ruler.personality {
        RulerPersonality::Diplomat => first.map(PlayerAction::TalkTo),
        RulerPersonality::Merchant => first.map(|t| PlayerAction::Invest(t, 20.0)),
        RulerPersonality::Zealot if targets.is_empty() => None,
        RulerPersonality::Zealot => Some(PlayerAction::OrganizeMeeting(targets.to_vec())),
        // A tyrant presses recruits as hard as their aggression allows
        RulerPersonality::Tyrant => first.map(|t| PlayerAction::Recruit(t, ruler.aggression.clamp(0.0, 1.0))),
        // An isolationist keeps to their own village unless unusually sociable
        RulerPersonality::Hermit => match first {
            Some(t) if ruler.sociability > HERMIT_REACHES_OUT => Some(PlayerAction::TalkTo(t)),
            _ => Some(PlayerAction::BuildEntity("garden".into())),
        },
        RulerPersonality::Builder => Some(PlayerAction::BuildEntity("granary".into())),
    }
}

/// Governance model (differs by ruler)
#[derive(Debug)]
pub enum Governance {
    Democracy,   // villagers vote
    Monarchy,    // one ruler decides
    Oligarchy,   // small group rules
    Theocracy,   // religious leaders rule
    Anarchy,     // no formal rule
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ruler(personality: RulerPersonality, aggression: f32, sociability: f32) -> AiRuler {
        AiRuler { personality, village: 0, aggression, sociability }
    }

    #[test]
    fn test_no_targets_does_not_panic() {
        for p in [RulerPersonality::Diplomat, RulerPersonality::Merchant, RulerPersonality::Zealot, RulerPersonality::Tyrant] {
            assert!(choose_action(&ruler(p, 0.5, 0.5), &[]).is_none(), "{:?}", p);
        }
        assert!(matches!(choose_action(&ruler(RulerPersonality::Builder, 0.5, 0.5), &[]), Some(PlayerAction::BuildEntity(_))));
    }

    #[test]
    fn test_hermit_keeps_to_themselves_unless_sociable() {
        let villager = 7;
        let shy = choose_action(&ruler(RulerPersonality::Hermit, 0.1, 0.1), &[villager]);
        assert!(matches!(shy, Some(PlayerAction::BuildEntity(_))));
        let warm = choose_action(&ruler(RulerPersonality::Hermit, 0.1, 0.9), &[villager]);
        assert!(matches!(warm, Some(PlayerAction::TalkTo(7))));
    }

    #[test]
    fn test_tyrant_recruits_with_their_aggression() {
        match choose_action(&ruler(RulerPersonality::Tyrant, 0.9, 0.2), &[3]) {
            Some(PlayerAction::Recruit(3, force)) => assert!((force - 0.9).abs() < 1e-6),
            other => panic!("expected Recruit, got {:?}", other),
        }
    }
}
