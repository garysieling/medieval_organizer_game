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
use crate::game::vocation::{trade_word, Craft, Kid, TownRole, Vocation};
use crate::sim::psychology::{AttachmentHistory, EpisodeKind};
use super::model::{ConvoPanelV, TalkPanelV, TalkRowV};
use super::Game;

/// Conversations kept in the talk history (oldest drop first)
pub const TALK_LOG_MAX: usize = 240;
/// The longest villager-to-villager exchange recorded
pub const CHAT_MAX_LINES: usize = 6;

/// The help-ask's lines (#1368): one neighbor asks another for a hand
pub(crate) const HELP_ASKS: &[&str] = &[
    "{2:could} {2:subj} spare a hand at my fence come the morrow?",
    "My cart's wheel wants a second pair of hands - {2:could} {2:subj} come by?",
    "The roof wants patching before the rain - {2:would} {2:subj} lend a hand?",
    "The woodpile's behind and my back's not what it was - a hand, neighbor?",
];
pub(crate) const HELP_YES: &[&str] = &[
    "Aye, I'll come by.",
    "Gladly - look for me before the day's old.",
    "I'll be there, never fear.",
];
pub(crate) const HELP_NO: &[&str] = &[
    "I've no hands to spare, neighbor.",
    "Can't - my own work's piled past the door.",
    "Not this week - ask me again when the rain's past.",
];
/// The favor-ask (#1388): the talk turns on what the asked actually
/// makes - a fish for coin, a pot of honey for the child's throat. The
/// walk's the same help-out follow-through; the goods themselves stay
/// an economy matter, not a chat one
pub(crate) const FAVOR_ASKS: &[(crate::game::vocation::Vocation, &str)] = &[
    (crate::game::vocation::Vocation::Fisher, "{2:would} {2:subj} spare a fat trout for a penny? The pot's thin this week."),
    (crate::game::vocation::Vocation::Beekeeper, "{2:could} {2:subj} spare a pot of honey? My child's throat's raw with the cough."),
    (crate::game::vocation::Vocation::Weaver, "{2:have} {2:subj} a length of cloth off the loom for a fair price?"),
    (crate::game::vocation::Vocation::Baker, "{2:would} {2:subj} spare a loaf on tick till market day?"),
    (crate::game::vocation::Vocation::Alewife, "Could I trouble {2:obj} for a jug of {2:poss} small ale?"),
    (crate::game::vocation::Vocation::Brewer, "A firkin of {2:poss} beer for the haymaking - name the price."),
    (crate::game::vocation::Vocation::Vintner, "{2:have} {2:subj} a bottle of the good wine for a feast day?"),
    (crate::game::vocation::Vocation::Shepherd, "{2:would} {2:subj} spare a fleece or two? My spindle's idle."),
    (crate::game::vocation::Vocation::Farmer, "{2:could} {2:subj} spare a sack of grain till the harvest comes in?"),
    (crate::game::vocation::Vocation::Shoemaker, "{2:could} {2:subj} mend my boots? The sole's near through."),
    (crate::game::vocation::Vocation::Taverner, "A bowl of {2:poss} pottage for a hungry evening - what's the price?"),
    (crate::game::vocation::Vocation::Perfumer, "A vial of {2:poss} scent for the house - what {2:do} {2:subj} ask?"),
    (crate::game::vocation::Vocation::Distiller, "A dram of the water of life for a cold night?"),
];
/// A favor granted reads like one brought round, not one fetched
pub(crate) const FAVOR_YES: &[&str] = &[
    "Aye, I'll bring it by before dark.",
    "Come by my bench before dusk - I'll set it aside.",
    "For {2:obj}? Aye - look for it at {2:poss} door.",
];
/// The children's tree: a pair of kids keeps to it
pub const KID_TREE: &str = "children_at_play";
/// Walk further than this (tiles) from whoever you're talking with and
/// the talk is over - you've left
pub const PLAYER_TALK_RANGE: f32 = 6.0;
/// A reply not given inside this many days and they've gone back to work
pub const PLAYER_TALK_DAYS: u64 = 8;
/// A line spoken stays fresh in the village's ears this many days:
/// tree lines said this recently are passed over for fresher words (#1369)
pub const FRESH_LINE_DAYS: u64 = 3;
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
    /// The wrong was owned and heard (#1251): not warm - but the door's
    /// no longer shut the way a slight leaves it
    Guarded,
}

/// Carried on the villager: how the last sit-down ended, so the next one
/// opens where it left off rather than from scratch
#[derive(Clone, Copy, Debug)]
pub struct TalkMemory {
    pub mood: TalkMood,
    /// The day it was set; past TALK_MEMORY_DAYS it fades
    pub tick: u64,
}

/// What one seat's told them that landed (E3-3, #1428): keyed by the
/// seat's player entity -> (deepest disclosure reached, day it did) -
/// per-player, fading on the same clock as TalkMemory. A second seat
/// finds nothing of the first's telling
#[derive(Debug, Default)]
pub struct SharedDepth {
    pub of: crate::engine::det::HashMap<Entity, (u8, u64)>,
}

/// How many times `of` has made amends to the bearer (#1251): the
/// count outlives the 20-day memory - a sorry repeated too often stops
/// landing, and that wears longer than the slight it was for
#[derive(Clone, Copy, Debug)]
pub struct AmendsTally {
    pub of: Entity,
    pub n: u8,
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
    /// A meal at a household's table (#1241)
    Meal,
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
    /// The seat's own organizer (E7-1, #1404): their ledger is what the
    /// asks' "you said" and the deep draws' already-heard read from -
    /// another seat's hearing stays out of it
    pub knower: Entity,
    pub villager: Speaker,
    pub player: Speaker,
    pub rel: Relation,
    /// The finer bond, when the village knows it (#1372): lovers and
    /// mentors reach different words than acquaintances
    pub kind: Option<crate::game::relationships::RelKind>,
    pub seed: u64,
    pub picks: Vec<usize>,
    /// Who they are, read ahead of what's on their mind
    pub profile: String,
    /// What's on their mind, said after their opening line
    pub gist: String,
    /// The replies on offer now: (act, its responsiveness tag, the line)
    pub options: Vec<(Act, crate::game::conversation::Respond, String)>,
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
    /// Too-deep asks already paid for (E3-2): each one the replay
    /// reports anew costs the bond a rung of nerve
    pub forward_seen: u8,
    /// The deepest disclosure the seat's given them this talk (E3-3) -
    /// lands on their TalkMemory when the talk closes
    pub shared: u8,
    /// How the picks so far have left them feeling (#1183)
    pub mood: TalkMood,
    /// How their audition of you read, once it's been answered (#1187)
    pub vibe: Option<crate::game::conversation::Vibe>,
    /// An argument under way (#1234): while it is, the choices are its
    pub argue: Option<Argument>,
    /// What the argument put on the record, each after the replay line
    /// it followed - the replay rewrites the record, these ride along
    pub argued: Vec<(usize, TalkLine)>,
    /// The inoculation under way (#1246): while it is, the choices are
    /// the counters to name
    pub inoculating: bool,
    /// The solved mystery's telling, while this is the one who asked and
    /// it's still untold: rides past the cap, last, so it's never crowded
    /// off and the picks before it keep their numbers
    pub tell: Option<String>,
    /// The repair move on offer (#1251): the talk opened on a slight, so
    /// the last slot is owning the wrong
    pub amends: bool,
    /// The listener's trade (#1252): with a hog, the last slot is letting
    /// them run - the monologue yields their own story, never the village's
    pub let_run: bool,
    /// A job going begging rides the front of the picks while it's
    /// live (#1336): the offer's the reason you walked over, so it's the
    /// first choice and their opening names it. Latched at open - a
    /// mid-talk lapse refuses on the pick, never shifts the numbers
    pub job: bool,
    /// The meeting invite's follow-up (#1337): while it's open the
    /// choices are the call's - when they gather, then where
    pub inviting: Option<InviteStep>,
    /// The safety test a disclosure's waiting on (E5-4, #1436): while
    /// it rides, the choices are its answers - the secret holds until
    /// one's picked
    pub safety_bid: Option<SafetyBid>,
    /// A passed test this talk (#1436): the next secrets come out
    /// without asking again
    pub bid_passed: bool,
    /// The act the offered replies answer (E5-4): a pick read against
    /// it is what a bid met, missed or refused moves
    pub call_act: Option<Act>,
}

/// The invitation's follow-up under way (#1337): they said they'd come
/// to a meeting and asked the call's own questions back - when, then
/// where. While one's open its picks are the talk's choices.
#[derive(Debug, Clone)]
pub enum InviteStep {
    /// "When are we gathering, then?" - the day picks, the same three
    /// the call panel offers
    When,
    /// "And where d'you want us?" - the day's picked (`days` out); the
    /// venue list's already resolved with its labels: the public
    /// spots, the org's learned homes, and "their call" riding last
    Where { days: u64, venues: Vec<(crate::game::organizing::Venue, String)> },
}

/// A bid for safety (E5-4, #1436): the test a villager runs before a
/// real telling - Afifi & Steuber's three shapes. `reveal` is the
/// secret it was holding; passing buys it back out
#[derive(Debug, Clone, Copy)]
pub struct SafetyBid {
    pub kind: SafetyBidKind,
    pub reveal: crate::game::desires::Reveal,
}

/// The three tests: the hypothetical asks after "a man", the
/// confidentiality check asks for your tongue, the probe tells someone
/// else's lesser secret and watches what you do with it
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafetyBidKind { Hypothetical, Confidence, Probe }

impl SafetyBid {
    /// The answers on offer while the test stands: the turn toward,
    /// the sidestep, and the turn away or leak
    pub fn labels(&self) -> Vec<String> {
        match self.kind {
            SafetyBidKind::Hypothetical => vec![
                "Hear him out - every house has its reasons.".into(),
                "That's no business of mine.".into(),
                "The reeve should know of it.".into()],
            SafetyBidKind::Confidence => vec![
                "My tongue holds - say on.".into(),
                "That depends what's said.".into(),
                "I make no promises.".into()],
            SafetyBidKind::Probe => vec![
                "That's between you and me.".into(),
                "Why tell me this at all?".into(),
                "The reeve should hear it.".into()],
        }
    }

    /// Which turn answer `i` reads as for this shape: the last option
    /// leaks what was held (against) for the hypothetical and probe -
    /// the confidence check's cold end is only a miss
    fn turn_of(&self, i: usize) -> crate::game::relationships::BidTurn {
        use crate::game::relationships::BidTurn;
        match (self.kind, i) {
            (_, 0) => BidTurn::Toward,
            (SafetyBidKind::Hypothetical, 2) | (SafetyBidKind::Probe, 2) => BidTurn::Against,
            (SafetyBidKind::Confidence, 1) | (SafetyBidKind::Probe, 1) => BidTurn::Pivot,
            _ => BidTurn::Away,
        }
    }
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
    /// How many asks fit beside the picks on offer now: the choice
    /// list is the job's take-it first, then the replies, then asks,
    /// capped at TALK_MAX_CHOICES.
    /// `asks` itself keeps the whole list - an ask that doesn't fit this
    /// turn can still come up on a later one.
    /// The naming of the counter when the other asks crowd it off
    /// (#1246): it rides one past the cap, last, so the pick numbers of
    /// everything before it don't move
    pub(crate) fn inoculation_beyond(&self) -> Option<crate::game::organizing::AskOffer> {
        self.asks.iter().skip(self.asks_shown())
            .find(|a| a.kind == crate::game::organizing::PropKind::Inoculate).copied()
    }

    /// The probing question (#1269) rides past the cap with the
    /// inoculation, last of the beyond asks so the pick numbers of
    /// everything before it don't move
    pub(crate) fn probe_beyond(&self) -> Option<crate::game::organizing::AskOffer> {
        self.asks.iter().skip(self.asks_shown())
            .find(|a| a.kind == crate::game::organizing::PropKind::Probe).copied()
    }

