//! The crisis cascade: famine feeds fever, fever fills the graveyard,
//! and sustained misery fills the grumbling - one hard stretch becoming
//! the next through hooks the systems already have. There's no crisis
//! meter: the census reads the same stats the sim keeps, writes the
//! dominos that fall into the village's tale, and story.rs names the
//! sequence when it sees it come apart in order.

use crate::session::*;

/// Days of one misery before it becomes something to organize around -
/// a hard week is weather; a hard fortnight is a grievance
const HARD_DAYS: u32 = 5;
/// The census's weekly beat shares land_tick's calendar
const CENSUS_DAY: u64 = 2;
/// Census cadence in days (tick % 7 == CENSUS_DAY)
const WEEK: u64 = 7;
/// Dominos older than this have settled into the year's telling, not
/// this stretch's
const TALE_DAYS: u64 = 150;
/// A named calamity rests this long before the village reaches for a
/// name again - next year's hunger is its own tale
const NAMED_COOLDOWN: u64 = 90;

impl Game {
    /// Famine as the village feels it: two in five living bellies hollow.
    /// The country's hunger isn't the village's water problem
    pub(in crate::session) fn village_famine(&self) -> bool {
        let (mut folk, mut hungry) = (0usize, 0usize);
        for &v in &self.villagers {
            if villager::is_dead(&self.world, v) { continue; }
            if self.world.get_component::<country::Outlander>(v).is_some() { continue; }
            folk += 1;
            if self.world.get_component::<Hunger>(v).map(|h| h.value < 40.0).unwrap_or(false) {
                hungry += 1;
            }
        }
        folk > 0 && hungry * 5 >= folk * 2
    }

    /// The day's crisis work: per-villager hardship streaks ripen into
    /// grievances, and the weekly census writes the dominos into the
    /// village's tale
    pub fn cascade_tick(&mut self, tick: u64) {
        self.misery_tick(tick);
        if tick % WEEK != CENSUS_DAY { return; }
        self.cascade_census(tick);
        self.cascade_name(tick);
    }

    /// Consecutive days of hunger or sickness, counted on the villager:
    /// past HARD_DAYS the discomfort becomes a grievance the sanction
    /// ladder and the campaigns can spend; the day it's met again the
    /// grievance lifts with the streak
    fn misery_tick(&mut self, tick: u64) {
        for &v in self.villagers.clone().iter() {
            if villager::is_dead(&self.world, v) { continue; }
            // the country's hard months are their own story
            if self.world.get_component::<country::Outlander>(v).is_some() { continue; }
            let hungry = self.world.get_component::<Hunger>(v)
                .map(|h| h.value < 40.0).unwrap_or(false);
            let sick = is_sick(&self.world, v);
            if self.world.get_component::<desires::Hardship>(v).is_none() {
                self.world.add_component(v, desires::Hardship::default());
            }
            let (days_hungry, days_sick) = {
                let Some(hs) = self.world.get_component_mut::<desires::Hardship>(v) else { continue };
                hs.hungry = if hungry { hs.hungry + 1 } else { 0 };
                hs.sick = if sick { hs.sick + 1 } else { 0 };
                (hs.hungry, hs.sick)
            };
            if hungry && days_hungry >= HARD_DAYS {
                desires::add_grievance(&mut self.world, v, desires::Problem::Hungry, tick + 30);
            } else if !hungry {
                desires::lift_grievance(&mut self.world, v, desires::Problem::Hungry);
            }
            if sick && days_sick >= HARD_DAYS {
                desires::add_grievance(&mut self.world, v, desires::Problem::Sick, tick + 30);
            } else if !sick {
                desires::lift_grievance(&mut self.world, v, desires::Problem::Sick);
            }
        }
    }

    /// The weekly census: which dominos stand this week - the want, the
    /// fever, the mounting dead, the curdling temper. Only crossings get
    /// written: a level held all month is one domino, not thirty
    fn cascade_census(&mut self, tick: u64) {
        let folk: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && self.world.get_component::<country::Outlander>(v).is_none())
            .collect();
        if folk.is_empty() { return; }
        let hardship = self.village_famine();
        let symptomatic = folk.iter().filter(|&&v| is_sick(&self.world, v)).count();
        let illness = symptomatic >= 3;
        // the dead of the last week, village dead only - the country's
        // losses are told at their own firesides
        let deaths = self.died_on.iter().enumerate()
            .filter(|(_, d)| d.map(|d| tick.saturating_sub(d) <= WEEK).unwrap_or(false))
            .filter(|(i, _)| self.villagers.get(*i)
                .map(|&v| self.world.get_component::<country::Outlander>(v).is_none())
                .unwrap_or(false))
            .count() >= 2;
        let grieved = folk.iter().filter(|&&v|
            self.world.get_component::<desires::InnerLife>(v)
                .map(|l| l.grievances.iter().any(|(_, u)| tick < *u)).unwrap_or(false)).count();
        let unrest = grieved >= 8;
        let levels = (hardship as u8) | ((illness as u8) << 1)
            | ((deaths as u8) << 2) | ((unrest as u8) << 3);
        let (risen, fallen) = (levels & !self.cascade_levels, self.cascade_levels & !levels);
        self.cascade_levels = levels;
        // the dominos, each with its beat in the log
        let mut beats: Vec<(crate::sim::story::EventKind, String)> = vec![];
        if risen & 1 != 0 {
            beats.push((crate::sim::story::EventKind::Hardship,
                "the stores are bare - folk count crumbs like coin".into()));
        }
        if risen & 2 != 0 {
            beats.push((crate::sim::story::EventKind::Illness,
                format!("the sickness walks the lanes - {} in the beds", symptomatic)));
        }
        if fallen & 2 != 0 {
            beats.push((crate::sim::story::EventKind::Recovery,
                "the sickness breaks - the lanes breathe again".into()));
        }
        if risen & 4 != 0 {
            beats.push((crate::sim::story::EventKind::Death,
                "the dead mount - the village counts its losses".into()));
        }
        if risen & 8 != 0 {
            beats.push((crate::sim::story::EventKind::Unrest,
                format!("the grumbling curdles - {} carry a grievance", grieved)));
        }
        for (kind, beat) in beats {
            self.cascade_tale.push(crate::sim::story::StoryEvent { tick, kind, description: beat.clone() });
            self.log
                .push_ev(EvTag::for_kind(kind), format!("t{}: {}.", tick, beat));
        }
        // the tale is a working window: forget the settled dominos, cap the rest
        self.cascade_tale.retain(|e| tick.saturating_sub(e.tick) <= TALE_DAYS);
        if self.cascade_tale.len() > 80 { self.cascade_tale.drain(..self.cascade_tale.len() - 80); }
    }

    /// When the dominos have fallen in order often enough for the tale
    /// to read as one calamity, the village reaches for a name - once per
    /// stretch, and only when fresh dominos have fallen since the last
    //  naming
    fn cascade_name(&mut self, tick: u64) {
        if let Some(at) = self.cascade_named_at {
            if tick.saturating_sub(at) < NAMED_COOLDOWN { return; }
            // a name is earned by fresh dominos, not the same old tale
            if !self.cascade_tale.iter().any(|e| e.tick > at) { return; }
        }
        let Some(st) = crate::sim::story::detect_cascade(&self.cascade_tale) else { return };
        self.cascade_named_at = Some(tick);
        self.log.push_ev(
            EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Notable),
            format!(
                "t{}: they will speak of this stretch as {}.",
                tick, st.title
            ),
        );
    }
}
