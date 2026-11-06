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

/// System: check contract fulfillment each tick; fulfilling a contract
/// credits every obligated party's reputation
pub struct ContractSystem;
impl crate::engine::ecs::System for ContractSystem {
    fn run(&mut self, world: &mut World) {
        for c in world.query::<Contract>() {
            let mut honored = Vec::new();
            if let Some(ct) = world.get_component_mut::<Contract>(c) {
                if ct.status == ContractStatus::Active && ct.terms.iter().all(|t| t.fulfilled) {
                    ct.status = ContractStatus::Fulfilled;
                    honored = obligated_parties(ct.terms.iter());
                }
            }
            for p in honored {
                if let Some(r) = world.get_component_mut::<Reputation>(p) {
                    r.contracts_honored += 1;
                    r.trust_score = (r.trust_score + HONOR_TRUST_GAIN).min(1.0);
                }
            }
        }
    }
}

/// Trust earned by an obligated party when a contract is fulfilled
pub const HONOR_TRUST_GAIN: f32 = 0.05;

/// Each entity obligated by `terms`, once
fn obligated_parties<'a>(terms: impl Iterator<Item = &'a Term>) -> Vec<Entity> {
    let mut out = Vec::new();
    for t in terms {
        if !out.contains(&t.obligated) { out.push(t.obligated); }
    }
    out
}

/// Breach a contract - reputation damage. The parties who left terms
/// unfulfilled lose `breach_penalty` trust; only an active contract can be
/// breached, so the penalty is never applied twice.
pub fn breach(world: &mut World, contract: Entity) {
    let (penalty, breachers) = match world.get_component_mut::<Contract>(contract) {
        Some(ct) if ct.status == ContractStatus::Active => {
            ct.status = ContractStatus::Breached;
            (ct.breach_penalty, obligated_parties(ct.terms.iter().filter(|t| !t.fulfilled)))
        }
        _ => return,
    };
    for p in breachers {
        if let Some(r) = world.get_component_mut::<Reputation>(p) {
            r.contracts_breached += 1;
            r.trust_score = (r.trust_score - penalty).max(0.0);
        }
    }
}

/// The wax cracks: a live pact between `breacher` and `other` is torn
/// up, and the breacher's name takes the cost of breaking it. Returns
/// whether a pact stood to break.
pub fn break_pact(world: &mut World, breacher: Entity, other: Entity) -> bool {
    let pact = world.query::<Contract>().into_iter().find(|&c|
        world.get_component::<Contract>(c)
            .map(|ct| matches!(ct.status, ContractStatus::Active | ContractStatus::Fulfilled)
                && ct.parties.contains(&breacher) && ct.parties.contains(&other))
            .unwrap_or(false));
    let Some(c) = pact else { return false; };
    if let Some(ct) = world.get_component_mut::<Contract>(c) { ct.status = ContractStatus::Breached; }
    if let Some(r) = world.get_component_mut::<Reputation>(breacher) {
        r.contracts_breached += 1;
        r.trust_score = (r.trust_score - 0.2).max(0.0);
    }
    true
}

/// Gold the player gives when sealing a mutual-aid pact
pub const PACT_AID: f32 = 10.0;

/// Whether `a` and `b` already have a pact (a contract between them)
pub fn has_pact(world: &World, a: Entity, b: Entity) -> bool {
    world.query::<Contract>().into_iter()
        .filter_map(|c| world.get_component::<Contract>(c))
        .any(|c| c.parties.contains(&a) && c.parties.contains(&b))
}

