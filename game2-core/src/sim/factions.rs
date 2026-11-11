//! Power centers the village didn't found: the parish's interest, the
//! market's, a trade grown thick enough to act as one, a household grown
//! wide enough to answer to its own name. A faction is an Organization
//! carrying a Faction tag; its membership is read from the world, never
//! stored - whoever fits the rule right now is of it.

use crate::engine::ecs::{Entity, World};
use crate::engine::det::HashMap;
use crate::game::household::{Building, BuildingKind};
use crate::game::villager;
use crate::game::vocation::Vocation;
use crate::sim::community::{Citizen, Organization};
use crate::sim::religion::Congregation;

/// What a power center is, and where its people come from
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FactionKind {
    /// The parish's interest - its people are the congregation's
    Church { congregation: Entity },
    /// The market's interest - those who keep its shops and stalls
    Market,
    /// A trade grown thick - those who hold its shops
    Guild { trade: Vocation },
    /// A household grown wide - those who live under its roof
    Family { home: Entity },
}

/// A power center, and the extraction right it holds - the claim is
/// descriptive for now; the physical flow is #970's to hang on it
#[derive(Debug)]
pub struct Faction {
    pub kind: FactionKind,
    pub claim: &'static str,
}

/// How many held shops of one trade make a fellowship
pub const GUILD_SHOPS: usize = 3;
/// How many souls under one roof make a house that counts
pub const FAMILY_SOULS: usize = 5;

/// Who holds the shops, by trade - a shop counts for the worker whose
/// name is on it, while they live
pub fn shops_by_trade(world: &World) -> HashMap<Vocation, Vec<Entity>> {
    let mut by: HashMap<Vocation, Vec<Entity>> = HashMap::default();
    for e in world.query::<Building>() {
        let Some(b) = world.get_component::<Building>(e) else { continue };
        let BuildingKind::Shop(v) = b.kind else { continue };
        if let Some(w) = b.worker.filter(|&w| !villager::is_dead(world, w)) {
            by.entry(v).or_default().push(w);
        }
    }
    by
}

/// Who's of the faction today - membership is read from the world,
/// never stored
pub fn members_of(world: &World, f: &Faction) -> Vec<Entity> {
    let alive = |&v: &Entity| !villager::is_dead(world, v);
    match f.kind {
        FactionKind::Church { congregation } =>
            world.get_component::<Congregation>(congregation)
                .map(|c| c.members.iter().copied().filter(alive).collect())
                .unwrap_or_default(),
        FactionKind::Market =>
            shops_by_trade(world).into_values().flatten().collect(),
        FactionKind::Guild { trade } =>
            shops_by_trade(world).get(&trade).cloned().unwrap_or_default(),
        FactionKind::Family { home } =>
            world.get_component::<Building>(home)
                .map(|b| b.occupants.iter().copied().filter(alive).collect())
                .unwrap_or_default(),
    }
}

/// The pull a faction could bring - the sum of its people's weight
pub fn pooled_influence(world: &World, members: &[Entity]) -> f32 {
    members.iter().map(|&v| world.get_component::<Citizen>(v)
        .map(|c| c.influence).unwrap_or(0.0)).sum()
}

/// The fact a speaker carries into talk for being of a faction
pub fn fact_of(kind: FactionKind) -> &'static str {
    match kind {
        FactionKind::Church { .. } => "parish_faction",
        FactionKind::Market => "market_faction",
        FactionKind::Guild { .. } => "guild_faction",
        FactionKind::Family { .. } => "kin_faction",
    }
}

/// A power center takes form: an Organization carrying the Faction tag.
/// Its leader is itself - no soul speaks for the interest, so member-
/// and leader-keyed surfaces never mistake it for a cause
pub fn spawn(world: &mut World, name: String, kind: FactionKind, claim: &'static str) -> Entity {
    let org = world.create_entity();
    world.add_component(org, Organization {
        name, members: vec![], leader: org,
        influence_pool: 0.0, resources: 0.0,
        charter: String::new(), successors: vec![],
    });
    world.add_component(org, Faction { kind, claim });
    org
}
