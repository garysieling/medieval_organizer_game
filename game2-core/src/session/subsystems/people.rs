//! People's lives: births and newcomers, care for the sick, letters, heirlooms and painted houses, smells, the life course and matchmaking.

use crate::session::*;

impl Game {
    /// Take a villager into the parish: a Faith of their own, and a name
    /// on the congregation's roll
    pub(in crate::session) fn join_parish(&mut self, v: Entity) {
        self.world.add_component(v, crate::sim::religion::Faith {
            religion: "the parish".into(), devotion: 0.4, traditions: vec![] });
        if let Some(c) = self.world.get_component_mut::<crate::sim::religion::Congregation>(self.congregation) {
            c.members.push(v);
        }
    }

    /// A child born to a full house: a Kid like the ones the village
    /// started with, bedded where they were born
    pub(in crate::session) fn spawn_kid(&mut self, home: Entity, tick: u64) {
        let (x, y) = self.world.get_component::<Pos>(home).map(|p| (p.x, p.y)).unwrap_or((14.0, 14.0));
        let v = villager::spawn_villager(&mut self.world, x + 0.5, y + 0.5);
        self.world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        self.world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        self.world.add_component(v, personality_from_seed(self.rng.next()));
        self.world.add_component(v, Satchel::default());
        self.world.add_component(v, Known::default());
        self.world.add_component(v, Kid);
        self.world.add_component(v, KidGrowth { age_days: 0, errands: vec![] });
        self.world.add_component(v, crate::sim::psychology::AttachmentHistory {
            anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
        self.world.add_component(v, crate::game::lifecycle::Age { years: 0.0, stage: crate::game::lifecycle::LifeStage::Infant });
        self.world.add_component(v, crate::game::lifecycle::Vitality {
            health: 1.0, max_age: 58.0 + (self.rng.next() % 35) as f32, cause_of_death: None });
        self.world.add_component(v, Purse { gold: 0.0 });
        self.world.add_component(v, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        if let Some(b) = self.world.get_component_mut::<household::Building>(home) { b.occupants.push(v); }
        // born out in the country, they belong to its town or castle
        if let Some(si) = self.sites.iter().position(|s| s.homes.contains(&home)) {
            self.world.add_component(v, country::Outlander { site: si });
        }
        // a village child is born into the parish
        if self.site_of(v).is_none() { self.join_parish(v); }
        self.villagers.push(v);
        // the parallel rolls grow with the roster or the next lookup panics
        self.was_sick.push(None);
        self.died_on.push(None);
        self.log.push(format!("t{}: a child is born - {}.", tick, vill_name(self.villagers.len() - 1)));
        note_story(&mut self.tales, &mut self.tales_told, &mut self.log, self.villagers.len() - 1, tick,
            crate::sim::story::EventKind::Birth, format!("{} is born.", vill_name(self.villagers.len() - 1)));
        // and the neighbours mark it: a small celebration at the house
        if let Some(p) = self.world.get_component::<Pos>(home) {
            self.local_celebration(tick, (p.x, p.y), None, "a new child");
        }
    }

    /// A full house gets a child every so often - two grown folk under
    /// one roof, and the roof fills out from there
    pub(in crate::session) fn maybe_birth(&mut self, tick: u64) {
        for &b in &self.building_entities.clone() {
            let full = self.world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Home
                    // beds cap at four like the first settling - the
                    // building's capacity is its stock limit, not beds
                    && bb.occupants.len() < 4
                    && bb.occupants.iter().filter(|&&o|
                        self.world.get_component::<Kid>(o).is_none()
                        && !crate::game::dungeon::unavailable(&self.world, o)).count() >= 2)
                .unwrap_or(false);
            if full && self.rng.next() % 3 == 0 { self.spawn_kid(b, tick); return; }
        }
    }

    /// A stranger settles: trade in hand and no roof over them yet -
    /// the first thing on their mind is a bed
    pub(in crate::session) fn maybe_migrant(&mut self, tick: u64) {
        let (x, y) = ((self.town_wall.0 + self.town_wall.2) as f32 / 2.0, self.town_wall.3 as f32 + 2.0);
        let v = villager::spawn_villager(&mut self.world, x, y);
        self.world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        self.world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        self.world.add_component(v, personality_from_seed(self.rng.next()));
        self.world.add_component(v, Satchel::default());
        self.world.add_component(v, Known::default());
        self.world.add_component(v, vocation::craft(roll_vocation(&mut self.rng),
            0.1 + (self.rng.next() % 40) as f32 / 100.0));
        self.world.add_component(v, Purse { gold: 8.0 });
        let years = 19.0 + (self.rng.next() % 42) as f32;
        self.world.add_component(v, crate::game::lifecycle::Age {
            years, stage: crate::game::lifecycle::stage_for(years) });
        self.world.add_component(v, crate::game::lifecycle::Vitality {
            health: 1.0, max_age: 58.0 + (self.rng.next() % 35) as f32, cause_of_death: None });
        self.world.add_component(v, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        self.world.add_component(v, crate::sim::psychology::AttachmentHistory {
            anxiety_episodes: 0, warmth_episodes: 0, episodes: vec![] });
        // a newcomer joins the parish
        self.join_parish(v);
        self.villagers.push(v);
        // the parallel rolls grow with the roster or the next lookup panics
        self.was_sick.push(None);
        self.died_on.push(None);
        // a roof if one's going: spare beds fill before anyone's a
        // stranger long; otherwise a bed is the first thing on their
        // mind. Same exclusions as the first settling - the herb hut is
        // hers alone, and the foreign houses aren't for our strangers
        let room = self.building_entities.iter().copied().find(|&b|
            self.world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Home && bb.occupants.len() < 4
                    && bb.name != "the herb hut" && bb.name != "a foreign house"
                    && !bb.name.starts_with("a cottage in ") && !bb.name.starts_with("the keep at ")
                    && !bb.name.starts_with("the hall at "))
                .unwrap_or(false));
        match room {
            Some(h) => self.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(v),
            None => {
                desires::add_grievance(&mut self.world, v, desires::Problem::NoHome, tick + 60);
                // roofless and newly arrived: give them the want so
                // `want_shopping` can send them to the tavern for a bed
                if let Some(life) = self.world.get_component_mut::<InnerLife>(v) {
                    life.wants.push(Want::Shelter);
                }
            }
        }
        self.log.push(format!("t{}: a stranger settles - {} has come to town.", tick, vill_name(self.villagers.len() - 1)));
    }

    /// The village walks a little slower for its own: a lame leg covers
    /// less ground in a day, and theirs check on them - kin, and the
    /// ones whose bond runs deep
    pub fn care_tick(&mut self, tick: u64) {
        if tick % 7 != 2 { return; }
        // a bonded pet comforts its own distress, no visitor needed - and
        // the pet needs no Pos, it lives at its owner's side by definition
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            // distressed = carrying a live problem, or wanting Joy unmet
            let distressed = !desires::problems_of(&self.world, v, tick).is_empty()
                || self.world.get_component::<desires::InnerLife>(v)
                    .map(|l| l.wants.contains(&Want::Joy)
                        && !desires::is_satisfied(&self.world, v, Want::Joy, tick))
                    .unwrap_or(false);
            if !distressed { continue; }
            if pets::comforting_pet(&self.world, v).is_some() {
                desires::satisfy(&mut self.world, v, Want::Joy, tick + 8);
            }
        }
        // one pass over every bond for the whole round of check-ins -
        // a scan per afflicted villager multiplied by every bond in the
        // village, which grew without bound over a long game
        let bonds = relationships::bond_map(&self.world);
        for &v in &self.villagers.clone() {
            let Some(cond) = self.world.get_component::<villager::Condition>(v)
                .map(|c| c.kind) else { continue };
            if villager::is_dead(&self.world, v) { continue; }
            // someone checks on them: kin under the same roof first,
            // else whoever knows them best
            let checker = household::home_of(&self.world, v)
                .and_then(|home| self.world.get_component::<Building>(home)
                    .map(|b| b.occupants.iter().copied()
                        .find(|&o| o != v && !villager::is_dead(&self.world, o)
                            && !crate::game::dungeon::unavailable(&self.world, o))))
                .flatten()
                .or_else(|| bonds.get(&v).into_iter().flatten()
                    .filter_map(|&r| self.world.get_component::<relationships::Relationship>(r))
                    .filter(|r| r.from != v && r.other == v
                        && !villager::is_dead(&self.world, r.from)
                        && !crate::game::dungeon::unavailable(&self.world, r.from))
                    .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|r| r.from));
            let Some(checker) = checker else { continue };
            // the visit: a bond kept warm, and the walk there counted
            relationships::record_interaction(&mut self.world, checker, relationships::Interaction {
                other: v, kind: relationships::InteractionKind::Conversation, intensity: 1.2 });
            desires::satisfy(&mut self.world, v, Want::Joy, tick + 8);
            if tick % 21 == 2 {
                self.say(checker, match cond {
                    "a lame leg" => "How's the leg treating you?",
                    "failing sight" => "Let me read it out for you.",
                    _ => "The quiet does you good, I hope.",
                }, CHAT_BUBBLE_SECS);
            }
        }
    }

