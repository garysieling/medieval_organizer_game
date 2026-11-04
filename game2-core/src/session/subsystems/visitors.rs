//! Visitors who come and go: the minstrel, the pedlar, the circus and the ink-boiler. Each spawns, walks its round and leaves.

use crate::session::*;

impl Game {
    /// The travelling minstrel rides in
    pub fn spawn_minstrel(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        let path = self.map.walk_path(start, (16, 16)).unwrap_or_default();
        self.minstrel = Some(Minstrel { e, path, playing_until: None });
        self.log.push(format!("t{}: a travelling minstrel has come to town.", self.tick));
    }

    /// One frame of the minstrel's visit: ride to the market, play a
    /// while, then ride off - and the town wants music again next season
    pub fn advance_minstrel(&mut self) {
        let Some(mut m) = self.minstrel.take() else { return };
        // the song's no good to anyone with the thatch alight: he turns
        // his horse for the road home
        if self.fire_at_feet(m.e, &m.path) {
            m.playing_until = Some(self.tick.saturating_sub(1));
            m.path.clear();
        }
        if m.playing_until.is_none() {
            // riding to the market
            let Some(p) = self.world.get_component_mut::<Pos>(m.e) else { return };
            if m.path.is_empty() {
                m.playing_until = Some(self.tick + 2);
            } else {
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.minstrel = Some(m);
        } else if self.tick <= m.playing_until.unwrap() {
            // playing at the market: everyone nearby gets their fill of it
            if let Some(p) = self.world.get_component::<Pos>(m.e).copied() {
                for &l in self.villagers.iter().chain(std::iter::once(&self.seats[0].player)) {
                    let near = self.world.get_component::<Pos>(l)
                        .map(|lp| (lp.x - p.x).abs() + (lp.y - p.y).abs() < 5.0).unwrap_or(false);
                    if near {
                        desires::satisfy(&mut self.world, l, Want::Music, self.tick + 90);
                        desires::satisfy(&mut self.world, l, Want::Joy, self.tick + 45);
                    }
                }
            }
            self.minstrel = Some(m);
        } else {
            // the road calls: ride off, and another comes by one day
            if let Some(p) = self.world.get_component_mut::<Pos>(m.e).copied() {
                let back = self.traveller_path.last().copied().unwrap_or((29, 14));
                if m.path.is_empty() {
                    m.path = self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back).unwrap_or_default();
                }
                if m.path.is_empty() {
                    self.world.destroy_entity(m.e);
                    self.next_minstrel = self.tick + 250 + self.rng.next() % 150;
                    return; // he's gone
                }
                let p = self.world.get_component_mut::<Pos>(m.e).unwrap();
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.minstrel = Some(m);
        }
    }

    /// The roving pedlar comes up the road
    pub fn spawn_pedlar(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        let path = self.map.walk_path(start, (16, 16)).unwrap_or_default();
        self.pedlar = Some(Pedlar { e, path, selling_until: None, stall: None, rumor_sold: false });
        self.log.push(format!("t{}: a pedlar comes up the road - pots, pins, and word of the next village.", self.tick));
    }

    /// One frame of the pedlar's visit: walk in, pitch the stall, trade
    /// gossip and oddities a few days, then the road takes him
    pub fn advance_pedlar(&mut self) {
        let Some(mut m) = self.pedlar.take() else { return };
        // no stall is worth a burning pitch: he shoulders his pack
        if self.fire_at_feet(m.e, &m.path) {
            m.selling_until = Some(self.tick.saturating_sub(1));
            m.path.clear();
        }
        if m.selling_until.is_none() {
            // walking to the market
            let Some(p) = self.world.get_component_mut::<Pos>(m.e) else { return };
            if m.path.is_empty() {
                // pitched: a stall of oddities nothing local supplies
                let (px, py) = (p.x, p.y);
                let stall = self.world.create_entity();
                self.world.add_component(stall, Pos { x: px, y: py });
                let mut stock = crate::engine::det::HashMap::default();
                stock.insert("ribbon".to_string(), 4);
                stock.insert("needle".to_string(), 6);
                stock.insert("lute_string".to_string(), 2);
                self.world.add_component(stall, household::Building {
                    kind: household::BuildingKind::TraderStall,
                    name: "the pedlar's pitch".into(),
                    worker: Some(m.e), stock, occupants: vec![],
                    capacity: 1, overseer: None,
                });
                m.stall = Some(stall);
                m.selling_until = Some(self.tick + 3);
                // and the word goes out: a few with a want for finery come
                let buyers: Vec<Entity> = self.villagers.iter().copied()
                    .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                        && !self.travels.iter().any(|t| t.who == v)
                        && self.world.get_component::<InnerLife>(v)
                            .map(|l| l.wants.contains(&Want::Finery)).unwrap_or(false))
                    .take(4).collect();
                for v in buyers { self.send(v, Goal::BuyFrom(stall, "ribbon", 1)); }
                // and he trades in the only currency he makes: news
                const RUMOURS: [&str; 4] = [
                    "Word from the next village - their harvest failed. Bread's dear there.",
                    "Their fisher drowned his boat last month. No fish on their market.",
                    "The lord raised the fifth to a quarter there. Hard times.",
                    "A wolf took a shepherd on their common. They say it walks upright.",
                ];
                let rumour = RUMOURS[(self.tick / 60 % 4) as usize];
                self.say(m.e, rumour, TALK_BUBBLE_SECS * 2);
            } else {
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.pedlar = Some(m);
        } else if self.tick <= m.selling_until.unwrap() {
            // the river tale: he sells it once, and it gets around -
            // true or not is a world parameter
            if !m.rumor_sold {
                m.rumor_sold = true;
                self.say(m.e, "They say a thing glows under the river - a pale stone, older than the walls.", TALK_BUBBLE_SECS * 2);
                // someone who hears it carries it on
                if let Some(&v) = self.villagers.iter().find(|&&v|
                    !villager::is_dead(&self.world, v)) {
                    self.say(v, "The pedlar swears he saw it glow.", CHAT_BUBBLE_SECS);
                }
            }
            self.pedlar = Some(m); // selling at the market
        } else {
            // the road calls: pack up and walk off
            if let Some(p) = self.world.get_component_mut::<Pos>(m.e).copied() {
                let back = self.traveller_path.last().copied().unwrap_or((29, 14));
                if m.path.is_empty() {
                    m.path = self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back).unwrap_or_default();
                }
                if m.path.is_empty() {
                    if let Some(s) = m.stall { self.world.destroy_entity(s); }
                    self.world.destroy_entity(m.e);
                    self.next_pedlar = self.tick + 60 + self.rng.next() % 60;
                    self.log.push(format!("t{}: the pedlar's moved on.", self.tick));
                    return;
                }
                let p = self.world.get_component_mut::<Pos>(m.e).unwrap();
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.pedlar = Some(m);
        }
    }

    /// The circus comes up the road and pitches on the meadow by the gate
    pub fn spawn_circus(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        // a meadow spot just outside the south wall
        let camp = self.map.nearest_buildable(self.town_wall.0 + 4, self.town_wall.3 + 3)
            .unwrap_or((20, 30));
        let path = self.map.walk_path(start, camp).unwrap_or_default();
        self.circus = Some(Circus { e, path, camped_until: None, camp });
        self.log.push(format!("t{}: a circus comes up the road - tents on the meadow by the gate.", self.tick));
    }

    /// One frame of the circus: walk in, camp a fortnight of wonder and
    /// light fingers, then leave - and maybe not alone
    pub fn advance_circus(&mut self) {
        let Some(mut c) = self.circus.take() else { return };
        // the meadow's burning or the road in is: the tents fold early
        if self.fire_at_feet(c.e, &c.path) {
            c.camped_until = Some(self.tick.saturating_sub(1));
            c.path.clear();
        }
        if c.camped_until.is_none() {
            let Some(p) = self.world.get_component_mut::<Pos>(c.e) else { return };
            if c.path.is_empty() {
                c.camped_until = Some(self.tick + 10);
                self.say(c.e, "Come see! Wonders from over the hill!", TALK_BUBBLE_SECS * 2);
            } else {
                let (nx, ny) = (c.path[0].0 as f32, c.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; c.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.circus = Some(c);
        } else {
            self.circus = Some(c);
        }
    }

    /// Each day the circus camps: wonder for those who come, a lighter
    /// purse for the same, and the pull grows on the ones the days here
    /// have worn thin - until the tents fold, and someone may go with them
    pub fn circus_tick(&mut self, tick: u64) {
        let camped = self.circus.as_ref().and_then(|c| c.camped_until);
        let Some(until) = camped else { return };
        let camp = self.circus.as_ref().map(|c| c.camp).unwrap();
        let org = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        if tick < until {
            // a night of wonders: joy for the ones who walk out, and a
            // few coins lighter walking home
            for &v in &self.villagers.clone() {
                if villager::is_dead(&self.world, v) { continue; }
                let near = self.world.get_component::<Pos>(v)
                    .map(|p| (p.x.round() as i32 - camp.0).abs() + (p.y.round() as i32 - camp.1).abs() < 8)
                    .unwrap_or(false);
                if !near { continue; }
                desires::satisfy(&mut self.world, v, Want::Joy, tick + 10);
                desires::satisfy(&mut self.world, v, Want::Music, tick + 10);
                if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                    if p.gold > 1.0 { p.gold -= 1.0; } // light fingers in the crowd
                }
                // the bored and the unbelonging feel the pull - belonging
                // is the counterweight
                if !org.contains(&v) && self.world.get_component::<TownRole>(v).is_none() {
                    desires::add_grievance(&mut self.world, v, desires::Problem::Tempted, until);
                }
            }
        } else {
            // the tents fold - and the road takes the one it most tempted
            let mut bond_n: crate::engine::det::HashMap<Entity, usize> = crate::engine::det::HashMap::default();
            self.world.iter::<relationships::Relationship>(|_, r| {
                *bond_n.entry(r.from).or_default() += 1;
                if r.other != r.from { *bond_n.entry(r.other).or_default() += 1; }
            });
            let mark = self.villagers.iter().copied()
                .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v) && !org.contains(&v)
                    && self.world.get_component::<TownRole>(v).is_none()
                    && desires::problems_of(&self.world, v, tick).contains(&desires::Problem::Tempted))
                .min_by_key(|&v| {
                    // the loneliest goes: fewest bonds is the lightest tie
                    bond_n.get(&v).copied().unwrap_or(0)
                });
            if let Some(m) = mark {
                let name = self.name_of(m);
                self.log.push(format!("t{}: {} has run off with the circus!", tick, name));
                // the road takes them whole - whatever errand they were
                // on is spent
                self.travels.retain(|t| t.who != m);
                self.send(m, Goal::Exile);
            }
            self.circus = None;
            self.next_circus = tick + 300 + self.rng.next() % 200;
            self.log.push(format!("t{}: the circus folds its tents and goes.", tick));
        }
    }

