//! The funeral procession: when the churchyard's day comes for a body,
//! the mourners form a column and walk it - the officiant leads, the
//! pallbearers shoulder the dead from where it fell, and the file
//! ends at the grave slot. Pallbearers are the dead's closest ties (the neighbours
//! in a pinch), a handful of faithful followers trail, and the priest -
//! or the old faith's elder when the dead kept the old ways - walks
//! ahead to say the words. One column at a time: burials that can't
//! form wait for the next day's check, and a body with no road or no
//! one to carry it is laid in where it lies, as before.

use crate::session::*;

/// Pallbearers at the shoulder
const CARRIERS: usize = 4;
/// Followers behind the box, give or take the day's mood - a
/// funeral benches the parish, so the file stays a handful, not a
/// holiday
const FOLLOWERS_MIN: i32 = 4;
const FOLLOWERS_SPREAD: i32 = 4;
/// A mourner's bond to the dead counts this much toward walking
const TIE_WEIGHT: f32 = 1.0;
/// Piety and devotion make a mourner likelier to fall in
const PIOUS_WEIGHT: f32 = 0.5;
const DEVOTION_WEIGHT: f32 = 0.3;

impl Game {
    /// Form the funeral column for `dead`, bound for grave `slot`.
    /// False when it can't march - no road to the plot, nobody to carry
    /// it, or another column already out - and the burial check falls
    /// back to laying the body in where it lies
    pub(in crate::session) fn begin_funeral(&mut self, tick: u64, dead: Entity, slot: (i32, i32)) -> bool {
        if self.formation.is_some() { return false; }
        if !villager::is_dead(&self.world, dead)
            || self.world.get_component::<villager::Buried>(dead).is_some() { return false; }
        let Some(body) = self.tile_of(dead) else { return false; };
        // the column's road: rally at the body, then the charted way to
        // the plot, waypoints as tile centres
        let Some(tiles) = self.trail_path(body, slot) else { return false; };
        let mut route: Vec<(f32, f32)> = vec![(body.0 as f32 + 0.5, body.1 as f32 + 0.5)];
        for t in tiles {
            let w = (t.0 as f32 + 0.5, t.1 as f32 + 0.5);
            if route.last().map(|&l| (l.0 - w.0).abs() > 0.01 || (l.1 - w.1).abs() > 0.01).unwrap_or(true) {
                route.push(w);
            }
        }
        let end = (slot.0 as f32 + 0.5, slot.1 as f32 + 0.5);
        if route.last().map(|&l| (l.0 - end.0).abs() > 0.01 || (l.1 - end.1).abs() > 0.01).unwrap_or(true) {
            route.push(end);
        }
        if route.len() < 2 { return false; }

        // who may march: the living, the village's own, grown folk with
        // hands to spare - never another column's people
        let eligible = |e: Entity| e != dead
            && !villager::is_dead(&self.world, e)
            && self.site_of(e).is_none()
            && !crate::game::dungeon::unavailable(&self.world, e)
            && self.world.get_component::<Craft>(e).is_some()
            && self.world.get_component::<villager::InFormation>(e).is_none();
        let dist = |e: Entity| self.tile_of(e)
            .map(|t| (t.0 - body.0).abs() + (t.1 - body.1).abs()).unwrap_or(i32::MAX);

        // the dead's bonds, warmest first - the shoulders of the box
        let mut ties: Vec<(f32, Entity)> = relationships::bonds_of(&self.world, dead).iter()
            .filter_map(|&r| self.world.get_component::<relationships::Relationship>(r).map(|rel| {
                let other = if rel.from == dead { rel.other } else { rel.from };
                (rel.strength, other)
            }))
            .filter(|&(_, o)| eligible(o))
            .collect();
        ties.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| routine::mix(a.1, 7).cmp(&routine::mix(b.1, 7))));
        let tie_of = |e: Entity| ties.iter().find(|&&(_, o)| o == e).map(|&(s, _)| s).unwrap_or(0.0);

        let mut carriers: Vec<Entity> = ties.iter()
            .filter(|&&(s, _)| s > 0.0).map(|&(_, o)| o).take(CARRIERS).collect();
        // no warm ties - or too few shoulders - the neighbours nearest
        // the body make up the box
        let mut nearest: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&e| eligible(e) && !carriers.contains(&e))
            .collect();
        nearest.sort_by_key(|&e| (dist(e), routine::mix(e, 11)));
        if carriers.is_empty() {
            carriers = nearest.iter().copied().take(CARRIERS).collect();
        } else {
            for &e in &nearest {
                if carriers.len() >= CARRIERS { break; }
                carriers.push(e);
            }
        }
        if carriers.is_empty() { return false; }

        // who says the words: the dead's own faith picks first - the
        // priest for the parish, the old faith's pious elder for the
        // stones - the other faith's officer behind them, and a column
        // with no clergy walks all the same
        let free = |e: Entity| eligible(e)
            && !self.travels.iter().any(|t| t.who == e);
        let priest = self.villagers.iter().copied().find(|&e| free(e)
            && self.world.get_component::<Craft>(e)
                .map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
        let leader = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .and_then(|c| c.leader).filter(|&e| free(e));
        let old_elder = {
            let mut folk: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&e| free(e) && routine::keeps_old_ways(&self.world, e)).collect();
            folk.sort_by_key(|&e| (dist(e), routine::mix(e, 13)));
            folk.iter().copied().find(|&e| routine::has_trait(&self.world, e,
                crate::game::traits::Trait::Pious))
                .or_else(|| folk.first().copied())
        };
        let old = routine::keeps_old_ways(&self.world, dead);
        let mut order: Vec<Option<Entity>> = if old {
            vec![old_elder, priest, leader]
        } else {
            vec![priest, leader, old_elder]
        };
        order.dedup();
        let officiant = order.into_iter().flatten()
            .find(|&e| !carriers.contains(&e)).filter(|&e| eligible(e));

        // the faithful fall in behind the box: ties, piety and devotion
        // make the mourners, the day's count lands anywhere in the band
        let mut mourners: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&e| eligible(e) && !carriers.contains(&e) && Some(e) != officiant).collect();
        mourners.sort_by(|&a, &b| {
            let score = |e: Entity| tie_of(e).max(0.0) * TIE_WEIGHT
                + if routine::has_trait(&self.world, e, crate::game::traits::Trait::Pious) { PIOUS_WEIGHT } else { 0.0 }
                + self.world.get_component::<crate::sim::religion::Faith>(e)
                    .map(|f| f.devotion * DEVOTION_WEIGHT).unwrap_or(0.0);
            score(b).partial_cmp(&score(a)).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| routine::mix(a, 17).cmp(&routine::mix(b, 17)))
        });
        // the day's count lands anywhere in the band, hashed off the
        // dead so no shared roll moves for the sake of a funeral
        let want = FOLLOWERS_MIN + (routine::mix(dead, 23) % FOLLOWERS_SPREAD as u64) as i32;
        let want = want.max(0) as usize;
        let followers: Vec<Entity> = mourners.into_iter().take(want).collect();

        let mut members: Vec<Entity> = Vec::new();
        if let Some(o) = officiant { members.push(o); }
        members.extend(carriers.iter().copied());
        members.extend(followers.iter().copied());

        let name = self.name_of(dead);
        if self.start_funeral(dead, slot, carriers, officiant, members, route) {
            self.log.push_ev(
                EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Notable),
                format!(
                    "t{}: the parish forms a funeral procession for {}.",
                    tick, name
                ),
            );
            true
        } else { false }
    }
}
