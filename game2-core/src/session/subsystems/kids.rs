//! Child labor: when a field needs hands, a nearby idle kid can be
//! hired for the day - the hands help the crop and the kid takes a
//! child's wages. But a small hand in the rows gets hurt sometimes,
//! and the village doesn't like to see it.

use crate::session::*;

/// A child's wages for a day in the rows
const KID_DAY_WAGE: f32 = 0.5;
/// How far a hirer reaches for a hand
const KID_NEAR: f32 = 14.0;
/// The chance in a hundred a worked day hurts the kid
const HURT_ODDS: i32 = 4;

impl Game {
    /// The kid labor day: farmers near a needy field hire nearby idle
    /// kids, the worked field grows faster for the hands, and the day's
    /// pay - and its risks - come due
    pub(in crate::session) fn kid_labor_tick(&mut self, tick: u64) {
        // pay off yesterday's hands first: wages, the day's growth, and
        // the hurt a small hand can take
        let workers: Vec<Entity> = self.world.query::<vocation::KidWork>()
            .iter().copied().collect();
        for kid in workers {
            let Some(w) = self.world.get_component::<vocation::KidWork>(kid)
                .map(|w| (w.field, w.hirer, w.pay)) else { continue };
            let (field, hirer, pay) = w;
            // a child's wages, if the hirer's purse holds them
            let purse = self.world.get_component_mut::<Purse>(hirer)
                .and_then(|p| { if p.gold >= pay { p.gold -= pay; Some(()) } else { None } }).is_some();
            if purse {
                if let Some(kp) = self.world.get_component_mut::<Purse>(kid) { kp.gold += pay; }
            }
            // the day's hands help the crop - a weeded row ripens faster
            if let Some(c) = self.world.get_component_mut::<villager::Crop>(field) {
                c.progress += 0.5;
            }
            // and the day counts like an errand toward coming of age
            if let Some(g) = self.world.get_component_mut::<vocation::KidGrowth>(kid) {
                if let Some((_, n)) = g.errands.iter_mut().find(|(e, _)| *e == hirer) { *n += 1; }
                else { g.errands.push((hirer, 1)); }
            }
            // small hands take their hurts: a worked day can cost blood,
            // and the village speaks of what it sees
            if self.rng.below(100) < HURT_ODDS {
                if let Some(h) = self.world.get_component_mut::<illness::Health>(kid) {
                    h.condition = (h.condition - 0.2).max(0.05);
                }
                let serious = self.rng.below(3) == 0;
                let witnessed = self.villagers.iter().copied().any(|v| v != kid
                    && !villager::is_dead(&self.world, v)
                    && (household::home_of(&self.world, v) == household::home_of(&self.world, kid)
                        || self.world.get_component::<Pos>(v).zip(self.world.get_component::<Pos>(kid))
                            .map(|(a, b)| (a.x - b.x).abs() + (a.y - b.y).abs() < 10.0).unwrap_or(false)));
                if serious || witnessed {
                    desires::add_grievance(&mut self.world, kid,
                        crate::game::desires::Problem::ChildLabor, tick + 300);
                }
            }
            self.world.remove_component::<vocation::KidWork>(kid);
        }
        // hire today's hands: a field that needs them, a farmer who
        // tends it, an idle kid near enough to call
        for i in 0..self.crop_fields.len() {
            let field = self.crop_fields[i];
            let Some(fp) = self.world.get_component::<Pos>(field).copied() else { continue };
            // one hand to a field - no crowds in the rows
            if self.world.query::<vocation::KidWork>().iter()
                .any(|&k| self.world.get_component::<vocation::KidWork>(k)
                    .map(|w| w.field == field).unwrap_or(false)) { continue; }
            // the hirer: a living farmer near enough to tend the crop
            let hirer = self.villagers.iter().copied().find(|&v|
                !villager::is_dead(&self.world, v)
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == vocation::Vocation::Farmer).unwrap_or(false)
                && self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - fp.x).abs() + (p.y - fp.y).abs() < KID_NEAR).unwrap_or(false));
            let Some(hirer) = hirer else { continue; };
            // the hand: an idle kid, not on a job, near enough to hear
            let kid = self.villagers.iter().copied().find(|&k|
                !villager::is_dead(&self.world, k)
                && self.world.get_component::<vocation::Kid>(k).is_some()
                && self.world.get_component::<vocation::KidWork>(k).is_none()
                && self.world.get_component::<OddJob>(k).is_none()
                && !self.travels.iter().any(|t| t.who == k)
                && self.world.get_component::<Pos>(k)
                    .map(|p| (p.x - fp.x).abs() + (p.y - fp.y).abs() < KID_NEAR).unwrap_or(false));
            let Some(kid) = kid else { continue; };
            let pay = (KID_DAY_WAGE + self.rng.below(4) as f32 * 0.1) * self.wage_mult();
            self.world.add_component(kid, vocation::KidWork { field, hirer, pay });
            self.send(kid, Goal::FarmWork(field));
        }
    }
}