    pub(crate) fn asks_shown(&self) -> usize {
        TALK_MAX_CHOICES.saturating_sub(self.options.len() + self.job as usize).min(self.asks.len())
    }

    /// The line `template` speaks as in this talk (E4-4): the offered
    /// and recorded strings are realized text, so matching a pick
    /// against an authored const means realizing the const with the
    /// pair patched the way `converse_with` patches it. `player_side`
    /// is the player's mouth; false is the villager's.
    pub fn said(&self, template: &'static str, player_side: bool) -> String {
        talk_said(&self.villager, &self.player, self.rel, template, player_side)
    }

    /// The choice list as shown: an argument's own steps while one's
    /// under way, else the replies then the asks that fit
    pub fn choice_labels(&self, world: &World) -> Vec<String> {
        if let Some(a) = &self.argue { return a.labels(); }
        // the invite's questions are the picks while it's open (#1337)
        if let Some(step) = &self.inviting {
            return match step {
                InviteStep::When => vec![
                    "Two days out - a quick word round".into(),
                    "Four days out - the word gets round".into(),
                    "A week out - the far ones can come".into(),
                ],
                InviteStep::Where { venues, .. } => venues.iter().map(|(_, l)| l.clone())
                    .chain(std::iter::once("Their call - they'll pick where".into())).collect(),
            };
        }
        // the safety test's answers, while it stands (E5-4): the
        // transcript's replies wait underneath until it's settled
        if let Some(b) = &self.safety_bid { return b.labels(); }
        if self.inoculating {
            return crate::game::inoculation::COUNTERS.iter().map(|&c| crate::game::inoculation::label(c).to_string())
                .chain(std::iter::once("Let it drop".to_string())).collect();
        }
        self.job.then(|| "About that job - I'll take it.".to_string()).into_iter()
            .chain(self.options.iter().map(|o| {
                // the words that pick a fight say so (#1348)
                if crate::game::persuasion::is_argumentative(&o.2)
                    || crate::game::persuasion::ARGUMENTATIVE_LINES.iter()
                        .any(|l| o.2 == self.said(l, true)) {
                    format!("{} - a quarrel", o.2)
                } else { o.2.clone() }
            }))
            .chain(self.asks.iter().take(self.asks_shown())
                .map(|o| crate::game::organizing::ask_label(world, self.knower, self.who, o)))
            .chain(self.inoculation_beyond().as_ref()
                .map(|o| crate::game::organizing::ask_label(world, self.knower, self.who, o)))
            .chain(self.probe_beyond().as_ref()
                .map(|o| crate::game::organizing::ask_label(world, self.knower, self.who, o)))
            .chain(self.tell.clone())
            .chain(self.amends.then(|| "Make amends for the other day".to_string()))
            .chain(self.let_run.then(|| "Let them run on".to_string())).collect()
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
    // the persona's fact table (E3-1, #1423): archetype, top values,
    // wound, lie and the tiers each area reaches
    if let Some(p) = world.get_component::<crate::game::persona::Persona>(who) {
        s.facts = p.fact_table();
        // the visible habits the wounds and lessons leave (E3-4, #1425):
        // neighbors' gossip and the ask gate's tell bonus both read them
        for t in p.tells() { s.status.push(t); }
        // a carried ghost is a kept thing - `keeps_wound` opens the
        // deep asks but wears no area of its own: the wound's own tell
        // slugs mark only the areas it actually hides behind
        if !p.ghosts.is_empty() { s.status.push("keeps_wound"); }
        // the ear's landmark (E4-7, #1416): their phrase and habit
        s.idiolect = Some(p.idiolect);
    }
    // the sky overhead (#1388): the day's weather rides as a status so
    // weather talk gates on what the day is actually doing - rain lines
    // only fall out of a wet sky
    let wx = world.query::<crate::sim::climate::Climate>().first()
        .and_then(|&e| world.get_component::<crate::sim::climate::Climate>(e))
        .and_then(crate::sim::precipitation::precip_kind);
    s.status.push(match wx {
        Some(k) if k.is_snow() => "wx-snow",
        Some(crate::sim::precipitation::PrecipKind::Haar) => "wx-fog",
        Some(_) => "wx-rain",
        None => "wx-fair",
    });
    // the voice's own weight in the room (#1249): bonds, seniority,
    // standing, the moral pull of a dead loss - read live, not stored
    s.pull = crate::game::organizing::pull_of(world, who);
    if let Some(p) = world.get_component::<Personality>(who) { s.traits = p.traits.clone(); }
    // the body the talk can pronoun by (E1-3, #1417) - set before the
    // child's early out so kids carry it too
    if let Some(sex) = world.get_component::<villager::Sex>(who) {
        s.sex = Some(match sex { villager::Sex::Female => "female", villager::Sex::Male => "male" });
    }
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
    // where they stand on the road out of helplessness - the same
    // issue reads resigned till their interest's connected (#1269)
    s.status.push(crate::game::desires::thaw_status(crate::game::desires::rung_of(world, who)));
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
    // an old wound colors their talk long before they'll name it (#1243)
    s.status.extend(crate::game::hooks::statuses(world, who));
    // their own archaism (E4-2, #1414): the devout and the old-ways keep
    // the full thou-pattern; the rest take thou only as a marked shift
    s.archaism = if s.ideology == "clergy"
        || s.traits.contains(&crate::game::traits::Trait::Pious)
        || s.status.iter().any(|st| *st == "old_ways")
        || s.facts.open.iter().any(|f| matches!(f.as_str(), "archetype:devout" | "archetype:old_guard")) { 2 } else { 1 };
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
/// The line `template` speaks as between this pair (E4-4): realized
/// with the speakers patched the way `converse_with` patches them, so
/// offered or recorded text can be matched against an authored const.
pub(in crate::session) fn talk_said(villager: &Speaker, player: &Speaker, rel: Relation,
             template: &str, player_side: bool) -> String {
    use crate::game::text::{realize, tv_for, Cx};
    let (mut v, mut p) = (villager.clone(), player.clone());
    v.rel = rel;
    v.tv = tv_for(&v, &p, rel, matches!(rel, Relation::Rival));
    p.rel = rel;
    p.tv = tv_for(&p, &v, rel, matches!(rel, Relation::Rival));
    let (speaker, listener) = if player_side { (&p, &v) } else { (&v, &p) };
    realize(template, &Cx::plain(speaker, listener, &[]),
        &mut crate::game::conversation::Rng::new(0))
}

fn replay(t: &PlayerTalk, recent: Option<&crate::engine::det::HashSet<&'static str>>) -> (Vec<(usize, String, Option<&'static str>)>, Option<Vec<(Act, crate::game::conversation::Respond, String)>>, Option<Ending>, Option<&'static str>, Option<Vibe>, Vec<&'static str>, u8, u8, Option<Act>) {
    let mut n = 0;
    let mut offered: Option<Vec<(Act, crate::game::conversation::Respond, String)>> = None;
    let tr = converse_with(&t.villager, &t.player, t.rel, t.kind, library(), recent, 1, &mut |ch: &[Choice]| {
        if n < t.picks.len() {
            n += 1;
            Some(t.picks[n - 1].min(ch.len().saturating_sub(1)))
        } else {
            offered = Some(ch.iter().map(|c| (c.act, c.respond, c.preview.clone())).collect());
            None
        }
    }, &mut TalkRng::new(t.seed));
    let lines = tr.turns.iter().map(|x| (x.speaker, x.text.clone(), x.line)).collect();
    let ending = if offered.is_some() { None } else { tr.ending };
    // the act the offered replies answer (E5-4): when the walk stopped
    // for a pick, the last turn is their call - a bid's met or missed
    // against it
    let call = if offered.is_some() { tr.turns.last().map(|t| t.act) } else { None };
    (lines, offered, ending, tr.trees_walked.last().copied(), tr.vibe, tr.asked, tr.forward, tr.shared, call)
}

impl Game {
    /// Tree lines the village has spoken lately - still fresh in ears (#1369)
    fn fresh_lines(&self) -> crate::engine::det::HashSet<&'static str> {
        self.said_lines.iter()
            .filter(|(_, day)| *day + FRESH_LINE_DAYS >= self.tick)
            .map(|(t, _)| *t).collect()
    }

    /// Mark the transcript's tree lines as spoken today, and let the
    /// village's memory of what's said fall off past the window
    fn note_said_lines(&mut self, lines: impl Iterator<Item = &'static str>) {
        for t in lines { self.said_lines.insert(t, self.tick); }
        let today = self.tick;
        self.said_lines.retain(|_, day| *day + FRESH_LINE_DAYS >= today);
    }

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

