//! The telling layer (#1445): a per-villager beat scheduler - candidate
//! emotional beats are nominated by their triggers, scored by how well
//! the player knows the person, the event's weight and the beat's
//! novelty, and capped at one major beat per villager per in-game week.
//! `storyteller_tick` stays the campaign-pressure cycle; this is the
//! person-scale one. New beat kinds register their triggers in
//! `beat_nominee` as their upstream systems land (E8-2's belief events,
//! E5-9's transcripts, E3/E5's disclosures).

use crate::session::*;
use crate::session::dialogue::TalkKind;
use crate::game::desires::{self, InnerLife, Problem};
use crate::sim::psychology::{AttachmentHistory, BeatKind, BeatMemory, EpisodeKind};

/// Every kind the design names; the unwired ones nominate nothing yet
const BEAT_KINDS: [BeatKind; 8] = [
    BeatKind::Loss,
    BeatKind::FirstDeclaration,
    BeatKind::Change,
    BeatKind::Remembered,
    BeatKind::Confession,
    BeatKind::Reproach,
    BeatKind::Return,
    BeatKind::ConvincingSurprise,
];

/// The player's ear is the opportunity (#1445): a villager who's had it
/// within the last few days - a talk, a meeting they attended, a hearing
/// - has somewhere a declaration can land
const EAR_DAYS: u64 = 3;

/// What a nomination carries back for the memory: who was lost, what
/// was declared - so the same moment doesn't beat twice
enum BeatDedup {
    /// The kinds that carry nothing to dedup (later kinds will)
    #[allow(dead_code)]
    None,
    Lost(Entity),
    Declared(Problem),
}

impl Game {
    /// The day's beats: per villager, the best-scored nomination fires
    /// if the week's cap hasn't already spent them
    pub(in crate::session) fn beat_tick(&mut self, tick: u64) {
        for &v in self.villagers.clone().iter() {
            if villager::is_dead(&self.world, v)
                || self.world.get_component::<crate::game::vocation::Kid>(v).is_some() {
                continue;
            }
            let mem = self.world.get_component::<BeatMemory>(v).cloned()
                .unwrap_or_else(|| BeatMemory {
                    last_major: u64::MAX, of_kind: vec![], lost: vec![], declared: vec![],
                });
            if mem.capped(tick) { continue; }
            // how well the player knows them scores the moment - a
            // stranger's beat is noise the log doesn't want
            let knows = self.player_knows(v);
            if knows <= 0.0 { continue; }
            let mut best: Option<(f32, BeatKind, String, BeatDedup)> = None;
            for kind in BEAT_KINDS {
                let Some((weight, line, dedup)) = self.beat_nominee(v, kind, tick, &mem) else { continue };
                let novelty = if mem.of_kind.iter().any(|(k, _)| *k == kind) { 0.5 } else { 1.0 };
                let score = knows * weight * novelty;
                if best.as_ref().map(|(s, ..)| score > *s).unwrap_or(true) {
                    best = Some((score, kind, line, dedup));
                }
            }
            let Some((_, kind, line, dedup)) = best else { continue };
            self.fire_beat(v, kind, &line, tick, mem, dedup);
        }
    }

    /// How well the player knows this villager: the rapport their talks
    /// and meetings built, or the bond itself - whichever reads warmer
    fn player_knows(&self, v: Entity) -> f32 {
        self.seats.iter().map(|s| {
            let rap = self.world.get_component::<crate::game::player::Rapport>(v)
                .and_then(|r| r.by_player.get(&s.player).copied()).unwrap_or(0.0);
            let rel = crate::game::relationships::find_relationship(&self.world, s.player, v)
                .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                .map(|r| r.strength).unwrap_or(0.0);
            rap.max(rel.max(0.0))
        }).fold(0.0f32, f32::max)
    }

    /// A kind's candidate for this villager today: the weight it'd land
    /// with and the line it says, or None when its trigger isn't met
    fn beat_nominee(&self, v: Entity, kind: BeatKind, tick: u64,
            mem: &BeatMemory) -> Option<(f32, String, BeatDedup)> {
        match kind {
            BeatKind::Loss => {
                // the bereavement is still on them, and this dead one
                // hasn't been mourned aloud yet
                let grieving = self.world.get_component::<InnerLife>(v)
                    .map(|l| l.grievances.iter().any(|(p, _)| *p == Problem::Bereaved))
                    .unwrap_or(false);
                if !grieving { return None; }
                let dead = self.world.get_component::<AttachmentHistory>(v)
                    .and_then(|h| h.episodes.iter().rev()
                        .find(|e| e.kind == EpisodeKind::Loss))
                    .map(|e| e.with)?;
                if mem.lost.contains(&dead) { return None; }
                let name = self.name_of(dead);
                Some((1.0,
                    format!("{}... I still look for them at the door.", name),
                    BeatDedup::Lost(dead)))
            }
            BeatKind::FirstDeclaration => {
                // a trouble they'd only held, and the player's ear
                // lately - they say it out loud for the first time
                let ear = self.seats.iter().any(|s|
                    self.world.get_component::<crate::game::player::TalkCount>(v)
                        .and_then(|t| t.last_day_by.get(&s.player).copied())
                        .map(|d| tick < d + EAR_DAYS).unwrap_or(false));
                if !ear { return None; }
                let p = self.world.get_component::<InnerLife>(v)?
                    .grievances.iter()
                    .map(|(p, _)| *p)
                    .find(|p| *p != Problem::Bereaved && !mem.declared.contains(p))?;
                Some((0.8,
                    format!("It's {}. There - I've said it out loud.", desires::wrong_phrase(p)),
                    BeatDedup::Declared(p)))
            }
            // the rest nominate as their systems land: Change off E8-2's
            // belief events, the transcripts' beats off E5-9, the
            // disclosures off E3/E5
            _ => None,
        }
    }

    /// The beat lands: the bubble over their head, the line on their
    /// record, a word in the log - and the memory marked so the week's
    /// cap and the same-moment dedup hold
    fn fire_beat(&mut self, v: Entity, kind: BeatKind, line: &str, tick: u64,
            mut mem: BeatMemory, dedup: BeatDedup) {
        self.say(v, line, TALK_BUBBLE_SECS);
        let id = self.push_talk(TalkKind::Chat, vec![v], vec![]);
        self.note_talk(id, Some(v), line);
        let thread = match kind {
            BeatKind::Loss => Thread::Lifecycle,
            _ => Thread::Other,
        };
        let who = self.name_of(v).to_string();
        self.log_entity_ev(
            EvTag::new(thread, Valence::Neutral, Sev::Minor),
            v,
            format!("t{}: {}: {}", tick, who, line),
        );
        mem.last_major = tick;
        mem.of_kind.push((kind, tick));
        match dedup {
            BeatDedup::Lost(d) => mem.lost.push(d),
            BeatDedup::Declared(p) => mem.declared.push(p),
            BeatDedup::None => {}
        }
        self.world.add_component(v, mem);
    }
}
