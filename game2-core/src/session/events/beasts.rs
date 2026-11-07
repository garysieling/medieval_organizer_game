//! The beasts: wolves and foxes, strays, plagues of rats and locusts, and the wild things in the fields.
//! Each fn here holds this family's arms of one handler in events/mod.rs.

use super::*;

impl Game {
    /// beasts: the `event_day` arms for this group (the rest fall to the default)
    pub(super) fn beasts_day(&mut self, id: EventId, _tick: u64) {
        match id {
            EventId::RatPlague => {
                // the rats' own ten days: every larder pays them one
                // day's toll where no cat walks - the tally goes in aux
                for &be in &self.building_entities.clone() {
                    if self.cat_watches(be) { continue; }
                    let roll = self.rare_roll(id);
                    let good = self.world.get_component::<household::Building>(be)
                        .and_then(|b| {
                            let mut stocked: Vec<&String> = b.stock.iter()
                                .filter(|(_g, q)| **q > 0).map(|(g, _)| g).collect();
                            stocked.sort();
                            // the rats go for the grain first, else
                            // whatever shelf the roll lands on
                            stocked.iter().find(|g| matches!(g.as_str(),
                                "wheat"|"flour"|"bread"|"barley"|"oats"|"grain"))
                                .or_else(|| stocked.get((roll as usize / 5) % stocked.len().max(1)))
                                .map(|g| (*g).clone())
                        });
                    let Some(g) = good else { continue };
                    let took = if let Some(mut b) = self.world.get_component_mut::<household::Building>(be) {
                        household::take_stock(&mut b, &g, 1)
                    } else { 0 };
                    if took > 0 {
                        if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux += 1; }
                    }
                }
            }
            EventId::CaterpillarYear => {
                // ten days of the slow creep: the beds get eaten a
                // little a day - not the locusts' hour, but it adds up
                for g in self.gardens.iter_mut() { g.2 = (g.2 - 0.15).max(0.0); }
            }
            EventId::WildBoar => {
                // five days of the sow rooting about: she eats the
                // beds a little a day, and she'll charge the fool who
                // goes near her - the roll says who draws her eye
                for g in self.gardens.iter_mut() { g.2 = (g.2 - 0.1).max(0.0); }
                let roll = self.rare_roll(id);
                if roll % 3 == 0 {
                    let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                        !villager::is_dead(&self.world, v)
                            && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                    if let Some(&v) = able.get((roll as usize / 7) % able.len().max(1)) {
                        if let Some(h) = self.world.get_component_mut::<Health>(v) {
                            if h.condition > 0.05 {
                                h.condition = (h.condition - 0.1).max(0.05);
                            }
                        }
                        self.say(v, "The sow got me - stay clear of her!", CHAT_BUBBLE_SECS);
                    }
                }
            }
            EventId::FishRun => {
                // six days the river gives: the day's catch goes on
                // the market's baskets, where folk actually eat
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings += 2;
                    // the catch rides in aux - a hungry village eats it
                    // the same day, so the tally's the honest signal
                    if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux += 2; }
                }
            }
            _ => {}
        }
    }

    /// beasts: the `event_began` arms for this group (the rest fall to the default)
    pub(super) fn beasts_began(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::RatPlague => {
                self.log.push(format!("t{}: a plague of rats - they're in the thatch, the larders, everywhere.", self.tick));
                self.toasts.push(("A plague of rats".into(), self.tick + 40));
            }
            EventId::WildBoar => {
                self.log.push(format!("t{}: a wild boar's rooting about the beds - keep clear of her, she's no fool's game.", self.tick));
                self.toasts.push(("A wild boar".into(), self.tick + 40));
            }
            EventId::OwlInTower => {
                self.log.push(format!("t{}: an owl's moved into the tower - the mice had best learn quiet ways.", self.tick));
                self.toasts.push(("An owl in the tower".into(), self.tick + 40));
            }
            EventId::FishRun => {
                self.log.push(format!("t{}: a fish run - the reach is alive with them, thick enough to grab with two hands.", self.tick));
                self.toasts.push(("A fish run".into(), self.tick + 40));
            }
            EventId::CaterpillarYear => {
                self.log.push(format!("t{}: a caterpillar year - the leaves are lacework, and the kids are out picking them off.", self.tick));
                self.toasts.push(("A caterpillar year".into(), self.tick + 40));
                // the kids have sport of it while the grown-ups fret
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    if self.world.get_component::<crate::game::vocation::Kid>(v).is_none() { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 15);
                }
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// beasts: the `event_moment` arms for this group (the rest fall to the default)
    pub(super) fn beasts_moment(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::WolfAtFold => {
                // the wolf's in the fold now - the flock scatters and
                // some are lost, the same way the night-run does it.
                // run it on a spare stream so the event's rolls don't
                // shift the village's own dice
                let keep = Rng(self.rng.0);
                self.wolf_night(self.tick);
                self.rng = keep;
                self.toasts.push(("Wolf at the fold!".into(), self.tick + 40));
            }
            EventId::FoxInYard => {
                if self.hens.is_empty() {
                    // the fox is seen all the same - nothing for it here
                    self.log.push(format!("t{}: a fox slinks through the yard in the night - nothing for it here.", self.tick));
                } else {
                    // a hen gone and the house grieves her - the same
                    // way the fox night does it, on a spare stream so
                    // the event's rolls don't shift the village's dice
                    let keep = Rng(self.rng.0);
                    self.fox_night(self.tick);
                    self.rng = keep;
                }
                self.toasts.push(("A fox in the yard!".into(), self.tick + 40));
            }
            EventId::HawkStoops => {
                // a hen gone off the green in daylight - the same take
                // the fox night does, but the folk see the shadow fall
                let hens_at = |w: &crate::engine::ecs::World, h|
                    crate::game::livestock::flock(w, h).iter().any(|&a|
                        w.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.kind == crate::game::livestock::LivestockKind::Hen).unwrap_or(false));
                if self.hen_houses.iter().any(|&h| hens_at(&self.world, h)) {
                    let h = *self.hen_houses.iter().find(|&&h| hens_at(&self.world, h)).unwrap();
                    if let Some(&a) = crate::game::livestock::flock(&self.world, h).first() {
                        self.world.destroy_entity(a);
                        self.hens.retain(|&e| e != a);
                        self.log.push_at_who(format!("t{}: a hawk stoops - a shadow falls on the green, and a hen's gone with it.", self.tick),
                            toast_spot(&self.world, h), None);
                        // the folk see the shadow fall - the house grieves
                        // her the same as the night's take; the dead
                        // and the held see nothing
                        if let Some(&occ) = self.world.get_component::<household::Building>(h)
                            .and_then(|b| b.occupants.iter().find(|&&o|
                                !crate::game::dungeon::unavailable(&self.world, o))) {
                            desires::add_grievance(&mut self.world, occ, desires::Problem::Loss, self.tick + 90);
                            self.say(occ, "The hawk took her, clean off the green.", CHAT_BUBBLE_SECS);
                        }
                    }
                } else {
                    self.log.push_at_who(format!("t{}: a hawk stoops on the green - and finds the yard empty.", self.tick),
                        toast_spot(&self.world, self.market_food), None);
                }
                self.toasts.push(("A hawk stoops!".into(), self.tick + 40));
            }
            EventId::Locusts => {
                // the cloud comes down once and strips the beds near
                // bare - the year's work eaten in an hour
                for g in self.gardens.iter_mut() { g.2 *= 0.15; }
                for p in self.mush_patches.iter_mut() {
                    p.next_ready = p.next_ready.max(self.tick + 60);
                }
                self.travels.retain(|t| !matches!(t.goal, Goal::ForageShroom(_)));
                self.log.push(format!("t{}: locusts - the air's alive with them, and where they pass there's nothing green.", self.tick));
                self.toasts.push(("Locusts!".into(), self.tick + 40));
            }
            EventId::Starlings => {
                // the sky writes itself: the whole village stops to
                // watch the shape of it, and the wonder sits easy after
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 15);
                }
                self.log.push(format!("t{}: a murmuration - ten thousand wings writing on the sky, and nobody's eyes are on their work.", self.tick));
                self.toasts.push(("A murmuration".into(), self.tick + 40));
            }
            EventId::BeeTree => {
                // honey for the taking: a rolled home's larder gains
                // the combs, and the finder pays the stings for them
                // an empty house keeps no bees: only homes with someone
                // living and about can bring the combs back
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home
                            && bb.occupants.iter().any(|&o|
                                !villager::is_dead(&self.world, o)
                                    && !crate::game::dungeon::unavailable(&self.world, o)))
                        .unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                if let Some(&h) = homes.get((roll as usize / 7) % homes.len().max(1)) {
                    if let Some(mut b) = self.world.get_component_mut::<household::Building>(h) {
                        household::add_stock(&mut b, "honey", 3);
                    }
                    // the combs come dear: the finder pays the stings
                    let occ = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)));
                    if let Some(o) = occ {
                        if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                            if hl.condition > 0.05 {
                                hl.condition = (hl.condition - 0.03).max(0.05);
                            }
                        }
                        self.say(o, "Worth every sting!", CHAT_BUBBLE_SECS);
                    }
                    self.log.push_at_who(format!("t{}: a bee tree found - the combs come back dripping, stings and all.", self.tick),
                        toast_spot(&self.world, h),
                        occ.and_then(|o| self.villagers.iter().position(|&v| v == o)));
                } else {
                    self.log.push(format!("t{}: a bee tree's humming in the wood - nobody ventures the combs.", self.tick));
                }
                self.toasts.push(("A bee tree".into(), self.tick + 40));
            }
            EventId::DeerInFields => {
                // a doe at dawn: she grazes one bed down to the stalks,
                // and the folk stand at the hedge to watch her all the
                // same - the lord's deer, best left alone
                let roll = self.rare_roll(id);
                let deer_at = if !self.gardens.is_empty() {
                    let gi = (roll as usize / 5) % self.gardens.len();
                    self.gardens[gi].2 *= 0.5;
                    Some((self.gardens[gi].1.0 as f32 + 0.5, self.gardens[gi].1.1 as f32 + 0.5))
                } else { None };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 10);
                }
                self.log.push_at_who(format!("t{}: a deer in the fields - a doe at dawn, grazing the beds slow, and the folk watch from the hedge.", self.tick),
                    deer_at, None);
                self.toasts.push(("A deer in the fields".into(), self.tick + 40));
            }
            EventId::CrowsMob => {
                // a mob drops on the larder where there's grain to be
                // had - the folk throw stones and the crows laugh
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home
                            && bb.stock.values().any(|&q| q > 0)).unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                if let Some(&h) = homes.get((roll as usize / 3) % homes.len().max(1)) {
                    let mut sorted: Vec<String> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.stock.iter().filter(|(_g, q)| **q > 0)
                            .map(|(g, _)| g.clone()).collect()).unwrap_or_default();
                    sorted.sort();
                    if let Some(g) = sorted.first() {
                        if let Some(mut b) = self.world.get_component_mut::<household::Building>(h) {
                            household::take_stock(&mut b, &g.clone(), 2);
                        }
                    }
                    self.log.push_at_who(format!("t{}: a mob of crows - they drop on the village in a black storm, and rise laughing.", self.tick),
                        toast_spot(&self.world, h), None);
                } else {
                    // nothing stocked: they strip a bed instead
                    let beds_at = if !self.gardens.is_empty() {
                        let gi = (roll as usize / 3) % self.gardens.len();
                        self.gardens[gi].2 *= 0.6;
                        Some((self.gardens[gi].1.0 as f32 + 0.5, self.gardens[gi].1.1 as f32 + 0.5))
                    } else { None };
                    self.log.push_at_who(format!("t{}: a mob of crows - they drop on the beds and strip what they find.", self.tick),
                        beds_at, None);
                }
                self.toasts.push(("A mob of crows".into(), self.tick + 40));
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// beasts: the `event_end` arms for this group (the rest fall to the default)
    pub(super) fn beasts_end(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::StrayDog => {
                // the loneliest heart takes him in - the dog decides;
                // a heart behind bars is no heart to him
                let mut bond_n: crate::engine::det::HashMap<Entity, usize> = crate::engine::det::HashMap::default();
                self.world.iter::<crate::game::relationships::Relationship>(|_, r| {
                    *bond_n.entry(r.from).or_default() += 1;
                    if r.other != r.from { *bond_n.entry(r.other).or_default() += 1; }
                });
                let mark = self.lonely_hearts().into_iter()
                    .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v))
                    .min_by_key(|&v| bond_n.get(&v).copied().unwrap_or(0));
                if let Some(m) = mark {
                    let name = self.name_of(m);
                    let pet = pets::spawn_pet(&mut self.world, "Pilgrim", Species::Dog, PetRole::Companion);
                    pets::adopt(&mut self.world, pet, m);
                    self.pets.push((pet, Species::Dog, (0.7, 0.4)));
                    desires::satisfy(&mut self.world, m, Want::Love, tick + 60);
                    self.log_entity(m, format!("t{}: the stray won't go - {} has a dog now.", tick, name));
                } else {
                    self.log.push(format!("t{}: the stray drifts on down the road.", tick));
                }
            }
            EventId::StrayCat => {
                // a player who's kept her company keeps her - her own
                // warmth decides before the hearths do; a hearth behind
                // bars is no hearth to her either way
                let cat = self.rare.iter().find(|r| r.id == EventId::StrayCat && !r.leaving)
                    .and_then(|r| r.e.map(|e| (e, r.petters.clone())));
                let tamed = cat.and_then(|(e, petters)| {
                    if self.world.get_component::<Pet>(e)?.bond_strength < STRAY_CAT_TAME_BOND { return None; }
                    // the kindest hand keeps her: most strokes wins, and
                    // seats break a tie in their own order
                    let mut best: Option<(Entity, u32)> = None;
                    for s in &self.seats {
                        if let Some(&n) = petters.get(&s.player) {
                            if best.map(|(_, bn)| n > bn).unwrap_or(true) { best = Some((s.player, n)); }
                        }
                    }
                    best.map(|(owner, _)| (e, owner))
                });
                if let Some((e, owner)) = tamed {
                    pets::adopt(&mut self.world, e, owner);
                    self.pets.push((e, Species::Cat, (0.7, 0.4)));
                    // she's yours now - the event lets its figure go, so
                    // the road doesn't walk her off and destroy her
                    if let Some(r) = self.rare.iter_mut().find(|r| r.id == EventId::StrayCat) { r.e = None; }
                    self.log.push_at_who(format!("t{}: the cat's made her choice - she's yours now.", tick),
                        toast_spot(&self.world, e), None);
                } else {
                    // the loneliest hearth takes her in - cats decide too;
                    // a hearth behind bars is no hearth to her
                    let mut bond_n: crate::engine::det::HashMap<Entity, usize> = crate::engine::det::HashMap::default();
                    self.world.iter::<crate::game::relationships::Relationship>(|_, r| {
                        *bond_n.entry(r.from).or_default() += 1;
                        if r.other != r.from { *bond_n.entry(r.other).or_default() += 1; }
                    });
                    let mark = self.lonely_hearts().into_iter()
                        .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v))
                        .min_by_key(|&v| bond_n.get(&v).copied().unwrap_or(0));
                    if let Some(m) = mark {
                        let name = self.name_of(m);
                        // a village cat earns her keep - Hunting puts her
                        // vermin watch to work, not just her company
                        let pet = pets::spawn_pet(&mut self.world, "Tib", Species::Cat, PetRole::Hunting);
                        pets::adopt(&mut self.world, pet, m);
                        self.pets.push((pet, Species::Cat, (0.7, 0.4)));
                        desires::satisfy(&mut self.world, m, Want::Love, tick + 60);
                        self.log_entity(m, format!("t{}: the cat's made her choice - {} has a cat now.", tick, name));
                    } else {
                        self.log.push(format!("t{}: the cat slips off the way she came.", tick));
                    }
                }
            }
            _ => {}
        }
    }
}
