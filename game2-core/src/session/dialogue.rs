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
/// What arguing a case your cause doesn't hold quietly costs (#1234)
const INSINCERE_CHILL: f32 = 0.1;
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
    /// The exchange ran cold (#1230) - a spat is no warmth beat
    pub cold: bool,
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
    /// An argument under way (#1234): while it is, the choices are its
    pub argue: Option<Argument>,
    /// What the argument put on the record, each after the replay line
    /// it followed - the replay rewrites the record, these ride along
    pub argued: Vec<(usize, TalkLine)>,
}

/// The one-on-one argument as it stands (#1234): which issue, then
/// which side, then the answer to their counter
#[derive(Debug, Clone)]
pub struct Argument {
    pub issues: Vec<crate::game::desires::IssueTag>,
    pub issue: Option<crate::game::desires::IssueTag>,
    pub stance: Option<crate::game::persuasion::Stance>,
}

impl Argument {
    pub fn labels(&self) -> Vec<String> {
        use crate::game::persuasion::{move_label, Stance, MOVES};
        let name = |t: &crate::game::desires::IssueTag| crate::game::desires::issue_name(t.clone());
        match (&self.issue, self.stance) {
            (None, _) => self.issues.iter().map(|t| format!("Argue {}", name(t)))
                .chain(std::iter::once("Let it drop".to_string())).collect(),
            (Some(t), None) => [Stance::For, Stance::Against].iter().map(|s| match s {
                Stance::For => format!("Argue for taking up {}", name(t)),
                Stance::Against => format!("Argue against stirring up {}", name(t)),
            }).chain(std::iter::once("Let it drop".to_string())).collect(),
            (Some(_), Some(_)) => MOVES.iter().map(|m| move_label(*m).to_string()).collect(),
        }
    }
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

    /// The choice list as shown: an argument's own steps while one's
    /// under way, else the replies then the asks that fit
    pub fn choice_labels(&self) -> Vec<String> {
        if let Some(a) = &self.argue { return a.labels(); }
        self.options.iter().map(|o| o.1.clone())
            .chain(self.asks.iter().take(self.asks_shown()).map(crate::game::organizing::ask_label)).collect()
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

/// The trade word a vocation speaks with in a meeting (#1238)
fn trade_word(v: Vocation) -> &'static str {
    match v {
        Vocation::Farmer => "farmer",
        Vocation::Baker => "baker",
        Vocation::Toolmaker => "smith",
        Vocation::Knight => "knight",
        Vocation::Tailor => "tailor",
        Vocation::Glazier => "glazier",
        Vocation::Printmaker => "printer",
        Vocation::Mason => "mason",
        Vocation::Priest => "priest",
        Vocation::Luthier => "luthier",
        Vocation::Shepherd => "shepherd",
        Vocation::Beekeeper => "beekeeper",
        Vocation::Weaver => "weaver",
        Vocation::Perfumer => "perfumer",
        Vocation::Painter => "painter",
        Vocation::Shoemaker => "shoemaker",
        Vocation::Bookbinder => "bookbinder",
        Vocation::Miner => "miner",
        Vocation::Fisher => "fisher",
        Vocation::Alewife => "alewife",
        Vocation::Brewer => "brewer",
        Vocation::Vintner => "vintner",
        Vocation::Distiller => "distiller",
        Vocation::Cooper => "cooper",
        Vocation::Taverner => "taverner",
    }
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
    s.trade = craft.map(trade_word);
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
        self.push_talk_cold(kind, with, lines, false)
    }

