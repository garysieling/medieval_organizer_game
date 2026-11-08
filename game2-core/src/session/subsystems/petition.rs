//! The wives' petition: drink's toll gathers voices until the castle has
//! to answer. The migrant count, or the bonded women of the heavy
//! drinkers - whichever crosses first sends a demand to the lord, and
//! his esteem decides whether the tap gets throttled.

use crate::session::*;

/// How many strangers on the roll before the village calls it an
/// invasion - one in seven newcomers is the wives' line
const MIGRANT_SHARE: f32 = 0.15;
/// How many wives with a drinking husband speak before the petition
/// goes up - three voices make a cause, per the organizer's ledger
const WIVES_SPEAKING: usize = 3;
/// The petition's ride to the castle and back: a week for the lord's
/// answer
const ANSWER_DAYS: u64 = 7;
/// The lord's bar: the village he esteems gets its ordinance; the
/// village he doesn't, a refusal the wives remember
const LORD_BAR: f32 = 0.5;

impl Game {
    /// The petition's day: raise it when a trigger crosses, answer it
    /// when the ride comes back - the ordinance or the refusal
    pub(in crate::session) fn petition_tick(&mut self, tick: u64) {
        if self.petition.is_none() && self.ordinance.is_none() {
            let folk = self.villagers.len().max(1) as f32;
            let strangers = self.villagers.iter().copied()
                .filter(|&v| self.world.get_component::<villager::Migrant>(v).is_some())
                .count() as f32;
            // the wives' voice: a woman with a bond to a man whose want
            // is a cup - the drink's toll speaks through who it costs
            // a woman's man is a Relationship whose other end holds her;
            // his want for a cup is the drink's toll she'd name
            let drinking_partner = |w: Entity| self.world.query::<crate::game::relationships::Relationship>()
                .iter().copied().any(|r| {
                    let Some(rel) = self.world.get_component::<crate::game::relationships::Relationship>(r) else { return false };
                    let him = if rel.from == w { rel.other } else if rel.other == w { rel.from } else { return false };
                    self.world.get_component::<crate::game::desires::InnerLife>(him)
                        .map(|l| l.wants.iter().any(|w2| matches!(w2, crate::game::desires::Want::Drink)))
                        .unwrap_or(false)
                });
            let wives = self.villagers.iter().copied().filter(|&w| {
                self.world.get_component::<villager::Sex>(w)
                    .map(|s| *s == villager::Sex::Female).unwrap_or(false)
                && drinking_partner(w)
            }).count();
            if strangers / folk > MIGRANT_SHARE || wives >= WIVES_SPEAKING {
                self.petition = Some(Petition { started: tick, answer_on: tick + ANSWER_DAYS });
                self.log.push(format!("t{}: the wives' petition goes up to the castle - drink's toll on the village, and the strangers who bring it.", tick));
            }
        }
        let Some(p) = self.petition else { return };
        if tick < p.answer_on { return; }
        self.petition = None;
        if self.lord_esteem >= LORD_BAR {
            self.ordinance = Some(Ordinance { passed_on: tick, price_mult: 1.5, licensed: true });
            self.log.push(format!("t{}: the lord grants the ordinance - drink dearer, earlier closing, no license no barrels.", tick));
        } else {
            self.repression_debt += 1;
            self.log.push(format!("t{}: the petition comes back refused - the lord owes the wives nothing he can see.", tick));
        }
    }
}
