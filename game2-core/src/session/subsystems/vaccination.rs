//! The vaccination act's three answers and the Leicester counter-
//! institution's slow working: the officer's offer resolves once, the
//! infants carry the scratch or the refusers pay the fine, and where
//! the Act's refused the village phases in its own method - the word
//! cried, the sick kept apart, contacts held, houses cleansed.

use crate::game::desires::{self, Problem};
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::livelihood::Purse;
use crate::session::events::VaxOffer;
use crate::session::*;
use crate::sim::illness::{LeicesterInstitution, SmallpoxVaccine};

/// Days of labour between one phase of the Leicester method and the next
const LEICESTER_PHASE_DAYS: u64 = 20;
/// Days the square takes to talk the Leicester way over before the
/// village's own institution rises - the player backs it, the village
/// builds it
const LEICESTER_FOUND_DAYS: u64 = 10;
/// The fine each refusing household pays the Act's officer
const REFUSER_FINE: f32 = 5.0;

impl Game {
    /// The Act's officer gets his answer: the scratch for the infants,
    /// the fine for the refusers, or the village's own counter-
    /// institution - once, and the offer's spent either way
    pub fn resolve_vaccination(&mut self, offer: VaxOffer, pick: usize) {
        self.vax_offer = None;
        self.vax_done = true;
        let at = toast_spot(&self.world, offer.authority);
        match pick {
            0 => {
                // the scratch: every infant carries the Act's mark for
                // life, and a principled few remember the compulsion
                let mut marked = 0;
                for &v in &self.villagers {
                    let infant = self.world.get_component::<Age>(v)
                        .map(|a| a.stage == LifeStage::Infant).unwrap_or(false);
                    if infant {
                        self.world.add_component(v, SmallpoxVaccine);
                        marked += 1;
                    }
                }
                let mut resentful = 0;
                for (i, &v) in self.villagers.iter().enumerate() {
                    if i % 4 != 0 { continue; }
                    let grown = self.world.get_component::<Age>(v)
                        .map(|a| matches!(a.stage, LifeStage::Adult | LifeStage::Elder))
                        .unwrap_or(false);
                    if grown {
                        desires::add_grievance(&mut self.world, v, Problem::Taxed, self.tick + 90);
                        resentful += 1;
                    }
                }
                self.log.push_ev_at_who(EvTag::new(Thread::Illness, Valence::Positive, Sev::Notable), format!(
                    "t{}: the lancet's scratch runs round the cradle-roll - {} infant{} carry the Act's mark, and {} remember the compulsion.",
                    self.tick, marked, if marked == 1 { "" } else { "s" }, resentful), at, None);
            }
            1 => {
                // the refusal: every household with an infant pays the
                // fine, and the league takes shape in the tavern - the
                // grievance's the village's own memory of it
                let mut houses: Vec<Entity> = vec![];
                for &v in &self.villagers {
                    let infant = self.world.get_component::<Age>(v)
                        .map(|a| a.stage == LifeStage::Infant).unwrap_or(false);
                    if infant {
                        if let Some(h) = crate::game::household::home_of(&self.world, v) {
                            if !houses.contains(&h) { houses.push(h); }
                        }
                    }
                }
                let house_count = houses.len();
                for h in houses {
                    let Some(b) = self.world.get_component::<crate::game::household::Building>(h) else { continue };
                    let members = b.occupants.clone();
                    for m in members {
                        if villager::is_dead(&self.world, m) { continue; }
                        if let Some(p) = self.world.get_component_mut::<Purse>(m) {
                            p.gold = (p.gold - REFUSER_FINE).max(0.0);
                        }
                        desires::add_grievance(&mut self.world, m, Problem::Taxed, self.tick + 120);
                    }
                }
                self.log.push_ev_at_who(EvTag::new(Thread::Law, Valence::Negative, Sev::Notable), format!(
                    "t{}: the Act's refused - {} household{} pay the fine, and the anti-vaccination league takes shape in the tavern.",
                    self.tick, house_count, if house_count == 1 { "" } else { "s" }), at, None);
            }
            2 => {
                // the Leicester way: the player backs the village's own
                // counter-institution - the square talks it over a few
                // days, then the village builds it herself, phase by
                // phase, and no officer's hand is on it
                self.leicester_backing = Some(self.tick);
                self.log.push_ev_at_who(EvTag::new(Thread::Illness, Valence::Positive, Sev::Notable), format!(
                    "t{}: the Leicester way's backed - the village'll take up its own line against the pox, no officer's hand on it.",
                    self.tick), at, None);
            }
            _ => {}
        }
    }

    /// The Leicester counter-institution's slow working: backed by the
    /// player, founded by the village a few days on, then a season's
    /// labour between phases - notification, isolation, quarantine of
    /// contacts, disinfection
    pub fn leicester_tick(&mut self, tick: u64) {
        if self.world.query::<LeicesterInstitution>().is_empty() {
            // the village's own answer: the player's backing's had its
            // days in the square, and now she rises
            if let Some(since) = self.leicester_backing {
                if tick >= since + LEICESTER_FOUND_DAYS {
                    self.world.add_component(self.market_food, LeicesterInstitution {
                        since: tick, phase: 0 });
                    self.leicester_backing = None;
                    self.log.push_ev(EvTag::new(Thread::Illness, Valence::Positive, Sev::Minor), format!("t{}: the village takes up the Leicester way - the word's cried first, the method to follow.", tick));
                }
            }
            return;
        }
        let Some(e) = self.world.query::<LeicesterInstitution>().first().copied() else { return };
        let since = self.world.get_component::<LeicesterInstitution>(e)
            .map(|l| l.since).unwrap_or(tick);
        let want = ((tick.saturating_sub(since) / LEICESTER_PHASE_DAYS).min(3)) as u8;
        let Some(l) = self.world.get_component_mut::<LeicesterInstitution>(e) else { return };
        while l.phase < want {
            l.phase += 1;
            let line = match l.phase {
                1 => "the sick are kept apart - isolation",
                2 => "the contacts are held - quarantine",
                _ => "the houses cleansed - disinfection, and the method holds",
            };
            self.log.push_ev(
                EvTag::new(Thread::Illness, Valence::Neutral, Sev::Minor),
                format!("t{}: Leicester - {}.", tick, line),
            );
        }
    }
}
