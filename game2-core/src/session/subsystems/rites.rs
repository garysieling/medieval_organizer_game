//! Rites of passage: the coming-of-age initiation at the church or the
//! stones, and the weddings that turn the heart-helper's lovers into
//! family under one roof

use crate::session::*;

/// A courtship runs this long before the couple is wed
const COURTSHIP_DAYS: u64 = 30;
/// And a few more days, staggered by the pair, so the village doesn't
/// hold every wedding on the same morning
const WEDDING_STAGGER: u64 = 10;
/// A rite or wedding that can't gather in this many days gives way
const RITE_PATIENCE: u64 = 12;
/// How close to the site tile folk must stand for the ceremony to hold
const RITE_REACH: f32 = 3.0;

impl Game {
    /// The day's rites: finish the ceremonies that gathered, keep the
    /// walkers walking, and wed the couples whose time has come
    pub(in crate::session) fn rites_tick(&mut self, tick: u64) {
        self.forgotten_parties(tick);
        self.rite_walks(tick);
        self.wedding_walks(tick);
        self.betroth_couples(tick);
    }

    /// Where a rite of this faith gathers near `from`: the church door
    /// for the parish, the stones or the grove for the old ways
    fn rite_site(&self, old: bool, from: (i32, i32)) -> Option<(i32, i32)> {
        if !old {
            let church = self.building_entities.iter().copied().find(|&b|
                self.world.get_component::<Building>(b)
                    .map(|bb| bb.kind == BuildingKind::Church).unwrap_or(false))?;
            let cp = self.world.get_component::<Pos>(church)?;
            let door = self.door_of(church)
                .unwrap_or((cp.x.round() as i32, cp.y.round() as i32 + 1));
            return self.at_the_door(door.0, door.1);
        }
        let dist = |t: &(i32, i32)| (t.0 - from.0).abs() + (t.1 - from.1).abs();
        if let Some(s) = self.stones.iter().copied().filter(|s| dist(s) <= 24)
            .min_by_key(dist) {
            return self.at_the_door(s.0, s.1);
        }
        self.map.trees.iter().copied().filter(|t| self.map.passable(t.0, t.1))
            .min_by_key(dist).filter(|t| dist(t) <= 24)
            .and_then(|t| self.at_the_door(t.0, t.1))
    }

