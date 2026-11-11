//! The village's own law: the daily count of friction that becomes
//! bye-laws, the weekly life of partnerships and sureties, and the
//! community's pull over the generations. Community justice, beside the
//! lord's - the fines are smaller, the ladder is the same.

use crate::session::*;
use crate::game::community_law::*;

/// A villager still about and answerable
fn live(g: &Game, v: Entity) -> bool {
    !villager::is_dead(&g.world, v) && !crate::game::dungeon::unavailable(&g.world, v)
}

/// The live grievances a villager carries today
fn live_problems(g: &Game, v: Entity, tick: u64) -> Vec<desires::Problem> {
    g.world.get_component::<InnerLife>(v)
        .map(|l| l.grievances.iter().filter(|(_, u)| tick < *u)
            .map(|(p, _)| *p).collect()).unwrap_or_default()
}

/// The bond between two souls (none remembered is none)
fn bond_between(g: &Game, a: Entity, b: Entity) -> f32 {
    relationships::find_relationship(&g.world, a, b)
        .and_then(|r| g.world.get_component::<relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0)
}

/// A name for the log
fn named(g: &Game, v: Entity) -> &'static str {
    g.villagers.iter().position(|x| *x == v).map(|i| vill_name(i)).unwrap_or("someone")
}

impl Game {
    /// Each fresh grievance the neighbors could make a rule about is an
    /// incident: three of a kind and the village agrees a bye-law; once
    /// the rule stands, a fresh incident is a violation and draws a fine
    pub(in crate::session) fn bylaw_tick(&mut self, tick: u64) {
        // diff today's live grievances against what we've counted - each
        // new (who, kind, expiry) triple is a fresh incident
        let mut current: crate::engine::det::HashSet<(Entity, ByeKind, u64)> = Default::default();
        let mut incidents: Vec<(Entity, ByeKind)> = Vec::new();
        for &v in self.villagers.iter() {
            let Some(life) = self.world.get_component::<InnerLife>(v) else { continue };
            for (p, until) in life.grievances.iter() {
                if tick >= *until { continue; }
                let Some(kind) = bye_kind(*p) else { continue };
                current.insert((v, kind, *until));
                if !self.bylaw_seen.contains(&(v, kind, *until)) { incidents.push((v, kind)); }
            }
        }
        self.bylaw_seen = current;
        for (v, kind) in incidents {
            let standing = self.bylaws.iter().position(|l| l.kind == kind);
            if let Some(i) = standing {
                self.bylaws[i].violations += 1;
                self.bylaw_penalty(v, kind, tick);
            } else {
                *self.bylaw_friction.entry(kind).or_insert(0) += 1;
                if self.bylaw_friction.get(&kind).copied().unwrap_or(0) >= ADOPT_AT {
                    self.bylaws.push(ByeLaw { kind, passed_on: tick, violations: 0 });
                    self.bylaw_friction.insert(kind, 0);
                    self.log.push(format!("t{}: the village agrees a rule - {}.", tick, bye_name(kind)));
                    // agreeing a rule is the community finding its voice
                    self.community_nudge(0.05);
                    // a village writing its own rules is a writ that
                    // reaches less far (#971)
                    self.lord_strength = (self.lord_strength - 0.02).max(0.0);
                }
            }
        }
    }

