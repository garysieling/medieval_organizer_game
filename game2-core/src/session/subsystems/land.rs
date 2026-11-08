//! The land and the sky: worn trails, the land's memory (canopy, the pit, effluent), fire, sky events and the harsh winter.

use crate::session::*;

impl Game {
    /// The ground greens over: each trail tile decays toward wild unless
    /// feet keep it open
    pub fn trail_tick(&mut self) {
        self.trails.retain(|_, t| {
            trails::step_trail(t, 0, 0.0, &TRAIL_PARAMS);
            t.strength > 0.01
        });
    }

    /// The land keeps score: canopy regrows only where nothing disturbs
    /// it, the reach carries the leavings downstream, and past a
    /// threshold the land's grievances land on the folk who feel them -
    /// all commons, the troubles everyone shares
    pub fn land_tick(&mut self, tick: u64) {
        // regrowth is for undisturbed ground: a trampled or built tile
        // never gets its canopy back
        for (&t, c) in self.land.tree_cover.iter_mut() {
            let trampled = self.trails.get(&t).map(|tr| tr.strength > 0.1).unwrap_or(false);
            if *c > 0.3 && *c < 1.0 && !trampled && !self.map.is_occupied(t.0, t.1) {
                *c = (*c + 0.005).min(1.0);
            }
        }
        // famine is the village's own measure - when two in five bellies
        // are hollow the leavings pile faster and the water's drawn anyway
        let famine = self.village_famine();
        // the reach: the town's leavings settle in downstream water
        if tick % 3 == 0 {
            let gain = if famine { 0.008 } else { 0.004 };
            for t in self.reach.clone() {
                let e = self.land.effluent.entry(t).or_insert(0.0);
                *e = (*e + gain).min(1.0);
            }
        }
        // but the river washes a little clean each day - foul water
        // clears once the village stops pouring in
        for e in self.land.effluent.values_mut() { *e = (*e - 0.0008).max(0.0); }
        // the thresholds - each lands on the folk who feel it, as commons
        let n = self.land.tree_cover.len().max(1) as f32;
        let mean_cover: f32 = self.land.tree_cover.values().sum::<f32>() / n;
        if tick % 7 == 2 {
            if mean_cover < 0.55 {
                for v in self.villagers.clone() {
                    let wood_worker = self.world.get_component::<Craft>(v)
                        .map(|c| matches!(c.vocation, Vocation::Toolmaker | Vocation::Luthier)).unwrap_or(false);
                    if wood_worker && !villager::is_dead(&self.world, v) {
                        desires::add_grievance(&mut self.world, v, desires::Problem::ThinWood, tick + 30);
                    }
                }
            }
            if self.land.pit_water >= 0.7 {
                for v in self.villagers.clone() {
                    let pit_worker = self.world.get_component::<Craft>(v)
                        .map(|c| matches!(c.vocation, Vocation::Mason | Vocation::Toolmaker)).unwrap_or(false);
                    if pit_worker && !villager::is_dead(&self.world, v) {
                        desires::add_grievance(&mut self.world, v, desires::Problem::DryPit, tick + 30);
                    }
                }
            }
            // a home within sniffing distance of the foul reach feels it
            let foul: Vec<(i32, i32)> = self.land.effluent.iter()
                .filter(|(_, &e)| e > 0.4).map(|(&t, _)| t).collect();
            if !foul.is_empty() {
                for v in self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    let near = household::home_of(&self.world, v)
                        .and_then(|h| self.world.get_component::<Pos>(h).copied())
                        .map(|hp| foul.iter().any(|&(x, y)|
                            (x as f32 - hp.x).abs() + (y as f32 - hp.y).abs() < 6.0))
                        .unwrap_or(false);
                    if near {
                        desires::add_grievance(&mut self.world, v, desires::Problem::StinkingReach, tick + 30);
                    }
                }
            }
            // foul water does worse than stink - the village drinks the
            // reach, and the reach gives back what the village poured in.
            // Each waterborne case fouls it further: Snow's loop
            for v in self.villagers.clone() {
                if villager::is_dead(&self.world, v) { continue; }
                let home_pos = household::home_of(&self.world, v)
                    .and_then(|h| self.world.get_component::<Pos>(h).copied());
                // the waterborne sick pour it back - the foulest they
                // touch is the tile of the reach nearest home
                let water_sick = self.world.query::<Disease>().iter().any(|&d| {
                    self.world.get_component::<Disease>(d)
                        .map(|dd| dd.patient == v && dd.kind.route() == illness::Route::Water
                            && dd.stage != Stage::Recovering)
                        .unwrap_or(false)
                });
                if water_sick {
                    if let Some(hp) = home_pos {
                        if let Some(&(x, y)) = self.reach.iter()
                            .min_by_key(|&&(x, y)| ((x as f32 - hp.x).abs() + (y as f32 - hp.y).abs()) as i32) {
                            let e = self.land.effluent.entry((x, y)).or_insert(0.0);
                            *e = (*e + 0.01).min(1.0);
                        }
                    }
                }
                // already carrying something - the water waits its turn
                if disease_of(&self.world, v).is_some() { continue; }
                let Some(hp) = home_pos else { continue };
                let load = self.land.effluent.iter()
                    .filter(|(&(x, y), &e)| e >= 0.5
                        && (x as f32 - hp.x).abs() + (y as f32 - hp.y).abs() < 6.0)
                    .map(|(_, &e)| e).fold(0.0f32, f32::max);
                if load >= 0.5 && self.rng.below(1000) < (load * if famine { 120.0 } else { 60.0 }) as i32 {
                    // fouler water carries the fouler fevers
                    // the foul water carries its fevers when their
                    // season's on them - out of it, the well holds
                    let day = (tick % 365) as u32;
                    let kind = if load >= 0.8 && self.rng.below(100) < 40
                        && DiseaseKind::Cholera.in_season(day) { DiseaseKind::Cholera }
                        else if load >= 0.6 && self.rng.below(100) < 30
                            && DiseaseKind::Typhoid.in_season(day) { DiseaseKind::Typhoid }
                        else if DiseaseKind::Dysentery.in_season(day) { DiseaseKind::Dysentery }
                        else { continue };
                    let severity = match kind {
                        DiseaseKind::Cholera => 0.55 + load * 0.3,
                        DiseaseKind::Typhoid => 0.45 + load * 0.2,
                        _ => 0.3 + load * 0.2,
                    };
                    let d = self.world.create_entity();
                    self.world.add_component(d, Disease::new(kind, v, severity));
                }
            }
            // and the unburied dead foul the water the same way - each
            // corpse's leavings settle in the reach nearest where they
            // fell; the churchyard's interred stay out of it
            for &v in self.villagers.clone().iter() {
                if !villager::is_dead(&self.world, v) { continue; }
                if self.world.get_component::<villager::Buried>(v).is_some() { continue; }
                if self.world.get_component::<country::Outlander>(v).is_some() { continue; }
                let Some(cp) = self.world.get_component::<Pos>(v).copied() else { continue };
                if let Some(&(x, y)) = self.reach.iter()
                    .min_by_key(|&&(x, y)| ((x as f32 - cp.x).abs() + (y as f32 - cp.y).abs()) as i32) {
                    let e = self.land.effluent.entry((x, y)).or_insert(0.0);
                    *e = (*e + 0.01).min(1.0);
                }
            }
            // the slow kinds find their season's opening: consumption
            // rides winter's damp indoors, the sweats the late-summer
            // heat - a rare roll in their window brings one through
            let day = (tick % 365) as u32;
            for (kind, chance) in [(DiseaseKind::Consumption, 8),
                (DiseaseKind::Sweats, 6)] {
                if kind.in_season(day) && self.rng.below(1000) < chance {
                    if let Some(&v) = self.villagers.iter().find(|&&v|
                        !villager::is_dead(&self.world, v) && disease_of(&self.world, v).is_none()) {
                        let d = self.world.create_entity();
                        self.world.add_component(d, Disease::new(kind, v, 0.4));
                    }
                }
            }
            // the ague rides the autumn air off the marshes: homes by the
            // river shake with it when the season turns, and it keeps
            // coming back - beating it barely teaches the body
            if let Some((s0, s1)) = DiseaseKind::Ague.season() {
                if (s0 as u64..=s1 as u64).contains(&(tick % 365)) {
                    for v in self.villagers.clone() {
                        if villager::is_dead(&self.world, v) || disease_of(&self.world, v).is_some() { continue; }
                        let Some(hp) = household::home_of(&self.world, v)
                            .and_then(|h| self.world.get_component::<Pos>(h).copied()) else { continue };
                        let marshy = (0..self.map.height as i32)
                            .flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                            .any(|(x, y)| self.map.get(x, y) == Some(Tile::River)
                                && (x as f32 - hp.x).abs() + (y as f32 - hp.y).abs() < 8.0);
                        if marshy && self.rng.below(1000) < 30 {
                            let d = self.world.create_entity();
                            self.world.add_component(d, Disease::new(DiseaseKind::Ague, v, 0.25));
                        }
                    }
                }
            }
            // the frightened look for a culprit: once sickness is abroad,
            // blame lands on a plausible-but-wrong target - the herb-wife
            // and her cures, or whoever's newest to the village
            let symptomatic = self.villagers.iter().filter(|&&v| is_sick(&self.world, v)).count();
            if symptomatic >= 2 && self.rng.below(100) < 25 {
                let blame_the_cures = self.rng.below(2) == 0;
                let target = if blame_the_cures {
                    self.herbologist.filter(|&h| !villager::is_dead(&self.world, h))
                        .or_else(|| self.villagers.iter().copied().find(|&v|
                            !villager::is_dead(&self.world, v) && !is_sick(&self.world, v)))
                } else {
                    // the newest face wears it
                    self.villagers.iter().copied().rev().find(|&v|
                        !villager::is_dead(&self.world, v) && !is_sick(&self.world, v))
                };
                if let Some(t) = target {
                    desires::add_grievance(&mut self.world, t, desires::Problem::Blamed, tick + 30);
                    self.say(t, "They say it's my doing.", CHAT_BUBBLE_SECS);
                }
            }
        }
    }

    /// Whether the disasters finished this building - nobody moves back in
    pub fn building_ruined(&self, e: Entity) -> bool {
        self.building_entities.iter().position(|&b| b == e)
            .map(|i| self.ruined.contains(&i)).unwrap_or(false)
    }

    /// A disaster finishes a building: its stock, hearth and trade are
    /// gone, whoever worked or lived there is out on the street, and the
    /// view dims what's left. `how` says what took it ("burned to
    /// shells", "under the flood")
    /// The midden's own score: doorsteps drip a little, the animal
    /// yards and the messier shops worse, and what isn't hauled,
    /// composted or dumped sits and stinks - a grievance on whoever
    /// sleeps beside it
    pub(in crate::session) fn filth_tick(&mut self, tick: u64) {
        use crate::game::household::BuildingKind;
        // the sources: every doorstep a little, the fold and the tavern's
        // mess more - the leavings land on the building's tile
        for &b in &self.building_entities {
            let Some(bb) = self.world.get_component::<household::Building>(b) else { continue };
            let Some(p) = self.world.get_component::<Pos>(b) else { continue };
            let drip = match bb.kind {
                BuildingKind::Home => 0.006,
                BuildingKind::Pasture => 0.03,
                BuildingKind::Shop(crate::game::vocation::Vocation::Taverner) => 0.02,
                BuildingKind::Shop(_) => 0.008,
                _ => 0.0,
            };
            if drip > 0.0 {
                let t = (p.x.round() as i32, p.y.round() as i32);
                let e = self.land.filth.entry(t).or_insert(0.0);
                *e = (*e + drip).min(1.0);
            }
        }
        // decay, a little spread off the foulest tiles, and the three
        // ways the leavings leave: the cart's haul to the water, the
        // compost that fields take faster, and the river's own carry
        let mut spread_from: Vec<(i32, i32)> = vec![];
        let mut dump_at: Vec<(i32, i32)> = vec![];
        for (&t, f) in self.land.filth.iter_mut() {
            let compost = matches!(self.map.get(t.0, t.1),
                Some(Tile::Wheat) | Some(Tile::Corn));
            *f = (*f - if compost { 0.01 } else { 0.004 }).max(0.0);
            if *f > 0.3 { spread_from.push(t); }
            if self.map.get(t.0, t.1) == Some(Tile::River) { dump_at.push(t); }
        }
        for t in spread_from {
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (t.0 + dx, t.1 + dy);
                if self.map.get(n.0, n.1).is_none() { continue; }
                let e = self.land.filth.entry(n).or_insert(0.0);
                *e = (*e + 0.002).min(1.0);
            }
        }
        for t in dump_at {
            let take = self.land.filth.get(&t).copied().unwrap_or(0.0) * 0.2;
            if let Some(f) = self.land.filth.get_mut(&t) { *f -= take; }
            let e = self.land.effluent.entry(t).or_insert(0.0);
            *e = (*e + take * 0.5).min(1.0);
        }
        // the cart errand, in the small: a household on a fouled doorstep
        // clears a share of it each day and the load lands downstream;
        // and the leavings by the door are a grievance on whoever's in
        for &b in &self.building_entities {
            let Some(bb) = self.world.get_component::<household::Building>(b) else { continue };
            let Some(p) = self.world.get_component::<Pos>(b) else { continue };
            if bb.kind != BuildingKind::Home || bb.occupants.is_empty() { continue; }
            let bt = (p.x.round() as i32, p.y.round() as i32);
            let f = self.land.filth.get(&bt).copied().unwrap_or(0.0);
            if f > 0.05 {
                let haul = (f * 0.15).min(0.02);
                if let Some(x) = self.land.filth.get_mut(&bt) { *x -= haul; }
                // dumped downstream: the nearest reach tile takes the cart's load
                if let Some(&r) = self.reach.first() {
                    let e = self.land.effluent.entry(r).or_insert(0.0);
                    *e = (*e + haul * 0.3).min(1.0);
                }
            }
            if f > 0.4 {
                let occs: Vec<Entity> = bb.occupants.clone();
                for occ in occs {
                    desires::add_grievance(&mut self.world, occ,
                        desires::Problem::StinkingReach, tick + 30);
                }
            }
        }
    }

    pub(in crate::session) fn ruin_building(&mut self, e: Entity, how: &str) {
        // the village's own store is the one roof the street can't lose -
        // it can burn and be robbed of its stock, but it stands
        if e == self.market_food { return; }
        let Some(i) = self.building_entities.iter().position(|&b| b == e) else { return };
        if self.ruined.contains(&i) { return; }
        self.ruined.insert(i);
        let (name, at) = {
            let b = self.world.get_component_mut::<Building>(e);
            let name = b.as_ref().map(|b| b.name.clone()).unwrap_or_else(|| "a house".into());
            if let Some(b) = b {
                b.stock.clear();
                b.occupants.clear();
                b.worker = None;
                b.overseer = None;
            }
            (name, self.world.get_component::<Pos>(e).map(|p| (p.x, p.y)))
        };
        // whoever walked to work there is between jobs now - the hiring
        // round finds them again with no shop to name
        for &v in self.villagers.clone().iter() {
            if let Some(c) = self.world.get_component_mut::<Craft>(v) {
                if c.workplace == Some(e) { c.workplace = None; }
            }
        }
        self.log.push_at_who(format!("t{}: the {} is {} - the street will miss it.", self.tick, name, how), at, None);
        self.toasts.push_at((format!("The {} is {}!", name, how), self.tick + 40), at);
    }

    /// Whatever's alight on the map runs its course: spreading a tile a
    /// day, burning out to scorched ground, and gutting what it reaches
    pub(in crate::session) fn fire_tick(&mut self, tick: u64) {
        if !self.fire.active() && self.fire.scorched.is_empty() { return; }
        let (lit, out) = self.fire.tick(&mut self.map, tick);
        // a building the fire holds loses what it holds - it burns while
        // it stands alight, restocking only once it's cold
        for &(x, y) in self.fire.burning.keys() {
            if let Some(&be) = self.building_entities.iter().find(|&&be| {
                self.world.get_component::<Pos>(be)
                    .map(|p| p.x.round() as i32 == x && p.y.round() as i32 == y).unwrap_or(false)
            }) {
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    b.stock.clear();
                }
            }
        }
        // and the canopy score forgets the trees the fire took
        for &t in &out {
            if let Some(c) = self.land.tree_cover.get_mut(&t) { *c = 0.0; }
            // a house the fire finishes is a ruin, not a shelter
            if let Some(&be) = self.building_entities.iter().find(|&&be| {
                self.world.get_component::<Pos>(be)
                    .map(|p| p.x.round() as i32 == t.0 && p.y.round() as i32 == t.1).unwrap_or(false)
            }) {
                self.ruin_building(be, "gone, burned to shells");
            }
        }
        // folk beside the flames run for home - errands and all. The
        // bucket line is the exception: they hold their ground at the
        // fire, wet and purposeful, where everyone else runs
        if self.fire.active() {
            let fleeing: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                !villager::is_dead(&self.world, v)
                    && !self.fire_crew.contains(&v)
                    // a seat's own villager walks where its master says -
                    // the flames don't cancel her errand either
                    && !self.seats.iter().any(|s| s.player == v)
                    && self.world.get_component::<Pos>(v)
                        .map(|p| self.fire.burns_near(p.x.round() as i32, p.y.round() as i32, 2))
                        .unwrap_or(false)
            }).collect();
            for v in fleeing {
                // but not toward the flames: home if it stands clear,
                // else the market store
                let safe = |e: Entity| self.world.get_component::<Pos>(e)
                    .map(|p| !self.fire.burns_near(p.x.round() as i32, p.y.round() as i32, 4))
                    .unwrap_or(false);
                let home = household::home_of(&self.world, v).filter(|&h| safe(h))
                    .or(Some(self.market_food).filter(|&h| safe(h)));
                if let Some(h) = home {
                    if self.send(v, Goal::HeadHome(h)) {
                        self.say(v, "Fire! Fire on the meadow!", CHAT_BUBBLE_SECS);
                        // the errand's off: the homeward way's the one that stays
                        self.travels.retain(|t| t.who != v || matches!(t.goal, Goal::HeadHome(_)));
                    }
                }
            }
            // and whoever's still in it when it closes takes the hurt -
            // the fire outruns the slowest feet. The bucket line stands
            // its ground: they've buckets and sense enough to take the
            // heat where the runners can't
            let caught: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                !villager::is_dead(&self.world, v)
                    && !self.fire_crew.contains(&v)
                    && self.world.get_component::<Pos>(v)
                        .map(|p| self.fire.burning.keys().any(|&(bx, by)| {
                            (p.x.round() as i32 - bx).abs() + (p.y.round() as i32 - by).abs() <= 1
                        }))
                        .unwrap_or(false)
            }).collect();
            for v in caught {
                if let Some(h) = self.world.get_component_mut::<Health>(v) {
                    h.condition = (h.condition - 0.2).max(0.0);
                }
            }
        }
        if !lit.is_empty() || !out.is_empty() {
            let at = self.fire.burning.keys().next().map(|&(bx, by)| (bx as f32 + 0.5, by as f32 + 0.5));
            self.log.push_at_who(format!("t{}: fire on the meadow - {} tiles alight, {} burnt out.", tick, lit.len(), out.len()), at, None);
        }
    }

    /// The bucket line: while anything's alight the nearest four able
    /// villagers answer it - to the water, then onto the flames - and
    /// hold their ground where everyone else runs for home. The line's
    /// cried once a blaze, in the log
    pub(in crate::session) fn fire_duty(&mut self) {
        if !self.fire.active() {
            if !self.fire_crew.is_empty() || !self.fire_spent.is_empty() {
                self.fire_crew.clear();
                self.fire_spent.clear();
                self.fire_line_called = false;
            }
            return;
        }
        // the dead, the held and the claimed fall off the line - buckets
        // and all, so a seat's own villager walks where its master says
        let mut dropped = vec![];
        self.fire_crew.retain(|&v| {
            let keep = !villager::is_dead(&self.world, v)
                && !crate::game::dungeon::unavailable(&self.world, v)
                && !self.seats.iter().any(|s| s.player == v)
                && !self.fire_spent.contains(&v);
            if !keep { dropped.push(v); }
            keep
        });
        for v in dropped {
            self.travels.retain(|t| t.who != v
                || !matches!(t.goal, Goal::FillBucket | Goal::DouseFire(_)));
        }
        // hands with nothing under way take up the bucket again
        let idle: Vec<Entity> = self.fire_crew.iter().copied()
            .filter(|&v| !self.travels.iter().any(|t| t.who == v)).collect();
        for v in idle {
            if self.task_firefighter(v) {
                self.say(v, "To the river!", CHAT_BUBBLE_SECS);
            } else {
                // no way to water from here - they're off the line
                self.fire_spent.insert(v);
                self.fire_crew.retain(|&c| c != v);
            }
        }
        // and the line tops up to four, nearest the flames first
        while self.fire_crew.len() < 4 {
            let Some(v) = self.nearest_firefighter() else { break };
            // pulled off whatever they were doing, the way the runners are
            self.travels.retain(|t| t.who != v);
            if self.task_firefighter(v) {
                self.fire_crew.push(v);
                self.say(v, "Fire! To the river!", CHAT_BUBBLE_SECS);
            } else {
                self.fire_spent.insert(v);
            }
        }
        if !self.fire_line_called && !self.fire_crew.is_empty() {
            self.fire_line_called = true;
            self.log.push(format!("t{}: the bucket line forms - {} to the river!", self.tick, self.fire_crew.len()));
        }
    }

    /// Whether `v` can answer the fire: alive, free, a villager of the
    /// village - not the country, not a child, not mid-meeting, and not
    /// away on the lord's or the country's business
    fn fire_fighter_able(&self, v: Entity) -> bool {
        if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { return false; }
        if self.world.get_component::<Kid>(v).is_some() { return false; }
        if self.world.get_component::<country::Outlander>(v).is_some() { return false; }
        if self.meeting.as_ref().map(|m| m.invited.contains(&v)).unwrap_or(false) { return false; }
        // a seat's own villager walks where its master says - the line
        // doesn't commandeer them
        if self.seats.iter().any(|s| s.player == v) { return false; }
        !self.travels.iter().any(|t| t.who == v && matches!(t.goal,
            Goal::CollectTax(_) | Goal::Cart(_) | Goal::Levy(..) | Goal::Deliver(_) | Goal::Remit(..)))
    }

    /// Sworn to answer: the brigade's own take the bell before the
    /// neighbourhood volunteers
    pub(in crate::session) fn is_brigade(&self, v: Entity) -> bool {
        self.brigade.and_then(|b| self.world.get_component::<Organization>(b))
            .map(|o| o.members.contains(&v)).unwrap_or(false)
    }

    /// The able villager nearest the flames who isn't already on the
    /// line - village index breaks ties, so the pick is stable. The
    /// brigade's sworn hands answer first wherever they stand.
    fn nearest_firefighter(&self) -> Option<Entity> {
        let burning: Vec<(i32, i32)> = self.fire.burning.keys().copied().collect();
        if burning.is_empty() { return None; }
        self.villagers.iter().enumerate()
            .filter(|(_, &v)| self.fire_fighter_able(v)
                && !self.fire_crew.contains(&v) && !self.fire_spent.contains(&v))
            .filter_map(|(i, &v)| {
                let p = self.world.get_component::<Pos>(v)?;
                let d = burning.iter().map(|&(bx, by)| {
                    (p.x.round() as i32 - bx).abs() + (p.y.round() as i32 - by).abs()
                }).min().unwrap_or(i32::MAX);
                Some((d, i, v))
            })
            // sworn hands first, then nearest - the village index keeps
            // the order stable inside either rank
            .min_by_key(|&(d, i, v)| (!self.is_brigade(v), d, i))
            .map(|(_, _, v)| v)
    }

    /// Set `v` walking the bucket line's first leg - to the water
    fn task_firefighter(&mut self, v: Entity) -> bool {
        self.send(v, Goal::FillBucket)
    }

    /// The bank tiles beside the water, nearest `from` first - where a
    /// bucket gets filled. The banks are scanned once and kept; the
    /// river doesn't move
    fn fire_banks_near(&mut self, from: (i32, i32)) -> Vec<(i32, i32)> {
        if self.fire_banks.is_none() {
            let mut banks = vec![];
            for y in 0..self.map.height as i32 {
                for x in 0..self.map.width as i32 {
                    if !matches!(self.map.get(x, y), Some(Tile::Water) | Some(Tile::River)) { continue; }
                    for &(dx, dy) in &[(-1,0),(1,0),(0,-1),(0,1),(-1,-1),(-1,1),(1,-1),(1,1)] {
                        let b = (x + dx, y + dy);
                        if self.map.passable(b.0, b.1) { banks.push(b); }
                    }
                }
            }
            banks.sort_unstable();
            banks.dedup();
            self.fire_banks = Some(banks);
        }
        let mut banks: Vec<(i32, i32)> = self.fire_banks.as_ref().unwrap().iter().copied()
            .filter(|&(x, y)| self.map.passable(x, y)).collect();
        banks.sort_by_key(|&(x, y)| ((x - from.0).abs() + (y - from.1).abs(), x, y));
        banks
    }

    /// The nearest bank `from` can actually walk to within the line's
    /// reach - a bank across the river isn't close for being near. The
    /// candidates are tried in distance order and the first with a path
    /// wins; `send` finds the same answer in its memo
    pub(in crate::session) fn fire_bank_walkable(&mut self, from: (i32, i32)) -> Option<(i32, i32)> {
        self.fire_banks_near(from).into_iter()
            .filter(|&(x, y)| (x - from.0).abs() + (y - from.1).abs() <= BUCKET_REACH)
            .find(|&b| self.walk_memo(from, b).is_some())
    }

    /// Where `who` stands to throw water on burning tile `t`: the
    /// nearest dry neighbour `from` can walk to - they don't stand in
    /// the flames, and a side they can't reach doesn't count. The tile
    /// itself only when nothing beside it will do
    pub(in crate::session) fn douse_spot_walkable(&mut self, t: (i32, i32), from: (i32, i32)) -> Option<(i32, i32)> {
        let mut spots: Vec<(i32, i32)> = [(-1,0),(1,0),(0,-1),(0,1),(-1,-1),(-1,1),(1,-1),(1,1)].iter()
            .map(|&(dx, dy)| (t.0 + dx, t.1 + dy))
            .filter(|&(x, y)| self.map.passable(x, y))
            .collect();
        spots.sort_by_key(|&(x, y)| ((x - from.0).abs() + (y - from.1).abs(), x, y));
        if self.map.passable(t.0, t.1) { spots.push(t); }
        spots.into_iter()
            .filter(|&(x, y)| (x - from.0).abs() + (y - from.1).abs() <= BUCKET_REACH)
            .find(|&s| self.walk_memo(from, s).is_some())
    }

    /// Send `who` (bucket in hand) to the nearest burning tile they can
    /// still reach. False when no flame answers - they walk off wet.
    fn send_fire_to(&mut self, who: Entity, from: (i32, i32)) -> bool {
        let mut targets: Vec<(i32, i32)> = self.fire.burning.keys().copied().collect();
        targets.sort_by_key(|&t| ((t.0 - from.0).abs() + (t.1 - from.1).abs(), t.0, t.1));
        // a handful is enough: the first reachable blaze takes the pail
        for t in targets.into_iter().take(8) {
            if self.send(who, Goal::DouseFire(t)) { return true; }
        }
        false
    }

    /// The bucket line's first leg done: at the water with an empty
    /// pail - on to the nearest flame they can still reach
    pub(in crate::session) fn fire_bucket_fill(&mut self, who: Entity) {
        if !self.fire.active() || !self.fire_crew.contains(&who) { return; }
        let Some(from) = self.world.get_component::<Pos>(who)
            .map(|p| (p.x.round() as i32, p.y.round() as i32)) else { return };
        if !self.send_fire_to(who, from) {
            self.fire_spent.insert(who);
            self.fire_crew.retain(|&c| c != who);
            self.say(who, "It's too far - let it burn.", CHAT_BUBBLE_SECS);
        }
    }

    /// The second leg done: the bucket's off on the flames. Back to the
    /// water while the fire lives; a tile someone else already saved
    /// just moves the pail to the next flame
    pub(in crate::session) fn fire_bucket_douse(&mut self, who: Entity, t: (i32, i32)) {
        if !self.fire_crew.contains(&who) { return; }
        if self.fire.burning.contains_key(&t) {
            self.fire.douse(t.0, t.1, self.tick);
            // a sworn hand - or a crew with the chest's gear - throws
            // past the bucket: the next flame over goes too
            let extra = (self.is_brigade(who) || self.brigade_gear >= 3) as usize;
            for _ in 0..extra {
                let Some(&(nx, ny)) = self.fire.burning.keys()
                    .min_by_key(|&&(bx, by)| (bx - t.0).abs() + (by - t.1).abs()
                        .max(if (bx, by) == t { i32::MAX } else { 0 })) else { break };
                if (nx, ny) == t { break; }
                self.fire.douse(nx, ny, self.tick);
            }
            self.say(who, "Got it!", CHAT_BUBBLE_SECS);
            if self.fire.active() { self.send(who, Goal::FillBucket); }
            return;
        }
        // already out - the pail's still full: the next flame alight
        if !self.fire.active() { return; }
        let Some(from) = self.world.get_component::<Pos>(who)
            .map(|p| (p.x.round() as i32, p.y.round() as i32)) else { return };
        if !self.send_fire_to(who, from) {
            self.fire_spent.insert(who);
            self.fire_crew.retain(|&c| c != who);
            self.say(who, "It's too far - let it burn.", CHAT_BUBBLE_SECS);
        }
    }

    /// The sky keeps its own calendar: a comet every few years, an
    /// eclipse now and then - and when one rides over the walls every
    /// villager reads their own omen into it. A crowd of small opinions.
    pub fn sky_tick(&mut self, tick: u64) {
        // it burns for three nights
        if let Some((_, until)) = self.sky {
            if tick >= until { self.sky = None; }
        }
        let kind = if tick % 300 == 150 { Some("comet") }
            else if tick % 450 == 225 { Some("eclipse") } else { None };
        let Some(kind) = kind else { return };
        if self.sky.is_some() { return; }
        self.sky = Some((kind, tick + 3));
        self.log.push(format!("t{}: a {} rides over the walls - everyone looks up.", tick, kind));
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            // each reads it by what they carry
            let omen = if self.world.get_component::<Craft>(v)
                .map(|c| c.vocation == Vocation::Priest).unwrap_or(false) {
                "A sign. Pray."
            } else if Some(v) == self.herbologist {
                "A gathering-night, that."
            } else {
                self.world.get_component::<InnerLife>(v)
                    .and_then(|l| l.grievances.first().map(|(p, _)| match p {
                        desires::Problem::Blamed => "It burns for me.",
                        desires::Problem::Broke => "Even the sky's poor.",
                        desires::Problem::Smelly => "Even it can't see the fold from there.",
                        desires::Problem::Taxed | desires::Problem::DeathDuty => "Does the lord tax the sky?",
                        _ => "A bad omen. I feel it.",
                    })).unwrap_or_else(|| {
                        self.world.get_component::<InnerLife>(v)
                            .map(|l| if l.wants.contains(&Want::Love) { "A star for wishes." }
                                else { "Look up." }).unwrap_or("Look up.")
                    })
            };
            self.say(v, omen, CHAT_BUBBLE_SECS * 2);
        }
    }

    /// Every other winter runs long: the snow keeps falling, the store
    /// thins, and the village either pulls together - an org worth the
    /// name shares what it has - or pulls apart, and looks for who to
    /// blame. The herb-wife's hut looks very different in a hard winter.
    pub fn winter_tick(&mut self, tick: u64) {
        let was = self.harsh_winter;
        self.harsh_winter = tick % 730 >= 400 && tick % 730 <= 545;
        // and the word of it comes first: ten days out, the wise say
        // winter'll run long - the village hears the lean months coming
        if tick % 730 == 390 {
            self.log.push(format!("t{}: the wise say winter'll run long this year - best lay by.", tick));
        }
        if self.harsh_winter && !was {
            self.log.push(format!("t{}: the winter sets in hard - and looks to stay.", tick));
        }
        if !self.harsh_winter { return; }
        // the snow keeps on
        if self.precip.is_none() { self.precip = Some(crate::sim::precipitation::PrecipKind::Flurries); }
        // the store thins
        if tick % 5 == 0 {
            if let Some(b) = self.world.get_component_mut::<Building>(self.market_food) {
                let _ = household::take_stock(b, "grain", 1);
            }
        }
        // and everyone's a little hungrier for it - except the dead and
        // the held, whose hunger isn't theirs to feel
        if tick % 10 == 0 {
            for &v in &self.villagers.clone() {
                if crate::game::dungeon::unavailable(&self.world, v) { continue; }
                if let Some(h) = self.world.get_component_mut::<Hunger>(v) {
                    if h.value > 8.0 { h.value -= 8.0; }
                }
            }
        }
        // the long winter is a grievance everyone shares
        if tick % 30 == 10 {
            for v in self.villagers.iter().take(4).copied().collect::<Vec<_>>() {
                if !villager::is_dead(&self.world, v) {
                    desires::add_grievance(&mut self.world, v, desires::Problem::LongWinter, tick + 40);
                }
            }
            // an org worth the name shares what it has - a weak village
            // looks for who to blame
            if self.lord_esteem > 0.5 {
                if tick % 60 == 10 {
                    for v in self.villagers.iter().copied()
                        .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).take(6)
                        .collect::<Vec<_>>() {
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 15);
                        if let Some(h) = self.world.get_component_mut::<Hunger>(v) {
                            h.value = (h.value + 15.0).min(100.0);
                        }
                    }
                    self.record_power(crate::sim::organized_power::ActionQuality::Transformational);
                    self.log.push(format!("t{}: the village pulls together - the org shares out what it has.", tick));
                }
            } else {
                // someone to blame: the herb-wife first, then the
                // heart-helper, then any role that isn't them
                let scapegoat = self.herbologist
                    .or(self.matchmaker)
                    .or(self.collector)
                    .filter(|&v| !villager::is_dead(&self.world, v));
                if let Some(s) = scapegoat {
                    desires::add_grievance(&mut self.world, s, desires::Problem::Blamed, tick + 45);
                    if tick % 60 == 10 {
                        self.log.push(format!("t{}: thin stores make thin tempers - they're saying things of the herb-wife.", tick));
                    }
                }
            }
        }
    }
}

