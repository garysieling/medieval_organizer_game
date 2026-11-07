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
/// The lord's intercession holds for a stretch
const EASE_DAYS: u64 = 30;
/// The standing the org needs to press the demand
const ORG_PRESS: usize = 6;

impl Game {
    /// The levy's day: the collector comes, the store bleeds, the org
    /// presses, the lord may intercede, and one day it lifts
    pub(in crate::session) fn imperial_tick(&mut self, tick: u64) {
        // the collector comes rarely and for long enough to hurt
        if self.imperial_project.is_none() && tick % 360 == 45 && self.rng.below(3) == 0 {
            self.imperial_project = Some(ImperialProject { until: tick + LEVY_DAYS,
                rate: LEVY_RATE, eased_until: 0 });
            self.log.push(format!("t{}: the emperor's collector rides through - the crown levies the stores for a great work far away.", tick));
        }
        let Some(proj) = self.imperial_project.as_ref() else { return };
        let (until, rate, mut eased) = (proj.until, proj.rate, proj.eased_until);
        if tick >= until {
            self.imperial_project = None;
            self.log.push(format!("t{}: the collector's road turns back - the emperor's levy is lifted.", tick));
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
                self.log.push(format!("t{}: the lord intercedes with the crown - the levy eases for a stretch.", tick));
            }
        }
        let take = if tick < eased { rate / 2 } else { rate };
        // the store bleeds first - the village empties before the
        // country does
        if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
            let took = take.min(f.servings);
            f.servings -= took;
            // the lord's cut still flows - his treasury takes its share
            self.takings += took as f32 * 0.3;
        }
    }
}
