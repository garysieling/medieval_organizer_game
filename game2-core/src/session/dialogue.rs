//! Talk in the village, through the conversation framework
//! (game::conversation + the village_library trees), kept on the TALK
//! panel at the foot of the side panel - apart from the event LOG.
//!
//! - Villager to villager: when two villagers meet in passing they hold a
//!   whole conversation. It isn't floated over the map (a street full of
//!   bubbles was noise); it goes into the talk history, and clicking
//!   either of them shows it on the TALK panel.
//! - The player's Talk: the villager opens from the trees, says what's on
//!   their mind (the concern or secret Talk has always drawn out), and the
//!   player answers by picking one of two replies - or takes their leave.
//!   Agreeing warms them a little; pushing back cools them.
//! - House meetings: what the attendees say is recorded on the panel as
//!   it's said (the meeting's own talk comes from conversation::run_meeting).
//!
//! The speakers are read off the world: their Personality traits, their
//! trade, what's on them right now (sick, hungry, drunk, grieving, old,
//! a child), and the bond between the two.

use crate::engine::ecs::{Entity, World};
use crate::game::conversation::{converse, converse_with, Act, Choice, DialogueTree, Ending, Relation, Rng as TalkRng, Speaker, Vibe};
use crate::game::dialogue_trees::village_library;
use crate::game::lifecycle::{Age, LifeStage};
use crate::game::relationships::{self, Interaction, InteractionKind, RelKind};
use crate::game::traits::Personality;
use crate::game::villager::{self, Hunger};
use crate::game::vocation::{Craft, Kid, TownRole, Vocation};
use crate::sim::psychology::{AttachmentHistory, EpisodeKind};
use super::model::{ConvoPanelV, TalkPanelV, TalkRowV};
use super::Game;

/// Conversations kept in the talk history (oldest drop first)
pub const TALK_LOG_MAX: usize = 240;
/// The longest villager-to-villager exchange recorded
pub const CHAT_MAX_LINES: usize = 6;
/// The children's tree: a pair of kids keeps to it
pub const KID_TREE: &str = "children_at_play";
/// Walk further than this (tiles) from whoever you're talking with and
/// the talk is over - you've left
pub const PLAYER_TALK_RANGE: f32 = 6.0;
/// A reply not given inside this many days and they've gone back to work
pub const PLAYER_TALK_DAYS: u64 = 8;
/// What a reply does to the bond: agreeing warms, pushing back cools
const AGREE_WARMTH: f32 = 0.3;
const PUSHBACK_CHILL: f32 = 0.2;
/// How much a good answer to their audition warms the bond (#1187) -
/// a heavier beat than a single agreeable word
const VIBE_WARMTH: f32 = 0.5;
/// How long they hold how the last talk ended (#1183)
const TALK_MEMORY_DAYS: u64 = 20;

/// How the last talk with them left off, as they remember it: the picks
/// the player made, last one loudest (#1183)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TalkMood {
    /// A kind answer, an honest reply, a friendly parting
    Warmed,
    /// Words spent but nothing kindled - or they were walked away from
    Cool,
    /// Talked back at, or shaken off mid-word
    Slighted,
}

/// Carried on the villager: how the last sit-down ended, so the next one
/// opens where it left off rather than from scratch
#[derive(Clone, Copy, Debug)]
pub struct TalkMemory {
    pub mood: TalkMood,
    /// The day it was set; past TALK_MEMORY_DAYS it fades
    pub tick: u64,
}

/// How many times they've been pressed to talk today (#1226): the tally
/// is the villager's, not any seat's - anyone's asking counts, and it
/// lifts at dawn like the routine does
#[derive(Clone, Copy, Debug)]
pub struct TalkTired {
    /// The day the tally's for - yesterday's count doesn't carry
    pub day: u64,
    /// Talks opened today, the current one counted
    pub n: u8,
}

/// What kind of talk a record holds
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TalkKind {
    /// Two villagers in passing
    Chat,
    /// The player's Talk with a villager
    Talk,
    /// A house meeting
    Meeting,
}

/// One line on the record. `who` None is narration ("You take your leave.")
#[derive(Debug, Clone)]
pub struct TalkLine {
    pub who: Option<Entity>,
    pub name: String,
    pub text: String,
    /// The line that matters: a concern or secret drawn out
    pub gist: bool,
}

/// One conversation, as the TALK panel shows it
#[derive(Debug, Clone)]
pub struct TalkRecord {
    pub id: u64,
    pub tick: u64,
    pub kind: TalkKind,
    /// Everyone in it (the player too, for a Talk or a meeting)
    pub with: Vec<Entity>,
    pub lines: Vec<TalkLine>,
    /// The warmth beat was counted - a record only warms once
    pub warmed: bool,
    /// A kind answer landed - a Talk warms on those, not push-backs
    pub positive: bool,
}

/// A Talk the player is in the middle of: replayed from its seed and the
/// replies picked so far each time a reply is picked (the framework runs a
/// conversation start to end; replaying a seeded run up to the next
/// choice is how it's paced across frames)
#[derive(Debug, Clone)]
pub struct PlayerTalk {
    pub id: u64,
    pub who: Entity,
    pub villager: Speaker,
    pub player: Speaker,
    pub rel: Relation,
    pub seed: u64,
    pub picks: Vec<usize>,
    /// Who they are, read ahead of what's on their mind
    pub profile: String,
    /// What's on their mind, said after their opening line
    pub gist: String,
    /// The replies on offer now: (act, the line)
    pub options: Vec<(Act, String)>,
    /// Which tree's fork offered them - a greeting on old_scores is no
    /// greeting
    pub opt_tree: Option<&'static str>,
    pub since: u64,
    /// Narration after the gist ("You talk the helplessness out of them.")
    pub notes: Vec<String>,
    /// Propositions on offer after the replies: the earned asks the
    /// bond reaches, the coin's priced variants, the nerve's pressed
    /// ones (#1225)
    pub asks: Vec<crate::game::organizing::AskOffer>,
    /// Asks already put this talk - a vibe re-roll mustn't deal them
    /// back in (#1188)
    pub asks_taken: Vec<crate::game::organizing::AskOffer>,
    /// Which of today's pressings this talk is (#1226): the first runs
    /// full, the second flat, the third's near talked out already
    pub tired: u8,
    /// Topics from `Transcript.asked` already checked for a draw - the
    /// replay accumulates them, the watermark marks what's been seen
    /// (#1189)
    pub asked_seen: usize,
    /// How the picks so far have left them feeling (#1183)
    pub mood: TalkMood,
    /// How their audition of you read, once it's been answered (#1187)
    pub vibe: Option<crate::game::conversation::Vibe>,
}

/// The most choices a Talk offers at once - the replies plus however
/// many asks still fit. The list stops at three.
pub const TALK_MAX_CHOICES: usize = 3;

