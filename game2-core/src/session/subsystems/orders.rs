//! The order's own month (#988): the lay brothers' labor keeps the
//! house's books and its store, the charter holds the roster at strength
//! (an order doesn't bleed members the way a village does), a fat house
//! plants a daughter grange under the same rule, and the richer the
//! house grows the nearer the seizing hand hovers over it

use crate::session::*;
use crate::session::country::{Outlander, SiteKind};

/// The order keeps its books monthly
const ORDER_EVERY: u64 = 30;
/// A month of a lay brother's labor earns the house this much gold
const LAY_LABOR: f32 = 3.0;
/// ... and puts this many servings in the grange's store
const LAY_SERVINGS: u32 = 2;
/// The charter holds the house at strength: the rolls refill from the
/// site's own outlanders, so the order keeps its numbers
const CHARTER_ROSTER: usize = 6;
/// A house needs this much over its endowment before it plants a daughter
const DAUGHTER_COST: f32 = 200.0;
/// A daughter is planted no farther than this from the mother house
const DAUGHTER_REACH: i32 = 60;
/// The baseline pull of the crown's eye each month
const RISK_TICK: f32 = 0.02;
/// A fat house draws the eye faster
const RISK_TREASURY: f32 = 0.0002;
/// The seizure threshold and the share the strongbox loses
const RISK_SEIZE: f32 = 1.0;
const SEIZE_SHARE: f32 = 0.6;

impl Game {
    /// The order's month: labor accrues, the charter holds the roster,
    /// a rich house franchises a daughter, and accumulation draws the
    /// expropriating hand - the writ's own answer to a rival treasury
    pub(in crate::session) fn order_tick(&mut self, tick: u64) {
        if tick % ORDER_EVERY != 0 { return; }
        for si in 0..self.sites.len() {
            let Some(mut o) = self.sites[si].sisterhood.take() else { continue };
            // the dead leave the rolls; the house itself endures - the
            // charter refills the ranks from the site's own outlanders
            o.members.retain(|&v| !villager::is_dead(&self.world, v));
            o.lay.retain(|&v| !villager::is_dead(&self.world, v));
            let taken: crate::engine::det::HashSet<Entity> =
                o.members.iter().chain(o.lay.iter()).copied().collect();
            // the charter holds the house at strength: the site's own
            // spare outlanders fill first, then any living villager -
            // a novice takes the habit, and the habit takes them to
            // the house
            let mut recruits: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&v| !taken.contains(&v) && !villager::is_dead(&self.world, v)
                    && self.world.get_component::<Outlander>(v)
                        .map(|out| out.site == si).unwrap_or(true))
                .collect();
            recruits.sort_by_key(|&v| if self.world.get_component::<Outlander>(v).is_some() { 0 } else { 1 });
            for v in recruits {
                if o.members.len() + o.lay.len() >= CHARTER_ROSTER { break; }
                o.lay.push(v);
                if self.world.get_component::<Outlander>(v).is_none() {
                    self.world.add_component(v, Outlander { site: si });
                }
            }

            // ora et labora: the conversi work the grange
            let lay_n = o.lay.len() as f32;
            self.sites[si].treasury += lay_n * LAY_LABOR;
            if let Some(fs) = self.world
                .get_component_mut::<villager::FoodSource>(self.sites[si].store)
            {
                fs.servings += (lay_n as u32) * LAY_SERVINGS;
            }

            // franchising: a house with the endowment plants a daughter
            // grange under the same charter - the rule, not the walls,
            // is what travels
            if self.sites[si].treasury > DAUGHTER_COST * 2.0 {
                if let Some(fi) = self.daughter_pick(si) {
                    self.sites[si].treasury -= DAUGHTER_COST;
                    self.sites[fi].treasury += DAUGHTER_COST;
                    self.sites[fi].mother = Some(si);
                    self.log.push(format!("t{}: {} plants a daughter house at {} under the same rule.",
                        tick, o.charter, self.sites[fi].name));
                }
            }

            // the crown's eye: the richer the house, the nearer the hand
            o.risk += RISK_TICK + self.sites[si].treasury * RISK_TREASURY;
            if o.risk >= RISK_SEIZE {
                let take = self.sites[si].treasury * SEIZE_SHARE;
                self.sites[si].treasury -= take;
                if let Some(liege) = self.sites[si].liege {
                    self.sites[liege].treasury += take;
                }
                let grabber = self.sites[si].liege.map(|l| self.sites[l].name).unwrap_or("the crown");
                self.log.push(format!("t{}: {}'s men descended on {} and carried off the strongboxes - {:.0} gold for {}.",
                    tick, grabber, self.sites[si].name, take, grabber));
                o.risk = 0.15;
            }
            self.sites[si].sisterhood = Some(o);
        }
    }

    /// The nearest free farm within reach of the mother house - the
    /// grange an order plants its daughter on, deterministic by site order
    fn daughter_pick(&self, mother: usize) -> Option<usize> {
        let (mx, my) = self.sites[mother].centre;
        self.sites.iter().enumerate()
            .filter(|(i, s)| *i != mother && s.kind == SiteKind::Farm && s.mother.is_none())
            .map(|(i, s)| { let dx = s.centre.0 - mx; let dy = s.centre.1 - my; (i, dx * dx + dy * dy) })
            .filter(|(_, d2)| *d2 <= DAUGHTER_REACH * DAUGHTER_REACH)
            .min_by_key(|(_, d2)| *d2)
            .map(|(i, _)| i)
    }
}
