//! The imperial levy: the emperor's collector rides through and the
//! granary bleeds for a project nobody here will ever see. Compliance
//! leaves the village thin but alive; the lord can intercede for a
//! stretch if he holds them in esteem; and the hunger that follows is
//! the village's own to bear.

use crate::session::*;

/// Days the levy runs: a harvest or two gone
const LEVY_DAYS: u64 = 90;
/// Servings the crown takes a day - thin but alive on compliance
const LEVY_RATE: u32 = 6;
/// A cart loads the crown's due this often - the take is real cargo on
/// the road now, not a bleed the books never saw (#970)
const LEVY_CART_EVERY: u64 = 10;
/// The lord's intercession holds for a stretch
const EASE_DAYS: u64 = 30;
/// The standing the org needs to press the demand
const ORG_PRESS: usize = 6;

impl Game {
    /// The levy's day: the collector comes, the org presses, the lord
    /// may intercede, the crown's cart loads for the keep every stretch,
    /// and one day it lifts
    pub(in crate::session) fn imperial_tick(&mut self, tick: u64) {
        // the collector comes rarely and for long enough to hurt
        if self.imperial_project.is_none() && tick % 360 == 45 && self.rng.below(3) == 0 {
            self.imperial_project = Some(ImperialProject { until: tick + LEVY_DAYS,
                rate: LEVY_RATE, eased_until: 0 });
            self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: the emperor's collector rides through - the crown levies the stores for a great work far away.", tick));
        }
        let Some(proj) = self.imperial_project.as_ref() else { return };
        let (until, rate, mut eased) = (proj.until, proj.rate, proj.eased_until);
        if tick >= until {
            self.imperial_project = None;
            self.log.push_ev(
                EvTag::new(Thread::Law, Valence::Positive, Sev::Notable),
                format!(
                    "t{}: the collector's road turns back - the emperor's levy is lifted.",
                    tick
                ),
            );
            return;
        }
        // the org presses the lord to intercede: standing enough, and
        // held in esteem enough that he'll stand the crown's displeasure
        if eased == 0 {
            let presses = self.world.get_component::<Organization>(self.org)
                .map(|o| o.members.len() >= ORG_PRESS).unwrap_or(false);
            if presses && self.lord_esteem >= 0.5 && tick % 10 == 0 && self.rng.below(3) == 0 {
                eased = tick + EASE_DAYS;
                self.imperial_project.as_mut().unwrap().eased_until = eased;
                self.log.push_ev(
                    EvTag::new(Thread::Law, Valence::Positive, Sev::Minor),
                    format!(
                        "t{}: the lord intercedes with the crown - the levy eases for a stretch.",
                        tick
                    ),
                );
            }
        }
        // the crown's cart: the take rides the road as real provisions
        // for the keep, where an ambush can reach it - the lord skims
        // his share off the top as it loads, like always
        if tick % LEVY_CART_EVERY != 5 { return; }
        let Some(rider) = self.collector
            .filter(|&c| !villager::is_dead(&self.world, c)
                && !crate::game::dungeon::unavailable(&self.world, c)
                && !self.travels.iter().any(|t| t.who == c)) else { return };
        let take = (if tick < eased { rate / 2 } else { rate }) * LEVY_CART_EVERY as u32;
        let took = self.world.get_component_mut::<villager::FoodSource>(self.market_food)
            .map(|f| { let n = take.min(f.servings); f.servings -= n; n }).unwrap_or(0);
        if took == 0 { return; }
        if let Some(s) = self.world.get_component_mut::<Satchel>(rider) {
            *s.goods.entry("provisions".into()).or_insert(0) += took;
        }
        self.takings += took as f32 * 0.3;
        let rp = self.world.get_component::<Pos>(rider).map(|p| (p.x, p.y));
        let Some((si, _)) = self.sites.iter().enumerate()
            .filter(|(_, s)| matches!(s.kind, country::SiteKind::Castle))
            .min_by_key(|(_, s)| rp.map(|(x, y)|
                ((s.centre.0 as f32 - x).powi(2) + (s.centre.1 as f32 - y).powi(2)) as i64)
                .unwrap_or(0)) else { return };
        let door = self.sites[si].homes.first()
            .and_then(|&k| self.world.get_component::<Pos>(k).copied())
            .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32))
            .unwrap_or(self.sites[si].centre);
        if self.send_far(rider, door, Goal::LevyCart(si)) {
            self.log.push_ev(
                EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                format!(
                    "t{}: the crown's cart loads for {} - {} provisions out of the store.",
                    tick, self.sites[si].name, took
                ),
            );
        }
    }
}
