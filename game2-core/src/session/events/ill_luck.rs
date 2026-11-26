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
                self.log.push_ev(
                    EvTag::new(Thread::Illness, Valence::Negative, Sev::Notable),
                    format!(
                        "t{}: a fouled well - the water's off, and the dippers come up green.",
                        self.tick
                    ),
                );
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
                let lost_at;
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
                    // capture after the move - the toast should point at
                    // the wood she's lost in, not where she was standing
                    lost_at = toast_spot(&self.world, k);
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
                    self.log_entity_ev(EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Notable), k, format!("t{}: a lost child - she's not come in for supper, and the wood's dark already.", self.tick));
                } else {
                    lost_at = None;
                    self.log.push_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: a child was late for supper - it gave the village a turn.",
                            self.tick
                        ),
                    );
                }
                self.toasts
                    .push_at(("A lost child".into(), self.tick + 40), lost_at);
            }
            _ => {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", self.tick, s.name),
                );
            }
        }
    }

    /// ill_luck: the `event_moment` arms for this group (the rest fall to the default)
    pub(super) fn ill_luck_moment(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::AlehouseBrawl => self.brawl_moment(),
            EventId::Assize => self.assize_moment(),
            EventId::CellarThief => self.cellar_thief_moment(),
            EventId::SheepInCradle => self.sheep_cradle_moment(),
            EventId::NoahRow => self.noah_row_moment(),
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
                        .map(|p| self.fire.ignite(&self.map, p.x.round() as i32, p.y.round() as i32, self.tick))
                        .unwrap_or(false);
                    let name = self.world.get_component::<household::Building>(h)
                        .map(|b| b.name.clone()).unwrap_or_else(|| "a house".into());
                    if lit {
                        if let Some((fx, fy)) = fire_at { self.emit_sfx("alarm", fx, fy, 1.0); }
                        self.log.push_ev_at_who(EvTag::new(Thread::Fire, Valence::Negative, Sev::Notable), format!("t{}: a chimney fire - smoke gone wrong at {}, and the neighbors with the buckets.", self.tick, name), fire_at, None);
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
                        self.log.push_ev_at_who(EvTag::new(Thread::Fire, Valence::Positive, Sev::Minor), format!("t{}: a chimney fire tried at {} - but the neighbors beat it out quick.", self.tick, name), fire_at, None);
                    }
                } else {
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Fire, Valence::Positive, Sev::Minor),
                        format!(
                            "t{}: a chimney fire - smoke gone wrong, but the neighbors put it out.",
                            self.tick
                        ),
                        fire_at,
                        None,
                    );
                }
                self.toasts.push_at(("A chimney fire!".into(), self.tick + 40), fire_at);
            }
            EventId::GranaryLeak => {
                // the rain got in: a share of the granary's stock
                // spoils where the water found it
                let leak_at = toast_spot(&self.world, self.market_food);
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    let spoiled = f.servings / 10;
                    f.servings = f.servings.saturating_sub(spoiled);
                    if spoiled > 0 {
                        self.log.push_ev_at_who(EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable), format!("t{}: a granary leak - the rain found a seam and a tenth of the stock's gone to rot.", self.tick), leak_at, None);
                    } else {
                        self.log.push_ev_at_who(EvTag::new(Thread::Economy, Valence::Negative, Sev::Minor), format!("t{}: a granary leak - the rain found a seam, but there's little enough left to spoil.", self.tick), leak_at, None);
                    }
                } else {
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Economy, Valence::Negative, Sev::Minor),
                        format!("t{}: a granary leak - the rain found a seam.", self.tick),
                        leak_at,
                        None,
                    );
                }
                self.toasts.push_at(("A granary leak".into(), self.tick + 40), leak_at);
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
                    self.log.push_ev_at_who(EvTag::new(Thread::Economy, Valence::Negative, Sev::Minor), format!("t{}: a cart overturns - the wheel found the rut and the load's in the mud.", self.tick), cart_at, None);
                } else {
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Economy, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: a cart overturns on the road - nothing much in her to spill.",
                            self.tick
                        ),
                        cart_at,
                        None,
                    );
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
                    // its own line names the manner - mark the death
                    // told so the frame check doesn't tell it again
                    // under the bare word (#1696)
                    self.mark_dead_told(v, "drowned in the pond".into());
                    let house = household::home_of(&self.world, v);
                    if let Some(hh) = house {
                        for &o in self.world.get_component::<household::Building>(hh)
                            .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                            if o == v || villager::is_dead(&self.world, o) { continue; }
                            desires::add_grievance(&mut self.world, o, desires::Problem::Grief, self.tick + 60);
                        }
                    }
                    self.log_entity_ev(EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major), v, format!("t{}: drowned in the pond - they pulled her out at the shallows, too late.", self.tick));
                } else {
                    let pond_at = self.pit.map(|(x, y)| (x as f32, y as f32));
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: a body in the pond - nobody of ours, thank the saints.",
                            self.tick
                        ),
                        pond_at,
                        None,
                    );
                }
                self.toasts.push_at(
                    ("Drowned in the pond".into(), self.tick + 40),
                    self.pit.map(|(x, y)| (x as f32, y as f32)),
                );
            }
            EventId::Foundling => {
                // a babe in a basket on the step: a rolled home takes
                // her in, and whose she is is the street's new talk
                // a real village roof with a bed to spare: a foreign
                // house or a hut with no room can't take her - and
                // neither can what's burned or drowned
                let homes: Vec<Entity> = self.building_entities.iter().enumerate()
                    .filter(|(i, &be)| !self.ruined.contains(i)
                        && self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home
                            && bb.occupants.len() < 4
                            && bb.occupants.iter().any(|&o| self.villagers.contains(&o)
                                && !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)))
                        .unwrap_or(false)).map(|(_, &be)| be).collect();
                let roll = self.rare_roll(id);
                let baby_at = homes.get((roll as usize / 7) % homes.len().max(1))
                    .and_then(|&h| toast_spot(&self.world, h));
                if let Some(&h) = homes.get((roll as usize / 7) % homes.len().max(1)) {
                    self.spawn_kid(h, self.tick);
                    self.log.push_ev_at_who(EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable), format!("t{}: a foundling - a babe in a basket on the step, and whose she is nobody's saying.", self.tick),
                        baby_at, Some(self.villagers.len() - 1));
                } else {
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: a foundling - a babe in a basket, but no door to lay her at.",
                            self.tick
                        ),
                        baby_at,
                        None,
                    );
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
                    self.say(b, &self.vv_said(b, a, "Say that again, I dare {2:obj}!"), CHAT_BUBBLE_SECS);
                    let who = self.villagers.iter().position(|&x| x == a);
                    self.log.push_ev_at_who(EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor), format!("t{}: a quarrel in the square - shouting, and the whole street turned out to watch.", self.tick), quarrel_at, who);
                } else {
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: words were had in the square - nothing came of it.",
                            self.tick
                        ),
                        quarrel_at,
                        None,
                    );
                }
                self.toasts.push_at(
                    ("A quarrel in the square".into(), self.tick + 40),
                    quarrel_at,
                );
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
                self.log.push_ev(EvTag::new(Thread::Economy, Valence::Positive, Sev::Minor), format!("t{}: the old oak falls - a crack like thunder in the night, and she's down. The wood splits up fair.", self.tick));
                self.toasts
                    .push(("The old oak falls".into(), self.tick + 40));
            }
            _ => {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", self.tick, s.name),
                );
            }
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
                        self.log_entity_ev(EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major), k, format!("t{}: they found her at the wood's edge, but the dark had kept her.", tick));
                    } else if crate::game::dungeon::unavailable(&self.world, k) {
                        // the castle holds her - no wood-road homecoming either
                        self.log_entity_ev(EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major), k, format!("t{}: the search ended - she's held where folk can't fetch her home.", tick));
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
                        self.log_entity_ev(EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable), k, format!("t{}: she's come out of the wood, hungry and sorry - a night in the dark is the end of it.", tick));
                    }
                } else {
                    self.log.push_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
                        format!("t{}: the late one came home at last.", tick),
                    );
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
        let who = brawlers
            .first()
            .and_then(|&v| self.villagers.iter().position(|&x| x == v));
        self.log.push_ev_at_who(
            EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Minor),
            format!(
                "t{}: an alehouse brawl at {} - {} come to blows over a spilled cup.",
                self.tick,
                place,
                names.join(" and ")
            ),
            at.map(|p| (p.x, p.y)),
            who,
        );
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
                self.log_entity_ev(
                    EvTag::new(Thread::Craft, Valence::Positive, Sev::Minor),
                    v,
                    format!(
                        "t{}: the ale-taster calls {}'s brew honest and true.",
                        self.tick, name
                    ),
                );
                desires::satisfy(&mut self.world, v, Want::Mastery, self.tick + 60);
            } else {
                if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                    p.gold = (p.gold - 3.0).max(0.0);
                }
                self.log_entity_ev(
                    EvTag::new(Thread::Craft, Valence::Negative, Sev::Minor),
                    v,
                    format!(
                        "t{}: the ale-taster fines {} - thin stuff, barely worth the water.",
                        self.tick, name
                    ),
                );
            }
        }
        if !heard {
            self.log.push_ev(
                EvTag::new(Thread::Craft, Valence::Neutral, Sev::Minor),
                format!(
                    "t{}: the ale-taster rides through - no brewer's bench to judge here.",
                    self.tick
                ),
            );
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
            self.log.push_ev(
                EvTag::new(Thread::Economy, Valence::Neutral, Sev::Minor),
                format!(
                    "t{}: a thief slips through in the night - but finds no fat cellar.",
                    self.tick
                ),
            );
            return;
        };
        // a living watch is why a fat cellar locks its door
        let watched = self.villagers.iter().copied().any(|v|
            !villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::game::vocation::Craft>(v)
                    .map(|c| c.vocation == crate::game::vocation::Vocation::Knight).unwrap_or(false));
        let name = self.world.get_component::<household::Building>(mark)
            .map(|b| b.name.clone()).unwrap_or_else(|| "a cellar".into());
        let mark_at = self.world.get_component::<Pos>(mark).map(|p| (p.x, p.y));
        if watched {
            self.log.push_ev_at_who(
                EvTag::new(Thread::Economy, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: the watch catches a thief at {} - the good stock stays.",
                    self.tick, name
                ),
                mark_at,
                None,
            );
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
        self.log.push_ev_at_who(
            EvTag::new(Thread::Economy, Valence::Negative, Sev::Notable),
            format!(
                "t{}: a thief in the night - half the drink walks out of {}.",
                self.tick, name
            ),
            mark_at,
            None,
        );
    }
}

