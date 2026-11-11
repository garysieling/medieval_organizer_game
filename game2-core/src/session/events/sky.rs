//! The sky and the weather: storms, droughts and floods, frosts and thaws, and the omens overhead.
//! Each fn here holds this family's arms of one handler in events/mod.rs.

use super::*;

impl Game {
    /// sky: the `event_day` arms for this group (the rest fall to the default)
    pub(super) fn sky_day(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::Hailstorm => {
                // the day it falls batters: gardens shredded, everyone stung
                if self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0 { return; }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
                for g in self.gardens.iter_mut() { g.2 *= 0.5; }
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition - 0.02).max(0.05);
                    }
                }
            }
            EventId::HeatWave => {
                // five days of scorch: folk droop in it, the beds ripen
                // fast in the heat, and the well's head drops day by day
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition - 0.01).max(0.05);
                    }
                }
                for g in self.gardens.iter_mut() { g.2 += 0.04; }
                self.land.pit_water = (self.land.pit_water - 0.05).max(0.0);
            }
            EventId::Drought => {
                // a fortnight dry: the well runs low, the beds wilt hard
                // against what the sun would give, and no patch fruits
                // until the dry's past - not tomorrow, the whole dry
                self.land.pit_water = (self.land.pit_water - 0.08).max(0.0);
                for g in self.gardens.iter_mut() { g.2 = (g.2 - 0.6).max(0.0); }
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(self.tick + 1);
                for p in self.mush_patches.iter_mut() {
                    p.next_ready = p.next_ready.max(until);
                }
            }
            EventId::Flood => {
                // the river comes up: the pit fills past what the
                // pump-line can take - a drowned pit yields nothing -
                // and the beds drink deep of the high water
                self.land.pit_water = (self.land.pit_water + 0.2).min(1.0);
                for g in self.gardens.iter_mut() { g.2 += 0.02; }
                // and the high water takes what stands lowest: a
                // riverside building a day, at most three in the flood
                let done = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0);
                if done < 3 {
                    let cands: Vec<Entity> = self.building_entities.iter().enumerate()
                        .filter(|(i, be)| !self.ruined.contains(i) && **be != self.market_food)
                        .filter(|(_, be)| self.world.get_component::<Pos>(**be).map(|p| {
                            let (x, y) = (p.x.round() as i32, p.y.round() as i32);
                            self.reach.iter().any(|&(rx, ry)| (x - rx).abs().max((y - ry).abs()) <= 3)
                        }).unwrap_or(false))
                        .map(|(_, be)| *be).collect();
                    if let Some(&be) = cands.get(((self.rare_roll(id) / 11) as usize) % cands.len().max(1)) {
                        self.ruin_building(be, "gone under the flood");
                        if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux += 1; }
                    }
                }
            }
            EventId::IndianSummer => {
                // five warm days out of season: the beds get one more
                // ripening in, and the gift of the warm stretch sits
                // easy on the folk
                for g in self.gardens.iter_mut() { g.2 += 0.05; }
                // the ease lasts as long as the warm stretch does
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(tick);
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, until);
                }
            }
            EventId::Thaw => {
                // the year's release: the ground works again, the melt
                // refills the well - but a thaw isn't a flood, it stops
                // short of the drowning line - and the folk feel spring
                // a thaw refills, never drains - a pit already past the
                // line is a flood's work, not a thaw's
                if self.land.pit_water < 0.6 {
                    self.land.pit_water = (self.land.pit_water + 0.05).min(0.6);
                }
                for g in self.gardens.iter_mut() { g.2 += 0.03; }
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(tick);
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, until);
                }
            }
            EventId::Aurora => {
                // two nights of sky-fire: folk take it for a sign, and
                // the praying comes easy while the sky burns
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(tick);
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, until);
                }
            }
            EventId::BloodMoon => {
                // two nights of the red omen: folk pray hard through it
                // and sleep the worse for the dread of it
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(tick);
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, until);
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        if h.condition > 0.05 {
                            h.condition = (h.condition - 0.01).max(0.05);
                        }
                    }
                }
            }
            EventId::DarkNoon => {
                // the sun goes out at midday: the prayers go up through
                // the dark, and the beds get no sun for the day
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(tick);
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, until);
                }
                for g in self.gardens.iter_mut() { g.2 = (g.2 - 0.02).max(0.0); }
            }
            EventId::WindStorm => {
                // two days of wind: the beds flatten in it, and the
                // thatch tears loose off one roof a day
                for g in self.gardens.iter_mut() { g.2 *= 0.8; }
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                if let Some(&h) = homes.get((roll as usize / 3) % homes.len().max(1)) {
                    // the tear takes thatch off the roof - a home with
                    // none stocked still loses it: the roof goes thin
                    let took = if let Some(mut b) = self.world.get_component_mut::<household::Building>(h) {
                        household::take_stock(&mut b, "thatch", 1)
                    } else { 0 };
                    if took == 0 {
                        for &v in &self.villagers.clone() {
                            if household::home_of(&self.world, v) != Some(h) { continue; }
                            desires::add_grievance(&mut self.world, v, desires::Problem::ThinWood, self.tick + 30);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// sky: the `event_began` arms for this group (the rest fall to the default)
    pub(super) fn sky_began(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::Hailstorm => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: a hailstorm batters the gardens - stones like shot.",
                        self.tick
                    ),
                );
                self.toasts.push(("Hailstones!".into(), self.tick + 40));
            }
            EventId::HeatWave => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: a heat wave settles in - the stones are hot to the hand.",
                        self.tick
                    ),
                );
                self.toasts.push(("A heat wave".into(), self.tick + 40));
            }
            EventId::Drought => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: a drought - the sky's forgotten rain, and the well knows it.",
                        self.tick
                    ),
                );
                self.toasts.push(("A drought".into(), self.tick + 40));
            }
            EventId::Flood => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: the river comes up - the reach is full and rising.",
                        self.tick
                    ),
                );
                self.toasts.push(("The river's up".into(), self.tick + 40));
            }
            EventId::IndianSummer => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Positive, Sev::Minor),
                    format!(
                        "t{}: an Indian summer - the year's given back a week of warm.",
                        self.tick
                    ),
                );
                self.toasts
                    .push(("An Indian summer".into(), self.tick + 40));
            }
            EventId::Thaw => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Positive, Sev::Minor),
                    format!(
                        "t{}: the thaw - the roofs drip and the ground's soft for the working.",
                        self.tick
                    ),
                );
                self.toasts.push(("The thaw".into(), self.tick + 40));
            }
            EventId::Aurora => {
                self.log.push_ev(EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor), format!("t{}: the aurora - the sky burns red and green, and the folk watch from their doors.", self.tick));
                self.toasts.push(("The aurora".into(), self.tick + 40));
            }
            EventId::BloodMoon => {
                self.log.push_ev(EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor), format!("t{}: a blood moon - the moon hangs red over the village, and the prayers go up.", self.tick));
                self.toasts.push(("A blood moon".into(), self.tick + 40));
            }
            EventId::DarkNoon => {
                self.log.push_ev(EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor), format!("t{}: a dark noon - the sun goes out at midday, and the chickens go to roost.", self.tick));
                self.toasts.push(("Dark noon".into(), self.tick + 40));
            }
            EventId::WindStorm => {
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: a windstorm - the thatch is tearing and the trees bend double.",
                        self.tick
                    ),
                );
                self.toasts.push(("A windstorm".into(), self.tick + 40));
            }
            _ => {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", self.tick, s.name),
                );
            }
        }
    }

    /// What each of them makes of a sky event: the pious read God's
    /// sign in it, the old-faith hear the stones' word, the rest see
    /// the weather - the reading's own worry lands with the heart
    fn sky_omen_read(&mut self, id: EventId) {
        use crate::game::traits::{Personality, Trait};
        for &v in self.villagers.clone().iter() {
            if villager::is_dead(&self.world, v) { continue; }
            let pious = self.world.get_component::<Personality>(v)
                .map(|p| p.traits.contains(&Trait::Pious)).unwrap_or(false);
            let old = crate::session::routine::keeps_old_ways(&self.world, v);
            let line = if pious { "A sign from God - the sky speaks of it." }
                else if old { "The stones knew - the old way reads this." }
                else { "Strange weather this season - it'll pass." };
            // the reading's worry lands light: the pious' sign firms
            // their faith a touch - no bond beat (an episode against no
            // bond turned the sim's long runs)
            if pious {
                if let Some(f) = self.world.get_component_mut::<crate::sim::religion::Faith>(v) {
                    f.devotion = (f.devotion + 0.02).min(1.0);
                }
            }
            if (self.rare_roll(id) as usize + v as usize) % 9 == 0 {
                self.say(v, line, CHAT_BUBBLE_SECS);
            }
        }
    }

    /// sky: the `event_moment` arms for this group (the rest fall to the default)
    pub(super) fn sky_moment(&mut self, id: EventId) {
        let s = spec(id);
        self.sky_omen_read(id);
        match id {
            EventId::SeaMonster => {
                // the deep one's sighted off the strand - no bite, no
                // wreck, just the shape in the water and the season's
                // talk after. What each of them sees in it reads their
                // own way: the pious fear it, the rest wonder
                use crate::sim::psychology::{record_episode, Episode, EpisodeKind};
                use crate::game::traits::{Personality, Trait};
                for &v in self.villagers.clone().iter() {
                    if villager::is_dead(&self.world, v) { continue; }
                    let pious = self.world.get_component::<Personality>(v)
                        .map(|t| t.traits.contains(&Trait::Pious)).unwrap_or(false);
                    let kind = if pious { EpisodeKind::Anxiety } else { EpisodeKind::Warmth };
                    record_episode(&mut self.world, v, Episode { with: 0, kind });
                    if (self.rare_roll(id) as usize + v as usize) % 7 == 0 {
                        self.say(v, if pious { "The deep one stirs - the water keeps its own."
                            } else { "A great thing in the water - seen it with my own eyes." },
                            CHAT_BUBBLE_SECS);
                    }
                }
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Beast, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: a shape in the water off the strand - the deep one's been seen.",
                        self.tick
                    ),
                    Some((self.village.0 as f32 + 0.5, self.village.1 as f32 + 0.5)),
                    None,
                );
                self.toasts.push(("The deep one!".into(), self.tick + 40));
            }
            EventId::LightningStrike => {
                // the bolt picks one of them - knocked flat where it
                // lands, not killed; the sky aims where it aims
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                if let Some(&v) = able.get((self.rare_roll(id) as usize / 7) % able.len().max(1)) {
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        // a strike only takes - the floor never heals
                        if h.condition > 0.05 {
                            h.condition = (h.condition - 0.25).max(0.05);
                        }
                    }
                    self.say(v, "...", CHAT_BUBBLE_SECS);
                    self.log_entity_ev(EvTag::new(Thread::Weather, Valence::Negative, Sev::Notable), v, format!("t{}: lightning strikes the village - one of them's knocked flat where it lands.", self.tick));
                } else {
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: lightning strikes the common - white fire in the grass.",
                            self.tick
                        ),
                        Some((self.village.0 as f32 + 0.5, self.village.1 as f32 + 0.5)),
                        None,
                    );
                }
                self.toasts.push(("Lightning!".into(), self.tick + 40));
                // and now and then the bolt itself is the spark: a knot
                // of dry grass catches and the fire has leave to run
                if self.rare_roll(id) % 5 < 2 { self.strike_fire(id); }
            }
            EventId::LateFrost => {
                // a killing frost out of season: the gardens are nipped
                // in a night, and the patches die back a full month
                for g in self.gardens.iter_mut() { g.2 *= 0.6; }
                for p in self.mush_patches.iter_mut() {
                    p.next_ready = p.next_ready.max(self.tick + 30);
                }
                // a forager mid-walk turns home - the patch his errand
                // was for is black under the frost
                self.travels
                    .retain(|t| !matches!(t.goal, Goal::ForageShroom(_)));
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Negative, Sev::Minor),
                    format!(
                        "t{}: a late frost - white on the beds in the morning, the shoots black.",
                        self.tick
                    ),
                );
                self.toasts.push(("A late frost".into(), self.tick + 40));
            }
            EventId::FirstSnow => {
                // the year's turning marked: the kids' wonder at the
                // first flakes, and the melt feeds the well a little
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    if self.world.get_component::<crate::game::vocation::Kid>(v).is_none() { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 15);
                }
                self.log.push_ev(
                    EvTag::new(Thread::Weather, Valence::Positive, Sev::Minor),
                    format!(
                        "t{}: the first snow - white on the roofs, the kids out in it early.",
                        self.tick
                    ),
                );
                self.toasts.push(("The first snow".into(), self.tick + 40));
            }
            EventId::MeteorShower => {
                // a night of falling stars: the whole village stops and
                // watches the sky, and the wonder of it sits easy after
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 15);
                }
                self.log.push_ev(EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor), format!("t{}: a meteor shower - the sky rains stars, and nobody's eyes are anywhere else.", self.tick));
                self.toasts.push(("Falling stars!".into(), self.tick + 40));
            }
            EventId::Mirage => {
                // the false city on the horizon: a marvel the whole
                // village sees and talks of, though nobody walks to it
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 15);
                }
                self.log.push_ev(EvTag::new(Thread::Weather, Valence::Neutral, Sev::Minor), format!("t{}: a mirage - a city of towers hangs on the horizon, and melts when you look twice.", self.tick));
                self.toasts.push(("A mirage".into(), self.tick + 40));
            }
            _ => {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", self.tick, s.name),
                );
            }
        }
    }

    /// The bolt's other work: a knot of flammable ground by the market
    /// catches together, so the fire has leave to run - the same rules
    /// a chimney fire answers to
    fn strike_fire(&mut self, id: EventId) {
        let roll = self.rare_roll(id);
        let (mx, my) = self.world.get_component::<Pos>(self.market_food)
            .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or(self.village);
        // flammable ground within a short walk of the market, gathered
        // in scan order so the pick stays deterministic
        let mut cells: Vec<(i32, i32)> = vec![];
        for y in (my - 10)..=(my + 10) {
            for x in (mx - 10)..=(mx + 10) {
                if (x - mx).abs() + (y - my).abs() > 10 { continue; }
                if self.fire.burning.contains_key(&(x, y)) || self.fire.scorched.contains_key(&(x, y)) { continue; }
                if crate::game::fire::Fire::flammable(&self.map, x, y) { cells.push((x, y)); }
            }
        }
        let Some(&c) = cells.get(((roll / 5) as usize) % cells.len().max(1)) else { return };
        let mut lit = 0;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            if self.fire.ignite(&self.map, c.0 + dx, c.1 + dy, self.tick) { lit += 1; }
        }
        if lit > 0 {
            self.log.push_ev_at_who(
                EvTag::new(Thread::Fire, Valence::Negative, Sev::Minor),
                format!(
                    "t{}: the bolt sets the grass alight - white fire in the dry grass.",
                    self.tick
                ),
                Some((c.0 as f32 + 0.5, c.1 as f32 + 0.5)),
                None,
            );
            self.toasts.push_at(
                ("Fire!".into(), self.tick + 40),
                Some((c.0 as f32 + 0.5, c.1 as f32 + 0.5)),
            );
        }
    }
}
