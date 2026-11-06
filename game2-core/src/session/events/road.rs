//! Folk on the road: the tinkers, pedlars, pilgrims, players and tradesmen who walk in and out.
//! Each fn here holds this family's arms of one handler in events/mod.rs.

use super::*;

impl Game {
    /// road: the `event_day` arms for this group (the rest fall to the default)
    pub(super) fn road_day(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::Tinker => {
                // pots mended: whoever's near his pitch gets a sharper
                // kit for a penny - once, not a penny a day for the same pot
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    // his work's already done on their kit: no second penny
                    if desires::is_satisfied(&self.world, v, Want::BetterTools, tick) { continue; }
                    // no penny, no mend
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::BetterTools, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                }
            }
            EventId::KnifeGrinder => {
                // an edge on the trades' steel for a penny; the kids get
                // the sparks for free - the show's the thing they remember
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    if self.world.get_component::<crate::game::vocation::Kid>(v).is_some() {
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 10);
                        continue;
                    }
                    // his wheel only earns off a working blade
                    if self.world.get_component::<crate::game::vocation::Craft>(v).is_none() { continue; }
                    if desires::is_satisfied(&self.world, v, Want::BetterTools, tick) { continue; }
                    // no penny, no edge - the poorest keep their dull steel
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::BetterTools, tick + 45);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                }
            }
            EventId::RagAndBone => {
                // a coin for the poorest rag-bundle in reach - one a day,
                // so a rich street sees him push an empty cart by
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let poorest = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold < 8.0).unwrap_or(false))
                    .min_by(|&a, &b| {
                        let ga = self.world.get_component::<Purse>(a).map(|p| p.gold).unwrap_or(0.0);
                        let gb = self.world.get_component::<Purse>(b).map(|p| p.gold).unwrap_or(0.0);
                        ga.partial_cmp(&gb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(v) = poorest {
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold += 1.0; }
                    self.say(v, "For the rags? Bless you, mister.", CHAT_BUBBLE_SECS);
                }
            }
            EventId::ChimneySweep => {
                // a penny a flue: homes in reach of his ladder breathe
                // easier (Security) - it's the hearth he cleans, not the
                // passer-by, so the reach runs on their home's door
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = household::home_of(&self.world, v)
                        .and_then(|h| self.world.get_component::<Pos>(h))
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    if desires::is_satisfied(&self.world, v, Want::Security, tick) { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Security, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                }
            }
            EventId::RatCatcher => {
                // his terrier earns: while he's about the store loses
                // nothing to vermin - the servings that weren't gnawed,
                // and his sack counts them
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings += 2;
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux += 1; }
            }
            EventId::Scribe => {
                // a penny a page for the illiterate: up to two letters a
                // day, carried to whoever their strongest bond is
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut wrote = 0;
                // one pass over every bond covers the whole queue of
                // customers - a scan per villager repeated the whole map
                let bonds = crate::game::relationships::bond_map(&self.world);
                for &v in &self.villagers.clone() {
                    if wrote >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || self.letters.iter().any(|&(f, _, _)| f == v) { continue; }
                    // he writes for the illiterate - the lettered write their own
                    if self.literate(v) { continue; }
                    // and only to the living - the dead can't be written to
                    let partner = bonds.get(&v).into_iter().flatten()
                        .filter_map(|&r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                        .filter(|rr| (rr.from == v || rr.other == v)
                            && !villager::is_dead(&self.world, if rr.from == v { rr.other } else { rr.from }))
                        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|rr| if rr.from == v { rr.other } else { rr.from });
                    let Some(to) = partner else { continue; };
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    // a penny a page - and it lands in the scribe's purse,
                    // the fee that keeps him on the road
                    if let Some(se) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.e) {
                        if let Some(sp) = self.world.get_component_mut::<Purse>(se) { sp.gold += 1.0; }
                    }
                    self.letters.push((v, to, tick + 30));
                    wrote += 1;
                }
            }
            EventId::ToothDrawer => {
                // one bad tooth a day: the worst jaw in reach gets the
                // pull, a penny the tooth - quick as a wink, he swears
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false)
                        && self.world.get_component::<Health>(v).map(|h| h.condition < 0.9).unwrap_or(false))
                    .min_by(|&a, &b| {
                        let ha = self.world.get_component::<Health>(a).map(|h| h.condition).unwrap_or(1.0);
                        let hb = self.world.get_component::<Health>(b).map(|h| h.condition).unwrap_or(1.0);
                        ha.partial_cmp(&hb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(v) = mark {
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.05).min(1.0);
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Out with it! - that's that, then.", CHAT_BUBBLE_SECS);
                }
            }
            EventId::BarberSurgeon => {
                // a bloodletting for a penny: it mostly does them good,
                // though now and then the knife slips - the barber's
                // honest enough to say so
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false)
                        && self.world.get_component::<Health>(v).map(|h| h.condition < 0.95).unwrap_or(false))
                    .min_by(|&a, &b| {
                        let ha = self.world.get_component::<Health>(a).map(|h| h.condition).unwrap_or(1.0);
                        let hb = self.world.get_component::<Health>(b).map(|h| h.condition).unwrap_or(1.0);
                        ha.partial_cmp(&hb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(v) = mark {
                    let slipped = self.rare_roll(id) % 4 == 0;
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        // a nick only ever cuts - the floor never lifts them
                        h.condition = if slipped { (h.condition - 0.03).max(0.01) }
                            else { (h.condition + 0.03).min(1.0) };
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, if slipped { "The knife slipped. A bandage'll set it." } else { "There. The humours flow easier." }, CHAT_BUBBLE_SECS);
                }
            }
            EventId::PilgrimBand => {
                // the road to the shrine passes here: a penny in the
                // bowl feeds the faith - up to three alms a day, and
                // only from those who can spare it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut alms = 0;
                for &v in &self.villagers.clone() {
                    if alms >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Faith, tick) { continue; }
                    // only the faithful drop a penny in - the bowl means
                    // nothing to those whose wants lie elsewhere
                    let faithful = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !faithful || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Go with God.", CHAT_BUBBLE_SECS);
                    alms += 1;
                }
            }
            EventId::RelicSeller => {
                // a saint's knuckle-bone for two gold: the faithful carry
                // the blessing a month and a half - the seal's his own work
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Faith, tick) { continue; }
                    let faithful = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !faithful || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, tick + 45);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                    self.say(v, "See the seal!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::Pardoner => {
                // a grievance off the heart for two gold: the paper's
                // real enough, though the seal's his own carving
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    // a live burden only - the faded ones weigh nothing
                    let burdened = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.grievances.iter().any(|(_, u)| tick <= *u)).unwrap_or(false);
                    if !near || !burdened { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !pays { continue; }
                    // the newest live weight off the heart
                    if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                        if let Some(i) = l.grievances.iter().rposition(|(_, u)| tick <= *u) {
                            l.grievances.remove(i);
                        }
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                    self.say(v, "Your sins made light!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::Juggler => {
                // three knives and a grin: the show's free, but the hat
                // comes round - a penny in from those who felt the joy
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Joy, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Joy)).unwrap_or(false);
                    if !feels_it { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 15);
                    let spare = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if spare {
                        if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    }
                    self.say(v, "A marvel!", CHAT_BUBBLE_SECS);
                }
            }
            EventId::BearWard => {
                // the bear dances and the hat goes round; and now and
                // then the bear forgets himself and takes a snap at the
                // nearest hand - a story either way
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Joy, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Joy)).unwrap_or(false);
                    if !feels_it { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 20);
                    let spare = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if spare {
                        if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    }
                }
                // the snap - a snap's worth telling for years
                if self.rare_roll(id) % 5 == 0 {
                    let nearest = self.villagers.iter().copied()
                        .filter(|&v| !villager::is_dead(&self.world, v)
                            && !crate::game::dungeon::unavailable(&self.world, v))
                        .filter_map(|v| self.world.get_component::<Pos>(v)
                            .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                        .filter(|(d, _)| *d < 4).min_by_key(|(d, _)| *d).map(|(_, v)| v);
                    if let Some(v) = nearest {
                        if let Some(h) = self.world.get_component_mut::<Health>(v) {
                            h.condition = (h.condition - 0.05).max(0.01);
                        }
                        self.say(v, "He bit me!", CHAT_BUBBLE_SECS);
                        self.log.push(format!("t{}: the bear took a snap at the crowd - a story for years.", self.tick));
                    }
                }
            }
            EventId::Puppeteer => {
                // Noah's flood and the devil's due: the kids are
                // enthralled for free; the grown-ups who watch over
                // their heads drop a penny in the hat
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Joy, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Joy)).unwrap_or(false);
                    if !feels_it { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 20);
                    // kids get the front row for free; the rest pay the hat
                    if self.world.get_component::<crate::game::vocation::Kid>(v).is_none() {
                        let spare = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                        if spare {
                            if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                        }
                    }
                }
            }
            EventId::BalladMonger => {
                // a penny a broadside: a tune that sticks, and all the
                // doings of the wide world besides
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Music, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Music)).unwrap_or(false);
                    if !feels_it { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Music, tick + 15);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Did you hear?", CHAT_BUBBLE_SECS);
                }
            }
            EventId::Deserter => {
                // hiding in the green: anyone who comes near knows his
                // face now - and knowing's sheltering, the law counts it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let sheltered = self.villagers.iter().copied()
                    .any(|v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                            .unwrap_or(false));
                if sheltered {
                    if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
                }
            }
            EventId::RefugeeFamily => {
                // their village burned - they eat from the store while
                // they camp: a serving a day is cheap for what they fled
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings = f.servings.saturating_sub(1);
                }
            }
            EventId::LeperBand => {
                // crusts go out at arm's length: a serving a day off the
                // store, and the faithful who carry them are counted.
                // Now and then the camp's cough brushes a near hand.
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings = f.servings.saturating_sub(1);
                }
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Faith, tick) { continue; }
                    let faithful = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                    if !faithful { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, tick + 20);
                }
                if self.rare_roll(id) % 8 == 0 {
                    let nearest = self.villagers.iter().copied()
                        .filter(|&v| !villager::is_dead(&self.world, v)
                            && !crate::game::dungeon::unavailable(&self.world, v))
                        .filter_map(|v| self.world.get_component::<Pos>(v)
                            .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                        .filter(|(d, _)| *d < 6).min_by_key(|(d, _)| *d).map(|(_, v)| v);
                    if let Some(v) = nearest {
                        if let Some(h) = self.world.get_component_mut::<Health>(v) {
                            h.condition = (h.condition - 0.04).max(0.01);
                        }
                        self.say(v, "The cough brushed me.", CHAT_BUBBLE_SECS);
                    }
                }
            }
            EventId::Friar => {
                // confession under the hawthorn: a live weight comes off
                // and the ear hears a want plain - no coin, just the telling
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut heard = 0;
                for &v in &self.villagers.clone() {
                    if heard >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let burdened = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.grievances.iter().any(|(_, u)| tick <= *u)).unwrap_or(false);
                    if !burdened { continue; }
                    // absolution takes the newest live weight
                    if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                        if let Some(i) = l.grievances.iter().rposition(|(_, u)| tick <= *u) {
                            l.grievances.remove(i);
                        }
                    }
                    // the telling surfaces a want the ear didn't know
                    // yet - the hear() path keeps the ledger's voice
                    let want = {
                        let wants: Vec<Want> = self.world.get_component::<desires::InnerLife>(v)
                            .map(|l| l.wants.clone()).unwrap_or_default();
                        let kn = self.world.get_component::<desires::Known>(v);
                        wants.into_iter().find(|w| kn
                            .map(|k| !k.revealed.contains(&desires::Reveal::Want(*w)))
                            .unwrap_or(true))
                    };
                    if let Some(w) = want {
                        let player = self.player;
                        desires::hear(&mut self.world, player, v, desires::Reveal::Want(w), tick);
                    }
                    self.say(v, "Go and carry it lighter.", CHAT_BUBBLE_SECS);
                    heard += 1;
                }
            }
            EventId::Heretic => {
                // a strange doctrine by the gate: every other soul who
                // hears it is shaken out of comfort; the others are
                // drawn to hear more - faith doesn't sit still for him
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for (i, &v) in self.villagers.clone().iter().enumerate() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    if i % 2 == 0 {
                        // shaken: the comfort lifts off the faithful
                        if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                            l.satisfied.retain(|(w, _)| *w != Want::Faith);
                        }
                    } else {
                        // drawn: the strange doctrine pulls the curious -
                        // the already-fed hear it again, and it lengthens
                        let curious = self.world.get_component::<desires::InnerLife>(v)
                            .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                        if curious {
                            desires::satisfy(&mut self.world, v, Want::Faith, tick + 20);
                        }
                    }
                }
            }
            EventId::Drover => {
                // sheep to market: a head a day off his drove into the
                // fold for three gold, bought by the nearest soul with coin
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                let buyer = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 3.0).unwrap_or(false))
                    .filter_map(|v| self.world.get_component::<Pos>(v)
                        .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                    .filter(|(d, _)| *d < 8).min_by_key(|(d, _)| *d).map(|(_, v)| v);
                if let (Some(f), Some(b)) = (fold, buyer) {
                    let (fx, fy) = self.world.get_component::<Pos>(f).map(|p| (p.x, p.y)).unwrap_or((at.0 as f32, at.1 as f32));
                    let e = crate::game::livestock::spawn_animal(&mut self.world,
                        crate::game::livestock::LivestockKind::Sheep, f, fx + 1.0, fy + 1.0);
                    // onto the flock's roll too - a sheep you can't see is no sheep
                    self.flock.push(e);
                    if let Some(p) = self.world.get_component_mut::<Purse>(b) { p.gold -= 3.0; }
                    self.say(b, "Good ewe, that one.", CHAT_BUBBLE_SECS);
                }
            }
            EventId::FishHawker => {
                // salt fish keeps all winter: a penny a fish for those
                // who'd know good food when it swims by
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::GoodFood, tick) { continue; }
                    let taste = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::GoodFood)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !taste || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 20);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Salt fish - keeps all winter!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::CornFactor => {
                // grain bought by the quarter: surplus only - no factor
                // starves a village for weight - and the coin lands in
                // the market's own hand, not the building's ledger
                let surplus = self.world.get_component::<villager::FoodSource>(self.market_food)
                    .map(|f| (f.servings.saturating_sub(4)).min(3)).unwrap_or(0);
                if surplus > 0 {
                    if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                        f.servings -= surplus;
                    }
                    let seller = self.world.get_component::<household::Building>(self.market_food)
                        .and_then(|b| b.worker.or(b.overseer))
                        .filter(|&w| !villager::is_dead(&self.world, w)
                            && !crate::game::dungeon::unavailable(&self.world, w))
                        .or_else(|| self.villagers.iter().copied()
                            .find(|&v| !villager::is_dead(&self.world, v)
                                && !crate::game::dungeon::unavailable(&self.world, v)));
                    if let Some(s) = seller {
                        if let Some(p) = self.world.get_component_mut::<Purse>(s) {
                            p.gold += surplus as f32;
                        }
                    }
                }
            }
            EventId::SpiceMerchant => {
                // pepper and cloves are dear: two gold a packet, good
                // food eased and a pinch of cure for those who buy
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::GoodFood, tick) { continue; }
                    let taste = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::GoodFood)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !taste || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                    // the cure: folk hold pepper's a physic
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.02).min(1.0);
                    }
                    self.say(v, "Things that cure what ails!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::SaltSeller => {
                // white gold stretches the winter: a gold a handful off
                // the nearest, and the salt makes the larder last longer
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                // nearest three who can pay - the cry catches closest ears
                let mut buyers: Vec<(i32, Entity)> = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false))
                    .filter_map(|v| self.world.get_component::<Pos>(v)
                        .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                    .filter(|(d, _)| *d < 8).collect();
                buyers.sort_by_key(|(d, _)| *d);
                for (_, v) in buyers.into_iter().take(3) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                        f.servings += 2;
                    }
                    self.say(v, "None goes bad who buys it!", CHAT_BUBBLE_SECS);
                }
            }
            EventId::WoolMerchant => {
                // fleece bought by the stone: the fold's wool stock sells
                // first, then the fresh clip off the sheep - a head clips
                // once a visit, goats don't shear, four stone a day at most
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    let sheep = self.flock.iter().copied()
                        .filter(|&a| self.world.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.kind == crate::game::livestock::LivestockKind::Sheep).unwrap_or(false))
                        .count();
                    let already = self.rare.iter().find(|r| r.id == id).map(|r| r.aux as usize).unwrap_or(0);
                    // what's in the stockroom goes first, then the backs
                    let stock_wool = self.world.get_component::<household::Building>(f)
                        .and_then(|b| b.stock.get("wool").copied()).unwrap_or(0).min(4) as usize;
                    let fresh = sheep.saturating_sub(already).min(4usize.saturating_sub(stock_wool));
                    let clip = stock_wool + fresh;
                    if clip > 0 {
                        if let Some(b) = self.world.get_component_mut::<household::Building>(f) {
                            let w = b.stock.entry("wool".into()).or_insert(0);
                            let taken = (*w).min(stock_wool as u32);
                            *w -= taken;
                        }
                        if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) {
                            r.aux += fresh as u64;
                        }
                        let shepherd = self.world.get_component::<household::Building>(f)
                            .and_then(|b| b.worker.or(b.overseer))
                            .filter(|&w| !villager::is_dead(&self.world, w)
                                && !crate::game::dungeon::unavailable(&self.world, w));
                        if let Some(s) = shepherd {
                            if let Some(p) = self.world.get_component_mut::<Purse>(s) {
                                p.gold += clip as f32;
                            }
                            self.say(s, "Top price, by the stone!", CHAT_BUBBLE_SECS);
                        }
                    }
                }
            }
            EventId::Bookseller => {
                // words that outlive the winter: three gold a book, one
                // a day, to the lettered - a book's joy outlasts the cold
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                // the roster plus the player - she can read too
                let cands: Vec<Entity> = self.villagers.iter().copied()
                    .chain(std::iter::once(self.player)).collect();
                for v in cands {
                    if sold >= 1 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    if !self.literate(v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    // a book lasts - the already-fed don't buy again today
                    let fed = desires::is_satisfied(&self.world, v, Want::Joy, tick);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 3.0).unwrap_or(false);
                    if !near || fed || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 40);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 3.0; }
                    self.say(v, "Words that outlive the winter!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::CharcoalBurner => {
                // his heap burns slow and warm at the woods edge: those
                // who pass feel it in their bones a little
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.01).min(1.0);
                    }
                }
            }
            EventId::LimeBurner => {
                // lime for the fields: folk carry it out to the beds
                // while he stands - the soil grows the sweeter for it
                for g in self.gardens.iter_mut() {
                    // no cap - a bed past ripe stays past ripe
                    g.2 += 0.02;
                }
            }
            EventId::Tanner => {
                // mind the smell: honest work, but the tan-pit's reek
                // hangs on whoever stands near his vats a while
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut nipped = 0;
                for &v in &self.villagers.clone() {
                    if nipped >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near { continue; }
                    // the fold's smell already hangs on a shepherd - a
                    // tanner's reek can't tell him anything new
                    let already = self.world.get_component::<crate::game::vocation::Craft>(v)
                        .map(|c| c.vocation == crate::game::vocation::Vocation::Shepherd).unwrap_or(false);
                    if already { continue; }
                    desires::add_grievance(&mut self.world, v, desires::Problem::Smelly, tick + 15);
                    self.say(v, "Mind the smell - it's honest work!", CHAT_BUBBLE_SECS);
                    nipped += 1;
                }
            }
            EventId::Glazier => {
                // glass for the church, light for the dark months: one
                // home a day gets its panes - the draft's out, the
                // light's in, the folk inside feel both
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut glazed = 0;
                for &v in &self.villagers.clone() {
                    if glazed >= 1 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let Some(h) = household::home_of(&self.world, v) else { continue };
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let occ: Vec<Entity> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default();
                    // a glazed home shows in its folk's light - done once
                    if occ.iter().next()
                        .map(|&o| desires::is_satisfied(&self.world, o, Want::Joy, tick))
                        .unwrap_or(false) { continue; }
                    for o in occ {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                            hl.condition = (hl.condition + 0.02).min(1.0);
                        }
                        desires::satisfy(&mut self.world, o, Want::Joy, tick + 10);
                    }
                    glazed += 1;
                }
            }
            EventId::Thatcher => {
                // roofs patched before the rain finds you: one home a
                // day - fresh thatch on the stock roll, dry heads under
                // it, and the leak's due pushed down the road
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let Some(h) = household::home_of(&self.world, v) else { continue };
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let done = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.stock.get("thatch").copied()).unwrap_or(0) > 0;
                    if done { continue; }
                    if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                        household::add_stock(b, "thatch", 1);
                    }
                    let occ: Vec<Entity> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default();
                    for o in occ {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                            hl.condition = (hl.condition + 0.02).min(1.0);
                        }
                    }
                    if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::GranaryLeak) {
                        // push it off, never pull it in - a sound roof
                        // postpones the leak, it doesn't summon it
                        *due = (*due).max(tick + 400);
                    }
                    break;
                }
            }
            EventId::WellDigger => {
                // he digs deep all week: those who stand near get a
                // cup of what he's striking - a taste of sweet water
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.01).min(1.0);
                    }
                }
            }
            EventId::FortuneTeller => {
                // your palm, love - the lines don't lie: a penny a palm,
                // the lines surface a want, and most times the telling's
                // kind - now and then it's dark and the penny bought truth
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut read = 0;
                for &v in &self.villagers.clone() {
                    if read >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !near || !pays { continue; }
                    // the lines surface a want the ear didn't know - if
                    // there's no line she can't already read, she won't
                    // take the penny for it
                    let want = {
                        let wants: Vec<Want> = self.world.get_component::<desires::InnerLife>(v)
                            .map(|l| l.wants.clone()).unwrap_or_default();
                        let kn = self.world.get_component::<desires::Known>(v);
                        wants.into_iter().find(|w| kn
                            .map(|k| !k.revealed.contains(&desires::Reveal::Want(*w)))
                            .unwrap_or(true))
                    };
                    let Some(w) = want else { continue };
                    // the organizer's own ledger hears it - the person,
                    // not the body politic
                    let player = self.player;
                    desires::hear(&mut self.world, player, v, desires::Reveal::Want(w), tick);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    // most palms read kind - a dark line's still the
                    // truth, and every palm reads its own lines
                    if (self.rare_roll(id) ^ (v as u64).wrapping_mul(0x9E3779B9)) % 4 != 0 {
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 15);
                        self.say(v, "The lines don't lie, love.", CHAT_BUBBLE_SECS);
                    } else {
                        self.say(v, "...ah. Best not say more.", CHAT_BUBBLE_SECS);
                    }
                    read += 1;
                }
            }
            EventId::Astrologer => {
                // he reads the sky over the village once: forewarned of
                // whatever's written nearest, and forewarned is forearmed
                let warned = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0;
                if warned { return; }
                let next = self.next_rare.iter()
                    .filter(|(i, _)| *i != id)
                    .min_by_key(|(_, d)| *d)
                    .map(|(i, d)| (*i, *d));
                if let Some((i, d)) = next {
                    let name = EVENTS.iter().find(|s| s.id == i).map(|s| s.name).unwrap_or("something");
                    // the due's already in days - no conversion
                    let days = d.saturating_sub(tick);
                    self.log.push(format!("t{}: the astrologer reads the sky - {} comes in about {} days.", tick, name, days.max(1)));
                }
                // forearmed: a day's warning's a kind of security for near folk
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    desires::satisfy(&mut self.world, v, Want::Security, tick + 20);
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
            }
            EventId::Alchemist => {
                // I buy strange things. Strange things only. - two gold
                // for whatever odd's in your satchel, whatever it is
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut bought = 0;
                for &v in &self.villagers.clone() {
                    if bought >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    // something, actually - a zero-count entry's a story,
                    // not a strange thing to sell
                    let good = self.world.get_component::<crate::game::livelihood::Satchel>(v)
                        .and_then(|s| s.goods.iter().find(|(_, &n)| n > 0)
                            .map(|(k, _)| k.clone()));
                    let Some(g) = good else { continue };
                    if let Some(s) = self.world.get_component_mut::<crate::game::livelihood::Satchel>(v) {
                        if let Some(n) = s.goods.get_mut(&g) {
                            *n = n.saturating_sub(1);
                            if *n == 0 { s.goods.remove(&g); }
                        }
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold += 2.0; }
                    self.say(v, "Strange. Just what I'm after.", CHAT_BUBBLE_SECS);
                    bought += 1;
                }
            }
            EventId::Quack => {
                // cures all: a penny a dose - most bottles ease (the
                // believing does the work), now and then the batch
                // itself's bad and the belly pays for it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut dosed = 0;
                for &v in &self.villagers.clone() {
                    if dosed >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    // a whole body gains nothing from the dose - he
                    // sells to the ailing, not the hale
                    let ails = self.world.get_component::<Health>(v)
                        .map(|h| h.condition < 1.0).unwrap_or(false);
                    if !near || !pays || !ails { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    // every bottle's its own brew
                    let brew = (self.rare_roll(id) ^ (v as u64).wrapping_mul(0x9E3779B9)) % 4;
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        if brew != 0 {
                            h.condition = (h.condition + 0.04).min(1.0);
                        } else {
                            h.condition = (h.condition - 0.04).max(0.01);
                            self.say(v, "...the belly doesn't sit right.", CHAT_BUBBLE_SECS);
                        }
                    }
                    dosed += 1;
                }
            }
            EventId::Midwife => {
                // babies caught, mothers kept: a home with little ones
                // gets her tending a day - the small one's stronger for
                // her hands and the house feels the blessing
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut tended = 0;
                for &v in &self.villagers.clone() {
                    if tended >= 1 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let Some(h) = household::home_of(&self.world, v) else { continue };
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let occ: Vec<Entity> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default();
                    // she comes where there's a small one to tend - a
                    // held child can't take her hands, and neither can a ghost
                    let has_kid = occ.iter().any(|&o|
                        self.world.get_component::<crate::game::vocation::Kid>(o).is_some()
                            && !villager::is_dead(&self.world, o)
                            && !crate::game::dungeon::unavailable(&self.world, o));
                    if !has_kid { continue; }
                    for o in occ {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        if self.world.get_component::<crate::game::vocation::Kid>(o).is_some() {
                            if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                                hl.condition = (hl.condition + 0.03).min(1.0);
                            }
                        }
                        desires::satisfy(&mut self.world, o, Want::Joy, tick + 10);
                    }
                    tended += 1;
                }
            }
            EventId::Sowgelder => {
                // beasts tended: his hands go over the fold - the stock
                // stands the stronger for it, and the wolves' day gets
                // pushed off while the doctored herd holds its scent
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    // his tending calms the fold - a brewing wolf night
                    // settles while his hands are on the stock
                    self.wolf_coming = false;
                    if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::WolfAtFold) {
                        // past the pack's own scheduling window - it
                        // always moves, never pulls a near wolf closer
                        *due = (*due).max(tick + 500);
                    }
                    let shepherd = self.world.get_component::<household::Building>(f)
                        .and_then(|b| b.worker.or(b.overseer))
                        .filter(|&w| !villager::is_dead(&self.world, w)
                            && !crate::game::dungeon::unavailable(&self.world, w));
                    if let Some(s) = shepherd {
                        desires::satisfy(&mut self.world, s, Want::Security, tick + 15);
                    }
                }
            }
            EventId::Farrier => {
                // shoes for hooves: two gold a shoeing once a visit -
                // a shod beast doesn't founder, and the shepherd's mind
                // rests easy on it
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    // no hooves, no shoes - an emptied fold buys nothing
                    let stock = self.flock.iter().any(|&a|
                        self.world.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.fold == f).unwrap_or(false));
                    let shod = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0;
                    if stock && !shod {
                        let shepherd = self.world.get_component::<household::Building>(f)
                            .and_then(|b| b.worker.or(b.overseer))
                            .filter(|&w| !villager::is_dead(&self.world, w)
                                && !crate::game::dungeon::unavailable(&self.world, w)
                                && self.world.get_component::<Purse>(w).map(|p| p.gold >= 2.0).unwrap_or(false));
                        if let Some(s) = shepherd {
                            if let Some(p) = self.world.get_component_mut::<Purse>(s) { p.gold -= 2.0; }
                            // sound stock from here out - new iron can't
                            // undo a loss already taken
                            desires::satisfy(&mut self.world, s, Want::Security, tick + 20);
                            self.say(s, "Hear that ring of iron!", CHAT_BUBBLE_SECS);
                            if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
                        }
                    }
                }
            }
            EventId::HorseCop => {
                // sound wind and mostly honest - he buys a head off the
                // fold a day, at a dealer's price: three gold, which is
                // cheap, and everyone knows it
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    let head = self.flock.iter().copied()
                        .find(|&a| self.world.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.fold == f).unwrap_or(false));
                    let shepherd = self.world.get_component::<household::Building>(f)
                        .and_then(|b| b.worker.or(b.overseer))
                        .filter(|&w| !villager::is_dead(&self.world, w)
                            && !crate::game::dungeon::unavailable(&self.world, w));
                    if let (Some(a), Some(s)) = (head, shepherd) {
                        self.flock.retain(|&x| x != a);
                        self.world.destroy_entity(a);
                        if let Some(p) = self.world.get_component_mut::<Purse>(s) { p.gold += 3.0; }
                        self.say(s, "Sound wind, and mostly honest!", CHAT_BUBBLE_SECS);
                    }
                }
            }
            EventId::CandleMaker => {
                // wax bought, tallow taken - light for the dark: a home's
                // tallow goes into his dipper and candles come back, a
                // coin for the wax
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let homes: Vec<Entity> = self.world.query::<household::Building>().iter().copied()
                    .filter(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
                    .collect();
                let mut traded = 0;
                for h in homes {
                    if traded >= 2 { break; }
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let first_able = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)));
                    let Some(o) = first_able else { continue };
                    // wax bought, tallow taken: whatever fat's on their
                    // shelf goes in his pot for a coin - most homes have
                    // none, and then he just sells the light
                    let tallow = self.world.get_component::<household::Building>(h)
                        .map(|b| *b.stock.get("tallow").unwrap_or(&0) > 0).unwrap_or(false);
                    let buyer = self.world.get_component::<Purse>(o)
                        .map(|p| p.gold >= 1.0).unwrap_or(false);
                    let mut done = false;
                    if tallow {
                        // the candles have to land before the tallow goes
                        if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                            if household::add_stock(b, "candle", 2) > 0 {
                                household::take_stock(b, "tallow", 1);
                                done = true;
                            }
                        }
                        if done {
                            if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold += 1.0; }
                        }
                    } else if buyer {
                        // he dips his own wax and sells the light for a penny
                        if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                            if household::add_stock(b, "candle", 2) > 0 { done = true; }
                        }
                        if done {
                            if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold -= 1.0; }
                        }
                    }
                    if !done { continue; }
                    for o in self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default() {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        desires::satisfy(&mut self.world, o, Want::Joy, tick + 5);
                    }
                    traded += 1;
                }
            }
            EventId::Potter => {
                // pots and crocks, a jug that doesn't weep: a penny a
                // crock, and a good pot keeps the stew right - the house
                // that owns one eats the better for it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let homes: Vec<Entity> = self.world.query::<household::Building>().iter().copied()
                    .filter(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
                    .collect();
                let mut sold = 0;
                for h in homes {
                    if sold >= 2 { break; }
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let buyer = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)
                                && self.world.get_component::<Purse>(o).map(|p| p.gold >= 1.0).unwrap_or(false)));
                    let Some(o) = buyer else { continue };
                    // the crock has to fit the shelf before the penny goes
                    let fits = self.world.get_component_mut::<household::Building>(h)
                        .map(|b| household::add_stock(b, "crock", 1) > 0).unwrap_or(false);
                    if !fits { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold -= 1.0; }
                    for o in self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default() {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        desires::satisfy(&mut self.world, o, Want::GoodFood, tick + 10);
                    }
                    sold += 1;
                }
            }
            EventId::Eggler => {
                // eggs, butter, cheese - what have your hens done? a
                // coin a basket for whatever the dairy's put by, from
                // whoever's actually about to take it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let homes: Vec<Entity> = self.world.query::<household::Building>().iter().copied()
                    .filter(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
                    .collect();
                let mut bought = 0;
                for h in homes {
                    if bought >= 2 { break; }
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    let good = self.world.get_component::<household::Building>(h)
                        .and_then(|b| ["eggs", "butter", "cheese"].iter()
                            .find(|g| *b.stock.get(**g).unwrap_or(&0) > 0)
                            .map(|g| g.to_string()));
                    let first_able = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)));
                    if !near { continue; }
                    let Some(g) = good else { continue };
                    let Some(o) = first_able else { continue };
                    if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                        household::take_stock(b, &g, 1);
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold += 1.0; }
                    self.say(o, "What have your hens done, eh?", CHAT_BUBBLE_SECS);
                    bought += 1;
                }
            }
            EventId::CheeseMonger => {
                // aged wheels bought, the older the better: the cellar's
                // oldest wheel goes for a monger's price - more for the
                // days on its rind - to the shepherd who made it
                if self.cellar.is_none() || self.cheeses.is_empty() { return; }
                let age = tick - self.cheeses[0];
                let worth = crate::game::goods::worth_at(&self.world, "cheese") * 2.0 * (1.0 + age as f32 / 60.0);
                self.cheeses.remove(0);
                let shepherd = self.shepherd
                    .filter(|&s| !villager::is_dead(&self.world, s)
                        && !crate::game::dungeon::unavailable(&self.world, s));
                if let Some(s) = shepherd {
                    if let Some(p) = self.world.get_component_mut::<Purse>(s) { p.gold += worth; }
                    self.say(s, "The older the better - fine rind!", CHAT_BUBBLE_SECS);
                }
            }
            EventId::Messenger => {
                // a letter, is there one here called-? news rides in for
                // one of them - most times a kind word from far off, and
                // now and then the word a man never wants
                let delivered = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0;
                if delivered { return; }
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                // the roll picks whose door - not always the same man's
                let Some(&v) = able.get((self.rare_roll(id) as usize / 7) % able.len().max(1)) else { return };
                if self.rare_roll(id) % 5 != 0 {
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 20);
                    self.say(v, "A letter for you - your sister's well!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a messenger rides in - a letter, and glad word with it.", tick));
                } else {
                    // the word a man never wants - grief, not stock gone
                    desires::add_grievance(&mut self.world, v, desires::Problem::Grief, tick + 30);
                    self.say(v, "A letter for you - I'm sorry to carry it.", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a messenger rides in - the letter brings no joy.", tick));
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
            }
            EventId::NewsCrier => {
                // the news gives them something to talk about: everyone
                // in earshot goes home with a day's more to say to the
                // one they talk to most
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                // one pass over every bond serves the whole crowd
                let bonds = crate::game::relationships::bond_map(&self.world);
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    // the news goes home to their closest living bond -
                    // the dead hear no gossip, and the record keeps its kind honest
                    let partner = bonds.get(&v).into_iter().flatten()
                        .filter_map(|&r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                        .filter(|rr| (rr.from == v || rr.other == v)
                            && !villager::is_dead(&self.world, if rr.from == v { rr.other } else { rr.from }))
                        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|rr| if rr.from == v { rr.other } else { rr.from });
                    if let Some(to) = partner {
                        crate::game::relationships::record_interaction(&mut self.world, v,
                            crate::game::relationships::Interaction {
                                other: to, kind: crate::game::relationships::InteractionKind::Conversation,
                                intensity: 0.2 });
                    }
                }
            }
            EventId::WineMerchant => {
                // "a cup before you buy": anyone in reach with a thirst
                // pays the pedlar's price for the road's vintage
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Drink, tick) { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Drink, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                }
            }
            _ => {}
        }
    }

    /// road: the `event_end` arms for this group (the rest fall to the default)
    pub(super) fn road_end(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::ChimneySweep => {
                // clean flues to the cap: the next chimney fire's far off
                if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::ChimneyFire) {
                    *due = (*due).max(tick + 150);
                }
                self.log.push(format!("t{}: every flue's clean to the cap - the sweep earned his soot.", tick));
            }
            EventId::RatCatcher => {
                // a cleared larder holds: the rat plague's far off
                if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::RatPlague) {
                    *due = (*due).max(tick + 150);
                }
                self.log.push(format!("t{}: a sack of rats for the road - the larder breathes easier.", tick));
            }
            EventId::Dowser => {
                // the forked stick dips or it doesn't - when it does, a
                // spring in the water table feeds the gardens on the mark
                // he left (it feeds them, mind: a drowned pit yields no
                // stone, so the mark's never on the pit itself)
                if self.rare_roll(id) % 4 == 0 {
                    self.log.push(format!("t{}: the stick never dipped - dry ground all round, says the diviner.", tick));
                } else {
                    for g in self.gardens.iter_mut() { g.2 = (g.2 + 0.3).min(1.0); }
                    self.log.push(format!("t{}: water sings under the meadow - the gardens drink it in.", tick));
                }
            }
            EventId::Recruiter => {
                // the levy marches off - and a young one may march with
                // him, if the drum caught one while he was here
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let kid = self.villagers.iter().copied()
                    .filter(|&v| v != self.player
                        && !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<crate::game::vocation::Kid>(v).is_some()
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false))
                    .min_by_key(|&v| self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs())
                        .unwrap_or(9999));
                match kid {
                    Some(v) if self.rare_roll(id) % 3 != 0 => {
                        // off the roster, and every index that pointed at
                        // the line shifts home with it - the parallel
                        // rolls (was_sick, died_on) shift too, or they'd
                        // hang the wrong records on everyone after him
                        let idx = self.villagers.iter().position(|&x| x == v).unwrap();
                        self.villagers.remove(idx);
                        self.was_sick.remove(idx);
                        self.died_on.remove(idx);
                        for seat in self.seats.iter_mut() {
                            if seat.selected_target > idx { seat.selected_target -= 1; }
                            else if seat.selected_target == idx {
                                seat.selected_target = idx.min(self.villagers.len().saturating_sub(1));
                            }
                        }
                        // his bed's free (the org rolls prune him next
                        // influence tick - the destroyed lack Health, so
                        // is_dead reads them gone)
                        for b in self.world.query::<household::Building>() {
                            if let Some(bb) = self.world.get_component_mut::<household::Building>(b) {
                                bb.occupants.retain(|&x| x != v);
                            }
                        }
                        self.world.destroy_entity(v);
                        self.log.push(format!("t{}: the levy took the youngest - they marched off at dawn.", tick));
                    }
                    _ => self.log.push(format!("t{}: the drum got no one - the village kept its young.", tick)),
                }
            }
            EventId::Deserter => {
                // sheltered, the law counts the knowing: a writ rides
                // hard behind him - the next one lands double
                if self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0 {
                    self.repression_debt += 1;
                    self.log.push(format!("t{}: riders asking after a deserter - a writ for harboring comes hard behind.", tick));
                } else {
                    self.log.push(format!("t{}: the deserter melted off at dawn - no one saw anything.", tick));
                }
            }
            EventId::CharcoalBurner => {
                // the heap comes down: the sacks get carried into the
                // village's first home - burns cleaner than wood all winter
                let shed = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false));
                if let Some(h) = shed {
                    if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                        household::add_stock(b, "coal", 20);
                    }
                }
                self.log.push(format!("t{}: the charcoal burner broke his heap - he left twenty good sacks of coal.", tick));
            }
            EventId::WellDigger => {
                // sweet water, not the pond's: the sour gut clears
                // where clean water runs, and the beds drink deep
                for &v in &self.villagers.clone() {
                    if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                        l.grievances.retain(|(p, _)| *p != desires::Problem::SourGut);
                    }
                }
                for g in self.gardens.iter_mut() {
                    g.2 += 0.05;
                }
                self.log.push(format!("t{}: the well-digger's shaft struck sweet water - the pond's days are done.", tick));
            }
            EventId::RefugeeFamily => {
                // fed and rested, the road's still a hard bed: some put
                // down roots where they camped - two more beds for it
                if self.rare_roll(id) % 3 == 0 {
                    self.log.push(format!("t{}: the refugee family moved on at dawn - the road took them north.", tick));
                } else {
                    // their faces are the roll's own: a temp stream off
                    // the event seed, the village's rng keeps its luck
                    let mut trng = Rng(self.rare_roll(id));
                    let (x, y) = ((self.town_wall.0 + self.town_wall.2) as f32 / 2.0, self.town_wall.3 as f32 + 2.0);
                    // a home with a spare bed takes them, if there is one
                    let home = self.world.query::<household::Building>().iter().copied()
                        .find(|&b| !self.building_ruined(b) && self.world.get_component::<household::Building>(b)
                            .map(|bb| bb.kind == household::BuildingKind::Home && bb.occupants.len() < 4)
                            .unwrap_or(false));
                    for _ in 0..2 {
                        let v = villager::spawn_villager(&mut self.world, x, y);
                        self.world.add_component(v, Health { condition: 0.7, immunity: 0.4 });
                        self.world.add_component(v, crate::sim::community::Citizen {
                            influence: 0.1, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
                        self.world.add_component(v, crate::game::traits::personality_from_seed(trng.next()));
                        self.world.add_component(v, crate::game::livelihood::Satchel::default());
                        self.world.add_component(v, desires::Known::default());
                        let voc = crate::session::roll_vocation(&mut trng);
                        let mut trng2 = Rng(trng.0);
                        self.world.add_component(v, desires::InnerLife {
                            wants: desires::sample_wants(
                                crate::game::vocation::SocialClass::Working, Some(voc), false,
                                &mut move || trng2.next()),
                            grievances: vec![], satisfied: vec![] });
                        self.world.add_component(v, crate::game::vocation::craft(voc, 0.2));
                        self.world.add_component(v, Purse { gold: 0.5 });
                        self.villagers.push(v);
                        // a roof where there's room (the camp's behind them)
                        if let Some(h) = home {
                            if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                                b.occupants.push(v);
                            }
                        }
                        // the parallel rolls grow with the roster
                        self.was_sick.push(None);
                        self.died_on.push(None);
                    }
                    self.log.push(format!("t{}: the refugee family stayed - two more beds in the village.", tick));
                }
            }
            _ => {}
        }
    }
}
