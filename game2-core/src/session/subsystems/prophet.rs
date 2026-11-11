//! The calling and the crowd: a villager with a voice takes up the word
//! when the grievances run deep (#878). The crowd it raises is fast and
//! thin - it gathers, it marches, it can't hold a seat. The movement
//! lives by a deadline the vision sets; when the day passes the crowd
//! either scatters or hardens into a remnant, and when the prophet dies
//! the movement mostly dies with them - unless the death itself
//! converts.

use crate::session::*;
use crate::game::prophetic::{self, Follower, Movement};

/// The calling is rare even when the pool runs deep - a few chances a
/// year, not one a season
const EMERGE_CHANCE: f32 = 0.005;
/// No calling before the village has had time to be wronged
const EMERGE_EARLIEST: u64 = 60;
/// The gather's cadence: the crowd walks out to the prophet's tile on
/// the holy-day beat
const GATHER_BEAT: u64 = 7;

impl Game {
    /// The movement's day: emergence, growth, the gather, the deadline,
    /// and the prophet's own mortality - in that order, so a death can't
    /// recruit on the same day it lands
    pub(in crate::session) fn prophet_tick(&mut self, tick: u64) {
        self.prophet_emerge(tick);
        self.prophet_grow(tick);
        self.prophet_gather(tick);
        self.prophet_deadline(tick);
        self.prophet_death(tick);
    }