impl Game {
    /// The sheep in the cradle (#1254): her ewe's gone, and she knows
    /// whose pot it's in. She walks to your door to say it; the house
    /// that has it bluffs until somebody's kindness lifts the lid
    pub(super) fn sheep_cradle_moment(&mut self) {
        let roll = self.rare_roll(EventId::SheepInCradle);
        let grown: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
            !villager::is_dead(&self.world, v)
                && !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<crate::game::vocation::Kid>(v).is_none()).collect();
        if grown.len() < 2 { return; }
        // a shepherd knows her ewe first; any grown hand can be the one robbed
        let shepherds: Vec<Entity> = grown.iter().copied().filter(|&v|
            self.world.get_component::<crate::game::vocation::Craft>(v)
                .map(|c| c.vocation == crate::game::vocation::Vocation::Shepherd)
                .unwrap_or(false)).collect();
        let victim = if shepherds.is_empty() {
            grown[(roll as usize / 3) % grown.len()]
        } else {
            shepherds[(roll as usize / 3) % shepherds.len()]
        };
        let vhome = household::home_of(&self.world, victim);
        // the thief keeps it in his own cradle: a grown soul in a house
        // that isn't hers
        let thieves: Vec<Entity> = grown.iter().copied().filter(|&v| {
            if v == victim { return false; }
            let h = household::home_of(&self.world, v);
            h.is_some() && h != vhome
        }).collect();
        if thieves.is_empty() { return; }
        let thief = thieves[(roll as usize / 11) % thieves.len()];
        self.ewe_theft = Some(EweTheft {
            victim, thief, fork_open: false, held: false });
        desires::add_grievance(&mut self.world, victim, desires::Problem::Loss, self.tick + 30);
        // she brings it to your door: a long walk to where you stand
        let at = self.world.get_component::<Pos>(self.player)
            .map(|p| (p.x.round() as i32, p.y.round() as i32));
        if let Some(at) = at { self.send_far(victim, at, Goal::TalkTo(self.player)); }
        let (vn, tn) = (self.name_of(victim), self.name_of(thief));
        let vat = toast_spot(&self.world, victim);
        let who = self.villagers.iter().position(|&v| v == victim);
        self.log.push_ev_at_who(
            EvTag::new(Thread::Law, Valence::Negative, Sev::Minor),
            format!(
                "t{}: {}'s ewe is gone from the fold - she swears it's {}'s pot it's in.",
                self.tick, vn, tn),
            vat, who);
        self.toasts.push_at((format!("{}'s ewe is gone", vn), self.tick + 40), vat);
    }

    /// The gift lifts the lid (#1254): something for the house and the
    /// cradle's no babe. The talk says what it found and the fork goes
    /// on the card - name him before the village, or keep the name
    pub(in crate::session) fn ewe_cracked(&mut self, seat: usize) {
        // another card's already up - the lid stays on until the
        // card's free to ask the fork (#1256)
        if self.dilemma.is_some() { return; }
        let Some(t) = self.ewe_theft.as_mut() else { return };
        if t.fork_open || t.held { return; }
        t.fork_open = true;
        let victim = t.victim;
        self.narrate_talk(seat,
            "The lid lifts on the cradle - it's no babe: a ewe folds there, wool dyed dark.");
        self.dilemma = Some(MoralKnot {
            text: "It's her ewe, folded into the cradle and dyed. The name's yours to spend.".into(),
            branches: vec![
                DilemmaBranch { text: "name him before the village".into(),
                    debt: 0.0, victim },
                DilemmaBranch { text: "keep the name - a word in his ear another day".into(),
                    debt: 0.0, victim },
            ],
            offered: self.tick });
    }

    /// The ewe's fork resolves (#1254): the name goes public and his
    /// standing collapses, or it stays in your pocket as leverage
    pub(in crate::session) fn resolve_ewe_fork(&mut self, pick: usize) {
        let Some(t) = self.ewe_theft.take() else { return };
        let (vn, tn) = (self.name_of(t.victim), self.name_of(t.thief));
        if pick == 0 {
            // expose: the parish knows what's in the cradle. The theft's
            // on their bond's record, his standing falls out from under
            // him, and the house that hid it wears the blame
            crate::game::relationships::record_interaction(&mut self.world, t.victim,
                crate::game::relationships::Interaction { other: t.thief,
                    kind: crate::game::relationships::InteractionKind::Betrayal, intensity: 0.8 });
            if self.world.get_component::<crate::sim::reputation_qsp::ReputationState>(t.thief).is_none() {
                self.world.add_component(t.thief, crate::sim::reputation_qsp::ReputationState {
                    stage: crate::sim::reputation_qsp::RepStage::Known, score: 0.45, transition_rates: vec![] });
            }
            for _ in 0..2 {
                if let Some(r) = self.world.get_component_mut::<crate::sim::reputation_qsp::ReputationState>(t.thief) {
                    crate::sim::reputation_qsp::apply_action(r, crate::sim::reputation_qsp::RepAction::Betrayal);
                }
            }
            if let Some(h) = household::home_of(&self.world, t.thief) {
                for &o in self.world.get_component::<household::Building>(h)
                    .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                    if villager::is_dead(&self.world, o) { continue; }
                    desires::add_grievance(&mut self.world, o,
                        desires::Problem::Blamed, self.tick + 60);
                }
            }
            let tat = toast_spot(&self.world, t.thief);
            let who = self.villagers.iter().position(|&v| v == t.thief);
            self.log.push_ev_at_who(
                EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                format!(
                    "t{}: the ewe comes out of {}'s cradle before the whole parish - he stands disgraced, and {} has her beast home.",
                    self.tick, tn, vn),
                tat, who);
            self.toasts.push_at((format!("{} stands disgraced", tn), self.tick + 40), tat);
        } else {
            // hold: the name stays in your pocket - his Talk carries the
            // squeeze until it's paid out
            self.ewe_theft = Some(EweTheft { fork_open: false, held: true, ..t });
            self.log.push_ev(
                EvTag::new(Thread::Law, Valence::Neutral, Sev::Minor),
                format!(
                    "t{}: you keep what you saw in {}'s cradle - a word for another day.",
                    self.tick, tn),
            );
        }
    }

    /// The squeeze pays out once (#1254): hush money out of his purse
    /// into yours, and the name's spent
    pub(in crate::session) fn ewe_squeezed(&mut self, seat: usize) {
        let Some(t) = self.ewe_theft.clone() else { return };
        if !t.held { return; }
        const EWE_HUSH: f32 = 4.0;
        let pe = self.seats[seat].player;
        let gold = self.world.get_component::<Purse>(t.thief)
            .map(|p| p.gold.min(EWE_HUSH)).unwrap_or(0.0);
        if let Some(p) = self.world.get_component_mut::<Purse>(t.thief) {
            p.gold -= gold;
        }
        if let Some(p) = self.world.get_component_mut::<Purse>(pe) {
            p.gold += gold;
        }
        self.ewe_theft = None;
        let tn = self.name_of(t.thief);
        self.narrate_talk(seat, &format!(
            "{tn} presses {gold:.0} gold into your hand, and what's in his cradle stays between you."));
    }
}