impl PlayerTalk {
    /// How many asks fit beside the replies on offer now: the choice
    /// list is the replies first, then asks, capped at TALK_MAX_CHOICES.
    /// `asks` itself keeps the whole list - an ask that doesn't fit this
    /// turn can still come up on a later one.
    pub(crate) fn asks_shown(&self) -> usize {
        TALK_MAX_CHOICES.saturating_sub(self.options.len()).min(self.asks.len())
    }
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
    if world.get_component::<Kid>(who).is_some() { s.status.push("child"); return s; }
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
    // and how far their faith has grown - a mystic's words carry
    // differently from a literalist's
    if let Some(t) = world.get_component::<crate::sim::religion::FaithStage>(who)
        .and_then(|st| st.title()) {
        s.status.push(t);
    }
    // one in five keeps the old ways - the meeting's rite takes their
    // words, not the church's (#1185)
    if super::routine::keeps_old_ways(world, who) { s.status.push("old_ways"); }
    // and the powers they're of - the talk can name what pulls on them
    for e in world.query::<crate::sim::factions::Faction>() {
        let Some(f) = world.get_component::<crate::sim::factions::Faction>(e) else { continue };
        if crate::sim::factions::members_of(world, f).contains(&who) {
            s.status.push(crate::sim::factions::fact_of(f.kind));
        }
    }
    // and what the life's made of them: the soul's not a score anyone
    // reads, but a marked one carries a name the talk can take up (#950)
    if let Some(vs) = world.get_component::<crate::game::virtue::Virtues>(who) {
        use crate::game::virtue::Virtue::*;
        let words = [(Charity, "kind_soul"), (Justice, "upright"), (Temperance, "steady"),
            (Prudence, "shrewd"), (Faith, "devout"), (Hope, "hopeful"), (Courage, "bold")];
        if vs.get(Charity) <= 0.30 { s.status.push("hard"); }
        else if let Some(&(_, w)) = words.iter()
            .max_by(|a, b| vs.get(a.0).partial_cmp(&vs.get(b.0)).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(k, _)| vs.get(*k) >= 0.65) {
            s.status.push(w);
        }
    }
    s
}

/// The bond between two villagers, in the conversation's terms. Two
/// villagers who've never spoken are still neighbours - RelKind::Stranger
/// only means no interactions yet, and the trees' stranger material is
/// for travellers on the road ("who art thou?"), not the miller's wife
pub fn relation_of(kind: Option<RelKind>) -> Relation {
    match relation_with_wanderer(kind) {
        Relation::Stranger => Relation::Acquaintance,
        r => r,
    }
}

/// The bond between a villager and the player, who really is the
/// stranger on the road until they've talked
pub fn relation_with_wanderer(kind: Option<RelKind>) -> Relation {
    match kind {
        Some(RelKind::Family | RelKind::Lover) => Relation::Kin,
        Some(RelKind::Friend) => Relation::Friend,
        Some(RelKind::Rival) => Relation::Rival,
        Some(RelKind::Acquaintance | RelKind::Mentor | RelKind::Apprentice) => Relation::Acquaintance,
        Some(RelKind::Stranger) | None => Relation::Stranger,
    }
}

/// Replay a Talk: the lines so far (speaker 0 the villager, 1 the
/// player), the replies on offer if it's the player's turn, and how it
/// ended if it has
fn replay(t: &PlayerTalk) -> (Vec<(usize, String)>, Option<Vec<(Act, String)>>, Option<Ending>, Option<&'static str>, Option<Vibe>, Vec<&'static str>) {
    let mut n = 0;
    let mut offered: Option<Vec<(Act, String)>> = None;
    let tr = converse_with(&t.villager, &t.player, t.rel, library(), 1, &mut |ch: &[Choice]| {
        if n < t.picks.len() {
            n += 1;
            Some(t.picks[n - 1].min(ch.len().saturating_sub(1)))
        } else {
            offered = Some(ch.iter().map(|c| (c.act, c.preview.clone())).collect());
            None
        }
    }, &mut TalkRng::new(t.seed));
    let lines = tr.turns.iter().map(|x| (x.speaker, x.text.clone())).collect();
    let ending = if offered.is_some() { None } else { tr.ending };
    (lines, offered, ending, tr.trees_walked.last().copied(), tr.vibe, tr.asked)
}

impl Game {
    pub(in crate::session) fn push_talk(&mut self, kind: TalkKind, with: Vec<Entity>, lines: Vec<TalkLine>) -> u64 {
        self.talk_next_id += 1;
        let id = self.talk_next_id;
        self.talk_log.push(TalkRecord { id, tick: self.tick, kind, with, lines, warmed: false, positive: false });
        if self.talk_log.len() > TALK_LOG_MAX {
            let extra = self.talk_log.len() - TALK_LOG_MAX;
            self.talk_log.drain(..extra);
        }
        self.warm_exchange(id);
        id
    }

    fn talk_record_mut(&mut self, id: u64) -> Option<&mut TalkRecord> {
        self.talk_log.iter_mut().rev().find(|r| r.id == id)
    }

    /// A real exchange between two is a warmth beat for both - only a
    /// chat or a Talk counts it, and only once both have spoken
    fn warm_exchange(&mut self, id: u64) {
        let Some(r) = self.talk_log.iter().find(|r| r.id == id) else { return };
        if r.warmed || r.with.len() != 2
            || matches!(r.kind, TalkKind::Meeting) { return }
        // a Talk warms on a kind answer, not a push-back
        if matches!(r.kind, TalkKind::Talk) && !r.positive { return }
        let (a, b) = (r.with[0], r.with[1]);
        if !(r.lines.iter().any(|l| l.who == Some(a))
            && r.lines.iter().any(|l| l.who == Some(b))) { return }
        let r = self.talk_record_mut(id).unwrap();
        r.warmed = true;
        let is_talk = matches!(r.kind, TalkKind::Talk);
        for m in [a, b] {
            crate::sim::psychology::record_episode(&mut self.world, m,
                crate::sim::psychology::Episode { with: if m == a { b } else { a },
                    kind: crate::sim::psychology::EpisodeKind::Warmth });
        }
        // a Talk sat down to is a real 1-1 - it deepens into a tie beyond
        // the warmth of the moment
        if is_talk {
            // a second sit-down before the rolls run: the first's depth
            // still lands before this one takes the slot
            if let Some(m) = self.world.get_component::<crate::sim::community::OneOnOne>(a) {
                let (w, d) = (m.with, m.depth);
                crate::game::relationships::record_interaction(&mut self.world, a,
                    crate::game::relationships::Interaction { other: w,
                        kind: crate::game::relationships::InteractionKind::Conversation, intensity: d });
            }
            self.world.add_component(a, crate::sim::community::OneOnOne { with: b, depth: 0.7 });
            // and the one doing the talking learns the craft of it
            crate::game::org_skills::practice(&mut self.world, a,
                crate::game::org_skills::OrgSkill::OneOnOne,
                crate::game::org_skills::MEETING_PRACTICE);
        }
    }

