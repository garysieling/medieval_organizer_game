//! Village mysteries for the seat: now and then somebody's asking after
//! this or that, and the answer's in the gossip folk pass on. Every so
//! often a Talk draws out one of the facts; hear them all and the
//! mystery's solved - carry the truth back to whoever wanted to know and
//! they remember it.

use crate::engine::ecs::Entity;
use crate::game::mysteries::{FACTS, MYSTERIES};
use crate::session::*;

/// A mystery the seat is piecing together: which one, which facts are
/// already heard (a bit per fact), and who's been asking
#[derive(Debug, Clone)]
pub struct SeatMystery {
    pub def: usize,
    pub known: u32,
    pub asker: Entity,
}

impl SeatMystery {
    /// How many facts are already heard
    pub fn heard(&self) -> usize { self.known.count_ones() as usize }
    /// All five in - the mystery's solved, waiting to be told
    pub fn solved(&self) -> bool { self.heard() >= FACTS }
    /// A fact not yet heard, picked by `roll` over what's left
    pub fn unheard(&self) -> Vec<usize> {
        (0..FACTS).filter(|i| self.known & (1 << i) == 0).collect()
    }
}

/// Days before another mystery can start once one's been told
const MYSTERY_WAIT: u64 = 12;
/// The daily chance a seat with nothing on gets a mystery going
const MYSTERY_CHANCE: i32 = 12;
/// The chance a Talk draws out a fact instead of small talk
const FACT_CHANCE: i32 = 12;

impl Game {
    /// Grown folk of the village who could actually be asked - and hear
    /// the answer: not children, not the dead or held, not country folk
    fn mystery_folk(&self) -> Vec<Entity> {
        self.villagers.iter().copied().filter(|&v| {
            self.world.get_component::<crate::game::vocation::Kid>(v).is_none()
                && !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<crate::session::country::Outlander>(v).is_none()
        }).collect()
    }

    /// Who's asking after `def`: the trade it suits, if the village's got
    /// one going, else any grown folk
    fn mystery_asker(&self, def: usize, folk: &[Entity], rng: &mut Rng) -> Option<Entity> {
        let suited = MYSTERIES[def].asker.and_then(|voc| folk.iter().copied().find(|&v|
            self.world.get_component::<crate::game::vocation::Craft>(v)
                .map(|c| c.vocation == voc).unwrap_or(false)));
        suited.or_else(|| folk.get(rng.below(folk.len() as i32) as usize).copied())
    }

    /// Dawn of each day: now and then a mystery starts doing the rounds -
    /// somebody's asking, and the gossip's out there to be heard
    pub(super) fn mystery_dawn(&mut self, tick: u64, rng: &mut Rng) {
        for seat in 0..self.seats.len() {
            if self.seats[seat].agent { continue; }
            // the asker's gone - died, or held where they can't be asked:
            // the question passes to another pair of ears, and the
            // gossip heard so far still counts
            if let Some(m) = &self.seats[seat].mystery {
                if crate::game::dungeon::unavailable(&self.world, m.asker) {
                    let folk = self.mystery_folk();
                    match self.mystery_asker(m.def, &folk, rng) {
                        Some(v) => {
                            let who = self.name_of(v);
                            if let Some(mm) = &mut self.seats[seat].mystery {
                                mm.asker = v;
                            }
                            self.log_entity_ev(
                                EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                                v,
                                format!("t{}: {} is asking after it now.", tick, who),
                            );
                        }
                        None => {
                            self.seats[seat].mystery = None;
                            self.log.push_ev(
                                EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                                format!("t{}: the question's gone unanswered.", tick),
                            );
                        }
                    }
                }
            }
            if self.seats[seat].mystery.is_some() { continue; }
            if tick < self.seats[seat].mystery_wait || rng.below(100) >= MYSTERY_CHANCE { continue; }
            let folk = self.mystery_folk();
            if folk.is_empty() { continue; }
            // which mystery: any but the one just told, once there's been one
            let n = MYSTERIES.len();
            let last = self.seats[seat].mystery_last;
            let def = if n > 1 && last < n {
                let mut d = rng.below((n - 1) as i32) as usize;
                if d >= last { d += 1; }
                d
            } else {
                rng.below(n as i32) as usize
            };
            let Some(asker) = self.mystery_asker(def, &folk, rng) else { continue };
            self.seats[seat].mystery_last = def;
            self.seats[seat].mystery = Some(SeatMystery { def, known: 0, asker });
            let title = MYSTERIES[def].title;
            let who = self.name_of(asker);
            self.say(
                asker,
                ASK_LINES[rng.below(ASK_LINES.len() as i32) as usize],
                TALK_BUBBLE_SECS * 2,
            );
            self.log_entity_ev(
                EvTag::new(Thread::Lifecycle, Valence::Neutral, Sev::Minor),
                asker,
                format!(
                    "t{}: {} has been asking after {} - folk might know.",
                    tick, who, title
                ),
            );
        }
    }

