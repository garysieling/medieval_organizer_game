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
    update_stage(state, delta < 0.0);
}

/// Map score to stage (compartment transitions). A very low score means
/// Disgraced only for someone who fell there (`fell`) or is already
/// disgraced; a stranger nobody has heard of is simply Unknown.
fn update_stage(state: &mut ReputationState, fell: bool) {
    state.stage = if state.score < 0.1 {
            if fell || state.stage == RepStage::Disgraced { RepStage::Disgraced } else { RepStage::Unknown }
        }
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
    } else if state.score < 0.3 {
        state.score += 0.002; // disgrace slowly forgotten, even from rock bottom
    }
    update_stage(state, false);
}

/// Spawn reputation state
pub fn spawn_reputation(world: &mut World, score: f32) -> Entity {
    let e = world.create_entity();
    let mut s = ReputationState {
        stage: RepStage::Unknown,
        score: score.clamp(0.0, 1.0),
        transition_rates: vec![],
    };
    update_stage(&mut s, false);
    world.add_component(e, s);
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(w: &World, e: Entity) -> &ReputationState {
        w.get_component::<ReputationState>(e).unwrap()
    }

    #[test]
    fn test_newcomer_is_unknown_not_disgraced() {
        let mut w = World::new();
        let e = spawn_reputation(&mut w, 0.0);
        assert_eq!(state(&w, e).stage, RepStage::Unknown);
    }

    #[test]
    fn test_betrayal_disgraces_and_rock_bottom_is_eventually_forgotten() {
        let mut w = World::new();
        let e = spawn_reputation(&mut w, 0.4);
        let s = w.get_component_mut::<ReputationState>(e).unwrap();
        apply_action(s, RepAction::Betrayal);
        assert_eq!(s.score, 0.0);
        assert_eq!(s.stage, RepStage::Disgraced);
        transition_tick(s);
        assert!(s.score > 0.0, "disgrace at exactly 0 must still fade");
        for _ in 0..60 { transition_tick(s); }
        assert_eq!(s.stage, RepStage::Unknown, "forgotten, not still disgraced");
    }

    #[test]
    fn test_good_deed_from_low_score_does_not_disgrace() {
        let mut w = World::new();
        let e = spawn_reputation(&mut w, 0.0);
        let s = w.get_component_mut::<ReputationState>(e).unwrap();
        apply_action(s, RepAction::KeptPromise); // 0.05: still below 0.1
        assert_eq!(s.stage, RepStage::Unknown);
    }
}
