//! Power and its pushback: sanctions and reputation, edicts, the dungeon and the guards, marks and the martyr, discoveries, organized power, and the festival.

use crate::session::*;

impl Game {
    /// The ladder of shame: when a villager's grievances pile past a
    /// rung, the village acts - one rung at a time, and the crowd's
    /// answer decides whether it lands
    pub fn sanction_tick(&mut self, tick: u64) {
        if tick % 7 != 4 { return; }
        // drop lapsed sanctions; the cold shoulder doesn't last forever
        self.sanctioned.retain(|(_, _, until)| tick < *until);
        // the most-grieved soul in the village
        let target = self.villagers.iter().copied()
            .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<TownRole>(v).is_none())
            .map(|v| (v, self.world.get_component::<InnerLife>(v)
                .map(|l| l.grievances.iter().filter(|(_, u)| tick < *u).count()).unwrap_or(0)))
            .filter(|(_, n)| *n >= 3)
            .max_by_key(|(_, n)| *n);
        let Some((v, n)) = target else { return };
        let current = self.sanctioned.iter().filter(|(e, _, _)| *e == v)
            .map(|(_, s, _)| *s).max();
        let next = sanctions::rung_for(n);
        // one rung at a time, and only upward
        let step = match (current, next) {
            // blame earns a bad word first; the ladder is climbed
            (None, Some(_)) => Some(Sanction::Gossip),
            (Some(cur), Some(want)) if want > cur => {
                let i = sanctions::LADDER.iter().position(|s| *s == cur).unwrap_or(0);
                Some(sanctions::LADDER[(i + 1).min(5)])
            }
            _ => None,
        };
        if let Some(s) = step { self.apply_sanction(v, s, tick); }
    }

    /// A rung lands: what the village does to its own
    pub(in crate::session) fn apply_sanction(&mut self, v: Entity, s: Sanction, tick: u64) {
        let name = self.villagers.iter().position(|x| *x == v).map(|i| vill_name(i)).unwrap_or("someone");
        // a bad word travels ahead of every rung
        self.rep_apply(v, RepAction::BadDeed);
        self.sanctioned.push((v, s, tick + 60));
        match s {
            Sanction::Gossip => {
                self.log.push(format!("t{}: the talk turns on {}.", tick, name));
                // the words they reach for, flourished
                self.say(v, &format!("They call me {}.", gossip_insult(v as usize)), TALK_BUBBLE_SECS);
            }
            Sanction::Shun => {
                self.log.push(format!("t{}: {} gets the cold shoulder - no one sits with them.", tick, name));
            }
            Sanction::Boycott => {
                self.log.push(format!("t{}: nobody buys from {} - the boycott bites.", tick, name));
            }
            Sanction::Picket => {
                self.log.push(format!("t{}: a crowd at {}'s door - they can't work for the press.", tick, name));
                self.say(v, "Let me through - I've work to do!", TALK_BUBBLE_SECS);
            }
            Sanction::Stocks => {
                // the crowd's answer decides: a respected soul shamed is a
                // scandal, a disliked one shamed is a lesson
                if self.rep_of(v) > 0.5 {
                    self.rep_apply(v, RepAction::GreatDeed); // sympathy lifts them
                    self.log.push(format!("t{}: the crowd would have none of it - {} walks free of the stocks, and the shame lands on the accusers.", tick, name));
                    self.say(v, "They wronged me, and you all saw it.", TALK_BUBBLE_SECS);
                } else {
                    self.rep_apply(v, RepAction::Betrayal);
                    self.log.push(format!("t{}: {} sits in the stocks by the store.", tick, name));
                    if let Some(store) = Some(self.market_food) { self.send(v, Goal::HeadHome(store)); }
                    self.say(v, "...all right. All right.", TALK_BUBBLE_SECS);
                }
            }
            Sanction::Exile => {
                self.log.push(format!("t{}: the village turns {} out - nothing left for them here.", tick, name));
                self.send(v, Goal::Exile);
            }
        }
    }

    /// Their standing with the crowd (seeded mid if never tracked)
    pub(in crate::session) fn rep_of(&mut self, v: Entity) -> f32 {
        if self.world.get_component::<ReputationState>(v).is_none() {
            self.world.add_component(v, ReputationState {
                stage: RepStage::Known, score: 0.45, transition_rates: vec![],
            });
        }
        self.world.get_component::<ReputationState>(v).map(|r| r.score).unwrap_or(0.45)
    }

    pub(in crate::session) fn rep_apply(&mut self, v: Entity, a: RepAction) {
        let _ = self.rep_of(v);
        if let Some(r) = self.world.get_component_mut::<ReputationState>(v) {
            crate::sim::reputation_qsp::apply_action(r, a);
        }
    }

    /// Is this shop's worker under boycott or worse? Their trade dries up
    pub fn boycotted(&self, shop: Entity) -> bool {
        let Some(worker) = self.world.get_component::<Building>(shop).and_then(|b| b.worker) else { return false };
        self.sanctioned.iter().any(|(e, s, _)| *e == worker && *s >= Sanction::Boycott)
    }

    /// Is this villager under the cold shoulder or worse?
    pub fn shunned(&self, v: Entity) -> bool {
        self.sanctioned.iter().any(|(e, s, _)| *e == v && *s >= Sanction::Shun)
    }

    /// The lord's arm: every season a writ comes down - a levy of grain,
    /// a fee in gold, a summons for one of the village's own. An org
    /// worth the name bargains it down; a strong one may stand against
    /// a summons - and pay for it when the next writ comes double.
    pub fn edict_tick(&mut self, tick: u64) {
        // word rides ahead of the bailiff: the village hears the writ's
        // coming a few days before it lands - time to gather, to bargain,
        // or to hide the grain
        if tick % 120 == 76 {
            self.log.push(format!("t{}: word on the road - the bailiff rides this way. A writ comes down in a few days.", tick));
        }
        if tick % 120 != 80 { return; }
        let kind = (tick / 120) % 4;
        // the debt counts the defiances - a summons reads it before the
        // writ's own weight consumes it
        let debt = self.repression_debt;
        let mult = if self.repression_debt > 0 { let m = 1 + self.repression_debt; self.repression_debt = 0; m } else { 1 };
        // the org's answer rides its standing with the lord
        let bargained = self.lord_esteem > 0.5;
        match kind {
            0 => {
                // a levy: grain off the store
                let want = (8 * mult) as f32;
                let take = if bargained { want / 2.0 } else { want } as u32;
                if let Some(b) = self.world.get_component_mut::<Building>(self.market_food) {
                    let _ = household::take_stock(b, "grain", take);
                }
                self.log.push(format!("t{}: a bailiff's writ - the castle levies {} grain{}.{}",
                    tick, take,
                    if bargained { " - the org talks him down" } else { "" },
                    if mult > 1 { " Double, for the last defiance." } else { "" }));
            }
            1 => {
                // a fee: gold off every purse that has it
                let want = (12 * mult) as f32;
                let each = if bargained { want / 2.0 } else { want };
                let mut taken = 0.0f32;
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                        let cut = (each / self.villagers.len() as f32).min(p.gold);
                        p.gold -= cut; taken += cut;
                    }
                }
                self.log.push(format!("t{}: a bailiff's writ - a fee of {:.0} gold off the village.{}",
                    tick, taken, if bargained { " Halved - the org's word counts." } else { "" }));
            }
            3 => {
                // the drink levy: the castle's share of every barrel in
                // the village - a quarter of what pours, off every stock
                let share = |q: u32| ((q as f32) * if bargained { 0.125 } else { 0.25 }).ceil() as u32;
                let mut taken = 0u32;
                for &be in &self.building_entities.clone() {
                    if let Some(b) = self.world.get_component_mut::<Building>(be) {
                        for g in ["ale", "beer", "wine", "spirits"] {
                            let q = b.stock.get(g).copied().unwrap_or(0);
                            if q > 0 { taken += household::take_stock(b, g, share(q)); }
                        }
                    }
                }
                self.log.push(format!("t{}: a bailiff's writ - the castle's share of the barrel, {} measures off the village's drink.{}",
                    tick, taken,
                    if bargained { " Halved - the org's word counts." } else { "" }));
            }
            _ => {
                // a summons: one of the village's own for 'questioning' -
                // a strong org can stand against it
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<TownRole>(v).is_none())
                    .max_by_key(|&v| self.world.get_component::<InnerLife>(v)
                        .map(|l| l.grievances.len()).unwrap_or(0));
                if let Some(m) = mark {
                    let member = self.world.get_component::<Organization>(self.org)
                        .map(|o| o.members.contains(&m)).unwrap_or(false);
                    if member && self.lord_esteem > 0.7 && debt >= 2 {
                        // the third defiance: the bailiff doesn't come back
                        // alone - a writ at max repression is a gibbet
                        let name = self.name_of(m);
                        self.world.add_component(m, villager::Dead);
                        if let Some(i) = self.villagers.iter().position(|&x| x == m) {
                            self.died_on[i] = Some(tick);
                        }
                        if let Some(p) = self.world.get_component::<Pos>(m) {
                            self.martyr_grave = Some((p.x.round() as i32, p.y.round() as i32));
                        }
                        self.repression_debt = 0;
                        self.log.push(format!("t{}: the bailiff returns with knights - {} is hanged for the defiance.", tick, name));
                        // mass grief: everyone saw it
                        for v in self.villagers.iter().take(8).copied().collect::<Vec<_>>() {
                            if !villager::is_dead(&self.world, v) {
                                desires::add_grievance(&mut self.world, v, desires::Problem::Grief, tick + 60);
                            }
                        }
                    } else if member && self.lord_esteem > 0.7 {
                        self.repression_debt += 1;
                        self.log.push(format!("t{}: a writ for {} - and the village stands against it. The bailiff rides home empty.", tick, self.name_of(m)));
                    } else {
                        self.log.push(format!("t{}: a summons - {} is taken for questioning.", tick, self.name_of(m)));
                        for _ in 0..2 { desires::add_grievance(&mut self.world, m, desires::Problem::WritBorne, tick + 60); }
                        // taken to the castle - held, not gone: the org
                        // ransoms them or the ladder climbs. The march
                        // out goes first - it's the last errand they run
                        // as a free pair of legs, then the mark lands
                        self.dungeon.push((m, tick));
                        self.travels.retain(|t| t.who != m);
                        self.send(m, Goal::Exile);
                        self.world.add_component(m, crate::game::dungeon::CastleHeld { since: tick });
                    }
                }
            }
        }
        // and the writ itself is a thing to organize about
        for v in self.villagers.iter().take(3).copied().collect::<Vec<_>>() {
            if !villager::is_dead(&self.world, v) {
                desires::add_grievance(&mut self.world, v, desires::Problem::WritBorne, tick + 30);
            }
        }
    }

    /// The castle's prisoners: a month to ransom them, or the ladder
    /// climbs to its top. The org's purse answers for its own.
    pub fn dungeon_tick(&mut self, tick: u64) {
        if self.dungeon.is_empty() || tick % 15 != 5 { return; }
        let mut resolved: Vec<(Entity, bool)> = vec![]; // (who, ransomed?)
        for &(who, taken) in &self.dungeon.clone() {
            let held = tick - taken;
            let members = self.world.get_component::<Organization>(self.org)
                .map(|o| o.members.clone()).unwrap_or_default();
            let purse = self.world.get_component::<PlayerCharacter>(self.player)
                .map(|p| p.resources).unwrap_or(0.0);
            if held >= 30 && held < 60 && !members.is_empty() && purse >= 20.0 {
                // the org pays the ransom: the purse answers for its own
                if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.player) {
                    p.resources -= 20.0;
                }
                resolved.push((who, true));
            } else if held >= 60 {
                // a month unanswered: the ladder climbs to its top
                resolved.push((who, false));
            }
        }
        for (who, ransomed) in resolved {
            self.dungeon.retain(|(e, _)| *e != who);
            let name = self.name_of(who);
            if ransomed {
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(who);
                // the walk out is spent with the ransom - else the exile
                // leg outruns the pardon and arrives a driving-out
                self.travels.retain(|t| t.who != who);
                self.log.push(format!("t{}: the org pays the ransom - {} walks home.", tick, name));
                if let Some(home) = household::home_of(&self.world, who).or(Some(self.market_food)) {
                    self.send(who, Goal::HeadHome(home));
                }
            } else {
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(who);
                self.world.add_component(who, villager::Dead);
                if let Some(i) = self.villagers.iter().position(|&x| x == who) {
                    self.died_on[i] = Some(tick);
                }
                self.log.push(format!("t{}: no ransom came - {} is hanged at the castle.", tick, name));
            }
        }
    }

    /// The king's patience has a length: posted guards stand their tour,
    /// then ride home - the streets are the village's again, and a
    /// crackdown that comes after has to post them fresh
    /// The cell lets go of one prisoner: the components clear, and a
    /// release campaign raised just for them is spent with the release
    pub(in crate::session) fn free_prisoner(&mut self, m: Entity, tick: u64) {
        self.world.remove_component::<crate::game::dungeon::Imprisoned>(m);
        self.world.remove_component::<crate::game::dungeon::ReleaseCampaign>(m);
        if let Some(rc) = self.world.get_component::<crate::game::dungeon::ReleaseOf>(m).map(|r| r.campaign) {
            self.world.remove_component::<crate::game::campaign::Campaign>(rc);
            self.world.remove_component::<crate::game::campaign::CampaignMeeting>(rc);
            self.world.remove_component::<crate::game::dungeon::ReleaseOf>(m);
        }
        self.log.push(format!("t{}: the cell stands open - {} walks home.", tick, self.name_of(m)));
    }

    pub fn guards_tick(&mut self, tick: u64) {
        // a stint inside is a stint inside: nobody sits past one, whatever
        // fresh crackdowns restamp a tour
        for &m in &self.villagers.clone() {
            let Some(since) = self.world.get_component::<crate::game::dungeon::Imprisoned>(m)
                .map(|i| i.since) else { continue; };
            if tick >= since + GUARD_TOUR { self.free_prisoner(m, tick); }
        }
        for seat in 0..self.seats.len() {
            let org = self.seats[seat].org;
            let stood_down = self.world.get_component::<crate::game::reactions::GuardsPosted>(org)
                .map(|g| tick >= g.since + GUARD_TOUR).unwrap_or(false);
            if !stood_down { continue; }
            self.world.remove_component::<crate::game::reactions::GuardsPosted>(org);
            // they stand where they were posted no more: this org's
            // knights ride home (another's may still be on their tour;
            // the gate watch is a different corps entirely)
            self.posted.retain(|&(o, _, _)| o != org);
            // and without the knights the cells can't hold: this org's held
            // walk free early, petitions answered or not
            let members = self.world.get_component::<Organization>(org)
                .map(|o| o.members.clone()).unwrap_or_default();
            for &m in &self.villagers.clone() {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(m).is_none() { continue; }
                // whose cell is it? the org that founded the release
                // campaign owns the prisoner; without one, membership says
                let ours = self.world.get_component::<crate::game::dungeon::ReleaseOf>(m)
                    .and_then(|r| self.world.get_component::<crate::game::campaign::Campaign>(r.campaign)
                        .map(|c| c.org == org))
                    .unwrap_or_else(|| members.contains(&m));
                if ours { self.free_prisoner(m, tick); }
            }
            self.log.push(format!("t{}: the knights ride home - the tour is done.", tick));
        }
    }

    /// What a cause leaves on people fades on its own clock: a season
    /// cools a moral shock (reactions::cool_off), a month dulls a
    /// blow-off (actions::lifts_with_time). Coaching is still the fast
    /// way back - this is just the floor under "stuck forever".
    pub fn marks_tick(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            crate::game::reactions::cool_off(&mut self.world, v, tick);
            crate::game::actions::lifts_with_time(&mut self.world, v, tick);
            // and a name fades the same way: esteem cools without upkeep,
            // even rock-bottom disgrace is eventually forgotten
            if let Some(r) = self.world.get_component_mut::<ReputationState>(v) {
                crate::sim::reputation_qsp::transition_tick(r);
            }
            // folk don't forget the man inside: a little pressure gathers
            // every day even when nobody's pressing - a cell can't hold
            // a name forever on suspicion alone
            if !villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                crate::game::dungeon::press(&mut self.world, v, 0.02);
                if crate::game::dungeon::release_check(&mut self.world, v, 1.0) {
                    let name = self.name_of(v);
                    self.log.push(format!("t{}: the village never stopped asking - {} walks free.", tick, name));
                    if let Some(home) = household::home_of(&self.world, v).or(Some(self.market_food)) {
                        self.send(v, Goal::HeadHome(home));
                    }
                }
            }
        }
    }

    /// The martyr's grave: they gather there, and the remembering is a
    /// kind of organizing no castle can stamp out
    pub fn martyr_tick(&mut self, tick: u64) {
        let Some(_) = self.martyr_grave else { return };
        if tick % 30 != 20 { return; }
        // a vigil: the org's members keep the grave, and it makes them
        for v in self.villagers.iter().take(5).copied().collect::<Vec<_>>() {
            if !villager::is_dead(&self.world, v) {
                desires::satisfy(&mut self.world, v, Want::Faith, tick + 15);
            }
        }
        self.record_power(crate::sim::organized_power::ActionQuality::Transactional);
        if tick % 90 == 20 {
            self.log.push(format!("t{}: they keep a vigil at the martyr's grave.", tick));
        }
    }

    /// The player is the org, not a body: when the first pair of hands
    /// stops, the work passes to the member who knew them best - or the
    /// kin under the same roof - and goes on
    pub fn continuation_tick(&mut self, tick: u64) {
        if self.player_passed || !villager::is_dead(&self.world, self.player) { return; }
        // the successor: an org member first - the organizing's
        // institutional - then kin under the same roof
        let members = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let org_pick = members.iter().copied()
            .filter(|&m| !villager::is_dead(&self.world, m) && m != self.player)
            .max_by_key(|&m| relationships::find_relationship(&self.world, self.player, m)
                .map(|r| self.world.get_component::<relationships::Relationship>(r)
                    .map(|rr| (rr.strength * 100.0) as u32).unwrap_or(0)).unwrap_or(0));
        let kin_pick = household::home_of(&self.world, self.player)
            .and_then(|h| self.world.get_component::<Building>(h)
                .map(|b| b.occupants.iter().copied()
                    .find(|&o| o != self.player && !villager::is_dead(&self.world, o))))
            .flatten();
        let succ = org_pick.or(kin_pick);
        self.player_passed = true;
        let Some(succ) = succ else {
            self.log.push(format!("t{}: the work dies with them - no hands left to take it up.", tick));
            return;
        };
        // the purse and the standing pass; the influence is theirs to make
        let (res, standing) = self.world.get_component::<PlayerCharacter>(self.player)
            .map(|p| (p.resources, p.influence)).unwrap_or((0.0, 0.0));
        self.world.add_component(succ, PlayerCharacter {
            name: self.name_of(succ), influence: standing * 0.5,
            authority: 0.0, resources: res, standing,
        });
        let name = self.name_of(succ);
        self.player = succ;
        self.log.push(format!("t{}: the work goes on - {} takes up the organizing.", tick, name));
        self.say(succ, "The work goes on. Someone has to.", CHAT_BUBBLE_SECS * 2);
    }

    /// The disclosure: a truth sits unsaid until it ripens, then it
    /// comes out - and the power it threatens answers. A strong org
    /// shields the discoverer; a weak one leaves him to the ladder.
    pub fn discovery_tick(&mut self, tick: u64) {
        if tick % 30 != 20 || self.disclosure_made { return; }
        if let Some((who, found, day)) = self.pending_discovery {
            // it ripens a fortnight, then comes out in a whisper
            if tick >= day + 14 {
                self.pending_discovery = None;
                self.disclosure_made = true;
                let name = self.name_of(who);
                self.log.push(format!("t{}: {} whispers it about - {}. The word's out now.", tick, name, found));
                if self.lord_esteem > 0.4 {
                    // an org worth the name stands with its own
                    self.log.push(format!("t{}: the village stands with {} - the truth was theirs to say.", tick, name));
                    self.say(who, "I said what I saw. That's all I did.", CHAT_BUBBLE_SECS * 2);
                } else {
                    // no one to stand with him: the threatened move - and
                    // the ladder's there for a man who talks treason
                    for _ in 0..3 {
                        desires::add_grievance(&mut self.world, who, desires::Problem::Blamed, tick + 90);
                    }
                    self.say(who, "I only said what I saw.", CHAT_BUBBLE_SECS * 2);
                }
            }
            return;
        }
        // what there is to find: the clerk's tallies, the old cures
        let found = if let Some(t) = self.translator {
            Some((t, "the lord's fifth is really a quarter"))
        } else if self.cured >= 3 {
            self.herbologist.map(|h| (h, "the old cures work, doctrine or no"))
        } else { None };
        if let Some((who, found)) = found {
            self.pending_discovery = Some((who, found, tick));
        }
    }

    /// People power: it builds as the org spends it and atrophies when
    /// idle - and the lord's estimate lags behind the truth, for he only
    /// learns what the street shows him
    pub fn power_tick(&mut self, tick: u64) {
        if tick % 7 != 3 { return; }
        let capacity = {
            if let Some(p) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                crate::sim::organized_power::step_idle(p,
                    &crate::sim::organized_power::PowerParams { atrophy_rate: 0.02, ..Default::default() });
                p.capacity
            } else { 0.0 }
        };
        // the lord's estimate creeps toward what's shown - perceived, not raw
        self.lord_esteem += (capacity - self.lord_esteem) * 0.08;
    }

    /// An org action in the street: power builds as it's spent
    pub fn record_power(&mut self, q: crate::sim::organized_power::ActionQuality) {
        let org = self.org;
        crate::sim::organized_power::record_action(&mut self.world, org, q);
    }

    /// The year's feast days: one mid-season each quarter, the village
    /// gathers at the market, the good stock comes out, and for a day
    /// the wants that never quite get fed are fed
    pub fn festival_tick(&mut self, tick: u64) {
        if tick % 90 != 45 { return; }
        const FEASTS: [&str; 4] = ["St. Swithin's day", "the Harvest Home", "Candlemas", "St. Martin's day"];
        let feast = FEASTS[(tick / 90 % 4) as usize];
        self.log.push(format!("t{}: {} - the village gathers at the market.", tick, feast));
        // the good stock comes out: bread, honey, whatever the stalls held
        let mut loaves = 0u32;
        for &be in &self.building_entities.clone() {
            if let Some(b) = self.world.get_component_mut::<Building>(be) {
                loaves += household::take_stock(b, "bread", 3);
                loaves += household::take_stock(b, "honey", 2);
            }
        }
        // and the barrel's tapped: whatever the village pours, the feast pours
        let mut poured = 0u32;
        for &be in &self.building_entities.clone() {
            if let Some(b) = self.world.get_component_mut::<Building>(be) {
                for d in ["ale", "beer", "wine", "spirits"] {
                    poured += household::take_stock(b, d, 2);
                }
            }
        }
        let gathered: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && self.world.get_component::<TownRole>(v).is_none()
                // the village's feast: the country keeps its own
                && self.site_of(v).is_none()
                && !self.shunned(v))
            .collect();
        // everyone comes to market for the day, in a ring round the cross
        let store = self.market_food;
        for &v in &gathered { self.travels.retain(|t| t.who != v); }
        self.gather_feast(tick, &gathered);
        // and somebody says so out loud for whoever's within earshot
        let voice = self.tavern.and_then(|t| self.world.get_component::<Building>(t))
            .and_then(|b| b.worker)
            .filter(|w| gathered.contains(w))
            .or_else(|| gathered.first().copied());
        if let Some(v) = voice {
            self.say(v, &format!("{} - the good stock's out!", feast), TALK_BUBBLE_SECS * 2);
        }
        for &v in &gathered {
            // the day feeds the wants that never quite get fed
            desires::satisfy(&mut self.world, v, Want::Joy, tick + 90);
            desires::satisfy(&mut self.world, v, Want::Music, tick + 90);
            if loaves > 0 {
                desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 30);
                if let Some(h) = self.world.get_component_mut::<Hunger>(v) { h.value = (h.value + 25.0).min(100.0); }
            }
            if poured > 0 {
                desires::satisfy(&mut self.world, v, Want::Drink, tick + 30);
            }
        }
        // bonds grow between the co-attendees: a pair or two each
        for (i, &v) in gathered.iter().enumerate() {
            for &o in gathered.iter().skip(i + 1).take(2) {
                relationships::record_interaction(&mut self.world, v, relationships::Interaction {
                    other: o, kind: relationships::InteractionKind::Conversation, intensity: 1.5 });
            }
        }
        // a few toasts surface what folk carry - the truth comes out in drink
        for &v in gathered.iter().take(4) {
            let line = self.world.get_component::<InnerLife>(v)
                .and_then(|l| l.grievances.first().map(|(p, _)| *p))
                .map(|p| match p {
                    desires::Problem::Blamed => "To my bread - whatever they say of it!",
                    desires::Problem::Taxed | desires::Problem::DeathDuty => "To what's left after the fifth!",
                    desires::Problem::Broke => "To better days!",
                    desires::Problem::Smelly => "To the fold - it smells of honest work!",
                    _ => "To the village!",
                }).unwrap_or("To the harvest!");
            self.say(v, line, CHAT_BUBBLE_SECS * 2);
        }
        // the tax collector is never invited but always shows
        if let Some(c) = self.collector {
            if !villager::is_dead(&self.world, c) {
                self.travels.retain(|t| t.who != c);
                self.send(c, Goal::HeadHome(store));
                self.say(c, "Uninvited. As ever.", CHAT_BUBBLE_SECS * 2);
            }
        }
    }
}
