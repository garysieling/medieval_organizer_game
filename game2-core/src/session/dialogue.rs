//! Villager-to-villager conversations on the map: when two villagers meet
//! in passing, the conversation framework (game::conversation, with the
//! village_library trees) plays out a whole exchange between them, one
//! speech bubble per turn, instead of a single canned remark.
//!
//! The speakers are read off the world: their Personality traits, their
//! trade, what's on them right now (sick, hungry, drunk, grieving, old),
//! and the bond between the two - so a sick weaver meeting her rival
//! talks differently from two friends at the well.

use crate::engine::ecs::{Entity, World};
use crate::game::conversation::{converse, Ending, Relation, Rng as TalkRng, Speaker};
use crate::game::dialogue_trees::village_library;
use crate::game::conversation::DialogueTree;
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::relationships::RelKind;
use crate::game::traits::Personality;
use crate::game::villager::{self, Hunger};
use crate::game::vocation::{Craft, TownRole, Vocation};
use crate::sim::psychology::{AttachmentHistory, EpisodeKind};
use super::{Game, FPS};

/// The longest exchange played out on the map: past this the pair have
/// usually walked on anyway, and a long talk crowds the view
pub const CONVO_MAX_LINES: usize = 6;
/// Frames between one line and the reply
pub const CONVO_BEAT_FRAMES: u32 = 120;
/// How long each line's bubble stays up (seconds) - a little past the beat
/// so the reply appears while the line it answers is still readable
pub const CONVO_BUBBLE_SECS: u32 = 3;
/// Drift apart further than this (tiles) and the talk breaks off
pub const CONVO_RANGE: f32 = 6.0;

/// A conversation being played out between two villagers
#[derive(Debug, Clone)]
pub struct LiveConvo {
    pub a: Entity,
    pub b: Entity,
    /// (who says it, what they say), in order
    pub lines: Vec<(Entity, String)>,
    /// The next line to say
    pub next: usize,
    /// The frame the next line is due
    pub due: u32,
    /// How it ended, for the log once it's played out
    pub ending: Option<Ending>,
    /// The most specific tree it opened on ("sickness", "old_scores", ...)
    pub lead: &'static str,
}

/// The village dialogue library, built once
fn library() -> &'static [DialogueTree] {
    static LIB: std::sync::OnceLock<Vec<DialogueTree>> = std::sync::OnceLock::new();
    LIB.get_or_init(village_library)
}

/// The fact vocabulary the trees speak for a trade
fn trade_status(v: Vocation) -> Option<&'static str> {
    Some(match v {
        Vocation::Toolmaker => "smith",
        Vocation::Mason => "mason",
        Vocation::Shepherd => "shepherd",
        Vocation::Beekeeper => "beekeeper",
        Vocation::Weaver => "weaver",
        Vocation::Fisher => "fisher",
        Vocation::Brewer | Vocation::Alewife => "brewer",
        Vocation::Baker => "baker",
        Vocation::Taverner => "taverner",
        Vocation::Cooper => "carpenter",
        _ => return None,
    })
}

/// `who` as a conversation speaker, read off the world at `tick`
pub fn speaker_of(world: &World, who: Entity, name: &str, tick: u64) -> Speaker {
    let mut s = Speaker::new(name);
    if let Some(p) = world.get_component::<Personality>(who) { s.traits = p.traits.clone(); }
    let craft = world.get_component::<Craft>(who).map(|c| c.vocation);
    s.ideology = match craft {
        Some(Vocation::Priest) => "clergy",
        Some(Vocation::Knight) => "lord",
        Some(Vocation::Perfumer) => "merchant",
        _ if crate::game::livelihood::has_role(world, who, TownRole::Trader) => "merchant",
        _ => "laborer",
    };
    if let Some(st) = craft.and_then(trade_status) { s.status.push(st); }
    if crate::game::livelihood::has_role(world, who, TownRole::Herbologist) { s.status.push("herb_wife"); }
    if super::is_sick(world, who) { s.status.push("sick"); }
    if world.get_component::<Hunger>(who).map(|h| h.value < 30.0).unwrap_or(false) { s.status.push("hungry"); }
    if world.get_component::<villager::Drunk>(who).map(|d| tick < d.until).unwrap_or(false) { s.status.push("drunk"); }
    if world.get_component::<Age>(who).map(|a| a.stage == LifeStage::Elder).unwrap_or(false) { s.status.push("elder"); }
    let grieving = world.get_component::<AttachmentHistory>(who).map(|h| h.episodes.iter()
        .any(|e| matches!(e.kind, EpisodeKind::Loss) && villager::is_dead(world, e.with))).unwrap_or(false);
    if grieving { s.status.push("grieving"); }
    s
}

