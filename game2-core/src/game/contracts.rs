//! Contracts: legal framework for agreements and breaches.
//! From backlog: "i sign a contract for X, if i break it then Y may or may not happen"

use crate::engine::ecs::{Entity, World};

/// A legal agreement between parties
#[derive(Debug)]
pub struct Contract {
    pub parties: Vec<Entity>,
    pub terms: Vec<Term>,
    pub status: ContractStatus,
    pub breach_penalty: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContractStatus {
    Active,
    Fulfilled,
    Breached,
    Void,
}

/// A contract term/obligation
#[derive(Debug)]
pub struct Term {
    pub description: String,
    pub obligated: Entity,     // who must do it
    pub fulfilled: bool,
}

/// Reputation from contract history
#[derive(Debug)]
pub struct Reputation {
    pub entity: Entity,
    pub trust_score: f32,    // contracts honored vs broken
    pub contracts_honored: u32,
    pub contracts_breached: u32,
}

/// System: check contract fulfillment each tick
pub struct ContractSystem;
impl crate::engine::ecs::System for ContractSystem {
    fn run(&mut self, world: &mut World) {
        for c in world.query::<Contract>() {
            if let Some(ct) = world.get_component_mut::<Contract>(c) {
                if ct.status == ContractStatus::Active && ct.terms.iter().all(|t| t.fulfilled) {
                    ct.status = ContractStatus::Fulfilled;
                }
            }
        }
    }
}

/// Breach a contract - reputation damage
pub fn breach(world: &mut World, contract: Entity) {
    if let Some(ct) = world.get_component::<Contract>(contract) {
        let _penalty = ct.breach_penalty;
        let _parties = ct.parties.clone();
    }
    if let Some(ct) = world.get_component_mut::<Contract>(contract) {
        ct.status = ContractStatus::Breached;
    }
}