    /// push_talk, with the exchange's temp (#1230): a cold chat doesn't
    /// count a warmth beat in warm_exchange
    pub(in crate::session) fn push_talk_cold(&mut self, kind: TalkKind, with: Vec<Entity>, lines: Vec<TalkLine>, cold: bool) -> u64 {
        self.talk_next_id += 1;
        let id = self.talk_next_id;
        self.talk_log.push(TalkRecord { id, tick: self.tick, kind, with, lines, warmed: false, positive: false, cold });
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
        if r.warmed || r.cold || r.with.len() != 2
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
            // but a re-talk the same day doesn't re-arm it (#1232): the
            // pair's beat already spent, the sit-down's depth stays what
            // the first talk armed
            if !crate::game::player::pick_spent_elsewhere(&self.world, b, a, self.tick, id) {
                self.world.add_component(a, crate::sim::community::OneOnOne { with: b, depth: 0.7 });
            }
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
                    // the take moves with the bond (#1230): a feuding
                    // pair's scene is a spat, anyone else's a warm one
                    let temp = if self.rel_kind(a, b) == Some(RelKind::Rival) { -1.0 } else { 0.6 };
                    self.push_talk_cold(TalkKind::Chat, vec![a, b], lines, temp <= super::SPAT_TEMP);
                    self.chat_lands(a, b, temp, a);
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
        // the exchange types what it does to the bond (#1230): the acts
        // and the close read warm or cold before the lines go on record
        let temp = crate::game::conversation::chat_temp(&tr);
        // who threw the sharp words, for the feud's tale if it comes to one
        let spat_by = {
            let pa = tr.turns.iter().filter(|t| t.speaker == 0 && t.act == Act::PushBack).count();
            let pb = tr.turns.iter().filter(|t| t.speaker == 1 && t.act == Act::PushBack).count();
            if pa >= pb { a } else { b }
        };
        let lines: Vec<TalkLine> = tr.turns.iter()
            // kids keep to the children's tree: the rest is grown-up talk
            .take_while(|t| !kids.0 || t.tree == KID_TREE)
            .take(CHAT_MAX_LINES)
            .map(|t| {
                let (who, name) = if t.speaker == 0 { (a, na.clone()) } else { (b, nb.clone()) };
                TalkLine { who: Some(who), name, text: t.text.clone(), gist: false }
            }).collect();
        if lines.is_empty() { return false; }
        self.push_talk_cold(TalkKind::Chat, vec![a, b], lines, temp <= super::SPAT_TEMP);
        self.chat_lands(a, b, temp, spat_by);
        true
    }