    /// Flames at a traveller's feet or on the road just ahead - reason
    /// enough for any of them to turn for home
    pub(in crate::session) fn fire_at_feet(&self, e: Entity, path: &[(i32, i32)]) -> bool {
        if !self.fire.active() { return false; }
        let near = self.world.get_component::<Pos>(e)
            .map(|p| self.fire.burns_near(p.x.round() as i32, p.y.round() as i32, 2))
            .unwrap_or(false);
        near || path.first().map(|&(x, y)| self.fire.on_fire(x, y)).unwrap_or(false)
    }

    /// The ink-maker comes up the road and pitches his cauldron on the
    /// meadow outside the gate - where the law wants a fire like his
    pub fn spawn_ink_boiler(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        // a clear spot beyond the south wall, off the circus's patch
        let camp = self.map.nearest_buildable(self.town_wall.0 + 8, self.town_wall.3 + 4)
            .unwrap_or((24, 30));
        let path = self.map.walk_path(start, camp).unwrap_or_default();
        self.ink_boiler = Some(InkBoiler { e, path, working_until: None, camp });
        self.log.push(format!("t{}: an ink-maker comes up the road - his cauldron will smoke on the meadow, where such fires belong.", self.tick));
    }

    /// One frame of the ink-maker's visit: walk out to the meadow, work
    /// the cauldron a few days, then the road takes him back
    pub fn advance_ink_boiler(&mut self) {
        let Some(mut b) = self.ink_boiler.take() else { return };
        // flames come for a pot of boiling oil too: he packs and goes
        if self.fire_at_feet(b.e, &b.path) {
            b.working_until = Some(self.tick.saturating_sub(1));
            b.path.clear();
        }
        if b.working_until.is_none() {
            let Some(p) = self.world.get_component_mut::<Pos>(b.e) else { return };
            if b.path.is_empty() {
                b.working_until = Some(self.tick + INK_BOIL_DAYS);
                self.say(b.e, "The oil takes its time - bring your crusts when it spits.", TALK_BUBBLE_SECS * 2);
            } else {
                let (nx, ny) = (b.path[0].0 as f32, b.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; b.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.ink_boiler = Some(b);
        } else if self.tick <= b.working_until.unwrap() {
            self.ink_boiler = Some(b); // the pot's on
        } else {
            // the varnish's made: pack up and walk off
            if let Some(p) = self.world.get_component_mut::<Pos>(b.e).copied() {
                let back = self.traveller_path.last().copied().unwrap_or((29, 14));
                if b.path.is_empty() {
                    b.path = self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back).unwrap_or_default();
                }
                if b.path.is_empty() {
                    self.world.destroy_entity(b.e);
                    self.next_ink_boiler = self.tick + 250 + self.rng.next() % 200;
                    self.log.push(format!("t{}: the ink-maker's moved on, smelling of his trade.", self.tick));
                    return;
                }
                let p = self.world.get_component_mut::<Pos>(b.e).unwrap();
                let (nx, ny) = (b.path[0].0 as f32, b.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; b.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.ink_boiler = Some(b);
        }
    }

    /// Each day the cauldron's on, the recipe walks its stages: two days
    /// of boil, the crusts and onions that take the grease out, the slate
    /// drip that says it's varnish, and the black going in - then the
    /// print shop's stocked with ink the road used to have to bring
    pub fn ink_boiler_tick(&mut self, tick: u64) {
        let Some(b) = &self.ink_boiler else { return };
        let Some(until) = b.working_until else { return };
        let (camp, be) = (b.camp, b.e);
        match tick - (until - INK_BOIL_DAYS) {
            2 => {
                // day three: a pound of crusts and a dozen onions go in
                // to degrease - someone carries them out and comes back
                // smelling of frying oil
                self.say(be, "A pound of crusts and an onion or two - they take the grease out.", TALK_BUBBLE_SECS);
                let candidates: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                    !villager::is_dead(&self.world, v)
                        && self.world.get_component::<TownRole>(v).is_none()
                        && !self.travels.iter().any(|t| t.who == v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - camp.0).abs() + (p.y.round() as i32 - camp.1).abs() <= 8)
                            .unwrap_or(false)
                }).collect();
                // ask until someone can actually make the walk
                for v in candidates {
                    if self.send(v, Goal::TalkTo(be)) {
                        self.log.push(format!("t{}: a villager carries the day's crusts out to the cauldron.", tick));
                        break;
                    }
                }
            }
            3 => {
                // day four: dripped on the slate it hangs like thin glue
                self.say(be, "See it hang off the slate? That's varnish now.", TALK_BUBBLE_SECS);
                self.log.push(format!("t{}: the oil's boiled down to varnish - sticky as thin glue off the slate.", tick));
            }
            4 => {
                // day five: the lampblack goes in - soot, not ash - and
                // it's ink; the print shop's stocked till he comes again
                let shop = self.building_entities.iter().copied().find(|&e| {
                    self.world.get_component::<Building>(e)
                        .map(|b| matches!(b.kind, household::BuildingKind::Shop(Vocation::Printmaker)))
                        .unwrap_or(false)
                }).or(self.trader_stall);
                if let Some(shop) = shop {
                    if let Some(b) = self.world.get_component_mut::<Building>(shop) {
                        household::add_stock(b, "ink", INK_YIELD);
                    }
                }
                self.say(be, "The black goes in - there's your ink.", TALK_BUBBLE_SECS);
                self.log.push(format!("t{}: lampblack into the varnish - the ink-maker sells {} ink before he goes.", tick, INK_YIELD));
            }
            // days one and two: the oil just boils, watched and smoking -
            // and now and then the vapours take fire, as they will; and
            // the acrolein reaches whoever's downwind - sharp on the eyes
            0 | 1 => {
                if self.rng.below(100) < OIL_FLARE_PCT && self.fire.ignite(&self.map, camp.0, camp.1) {
                    self.say(be, "She's caught - keep back!", TALK_BUBBLE_SECS);
                    self.log.push(format!("t{}: the oil takes fire on the meadow.", tick));
                }
                let near: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                    !villager::is_dead(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - camp.0).abs() + (p.y.round() as i32 - camp.1).abs() <= 6)
                            .unwrap_or(false)
                }).collect();
                for v in near {
                    desires::add_grievance(&mut self.world, v, desires::Problem::Smelly, tick + 30);
                }
            }
            _ => {}
        }
    }
}
