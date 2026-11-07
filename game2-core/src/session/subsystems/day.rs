//! The day: livelihood_tick runs each subsystem's daily tick in order, plus the town's own business (work, errands, taxes).

use crate::session::*;

impl Game {
    /// The town's business each day: work, errands, the tax round, the
    /// trader's restock, the odd minstrel, a tune in the street
    pub fn livelihood_tick(&mut self, tick: u64) {
        let prelude = crate::infra::perf::scope("sim.day.prelude");
        // a seat's own belly sees to itself with the morning: below a
        // growl the satchel's cheapest bite goes down on its own -
        // staples fill, a sweet's a taste, drink's no meal
        for seat in 0..self.seats.len() {
            let pe = self.seats[seat].player;
            let hungry = self.world.get_component::<Hunger>(pe).map(|h| h.value < 40.0).unwrap_or(false);
            if !hungry { continue; }
            let Some(s) = self.world.get_component::<Satchel>(pe) else { continue };
            let bite = s.goods.iter().filter(|(g, &n)| n > 0
                    && goods::spec(g).map(|sp| {
                        if sp.tags.contains(&goods::Tag::StapleFood) { Some(60.0) }
                        else if sp.tags.contains(&goods::Tag::Fruit) { Some(30.0) }
                        else if sp.tags.contains(&goods::Tag::Sweet) { Some(15.0) }
                        else { None }
                    }).flatten().is_some())
                .min_by(|a, b| goods::worth_at(&self.world, a.0)
                    .partial_cmp(&goods::worth_at(&self.world, b.0)).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(g, _)| g.clone());
            let Some(good) = bite else { continue };
            let fill = goods::spec(&good).map(|sp| {
                if sp.tags.contains(&goods::Tag::StapleFood) { 60.0 }
                else if sp.tags.contains(&goods::Tag::Fruit) { 30.0 }
                else { 15.0 }
            }).unwrap_or(0.0);
            if let Some(s) = self.world.get_component_mut::<Satchel>(pe) {
                if let Some(n) = s.goods.get_mut(&good) { *n -= 1; if *n == 0 { s.goods.remove(&good); } }
            }
            if let Some(h) = self.world.get_component_mut::<Hunger>(pe) { h.value = (h.value + fill).min(100.0); }
            self.log.push(format!("t{}: you eat the {} from your satchel.", tick, good));
        }
        // an organizer's founding call that goes unanswered too long lapses
        // on its own, same as "let it rest" - otherwise a seat with no
        // control wired to Command::Decide (the browser canvas shim has
        // none yet) would hold the whole game at half pace forever
        for seat in 0..self.seats.len() {
            let stale = self.seats[seat].pending_founding.as_ref()
                .map(|d| tick.saturating_sub(d.offered_tick) >= FOUNDING_OFFER_EXPIRY_DAYS)
                .unwrap_or(false);
            if stale {
                let d = self.seats[seat].pending_founding.take().unwrap();
                self.log.push(format!("t{}: the talk stays talk for now - no cause taken up.", tick));
                self.note_talk(d.record, None, "The talk stays talk for now - no cause taken up.");
            }
        }
        // the felt worths slide a little back toward ordinary each day
        goods::drift(&mut self.world);
        // market day, once a month: the org's members sell their work and
        // the purse sees its share of the takings
        if tick % 30 == 20 {
            for seat in 0..self.seats.len() {
                let earned = crate::game::player::market_day(&mut self.world,
                    self.seats[seat].player, self.seats[seat].org);
                if earned > 0.0 {
                    self.log.push(format!("t{}: market day - the org sells its members' work; {:.0} gold to the purse.", tick, earned));
                }
            }
        }
        // the trader's stall restocks an exotic every couple of days
        if tick % livelihood::TRADER_RESTOCK_DAYS == 0 {
            if let Some(s) = self.trader_stall { livelihood::restock_trader(&mut self.world, s, tick); }
        }
        // workplaces nobody works stand dimmed - the derelicts of the street
        let was_derelict = self.derelict.clone();
        self.derelict.clear();
        for (i, e) in self.building_entities.iter().enumerate() {
            if self.world.get_component::<Building>(*e).map(household::derelict).unwrap_or(false) {
                self.derelict.insert(i);
                if !was_derelict.contains(&i) {
                    let name = self.world.get_component::<Building>(*e)
                        .map(|b| b.name.clone()).unwrap_or_else(|| "a house".into());
                    self.log.push_at_who(format!("t{}: {} stands derelict - nobody works it.", tick, name),
                        toast_spot(&self.world, *e), None);
                }
            }
        }
        // word rides a few days ahead of whoever's on the road
        if tick + 3 == self.next_minstrel {
            self.log.push(format!("t{}: word is a minstrel's on the road.", tick));
        }
        if tick + 3 == self.next_pedlar {
            self.log.push(format!("t{}: word is the pedlar's due with the gossip.", tick));
        }
        if tick + 3 == self.next_circus {
            self.log.push(format!("t{}: word is a circus camps outside the walls come the week's end.", tick));
        }
        if tick + 3 == self.next_ink_boiler {
            self.log.push(format!("t{}: word is the ink-maker's coming to boil his oil.", tick));
        }
        // a minstrel rides in every so often
        if self.minstrel.is_none() && tick >= self.next_minstrel { self.spawn_minstrel(); }
        // and the pedlar comes by every few weeks - the game's newspaper
        if self.pedlar.is_none() && tick >= self.next_pedlar { self.spawn_pedlar(); }
        // and the village grows the way it shrinks - one at a time: a
        // child in a full house, a stranger at the gate
        if tick > 0 && tick % 40 == 0 { self.maybe_birth(tick); }
        if tick > 0 && tick % 60 == 30 { self.maybe_migrant(tick); }
        // and a circus camps outside the walls every year or so
        if self.circus.is_none() && tick >= self.next_circus { self.spawn_circus(); }
        // and the ink-maker boils his oil where the law wants it: outside
        if self.ink_boiler.is_none() && tick >= self.next_ink_boiler { self.spawn_ink_boiler(); }
        // the collector walks his round
        if let Some(c) = self.collector {
            if !self.travels.iter().any(|t| t.who == c) {
                if let Some(target) = livelihood::tax_due(&self.world, c, tick) {
                    self.send(c, Goal::CollectTax(target));
                }
            }
            // and rides out with the takings: the lord's cut is the
            // lord's - his books hand it over, whatever his own purse
            // holds, and what's his stays his
            if tick % 60 == 45 && self.takings > 0.0 {
                let remit = self.takings;
                if let Some(p) = self.world.get_component_mut::<Purse>(c) { p.gold -= remit.min(p.gold); }
                self.takings = 0.0;
                self.log.push(format!("t{}: the collector rides to the castle - {:.0} gold of the lord's cut goes over the wall.", tick, remit));
                // and he rides it there himself: the road to the nearest
                // keep, a meal at its store, and the long way home
                let cp = self.world.get_component::<Pos>(c).map(|p| (p.x, p.y));
                if let Some((si, _)) = self.sites.iter().enumerate()
                    .filter(|(_, s)| matches!(s.kind, country::SiteKind::Castle))
                    .min_by_key(|(_, s)| cp.map(|(x, y)|
                        ((s.centre.0 as f32 - x).powi(2) + (s.centre.1 as f32 - y).powi(2)) as i64)
                        .unwrap_or(0)) {
                    let door = self.sites[si].homes.first()
                        .and_then(|&k| self.world.get_component::<Pos>(k).copied())
                        .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32))
                        .unwrap_or(self.sites[si].centre);
                    let _ = self.send_far(c, door, Goal::Remit(si));
                }
            }
        }
        drop(prelude);
        // the herb-wife keeps her shelf brewed and her patches picked
        crate::timed!("sim.herbologist", self.herbologist_tick(tick));
        // the fold grazes, yields, and now and then howls
        crate::timed!("sim.livestock", self.livestock_tick(tick));
        // the heart-helper walks a lonely heart toward someone
        crate::timed!("sim.matchmaker", self.matchmaker_tick(tick));
        // a poor soul gathers in the wood, and damp years bring the fire
        crate::timed!("sim.mushroom", self.mushroom_tick(tick));
        // the skeps yield, the church burns wax, a swarm wanders off
        crate::timed!("sim.apiary", self.apiary_tick(tick));
        // the larder keeps its own score: spoilage, vermin, the cat's job
        crate::timed!("sim.spoilage", self.spoilage_tick(tick));
        // the dead hand on: benches, purses, and the lord's fifth
        crate::timed!("sim.succession", self.succession_tick(tick));
        // and the young grow into their trades
        crate::timed!("sim.lifecycle", self.lifecycle_tick(tick));
        // the worn paths fade toward green where no one walks
        crate::timed!("sim.trail", self.trail_tick());
        // the land keeps score: canopy, pit water, the reach
        crate::timed!("sim.land", self.land_tick(tick));
        // the ladder of shame climbs one rung at a time
        crate::timed!("sim.sanction", self.sanction_tick(tick));
        // and the blood feuds simmer or settle under it
        crate::timed!("sim.feuds", self.feud_tick(tick));
        // and the year has its feast days
        crate::timed!("sim.festival", self.festival_tick(tick));
        let hearths = crate::infra::perf::scope("sim.day.hearths");
        // winter hearths burn a bundle every few days - faggots, turf,
        // the dry leavings, charcoal off the road; a house with nothing
        // to burn eats cold, and that's a trouble the village can hear
        if tick % 360 >= 270 && tick % 3 == 0 {
            for &h in &self.building_entities.clone() {
                let Some(b) = self.world.get_component::<Building>(h) else { continue };
                if b.kind != household::BuildingKind::Home { continue; }
                let occupants = b.occupants.clone();
                let fuel = crate::game::goods::slot_goods(crate::game::goods::Tag::Fuel)
                    .into_iter().find(|&g| b.stock.get(g).copied().unwrap_or(0) > 0);
                // else the house burns what its folk carry home - a
                // bundle in the satchel keeps the hearth same as a stack
                let hp = self.world.get_component::<Pos>(h).copied();
                let carried = if fuel.is_none() {
                    crate::game::goods::slot_goods(crate::game::goods::Tag::Fuel).into_iter()
                        .find_map(|g| occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                            // and actually home - the bundle has to be
                            // under the roof to warm it
                            && hp.and_then(|hp| self.world.get_component::<Pos>(o)
                                .map(|op| (op.x - hp.x).abs() + (op.y - hp.y).abs() <= 7.0)).unwrap_or(false)
                            && self.world.get_component::<Satchel>(o)
                                .map(|s| s.goods.get(g).copied().unwrap_or(0) > 0).unwrap_or(false))
                            .map(|o| (o, g)))
                } else { None };
                match fuel.or(carried.map(|(_, g)| g)) {
                    Some(g) => {
                        if let Some((o, _)) = carried {
                            if let Some(s) = self.world.get_component_mut::<Satchel>(o) {
                                *s.goods.get_mut(g).unwrap() -= 1;
                            }
                        } else if let Some(b) = self.world.get_component_mut::<Building>(h) {
                            household::take_stock(b, g, 1);
                        }
                    }
                    None => {
                        for &o in &occupants {
                            if !villager::is_dead(&self.world, o) {
                                desires::add_grievance(&mut self.world, o,
                                    desires::Problem::ColdHearth, tick + 9);
                            }
                        }
                        // and a cold house does something about it: a free
                        // hand goes out for a bundle tomorrow - never a
                        // villager a seat steers, that errand's theirs to give
                        if let Some(hand) = occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                            && !crate::game::dungeon::unavailable(&self.world, o)
                            && !self.travels.iter().any(|t| t.who == o)
                            && !self.seats.iter().any(|s| s.player == o)) {
                            if let Some(op) = self.world.get_component::<Pos>(hand).copied() {
                                if let Some(t) = self.gather_site("faggots", op) {
                                    self.send_gather(hand, "faggots", t);
                                }
                            }
                        }
                    }
                }
            }
        }
        drop(hearths);
        // the gardens fill in a little more each week
        crate::timed!("sim.garden", self.garden_tick(tick));
        // and the kitchen pot: staples off the garden beds feed the
        // hungry at home, and a week's peas and roots become pottage
        crate::timed!("sim.kitchen", self.kitchen_tick(tick));
        // and the circus, while it camps, works its pull
        crate::timed!("sim.circus", self.circus_tick(tick));
        // the oil boils, the crusts fry, and the black goes in
        crate::timed!("sim.ink_boiler", self.ink_boiler_tick(tick));
        // and the hundred small fortunes: what's due comes on
        crate::timed!("sim.events", self.events_tick(tick));
        // and whatever the day caught, the fire runs on
        crate::timed!("sim.fire", self.fire_tick(tick));
        // milk becomes wheels in the dark, and a noble buys the old ones
        crate::timed!("sim.cheese", self.cheese_tick(tick));
        // ale sours, wine turns, and the still owes the wise woman
        crate::timed!("sim.drink", self.drink_tick(tick));
        // the keeper lays in a barrel, and the bench fills
        crate::timed!("sim.tavern", self.tavern_tick(tick));
        // the day's work leaves its smell - unless the rain washes it off
        crate::timed!("sim.smell", self.smell_tick(tick));
        // power atrophies; the lord's estimate creeps toward the truth
        crate::timed!("sim.power", self.power_tick(tick));
        // and now and then something rides over the walls
        crate::timed!("sim.sky", self.sky_tick(tick));
        // and the soundscape's cues rise with the days
        crate::timed!("sim.sfx", self.sfx_tick(tick));
        // and a truth sits unsaid somewhere, ripening
        crate::timed!("sim.discovery", self.discovery_tick(tick));
        // and the castle sends its writs on a season's turn
        crate::timed!("sim.edict", self.edict_tick(tick));
        // and every other winter runs long and tests the village
        crate::timed!("sim.winter", self.winter_tick(tick));
        // and the painter lays color on the village's halls
        crate::timed!("sim.painter", self.painter_tick(tick));
        // and now and then a maker makes a thing to outlast them
        crate::timed!("sim.heirloom", self.heirloom_tick(tick));
        // and the literate put word to those they think of
        crate::timed!("sim.letter", self.letter_tick(tick));
        // and theirs check on the ones who need checking on
        crate::timed!("sim.care", self.care_tick(tick));
        // and the river gives up its catch, season on season
        crate::timed!("sim.fisher", self.fisher_tick(tick));
        // and if the player's gone, the work goes on in another pair of hands
        crate::timed!("sim.continuation", self.continuation_tick(tick));
        // and the martyr's grave keeps its vigil
        crate::timed!("sim.martyr", self.martyr_tick(tick));
        // and the castle's prisoners wait on the org's answer
        crate::timed!("sim.dungeon", self.dungeon_tick(tick));
        // and the posted knights stand only their tour
        crate::timed!("sim.guards", self.guards_tick(tick));
        // and the marks a cause leaves fade on their own clock
        crate::timed!("sim.marks", self.marks_tick(tick));
        let mut rng = Rng(self.rng.next());
        // the country's carts and stewards, and its granaries
        crate::timed!("sim.country", self.country_tick(tick));
        // the holy day calls the faithful before anyone's planned the rest
        // the weather resolves before anyone lifts a tool: a rolled event
        // weighs on the day's harvest and health alike
        crate::timed!("sim.weather", self.weather_tick(tick));
        // the priest reads the old books; what they teach helps the sick
        crate::timed!("sim.magic", self.magic_tick(tick));
        // new traders carry the Merchant the guild's compact wants
        crate::timed!("sim.guild", self.guild_tick(tick));
        // the newly dead are mourned by those who held them close
        crate::timed!("sim.grief", self.grief_tick(tick));
        // ties cool a little each day: bad blood settles, warmth cools
        crate::timed!("sim.rels", self.rel_tick());
        // the rolls are updated: the dead leave them, the sway re-pools,
        // and a sit-down held yesterday deepens into a real tie
        crate::timed!("sim.influence", self.influence_tick(tick));
        // the fields ripen at the season's pace; the first ears coming
        // in are worth saying aloud once
        crate::timed!("sim.crops", self.crop_tick(tick));
        // starving hurts, eating well mends - the unsick only
        crate::timed!("sim.nourish", self.nourish_tick());
        // the year's turn: the org takes the harvest in together
        crate::timed!("sim.harvest", self.harvest_tick(tick));
        // the wood grows, spreads, burns and regrows - the fuel commons
        crate::timed!("sim.forest", self.forest_tick(tick));
        crate::timed!("sim.routine_dawn", self.routine_dawn(tick, &mut rng));
        // pickets and marches call their crowds back each dawn
        crate::timed!("sim.protest_dawn", self.protest_dawn(tick));
        // the office gathers whoever's keeping the holy day - church
        // rows or the stones, priest or no priest
        crate::timed!("sim.faith", self.service_tick(tick));
        // and folk who've gotten to know the Wanderer ask for small jobs
        crate::timed!("sim.quests", self.quest_dawn(tick, &mut rng));
        // now and then a mystery starts doing the rounds - the gossip's
        // out there to be heard a fact at a time
        crate::timed!("sim.mystery", self.mystery_dawn(tick, &mut rng));
        let folk = crate::infra::perf::scope("sim.day.villagers");
        let villagers = self.villagers.clone();
        // a promised visit outranks the day's routine but not the day's
        // own claims: round_back keeps the hungry, the traveling, the
        // gathered, the seat's own, the drunk, the sick and the meeting's
        // folk back - kids, officials and the rest are all fair game
        for &v in &villagers {
            // the marching keep step - the day can wait on them
            if self.world.get_component::<villager::InFormation>(v).is_some() { continue; }
            self.round_back(v, tick);
        }
        for i in 0..villagers.len() {
            let v = villagers[i];
            if crate::game::dungeon::unavailable(&self.world, v) { continue; }
            // hunger comes first: a starving person abandons any errand
            // to look for food - even one already under way. And looking
            // means walking: the store has the day's meal - except the
            // exile's road, which no belly's free to abandon
            let hungry = self.world.get_component::<Hunger>(v).map(|h| h.value < 30.0).unwrap_or(false);
            if hungry {
                // an exile walks on, belly and all - and the man riding
                // the lord's gold doesn't set it down for lunch; the
                // keep's table feeds him when he gets there
                self.travels.retain(|t| t.who != v
                    || matches!(t.goal, Goal::Exile | Goal::Remit(_)));
                if !self.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::Exile | Goal::Remit(_))) {
                    let at_store = self.world.get_component::<Pos>(self.store_of(v))
                        .zip(self.world.get_component::<Pos>(v))
                        .map(|(s, p)| (s.x - p.x).abs() + (s.y - p.y).abs() <= 1.0)
                        .unwrap_or(false);
                    if at_store { self.arrive(v, Goal::Eat); }
                    else if !self.send(v, Goal::Eat) && self.site_of(v).is_some() {
                        // out on the country's roads: the nearest granary
                        // with food in it, however far
                        let sp = self.world.get_component::<Pos>(self.store_of(v)).copied();
                        if let Some(d) = sp.and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32)) {
                            self.send_far(v, d, Goal::Eat);
                        }
                    }
                }
                continue;
            }
            if self.travels.iter().any(|t| t.who == v) { continue; }
            // a carter at the market, a steward on the road: the country's
            // business has them
            if self.is_away(v) { continue; }
            // today's rite or feast already has them
            let gathered = self.world.get_component::<routine::Routine>(v)
                .map(|r| r.day == tick && matches!(r.mode, routine::Mode::Worshipping | routine::Mode::Feasting))
                .unwrap_or(false);
            if gathered { continue; }
            if self.world.get_component::<Kid>(v).is_some() {
                // a child off the errand list plays - or sits a spell
                // at the schoolhouse, when it's open and they lean that way
                if self.world.get_component::<OddJob>(v).is_none() {
                    if !self.school_draw(v, &mut rng) { self.plan_play(v, tick, &mut rng); }
                }
                continue;
            }
            // the recluse answers to the dark, not the village's day -
            // the cave plan runs before the role's skip can take him
            if Some(v) == self.hermit { self.plan_cave(v, tick); continue; }
            if self.world.get_component::<TownRole>(v).is_some() { continue; }
            // a seat's own villager answers only its commands - never a
            // routine walk sent behind its back
            if self.seats.iter().any(|s| s.player == v) { continue; }
            // last night's cup still has them - they sleep it off
            if self.world.get_component::<villager::Drunk>(v)
                .map(|d| tick < d.until).unwrap_or(false) { self.plan_rest(v, tick); continue; }
            // at a meeting they're not off working
            if self.meeting.as_ref().map(|m| m.invited.contains(&v)).unwrap_or(false) { continue; }
            // the sick call on the herb-wife: out to her hut for a remedy,
            // a call at most every fortnight
            if is_sick(&self.world, v) {
                if let (Some(hut), Some(her)) = (self.herb_hut, self.herbologist) {
                    let seen = self.remedy_visits.iter().find(|(e, _)| *e == v).map(|(_, d)| *d).unwrap_or(0);
                    if !villager::is_dead(&self.world, her) && tick >= seen + 15 && self.send(v, Goal::SeekRemedy(hut)) { continue; }
                }
            }
            // a shopping day, now and then: something they want is for sale
            if rng.below(100) < 12 {
                if let Some(g) = self.want_shopping(v, tick) {
                    let good = match &g { Goal::BuyAt(_, good, _) => *good, _ => "something" };
                    if self.send(v, g) { self.plan_errand(v, tick, routine::Spot::Errand(good)); continue; }
                }
            }
            // the day's mode: work, a stall on market day, a stroll, a day in
            match self.plan_day(v, tick, &mut rng) {
                routine::DayPlan::Work => {}
                routine::DayPlan::Sell | routine::DayPlan::Free => continue,
            }
            // out of raw goods: send a kid, or go get it, or sit with the problem
            if let Some(input) = livelihood::needs_input(&self.world, v) {
                if self.errand_for_input(v, input, i) {
                    // a kid may have taken the errand: then they wait at the bench
                    if self.travels.iter().any(|t| t.who == v) {
                        self.plan_errand(v, tick, routine::Spot::Supplies(input));
                    } else { self.plan_bench(v, tick); }
                } else {
                    desires::add_grievance(&mut self.world, v, Problem::NoInput(input), tick + 3);
                    self.plan_bench(v, tick);
                }
                continue;
            }
            // a day at the craft, at the bench
            self.plan_bench(v, tick);
            let (made, up) = self.bench_work(v, tick);
            let _ = made;
            if let Some(level) = up {
                let lvl = vocation::level_name(level);
                let title = self.world.get_component::<Craft>(v)
                    .map(|c| format!("{} {}", lvl, vocation::vocation_name(c.vocation)))
                    .unwrap_or_else(|| lvl.to_string());
                self.toasts.push_at((format!("{} is now {} {}!", vill_name(i), vocation::article(&title), title), tick + 30),
                    toast_spot(&self.world, v));
                self.log_villager(i, format!("t{}: {} is now {} {}.", tick, vill_name(i), vocation::article(&title), title));
            }
        }
        drop(folk);
        // the print shop's work is the village's research: only the
        // printmakers who actually kept the bench today count
        crate::timed!("sim.prison", self.cell_tick(tick));
        crate::timed!("sim.post", self.post_tick(tick));
        crate::timed!("sim.school", self.school_tick(tick));
        crate::timed!("sim.tech", self.tech_tick(tick));
        crate::timed!("sim.cutaways", self.cutaways_tick(tick));
        // and the village's tune follows the day the village had
        crate::timed!("sim.music", self.music_tick(tick));
        // faith unkept fades a little each day
        crate::timed!("sim.devotion", {
            use crate::engine::ecs::System;
            crate::sim::religion::DevotionSystem.run(&mut self.world);
        });
        // and the weather rolls through: an event thins the fields and
        // chills or scorches the bones while it lasts
        let _lutes = crate::infra::perf::scope("sim.day.lutes_and_purses");
        // a tune in the street: anyone with a lute plays, and it lifts the
        // want for music and for joy around them
        for (i, &v) in villagers.iter().enumerate() {
            if villager::is_dead(&self.world, v) { continue; }
            let has_lute = self.world.get_component::<Satchel>(v)
                .map(|s| s.goods.get("lute").copied().unwrap_or(0) > 0).unwrap_or(false);
            if has_lute && rng.below(8) == 0 {
                self.say(v, "\u{266a}", CHAT_BUBBLE_SECS);
                if let Some(p) = self.world.get_component::<Pos>(v).copied() {
                    let pe = self.player;
                   for &l in self.villagers.iter().chain(std::iter::once(&pe)) {
                        let near = self.world.get_component::<Pos>(l)
                            .map(|lp| (lp.x - p.x).abs() + (lp.y - p.y).abs() < 3.0).unwrap_or(false);
                        if near {
                            desires::satisfy(&mut self.world, l, Want::Music, tick + 60);
                            desires::satisfy(&mut self.world, l, Want::Joy, tick + 30);
                        }
                    }
                }
                let _ = i;
            }
        }
        // every seat's purse mirrors the day's trading, not just seat 0's;
        // agent seats have no PlayerCharacter and sync_purse no-ops on them
        for s in 0..self.seats.len() {
            let pe = self.seats[s].player; sync_purse(&mut self.world, pe, false);
        }
    }

    /// The print shop's work is the village's research: each printmaker
    /// who kept the bench today - fetching its supplies counts, a day
    /// off, a rite or a shopping trip doesn't - puts a point into the
    /// next art the track offers (game::tech_tree)
    pub(crate) fn tech_tick(&mut self, tick: u64) {
        let hands = self.villagers.iter().filter(|&&v| {
            !villager::is_dead(&self.world, v)
                && self.world.get_component::<Kid>(v).is_none()
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == Vocation::Printmaker).unwrap_or(false)
                && self.world.get_component::<routine::Routine>(v)
                    .map(|r| r.day == tick && r.mode == routine::Mode::Working
                        && !matches!(r.spot, routine::Spot::Errand(_)))
                    .unwrap_or(false)
        }).count() as f32;
        if hands <= 0.0 { return; }
        if let Some(next) = crate::game::tech_tree::next_researchable(&self.tech).map(str::to_string) {
            if crate::game::tech_tree::research(&mut self.tech, &next, hands) {
                // a discovery: the bench worked it out - the village hears
                self.log.push(format!("t{}: the print shop works out {}!", tick, next));
            }
        }
    }

    /// Which workplaces show their cutaway today: a print shop whose
    /// printer kept the bench (the same Working-routine read as research)
    pub(crate) fn cutaways_tick(&mut self, tick: u64) {
        self.cutaways.clear();
        for (i, &e) in self.building_entities.iter().enumerate() {
            let Some(b) = self.world.get_component::<Building>(e) else { continue };
            if !matches!(b.kind, household::BuildingKind::Shop(Vocation::Printmaker))
                || household::derelict(b) { continue; }
            let Some(w) = b.worker else { continue };
            if self.world.get_component::<routine::Routine>(w)
                .map(|r| r.day == tick && r.mode == routine::Mode::Working)
                .unwrap_or(false) {
                self.cutaways.insert(i);
            }
        }
    }

    /// The village's tune, steered by the day the village had: burning
    /// feuds and fresh graves weigh on it, full purses and new learning
    /// lift it. When a new mood holds long enough to commit, the change
    /// is worth a line in the log
    pub(crate) fn music_tick(&mut self, tick: u64) {
        use crate::game::music;
        // the village's mood is the village's own: outlanders' purses,
        // children and graves don't lean it
        let live: Vec<(usize, Entity)> = self.villagers.iter().copied().enumerate()
            .filter(|(_, v)| !villager::is_dead(&self.world, *v) && self.site_of(*v).is_none()).collect();
        let n = live.len().max(1) as f32;
        let burning = self.world.query::<crate::game::feuds::Feud>().iter()
            .filter(|&&e| self.world.get_component::<crate::game::feuds::Feud>(e)
                .map(|f| f.status == crate::game::feuds::FeudStatus::Burning
                    && self.site_of(f.offender).is_none() && self.site_of(f.aggrieved).is_none())
                .unwrap_or(false))
            .count() as f32;
        let graves = self.villagers.iter().copied().enumerate()
            .filter(|(i, v)| self.died_on.get(*i).and_then(|d| d.map(|x| tick - x < 30))
                .unwrap_or(false) && self.site_of(*v).is_none()).count() as f32;
        let kids = live.iter().filter(|(_, v)| self.world.get_component::<Kid>(*v).is_some())
            .count() as f32;
        let purse = live.iter().filter_map(|(_, v)| self.world.get_component::<Purse>(*v)
            .map(|p| p.gold)).sum::<f32>() / n;
        let learned = self.tech.iter().filter(|t| t.discovered).count() as f32
            / self.tech.len().max(1) as f32;
        let m = music::WorldMetrics {
            growth: (kids / n * 4.0).min(1.0),
            conflict: (burning / 2.0).min(1.0),
            loss: (graves / n * 8.0).min(1.0),
            discovery: learned,
            prosperity: (purse / 50.0).min(1.0),
        };
        let before = self.world.get_component::<music::MusicState>(self.music)
            .map(|s| s.mood);
        if let Some(st) = self.world.get_component_mut::<music::MusicState>(self.music) {
            music::evolve(st, &m);
        }
        let after = self.world.get_component::<music::MusicState>(self.music)
            .map(|s| s.mood);
        if before != after {
            self.log.push(format!("t{}: the village's tune turns - {} now.",
                tick, music::recommend_mood(after.unwrap_or(music::Mood::Calm))));
            // the drop lands on the wire: the sound layer hears the turn
            let (cx, cy) = self.world.get_component::<Pos>(self.seats[0].player)
                .map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
            self.sfx.push(crate::session::SfxCue { kind: "mood", x: cx, y: cy, intensity: 0.7 });
        }
    }

    /// The weather rolls through: when the sky clears the day rolls the
    /// next event - season-weighted, most days fair - and while an event
    /// lasts its heat or cold wears on the village's health
    pub(crate) fn weather_tick(&mut self, tick: u64) {
        use crate::sim::weather::*;
        use crate::engine::ecs::System;
        WeatherSystem.run(&mut self.world);
        let (kind, duration) = self.world.get_component::<CurrentWeather>(self.weather)
            .map(|w| (w.kind, w.duration)).unwrap_or((Weather::Clear, 0));
        if kind == Weather::Clear || duration == 0 {
            let season = self.world.get_component::<climate::Climate>(self.climate)
                .map(|c| c.season).unwrap_or(climate::Season::Summer);
            let roll = self.rng.next() % 100;
            let next = match (season, roll) {
                (climate::Season::Winter, 0..=7) => Weather::Frost,
                (climate::Season::Winter, 8..=11) => Weather::Storm,
                (climate::Season::Summer, 0..=7) => Weather::HeatWave,
                (climate::Season::Summer, 8..=11) => Weather::Drought,
                (_, 0..=5) => Weather::Storm,
                (_, 6..=9) => Weather::Drought,
                _ => Weather::Clear,
            };
            let intensity = 0.4 + (self.rng.next() % 60) as f32 / 100.0;
            let days = 2 + (self.rng.next() % 2) as u32;
            if let Some(w) = self.world.get_component_mut::<CurrentWeather>(self.weather) {
                w.kind = next;
                w.intensity = if next == Weather::Clear { 0.0 } else { intensity };
                w.duration = if next == Weather::Clear { 1 } else { days };
            }
            let word = match next {
                Weather::Storm => Some("a storm rolls in - the fields will feel it"),
                Weather::Drought => Some("the ground dries and cracks - a lean spell"),
                Weather::HeatWave => Some("a heat wave settles over the village"),
                Weather::Frost => Some("a hard frost grips the fields"),
                Weather::Clear => None,
            };
            if let Some(w) = word { self.log.push(format!("t{}: {}.", tick, w)); }
            // a storm takes a standing hall with it
            if next == Weather::Storm {
                for seat in 0..self.seats.len() {
                    let oe = self.seats[seat].org;
                    if crate::game::organizing::storm_strikes_hall(&mut self.world, oe) {
                        let oname = self.world.get_component::<Organization>(oe)
                            .map(|o| o.name.clone()).unwrap_or_default();
                        let hall = household::home_of(&self.world, self.player)
                            .and_then(|h| toast_spot(&self.world, h));
                        self.log.push_at_who(format!("t{}: the storm takes {}'s roof - the hall's unfit to meet in.", tick, oname), hall, None);
                    }
                }
            }
        }
        // while an extreme holds, the village's health wears a little
        if let Some(w) = self.world.get_component::<CurrentWeather>(self.weather) {
            let drain = health_effect(w);
            if drain < 0.0 {
                for &v in self.villagers.iter().filter(|&&v|
                    !villager::is_dead(&self.world, v) && self.site_of(v).is_none())
                    .collect::<Vec<_>>() {
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        // exposure wears a body down; it doesn't kill
                        // outright, and it doesn't mend a failing one
                        h.condition = (h.condition + drain).max(0.2).min(h.condition);
                    }
                }
            }
        }
    }

    /// The priest's reading: some days an old text yields a practice the
    /// parish can use; each page of the healing arts sees the sick mend
    /// a little faster
    pub(crate) fn magic_tick(&mut self, tick: u64) {
        use crate::sim::magic;
        // a reading day comes for the priest about twice a month
        let priest = self.villagers.iter().copied().find(|&v|
            !crate::game::dungeon::unavailable(&self.world, v) && self.site_of(v).is_none()
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
        let sources = magic::ancient_sources();
        // the old library is finite - when every text's been read the
        // shelf holds no more discoveries
        let unread = self.grimoire.discoveries.len() < sources.len();
        if priest.is_some() && unread && self.rng.next() % 15 == 0 {
            let (src, domain, strength) = sources[self.grimoire.discoveries.len()];
            let d = magic::ingest_text(&mut self.grimoire, src, domain, strength);
            let use_ = magic::magic_effects(domain).first().copied().unwrap_or("useful lore");
            match priest {
                Some(p) => self.log_entity(p, format!("t{}: the priest's reading turns up {} - the parish learns to {}.",
                    tick, d.name, use_)),
                None => self.log.push(format!("t{}: the priest's reading turns up {} - the parish learns to {}.",
                    tick, d.name, use_)),
            }
        }
        // the parish physick: each healing practice the books taught
        // sees the sick mend a little faster
        let healing = self.grimoire.domains_known.get("Healing").copied().unwrap_or(0) as f32;
        if healing > 0.0 {
            for v in self.villagers.iter().copied().filter(|&v|
                !villager::is_dead(&self.world, v) && self.site_of(v).is_none()
                    && is_sick(&self.world, v)).collect::<Vec<_>>() {
                if let Some(h) = self.world.get_component_mut::<Health>(v) {
                    h.condition = (h.condition + 0.004 * healing).min(1.0);
                }
            }
        }
    }

    /// The wood's day: a stand that grows, spreads, burns and regrows.
    /// Its coverage is the fuel commons - a thin wood makes faggots dear,
    /// a thick one keeps them cheap; and a fire's worth a line when it
    /// catches and when it burns out
    pub(crate) fn forest_tick(&mut self, tick: u64) {
        use crate::sim::emergence::Cell;
        let mut was_burning = self.forest.cells.iter().flatten().any(|c| *c == Cell::Burning);
        // lightning doesn't come to the wood often - a dry day's chance
        let cur = self.world.get_component::<crate::sim::weather::CurrentWeather>(self.weather)
            .map(|w| (w.kind, w.intensity)).unwrap_or((crate::sim::weather::Weather::Clear, 0.0));
        let spark = cur.0 == crate::sim::weather::Weather::Drought
            && cur.1 > 0.7 && self.forest_rng.below(20) == 0;
        let mut ignited = false;
        if spark {
            if let Some((x, y)) = self.forest.cells.iter().enumerate()
                .flat_map(|(y, row)| row.iter().enumerate().map(move |(x, c)| (x, y, c)))
                .find(|(_, _, c)| **c == Cell::Tree).map(|(x, y, _)| (x, y)) {
                self.forest.cells[y][x] = Cell::Burning;
                ignited = true;
                self.log.push(format!("t{}: fire breaks out in the wood!", tick));
            }
        }
        self.forest.step();
        let burning = self.forest.cells.iter().flatten().any(|c| *c == Cell::Burning);
        // a struck tree that had no neighbours catches and dies in the
        // same step - the end log still owes it a line
        was_burning |= ignited;
        if was_burning && !burning {
            self.log.push(format!("t{}: the wood fire's out - bare for a while.", tick));
        }
        // the fuel price leans on the real wood's canopy - the mapped
        // stand the gatherers actually thin, not the automaton's grid
        let covers: Vec<f32> = self.land.tree_cover.values().copied().collect();
        if !covers.is_empty() {
            let cover = covers.iter().sum::<f32>() / covers.len() as f32;
            let nudge = if cover < 0.3 { 0.004 } else if cover > 0.6 { -0.004 } else { 0.0 };
            if nudge != 0.0 {
                crate::game::goods::nudge(&mut self.world, "faggots", nudge);
            }
        }
    }

    /// A bench day, wherever it's driven from - the day's loop, an agent
    /// seat, or the player's own hands: the units are remembered so the
    /// card can say what they made
    pub(in crate::session) fn bench_work(&mut self, v: Entity, tick: u64) -> (f32, Option<vocation::Level>) {
        // watch the shelves a bench day can touch: the worker's own shop,
        // and the senior's when a tasked hand sends part of the units
        // that way - then the ledger writes what actually moved
        let mut shops: Vec<Entity> = vec![];
        if let Some(c) = self.world.get_component::<Craft>(v) {
            if let Some(b) = c.workplace { shops.push(b); }
            if let Some(b) = c.tasked_by
                .and_then(|s| self.world.get_component::<Craft>(s)).and_then(|sc| sc.workplace) {
                if !shops.contains(&b) { shops.push(b); }
            }
        }
        let _ledger = crate::infra::perf::scope("sim.bench_ledger");
        let before: Vec<_> = shops.iter()
            .filter_map(|&b| self.world.get_component::<Building>(b).map(|bb| (b, bb.stock.clone())))
            .collect();
        let (made, up) = livelihood::work_day(&mut self.world, v, tick);
        if made > 0.0 {
            let e = self.produced.entry(v).or_insert((tick, 0.0));
            if e.0 != tick { *e = (tick, 0.0); }
            e.1 += made;
        }
        for (b, stock0) in before {
            let Some(after) = self.world.get_component::<Building>(b).map(|bb| bb.stock.clone()) else { continue };
            let mut delta: Vec<(String, i32)> = vec![];
            for (g, &n1) in after.iter() {
                let d = n1 as i32 - stock0.get(g).copied().unwrap_or(0) as i32;
                if d != 0 { delta.push((g.clone(), d)); }
            }
            for (g, &n0) in stock0.iter() {
                if !after.contains_key(g) { delta.push((g.clone(), -(n0 as i32))); }
            }
            if delta.is_empty() { continue; }
            let e = self.shop_flow.entry(b).or_insert((tick, vec![]));
            if e.0 != tick { *e = (tick, vec![]); }
            for (g, n) in delta {
                match e.1.iter_mut().find(|(f, _)| *f == g) {
                    Some((_, m)) => *m += n,
                    None => e.1.push((g, n)),
                }
            }
        }
        drop(_ledger);
        (made, up)
    }
}