impl Game {
    /// The sheriff's day: the law's arm out of the castle answers a
    /// poacher in the reserve or a trespasser with the book - a body
    /// walking the wooded ring under his castle's law reads as a crime
    pub(in crate::session) fn sheriff_tick(&mut self, tick: u64) {
        for i in 0..self.sites.len() {
            if self.sites[i].kind != country::SiteKind::Castle || self.sites[i].sheriff.is_none() { continue; }
            let (cx, cy, r) = { let s = &self.sites[i]; (s.centre.0, s.centre.1, s.radius) };
            // who's on his ground: a village body on a wooded tile inside
            // the chase's ring, not his own folk and not the sheriff
            for v in self.villagers.clone() {
                if villager::is_dead(&self.world, v) || self.site_of(v).is_some() { continue; }
                let p = match self.world.get_component::<Pos>(v) { Some(p) => *p, None => continue };
                let d = (p.x.round() as i32 - cx).abs() + (p.y.round() as i32 - cy).abs();
                if d > r + 4 { continue; }
                // a body in the woods where only the hunt's law stands:
                // the book answers with the fine and the log names the crime
                if self.map.trees.iter().any(|&(tx, ty)|
                    (p.x.round() as i32 - tx).abs() <= 1 && (p.y.round() as i32 - ty).abs() <= 1) {
                    self.apply_penalty(i, v, "poaching", tick);
                    self.log_entity(v, format!("t{}: {} taken poaching in the chase - the sheriff answers.",
                        tick, self.name_of(v)));
                    break;
                }
            }
        }
    }