/// Seal a mutual-aid pact: `giver` hands `receiver` PACT_AID gold of aid
/// out of `gold`, which counts as an Aid interaction between them (the
/// strongest trust-builder), recorded as a fulfilled contract so each pair
/// makes only one. Returns the contract, or None if they already have a
/// pact or the giver can't afford it (nothing changes then).
pub fn seal_aid_pact(world: &mut World, giver: Entity, receiver: Entity, gold: &mut f32) -> Option<Entity> {
    // a seal takes wax when there's wax to be had - the beekeeper's
    // season matters to more than the table
    if let Some(pos) = world.get_component::<crate::game::villager::Pos>(giver).map(|p| *p) {
        if let Some(shop) = crate::game::livelihood::seller_for(world, "wax", pos) {
            if let Some(b) = world.get_component_mut::<crate::game::household::Building>(shop) {
                crate::game::household::take_stock(b, "wax", 1);
            }
        }
    }
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    if giver == receiver || *gold < PACT_AID || has_pact(world, giver, receiver) { return None; }
    *gold -= PACT_AID;
    // a name to keep or break: both parties carry contract history
    for p in [giver, receiver] {
        if world.get_component::<Reputation>(p).is_none() {
            world.add_component(p, Reputation {
                entity: p, trust_score: 0.5, contracts_honored: 0, contracts_breached: 0 });
        }
    }
    record_interaction(world, giver, Interaction { other: receiver, kind: InteractionKind::Aid, intensity: 1.0 });
    let c = world.create_entity();
    world.add_component(c, Contract {
        parties: vec![giver, receiver],
        terms: vec![Term { description: format!("give {:.0} gold of aid", PACT_AID), obligated: giver, fulfilled: true }],
        status: ContractStatus::Fulfilled,
        breach_penalty: 0.0,
    });
    Some(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;

    fn setup(w: &mut World, miller_done: bool) -> (Entity, Entity, Entity) {
        let miller = w.create_entity();
        let baker = w.create_entity();
        w.add_component(miller, Reputation { entity: miller, trust_score: 0.8, contracts_honored: 0, contracts_breached: 0 });
        w.add_component(baker, Reputation { entity: baker, trust_score: 0.7, contracts_honored: 0, contracts_breached: 0 });
        let c = w.create_entity();
        w.add_component(c, Contract {
            parties: vec![miller, baker],
            terms: vec![
                Term { description: "deliver flour".into(), obligated: miller, fulfilled: miller_done },
                Term { description: "pay for flour".into(), obligated: baker, fulfilled: true },
            ],
            status: ContractStatus::Active,
            breach_penalty: 0.3,
        });
        (c, miller, baker)
    }

    #[test]
    fn test_breach_penalizes_only_the_party_that_failed() {
        let mut w = World::new();
        let (c, miller, baker) = setup(&mut w, false);
        breach(&mut w, c);
        assert_eq!(w.get_component::<Contract>(c).unwrap().status, ContractStatus::Breached);
        let m = w.get_component::<Reputation>(miller).unwrap();
        assert_eq!(m.contracts_breached, 1);
        assert!((m.trust_score - 0.5).abs() < 1e-6);
        let b = w.get_component::<Reputation>(baker).unwrap();
        assert_eq!(b.contracts_breached, 0);
        assert!((b.trust_score - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_breach_twice_penalizes_once() {
        let mut w = World::new();
        let (c, miller, _) = setup(&mut w, false);
        breach(&mut w, c);
        breach(&mut w, c);
        assert_eq!(w.get_component::<Reputation>(miller).unwrap().contracts_breached, 1);
    }

    #[test]
    fn test_fulfilled_contract_credits_obligated_parties() {
        let mut w = World::new();
        let (c, miller, baker) = setup(&mut w, true);
        ContractSystem.run(&mut w);
        ContractSystem.run(&mut w);
        assert_eq!(w.get_component::<Contract>(c).unwrap().status, ContractStatus::Fulfilled);
        for p in [miller, baker] {
            assert_eq!(w.get_component::<Reputation>(p).unwrap().contracts_honored, 1);
        }
        // A fulfilled contract can no longer be breached
        breach(&mut w, c);
        assert_eq!(w.get_component::<Reputation>(miller).unwrap().contracts_breached, 0);
    }

    #[test]
    fn test_aid_pact_builds_trust_once_per_pair() {
        use crate::game::relationships::{find_relationship, Relationship};
        let mut w = World::new();
        let (player, villager) = (w.create_entity(), w.create_entity());
        let mut gold = 25.0;
        assert!(seal_aid_pact(&mut w, player, villager, &mut gold).is_some());
        assert_eq!(gold, 15.0);
        let r = find_relationship(&w, player, villager).unwrap();
        assert!((w.get_component::<Relationship>(r).unwrap().strength - 0.12).abs() < 1e-6);
        // one pact per pair, and no charge for asking again
        assert!(seal_aid_pact(&mut w, player, villager, &mut gold).is_none());
        assert_eq!(gold, 15.0);
        // can't afford another with someone else
        let other = w.create_entity();
        let mut poor = 5.0;
        assert!(seal_aid_pact(&mut w, player, other, &mut poor).is_none());
        assert_eq!(poor, 5.0);
    }
}