    /// The chat's acts land on the bond (#1230): a kind talk warms as it
    /// was one-sided, a spat cools it - the pair's own bent leans on the
    /// landing a little, and past the Friend mark street talk only keeps
    /// the bond warm, it doesn't deepen it. A spat goes on the record as
    /// a quarrel beat, and quarrels piling up while the bond's already
    /// cold open a feud.
    pub(in crate::session) fn chat_lands(&mut self, a: Entity, b: Entity, temp: f32, spat_by: Entity) {
        use crate::session::gamelog::{EvTag, Sev, Thread, Valence};
        let bias = crate::game::conversation::pair_bias(a, b);
        let cold = temp <= super::SPAT_TEMP;
        let mut inten = if cold {
            // a real quarrel cools more than a sharp word or two; the
            // pair's bent means their slights land lighter (warm pair)
            // or their kindnesses thinner (cold pair), a quarter either way
            (0.15 + 0.25 * temp.abs()).min(0.4) * (1.0 - 0.25 * bias)
        } else {
            super::CHAT_INTENSITY * (0.6 + 0.4 * temp.max(0.0)) * (1.0 + 0.25 * bias)
        };
        if !cold {
            let s = crate::game::relationships::find_relationship(&self.world, a, b)
                .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                .map(|r| r.strength).unwrap_or(0.0);
            // acquaintance is as deep as a street chat takes it: friends
            // nod and stay friends, the deepening wants a shared table
            if s >= super::CHAT_FRIEND_CAP { inten *= 0.2; }
        }
        relationships::record_interaction(&mut self.world, a, Interaction {
            other: b,
            kind: if cold { InteractionKind::Conflict } else { InteractionKind::Conversation },
            intensity: inten,
        });
        if !cold { return; }
        let (na, nb) = (self.name_of(a), self.name_of(b));
        let at = super::toast_spot(&self.world, a);
        let who = self.villagers.iter().position(|&v| v == a);
        self.log.push_ev_at_who(
            EvTag::new(Thread::Other, Valence::Negative, Sev::Minor),
            format!("t{}: {} and {} exchanged sharp words", self.tick, na, nb),
            at, who);
        self.toasts.push_at((format!("{} and {} exchanged sharp words", na, nb), self.tick + 40), at);
        // the tally (#1230): this many spats in a window while the bond's
        // already cold opens a feud, the counter spent once it does
        let key = (a.min(b), a.max(b));
        let (n, first) = self.spats.get(&key).copied().unwrap_or((0, self.tick));
        let (n, first) = if self.tick > first + super::SPAT_WINDOW { (1, self.tick) } else { (n + 1, first) };
        self.spats.insert(key, (n, first));
        if n < super::SPAT_FEUD_N { return; }
        let bond = crate::game::relationships::find_relationship(&self.world, a, b)
            .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
            .map(|r| r.strength).unwrap_or(0.0);
        if bond > -0.1 { return; }
        let feuding = self.world.query::<crate::game::feuds::Feud>().iter().any(|&f| {
            self.world.get_component::<crate::game::feuds::Feud>(f).map(|x|
                !matches!(x.status, crate::game::feuds::FeudStatus::Resolved)
                    && x.offender.min(x.aggrieved) == key.0
                    && x.offender.max(x.aggrieved) == key.1).unwrap_or(false)
        });
        if feuding { return; }
        crate::game::feuds::spawn_feud(&mut self.world, spat_by,
            if spat_by == a { b } else { a }, "sharp words");
        self.spats.remove(&key);
        self.log.push_ev_at_who(
            EvTag::new(Thread::Other, Valence::Negative, Sev::Notable),
            format!("t{}: Bad blood between {} and {} - the quarrels have come to a feud", self.tick, na, nb),
            at, who);
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
            // the false traveller carries the motive's status: it keys
            // the asking lines that don't fit the pedlar's cry (#1258)
            if let Some(ft) = self.false_traveller {
                if ft.e == v {
                    villager.status.push(match ft.motive {
                        super::events::FalseMotive::Spy => "ft-spy",
                        super::events::FalseMotive::Thief => "ft-thief",
                        _ => "ft-runaway",
                    });
                }
            }
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
            // the sheep in the cradle (#1254): she'll tell you whose
            // pot it's in; the house that has it evades until a kindness
            // lifts the lid; and once the name's held the squeeze sits
            // in his talk until it's paid
            if let Some(t) = &self.ewe_theft {
                if !t.fork_open && !t.held {
                    if v == t.victim {
                        villager.status.push("ewe_victim");
                    } else {
                        let thome = crate::game::household::home_of(&self.world, t.thief);
                        if v == t.thief
                            || (thome.is_some()
                                && thome == crate::game::household::home_of(&self.world, v)) {
                            villager.status.push("ewe_secret");
                        }
                    }
                } else if t.held && v == t.thief {
                    villager.status.push("ewe_leverage");
                }
            }
            // the Noah row (#1257): while the row's still burning, a
            // Talk to either half of it is the mediator's opening
            if let Some(r) = &self.noah_row {
                if r.until == self.tick && (v == r.a || v == r.b) {
                    villager.status.push("rowing");
                }
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
            tired, argue: None, argued: vec![],
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
            if safe && !crate::game::player::pick_spent_today(&self.world, t.who, pe, self.tick) {
                // safe folk warm faster - the audition carries more
                // weight than a single kind word; but only where the
                // day's beat isn't already spent (#1232)
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
            lines.extend(t.argued.iter().filter(|(at, _)| *at == k + 1).map(|(_, l)| l.clone()));
        }
        if said.is_empty() { lines.extend(Self::gist_lines(&t, &vname)); }
        lines.extend(t.argued.iter().filter(|(at, _)| *at > said.len())
            .map(|(_, l)| l.clone()));
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
            if !opts.is_empty() && t.asks.is_empty() && !t.picks.is_empty() && t.argue.is_none()
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
        if let (Some(_), Some(i)) = (&t.argue, pick) { self.argue_pick_at(seat, i); return; }
        let (who, n, asks, first_pick) = (t.who, t.options.len(), t.asks_shown(), t.picks.is_empty());
        let Some(i) = pick.filter(|&i| i < n + asks) else {
            if pick.is_none() { self.close_player_talk(seat, Some("You take your leave.")); }
            return;
        };
        // a pick past the replies is a proposition: the ask lands or it
        // doesn't, and the word of it goes on the record
        if i >= n {
            let o = t.asks[i - n];
            if o.kind == crate::game::organizing::PropKind::Argue {
                self.open_argument(seat, o);
                return;
            }
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
        let pick_text = t.options[i].1.clone();
        let pe = self.seats[seat].player;
        let (tid, twho, old_scores, opt_tree) =
            (t.id, t.who, t.opt_tree == Some("old_scores"), t.opt_tree);
        // an audition pick isn't read by its act - standing up to a
        // hothead is the right answer, not a slight; the vibe read it
        // sets is the whole effect (#1187)
        let probing = t.opt_tree == Some(crate::game::conversation::PROBE_TREE);
        // how you answer them counts for a little either way - and it
        // lingers: the last pick's mood is what they carry out (#1183)
        // the day's first bond-moving pick lands the ledger beat; picks
        // after it still play their lines - the talk's the talk - but
        // they don't grind the bond (#1232)
        let spent = crate::game::player::pick_spent_today(&self.world, who, pe, self.tick);
        match act {
            _ if probing => {}
            Act::PushBack => {
                if !spent { relationships::record_interaction(&mut self.world, pe,
                    Interaction { other: who, kind: InteractionKind::Conflict, intensity: PUSHBACK_CHILL }); }
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Slighted; } }
            Act::Agree | Act::Answer => {
                if !spent { relationships::record_interaction(&mut self.world, pe,
                    Interaction { other: who, kind: InteractionKind::Conversation, intensity: AGREE_WARMTH }); }
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
        // a pick that can move the bond spends the day's beat, either
        // valence - a morning slight and an evening kindness don't both
        // count (#1232); a rival's cold greet moves nothing and spends
        // nothing, and an audition read isn't a pick at all
        let bond_moving = !probing && match act {
            Act::PushBack | Act::Agree | Act::Answer => true,
            Act::Greet | Act::Farewell => !old_scores,
            _ => false,
        };
        if bond_moving && !spent {
            crate::game::player::note_pick_moved(&mut self.world, who, pe, self.tick, tid);
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
        // the sheep in the cradle (#1254): the gift lifts the lid and
        // puts the fork on the card; the squeeze spends the held name;
        // the robbed one's answer names him in the record
        if opt_tree == Some(crate::game::dialogue_trees::EWE_TREE)
            && pick_text == crate::game::dialogue_trees::EWE_GIFT {
            self.ewe_cracked(seat);
        } else if opt_tree == Some(crate::game::dialogue_trees::EWE_PRICE_TREE)
            && pick_text == crate::game::dialogue_trees::EWE_SQUEEZE {
            self.ewe_squeezed(seat);
        } else if opt_tree == Some(crate::game::dialogue_trees::EWE_VICTIM_TREE)
            && pick_text == crate::game::dialogue_trees::EWE_WHOSE_POT {
            if let Some(th) = self.ewe_theft.as_ref().map(|t| t.thief) {
                let tn = self.name_of(th);
                self.narrate_talk(seat, &format!("She names him at last: {tn}."));
            }
        }
        // the Noah row (#1257): the mediator's word talks one down, or
        // the pick takes a side where the street can see it
        if opt_tree == Some(crate::game::dialogue_trees::NOAH_ROW_TREE) {
            if pick_text == crate::game::dialogue_trees::NOAH_PEACE {
                self.row_calmed(seat, twho);
            } else if pick_text == crate::game::dialogue_trees::NOAH_SIDE_THEM {
                self.row_sided(seat, twho);
            }
        }
        // the false traveller (#1258): naming the disguise puts the
        // plays on the card - say nothing, cry him out, or feed a story
        if matches!(opt_tree, Some(crate::game::dialogue_trees::FT_SPY_TREE)
            | Some(crate::game::dialogue_trees::FT_THIEF_TREE)
            | Some(crate::game::dialogue_trees::FT_RUN_TREE))
            && (pick_text == crate::game::dialogue_trees::FT_ACCUSE
                || pick_text == crate::game::dialogue_trees::FT_ACCUSE_RUN) {
            self.false_accused(seat);
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.picks.push(i); cur.since = self.tick; }
        self.refresh_player_talk(seat);
    }

    /// Put a line from the argument on the talk's record, after what's
    /// been said so far (#1234)
    fn argue_line(&mut self, seat: usize, who: Option<Entity>, text: String) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let at = replay(t).0.len().max(1);
        let name = match who { Some(w) if w == t.who => t.villager.name.clone(), Some(w) => self.speaker_name(w), None => String::new() };
        if let Some(cur) = self.seats[seat].talk.as_mut() {
            cur.argued.push((at, TalkLine { who, name, text, gist: false }));
        }
    }

    /// "Hear me out on something" (#1234): the argument opens on the live
    /// issues worth arguing - or, with none, there's nothing to argue
    fn open_argument(&mut self, seat: usize, o: crate::game::organizing::AskOffer) {
        let (pe, oe) = (self.seats[seat].player, self.seats[seat].org);
        let Some(t) = self.seats[seat].talk.as_mut() else { return };
        t.asks.retain(|a| *a != o);
        t.asks_taken.push(o);
        let who = t.who;
        let issues = crate::game::persuasion::arguable_issues(&self.world, oe, pe, who);
        if issues.is_empty() {
            self.argue_line(seat, None, "There's nothing live between you to argue just now.".into());
        } else if let Some(t) = self.seats[seat].talk.as_mut() {
            t.argue = Some(Argument { issues, issue: None, stance: None });
        }
        self.refresh_player_talk(seat);
    }

    /// A pick inside the argument (#1234): the issue, then the side -
    /// they answer it from their temper - then how you meet their
    /// answer, which resolves it. A won case sets them thinking; they
    /// come round days later (`persuasion::mature`). Arguing what your
    /// cause doesn't hold is allowed and quietly costs you with them.
    fn argue_pick_at(&mut self, seat: usize, i: usize) {
        use crate::game::persuasion::{self, Outcome, Stance, MOVES};
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let Some(a) = t.argue.clone() else { return };
        if i >= a.labels().len() { return; }
        let (who, vname) = (t.who, t.villager.name.clone());
        let (pe, oe) = (self.seats[seat].player, self.seats[seat].org);
        match (a.issue.clone(), a.stance) {
            (None, _) => {
                if i >= a.issues.len() { return self.drop_argument(seat); }
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    cur.argue = Some(Argument { issue: Some(a.issues[i].clone()), ..a });
                }
            }
            (Some(issue), None) => {
                if i >= 2 { return self.drop_argument(seat); }
                let stance = if i == 0 { Stance::For } else { Stance::Against };
                self.argue_line(seat, Some(pe), persuasion::case_line(stance, &issue));
                let traits = self.world.get_component::<Personality>(who).map(|p| p.traits.clone()).unwrap_or_default();
                self.argue_line(seat, Some(who), persuasion::counter_line(&traits, stance, &issue));
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    cur.argue = Some(Argument { stance: Some(stance), ..a });
                }
            }
            (Some(issue), Some(stance)) => {
                let m = MOVES[i];
                self.argue_line(seat, Some(pe), format!("You {}.", persuasion::move_label(m).to_lowercase()));
                let roll = (self.rng.next() % 1000) as f32 / 1000.0;
                let outcome = persuasion::resolve(&self.world, pe, who, m, roll);
                let mag = persuasion::magnitude(&self.world, pe, who, &issue, m, self.tick);
                persuasion::push_argument(&mut self.world, who, &issue, stance, outcome, mag, self.tick);
                // a case the cause doesn't hold leaves its mark - no word
                // of it, just a cooler bond
                if persuasion::insincere(&self.world, oe, pe, &issue, stance) {
                    relationships::record_interaction(&mut self.world, pe,
                        Interaction { other: who, kind: InteractionKind::Conflict, intensity: INSINCERE_CHILL });
                }
                let note = match outcome {
                    Outcome::Won => format!("{vname} turns it over. They'll think on it."),
                    Outcome::Stalemate => format!("{vname} shrugs. Neither of you moves the other."),
                    Outcome::Lost => format!("{vname}'s answer carries it - if anything, you've argued them the other way."),
                    Outcome::Pressed => format!("{vname} bristles - you've pushed too hard."),
                };
                if outcome == Outcome::Pressed {
                    if !crate::game::player::pick_spent_today(&self.world, who, pe, self.tick) {
                        relationships::record_interaction(&mut self.world, pe,
                            Interaction { other: who, kind: InteractionKind::Conflict, intensity: PUSHBACK_CHILL });
                        let tid = self.seats[seat].talk.as_ref().map(|t| t.id).unwrap_or(0);
                        crate::game::player::note_pick_moved(&mut self.world, who, pe, self.tick, tid);
                    }
                    if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Slighted; }
                }
                self.argue_line(seat, None, note);
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.argue = None; }
            }
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.since = self.tick; }
        self.refresh_player_talk(seat);
    }

    fn drop_argument(&mut self, seat: usize) {
        self.argue_line(seat, None, "You let it drop.".into());
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.argue = None; }
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
            Some(t) if in_talk => t.choice_labels(),
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
        if let Some((_, events)) = &s.testimony_offer {
            return Some(ConvoPanelV {
                title: "THE CALL - WHAT WILL YOU TELL".into(),
                rows: vec![
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "Tell them why you're here: a thing you lived through, then what this room shares, then what tonight asks.".into() },
                ],
                options: events.clone(),
                decides: true, leave: false });
        }
        if s.frame_offer.is_some() {
            let mut options: Vec<String> = vec![
                "Put the matter - what ails the village".into(),
                "Probe - what do you actually want, and for whom?".into(),
            ];
            if self.testimony_ready(seat) { options.push("Testify - tell them why you're here".into()); }
            return Some(ConvoPanelV {
                title: "THE CALL - WHAT WILL YOU ASK".into(),
                rows: vec![
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "The word's out. When they're in the room, what's the question?".into() },
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None,
                        text: "Put the matter and they'll talk the village's troubles. Ask what they want and you'll hear them - and it stays with them after.".into() },
                ],
                options,
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
                options: t.choice_labels(),
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