    /// The calling: rare, and only where the grievances run deep - the
    /// village's most agency-grown adult takes up the word, and the
    /// vision names its day
    fn prophet_emerge(&mut self, tick: u64) {
        if self.prophet_movement.is_some() || tick < EMERGE_EARLIEST { return; }
        if prophetic::grievance_pool(&self.world, &self.villagers, tick) < prophetic::EMERGE_POOL { return; }
        if (self.rng.next() % 1000) as f32 / 1000.0 >= EMERGE_CHANCE { return; }
        let players: Vec<Entity> = self.seats.iter().map(|s| s.player).collect();
        let Some(prophet) = prophetic::pick_prophet(&self.world, &self.villagers, &players) else { return };
        self.prophet_movement = Some(Movement {
            prophet, since: tick,
            deadline: tick + prophetic::DEADLINE_DAYS,
            remnant: false, martyred: false,
        });
        let at = prophetic::prophet_tile(&self.world, self.prophet_movement.as_ref().unwrap())
            .map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5));
        self.log.push_ev_at_who(
            EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
            format!("t{}: {} has taken the calling - and names the day it all comes due.", tick, self.name_of(prophet)),
            at, None);
    }

    /// The growth: vision x grievance x reach, vastly faster than
    /// organizing - but every head the crowd gains is a thin one
    fn prophet_grow(&mut self, tick: u64) {
        let Some(m) = &self.prophet_movement else { return };
        if m.remnant || villager::is_dead(&self.world, m.prophet) { return; }
        let expected = prophetic::daily_growth(&self.world, m, &self.villagers, tick);
        if expected <= 0.0 { return; }
        let mut want = expected.floor() as usize;
        if (self.rng.next() % 1000) as f32 / 1000.0 < expected.fract() { want += 1; }
        if want == 0 { return; }
        // candidates: living adults not already the prophet's - the
        // roster order's the draw pool, the roll picks among it
        let mut pool: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && v != m.prophet
                && self.world.get_component::<Follower>(v).is_none()
                && self.world.get_component::<crate::game::lifecycle::Age>(v)
                    .map(|a| a.stage == crate::game::lifecycle::LifeStage::Adult
                        || a.stage == crate::game::lifecycle::LifeStage::Elder)
                    .unwrap_or(false))
            .collect();
        let mut have = prophetic::followers(&self.world, &self.villagers).len();
        for _ in 0..want {
            if have >= prophetic::CROWD_CAP || pool.is_empty() { break; }
            let i = self.rng.below(pool.len() as i32) as usize;
            let v = pool.swap_remove(i);
            self.world.add_component(v, Follower { since: tick, remnant: false });
            have += 1;
        }
    }

    /// The gather: on the holy-day beat the crowd walks out to the
    /// prophet's tile - the vigil's pattern, before there's a grave
    fn prophet_gather(&mut self, tick: u64) {
        let Some(m) = &self.prophet_movement else { return };
        if villager::is_dead(&self.world, m.prophet) { return; }
        if tick % GATHER_BEAT != 0 { return; }
        let (remnant, left, name) = (m.remnant, m.deadline.saturating_sub(tick), self.name_of(m.prophet));
        let Some(at) = prophetic::prophet_tile(&self.world, m) else { return };
        for v in prophetic::followers(&self.world, &self.villagers) {
            if self.travels.iter().any(|t| t.who == v) { continue; }
            if self.seats.iter().any(|s| s.player == v) { continue; }
            self.send_tile_walk(v, at, Goal::Wander, 40);
        }
        // the day draws near: the closer the deadline, the louder the word
        if !remnant && left > 0 && (left == 14 || left == 7 || left == 1) {
            self.log.push_ev_at_who(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor),
                format!("t{}: {} says the day is {} off - the crowd can barely wait.", tick, name, left),
                Some((at.0 as f32 + 0.5, at.1 as f32 + 0.5)), None);
        }
    }

    /// The disconfirmation: the day comes and nothing comes with it -
    /// the crowd mostly scatters; sometimes the few go deeper instead
    /// (the Munster branch)
    fn prophet_deadline(&mut self, tick: u64) {
        let Some(m) = &self.prophet_movement else { return };
        if m.remnant || tick < m.deadline { return; }
        let name = self.name_of(m.prophet);
        if (self.rng.next() % 1000) as f32 / 1000.0 < 0.6 {
            // the crowd dissolves - the day passed, and the day after is
            // just a day
            for &v in &self.villagers.clone() {
                self.world.remove_component::<Follower>(v);
            }
            self.prophet_movement = None;
            self.log.push_ev(
                EvTag::new(Thread::Org, Valence::Neutral, Sev::Notable),
                format!(
                    "t{}: the named day came and went - the crowd drifts back to its work.",
                    tick
                ),
            );
        } else {
            // the remnant: most leave, the hardened few stay and the
            // vision grows teeth
            let crowd = prophetic::followers(&self.world, &self.villagers);
            let keep: Vec<Entity> = crowd.iter().take(prophetic::REMNANT_KEEP).copied().collect();
            for &v in &crowd {
                if keep.contains(&v) {
                    if let Some(f) = self.world.get_component_mut::<Follower>(v) { f.remnant = true; }
                } else {
                    self.world.remove_component::<Follower>(v);
                }
            }
            if let Some(m) = &mut self.prophet_movement {
                m.remnant = true;
            }
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Negative, Sev::Notable), format!("t{}: the day failed - but {} won't hear it, and the few who stayed grow harder.", tick, name));
        }
    }

    /// The prophet's mortality: cut the head off and the movement
    /// mostly dies - unless the rare roll turns the death into the
    /// recruiting event, and then the grave does the growing
    fn prophet_death(&mut self, tick: u64) {
        let Some(m) = &self.prophet_movement else { return };
        if m.martyred || !villager::is_dead(&self.world, m.prophet) { return; }
        let (at, name, remnant) = (prophetic::prophet_tile(&self.world, m),
            self.name_of(m.prophet), m.remnant);
        if (self.rng.next() % 1000) as f32 / 1000.0 < prophetic::MARTYRDOM_CHANCE {
            // the conversion: they made a martyr of him, and the grave
            // becomes the pulpit - a surge of the half-convinced sign on
            self.martyr_grave = at;
            if let Some(m) = &mut self.prophet_movement { m.martyred = true; }
            let mut joined = 0usize;
            for &v in &self.villagers.clone() {
                if joined >= prophetic::MARTYR_SURGE { break; }
                if villager::is_dead(&self.world, v) { continue; }
                if self.world.get_component::<Follower>(v).is_some() { continue; }
                let adult = self.world.get_component::<crate::game::lifecycle::Age>(v)
                    .map(|a| a.stage == crate::game::lifecycle::LifeStage::Adult
                        || a.stage == crate::game::lifecycle::LifeStage::Elder)
                    .unwrap_or(false);
                if !adult { continue; }
                self.world.add_component(v, Follower { since: tick, remnant });
                joined += 1;
            }
            self.log.push_ev_at_who(EvTag::new(Thread::Org, Valence::Negative, Sev::Major),
                format!("t{}: they made a martyr of {} - and the grave draws a crowd the man never did.", tick, name),
                at.map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5)), None);
        } else {
            // the collapse: no head, no movement - the crowd was always
            // his, and it's no one's now
            for &v in &self.villagers.clone() {
                self.world.remove_component::<Follower>(v);
            }
            self.prophet_movement = None;
            self.log.push_ev(EvTag::new(Thread::Org, Valence::Neutral, Sev::Minor), format!("t{}: with {} gone the fire goes out of them - the crowd's just neighbours again.", tick, name));
        }
    }
}