    /// Add a line to a conversation on the record (a meeting, as it's said)
    pub fn note_talk(&mut self, id: u64, who: Option<Entity>, text: &str) {
        let name = who.map(|w| self.speaker_name(w)).unwrap_or_default();
        if let Some(r) = self.talk_record_mut(id) {
            r.lines.push(TalkLine { who, name, text: text.to_string(), gist: false });
        }
    }

    /// Start a house meeting's record: everyone who came, and the host
    pub fn open_meeting_record(&mut self, host: Entity, came: &[Entity]) -> u64 {
        let mut with = vec![host];
        with.extend_from_slice(came);
        self.push_talk(TalkKind::Meeting, with, vec![])
    }

    /// A meeting already talked through (a campaign meeting): put the
    /// whole exchange on the record at once. (speaker, name, text); a None
    /// speaker is the room together - named, but nobody in particular
    pub fn record_meeting(&mut self, with: Vec<Entity>, said: Vec<(Option<Entity>, String, String)>) {
        if said.is_empty() { return; }
        let lines = said.into_iter().map(|(who, name, text)| TalkLine { who, name, text, gist: false }).collect();
        self.push_talk(TalkKind::Meeting, with, lines);
    }

    /// A campaign's talk - a meeting's deliberation, or the arc that
    /// follows a march, a strike or a townhall - goes on the TALK panel in
    /// the order it was said, and the exchange that settled it hangs over
    /// the two who said it. The other side speaks as "The hall", the room
    /// together as "The room".
    pub fn record_campaign_talk(&mut self, with: Vec<Entity>, d: &crate::game::campaign_dialogue::Deliberation) {
        use crate::game::campaign_dialogue::Voice;
        let said: Vec<(Option<Entity>, String, String)> = d.said.iter().map(|s| {
            let name = match (s.speaker, s.voice) {
                (Some(e), _) => self.speaker_name(e),
                (None, Voice::Opponent) => "The hall".to_string(),
                (None, _) => "The room".to_string(),
            };
            (s.speaker, name, s.text.clone())
        }).collect();
        self.record_meeting(with, said);
        for s in d.decisive() {
            if let Some(e) = s.speaker { self.say(e, &s.text, 4); }
        }
    }

    /// The name a line is kept under: a player's own name (each seat's
    /// panel shows its own lines as "You" - talk_panel_at), else the villager's
    pub(in crate::session) fn speaker_name(&self, e: Entity) -> String {
        match self.world.get_component::<crate::game::player::PlayerCharacter>(e) {
            Some(p) if self.seats.iter().any(|s| s.player == e) => p.name.clone(),
            _ => self.name_of(e),
        }
    }

    /// Two villagers meet: they hold a conversation, which goes on the
    /// record (not over the map). Children talk among themselves - a
    /// child and a grown-up just nod. Returns whether they talked.
    pub fn villagers_chat(&mut self, a: Entity, b: Entity) -> bool {
        if a == b { return false; }
        let (na, nb) = (self.name_of(a), self.name_of(b));
        let mut sa = speaker_of(&self.world, a, &na, self.tick);
        let mut sb = speaker_of(&self.world, b, &nb, self.tick);
        let kids = (sa.status.contains(&"child"), sb.status.contains(&"child"));
        if kids.0 != kids.1 { return false; }
        // two members of one cause carry its register in the street too
        // (#1228): the org they both belong to puts the fact on them both
        let shared_cause = self.world.query::<crate::sim::community::Organization>().iter().any(|&o|
            self.world.get_component::<crate::sim::community::Organization>(o)
                .map(|g| g.members.contains(&a) && g.members.contains(&b)).unwrap_or(false));
        if shared_cause {
            sa.status.push("cause_member");
            sb.status.push("cause_member");
        }
        // a pair the table knows plays their signature scene instead of
        // small talk (#1227): depth where the village has a story, and
        // the take it plays moves with the bond - then a fortnight's
        // quiet before the scene's their own again
        const SCENE_COOLDOWN: u64 = 14;
        if !kids.0 {
            let key = (a.min(b), a.max(b));
            let fresh = self.scene_marks.get(&key).map(|t| self.tick >= t + SCENE_COOLDOWN).unwrap_or(true);
            if fresh {
                if let Some((_scene, variant, order)) =
                    crate::game::scenes::scene_for(&self.world, a, b, self.tick) {
                    let lines: Vec<TalkLine> = variant.lines.iter().take(CHAT_MAX_LINES).map(|&(i, text)| {
                        let (who, name) = if i == 0 { (order.0, self.name_of(order.0)) }
                            else { (order.1, self.name_of(order.1)) };
                        TalkLine { who: Some(who), name, text: text.to_string(), gist: false }
                    }).collect();
                    self.scene_marks.insert(key, self.tick);
                    self.push_talk(TalkKind::Chat, vec![a, b], lines);
                    return true;
                }
            }
        }
        let rel = relation_of(self.rel_kind(a, b));
        // seeded off the meeting itself, not the game's rng: chatter on the
        // street mustn't shift every other roll in the sim
        let seed = self.tick.wrapping_mul(0x9E37_79B9)
            ^ ((a as u64) << 32) ^ (b as u64) ^ self.last_chat.len() as u64;
        let tr = converse(&sa, &sb, rel, library(), &mut TalkRng::new(seed));
        let lines: Vec<TalkLine> = tr.turns.iter()
            // kids keep to the children's tree: the rest is grown-up talk
            .take_while(|t| !kids.0 || t.tree == KID_TREE)
            .take(CHAT_MAX_LINES)
            .map(|t| {
                let (who, name) = if t.speaker == 0 { (a, na.clone()) } else { (b, nb.clone()) };
                TalkLine { who: Some(who), name, text: t.text.clone(), gist: false }
            }).collect();
        if lines.is_empty() { return false; }
        self.push_talk(TalkKind::Chat, vec![a, b], lines);
        true
    }

