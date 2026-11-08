//! The season hands: country laborers come in for the big work on their
//! own cadence, drink the tap dry, and leave the night's brawls behind
//! them. The village remembers the fists - and the wives' petition
//! counts them.

use crate::session::*;

/// The season's cadence: every ~120 ticks a work call goes out
const SEASON_GAP: u64 = 120;
/// The day in the cycle the call lands
const SEASON_AT: u64 = 77;
/// How long the season lasts the hands
const SEASON_DAYS: u64 = 30;
/// The odds a drunk hand's night ends in the mud, each check
const BRAWL_ODDS: i32 = 25;

impl Game {
    /// The laborers' day: the call, the season's comings and goings,
    /// and the fists the nights leave behind
    pub(in crate::session) fn laborer_tick(&mut self, tick: u64) {
        // the work call: flag country folk and walk them in for the season
        if tick % SEASON_GAP == SEASON_AT {
            let n = 2 + self.rng.below(4);
            let door = self.town_wall;
            let mid = ((door.0 + door.2) / 2, (door.1 + door.3) / 2);
            let mut flagged = 0;
            let villagers = self.villagers.clone();
            for v in villagers {
                if flagged >= n { break; }
                if self.world.get_component::<country::Outlander>(v).is_none()
                    || self.world.get_component::<villager::Laborer>(v).is_some()
                    || villager::is_dead(&self.world, v) { continue; }
                self.world.add_component(v, villager::Laborer { until: tick + SEASON_DAYS });
                self.send_far(v, mid, Goal::SeasonWork(mid));
                self.log.push(format!("t{}: the season's hands come in for the work.", tick));
                flagged += 1;
            }
        }
        // the season ends and the nights take their toll
        let villagers = self.villagers.clone();
        for v in villagers.iter().copied() {
            let Some(l) = self.world.get_component::<villager::Laborer>(v).map(|l| l.until) else { continue };
            if tick >= l {
                self.world.remove_component::<villager::Laborer>(v);
                // home is their site's ground - the season walk runs
                // both ways: send_far to the centre stands them there
                if let Some(centre) = self.world.get_component::<country::Outlander>(v)
                    .and_then(|o| self.sites.get(o.site)).map(|s| s.centre) {
                    self.send_far(v, centre, Goal::SeasonWork(centre));
                }
                continue;
            }
            // a drunk hand near the tavern: the night ends in the mud,
            // a feud's born, and the village remembers the fists
            let drunk = self.world.get_component::<villager::Drunk>(v)
                .map(|d| d.until > tick).unwrap_or(false);
            if drunk && self.rng.below(100) < BRAWL_ODDS {
                let wit = villagers.iter().copied().find(|&w| w != v
                    && !villager::is_dead(&self.world, w)
                    && self.world.get_component::<country::Outlander>(w).is_none()
                    && self.world.get_component::<Pos>(w).zip(self.world.get_component::<Pos>(v))
                        .map(|(a, b)| (a.x - b.x).abs() + (a.y - b.y).abs() < 12.0).unwrap_or(false));
                if let Some(wit) = wit {
                    crate::game::feuds::spawn_feud(&mut self.world, v, wit, "a stranger's fist in the street");
                    if let Some(h) = self.world.get_component_mut::<crate::sim::illness::Health>(v) {
                        h.condition = (h.condition - 0.15).max(0.05);
                    }
                    desires::add_grievance(&mut self.world, wit,
                        crate::game::desires::Problem::Rowdy, tick + 300);
                    self.log.push(format!("t{}: a season hand's night ends in the mud - {} saw it.",
                        tick, self.name_of(wit)));
                    self.world.remove_component::<villager::Drunk>(v);
                }
            }
        }
    }
}
