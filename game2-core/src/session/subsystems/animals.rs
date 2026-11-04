//! Animals: the fold and the yard hens, the nights the wolves and the fox come, the apiary, and the ambient sound cues.

use crate::session::*;

impl Game {
    /// The fold's day: the grazing point drifts, shearing comes round
    /// each season, and now and then a wolf comes in the night
    pub fn livestock_tick(&mut self, tick: u64) {
        // hens lay in their yards; a hungry one at home takes an egg
        for &h in &self.hen_houses.clone() {
            let (eggs, occupants) = self.world.get_component::<Building>(h)
                .map(|b| (b.stock.get("eggs").copied().unwrap_or(0), b.occupants.clone()))
                .unwrap_or((0, vec![]));
            if eggs < 6 {
                if let Some(b) = self.world.get_component_mut::<Building>(h) { household::add_stock(b, "eggs", 1); }
            }
            if let Some(occ) = occupants.into_iter()
                .find(|&o| self.world.get_component::<Hunger>(o).map(|hu| hu.value < 40.0).unwrap_or(false)) {
                if let Some(b) = self.world.get_component_mut::<Building>(h) {
                    if household::take_stock(b, "eggs", 1) > 0 {
                        if let Some(hu) = self.world.get_component_mut::<Hunger>(occ) { hu.value = (hu.value + 25.0).min(100.0); }
                        self.say(occ, "eggs for breakfast", CHAT_BUBBLE_SECS);
                    }
                }
            }
        }
        // a fox in the yard, now and then - the hens give its coming away
        if tick % 90 == 65 && !self.hens.is_empty() && self.rng.below(100) < 8 {
            self.fox_coming = true;
            self.log.push(format!("t{}: the hens won't settle at dusk - something prowls the hedges.", tick));
        }
        if tick % 90 == 75 && self.fox_coming { self.fox_coming = false; self.fox_night(tick); }
        let Some(fold) = self.fold else { return };
        // the byre gives back what the flock drops: a week of leavings,
        // dried, burns as good as a faggot when the wood runs thin
        if tick % 7 == 1 {
            if let Some(b) = self.world.get_component_mut::<Building>(fold) {
                household::add_stock(b, "dung", 1);
            }
        }
        // move the grazing point every few days; a sheepdog keeps it tight
        if tick >= self.next_graze {
            if let Some(fp) = self.world.get_component::<Pos>(fold).copied() {
                let r = self.shepherd.filter(|&sh| self.pets.iter().any(|(e, s, _)| *s == Species::Dog
                    && self.world.get_component::<Pet>(*e).map(|p| p.owner == Some(sh)).unwrap_or(false)))
                    .map(|_| 4).unwrap_or(7);
                // the flock grazes around where it actually is, but the
                // pasture's still the pasture: a wandered middle is
                // tethered to the fold so days of offsets can't drift
                // the whole flock across the map
                let (cx, cy) = livestock::centroid(&self.world, fold)
                    .map(|(cx, cy)| (fp.x + (cx - fp.x).clamp(-10.0, 10.0),
                                     fp.y + (cy - fp.y).clamp(-10.0, 10.0)))
                    .unwrap_or((fp.x, fp.y));
                self.graze_at = ((cx + self.rng.around(r) as f32).clamp(1.0, self.map.width as f32 - 2.0),
                                 (cy + self.rng.around(r) as f32).clamp(1.0, self.map.height as f32 - 2.0));
            }
            self.next_graze = tick + 2 + self.rng.below(3) as u64;
        }
        // shearing day, mid-season: wool into the fold's stock
        if tick % 90 == 45 {
            let (sheep, _) = livestock::counts(&self.world, fold);
            if sheep > 0 {
                if let Some(b) = self.world.get_component_mut::<Building>(fold) {
                    let got = household::add_stock(b, "wool", (sheep * 2) as u32);
                    if got > 0 { self.log.push(format!("t{}: shearing day at the fold - {} wool.", tick, got)); }
                }
            }
        }
        // a dark shape crosses the meadow in the night, now and then -
        // the mud tells on it a few days out
        if tick % 90 == 50 && self.rng.below(100) < 10 {
            self.wolf_coming = true;
            self.log.push(format!("t{}: wolf tracks in the mud by the fold - the dogs won't settle.", tick));
        }
        if tick % 90 == 60 && self.wolf_coming { self.wolf_coming = false; self.wolf_night(tick); }
        // the shepherd walks out to the flock most days, but not hungry
        if let Some(sh) = self.shepherd {
            let fed = self.world.get_component::<Hunger>(sh).map(|h| h.value >= 30.0).unwrap_or(true);
            if tick % 3 == 0 && fed && !villager::is_dead(&self.world, sh)
                && !self.travels.iter().any(|t| t.who == sh) {
                self.send(sh, Goal::Herd(fold));
            }
        }
    }

    /// A dark shape crosses the meadow: the flock scatters, some are lost
    pub(in crate::session) fn wolf_night(&mut self, tick: u64) {
        let Some(fold) = self.fold else { return };
        let mut animals = livestock::flock(&self.world, fold);
        let losses = (self.rng.below(3) as usize).min(animals.len());
        for &a in animals.iter().take(losses) {
            self.world.destroy_entity(a);
            self.flock.retain(|&e| e != a);
        }
        animals.drain(..losses.min(animals.len()));
        for a in animals {
            if let Some(p) = self.world.get_component_mut::<Pos>(a) {
                p.x = (p.x + self.rng.around(6) as f32).clamp(1.0, self.map.width as f32 - 2.0);
                p.y = (p.y + self.rng.around(6) as f32).clamp(1.0, self.map.height as f32 - 2.0);
            }
        }
        self.log.push(format!("t{}: a wolf got into the fold in the night - {} lost.", tick, losses));
        if let Some(sh) = self.shepherd {
            desires::add_grievance(&mut self.world, sh, desires::Problem::Loss, tick + 90);
            self.say(sh, "The wolf came in the night.", CHAT_BUBBLE_SECS);
        }
    }