impl Game {
    /// The Noah row (#1257): a marriage's disagreement carried into the
    /// street. The matter's seeded from real state - a member's spouse
    /// who won't stomach the meetings, the want that ate the rent, or
    /// the folly she won't follow him into. The street turns out, each
    /// watcher takes a side by their own bonds, and both halves wear
    /// the mark of it. A house that keeps rowing in public wears its
    /// own bond down - the street sees the crack first
    pub(in crate::session) fn noah_row_moment(&mut self) {
        use crate::game::relationships as rel;
        let tick = self.tick;
        // standing marriages: the union holds (form 0), both halves
        // alive and about, grown. Each pair counted once, by the
        // villagers' own order
        let mut pairs: Vec<(Entity, Entity)> = vec![];
        for &v in self.villagers.iter() {
            if villager::is_dead(&self.world, v)
                || crate::game::dungeon::unavailable(&self.world, v)
                || self.world.get_component::<crate::game::vocation::Kid>(v).is_some() { continue; }
            let Some(m) = self.world.get_component::<rel::Married>(v) else { continue };
            if m.form != 0 { continue; }
            let s = m.spouse;
            if villager::is_dead(&self.world, s)
                || crate::game::dungeon::unavailable(&self.world, s)
                || self.world.get_component::<crate::game::vocation::Kid>(s).is_some() { continue; }
            let (vi, si) = (self.villagers.iter().position(|&x| x == v).unwrap_or(usize::MAX),
                self.villagers.iter().position(|&x| x == s).unwrap_or(usize::MAX));
            if vi < si { pairs.push((v, s)); }
        }
        if pairs.is_empty() {
            self.log.push_ev(EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                format!("t{tick}: raised voices behind a shutter somewhere - nothing came of it."));
            return;
        }
        // each pair's own matter: the meetings one of them keeps and the
        // other won't stomach, the want that spent the rent, or the ark
        // itself. Villager order, first seed that fits
        let mut seeded: Vec<(Entity, Entity, crate::session::events::RowMatter)> = vec![];
        for &(a, b) in &pairs {
            // the cause row: one's signed on, the other won't have it
            let mut matter = None;
            for st in self.seats.iter() {
                let mem = |x| self.world.get_component::<crate::sim::community::Organization>(st.org)
                    .map(|o| o.members.contains(&x)).unwrap_or(false);
                let (member, _) = match (mem(a), mem(b)) {
                    (true, false) => (a, b),
                    (false, true) => (b, a),
                    _ => continue,
                };
                let issue = crate::game::campaign::campaign_of(&self.world, st.org)
                    .and_then(|ce| self.world.get_component::<crate::game::campaign::Campaign>(ce))
                    .map(|c| c.issue.clone());
                if let Some(issue) = issue {
                    matter = Some(crate::session::events::RowMatter::Cause { member, issue });
                    break;
                }
            }
            // wants against means: a costly unsatisfied want in a house
            // that's short - the rent that bought a lute
            if matter.is_none() {
                for &(who, other) in &[(a, b), (b, a)] {
                    let short = desires::problems_of(&self.world, who, tick)
                        .contains(&desires::Problem::Broke)
                        || desires::problems_of(&self.world, other, tick)
                            .contains(&desires::Problem::Broke)
                        || self.world.get_component::<Purse>(who)
                            .map(|p| p.gold < 3.0).unwrap_or(false)
                        || self.world.get_component::<Purse>(other)
                            .map(|p| p.gold < 3.0).unwrap_or(false);
                    if !short { continue; }
                    let want = self.world.get_component::<desires::InnerLife>(who)
                        .map(|l| l.wants.iter().copied().find(|&w|
                            matches!(w, desires::Want::Instrument | desires::Want::Finery
                                | desires::Want::BetterTools | desires::Want::Drink)
                            && !desires::is_satisfied(&self.world, who, w, tick)))
                        .flatten();
                    if let Some(want) = want {
                        matter = Some(crate::session::events::RowMatter::Want { want, spender: who });
                        break;
                    }
                }
            }
            seeded.push((a, b, matter.unwrap_or(crate::session::events::RowMatter::Folly)));
        }
        let roll = self.rare_roll(EventId::NoahRow);
        let (a, b, matter) = seeded[(roll as usize / 3) % seeded.len()].clone();
        let at = toast_spot(&self.world, a);
        // the house's tally on the street's book: the count wears the
        // marital bond harder each time the row's a public one
        let key = if self.villagers.iter().position(|&x| x == a)
            < self.villagers.iter().position(|&x| x == b) { (a, b) } else { (b, a) };
        let nth = self.row_count.get(&key).copied().unwrap_or(0) + 1;
        self.row_count.insert(key, nth);
        let (an, bn) = (self.name_of(a), self.name_of(b));
        // the words themselves, by the matter - and who's saying them
        let (accuser, accused) = match &matter {
            crate::session::events::RowMatter::Cause { member, .. } =>
                (if a == *member { b } else { a }, *member),
            crate::session::events::RowMatter::Want { spender, .. } =>
                (if a == *spender { b } else { a }, *spender),
            crate::session::events::RowMatter::Folly =>
                (if roll % 2 == 0 { a } else { b }, if roll % 2 == 0 { b } else { a }),
        };
        let (accuse, defend) = match &matter {
            crate::session::events::RowMatter::Cause { .. } =>
                ("{2:poss} meetings! I'll not have {2:obj} wasting {2:poss} nights in their company!",
                 "It's bread for the both of us, love."),
            crate::session::events::RowMatter::Want { want, .. } => match want {
                desires::Want::Instrument =>
                    ("A lute! The rent's not yet paid and a lute it was!",
                     "A body's got to have summat to sing with."),
                desires::Want::Finery =>
                    ("Ribbons and silks - and the roof lets the rain in!",
                     "A body's allowed one fine thing."),
                desires::Want::BetterTools =>
                    ("New chisels, was it? We eat air this week, then.",
                     "Blunt tools earn nothing."),
                _ =>
                    ("The alehouse got our flour money again!",
                     "A dry throat's no way to end a day."),
            },
            crate::session::events::RowMatter::Folly =>
                ("I'll not come in {2:poss} ark!",
                 "It's a roof-beam, love - a roof-beam!"),
        };
        self.say(accuser, accuse, CHAT_BUBBLE_SECS);
        self.say(accused, defend, CHAT_BUBBLE_SECS);
        // the row's own record: each half carries the conflict, and the
        // union's bond wears - heavier each time the street's heard it
        rel::record_interaction(&mut self.world, a,
            rel::Interaction { other: b, kind: rel::InteractionKind::Conflict, intensity: 0.5 });
        rel::record_interaction(&mut self.world, b,
            rel::Interaction { other: a, kind: rel::InteractionKind::Conflict, intensity: 0.5 });
        let wear = 0.06 * nth as f32;
        for &(who, other) in &[(a, b), (b, a)] {
            if let Some(bond) = self.world.get_component_mut::<crate::sim::psychology::Bond>(who) {
                if bond.target == other {
                    bond.security = (bond.security - wear).max(0.0);
                }
            }
        }
        // the mark on both standings: the street saw the crack
        self.rep_apply(a, crate::sim::reputation_qsp::RepAction::BadDeed);
        self.rep_apply(b, crate::sim::reputation_qsp::RepAction::BadDeed);
        // the street turns out and takes its sides: each watcher's own
        // bonds say which shoulder they stand at. On a cause row the
        // member's side-takers are the cause's opening
        let mut for_a = 0usize;
        let mut for_b = 0usize;
        let mut woken = 0usize;
        if let Some((ax, ay)) = at {
            for &w in self.villagers.iter() {
                if w == a || w == b || villager::is_dead(&self.world, w)
                    || crate::game::dungeon::unavailable(&self.world, w)
                    || self.world.get_component::<crate::game::vocation::Kid>(w).is_some() { continue; }
                let near = self.world.get_component::<Pos>(w)
                    .map(|p| (p.x - ax).abs() + (p.y - ay).abs() <= 10.0).unwrap_or(false);
                if !near { continue; }
                let Some(fav) = crate::game::reactions::take_side(&self.world, w, a, b) else { continue };
                let unfav = if fav == a { b } else { a };
                if fav == a { for_a += 1; } else { for_b += 1; }
                rel::record_interaction(&mut self.world, w,
                    rel::Interaction { other: fav, kind: rel::InteractionKind::Conversation,
                        intensity: 0.1 });
                rel::record_interaction(&mut self.world, w,
                    rel::Interaction { other: unfav, kind: rel::InteractionKind::Conflict,
                        intensity: 0.05 });
                if let crate::session::events::RowMatter::Cause { member, ref issue } = matter {
                    if fav == member
                        && crate::game::reactions::witness(&mut self.world, w, issue,
                            &[member], tick) { woken += 1; }
                }
            }
        }
        let tilt = if for_a > for_b { format!(" The street's mostly with {an}.") }
            else if for_b > for_a { format!(" The street's mostly with {bn}.") }
            else { String::new() };
        let woke = if woken > 0 { format!(" {woken} of the watchers heard the cause in it.") }
            else { String::new() };
        let again = if nth >= 3 { " The street's seen this pair before - the crack's showing.".to_string() }
            else { String::new() };
        let what = match &matter {
            crate::session::events::RowMatter::Cause { .. } => "over the meetings".to_string(),
            crate::session::events::RowMatter::Want { want, .. } => format!(
                "over the {}", match want {
                    desires::Want::Instrument => "lute",
                    desires::Want::Finery => "ribbons",
                    desires::Want::BetterTools => "tools",
                    _ => "ale money" }),
            crate::session::events::RowMatter::Folly => "over his ark".to_string(),
        };
        let who = self.villagers.iter().position(|&v| v == a);
        self.log.push_ev_at_who(
            EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Notable),
            format!("t{tick}: {an} and {bn} row in the street {what} - the whole row's on show.{tilt}{woke}{again}"),
            at, who);
        self.toasts.push_at(("A row in the street".into(), tick + 40), at);
        self.noah_row = Some(crate::session::events::NoahRow {
            a, b, matter, until: tick });
    }

    /// The mediator's word lands (#1257): talked down before the whole
    /// street, the row goes out like a lamp. The wear the row already
    /// did stands; what heals is the heat
    pub(in crate::session) fn row_calmed(&mut self, seat: usize, who: Entity) {
        let Some(r) = self.noah_row.take() else { return };
        if who != r.a && who != r.b { self.noah_row = Some(r); return; }
        let pe = self.seats[seat].player;
        use crate::game::relationships as rel;
        for &e in &[r.a, r.b] {
            rel::record_interaction(&mut self.world, e,
                rel::Interaction { other: pe, kind: rel::InteractionKind::Conversation,
                    intensity: 0.3 });
        }
        let (an, bn) = (self.name_of(r.a), self.name_of(r.b));
        self.narrate_talk(seat, "The words go out of it - the street drifts off, and it's just the two of them breathing.");
        self.log.push_ev(EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor),
            format!("t{}: talked down - {an} and {bn}'s row goes out like a lamp.", self.tick));
    }

    /// The mediator takes a side instead (#1257): the sided-with warms
    /// to the voice that backed them, the other cools - and the row
    /// keeps burning
    pub(in crate::session) fn row_sided(&mut self, seat: usize, favored: Entity) {
        let Some(r) = self.noah_row.as_ref() else { return };
        if favored != r.a && favored != r.b { return; }
        let (other, pe) = (if favored == r.a { r.b } else { r.a }, self.seats[seat].player);
        use crate::game::relationships as rel;
        rel::record_interaction(&mut self.world, favored,
            rel::Interaction { other: pe, kind: rel::InteractionKind::Conversation,
                intensity: 0.3 });
        rel::record_interaction(&mut self.world, other,
            rel::Interaction { other: pe, kind: rel::InteractionKind::Conflict,
                intensity: 0.2 });
        let (fn_, on) = (self.name_of(favored), self.name_of(other));
        self.narrate_talk(seat,
            "You take a side where the street can see it - and the row burns on.");
        self.log.push_ev(EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
            format!("t{}: you took {fn_}'s part in it - {on} won't forget whose side.", self.tick));
    }
}
