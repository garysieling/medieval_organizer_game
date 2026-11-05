//! Ill luck: fires, fouled wells, lost children, quarrels and brawls, the assize and the cellar thief.
//! Each fn here holds this family's arms of one handler in events/mod.rs.

use super::*;

impl Game {
    /// ill_luck: the `event_day` arms for this group (the rest fall to the default)
    pub(super) fn ill_luck_day(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::FouledWell => {
                // a week of sour water: folk who drink it get the
                // sour gut - the roll picks who draws the bad dipper
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(tick);
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                let roll = self.rare_roll(id);
                if !able.is_empty() {
                    // two consecutive from the roll's offset - a stride
                    // that wraps can land on the same soul twice
                    let off = (roll as usize / 3) % able.len();
                    for i in 0..able.len().min(2) {
                        let v = able[(off + i) % able.len()];
                        desires::add_grievance(&mut self.world, v, desires::Problem::SourGut, until);
                    }
                }
            }
            _ => {}
        }
    }

    /// ill_luck: the `event_began` arms for this group (the rest fall to the default)
    pub(super) fn ill_luck_began(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::FouledWell => {
                self.log.push(format!("t{}: a fouled well - the water's off, and the dippers come up green.", self.tick));
                self.toasts.push(("A fouled well".into(), self.tick + 40));
            }
            EventId::LostChild => {
                // two days out in the wood: a rolled kid's gone, and
                // the house wears the worry while the village searches
                let kids: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<crate::game::vocation::Kid>(v).is_some()).collect();
                let roll = self.rare_roll(id);
                let lost_at = kids.get((roll as usize / 5) % kids.len().max(1))
                    .and_then(|&k| toast_spot(&self.world, k));
                if let Some(&k) = kids.get((roll as usize / 5) % kids.len().max(1)) {
                    // the kid herself rides in aux - a roster shift can't
                    // send the wrong one home
                    if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = k as u64 + 1; }
                    // she's gone: walked to the wood's edge while the
                    // village searches
                    let wood = self.mush_patches.first().map(|p| p.tile).unwrap_or((2, 2));
                    if let Some(p) = self.world.get_component_mut::<Pos>(k) {
                        p.x = wood.0 as f32;
                        p.y = wood.1 as f32;
                    }
                    let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                        .unwrap_or(self.tick + 2 * 80);
                    if let Some(hh) = household::home_of(&self.world, k) {
                        for &o in self.world.get_component::<household::Building>(hh)
                            .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                            if o == k || villager::is_dead(&self.world, o)
                                || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                            desires::add_grievance(&mut self.world, o, desires::Problem::Grief, until);
                        }
                    }
                    self.log.push(format!("t{}: a lost child - she's not come in for supper, and the wood's dark already.", self.tick));
                } else {
                    self.log.push(format!("t{}: a child was late for supper - it gave the village a turn.", self.tick));
                }
                self.toasts.push_at(("A lost child".into(), self.tick + 40), lost_at);
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// ill_luck: the `event_moment` arms for this group (the rest fall to the default)
    pub(super) fn ill_luck_moment(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::AlehouseBrawl => self.brawl_moment(),
            EventId::Assize => self.assize_moment(),
            EventId::CellarThief => self.cellar_thief_moment(),
            EventId::ChimneyFire => {
                // a rolled home's flue catches - the fire ticks and
                // spreads the way any fire does, and the house grieves
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                let fire_at = homes.get((roll as usize / 3) % homes.len().max(1))
                    .and_then(|&h| toast_spot(&self.world, h));
                if let Some(&h) = homes.get((roll as usize / 3) % homes.len().max(1)) {
                    let lit = self.world.get_component::<Pos>(h).copied()
                        .map(|p| self.fire.ignite(&self.map, p.x.round() as i32, p.y.round() as i32))
                        .unwrap_or(false);
                    let name = self.world.get_component::<household::Building>(h)
                        .map(|b| b.name.clone()).unwrap_or_else(|| "a house".into());
                    if lit {
                        self.log.push(format!("t{}: a chimney fire - smoke gone wrong at {}, and the neighbors with the buckets.", self.tick, name));
                        for &o in self.world.get_component::<household::Building>(h)
                            .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                            if villager::is_dead(&self.world, o)
                                || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                            // her own hearth's burned out, not a wolf's take
                            desires::add_grievance(&mut self.world, o, desires::Problem::NoHome, self.tick + 30);
                        }
                    } else {
                        // a scorched or burning house can't catch twice -
                        // the neighbors beat it out quick this time
                        self.log.push(format!("t{}: a chimney fire tried at {} - but the neighbors beat it out quick.", self.tick, name));
                    }
                } else {
                    self.log.push(format!("t{}: a chimney fire - smoke gone wrong, but the neighbors put it out.", self.tick));
                }
                self.toasts.push_at(("A chimney fire!".into(), self.tick + 40), fire_at);
            }
            EventId::GranaryLeak => {
                // the rain got in: a share of the granary's stock
                // spoils where the water found it
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    let spoiled = f.servings / 10;
                    f.servings = f.servings.saturating_sub(spoiled);
                    if spoiled > 0 {
                        self.log.push(format!("t{}: a granary leak - the rain found a seam and a tenth of the stock's gone to rot.", self.tick));
                    } else {
                        self.log.push(format!("t{}: a granary leak - the rain found a seam, but there's little enough left to spoil.", self.tick));
                    }
                } else {
                    self.log.push(format!("t{}: a granary leak - the rain found a seam.", self.tick));
                }
                self.toasts.push_at(("A granary leak".into(), self.tick + 40),
                    toast_spot(&self.world, self.market_food));
            }
            EventId::CartOverturns => {
                // the wheel finds the rut: a rolled home's load spills
                // on the road and the driver takes the loss - and no
                // driver means no cart, so the home needs someone about
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home
                            && bb.stock.values().any(|&q| q > 0)
                            && bb.occupants.iter().any(|&o|
                                !villager::is_dead(&self.world, o)
                                    && !crate::game::dungeon::unavailable(&self.world, o)))
                        .unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                let cart_at = homes.get((roll as usize / 3) % homes.len().max(1))
                    .and_then(|&h| toast_spot(&self.world, h));
                if let Some(&h) = homes.get((roll as usize / 3) % homes.len().max(1)) {
                    let mut sorted: Vec<String> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.stock.iter().filter(|(_g, q)| **q > 0)
                            .map(|(g, _)| g.clone()).collect()).unwrap_or_default();
                    sorted.sort();
                    if let Some(g) = sorted.first() {
                        if let Some(mut b) = self.world.get_component_mut::<household::Building>(h) {
                            household::take_stock(&mut b, &g.clone(), 3);
                        }
                    }
                    // the driver takes it hard
                    let driver = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)));
                    if let Some(d) = driver {
                        // the spilled load's picked off the road, not wolf-taken
                        desires::add_grievance(&mut self.world, d, desires::Problem::Robbed, self.tick + 20);
                        self.say(d, "The wheel found the rut - the whole load's in the mud!", CHAT_BUBBLE_SECS);
                    }
                    self.log.push(format!("t{}: a cart overturns - the wheel found the rut and the load's in the mud.", self.tick));
                } else {
                    self.log.push(format!("t{}: a cart overturns on the road - nothing much in her to spill.", self.tick));
                }
                self.toasts.push_at(("A cart overturns".into(), self.tick + 40), cart_at);
            }
            EventId::DrownedInPond => {
                // a rolled soul goes to the pond and doesn't come back:
                // the house grieves her - the funeral machinery sees
                // her from there
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                let roll = self.rare_roll(id);
                if let Some(&v) = able.get((roll as usize / 5) % able.len().max(1)) {
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = 0.0;
                    }
                    let house = household::home_of(&self.world, v);
                    if let Some(hh) = house {
                        for &o in self.world.get_component::<household::Building>(hh)
                            .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                            if o == v || villager::is_dead(&self.world, o) { continue; }
                            desires::add_grievance(&mut self.world, o, desires::Problem::Grief, self.tick + 60);
                        }
                    }
                    self.log.push(format!("t{}: drowned in the pond - they pulled her out at the shallows, too late.", self.tick));
                } else {
                    self.log.push(format!("t{}: a body in the pond - nobody of ours, thank the saints.", self.tick));
                }
                self.toasts.push_at(("Drowned in the pond".into(), self.tick + 40),
                    self.pit.map(|(x, y)| (x as f32, y as f32)));
            }
            EventId::Foundling => {
                // a babe in a basket on the step: a rolled home takes
                // her in, and whose she is is the street's new talk
                // a real village roof with a bed to spare: a foreign
                // house or a hut with no room can't take her
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home
                            && bb.occupants.len() < 4
                            && bb.occupants.iter().any(|&o| self.villagers.contains(&o)
                                && !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)))
                        .unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                let baby_at = homes.get((roll as usize / 7) % homes.len().max(1))
                    .and_then(|&h| toast_spot(&self.world, h));
                if let Some(&h) = homes.get((roll as usize / 7) % homes.len().max(1)) {
                    self.spawn_kid(h, self.tick);
                    self.log.push(format!("t{}: a foundling - a babe in a basket on the step, and whose she is nobody's saying.", self.tick));
                } else {
                    self.log.push(format!("t{}: a foundling - a babe in a basket, but no door to lay her at.", self.tick));
                }
                self.toasts.push_at(("A foundling".into(), self.tick + 40), baby_at);
            }
            EventId::Quarrel => {
                // two tempers meet in the square: the whole street
                // hears it, and neither side walks away sweet
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<crate::game::vocation::Kid>(v).is_none()).collect();
                let roll = self.rare_roll(id);
                let quarrel_at = able.get((roll as usize / 3) % able.len().max(1))
                    .and_then(|&v| toast_spot(&self.world, v));
                if able.len() >= 2 {
                    let a = able[(roll as usize / 3) % able.len()];
                    let b = able[(roll as usize / 3 + 1) % able.len()];
                    use crate::game::relationships::{Interaction, InteractionKind};
                    crate::game::relationships::record_interaction(&mut self.world, a,
                        Interaction { other: b, kind: InteractionKind::Conflict, intensity: 0.4 });
                    crate::game::relationships::record_interaction(&mut self.world, b,
                        Interaction { other: a, kind: InteractionKind::Conflict, intensity: 0.4 });
                    self.say(a, "You want a piece of me?!", CHAT_BUBBLE_SECS);
                    self.say(b, "Say that again, I dare you!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a quarrel in the square - shouting, and the whole street turned out to watch.", self.tick));
                } else {
                    self.log.push(format!("t{}: words were had in the square - nothing came of it.", self.tick));
                }
                self.toasts.push_at(("A quarrel in the square".into(), self.tick + 40), quarrel_at);
            }
            EventId::OldOakFalls => {
                // she was older than the church: the landmark's gone,
                // but the wood splits up fair - a share into every
                // satchel, where a craftsman's hands can use it
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    if let Some(s) = self.world.get_component_mut::<crate::game::livelihood::Satchel>(v) {
                        *s.goods.entry("wood".into()).or_insert(0) += 1;
                    }
                }
                if let Some(&v) = self.villagers.iter().find(|&&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)) {
                    self.say(v, "She was older than the church...", CHAT_BUBBLE_SECS);
                }
                self.log.push(format!("t{}: the old oak falls - a crack like thunder in the night, and she's down. The wood splits up fair.", self.tick));
                self.toasts.push(("The old oak falls".into(), self.tick + 40));
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// ill_luck: the `event_end` arms for this group (the rest fall to the default)
    pub(super) fn ill_luck_end(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::LostChild => {
                // she turns up at the wood's edge, hungry and sorry:
                // the house's worry lifts and her belly's the worse for it
                let eid = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0);
                let k = if eid > 0 { Some((eid - 1) as Entity) } else { None };
                let k = k.filter(|k| self.villagers.contains(k));
                if let Some(k) = k {
                    if villager::is_dead(&self.world, k) {
                        // the search found her too late - no homecoming to give
                        self.log.push(format!("t{}: they found her at the wood's edge, but the dark had kept her.", tick));
                    } else if crate::game::dungeon::unavailable(&self.world, k) {
                        // the castle holds her - no wood-road homecoming either
                        self.log.push(format!("t{}: the search ended - she's held where folk can't fetch her home.", tick));
                    } else {
                        if let Some(h) = self.world.get_component_mut::<Health>(k) {
                            if h.condition > 0.05 {
                                h.condition = (h.condition - 0.05).max(0.05);
                            }
                        }
                        // she walks back in the door she left
                        if let Some(hh) = household::home_of(&self.world, k) {
                            let home_tile = self.world.get_component::<Pos>(hh).copied();
                            if let (Some(p), Some(ht)) = (self.world.get_component_mut::<Pos>(k), home_tile) {
                                p.x = ht.x;
                                p.y = ht.y;
                            }
                            for &o in self.world.get_component::<household::Building>(hh)
                                .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                                if o == k || villager::is_dead(&self.world, o)
                                    || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                                desires::satisfy(&mut self.world, o, Want::Joy, tick + 20);
                            }
                        }
                        self.log.push(format!("t{}: she's come out of the wood, hungry and sorry - a night in the dark is the end of it.", tick));
                    }
                } else {
                    self.log.push(format!("t{}: the late one came home at last.", tick));
                }
            }
            _ => {}
        }
    }

    /// The alehouse earns its other name: a cup in and the insults come
    /// out. Bruises, not blades - the drunk and the thirsty swing first
    pub(super) fn brawl_moment(&mut self) {
        // where the drink pours: the tavern is the village's taproom now,
        // so the bench with the barrel on it gets the brawl; failing that
        // whichever drink trade has stock, and failing that the market
        let taproom = self.tavern.filter(|&t|
            self.world.get_component::<household::Building>(t)
                .map(|b| b.stock.iter().any(|(g, q)|
                    crate::game::vocation::is_drink(g) && *q > 0)).unwrap_or(false))
        .or_else(|| self.building_entities.iter().copied().find(|&be|
            self.world.get_component::<household::Building>(be)
                .map(|b| matches!(b.kind, household::BuildingKind::Shop(
                    crate::game::vocation::Vocation::Alewife | crate::game::vocation::Vocation::Brewer
                    | crate::game::vocation::Vocation::Vintner | crate::game::vocation::Vocation::Distiller))
                    && b.stock.iter().any(|(g, q)| crate::game::vocation::is_drink(g) && *q > 0))
                .unwrap_or(false)));
        let at = taproom.and_then(|be| self.world.get_component::<Pos>(be).copied())
            .or_else(|| self.world.get_component::<Pos>(self.market_food).copied());
        let mut brawlers: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<crate::game::vocation::TownRole>(v).is_none())
            .filter(|&v| {
                let drunk = self.world.get_component::<villager::Drunk>(v)
                    .map(|d| self.tick < d.until).unwrap_or(false);
                let near = at.map(|a| self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - a.x).abs() + (p.y - a.y).abs() < 8.0).unwrap_or(false))
                    .unwrap_or(false);
                drunk || near
            }).collect();
        if brawlers.is_empty() {
            // a brawl's still a brawl if the street's free - but a man
            // in the lock-up is no man for a punch-up: the able come
            // first, the window after
            brawlers = self.villagers.iter().copied()
                .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).take(4).collect();
        }
        brawlers.truncate(2);
        let names: Vec<String> = brawlers.iter().map(|&v| self.name_of(v)).collect();
        for &v in &brawlers {
            if let Some(h) = self.world.get_component_mut::<Health>(v) {
                h.condition = (h.condition - 0.05).max(0.05);
            }
            self.say(v, "Say that again!", CHAT_BUBBLE_SECS);
        }
        let place = taproom
            .and_then(|be| self.world.get_component::<household::Building>(be).map(|b| b.name.clone()))
            .unwrap_or_else(|| "the market".into());
        self.log.push(format!("t{}: an alehouse brawl at {} - {} come to blows over a spilled cup.",
            self.tick, place, names.join(" and ")));
    }

    /// The assize of ale: the taster walks the drink-shops. Thin brews
    /// are fined; an honest master's name is praised
    pub(super) fn assize_moment(&mut self) {
        let mut heard = false;
        for &v in &self.villagers.clone() {
            let Some(c) = self.world.get_component::<crate::game::vocation::Craft>(v) else { continue };
            if !matches!(c.vocation, crate::game::vocation::Vocation::Alewife
                | crate::game::vocation::Vocation::Brewer | crate::game::vocation::Vocation::Vintner
                | crate::game::vocation::Vocation::Distiller) { continue; }
            heard = true;
            let name = self.name_of(v);
            if crate::game::vocation::level_of(c.skill) == crate::game::vocation::Level::Master {
                self.log.push(format!("t{}: the ale-taster calls {}'s brew honest and true.", self.tick, name));
                desires::satisfy(&mut self.world, v, Want::Mastery, self.tick + 60);
            } else {
                if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                    p.gold = (p.gold - 3.0).max(0.0);
                }
                self.log.push(format!("t{}: the ale-taster fines {} - thin stuff, barely worth the water.", self.tick, name));
            }
        }
        if !heard {
            self.log.push(format!("t{}: the ale-taster rides through - no brewer's bench to judge here.", self.tick));
        }
    }

    /// A cellar worth robbing is a cellar worth watching: the drink
    /// trade's stock draws a thief in the night, and the watch earns
    /// its keep when one walks the walls
    pub(super) fn cellar_thief_moment(&mut self) {
        // the fattest drink-stock in the village draws him
        let mark = self.building_entities.iter().copied()
            .filter_map(|be| self.world.get_component::<household::Building>(be)
                .map(|b| (be, b.stock.iter()
                    .filter(|(g, _)| crate::game::vocation::is_drink(g))
                    .map(|(g, q)| crate::game::goods::worth_at(&self.world, g) * *q as f32).sum::<f32>())))
            .filter(|(_, v)| *v >= 20.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(be, _)| be);
        let Some(mark) = mark else {
            self.log.push(format!("t{}: a thief slips through in the night - but finds no fat cellar.", self.tick));
            return;
        };
        // a living watch is why a fat cellar locks its door
        let watched = self.villagers.iter().copied().any(|v|
            !villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::game::vocation::Craft>(v)
                    .map(|c| c.vocation == crate::game::vocation::Vocation::Knight).unwrap_or(false));
        let name = self.world.get_component::<household::Building>(mark)
            .map(|b| b.name.clone()).unwrap_or_else(|| "a cellar".into());
        if watched {
            self.log.push(format!("t{}: the watch catches a thief at {} - the good stock stays.", self.tick, name));
            return;
        }
        // half the drink walks off; the shop's keeper wears the grievance
        let worker = self.world.get_component::<household::Building>(mark).and_then(|b| b.worker);
        if let Some(b) = self.world.get_component_mut::<household::Building>(mark) {
            for g in ["ale", "beer", "wine", "spirits"] {
                let q = b.stock.get(g).copied().unwrap_or(0);
                household::take_stock(b, g, (q + 1) / 2);
            }
        }
        if let Some(w) = worker {
            desires::add_grievance(&mut self.world, w, desires::Problem::Robbed, self.tick + 60);
        }
        self.log.push(format!("t{}: a thief in the night - half the drink walks out of {}.", self.tick, name));
    }
}
