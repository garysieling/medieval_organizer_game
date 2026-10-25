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

/// Pick an action for an AI ruler based on personality + state
pub fn choose_action(ruler: &AiRuler, targets: &[Entity]) -> PlayerAction {
    match ruler.personality {
        RulerPersonality::Diplomat => PlayerAction::TalkTo(targets[0]),
        RulerPersonality::Merchant => PlayerAction::Invest(targets[0], 20.0),
        RulerPersonality::Zealot => PlayerAction::OrganizeMeeting(targets.to_vec()),
        RulerPersonality::Tyrant => PlayerAction::Recruit(targets[0], 0.7),
        RulerPersonality::Hermit => PlayerAction::TalkTo(targets[0]),
        RulerPersonality::Builder => PlayerAction::BuildEntity("granary".into()),
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
