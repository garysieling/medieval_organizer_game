//! Village mysteries for the seat: now and then somebody's asking after
//! this or that, and the answer's in the gossip folk pass on. Every so
//! often a Talk draws out one of the facts; hear them all and the
//! mystery's solved - carry the truth back to whoever wanted to know and
//! they remember it.

use crate::engine::ecs::Entity;
use crate::game::mysteries::{Band, Dim, Mystery, Payoff, DIMS, MYSTERIES};
use crate::session::*;

/// A mystery the seat is piecing together: which one, which clues are
/// already heard (a bit per clue), and who's been asking
#[derive(Debug, Clone)]
pub struct SeatMystery {
    pub def: usize,
    pub known: u32,
    pub asker: Entity,
}

impl SeatMystery {
    /// The story it's working on
    pub fn story(&self) -> &'static Mystery { &MYSTERIES[self.def] }
    /// How many clues are already heard
    pub fn heard(&self) -> usize { self.known.count_ones() as usize }
    /// How many of one direction's clues are in - the deep stories can
    /// ask two of a kind (#1350)
    pub fn heard_dim(&self, dim: Dim) -> usize {
        self.story().clues.iter().enumerate()
            .filter(|(i, c)| c.dim == dim && self.known & (1 << i) != 0)
            .count()
    }
    /// Enough heard, and every direction the story wants covered - not
    /// a flat count: a hard ask wants two of a kind (#1350)
    pub fn solved(&self) -> bool {
        let d = self.story();
        self.heard() >= d.need
            && d.dims.iter().all(|&(dim, n)| self.heard_dim(dim) >= n)
    }
    /// A clue not yet heard, picked by `roll` over what's left
    pub fn unheard(&self) -> Vec<usize> {
        (0..self.story().clues.len()).filter(|i| self.known & (1 << i) == 0).collect()
    }
}

/// What a telling came to, for whoever's showing it
#[derive(Debug, Clone, Default)]
pub struct MysteryTold {
    pub who: String,
    pub wonders: &'static str,
    pub thanks: &'static str,
    /// The payoff, in a line ("Ada asks you to their table.")
    pub payoff: String,
}