    /// The player has just talked with `v` (Talk's own effects are done):
    /// open the conversation on the TALK panel - their opening line, then
    /// `gist` (what Talk drew out of them), then the player's replies.
    /// `rel` is the bond as it stood before this talk.
    pub fn open_player_talk(&mut self, seat: usize, v: Entity, rel: Relation, gist: &str) {
        let pe = self.seats[seat].player;
        let rare = self.rare.iter().find(|r| r.e == Some(v)).map(|r| r.id);
        let name = rare.map(|id| super::events::spec(id).name.to_string())
            .unwrap_or_else(|| self.name_of(v));
        let mut villager = speaker_of(&self.world, v, &name, self.tick);
        if let Some(id) = rare {
            // a visitor has no villager life to read: the facts come off
            // the event - its kind, its archetype, two seed-rolled quirks,
            // and whether you've already spoken this stay (#1168)
            villager.status.push("visitor");
            villager.status.push(super::events::visitor_archetype(id));
            if let Some(s) = super::events::visit_slug(id) { villager.status.push(s); }
            const QUIRKS: [crate::game::traits::Trait; 8] = [
                crate::game::traits::Trait::Kind, crate::game::traits::Trait::Proud,
                crate::game::traits::Trait::Gregarious, crate::game::traits::Trait::Hotheaded,
                crate::game::traits::Trait::Ambitious, crate::game::traits::Trait::Stingy,
                crate::game::traits::Trait::Pious, crate::game::traits::Trait::Lazy,
            ];
            let q = super::routine::mix(v, 0x51ED);
            villager.traits.push(QUIRKS[(q % 8) as usize]);
            let q2 = (q / 8) % 8;
            if q2 != q % 8 { villager.traits.push(QUIRKS[q2 as usize]); }
            // spoken before, this stay: a Talk with him is already on the
            // record (act_talk's stamp can't be the tell - it lands first)
            if self.talk_log.iter().any(|r| matches!(r.kind, TalkKind::Talk)
                && r.with.contains(&v)) {
                villager.status.push("spoken");
            }
        }
        if rare.is_none() {
            // what they've told you opens the "trouble they told" ask;
            // what they still keep opens the deeper one (#1189)
            if self.world.get_component::<crate::game::desires::Known>(v)
                .map(|k| !k.revealed.is_empty()).unwrap_or(false) {
                villager.status.push("shared");
            }
            if !crate::game::desires::unrevealed(&self.world, v, self.tick).is_empty() {
                villager.status.push("keeps_secret");
            }
            // a live mystery is its own ask: what folk say of it (#1189)
            if self.seats[seat].mystery.as_ref().map(|m| !m.solved()).unwrap_or(false) {
                villager.status.push("mystery");
            }
        }
        // how well they know you colours the talk - the trees key on the
        // stage as a status fact
        match crate::game::organizing::stage_of(&self.world, pe, v) {
            // a first hello is still the street's weather: this talk
            // itself already sits on the bond's tally, so Acquaintance
            // only earns the neighbor register once they've met before
            crate::game::organizing::RelStage::Acquaintance => {
                let met_before = crate::game::relationships::find_relationship(&self.world, pe, v)
                    .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                    .map(|r| r.interactions > 1).unwrap_or(false);
                if met_before { villager.status.push("acquainted"); }
            }
            crate::game::organizing::RelStage::Known => villager.status.push("known"),
            crate::game::organizing::RelStage::Follower => villager.status.push("follower"),
            crate::game::organizing::RelStage::CoLeader => villager.status.push("coleader"),
            _ => {}
        }
        // the seat's own wears the cause's colors too (#1228): the fact
        // lands on their Speaker so the cause can key lines off it
        if self.world.get_component::<crate::sim::community::Organization>(self.seats[seat].org)
            .map(|o| o.members.contains(&v)).unwrap_or(false) {
            villager.status.push("cause_member");
        }
        let mut asks = crate::game::organizing::ask_offers(&self.world, self.seats[seat].org, v, 0);
        // pressing is the act (#1226): the tally counts this talk the
        // moment it opens, whoever's seat asked - and it lifts at dawn
        let tired = {
            let so_far = self.world.get_component::<TalkTired>(v)
                .filter(|t| t.day == self.tick).map(|t| t.n).unwrap_or(0);
            let t = TalkTired { day: self.tick, n: so_far.saturating_add(1) };
            self.world.add_component(v, t);
            t.n
        };
        // they won't hear a proposition from you while the last talk
        // still smarts - a cold ending closes what they'll offer (#1183);
        // a pressed villager keeps them to themselves just the same (#1226)
        if self.talk_mood_of(v) == Some(TalkMood::Slighted) || tired >= 2 { asks.clear(); }
        let pname = self.world.get_component::<crate::game::player::PlayerCharacter>(pe)
            .map(|p| p.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| "friend".into());
        // spoken to, "The Wanderer" is "wanderer": "A peaceful day to thee, wanderer."
        let pname = match pname.strip_prefix("The ") { Some(rest) => rest.to_lowercase(), None => pname };
        let player = Speaker::new(&pname);
        let seed = self.tick.wrapping_mul(0x2545_F491) ^ ((v as u64) << 24) ^ self.talk_next_id;
        self.close_player_talk(seat, None);
        // they stop where they are for the talk: a walk under way would
        // carry them off mid-line, and no new one dispatches while it's
        // open (in_player_talk gates the sends and the walk systems)
        self.travels.retain(|t| t.who != v);
        self.world.add_component(v, villager::InPlayerTalk);
        let id = self.push_talk(TalkKind::Talk, vec![pe, v], vec![]);
        // a visitor's standing is the road's own blurb, not a villager
        // backstory built off statuses it doesn't have (#1168)
        let profile = rare.map(|i| super::events::visitor_blurb(i).to_string())
            .unwrap_or_else(|| crate::game::dialogue_trees::backstory(&villager));
        self.seats[seat].talk = Some(PlayerTalk {
            id, who: v, villager, player, rel, seed, picks: vec![],
            profile,
            gist: gist.to_string(),
            options: vec![], opt_tree: None, since: self.tick, notes: vec![], asks,
            asks_taken: vec![], asked_seen: 0, mood: TalkMood::Cool, vibe: None,
            tired,
        });
        self.refresh_player_talk(seat);
    }

