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
                    title: st.title.clone(), build_until: tick + BUILD_DAYS, stood_on: None });
                self.map.occupy(spot.0, spot.1);
                self.log.push(format!("t{}: {}'s lord pays for a monument to {} - {}. It rises at the cross.",
                    tick, self.sites[ci].name, name, st.title));
                break;
            }
        }
        // it stands when its days are done
        for m in self.monuments.iter_mut().filter(|m| m.stood_on.is_none() && tick >= m.build_until) {
            m.stood_on = Some(tick);
            let name = vill_name(m.honoree);
            self.log.push(format!("t{}: the monument stands - {} is remembered. {}'s stone rises at the cross.",
                tick, name, name));
        }
        // and the folk who pass it remember: the monument's pull
        // reaches further than a cairn's
        for m in self.monuments.iter().filter(|m| m.stood_on.is_some()) {
            for &v in &self.villagers.clone() {
                if villager::is_dead(&self.world, v)
                    || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                let near = self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - m.pos.0 as f32).abs() + (p.y - m.pos.1 as f32).abs() <= REMEMBER_NEAR)
                    .unwrap_or(false);
                if !near || self.rng.below(30) != 0 { continue; }
                let honoree = self.villagers[m.honoree];
                crate::sim::psychology::record_episode(&mut self.world, v,
                    crate::sim::psychology::Episode { with: honoree,
                        kind: crate::sim::psychology::EpisodeKind::Remembrance });
            }
        }
    }
}