    /// The site's day: hands on the scaffold count the labor, a day's
    /// work's a day's raise, and a crew that never forms leaves the
    /// foundation to the weather
    pub(in crate::session) fn site_tick(&mut self, tick: u64) {
        let mut done: Vec<usize> = vec![];
        for (i, site) in self.construction_sites.iter_mut().enumerate() {
            // who's on it: able hands standing at the tile, not the sick
            // or the marched-off or the young
            let on_site: Vec<Entity> = site.crew.iter().copied()
                .filter(|&v| self.world.get_component::<Pos>(v)
                    .map(|p| (p.x.round() as i32 - site.tile.0).abs()
                        + (p.y.round() as i32 - site.tile.1).abs() < 4)
                    .unwrap_or(false)).collect();
            if !on_site.is_empty() {
                site.progress += on_site.len() as u32;
                site.stalled_days = 0;
            } else {
                site.stalled_days += 1;
            }
            if site.progress >= site.need { done.push(i); }
        }
        for i in done.into_iter().rev() {
            let site = self.construction_sites.remove(i);
            let b = self.country_building(site.kind, "the new work".into(),
                site.tile.0, site.tile.1);
            self.log.push(format!("t{}: the work stands at last - the village's {} roof rises.", tick, b));
            self.built_count += 1;
        }
        // a site whose crew never came goes to weather: the log names it once
        for site in self.construction_sites.iter_mut() {
            if site.stalled_days == 7 {
                self.log.push(format!("t{}: the work on the new ground stands unfinished - no hands came.", tick));
            }
        }
    }