    /// The written word: the literate put word to those they think of -
    /// it travels a week, and the ones who can't read pay the priest a
    /// penny to hear it. The pedlar carries the post.
    pub fn letter_tick(&mut self, tick: u64) {
        // arriving: a letter lands after a week on the road
        let due: Vec<(Entity, Entity, u64)> = self.letters.iter().copied()
            .filter(|(_, _, d)| tick >= *d).collect();
        self.letters.retain(|(_, _, d)| tick < *d);
        for (from, to, _) in due {
            if villager::is_dead(&self.world, to) { continue; }
            // the illiterate pay the priest a penny to hear it read out
            let literate = self.literate(to);
            if !literate {
                // their own priest: a castle's chaplain reads for the castle
                let home = self.site_of(to);
                if let Some(priest) = self.villagers.iter().copied().find(|&v|
                    self.world.get_component::<Craft>(v)
                        .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
                        && self.site_of(v) == home
                        && !crate::game::dungeon::unavailable(&self.world, v)) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(to) {
                        if p.gold > 1.0 {
                            p.gold -= 1.0;
                            if let Some(pp) = self.world.get_component_mut::<Purse>(priest) { pp.gold += 1.0; }
                        }
                    }
                    self.say(to, "The Father read it out - word from home.", CHAT_BUBBLE_SECS);
                }
            }
            // a letter is a bond kept warm across distance - but the
            // grave writes nothing; the bond only warms for the living
            if !villager::is_dead(&self.world, from) {
                relationships::record_interaction(&mut self.world, from, relationships::Interaction {
                    other: to, kind: relationships::InteractionKind::Conversation, intensity: 1.5 });
            }
            desires::satisfy(&mut self.world, to, Want::Joy, tick + 15);
            self.log.push(format!("t{}: a letter comes for {}.", tick, self.name_of(to)));
        }
        // writing: the literate put word to someone they think of
        if tick % 30 != 15 { return; }
        if self.letters.len() >= 3 { return; }
        let writer = self.villagers.iter().copied().find(|&v|
            !villager::is_dead(&self.world, v) && self.literate(v)
            && self.world.get_component::<relationships::Relationship>(v).is_none());
        let Some(writer) = writer else { return };
        // someone they think of: the kindest bond they have, or a lover
        let to = relationships::bonds_of(&self.world, writer).into_iter()
            .filter_map(|r| self.world.get_component::<relationships::Relationship>(r))
            .filter(|r| r.from == writer)
            .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
            .map(|r| r.other);
        if let Some(to) = to {
            self.letters.push((writer, to, tick + 7));
            self.say(writer, "Put it in a letter - they'll read it when the pedlar comes.", CHAT_BUBBLE_SECS);
        }
    }

    /// Can they read? The priest, the printmaker, the clerk - and you.
    pub fn literate(&self, v: Entity) -> bool {
        v == self.player
            || Some(v) == self.translator
            || self.world.get_component::<Craft>(v)
                .map(|c| matches!(c.vocation, Vocation::Priest | Vocation::Printmaker)).unwrap_or(false)
    }

    /// Now and then a maker makes a thing to outlast them - a lute, a
    /// window, a fine toolset - and it gathers stories the way a person
    /// does, and passes to kin when the hands that held it stop.
    pub fn heirloom_tick(&mut self, tick: u64) {
        // the dead pass their things on: a bequest picks kin
        for &item in &self.heirlooms.clone() {
            let Some(h) = self.world.get_component::<villager::Heirloom>(item)
                .map(|h| (h.owner, h.name)) else { continue };
            let (owner, name) = h;
            if !villager::is_dead(&self.world, owner) { continue; }
            // kin first: someone under the same roof
            let kin = household::home_of(&self.world, owner)
                .and_then(|home| self.world.get_component::<Building>(home)
                    .map(|b| b.occupants.iter().copied()
                        .find(|&o| o != owner && !villager::is_dead(&self.world, o))));
            let Some(kin) = kin.flatten() else { continue };
            if let Some(h) = self.world.get_component_mut::<villager::Heirloom>(item) {
                h.owner = kin;
                h.stories.push(format!("passed from the dead, day {}", tick));
            }
            self.log.push(format!("t{}: {} by old hands passes to {} - a thing made to outlast.",
                tick, name, self.name_of(kin)));
            self.say(kin, "It's mine now. I'd rather have them back.", CHAT_BUBBLE_SECS);
        }
        if tick % 90 != 60 { return; }
        // a masterwork: the surest hands in the village lay one down
        let maker = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter_map(|v| self.world.get_component::<Craft>(v)
                .map(|c| (v, c.vocation, c.skill)))
            .filter(|(_, voc, s)| *s > 0.7 && matches!(voc,
                Vocation::Luthier | Vocation::Glazier | Vocation::Toolmaker | Vocation::Weaver | Vocation::Tailor))
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(v, voc, _)| (v, voc));
        let Some((maker, voc)) = maker else { return };
        // no hoarding: a maker's masterwork is a once-a-lifetime thing
        if self.heirlooms.iter().any(|&i| self.world.get_component::<villager::Heirloom>(i)
            .map(|h| h.maker == maker).unwrap_or(false)) { return; }
        let name = match voc {
            Vocation::Luthier => "a lute",
            Vocation::Glazier => "a window",
            Vocation::Toolmaker => "a fine toolset",
            Vocation::Weaver => "a woven hanging",
            _ => "a keepsake",
        };
        let item = self.world.create_entity();
        self.world.add_component(item, villager::Heirloom {
            maker, owner: maker, name, born_tick: tick, stories: vec![] });
        self.heirlooms.push(item);
        self.log.push(format!("t{}: {} lays down {} - a thing made to outlast its maker.", tick, self.name_of(maker), name));
    }

    /// The painter's trade: a pot of paint becomes a bright hall -
    /// color that fades with the seasons, so the work comes round again.
    /// The org's hall painted is a power claim the street can see.
    pub fn painter_tick(&mut self, tick: u64) {
        // the seasons take the color back - a commission's never done
        self.painted.retain(|(_, day)| tick < *day + 365);
        if tick % 14 != 7 { return; }
        let Some(painter) = self.villagers.iter().copied().find(|&v|
            self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Painter).unwrap_or(false))
        else { return };
        let Some(shop) = self.world.get_component::<Craft>(painter).and_then(|c| c.workplace) else { return };
        let took = self.world.get_component_mut::<Building>(shop)
            .map(|b| household::take_stock(b, "paint", 1)).unwrap_or(0);
        if took == 0 { return; }
        // the org's hall first - a painted hall is a power claim - then
        // whatever's gone gray
        let org_hall = household::home_of(&self.world, self.player);
        let target = org_hall.filter(|h| !self.painted.iter().any(|(b, _)| b == h))
            .or_else(|| self.building_entities.iter().copied().find(|&b|
                !self.painted.iter().any(|(pb, _)| *pb == b)
                && self.world.get_component::<Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)));
        if let Some(b) = target {
            self.painted.push((b, tick));
            let name = self.world.get_component::<Building>(b).map(|bb| bb.name.clone()).unwrap_or_default();
            self.log.push(format!("t{}: {} gets its colors - the paint shop's work shows.", tick, name));
            if Some(b) == org_hall {
                self.record_power(crate::sim::organized_power::ActionQuality::Transactional);
            }
        }
    }

    /// The day's work leaves its smell: the fold, the smoke, the cut.
    /// The rain washes everyone clean.
    pub fn smell_tick(&mut self, tick: u64) {
        let _ = tick;
        let raining = self.precip.map(|p| matches!(p,
            crate::sim::precipitation::PrecipKind::Rain
            | crate::sim::precipitation::PrecipKind::HeavyRain
            | crate::sim::precipitation::PrecipKind::Showers)).unwrap_or(false);
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if raining {
                if let Some(s) = self.world.get_component_mut::<Smell>(v) { s.intensity = 0.0; }
                continue;
            }
            let kind = self.world.get_component::<Craft>(v).and_then(|c| match c.vocation {
                Vocation::Shepherd => Some("the fold"),
                Vocation::Beekeeper => Some("smoke and wax"),
                Vocation::Mason => Some("the cut"),
                Vocation::Weaver => Some("the dye"),
                Vocation::Alewife | Vocation::Brewer | Vocation::Distiller => Some("the mash"),
                Vocation::Vintner => Some("the press"),
                Vocation::Cooper => Some("the pitch"),
                _ => None,
            });
            // and the herb-wife always carries her bitter herbs
            let kind = kind.or_else(|| if Some(v) == self.herbologist { Some("bitter herbs") } else { None });
            if let Some(kind) = kind {
                if let Some(s) = self.world.get_component_mut::<Smell>(v) {
                    // perfume lingers under the work-smell but doesn't grow
                    if s.kind != "roses" {
                        s.intensity = (s.intensity + 0.15).min(1.0);
                        s.kind = kind;
                    }
                } else {
                    self.world.add_component(v, Smell { intensity: 0.15, kind });
                }
            } else if let Some(s) = self.world.get_component_mut::<Smell>(v) {
                if s.kind == "roses" { s.intensity = (s.intensity - 0.02).max(0.0); }
            }
        }
    }

    /// What they smell of, if anything worth mentioning
    pub fn smells_of(&self, v: Entity) -> Option<&'static str> {
        self.world.get_component::<Smell>(v)
            .filter(|s| s.intensity > 0.6).map(|s| s.kind)
    }

    /// The young grow: each day the kids age, and on the day they come of
    /// age they take up a trade - the master they ran the most errands
    /// for, else whatever trade the village can use
    pub fn lifecycle_tick(&mut self, tick: u64) {
        let mut growing = self.villagers.clone();
        // the seats' wanderers grow too - a picked childhood isn't
        // forever; an agent's seat is a villager already counted above
        for s in &self.seats {
            if !s.agent { growing.push(s.player); }
        }
        for &v in growing.iter() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(g) = self.world.get_component_mut::<KidGrowth>(v) else { continue };
            g.age_days += 1;
            if g.age_days < vocation::COMING_OF_AGE { continue; }
            let errands = g.errands.clone();
            self.world.remove_component::<Kid>(v);
            self.world.remove_component::<KidGrowth>(v);
            // grown, the picked figure is a grown one too
            if let Some(a) = self.world.get_component_mut::<crate::game::player::Avatar>(v) {
                a.kid = false;
            }
            // the hirer they ran for most is their likeliest master
            let master = errands.iter().max_by_key(|(_, n)| *n).map(|(e, _)| *e)
                .filter(|&m| !villager::is_dead(&self.world, m)
                    && self.world.get_component::<Craft>(m).is_some());
            let voc = master.and_then(|m| self.world.get_component::<Craft>(m).map(|c| c.vocation))
                .unwrap_or_else(|| roll_vocation(&mut self.rng));
            let mut c = vocation::craft(voc, 0.15);
            c.master = master;
            c.tasked_by = master;
            self.world.add_component(v, c);
            let name = self.world.get_component::<PlayerCharacter>(v).map(|p| p.name.clone())
                .or_else(|| self.villagers.iter().position(|x| *x == v).map(|i| vill_name(i).to_string()))
                .unwrap_or_else(|| "a youngster".into());
            let trade = vocation::vocation_name(voc).to_lowercase();
            self.log.push(format!("t{}: {} takes up the {}.", tick, name, trade));
            self.toasts.push_at((format!("{} comes of age - a {}!", name, trade), tick + 30),
                toast_spot(&self.world, v));
            self.celebrate_person(tick, v, &format!("{} comes of age", name));
        }
        // and the years keep count: a day of the village is about a week
        // of life (a youngster's 730 days is their fourteenth year);
        // when the years outrun the span they were given, the village
        // buries them
        for (i, &v) in self.villagers.iter().enumerate().collect::<Vec<_>>() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(a) = self.world.get_component_mut::<crate::game::lifecycle::Age>(v) else { continue };
            a.years += 1.0 / 52.0;
            a.stage = crate::game::lifecycle::stage_for(a.years);
            let years = a.years;
            let span = self.world.get_component::<crate::game::lifecycle::Vitality>(v)
                .map(|x| x.max_age).unwrap_or(f32::MAX);
            if years >= span {
                if let Some(h) = self.world.get_component_mut::<Health>(v) {
                    h.condition = 0.0;
                } else {
                    self.world.add_component(v, villager::Dead);
                }
                if let Some(vx) = self.world.get_component_mut::<crate::game::lifecycle::Vitality>(v) {
                    vx.cause_of_death.get_or_insert_with(|| "old age".into());
                }
                self.log.push(format!("t{}: {} dies at {} winters, of old age.", tick,
                    vill_name(i), years.round() as u32));
            }
        }
        // the dead give up their beds - but not before the heirs have
        // read the house: a day's grace from the day they fell, then
        // the bed goes free
        let dead: Vec<(usize, Entity)> = self.villagers.iter().enumerate()
            .filter(|&(_, &v)| villager::is_dead(&self.world, v))
            .map(|(i, &v)| (i, v)).collect();
        for &(i, d) in &dead {
            // the true death day, when the roster knows it - an event
            // late in the day shouldn't hold the bed an extra day
            match self.died_on.get(i).and_then(|x| *x) {
                Some(d0) => { self.dead_since.insert(d, d0); }
                None => { self.dead_since.entry(d).or_insert(tick); }
            }
        }
        let evict: Vec<Entity> = dead.iter().map(|(_, d)| *d)
            .filter(|d| self.dead_since.get(d).copied().unwrap_or(tick) < tick).collect();
        self.dead_since.retain(|d, _| dead.iter().any(|(_, e)| e == d));
        if !evict.is_empty() {
            for b in self.world.query::<household::Building>() {
                if let Some(bd) = self.world.get_component_mut::<household::Building>(b) {
                    if bd.occupants.iter().any(|o| evict.contains(o)) {
                        bd.occupants.retain(|o| !evict.contains(o));
                    }
                }
            }
        }
    }

    /// The year's turn: when autumn comes round the org takes the
    /// harvest in together - a shared lift for the members' ties and
    /// the cause's own strength
    pub fn harvest_tick(&mut self, tick: u64) {
        let season = self.world.get_component::<crate::sim::climate::Climate>(self.climate)
            .map(|c| c.season).unwrap_or(crate::sim::climate::Season::Summer);
        let year = tick / 360; // SeasonSystem's own calendar: 90 ticks a season
        if season != crate::sim::climate::Season::Autumn || year == self.harvest_year { return; }
        self.harvest_year = year;
        for seat in 0..self.seats.len() {
            let oe = self.seats[seat].org;
            let took = crate::game::organizing::harvest_together(&mut self.world, oe);
            if took > 0 {
                let oname = self.world.get_component::<Organization>(oe)
                    .map(|o| o.name.clone()).unwrap_or_default();
                self.log.push(format!("t{}: {} takes the harvest in together - {} share the lift.", tick, oname, took));
            }
        }
    }

    /// Starving hurts, eating well mends the unsick - the day's beat of
    /// it, while weather keeps to its own floored drain in weather_tick
    pub fn nourish_tick(&mut self) {
        let sick: Vec<Entity> = self.world.query::<crate::sim::illness::Disease>().iter()
            .filter_map(|&d| self.world.get_component::<crate::sim::illness::Disease>(d)
                .map(|x| x.patient)).collect();
        crate::game::villager::nourishment_step(&mut self.world, &sick);
    }

    /// The fields ripen a little each day at the season's pace, and the
    /// first ears coming in are worth saying aloud once
    pub fn crop_tick(&mut self, tick: u64) {
        use crate::engine::ecs::System;
        use crate::game::villager::{Crop, FoodSource};
        crate::game::villager::CropGrowthSystem.run(&mut self.world);
        if self.crop_noted { return; }
        let grown: u32 = self.world.query::<Crop>().iter()
            .filter_map(|&e| self.world.get_component::<Crop>(e))
            .map(|c| c.grown).sum();
        if grown > 0 {
            self.crop_noted = true;
            let stored: u32 = self.world.query::<FoodSource>().iter()
                .filter_map(|&e| self.world.get_component::<FoodSource>(e))
                .map(|f| f.servings).sum();
            self.log.push(format!("t{}: the first ears come in - {} servings stand in store.", tick, stored));
        }
    }

    /// The rolls get updated daily: the dead leave them, each org's sway
    /// re-pools from its living members, a sit-down held yesterday
    /// deepens into a real tie, and a muscle unused goes soft
    pub fn influence_tick(&mut self, tick: u64) {
        use crate::engine::ecs::System;
        let before = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(
            self.seats[0].org).map(|p| p.capacity).unwrap_or(0.0);
        crate::sim::community::InfluenceSystem.run(&mut self.world);
        crate::sim::community::OneOnOneSystem.run(&mut self.world);
        crate::sim::organized_power::PowerAtrophySystem.run(&mut self.world);
        // the day the muscle's gone soft is worth saying aloud
        let after = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(
            self.seats[0].org).map(|p| p.capacity).unwrap_or(0.0);
        if before >= 0.05 && after < 0.05 {
            self.log.push(format!("t{}: the cause's muscle has gone soft - the habit of meeting is lost.", tick));
        }
    }

    /// Ties drift a little toward neutral each day - bad blood settles,
    /// warmth cools, and a label the tie outgrew comes off
    pub fn rel_tick(&mut self) {
        use crate::engine::ecs::System;
        crate::game::relationships::RelationshipSystem.run(&mut self.world);
    }

    /// The newly dead are mourned: a bond to them, or a tie worth
    /// grieving, takes a Loss to the mourner's attachment history - and
    /// the grief the village already knows how to carry
    pub fn grief_tick(&mut self, tick: u64) {
        use crate::sim::psychology::{Bond, Episode, EpisodeKind, Mourned, GRIEF_TIE, record_episode};
        use crate::game::relationships::{find_relationship, Relationship};
        // the roster and the seats' wanderers alike - the Wanderer's
        // bonds grieve like anyone's, and their death's mourned too
        let mut folk: Vec<Entity> = self.villagers.clone();
        for s in &self.seats {
            if !folk.contains(&s.player) { folk.push(s.player); }
        }
        let newly_dead: Vec<Entity> = folk.iter().copied()
            .filter(|&v| villager::is_dead(&self.world, v)
                && self.world.get_component::<Mourned>(v).is_none()).collect();
        for d in newly_dead {
            self.world.add_component(d, Mourned);
            let mourners: Vec<Entity> = folk.iter().copied().filter(|&m| m != d
                && !villager::is_dead(&self.world, m)
                && (self.world.get_component::<Bond>(m).map(|b| b.target == d).unwrap_or(false)
                    || find_relationship(&self.world, m, d)
                        .and_then(|r| self.world.get_component::<Relationship>(r))
                        .map(|r| r.strength >= GRIEF_TIE).unwrap_or(false))).collect();
            if mourners.is_empty() { continue; }
            let names: Vec<String> = mourners.iter().take(3)
                .map(|&m| self.name_of(m)).collect();
            if !names.is_empty() {
                self.log.push(format!("t{}: {} mourn {}.", tick, names.join(" and "), self.name_of(d)));
            }
            for m in mourners {
                record_episode(&mut self.world, m, Episode { with: d, kind: EpisodeKind::Loss });
                crate::game::desires::add_grievance(&mut self.world, m,
                    crate::game::desires::Problem::Bereaved, tick + 60);
            }
        }
    }

    /// The heart-helper's rounds: she keeps her corner of the market, and
    /// now and then a lonely heart is nudged to come and sit with her
    pub fn matchmaker_tick(&mut self, tick: u64) {
        let Some(mm) = self.matchmaker else { return };
        if tick % 4 != 0 || villager::is_dead(&self.world, mm) { return; }
        // a heart behind bars can't answer the heart-helper - skip it
        // for one that can, or the lock-up wastes the day's match
        if let Some(&a) = self.lonely_hearts().iter().find(|&&a| {
            a != mm && !crate::game::dungeon::unavailable(&self.world, a)
            && self.world.get_component::<Hunger>(a).map(|h| h.value >= 30.0).unwrap_or(true)
            && !self.travels.iter().any(|t| t.who == a)
        }) { self.send(a, Goal::TalkTo(mm)); }
    }

    /// Villagers carrying an unmet Want::Love and no sweetheart yet
    pub fn lonely_hearts(&self) -> Vec<Entity> {
        self.villagers.iter().copied().filter(|&v| {
            // the grave and the lock-up take no suitors - no bonds
            // written to a cell, no heart sent courting from one
            !crate::game::dungeon::unavailable(&self.world, v)
            // the roles keep their own rounds; a heart called to market
            // would never wander home again
            && self.world.get_component::<TownRole>(v).is_none()
            // and no one courts the shunned
            && !self.shunned(v)
            // grown, not a kid: kids have no craft
            && self.world.get_component::<Craft>(v).is_some()
            && self.world.get_component::<InnerLife>(v)
                .map(|l| l.wants.contains(&desires::Want::Love)
                    && !l.satisfied.iter().any(|(w, u)| *w == desires::Want::Love && *u > self.tick))
                .unwrap_or(false)
            && !self.has_rel(relationships::RelKind::Lover, v)
        }).collect()
    }

    /// Does `v` stand in any relationship of `kind`?
    pub fn has_rel(&self, kind: relationships::RelKind, v: Entity) -> bool {
        self.world.find::<relationships::Relationship>(|_, r| {
            r.kind == kind && (r.from == v || r.other == v)
        }).is_some()
    }

    /// The relationship standing between `a` and `b`, if any
    pub(in crate::session) fn rel_kind(&self, a: Entity, b: Entity) -> Option<relationships::RelKind> {
        relationships::find_relationship(&self.world, a, b)
            .and_then(|e| self.world.get_component::<relationships::Relationship>(e))
            .map(|r| r.kind)
    }

    pub(in crate::session) fn name_of(&self, e: Entity) -> String {
        self.villagers.iter().position(|&x| x == e).map(|i| vill_name(i).to_string())
            .unwrap_or_else(|| "someone".into())
    }

    /// Try to kindle something between `a` and another lonely heart.
    /// `broker` pockets the fee if a match catches; `who` is who the log
    /// credits - "the heart-helper", or "you" when the player brokers it
    pub fn match_attempt(&mut self, a: Entity, tick: u64, broker: Option<Entity>, who: &str) {
        let tried = |x: Entity, y: Entity| self.match_tried.iter()
            .any(|&(p, q, until)| ((p == x && q == y) || (p == y && q == x)) && until > tick);
        let bs: Vec<Entity> = self.lonely_hearts().into_iter()
            .filter(|&b| b != a && !tried(a, b)
                && !matches!(self.rel_kind(a, b), Some(relationships::RelKind::Family | relationships::RelKind::Lover)))
            .collect();
        let Some(&b) = bs.first() else {
            self.say(a, "No heart left to mend - not today.", TALK_BUBBLE_SECS);
            return;
        };
        // the roll: shared wants draw folks together; feuded blood works against
        let shared = self.world.get_component::<InnerLife>(a).map(|la| la.wants.iter()
            .filter(|w| self.world.get_component::<InnerLife>(b).map(|lb| lb.wants.contains(w)).unwrap_or(false))
            .count()).unwrap_or(0);
        let feuded = self.rel_kind(a, b) == Some(relationships::RelKind::Rival);
        // and smell tells on a suitor: "she smells of the fold" - a dab
        // of perfume, though, is no smell at all
        let smelly = [a, b].iter().any(|&v|
            self.smells_of(v).map(|k| k != "roses").unwrap_or(false));
        if smelly { self.say(a, "...you smell of the work, friend.", CHAT_BUBBLE_SECS); }
        let spark = 0.5 + shared as f32 * 0.1 - if feuded { 0.15 } else { 0.0 }
            - if smelly { 0.15 } else { 0.0 };
        // a deterministic spark per pair and month - no rng spent, so the
        // same village always courts the same way
        let roll = ((a as u64).wrapping_mul(73856093) ^ (b as u64).wrapping_mul(19349663)
            ^ (tick / 30).wrapping_mul(83492791)) % 1000;
        if (roll as f32) / 1000.0 < spark {
            for (x, y) in [(a, b), (b, a)] {
                relationships::add_relationship(&mut self.world, relationships::Relationship {
                    from: x, other: y, kind: relationships::RelKind::Lover, strength: 0.6, interactions: 1 });
            }
            desires::satisfy(&mut self.world, a, desires::Want::Love, tick + 365);
            desires::satisfy(&mut self.world, b, desires::Want::Love, tick + 365);
            if let Some(pe) = broker {
                if let Some(p) = self.world.get_component_mut::<Purse>(pe) { p.gold += 2.0; }
            }
            let (an, bn) = (self.name_of(a), self.name_of(b));
            self.log.push(format!("t{}: {} brought {} and {} together{}.", tick, who, an, bn,
                if feuded { " - a match the families won't like" } else { "" }));
            // a brokered match is power shown in the street
            if let Some(pe) = broker {
                if pe == self.player { self.record_power(crate::sim::organized_power::ActionQuality::Transactional); }
            }
            self.say(a, "Come, I want you to meet someone.", TALK_BUBBLE_SECS);
        } else {
            self.match_tried.push((a, b, tick + 30));
            self.say(a, "Perhaps another heart.", TALK_BUBBLE_SECS);
            // a cold answer can start bad blood: the saga's way - unless
            // the two are already at feud over something else
            let bad_blood = self.world.query::<crate::game::feuds::Feud>().into_iter().any(|e| {
                let f = self.world.get_component::<crate::game::feuds::Feud>(e).unwrap();
                f.status != crate::game::feuds::FeudStatus::Resolved
                    && ((f.offender == a && f.aggrieved == b) || (f.offender == b && f.aggrieved == a))
            });
            if !bad_blood {
                crate::game::feuds::spawn_feud(&mut self.world, b, a, "a turned-down suit");
                let (an, bn) = (self.name_of(a), self.name_of(b));
                self.log.push(format!("t{}: {} won't forget {}'s cold answer - bad blood now.", tick, an, bn));
            }
        }
    }
}