    /// Re-run the seat's Talk up to now and put it on the record; once
    /// it's over (a farewell, a walk-off, nothing left to say) close it
    fn refresh_player_talk(&mut self, seat: usize) {
        let Some(t) = self.seats[seat].talk.clone() else { return };
        let pe = self.seats[seat].player;
        let (said, offered, ending, tree, vibe, asked) = replay(&t);
        let vname = t.villager.name.clone();
        // the audition's read, the first time it's answered (#1187): a
        // good answer warms them and opens the deep material; a miss
        // leaves it polite - no propositions for someone you don't vibe
        if let (None, Some(v)) = (t.vibe, vibe) {
            let safe = v == Vibe::Safe;
            let note = if safe {
                format!("A good answer. {vname} warms to you - the talk goes deeper.")
            } else {
                format!("{vname} smiles, polite, and keeps it to the weather.")
            };
            // the read moves the asks a rung (#1188): someone who took
            // you as safe hears a deeper ask early; a miss keeps what
            // they offer slow - anything already put stays spent
            let mut asks = crate::game::organizing::ask_offers(
                &self.world, self.seats[seat].org, t.who, if safe { 1 } else { -1 });
            asks.retain(|o| !t.asks_taken.iter().any(|d| d.kind == o.kind && d.key == o.key));
            if let Some(cur) = self.seats[seat].talk.as_mut() {
                cur.vibe = Some(v);
                cur.asks = asks;
                if safe { cur.mood = TalkMood::Warmed; }
            }
            if safe {
                // safe folk warm faster - the audition carries more
                // weight than a single kind word
                relationships::record_interaction(&mut self.world, pe,
                    Interaction { other: t.who, kind: InteractionKind::Conversation, intensity: VIBE_WARMTH });
            }
            self.narrate_talk(seat, &note);
            return;
        }
        let mut lines: Vec<TalkLine> = vec![];
        for (k, (sp, text)) in said.iter().enumerate() {
            let (who, name) = if *sp == 0 { (t.who, vname.clone()) } else { (pe, self.speaker_name(pe)) };
            lines.push(TalkLine { who: Some(who), name, text: text.clone(), gist: false });
            if k == 0 { lines.extend(Self::gist_lines(&t, &vname)); }
        }
        if said.is_empty() { lines.extend(Self::gist_lines(&t, &vname)); }
        if let Some(r) = self.talk_record_mut(t.id) { r.lines = lines; }
        // a question that draws (#1189): asking after their trouble is
        // the way a kept secret comes out - the same reveal a talk's
        // opening rolls for; and asking after the whisper pulls a piece
        // of the mystery, the way the gossip line does
        for label in asked.iter().skip(t.asked_seen) {
            use crate::game::dialogue_trees::Draw;
            let draw = crate::game::dialogue_trees::ASK_TOPICS.iter()
                .find(|tp| tp.label == *label).map(|tp| tp.draws).unwrap_or(Draw::No);
            match draw {
                Draw::No => {}
                Draw::Secret => {
                    let pick = self.rng.next();
                    if let Some(r) = crate::game::desires::next_reveal(&self.world, t.who, self.tick, pick) {
                        let (did, said_line) = crate::game::desires::reveal_speech(&vname, &r);
                        if let Some(issue) = crate::game::desires::hear(&mut self.world, pe, t.who, r, self.tick) {
                            use crate::session::gamelog::{EvTag, Sev, Thread, Valence};
                            self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                            self.log_entity_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                                pe, format!("t{}: Organizer discovers issue {}", self.tick, issue));
                        }
                        if let Some(rec) = self.talk_record_mut(t.id) {
                            rec.lines.push(TalkLine { who: None, name: String::new(), text: did.clone(), gist: false });
                            rec.lines.push(TalkLine { who: Some(t.who), name: vname.clone(), text: said_line, gist: false });
                        }
                        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.notes.push(did); }
                    }
                }
                Draw::Mystery => {
                    if let Some(fact) = self.mystery_fact_asked(seat) {
                        if let Some(rec) = self.talk_record_mut(t.id) {
                            rec.lines.push(TalkLine { who: Some(t.who), name: vname.clone(), text: fact, gist: false });
                        }
                        if let Some(m) = &self.seats[seat].mystery {
                            let def = &crate::game::mysteries::MYSTERIES[m.def];
                            let note = format!("A piece of {} - {} of {} heard. Worth remembering.",
                                def.title, m.known.count_ones(), def.facts.len());
                            if let Some(cur) = self.seats[seat].talk.as_mut() { cur.notes.push(note); }
                        }
                    }
                }
            }
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.asked_seen = asked.len(); }
        self.warm_exchange(t.id);
        // the talk's spent when every reply left is an exit and there's no
        // ask to hang on (#1179): they take their leave as much as you do,
        // so close on a mutual parting rather than an exit menu
        if let Some(opts) = &offered {
            if !opts.is_empty() && t.asks.is_empty() && !t.picks.is_empty()
                && opts.iter().all(|(a, _)| *a == Act::Farewell) {
                let note = format!("The talk runs its course. You and {vname} part on a kind word.");
                // a spent talk parts kindly - unless the last word chosen
                // was a slight, which is what they keep (#1183)
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    if cur.mood != TalkMood::Slighted { cur.mood = TalkMood::Warmed; }
                }
                self.close_player_talk(seat, Some(&note));
                return;
            }
        }
        // the third pressing of a day finds them nearly talked out
        // (#1226): a word or two is all they still have - then the
        // parting, on the talk's own say-so. A real ending under way
        // (a walk-off) still lands as it would
        if t.tired >= 3 && !t.picks.is_empty()
            && matches!(&offered, Some(o) if !o.is_empty()) {
            self.close_player_talk(seat,
                Some(&format!("You and {vname} run out of things to say, and part.")));
            return;
        }
        match offered {
            Some(opts) if !opts.is_empty() => {
                // a pressed talk comes one word at a time (#1226): the
                // flat of the day shows as a shorter list, not new lines
                let opts = if t.tired >= 2 { opts.into_iter().take(1).collect() } else { opts };
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.options = opts; cur.opt_tree = tree; }
            }
            _ => {
                // a fresh talk that opens with no replies on offer would
                // hang up the moment it started (#1058): keep it open on
                // the leave alone, so the player's the one to end it.
                // A real ending (a walk-off) still closes as before.
                if t.picks.is_empty() && ending.is_none() {
                    if let Some(cur) = self.seats[seat].talk.as_mut() {
                        cur.options = vec![(Act::Farewell, "I take my leave.".into())];
                        cur.opt_tree = tree;
                    }
                    return;
                }
                let note = match ending {
                    Some(Ending::WalksOff) => Some(format!("{} turns and walks off.", vname)),
                    Some(Ending::Exhausted) => Some(format!("You and {vname} run out of things to say, and part.")),
                    _ => None,
                };
                // shaken off or talked dry: the ending is what they keep -
                // a walk-off is its own slight; anything else just keeps
                // the mood the picks already set (#1183)
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    if matches!(ending, Some(Ending::WalksOff)) { cur.mood = TalkMood::Slighted; }
                }
                self.close_player_talk(seat, note.as_deref());
            }
        }
    }

    /// A note on the seat's Talk record ("You talk the helplessness out of
    /// them."), kept through later replays
    pub fn narrate_talk(&mut self, seat: usize, text: &str) {
        let Some(t) = self.seats[seat].talk.as_mut() else { return };
        t.notes.push(text.to_string());
        self.refresh_player_talk(seat);
    }

    /// What Talk drew out of them, and any narration that came of it
    fn gist_lines(t: &PlayerTalk, vname: &str) -> Vec<TalkLine> {
        let mut v = vec![];
        if !t.profile.is_empty() {
            v.push(TalkLine { who: None, name: String::new(), text: t.profile.clone(), gist: false });
        }
        // a pressed-again villager has no fresh mind to speak (#1226):
        // the gist's already been said today, and they say so
        if t.tired >= 2 {
            v.push(TalkLine { who: Some(t.who), name: vname.to_string(),
                text: "Still here, then.".into(), gist: false });
        } else if !t.gist.is_empty() {
            v.push(TalkLine { who: Some(t.who), name: vname.to_string(), text: t.gist.clone(), gist: true });
        }
        v.extend(t.notes.iter().map(|n| TalkLine { who: None, name: String::new(), text: n.clone(), gist: false }));
        v
    }

    /// What a sworn villager opens a Talk with (#1228): the cause's own
    /// weather, not the sky's - live org state where their stage earns
    /// it. A real secret or a whispered clue outranks it; so does no
    /// cause at all.
    pub(in crate::session) fn cause_gist(&self, seat: usize, v: Entity) -> Option<(String, String)> {
        use crate::game::organizing::{MeetingPlan, RelStage};
        let pe = self.seats[seat].player;
        if !matches!(crate::game::organizing::stage_of(&self.world, pe, v),
            RelStage::Follower | RelStage::CoLeader) { return None; }
        let plan = self.world.get_component::<MeetingPlan>(pe);
        let issue = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)
            .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c))
            .map(|c| crate::game::desires::issue_name(c.issue.clone()));
        let line = if let Some(d) = plan.and_then(|p| p.for_day) {
            let when = match d.saturating_sub(self.tick) {
                0 => "today".to_string(),
                1 => "tomorrow".to_string(),
                n => format!("in {} days", n),
            };
            match &issue {
                Some(i) => format!("{} won't wait - we gather {}.", i, when),
                None => format!("We gather {} - my door's ready.", when),
            }
        } else if plan.map(|p| !p.holders().is_empty()).unwrap_or(false) {
            let n = plan.map(|p| p.holders().len()).unwrap_or(0);
            match &issue {
                Some(i) => format!("{} - we've {} promised already.", i, n),
                None => format!("We've {} promised for the next meet.", n),
            }
        } else if let Some(i) = issue {
            format!("{} sits on folk's minds - I keep it in mine too.", i)
        } else if self.world.get_component::<crate::sim::community::Organization>(self.seats[seat].org)
            .map(|o| o.members.iter().filter(|&&m| !villager::is_dead(&self.world, m)).count() > 1)
            .unwrap_or(false) {
            "We stand ready - say the word and we move.".to_string()
        } else { return None; };
        Some((format!("{} talks the cause", self.name_of(v)), line))
    }

    /// How the last sit-down with them ended, while it's still fresh
    /// (#1183): the next talk opens where the last one left off
    pub(in crate::session) fn talk_mood_of(&self, who: Entity) -> Option<TalkMood> {
        self.world.get_component::<TalkMemory>(who)
            .filter(|m| self.tick < m.tick + TALK_MEMORY_DAYS)
            .map(|m| m.mood)
    }

    /// End the seat's Talk, with a closing note on the record if given.
    /// Whoever it was with lets go of the pick once it's over (#944),
    /// and remembers how it ended (#1183)
    pub fn close_player_talk(&mut self, seat: usize, note: Option<&str>) {
        let Some(t) = self.seats[seat].talk.take() else { return };
        self.world.add_component(t.who, TalkMemory { mood: t.mood, tick: self.tick });
        // they're their own again - unless another seat still holds them
        if !self.seats.iter().any(|s| s.talk.as_ref().map(|k| k.who == t.who).unwrap_or(false)) {
            self.world.remove_component::<villager::InPlayerTalk>(t.who);
        }
        if self.villagers.get(self.seats[seat].selected_target) == Some(&t.who) {
            self.seats[seat].picked = false;
            self.seats[seat].pick_near = false;
        }
        if let Some(n) = note {
            if let Some(r) = self.talk_record_mut(t.id) {
                r.lines.push(TalkLine { who: None, name: String::new(), text: n.to_string(), gist: false });
            }
        }
    }

    /// The player picks reply `pick` of those on offer, or None to take
    /// their leave
    pub fn talk_pick_at(&mut self, seat: usize, pick: Option<usize>) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let (who, n, asks, first_pick) = (t.who, t.options.len(), t.asks_shown(), t.picks.is_empty());
        let Some(i) = pick.filter(|&i| i < n + asks) else {
            if pick.is_none() { self.close_player_talk(seat, Some("You take your leave.")); }
            return;
        };
        // a pick past the replies is a proposition: the ask lands or it
        // doesn't, and the word of it goes on the record
        if i >= n {
            let o = t.asks[i - n];
            let (pe, oe, name) = (self.seats[seat].player, self.seats[seat].org, t.villager.name.clone());
            let roll = (self.rng.next() % 1000) as f32 / 1000.0;
            let (msg, named) = crate::game::organizing::propose_keyed(
                &mut self.world, pe, oe, who, o.kind, &name, roll, self.tick, o.key, o.price);
            // an introduction's name is the one to look at: select them
            // and ask the camera over
            if let Some(f) = named {
                if let Some(i) = self.villagers.iter().position(|&v| v == f) {
                    self.seats[seat].selected_building = None;
                    self.seats[seat].selected_target = i;
                    self.seats[seat].picked = true;
                    self.seats[seat].pick_near = self.dist_to_at(seat, i) <= super::SHOUT_DIST;
                }
                if let Some(pos) = self.world.get_component::<villager::Pos>(f) {
                    self.seats[seat].focus = Some((pos.x, pos.y, self.frame_n));
                }
            }
            self.say(who, &msg, 5);
            self.narrate_talk(seat, &msg);
            if let Some(cur) = self.seats[seat].talk.as_mut() {
                cur.asks.retain(|a| *a != o);
                cur.asks_taken.push(o);
            }
            self.refresh_player_talk(seat);
            return;
        }
        let act = t.options[i].0;
        let pe = self.seats[seat].player;
        // an audition pick isn't read by its act - standing up to a
        // hothead is the right answer, not a slight; the vibe read it
        // sets is the whole effect (#1187)
        let probing = t.opt_tree == Some(crate::game::conversation::PROBE_TREE);
        // how you answer them counts for a little either way - and it
        // lingers: the last pick's mood is what they carry out (#1183)
        match act {
            _ if probing => {}
            Act::PushBack => { relationships::record_interaction(&mut self.world, pe,
                Interaction { other: who, kind: InteractionKind::Conflict, intensity: PUSHBACK_CHILL });
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Slighted; } }
            Act::Agree | Act::Answer => {
                relationships::record_interaction(&mut self.world, pe,
                    Interaction { other: who, kind: InteractionKind::Conversation, intensity: AGREE_WARMTH });
                if let Some(r) = self.talk_record_mut(t.id) { r.positive = true; }
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Warmed; }
            }
            // a kind word back or a friendly leave-taking is warmth too -
            // but a rival's greeting and farewells aren't kind ones
            Act::Greet | Act::Farewell if t.opt_tree != Some("old_scores") => {
                if let Some(r) = self.talk_record_mut(t.id) { r.positive = true; }
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Warmed; }
            }
            _ => {}
        }
        // a visitor's first answer carries the deal: Agree is "I'll take
        // it", PushBack on the darker kinds sends them down the road
        // (#1168)
        if first_pick {
            if let Some(ri) = self.rare.iter().position(|r| r.e == Some(who)) {
                let id = self.rare[ri].id;
                match act {
                    Act::Agree => self.apply_visitor_offer(seat, who, id),
                    Act::PushBack if super::events::visitor_dismissible(id) => {
                        let n = format!("{} takes your meaning and heads for the road.",
                            super::events::spec(id).name);
                        self.narrate_talk(seat, &n);
                        self.depart_event(ri);
                    }
                    _ => {}
                }
            }
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.picks.push(i); cur.since = self.tick; }
        self.refresh_player_talk(seat);
    }

    /// "I'll take it" with a road visitor: the purse pays or gains, the
    /// satchel takes the ware, and the note goes on the talk's record
    /// (#1168)
    fn apply_visitor_offer(&mut self, seat: usize, who: Entity, id: super::events::EventId) {
        use crate::game::livelihood::{Purse, Satchel};
        let o = super::events::visitor_offer(id);
        let pe = self.seats[seat].player;
        if o.gold_out > 0.0 {
            let gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
            if gold < o.gold_out {
                self.narrate_talk(seat, "Your purse comes up short - the deal dies there.");
                return;
            }
            if let Some(purse) = self.world.get_component_mut::<Purse>(pe) {
                purse.gold -= o.gold_out;
            }
        }
        if o.gold_in > 0.0 {
            if let Some(purse) = self.world.get_component_mut::<Purse>(pe) {
                purse.gold += o.gold_in;
            }
        }
        if let Some(g) = o.good {
            if let Some(sat) = self.world.get_component_mut::<Satchel>(pe) {
                *sat.goods.entry(g.to_string()).or_insert(0) += 1;
            } else {
                let mut sat = Satchel::default();
                sat.goods.insert(g.to_string(), 1);
                self.world.add_component(pe, sat);
            }
        }
        if o.stay_extra > 0 {
            if let Some(r) = self.rare.iter_mut().find(|r| r.e == Some(who)) {
                r.until = r.until.map(|u| u + o.stay_extra);
            }
        }
        if !o.note.is_empty() { self.narrate_talk(seat, o.note); }
    }

    /// Whether `who` is in a Talk with a seat's player right now: while
    /// it is, they take no walks - a conversation can't hold at a
    /// distance, so a travel or the pull home doesn't carry them off
    /// mid-line
    pub(in crate::session) fn in_player_talk(&self, who: Entity) -> bool {
        self.world.get_component::<villager::InPlayerTalk>(who).is_some()
    }

    /// Each frame: a Talk left hanging ends once the player walks away,
    /// they're taken off the map, or too long passes without a reply
    pub fn advance_player_talks(&mut self) {
        for seat in 0..self.seats.len() {
            let Some(t) = self.seats[seat].talk.as_ref() else { continue };
            let who = t.who;
            let stale = self.tick > t.since + PLAYER_TALK_DAYS;
            let gone = villager::is_dead(&self.world, who) || crate::game::dungeon::unavailable(&self.world, who);
            let far = match (self.world.get_component::<villager::Pos>(who), self.world.get_component::<villager::Pos>(self.seats[seat].player)) {
                (Some(p), Some(q)) => (p.x - q.x).powi(2) + (p.y - q.y).powi(2) > PLAYER_TALK_RANGE * PLAYER_TALK_RANGE,
                _ => true,
            };
            if gone { self.close_player_talk(seat, None); }
            else if far { self.close_player_talk(seat, Some("You walk on.")); }
            else if stale { let n = format!("{} goes back to their work.", self.name_of(who)); self.close_player_talk(seat, Some(&n)); }
        }
    }

    /// The TALK panel for a seat: the picked villager's conversations
    /// (with you or anyone), or else your own - Talk and meetings - with
    /// the replies on offer if you're mid-Talk
    pub fn talk_panel_at(&self, seat: usize) -> TalkPanelV {
        let s = &self.seats[seat];
        let picked = (s.picked && s.selected_building.is_none())
            .then(|| self.villagers.get(s.selected_target).copied()).flatten();
        let focus = picked.unwrap_or(s.player);
        let title = match picked { Some(v) => format!("TALK - {}", self.name_of(v)), None => "TALK".to_string() };
        let mut rows: Vec<TalkRowV> = vec![];
        let mut shown: Vec<u64> = vec![];
        // newest conversations last, as many as could ever fit
        for r in self.talk_log.iter().rev().filter(|r| r.with.contains(&focus) && !r.lines.is_empty()).take(4) {
            shown.push(r.id);
            let mut block: Vec<TalkRowV> = vec![TalkRowV { name: None, text: format!("day {}", r.tick), gist: false, you: false, rule: true, who: None }];
            block.extend(r.lines.iter().map(|l| TalkRowV {
                // your own lines read "You"; another seat's keep their name
                name: if l.who == Some(s.player) { Some("You".into()) } else { (!l.name.is_empty()).then(|| l.name.clone()) },
                text: l.text.clone(), gist: l.gist,
                you: l.who.map(|w| w == s.player).unwrap_or(false), rule: false,
                who: l.who.and_then(|e| self.villagers.iter().position(|v| *v == e)),
            }));
            block.extend(rows);
            rows = block;
        }
        let in_talk = matches!(&s.talk, Some(t) if shown.contains(&t.id));
        let options = match &s.talk {
            Some(t) if in_talk => t.options.iter().map(|o| o.1.clone())
                .chain(t.asks.iter().take(t.asks_shown()).map(|a| crate::game::organizing::ask_label(a))).collect(),
            _ => vec![],
        };
        let empty = if picked.is_some() { "Nothing heard from them yet." } else { "Talk to someone (T) and it shows here." };
        if rows.is_empty() { rows.push(TalkRowV { name: None, text: empty.into(), gist: false, you: false, rule: false, who: None }); }
        TalkPanelV { title, rows, options, in_talk }
    }

    /// A talk record's lines as conversation-box rows: "You" leads the
    /// seat's own lines; the speaker carries their villager index so a
    /// click on the line selects them
    fn record_rows(&self, seat: usize, id: u64) -> Vec<TalkRowV> {
        let Some(r) = self.talk_log.iter().find(|r| r.id == id) else { return vec![] };
        let pe = self.seats[seat].player;
        r.lines.iter().map(|l| TalkRowV {
            name: if l.who == Some(pe) { Some("You".into()) } else { (!l.name.is_empty()).then(|| l.name.clone()) },
            text: l.text.clone(), gist: l.gist,
            you: l.who == Some(pe), rule: false,
            who: l.who.and_then(|e| self.villagers.iter().position(|v| *v == e)),
        }).collect()
    }

    /// The conversation box for a seat, top-right of the map: the founding
    /// call a house meeting left open first, then a meeting under way,
    /// then the seat's Talk
    pub fn convo_panel_at(&self, seat: usize) -> Option<ConvoPanelV> {
        let s = &self.seats[seat];
        if self.vax_offer.is_some() {
            return Some(ConvoPanelV {
                title: "THE VACCINATION ACT".into(),
                rows: vec![
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "The Act's officer reads the order on the square: every infant takes the lancet's scratch, and the refusers pay the fine.".into() },
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "A third way's whispered in the crowd - the Leicester method: the word cried, the sick kept apart, contacts held, houses cleansed. Your call.".into() },
                ],
                options: vec![
                    "Let the Act stand - the infants take the scratch".into(),
                    "Refuse it - the refusers pay, and the league forms".into(),
                    "The Leicester way - the village holds its own line".into(),
                ],
                decides: true, leave: false });
        }
        let cultish = self.world.get_component::<crate::sim::religion::Congregation>(self.congregation)
            .map(|c| c.cultish).unwrap_or(false);
        if cultish && self.proposal.is_none() && self.seat_offer.is_none()
            && self.emergency.is_none() && self.dilemma.is_none() {
            return Some(ConvoPanelV {
                title: "THE TIGHTENED FOLD".into(),
                rows: vec![TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "The fold's own cage tightens - a soul or two want out.".into() }],
                options: vec![
                    "Shelter the one who wants out".into(),
                    "Bring the outside word in - the milieu breaks".into(),
                    "Call the lord's hand on it - the coercion".into(),
                ],
                decides: true, leave: false });
        }
        if self.counter_intel >= 2 && self.proposal.is_none() && self.seat_offer.is_none()
            && self.emergency.is_none() && (s.talk.is_none() || seat == 0)
            && self.counter_played != Some(self.tick) {
            let mut options = vec!["Feed the watcher false word - the eyes read what we show".into()];
            if self.counter_intel >= 3 {
                options.push("Read the watcher out - his own papers in the open".into());
            }
            return Some(ConvoPanelV {
                title: "THE ORG'S OWN EYES".into(),
                rows: vec![TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "The cause's own eyes read the lord's - counter the watching.".into() }],
                options, decides: true, leave: false });
        }
        if self.emergency.is_some() {
            let rows = vec![
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "The lord's man calls it the cause's own brawl - the square's banned while the tale stands.".into() },
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "Gather the witnesses before the tale's just how things are.".into() },
            ];
            return Some(ConvoPanelV {
                title: "THE LORD'S EMERGENCY".into(),
                rows, options: vec!["Gather the witnesses".into()],
                decides: true, leave: false });
        }
        if self.proposal.is_some() && self.plant.is_some() {
            let options = vec![
                "Marshal the discipline - the cause stays honest".into(),
                "Verify first - ask who put them to it".into(),
                "Throw them out - suspicion enough".into(),
            ];
            let rows = vec![
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "One of them's for the torch - burn the lord's stores, they say, now.".into() },
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "Answer it and the cause reads as honest or as the lord's own tale. Your call.".into() },
            ];
            return Some(ConvoPanelV {
                title: "THE VIOLENT WORD".into(),
                rows, options, decides: true, leave: false });
        }
        if let Some(o) = &self.seat_offer {
            let lord = self.name_of(o.from);
            let options = vec![
                "Take the seat - sit at their board".into(),
                "Refuse it - the cause stays the streets".into(),
                "Ask what it really carries".into(),
            ];
            let rows = vec![
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: format!("{}'s man brings an offer: a seat on the board.", lord) },
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: "Take it and the cause sits at their table - or at their mercy. Your call.".into() },
            ];
            return Some(ConvoPanelV {
                title: "THE LORD'S OFFER".into(),
                rows, options, decides: true, leave: false });
        }
        if s.meet_offer.is_some() {
            return Some(ConvoPanelV {
                title: "THE CALL - WHEN DO THEY GATHER".into(),
                rows: vec![
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "The plan's made - the word just wants a day.".into() },
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "Till then the asks still count - roles and yeses fill until the door.".into() },
                ],
                options: vec![
                    "Two days out - a quick word round".into(),
                    "Four days out - the word gets round".into(),
                    "A week out - the far ones can come".into(),
                ],
                decides: true, leave: false });
        }
        if let Some(d) = &s.pending_founding {
            let mut rows = self.record_rows(seat, d.record);
            let (lean, options) = match &d.vote {
                Some(v) => {
                    let issue = crate::game::desires::issue_name(v.winner.clone());
                    (format!("The room leans to {} - your call.", issue), vec![
                        format!("Take it on, {} - a burden shared", issue),
                        format!("Take it on, {} - and name the foe", issue),
                        format!("Take it on, {} - a duty the village owes", issue),
                        "Let it rest - keep talking".into(),
                    ])
                }
                None => {
                    let mut opts = vec![];
                    if !d.meet_again.is_empty() { opts.push("Meet again - they'll come back".into()); }
                    if d.street { opts.push("Take it to the street - show the numbers".into()); }
                    opts.push("Let it rest - keep talking".into());
                    ("They'd gather again - or take it further. Your call.".to_string(), opts)
                }
            };
            rows.push(TalkRowV { name: None, gist: true, you: false, rule: false, who: None, text: lean });
            return Some(ConvoPanelV {
                title: format!("{} - YOUR CALL", d.org_name.to_uppercase()),
                rows, options, decides: true, leave: false });
        }
        // the campaign's agenda slate: the motions the cause's size can
        // carry tonight, then "just meet" - the street verbs live here
        // now, voted on the floor, not on a button (#1157)
        if let Some((motions, _)) = &s.pending_agenda {
            let oname = self.world.get_component::<crate::sim::community::Organization>(s.org)
                .map(|o| o.name.clone()).unwrap_or_default();
            let n = crate::game::campaign::campaign_of(&self.world, s.org)
                .map(|c| crate::game::campaign::living_members(&self.world, c)).unwrap_or(0);
            let mut options: Vec<String> = motions.iter()
                .map(|&m| crate::game::campaign::motion_label(m)).collect();
            options.push("Just meet - the resolves, no action".into());
            return Some(ConvoPanelV {
                title: format!("{} - THE FLOOR'S YOURS", oname.to_uppercase()),
                rows: vec![TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                    text: format!("{} stand with the cause - what do they take up?", n) }],
                options, decides: true, leave: false });
        }
        if let Some(m) = &self.meeting {
            if m.seat == seat {
                let mut rows = self.record_rows(seat, m.talk_id);
                if m.talk_from.is_none() {
                    rows.push(TalkRowV { name: None, gist: false, you: false, rule: false, who: None,
                        text: "folk are on their way".into() });
                }
                return Some(ConvoPanelV {
                    title: format!("HOUSE MEETING - {}", m.label.to_uppercase()),
                    rows, options: vec![], decides: false, leave: false });
            }
        }
        if let Some(t) = &s.talk {
            return Some(ConvoPanelV {
                title: format!("TALK - {}", t.villager.name.to_uppercase()),
                rows: self.record_rows(seat, t.id),
                options: t.options.iter().map(|o| o.1.clone())
                    .chain(t.asks.iter().take(t.asks_shown()).map(|a| crate::game::organizing::ask_label(a))).collect(),
                decides: false, leave: true });
        }
        // the traveller's ask (#1085): last on the box - a live talk, a
        // meeting under way, every real decision all outrank a pedlar
        if self.seats[seat].survey.is_some() {
            return Some(ConvoPanelV {
                title: "THE TRAVELLER ASKS".into(),
                rows: self.survey_rows(seat),
                options: crate::session::survey::SURVEY_OPTIONS.iter().map(|s| s.to_string()).collect(),
                decides: true, leave: false });
        }
        None
    }
}