    /// The water's day: the rain raises it, the low ground drinks, the
    /// crops under it are gone - and when the weather dries, the shallow
    /// cells give the land back
    pub(in crate::session) fn flood_tick(&mut self, tick: u64) {
        // the rain raises the water; a dry day's sun lets it down
        let wet_day = matches!(self.precip, Some(crate::sim::precipitation::PrecipKind::Rain)
            | Some(crate::sim::precipitation::PrecipKind::HeavyRain)
            | Some(crate::sim::precipitation::PrecipKind::Showers));
        if wet_day {
            self.flood.raise(crate::game::flood::RAIN_LEVEL);
        } else if self.flood.level > 0.0 {
            self.flood.recede(0.02);
        }
        // the wetting front: the cells under the level soak in
        self.flood.wet(&self.map, &self.heights, tick);
        // a crop under the water is lost - the harvest doesn't come back
        for &(x, y) in self.flood.cells.keys() {
            if let Some(&ce) = self.crop_fields.iter().find(|&&ce| {
                self.world.get_component::<Pos>(ce)
                    .map(|p| p.x.round() as i32 == x && p.y.round() as i32 == y).unwrap_or(false)
            }) {
                if let Some(c) = self.world.get_component_mut::<crate::game::villager::Crop>(ce) {
                    if c.growth > 0.0 {
                        c.growth = 0.0;
                        self.log.push(format!("t{}: the water takes a field - the year's work's under it.", tick));
                    }
                }
            }
        }
        // a building in deep water is a ruin; in shallow, it's waterlogged
        for (&(x, y), &band) in self.flood.cells.clone().iter() {
            if let Some(&be) = self.building_entities.iter().find(|&&be| {
                self.world.get_component::<Pos>(be)
                    .map(|p| p.x.round() as i32 == x && p.y.round() as i32 == y).unwrap_or(false)
            }) {
                if band >= 2 {
                    self.ruin_building(be, "gone, under the water");
                } else if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    b.stock.clear(); // the shallow water spoils what's stored
                }
            }
        }
    }
}