    pub(in crate::session) fn talk_record_mut(&mut self, id: u64) -> Option<&mut TalkRecord> {
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

    /// The line `text` as `who` speaks it to `pe` (E4-4): the ambient
    /// strings - greetings, remarks, the cold gist - speak in the
    /// pair's own register too
    pub(in crate::session) fn pair_said(&self, who: Entity, pe: Entity, text: &str) -> String {
        let rel = relation_with_wanderer(self.rel_kind(pe, who));
        talk_said(&speaker_of(&self.world, who, &self.name_of(who), self.tick),
                  &self.player_speaker(pe), rel, text, false)
    }

    /// A line said villager-to-villager outside a Talk (E4-4)
    pub(in crate::session) fn vv_said(&self, a: Entity, b: Entity, text: &str) -> String {
        let rel = relation_with_wanderer(self.rel_kind(a, b));
        talk_said(&speaker_of(&self.world, a, &self.name_of(a), self.tick),
                  &speaker_of(&self.world, b, &self.name_of(b), self.tick),
                  rel, text, false)
    }

    /// The player's own line aimed at the villager (E4-4)
    pub(in crate::session) fn player_line(&self, seat: usize, who: Entity, text: &str) -> String {
        let pe = self.seats[seat].player;
        let rel = relation_with_wanderer(self.rel_kind(pe, who));
        talk_said(&speaker_of(&self.world, who, &self.name_of(who), self.tick),
                  &self.player_speaker(pe), rel, text, true)
    }

    /// The player's Speaker for realizing lines aimed at them (E4-4):
    /// name only - a tree's own wiring layers on rhetoric status
    pub(in crate::session) fn player_speaker(&self, pe: Entity) -> Speaker {
        let pname = self.world.get_component::<crate::game::player::PlayerCharacter>(pe)
            .map(|p| p.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| "friend".into());
        // spoken to, "The Wanderer" is "wanderer": "A peaceful day to thee, wanderer."
        let pname = match pname.strip_prefix("The ") { Some(rest) => rest.to_lowercase(), None => pname };
        Speaker::new(&pname)
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
        // a spat still fresh between them can be patched over a passing
        // word (#1251): the kinder of the pair offers the amends, the
        // other answers by trait; a refused peace leaves the quarrel
        // standing. It preempts the signature scene - the quarrel's the
        // livelier story
        let key = (a.min(b), a.max(b));
        let spat_fresh = self.spats.get(&key)
            .map(|&(_, first)| self.tick <= first + super::SPAT_WINDOW).unwrap_or(false);
        if !kids.0 && spat_fresh {
            let seed = self.tick.wrapping_mul(0x9E37_79B9)
                ^ ((a as u64) << 32) ^ (b as u64) ^ self.last_chat.len() as u64 ^ 0xA4E1_D5;
            // the kinder of the pair extends the hand: kindly or
            // sociable lifts, a proud or cruel bent lowers - ties fall
            // to the earlier entity
            let bent = |s: &Speaker| -> f32 {
                s.traits.iter().map(|t| match t {
                    crate::game::traits::Trait::Kind => 1.0,
                    crate::game::traits::Trait::Gregarious => 0.5,
                    crate::game::traits::Trait::Proud => -0.5,
                    crate::game::traits::Trait::Cruel => -1.0,
                    _ => 0.0,
                }).sum()
            };
            let maker_first = (bent(&sa), -(a as i64)) >= (bent(&sb), -(b as i64));
            let (maker, target) = if maker_first { (a, b) } else { (b, a) };
            let mtraits = if maker_first { &sa.traits } else { &sb.traits };
            let mut mr = TalkRng::new(seed);
            if (mr.below(1000) as f32) < crate::game::amends::attempt_chance(mtraits) * 1000.0 {
                use crate::game::amends::{self, Answer};
                let ttraits = if maker_first { &sb.traits } else { &sa.traits };
                let refused = (mr.below(1000) as f32)
                    < amends::refusal_chance(ttraits) * 1000.0;
                let answer = amends::their_answer(ttraits, refused);
                let prior = self.world.get_component::<AmendsTally>(maker)
                    .filter(|x| x.of == target).map(|x| x.n).unwrap_or(0);
                let (maker_sp, target_sp) = if maker_first { (&sa, &sb) } else { (&sb, &sa) };
                let srel = relation_of(self.rel_kind(a, b));
                // the no's shape (E4-5): a guarded villager's refusal
                // runs long and soft where a proud one's stays short
                let said = talk_said(target_sp, maker_sp, srel, amends::answer_line(answer), false);
                let spoken = if answer == Answer::Refused {
                    let st = crate::game::text::stance_of(target_sp, maker_sp);
                    let ops = crate::game::text::shape(Act::PushBack, &st, None);
                    crate::game::text::shaped(&ops, &said, None, &st,
                        &mut TalkRng::new(seed ^ 0xA115))
                } else { said };
                let lines = vec![
                    TalkLine { who: Some(maker), name: self.name_of(maker),
                        text: talk_said(maker_sp, target_sp, srel, amends::AMENDS_ASK, false), gist: false },
                    TalkLine { who: Some(target), name: self.name_of(target),
                        text: spoken, gist: false },
                ];
                self.world.add_component(maker,
                    AmendsTally { of: target, n: prior.saturating_add(1) });
                if answer == Answer::Refused {
                    // the peace refused is its own beat - the quarrel
                    // stands, but it was offered and heard
                    self.push_talk_cold(TalkKind::Chat, vec![a, b], lines, true);
                    return true;
                }
                self.spats.remove(&key);
                relationships::record_interaction(&mut self.world, maker, Interaction {
                    other: target, kind: InteractionKind::Conversation,
                    intensity: amends::warmth_back(prior) });
                self.push_talk_cold(TalkKind::Chat, vec![a, b], lines, false);
                return true;
            }
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
                    // the scene's lines speak through the pair's own tv
                    // (E4-4): kin hear thee where strangers heard you
                    let srel = relation_of(self.rel_kind(a, b));
                    let s0 = if order.0 == a { &sa } else { &sb };
                    let s1 = if order.1 == a { &sa } else { &sb };
                    let lines: Vec<TalkLine> = variant.lines.iter().take(CHAT_MAX_LINES).map(|&(i, text)| {
                        let (who, name) = if i == 0 { (order.0, self.name_of(order.0)) }
                            else { (order.1, self.name_of(order.1)) };
                        TalkLine { who: Some(who), name,
                            text: talk_said(s0, s1, srel, text, i == 1), gist: false }
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
        // a passing chat is exposed by its kind (E3-2): the street hears
        sa.open = self.open_toward(a, b, false, None);
        sb.open = self.open_toward(b, a, false, None);
        // seeded off the meeting itself, not the game's rng: chatter on the
        // street mustn't shift every other roll in the sim
        let seed = self.tick.wrapping_mul(0x9E37_79B9)
            ^ ((a as u64) << 32) ^ (b as u64) ^ self.last_chat.len() as u64;
        let recent = self.fresh_lines();
        let tr = converse(&sa, &sb, rel, self.rel_kind(a, b), library(), Some(&recent), &mut TalkRng::new(seed));
        self.note_said_lines(tr.turns.iter().filter_map(|t| t.line));
        // the exchange types what it does to the bond (#1230): the acts
        // and the close read warm or cold before the lines go on record
        let temp = crate::game::conversation::chat_temp(&tr);
        // who threw the sharp words, for the feud's tale if it comes to one
        let spat_by = {
            let pa = tr.turns.iter().filter(|t| t.speaker == 0 && t.act == Act::PushBack).count();
            let pb = tr.turns.iter().filter(|t| t.speaker == 1 && t.act == Act::PushBack).count();
            if pa >= pb { a } else { b }
        };
        let mut lines: Vec<TalkLine> = tr.turns.iter()
            // kids keep to the children's tree: the rest is grown-up talk
            .take_while(|t| !kids.0 || t.tree == KID_TREE)
            .take(CHAT_MAX_LINES)
            .map(|t| {
                let (who, name) = if t.speaker == 0 { (a, na.clone()) } else { (b, nb.clone()) };
                TalkLine { who: Some(who), name, text: t.text.clone(), gist: false }
            }).collect();
        if lines.is_empty() { return false; }
        // inside news travels (#1287): a member of the cause hands a
        // piece on - whole to a fellow member, a little bent past that,
        // and marked where it lands on a wrong ear
        if !kids.0 { self.rumor_pass(a, b, &mut lines, seed); }
        // a neighbor asks a neighbor for a hand (#1368): rare, and the
        // kind and sociable dare to ask where the proud won't - a yes
        // sends them by to lend it, a no or a can't stings a little
        if !kids.0 {
            let mut hr = TalkRng::new(seed ^ 0xA11F_5EED);
            let bent = |t: &[crate::game::traits::Trait]| -> f32 {
                t.iter().map(|t| match t {
                    crate::game::traits::Trait::Kind => 1.0,
                    crate::game::traits::Trait::Gregarious => 0.5,
                    crate::game::traits::Trait::Proud => -0.5,
                    crate::game::traits::Trait::Cruel => -1.0,
                    _ => 0.0,
                }).sum()
            };
            let asker_a = (bent(&sa.traits), -(a as i64)) >= (bent(&sb.traits), -(b as i64));
            let (asker, asked) = if asker_a { (a, b) } else { (b, a) };
            let (asker_s, asked_s) = if asker_a { (&sa, &sb) } else { (&sb, &sa) };
            let (asker_name, asked_name) = if asker_a { (na.clone(), nb.clone()) } else { (nb.clone(), na.clone()) };
            let atraits = if asker_a { &sa.traits } else { &sb.traits };
            let ttraits = if asker_a { &sb.traits } else { &sa.traits };
            let rate = 0.04 * (1.0 + 0.5 * bent(atraits).max(0.0));
            let gone = |e: Entity| super::is_sick(&self.world, e)
                || self.world.get_component::<super::Pos>(e).is_none();
            if (hr.below(1000) as f32) < rate * 1000.0 && !gone(asker) && !gone(asked) {
                // what the asked makes is what they can be asked for
                // (#1388): the fish for coin, the pot of honey
                let favor = self.world.get_component::<super::Craft>(asked)
                    .and_then(|c| FAVOR_ASKS.iter().find(|(v, _)| *v == c.vocation).map(|(_, t)| *t));
                let favor = favor.filter(|_| hr.below(2) == 0);
                lines.push(TalkLine { who: Some(asker), name: asker_name,
                    text: talk_said(asker_s, asked_s, rel, favor.unwrap_or_else(|| HELP_ASKS[hr.below(HELP_ASKS.len())]), false), gist: false });
                let refuse = (hr.below(1000) as f32)
                    < (0.25 - 0.08 * bent(ttraits)).clamp(0.05, 0.9) * 1000.0;
                let tile = self.world.get_component::<super::Pos>(asker)
                    .map(|p| (p.x.round() as i32, p.y.round() as i32));
                // a promise only binds if the road's theirs to take:
                // already bound elsewhere, that's the can't, not the won't
                let free = !self.travels.iter().any(|t| t.who == asked);
                let sent = !refuse && free && tile
                    .map(|t| self.send_far(asked, t, super::Goal::HelpOut(asker)))
                    .unwrap_or(false);
                let pool = if !sent { HELP_NO }
                    else if favor.is_some() { FAVOR_YES }
                    else { HELP_YES };
                let said = talk_said(asked_s, asker_s, rel, pool[hr.below(pool.len())], false);
                let text = if !sent {
                    // the no's shape (E4-5): guarded runs long and soft
                    let st = crate::game::text::stance_of(asked_s, asker_s);
                    let ops = crate::game::text::shape(Act::PushBack, &st, None);
                    crate::game::text::shaped(&ops, &said, None, &st, &mut hr)
                } else { said };
                lines.push(TalkLine { who: Some(asked), name: asked_name,
                    text, gist: false });
                if !sent {
                    relationships::record_interaction(&mut self.world, asker,
                        relationships::Interaction { other: asked,
                            kind: relationships::InteractionKind::Conflict, intensity: 0.15 });
                }
            }
        }
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

    /// Inside news travels (#1287): a member of the cause hands a piece
    /// on - whole to a fellow member, a little bent past that, and
    /// marked where it lands on a wrong ear. One piece a chat is plenty,
    /// and a pair doesn't re-tell inside the week
    pub(in crate::session) fn rumor_pass(&mut self, a: Entity, b: Entity, lines: &mut Vec<TalkLine>, seed: u64) {
        use crate::game::rumor as rum;
        use crate::sim::community::Organization;
        const RUMOR_COOLDOWN: u64 = 7;
        let key = (a.min(b), a.max(b));
        let fresh = self.rumor_marks.get(&key).map(|t| self.tick >= t + RUMOR_COOLDOWN).unwrap_or(true);
        if !fresh { return; }
        let orgs = |w: &World, v: Entity| -> Vec<Entity> {
            w.query::<Organization>().iter().copied()
                .filter(|&o| w.get_component::<Organization>(o)
                    .map(|g| g.members.contains(&v)).unwrap_or(false)).collect()
        };
        for (teller, hearer) in [(a, b), (b, a)] {
            for org in orgs(&self.world, teller) {
                let table = rum::news_of(&self.world, org, self.tick, &|e| self.name_of(e));
                if table.is_empty() { continue; }
                let hm = orgs(&self.world, hearer).contains(&org);
                // what's still news to them: a member's read the table
                // already - the lines are flavor; a non-member checks
                // their store for a piece on this topic
                let fresh_news: Vec<rum::News> = if hm { table } else {
                    table.into_iter().filter(|n|
                        self.world.get_component::<rum::Rumor>(hearer)
                            .map(|r| !r.heard(n.topic())).unwrap_or(true)).collect()
                };
                if fresh_news.is_empty() { continue; }
                // who may hear it: a fellow member shares freely, a
                // friendly ear leans in, a wrong ear - rival to the
                // leader or the teller - overhears and gets marked
                let wrong_ear = !hm && (self.rel_kind(hearer, teller) == Some(RelKind::Rival)
                    || self.world.get_component::<Organization>(org)
                        .map(|o| self.rel_kind(hearer, o.leader) == Some(RelKind::Rival)).unwrap_or(false));
                let friendly = !hm && !wrong_ear && matches!(self.rel_kind(hearer, teller),
                    Some(RelKind::Friend) | Some(RelKind::Family) | Some(RelKind::Lover)
                        | Some(RelKind::Acquaintance));
                let chance = if hm { 40 } else if wrong_ear { 20 } else if friendly { 20 } else { 0 };
                let mut mr = TalkRng::new(seed ^ ((teller as u64) << 8) ^ (hearer as u64));
                if chance == 0 || mr.below(100) >= chance { continue; }
                let news = fresh_news[mr.below(fresh_news.len())].clone();
                // the drift: a member tells the table's truth, bent on
                // the line now and then; a non-member keeps what's said
                let spoken = if mr.below(100) < 30 {
                    rum::bend(&self.world, org, &news, mr.below(u32::MAX as usize) as u64, &|e| self.name_of(e))
                } else { news };
                if !hm {
                    if self.world.get_component::<rum::Rumor>(hearer).is_none() {
                        self.world.add_component(hearer, rum::Rumor::default());
                    }
                    self.world.get_component_mut::<rum::Rumor>(hearer).unwrap()
                        .hear(rum::RumorFact { news: spoken.clone(), wrong_ear });
                }
                self.rumor_marks.insert(key, self.tick);
                let tn = self.name_of(teller);
                let hn = self.name_of(hearer);
                lines.push(TalkLine { who: Some(teller), name: tn,
                    text: rum::line_of(&spoken), gist: false });
                lines.push(TalkLine { who: Some(hearer), name: hn,
                    text: if hm { "Aye - keep it quiet, mind.".into() }
                        else { "They don't miss much, do they.".into() },
                    gist: false });
                return;
            }
        }
    }

    /// The player has just talked with `v` (Talk's own effects are done):
    /// open the conversation on the TALK panel - their opening line, then
    /// `gist` (what Talk drew out of them), then the player's replies.
    /// `rel` is the bond as it stood before this talk.
    pub fn open_player_talk(&mut self, seat: usize, v: Entity, rel: Relation, gist: &str) {
        self.open_player_talk_familiar(seat, v, rel, gist, None);
    }

    /// `talks` is the bond's count before this talk's own opening beat
    /// deepened it (E3-2): act_talk lands an interaction before the
    /// panel opens, and the personal tier's "two talks" means prior ones
    pub(in crate::session) fn open_player_talk_familiar(&mut self, seat: usize, v: Entity,
        rel: Relation, gist: &str, talks: Option<u32>) {
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
            if !crate::game::desires::known(&self.world, self.seats[seat].player, v).is_empty() {
                villager.status.push("shared");
            }
            if !crate::game::desires::unrevealed(&self.world, self.seats[seat].player, v, self.tick).is_empty() {
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
        // the line they read you by moves what they'll hear a rung (#1236)
        let mut asks = crate::game::organizing::ask_offers(&self.world, self.seats[seat].org, v,
            crate::game::rhetoric::ask_shift(&self.world, v, self.seats[seat].org), self.tick);
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
        // their read of your rhetoric rides on your Speaker (#1236), so
        // a tree can key `other:status:firebrand`
        let mut player = self.player_speaker(pe);
        let org = self.seats[seat].org;
        player.status.extend(crate::game::rhetoric::hardened(&self.world, v, org).iter().map(|l| l.status()));
        let mut opening: Vec<String> = crate::game::rhetoric::register_note(&self.world, v, org, &villager.name)
            .into_iter().collect();
        if let Some(q) = crate::game::rhetoric::take_turn(&mut self.world, v, org) {
            opening.push(format!("{} names the turn: {q}", villager.name));
        }
        // who taught them, named the once (#1239)
        if let Some(mk) = crate::game::mentorship::untold(&self.world, v) {
            let who = match mk.by {
                crate::game::mentorship::Teacher::Person(t) => self.name_of(t),
                crate::game::mentorship::Teacher::Cause(_) => "the cause's meetings".into(),
            };
            crate::game::mentorship::tell(&mut self.world, v);
            opening.push(format!("{}: \"{}\"", villager.name, crate::game::mentorship::saying(mk, &who)));
        }
        // a hook laid to rest, remembered the once (#1243)
        if let Some((id, by)) = crate::game::hooks::untold(&self.world, v) {
            let friend = self.name_of(by);
            crate::game::hooks::tell(&mut self.world, v, id);
            let by_you = by == self.seats[seat].player;
            opening.push(format!("{}: \"{}\"", villager.name, crate::game::hooks::remembered(id, by_you, &friend)));
        }
        // the pair's own memory surfaces (E6-2): the top episodes by
        // the retrieval score become `mem:*` facts the trees can key
        // and a callback line - every fact traces to a real episode
        {
            use crate::game::episodes;
            let eps: Vec<episodes::PairEp> = self.world.get_component::<episodes::PairMemory>(v)
                .filter(|m| m.pe == pe).map(|m| m.eps.clone()).unwrap_or_default();
            let remembered = episodes::top(&eps, self.tick, &[], 2);
            for e in &remembered { villager.facts.open.push(episodes::fact(e.kind).into()); }
            if let Some(e) = remembered.first() {
                opening.push(format!("{}: \"{}\"", villager.name, episodes::callback(e.kind)));
            }
        }
        // a whole mystery they've been asking after: the telling's on offer
        let tell = (rare.is_none() && self.mystery_due(seat, v)).then(|| {
            let m = self.seats[seat].mystery.as_ref().map(|m| m.def).unwrap_or(0);
            format!("Tell them what you pieced together about {}", crate::game::mysteries::MYSTERIES[m].title)
        });
        // a job going begging opens the talk (#1336): the offer's the
        // reason you walked over, so their opening names it and the
        // take-it pick leads - latched now so the numbers hold. Same
        // gate the card's button had: offer live, hands free, earshot
        let (px, py) = self.player_pos_at(seat);
        let near = self.world.get_component::<villager::Pos>(v)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= super::TALK_DIST)
            .unwrap_or(false);
        let job = rare.is_none() && near && self.seats[seat].quest.is_none()
            && self.quest_offer_at(v).is_some();
        if job {
            if let Some(q) = self.quest_offer_at(v) {
                opening.push(format!("{}: \"{}\"", villager.name, self.pending_ask_line(q)));
            }
        }
        let seed = self.tick.wrapping_mul(0x2545_F491) ^ ((v as u64) << 24) ^ self.talk_next_id;
        self.close_player_talk(seat, None);
        // they stop where they are for the talk: a walk under way would
        // carry them off mid-line, and no new one dispatches while it's
        // open (in_player_talk gates the sends and the walk systems)
        self.travels.retain(|t| t.who != v);
        self.world.add_component(v, villager::InPlayerTalk);
        // how far they'll open to this seat (E3-2): a talk's a private
        // 1-1 unless ears are near. `talks` is the count before this
        // talk's own greeting deepened the bond
        villager.open = self.open_toward(v, pe, true, talks);
        let id = self.push_talk(TalkKind::Talk, vec![pe, v], vec![]);
        // a visitor's standing is the road's own blurb, not a villager
        // backstory built off statuses it doesn't have (#1168)
        let profile = rare.map(|i| super::events::visitor_blurb(i).to_string())
            .unwrap_or_else(|| crate::game::dialogue_trees::backstory(&villager));
        let kind = self.rel_kind(pe, v);
        self.seats[seat].talk = Some(PlayerTalk {
            id, who: v, knower: pe, villager, player, rel, kind, seed, picks: vec![],
            profile,
            gist: gist.to_string(),
            options: vec![], opt_tree: None, since: self.tick, notes: opening, asks,
            asks_taken: vec![], asked_seen: 0, forward_seen: 0, shared: 0,
            // a villager who once heard this seat's testimony opens a
            // rung warmer - they heard the story, they know the teller (#1333)
            mood: if crate::sim::psychology::heard_testimony(&self.world, v, pe) {
                TalkMood::Warmed } else { TalkMood::Cool },
            vibe: None,
            tired, argue: None, argued: vec![], inoculating: false, tell,
            // a talk opened on a slight carries the repair move (#1251)
            amends: self.talk_mood_of(v) == Some(TalkMood::Slighted),
            // and a hog carries the listener's trade (#1252)
            let_run: self.world.get_component::<crate::game::traits::Personality>(v)
                .map(|p| p.traits.contains(&crate::game::traits::Trait::SelfAbsorbed))
                .unwrap_or(false),
            job, inviting: None,
            safety_bid: None, bid_passed: false, call_act: None,
        });
        self.refresh_player_talk(seat);
    }

    /// Re-run the seat's Talk up to now and put it on the record; once
    /// it's over (a farewell, a walk-off, nothing left to say) close it
    fn refresh_player_talk(&mut self, seat: usize) {
        let Some(t) = self.seats[seat].talk.clone() else { return };
        let pe = self.seats[seat].player;
        let recent = self.fresh_lines();
        let (said, offered, ending, tree, vibe, asked, forward, shared, call_act) = replay(&t, Some(&recent));
        if shared > t.shared {
            // the deepest telling so far this talk (E3-3): the running
            // gate already lives in the replay - this only carries the
            // count to the parting for persistence; the speaker
            // snapshot stays as-opened so earlier picks replay exact
            if let Some(cur) = self.seats[seat].talk.as_mut() { cur.shared = shared; }
        }
        // a too-deep ask costs the bond its nerve (E3-2): each new
        // forward mark on the replay drops the rung a step - and a
        // safe read's deeper offers come back off the table, the same
        // reshift a missed audition would have dealt
        if forward > t.forward_seen {
            let who = t.who;
            if let Some(re) = crate::game::relationships::find_relationship(&self.world, pe, who)
                .and_then(|e| self.world.get_component_mut::<crate::game::relationships::Relationship>(e)) {
                for _ in t.forward_seen..forward { re.safety = re.safety.lower(); }
            }
            // an ask that came before the bond was ready is a memory
            // they keep (E6-2): "you asked too soon" can come back
            crate::game::episodes::file(&mut self.world, who, pe,
                crate::game::episodes::MemKind::AskedTooSoon, self.tick, &["ask"]);
            if t.vibe == Some(Vibe::Safe) {
                let mut asks = crate::game::organizing::ask_offers(
                    &self.world, self.seats[seat].org, t.who,
                    -1 + crate::game::rhetoric::ask_shift(&self.world, t.who, self.seats[seat].org),
                    self.tick);
                asks.retain(|o| !t.asks_taken.iter().any(|d| d.kind == o.kind && d.key == o.key));
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.asks = asks; }
            }
            if let Some(cur) = self.seats[seat].talk.as_mut() { cur.forward_seen = forward; }
        }
        self.note_said_lines(said.iter().filter_map(|x| x.2));
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
                &self.world, self.seats[seat].org, t.who,
                if safe { 1 } else { -1 } + crate::game::rhetoric::ask_shift(&self.world, t.who, self.seats[seat].org), self.tick);
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
        for (k, (sp, text, _line)) in said.iter().enumerate() {
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
        let mut drawn: Vec<TalkLine> = vec![];
        // a test standing holds every telling for its answer (E5-4):
        // the ask's spent either way - the secret just doesn't move
        let mut bid_pending = t.safety_bid.is_some();
        for label in asked.iter().skip(t.asked_seen) {
            use crate::game::dialogue_trees::Draw;
            let draw = crate::game::dialogue_trees::ASK_TOPICS.iter()
                .find(|tp| tp.label == *label).map(|tp| tp.draws).unwrap_or(Draw::No);
            match draw {
                Draw::No => {}
                Draw::Secret if bid_pending => {}
                Draw::Secret => {
                    let pick = self.rng.next();
                    if let Some(r) = crate::game::desires::next_reveal(&self.world, pe, t.who, self.tick, pick) {
                        if self.reveal_gated(seat, &t) {
                            // a fresh secret tests the bond's nerve first
                            // (E5-4): the bid asks, the telling holds until
                            // the player's answered it
                            let (line, kind) = self.safety_bid_line(seat, &t, r, pick);
                            drawn.push(TalkLine { who: Some(t.who), name: vname.clone(), text: line, gist: false });
                            if let Some(cur) = self.seats[seat].talk.as_mut() {
                                cur.safety_bid = Some(SafetyBid { kind, reveal: r });
                            }
                            bid_pending = true;
                            continue;
                        }
                        let (did, said_line) = crate::game::desires::reveal_speech(&vname, &r);
                        if let Some(issue) = crate::game::desires::hear(&mut self.world, pe, t.who, r, self.tick) {
                            use crate::session::gamelog::{EvTag, Sev, Thread, Valence};
                            self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                            self.log_entity_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                                pe, format!("t{}: Organizer discovers issue {}", self.tick, issue));
                        }
                        drawn.push(TalkLine { who: None, name: String::new(), text: did.clone(), gist: false });
                        drawn.push(TalkLine { who: Some(t.who), name: vname.clone(), text: said_line, gist: false });
                        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.notes.push(did); }
                    } else if let Some(r) = crate::game::desires::catchup_reveal(&self.world, pe, t.who, self.tick, pick) {
                        // the well's not dry - it circles (#1347): an old
                        // truth they still carry comes back around,
                        // re-read as catching up, not a fresh telling
                        let (did, said_line) = crate::game::desires::catchup_speech(&vname, &r);
                        drawn.push(TalkLine { who: None, name: String::new(), text: did.clone(), gist: false });
                        drawn.push(TalkLine { who: Some(t.who), name: vname.clone(), text: said_line, gist: false });
                        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.notes.push(did); }
                    }
                }
                Draw::Mystery => {
                    if let Some(fact) = self.mystery_fact_asked(seat) {
                        drawn.push(TalkLine { who: Some(t.who), name: vname.clone(), text: fact, gist: false });
                        if let Some(m) = &self.seats[seat].mystery {
                            let def = &crate::game::mysteries::MYSTERIES[m.def];
                            let note = format!("A piece of {} - {} of {} heard. Worth remembering.",
                                def.title, m.known.count_ones(), def.clues.len());
                            if let Some(cur) = self.seats[seat].talk.as_mut() { cur.notes.push(note); }
                        }
                    }
                }
            }
        }
        // a draw's lines ride the talk's argued record (#1347): pushed
        // straight to `rec.lines` they'd wash out of the next refresh's
        // replay; carried on `argued` they re-emit where they fell
        if !drawn.is_empty() {
            let at = said.len().max(1);
            if let Some(cur) = self.seats[seat].talk.as_mut() {
                for l in drawn.iter() { cur.argued.push((at, l.clone())); }
            }
            if let Some(rec) = self.talk_record_mut(t.id) { rec.lines.extend(drawn); }
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.asked_seen = asked.len(); }
        self.warm_exchange(t.id);
        // the talk's spent when every reply left is an exit and there's no
        // ask to hang on (#1179): they take their leave as much as you do,
        // so close on a mutual parting rather than an exit menu
        if let Some(opts) = &offered {
            if !opts.is_empty() && t.asks.is_empty() && !t.picks.is_empty() && t.argue.is_none() && !t.inoculating && t.tell.is_none()
                && !bid_pending
                && opts.iter().all(|(a, _, _)| *a == Act::Farewell) {
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
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.options = opts; cur.opt_tree = tree; cur.call_act = call_act; }
            }
            _ => {
                // a fresh talk that opens with no replies on offer would
                // hang up the moment it started (#1058): keep it open on
                // the leave alone, so the player's the one to end it.
                // A real ending (a walk-off) still closes as before.
                if t.picks.is_empty() && ending.is_none() {
                    if let Some(cur) = self.seats[seat].talk.as_mut() {
                        cur.options = vec![(Act::Farewell, crate::game::conversation::Respond::Validate, "I take my leave.".into())];
                        cur.opt_tree = tree;
                        cur.call_act = call_act;
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

    /// The openness `who` carries toward `to` (E3-2, #1424): the bond's
    /// parts as the components read them, last sit-down's mood, the
    /// talks behind them, and ears near enough to hear. `private` marks
    /// the talk a 1-1; anything else (a passing chat, a table) is
    /// exposed by its kind alone. `talks` overrides the count when the
    /// caller knows the bond's tally was taken before this talk's
    /// opening beat landed on it
    fn open_toward(&self, who: Entity, to: Entity, private: bool,
        talks: Option<u32>) -> crate::game::conversation::Open {
        use crate::game::relationships::SafetyRung;
        let mut o = crate::game::conversation::Open {
            earshot: !private || self.ears_near(who, to),
            ..Default::default() };
        if let Some(r) = crate::game::relationships::find_relationship(&self.world, who, to)
            .and_then(|e| self.world.get_component::<crate::game::relationships::Relationship>(e)) {
            // a seeded strength reconciles under intimacy before the
            // parts read - same numbers the bond's own bookkeeping sees
            let mut r2 = crate::game::relationships::Relationship {
                from: r.from, other: r.other, kind: r.kind, strength: r.strength,
                interactions: r.interactions, intimacy: r.intimacy,
                trust: r.trust, safety: r.safety,
                pivots: r.pivots, deflects: r.deflects, bids_met: r.bids_met };
            r2.recompose();
            o.intimacy = r2.intimacy;
            o.benevolence = r2.trust.benevolence;
            o.integrity = r2.trust.integrity;
            o.safety = match r2.safety {
                SafetyRung::Inclusion => 0.0, SafetyRung::Learner => 1.0 / 3.0,
                SafetyRung::Contributor => 2.0 / 3.0, SafetyRung::Challenger => 1.0 };
            o.talks = r2.interactions;
        }
        if let Some(n) = talks { o.talks = n; }
        o.mood = match self.talk_mood_of(who) {
            Some(TalkMood::Warmed) => 0.5, Some(TalkMood::Guarded) => -0.2,
            Some(TalkMood::Slighted) => -0.5, _ => 0.0 };
        // what THIS seat's told them that landed, while it's still
        // fresh (E3-3): another seat's telling is not yours
        o.shared = self.world.get_component::<SharedDepth>(who)
            .and_then(|s| s.of.get(&to).copied())
            .filter(|(_, day)| self.tick < day + TALK_MEMORY_DAYS)
            .map(|(depth, _)| depth).unwrap_or(0);
        o
    }

    /// An open street is no confessional (E3-2): anyone standing within
    /// earshot of `who` hears the talk
    fn ears_near(&self, who: Entity, to: Entity) -> bool {
        const EARSHOT: f32 = 5.0;
        let Some(wp) = self.world.get_component::<villager::Pos>(who).map(|p| (p.x, p.y))
            else { return false };
        self.villagers.iter().any(|&v| v != who && v != to
            && !villager::is_dead(&self.world, v)
            && self.world.get_component::<villager::Pos>(v)
                .map(|p| ((p.x - wp.0).powi(2) + (p.y - wp.1).powi(2)).sqrt() <= EARSHOT)
                .unwrap_or(false))
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
        // what they were told stays told (E3-3): keyed under this
        // seat's player, refreshed only when the talk told something -
        // small talk neither revives an expired telling nor rewinds it
        if t.shared > 0 {
            let pe = self.seats[seat].player;
            let mut sd = self.world.get_component::<SharedDepth>(t.who)
                .map(|s| s.of.clone()).unwrap_or_default();
            let prev = sd.get(&pe).copied()
                .filter(|(_, day)| self.tick < day + TALK_MEMORY_DAYS)
                .map(|(depth, _)| depth).unwrap_or(0);
            sd.insert(pe, (prev.max(t.shared), self.tick));
            self.world.add_component(t.who, SharedDepth { of: sd });
        }
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
        if let (true, Some(i)) = (t.inoculating, pick) { self.inoculate_pick_at(seat, i); return; }
        if let (Some(_), Some(i)) = (&t.inviting, pick) { self.invite_pick_at(seat, i); return; }
        // a safety test standing takes its own answers (E5-4, #1436)
        if let (Some(b), Some(i)) = (t.safety_bid, pick) { self.safety_bid_pick_at(seat, i, b); return; }
        // and walking out on one is the miss itself: they reached and
        // were left standing - the rung pays it as a turn away, spent
        // beat or no, the way a too-deep ask's drop lands
        if let (Some(_), None) = (t.safety_bid, pick) {
            let (who, pe) = (t.who, self.seats[seat].player);
            relationships::push_bid(&mut self.world, pe, who, relationships::BidTurn::Away);
            self.close_player_talk(seat, Some("You take your leave."));
            return;
        }
        let (who, n, asks, first_pick) = (t.who, t.options.len(), t.asks_shown(), t.picks.is_empty());
        let extra = t.inoculation_beyond();
        let probe_extra = t.probe_beyond();
        // the job's the first pick while its offer's latched (#1336):
        // the bound counts it, then everything numbers one down into
        // the reply/ask space the rest of this reads
        let job = t.job;
        let tell_at = n + asks + extra.is_some() as usize + probe_extra.is_some() as usize;
        let amends_at = tell_at + t.tell.is_some() as usize;
        let let_run_at = amends_at + t.amends as usize;
        let Some(i) = pick.filter(|&i| i < let_run_at + t.let_run as usize + job as usize) else {
            if pick.is_none() { self.close_player_talk(seat, Some("You take your leave.")); }
            return;
        };
        if job && i == 0 { self.talk_take_job(seat); return; }
        let i = i - job as usize;
        // the last pick, when it's there: tell them the mystery's answer
        if t.tell.is_some() && i == tell_at {
            self.talk_tell_mystery(seat);
            return;
        }
        // past even the telling: own the wrong word (#1251)
        if t.amends && i == amends_at {
            self.talk_amends(seat);
            return;
        }
        // and past the amends, if both ride at once: hold the space (#1252)
        if t.let_run && i == let_run_at {
            self.talk_let_run(seat);
            return;
        }
        // a pick past the replies is a proposition: the ask lands or it
        // doesn't, and the word of it goes on the record
        if i >= n {
            let o = if i - n == asks && extra.is_some() { extra.unwrap() }
                else if i - n == asks + extra.is_some() as usize && probe_extra.is_some() {
                    probe_extra.unwrap() }
                else { t.asks[i - n] };
            if o.kind == crate::game::organizing::PropKind::Argue {
                self.open_argument(seat, o);
                return;
            }
            if o.kind == crate::game::organizing::PropKind::Inoculate {
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    cur.asks.retain(|a| *a != o);
                    cur.asks_taken.push(o);
                    cur.inoculating = true;
                }
                let il = self.player_line(seat, who, "How {2:do} {2:subj} think the lord will answer, when he hears {2:subj} {2:have} signed on?");
                self.argue_line(seat, Some(self.seats[seat].player), il);
                self.refresh_player_talk(seat);
                return;
            }
            if o.kind == crate::game::organizing::PropKind::Scout {
                let read = self.scout_read(seat);
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    cur.asks.retain(|a| *a != o);
                    cur.asks_taken.push(o);
                }
                self.argue_line(seat, Some(self.seats[seat].player),
                    "Who do folk turn to around here, when the roof leaks?".into());
                for l in read { self.argue_line(seat, Some(who), l); }
                self.refresh_player_talk(seat);
                return;
            }
            // the probing question (#1269): the want it plants ripens
            // in its own time - the talk's share of the thaw move the
            // meeting carries
            if o.kind == crate::game::organizing::PropKind::Probe {
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    cur.asks.retain(|a| *a != o);
                    cur.asks_taken.push(o);
                }
                let pl = self.player_line(seat, who, "What {2:do} {2:subj} actually want, when all's said and done?");
                self.argue_line(seat, Some(self.seats[seat].player), pl);
                let seed = self.rng.next();
                let pe = self.seats[seat].player;
                match crate::game::desires::plant_want(&mut self.world, who, pe, self.tick, seed) {
                    Some(w) => self.argue_line(seat, Some(who), format!(
                        "I ... I'd like {}, I'd warrant. I'd never thought to ask myself that.",
                        crate::game::desires::want_phrase(w))),
                    None => {
                        // nothing left to plant - but the question still
                        // draws what their life's taken on since (#1347):
                        // a fresh reveal, an old burden come back
                        // around, or a plain "I've wants enough"
                        let pick = self.rng.next();
                        let name = self.name_of(who);
                        if let Some(r) = crate::game::desires::next_reveal(&self.world, pe, who, self.tick, pick) {
                            if self.reveal_gated(seat, self.seats[seat].talk.as_ref().unwrap()) {
                                // the asked-for telling still tests the
                                // nerve first (E5-4): the ask's spent,
                                // the secret holds for the answer
                                let t2 = self.seats[seat].talk.clone().unwrap();
                                let (line, kind) = self.safety_bid_line(seat, &t2, r, pick);
                                self.argue_line(seat, Some(who), line);
                                if let Some(cur) = self.seats[seat].talk.as_mut() {
                                    cur.safety_bid = Some(SafetyBid { kind, reveal: r });
                                }
                                self.refresh_player_talk(seat);
                                return;
                            }
                            let (_did, said) = crate::game::desires::reveal_speech(&name, &r);
                            self.argue_line(seat, Some(who), said);
                            if let Some(issue) = crate::game::desires::hear(&mut self.world, pe, who, r, self.tick) {
                                use crate::session::gamelog::{EvTag, Sev, Thread, Valence};
                                self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                                self.log_entity_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                                    pe, format!("t{}: Organizer discovers issue {}", self.tick, issue));
                            }
                        } else if let Some(r) = crate::game::desires::catchup_reveal(&self.world, pe, who, self.tick, pick) {
                            let (_did, said) = crate::game::desires::catchup_speech(&name, &r);
                            self.argue_line(seat, Some(who), said);
                        } else {
                            self.argue_line(seat, Some(who),
                                "Want? I've wants enough, friend. Ask me another.".into());
                        }
                    }
                }
                self.refresh_player_talk(seat);
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
            // the first invitation to an un-called meet earns the
            // follow-up (#1337): they'll come - then "when are we
            // gathering?" The call panel's day pick stays the way when
            // no invite asks it first
            if o.kind == crate::game::organizing::PropKind::ComeAlong {
                let came = self.world.get_component::<crate::game::organizing::MeetingPlan>(oe)
                    .map(|p| p.coming.contains(&who) && p.for_day.is_none())
                    .unwrap_or(false);
                if came {
                    if let Some(cur) = self.seats[seat].talk.as_mut() {
                        cur.inviting = Some(InviteStep::When);
                    }
                    self.argue_line(seat, Some(who), "I'll come. When are we gathering, then?".into());
                }
            }
            self.refresh_player_talk(seat);
            return;
        }
        let (act, pick_respond) = (t.options[i].0, t.options[i].1);
        let pick_text = t.options[i].2.clone();
        // the pick's special effects match on the realized text (E4-4):
        // the pair cloned now so `t`'s borrow is free for the effects
        let (said_v, said_p, said_rel) =
            (t.villager.clone(), t.player.clone(), t.rel);
        let pe = self.seats[seat].player;
        let (tid, twho, old_scores, opt_tree, call_act) =
            (t.id, t.who, t.opt_tree == Some("old_scores"), t.opt_tree, t.call_act);
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
        // every offered line is a way of *receiving* what they said
        // (E6-3): its respond tag lands on the bond's parts - reflect
        // and care warm them, a pivot is remembered. Once a day, like
        // the beat; an audition read isn't a pick at all
        let bond_moving = !probing && (match act {
            Act::PushBack | Act::Agree | Act::Answer => true,
            Act::Greet | Act::Farewell => !old_scores,
            _ => false,
        } || !matches!(pick_respond, crate::game::conversation::Respond::Validate));
        if bond_moving && !spent {
            relationships::push_respond(&mut self.world, pe, who, pick_respond);
        }
        // a reach met, half-met, missed or refused (E5-4, #1436): when
        // their call was a bid, the pick's act is the turn it reads -
        // engaging warms and builds the nerve under the day's beat;
        // the miss and the refusal are costs, and costs land whenever
        // they're dealt, the way a too-deep ask's rung drop does
        if call_act == Some(Act::Bid) && !probing {
            let turn = match act {
                Act::Agree | Act::Answer | Act::Question => relationships::BidTurn::Toward,
                Act::PushBack => relationships::BidTurn::Against,
                Act::Farewell => relationships::BidTurn::Away,
                _ => relationships::BidTurn::Pivot,
            };
            let gain = matches!(turn, relationships::BidTurn::Toward
                | relationships::BidTurn::Pivot);
            if !gain || !spent {
                relationships::push_bid(&mut self.world, pe, who, turn);
            }
        }
        // the line you take is heard by the one you're talking to - the
        // day's first pick with them, like the bond (#1236)
        if bond_moving && !spent {
            use crate::game::rhetoric::{witness, Line};
            let org = self.seats[seat].org;
            match act {
                Act::PushBack => witness(&mut self.world, who, org, Line::Firebrand, self.tick),
                Act::Agree => witness(&mut self.world, who, org, Line::Patient, self.tick),
                _ => {}
            }
        }
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
            && pick_text == talk_said(&said_v, &said_p, said_rel,
                crate::game::dialogue_trees::EWE_GIFT, true) {
            self.ewe_cracked(seat);
        } else if opt_tree == Some(crate::game::dialogue_trees::EWE_PRICE_TREE)
            && pick_text == talk_said(&said_v, &said_p, said_rel,
                crate::game::dialogue_trees::EWE_SQUEEZE, true) {
            self.ewe_squeezed(seat);
        } else if opt_tree == Some(crate::game::dialogue_trees::EWE_VICTIM_TREE)
            && pick_text == talk_said(&said_v, &said_p, said_rel,
                crate::game::dialogue_trees::EWE_WHOSE_POT, true) {
            if let Some(th) = self.ewe_theft.as_ref().map(|t| t.thief) {
                let tn = self.name_of(th);
                self.narrate_talk(seat, &format!("She names him at last: {tn}."));
            }
        }
        // the Noah row (#1257): the mediator's word talks one down, or
        // the pick takes a side where the street can see it
        if opt_tree == Some(crate::game::dialogue_trees::NOAH_ROW_TREE) {
            if pick_text == talk_said(&said_v, &said_p, said_rel,
                crate::game::dialogue_trees::NOAH_PEACE, true) {
                self.row_calmed(seat, twho);
            } else if pick_text == talk_said(&said_v, &said_p, said_rel,
                crate::game::dialogue_trees::NOAH_SIDE_THEM, true) {
                self.row_sided(seat, twho);
            }
        }
        // the false traveller (#1258): naming the disguise puts the
        // plays on the card - say nothing, cry him out, or feed a story
        if matches!(opt_tree, Some(crate::game::dialogue_trees::FT_SPY_TREE)
            | Some(crate::game::dialogue_trees::FT_THIEF_TREE)
            | Some(crate::game::dialogue_trees::FT_RUN_TREE))
            && (pick_text == talk_said(&said_v, &said_p, said_rel,
                    crate::game::dialogue_trees::FT_ACCUSE, true)
                || pick_text == talk_said(&said_v, &said_p, said_rel,
                    crate::game::dialogue_trees::FT_ACCUSE_RUN, true)) {
            self.false_accused(seat);
        }
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.picks.push(i); cur.since = self.tick; }
        self.refresh_player_talk(seat);
        // an argumentative line is the argument's way in now (#1348):
        // the ask's gone from the menu - the words themselves open the
        // same flow, where they know you well enough to take it up
        if crate::game::persuasion::is_argumentative(&pick_text)
            || crate::game::persuasion::ARGUMENTATIVE_LINES.iter()
                .any(|l| pick_text == talk_said(&said_v, &said_p, said_rel, l, true)) {
            let stage = crate::game::organizing::stage_of(&self.world,
                self.seats[seat].player, who);
            if crate::game::persuasion::can_argue(stage) {
                self.open_argument(seat, crate::game::organizing::AskOffer {
                    kind: crate::game::organizing::PropKind::Argue,
                    key: crate::game::organizing::AskKey::Earned, price: 0.0 });
            }
        }
    }

    /// The scouting read (#1249): who the room already follows, said in
    /// fuzzy words - no numbers, you build the map yourself. The names
    /// are the village's strongest voices (pull, computed live), one a
    /// resented one if they hold a grudge, and their own read on how
    /// far their own voice carries
    fn scout_read(&self, seat: usize) -> Vec<String> {
        use crate::game::relationships::{RelKind, Relationship};
        let Some(t) = self.seats[seat].talk.as_ref() else { return vec![] };
        let who = t.who;
        // clear-voiced: above this a name's worth naming
        const CLEAR: f32 = 0.85;
        let mut ranked: Vec<(f32, Entity)> = self.villagers.iter().copied()
            .filter(|&v| v != who && !crate::game::villager::is_dead(&self.world, v))
            .map(|v| (crate::game::organizing::pull_of(&self.world, v), v))
            .collect();
        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1)));
        let mut out: Vec<String> = vec![];
        let clear: Vec<Entity> = ranked.iter().take(2)
            .filter(|(p, _)| *p >= CLEAR).map(|&(_, v)| v).collect();
        if clear.is_empty() {
            out.push("Folk keep their own counsel here - nobody the whole room looks to.".into());
        } else {
            out.push(format!("When the roof leaks, it's {} folk fetch.", self.name_of(clear[0])));
            if clear.len() > 1 {
                out.push(format!("And {} gets heard, most times.", self.name_of(clear[1])));
            }
        }
        // who they resent: a rival bond they carry, worst first
        let mut rivals: Vec<(f32, Entity)> = crate::game::relationships::bonds_of(&self.world, who)
            .iter().filter_map(|&b| {
                let r = self.world.get_component::<Relationship>(b)?;
                if r.kind != RelKind::Rival { return None; }
                let other = if r.from == who { r.other } else { r.from };
                if crate::game::villager::is_dead(&self.world, other) { return None; }
                Some((r.strength.abs(), other))
            }).collect();
        rivals.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.cmp(&b.1)));
        if let Some(&(_, r)) = rivals.first() {
            out.push(format!("{} though - I'd not follow them anywhere, mind.", self.name_of(r)));
        }
        let own = crate::game::organizing::pull_of(&self.world, who);
        out.push(if own >= 1.2 { "Me? Folk come to me when it counts.".into() }
            else if own >= 0.8 { "Me? They hear me out, mostly.".into() }
            else { "Me? Nobody much minds me.".into() });
        out
    }

    /// "About the other day" (#1251): owning the wrong word - a beat of
    /// the talk spent on it, their cool answer part of the price.
    /// Accepted, the door reopens - guarded, not warm, but they'll hear
    /// an ask again; refused, the chill stands. The count of sorries
    /// ticks either way - repeated sorries stop landing
    fn talk_amends(&mut self, seat: usize) {
        use crate::game::amends::{self, Answer};
        let (pe, oe) = (self.seats[seat].player, self.seats[seat].org);
        let Some(t) = self.seats[seat].talk.as_mut() else { return };
        t.amends = false;
        let who = t.who;
        let prior = self.world.get_component::<AmendsTally>(who)
            .filter(|a| a.of == pe).map(|a| a.n).unwrap_or(0);
        let traits: Vec<crate::game::traits::Trait> = self.world
            .get_component::<crate::game::traits::Personality>(who)
            .map(|p| p.traits.clone()).unwrap_or_default();
        let roll = (self.rng.next() % 1000) as f32 / 1000.0;
        let answer = amends::their_answer(&traits, roll < amends::refusal_chance(&traits));
        let spoken = if answer == Answer::Refused {
            let (vs, ps, tseed) = self.seats[seat].talk.as_ref()
                .map(|t| (t.villager.clone(), t.player.clone(), t.seed))
                .unwrap_or_else(|| (Speaker::new(""), Speaker::new(""), 0));
            let st = crate::game::text::stance_of(&vs, &ps);
            let ops = crate::game::text::shape(Act::PushBack, &st, None);
            crate::game::text::shaped(&ops, amends::answer_line(answer),
                None, &st, &mut TalkRng::new(tseed ^ 0xA116))
        } else {
            amends::answer_line(answer).into()
        };
        self.argue_line(seat, Some(pe), amends::AMENDS_ASK.into());
        self.argue_line(seat, Some(who), spoken);
        self.world.add_component(who, AmendsTally { of: pe, n: prior.saturating_add(1) });
        if answer == Answer::Refused {
            // the chill stands - the memory's still the slight, so the
            // next hello opens cold again; but the word was heard, and
            // it counts on the tally
            if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Slighted; }
            self.narrate_talk(seat, "They turn from it. The amends is heard, and refused.");
        } else {
            relationships::record_interaction(&mut self.world, pe, Interaction {
                other: who, kind: InteractionKind::Conversation,
                intensity: amends::warmth_back(prior) });
            if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Guarded; }
            // repair reopens the ask path: the asks a slight cleared
            // come back on the table - the work isn't over, it's heard
            let mut asks = crate::game::organizing::ask_offers(
                &self.world, oe, who, crate::game::rhetoric::ask_shift(&self.world, who, oe), self.tick);
            if let Some(cur) = self.seats[seat].talk.as_mut() {
                asks.retain(|o| !cur.asks_taken.iter().any(|d| d.kind == o.kind && d.key == o.key));
                cur.asks = asks;
            }
            self.narrate_talk(seat, "The wrong's owned. They hear you out - guarded, but hearing.");
        }
        self.refresh_player_talk(seat);
    }

    /// "Let them run on" (#1252): holding the space a hog fills. The
    /// monologue yields what they'd never trade for - their own story:
    /// an unrevealed trouble or want if they've one left, and their
    /// opinions of everyone either way. The village's business - the
    /// rumors, the mysteries - isn't theirs to give
    fn talk_let_run(&mut self, seat: usize) {
        let (pe, who) = (self.seats[seat].player, self.seats[seat].talk.as_ref().map(|t| t.who).unwrap());
        if let Some(t) = self.seats[seat].talk.as_mut() { t.let_run = false; }
        use crate::game::desires;
        let run_pick = self.rng.next();
        let run_fresh = desires::next_reveal(&self.world, pe, who, self.tick, run_pick);
        // an old trouble's still worth the running on (#1347): the
        // still-carrying line when the fresh well's dry
        let run_reveal = run_fresh.clone()
            .or_else(|| desires::catchup_reveal(&self.world, pe, who, self.tick, run_pick));
        if let Some(r) = run_reveal {
            let name = self.name_of(who);
            let (_did, said) = if run_fresh.is_some() {
                desires::reveal_speech(name.as_str(), &r)
            } else {
                desires::catchup_speech(name.as_str(), &r)
            };
            self.argue_line(seat, Some(who), self.pair_said(who, pe, &said));
            if let Some(issue) = desires::hear(&mut self.world, pe, who, r, self.tick) {
                self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                self.log_entity_ev(
                    super::gamelog::EvTag::new(
                        super::gamelog::Thread::Org,
                        super::gamelog::Valence::Positive,
                        super::gamelog::Sev::Minor),
                    pe, format!("t{}: Organizer discovers issue {}", self.tick, issue));
            }
        }
        // and the rest of the run: their past, their opinions of everyone
        let someones: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| v != who && !villager::is_dead(&self.world, v)).collect();
        // nobody else left to opine on? a hog names themselves just fine
        let other = someones.get((self.rng.next() as usize) % someones.len().max(1))
            .copied().unwrap_or(who);
        // `{someone}` resolves through the realizer's refs (E4-1): the
        // referent carries the body facts the grammar layers will read
        let refs = [crate::game::text::Referent {
            slot: "someone",
            text: self.name_of(other),
            sex: self.world.get_component::<villager::Sex>(other)
                .map(|s| match s { villager::Sex::Female => "female", villager::Sex::Male => "male" }),
            alive: !villager::is_dead(&self.world, other),
            count: 1,
        }];
        let run = crate::game::conversation::HOG_RUN;
        let template = run[(self.rng.next() as usize) % run.len()];
        let line = match self.seats[seat].talk.as_ref() {
            Some(t) => crate::game::text::realize(template,
                &crate::game::text::Cx::plain(&t.villager, &t.player, &refs),
                &mut TalkRng::new(t.seed ^ run_pick)),
            None => {
                let rel = relation_with_wanderer(self.rel_kind(pe, who));
                talk_said(&speaker_of(&self.world, who, &self.name_of(who), self.tick),
                          &self.player_speaker(pe), rel, template, false)
                    .replace("{someone}", &refs[0].text)
            }
        };
        self.argue_line(seat, Some(who), line);
        self.narrate_talk(seat, "You let them run - they fill the space with themselves.");
        self.refresh_player_talk(seat);
    }

    /// The job pick at the front of the list (#1336): the offer's still
    /// live, the hands are free and they're in earshot - the ask comes
    /// off the begging list onto the seat's card, and the exchange goes
    /// on the record the way an argument's lines do
    fn talk_take_job(&mut self, seat: usize) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let who = t.who;
        let live = self.quest_offer_at(who).is_some();
        let free = self.seats[seat].quest.is_none();
        let (px, py) = self.player_pos_at(seat);
        let near = self.world.get_component::<villager::Pos>(who)
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() <= super::TALK_DIST)
            .unwrap_or(false);
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.job = false; }
        if !(live && free && near) {
            self.narrate_talk(seat,
                "The job's off the table - the offer's lapsed, or your hands are full.");
            return;
        }
        let i = self.pending_quests.iter().position(|q| q.giver == who).unwrap();
        let quest = self.take_pending_quest(seat, i);
        let ask = self.quest_ask_line(&quest);
        let pe = self.seats[seat].player;
        self.argue_line(seat, Some(who), ask);
        self.argue_line(seat, Some(pe), "About that job - I'll take it.".into());
        self.refresh_player_talk(seat);
    }

    /// "I've pieced it together": the mystery told across the talk - your
    /// word, their wonder and thanks, and what it came to - and the talk
    /// goes on from there
    fn talk_tell_mystery(&mut self, seat: usize) {
        let Some(t) = self.seats[seat].talk.as_mut() else { return };
        t.tell = None;
        let who = t.who;
        let pe = self.seats[seat].player;
        let Some(told) = self.mystery_tell(seat, who) else { self.refresh_player_talk(seat); return };
        self.argue_line(seat, Some(pe), "I've heard all of it now - here's what the gossip adds up to.".into());
        self.argue_line(seat, Some(who), told.wonders.into());
        self.argue_line(seat, Some(who), told.thanks.into());
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.mood = TalkMood::Warmed; }
        self.narrate_talk(seat, &told.payoff);
    }

    /// Put a line from the argument on the talk's record, after what's
    /// been said so far (#1234)
    fn argue_line(&mut self, seat: usize, who: Option<Entity>, text: String) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let recent = self.fresh_lines();
        let at = replay(t, Some(&recent)).0.len().max(1);
        let name = match who { Some(w) if w == t.who => t.villager.name.clone(), Some(w) => self.speaker_name(w), None => String::new() };
        if let Some(cur) = self.seats[seat].talk.as_mut() {
            cur.argued.push((at, TalkLine { who, name, text, gist: false }));
        }
    }

    /// Is this telling earned yet (E5-4, #1436)? Below the learner
    /// rung a fresh secret tests the bond's nerve first - a passed bid
    /// this talk already cleared it, and a bond that's climbed holds it
    fn reveal_gated(&self, seat: usize, t: &PlayerTalk) -> bool {
        use crate::game::relationships::SafetyRung;
        if t.bid_passed { return false; }
        let pe = self.seats[seat].player;
        let rung = crate::game::relationships::find_relationship(&self.world, pe, t.who)
            .and_then(|e| self.world.get_component::<crate::game::relationships::Relationship>(e))
            .map(|r| r.safety).unwrap_or_default();
        rung == SafetyRung::Inclusion
    }

    /// The safety test's words and shape for a held secret (E5-4): the
    /// probe tells someone else's lesser secret (real content - it
    /// files on the third party through `hear`), the check asks for
    /// the tongue, the hypothetical hides the telling behind "a man"
    pub(in crate::session) fn safety_bid_line(&mut self, seat: usize, t: &PlayerTalk,
                      r: crate::game::desires::Reveal, pick: u64)
                      -> (String, SafetyBidKind) {
        let (who, pe) = (t.who, self.seats[seat].player);
        use crate::game::desires::{self, Reveal};
        // the probe tells you someone else's lesser secret and watches:
        // a live trouble on a third party, told through the real system
        if pick % 3 == 2 {
            let third = self.villagers.iter().copied()
                .filter(|&v| v != who && !villager::is_dead(&self.world, v))
                .find(|&v| desires::next_reveal(&self.world, pe, v, self.tick, pick).is_some());
            if let Some(v3) = third {
                let dr = desires::next_reveal(&self.world, pe, v3, self.tick, pick).unwrap();
                let n3 = self.name_of(v3);
                let (did, _said) = desires::reveal_speech(&n3, &dr);
                desires::hear(&mut self.world, pe, v3, dr, self.tick);
                let line = format!("Word is, {} - but it wasn't from me {{2:subj}} heard it.", did);
                return (self.pair_said(who, pe, &line), SafetyBidKind::Probe);
            }
        }
        // the check, or "a man" - wants get the tongue asked, troubles
        // come in disguise
        match r {
            Reveal::Want(_) => (t.said("{2:can} {2:subj} hold {2:poss} tongue? There's a thing I'd near have said.",
                false), SafetyBidKind::Confidence),
            Reveal::Problem(_) => (t.said("Say a man kept back a sack from the tithe - what should be done with him, {2:think} {2:subj}?",
                false), SafetyBidKind::Hypothetical),
        }
    }

    /// The test answered (E5-4, #1436): a turn toward brings the held
    /// telling out and the nerve earns a mark; the sidestep keeps it
    /// held for another day; the miss or the leak costs - and the leak
    /// leaves the confidence's record behind it
    fn safety_bid_pick_at(&mut self, seat: usize, i: usize, bid: SafetyBid) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let (who, vname, pe) = (t.who, t.villager.name.clone(), self.seats[seat].player);
        let turn = bid.turn_of(i);
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.safety_bid = None; }
        // the answer's heard as your line first, the way any pick is
        if let Some(l) = bid.labels().get(i).cloned() {
            self.argue_line(seat, Some(pe), l);
        }
        use crate::game::relationships::BidTurn;
        // the test's answer always counts (E5-4): a kept reach earns,
        // a miss or a leak costs - the answer's a discrete event, not a
        // grindable pick, so it sits outside the day's beat
        relationships::push_bid(&mut self.world, pe, who, turn);
        match turn {
            BidTurn::Toward => {
                if let Some(cur) = self.seats[seat].talk.as_mut() {
                    cur.bid_passed = true; cur.mood = TalkMood::Warmed; }
                // the kept word is a memory too (E6-2's file; E5-4's
                // kind): #1443's ledger reads it as the confidence held
                crate::game::episodes::file(&mut self.world, who, pe,
                    crate::game::episodes::MemKind::KeptConfidence, self.tick, &["trust", "secret"]);
                let (did, said) = crate::game::desires::reveal_speech(&vname, &bid.reveal);
                self.argue_line(seat, None, did.clone());
                self.argue_line(seat, Some(who), said);
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.notes.push(did); }
                if let Some(issue) = crate::game::desires::hear(&mut self.world, pe, who, bid.reveal, self.tick) {
                    use crate::session::gamelog::{EvTag, Sev, Thread, Valence};
                    self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                    self.log_entity_ev(EvTag::new(Thread::Org, Valence::Positive, Sev::Minor),
                        pe, format!("t{}: Organizer discovers issue {}", self.tick, issue));
                }
            }
            BidTurn::Pivot => {
                self.argue_line(seat, Some(who),
                    self.pair_said(who, pe, "Then never mind me. Another hour for it, maybe."));
            }
            _ => {
                // the leak is the sharpest cut: a confidence taken to
                // the reeve is a memory they carry (its record is what
                // #1443's confidence ledger reads back)
                if turn == BidTurn::Against {
                    crate::game::episodes::file(&mut self.world, who, pe,
                        crate::game::episodes::MemKind::LeakedConfidence, self.tick, &["trust", "secret"]);
                    self.argue_line(seat, Some(who),
                        self.pair_said(who, pe, "Then forget I spoke. {2:subj} {2:be} not the one I took {2:obj} for."));
                } else {
                    self.argue_line(seat, Some(who),
                        self.pair_said(who, pe, "Forget I spoke of it. It was nothing to say."));
                }
            }
        }
        self.refresh_player_talk(seat);
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
                let cline = self.pair_said(who, self.seats[seat].player,
                    &persuasion::counter_line(&traits, stance, &issue));
                self.argue_line(seat, Some(who), cline);
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
                // how you argued it is a line they'll remember (#1236)
                {
                    use crate::game::rhetoric::Line;
                    let line = match m {
                        persuasion::ArgueMove::Press => Line::Firebrand,
                        persuasion::ArgueMove::HearOut => Line::Patient,
                        persuasion::ArgueMove::Appeal => Line::Merciful,
                    };
                    crate::game::rhetoric::witness(&mut self.world, who, oe, line, self.tick);
                }
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

    /// A pick inside the inoculation (#1246): name the counter, they
    /// say their fear of it, you rehearse the answer - and it'll land
    /// on someone ready for it
    fn inoculate_pick_at(&mut self, seat: usize, i: usize) {
        use crate::game::inoculation::{self as inoc, COUNTERS};
        let Some(t) = self.seats[seat].talk.as_mut() else { return };
        t.inoculating = false;
        let who = t.who;
        let pe = self.seats[seat].player;
        let Some(&c) = COUNTERS.get(i) else {
            self.argue_line(seat, None, "You let it drop.".into());
            self.refresh_player_talk(seat);
            return;
        };
        let fl = self.pair_said(who, pe, inoc::fear(c));
                    self.argue_line(seat, Some(who), fl);
        self.argue_line(seat, Some(pe), self.player_line(seat, who, inoc::answer(c)));
        inoc::shield(&mut self.world, who, c);
        let vname = self.name_of(who);
        self.argue_line(seat, None, format!("{vname} nods slowly - they'll know it when it comes."));
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.since = self.tick; }
        self.refresh_player_talk(seat);
    }

    fn drop_argument(&mut self, seat: usize) {
        self.argue_line(seat, None, "You let it drop.".into());
        if let Some(cur) = self.seats[seat].talk.as_mut() { cur.argue = None; }
        self.refresh_player_talk(seat);
    }

    /// The venues a where-pick can name (#1337): the square, then the
    /// church steps and the tavern when they stand, then the homes the
    /// org's learned - the invited's own door isn't listed twice, it's
    /// what "their call" means
    fn invite_venues(&self, seat: usize) -> Vec<(crate::game::organizing::Venue, String)> {
        use crate::game::organizing::Venue;
        let oe = self.seats[seat].org;
        let mut out = vec![(Venue::Square, self.venue_label(Venue::Square))];
        if self.church.is_some() { out.push((Venue::Church, self.venue_label(Venue::Church))); }
        if self.tavern.is_some() { out.push((Venue::Tavern, self.venue_label(Venue::Tavern))); }
        if let Some(kv) = self.world.get_component::<crate::game::organizing::KnownVenues>(oe) {
            for &(h, b) in &kv.homes {
                if crate::game::villager::is_dead(&self.world, h)
                    || self.world.get_component::<villager::Pos>(b).is_none() { continue; }
                out.push((Venue::House(h, b), self.venue_label(Venue::House(h, b))));
            }
        }
        out
    }

    /// A pick inside the invitation's follow-up (#1337): the when, then
    /// the where - and the word goes out with both settled, the same
    /// landing the call panel's day pick makes
    fn invite_pick_at(&mut self, seat: usize, i: usize) {
        let Some(t) = self.seats[seat].talk.as_ref() else { return };
        let Some(step) = t.inviting.clone() else { return };
        let (who, pe, oe) = (t.who, self.seats[seat].player, self.seats[seat].org);
        match step {
            InviteStep::When => {
                let Some(&days) = [2u64, 4, 7].get(i) else { return };
                let day_line = match days {
                    2 => "Two days' time - spread the word quick.",
                    4 => "Four days - let the word get round.",
                    _ => "A week - the far ones can come.",
                };
                self.argue_line(seat, Some(pe), day_line.into());
                // a venue already promised (a lent house, a pick that
                // ran ahead) settles the where itself - the call just
                // wants its day
                let set = self.world.get_component::<crate::game::organizing::MeetingPlan>(oe)
                    .map(|p| p.venue.is_some()).unwrap_or(false);
                if set {
                    if let Some(cur) = self.seats[seat].talk.as_mut() { cur.inviting = None; }
                    self.the_words_out(seat, days);
                    self.argue_line(seat, Some(who), "I'll be there.".into());
                } else {
                    let venues = self.invite_venues(seat);
                    if let Some(cur) = self.seats[seat].talk.as_mut() {
                        cur.inviting = Some(InviteStep::Where { days, venues });
                    }
                    self.argue_line(seat, Some(who), "And where d'you want us?".into());
                }
                self.refresh_player_talk(seat);
            }
            InviteStep::Where { days, venues } => {
                let home = crate::game::household::home_of(&self.world, who);
                let pick_venue = if i == venues.len() {
                    // "their call": their own door if they have one -
                    // and the org learns it - else the square
                    home.map(|b| crate::game::organizing::Venue::House(who, b))
                        .unwrap_or(crate::game::organizing::Venue::Square)
                } else {
                    let Some(&(v, _)) = venues.get(i) else { return };
                    v
                };
                if let crate::game::organizing::Venue::House(h, b) = pick_venue {
                    crate::game::organizing::learn_venue(&mut self.world, oe, h, b);
                }
                if let Some(p) = self.world.get_component_mut::<crate::game::organizing::MeetingPlan>(oe) {
                    p.venue = Some(pick_venue);
                }
                if let Some(cur) = self.seats[seat].talk.as_mut() { cur.inviting = None; }
                let label = self.venue_label(pick_venue);
                self.argue_line(seat, Some(pe), format!("At {}.", label));
                self.the_words_out(seat, days);
                self.argue_line(seat, Some(who), "I'll be there.".into());
                self.refresh_player_talk(seat);
            }
        }
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
            let mut block: Vec<TalkRowV> = vec![TalkRowV { name: None, text: format!("day {}", r.tick), gist: false, you: false, rule: true, who: None, mood: 0 }];
            block.extend(r.lines.iter().map(|l| {
                let who = l.who.and_then(|e| self.villagers.iter().position(|v| *v == e));
                TalkRowV {
                    // your own lines read "You"; another seat's keep their name
                    name: if l.who == Some(s.player) { Some("You".into()) } else { (!l.name.is_empty()).then(|| l.name.clone()) },
                    text: l.text.clone(), gist: l.gist,
                    you: l.who.map(|w| w == s.player).unwrap_or(false), rule: false,
                    who,
                    mood: who.map(|_| self.row_mood(seat, r.id, l.who.unwrap_or(s.player))).unwrap_or(0),
                }
            }));
            block.extend(rows);
            rows = block;
        }
        let in_talk = matches!(&s.talk, Some(t) if shown.contains(&t.id));
        let options = match &s.talk {
            Some(t) if in_talk => t.choice_labels(&self.world),
            _ => vec![],
        };
        let empty = if picked.is_some() { "Nothing heard from them yet." } else { "Talk to someone (T) and it shows here." };
        if rows.is_empty() { rows.push(TalkRowV { name: None, text: empty.into(), gist: false, you: false, rule: false, who: None, mood: 0 }); }
        TalkPanelV { title, rows, options, in_talk }
    }

    /// The convo box's mood mark for a speaker (#1320): the seat's read
    /// of their bond plus how this talk's going - 0 neutral, 1 warm,
    /// 2 cold, 3 angry. The live Talk's mood wins (a slight outranks an
    /// old warmth); the bond fills in where there's no live read
    fn row_mood(&self, seat: usize, record: u64, speaker: Entity) -> u8 {
        let s = &self.seats[seat];
        let live = s.talk.as_ref().filter(|t| t.id == record && t.who == speaker);
        match live.map(|t| t.mood) {
            Some(TalkMood::Slighted) => return 3,
            Some(TalkMood::Warmed) => return 1,
            Some(TalkMood::Guarded) => return 2,
            _ => {}
        }
        // a Cool talk that opened on a remembered slight keeps the
        // angry read until the amends land and flip it to Guarded
        if live.map(|t| t.amends).unwrap_or(false) { return 3; }
        let Some(r) = crate::game::relationships::find_relationship(&self.world, s.player, speaker)
            .and_then(|r| self.world.get_component::<crate::game::relationships::Relationship>(r))
        else { return 0 };
        if r.kind == RelKind::Rival { 3 }
        else if r.strength < 0.0 { 2 }
        else if matches!(r.kind, RelKind::Friend | RelKind::Lover | RelKind::Family)
            || crate::game::relationships::hearts_of(r.strength) >= 1.5 { 1 }
        else { 0 }
    }

    /// A talk record's lines as conversation-box rows: "You" leads the
    /// seat's own lines; the speaker carries their villager index so a
    /// click on the line selects them
    fn record_rows(&self, seat: usize, id: u64) -> Vec<TalkRowV> {
        let Some(r) = self.talk_log.iter().find(|r| r.id == id) else { return vec![] };
        let pe = self.seats[seat].player;
        r.lines.iter().map(|l| {
            let who = l.who.and_then(|e| self.villagers.iter().position(|v| *v == e));
            TalkRowV {
                name: if l.who == Some(pe) { Some("You".into()) } else { (!l.name.is_empty()).then(|| l.name.clone()) },
                text: l.text.clone(), gist: l.gist,
                you: l.who == Some(pe), rule: false,
                who,
                mood: who.map(|_| self.row_mood(seat, id, l.who.unwrap_or(pe))).unwrap_or(0),
            }
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
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                        text: "The Act's officer reads the order on the square: every infant takes the lancet's scratch, and the refusers pay the fine.".into() },
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                rows: vec![TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                rows: vec![TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                    text: "The cause's own eyes read the lord's - counter the watching.".into() }],
                options, decides: true, leave: false });
        }
        if self.emergency.is_some() {
            let rows = vec![
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                    text: "The lord's man calls it the cause's own brawl - the square's banned while the tale stands.".into() },
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                    text: "One of them's for the torch - burn the lord's stores, they say, now.".into() },
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                    text: format!("{}'s man brings an offer: a seat on the board.", lord) },
                TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                        text: "The plan's made - the word just wants a day.".into() },
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                        text: "The word's out. When they're in the room, what's the question?".into() },
                    TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
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
            rows.push(TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0, text: lean });
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
                rows: vec![TalkRowV { name: None, gist: true, you: false, rule: false, who: None, mood: 0,
                    text: format!("{} stand with the cause - what do they take up?", n) }],
                options, decides: true, leave: false });
        }
        if let Some(m) = &self.meeting {
            if m.seat == seat {
                let mut rows = self.record_rows(seat, m.talk_id);
                if m.talk_from.is_none() {
                    rows.push(TalkRowV { name: None, gist: false, you: false, rule: false, who: None, mood: 0,
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
                options: t.choice_labels(&self.world),
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