/// Days a mystery's follow-up job waits for you to take it
const LEAD_DAYS: u64 = 6;
/// How far the word going round warms the rest of the village
const WORD_RAPPORT: f32 = 0.1;
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
            // which mystery: one of the seat's band (#1350) - the first
            // asks come easy, the deep stories come round as its own
            // solves rack up - and any but the one just told
            let band = self.mystery_band(seat);
            let last = self.seats[seat].mystery_last;
            let pool: Vec<usize> = (0..MYSTERIES.len())
                .filter(|&i| MYSTERIES[i].band() == band && i != last)
                .collect();
            if pool.is_empty() { continue; }
            let def = pool[rng.below(pool.len() as i32) as usize];
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
        if self.seats[seat].mystery.as_ref().map(|m| m.solved()).unwrap_or(true)
            || self.rng.below(100) >= FACT_CHANCE { return None; }
        self.mystery_fact_take(seat)
    }

    /// Asked after the mystery outright (#1189): a piece for the asking
    /// if one's still unheard - the same marking, without the roll
    pub(in crate::session) fn mystery_fact_asked(&mut self, seat: usize) -> Option<String> {
        self.mystery_fact_take(seat)
    }

    fn mystery_fact_take(&mut self, seat: usize) -> Option<String> {
        let m = self.seats[seat].mystery.clone()?;
        if m.solved() { return None; }
        let left = m.unheard();
        if left.is_empty() { return None; }
        let i = left[self.rng.below(left.len() as i32) as usize];
        if let Some(mm) = &mut self.seats[seat].mystery { mm.known |= 1 << i; }
        let def = &MYSTERIES[m.def];
        let text = def.clues[i].text.to_string();
        let m2 = self.seats[seat].mystery.clone().unwrap();
        if m2.solved() {
            // the seat's band steps up on the solve itself - carrying it
            // back to the asker's a deed of its own, not part of the ramp
            self.seats[seat].mystery_solved += 1;
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
    /// mystery's whole: tell them what the gossip added up to (the
    /// context button's way - the talk panel's is `mystery_tell`)
    pub fn mystery_deliver_at(&mut self, seat: usize) -> String {
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
        let told = self.mystery_tell(seat, v).unwrap_or_default();
        format!("{}: \"{}\" {}", told.who, told.wonders, told.payoff)
    }

    /// Whether the seat has a whole mystery `v` is waiting to hear - the
    /// talk panel offers the telling while it does
    pub fn mystery_due(&self, seat: usize, v: Entity) -> bool {
        self.seats[seat].mystery.as_ref().map(|m| m.solved() && m.asker == v).unwrap_or(false)
            && !crate::game::dungeon::unavailable(&self.world, v)
    }

    /// Tell `v` the solved mystery: they take it in, you're friends, they
    /// open up as for a job seen to, and the mystery pays off its own
    /// way (`Payoff`). None if there's nothing due them
    pub(in crate::session) fn mystery_tell(&mut self, seat: usize, v: Entity) -> Option<MysteryTold> {
        if !self.mystery_due(seat, v) { return None; }
        let tick = self.tick;
        let m = self.seats[seat].mystery.clone()?;
        let def = &MYSTERIES[m.def];
        let who = self.name_of(v);
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
        // a question answered is a deed done them: they read as known,
        // and tell you a thing of their own the way a job's giver does
        self.quest_bond(seat, v);
        self.seats[seat].mystery = None;
        self.seats[seat].mystery_wait = tick + MYSTERY_WAIT;
        let payoff = self.mystery_payoff(seat, v, def.payoff);
        self.log_entity_ev(
            EvTag::new(Thread::Player, Valence::Positive, Sev::Notable),
            v,
            format!(
                "t{}: you tell {} what you pieced together about {}. {} {}",
                tick, who, def.title, def.thanks, payoff
            ),
        );
        self.toasts.push_at((format!("Mystery told: {}", payoff), tick + 80),
            toast_spot(&self.world, v));
        self.say(v, def.thanks, TALK_BUBBLE_SECS * 3);
        Some(MysteryTold { who, wonders: def.wonders, thanks: def.thanks, payoff })
    }

    /// What the telling's worth, landed: the coin, the invitation, the
    /// job, or the word going round. Returns the line that says so
    fn mystery_payoff(&mut self, seat: usize, v: Entity, payoff: Payoff) -> String {
        let tick = self.tick;
        let who = self.name_of(v);
        let pe = self.seats[seat].player;
        // no roof to ask you under: the asker talks it about instead
        let payoff = match payoff {
            Payoff::Table if self.seats[seat].table.is_some()
                || crate::game::household::home_of(&self.world, v).is_none() => Payoff::Word,
            p => p,
        };
        match payoff {
            Payoff::Gift(gold) => {
                // both purses, as a job's pay: the panel reads one, trade the other
                if let Some(p) = self.world.get_component_mut::<crate::game::player::PlayerCharacter>(pe) { p.resources += gold; }
                if let Some(p) = self.world.get_component_mut::<crate::game::livelihood::Purse>(pe) { p.gold += gold; }
                format!("{} presses {:.0}g on you for the trouble.", who, gold)
            }
            Payoff::Table => {
                self.seats[seat].table_invite =
                    Some((v, tick + crate::session::subsystems::table::INVITE_DAYS));
                format!("{} asks you to their table.", who)
            }
            Payoff::Lead => {
                let mut rng = Rng(self.rng.next());
                let Some(t) = self.note_target(v, &mut rng) else {
                    return self.mystery_payoff(seat, v, Payoff::Word);
                };
                self.pending_quests.retain(|q| q.giver != v);
                self.pending_quests.push(crate::session::quests::PendingQuest {
                    giver: v, kind: crate::session::quests::QuestKind::Note,
                    target: crate::session::quests::QuestTarget::Villager(t),
                    pickup: None, expires: tick + LEAD_DAYS,
                });
                let to = self.name_of(t);
                if self.seats[seat].quest.is_none() {
                    let i = self.pending_quests.len() - 1;
                    self.take_pending_quest(seat, i);
                    format!("{} has a note for {} on the back of it - the job's yours.", who, to)
                } else {
                    format!("{} has a note for {} on the back of it, when your hands are free.", who, to)
                }
            }
            Payoff::Word => {
                let folk: Vec<Entity> = self.mystery_folk().into_iter().filter(|&f| f != v).collect();
                for f in &folk {
                    let mut rap = self.world.get_component::<crate::game::player::Rapport>(*f)
                        .cloned().unwrap_or_default();
                    let e = rap.by_player.entry(pe).or_insert(0.0);
                    *e = (*e + WORD_RAPPORT).min(1.0);
                    self.world.add_component(*f, rap);
                }
                format!("{} tells it all over the village - folk think the better of you.", who)
            }
        }
    }

    /// The panel's line for the mystery under way: the question, how
    /// much of the gossip is in, and how far each direction's come -
    /// with the asker carried as data so their name can be a button (#646)
    pub fn mystery_panel_line(&self, seat: usize) -> Option<MysteryV> {
        let m = self.seats[seat].mystery.as_ref()?;
        let def = &MYSTERIES[m.def];
        Some(MysteryV {
            title: def.title.to_string(),
            heard: m.heard(),
            total: def.clues.len(),
            dims: DIMS.iter().map(|&d| {
                let want = def.dims.iter().find(|&&(dd, _)| dd == d)
                    .map(|&(_, n)| n).unwrap_or(0);
                (d.word().to_string(), m.heard_dim(d), want)
            }).collect(),
            asker_name: self.name_of(m.asker),
            asker: self.villagers.iter().position(|&v| v == m.asker),
            asker_at: self.world.get_component::<Pos>(m.asker).map(|p| (p.x, p.y)),
        })
    }

    /// Which band of the pool the seat's next mystery draws from - its
    /// own count of solved ones, so each seat walks its own ramp (#1350)
    fn mystery_band(&self, seat: usize) -> Band {
        match self.seats[seat].mystery_solved {
            0 => Band::Easy,
            1 | 2 => Band::Medium,
            _ => Band::Hard,
        }
    }
}

/// What folk say when a mystery starts doing the rounds
const ASK_LINES: [&str; 3] = [
    "If anyone asks after it, I'll tell them you did.",
    "It's been nagging at me something fierce.",
    "Word is it's gone quiet - too quiet.",
];
