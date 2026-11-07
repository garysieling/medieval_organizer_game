//! The post: each town and castle keeps an office and a rider. A
//! village letter rides the village's own abstract post still; word
//! for the country goes with a courier - posted, picked up, walked,
//! and landed - and timing is the feature.

use crate::session::*;

/// The village stop on a courier's round: the market is the handoff
pub(in crate::session) const VILLAGE_STOP: usize = usize::MAX;

impl Game {
    /// The post's day: stamp fresh letters for the road, send the
    /// riders out on their rounds, and land what the road brings home
    pub(in crate::session) fn post_tick(&mut self, tick: u64) {
        // stamp fresh letters: the office that owes the delivery
        let ends: Vec<(Option<usize>, Option<usize>)> = self.letters.iter()
            .map(|l| (self.site_of(l.from), self.site_of(l.to))).collect();
        let stamps: Vec<Option<usize>> = self.letters.iter().zip(&ends)
            .map(|(l, &(fs, ts))| {
                if l.office.is_some() || l.courier.is_some() { return l.office; }
                if fs.is_none() && ts.is_none() { return None; }
                if fs == ts { return Some(usize::MAX - 1); } // marker: same green
                let site = ts.or(fs).map(|si| self.office_for(si)).unwrap_or(usize::MAX);
                if site == usize::MAX { None } else { Some(site) }
            }).collect();
        for (l, s) in self.letters.iter_mut().zip(stamps) {
            if l.office.is_some() || l.courier.is_some() || s.is_none() { continue; }
            let site = s.unwrap();
            if site == usize::MAX - 1 { l.due = tick + 1; continue; }
            l.office = Some(site);
            // a penny a page: the sender pays the post
            let pays = self.world.get_component_mut::<Purse>(l.from)
                .map(|p| if p.gold >= 1.0 { p.gold -= 1.0; true } else { false })
                .unwrap_or(false);
            if pays {
                if let Some(c) = self.couriers.iter().find(|c| c.office == site) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(c.e) { p.gold += 1.0; }
                }
            }
        }
        // and the riders go out on their days
        for i in 0..self.couriers.len() {
            if self.couriers[i].on_road || tick < self.couriers[i].depart { continue; }
            let office = self.couriers[i].office;
            let e = self.couriers[i].e;
            self.couriers[i].on_road = true;
            // pick up everything his office's post owes the road
            for l in &mut self.letters {
                if l.office == Some(office) && l.courier.is_none() { l.courier = Some(e); }
            }
            self.post_next_leg(i, tick);
        }
    }

    /// The courier reaches a stop: land what the bag owes it, gather
    /// what the office owes him back, and ride the next leg - or home
    pub(in crate::session) fn arrive_post(&mut self, who: Entity, stop: usize) {
        let tick = self.tick;
        let Some(i) = self.couriers.iter().position(|c| c.e == who) else { return };
        // land the letters this stop takes
        for j in 0..self.letters.len() {
            let l = self.letters[j];
            if l.courier != Some(who) { continue; }
            let there = if stop == VILLAGE_STOP { self.site_of(l.to).is_none() }
                else { self.site_of(l.to) == Some(stop) };
            if !there { continue; }
            if villager::is_dead(&self.world, l.to) {
                // the word comes back: they're gone - the letter turns
                // for the writer on the next leg
                self.letters[j] = Letter { from: l.to, to: l.from,
                    office: l.office, courier: Some(who), due: tick + 30 };
                self.log.push(format!("t{}: a letter comes back - {} is gone.", tick, self.name_of(l.to)));
            } else {
                self.deliver_letter(l, tick);
                self.letters.remove(j);
            }
        }
        // gather the village's handoff: letters his office owes, posted
        // to ride out
        let office = self.couriers[i].office;
        for l in &mut self.letters {
            if l.office == Some(office) && l.courier.is_none() { l.courier = Some(who); }
        }
        self.post_next_leg(i, tick);
    }

    /// The rider's next stop on the round: the village first, the other
    /// settlements after, and home last
    fn post_next_leg(&mut self, i: usize, _tick: u64) {
        let c = &mut self.couriers[i];
        let office = c.office;
        let e = c.e;
        // the round: the village market, then every settlement with an
        // office but his own, then home
        let mut stops: Vec<usize> = vec![VILLAGE_STOP];
        for &(si, _) in &self.post_offices {
            if si != office { stops.push(si); }
        }
        stops.push(office);
        if c.leg >= stops.len() {
            c.leg = 0;
            c.on_road = false;
            c.depart = self.tick + 4 + (office as u64 * 3) % 5;
            return;
        }
        let stop = stops[c.leg];
        c.leg += 1;
        let at = if stop == VILLAGE_STOP {
            self.world.get_component::<Pos>(self.market_food)
                .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or(self.village)
        } else {
            self.sites[stop].centre
        };
        self.send_far(e, at, Goal::Post(stop));
    }

    /// The word lands: the priest reads for the illiterate, the bond
    /// warms, and the day gains a little joy - the same welcome the
    /// village's own post gives
    pub(in crate::session) fn deliver_letter(&mut self, l: Letter, tick: u64) {
        let Letter { from, to, .. } = l;
        // the illiterate pay the priest a penny to hear it read out
        let literate = self.literate(to);
        if !literate {
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

    /// The office that owes a site its post: its own if it keeps one,
    /// the nearest office else
    fn office_for(&self, site: usize) -> usize {
        if self.post_offices.iter().any(|&(si, _)| si == site) { return site; }
        let c = self.sites[site].centre;
        self.post_offices.iter()
            .map(|&(si, _)| { let d = self.sites[si].centre;
                ((d.0 - c.0).pow(2) + (d.1 - c.1).pow(2), si) })
            .min().map(|(_, si)| si).unwrap_or(usize::MAX)
    }
}
