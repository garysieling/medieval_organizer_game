//! Findings: hoards, springs, honey and truffles, good years for berries and mushrooms, and what the storm leaves.
//! Each fn here holds this family's arms of one handler in events/mod.rs.

use super::*;

impl Game {
    /// findings: the `event_day` arms for this group (the rest fall to the default)
    pub(super) fn findings_day(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::HerbFlush => {
                // ten days of gather: the hedge is thick with simples -
                // the day's pick goes to the herb hut's stock by its
                // true name, where the herb-wife can actually brew it
                if let Some(hut) = self.herb_hut {
                    let herbs = crate::game::herbology::medicinal_herbs();
                    if !herbs.is_empty() {
                        let roll = self.rare_roll(id);
                        let (name, _, _) = herbs[(roll as usize / 3) % herbs.len()];
                        if let Some(mut b) = self.world.get_component_mut::<household::Building>(hut) {
                            household::add_stock(&mut b, name, 2);
                        }
                    }
                }
            }
            EventId::BerryYear => {
                // a fortnight of loaded hedges: the kids strip them and
                // the baskets come to the market a little each day - the
                // tally rides in aux, hungry mouths take it same-day
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings += 1;
                    if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux += 1; }
                }
            }
            EventId::ProdigalSon => {
                // the tales are the talk of the street: a pair of
                // listeners a day get the lift of it, and they talk
                // it over between themselves
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                let roll = self.rare_roll(id);
                if !able.is_empty() {
                    let off = (roll as usize / 3) % able.len();
                    let mut pair: Vec<Entity> = vec![];
                    for i in 0..able.len().min(2) {
                        let v = able[(off + i) % able.len()];
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 10);
                        pair.push(v);
                    }
                    if pair.len() == 2 {
                        use crate::game::relationships::{Interaction, InteractionKind};
                        crate::game::relationships::record_interaction(&mut self.world, pair[0],
                            Interaction { other: pair[1], kind: InteractionKind::Conversation, intensity: 0.3 });
                    }
                }
            }
            _ => {}
        }
    }

    /// findings: the `event_began` arms for this group (the rest fall to the default)
    pub(super) fn findings_began(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::MushroomFlush => {
                // the wet fortnight's gift: every patch flushes at once,
                // and the wood's edges sprout a few besides - picked
                // through by the end (aux counts the bonus caps)
                for p in self.mush_patches.iter_mut() { p.next_ready = self.tick; }
                let mut added = 0u64;
                for (i, &(tx, ty)) in self.map.trees.iter().step_by(13).take(3).enumerate() {
                    if let Some(t) = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (tx + dx, ty + dy)))
                        .find(|&(x, y)| self.map.passable(x, y)
                            && !self.mush_patches.iter().any(|p| p.tile == (x, y))) {
                        let kind = if i % 2 == 0 { MushKind::Chanterelle } else { MushKind::FieldMushroom };
                        self.mush_patches.push(MushPatch { tile: t, kind, next_ready: self.tick });
                        added += 1;
                    }
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = added; }
                self.log.push(format!("t{}: a mushroom flush - the wood's full of caps after the rain.", self.tick));
                self.toasts.push(("Mushrooms!".into(), self.tick + 40));
            }
            EventId::HerbFlush => {
                self.log.push(format!("t{}: a flush of herbs - the hedge is thick with simples, worth the picking while it lasts.", self.tick));
                self.toasts.push(("A flush of herbs".into(), self.tick + 40));
            }
            EventId::BerryYear => {
                self.log.push(format!("t{}: a berry year - the hedges are black with them, and the kids' mouths are stained already.", self.tick));
                self.toasts.push(("A berry year".into(), self.tick + 40));
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// findings: the `event_moment` arms for this group (the rest fall to the default)
    pub(super) fn findings_moment(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::CoinHoard => {
                // the spade rings on a pot: a rolled finder's purse
                // fills, and the whole street asks whose it really is
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<crate::game::vocation::Kid>(v).is_none()).collect();
                let roll = self.rare_roll(id);
                if let Some(&v) = able.get((roll as usize / 5) % able.len().max(1)) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                        p.gold += 5.0;
                    }
                    self.say(v, "The spade rang on something... it's a pot. It's full of them!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a coin hoard - a pot of old silver turned up in the field, and whose it is nobody'll say loud.", self.tick));
                } else {
                    self.log.push(format!("t{}: a pot of old silver turned up - and walked off before anyone could claim it.", self.tick));
                }
                self.toasts.push(("A coin hoard".into(), self.tick + 40));
            }
            EventId::OldStones => {
                // the plow turns up someone's old work: stone for the
                // robbing, and a marvel to touch - who laid them, and
                // what were they for?
                let roll = self.rare_roll(id);
                for (i, &v) in self.villagers.clone().iter().enumerate() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, self.tick + 10);
                    // good stone's good stone: a share into satchels,
                    // where the masons can draw it - a shelf can't
                    if i as u64 % 2 == roll % 2 {
                        if let Some(s) = self.world.get_component_mut::<crate::game::livelihood::Satchel>(v) {
                            *s.goods.entry("stone".into()).or_insert(0) += 1;
                        }
                    }
                }
                self.log.push(format!("t{}: old stones - the plow turned up someone's old work, and who laid them nobody knows.", self.tick));
                self.toasts.push(("Old stones".into(), self.tick + 40));
            }
            EventId::SpringBubbles => {
                // new water: the pit's head rises where it's low - a
                // spring can't flood a full pit, only feed a dry one
                if self.land.pit_water < 0.5 {
                    self.land.pit_water = (self.land.pit_water + 0.1).min(0.5);
                }
                // a spring doesn't trim a bed that's already run high
                for g in self.gardens.iter_mut() { g.2 += 0.1; }
                self.log.push(format!("t{}: a spring bubbles up - cold sweet water out of the ground where none ran before.", self.tick));
                self.toasts.push(("A spring bubbles up".into(), self.tick + 40));
            }
            EventId::HoneyTree => {
                // a hive in a hollow trunk: the combs come back in
                // hands that can trade them - two satchels' worth,
                // and the first finder pays the stings
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                let roll = self.rare_roll(id);
                if !able.is_empty() {
                    let off = (roll as usize / 3) % able.len();
                    for i in 0..able.len().min(2) {
                        let v = able[(off + i) % able.len()];
                        if let Some(s) = self.world.get_component_mut::<crate::game::livelihood::Satchel>(v) {
                            // a satchel holds ten: the combs fill what
                            // room's left, they don't overflow it
                            let held: u32 = s.goods.values().sum();
                            let room = crate::game::livelihood::SATCHEL_CAP.saturating_sub(held);
                            if room > 0 {
                                *s.goods.entry("honey".into()).or_insert(0) += room.min(2);
                            }
                        }
                    }
                    if let Some(h) = self.world.get_component_mut::<Health>(able[off]) {
                        if h.condition > 0.05 {
                            h.condition = (h.condition - 0.03).max(0.05);
                        }
                    }
                    self.say(able[off], "Worth every sting - look at the comb on that!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a honey tree - a hive in a hollow trunk, and the combs came back dripping.", self.tick));
                } else {
                    self.log.push(format!("t{}: a honey tree hums in the wood - nobody ventures the combs.", self.tick));
                }
                self.toasts.push(("A honey tree".into(), self.tick + 40));
            }
            EventId::Truffle => {
                // the sow noses it out in the oak-leaves: worth real
                // coin - a rolled finder's purse feels it same day
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<crate::game::vocation::Kid>(v).is_none()).collect();
                let roll = self.rare_roll(id);
                if let Some(&v) = able.get((roll as usize / 5) % able.len().max(1)) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                        p.gold += 3.0;
                    }
                    self.say(v, "The sow nosed it out - the apothecary pays dear for these!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a truffle - the sow nosed it out in the oak-leaves, and the apothecary pays dear for it.", self.tick));
                } else {
                    self.log.push(format!("t{}: a sow nosed something out in the oak-leaves - gone before anyone saw what.", self.tick));
                }
                self.toasts.push(("A truffle".into(), self.tick + 40));
            }
            EventId::StormWashUp => {
                // the reach gives back what the storm took elsewhere:
                // driftwood and a barrel on the strand - into hands
                // that can use them, what room's left and no more
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                let roll = self.rare_roll(id);
                if !able.is_empty() {
                    // driftwood only - wood's the leavings hands can use
                    let off = (roll as usize / 3) % able.len();
                    for i in 0..able.len().min(3) {
                        let v = able[(off + i) % able.len()];
                        if let Some(s) = self.world.get_component_mut::<crate::game::livelihood::Satchel>(v) {
                            let held = s.goods.get("wood").copied().unwrap_or(0);
                            let room = crate::game::livelihood::SATCHEL_CAP.saturating_sub(held);
                            if room > 0 {
                                *s.goods.entry("wood".into()).or_insert(0) += 1;
                            }
                        }
                    }
                    self.say(able[off], "The reach gives back - driftwood and a barrel on the strand!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a storm's leavings - the reach gives back driftwood and a barrel, and the strand's picked clean by noon.", self.tick));
                } else {
                    self.log.push(format!("t{}: a storm's leavings on the strand - nobody to pick them.", self.tick));
                }
                self.toasts.push(("A storm's leavings".into(), self.tick + 40));
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// findings: the `event_end` arms for this group (the rest fall to the default)
    pub(super) fn findings_end(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::MushroomFlush => {
                // the flush's own caps are picked through: its bonus
                // patches (the last `aux` pushed) leave the wood for good,
                // and anyone mid-walk to one wanders home instead
                let added = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) as usize;
                if added > 0 {
                    let kept = self.mush_patches.len() - added;
                    self.mush_patches.truncate(kept);
                    for t in self.travels.iter_mut() {
                        if let Goal::ForageShroom(pi) = t.goal {
                            // the cap's gone - they stand where they are,
                            // they don't finish the dead walk to it
                            if pi >= kept { t.goal = Goal::Wander; t.path.clear(); }
                        }
                    }
                }
                self.log.push(format!("t{}: the flush is picked through - the shade keeps what it keeps.", tick));
            }
            _ => {}
        }
    }
}
