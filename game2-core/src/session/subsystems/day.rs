//! The day: livelihood_tick runs each subsystem's daily tick in order, plus the town's own business (work, errands, taxes).

use crate::session::*;

impl Game {
    /// The town's business each day: work, errands, the tax round, the
    /// trader's restock, the odd minstrel, a tune in the street
    pub fn livelihood_tick(&mut self, tick: u64) {
        let prelude = crate::infra::perf::scope("sim.day.prelude");
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
        self.derelict.clear();
        for (i, e) in self.building_entities.iter().enumerate() {
            if self.world.get_component::<Building>(*e).map(household::derelict).unwrap_or(false) {
                self.derelict.insert(i);
            }
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
        crate::timed!("sim.routine_dawn", self.routine_dawn(tick, &mut rng));
        // and folk who've gotten to know the Wanderer ask for small jobs
        crate::timed!("sim.quests", self.quest_dawn(tick, &mut rng));
        let folk = crate::infra::perf::scope("sim.day.villagers");
        let villagers = self.villagers.clone();
        for i in 0..villagers.len() {
            let v = villagers[i];
            if crate::game::dungeon::unavailable(&self.world, v) { continue; }
            // hunger comes first: a starving person abandons any errand
            // to look for food - even one already under way. And looking
            // means walking: the store has the day's meal - except the
            // exile's road, which no belly's free to abandon
            let hungry = self.world.get_component::<Hunger>(v).map(|h| h.value < 30.0).unwrap_or(false);
            if hungry {
                self.travels.retain(|t| t.who != v || matches!(t.goal, Goal::Exile));
                // an exile walks on, belly and all - the road's the
                // only errand his sentence leaves him
                if !self.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::Exile)) {
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
                // a child off the errand list plays
                if self.world.get_component::<OddJob>(v).is_none() { self.plan_play(v, tick, &mut rng); }
                continue;
            }
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
                let voc = self.world.get_component::<Craft>(v).map(|c| vocation::vocation_name(c.vocation)).unwrap_or("trade");
                self.toasts.push_at((format!("{} is now a {}!", vill_name(i), vocation::level_name(level)), tick + 30),
                    toast_spot(&self.world, v));
                self.log.push(format!("t{}: {} is now a {} {}.", tick, vill_name(i), vocation::level_name(level).to_lowercase(), voc));
            }
        }
        drop(folk);
        // the print shop's work is the village's research: only the
        // printmakers who actually kept the bench today count
        crate::timed!("sim.tech", self.tech_tick(tick));
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