    /// A Talk draws out a fact of the mystery under way, now and then:
    /// the gossip line the villager says, and the fact marked heard.
    /// Returns the line to speak, having already noted it.
    pub(in crate::session) fn mystery_fact_at(&mut self, seat: usize) -> Option<String> {
        let m = self.seats[seat].mystery.clone()?;
        if m.solved() { return None; }
        if self.rng.below(100) >= FACT_CHANCE { return None; }
        let left = m.unheard();
        let i = left[self.rng.below(left.len() as i32) as usize];
        if let Some(mm) = &mut self.seats[seat].mystery { mm.known |= 1 << i; }
        let def = &MYSTERIES[m.def];
        let text = def.facts[i].to_string();
        let m2 = self.seats[seat].mystery.clone().unwrap();
        if m2.solved() {
            // name whoever's been asking - the log line's clickable, the
            // toast jumps to them (#646)
            let who = self.name_of(m2.asker);
            self.toasts.push_at(
                (
                    format!("Mystery solved: {} - {} wants to hear it.", def.title, who),
                    self.tick + 80,
                ),
                toast_spot(&self.world, m2.asker),
            );
            self.log_entity_ev(
                EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Notable),
                m2.asker,
                format!(
                    "t{}: that's all of it - the mystery of {} is {}'s to tell.",
                    self.tick, def.title, who
                ),
            );
        }
        Some(text)
    }

    /// The picked villager is the one who's been asking, and the
    /// mystery's whole: tell them what the gossip added up to
    pub fn mystery_deliver_at(&mut self, seat: usize) -> String {
        let tick = self.tick;
        let Some(m) = self.seats[seat].mystery.clone() else {
            return "You've no mystery to tell.".into();
        };
        if !m.solved() { return "You haven't heard all of it yet.".into(); }
        let v = self.villagers[self.seats[seat].selected_target];
        if v != m.asker { return "They weren't the one asking.".into(); }
        if crate::game::dungeon::unavailable(&self.world, v) {
            return "They can't hear you - they've gone from the village.".into();
        }
        // within earshot, like the job handover
        let (px, py) = self.player_pos_at(seat);
        let near = self.world.get_component::<Pos>(v)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= TALK_DIST)
            .unwrap_or(false);
        if !near { return "They're too far off to hear you - walk over first.".into(); }
        let def = &MYSTERIES[m.def];
        let who = self.name_of(v);
        self.say(v, def.wonders, TALK_BUBBLE_SECS * 2);
        // the truth's a debt paid in full: they remember it, and you're
        // friends now - a bond deep enough to read Friend and stay there
        let pe = self.seats[seat].player;
        for _ in 0..20 {
            crate::game::relationships::record_interaction(&mut self.world, pe,
                crate::game::relationships::Interaction {
                    other: v, kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
            let friend = crate::game::relationships::find_relationship(&self.world, pe, v)
                .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                .map(|r| r.kind == crate::game::relationships::RelKind::Friend)
                .unwrap_or(false);
            if friend { break; }
        }
        self.seats[seat].mystery = None;
        self.seats[seat].mystery_wait = tick + MYSTERY_WAIT;
        self.log_entity_ev(
            EvTag::new(Thread::Player, Valence::Positive, Sev::Minor),
            v,
            format!(
                "t{}: you tell {} what you pieced together about {}. {}",
                tick, who, def.title, def.thanks
            ),
        );
        self.say(v, def.thanks, TALK_BUBBLE_SECS * 3);
        format!("{}: \"{}\"", who, def.wonders)
    }

    /// The panel's line for the mystery under way: the question and how
    /// much of the gossip is in, with the asker carried as data so their
    /// name can be a button (#646)
    pub fn mystery_panel_line(&self, seat: usize) -> Option<MysteryV> {
        let m = self.seats[seat].mystery.as_ref()?;
        let def = &MYSTERIES[m.def];
        Some(MysteryV {
            title: def.title.to_string(),
            heard: m.heard(),
            total: FACTS,
            asker_name: self.name_of(m.asker),
            asker: self.villagers.iter().position(|&v| v == m.asker),
            asker_at: self.world.get_component::<Pos>(m.asker).map(|p| (p.x, p.y)),
        })
    }
}

/// What folk say when a mystery starts doing the rounds
const ASK_LINES: [&str; 3] = [
    "If anyone asks after it, I'll tell them you did.",
    "It's been nagging at me something fierce.",
    "Word is it's gone quiet - too quiet.",
];