    /// Who stands for the faith at a rite: the priest at the church, an
    /// elder of the old ways at the stones - any grown hand will do in
    /// a pinch, nearest the site first so the walk stays short
    fn rite_officiant(&self, exclude: &[Entity], old: bool, site: (i32, i32)) -> Option<Entity> {
        let mut folk: Vec<Entity> = self.villagers.iter().copied().filter(|&o| {
            !exclude.contains(&o)
                && self.site_of(o).is_none()
                && !villager::is_dead(&self.world, o)
                && !crate::game::dungeon::unavailable(&self.world, o)
                && self.world.get_component::<Craft>(o).is_some()
        }).collect();
        folk.sort_by_key(|&o| self.tile_of(o)
            .map(|t| (t.0 - site.0).abs() + (t.1 - site.1).abs()).unwrap_or(i32::MAX));
        if old {
            let pious = |o: Entity| routine::has_trait(&self.world, o,
                crate::game::traits::Trait::Pious);
            if let Some(e) = folk.iter().copied().find(|&o|
                routine::keeps_old_ways(&self.world, o) && pious(o)) { return Some(e); }
            if let Some(e) = folk.iter().copied().find(|&o|
                routine::keeps_old_ways(&self.world, o)) { return Some(e); }
        } else if let Some(e) = folk.iter().copied().find(|&o|
            self.world.get_component::<Craft>(o)
                .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)) {
            return Some(e);
        }
        folk.first().copied()
    }

    /// Send `who` walking to the rite tile unless they're already on a walk
    fn send_attend(&mut self, who: Entity, site: (i32, i32)) {
        if self.travels.iter().any(|t| t.who == who) { return; }
        let _ = self.send_tile_walk(who, site, Goal::AttendRite, 48);
    }

    /// The new adult's walk to their rite; false when the village can't
    /// hold one today and they take the ordinary coming-of-age instead
    pub(in crate::session) fn begin_rite(&mut self, tick: u64, v: Entity) -> bool {
        let Some(from) = self.tile_of(v) else { return false };
        let old = routine::keeps_old_ways(&self.world, v);
        let Some(site) = self.rite_site(old, from) else { return false };
        let Some(by) = self.rite_officiant(&[v], old, site) else { return false };
        self.world.add_component(v, villager::PendingRite { site, by, old, since: tick });
        self.send_attend(v, site);
        self.send_attend(by, site);
        true
    }

    /// How close `e` stands to the rite tile
    fn at_site(&self, e: Entity, site: (i32, i32), reach: f32) -> bool {
        self.world.get_component::<Pos>(e).map(|p|
            (p.x - site.0 as f32).abs() + (p.y - site.1 as f32).abs() <= reach)
            .unwrap_or(false)
    }

    /// The officiant is there, or too gone to matter - a dead priest
    /// or one behind bars doesn't hold the rite hostage
    fn officiant_gone(&self, by: Entity) -> bool {
        villager::is_dead(&self.world, by) || !self.villagers.contains(&by)
            || crate::game::dungeon::unavailable(&self.world, by)
    }

    /// Rites owed a household party: the coming-of-age celebration the
    /// rite came before
    fn forgotten_parties(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            if self.world.get_component::<villager::DueParty>(v).is_none() { continue; }
            self.world.remove_component::<villager::DueParty>(v);
            if villager::is_dead(&self.world, v) { continue; }
            self.celebrate_person(tick, v, &format!("{} comes of age", self.name_of(v)));
        }
    }

    /// Pending initiations: the gathered rite is held, the stragglers
    /// are walked again, and the ones that never gather give way
    fn rite_walks(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            let Some(p) = self.world.get_component::<villager::PendingRite>(v).copied()
                else { continue };
            if villager::is_dead(&self.world, v) || self.site_of(v).is_some() {
                self.world.remove_component::<villager::PendingRite>(v);
                continue;
            }
            let by_near = self.officiant_gone(p.by) || self.at_site(p.by, p.site, RITE_REACH + 1.0);
            if self.at_site(v, p.site, RITE_REACH) && by_near {
                self.complete_rite(tick, v, p);
            } else if tick - p.since > RITE_PATIENCE {
                self.world.remove_component::<villager::PendingRite>(v);
                self.world.add_component(v, villager::Initiated);
                self.world.add_component(v, villager::DueParty);
            } else {
                self.send_attend(v, p.site);
                if !self.officiant_gone(p.by) { self.send_attend(p.by, p.site); }
            }
        }
    }

    /// The gathered faithful make the initiate - the parish's Christian
    /// or the old stones' own - and the household party is booked for
    /// tomorrow
    fn complete_rite(&mut self, tick: u64, v: Entity, p: villager::PendingRite) {
        self.world.remove_component::<villager::PendingRite>(v);
        self.world.add_component(v, villager::Initiated);
        self.world.add_component(v, villager::DueParty);
        let name = self.name_of(v);
        let at = (p.site.0 as f32, p.site.1 as f32);
        let why = if p.old { format!("{}'s rite of the old stones", name) }
            else { format!("{}'s rite of initiation", name) };
        self.local_celebration(tick, at, Some(v), &why);
        if !self.officiant_gone(p.by) {
            self.say(p.by, if p.old { "The stones receive you." } else { "The parish receives you." },
                TALK_BUBBLE_SECS * 2);
        }
        self.log_entity(v, format!("t{}: {} is {}.", tick, name,
            if p.old { "sworn to the old stones" } else { "made a Christian of the parish" }));
    }

    /// Betrothed couples: the gathered wedding is held, the stragglers
    /// walked again, and the ones that never gather come apart
    fn wedding_walks(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            let Some(b) = self.world.get_component::<villager::Betrothed>(v).copied()
                else { continue };
            let p = b.partner;
            if villager::is_dead(&self.world, v) || villager::is_dead(&self.world, p)
                || self.site_of(v).is_some() || self.site_of(p).is_some() {
                self.unbetroth(v, p);
                continue;
            }
            // a partner behind bars waits the wedding out - the betrothal
            // stands, the walks wait for their release
            if crate::game::dungeon::unavailable(&self.world, v)
                || crate::game::dungeon::unavailable(&self.world, p) { continue; }
            let by_near = self.officiant_gone(b.by) || self.at_site(b.by, b.site, RITE_REACH + 1.0);
            if self.at_site(v, b.site, RITE_REACH) && self.at_site(p, b.site, RITE_REACH) && by_near {
                self.complete_wedding(tick, v, p, b);
            } else if tick - b.since > RITE_PATIENCE {
                self.unbetroth(v, p);
                // and the courtship is spent - they don't wed on this record
                let spent = [(v, p), (p, v)];
                self.courtships.retain(|&(a, bb, _)| !spent.contains(&(a, bb)));
            } else {
                self.send_attend(v, b.site);
                self.send_attend(p, b.site);
                if !self.officiant_gone(b.by) { self.send_attend(b.by, b.site); }
            }
        }
    }

    fn unbetroth(&mut self, a: Entity, b: Entity) {
        self.world.remove_component::<villager::Betrothed>(a);
        self.world.remove_component::<villager::Betrothed>(b);
    }

    /// The gathered faithful wed the couple - lovers become family, one
    /// roof takes them both, and the village throws the day
    fn complete_wedding(&mut self, tick: u64, a: Entity, b: Entity, wed: villager::Betrothed) {
        self.unbetroth(a, b);
        let spent = [(a, b), (b, a)];
        self.courtships.retain(|&(x, y, _)| !spent.contains(&(x, y)));
        // the bond is Family now, both ways round
        for e in self.world.query::<relationships::Relationship>().into_iter().collect::<Vec<_>>() {
            let Some(r) = self.world.get_component::<relationships::Relationship>(e) else { continue };
            if r.kind == relationships::RelKind::Lover
                && ((r.from == a && r.other == b) || (r.from == b && r.other == a)) {
                if let Some(rm) = self.world.get_component_mut::<relationships::Relationship>(e) {
                    rm.kind = relationships::RelKind::Family;
                    rm.strength = rm.strength.max(0.8);
                }
            }
        }
        self.merge_households(a, b);
        let (an, bn) = (self.name_of(a), self.name_of(b));
        let at = (wed.site.0 as f32, wed.site.1 as f32);
        let why = if wed.old { format!("{} and {}'s handfast", an, bn) }
            else { format!("{} and {}'s wedding", an, bn) };
        self.local_celebration(tick, at, Some(a), &why);
        if !self.officiant_gone(wed.by) {
            self.say(wed.by, if wed.old { "Handfast under the old sky." } else { "Man and wife." },
                TALK_BUBBLE_SECS * 2);
        }
        self.log_entity(a, format!("t{}: {} and {} are wed.", tick, an, bn));
        self.toasts.push_at((format!("{} and {} are wed!", an, bn), tick + 30), Some(at));
        for e in [a, b] {
            desires::satisfy(&mut self.world, e, Want::Joy, tick + 60);
        }
    }

    /// One roof for the wed: the partner without a bed moves into the
    /// other's house - the roomier home wins, and the full-house birth
    /// roll does the rest
    fn merge_households(&mut self, a: Entity, b: Entity) {
        let ha = household::home_of(&self.world, a);
        let hb = household::home_of(&self.world, b);
        if ha == hb { return; }
        let home_kind = |h: Option<Entity>| h.filter(|&e| self.world.get_component::<Building>(e)
            .map(|bb| bb.kind == BuildingKind::Home).unwrap_or(false));
        let beds = |h: Entity| self.world.get_component::<Building>(h)
            .map(|bb| bb.occupants.len()).unwrap_or(0);
        let (home, mover, old_home) = match (home_kind(ha), home_kind(hb)) {
            (Some(x), Some(y)) => if beds(x) <= beds(y) { (x, b, hb) } else { (y, a, ha) },
            (Some(x), None) => (x, b, hb),
            (None, Some(y)) => (y, a, ha),
            (None, None) => return,
        };
        if let Some(oh) = old_home {
            if let Some(bb) = self.world.get_component_mut::<Building>(oh) {
                bb.occupants.retain(|&o| o != mover);
            }
        }
        if let Some(bb) = self.world.get_component_mut::<Building>(home) {
            bb.occupants.push(mover);
        }
    }

    /// Courtships whose time has come: both partners are called to the
    /// church, or to the stones when the old ways keep them both
    fn betroth_couples(&mut self, tick: u64) {
        // spent records first: the dead, the gone, and the already-wed
        let spent: Vec<(Entity, Entity)> = self.courtships.iter().copied()
            .filter(|&(a, b, _)| villager::is_dead(&self.world, a)
                || villager::is_dead(&self.world, b)
                || self.site_of(a).is_some() || self.site_of(b).is_some()
                || self.rel_kind(a, b) != Some(relationships::RelKind::Lover))
            .map(|(a, b, _)| (a, b)).collect();
        if !spent.is_empty() {
            self.courtships.retain(|&(a, b, _)| !spent.contains(&(a, b)));
        }
        for i in 0..self.courtships.len() {
            let (a, b, since) = self.courtships[i];
            // the pair's own stagger keeps the weddings spread over days
            let stagger = (routine::mix(a, 7) ^ routine::mix(b, 13)) % WEDDING_STAGGER;
            if tick < since + COURTSHIP_DAYS + stagger { continue; }
            // a heart behind bars can't stand at the altar today - the
            // courtship keeps and the call waits for their release
            if crate::game::dungeon::unavailable(&self.world, a)
                || crate::game::dungeon::unavailable(&self.world, b) { continue; }
            if self.world.get_component::<villager::Betrothed>(a).is_some()
                || self.world.get_component::<villager::Betrothed>(b).is_some() { continue; }
            let Some(from) = self.tile_of(a) else { continue };
            let old = routine::keeps_old_ways(&self.world, a)
                && routine::keeps_old_ways(&self.world, b);
            let Some(site) = self.rite_site(old, from) else { continue };
            let Some(by) = self.rite_officiant(&[a, b], old, site) else { continue };
            for (x, y) in [(a, b), (b, a)] {
                self.world.add_component(x, villager::Betrothed {
                    partner: y, site, by, old, since: tick });
                self.send_attend(x, site);
            }
            self.send_attend(by, site);
            // the matchmaker's own fee and the dowry's little move: the
            // broker takes a penny for the brokering, and a gift crosses
            // between the pair's households to seal the word
            let gift = self.world.get_component::<Purse>(a).map(|p| p.gold * 0.1).unwrap_or(0.0);
            if gift > 0.01 {
                if let Some(pa) = self.world.get_component_mut::<Purse>(a) { pa.gold -= gift; }
                if let Some(pb) = self.world.get_component_mut::<Purse>(b) { pb.gold += gift; }
            }
            let (an, bn) = (self.name_of(a), self.name_of(b));
            self.log_entity(a, format!("t{}: {} and {} are to be wed.", tick, an, bn));
        }
    }
}