/// The bond between two villagers, in the conversation's terms. Two
/// villagers who've never spoken are still neighbours - RelKind::Stranger
/// only means no interactions yet, and the trees' stranger material is
/// for travellers on the road ("who art thou?"), not the miller's wife
pub fn relation_of(kind: Option<RelKind>) -> Relation {
    match kind {
        Some(RelKind::Family | RelKind::Lover) => Relation::Kin,
        Some(RelKind::Friend) => Relation::Friend,
        Some(RelKind::Rival) => Relation::Rival,
        Some(RelKind::Acquaintance | RelKind::Mentor | RelKind::Apprentice | RelKind::Stranger) | None => Relation::Acquaintance,
    }
}

impl Game {
    /// Is `who` in a conversation being played out right now?
    pub fn in_convo(&self, who: Entity) -> bool {
        self.convos.iter().any(|c| c.a == who || c.b == who)
    }

    /// Two villagers meet: run the conversation between them and start
    /// playing it out (the first line goes up now). `a` opens. Returns
    /// whether a conversation began - not when either is already talking.
    pub fn start_convo(&mut self, a: Entity, b: Entity) -> bool {
        if a == b || self.in_convo(a) || self.in_convo(b) { return false; }
        let (na, nb) = (self.name_of(a), self.name_of(b));
        let sa = speaker_of(&self.world, a, &na, self.tick);
        let sb = speaker_of(&self.world, b, &nb, self.tick);
        let rel = relation_of(self.rel_kind(a, b));
        // seeded off the meeting itself, not the game's rng: chatter on the
        // street mustn't shift every other roll in the sim
        let seed = self.tick.wrapping_mul(0x9E37_79B9)
            ^ ((a as u64) << 32) ^ (b as u64) ^ self.last_chat.len() as u64;
        let tr = converse(&sa, &sb, rel, library(), &mut TalkRng::new(seed));
        let lines: Vec<(Entity, String)> = tr.turns.iter().take(CONVO_MAX_LINES)
            .map(|t| (if t.speaker == 0 { a } else { b }, t.text.clone())).collect();
        if lines.is_empty() { return false; }
        let lead = tr.trees_walked.first().copied().unwrap_or("village_smalltalk");
        // cut short on the map, it never reached its ending
        let ending = if lines.len() == tr.turns.len() { tr.ending } else { None };
        self.convos.push(LiveConvo { a, b, lines, next: 0, due: self.frame_n, ending, lead });
        self.advance_convos();
        true
    }

    /// Say each conversation's next line when it's due; break off talks
    /// whose speakers have walked apart, died or been taken off the map
    pub fn advance_convos(&mut self) {
        let frame = self.frame_n;
        let mut convos = std::mem::take(&mut self.convos);
        convos.retain_mut(|c| {
            let gone = |e: Entity| villager::is_dead(&self.world, e)
                || crate::game::dungeon::unavailable(&self.world, e);
            if gone(c.a) || gone(c.b) { return false; }
            let apart = match (self.world.get_component::<villager::Pos>(c.a), self.world.get_component::<villager::Pos>(c.b)) {
                (Some(p), Some(q)) => (p.x - q.x).powi(2) + (p.y - q.y).powi(2) > CONVO_RANGE * CONVO_RANGE,
                _ => true,
            };
            if apart { return false; }
            if frame < c.due { return true; }
            let (who, text) = c.lines[c.next].clone();
            // a non-passing bubble (a Talk reply, a meeting thought) can be
            // up over `who` right now; don't spend the turn until it shows
            if !self.say_in_passing(who, &text, CONVO_BUBBLE_SECS) { return true; }
            c.next += 1;
            c.due = frame + CONVO_BEAT_FRAMES;
            if c.next < c.lines.len() { return true; }
            // played out: a walk-off is worth a line in the log
            if c.ending == Some(Ending::WalksOff) {
                let (na, nb) = (self.name_of(c.a), self.name_of(c.b));
                let leaver = if who == c.a { &na } else { &nb };
                let left = if who == c.a { &nb } else { &na };
                self.log.push(format!("t{}: {} turned on their heel and left {} standing.", self.tick, leaver, left));
            }
            false
        });
        // a talk started mid-pass (none do today) would land in self.convos
        convos.append(&mut self.convos);
        self.convos = convos;
    }
}

