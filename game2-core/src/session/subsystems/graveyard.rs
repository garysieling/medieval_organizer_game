//! The churchyard: a body lies in where it fell for a few days so the
//! neighbours can see it, then the funeral carries it to a grave slot
//! beside the church and it never moves again. Village dead only - the
//! country's folk keep the old custom, and the martyr's plot stays where
//! the gibbet made it. #674 builds the ceremony around this carry.

use crate::session::*;

/// Days a body lies in before the funeral
const LYING_IN_DAYS: u64 = 3;
/// The most plots the churchyard keeps - a village this size never fills it
const GRAVE_SLOTS: usize = 96;
/// How far from the church a plot may be (Chebyshev)
const GRAVE_REACH: i32 = 7;

impl Game {
    /// The churchyard's day: whoever has lain in long enough is carried
    /// to the next grave slot, in the order they died
    pub(in crate::session) fn graveyard_tick(&mut self, tick: u64) {
        if self.graveyard.is_none() {
            self.graveyard = Some(self.plan_graveyard());
        }
        let has_slots = self.graveyard.as_ref().map(|g| !g.slots.is_empty()).unwrap_or(false);
        if !has_slots { return; }
        // the day's burials, oldest death first (roster order breaks ties)
        let mut due: Vec<(u64, usize)> = (0..self.villagers.len())
            .filter(|&i| {
                let v = self.villagers[i];
                villager::is_dead(&self.world, v)
                    && self.world.get_component::<villager::Buried>(v).is_none()
                    && self.world.get_component::<country::Outlander>(v).is_none()
                    && !self.at_martyrs_grave(v)
                    && self.died_on.get(i).and_then(|d| *d)
                        .map(|d| tick >= d + LYING_IN_DAYS).unwrap_or(false)
            })
            .filter_map(|i| self.died_on[i].map(|d| (d, i)))
            .collect();
        due.sort_unstable();
        for (_, i) in due {
            // one column at a time: while a funeral (or any column) is
            // out, the rest of the day's burials wait for tomorrow
            if self.formation.is_some() { break; }
            let Some((sx, sy)) = self.graveyard.as_ref()
                .and_then(|g| g.slots.get(g.taken)).copied() else { break };
            let v = self.villagers[i];
            // the mourners carry the body to its plot; when no column
            // can form the old road holds - laid in where the slot is
            if !self.begin_funeral(tick, v, (sx, sy)) {
                if let Some(gy) = &mut self.graveyard { gy.taken += 1; }
                if let Some(p) = self.world.get_component_mut::<Pos>(v) {
                    *p = Pos { x: sx as f32, y: sy as f32 };
                }
                self.world.add_component(v, villager::Buried);
                let name = vill_name(i);
                // the priest says the words where the village keeps one
                let said = if self.church.is_some() {
                    " The priest says the words."
                } else { "" };
                self.log.push(format!("t{}: {} is laid to rest in the churchyard.{}", tick, name, said));
                continue;
            }
            if let Some(gy) = &mut self.graveyard { gy.taken += 1; }
        }
    }

    /// Whether this dead villager is the one on the martyr's plot - the
    /// gibbet's grave is the marker itself and stays where it is
    fn at_martyrs_grave(&self, v: Entity) -> bool {
        let Some(mg) = self.martyr_grave else { return false };
        self.world.get_component::<Pos>(v)
            .map(|p| (p.x.round() as i32, p.y.round() as i32) == mg)
            .unwrap_or(false)
    }

    /// The churchyard's plots: the grass tiles nearest the church, in
    /// burial order (closest first, then rows). Planned once; an empty
    /// list just means no burials move - the dead keep lying where they fell.
    fn plan_graveyard(&self) -> Graveyard {
        let mut slots = vec![];
        if let Some(ch) = self.church {
            if let Some(cp) = self.world.get_component::<Pos>(ch) {
                let (cx, cy) = (cp.x.round() as i32, cp.y.round() as i32);
                let mut cand: Vec<(i32, i32)> = vec![];
                for dy in -GRAVE_REACH..=GRAVE_REACH {
                    for dx in -GRAVE_REACH..=GRAVE_REACH {
                        let (x, y) = (cx + dx, cy + dy);
                        if dx.abs().max(dy.abs()) > GRAVE_REACH { continue; }
                        if !matches!(self.map.get(x, y), Some(Tile::Grass)) { continue; }
                        if self.map.is_occupied(x, y) { continue; }
                        cand.push((x, y));
                    }
                }
                cand.sort_by_key(|&(x, y)| {
                    let d = (x - cx).abs().max((y - cy).abs());
                    (d, y, x)
                });
                cand.truncate(GRAVE_SLOTS);
                slots = cand;
            }
        }
        // the lychgate marks the way in: the walkable grass nearest the
        // first plot, on the church side of it
        let gate = (|| {
            let ch = self.church?;
            let cp = self.world.get_component::<Pos>(ch)?;
            let (cx, cy) = (cp.x.round() as i32, cp.y.round() as i32);
            let (fx, fy) = *slots.first()?;
            let mut cand: Vec<(i32, i32)> = vec![];
            for dy in -GRAVE_REACH..=GRAVE_REACH {
                for dx in -GRAVE_REACH..=GRAVE_REACH {
                    let (x, y) = (cx + dx, cy + dy);
                    // any walkable ground will do - the paths and lanes
                    // into the churchyard aren't plots, and the gate
                    // belongs over one
                    if slots.contains(&(x, y))
                        || self.map.is_occupied(x, y) || !self.map.passable(x, y) { continue; }
                    cand.push((x, y));
                }
            }
            cand.sort_by_key(|&(x, y)| {
                ((x - fx).abs().max((y - fy).abs()), (x - cx).abs().max((y - cy).abs()), y, x)
            });
            cand.first().copied()
        })();
        Graveyard { slots, taken: 0, gate }
    }
}
