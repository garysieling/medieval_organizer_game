//! QSP-style compartment model for reputation (not health).
//! States: Unknown->Known->Trusted->Esteemed->Disgraced. Transition rates between.

use crate::engine::ecs::{Entity, World};

/// Reputation compartments (stages like QSP compartments)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepStage {
    Unknown,      // no one knows them
    Known,        // recognized face
    Trusted,      // people rely on them
    Esteemed,     // high standing
    Disgraced,    // reputation ruined
}

/// Reputation state for an entity
#[derive(Debug)]
pub struct ReputationState {
    pub stage: RepStage,
    pub score: f32,              // continuous reputation value
    pub transition_rates: Vec<(RepStage, RepStage, f32)>, // (from, to, rate)
}

/// Actions that change reputation
#[derive(Debug)]
pub enum RepAction {
    GoodDeed,       // helps -> improves
    KeptPromise,    // reliability
    GreatDeed,      // big boost
    BadDeed,        // harms -> drops
    BrokenPromise,  // reliability loss
    Betrayal,       // massive drop
}

/// Apply an action -> reputation transitions
pub fn apply_action(state: &mut ReputationState, action: RepAction) {
    let delta = match action {
        RepAction::GoodDeed => 0.1,
        RepAction::KeptPromise => 0.05,
        RepAction::GreatDeed => 0.25,
        RepAction::BadDeed => -0.15,
        RepAction::BrokenPromise => -0.1,
        RepAction::Betrayal => -0.5,
    };
    state.score = (state.score + delta).clamp(0.0, 1.0);
    update_stage(state);
}

/// Map score to stage (compartment transitions)
fn update_stage(state: &mut ReputationState) {
    state.stage = if state.score < 0.1 { RepStage::Disgraced }
        else if state.score < 0.3 { RepStage::Unknown }
        else if state.score < 0.5 { RepStage::Known }
        else if state.score < 0.8 { RepStage::Trusted }
        else { RepStage::Esteemed };
}

/// QSP-style: reputation flows between compartments over time
pub fn transition_tick(state: &mut ReputationState) {
    // Decay toward neutral without reinforcement (rates between compartments)
    if state.score > 0.5 {
        state.score -= 0.005; // esteem fades without upkeep
    } else if state.score < 0.3 && state.score > 0.0 {
        state.score += 0.002; // disgrace slowly forgotten
    }
    update_stage(state);
}

/// Spawn reputation state
pub fn spawn_reputation(world: &mut World, score: f32) -> Entity {
    let e = world.create_entity();
    let mut s = ReputationState {
        stage: RepStage::Unknown,
        score: score.clamp(0.0, 1.0),
        transition_rates: vec![],
    };
    update_stage(&mut s);
    world.add_component(e, s);
    e
}
