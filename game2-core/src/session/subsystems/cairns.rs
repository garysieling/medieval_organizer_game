//! Cairns: when the village loses one of its own and the dead go
//! unmourned, the ones who loved them raise a pile of stones where folk
//! walk. Every stone a hand lays makes it taller, and the dead it keeps
//! are not forgotten.

use crate::session::*;

/// The most cairns the village stands at once
const CAIRN_CAP: usize = 4;
/// Days gone before a death ripens into a pile
const RIPEN_DAYS: u64 = 7;
/// The tallest a pile gets - folk still lay stones, they just make no
/// bigger mark
const FULL_PILE: u32 = 12;

impl Game {
    /// The cairns' day: a death ripens into a site, a bonded hand lays
    /// a stone, a passerby remembers
    pub(in crate::session) fn cairn_tick(&mut self, tick: u64) {
        // ripen: the dead a week gone and unmourned get their pile
        if self.cairns.len() < CAIRN_CAP {
            for i in 0..self.villagers.len() {
                let Some(d) = self.died_on.get(i).and_then(|x| *x) else { continue };
                if tick < d + RIPEN_DAYS { continue; }
                if self.cairns.iter().any(|c| c.honoree == i) { continue; }
                let honoree = self.villagers[i];
                // nobody raises a stone for a stranger: someone has to
                // have loved them
                // a nod isn't a bond: the pile rises for the ones a
                // friend or family weeps for, not a passing acquaintance
                let loved = self.villagers.iter().copied()
                    .filter(|&v| v != honoree && !villager::is_dead(&self.world, v))
                    .any(|v| self.world.query::<relationships::Relationship>()
                        .iter().any(|&e| {
                            let r = self.world.get_component::<relationships::Relationship>(e).unwrap();
                            r.strength >= 0.5 && ((r.from == v && r.other == honoree)
                                || (r.other == v && r.from == honoree))
                        }));
                if !loved { continue; }
                // a wayside spot near where they lived - folk pass it
                let home = household::home_of(&self.world, honoree);
                let anchor = home.and_then(|h| self.world.get_component::<Pos>(h).copied())
                    .map(|p| (p.x.round() as i32, p.y.round() as i32))
                    .unwrap_or(self.village);
                let spot = self.map.nearest_passable(anchor.0 + 2, anchor.1 + 2)
                    .unwrap_or((anchor.0 + 2, anchor.1 + 2));
                let name = vill_name(i);
                let deed = self.world.get_component::<Craft>(honoree)
                    .map(|c| vocation::trade_name(c.vocation)).unwrap_or("the bench");
                self.cairns.push(Cairn { pos: spot, honoree: i,
                    deed: format!("{} of {}", name, deed), stones: 1 });
                self.map.occupy(spot.0, spot.1);
                self.log.push(format!("t{}: a cairn rises for {} - the first stone's laid where folk walk.", tick, name));
                break;
            }
        }
        // the pile grows: each day one hand lays one stone - the
        // bonded first, then any passerby
        for ci in 0..self.cairns.len() {
            let c = self.cairns[ci].pos;
            let honoree = self.villagers[self.cairns[ci].honoree];
            if self.cairns[ci].stones >= FULL_PILE { continue; }
            // the bonded take up a stone first - grief finds the work
            let mut layer = None;
            let mut passerby = false;
            for &v in &self.villagers.clone() {
                if villager::is_dead(&self.world, v)
                    || crate::game::dungeon::unavailable(&self.world, v)
                    || self.travels.iter().any(|t| t.who == v)
                    || self.seats.iter().any(|s| s.player == v) { continue; }
                let near = self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - c.0 as f32).abs() + (p.y - c.1 as f32).abs() <= 12.0)
                    .unwrap_or(false);
                if !near { continue; }
                let bonded = self.world.query::<relationships::Relationship>()
                    .iter().any(|&e| {
                        let r = self.world.get_component::<relationships::Relationship>(e).unwrap();
                        (r.from == v && r.other == honoree) || (r.other == v && r.from == honoree)
                    });
                if bonded { layer = Some(v); break; }
                // and the wayfarer lays one - a stone for a stranger is
                // cheap and still remembered
                if self.rng.below(20) == 0 { passerby = true; layer = Some(v); break; }
            }
            if let Some(v) = layer {
                self.cairns[ci].stones += 1;
                crate::sim::psychology::record_episode(&mut self.world, v,
                    crate::sim::psychology::Episode { with: honoree,
                        kind: crate::sim::psychology::EpisodeKind::Remembrance });
                if passerby && self.rng.below(10) == 0 {
                    let name = self.villagers.iter().position(|&x| x == v).map(vill_name).unwrap_or("a wayfarer");
                    self.log.push(format!("t{}: {} lays a stone on the cairn - {} is not forgotten.",
                        self.tick, name, self.cairns[ci].deed));
                }
            }
        }
    }
}
