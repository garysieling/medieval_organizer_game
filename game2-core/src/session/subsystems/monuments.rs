//! Monuments: the village remembers only its greatest few. When a
//! villager's tale earns it, the lord of the castle pays for a standing
//! stone that rises over days where folk walk - and the ones who pass
//! it remember.

use crate::session::*;

/// The most monuments standing at once - the village honors only its
/// greatest few
const MONUMENT_CAP: usize = 4;
/// The story intensity a deed needs to earn one
const WORTHY: f32 = 0.6;
/// What the lord's gold buys: it comes out of the castle's treasury
const MONUMENT_COST: f32 = 40.0;
/// Days a monument rises over
const BUILD_DAYS: u64 = 20;
/// How near a passerby must come to remember (a cairn reaches 12)
const REMEMBER_NEAR: f32 = 18.0;

impl Game {
    /// The monuments' day: a worthy tale proposes one, the lord pays,
    /// it rises over days, and folk remember near it
    pub(in crate::session) fn monument_tick(&mut self, tick: u64) {
        // a tale worth a stone: the day's proposals
        if self.monuments.iter().filter(|m| m.stood_on.is_none() || m.build_until <= tick).count() < MONUMENT_CAP {
            for i in 0..self.villagers.len() {
                if self.monuments.iter().any(|m| m.honoree == i) { continue; }
                let Some(st) = self.tales.get(&i).and_then(|t| crate::sim::story::detect_story(t)) else { continue };
                if st.intensity < WORTHY { continue; }
                // the lord pays for the honor of the country: the
                // nearest castle's treasury fronts it
                let Some(ci) = (0..self.sites.len()).filter(|&c| self.sites[c].kind == country::SiteKind::Castle)
                    .min_by_key(|&c| (self.sites[c].centre.0 - self.village.0).abs()
                        + (self.sites[c].centre.1 - self.village.1).abs()) else { continue };
                if self.sites[ci].treasury < MONUMENT_COST { continue; }
                self.sites[ci].treasury -= MONUMENT_COST;
                // it rises where the village walks - near the market cross
                let spot = self.map.nearest_passable(self.village.0 + 3, self.village.1 + 3)
                    .unwrap_or(self.village);
                let name = vill_name(i);
                self.monuments.push(Monument { pos: spot, honoree: i,
                    title: st.title.clone(), build_until: tick + BUILD_DAYS, stood_on: None,
                    kind: MonumentKind::Column });
                self.map.occupy(spot.0, spot.1);
                self.log.push(format!("t{}: {}'s lord pays for a monument to {} - {}. It rises at the cross.",
                    tick, self.sites[ci].name, name, st.title));
                break;
            }
        }
        // the org's own ring: standing enough and the purse to raise
        // it, it resolves to set a circle of stones - the greatest
        // honor a village keeps
        const ORG_RING_COST: f32 = 200.0; // five times a column
        const ORG_STANDING: usize = 8;
        const CIRCLE_BUILD_DAYS: u64 = 40;
        if self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.len() >= ORG_STANDING && o.resources >= ORG_RING_COST
                && o.influence_pool >= 5.0).unwrap_or(false)
            && !self.monuments.iter().any(|m| m.kind == MonumentKind::Circle) {
            if let Some(o) = self.world.get_component_mut::<Organization>(self.org) {
                o.resources -= ORG_RING_COST;
            }
            let spot = self.map.nearest_passable(self.village.0 - 4, self.village.1 - 4)
                .unwrap_or((self.village.0 - 4, self.village.1 - 4));
            let oname = self.world.get_component::<Organization>(self.org)
                .map(|o| o.name.clone()).unwrap_or("the guild".into());
            self.monuments.push(Monument { pos: spot, honoree: usize::MAX,
                title: format!("the ring of {}", oname), build_until: tick + CIRCLE_BUILD_DAYS,
                stood_on: None, kind: MonumentKind::Circle });
            self.map.occupy(spot.0, spot.1);
            self.log.push(format!("t{}: {} resolves to raise a ring of stones - the stones stand on the hill by the village.",
                tick, oname));
        }
        // it stands when its days are done
        for m in self.monuments.iter_mut().filter(|m| m.stood_on.is_none() && tick >= m.build_until) {
            m.stood_on = Some(tick);
            if m.kind == MonumentKind::Circle {
                self.log.push(format!("t{}: the stones stand - {}'s ring is raised on the hill.", tick, m.title));
            } else {
                let name = if m.honoree == usize::MAX { "the one who moved on" } else { vill_name(m.honoree) };
                self.log.push(format!("t{}: the monument stands - {} is remembered. {}'s stone rises at the cross.",
                    tick, name, name));
            }
        }
        // and the folk who pass it remember: the monument's pull
        // reaches further than a cairn's
        for m in self.monuments.iter().filter(|m| m.stood_on.is_some()) {
            // the ring's pull reaches the whole village - the org's own
            // monument is the bigger mark
            let reach = if m.kind == MonumentKind::Circle { 30.0 } else { REMEMBER_NEAR };
            let odds = if m.kind == MonumentKind::Circle { 20 } else { 30 };
            for &v in &self.villagers.clone() {
                if villager::is_dead(&self.world, v)
                    || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                let near = self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - m.pos.0 as f32).abs() + (p.y - m.pos.1 as f32).abs() <= reach)
                    .unwrap_or(false);
                if !near || self.rng.below(odds) != 0 { continue; }
                let honoree = if m.honoree == usize::MAX { self.org } else { self.villagers[m.honoree] };
                crate::sim::psychology::record_episode(&mut self.world, v,
                    crate::sim::psychology::Episode { with: honoree,
                        kind: crate::sim::psychology::EpisodeKind::Remembrance });
            }
        }
        // a disbanded org loses its ring before it stands - no pile for
        // a guild that isn't
        let org_gone = self.world.get_component::<Organization>(self.org).is_none();
        if org_gone {
            self.monuments.retain(|m| !(m.kind == MonumentKind::Circle
                && m.stood_on.is_none()));
        }
    }
}