    /// A fox in the yard: a hen gone in the night, and the household grieves it
    pub(in crate::session) fn fox_night(&mut self, tick: u64) {
        let hens_at = |h| livestock::flock(&self.world, h).iter()
            .any(|&a| self.world.get_component::<Livestock>(a).map(|l| l.kind == LivestockKind::Hen).unwrap_or(false));
        let Some(&h) = self.hen_houses.iter().find(|&&h| hens_at(h)) else { return };
        let Some(&a) = livestock::flock(&self.world, h).first() else { return };
        self.world.destroy_entity(a);
        self.hens.retain(|&e| e != a);
        let name = self.world.get_component::<Building>(h).map(|b| b.name.clone()).unwrap_or_else(|| "the yard".into());
        self.log.push(format!("t{}: a fox got into {} - a hen gone in the night.", tick, name));
        // the house grieves her through whoever can feel it - the
        // held and the dead take no grief
        if let Some(&occ) = self.world.get_component::<Building>(h)
            .and_then(|b| b.occupants.iter().find(|&&o|
                !crate::game::dungeon::unavailable(&self.world, o))) {
            desires::add_grievance(&mut self.world, occ, desires::Problem::Loss, tick + 90);
            self.say(occ, "The fox took her. Best hen I had.", CHAT_BUBBLE_SECS);
        }
    }

    /// The apiary's year: honey and wax at midsummer, candles down at the
    /// church, and now and then a swarm takes a liking to someone
    pub fn apiary_tick(&mut self, tick: u64) {
        let Some(ay) = self.apiary else { return };
        // the yield, in season: honey for the table, wax for the seals
        if tick % 90 == 50 {
            let n = apiary::hives_of(&self.world, ay).len() as u32;
            if let Some(b) = self.world.get_component_mut::<Building>(ay) {
                household::add_stock(b, "honey", n * 2);
                household::add_stock(b, "wax", n);
            }
            if n > 0 { self.log.push(format!("t{}: the skeps yield - {} honey, {} wax.", tick, n * 2, n)); }
            // a season's candles for the church, from the season's wax
            if let (Some(church), Some(b)) = (self.church, self.world.get_component_mut::<Building>(ay)) {
                let wax = household::take_stock(b, "wax", n);
                if let Some(cb) = self.world.get_component_mut::<Building>(church) {
                    household::add_stock(cb, "candle", wax);
                }
            }
        }
        // the church burns a candle a season; when it runs out it goes dark
        if tick % 90 == 20 {
            if let Some(church) = self.church {
                let burnt = self.world.get_component_mut::<Building>(church)
                    .map(|b| household::take_stock(b, "candle", 1)).unwrap_or(0);
                if burnt == 0 {
                    if let Some(pr) = self.villagers.iter().copied().find(|&v|
                        self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
                        && !crate::game::dungeon::unavailable(&self.world, v)) {
                        desires::add_grievance(&mut self.world, pr, desires::Problem::NoInput("wax"), tick + 90);
                        self.say(pr, "The candles are done - the church is dark.", CHAT_BUBBLE_SECS);
                    }
                }
            }
        }
        // a swarm takes a liking to someone - harmless, memorable; a
        // swarm does not favor the grave, and the cell is out of reach
        if tick % 90 == 70 && self.rng.below(100) < 8 {
            let able: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).collect();
            let Some(who) = able.get((self.rng.below(able.len() as i32).max(0)) as usize).copied()
            else { return };
            self.swarm = Some((who, tick + 1));
            let name = self.villagers.iter().position(|x| *x == who).map(|x| vill_name(x)).unwrap_or("someone");
            self.log.push(format!("t{}: a swarm took a liking to {} - it follows them about.", tick, name));
        }
        if let Some((_, until)) = self.swarm { if tick >= until { self.swarm = None; } }
        // the beekeeper walks out to the skeps most days, but not hungry
        if let Some(bk) = self.beekeeper {
            let fed = self.world.get_component::<Hunger>(bk).map(|h| h.value >= 30.0).unwrap_or(true);
            if tick % 3 == 1 && fed && !villager::is_dead(&self.world, bk)
                && !self.travels.iter().any(|t| t.who == bk) {
                self.send(bk, Goal::Herd(ay));
            }
        }
    }

    /// The soundscape's cues: the ambient bed (rain falling, the smithy
    /// ringing on a workday) and the moments worth a sound of their own
    pub fn sfx_tick(&mut self, tick: u64) {
        // the ambient bed: rain when it falls, the smithy's rhythm on a
        // workday - quiet state cues the audio layer keeps under it all
        if self.precip.is_some() && tick % 20 == 0 {
            self.emit_sfx("rain", 20.0, 15.0, 0.4);
        }
        if tick % 40 == 10 {
            self.emit_sfx("anvil", 19.0, 19.0, 0.3); // the smithy's rhythm
        }
        // the moments: a festival hums, a comet hushes, the hanging's
        // grim - the big things get a cue of their own where they happen
        if tick % 90 == 45 {
            self.emit_sfx("festival", 16.0, 16.0, 0.9);
        }
        if let Some((_, until)) = self.sky {
            if until == tick + 2 { self.emit_sfx("wonder", 20.0, 5.0, 0.7); }
        }
    }
}