/// Frames a whole conversation of `n` lines takes to play out
pub fn convo_frames(n: usize) -> u32 {
    n.saturating_sub(1) as u32 * CONVO_BEAT_FRAMES + CONVO_BUBBLE_SECS * FPS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::traits::Trait;

    #[test]
    fn test_speaker_reads_traits_trade_and_sickness_off_the_world() {
        let mut g = Game::new();
        let v = g.villagers[1];
        g.world.add_component(v, Personality { traits: vec![Trait::Hotheaded] });
        g.world.add_component(v, crate::game::vocation::craft(Vocation::Weaver, 0.5));
        g.world.add_component(v, Hunger { value: 5.0 });
        let s = speaker_of(&g.world, v, "Eliza", g.tick);
        assert_eq!(s.traits, vec![Trait::Hotheaded]);
        assert!(s.status.contains(&"weaver") && s.status.contains(&"hungry"), "{:?}", s.status);
        g.world.add_component(v, crate::game::vocation::craft(Vocation::Priest, 0.5));
        assert_eq!(speaker_of(&g.world, v, "Eliza", g.tick).ideology, "clergy");
    }

    #[test]
    fn test_bonds_map_onto_conversation_relations() {
        assert_eq!(relation_of(Some(RelKind::Family)), Relation::Kin);
        assert_eq!(relation_of(Some(RelKind::Rival)), Relation::Rival);
        assert_eq!(relation_of(Some(RelKind::Friend)), Relation::Friend);
        // neighbours who've never spoken aren't travellers on the road
        assert_eq!(relation_of(None), Relation::Acquaintance);
        assert_eq!(relation_of(Some(RelKind::Stranger)), Relation::Acquaintance);
    }

    #[test]
    fn test_two_villagers_meeting_play_out_a_conversation_in_turns() {
        let mut g = Game::new();
        let (a, b) = (g.villagers[1], g.villagers[2]);
        let p = *g.world.get_component::<villager::Pos>(a).unwrap();
        *g.world.get_component_mut::<villager::Pos>(b).unwrap() = villager::Pos { x: p.x + 0.5, y: p.y };
        assert!(g.start_convo(a, b));
        assert!(!g.start_convo(a, b), "one conversation at a time each");
        let first = g.bubbles.iter().find(|x| x.0 == a).map(|x| x.1.clone()).expect("the opener speaks at once");
        let total = g.convos[0].lines.len();
        assert!(total >= 2, "a conversation, not a remark");
        // hold them together and let the talk run its course
        let mut heard: Vec<(Entity, String)> = vec![(a, first)];
        for _ in 0..convo_frames(total) + 10 {
            g.frame_n += 1;
            *g.world.get_component_mut::<villager::Pos>(a).unwrap() = p;
            *g.world.get_component_mut::<villager::Pos>(b).unwrap() = villager::Pos { x: p.x + 0.5, y: p.y };
            g.advance_convos();
            for x in &g.bubbles {
                if (x.0 == a || x.0 == b) && !heard.iter().any(|h| h.1 == x.1) { heard.push((x.0, x.1.clone())); }
            }
        }
        assert!(g.convos.is_empty(), "played out");
        assert!(heard.len() >= 2 && heard.iter().any(|h| h.0 == b), "both speak: {:?}", heard);
    }

    #[test]
    fn test_walking_apart_breaks_the_talk_off() {
        let mut g = Game::new();
        let (a, b) = (g.villagers[1], g.villagers[2]);
        let p = *g.world.get_component::<villager::Pos>(a).unwrap();
        *g.world.get_component_mut::<villager::Pos>(b).unwrap() = villager::Pos { x: p.x + 0.5, y: p.y };
        assert!(g.start_convo(a, b));
        *g.world.get_component_mut::<villager::Pos>(b).unwrap() = villager::Pos { x: p.x + CONVO_RANGE + 1.0, y: p.y };
        g.advance_convos();
        assert!(g.convos.is_empty());
    }

    #[test]
    fn test_a_sick_villager_opens_on_the_sickness_tree() {
        let mut g = Game::new();
        let (a, b) = (g.villagers[1], g.villagers[2]);
        let p = *g.world.get_component::<villager::Pos>(a).unwrap();
        *g.world.get_component_mut::<villager::Pos>(b).unwrap() = villager::Pos { x: p.x + 0.5, y: p.y };
        g.world.add_component(a, crate::sim::illness::Health { condition: 0.5, immunity: 0.1 });
        let mut d = crate::sim::illness::Disease::new(crate::sim::illness::DiseaseKind::Influenza, a, 0.5);
        d.incubation = 0;
        d.stage = crate::sim::illness::Stage::Moderate;
        let de = g.world.create_entity();
        g.world.add_component(de, d);
        assert!(speaker_of(&g.world, a, "x", g.tick).status.contains(&"sick"));
        assert!(g.start_convo(a, b));
        assert!(g.convos[0].lead.contains("sick"), "led with {}", g.convos[0].lead);
    }

    #[test]
    fn test_villagers_talk_to_each_other_in_a_running_game() {
        let mut g = Game::new();
        let mut started = 0;
        let mut lines_said = 0;
        let mut seen: Vec<(Entity, Entity, usize)> = vec![];
        for _ in 0..FPS * 60 * 3 {
            g.update();
            for c in &g.convos {
                match seen.iter_mut().find(|s| s.0 == c.a && s.1 == c.b) {
                    Some(s) => s.2 = s.2.max(c.next),
                    None => { started += 1; seen.push((c.a, c.b, c.next)); }
                }
            }
        }
        lines_said += seen.iter().map(|s| s.2).sum::<usize>();
        eprintln!("convos started {}, lines said {}, per convo {:?}", started, lines_said,
            seen.iter().map(|s| s.2).collect::<Vec<_>>());
        assert!(started > 0, "villagers who meet should talk");
        assert!(lines_said > started, "and get past the opening line");
    }
}
