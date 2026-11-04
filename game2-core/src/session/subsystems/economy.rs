//! The economy's daily turns: spoilage and the shop's cat, inheritance, gardens and kitchens, cheese, the tavern and the drink trades, fishing, and the errands that fetch inputs.

use crate::session::*;

impl Game {
    /// The larder keeps its own score: stock spoils by its half-life,
    /// vermin get at the grain where no cat walks, and a poor soul eats
    /// what shouldn't be eaten
    pub fn spoilage_tick(&mut self, tick: u64) {
        // weekly rot: bread and dairy go fast, grain holds a while
        if tick % 7 == 3 {
            let week_of = |good: &str| match good {
                "bread" | "milk" | "eggs" => 1u64,
                "wheat" => 4,
                _ => 0,
            };
            for &be in &self.building_entities.clone() {
                let mut loss: Vec<(String, u32)> = vec![];
                if let Some(b) = self.world.get_component::<Building>(be) {
                    for (g, &q) in b.stock.iter() {
                        let w = week_of(g);
                        // granaries draw the mice: wheat goes twice as fast
                        // where the grain's piled - unless a cat walks there
                        let vermin = b.kind == BuildingKind::Granary && g == "wheat"
                            && !self.cat_watches(be);
                        let extra = if vermin { 1 } else { 0 };
                        if w > 0 && q > 0 && tick % (7 * w) == 3 {
                            loss.push((g.clone(), 1 + extra));
                        } else if extra > 0 && tick % 28 == 3 { loss.push((g.clone(), extra)); }
                    }
                }
                if loss.is_empty() { continue; }
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    for (g, n) in &loss { household::take_stock(b, g, *n); }
                }
            }
        }
        // a broke, hungry soul eats what shouldn't be eaten
        if tick % 9 == 4 {
            if let Some(v) = self.villagers.iter().copied().find(|&v| {
                !villager::is_dead(&self.world, v)
                && self.world.get_component::<Purse>(v).map(|p| p.gold < 2.0).unwrap_or(false)
                && self.world.get_component::<Hunger>(v).map(|h| h.value < 25.0).unwrap_or(false)
            }) {
                desires::add_grievance(&mut self.world, v, desires::Problem::SourGut, tick + 30);
                self.say(v, "It was all there was.", TALK_BUBBLE_SECS);
            }
        }
    }

    /// Does a hunting cat walk this building's yard? Vermin keep away
    pub(in crate::session) fn cat_watches(&self, building: Entity) -> bool {
        // the tower owl hunts the whole village while she's up - and
        // from the day she's due, so her landing day's a safe day too
        let owl = crate::session::events::EventId::OwlInTower;
        if self.rare.iter().any(|r| r.id == owl && !r.leaving)
            || self.next_rare.iter().any(|&(id, due)| id == owl && due <= self.tick) {
            return true;
        }
        let Some(b) = self.world.get_component::<Building>(building) else { return false };
        b.occupants.iter().any(|&occ| self.pets.iter().any(|(e, s, _)| {
            *s == Species::Cat && self.world.get_component::<Pet>(*e)
                .map(|p| p.owner == Some(occ)).unwrap_or(false)
                && self.world.get_component::<PetRole>(*e).map(|r| matches!(r, PetRole::Hunting)).unwrap_or(false)
        }))
    }

    /// The dead hand on: a master's bench goes to the senior apprentice,
    /// else to kin under their roof, else to a journeyman of the trade -
    /// else it stands shuttered. Four parts of the purse go with the
    /// bench; the lord's fifth is the death-duty
    pub fn succession_tick(&mut self, tick: u64) {
        let dead_new: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| villager::is_dead(&self.world, v) && !self.succeeded.contains(&v))
            .collect();
        for dead in dead_new {
            self.succeeded.push(dead);
            self.inherit_shop(dead, tick);
        }
    }

    /// One villager's affairs settled
    pub(in crate::session) fn inherit_shop(&mut self, dead: Entity, tick: u64) {
        let (Some(shop), voc) = self.world.get_component::<Craft>(dead)
            .map(|c| (c.workplace, c.vocation)).unwrap_or((None, Vocation::Farmer))
        else { return };
        if self.world.get_component::<Building>(shop).is_none() { return }
        // no bench passes to a grave or a cell - the heir must be able
        // to stand behind it
        let heir = self.villagers.iter().copied()
            .find(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<Craft>(v).map(|c| c.master == Some(dead)).unwrap_or(false))
            .or_else(|| {
                let home = household::home_of(&self.world, dead)?;
                self.world.get_component::<Building>(home)?.occupants.iter().copied()
                    .find(|&o| o != dead && !crate::game::dungeon::unavailable(&self.world, o)
                        && self.world.get_component::<Kid>(o).is_none())
            })
            .or_else(|| self.villagers.iter().copied()
                .find(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                    && self.world.get_component::<Craft>(v)
                        .map(|c| c.vocation == voc && c.workplace.is_none()).unwrap_or(false)));
        let dname = self.villagers.iter().position(|v| *v == dead).map(|i| vill_name(i)).unwrap_or("the dead");
        match heir {
            Some(h) => {
                // a shop only has the one master - give up any other bench
                let old = self.world.get_component::<Craft>(h).and_then(|c| c.workplace);
                if let Some(old) = old {
                    if old != shop {
                        if let Some(b) = self.world.get_component_mut::<Building>(old) { b.worker = None; }
                    }
                }
                if let Some(c) = self.world.get_component_mut::<Craft>(h) {
                    c.workplace = Some(shop); c.master = None;
                }
                if let Some(b) = self.world.get_component_mut::<Building>(shop) { b.worker = Some(h); }
                // four parts pass on; the fifth is the lord's
                let gold = self.world.get_component::<Purse>(dead).map(|p| p.gold).unwrap_or(0.0);
                if let Some(p) = self.world.get_component_mut::<Purse>(dead) { p.gold = 0.0; }
                if let Some(p) = self.world.get_component_mut::<Purse>(h) { p.gold += gold * 0.8; }
                desires::add_grievance(&mut self.world, h, desires::Problem::DeathDuty, tick + 60);
                let hname = self.villagers.iter().position(|v| *v == h).map(|i| vill_name(i)).unwrap_or("the heir");
                self.log.push(format!("t{}: {} takes up {}'s bench; the lord takes his fifth.", tick, hname, dname));
            }
            None => {
                if let Some(b) = self.world.get_component_mut::<Building>(shop) { b.worker = None; }
                let bname = self.world.get_component::<Building>(shop).map(|b| b.name.clone()).unwrap_or_else(|| "the shop".into());
                self.log.push(format!("t{}: {} stands shuttered - no heir for {}'s bench.", tick, bname, dname));
            }
        }
    }

    /// The gardens grow: a bed fills in over the weeks, and when the
    /// herb-wife's shelves are thin she tends the gardens whose folk
    /// don't begrudge her - her flowers, not the commons
    pub fn garden_tick(&mut self, tick: u64) {
        if tick % 7 != 5 { return; }
        for (_, _, g, _) in self.gardens.iter_mut() { *g = (*g + 0.2).min(1.2); }
        // a full kitchen bed gets picked by its own folk - the pot
        // waits on nobody's charity
        let kitchen_picks: Vec<(usize, (i32, i32), Entity)> = self.gardens.iter().enumerate()
            .filter(|(_, (_, _, g, kitchen))| *kitchen && *g >= 1.0)
            .filter_map(|(i, (home, t, _, _))| {
                let occ = self.world.get_component::<Building>(*home)
                    .map(|b| b.occupants.iter().copied().find(|&o|
                        !villager::is_dead(&self.world, o)
                        && !crate::game::dungeon::unavailable(&self.world, o)
                        && !self.travels.iter().any(|tr| tr.who == o)
                        // only a hand close enough to reach the bed in a
                        // short walk can be sent - else the errand dies
                        // on send and the crop stands unharvested
                        && self.world.get_component::<Pos>(o)
                            .map(|p| (p.x.round() as i32 - t.0).abs() + (p.y.round() as i32 - t.1).abs() <= 16)
                            .unwrap_or(false)))
                    .flatten();
                occ.map(|o| (i, *t, o))
            }).collect();
        for (gi, t, occ) in kitchen_picks {
            self.send_tile_walk(occ, t, Goal::TendGarden(gi), 16);
        }
        // the herb-wife tends a full herb garden her folk don't begrudge
        // her - the pickings go on her shelf rather than the house's
        if let Some(her) = self.herbologist {
            let thin = self.herb_hut.map(|hut| {
                self.world.get_component::<Building>(hut)
                    .map(|b| herbology::medicinal_herbs().iter()
                        .map(|(n, _, _)| b.stock.get(*n).copied().unwrap_or(0)).sum::<u32>() < 6)
                    .unwrap_or(false)
            }).unwrap_or(false);
            if thin && !villager::is_dead(&self.world, her)
                && !self.travels.iter().any(|t| t.who == her) {
                let pick = self.gardens.iter().enumerate()
                    .filter(|(_, (home, _, g, kitchen))| !kitchen && *g >= 1.0 && self.world.get_component::<Building>(*home)
                        .map(|b| b.occupants.iter().all(|&o| {
                            self.world.get_component::<InnerLife>(o)
                                .map(|l| l.grievances.is_empty()).unwrap_or(true)
                        })).unwrap_or(false))
                    .map(|(i, _)| i).next();
                if let Some(gi) = pick {
                    if let Some((_, t, _, _)) = self.gardens.get(gi).copied() {
                        self.send_tile_walk(her, t, Goal::TendGarden(gi), 16);
                    }
                }
            }
        }
    }

    /// The kitchen pot: a hungry soul at home takes a staple off the
    /// shelf the way they take an egg off the hens, and once a week a
    /// house with peas, roots and potherbs in the cupboard boils them
    /// down to pottage - the poor table's whole dinner
    pub fn kitchen_tick(&mut self, tick: u64) {
        for &h in &self.building_entities.clone() {
            let (kind, occupants) = match self.world.get_component::<Building>(h) {
                Some(b) => (b.kind, b.occupants.clone()),
                None => continue,
            };
            if kind != household::BuildingKind::Home { continue; }
            // the pot: peas + roots + potherbs down to two of pottage
            if tick % 7 == 3 {
                let can = self.world.get_component::<Building>(h)
                    .map(|b| ["peas", "roots", "potherbs"].iter()
                        .all(|g| b.stock.get(*g).copied().unwrap_or(0) > 0))
                    .unwrap_or(false);
                if can {
                    if let Some(b) = self.world.get_component_mut::<Building>(h) {
                        for g in ["peas", "roots", "potherbs"] { household::take_stock(b, g, 1); }
                        household::add_stock(b, "pottage", 2);
                    }
                }
            }
            // a hungry one at home takes a staple off the shelf - home
            // meaning under the roof tonight, not just on the lease
            let hp = self.world.get_component::<Pos>(h).copied();
            if let Some(occ) = occupants.into_iter()
                .find(|&o| !villager::is_dead(&self.world, o)
                    && !crate::game::dungeon::unavailable(&self.world, o)
                    && hp.and_then(|hp| self.world.get_component::<Pos>(o)
                        .map(|op| ((op.x - hp.x).abs() + (op.y - hp.y).abs()) <= 5.0)).unwrap_or(false)
                    && self.world.get_component::<Hunger>(o).map(|hu| hu.value < 40.0).unwrap_or(false)) {
                let meal = crate::game::goods::slot_goods(crate::game::goods::Tag::StapleFood)
                    .into_iter().find(|&g| self.world.get_component::<Building>(h)
                        .map(|b| b.stock.get(g).copied().unwrap_or(0) > 0).unwrap_or(false));
                if let Some(g) = meal {
                    if let Some(b) = self.world.get_component_mut::<Building>(h) {
                        if household::take_stock(b, g, 1) > 0 {
                            if let Some(hu) = self.world.get_component_mut::<Hunger>(occ) { hu.value = (hu.value + 25.0).min(100.0); }
                            self.say(occ, "a bite from the pot", CHAT_BUBBLE_SECS);
                        }
                    }
                }
            }
        }
    }

    /// The cellar's clock: milk down becomes wheels, the wheels age in
    /// the dark, and now and then a noble rides in and pays triple for a
    /// well-aged one - this village's cheese is the stone-cellar cheese
    pub fn cheese_tick(&mut self, tick: u64) {
        if self.cellar.is_none() { return; }
        let Some(fold) = self.fold else { return };
        // milk down: two pails make a wheel, up to a full rack
        if tick % 7 == 6 && self.cheeses.len() < 8 {
            let took = self.world.get_component_mut::<Building>(fold)
                .map(|b| household::take_stock(b, "milk", 2)).unwrap_or(0);
            if took >= 2 { self.cheeses.push(tick); }
        }
        // a noble buyer: the oldest wheel well past its year goes for triple
        if tick % 90 == 75 && !self.cheeses.is_empty() {
            let age = tick - self.cheeses[0];
            if age > 60 {
                self.cheeses.remove(0);
                let worth = goods::worth_at(&self.world, "cheese") * 3.0 * (1.0 + age as f32 / 60.0);
                if let Some(sh) = self.shepherd {
                    if let Some(p) = self.world.get_component_mut::<Purse>(sh) { p.gold += worth; }
                }
                self.log.push(format!(
                    "t{}: a noble pays triple for the {} cheese - a wheel {} days aged.",
                    tick, self.cellar_trait, age));
            }
        }
    }

    /// The tavern's day: the keeper lays in a barrel, and the bench fills.
    /// The one table in the village where a cup comes with company.
    pub fn tavern_tick(&mut self, tick: u64) {
        let Some(tavern) = self.tavern else { return };
        // a dead or departed keeper means a dark house: no barrels laid
        // in, no table kept
        let keeper = self.world.get_component::<Building>(tavern).and_then(|b| b.worker)
            .filter(|&k| !villager::is_dead(&self.world, k)
                && !crate::game::dungeon::unavailable(&self.world, k));
        let Some(keeper) = keeper else { return };
        if tick % 2 == 0 { self.tavern_restock(tick, tavern, keeper); }
        self.tavern_bench(tick, tavern, keeper);
    }

    /// The keeper lays in a barrel. He brews nothing: what runs low on
    /// his shelf he buys off whichever trade makes it, at the wholesale
    /// cut, and pours it back out at the street price - that margin is
    /// his whole living. The drink trades get the customer they never
    /// had, and an empty shelf leans the good dear like any other sale.
    pub(in crate::session) fn tavern_restock(&mut self, tick: u64, tavern: Entity, keeper: Entity) {
        // cheapest first: the everyday pour is what the village's purses
        // actually reach, and a keeper who spends his float on aqua vitae
        // nobody can buy has a full shelf and an empty till
        for good in ["ale", "beer", "wine", "spirits"] {
            let have = self.world.get_component::<Building>(tavern)
                .map(|b| b.stock.get(good).copied().unwrap_or(0)).unwrap_or(0);
            if have >= TAVERN_BAR_LOW { continue; }
            let want = TAVERN_BARREL.min(household::stock_room(
                self.world.get_component::<Building>(tavern).unwrap(), good));
            if want == 0 { continue; }
            // whoever has it on the shelf, nearest first: the trade that
            // made it, or the road's stall. The stall matters - a tavern
            // is the house that sells the road's wine, which is the whole
            // of what set it apart from an alewife's bench, and it keeps
            // the bar wet in a village whose own drink trade runs short
            let at = match self.world.get_component::<Pos>(tavern) { Some(p) => *p, None => return };
            let source = self.building_entities.iter().copied().filter(|&be| {
                be != tavern && self.world.get_component::<Building>(be).map(|b|
                    (matches!(b.kind, BuildingKind::Shop(v) if vocation::output(v) == Some(good))
                        || (b.kind == BuildingKind::TraderStall && vocation::imported(good)))
                        && b.worker.is_some()
                        && b.stock.get(good).copied().unwrap_or(0) > 0).unwrap_or(false)
            }).min_by_key(|&be| self.world.get_component::<Pos>(be)
                .map(|p| ((p.x - at.x).abs() + (p.y - at.y).abs()) as i32).unwrap_or(i32::MAX));
            let Some(source) = source else { continue };
            if self.boycotted(source) { continue; }
            let Some(maker) = self.world.get_component::<Building>(source).and_then(|b| b.worker) else { continue };
            // he buys what his purse reaches, no more - and he keeps a
            // reserve back. His is the only trade that pays for its stock
            // before it sells any, so a keeper who commits the last of
            // his coin to one barrel never buys a second
            let unit = goods::worth_at(&self.world, good) * TAVERN_WHOLESALE;
            let purse = self.world.get_component::<Purse>(keeper).map(|p| p.gold).unwrap_or(0.0);
            if unit <= 0.0 { continue; }
            let committable = (purse - TAVERN_RESERVE).max(0.0) * TAVERN_SPEND_SHARE;
            let afford = (committable / unit).floor().max(0.0) as u32;
            let qty = want.min(afford);
            if qty == 0 { continue; }
            let taken = self.world.get_component_mut::<Building>(source)
                .map(|b| household::take_stock(b, good, qty)).unwrap_or(0);
            if taken == 0 { continue; }
            let paid = unit * taken as f32;
            if let Some(p) = self.world.get_component_mut::<Purse>(keeper) { p.gold -= paid; }
            if let Some(p) = self.world.get_component_mut::<Purse>(maker) { p.gold += paid; }
            if let Some(b) = self.world.get_component_mut::<Building>(tavern) {
                household::add_stock(b, good, taken);
            }
            // a shelf the barrel came off leans the good a little dear
            let bare = self.world.get_component::<Building>(source)
                .map(|b| b.stock.get(good).copied().unwrap_or(0) == 0).unwrap_or(false);
            if bare { goods::nudge(&mut self.world, good, goods::BARE_NUDGE); }
            let kn = self.name_of(keeper);
            let mn = self.name_of(maker);
            self.log.push(format!("t{}: {} lays in {} measures of {} off {} for the tavern, {:.0} gold.",
                tick, kn, taken, good, mn, paid));
            // one barrel a trip: whatever he could lay hands on cheapest
            // is the day's lay-in, and the till decides the next one
            return;
        }
    }

    /// The common table. The difference between a cup at the brewery and
    /// a cup at the tavern is who else is holding one: everyone at the
    /// bench on the same day buys a cup off the bar - the keeper's whole
    /// till - and falls into company with everyone else who did, which
    /// is how strangers in a village stop being strangers.
    pub(in crate::session) fn tavern_bench(&mut self, tick: u64, tavern: Entity, keeper: Entity) {
        let at = match self.world.get_component::<Pos>(tavern) { Some(p) => *p, None => return };
        // a dry house draws nobody: the table only fills if there's
        // something in the barrel
        let wet = self.world.get_component::<Building>(tavern)
            .map(|b| b.stock.iter().any(|(g, q)| *q > 0 && vocation::is_drink(g))).unwrap_or(false);
        if !wet { return; }
        let mut sitting: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
            v != keeper && !villager::is_dead(&self.world, v)
                && self.world.get_component::<Kid>(v).is_none()
                && !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - at.x).abs() + (p.y - at.y).abs() <= TAVERN_TABLE_REACH)
                    .unwrap_or(false)
        }).collect();
        // a bench only seats so many; the rest drink standing and keep
        // their own counsel
        sitting.truncate(TAVERN_BENCH);
        if sitting.is_empty() { return; }
        // the register, so the night's company is the night's company
        self.tavern_table.retain(|&(_, d)| tick.saturating_sub(d) < 30);
        for &v in &sitting {
            if !self.tavern_table.iter().any(|&(w, d)| w == v && d == tick) {
                self.tavern_table.push((v, tick));
            }
        }
        // a cup goes down at the bench. This is the keeper's whole till:
        // he laid the barrel in at the wholesale cut and it leaves his
        // shelf at the street price, one cup at a time. The thirsty buy
        // cheapest-first, same as they would walking to the brewery.
        // Only those who actually bought a cup are company at the
        // bench - a dry patron owes the keeper nothing and gets nothing
        let mut served: Vec<Entity> = Vec::new();
        for &v in &sitting.clone() {
            if desires::is_satisfied(&self.world, v, Want::Drink, tick) { continue; }
            let pour = ["ale", "beer", "wine", "spirits"].into_iter().find(|&g| {
                let on_tap = self.world.get_component::<Building>(tavern)
                    .map(|b| b.stock.get(g).copied().unwrap_or(0) > 0).unwrap_or(false);
                let affords = self.world.get_component::<Purse>(v)
                    .map(|p| p.gold >= goods::worth_at(&self.world, g)).unwrap_or(false);
                on_tap && affords
            });
            let Some(pour) = pour else { continue };
            if household::buy_at_shop(&mut self.world, tavern, v, pour, 1).is_none() { continue; }
            desires::satisfy(&mut self.world, v, Want::Drink, tick + 60);
            served.push(v);
            // drunk at the bench, not carried home in the bag
            if let Some(sa) = self.world.get_component_mut::<Satchel>(v) {
                if let Some(q) = sa.goods.get_mut(pour) { *q = q.saturating_sub(1); }
            }
            if self.rng.below(100) < TAVERN_DRUNK_PCT {
                self.world.add_component(v, villager::Drunk { until: tick + 2 });
                self.say(v, "One more won't hurt.", CHAT_BUBBLE_SECS);
            }
        }
        // and the company. Everyone who bought a cup today has met
        // everyone else who did, which is how strangers in a village
        // stop being strangers. What the drink turns into when it turns
        // bad is the AlehouseBrawl event's business, not a second
        // mechanism's - it looks for the drunks this bench makes
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        for i in 0..served.len() {
            for j in (i + 1)..served.len() {
                record_interaction(&mut self.world, served[i], Interaction {
                    other: served[j], kind: InteractionKind::Conversation, intensity: CHAT_INTENSITY });
            }
            // an evening in company is an evening not spent alone
            desires::satisfy(&mut self.world, served[i], Want::Joy, tick + 20);
        }
        // the keeper hears all of it - that's the trade
        for &v in &served {
            record_interaction(&mut self.world, keeper, Interaction {
                other: v, kind: InteractionKind::Conversation, intensity: CHAT_INTENSITY });
        }
    }

    /// The drink trade's own clocks: small ale sours inside the week and
    /// the spoil turns to vinegar (which still sells for something); wine
    /// keeps the year in a good cask and forever in the cellar's dark,
    /// but turns on a plain shelf. Beer and the water of life keep - and
    /// every fortnight the still slips the wise woman a measure for her
    /// tinctures
    pub fn drink_tick(&mut self, tick: u64) {
        if tick % 3 == 2 {
            for &be in &self.building_entities.clone() {
                let ale = self.world.get_component::<Building>(be)
                    .map(|b| b.stock.get("ale").copied().unwrap_or(0)).unwrap_or(0);
                if ale == 0 { continue; }
                let sour = (ale / 4).max(1);
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    let lost = household::take_stock(b, "ale", sour);
                    household::add_stock(b, "vinegar", lost / 2);
                }
            }
        }
        if tick % 7 == 5 {
            for &be in &self.building_entities.clone() {
                let (wine, kind, casks) = self.world.get_component::<Building>(be)
                    .map(|b| (b.stock.get("wine").copied().unwrap_or(0), b.kind,
                        b.stock.get("casks").copied().unwrap_or(0)))
                    .unwrap_or((0, BuildingKind::Home, 0));
                if wine == 0 || kind == BuildingKind::Cellar { continue; }
                // hooped and banged, a cask buys the wine twice the time
                if casks > 0 && tick % 14 != 5 { continue; }
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    if household::take_stock(b, "wine", 1) > 0 {
                        household::add_stock(b, "vinegar", 1);
                    }
                }
            }
        }
        // the water of life is the wise woman's solvent: the still owes
        // her a measure a fortnight, like the monastery always did
        if tick % 15 == 10 {
            if let Some(hut) = self.herb_hut {
                let thin = self.world.get_component::<Building>(hut)
                    .map(|b| b.stock.get("spirits").copied().unwrap_or(0) < 2).unwrap_or(false);
                if thin {
                    let still = self.building_entities.iter().copied().find(|&be|
                        self.world.get_component::<Building>(be)
                            .map(|b| b.kind == BuildingKind::Shop(Vocation::Distiller)
                                && b.stock.get("spirits").copied().unwrap_or(0) > 0)
                            .unwrap_or(false));
                    if let Some(still) = still {
                        if let Some(b) = self.world.get_component_mut::<Building>(still) {
                            if household::take_stock(b, "spirits", 1) > 0 {
                                if let Some(h) = self.world.get_component_mut::<Building>(hut) {
                                    household::add_stock(h, "spirits", 1);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The river gives: a catch for the fish-smoke house, better in a
    /// flood year and thin in the ice - and now and then the water
    /// brings up something strange, and the pedlar sells the tale
    pub fn fisher_tick(&mut self, tick: u64) {
        if tick % 7 != 4 { return; }
        // the bench makes the fisher: whoever keeps the fish-smoke house
        // works the river - so the heir takes the catch, and the dead
        // and the held take none
        let Some(hut) = self.world.query::<Building>().iter().copied()
            .find(|&b| self.world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Shop(Vocation::Fisher)).unwrap_or(false))
        else { return };
        let Some(fisher) = self.world.get_component::<Building>(hut)
            .and_then(|b| b.worker)
            .filter(|&w| !crate::game::dungeon::unavailable(&self.world, w))
        else { return };
        // the catch: winter's thin, the rest of the year's fuller
        let catch = if tick % 365 >= 270 { 1 } else { 3 };
        if let Some(b) = self.world.get_component_mut::<Building>(hut) {
            household::add_stock(b, "fish", catch);
        }
        // and now and then the water brings up something strange - a pale
        // stone, a thing that glows - and the tale gets around
        if tick % 70 == 60 {
            self.log.push(format!("t{}: the fisher pulls a pale stone from the river - cold, and faintly lit.", tick));
            self.say(fisher, "The river keeps its own.", CHAT_BUBBLE_SECS);
        }
    }

    /// What this person might walk out to buy today: a good that feeds an
    /// unsatisfied want, stocked by some working shop
    pub fn want_shopping(&mut self, v: Entity, tick: u64) -> Option<Goal> {
        const WANT_GOODS: [(Want, &str); 11] = [
            (Want::GoodFood, "bread"), (Want::Finery, "garments"), (Want::Faith, "faith"),
            (Want::Security, "security"), (Want::Instrument, "lute"), (Want::BetterTools, "tools"),
            // the everyday pour first - the dearer the drink, the fewer
            // the purses that reach it
            (Want::Drink, "ale"), (Want::Drink, "beer"), (Want::Drink, "wine"),
            (Want::Drink, "spirits"),
            // a bed at the tavern is no roof of one's own, but it's a
            // night out of the weather while they wait for one
            (Want::Shelter, "lodging"),
        ];
        let life = self.world.get_component::<InnerLife>(v)?;
        let wants = life.wants.clone();
        let pos = *self.world.get_component::<Pos>(v)?;
        let gold = self.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
        for (w, good) in WANT_GOODS {
            if !wants.contains(&w) || desires::is_satisfied(&self.world, v, w, tick) { continue; }
            if gold < goods::worth_at(&self.world, good) { continue; }
            // a bed at the tavern is for whoever hasn't one of their own;
            // a soul with a roof who still wants one wants a better roof,
            // and the keeper can't sell them that
            if good == "lodging" && household::home_of(&self.world, v).is_some() { continue; }
            if let Some(shop) = livelihood::shop_for(&self.world, good, pos) {
                return Some(Goal::BuyAt(shop, good, Some(w)));
            }
        }
        None
    }

    /// Find `input` for worker `v`: send a nearby idle kid to fetch it for
    /// pay when one's about, else walk for it - a seller's satchel, or a
    /// gather site when the land provides
    pub fn errand_for_input(&mut self, v: Entity, input: &'static str, _i: usize) -> bool {
        let pos = match self.world.get_component::<Pos>(v) { Some(p) => *p, None => return false };
        // a kid can run it: any child about - nearby, idle, not already
        // on a job. The village grows, so it's the Kid mark that counts,
        // not whichever three were born first
        if self.rng.below(100) < 40 {
            let kids: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&k| self.world.get_component::<Kid>(k).is_some()).collect();
            for k in kids {
                if villager::is_dead(&self.world, k) || self.travels.iter().any(|t| t.who == k)
                    || self.world.get_component::<OddJob>(k).is_some() { continue; }
                let near = self.world.get_component::<Pos>(k)
                    .map(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() < 7.0).unwrap_or(false);
                if !near { continue; }
                if let Some(from) = livelihood::seller_for(&self.world, input, pos)
                    .filter(|&s| !self.boycotted(s)) {
                    let pay = livelihood::ODD_JOB_MIN + self.rng.below(4) as f32;
                    self.world.add_component(k, OddJob {
                        hirer: v, good: input.to_string(), qty: 2, from, to: v, pay, loaded: false });
                    self.send(k, Goal::RunOddJob);
                    return true;
                }
            }
        }
        // go themselves: a seller's satchel, the stall, or gather it
        if let Some(s) = livelihood::seller_for(&self.world, input, pos)
            .filter(|&s| !self.boycotted(s)) {
            return self.send(v, Goal::BuyFrom(s, input, 2));
        }
        if vocation::gatherable(input) {
            if let Some(t) = self.gather_site(input, pos) {
                return self.send_gather(v, input, t);
            }
        }
        false
    }

    /// Where to gather `good` nearest `at`: wood off a tree tile, stone
    /// off a rock or boulder, sand off a tile lapping water
    pub fn gather_site(&self, good: &str, at: Pos) -> Option<(i32, i32)> {
        let dist = |x: i32, y: i32| ((x as f32 - at.x).abs() + (y as f32 - at.y).abs()) as i32;
        match good {
            "wood" => self.map.trees.iter().copied()
                .min_by_key(|&(x, y)| dist(x, y)),
            "stone" => self.stones.iter().copied()
                .chain((0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                    .filter(|&(x, y)| self.map.get(x, y) == Some(Tile::Rock)))
                .min_by_key(|&(x, y)| dist(x, y)),
            "sand" => (0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                .filter(|&(x, y)| self.map.passable(x, y)
                    && NEIGHBOURS.iter().any(|(dx, dy)| matches!(self.map.get(x + dx, y + dy), Some(Tile::Water) | Some(Tile::River))))
                .min_by_key(|&(x, y)| dist(x, y)),
            // faggots come off the same trees as the craft-wood - the
            // leavings and deadfall, not the timber
            "faggots" => self.map.trees.iter().copied()
                .min_by_key(|&(x, y)| dist(x, y)),
            // turf cuts off the wet ground by the river, same ground the
            // marsh fevers rise off
            "turf" => (0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                .filter(|&(x, y)| self.map.get(x, y) == Some(Tile::Grass) && self.map.passable(x, y)
                    && NEIGHBOURS.iter().any(|(dx, dy)| matches!(self.map.get(x + dx, y + dy), Some(Tile::Water) | Some(Tile::River))))
                .min_by_key(|&(x, y)| dist(x, y)),
            _ => None,
        }
    }
}
