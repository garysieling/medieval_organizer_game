//! The powers the village grows on its own: a trade grown thick enough
//! acts as one, a household grown wide answers to its name. The parish
//! and the market are seeded at the start; the rest emerge, and once a
//! faction forms it keeps.

use crate::session::*;
use crate::sim::factions::*;

impl Game {
    /// The factions' weekly accounting: their pull is their people's
    /// (refreshed from live membership), and new interests form where
    /// the facts cross a line
    pub(in crate::session) fn faction_tick(&mut self, tick: u64) {
        if tick % 7 != 5 { return; }
        let mut pools: Vec<(Entity, f32)> = Vec::new();
        for e in self.world.query::<Faction>() {
            let Some(f) = self.world.get_component::<Faction>(e) else { continue };
            pools.push((e, pooled_influence(&self.world, &members_of(&self.world, f))));
        }
        for (e, pool) in pools {
            if let Some(o) = self.world.get_component_mut::<Organization>(e) {
                o.influence_pool = pool;
            }
        }
        // a trade grown thick enough acts as one
        for (trade, holders) in shops_by_trade(&self.world).iter() {
            if holders.len() < GUILD_SHOPS { continue; }
            let kind = FactionKind::Guild { trade: *trade };
            if self.has_faction(kind) { continue; }
            let name = format!("the {} fellowship",
                crate::game::vocation::vocation_name(*trade).to_lowercase());
            let org = spawn(&mut self.world, name.clone(), kind, "the craft's dues");
            self.factions.push(org);
            self.log.push(format!("t{}: {} takes form - the trade keeps its own.", tick, name));
        }
        // a household grown wide answers to its own name
        for &b in self.building_entities.iter() {
            let Some(house) = self.world.get_component::<Building>(b) else { continue };
            if house.kind != BuildingKind::Home { continue; }
            let souls: Vec<Entity> = house.occupants.iter().copied()
                .filter(|&v| !villager::is_dead(&self.world, v)).collect();
            if souls.len() < FAMILY_SOULS { continue; }
            let kind = FactionKind::Family { home: b };
            if self.has_faction(kind) { continue; }
            let name = format!("the house of {}", self.name_of(souls[0]));
            let org = spawn(&mut self.world, name.clone(), kind, "the kin's call");
            self.factions.push(org);
            self.log.push(format!("t{}: {} takes form - wide enough to answer to its own name.", tick, name));
        }
    }

    /// A faction of this kind already stands
    fn has_faction(&self, kind: FactionKind) -> bool {
        self.world.query::<Faction>().iter().any(|&e|
            self.world.get_component::<Faction>(e).map(|f| f.kind == kind).unwrap_or(false))
    }

    /// Who holds the claim on the market's own stream - the stalls'
    /// fees. The faction whose people run the market holds it while any
    /// of them stand: its claim names the fees outright (#970)
    pub fn market_claimant(&self) -> Option<Entity> {
        self.factions.iter().copied().find(|&e| {
            self.world.get_component::<Faction>(e)
                .filter(|f| f.kind == FactionKind::Market)
                .map(|f| !members_of(&self.world, f).is_empty())
                .unwrap_or(false)
        })
    }
}
