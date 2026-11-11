//! The wood: the herb-wife's round, wild mushrooms and the lore of which ones kill, and the slow ailments (the amatoxin course, ergot).

use crate::session::*;

impl Game {
    /// The safety note from the lit review: real traits in-world carry
    /// the disclaimer, once, where folk will read it
    pub(in crate::session) fn disclaim_traits_once(&mut self) {
        if !self.shroom_disclaimed {
            self.shroom_disclaimed = true;
            self.log.push(mycology::FORAGING_DISCLAIMER.to_string());
        }
    }

    /// The cap tells its story - or doesn't. The read decides what they
    /// THINK it is: a skilled eye sees the true kind, a hungry one sees
    /// the lookalike, and folk rules ("a poisoned cap blackens silver")
    /// can clear a death outright. They eat what they believe is food or
    /// dream; the effect that lands is always the true cap's
    /// Belly or satchel: the eat feeds a hungered forager, and sacks a
    /// cap for the one who can't hunger - the player keeps what he picks
    fn feed_or_sack(&mut self, who: Entity, food: f32) {
        if let Some(h) = self.world.get_component_mut::<Hunger>(who) {
            h.value = (h.value + food).min(100.0);
        } else if food > 0.0 {
            if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                *s.goods.entry("mushrooms".to_string()).or_insert(0) += 1;
            }
        }
    }

    /// A turned belly: villagers take a gut disease; the player, who
    /// carries no Health for one to work on, takes the mark instead -
    /// it slows their walking and shows on their card
    fn gut_turn(&mut self, who: Entity, name: &str, severity: f32) {
        if self.world.get_component::<crate::sim::illness::Health>(who).is_some() {
            let d = self.world.create_entity();
            let mut gut = Disease::new(DiseaseKind::Gripes, who, severity);
            gut.name = name.into();
            gut.incubation = 0;
            gut.stage = Stage::Mild;
            self.world.add_component(d, gut);
        } else {
            // a belly already turning keeps the worse of the two - the
            // longer until, and the harsher name if the marks differ
            let (kind, until) = self.world.get_component::<villager::Condition>(who)
                .filter(|c| self.tick < c.until)
                .map(|c| (c.kind, c.until.max(self.tick + 2)))
                .unwrap_or(("a turned gut", self.tick + 2));
            self.world.add_component(who, villager::Condition { kind, until });
        }
    }

    pub(in crate::session) fn shroom_take(&mut self, who: Entity, pi: usize, kind: MushKind, tick: u64) {
        let spec = kind.spec();
        let skilled = self.world.get_component::<Craft>(who).map(|c| c.skill).unwrap_or(0.2);
        // the learned eye never misreads a patch it knows
        let known = self.lore_of(who).contains(&pi);
        let roll = (who as u64).wrapping_mul(2654435761).wrapping_add(tick) % 1000;
        let right = known || roll < mycology::read_chance(skilled);
        // folk rule: "the silver stayed bright" - confident, and wrong
        let folk = !right && roll % 10 < 3;
        let believed = if right { kind }
            else if folk { MushKind::FieldMushroom }
            else { spec.lookalike };
        if !believed.believed_edible() {
            // they leave what they think is a gut-turn or death
            let line = if right { spec.trait_line } else { "Off-colour gills. Leave them." };
            if right { self.disclaim_traits_once(); }
            self.say(who, line, TALK_BUBBLE_SECS);
            return;
        }
        if right {
            self.say(who, spec.eat_line, TALK_BUBBLE_SECS);
        } else if folk {
            self.say(who, "The silver stayed bright - it's good.", TALK_BUBBLE_SECS);
        } else {
            self.say(who, believed.spec().eat_line, TALK_BUBBLE_SECS);
        }
        // they eat the TRUE cap, whatever they believe it is
        match spec.effect {
            MushEffect::Meal => {
                self.feed_or_sack(who, spec.food);
            }
            MushEffect::TurnsStomach => {
                // the jack-o'-lantern's lesson: a sour night, and they
                // mark the patch after
                self.feed_or_sack(who, spec.food * 0.5);
                self.gut_turn(who, "a sour gut", 0.15);
                self.learn_shroom(who, pi);
            }
            MushEffect::Vision => {
                self.feed_or_sack(who, spec.food);
                // the field speaks - most keep the gift, some take a dark turn
                if self.shroom_rng.below(100) < 20 {
                    self.gut_turn(who, "a fearful turn", 0.18);
                    self.say(who, "The field opened its mouth.", CHAT_BUBBLE_SECS);
                } else {
                    desires::satisfy(&mut self.world, who, Want::Joy, tick + 30);
                }
                // dangerous knowledge: the priest calls the wood's counsel
                // devil's work, when word of it gets about
                if self.shroom_rng.below(100) < 8 {
                    if let Some(pr) = self.villagers.iter().copied().find(|&v| {
                        self.world
                            .get_component::<Craft>(v)
                            .map(|c| c.vocation == Vocation::Priest)
                            .unwrap_or(false)
                            && !crate::game::dungeon::unavailable(&self.world, v)
                    }) {
                        self.say(
                            pr,
                            "The wood's counsel is the devil's work.",
                            CHAT_BUBBLE_SECS,
                        );
                        self.log_entity_ev(
                            EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                            pr,
                            format!("t{}: the father preaches against the little caps.", tick),
                        );
                    }
                }
            }
            MushEffect::Delirium => {
                // the red cap's dream: wild, rarely fatal - the fed take
                // it on purpose, the hungry learn why folk leave it
                self.feed_or_sack(who, spec.food * 0.5);
                self.gut_turn(who, "the red cap's dream", 0.28);
                desires::satisfy(&mut self.world, who, Want::Joy, tick + 10);
            }
            MushEffect::Amatoxin => {
                // nothing, at first. The belly lands a day on, when the
                // meal's half forgotten - then the false mend, then the
                // liver. The player can't die of it - his gut turns
                // instead, and the patch's lesson lands hard
                self.feed_or_sack(who, spec.food);
                if self.world.get_component::<crate::sim::illness::Health>(who).is_some() {
                    self.ailments.push(PendingAil { who, due: tick + 1, phase: AilPhase::ShroomBelly(pi) });
                } else {
                    let until = self.world.get_component::<villager::Condition>(who)
                        .filter(|c| tick < c.until)
                        .map(|c| c.until.max(tick + 4)).unwrap_or(tick + 4);
                    self.world.add_component(who, villager::Condition { kind: "gut-poisoned", until });
                    self.learn_shroom(who, pi);
                    self.say(who, "Your belly knows what the wood did to it.", CHAT_BUBBLE_SECS);
                }
            }
        }
    }

    /// The herb-wife's day: brew what the hedge gave into remedies while
    /// there's stock, or walk out to a ready patch when it runs low
    pub fn herbologist_tick(&mut self, tick: u64) {
        let (Some(her), Some(hut)) = (self.herbologist, self.herb_hut) else { return };
        if villager::is_dead(&self.world, her) || self.travels.iter().any(|t| t.who == her) { return; }
        // keep a few cures on the shelf while there's a raw herb to brew
        if herbology::remedies_on_hand(&self.world, her) < 5
            && herbology::brew_from_stock(&mut self.world, her, hut).is_some() { return; }
        // shelves and stocks both thin? out to the nearest ready patch.
        // Only what she can brew counts toward stocked - a tincture herb
        // without the spirit to steep it is stock, not remedy, and
        // shouldn't keep her home from the picking. But when there's
        // nothing ready to pick, stocked or not, she walks home fed
        let spirits = self.world.get_component::<Building>(hut)
            .map(|b| b.stock.get("spirits").copied().unwrap_or(0) > 0).unwrap_or(false);
        let raw: u32 = self.world.get_component::<Building>(hut)
            .map(|b| herbology::medicinal_herbs().iter()
                .filter(|(_, prep, _)| *prep != herbology::PrepMethod::Tincture || spirits)
                .map(|(n, _, _)| b.stock.get(*n).copied().unwrap_or(0)).sum()).unwrap_or(0);
        if raw < 6 {
            let hp = self.world.get_component::<Pos>(hut).copied();
            // nearest ready patch first, and the next if she can't walk
            // it - a tuft she can't reach keeps no wise woman in town
            let mut picks: Vec<(usize, i32)> = self.herb_patches.iter().enumerate()
                .filter(|(_, p)| tick >= p.next_ready)
                .map(|(i, p)| (i,
                    hp.map(|h| ((h.x - p.tile.0 as f32).abs() + (h.y - p.tile.1 as f32).abs()) as i32)
                        .unwrap_or(0)))
                .collect();
            picks.sort_by_key(|(_, d)| *d);
            for (i, _) in picks {
                if self.send_forage(her, i) { return; }
            }
        }
        // stocked up or nothing pickable - she walks home at the end of
        // the day rather than camp by the last patch, but not hungry,
        // or the walk west fights the pull to food
        let fed = self.world.get_component::<Hunger>(her).map(|h| h.value >= 30.0).unwrap_or(true);
        if fed {
            if let (Some(hp), Some(p)) = (self.world.get_component::<Pos>(hut).copied(),
                                          self.world.get_component::<Pos>(her).copied()) {
                let far = (p.x - hp.x).abs() + (p.y - hp.y).abs() > 4.0;
                if far { self.send(her, Goal::HeadHome(hut)); }
            }
        }
    }

    /// The patch indices `who` has learned are death - from the belly's
    /// own lesson, or the herb-wife's teaching
    pub(in crate::session) fn lore_of(&self, who: Entity) -> &[usize] {
        self.shroom_lore.iter().find(|&(w, _)| *w == who)
            .map(|(_, l)| l.as_slice()).unwrap_or(&[])
    }

    /// `who` marks patch `pi` as death: they won't gather it again, and
    /// it's lore they can pass on
    pub(in crate::session) fn learn_shroom(&mut self, who: Entity, pi: usize) {
        let known = &mut self.shroom_lore;
        match known.iter_mut().find(|(w, _)| *w == who) {
            Some((_, l)) if !l.contains(&pi) => l.push(pi),
            Some(_) => {}
            None => known.push((who, vec![pi])),
        }
    }

    /// How wet this year's harvest ran: ergot loves a sodden year on the
    /// rye. The mean of the thirty days before the rye comes in - absolute
    /// days, so a wet year reads wet and a dry one dry
    pub(in crate::session) fn harvest_wetness(&self, tick: u64) -> f32 {
        let lo = tick.saturating_sub(30);
        (lo..=tick).map(|d| climate::wetness(Season::Autumn, d as u32)).sum::<f32>() / (tick - lo + 1) as f32
    }

    /// A poor soul gathers in the wood when the larder or the purse runs
    /// light; and a damp year brings Saint Anthony's fire through the rye
    pub fn mushroom_tick(&mut self, tick: u64) {
        if tick % 3 == 0 {
            // a hungry soul gathers in the wood; the fed stay home - and
            // the learned stay clear of the caps they know are death
            let pick = self.mush_patches.iter().enumerate()
                .filter(|(_, p)| tick >= p.next_ready)
                .find_map(|(pi, _)| {
                    self.villagers.iter().copied().find(|&v| {
                        // the roles keep their own rounds - a hungry herb-wife
                        // would never come back from the wood
                        !crate::game::dungeon::unavailable(&self.world, v) && !self.travels.iter().any(|t| t.who == v)
                        && self.world.get_component::<TownRole>(v).is_none()
                        && self.world.get_component::<Hunger>(v).map(|h| h.value < 40.0).unwrap_or(false)
                        && !self.shroom_breaks.iter().any(|&(w, d)| w == v && tick < d + 5)
                        && !self.lore_of(v).contains(&pi)
                    }).map(|v| (pi, v))
                });
            if let Some((pi, v)) = pick { self.send_shroom(v, pi); }
        }
        // the wood teaches: the herb-wife tells a passing neighbour which
        // caps to leave, a patch at a time
        if tick % 25 == 7 { self.teach_shrooms(tick); }
        // the amatoxin course keeps its own calendar: belly, false mend, liver
        crate::timed!("sim.ailment", self.ailment_tick(tick));
        // a damp year: the rye carries the fire and the bread gets the blame.
        // The wetter the harvest the surer the outbreak
        let chance = (self.harvest_wetness(tick) * 30.0) as i32;
        if tick % 365 == 210 && self.rng.below(100) < chance { self.ergot(tick); }
    }

    /// The herb-wife passes her wood-lore on: any neighbour standing near
    /// learns one deadly patch they didn't know. An elder forager is worth
    /// having - this is what she's for
    pub(in crate::session) fn teach_shrooms(&mut self, _tick: u64) {
        let Some(her) = self.herbologist else { return };
        if villager::is_dead(&self.world, her) { return; }
        let Some(hp) = self.world.get_component::<Pos>(her).copied() else { return };
        let deadly: Vec<usize> = self.mush_patches.iter().enumerate()
            .filter(|(_, p)| p.deadly()).map(|(i, _)| i).collect();
        for &v in self.villagers.clone().iter() {
            if v == her || villager::is_dead(&self.world, v) { continue; }
            let near = self.world.get_component::<Pos>(v)
                .map(|p| (p.x - hp.x).abs() + (p.y - hp.y).abs() < 6.0).unwrap_or(false);
            if !near { continue; }
            if let Some(&pi) = deadly.iter().find(|pi| !self.lore_of(v).contains(pi)) {
                self.learn_shroom(v, pi);
                self.say(her, "The pale cap by the wood is death - mark it.", CHAT_BUBBLE_SECS);
            }
        }
    }

    /// The ailments that keep their own calendar: the amatoxin course
    /// lands in phases days after the meal, and the brothers of Saint
    /// Anthony make their rounds after the fire's named
    pub(in crate::session) fn ailment_tick(&mut self, tick: u64) {
        let due: Vec<PendingAil> = self.ailments.iter().copied()
            .filter(|a| tick >= a.due).collect();
        self.ailments.retain(|a| tick < a.due);
        for a in due {
            if villager::is_dead(&self.world, a.who) { continue; }
            match a.phase {
                // the belly lands a day on, when the meal's half forgotten -
                // the poison's already at work while they blame last night's
                // pottage
                AilPhase::ShroomBelly(pi) => {
                    let d = self.world.create_entity();
                    let mut belly = Disease::new(DiseaseKind::Gripes, a.who, 0.4);
                    belly.name = "a bad belly".into();
                    belly.incubation = 0; // the ailments queue keeps this calendar
                    belly.stage = Stage::Moderate;
                    self.world.add_component(d, belly);
                    self.say(a.who, "Something I ate. It'll pass.", CHAT_BUBBLE_SECS);
                    // the belly tells on the patch: this one's learned
                    self.learn_shroom(a.who, pi);
                    self.ailments.push(PendingAil { who: a.who, due: tick + 2, phase: AilPhase::ShroomRemission });
                }
                // the false mend: they seem to turn the corner - the liver's
                // already failing. This is the apothecary's window, the only
                // one they get
                AilPhase::ShroomRemission => {
                    for d in self.world.query::<Disease>() {
                        let hit = self.world.get_component::<Disease>(d)
                            .map(|dd| dd.patient == a.who && dd.name == "a bad belly").unwrap_or(false);
                        if hit {
                            let dd = self.world.get_component_mut::<Disease>(d).unwrap();
                            dd.severity = 0.05;
                            dd.stage = Stage::Recovering;
                        }
                    }
                    self.say(a.who, "It's passing. I'll be right by Friday.", CHAT_BUBBLE_SECS);
                    self.ailments.push(PendingAil { who: a.who, due: tick + 4, phase: AilPhase::ShroomLiver });
                }
                // the liver gives out, days after the meal - few walk back
                // from this. Unless the herb-wife turned it: a belly she
                // cleared in the window stays clear
                AilPhase::ShroomLiver => {
                    let still_sick = self.world.query::<Disease>().iter().any(|&d|
                        self.world.get_component::<Disease>(d)
                            .map(|dd| dd.patient == a.who && dd.name == "a bad belly").unwrap_or(false));
                    if !still_sick { continue; }
                    let d = self.world.create_entity();
                    let mut liver = Disease::new(DiseaseKind::Gripes, a.who, 0.85);
                    liver.name = "the liver gives out".into();
                    liver.incubation = 0;
                    liver.stage = Stage::Severe;
                    self.world.add_component(d, liver);
                    let name = self.name_of(a.who);
                    self.log_entity_ev(
                        EvTag::new(Thread::Illness, Valence::Negative, Sev::Notable),
                        a.who,
                        format!(
                            "t{}: {} wastes where they stand - the wood keeps its secrets.",
                            tick, name
                        ),
                    );
                }
                // the brothers of Saint Anthony tend the fire's victims:
                // a hospice bed and a dose of care
                AilPhase::AntonineCare => {
                    for d in self.world.query::<Disease>() {
                        let hit = self.world.get_component::<Disease>(d)
                            .map(|dd| dd.patient == a.who && dd.name.contains("fire")).unwrap_or(false);
                        if hit {
                            let dd = self.world.get_component_mut::<Disease>(d).unwrap();
                            dd.severity = (dd.severity - 0.12).max(0.05);
                        }
                    }
                }
            }
        }
    }

    /// Saint Anthony's fire: folk sicken in the convulsive and the burning
    /// forms, the village finds someone to blame, and the brothers of
    /// Saint Anthony keep a hospice bed for the victims
    pub(in crate::session) fn ergot(&mut self, tick: u64) {
        // the wetter the harvest the wider the fire: a sodden year's rye
        // carries more of it
        let victims = 2 + if self.harvest_wetness(tick) > 0.45 { 2 } else { 0 };
        for &v in self.villagers.iter().take(victims) {
            if !villager::is_dead(&self.world, v) {
                // most take the convulsive fire; some the burning one -
                // the gangrenous form is rarer and crueller
                let gangrenous = self.shroom_rng.below(100) < 25;
                let (name, stage, severity) = if gangrenous {
                    ("the burning fire", Stage::Severe, 0.7)
                } else {
                    ("the convulsive fire", Stage::Moderate, 0.45)
                };
                let d = self.world.create_entity();
                let mut fire = Disease::new(DiseaseKind::Ergotism, v, severity);
                fire.name = name.into();
                fire.stage = stage;
                self.world.add_component(d, fire);
                // the brothers make their rounds: a dose of care in the
                // days after the fire's named, if there's a church to
                // house them
                if self.church.is_some() {
                    self.ailments.push(PendingAil { who: v, due: tick + 4, phase: AilPhase::AntonineCare });
                }
            }
        }
        self.log.push_ev(
            EvTag::new(Thread::Illness, Valence::Negative, Sev::Notable),
            format!(
                "t{}: a burning fire runs through the village - they say it's in the bread.",
                tick
            ),
        );
        if let Some(ch) = self.church {
            self.log.push_ev_at_who(
                EvTag::new(Thread::Illness, Valence::Positive, Sev::Minor),
                format!(
                    "t{}: the brothers of Saint Anthony take in the burned.",
                    tick
                ),
                toast_spot(&self.world, ch),
                None,
            );
        }
        // who carries the blame: the baker if there is one; else the
        // wood's counsel - the herb-wife's - is the nearest dark thing;
        // else some poor soul the finger finds
        let baker = self.villagers.iter().copied().find(|&v|
            self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Baker).unwrap_or(false)
            && !crate::game::dungeon::unavailable(&self.world, v));
        let scapegoat = baker.or_else(|| {
            if self.shroom_rng.below(100) < 50 { self.herbologist.filter(|&h| !crate::game::dungeon::unavailable(&self.world, h)) }
            else { self.villagers.iter().copied().find(|&v| !crate::game::dungeon::unavailable(&self.world, v)) }
        });
        if let Some(b) = scapegoat {
            desires::add_grievance(&mut self.world, b, desires::Problem::Blamed, tick + 90);
            let line = if Some(b) == baker { "They say my bread did it." }
                else if Some(b) == self.herbologist { "They say my cures are curses." }
                else { "They look at me like I brought it." };
            self.say(b, line, CHAT_BUBBLE_SECS);
        }
        // the priest's reading: a judgment, a curse, a sermon waiting
        if let Some(pr) = self.villagers.iter().copied().find(|&v|
            self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
            && !crate::game::dungeon::unavailable(&self.world, v)) {
            if self.shroom_rng.below(100) < 60 {
                self.say(pr, "A judgment on the village - repent.", CHAT_BUBBLE_SECS);
            }
        }
    }
}