    /// A broken rule: a fine off the purse, and the neighbors' word
    /// behind it - blame that feeds the same ladder the grievances do.
    /// A purse too thin for the fine asks a surety of kin
    fn bylaw_penalty(&mut self, v: Entity, kind: ByeKind, tick: u64) {
        let name = named(self, v);
        let purse = self.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
        let fine = (purse * BYE_FINE_SHARE).max(BYE_FINE_MIN);
        if purse >= fine {
            if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= fine; }
            self.log_entity(v, format!("t{}: {} is fined for breaking the rule - {}.", tick, name, bye_name(kind)));
        } else {
            // what's owed past the purse is a debt; kin and near
            // neighbors can stand surety for it
            let shortfall = fine - purse;
            if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold = 0.0; }
            if let Some(voucher) = self.find_voucher(v, shortfall) {
                let vname = named(self, voucher);
                let bond = shortfall * BOND_SHARE;
                if let Some(p) = self.world.get_component_mut::<Purse>(voucher) { p.gold -= shortfall + bond; }
                self.vouches.push(Vouch { voucher, debtor: v, bond, owed: shortfall, made_on: tick });
                self.log.push(format!("t{}: {} stands surety for {} - the fine's paid, and gold's put up against the word.", tick, vname, name));
            } else {
                self.log_entity(v, format!("t{}: {} can't pay the rule's fine - the village remembers it.", tick, name));
            }
        }
        // the neighbors' word feeds the same ladder - but the penalty's
        // own blame isn't itself a fresh incident of the rule broken
        desires::add_grievance(&mut self.world, v, desires::Problem::Blamed, tick + 60);
        self.bylaw_seen.insert((v, ByeKind::Conduct, tick + 60));
        // a fresh offense while a surety stands is a default: the bond
        // pays the debt, or the voucher answers for the word - a surety
        // just now made for this fine doesn't default on the spot
        if let Some(i) = self.vouches.iter().position(|u| u.debtor == v && u.made_on < tick) {
            let u = self.vouches.remove(i);
            let short = u.owed - u.bond;
            if short > 0.001 {
                let vname = named(self, u.voucher);
                self.log.push(format!("t{}: the surety fails - {}'s bond's spent and they answer for the word.", tick, vname));
                self.apply_sanction(u.voucher, Sanction::Gossip, tick);
            } else {
                self.log.push(format!("t{}: {} breaks the rule again - the surety's bond pays the debt.", tick, name));
            }
            self.community_nudge(-0.03);
        }
    }

    /// Kin and near neighbors with the purse and the standing to vouch
    fn find_voucher(&self, debtor: Entity, need: f32) -> Option<Entity> {
        if self.community_strength < VOUCH_MIN_STRENGTH { return None; }
        if self.vouches.iter().any(|u| u.debtor == debtor) { return None; }
        relationships::bonds_of(&self.world, debtor).into_iter()
            .filter_map(|re| self.world.get_component::<relationships::Relationship>(re)
                .map(|r| if r.from == debtor { r.other } else { r.from }))
            .filter(|&b| b != debtor && live(self, b))
            .find(|&b| bond_between(self, debtor, b) >= VOUCH_MIN_BOND
                && self.world.get_component::<Purse>(b)
                    .map(|p| p.gold >= need * (1.0 + BOND_SHARE)).unwrap_or(false))
    }

    /// Partnerships wax and wane with the bond between their members:
    /// sworn on a strong bond over a common grievance, prime while it
    /// holds, and a sour break leaves rancor the priest settles on the
    /// holy day
    pub(in crate::session) fn partnership_tick(&mut self, tick: u64) {
        if routine::is_holy(tick) { self.arbitrate_disputes(tick); }
        if tick % 7 != 2 { return; }
        // sworn on a strong bond over a shared wrong, and each soul in
        // one compact only
        'outer: for i in 0..self.villagers.len() {
            if self.partnerships.iter().filter(|p| p.stage != PartnerStage::Dissolved).count() >= PARTNER_CAP { break; }
            let a = self.villagers[i];
            if !live(self, a) || self.partnered(a) { continue; }
            for j in (i + 1)..self.villagers.len() {
                let b = self.villagers[j];
                if !live(self, b) || self.partnered(b) { continue; }
                if bond_between(self, a, b) < PARTNER_MIN_BOND { continue; }
                // the common objective: a wrong they both carry
                let pa = live_problems(self, a, tick);
                if !pa.iter().any(|p| live_problems(self, b, tick).contains(p)) { continue; }
                self.partnerships.push(Partnership { members: vec![a, b],
                    formed_on: tick, strength: 0.5, stage: PartnerStage::Forming,
                    rancor: 0.0, settled: false });
                self.log.push(format!("t{}: {} and {} swear to shared work.", tick, named(self, a), named(self, b)));
                self.community_nudge(0.03);
                continue 'outer;
            }
        }
        // the compact tracks the bond: hold it and it primes, let it
        // sour and the shared work falls away
        for pi in 0..self.partnerships.len() {
            if self.partnerships[pi].stage == PartnerStage::Dissolved { continue; }
            let (a, b) = (self.partnerships[pi].members[0], self.partnerships[pi].members[1]);
            let bond = bond_between(self, a, b);
            let (an, bn) = (named(self, a), named(self, b));
            let p = &mut self.partnerships[pi];
            p.strength = (p.strength + (bond - 0.4) * 0.4).clamp(0.0, 1.0);
            if p.strength >= PEAK_AT && p.stage == PartnerStage::Forming { p.stage = PartnerStage::Peak; }
            if p.strength < 0.35 { p.stage = PartnerStage::Decline; }
            if p.strength <= DISSOLVE_AT {
                p.stage = PartnerStage::Dissolved;
                p.rancor = if bond < 0.0 { (-bond).min(1.0) } else { 0.0 };
                p.settled = p.rancor <= 0.0;
                self.log.push(format!("t{}: the partnership of {} and {} breaks{}.", tick, an, bn,
                    if p.rancor > 0.0 { " - hard words remain" } else { " clean" }));
                self.community_nudge(if self.partnerships[pi].rancor > 0.0 { -0.04 } else { -0.01 });
            }
        }
    }

    /// Already bound to a live compact
    fn partnered(&self, v: Entity) -> bool {
        self.partnerships.iter()
            .any(|p| p.stage != PartnerStage::Dissolved && p.members.contains(&v))
    }

    /// The parish settles what sour partings leave - the priest's word
    /// on the holy day, milder than a feud's arbitration
    fn arbitrate_disputes(&mut self, tick: u64) {
        let priest = self.villagers.iter().copied().find(|&v|
            live(self, v)
                && self.site_of(v).is_none()
                && self.world.get_component::<Craft>(v)
                    .map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
        if priest.is_none() { return; }
        let Some(pi) = (0..self.partnerships.len())
            .filter(|&pi| self.partnerships[pi].stage == PartnerStage::Dissolved
                && !self.partnerships[pi].settled)
            .max_by(|&a, &b| self.partnerships[a].rancor
                .partial_cmp(&self.partnerships[b].rancor).unwrap_or(std::cmp::Ordering::Equal))
        else { return };
        self.partnerships[pi].rancor -= 0.25;
        if self.partnerships[pi].rancor <= 0.0 {
            self.partnerships[pi].settled = true;
            let (a, b) = (self.partnerships[pi].members[0], self.partnerships[pi].members[1]);
            self.log.push(format!("t{}: the priest makes peace between {} and {}.", tick, named(self, a), named(self, b)));
            relationships::record_interaction(&mut self.world, a,
                relationships::Interaction { other: b,
                    kind: relationships::InteractionKind::Conversation, intensity: 0.3 });
            self.community_nudge(0.02);
        }
    }

    /// The debtor pays their voucher back a share a week; made good and
    /// the bond comes home
    pub(in crate::session) fn pledge_tick(&mut self, tick: u64) {
        if tick % 7 != 3 { return; }
        let mut done: Vec<usize> = Vec::new();
        for i in 0..self.vouches.len() {
            let (voucher, debtor, bond) = { let u = &self.vouches[i];
                (u.voucher, u.debtor, u.bond) };
            let pay = (self.vouches[i].owed * REPAY_SHARE).min(REPAY_CAP);
            let have = self.world.get_component::<Purse>(debtor).map(|p| p.gold).unwrap_or(0.0);
            let pay = pay.min(have);
            if pay > 0.001 {
                if let Some(p) = self.world.get_component_mut::<Purse>(debtor) { p.gold -= pay; }
                if let Some(p) = self.world.get_component_mut::<Purse>(voucher) { p.gold += pay; }
                self.vouches[i].owed -= pay;
            }
            if self.vouches[i].owed <= 0.001 {
                if let Some(p) = self.world.get_component_mut::<Purse>(voucher) { p.gold += bond; }
                self.log.push(format!("t{}: {} has made good - {}'s bond comes home.", tick, named(self, debtor), named(self, voucher)));
                done.push(i);
            }
        }
        for i in done.into_iter().rev() { self.vouches.remove(i); self.community_nudge(0.02); }
    }

    /// The community's pull drifts toward ordinary between its doings;
    /// the log tells the band's turns once each
    pub(in crate::session) fn community_tick(&mut self, tick: u64) {
        if tick % 7 != 6 { return; }
        self.community_strength = (self.community_strength
            + (COMMUNITY_BASELINE - self.community_strength) * COMMUNITY_DECAY)
            .clamp(0.0, 1.0);
        let band = if self.community_strength < COMMUNITY_LOW { 0 }
            else if self.community_strength < COMMUNITY_HIGH { 1 } else { 2 };
        if band != self.community_band {
            self.community_band = band;
            self.log.push(format!("t{}: {}", tick, match band {
                0 => "the village is only houses - no pull between them.",
                1 => "the village pulls together.",
                _ => "the village speaks with one voice.",
            }));
        }
    }

    /// What its doings do to the community's pull
    fn community_nudge(&mut self, d: f32) {
        self.community_strength = (self.community_strength + d).clamp(0.0, 1.0);
    }
}
