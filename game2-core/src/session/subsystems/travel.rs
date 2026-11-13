//! Walks: sending someone somewhere (path memo, the door, forage and gather trips), advancing the walks under way, and what happens on arrival.

use crate::session::*;

impl Game {
    /// Send `who` walking toward `goal` (a person, a shop or a tile).
    /// Returns whether a way was found - otherwise nothing happens
    pub fn send(&mut self, who: Entity, goal: Goal) -> bool {
        // a body in the stocks goes nowhere until the sentence runs
        if self.world.get_component::<villager::InStocks>(who).is_some() { return false; }
        // the held take no errands - the jailer's rounds aren't theirs;
        // the dead take none either, however short the walk; and the
        // marching take none - the column's walking them. Nor one in a
        // Talk with a player: the conversation holds them where they stand
        if crate::game::dungeon::unavailable(&self.world, who) { return false; }
        if self.world.get_component::<villager::InFormation>(who).is_some() { return false; }
        if self.in_player_talk(who) { return false; }
        let Some(from) = self.world.get_component::<Pos>(who).copied() else { return false };
        let to = match &goal {
            Goal::SeekRemedy(e) | Goal::Herd(e) | Goal::HeadHome(e) => self.world.get_component::<Pos>(*e).copied()
                .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32)),
            Goal::TendGarden(i) => self.gardens.get(*i).map(|(_, t, _, _)| *t),
            Goal::Exile => {
                // the nearest edge of the world
                self.world.get_component::<Pos>(who).copied().and_then(|from| {
                    let (w, h) = (self.map.width as i32, self.map.height as i32);
                    [(2, from.y.round() as i32), (w - 3, from.y.round() as i32),
                     (from.x.round() as i32, 2), (from.x.round() as i32, h - 3)]
                        .iter().copied()
                        .filter(|&(x, y)| self.map.passable(x, y))
                        .min_by_key(|&(x, y)| ((x as f32 - from.x).abs() + (y as f32 - from.y).abs()) as i32)
                })
            }
            Goal::BuyFrom(e, _, _) | Goal::BuyAt(e, _, _) | Goal::CollectTax(e) | Goal::TalkTo(e) =>
                self.world.get_component::<Pos>(*e).copied().map(|p| (p.x.round() as i32, p.y.round() as i32)),
            Goal::Wander => Some((from.x.round() as i32, from.y.round() as i32)),
            Goal::Gather(..) => return false, // gathers go through send_gather
            Goal::Cart(_) | Goal::Levy(..) | Goal::Deliver(_) | Goal::Remit(..) | Goal::Post(_)
            | Goal::Owe(..) | Goal::Gift(..) | Goal::Escort(..) | Goal::LevyCart(_)
            | Goal::FieldPost(_) => return false, // country walks go through send_far
            Goal::Forage(_) | Goal::ForageShroom(_) => return false, // forages go through send_forage/send_shroom
            Goal::AttendRite => return false, // rites walk to their tile through send_tile_walk
            Goal::RunOddJob => self.world.get_component::<OddJob>(who)
                .and_then(|j| self.world.get_component::<Pos>(j.from).copied())
                .map(|p| (p.x.round() as i32, p.y.round() as i32)),
            Goal::FarmWork(f) => self.world.get_component::<Pos>(*f).copied()
                .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32)),
            Goal::SeasonWork(t) => self.at_the_door(t.0, t.1),
            Goal::GoStocks => self.stocks_tile.map(|(x, y)| (x, y)),
            Goal::GoGaol => self.village_gaol.or(self.castle_gaol),
            Goal::BuildWork(t) => Some(*t),
            Goal::FillBucket => self.fire_bank_walkable((from.x.round() as i32, from.y.round() as i32)),
            Goal::DouseFire(t) => self.douse_spot_walkable(*t, (from.x.round() as i32, from.y.round() as i32)),
            Goal::Eat => {
                // the village table while it's laid: folk eat at their
                // own market - or the field that stands closer to them -
                // and only a bare shelf sends them hunting for the next
                let own = self.store_of(who);
                let stocked = |f: Entity| self.world.get_component::<villager::FoodSource>(f)
                    .map(|fs| fs.servings > 0).unwrap_or(false);
                let nearest_village_source = || std::iter::once(own)
                    .chain(if own == self.market_food { self.crop_fields.iter().copied().collect() }
                        else { vec![] })
                    .filter(|&f| stocked(f))
                    .min_by_key(|&f| self.world.get_component::<Pos>(f)
                        .map(|fp| ((fp.x - from.x).abs() + (fp.y - from.y).abs()) as i32).unwrap_or(i32::MAX));
                if let Some(f) = nearest_village_source() {
                    self.world.get_component::<Pos>(f)
                        .map(|p| (p.x.round() as i32, p.y.round() as i32))
                } else {
                    self.world.query::<villager::FoodSource>().iter().copied()
                        .filter(|&f| self.world.get_component::<villager::FoodSource>(f)
                            .map(|fs| fs.servings > 0).unwrap_or(false))
                        .filter_map(|f| self.world.get_component::<Pos>(f)
                            .map(|fp| ((fp.x.round() as i32, fp.y.round() as i32),
                                (fp.x - from.x).abs() + (fp.y - from.y).abs())))
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        .map(|(p, _)| p)
                }
            }
        };
        let Some(to) = to else { return false };
        // nobody hikes halfway to the next village on an errand (the tax
        // man's round reaches the outskirts - that's his job; so does a
        // sick walk out to the herb hut)
        let legs = ((to.0 as f32 - from.x).abs() + (to.1 as f32 - from.y).abs()) as i32;
        // urgent errands get the long reach: the sick, the hungry, the
        // flock, home - and a worker's inputs, or the craft starves
        let cap = match &goal { Goal::CollectTax(_) => 24,
            Goal::SeekRemedy(_) | Goal::Herd(_) | Goal::HeadHome(_) | Goal::Eat
            | Goal::BuyFrom(..) | Goal::BuyAt(..) => 30,
            // the bucket line reaches as far as the tax round does -
            // a sworn hand takes the whole parish for a radius
            Goal::FillBucket | Goal::DouseFire(_) => {
                if self.is_brigade(who) { BUCKET_REACH * 2 } else { BUCKET_REACH }
            }
            Goal::Exile => 80, Goal::TendGarden(_) => 16, _ => 9 };
        if legs > cap { return false; }
        if to == (from.x.round() as i32, from.y.round() as i32) {
            self.travels.push(Travel { who, path: vec![to], goal });
            return true;
        }
        match self.walk_memo((from.x.round() as i32, from.y.round() as i32), to) {
            Some(mut path) => { path.push(to); self.travels.push(Travel { who, path, goal }); true }
            None => false,
        }
    }

    /// Raise an audio cue: a kind, a place, an intensity - the audio
    /// layer turns it into sound
    pub fn emit_sfx(&mut self, kind: &'static str, x: f32, y: f32, intensity: f32) {
        self.sfx.push(SfxCue { kind, x, y, intensity });
    }

    /// The search is the game's hot line - a path's the same all frame,
    /// so remember it until the next one. Frame-scoped, not day-scoped:
    /// trail strength shifts underfoot as folk walk, and a path priced
    /// on this morning's wear shouldn't stand in for the evening's
    pub(in crate::session) fn walk_memo(&mut self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        if self.path_tick != self.frame_n as u64 {
            self.path_cache.clear();
            self.path_tick = self.frame_n as u64;
        }
        if let Some(cached) = self.path_cache.get(&(from, to)) { return cached.clone(); }
        let p = self.trail_path(from, to);
        self.path_cache.insert((from, to), p.clone());
        p
    }

    /// The walk from `from` to `to` that keeps to the beaten ways: a
    /// worn trail or a road costs less than wild grass, so folk stick
    /// to the paths the village actually uses instead of cutting
    /// fresh lines over the fields.
    pub(in crate::session) fn trail_path(&self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        Self::trail_path_on(&self.map, &self.trails, &self.flood, from, to)
    }

    /// `trail_path` over its parts, for callers holding field borrows
    /// that a whole-`self` method call would conflict with.
    pub(in crate::session) fn trail_path_on(map: &TileMap, worn_map: &crate::engine::det::HashMap<(i32, i32), Trail>, flood: &crate::game::flood::Flood, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        // the cheapest a tile can be: a trail worn to saturation
        const FLOOR: f32 = 1.0 - TRAIL_K * TRAIL_PARAMS.saturation;
        crate::timed!("sim.walk_path", map.walk_path_weighted(from, to, FLOOR, |x, y| {
            // a tile under the water doesn't take a step - the flood
            // routes folk round the wetting, not through it
            if flood.flooded(x, y) { return f32::INFINITY; }
            // the ford takes the wade, worn or not: a well-trodden
            // shallows is still a river to cross (#1074)
            if map.get(x, y) == Some(Tile::Ford) { return FORD_WADE; }
            // roads aren't free for the planner: advance_travels prices
            // a step by the trail actually recorded on the tile, so a
            // road only earns its keep once folk have worn it in too
            let worn = worn_map.get(&(x, y)).map(|t| t.strength).unwrap_or(0.0);
            trails::movement_cost(1.0, worn, TRAIL_K)
        }))
    }

    /// A passable tile right next to a building's (occupied) one: where a
    /// caller at the door actually stands, south side preferred
    pub(in crate::session) fn at_the_door(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        if self.map.passable(x, y) { return Some((x, y)); }
        [(0, 1), (-1, 0), (1, 0), (0, -1), (-1, 1), (1, 1), (-1, -1), (1, -1)]
            .iter().map(|(dx, dy)| (x + dx, y + dy))
            .find(|&(qx, qy)| self.map.passable(qx, qy))
    }

    /// Walk `who` to tile `t` carrying `goal` (shared by the forage sends)
    pub(in crate::session) fn send_tile_walk(&mut self, who: Entity, t: (i32, i32), goal: Goal, max_d: i32) -> bool {
        if crate::game::dungeon::unavailable(&self.world, who) || self.in_player_talk(who) { return false; }
        let Some(from) = self.world.get_component::<Pos>(who).copied() else { return false };
        if (t.0 - from.x.round() as i32).abs() + (t.1 - from.y.round() as i32).abs() > max_d { return false; }
        match self.walk_memo((from.x.round() as i32, from.y.round() as i32), t) {
            Some(mut path) => { path.push(t); self.travels.push(Travel { who, path, goal }); true }
            None => false,
        }
    }

    /// Send `who` out to pick herb patch `pi`
    pub fn send_forage(&mut self, who: Entity, pi: usize) -> bool {
        let Some(t) = self.herb_patches.get(pi).map(|p| p.tile) else { return false };
        self.send_tile_walk(who, t, Goal::Forage(pi), 16)
    }

    /// Send `who` out to gather at mushroom patch `pi`
    pub fn send_shroom(&mut self, who: Entity, pi: usize) -> bool {
        let Some(t) = self.mush_patches.get(pi).map(|p| p.tile) else { return false };
        self.send_tile_walk(who, t, Goal::ForageShroom(pi), 16)
    }

    /// Send `who` to gather `good` at tile `t`
    pub fn send_gather(&mut self, who: Entity, good: &'static str, t: (i32, i32)) -> bool {
        // the held take no errands - a forage is no freer a walk than
        // the others - and one in a Talk holds their spot for it
        if crate::game::dungeon::unavailable(&self.world, who) || self.in_player_talk(who) { return false; }
        let Some(from) = self.world.get_component::<Pos>(who).copied() else { return false };
        // gathers get the forage-errand reach: wood and stone have no
        // other source, and the craft can't wait on a near tree
        if (t.0 - from.x.round() as i32).abs() + (t.1 - from.y.round() as i32).abs() > 16 { return false; }
        match self.trail_path((from.x.round() as i32, from.y.round() as i32), t) {
            Some(mut path) => { path.push(t); self.travels.push(Travel { who, path, goal: Goal::Gather(good, t) }); true }
            None => false,
        }
    }

    /// Step `p` along `path` for up to `budget` tiles: every waypoint the
    /// budget reaches is landed on (not leapt over, however fast the
    /// clock), the rest goes toward the next. True when the path ran out.
    pub(in crate::session) fn follow_path(p: &mut Pos, path: &mut Vec<(i32, i32)>, mut budget: f32) -> bool {
        while let Some(&(wx, wy)) = path.first() {
            if budget <= 0.0 { return false; }
            let (nx, ny) = (wx as f32, wy as f32);
            let (dx, dy) = (nx - p.x, ny - p.y);
            let d = (dx * dx + dy * dy).sqrt();
            if d <= budget { p.x = nx; p.y = ny; path.remove(0); budget -= d; }
            else { p.x += dx / d * budget; p.y += dy / d * budget; return false; }
        }
        true
    }

    /// One frame of every walk under way: step along the path, and do the
    /// errand when it runs out
    pub fn advance_travels(&mut self) {
        let mut arrived: Vec<(Entity, Goal)> = vec![];
        let pace = self.walk_pace();
        // the country's carters and stewards go by cart and horse, and a
        // knight takes his mount for the long roads
        let riders: Vec<Entity> = self.sites.iter().filter_map(|s| s.carrier).collect();
        let mounted: Vec<Entity> = self.world.query::<villager::Mounted>().iter().copied().collect();
        for t in self.travels.iter_mut() {
            if t.path.is_empty() { continue; }
            // the dead finish no errands - the errand ends where they fell
            if villager::is_dead(&self.world, t.who) { t.path.clear(); continue; }
            // one in a Talk with a player stands for it: the walk waits
            // where it is and picks back up when the talk lets go
            if self.world.get_component::<villager::InPlayerTalk>(t.who).is_some() { continue; }
            // a well-worn path is quicker underfoot - trails::movement_cost
            // is the one place the worn-way discount is worked out
            let worn = self.trails.get(&t.path[0]).map(|tr| tr.strength).unwrap_or(0.0);
            // a lame leg covers less ground in a day; the blind know
            // every path - sight is for the stalls, not the roads
            let slow = self.world.get_component::<villager::Condition>(t.who)
                .filter(|c| self.tick < c.until)
                .map(|c| match c.kind { "a lame leg" => 0.6, "no sight" => 1.0, _ => 0.7 }).unwrap_or(1.0);
            let ride = if riders.contains(&t.who) || mounted.contains(&t.who) { country::RIDE_PACE } else { 1.0 };
            // a ford tile in the walk costs the wade itself (#1074)
            let wade = if self.map.get(t.path[0].0, t.path[0].1) == Some(Tile::Ford) { WADE_SLOWDOWN } else { 1.0 };
            let stride = MEETING_WALK * pace / trails::movement_cost(1.0, worn, TRAIL_K) * slow * ride * wade;
            let Some(p) = self.world.get_component_mut::<Pos>(t.who) else { t.path.clear(); continue };
            // spend the frame's stride on every waypoint it reaches: a fast
            // clock still walks the tiles between, it doesn't leap them
            let mut budget = stride;
            loop {
                let Some(&(wx, wy)) = t.path.first() else { arrived.push((t.who, t.goal.clone())); break };
                let (nx, ny) = (wx as f32, wy as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                let (stepx, stepy, landed) = if d <= budget { (nx, ny, true) }
                    else { (p.x + dx / d * budget, p.y + dy / d * budget, false) };
                // never step somewhere they can't stand: a wall in the way
                // aborts the errand rather than leaving them walking into it
                // (and starving on the spot) forever - an exile aborting at
                // the rocks still takes the road, else he stands banished
                // and breathing at the border until hunger does it
                if !self.map.passable(stepx.round() as i32, stepy.round() as i32) {
                    if matches!(t.goal, Goal::Exile) { arrived.push((t.who, t.goal.clone())); }
                    t.path.clear(); break;
                }
                let ot = (p.x.round() as i32, p.y.round() as i32);
                p.x = stepx; p.y = stepy;
                let nt = (p.x.round() as i32, p.y.round() as i32);
                if nt != ot {
                    let tr = self.trails.entry(nt).or_insert(Trail::wild());
                    trails::step_trail(tr, 1, 0.0, &TRAIL_PARAMS);
                }
                if !landed { break; }
                t.path.remove(0);
                budget -= d;
            }
        }
        self.travels.retain(|t| !t.path.is_empty());
        for (who, goal) in arrived { self.arrive(who, goal); }
        // the mount goes when the ride does: arrived, cancelled, or
        // otherwise no longer under way
        for &m in &mounted {
            if !self.travels.iter().any(|t| t.who == m) {
                self.world.remove_component::<villager::Mounted>(m);
            }
        }
        // no ride, no flag: whoever's off the road seeks food again
        let stray: Vec<Entity> = self.world.query::<villager::OnTheRoad>().iter().copied()
            .filter(|&w| !self.travels.iter().any(|t| t.who == w
                && matches!(t.goal, Goal::Remit(..) | Goal::LevyCart(_))))
            .collect();
        for w in stray { self.world.remove_component::<villager::OnTheRoad>(w); }
    }

    /// What happens when a walk reaches its end: the trade, the
    /// collection, the gather, the delivery
    pub fn arrive(&mut self, who: Entity, goal: Goal) {
        let tick = self.tick;
        match goal {
            Goal::BuyFrom(seller, good, qty) => {
                // seller may be a person (satchel) or a building (the stall)
                let mut paid = if self.world.get_component::<Building>(seller).is_some() {
                    household::buy_at_shop(&mut self.world, seller, who, good, qty)
                } else {
                    livelihood::trade_person(&mut self.world, who, seller, good, qty)
                };
                // short of coin, a neighbour still swaps: the buyer offers
                // out of the satchel toward what the ask is worth, and a
                // few coppers to boot squares it
                if paid.is_none() && self.world.get_component::<Building>(seller).is_none() {
                    let want_worth = goods::worth_at(&self.world, good) * qty as f32;
                    let offer = livelihood::pick_offer(&self.world, who, want_worth, good);
                    if !offer.is_empty() {
                        let offer_refs: Vec<(&str, u32)> = offer.iter()
                            .map(|(g, q)| (g.as_str(), *q)).collect();
                        if let Some(deal) = livelihood::barter_person(
                            &mut self.world, who, seller, &[(good, qty)], &offer_refs) {
                            paid = Some(deal.boot);
                            let an = self.villagers.iter().position(|v| *v == who).map(vill_name);
                            let bn = self.villagers.iter().position(|v| *v == seller).map(vill_name);
                            if let (Some(a), Some(b)) = (an, bn) {
                                self.log_entity_ev(
                                    EvTag::new(Thread::Economy, Valence::Neutral, Sev::Minor),
                                    who,
                                    format!(
                                        "t{}: {} swaps {} with {}.",
                                        tick,
                                        a,
                                        livelihood::barter_terms(
                                            &offer,
                                            &[(good.into(), qty)],
                                            deal.boot
                                        ),
                                        b
                                    ),
                                );
                            }
                            // the deal lands on the bond: fair swaps warm a
                            // little, a short sack leaves a mark, generosity
                            // reads as aid
                            use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
                            let (kind, who_records) = if deal.fairness < 0.85 {
                                (InteractionKind::Conflict, seller)
                            } else if deal.fairness > 1.15 {
                                (InteractionKind::Aid, who)
                            } else {
                                (InteractionKind::Conversation, who)
                            };
                            record_interaction(&mut self.world, who_records,
                                Interaction { other: if who_records == who { seller } else { who },
                                    kind, intensity: (1.0 - deal.fairness).abs().clamp(0.2, 1.0) });
                        }
                    }
                }
                // trade with the road's accounts loses a little to the
                // reckoning - unless the village has a clerk for it
                if Some(seller) == self.trader_stall && vocation::imported(good)
                    && self.numeracy != Numeracy::Positional && self.translator.is_none() {
                    self.reckonings += 1;
                    // the tally leans on what was actually paid - a shelf
                    // emptied just now doesn't swell the reckoning's cut
                    let skim = paid.unwrap_or(0.0) * 0.1;
                    if let Some(p) = self.world.get_component_mut::<Purse>(who) {
                        p.gold = (p.gold - skim).max(0.0);
                    }
                    if self.reckonings % 3 == 0 {
                        desires::add_grievance(&mut self.world, who, desires::Problem::ShortChanged, tick + 40);
                        if self.reckonings == 3 {
                            self.log_entity_ev(
                                EvTag::new(Thread::Economy, Valence::Negative, Sev::Minor),
                                who,
                                format!(
                                    "t{}: the reckoning didn't add up - he counted it his way.",
                                    tick
                                ),
                            );
                        }
                    }
                    // enough of these and a clerk steps up to keep the
                    // tallies straight - the dragoman problem solved
                    if self.reckonings >= 8 {
                        let clerk = self.villagers.iter().copied().find(|&v|
                            self.world.get_component::<Craft>(v)
                                .map(|c| c.vocation == Vocation::Printmaker || c.vocation == Vocation::Priest)
                                .unwrap_or(false)
                            && !crate::game::dungeon::unavailable(&self.world, v));
                        if let Some(c) = clerk {
                            self.translator = Some(c);
                            self.translator_days = 0;
                            let name = self.name_of(c);
                            self.log_entity_ev(EvTag::new(Thread::Economy, Valence::Positive, Sev::Minor), c, format!("t{}: {} keeps the tallies straight now - no more foreign reckonings.", tick, name));
                        }
                    }
                }
                // a dab of perfume: the buyer smells of roses for days
                if good == "perfume" {
                    if let Some(s) = self.world.get_component_mut::<Smell>(who) {
                        s.kind = "roses"; s.intensity = 0.8;
                    } else {
                        self.world.add_component(who, Smell { kind: "roses", intensity: 0.8 });
                    }
                }
            }
            Goal::BuyAt(shop, good, want) => {
                if household::buy_at_shop(&mut self.world, shop, who, good, 1).is_some() {
                    if let Some(w) = want {
                        desires::satisfy(&mut self.world, who, w, tick + 60);
                        // consumable goods are enjoyed, not kept
                        if matches!(good, "bread" | "faith" | "security" | "pamphlets" | "garments"
                            | "lodging") {
                            if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                                if let Some(q) = s.goods.get_mut(good) { *q = q.saturating_sub(1); }
                            }
                        }
                    }
                    if vocation::is_drink(good) {
                        // a cup goes down at the counter, not home in the
                        // bag: company, a small lift - and sometimes the
                        // morning after
                        desires::satisfy(&mut self.world, who, Want::Drink, tick + 60);
                        desires::satisfy(&mut self.world, who, Want::Joy, tick + 15);
                        if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                            if let Some(q) = s.goods.get_mut(good) { *q = q.saturating_sub(1); }
                        }
                        // the keeper hears everyone's business - a cup
                        // buys a chat, and the alehouse earns its name
                        if let Some(k) = self.world.get_component::<Building>(shop)
                            .and_then(|b| b.worker).filter(|&k| k != who) {
                            relationships::record_interaction(&mut self.world, who, relationships::Interaction {
                                other: k, kind: relationships::InteractionKind::Conversation,
                                intensity: CHAT_INTENSITY,
                            });
                        }
                        if self.rng.below(100) < 15 {
                            self.world.add_component(who, villager::Drunk { until: tick + 2 });
                            self.say(who, "One more won't hurt.", CHAT_BUBBLE_SECS);
                        }
                    }
                }
            }
            Goal::Gather(good, tile) => {
                // the land keeps score: the canopy thins, the pit fills
                let qty = match good {
                    "wood" => {
                        let cover = self.land.tree_cover.get(&tile).copied().unwrap_or(0.6);
                        if let Some(c) = self.land.tree_cover.get_mut(&tile) { *c = (*c - 0.12).max(0.0); }
                        crate::sim::environment::wood_yield(cover)
                    }
                    "stone" => {
                        let y = crate::sim::environment::stone_yield(self.land.pit_water);
                        self.land.pit_water = (self.land.pit_water + 0.03).min(1.0);
                        if self.land.pit_water >= 1.0 && self.pit == Some(tile) {
                            self.log.push_ev_at_who(
                                EvTag::new(Thread::Economy, Valence::Negative, Sev::Minor),
                                format!(
                                    "t{}: the pit's drowned - no stone comes out of a pond.",
                                    tick
                                ),
                                Some((tile.0 as f32, tile.1 as f32)),
                                None,
                            );
                        }
                        y
                    }
                    _ => 2,
                };
                if qty > 0 {
                    if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                        *s.goods.entry(good.to_string()).or_insert(0) += qty;
                    }
                    // and a full bag walks home: nobody camps at the
                    // woods with the day's gather on their back
                    let home = household::home_of(&self.world, who);
                    let door = home.and_then(|h| self.world.get_component::<Pos>(h).copied())
                        .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32));
                    if let (Some(h), Some(d)) = (home, door) {
                        self.send_far(who, d, Goal::HeadHome(h));
                    }
                }
            }
            Goal::CollectTax(target) => {
                let Some(c) = self.collector else { return };
                if villager::is_dead(&self.world, target) { return; }
                // the lord's reply turns on his estimate of the org:
                // a stranger he squeezes, a power he negotiates, a force
                // he passes by
                if target == self.player && self.lord_esteem > 0.6 {
                    self.log_entity_ev(EvTag::new(Thread::Law, Valence::Positive, Sev::Minor), target, format!("t{}: the collector passes you by - the lord knows an organized man when he sees one.", tick));
                    return;
                }
                let cut = livelihood::collect_tax(&mut self.world, c, target, tick);
                // the levy rides the books as the lord's, not the man's
                self.takings += cut;
                let taken = cut
                    * if target == self.player && self.lord_esteem > 0.3 {
                        self.log_entity_ev(
                            EvTag::new(Thread::Law, Valence::Positive, Sev::Notable),
                            target,
                            format!(
                                "t{}: the lord learns to tread lightly - the cut is halved.",
                                tick
                            ),
                        );
                        0.5
                    } else { 1.0 };
                // the assize's share on the barrel: a shop stocked with
                // drink pays a cut on what it pours - the alehouse, the
                // brewery and the still are the lord's best takings
                let excise = self.world.get_component::<Craft>(target)
                    .and_then(|cr| cr.workplace)
                    .and_then(|b| self.world.get_component::<Building>(b))
                    .map(|b| b.stock.iter()
                        .filter(|(g, _)| vocation::is_drink(g))
                        .map(|(g, q)| goods::worth_at(&self.world, g) * *q as f32).sum::<f32>())
                    .map(|v| (v * livelihood::DRINK_EXCISE).round())
                    .unwrap_or(0.0);
                if excise > 0.0 {
                    // he takes what the purse can bear, no more - the
                    // lord can't coin what isn't there
                    let paid = self.world.get_component::<Purse>(target)
                        .map(|p| p.gold.min(excise)).unwrap_or(0.0);
                    if let Some(p) = self.world.get_component_mut::<Purse>(target) { p.gold -= paid; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(c) { p.gold += paid; }
                    self.takings += paid;
                    let name = self.villagers.iter().position(|v| *v == target)
                        .map(|i| vill_name(i)).unwrap_or("you");
                    self.log_entity_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Minor), target, format!("t{}: the collector counts {}'s barrels - the ale excise comes to {:.0} gold.", tick, name, excise));
                }
                // a grant nobody kept watch over is clawed back: if the
                // campaign that won it has lapsed, the collector takes the
                // full cut though the grant still stands on paper
                let seat_orgs: Vec<Entity> = self.seats.iter().map(|s| s.org).collect();
                for o in seat_orgs {
                    if let Some(camp) = crate::game::campaign::campaign_of(&self.world, o) {
                        if crate::game::reactions::hollow_if_unguarded(&mut self.world, camp, tick) {
                            self.log.push_ev(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!("t{}: the lord's grant still stands on paper - but nobody kept watch, and the collector takes the full cut. Meet again and put your demands to the Reeve to win it back.", tick));
                        }
                    }
                }
                // a grant carried home lightens the round for the cause's
                // own - wherever the grant was won, not just seat zero's
                // fight: half the cut back and the taxing doesn't count;
                // a partial grant gives a quarter back and the grievance
                // stands; a hollowed one gives nothing
                let relief = self.seats.iter()
                    .filter(|s| self.world.get_component::<Organization>(s.org)
                        .map(|o| o.members.contains(&target)).unwrap_or(false))
                    .filter_map(|s| self.world.get_component::<crate::game::reactions::LordGrant>(s.org))
                    .filter(|g| g.issue == "the taxes")
                    .map(|g| g.relief(tick))
                    .fold(0.0f32, f32::max);
                let granted = relief > 0.0;
                if granted {
                    let back = taken * relief;
                    if let Some(p) = self.world.get_component_mut::<Purse>(target) { p.gold += back; }
                    if relief >= crate::game::reactions::FULL_RELIEF {
                        crate::game::desires::lift_grievance(&mut self.world, target, crate::game::desires::Problem::Taxed);
                    }
                }
                self.say(target, "The lord's share, if you please.", CHAT_BUBBLE_SECS);
                if target == self.player {
                    self.log_entity_ev(
                        EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                        target,
                        format!(
                            "t{}: the tax collector takes his cut from you ({:.0} gold).",
                            tick, taken
                        ),
                    );
                }
                if granted {
                    let name = self.villagers.iter().position(|v| *v == target).map(|i| vill_name(i)).unwrap_or("them");
                    if relief >= crate::game::reactions::FULL_RELIEF {
                        self.log_entity_ev(
                            EvTag::new(Thread::Law, Valence::Positive, Sev::Minor),
                            target,
                            format!(
                                "t{}: the lord's grant covers {} - half the cut comes back.",
                                tick, name
                            ),
                        );
                    } else {
                        self.log_entity_ev(EvTag::new(Thread::Law, Valence::Positive, Sev::Minor), target, format!("t{}: the lord's part-grant covers {} - a quarter of the cut comes back.", tick, name));
                    }
                }
            }
            Goal::TalkTo(e) => {
                // whoever they meant to greet is gone, behind bars, or
                // down the road and dusted - no conversation at any of them
                if crate::game::dungeon::unavailable(&self.world, e)
                    || self.world.get_component::<Pos>(e).is_none() {
                    self.say(who, "Oh.", TALK_BUBBLE_SECS);
                    // a visit promised to someone gone is let go
                    if self.world.get_component::<crate::game::organizing::RoundBack>(who)
                        .map(|rb| rb.to == e).unwrap_or(false) {
                        self.world.remove_component::<crate::game::organizing::RoundBack>(who);
                    }
                } else if Some(e) == self.matchmaker && who != e {
                    // a lonely heart comes to sit with the heart-helper
                    self.match_attempt(who, tick, Some(e), "the heart-helper");
                } else if self.ink_boiler.as_ref().map(|b| b.e) == Some(e) && who != e {
                    // crusts for the pot, and a fried taste for carrying them
                    desires::satisfy(&mut self.world, who, Want::GoodFood, tick + 30);
                    self.say(who, "The crusts come back fried.", CHAT_BUBBLE_SECS);
                } else {
                    // a promised visit only counts if you're actually
                    // there: the walk was set out for where you stood that
                    // morning - an empty doorstep is noted and tried again,
                    // a few times over before they stop looking
                    let visit = self.world.get_component::<crate::game::organizing::RoundBack>(who)
                        .filter(|rb| rb.to == e).copied();
                    if let Some(rb) = visit {
                        let near = match (self.world.get_component::<Pos>(who), self.world.get_component::<Pos>(e)) {
                            (Some(a), Some(b)) => (a.x - b.x).abs() + (a.y - b.y).abs() <= VISIT_RANGE,
                            _ => false,
                        };
                        if !near {
                            if rb.misses + 1 >= VISIT_MISSES {
                                self.world
                                    .remove_component::<crate::game::organizing::RoundBack>(who);
                                let name = self
                                    .villagers
                                    .iter()
                                    .position(|v| *v == who)
                                    .map(|i| vill_name(i))
                                    .unwrap_or("someone");
                                self.log_entity_ev(
                                    EvTag::new(Thread::Law, Valence::Positive, Sev::Minor),
                                    who,
                                    format!(
                                        "t{}: {} stops looking for you - the road's run out.",
                                        tick, name
                                    ),
                                );
                            } else if let Some(rb) = self
                                .world
                                .get_component_mut::<crate::game::organizing::RoundBack>(who)
                            {
                                rb.misses += 1;
                                rb.after = tick + 1;
                            }
                            return;
                        }
                        self.world.remove_component::<crate::game::organizing::RoundBack>(who);
                    }
                    // stand and chat: a conversation, a step toward a tie
                    relationships::record_interaction(&mut self.world, who, relationships::Interaction {
                        other: e, kind: relationships::InteractionKind::Conversation, intensity: CHAT_INTENSITY,
                    });
                    self.say(who, "Good day to you.", TALK_BUBBLE_SECS);
                    // an agent seat's chat is a talk like the Wanderer's:
                    // standing beside the one the job was for hands it
                    // over - the seat's own quest, not the villager's
                    if let Some(si) = self.seats.iter().position(|s| s.player == who) {
                        if let Some(note) = self.quest_talk_done(si, e) {
                            self.log.push_ev(
                                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
                                note,
                            );
                        }
                    }
                }
            }
            Goal::Wander | Goal::AttendRite => {}
            Goal::Eat => {
                // a real meal off the nearest table - if the day's hunger
                // system already fed them mid-errand, no second helping
                let still_hungry = self.world.get_component::<Hunger>(who)
                    .map(|h| h.value < 100.0).unwrap_or(false);
                let table = self.world.get_component::<Pos>(who).copied()
                    .map(|p| self.world.query::<villager::FoodSource>().iter().copied()
                        .filter(|&f| self.world.get_component::<villager::FoodSource>(f)
                            .map(|fs| fs.servings > 0).unwrap_or(false))
                        .filter(|&f| self.world.get_component::<Pos>(f)
                            .map(|fp| (fp.x - p.x).abs() + (fp.y - p.y).abs() < 3.0).unwrap_or(false))
                        .next());
                if still_hungry {
                    if let Some(Some(t)) = table {
                        let served = self.world.get_component_mut::<villager::FoodSource>(t)
                            .map(|f| { f.servings -= 1; true }).unwrap_or(false);
                        if served {
                            if let Some(h) = self.world.get_component_mut::<Hunger>(who) { h.value = 100.0; }
                        }
                    }
                }
            }
            Goal::Herd(_) => {}
            Goal::HeadHome(_) => {}
            Goal::FillBucket => self.fire_bucket_fill(who),
            Goal::DouseFire(t) => self.fire_bucket_douse(who, t),
            Goal::Cart(si) => self.arrive_cart(who, si),
            Goal::Levy(castle, town) => self.arrive_levy(who, castle, town),
            Goal::Deliver(castle) => self.arrive_deliver(who, castle),
            Goal::Remit(castle, gold) => self.arrive_remit(who, castle, gold),
            Goal::Post(stop) => self.arrive_post(who, stop),
            Goal::Owe(from, to, gold) => self.arrive_owe(who, from, to, gold),
            Goal::Gift(from, to) => self.arrive_gift(who, from, to),
            Goal::Escort(from, to) => self.arrive_escort(who, from, to),
            Goal::LevyCart(castle) => self.arrive_levy_cart(who, castle),
            // the post's marker already has them held to the town - the
            // arrival's just the walk done
            Goal::FieldPost(_) => {}
            Goal::TendGarden(gi) => {
                // weeding and picking: the bed yields into the house's
                // stock, and the tending itself is a small joy
                if let Some((home, _, g, kitchen)) = self.gardens.get_mut(gi) {
                    if *g >= 1.0 {
                        *g = 0.0;
                        // a kitchen bed comes up roots, potherbs and peas
                        // in turn; a herb bed keeps its flowers
                        let good = if *kitchen {
                            ["roots", "potherbs", "peas"][(tick / 7 % 3) as usize]
                        } else if tick % 2 == 0 { "marigold" } else { "feverfew" };
                        if let Some(b) = self.world.get_component_mut::<Building>(*home) {
                            household::add_stock(b, good, 1);
                        }
                        let home = *home;
                        if let Some(b) = self.world.get_component::<Building>(home) {
                            for &o in &b.occupants.clone() {
                                desires::satisfy(&mut self.world, o, Want::Joy, tick + 20);
                            }
                        }
                    }
                }
            }
            Goal::Exile => {
                // at the world's edge they don't come back - unless the
                // castle's keeping them: the dungeon's a held state, and
                // the org may yet ransom them home
                let name = self.villagers.iter().position(|v| *v == who).map(|i| vill_name(i)).unwrap_or("someone");
                if self.dungeon.iter().any(|(e, _)| *e == who) {
                    self.log_entity_ev(
                        EvTag::new(Thread::Law, Valence::Negative, Sev::Notable),
                        who,
                        format!(
                            "t{}: {} is held at the castle - the org may yet ransom them.",
                            tick, name
                        ),
                    );
                } else {
                    self.world.add_component(who, villager::Dead);
                    self.log_entity_ev(
                        EvTag::new(Thread::Lifecycle, Valence::Negative, Sev::Major),
                        who,
                        format!("t{}: {} is driven out - the road takes them.", tick, name),
                    );
                }
            }
            Goal::ForageShroom(pi) => {
                self.shroom_breaks.push((who, tick));
                if let Some(p) = self.mush_patches.get_mut(pi) { p.next_ready = tick + 60; }
                let Some(kind) = self.mush_patches.get(pi).map(|p| p.kind) else { return };
                self.shroom_take(who, pi, kind, tick);
            }
            Goal::Forage(pi) => {
                // picked clean for a season; two bundles dry on the hut's beams
                if let Some(p) = self.herb_patches.get_mut(pi) { p.next_ready = tick + 90; }
                let herb = self.herb_patches.get(pi).map(|p| p.herb);
                if let (Some(herb), Some(hut)) = (herb, self.herb_hut) {
                    if let Some(b) = self.world.get_component_mut::<Building>(hut) {
                        household::add_stock(b, herb, 2);
                    }
                }
            }
            Goal::SeekRemedy(_) => {
                if villager::is_dead(&self.world, who) { return; }
                self.remedy_visits.push((who, tick));
                let Some(her) = self.herbologist else { return };
                if villager::is_dead(&self.world, her) { return; }
                let illness = disease_of(&self.world, who).map(|(n, _)| n);
                let hut = self.herb_hut.unwrap_or(0);
                let efficacy = herbology::best_remedy(&mut self.world, her)
                    .or_else(|| herbology::brew_from_stock(&mut self.world, her, hut));
                match efficacy {
                    Some(eff) => {
                        herbology::apply_efficacy(&mut self.world, who, eff);
                        let fee = (2.0 + eff * 8.0).min(8.0)
                            .min(self.world.get_component::<Purse>(who).map(|p| p.gold).unwrap_or(0.0));
                        if let Some(p) = self.world.get_component_mut::<Purse>(who) { p.gold -= fee; }
                        if let Some(p) = self.world.get_component_mut::<Purse>(her) { p.gold += fee; }
                        let patient = self.villagers.iter().position(|v| *v == who).map(|i| vill_name(i)).unwrap_or("someone");
                        self.say(her, "Steep it and rest - it'll turn.", CHAT_BUBBLE_SECS);
                        self.cured += 1;
                        self.log_entity_ev(
                            EvTag::new(Thread::Illness, Valence::Positive, Sev::Minor),
                            who,
                            format!(
                                "t{}: {} treats {}'s {}.",
                                tick,
                                vill_name(HERB_IDX),
                                patient,
                                illness.unwrap_or_else(|| "illness".into())
                            ),
                        );
                    }
                    None => {
                        self.say(
                            her,
                            "Nothing in the hedge for that today.",
                            CHAT_BUBBLE_SECS,
                        );
                    }
                }
            }
            Goal::SeasonWork(_) => { /* the season's walk lands them at the village - they stay till the Laborer mark says home */ }
            Goal::GoStocks => {
                // in the stocks: pinned for the sentence's days
                if let Some(t) = self.stocks_tile {
                    if let Some(p) = self.world.get_component_mut::<Pos>(who) {
                        p.x = t.0 as f32 + 0.5; p.y = t.1 as f32 + 0.5;
                    }
                }
                self.world.add_component(who, villager::InStocks { until: self.tick + 7 });
            }
            Goal::GoGaol => {
                // the gaol takes them: the Pos holds at the cell tile
                let tile = self.village_gaol.or(self.castle_gaol);
                if let Some(t) = tile {
                    if let Some(p) = self.world.get_component_mut::<Pos>(who) {
                        p.x = t.0 as f32 + 0.5; p.y = t.1 as f32 + 0.5;
                    }
                }
            }
            Goal::BuildWork(t) => {
                // the laborer reaches the site: the crew counts them on it
                if let Some(p) = self.world.get_component_mut::<Pos>(who) {
                    p.x = t.0 as f32 + 0.5; p.y = t.1 as f32 + 0.5;
                }
            }
            Goal::FarmWork(f) => {
                // stood at the field: the day's work runs under the KidWork
                // marker - the kid_labor tick pays it out at dusk
                if self.world.get_component::<vocation::KidWork>(who)
                    .map(|w| w.field == f).unwrap_or(false) {
                    if let Some(p) = self.world.get_component::<Pos>(f).copied() {
                        if let Some(pos) = self.world.get_component_mut::<Pos>(who) {
                            pos.x = p.x + 0.5; pos.y = p.y + 0.5;
                        }
                    }
                }
            }
            Goal::RunOddJob => {
                let Some(job) = self.world.get_component_mut::<OddJob>(who)
                    .map(|j| OddJob { hirer: j.hirer, good: j.good.clone(), qty: j.qty, from: j.from, to: j.to, pay: j.pay, loaded: j.loaded })
                else { return };
                if !job.loaded {
                    // at the source: the hirer already paid for it - goods
                    // into the kid's satchel, then walk them to the hirer
                    let took = if let Some(b) = self.world.get_component_mut::<Building>(job.from) {
                        household::take_stock(b, &job.good, job.qty)
                    } else {
                        self.world.get_component_mut::<Satchel>(job.from)
                            .and_then(|s| s.goods.get_mut(&job.good).map(|q| { let n = (*q).min(job.qty); *q -= n; n }))
                            .unwrap_or(0)
                    };
                    if took == 0 { self.world.remove_component::<OddJob>(who); return; }
                    if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                        *s.goods.entry(job.good.clone()).or_insert(0) += took;
                    }
                    if let Some(j) = self.world.get_component_mut::<OddJob>(who) { j.loaded = true; }
                    // walk the second leg to the hirer
                    if let (Some(from_p), Some(to_p)) = (
                        self.world.get_component::<Pos>(who).copied(),
                        self.world.get_component::<Pos>(job.to).copied()) {
                        let to = (to_p.x.round() as i32, to_p.y.round() as i32);
                        if let Some(mut path) = self.trail_path((from_p.x.round() as i32, from_p.y.round() as i32), to) {
                            path.push(to);
                            self.travels.push(Travel { who, path, goal: Goal::RunOddJob });
                        }
                    }
                } else {
                    // delivered: goods to the hirer, pay to the kid. A hirer
                    // who died waiting gets no satchel and pays no coin - the
                    // goods land in the place's stock if there is one
                    let qty = self.world.get_component_mut::<Satchel>(who)
                        .and_then(|s| s.goods.get_mut(&job.good).map(|q| { let n = (*q).min(job.qty); *q -= n; n }))
                        .unwrap_or(0);
                    if villager::is_dead(&self.world, job.hirer) {
                        if let Some(b) = self.world.get_component_mut::<Building>(job.to) {
                            *b.stock.entry(job.good.clone()).or_insert(0) += qty;
                        } else {
                            // no place to leave them: the kid keeps what they carried
                            if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                                *s.goods.entry(job.good.clone()).or_insert(0) += qty;
                            }
                        }
                        self.say(who, "...he's gone.", CHAT_BUBBLE_SECS);
                        self.world.remove_component::<OddJob>(who);
                        return;
                    }
                    if let Some(s) = self.world.get_component_mut::<Satchel>(job.hirer) {
                        *s.goods.entry(job.good).or_insert(0) += qty;
                    }
                    if let Some(h) = self.world.get_component_mut::<Purse>(job.hirer) { h.gold -= job.pay.min(h.gold); }
                    if let Some(k) = self.world.get_component_mut::<Purse>(who) { k.gold += job.pay; }
                    // an errand run is a bond: the hirer they run for
                    // most is their likeliest master
                    if let Some(g) = self.world.get_component_mut::<KidGrowth>(who) {
                        if let Some((_, n)) = g.errands.iter_mut().find(|(e, _)| *e == job.hirer) { *n += 1; }
                        else { g.errands.push((job.hirer, 1)); }
                    }
                    self.say(who, "All delivered!", CHAT_BUBBLE_SECS);
                    self.world.remove_component::<OddJob>(who);
                }
            }
        }
    }
}
