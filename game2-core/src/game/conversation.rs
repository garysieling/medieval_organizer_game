//! Conversation framework: weighted call/response trees for two speakers,
//! and a looser, concept-threaded model for house meetings.
//!
//! Research basis (wiki/literature-reviews/dialogue-systems.md):
//! - Ruskin (Valve, GDC 2012): a rule database, not a branching script.
//!   Trees carry criteria on world facts; the most specific matching tree
//!   (most criteria) leads, and when it bottoms out the conversation falls
//!   through to the next-most-specific - generic small talk is the
//!   always-on tail, so nobody is ever left without something to say.
//! - CiF / Prom Week (McCoy et al.): traits and statuses *weight* replies
//!   rather than gate them - personality biases the distribution of the
//!   forks, it doesn't decide them. People still surprise you.
//! - Facade (Mateas & Stern): discourse acts - respond to the act, not the
//!   words; a meeting reply can bind to the last speaker, the organizer's
//!   open question, or an earlier statement.
//! - Versu (Evans & Short): once-only quips. Said lines are consumed, so a
//!   conversation runs out of material and dries up into small talk rather
//!   than repeating itself - the "they walk off" ending falls out of
//!   exhaustion, not a special case.
//!
//! The same structures serve NPC-NPC simulation and player-NPC play:
//! `converse` walks both sides; `converse_with` hands the player's turns
//! to a chooser that sees two sampled options and may end it early.

use crate::engine::det::HashSet;
use crate::engine::ecs::Entity;
use crate::game::relationships::RelKind;

use crate::game::traits::Trait;

/// Deterministic RNG (SplitMix64); no external crates, seeded runs replay
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    /// A stream off this one's state, salted: the slip pass rolls on
    /// its own stream (E5-8) so a replay with more picks rolls the
    /// same turns the same way
    fn fork(&self, salt: u64) -> Rng {
        Rng(self.0 ^ salt)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1)
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform in [0, n)
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next_u64() % n as u64) as usize }
    }

    /// Weighted pick over a slice; returns the index. Zero or negative
    /// weights can't be picked.
    pub fn weighted(&mut self, weights: &[f32]) -> usize {
        let total: f32 = weights.iter().copied().filter(|w| *w > 0.0).sum();
        if total <= 0.0 { return self.below(weights.len()); }
        let mut roll = self.next_f32() * total;
        for (i, w) in weights.iter().enumerate() {
            if *w <= 0.0 { continue; }
            roll -= w;
            if roll < 0.0 { return i; }
        }
        weights.len() - 1
    }
}

/// A person in a conversation: enough of a profile to roll facts against.
/// `status` carries circumstance ("sick", "grieving", "hungry", "drunk");
/// `ideology` carries vocation ("laborer", "clergy", "merchant", "lord").
/// What of a persona the fact stream can speak (E3-1, #1423): the open
/// half (archetype, values) is voice-observable and folds under every
/// prefix; the hidden half (wound, lie, disclosure tiers) stays
/// self-only until a later ticket's disclosure opens it
#[derive(Debug, Default, Clone)]
pub struct FactTable {
    /// `archetype:*`, `value:*` - folded under self/other/any
    pub open: Vec<String>,
    /// `ghost:*`, `lie:*`, `tier:*` - self-only for now
    pub hidden: Vec<String>,
}

/// A persona-lite profile for speakers built without a world
/// (talk_lab, dialogue_gen): an archetype read off the traits and two
/// values sampled off the salt, so `archetype:*`/`value:*` still reach
/// synthetic transcripts. Wounds need the E2-3 record - hidden stays
/// empty here
pub fn lite_facts(traits: &[Trait], salt: u64) -> FactTable {
    use crate::game::persona::{draw, VALUES};
    use crate::game::traits::Trait::*;
    let arch = if traits.contains(&Pious) { "devout" }
        else if traits.contains(&Kind) { "soft_heart" }
        else if traits.contains(&Ambitious) { "striver" }
        else if traits.contains(&Hotheaded) { "firebrand" }
        else if traits.contains(&Proud) { "climber" }
        else if traits.contains(&Gregarious) { "tongue_of_the_well" }
        else if traits.contains(&Stolid) { "old_guard" }
        else if traits.contains(&Wary) { "doubter" }
        else if traits.contains(&Lazy) { "merrymaker" }
        else { "quiet_hand" };
    let a = (draw(salt, 300) * 10.0) as usize % VALUES.len();
    let mut b = (draw(salt, 301) * 10.0) as usize % VALUES.len();
    if b == a { b = (b + 1) % VALUES.len(); }
    FactTable {
        open: vec![format!("archetype:{arch}"),
            format!("value:{}", VALUES[a].to_lowercase()),
            format!("value:{}", VALUES[b].to_lowercase())],
        hidden: vec![],
    }
}

#[derive(Debug, Clone)]
pub struct Speaker {
    pub name: String,
    pub traits: Vec<Trait>,
    pub ideology: &'static str,
    pub status: Vec<&'static str>,
    /// The speaker's trade ("farmer", "baker") - whose counsel weighs on
    /// which matter (#1238)
    pub trade: Option<&'static str>,
    /// How far their voice carries in the room (#1249): ~0.5 is nobody
    /// in particular; `speaker_of` reads it off the world, hand-made
    /// speakers default to a middling voice
    pub pull: f32,
    /// "male"/"female" when the world knows the body (E1-3, #1417) -
    /// the fact stream's pronoun source for `text.rs`'s {he}/{him}/{his}
    pub sex: Option<&'static str>,
    /// The persona's fact table (E3-1, #1423): `speaker_of` fills it
    /// off the world; hand-made speakers can carry a lite profile
    /// (`lite_facts`) or none
    pub facts: FactTable,
    /// How far they'll open to whoever they face (E3-2, #1424):
    /// `speaker_of` is pair-blind, so the caller who knows the bond
    /// patches it; a stranger's is the default - nothing reached
    pub open: Open,
    /// Thou or you to whoever they face (E4-2, #1414): patched per-pair
    /// like `open`; a hand-made speaker's default is you
    pub tv: crate::game::text::Tv,
    /// Their own archaism: 0 modern, 1 thou only as a marked shift, 2
    /// the full thou-pattern - `speaker_of` reads it off the persona
    pub archaism: u8,
    /// The bond to whoever they face - `{addr}`'s intimacy reads it
    pub rel: Relation,
    /// What they'd ask at the door (E5-3, #1435): the fear or need a
    /// doorknob worry keys off - the talk-open seeds it off the world;
    /// hand-made speakers carry none
    pub worry: Option<crate::game::desires::Reveal>,
    /// The leak they could make (E5-8, #1439): a strong ghost under a
    /// high-strung temperament shows through the words - `speaker_of`
    /// seeds it off the persona; hand-made speakers carry none
    pub slip: Option<Slip>,
    /// Whether the arc's one slip is still unspent (E5-8): the caller
    /// enforces the budget - 0 for hand-made speakers until a caller
    /// opts in; the transcript's `slips` reports what fired so the
    /// caller can mark the arc spent
    pub slips_left: u8,
    /// Their humor (E5-6, #1437): the style draw and the grievance its
    /// barbs lean on - `speaker_of` seeds it off the persona and
    /// traits; hand-made speakers carry none and joke the tree's lines
    pub joke: Option<Joke>,
    /// The ear's landmark (E4-7, #1416): a pet phrase and a habit,
    /// each unique in the village, each spent once a talk -
    /// `speaker_of` fills it off the persona
    pub idiolect: Option<crate::game::idiolect::Idiolect>,
    /// The story they'd tell if asked (E4-8, #1426): their deepest
    /// prehistory event and the frame their stake sets on it -
    /// the talk-open seeds it where the world can resolve the record's
    /// names; hand-made speakers carry none
    pub story: Option<crate::game::storytelling::StorySeed>,
}

impl Speaker {
    pub fn new(name: &str) -> Self {
        Speaker { name: name.to_string(), traits: vec![], ideology: "laborer", status: vec![], trade: None, pull: 0.8, sex: None, facts: FactTable::default(), open: Open::default(), tv: crate::game::text::Tv::You, archaism: 1, rel: Relation::Stranger, worry: None, slip: None, slips_left: 0, joke: None, idiolect: None, story: None }
    }
    /// Fact strings this speaker contributes ("trait:hotheaded", ...)
    fn own_facts(&self) -> Vec<String> {
        let mut v: Vec<String> = self.traits.iter().map(|t| format!("trait:{}", trait_name(*t))).collect();
        v.push(format!("ideology:{}", self.ideology));
        v.extend(self.status.iter().map(|s| format!("status:{}", s)));
        if let Some(sex) = self.sex { v.push(format!("sex:{}", sex)); }
        v.extend(self.facts.open.iter().cloned());
        // where the pair stands in each area (E3-2): the gate a topic's
        // tier reads - open facts carry no tell bonus; the ask side
        // names a seen tell and earns it on the spot
        let talkative = self.traits.contains(&Trait::Gregarious);
        let guarded = self.traits.contains(&Trait::Wary);
        for &a in AREAS {
            for t in 0..=3u8 {
                let reach = if self.open.reached(t, talkative, guarded, false)
                    { "open" } else { "closed" };
                v.push(format!("{}:{}:{}", reach, area_name(a), t));
            }
        }
        v
    }

    /// The areas a speaker's visible wear tells on (E3-2): a grief
    /// borne, a wound's quiet - the colored talk the asker already
    /// saw. Naming one is the ask that attention earns
    pub fn tell_areas(&self) -> Vec<Area> {
        let mut v: Vec<Area> = vec![];
        let mut mark = |a: Area| if !v.contains(&a) { v.push(a) };
        for st in &self.status {
            match *st {
                "grieving" => mark(Area::Heart),
                "sick" | "hungry" => mark(Area::Body),
                "keeps_secret" => mark(Area::Trouble),
                _ if st.starts_with("hook:") => { mark(Area::Trouble); mark(Area::Heart); }
                // a fact's visible habit (E3-4): the wear names the ask
                // its area hides behind
                _ => if let Some(c) = crate::game::fact_content::content_for_tell(st) {
                    mark(c.area);
                }
            }
        }
        v
    }
}

/// A leak the realizer can produce (E5-8, #1439): keyed to a strong
/// ghost under a high-strung temperament, at most once to a
/// relationship's whole arc - the caller passes the remaining budget
/// on `Speaker::slips_left` and reads what fired off `Transcript::slips`
#[derive(Debug, Clone, PartialEq)]
pub struct Slip {
    /// How prone they are, 0..1 - the ghost's hold folded with their
    /// neuroticism; the per-line roll scales off it so the landmark
    /// stays rare
    pub prone: f32,
    /// The one the ghost keeps, where the wound is a death: the
    /// name-slip confuses the listener with them, the tense slip keeps
    /// them in the present tense and catches itself
    pub dead: Option<Dead>,
}

/// The dead a slip names (E5-8): kin word, a name, the trade they
/// stood at - seeded once from the persona, or a real dead villager's
/// name when the world has one to give
#[derive(Debug, Clone, PartialEq)]
pub struct Dead {
    /// "brother" - said "my brother"
    pub kin: &'static str,
    pub name: String,
    /// "carter" - what they worked at
    pub trade: &'static str,
}

/// Which shape a slip took (E5-8), for the record and the lab's display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlipKind {
    /// The dead kept in the present tense, caught mid-word:
    /// "my brother is… was a carter"
    Tense,
    /// The listener called by the dead one's name
    Name,
    /// The sentence cut off where ears are near
    CutOff,
}

/// The ghost kinds whose wound is a death the speaker kept (E5-8):
/// the fever's row, the duty's dead, the betrothal's taken, the levy's
/// gone - only those carry a `dead` the name and tense slips can say
fn death_ghost(k: crate::game::persona::EventKind) -> bool {
    use crate::game::persona::EventKind::*;
    matches!(k, Fever | DeathDuty | BrokenBetrothal | Levy)
}

/// The kin word a slipped ghost puts on its dead (E5-8): a betrothal's
/// is the sweetheart, the rest draw on the family's own
const DEAD_KIN: &[&str] = &["brother", "sister", "husband", "wife",
    "mother", "father", "son", "daughter"];
const BETROTHAL_KIN: &[&str] = &["sweetheart", "betrothed"];
/// Dead names for a slip when the world has none to lend (E5-8)
const DEAD_NAMES: &[&str] = &["Corvin", "Edgar", "Wystan", "Hal", "Brannoc",
    "Anselm", "Osric", "Maud", "Elgiva", "Tilda", "Bess", "Wenna"];
/// The trade the tense slip gives the dead (E5-8)
const DEAD_TRADES: &[&str] = &["carter", "miller", "plowman", "weaver",
    "thatcher", "herdsman", "baker", "smith"];

/// The slip a persona could leak (E5-8, #1439): the strongest ghost
/// carries it, gated on a hard-held wound under a high-strung
/// temperament - the Big Five's N is `temperament[4]`. `dead_name`,
/// when the world gives one, is a real dead villager's name
pub fn slip_of(p: &crate::game::persona::Persona, dead_name: Option<String>) -> Option<Slip> {
    use crate::game::persona::EventKind::BrokenBetrothal;
    let g = p.ghosts.iter().max_by(|a, b| a.strength
        .partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))?;
    let nerve = p.temperament[4];
    if g.strength < 0.55 || nerve < 0.55 { return None; }
    let pick = |n: u64, pool: &[&'static str]| pool[(p.draw(n) * pool.len() as f32) as usize % pool.len()];
    let dead = if death_ghost(g.kind) {
        let kin = if g.kind == BrokenBetrothal { pick(91, BETROTHAL_KIN) }
            else { pick(91, DEAD_KIN) };
        Some(Dead { kin, name: dead_name.unwrap_or_else(|| pick(92, DEAD_NAMES).to_string()),
            trade: pick(93, DEAD_TRADES) })
    } else { None };
    Some(Slip { prone: (g.strength * (0.4 + 0.6 * nerve)).clamp(0.0, 1.0), dead })
}

/// Martin's four humor styles (E5-6, #1437): the style is how they
/// cope, the target's what they resent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JokeStyle {
    /// Bonds - the laugh that includes everyone
    Affiliative,
    /// Copes - laughing the hardship off
    SelfEnhancing,
    /// At another's expense - the barb
    Aggressive,
    /// At their own expense - the wound worn light
    SelfDefeating,
}

/// What a joke lands on (E5-6)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JokeTarget {
    Lord,
    Reeve,
    Priest,
    /// The speaker themselves - self-enhancing and self-defeating both
    /// aim here, kindly or not
    Oneself,
    Weather,
    /// A named feud or just "some folk" - the barb that isn't power
    Rival,
}

impl JokeTarget {
    /// Power's name: the targets the public channel can't carry - the
    /// done-when's "exposure above zero" is the talk's own channel
    pub fn power(self) -> bool {
        matches!(self, JokeTarget::Lord | JokeTarget::Reeve | JokeTarget::Priest)
    }
}

/// How a joke landed on its hearer (E5-6): the reply's fork - who
/// laughs shows who's aligned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JokeReceipt {
    /// The open laugh - alignment said out loud
    Laughed,
    /// Topped it with one of their own - alignment shown, and given back
    AddedTo,
    /// Warned it quiet - the word's dangerous, wherever they stand
    Warned,
    /// Let it pass - neither aligned nor against
    Quiet,
}

impl JokeReceipt {
    /// The discourse move the receipt turn wears
    fn act(self) -> Act {
        match self {
            JokeReceipt::Laughed => Act::Agree,
            JokeReceipt::AddedTo => Act::Joke,
            JokeReceipt::Warned => Act::PushBack,
            JokeReceipt::Quiet => Act::Remark,
        }
    }
    /// How the receipt receives the joke (E3-3): a warning is care for
    /// the speaker's neck; silence steps around it
    fn respond(self) -> Respond {
        match self {
            JokeReceipt::Laughed | JokeReceipt::AddedTo => Respond::Validate,
            JokeReceipt::Warned => Respond::Care,
            JokeReceipt::Quiet => Respond::Deflect,
        }
    }
    /// The set its line draws on
    fn lines(self) -> &'static [&'static str] {
        match self {
            JokeReceipt::Laughed => JOKE_LAUGHED,
            JokeReceipt::AddedTo => JOKE_ADDED,
            JokeReceipt::Warned => JOKE_WARNED,
            JokeReceipt::Quiet => JOKE_QUIET,
        }
    }
}

/// What a speaker resents (E5-6): the strongest ghost's lie, worded
/// for the {grievance} slot and aimed at the power it names
#[derive(Debug, Clone, Copy)]
pub struct Grievance {
    /// The target their aggressive jokes strike
    pub target: JokeTarget,
    /// The wound's word - fills the {grievance} slot ("the lord's wars")
    pub word: &'static str,
    /// The lie it came off: a hearer holding the same one is aligned
    pub lie: crate::game::persona::Belief,
}

/// A speaker's humor (E5-6): the draw weights over the four styles,
/// off their traits and temperament until the persona's own model
/// lands, and the grievance the barbs lean on
#[derive(Debug, Clone)]
pub struct Joke {
    /// [affiliative, self-enhancing, aggressive, self-defeating]
    pub styles: [f32; 4],
    /// The resentment the {grievance} slot and aggressive target read;
    /// none for the unwounded, whose barbs aim at whoever's about
    pub grievance: Option<Grievance>,
}

/// A joke said and received this talk (E5-6): who told it, what it
/// struck (matrix jokes only - a tree's own joke carries no target),
/// and how the hearer took it. The session reads it for alignment
#[derive(Debug, Clone, Copy)]
pub struct JokeBeat {
    pub joker: usize,
    pub style: Option<JokeStyle>,
    pub target: Option<JokeTarget>,
    pub receipt: JokeReceipt,
}

/// The grievance a lie makes (E5-6): the wound's word for the joke's
/// slot, aimed at whoever the lie blames. The true lessons - that we
/// were strong, that truth comes out - carry no grievance
fn grievance_of(l: crate::game::persona::Belief) -> Option<Grievance> {
    use crate::game::persona::Belief::*;
    use JokeTarget::*;
    Some(Grievance { lie: l, target: match l {
        LordAlwaysWins | WarsTakeSons | LordTakesFromDead => Lord,
        MillOwnsUs | NeverCrossReeve => Reeve,
        GodSends => Priest,
        PromisesForKeepers | KeepHidden => Oneself,
        SomeoneToBlame => Rival,
        StrongTogether | TruthComesOut => return None,
    }, word: match l {
        LordAlwaysWins => "the lord's men",
        WarsTakeSons => "the lord's wars",
        LordTakesFromDead => "the death-duty",
        MillOwnsUs => "the mill's take",
        NeverCrossReeve => "the reeve's count",
        GodSends => "God's sending",
        PromisesForKeepers => "my own word",
        KeepHidden => "the truth of me",
        SomeoneToBlame => "the blame",
        StrongTogether | TruthComesOut => return None,
    } })
}

/// The lies a ghost teaches about the self's worth (E5-6): while one
/// holds strong, the humor turns inward - the self-defeating joke is
/// the wound worn where the village can see it
fn self_worth_lie(l: crate::game::persona::Belief) -> bool {
    matches!(l, crate::game::persona::Belief::KeepHidden
        | crate::game::persona::Belief::PromisesForKeepers)
}

/// The humor a persona carries (E5-6, #1437): traits set the styles -
/// the sociable bond, the proud cope, the hothead barbs - and a strong
/// self-worth lie sharpens the self-defeating edge. `None` for nobody:
/// everyone jokes some, so an absent persona is the only empty profile
pub fn joke_of(p: &crate::game::persona::Persona, traits: &[Trait]) -> Joke {
    let mut styles = [0.8f32, 0.3, 0.15, 0.15];
    for t in traits {
        match t {
            Trait::Gregarious | Trait::Kind => styles[0] += 0.6,
            Trait::Pious => styles[0] += 0.2,
            Trait::Proud | Trait::Ambitious | Trait::Steady => styles[1] += 0.5,
            Trait::SelfAbsorbed => styles[1] += 0.3,
            Trait::Hotheaded | Trait::Cruel => styles[2] += 0.9,
            Trait::Stingy => styles[2] += 0.2,
            Trait::Lazy | Trait::Stolid => styles[3] += 0.2,
            _ => {}
        }
    }
    // the wound's weight: a self-worth lie turns the humor inward
    for g in &p.ghosts {
        if self_worth_lie(g.lie) { styles[3] += g.strength * 1.2; }
    }
    // a wary one jokes less - the whole draw thins
    if traits.contains(&Trait::Wary) { styles.iter_mut().for_each(|s| *s *= 0.5); }
    let grievance = p.ghosts.iter()
        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
        .and_then(|g| grievance_of(g.lie));
    Joke { styles, grievance }
}

pub fn trait_name(t: Trait) -> &'static str {
    match t {
        Trait::Lazy => "lazy",
        Trait::Ambitious => "ambitious",
        Trait::Kind => "kind",
        Trait::Stingy => "stingy",
        Trait::Pious => "pious",
        Trait::Hotheaded => "hotheaded",
        Trait::Gregarious => "gregarious",
        Trait::Proud => "proud",
        Trait::Steady => "steady",
        Trait::Wary => "wary",
        Trait::Stolid => "stolid",
        Trait::Cruel => "cruel",
        Trait::SelfAbsorbed => "self-absorbed",
    }
}

/// A subject area the per-area openness gate knows (E3-2, #1424):
/// one reach serves every area today - the same bond parts feed each -
/// but the gate is per-area so later work can weigh one differently
/// (the lord's name, a burned past)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    Village,
    Work,
    Body,
    Road,
    Heart,
    Trouble,
}

/// The areas in order, for fact emission
pub const AREAS: &[Area] = &[
    Area::Village, Area::Work, Area::Body, Area::Road, Area::Heart, Area::Trouble,
];

pub fn area_name(a: Area) -> &'static str {
    match a {
        Area::Village => "village",
        Area::Work => "work",
        Area::Body => "body",
        Area::Road => "road",
        Area::Heart => "heart",
        Area::Trouble => "trouble",
    }
}

/// What feeds a pair's openness to the speaker carrying it (E3-2,
/// #1424): the bond's parts (E6-1), last talk's mood, the talks behind
/// them, and whether ears are near. A `Speaker` carries it toward
/// whoever they face; a stranger's `Open` is the default - nothing
/// reached, nothing shared
#[derive(Debug, Default, Clone, Copy)]
pub struct Open {
    /// The bond's intimacy - breadth and depth reached together
    pub intimacy: f32,
    /// Mayer's benevolence - they mean you well
    pub benevolence: f32,
    /// Mayer's integrity - they keep their word
    pub integrity: f32,
    /// The safety rung as 0..1 (E6-1's Clark ladder)
    pub safety: f32,
    /// Last talk's mood, -1..1 (the `TalkMemory`)
    pub mood: f32,
    /// Talks had together - tier 1's "after two talks" gate
    pub talks: u32,
    /// Someone can hear: the exposure term, and tier 3's hard bar
    pub earshot: bool,
    /// What the asker's given back of themselves (E3-3): the count of
    /// their disclosures that landed. Private wants one, the core two -
    /// openness without reciprocation stalls at the personal layer
    pub shared: u8,
}

impl Open {
    /// Naming a tell you actually saw earns this much (E3-2):
    /// attention, not persistence
    pub const TELL_BONUS: f32 = 0.2;

    /// The per-area openness (E3-2's formula): the same inputs serve
    /// every area for now - areas differ when their content lands
    pub fn reach(&self) -> f32 {
        0.30 * self.intimacy + 0.25 * self.benevolence + 0.20 * self.integrity
            + 0.15 * self.safety + 0.10 * self.mood
            - if self.earshot { 0.20 } else { 0.0 }
    }

    /// Whether `tier` stands open for this pair: public always,
    /// personal after two talks, private at openness .4, core at .7
    /// unheard. Temperament scales it (the design's extraversion/
    /// neuroticism): a talkative speaker lowers the personal bar to
    /// one talk; a guarded one raises the core bar. `tell` names the
    /// bonus a seen tell earns
    pub fn reached(&self, tier: u8, talkative: bool, guarded: bool, tell: bool) -> bool {
        match tier {
            0 => true,
            // a tell substitutes for warmth, not for time spent: the
            // personal layer still wants the talks behind it
            1 => self.talks >= if talkative { 1 } else { 2 },
            // private wants warmth *and* a piece of yourself given back
            2 => self.shared >= 1
                && self.reach() + if tell { Self::TELL_BONUS } else { 0.0 } >= 0.4,
            // the core wants it mutual: two landed disclosures, unheard
            _ => self.shared >= 2 && !self.earshot
                && self.reach() + if tell { Self::TELL_BONUS } else { 0.0 }
                    >= if guarded { 0.8 } else { 0.7 },
        }
    }
}

/// What stands between two speakers. Folds into the pair's facts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Relation {
    Stranger,
    Acquaintance,
    Friend,
    Kin,
    Rival,
}

fn relation_name(r: Relation) -> &'static str {
    match r {
        Relation::Stranger => "stranger",
        Relation::Acquaintance => "acquaintance",
        Relation::Friend => "friend",
        Relation::Kin => "kin",
        Relation::Rival => "rival",
    }
}

fn kind_name(k: RelKind) -> &'static str {
    match k {
        RelKind::Stranger => "stranger",
        RelKind::Acquaintance => "acquaintance",
        RelKind::Friend => "friend",
        RelKind::Rival => "rival",
        RelKind::Family => "family",
        RelKind::Lover => "lover",
        RelKind::Mentor => "mentor",
        RelKind::Apprentice => "apprentice",
    }
}

/// The world state a rule is matched against (Ruskin's "query"), flattened
/// to strings so criteria stay declarative and content stays data.
/// - `self:*` facts are about the speaker being evaluated for
/// - `other:*` facts are about the person they're talking to
/// - `any:*`  facts hold if either of the pair carries them
/// - `pair:*` facts are about the relation between them
#[derive(Debug, Default)]
pub struct Facts {
    set: HashSet<String>,
}

impl Facts {
    /// Facts for `speaker` choosing a line while talking to `other`
    pub fn for_roles(speaker: &Speaker, other: &Speaker, rel: Relation) -> Facts {
        let mut f = Facts::default();
        // the hidden half is the speaker's own until disclosed (E3-1)
        for x in speaker.own_facts().into_iter().chain(speaker.facts.hidden.iter().cloned()) {
            f.set.insert(format!("self:{}", x));
        }
        for x in other.own_facts() { f.set.insert(format!("other:{}", x)); }
        for x in speaker.own_facts().into_iter().chain(other.own_facts()) {
            f.set.insert(format!("any:{}", x));
        }
        f.set.insert(format!("pair:relation:{}", relation_name(rel)));
        f
    }

    /// `for_roles`, plus the pair's finer bond as `pair:kind:*` when the
    /// session knows it (#1372): the coarse relation still carries the
    /// cascade, the fine kind lets lovers, mentors, apprentices and
    /// family be heard as themselves
    pub fn for_roles_kind(speaker: &Speaker, other: &Speaker, rel: Relation, kind: Option<RelKind>) -> Facts {
        let mut f = Facts::for_roles(speaker, other, rel);
        if let Some(k) = kind { f.set.insert(format!("pair:kind:{}", kind_name(k))); }
        f
    }

    /// `for_roles_kind`, plus the talk's setting as `pair:place:*` /
    /// `pair:channel:*` (E5-9, #1446): trees and lines read which
    /// transcript the room asks for
    pub fn for_roles_setting(speaker: &Speaker, other: &Speaker, rel: Relation, kind: Option<RelKind>, setting: &crate::game::setting::Setting) -> Facts {
        let mut f = Facts::for_roles_kind(speaker, other, rel, kind);
        for fact in setting.facts() { f.set.insert(fact); }
        f
    }

    pub fn has(&self, fact: &str) -> bool { self.set.contains(fact) }

    /// Add a fact by hand - for a query built from something other than
    /// two speakers (the campaign decision library reads campaign state)
    pub fn insert(&mut self, fact: impl Into<String>) { self.set.insert(fact.into()); }
}

/// One test on the facts: all of a rule's criteria must hold for it to be
/// eligible. Criterion count is also the rule's specificity score (Ruskin).
#[derive(Debug, Clone, Copy)]
pub enum Criterion {
    /// The fact must be present
    Has(&'static str),
    /// The fact must be absent
    Lacks(&'static str),
}

impl Criterion {
    pub fn holds(&self, f: &Facts) -> bool {
        match self {
            Criterion::Has(x) => f.has(x),
            Criterion::Lacks(x) => !f.has(x),
        }
    }
}

/// What a line does in the exchange (Facade's discourse acts, slimmed).
/// Replies stay coherent because they answer the *act*, not the words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Act {
    Greet,
    Remark,
    Question,
    Answer,
    Agree,
    PushBack,
    Farewell,
    /// A question carrying a fear, usually at the door (E5-1, #1430)
    Worry,
    /// A small reach for attention; a safety bid tests before telling
    Bid,
    /// Style and target from the persona, gated by setting
    Joke,
    /// A job tied to a ghost; extends organizing::Deeds
    Errand,
    /// A visible refusal of a topic above its tier
    Evade,
    /// A leak the realizer produces, at most once per arc
    Slip,
}

/// How a reply receives what was just said (E3-3, #1428): the
/// responsiveness read `chat_temp` weighs. Authors may tag a `Reply`
/// directly; untagged ones derive from their act (`respond_of`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Respond {
    /// Takes up what was said and hands it back heard
    Reflect,
    /// Lets it stand - agreement, acknowledgment
    Validate,
    /// Meets it with warmth or worry for the speaker
    Care,
    /// Answers it with a way forward
    Advise,
    /// Steps around it
    Deflect,
    /// Turns it into a question of their own
    PivotAsk,
}

/// The responsiveness an untagged reply carries by default (E3-3):
/// an answer reflects, agreement validates, a question back pivots
pub fn respond_of(act: Act) -> Respond {
    match act {
        Act::Answer => Respond::Reflect,
        Act::Agree | Act::Remark | Act::Greet | Act::Farewell | Act::Bid | Act::Joke => Respond::Validate,
        Act::Errand => Respond::Advise,
        Act::Question | Act::Worry => Respond::PivotAsk,
        // a leak undercuts the answer it rides on - a misstep, not
        // warmth (E5-1)
        Act::PushBack | Act::Evade | Act::Slip => Respond::Deflect,
    }
}

/// A player disclosure settling or not (E3-3): each lands a step
/// deeper than the last, and only as deep as the pair's openness
/// holds - a civil greeting holds a first telling; what goes in past
/// the bond reads as strange
fn note_disclosure(tr: &mut Transcript, open: &mut Open) {
    if tr.shared >= 3 { return; }
    let depth = tr.shared + 1;
    if open.reach() >= depth as f32 * 0.2 {
        tr.shared = depth;
        // the landed telling deepens the running pair-open mid-talk:
        // later beats (rebuilt asks, replies) gate on it - the stored
        // snapshot stays untouched so replays keep earlier picks valid
        open.shared = open.shared.max(depth);
    } else { tr.overshot = true; }
}

/// The doorknob (E5-3, #1435): a goodbye's when the real question
/// comes out - "one thing, before you go". The caller may fire one
/// last worry question; the player answers plainly, reassures, or
/// asks why - and the why opens the tier whenever the pair's openness
/// allows, a door that stays shut counting as an evasion's tell
/// (E5-2). Once a talk, and only once the talk's had its say.
/// True when the response menu's standing - the walk pauses for it
fn worry_beat(
    tr: &mut Transcript,
    pair: &mut [Speaker; 2],
    caller: usize,
    responder: usize,
    channel: crate::game::setting::Channel,
    used: &mut HashSet<&'static str>,
    recent: &HashSet<&'static str>,
    choose: &mut dyn FnMut(&[Choice]) -> Option<usize>,
    rng: &mut Rng,
) -> bool {
    if tr.trees_walked.contains(&WORRY_TREE) || tr.turns.len() < 4 { return false; }
    let Some(r) = pair[caller].worry else { return false };
    let Some(w) = worry_of_reveal(r) else { return false };
    if rng.next_f32() >= 0.55 { return false; }
    tr.trees_walked.push(WORRY_TREE);
    let (qt, _) = fresh_line(w.asks, &pair[caller], &pair[responder], used, recent, false, rng)
        .unwrap_or((w.asks[0], String::new()));
    used.insert(qt);
    tr.turns.push(Turn { channel, speaker: caller, act: Act::Worry, respond: None,
        text: fill(qt, &pair[caller], &pair[responder], rng),
        concepts: vec!["worry"], tree: WORRY_TREE, line: Some(qt) });
    let say = |set: &[&'static str], sp: usize,
                   used: &mut HashSet<&'static str>, rng: &mut Rng| {
        let (t, _) = fresh_line(set, &pair[sp], &pair[1 - sp], used, recent, false, rng)
            .unwrap_or((set[0], String::new()));
        used.insert(t);
        t
    };
    match choose(&[
        Choice { act: Act::Answer, respond: Respond::Validate,
            preview: "Answer plainly".into() },
        Choice { act: Act::Agree, respond: Respond::Care,
            preview: "Reassure them".into() },
        Choice { act: Act::Question, respond: Respond::PivotAsk,
            preview: "Ask why".into() },
    ]) {
        None => return true,
        Some(0) => {
            let t = say(WORRY_ANSWERS, responder, used, rng);
            tr.turns.push(Turn { channel, speaker: responder, act: Act::Answer,
                respond: Some(Respond::Validate),
                text: fill(t, &pair[responder], &pair[caller], rng),
                concepts: vec!["worry"], tree: WORRY_TREE, line: Some(t) });
        }
        Some(1) => {
            let t = say(WORRY_SOOTHE, responder, used, rng);
            tr.turns.push(Turn { channel, speaker: responder, act: Act::Agree,
                respond: Some(Respond::Care),
                text: fill(t, &pair[responder], &pair[caller], rng),
                concepts: vec!["worry"], tree: WORRY_TREE, line: Some(t) });
        }
        Some(2) => {
            let t = say(WORRY_WHY, responder, used, rng);
            tr.turns.push(Turn { channel, speaker: responder, act: Act::Question,
                respond: Some(Respond::PivotAsk),
                text: fill(t, &pair[responder], &pair[caller], rng),
                concepts: vec!["worry"], tree: WORRY_TREE, line: Some(t) });
            let talkative = pair[caller].traits.contains(&Trait::Gregarious);
            let guarded = pair[caller].traits.contains(&Trait::Wary);
            if pair[caller].open.reached(w.tier, talkative, guarded, true) {
                // the tier opens (E5-3): the fear or need's told in its
                // own words - the session hears it onto the ledger
                let t = say(w.truths, caller, used, rng);
                tr.turns.push(Turn { channel, speaker: caller, act: Act::Answer,
                    respond: Some(Respond::Reflect),
                    text: fill(t, &pair[caller], &pair[responder], rng),
                    concepts: vec!["worry"], tree: WORRY_TREE, line: Some(t) });
                tr.worry_told = Some(r);
                note_disclosure(tr, &mut pair[caller].open);
            } else {
                // the door stays shut - the turn-aside's the tell
                // (E5-2): it counts toward their silhouette
                tr.evasions.push((caller, w.area));
                let t = say(evade_set(&pair[caller]), caller, used, rng);
                tr.turns.push(Turn { channel, speaker: caller, act: Act::Evade,
                    respond: Some(Respond::Deflect),
                    text: fill(t, &pair[caller], &pair[responder], rng),
                    concepts: vec!["worry"], tree: WORRY_TREE, line: Some(t) });
            }
        }
        _ => {}
    }
    false
}

/// A line said (E5-8, #1439): pushes the turn, then the slip pass
/// rolls - a speaker with a profile and budget left may leak through
/// the words just said: the turn keeps its spent line, its text gets
/// corrupted, and its act marks the slip. The roll rides its own rng
/// stream (`say`'s caller forks it once a walk) so the same turns
/// roll the same way on every replay, however many picks stand
fn say(
    tr: &mut Transcript,
    pair: &[Speaker; 2],
    slip_rng: &mut Rng,
    slipped: &mut bool,
    t: Turn,
) {
    tr.turns.push(t);
    let t = tr.turns.last_mut().unwrap();
    let sp = t.speaker;
    if *slipped || pair[sp].slips_left == 0 || pair[sp].slip.is_none() { return; }
    if !matches!(t.act, Act::Remark | Act::Answer | Act::Agree
        | Act::Question | Act::PushBack) || t.tree == WORRY_TREE { return; }
    let profile = pair[sp].slip.clone().unwrap();
    if slip_rng.next_f32() >= SLIP_RATE * profile.prone { return; }
    let Some(kind) = slip_corrupt(t, &pair[1 - sp].name, &profile.dead, slip_rng) else { return };
    t.act = Act::Slip;
    t.concepts = vec!["slip"];
    *slipped = true;
    tr.slips.push((sp, kind));
}

/// The rareness of a leak (E5-8): per eligible line, scaled by the
/// speaker's prone - a hard-held ghost under a high-strung temperament
/// shows roughly once an arc
const SLIP_RATE: f32 = 0.012;

/// Where `name` stands as its own word in `text` (E5-8): the name
/// slip confuses a person, not a name that rides inside a longer one
/// ("Ash" in "Ashby" isn't the man)
fn word_at(text: &str, name: &str) -> Option<usize> {
    text.match_indices(name).find(|&(i, _)| {
        let before = i == 0 || !text.as_bytes()[i - 1].is_ascii_alphanumeric();
        let after = i + name.len() == text.len()
            || !text.as_bytes()[i + name.len()].is_ascii_alphanumeric();
        before && after
    }).map(|(i, _)| i)
}

/// Corrupt the just-said words into one of the slip's shapes (E5-8):
/// the listener called by the dead one's name, the dead kept in the
/// present tense and caught mid-word, or the sentence cut off where
/// the channel says ears are near. The pick rotates on a seed draw
/// and falls back through whichever shape the words can carry
fn slip_corrupt(t: &mut Turn, listener: &str, dead: &Option<Dead>, rng: &mut Rng) -> Option<SlipKind> {
    use SlipKind::*;
    let order: &[SlipKind] = match rng.below(3) {
        0 => &[Name, Tense, CutOff],
        1 => &[Tense, CutOff, Name],
        _ => &[CutOff, Name, Tense],
    };
    for &kind in order {
        match kind {
            Name => {
                let Some(d) = dead else { continue };
                let Some(at) = word_at(&t.text, listener) else { continue };
                t.text.replace_range(at..at + listener.len(), &d.name);
                return Some(Name);
            }
            CutOff => {
                if t.channel != crate::game::setting::Channel::Public
                    || t.text.len() < 24 { continue; }
                // the word dies halfway: cut where the speech is thickest,
                // back off to a word's edge
                let at = (t.text.len() * (40 + rng.below(30)) / 100)
                    .min(t.text.len() - 1);
                let cut = t.text[..at].rfind(' ').unwrap_or(at);
                t.text.truncate(cut);
                t.text.push_str(" -");
                return Some(CutOff);
            }
            Tense => {
                let Some(d) = dead else { continue };
                // the dead's kin word said in the present tense catches
                // itself; else the kept thing surfaces whole after the
                // line - "my brother Corvin is... was a carter"
                if let Some(ki) = word_at(&t.text, d.kin) {
                    if let Some(ci) = t.text[ki..].find(" is ").filter(|&ci| ci < 60) {
                        t.text.replace_range(ki + ci..ki + ci + 4, " is... was ");
                        return Some(Tense);
                    }
                }
                t.text = format!("{} My {} {} is... was a {}.",
                    t.text.trim_end(), d.kin, d.name, d.trade);
                return Some(Tense);
            }
        }
    }
    None
}

/// How a conversation closes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// A polite parting
    Farewell,
    /// One of them turned and left mid-talk
    WalksOff,
    /// They ran out of things to say and drifted apart
    Exhausted,
    /// The player chose to leave (converse_with only)
    LeavesEarly,
}

/// One fork a responder can take: a weighted, criteria-gated reply that
/// may drop the conversation a level (`then`) or close it (`end`).
/// All content is static so trees can live in `const`s.
pub struct Reply {
    /// Facts the responder must satisfy for this fork to exist at all -
    /// how a sick or grieving speaker *gains* a path nobody else has
    pub criteria: &'static [Criterion],
    /// Base draw weight at this fork
    pub weight: f32,
    /// (trait, multiplier) applied to the weight when the speaker has it -
    /// the personality bias (CiF's social considerations)
    pub bias: &'static [(Trait, f32)],
    /// What this line does in the exchange
    pub act: Act,
    /// How it receives the call (E3-3): `None` derives from the act
    pub respond: Option<Respond>,
    /// What it's about; a house meeting threads on these
    pub concepts: &'static [&'static str],
    /// Ways to say it; `{other}` is filled with the listener's name.
    /// Once-only: a variant is spent when spoken (Versu's exclusivity)
    pub lines: &'static [&'static str],
    /// Relation-keyed alternates for this beat - warmer or colder ways to
    /// say the same thing. A matching voice outranks `lines` until spent
    pub voice: &'static [Voice],
    /// The level below, if this fork takes the talk deeper
    pub then: Option<Exchange>,
    /// If set, speaking this line ends the conversation
    pub end: Option<Ending>,
}

/// One register a beat can speak in: when the speaker's facts satisfy
/// `criteria`, these variants say the line in that register's voice -
/// a relation's warmth, a rank's address, whatever a fact can name.
pub struct Voice {
    /// Facts the *speaker's* facts must satisfy for this register
    /// (`self:` is the one speaking; `pair:`/`other:` as in for_roles)
    pub criteria: &'static [Criterion],
    /// Alternate phrasings in that register's voice
    pub lines: &'static [&'static str],
}

/// One call/response level: whoever holds the floor speaks a variant of
/// `calls`; the other picks among `replies`. An exchange with no replies
/// is just a beat - the talk moves on to whoever speaks next.
pub struct Exchange {
    /// What the calls do / are about
    pub act: Act,
    pub concepts: &'static [&'static str],
    /// Ways to open this beat; `{other}` fills with the listener's name
    pub calls: &'static [&'static str],
    /// Relation-keyed alternates for the calls - the register a friend or
    /// a rival opens this beat in
    pub voice: &'static [Voice],
    /// The forks the other side can take
    pub replies: &'static [Reply],
}

/// A whole dialogue tree plus the circumstances it speaks to. The library
/// holds many; the most specific match leads (Ruskin: specificity is the
/// criterion count; library order breaks ties).
#[derive(Clone)]
pub struct DialogueTree {
    pub name: &'static str,
    pub criteria: &'static [Criterion],
    /// Where the tree starts when it opens the conversation
    pub root: &'static Exchange,
    /// Where the tree is joined when the walk falls through to it
    /// mid-conversation - the generic tail skips re-greeting and picks up
    /// at the chit-chat
    pub rejoin: Option<&'static Exchange>,
}

/// A thing said and recorded
#[derive(Debug)]
pub struct Turn {
    /// Index into the speakers pair (0 = initiator)
    pub speaker: usize,
    pub act: Act,
    /// The reply's responsiveness tag, resolved at the pick (E3-3);
    /// calls and fixed ask-path turns carry theirs or none
    pub respond: Option<Respond>,
    pub text: String,
    pub concepts: Vec<&'static str>,
    /// Which tree the line came from - handy for a grader
    pub tree: &'static str,
    /// The template the text was filled from, when one was - the village's
    /// freshness memory (#1369) counts templates, not rendered words
    pub line: Option<&'static str>,
    /// Which transcript the line was said in (E5-9, #1446): the public
    /// one or the backstage one, off the talk's setting
    pub channel: crate::game::setting::Channel,
}

/// The full record of a two-person conversation
#[derive(Debug)]
pub struct Transcript {
    pub speakers: [String; 2],
    /// Which trees the cascade walked, in order
    pub trees_walked: Vec<&'static str>,
    pub turns: Vec<Turn>,
    pub ending: Option<Ending>,
    /// How the audition went, when the caller probed the responder
    /// (#1187): a good answer reads safe, a wrong note a mismatch -
    /// a compatibility read, not a grudge
    pub vibe: Option<Vibe>,
    /// The labels of the topics the player asked after (#1189), in
    /// order - the session diffs them for what a question drew out
    pub asked: Vec<&'static str>,
    /// How many questions went in above the pair's openness and came
    /// back turned aside (E3-2): each costs the bond a rung of nerve
    pub forward: u8,
    /// Which closed area each turn-aside stood over (E5-2, #1434):
    /// (the evader's pair index, the topic's area) - the seat's ledger
    /// counts them toward a silhouette
    pub evasions: Vec<(usize, Area)>,
    /// The doorknob worry that opened its tier (E5-3, #1435): the why
    /// asked and the pair's openness held, so the fear or need got
    /// told - the session hears it onto the seat's ledger
    pub worry_told: Option<crate::game::desires::Reveal>,
    /// The leaks the realizer made this talk (E5-8, #1439): (who
    /// slipped, which shape) - the caller marks its arc budget spent
    /// off this, one slip to a relationship's whole run of talks
    pub slips: Vec<(usize, SlipKind)>,
    /// The jokes this talk told and how they landed (E5-6, #1437):
    /// who joked, what it struck, and the hearer's receipt - who
    /// laughs shows who's aligned
    pub jokes: Vec<JokeBeat>,
    /// The depth of yourself the responder gave and the pair could hold
    /// (E3-3): each landed disclosure goes a step deeper; the session
    /// keeps the deepest reached so later asks find it mutual
    pub shared: u8,
    /// A disclosure went in past what the pair could hold (E3-3): a
    /// tier-3 confession to a stranger reads as strange
    pub overshot: bool,
    /// Where the talk stood and who could hear (E5-9, #1446): the
    /// setting every turn's channel tag came off
    pub setting: crate::game::setting::Setting,
}

/// What the trust-probe read off the answer (#1187)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vibe {
    /// The answer landed - they open up
    Safe,
    /// It didn't - polite, but the talk stays shallow
    Mismatch,
}

/// A trait's safety test (#1187): early in a talk the caller probes the
/// responder with the question their own character asks of the world -
/// the stingy test generosity, the pious faith, the hotheaded whether
/// you'll stand your ground. `pass` resumes the talk where it was;
/// `fail` leaves it on the polite tail.
pub struct Probe {
    /// The trait that carries this test
    pub who: Trait,
    /// What the call does / is about
    pub act: Act,
    pub concepts: &'static [&'static str],
    /// The testing line - a question, a bid, a challenge
    pub calls: &'static [&'static str],
    /// The answer that lands, and the act it answers with
    pub pass: (Act, &'static [&'static str]),
    /// The wrong note, and the act it answers with
    pub fail: (Act, &'static [&'static str]),
}

/// How the exchange landed on the pair (#1230), -1..1: push-backs cool,
/// agrees and answers warm, and the close carries the most weight - a
/// walk-off sours where a kind parting leaves it well. One sharp word in
/// a long easy talk isn't a spat; the score only goes cold when the cold
/// acts outnumber the kind ones.
pub fn chat_temp(tr: &Transcript) -> f32 {
    let mut score = 0i32;
    for t in &tr.turns {
        // a tagged reply reads on its responsiveness (E3-3): care and
        // reflection warm the talk, a deflection or a pivot cools it;
        // untagged turns score on their act as they ever did
        score += match t.respond {
            Some(Respond::Care) => 2,
            Some(Respond::Reflect | Respond::Validate | Respond::Advise) => 1,
            Some(Respond::Deflect | Respond::PivotAsk) => -1,
            None => match t.act {
                Act::PushBack => -1,
                Act::Agree | Act::Answer => 1,
                // the reveal acts (E5-1): a reach warms, a pestered
                // fear, a turned-aside topic or a leaked word cools
                Act::Bid | Act::Joke | Act::Errand => 1,
                Act::Worry | Act::Evade | Act::Slip => -1,
                _ => 0,
            },
        };
    }
    score += match tr.ending {
        Some(Ending::WalksOff) => -2,
        Some(Ending::Farewell) => 1,
        _ => 0,
    };
    (score as f32 / 3.0).clamp(-1.0, 1.0)
}

/// The pair's hidden constant (#1230): two folk just get along or just
/// don't, a little - hashed off the pair's ids, no stored state, nothing
/// an event can shift. -1..1; a warm read means their kindnesses land a
/// touch heavier and their slights a touch lighter.
pub fn pair_bias(a: Entity, b: Entity) -> f32 {
    let (lo, hi) = (a.min(b) as u64, a.max(b) as u64);
    let h = lo.wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ hi.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let h = h ^ (h >> 33);
    (h % 2001) as f32 / 1000.0 - 1.0
}

/// The tree name the audition beat reports as (#1187)
pub const PROBE_TREE: &str = "the_audition";

/// The tree name your own questions report as (#1189)
pub const ASK_TREE: &str = "the_asking";

/// A too-deep ask, turned aside (E3-2): the refusal is seen, the talk
/// goes on - and a tell seen another day may yet open it
/// The visible turn-aside an over-deep ask earns (E3-2); E5-1 tags the
/// act `Evade` and the same set answers gossip it shouldn't
pub const DEFLECTED: &[&str] = &[
    "That's not for the telling, friend.",
    "We don't know each other so well as that.",
    "Ask me another.",
    "Some things keep.",
];

/// How a temperament turns the same ask aside (E5-2, #1434): the
/// hothead snaps, the sociable change the subject, the pious hand it
/// to God, the proud answer the question you didn't put
const EVADE_SNAP: &[&str] = &[
    "That's a nose where it doesn't belong.",
    "Keep {2:poss} questions to {2:self}.",
    "Ask the wind - it'll tell {2:obj} as much.",
];
const EVADE_CHANGE: &[&str] = &[
    "Now that reminds me - there's a thing I meant to say.",
    "Ha! Never mind that - how's {2:poss} own day gone?",
    "We'll talk of happier things, {2:subj} and I.",
];
const EVADE_GOD: &[&str] = &[
    "That's God's business, not ours.",
    "The good Lord keeps some books to Himself.",
    "Let it lie - what's hid is hid for a reason.",
];
const EVADE_PIVOT: &[&str] = &[
    "What {2:subj} ought to ask is a different matter.",
    "I'll tell {2:obj} a worthier thing than that.",
    "That's no question for the likes of me.",
];

/// Every evade set, for the text audit's authored-line census (E5-2)
#[doc(hidden)]
pub const EVADE_SETS: &[&[&str]] = &[DEFLECTED, EVADE_SNAP, EVADE_CHANGE, EVADE_GOD, EVADE_PIVOT];

/// The authored jokes (E5-6): style by target, no generation - the
/// choice of joke carries the personality. `{grievance}` slots fill
/// from the speaker's own grievance word; lines without one stand for
/// anyone. Cells the style can't voice go unwritten
const JOKES: &[(JokeStyle, JokeTarget, &[&str])] = &[
    // affiliative - the laugh that includes everyone
    (JokeStyle::Affiliative, JokeTarget::Weather, &[
        "I asked the sky for a dry week and it sent two wet ones, to be sure I meant it.",
        "Rain again - the cabbages at least think kindly of somebody.",
        "They say it clears by Sunday. They said the same last Sunday, and the one before.",
        "A fine day for ducks, my mother called every day that drowned us.",
    ]),
    (JokeStyle::Affiliative, JokeTarget::Lord, &[
        "The lord's table never wants - comforting to know somebody's fed, I always say.",
        "I hear the great hall's roof leaks too. Even stone gets rained on.",
        "May the lord live long and merry - a new one'd only learn the taxes fresh.",
    ]),
    (JokeStyle::Affiliative, JokeTarget::Reeve, &[
        "The reeve counts honest, for a man whose master wrote the numbers.",
        "I don't envy the reeve - he has to look us in the face while he does it.",
        "Poor reeve - the only man in the village nobody invites to drink.",
    ]),
    (JokeStyle::Affiliative, JokeTarget::Priest, &[
        "The sermon ran long again - but the bench is soft, and a body's feet are grateful.",
        "The candles outlasted the sermon once. Once.",
        "The church roof drips on the good people's heads the same as mine - we're level in the wet.",
    ]),
    (JokeStyle::Affiliative, JokeTarget::Oneself, &[
        "I'm a man of simple tastes - bread, ale, and a fire somebody else paid for.",
        "I've a gift for joining the slow line - a talent, I tell the wife.",
        "My memory's a marvel - it forgets the work and keeps the grudges.",
    ]),
    (JokeStyle::Affiliative, JokeTarget::Rival, &[
        "I won't hear a word against my old foe - nobody else will talk to him either.",
        "We argue like brothers, which is to say we share nothing but the argument.",
        "He's a good man, my rival - asks him the day he owes and he's suddenly deaf.",
    ]),
    // self-enhancing - laughing the hardship off
    (JokeStyle::SelfEnhancing, JokeTarget::Weather, &[
        "Wet feet dry. That's what I tell the wet feet.",
        "Cold keeps the fleas down - there's a mercy hid in everything, my gran used to say.",
        "It rains on the reeve's roof the same as mine - we're level in the wet.",
    ]),
    (JokeStyle::SelfEnhancing, JokeTarget::Lord, &[
        "The lord has his hall and I've my health - I call it near even.",
        "He can tax the grain. The jokes he can't reach.",
        "They keep the hall warm and me honest - each to his own fire.",
    ]),
    (JokeStyle::SelfEnhancing, JokeTarget::Reeve, &[
        "The reeve's count don't frighten me - I've counted worse. My own debts, mostly.",
        "He takes the tenth. I keep the jokes - I know which weighs lighter.",
    ]),
    (JokeStyle::SelfEnhancing, JokeTarget::Oneself, &[
        "Fortune owes me a debt - I keep the tally patient.",
        "I don't lack much - just the means to fix most of it.",
        "Thin times make a man quick. I've never been quicker.",
    ]),
    // aggressive - the barb. Power's names are for where power can't
    // hear: the channel, not the courage, is the gate
    (JokeStyle::Aggressive, JokeTarget::Lord, &[
        "The lord's justice: a rope for the man who steals bread, a feast for the man who steals fields.",
        "They say the lord loves us like sons - he takes from us like it too.",
        "The lord's men win every fight they start - brave souls, against a man holding a hoe.",
        "What's between the lord and the weather? The weather takes no thanks.",
        "{grievance} - the lord collects even what we haven't got.",
    ]),
    (JokeStyle::Aggressive, JokeTarget::Reeve, &[
        "The reeve's arithmetic: two of yours, three of his - it always balances.",
        "Give the reeve an honest answer and he'll count it twice and charge you for one.",
        "The reeve knows every store down to the last onion - a gift, truly. Like a rat's.",
        "{grievance} - you'd think the counting fed him.",
    ]),
    (JokeStyle::Aggressive, JokeTarget::Priest, &[
        "The priest preaches the meek inherit - convenient, since he holds his already.",
        "He'll pray over your soul for nothing - the burial's where the bill lands.",
        "Holy patience, he calls it - I've watched it wait a week for the tithe.",
    ]),
    (JokeStyle::Aggressive, JokeTarget::Rival, &[
        "Some folk plow straight furrows. Others plow excuses.",
        "There's a man could talk the moss off a stone - and does, instead of work.",
        "I'd trust his word as far as I could throw his barn - and his barn's a fine size.",
        "{grievance} - pride's the only crop that ever came up heavy.",
    ]),
    // self-defeating - the wound worn light
    (JokeStyle::SelfDefeating, JokeTarget::Oneself, &[
        "I promised the fence by winter. Kept my word - the winter after.",
        "My word's worth what it's writ on - nothing, mostly.",
        "I aimed high once. My aim's improved - now I aim for dinner.",
        "They say every man has his use. I'm still asking round for mine.",
        "{grievance} - that's the sum of me, near enough.",
    ]),
];

/// The four ways a joke lands (E5-6): the hearer's fork. Who laughs
/// shows who's aligned
const JOKE_LAUGHED: &[&str] = &[
    "Ha! True as rain, that.",
    "Ha - I'll carry that one home with me.",
    "Ha! Aye - and the half of it's left unsaid.",
    "Ha! Oh, that's a good one - I'll owe {2:obj} for it.",
];
const JOKE_ADDED: &[&str] = &[
    "Aye - and worse I've heard, and believed every word.",
    "Ha - don't stop there, {2:subj} {2:be} just warming.",
    "True - and I've one to match it, when I've drunk enough.",
    "Aye - {2:subj} missed the best of it: it gets worse.",
];
const JOKE_WARNED: &[&str] = &[
    "Careful - walls have ears, and ears have masters.",
    "Ssh - that one's a rope's worth if it travels.",
    "Laugh softer - the wrong mouth's always near.",
    "Mind {2:poss} tongue - some jokes keep better indoors.",
];
const JOKE_QUIET: &[&str] = &[
    "Mm. As {2:subj} {2:say}.",
    "Hm. A man's humor's his own.",
    "Mm.",
    "I heard it. I'll not repeat it.",
];

/// Every joke and receipt set, for the text audit's authored-line
/// census (E5-6)
#[doc(hidden)]
pub fn joke_sets() -> Vec<&'static [&'static str]> {
    JOKES.iter().map(|(_, _, s)| *s)
        .chain([JOKE_LAUGHED, JOKE_ADDED, JOKE_WARNED, JOKE_QUIET])
        .collect()
}

/// Which cell a matrix line template belongs to (E5-6): the receipt's
/// record reads the target off the line that drew it
fn joke_cell_of(line: &'static str) -> Option<(JokeStyle, JokeTarget)> {
    JOKES.iter().find(|(_, _, lines)| lines.contains(&line))
        .map(|&(s, t, _)| (s, t))
}

/// How likely a hearer is to give each receipt (E5-6): aligned by the
/// shared lie, warm by the bond, careful where the law can hear, stony
/// where they're built that way
fn receipt_weights(joker_grievance: Option<Grievance>, target: Option<JokeTarget>,
                   hearer: &Speaker, channel: crate::game::setting::Channel) -> [f32; 4] {
    let mut w = [0.35f32, 0.12, 0.10, 0.40]; // laugh, add, warn, quiet
    let aligned = match (joker_grievance,
        hearer.joke.as_ref().and_then(|j| j.grievance)) {
        (Some(a), Some(b)) => a.lie == b.lie,
        _ => false,
    };
    if aligned { w[0] += 0.4; w[1] += 0.25; w[3] -= 0.25; }
    if matches!(hearer.rel, Relation::Friend | Relation::Kin) { w[0] += 0.2; }
    if matches!(hearer.rel, Relation::Rival) { w[3] += 0.3; w[0] -= 0.15; }
    if channel == crate::game::setting::Channel::Public
        && target.is_some_and(JokeTarget::power) { w[2] += 0.35; }
    if hearer.traits.contains(&Trait::Wary) || hearer.traits.contains(&Trait::Stolid) { w[3] += 0.25; }
    w
}

/// The joke beat (E5-6, #1437): once a talk, after the talk's found its
/// feet, a profiled caller cracks the joke their style and grievance
/// draw - and the hearer's reply forks on how they take it. Public
/// talk can't carry an aggressive joke at power: the draw falls back
/// to a safer cell. Returns true when it paused on a menu
fn joke_beat(
    tr: &mut Transcript,
    pair: &[Speaker; 2],
    caller: usize,
    responder: usize,
    channel: crate::game::setting::Channel,
    used: &mut HashSet<&'static str>,
    recent: &HashSet<&'static str>,
    player: usize,
    cap: usize,
    tree: &'static str,
    choose: &mut dyn FnMut(&[Choice]) -> Option<usize>,
    rng: &mut Rng,
) -> bool {
    // one joke a talk at most (the beat's beats are recorded by who
    // told them), and it needs room for its two turns plus the
    // exchange it delays
    if tr.turns.len() < 4 || tr.turns.len() + 4 > cap
        || tr.jokes.iter().any(|b| b.joker == caller) {
        return false;
    }
    let Some(j) = pair[caller].joke.as_ref() else { return false };
    // the grieving and the children don't joke
    if pair[caller].status.contains(&"grieving") || pair[caller].status.contains(&"child")
        || pair[responder].status.contains(&"child") { return false; }
    // one joke a talk at most, and not every talk - a beat that fires
    // at every sitting stops saying anything about who tells it
    if rng.next_f32() >= 0.10 { return false; }
    // the draw: style weighted, then the cell. Aggressive strikes the
    // grievance's target (the reeve by default, for the unresentful
    // hothead); the inward styles aim at the self; the rest take
    // whatever their table holds
    let style_names = [JokeStyle::Affiliative, JokeStyle::SelfEnhancing,
        JokeStyle::Aggressive, JokeStyle::SelfDefeating];
    let mut styles = j.styles;
    for _ in 0..3 {
        if styles.iter().all(|s| *s <= 0.0) { return false; }
        let style = style_names[rng.weighted(&styles)];
        let target = match style {
            JokeStyle::Aggressive => j.grievance.map(|g| g.target)
                .unwrap_or(JokeTarget::Reeve),
            JokeStyle::SelfDefeating => JokeTarget::Oneself,
            JokeStyle::Affiliative => [JokeTarget::Weather, JokeTarget::Lord,
                JokeTarget::Reeve, JokeTarget::Priest, JokeTarget::Oneself,
                JokeTarget::Rival][rng.below(6)],
            JokeStyle::SelfEnhancing => [JokeTarget::Weather, JokeTarget::Lord,
                JokeTarget::Reeve, JokeTarget::Oneself][rng.below(4)],
        };
        // the exposure gate (E5-6's done-when): a barb at power waits
        // for the hidden transcript - where ears can hear, the draw
        // goes again with the aggressive edge struck out
        if style == JokeStyle::Aggressive && target.power()
            && channel == crate::game::setting::Channel::Public {
            styles[2] = 0.0;
            continue;
        }
        let Some(&(_, _, lines)) = JOKES.iter()
            .find(|&&(s, t, _)| s == style && t == target) else { continue };
        let fitting: Vec<&'static str> = lines.iter().copied()
            .filter(|l| !l.contains("{grievance}") || j.grievance.is_some()).collect();
        let Some((t, _)) = fresh_line(&fitting, &pair[caller], &pair[responder], used, recent, false, rng)
            else { return false };
        used.insert(t);
        // the {grievance} slot is the wound's own word, where the joke
        // carries one - a ref like `{someone}`'s, named by the draw
        let refs = [crate::game::text::Referent { slot: "grievance",
            text: j.grievance.map(|g| g.word).unwrap_or("the trouble").to_string(),
            sex: None, alive: true, count: 1 }];
        let text = crate::game::text::realize(t,
            &crate::game::text::Cx::plain(&pair[caller], &pair[responder], &refs), rng);
        // the aside wears the upcoming exchange's tree - it answers
        // inside the cascade, not off it
        tr.turns.push(Turn { channel, speaker: caller,
            act: Act::Joke, respond: None, text,
            concepts: vec!["joke"], tree, line: Some(t) });
        // the receipt: who laughs shows who's aligned. A player gets
        // the four forks as a menu; a villager rolls them off the bond,
        // the shared lie, and who's listening
        let kinds = [JokeReceipt::Laughed, JokeReceipt::AddedTo,
            JokeReceipt::Warned, JokeReceipt::Quiet];
        let kind = if responder == player {
            // the line each fork would say, rolled with the menu so the
            // preview is exactly what lands (the probe's shape)
            let mut choices: Vec<Choice> = vec![];
            let mut forks: Vec<&'static str> = vec![];
            for k in kinds {
                let t = fresh_line(k.lines(), &pair[responder], &pair[caller], used, recent, false, rng)
                    .map(|(t, _)| t).unwrap_or(k.lines()[0]);
                forks.push(t);
                choices.push(Choice { act: k.act(), respond: k.respond(),
                    preview: fill(t, &pair[responder], &pair[caller], rng) });
            }
            match choose(&choices) {
                None => return true,
                Some(i) => {
                    let t = forks[i.min(forks.len() - 1)];
                    used.insert(t);
                    let kind = kinds[i.min(kinds.len() - 1)];
                    tr.turns.push(Turn { channel, speaker: responder,
                        act: kind.act(), respond: Some(kind.respond()),
                        text: fill(t, &pair[responder], &pair[caller], rng),
                        concepts: vec!["joke", "joke_receipt"], tree, line: Some(t) });
                    tr.jokes.push(JokeBeat { joker: caller, style: Some(style),
                        target: Some(target), receipt: kind });
                    return false;
                }
            }
        } else {
            kinds[rng.weighted(&receipt_weights(j.grievance, Some(target), &pair[responder], channel))]
        };
        let (t, _) = fresh_line(kind.lines(), &pair[responder], &pair[caller], used, recent, false, rng)
            .unwrap_or((kind.lines()[0], String::new()));
        used.insert(t);
        tr.turns.push(Turn { channel, speaker: responder,
            act: kind.act(), respond: Some(kind.respond()),
            text: fill(t, &pair[responder], &pair[caller], rng),
            concepts: vec!["joke", "joke_receipt"], tree, line: Some(t) });
        tr.jokes.push(JokeBeat { joker: caller, style: Some(style),
            target: Some(target), receipt: kind });
        return false;
    }
    false
}

/// The tree name the doorknob worry reports as (E5-3, #1435)
pub const WORRY_TREE: &str = "the_doorknob";

/// A worry a speaker carries to the door (E5-3): the fear or need the
/// question's built on, the closed area a turned-aside why counts
/// under, and the openness tier the why must reach to get the truth
#[derive(Debug, Clone, Copy)]
pub struct Worry {
    pub area: Area,
    pub tier: u8,
    /// the two-or-three questions the fear or need puts at the door
    pub asks: &'static [&'static str],
    /// what the truth sounds like when the pair's openness reaches
    pub truths: &'static [&'static str],
}

/// The plain answers "answer plainly" draws on (E5-3): a truthful
/// hedge, whichever way the question's honestly met
const WORRY_ANSWERS: &[&str] = &[
    "Not that I've seen, friend.",
    "Aye - take it for so.",
    "I couldn't say truly.",
    "From what I've seen - aye.",
];

/// The reassurances (E5-3): a small warm word, no promise in it
const WORRY_SOOTHE: &[&str] = &[
    "Set {2:poss} heart easy - it'll hold.",
    "That fear won't find {2:obj} here.",
    "There's naught in it worth the carrying.",
];

/// Naming the why (E5-3): the marked pick - the answer's gated on how
/// far they can open, and a door that stays shut is its own tell
const WORRY_WHY: &[&str] = &[
    "And why do {2:subj} ask?",
    "What sits behind that, {other}?",
];

/// The worry a want or a problem puts at the door (E5-3): two or three
/// questions per fear and need, each with the truth the why reaches.
/// None when nothing here would carry a worry out loud
pub fn worry_of_reveal(r: crate::game::desires::Reveal) -> Option<Worry> {
    use crate::game::desires::{Problem as P, Reveal as R, Want as W};
    use Area::*;
    let w = Worry { area: Trouble, tier: 2, asks: &[], truths: &[] };
    Some(match r {
        R::Want(W::GoodFood) => Worry { area: Body, asks: &[
            "Does the bread keep fair measure, where {2:subj} lodges?",
            "Is a full loaf cheap enough for a working man's table?",
        ], truths: &[
            "My own pot's been thin a while - a body's always counting.",
        ], ..w },
        R::Want(W::Finery) => Worry { area: Village, asks: &[
            "Do folk here hold a patched sleeve against a body?",
            "Would they look twice at a secondhand cloak?",
        ], truths: &[
            "I've nothing fit for a church door - it shows at every gathering.",
        ], ..w },
        R::Want(W::Faith) => Worry { area: Heart, tier: 3, asks: &[
            "Do the parish bells call true here?",
            "Is the priest a man to trust a soul to?",
        ], truths: &[
            "My prayers run thin of late - I want a word that reaches.",
        ], ..w },
        R::Want(W::Shelter) => Worry { asks: &[
            "Is there room at a hearth for one more?",
            "Do roofs come dear in this village?",
        ], truths: &[
            "I've no door that's mine to shut.",
        ], ..w },
        R::Want(W::Security) => Worry { asks: &[
            "Does the watch keep its rounds after dark?",
            "Is the road safe for a lone cart?",
        ], truths: &[
            "A body can't sleep when the door's only wood.",
        ], ..w },
        R::Want(W::Mastery) => Worry { area: Work, asks: &[
            "Does a craft rise here by merit?",
            "Would a master take on a willing hand?",
        ], truths: &[
            "I'm better than my bench - it gnaws at me of a night.",
        ], ..w },
        R::Want(W::Music) => Worry { area: Heart, asks: &[
            "Does anyone here play of an evening?",
            "Is there music at the feasts?",
        ], truths: &[
            "It's a year since a tune lifted me.",
        ], ..w },
        R::Want(W::Instrument) => Worry { area: Heart, asks: &[
            "Would a lute find a buyer in the market?",
            "Is there a luthier nearer than the town?",
        ], truths: &[
            "I dream of strings of my own - daft, at my age.",
        ], ..w },
        R::Want(W::BetterTools) => Worry { area: Work, asks: &[
            "Do tools come fair at the market?",
            "Would a smith sell on trust?",
        ], truths: &[
            "My hands outwork my kit - blunt steel steals my hours.",
        ], ..w },
        R::Want(W::DifferentTrade) => Worry { area: Work, tier: 3, asks: &[
            "Do folk mind one who'd change their trade?",
            "Would the village forgive a hand that turned?",
        ], truths: &[
            "This bench was never mine - another life's been calling.",
        ], ..w },
        R::Want(W::Love) => Worry { area: Heart, tier: 3, asks: &[
            "Is this a village that marries kindly?",
            "Do folk here look kindly on a new face?",
        ], truths: &[
            "The evenings are the longest part of it.",
        ], ..w },
        R::Want(W::Joy) => Worry { area: Heart, asks: &[
            "Do children still get their games here?",
            "Is there any frolic left after the day's work?",
        ], truths: &[
            "Everything's work - I'd forgot a day could ring.",
        ], ..w },
        R::Want(W::Drink) => Worry { area: Heart, asks: &[
            "Does the alehouse keep a friendly fire?",
            "Is the ale good in this parish?",
        ], truths: &[
            "A cup with company does more for me than it should.",
        ], ..w },
        R::Problem(P::NoHome) => Worry { asks: &[
            "Does the village let a bed to a stranger?",
            "Are there houses to be had here?",
        ], truths: &[
            "I've slept under more hedges than roofs this year.",
        ], ..w },
        R::Problem(P::NoWorkplace) => Worry { area: Work, asks: &[
            "Is there bench work for a new hand?",
            "Do the crafts take on here?",
        ], truths: &[
            "I've a trade and nowhere to stand it.",
        ], ..w },
        R::Problem(P::NoMaster) => Worry { area: Work, asks: &[
            "Are the masters here good to their 'prentices?",
            "Does a master beat his learners much?",
        ], truths: &[
            "I'm bound to no one - a trade half-learned is half a life.",
        ], ..w },
        R::Problem(P::NoInput(_)) => Worry { area: Work, asks: &[
            "Is there stuff to be had for honest work here?",
            "Do materials come dear in this market?",
        ], truths: &[
            "My bench sits idle for want of stuff.",
        ], ..w },
        R::Problem(P::DerelictNearby) => Worry { area: Village, tier: 1, asks: &[
            "Does the village see to its empty shops?",
            "Do folk mind a shuttered house?",
        ], truths: &[
            "The husk next door pulls my whole street down with it.",
        ], ..w },
        R::Problem(P::OverseerCut) => Worry { tier: 3, asks: &[
            "Does the overseer take his cut openly?",
            "Would the lord hear a word against his man?",
        ], truths: &[
            "He skims my measure and there's none to tell.",
        ], ..w },
        R::Problem(P::Taxed) => Worry { tier: 3, asks: &[
            "Do the collectors take eggs as well as coin?",
            "Does the reeve count fair at reckoning?",
        ], truths: &[
            "They weighed my cupboards last Lady Day and found them light.",
        ], ..w },
        R::Problem(P::Broke) => Worry { asks: &[
            "Does bread come on credit in this village?",
            "Will a baker take a promise till market day?",
        ], truths: &[
            "Three coins won't stretch to Sunday.",
        ], ..w },
        R::Problem(P::Smelly) => Worry { area: Village, tier: 1, asks: &[
            "Do folk hold the smell of honest work against a body?",
            "Would they sit by me at table?",
        ], truths: &[
            "The work hangs on me - I've seen the looks.",
        ], ..w },
        R::Problem(P::Loss) => Worry { tier: 3, asks: &[
            "Do wolves come down this far?",
            "Are the folds safe at night?",
        ], truths: &[
            "The pack took half my fold one night - I heard them go.",
        ], ..w },
        R::Problem(P::Blamed) => Worry { area: Village, tier: 3, asks: &[
            "Does the village still talk of the fire?",
            "Would they let a body's name mend?",
        ], truths: &[
            "They say my bread lit it - it's years and the name won't wash.",
        ], ..w },
        R::Problem(P::SourGut) => Worry { area: Body, tier: 1, asks: &[
            "Is the market's stock fresh here?",
            "Would {2:subj} eat from a travelling pack?",
        ], truths: &[
            "Bad stock turned my gut - it's not right since.",
        ], ..w },
        R::Problem(P::ChildLabor) => Worry { area: Village, asks: &[
            "Do the children get their games here?",
            "Would the parish speak against small hands in the fields?",
        ], truths: &[
            "My boy's hands should hold a hoop, not a hoe - the village watches.",
        ], ..w },
        R::Problem(P::Rowdy) => Worry { area: Village, asks: &[
            "Do the season hands drink rough here?",
            "Is the street quiet of a night?",
        ], truths: &[
            "They'll not forget the mud I made of their green.",
        ], ..w },
        R::Problem(P::DeathDuty) => Worry { tier: 3, asks: &[
            "Does the lord take his fifth kindly?",
            "Would a widow's ask reach the castle?",
        ], truths: &[
            "The lord's man counted the dead one's spoons - it was owed, they said.",
        ], ..w },
        R::Problem(P::Bereaved) => Worry { area: Heart, tier: 3, asks: &[
            "Do folk here sit with the grieving?",
            "Does the village remember its dead kindly?",
        ], truths: &[
            "The bed's still half-made on their side.",
        ], ..w },
        R::Problem(P::ThinWood) => Worry { asks: &[
            "Is firewood easy come by here?",
            "Does the wood give out this side of the hill?",
        ], truths: &[
            "I walk further each year and come back lighter.",
        ], ..w },
        R::Problem(P::DryPit) => Worry { area: Work, asks: &[
            "Is stone to be had for building?",
            "Does the pit still give?",
        ], truths: &[
            "The pit drowned and took my living with it.",
        ], ..w },
        R::Problem(P::StinkingReach) => Worry { area: Village, tier: 1, asks: &[
            "Does the reach still stink below the mill?",
            "Would a body's washing come clean in that water?",
        ], truths: &[
            "My street smells before it's seen.",
        ], ..w },
        R::Problem(P::ShortChanged) => Worry { asks: &[
            "Do foreign counts run honest at the fair?",
            "Would they weigh it over if asked?",
        ], truths: &[
            "The foreign count came home short - I was too proud to weigh it over.",
        ], ..w },
        R::Problem(P::Tempted) => Worry { area: Road, asks: &[
            "Does the road ever call {2:obj}?",
            "Would {2:subj} go, if the circus came back?",
        ], truths: &[
            "The road's been singing to me - the days here run grey.",
        ], ..w },
        R::Problem(P::WritBorne) => Worry { tier: 3, asks: &[
            "Does the bailiff's writ come quick here?",
            "Would the castle hear a poor word?",
        ], truths: &[
            "A writ rides for me - each knock at the door could be it.",
        ], ..w },
        R::Problem(P::LongWinter) => Worry { asks: &[
            "Do the stores hold this far in?",
            "Does the village share when the winter runs long?",
        ], truths: &[
            "My bin'll be scraped before the thaw.",
        ], ..w },
        R::Problem(P::ColdHearth) => Worry { area: Body, asks: &[
            "Is there fuel to be had for a cold hearth?",
            "Would a neighbour share a peat?",
        ], truths: &[
            "The house eats cold - the children wear their coats to bed.",
        ], ..w },
        R::Problem(P::TheDark) => Worry { area: Heart, tier: 3, asks: &[
            "Are the nights long for {2:obj} too?",
            "Does the dark keep {2:obj}?",
        ], truths: &[
            "The dark's kept me since I can remember - the candle never lasts.",
        ], ..w },
        R::Problem(P::Grief) => Worry { area: Heart, tier: 3, asks: &[
            "Does the village still speak of the hanging?",
            "Do they let a body be, after?",
        ], truths: &[
            "I watched it - the rope and all - it won't leave the backs of my eyes.",
        ], ..w },
        R::Problem(P::Robbed) => Worry { tier: 3, asks: &[
            "Do folk bar their cellar doors here?",
            "Would the watch look into a walked-off stock?",
        ], truths: &[
            "My cellar walked off in the night - someone's eating my winter.",
        ], ..w },
        R::Problem(P::Hungry) => Worry { area: Body, asks: &[
            "Does the pot fill easier here?",
            "Is there day's work for a day's bread?",
        ], truths: &[
            "The pot's been bare three days - the belly knows the count.",
        ], ..w },
        R::Problem(P::Sick) => Worry { area: Body, asks: &[
            "Do the fevers ride this village?",
            "Is there a herb-wife within reach?",
        ], truths: &[
            "The fever won't loose me - the bed's grown into me.",
        ], ..w },
        R::Problem(P::Doctrine) => Worry { area: Village, asks: &[
            "Does the parish preach one thing and live another here?",
            "Would the green's plainer word be heard?",
        ], truths: &[
            "The church says wait - the green says now.",
        ], ..w },
    })
}

/// Every Reveal the worry table speaks to (E5-3), for the text
/// audit's authored-line census - `NoInput` carries a standing input
/// so the parameterized variant's covered too
#[doc(hidden)]
pub const WORRY_CENSUS: &[crate::game::desires::Reveal] = {
    use crate::game::desires::{Problem::*, Reveal::*, Want::*};
    &[
        Want(GoodFood), Want(Finery), Want(Faith), Want(Shelter),
        Want(Security), Want(Mastery), Want(Music), Want(Instrument),
        Want(BetterTools), Want(DifferentTrade), Want(Love), Want(Joy),
        Want(Drink),
        Problem(NoHome), Problem(NoWorkplace), Problem(NoMaster),
        Problem(NoInput("grain")), Problem(DerelictNearby),
        Problem(OverseerCut), Problem(Taxed), Problem(Broke),
        Problem(Smelly), Problem(Loss), Problem(Blamed),
        Problem(SourGut), Problem(ChildLabor), Problem(Rowdy),
        Problem(DeathDuty), Problem(Bereaved), Problem(ThinWood),
        Problem(DryPit), Problem(StinkingReach), Problem(ShortChanged),
        Problem(Tempted), Problem(WritBorne), Problem(LongWinter),
        Problem(ColdHearth), Problem(TheDark), Problem(Grief),
        Problem(Robbed), Problem(Hungry), Problem(Sick),
        Problem(Doctrine),
    ]
};

/// Every worry set - each fear and need's questions and truths, plus
/// the generic moves - for the text audit's authored-line census
/// (E5-3)
#[doc(hidden)]
pub fn worry_sets() -> Vec<&'static [&'static str]> {
    let mut out = vec![WORRY_ANSWERS, WORRY_SOOTHE, WORRY_WHY];
    for r in WORRY_CENSUS {
        if let Some(w) = worry_of_reveal(*r) {
            out.push(w.asks);
            out.push(w.truths);
        }
    }
    out
}

/// The set a speaker turns aside with (E5-2): their first matching
/// temperament, the plain set when none of the four show
fn evade_set(s: &Speaker) -> &'static [&'static str] {
    use crate::game::traits::Trait;
    if s.traits.contains(&Trait::Hotheaded) { EVADE_SNAP }
    else if s.traits.contains(&Trait::Gregarious) { EVADE_CHANGE }
    else if s.traits.contains(&Trait::Pious) { EVADE_GOD }
    else if s.traits.contains(&Trait::Proud) { EVADE_PIVOT }
    else { DEFLECTED }
}

/// Whether a topic's on the table for this asked-about speaker: every
/// `needs` carried (or `any` one of the alternates when needs is
/// empty, open to all when both are), and none of `not`.
pub fn topic_open(t: &crate::game::dialogue_trees::Topic, s: &Speaker) -> bool {
    let has = |n: &str| s.status.iter().any(|st| *st == n);
    let open = if !t.needs.is_empty() { t.needs.iter().all(|n| has(n)) }
        else if !t.any.is_empty() { t.any.iter().any(|n| has(n)) }
        else { true };
    open && t.not.iter().all(|n| !has(n))
}

/// The test this speaker would put to someone new: the first trait they
/// carry that probes anything
pub fn probe_for(s: &Speaker) -> Option<&'static Probe> {
    s.traits.iter().find_map(|t| {
        crate::game::dialogue_trees::PROBE_TESTS.iter().find(|p| p.who == *t)
    })
}

impl Transcript {
    /// The turns as printable lines: "Eliza: Forgive me, I've a cough..."
    pub fn lines(&self) -> Vec<String> {
        self.turns.iter()
            .map(|t| format!("{}: {}", self.speakers[t.speaker], t.text))
            .collect()
    }
}

/// Safety bound so an unlucky cascade can't loop forever
const MAX_TURNS: usize = 24;

/// Villager-to-villager chatter wraps earlier than a player talk:
/// nobody's steering it, so the talk closes before it wanders (#1368)
const AMBIENT_MAX_TURNS: usize = 16;

/// A goodbye this early is a brush-off, not a parting (#1368): while the
/// talk's still finding its feet a pool of nothing but partings falls
/// through to the next tree instead of closing
const MIN_LIVE_TURNS: usize = 4;

/// This many turns with no new tree walked and nothing new spoken of is
/// the talk circling - they drift apart (#1368)
const DOLDRUMS: usize = 6;

/// Reply forks whose lines are all spent count as exhausted (Versu's
/// exclusivity): a spent fork stops being offered, which is how a
/// conversation runs out of material.
fn eligible<'a>(replies: &'a [Reply], facts: &Facts, used: &HashSet<&'static str>) -> Vec<&'a Reply> {
    replies.iter()
        .filter(|r| r.criteria.iter().all(|c| c.holds(facts)))
        .filter(|r| r.lines.iter().chain(
            r.voice.iter().filter(|v| v.criteria.iter().all(|c| c.holds(facts))).flat_map(|v| v.lines.iter())
        ).any(|l| !used.contains(*l)))
        .collect()
}

/// The weight a reply draws with: base weight times every trait
/// multiplier the speaker carries (CiF-style biasing, never gating)
fn draw_weight(r: &Reply, speaker: &Speaker) -> f32 {
    let mut w = r.weight;
    for (t, m) in r.bias {
        if speaker.traits.contains(t) { w *= m; }
    }
    // the conversation hog (#1252): trading information is the one move
    // they can't make - a question back is a turn not about them, so it
    // near-never comes; whatever they'd answer turns to their own story
    if speaker.traits.contains(&Trait::SelfAbsorbed) {
        if r.act == Act::Question { w *= 0.05; }
        else if matches!(r.act, Act::Remark | Act::Answer) { w *= 1.4; }
    }
    w
}

/// How well a reply's act follows the call's (#1367): a proper answer
/// draws at full strength, a passable side-step only speaks when
/// nothing better's on offer, and a true non-sequitur never does - the
/// pool filters to the best tier present
pub fn act_fit(call: Act, reply: Act) -> u8 {
    match call {
        // a question wants its answer; a remark's a dodge when no
        // answer's left, but a question back or a goodbye never follows
        Act::Question => match reply {
            Act::Answer | Act::Agree | Act::PushBack => 2,
            // an evasion answers an unwelcome question; a leak IS the
            // answer slipping out sideways (E5-1)
            Act::Evade | Act::Slip => 2,
            // a bid, a joke or a worry back is a passable side-step
            Act::Remark | Act::Bid | Act::Joke | Act::Worry => 1,
            _ => 0,
        },
        // a door-side fear asks like a question does - and "ask why"
        // is one of the doors it opens (E5-1)
        Act::Worry => match reply {
            Act::Answer | Act::Agree | Act::PushBack | Act::Question => 2,
            Act::Remark => 1,
            _ => 0,
        },
        // a goodbye takes a goodbye (or a last remark), nothing else
        Act::Farewell => match reply {
            Act::Farewell | Act::Remark => 2,
            _ => 0,
        },
        // a reach is answered by the turn (E5-4): engaging it, the
        // half-turn, the miss or the refusal are all its shapes
        Act::Bid => 2,
        // a statement leaves the floor open; the exit only rides once
        // the talk's run dry - the rule player options already use
        _ => match reply {
            Act::Farewell => 1,
            _ => 2,
        },
    }
}

/// What a hog's monologue opens on, when the talk's own draw gives
/// nothing back (#1252): they don't pass on the village's business -
/// they hold forth about their own
pub const HOG_GISTS: &[&str] = &[
    "{2:subj} {2:have} caught me mid-story - I was just telling myself about the finest cart I ever built. {2:own} now, as it happens.",
    "Ah, good - an ear. Did I ever tell {2:obj} how I came to this village? It's a tale worth the walking.",
    "I've been fair bursting to tell someone: my stew last night was the best this house has seen. Sit, sit.",
];

/// Let-them-run lines (#1252): when you hold the space open, what fills
/// it is them - their past, their opinions of everyone. `{someone}`
/// fills with a villager's name
pub const HOG_RUN: &[&str] = &[
    "Did I ever tell {2:obj} of the winter I out-walked the snow? Three days, and my boots never once let in.",
    "{someone}? Lovely sort. Doesn't half go on, though.",
    "Folk bring me their troubles, {2:subj} {2:know} - I've always a word for them. Usually mine.",
    "I've the strongest back in this village, and the softest bed to prove it.",
    "{someone} does the work well enough, I'll grant - though nothing to how my mother taught me.",
];

/// A template rendered for the pair, through the realizer (E4-1,
/// #1403): `{other}` is the listener's name, `{trade}`/`{other_trade}`
/// each side's work-noun (#1181); the talk's rng backs any draws the
/// grammar layers add
pub(crate) fn fill(t: &str, speaker: &Speaker, other: &Speaker, rng: &mut Rng) -> String {
    crate::game::text::realize(t, &crate::game::text::Cx::plain(speaker, other, &[]), rng)
}

/// An unused variant of a line set: the template (so it can be spent) and
/// the rendered text with the pair's slots filled
fn fresh_line(lines: &[&'static str], speaker: &Speaker, other: &Speaker, used: &HashSet<&'static str>, recent: &HashSet<&'static str>, ritual: bool, rng: &mut Rng) -> Option<(&'static str, String)> {
    let fresh: Vec<&'static str> = lines.iter().copied().filter(|l| !used.contains(*l)).collect();
    // the village remembers (#1369): a content line someone said lately
    // is passed over for fresher words - ritual speech and the stock
    // phrases are common property, and when everything left is a repeat
    // the talk repeats rather than starve
    let pick: Vec<&'static str> = if ritual { fresh } else {
        let newer: Vec<&'static str> = fresh.iter().copied()
            .filter(|l| !recent.contains(*l) || crate::game::dialogue_trees::STOCK_LINES.contains(l)).collect();
        if newer.is_empty() { fresh } else { newer }
    };
    pick.get(rng.below(pick.len())).map(|&t| (t, fill(t, speaker, other, rng)))
}

/// A variant in the speaker's register: voice sets whose criteria the
/// speaker's facts satisfy are drawn from first, in authored order; the
/// base lines take over once the register's material is spent
fn fresh_voiced(lines: &[&'static str], voice: &[Voice], facts: &Facts, speaker: &Speaker, other: &Speaker, used: &HashSet<&'static str>, recent: &HashSet<&'static str>, ritual: bool, rng: &mut Rng) -> Option<(&'static str, String)> {
    for v in voice {
        if v.criteria.iter().all(|c| c.holds(facts)) {
            if let Some(f) = fresh_line(v.lines, speaker, other, used, recent, ritual, rng) { return Some(f); }
        }
    }
    fresh_line(lines, speaker, other, used, recent, ritual, rng)
}

/// The matching trees, most specific first (Ruskin's scoring: criterion
/// count; a tie keeps library order so authors control the fall-through)
fn cascade<'a>(library: &'a [DialogueTree], initiator_facts: &Facts) -> Vec<&'a DialogueTree> {
    let mut v: Vec<(usize, &'a DialogueTree)> = library.iter().enumerate()
        .filter(|(_, t)| t.criteria.iter().all(|c| c.holds(initiator_facts)))
        .map(|(i, t)| (i, t))
        .collect();
    v.sort_by(|(ia, ta), (ib, tb)| tb.criteria.len().cmp(&ta.criteria.len()).then(ia.cmp(ib)));
    v.into_iter().map(|(_, t)| t).collect()
}

/// What the player is offered at their turn: up to `n` sampled forks plus
/// the standing option to walk away (choose returns None).
pub struct Choice {
    /// The index of the fork among the node's replies... but for the
    /// caller what matters is which offered option was picked
    pub act: Act,
    /// How the line receives what was said (E6-3): the authored tag
    /// where the reply carries one, else the act's default - it moves
    /// the bond's parts when the pick lands
    pub respond: Respond,
    /// The line they'd say - the player picks a *way of speaking*
    pub preview: String,
}

/// Run a whole NPC-NPC conversation: the initiator opens the most
/// specific matching tree, the other answers, and the walk descends
/// call/response levels until an ending fork, an exhausted tree (which
/// falls through to the next-most-specific), or exhaustion of the generic
/// tail itself. Lines are spent as spoken; `max_turns` bounds the walk.
pub fn converse(
    a: &Speaker,
    b: &Speaker,
    rel: Relation,
    kind: Option<RelKind>,
    library: &[DialogueTree],
    recent: Option<&HashSet<&'static str>>,
    rng: &mut Rng,
) -> Transcript {
    converse_at(a, b, rel, kind, library, recent, &crate::game::setting::STREET, rng)
}

/// As `converse`, in a named setting (E5-9, #1446): the place and the
/// ears pick the transcript. A warm bond (friend or kin) is itself the
/// invitation backstage - callers pass the place and whether the law's
/// ear is on it, and the bond does the rest
pub fn converse_at(
    a: &Speaker,
    b: &Speaker,
    rel: Relation,
    kind: Option<RelKind>,
    library: &[DialogueTree],
    recent: Option<&HashSet<&'static str>>,
    setting: &crate::game::setting::Setting,
    rng: &mut Rng,
) -> Transcript {
    converse_with_at(a, b, rel, kind, library, recent, usize::MAX, setting,
        &mut |_| Some(0), rng)
}

/// As `converse`, but speaker `player`'s response turns go through
/// `choose`: it sees up to two weighted-sampled options (as previews of
/// what they'd say) and returns the option index, or None to end the
/// conversation early (Ending::LeavesEarly).
pub fn converse_with(
    a: &Speaker,
    b: &Speaker,
    rel: Relation,
    kind: Option<RelKind>,
    library: &[DialogueTree],
    recent: Option<&HashSet<&'static str>>,
    player: usize,
    choose: &mut dyn FnMut(&[Choice]) -> Option<usize>,
    rng: &mut Rng,
) -> Transcript {
    converse_with_at(a, b, rel, kind, library, recent, player,
        &crate::game::setting::STREET, choose, rng)
}

/// `converse_with`, in a named setting (E5-9, #1446): as `converse_at`,
/// a warm bond is itself the invitation backstage
pub fn converse_with_at(
    a: &Speaker,
    b: &Speaker,
    rel: Relation,
    kind: Option<RelKind>,
    library: &[DialogueTree],
    recent: Option<&HashSet<&'static str>>,
    player: usize,
    setting: &crate::game::setting::Setting,
    choose: &mut dyn FnMut(&[Choice]) -> Option<usize>,
    rng: &mut Rng,
) -> Transcript {
    // the invitation the bond buys (E5-9): friend and kin hear the
    // hidden transcript at a backstage place; a stranger gets the
    // street voice even at the kitchen table
    let mut setting = *setting;
    setting.invited |= matches!(rel, Relation::Friend | Relation::Kin);
    // no village memory (synthetic talks): nothing reads as a repeat
    let no_recent = HashSet::default();
    let recent = recent.unwrap_or(&no_recent);
    // owned clones: a landed telling deepens `open.shared` mid-loop
    // (E3-3) - the snapshot callers passed stays immutable for replay
    let mut pair = [a.clone(), b.clone()];
    // each side's address is computed for the pair (E4-2): the power
    // semantic, gated by the speaker's own archaism - a rival's thou is
    // contempt made audible
    for i in 0..2 {
        pair[i].rel = rel;
        pair[i].tv = crate::game::text::tv_for(&pair[i], &pair[1 - i], rel, matches!(rel, Relation::Rival));
    }
    let mut tr = Transcript {
        speakers: [a.name.clone(), b.name.clone()],
        trees_walked: vec![],
        turns: vec![],
        ending: None,
        vibe: None,
        asked: vec![],
        forward: 0,
        evasions: vec![],
        worry_told: None,
        slips: vec![],
        jokes: vec![],
        shared: 0,
        overshot: false,
        setting,
    };
    let mut used: HashSet<&'static str> = HashSet::default();
    // topics already asked this talk - each one's a beat, once
    let mut asked: HashSet<usize> = HashSet::default();
    // the slip's own stream (E5-8, #1439): forked before the walk
    // touches the shared rng, so the same turns roll the same way on
    // every replay however many picks stand - one leak a talk, and
    // only while the caller's arc budget lasts
    let mut slip_rng = rng.fork(0x5EED_5119);
    let mut slipped = false;
    // the transcript tag (E5-9, #1446): every line this walk pushes
    // carries the channel the setting speaks - the beat scheduler
    // (#1445) flips single lines later, when the hidden transcript's
    // said aloud to power
    let channel = setting.channel();
    let trees = cascade(library, &Facts::for_roles_setting(a, b, rel, kind, &setting));
    let mut trees = trees.iter().peekable();
    let mut node: Option<&Exchange> = trees.next().map(|t| {
        tr.trees_walked.push(t.name);
        t.root
    });
    // The tree the current node belongs to, for the transcript
    let mut node_tree = tr.trees_walked.first().copied().unwrap_or("");
    // The lead/respond split: the initiator speaks every call, the other
    // answers - a call/response ping-pong like an RPG dialogue tree.
    // Which fork the answerer takes still steers where the talk goes.
    let (caller, responder) = (0usize, 1usize);

    let advance_to_rejoin = |trees: &mut std::iter::Peekable<std::slice::Iter<&DialogueTree>>| -> Option<(&'static str, &'static Exchange)> {
        while let Some(t) = trees.next() {
            if let Some(rej) = t.rejoin {
                return Some((t.name, rej));
            }
        }
        None
    };

    // The audition (#1187): once the first exchange of a player talk
    // lands, the caller tests the responder with the probe their own
    // trait carries - one beat, small stakes. Kin and old friends don't
    // re-audition, and there's no audition where nobody's playing.
    let audition = player != usize::MAX
        && responder == player
        && matches!(rel, Relation::Stranger | Relation::Acquaintance);
    let mut probed = false;

    // the doldrums watch: a turn advances the talk when it walks a new
    // tree or says something not yet said (#1368)
    let mut said: HashSet<&'static str> = HashSet::default();
    let mut last_advance = 0usize;
    let mut last_tree = node_tree;
    let cap = if player == usize::MAX { AMBIENT_MAX_TURNS } else { MAX_TURNS };
    for _ in 0..cap {
        // ambient talks are bounded in turns, not just passes: beats
        // and echoes push extra lines that can't ride free
        if player == usize::MAX && tr.turns.len() >= cap - 1 {
            tr.ending.get_or_insert(Ending::Exhausted);
            break;
        }
        if tr.turns.len() - last_advance >= DOLDRUMS {
            tr.ending = Some(Ending::Exhausted);
            break;
        }
        if audition && !probed && tr.turns.len() == 2 {
            probed = true;
            if let Some(p) = probe_for(&pair[caller]) {
                if let Some((ct, call)) = fresh_line(p.calls, &pair[caller], &pair[responder], &used, recent, matches!(p.act, Act::Greet | Act::Farewell), rng) {
                    used.insert(ct);
                    tr.trees_walked.push(PROBE_TREE);
                    say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: caller, act: p.act, respond: None, text: call,
                        concepts: p.concepts.to_vec(), tree: PROBE_TREE, line: Some(ct) });
                    // the tested side gets the two forks the probe
                    // holds - the answer that lands and the wrong note
                    let mut choices: Vec<Choice> = vec![];
                    let mut forks: Vec<(&'static str, bool)> = vec![];
                    for ((act, lines), vibe) in [(p.pass, true), (p.fail, false)] {
                        let (t, preview) = fresh_line(lines, &pair[responder], &pair[caller], &used, recent, matches!(act, Act::Greet | Act::Farewell), rng)
                            .unwrap_or((lines[0], fill(lines[0], &pair[responder], &pair[caller], rng)));
                        choices.push(Choice { act, respond: respond_of(act), preview });
                        forks.push((t, vibe));
                    }
                    // no positional tell - which fork lands changes
                    if rng.below(2) == 1 { choices.swap(0, 1); forks.swap(0, 1); }
                    match choose(&choices) {
                        Some(i) => {
                            let (t, vibe) = forks[i];
                            used.insert(t);
                            let tag = respond_of(choices[i].act);
                            say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: responder, act: choices[i].act,
                                respond: Some(tag),
                                text: fill(t, &pair[responder], &pair[caller], rng),
                                concepts: p.concepts.to_vec(), tree: PROBE_TREE, line: Some(t) });
                            // answering their probe is telling of
                            // yourself (E3-3): it lands as deep as the
                            // pair's openness holds
                            if responder == player
                                && matches!(choices[i].act, Act::Answer | Act::Agree) {
                                note_disclosure(&mut tr, &mut pair[1 - player].open);
                            }
                            tr.vibe = Some(if vibe { Vibe::Safe } else { Vibe::Mismatch });
                            // a good answer picks the talk up where it
                            // left off; a miss leaves it on the polite
                            // tail, smiling about the weather
                            if !vibe {
                                node = Some(&crate::game::dialogue_trees::SMALL_CHAT);
                                node_tree = "village_smalltalk";
                                tr.trees_walked.push("village_smalltalk");
                            }
                            continue;
                        }
                        None => { tr.ending = Some(Ending::LeavesEarly); break; }
                    }
                }
            }
        }
        // The joke (E5-6, #1437): once the talk's found its feet, a
        // profiled caller may crack the one their humor draws - the
        // beat owns the hearer's fork
        let had = tr.turns.len();
        if joke_beat(&mut tr, &pair, caller, responder, channel, &mut used, recent, player, cap, node_tree, choose, rng) {
            tr.ending = Some(Ending::LeavesEarly); break;
        }
        // a spoken joke was this pass's exchange - it costs the turn
        // budget a call and answer would
        if tr.turns.len() != had { continue; }
        let Some(ex) = node else { tr.ending = Some(Ending::Exhausted); break };
        // a joke's echo (E5-6): when the reply just taken was one of
        // the trees' own jokes (the beat forks its own receipt), the
        // caller answers the joke before the exchange speaks - laugh,
        // top it, warn it quiet, or let it pass
        if let Some(jt) = tr.turns.last()
            .filter(|t| t.act == Act::Joke && t.speaker == responder
                && !t.concepts.contains(&"joke_receipt"))
            .and_then(|t| t.line) {
            let (style, target) = joke_cell_of(jt)
                .map(|(s, t)| (Some(s), Some(t))).unwrap_or((None, None));
            let jg = pair[responder].joke.as_ref().and_then(|j| j.grievance);
            let kinds = [JokeReceipt::Laughed, JokeReceipt::AddedTo,
                JokeReceipt::Warned, JokeReceipt::Quiet];
            let kind = kinds[rng.weighted(&receipt_weights(jg, target, &pair[caller], channel))];
            let (t, _) = fresh_line(kind.lines(), &pair[caller], &pair[responder], &used, recent, false, rng)
                .unwrap_or((kind.lines()[0], String::new()));
            used.insert(t);
            // the echo wears the upcoming exchange's tree - an aside,
            // not a step off the cascade
            tr.turns.push(Turn { channel, speaker: caller,
                act: kind.act(), respond: Some(kind.respond()),
                text: fill(t, &pair[caller], &pair[responder], rng),
                concepts: vec!["joke", "joke_receipt"], tree: node_tree, line: Some(t) });
            tr.jokes.push(JokeBeat { joker: responder, style, target, receipt: kind });
            // the exchange stands unconsumed - next pass speaks it
            continue;
        }
        // The floor-holder opens the beat, in the register their own
        // facts put them in (a lord's condescension, a rival's chill)
        let call_facts = Facts::for_roles_setting(&pair[caller], &pair[responder], rel, kind, &setting);
        let Some((call_t, call)) = fresh_voiced(ex.calls, ex.voice, &call_facts, &pair[caller], &pair[responder], &used, recent, matches!(ex.act, Act::Greet | Act::Farewell), rng) else {
            // Nothing fresh to call with: fall through to the next rejoinable tree,
            // or dry up entirely at the generic tail
            match advance_to_rejoin(&mut trees) {
                Some((name, rej)) => { tr.trees_walked.push(name); node = Some(rej); node_tree = name; }
                None => tr.ending = Some(Ending::Exhausted),
            }
            continue;
        };
        used.insert(call_t);
        say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: caller, act: ex.act, respond: None, text: call,
            concepts: ex.concepts.to_vec(), tree: node_tree, line: Some(call_t) });

        // The other side picks a fork
        let facts = Facts::for_roles_setting(&pair[responder], &pair[caller], rel, kind, &setting);
        let pool = eligible(ex.replies, &facts, &used);
        // the reply has to follow what was just said (#1367): the pool
        // keeps only the best-fitting acts on offer, and when nothing
        // even passable remains the talk shifts or dries up rather than
        // speak a line that doesn't follow
        let fit = pool.iter().map(|r| act_fit(ex.act, r.act)).max().unwrap_or(0);
        let pool: Vec<&Reply> = if fit == 0 {
            vec![]
        } else {
            pool.into_iter().filter(|r| act_fit(ex.act, r.act) == fit).collect()
        };
        // a goodbye before the talk's had its say is a brush-off, not a
        // parting (#1368): early on, a pool of nothing but partings
        // falls through to the next tree just like an empty one
        let early_parting = tr.turns.len() < MIN_LIVE_TURNS && !pool.is_empty()
            && pool.iter().all(|r| r.end == Some(Ending::Farewell));
        if pool.is_empty() || early_parting {
            // Nothing left to say back: the floor-holder shifts to a more
            // generic tree if there is one, else the talk dries up
            match advance_to_rejoin(&mut trees) {
                Some((name, rej)) => { tr.trees_walked.push(name); node = Some(rej); node_tree = name; }
                None => tr.ending = Some(Ending::Exhausted),
            }
            continue;
        }
        let weights: Vec<f32> = pool.iter().map(|r| draw_weight(r, &pair[responder])).collect();

        // (fork index, line template): the fork picked and the variant it
        // will speak. For a player turn the variants are rolled with the
        // options so the preview is exactly the line they get.
        let (pick, line_t): (usize, &'static str) = if responder == player {
            // The player sees up to two weighted-sampled options without replacement + "leave"
            let mut available: Vec<usize> = (0..pool.len()).collect();
            let mut sampled: Vec<usize> = vec![];
            let target_count = 2.min(pool.len());
            while sampled.len() < target_count && !available.is_empty() {
                let avail_weights: Vec<f32> = available.iter().map(|&i| weights[i]).collect();
                let chosen_avail_idx = rng.weighted(&avail_weights);
                let chosen_pool_idx = available.remove(chosen_avail_idx);
                sampled.push(chosen_pool_idx);
            }
            let mut rolled: Vec<(&'static str, Choice)> = vec![];
            for &i in &sampled {
                let r = pool[i];
                let (t, preview) = fresh_voiced(r.lines, r.voice, &facts, &pair[responder], &pair[caller], &used, recent, matches!(r.act, Act::Greet | Act::Farewell), rng)
                    .map(|(t, s)| (t, s))
                    .unwrap_or((r.lines[0], fill(r.lines[0], &pair[responder], &pair[caller], rng)));
                rolled.push((t, Choice { act: r.act, respond: r.respond.unwrap_or_else(|| respond_of(r.act)), preview }));
            }
            // a farewell's the tail of a talk, not a per-turn option (#1179):
            // it only sits on the list once the talk's run dry - while
            // anything live's on offer, the exits stay off it
            let mut pairs: Vec<(usize, (&'static str, Choice))> =
                sampled.iter().copied().zip(rolled.into_iter()).collect();
            if pairs.iter().any(|(_, (_, c))| c.act != Act::Farewell) {
                pairs.retain(|(_, (_, c))| c.act != Act::Farewell);
            }
            // your own questions ride one of the three options once the
            // hello's had its answer (#1189): the slot opens the topics
            // they can speak to, each a once-a-talk beat. A dry list is
            // farewells alone - there's no asking into a finished talk.
            // the tier gate (E3-2): a topic opens on the asked-about's
            // open facts; one closed but carrying a tell the asker saw
            // still lists - naming the tell earns its bonus
            let mut open_topics: Vec<usize> = vec![];
            let mut tell_topics: Vec<usize> = vec![];
            {
                let tells = pair[caller].tell_areas();
                for (ti, t) in crate::game::dialogue_trees::ASK_TOPICS.iter().enumerate() {
                    if asked.contains(&ti) || !topic_open(t, &pair[caller]) { continue; }
                    if call_facts.has(&format!("self:open:{}:{}", area_name(t.area), t.tier)) {
                        open_topics.push(ti);
                    } else if t.tier >= 2 && tells.contains(&t.area) {
                        // seen it, can't reach it - asking reads forward.
                        // Only the deep asks ride a tell: the personal
                        // layer is time-spent, nothing observed buys it
                        tell_topics.push(ti);
                    }
                }
            }
            let live = pairs.iter().any(|(_, (_, c))| c.act != Act::Farewell);
            if live && tr.turns.len() >= 2 && !(open_topics.is_empty() && tell_topics.is_empty()) && pairs.len() < 3 {
                pairs.push((usize::MAX, ("", Choice {
                    act: Act::Question, respond: Respond::Validate,
                    preview: "Put a question to them".into() })));
            }
            let choices: Vec<Choice> = pairs.iter().map(|(_, (_, c))| Choice { act: c.act, respond: c.respond, preview: c.preview.clone() }).collect();
            // the doorknob (E5-3, #1435): when the talk's run dry and
            // all that's left is exits, the caller's worry rides in
            // ahead of the parting - answered, soothed, or asked why,
            // then the farewells show
            if pairs.iter().all(|(_, (_, c))| c.act == Act::Farewell)
                && worry_beat(&mut tr, &mut pair, caller, responder,
                    channel, &mut used, recent, choose, rng) {
                tr.ending = Some(Ending::LeavesEarly);
                break;
            }
            match choose(&choices).and_then(|i| pairs.get(i)) {
                Some(&(usize::MAX, _)) => {
                    // the topics open as a pick of their own - one of
                    // yours, their answer, then the talk picks back up.
                    // A tell-named ask rides the same menu (E3-2)
                    let menu: Vec<usize> = open_topics.iter().chain(tell_topics.iter()).copied().collect();
                    let mut tch: Vec<Choice> = menu.iter().map(|&ti| Choice {
                        act: Act::Question, respond: Respond::Validate,
                        preview: format!("Ask about {}",
                            crate::game::dialogue_trees::ASK_TOPICS[ti].label) }).collect();
                    tch.push(Choice { act: Act::Remark, respond: Respond::Validate,
                        preview: "Say nothing of it".into() });
                    match choose(&tch) {
                        Some(i) if i < menu.len() => {
                            let tp = &crate::game::dialogue_trees::ASK_TOPICS[menu[i]];
                            asked.insert(menu[i]);
                            if !tr.trees_walked.contains(&ASK_TREE) { tr.trees_walked.push(ASK_TREE); }
                            let (qt, _) = fresh_line(tp.ask, &pair[responder], &pair[caller], &used, recent, false, rng)
                                .unwrap_or((tp.ask[0], String::new()));
                            used.insert(qt);
                            say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: responder, act: Act::Question,
                                respond: Some(Respond::PivotAsk),
                                text: fill(qt, &pair[responder], &pair[caller], rng),
                                concepts: tp.concepts.to_vec(), tree: ASK_TREE, line: Some(qt) });
                            if tell_topics.contains(&menu[i]) {
                                let talkative = pair[caller].traits.contains(&Trait::Gregarious);
                                let guarded = pair[caller].traits.contains(&Trait::Wary);
                                if !pair[caller].open.reached(tp.tier, talkative, guarded, true) {
                                    // naming the tell wasn't enough
                                    // (E3-2): the ask reads as forward -
                                    // a visible turn-aside, talk goes on
                                    tr.vibe = Some(Vibe::Mismatch);
                                    tr.forward += 1;
                                    // the turn-aside counts on the seat's
                                    // ledger by the area it stood over
                                    // (E5-2): enough of them and the
                                    // shape names itself
                                    tr.evasions.push((caller, tp.area));
                                    let set = evade_set(&pair[caller]);
                                    let (dt, _) = fresh_line(set, &pair[caller], &pair[responder], &used, recent, false, rng)
                                        .unwrap_or((set[0], String::new()));
                                    used.insert(dt);
                                    say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: caller, act: Act::Evade,
                                        respond: Some(Respond::Deflect),
                                        text: fill(dt, &pair[caller], &pair[responder], rng),
                                        concepts: tp.concepts.to_vec(), tree: ASK_TREE, line: Some(dt) });
                                    continue;
                                }
                            }
                            tr.asked.push(tp.label);
                            // a covered fact tells in its own words at
                            // that depth, but only to the "what you keep"
                            // asks (Draw::Secret): the named-status asks
                            // the tale itself (E4-8, #1426): a story ask's
                            // answer is composed off the speaker's seed in
                            // Labov's parts - no template, no `used` line
                            if matches!(tp.draws, crate::game::dialogue_trees::Draw::Story)
                                && pair[caller].story.is_some() {
                                let (_, text) = crate::game::storytelling::tell(
                                    pair[caller].story.as_ref().unwrap());
                                say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: caller, act: Act::Answer,
                                    respond: Some(Respond::Reflect),
                                    text, concepts: tp.concepts.to_vec(), tree: ASK_TREE, line: None });
                                continue;
                            }
                            // keep their authored answers (E3-4, #1425)
                            let (at, _) = if matches!(tp.draws,
                                    crate::game::dialogue_trees::Draw::Secret) {
                                crate::game::fact_content::telling(
                                        &call_facts, tp.area, tp.tier,
                                        rng.next_f32(), &used)
                                    .map(|l| (l, String::new()))
                                    .or_else(|| fresh_line(tp.tell, &pair[caller],
                                        &pair[responder], &used, recent, false, rng))
                                    .unwrap_or((tp.tell[0], String::new()))
                            } else {
                                fresh_line(tp.tell, &pair[caller],
                                    &pair[responder], &used, recent, false, rng)
                                    .unwrap_or((tp.tell[0], String::new()))
                            };
                            used.insert(at);
                            say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: caller, act: Act::Answer,
                                respond: Some(Respond::Reflect),
                                text: fill(at, &pair[caller], &pair[responder], rng),
                                concepts: tp.concepts.to_vec(), tree: ASK_TREE, line: Some(at) });
                            continue;
                        }
                        // "say nothing of it" picks the talk up where it stood
                        Some(_) => continue,
                        None => { tr.ending = Some(Ending::LeavesEarly); break; }
                    }
                }
                Some(&(i, (t, _))) => (i, t),
                None => { tr.ending = Some(Ending::LeavesEarly); break; }
            }
        } else {
            let i = rng.weighted(&weights);
            match fresh_voiced(pool[i].lines, pool[i].voice, &facts, &pair[responder], &pair[caller], &used, recent, matches!(pool[i].act, Act::Greet | Act::Farewell), rng) {
                Some((t, _)) => (i, t),
                None => { tr.ending = Some(Ending::Exhausted); break; }
            }
        };
        let reply = pool[pick];
        used.insert(line_t);
        let tag = reply.respond.unwrap_or_else(|| respond_of(reply.act));
        // the reply's shape (E4-5): a dispreferred answer - or a
        // guarded, cold agreement - draws the padding ops its stance
        // calls for; a proud, certain no still lands bare
        let st = crate::game::text::stance_of(&pair[responder], &pair[caller]);
        let ops = crate::game::text::shape(reply.act, &st, None);
        let text = crate::game::text::shaped(&ops,
            &fill(line_t, &pair[responder], &pair[caller], rng), None, &st, rng);
        say(&mut tr, &pair, &mut slip_rng, &mut slipped, Turn { channel,  speaker: responder, act: reply.act, respond: Some(tag),
            text,
            concepts: reply.concepts.to_vec(), tree: node_tree, line: Some(line_t) });
        // telling of yourself (E3-3): a substantive answer is a
        // disclosure; each lands as deep as the pair's openness holds -
        // a confession past the bond reads as strange
        if responder == player && matches!(reply.act, Act::Answer | Act::Agree)
            && matches!(tag, Respond::Reflect | Respond::Validate | Respond::Care | Respond::Advise) {
            note_disclosure(&mut tr, &mut pair[1 - player].open);
        }
        // the beat advanced the talk if it walked a new tree or named
        // something unsaid (#1368)
        let advanced = node_tree != last_tree
            || tr.turns.iter().rev().take(2)
                .flat_map(|t| t.concepts.iter()).any(|c| said.insert(*c));
        if advanced { last_advance = tr.turns.len(); last_tree = node_tree; }

        if let Some(end) = reply.end {
            tr.ending = Some(end);
            break;
        }
        node = match &reply.then {
            Some(next) => Some(next),
            None => match advance_to_rejoin(&mut trees) {
                Some((name, rej)) => { tr.trees_walked.push(name); node_tree = name; Some(rej) }
                // The generic tail keeps drifting: stay on this node until
                // its material is spent (the small-talk doldrums)
                None => node,
            },
        };
    }
    if tr.ending.is_none() {
        tr.ending = Some(Ending::Exhausted);
    }
    // the ear's landmark (E4-7, #1416): each speaker's phrase and habit
    // land once, on the first slots they fit - deterministic per turn,
    // so the replay this builds from keeps its prefix
    crate::game::idiolect::apply(&mut tr, &pair, player);
    tr
}

// ---------------------------------------------------------------------
// House meetings: a looser model. Statements carry concept tags and each
// reply binds to the last speaker, the organizer's open question, or an
// earlier statement - whichever pulls hardest.

/// A thing said on the meeting record
#[derive(Debug)]
pub struct Statement {
    /// Index into the `attendees` slice the meeting was run with
    pub speaker: usize,
    pub act: Act,
    pub text: String,
    pub concepts: Vec<&'static str>,
    /// Index of the statement it answers, if it answers one
    pub replying_to: Option<usize>,
    /// What it does to what it answers - the floor's threading, explicit
    /// (#1186): answers the ask, builds on the point, objects, or opens a
    /// tangent. Framing marks the rite, the asks and the send-off
    pub kind: ReplyKind,
}

/// What a floor statement does to the statement it answers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyKind {
    /// the rite, the asks, the send-off - not floor business
    Framing,
    /// an on-topic answer to the open question (or a pull-back re-ask)
    Answer,
    /// takes the last point up - agrees, extends, elaborates
    BuildOn,
    /// pushes back on it
    Object,
    /// off the thread entirely - gets the floor only when nothing real
    /// is on offer
    NewTopic,
}

/// A line an attendee could bring to a meeting
#[derive(Debug, Clone, Copy)]
pub struct MeetingLine {
    pub act: Act,
    /// What it's about - replies bind on concept overlap
    pub concepts: &'static [&'static str],
    /// (trait, multiplier) on the bid when the speaker has it
    pub bias: &'static [(Trait, f32)],
    /// Ways to say it; `{other}` fills with the prior speaker's name
    pub lines: &'static [&'static str],
}

/// The organizer's agenda item: a question with the concepts it invites
pub struct MeetingPrompt {
    /// The question as asked, `{org}` fills with the organizer's name
    pub text: &'static str,
    /// Concepts an answer should touch to count as answering it
    pub concepts: &'static [&'static str],
}

/// The framing every meeting gets: one short rite to open it, one to
/// close it (#1185) - pools of words for whoever offers. A devout
/// attendee brings the prayer (old-ways folk their own words), a
/// gregarious one a verse, and when nobody offers the organizer does
/// the plain word. A believer gives the send-off, else the organizer.
pub struct MeetingRites {
    /// a believer's opening prayer
    pub prayers: &'static [&'static str],
    /// the old faith's opening words
    pub old_rites: &'static [&'static str],
    /// a gregarious attendee's opening verse
    pub verses: &'static [&'static str],
    /// the organizer's plain opening, when the room offers nothing
    pub plain_opens: &'static [&'static str],
    /// a believer's benediction
    pub blessings: &'static [&'static str],
    /// the old faith's send-off
    pub old_closes: &'static [&'static str],
    /// the organizer's plain close, when no believer sends them off
    pub plain_closes: &'static [&'static str],
    /// the organizer pulling a drifted room back to the matter (#1186)
    pub pull_backs: &'static [&'static str],
}

/// Would the room look to this one for the rite? The devout - the pious,
/// the clergy, marked souls - carry the word
fn devout(s: &Speaker) -> bool {
    s.traits.contains(&Trait::Pious) || s.ideology == "clergy"
        || s.status.iter().any(|st| matches!(*st, "devout" | "seeker" | "mystic" | "sage"))
}

/// One who keeps the old ways gets the old words, not the church's
fn old_ways(s: &Speaker) -> bool {
    s.status.contains(&"old_ways")
}

/// The floor a speaker's temper bids for (#1252): a meeting is a held
/// audience, and holding an audience is the whole of a hog's art
fn bid_traits(sp: &Speaker) -> f32 {
    let mut m = 1.0f32;
    for (t, x) in [(Trait::Gregarious, 1.5), (Trait::Proud, 0.8), (Trait::Ambitious, 1.2),
        (Trait::SelfAbsorbed, 1.8)] {
        if sp.traits.contains(&t) { m *= x; }
    }
    m
}

/// The lines a push-back speaks with: the room's answer to a hog on the
/// floor is its own flavor - push back at them and it's the floor the
/// room talks about, not the matter (#1252)
fn floor_lines<'a>(l: &'a MeetingLine, target: &Speaker) -> &'a [&'static str] {
    if l.act == Act::PushBack && target.traits.contains(&Trait::SelfAbsorbed) {
        HOG_FLOOR
    } else { l.lines }
}

/// The room pushing back on a hog (#1252): `{other}` is their name,
/// `{other_trade}` their work-noun - "the miller holds the floor again"
pub const HOG_FLOOR: &[&str] = &[
    "{other} holds the floor again.",
    "Aye, {other_trade}, we've heard this verse - twice now.",
    "Give the floor a rest, {other}.",
];

/// How long the organizer's question stays the loudest thing in the room:
/// after this many answers it stops pulling replies to itself
const QUESTION_PULL: usize = 2;

/// How strongly a just-answered statement pulls a follow-up vs an older
/// one, and how much the open question outpulls both. The old threads
/// pull weak: a reply should land on the point just made, not stale
/// business (#1186)
const LAST_PULL: f32 = 2.5;
const OLD_PULL: f32 = 1.0;
const QUESTION_SCORE: f32 = 3.0;

/// The voice should fit the matter (#1238): a trade's counsel weighs more
/// on its own ground, a mismatched trade's less, and where no trade owns
/// the matter the village's memory - an elder - carries the weight
pub const TRADE_FIT: f32 = 1.4;
pub const TRADE_MISFIT: f32 = 0.85;
pub const ELDER_FIT: f32 = 1.2;

/// The trades that own a meeting matter; empty for the matters no craft
/// owns (rents, roads, fuel, grief, sickness)
pub fn trades_owning(concept: &str) -> &'static [&'static str] {
    match concept {
        "concern:harvest" => &["farmer", "shepherd", "beekeeper"],
        "concern:food" => &["baker", "alewife", "brewer", "taverner", "farmer"],
        "concern:wages" => &["smith", "mason", "cooper", "shoemaker", "weaver", "tailor", "miner"],
        "concern:bridge" => &["mason", "cooper"],
        "concern:faith" => &["priest"],
        "concern:commons" => &["shepherd", "farmer", "beekeeper"],
        _ => &[],
    }
}

/// The trades an `IssueTag` belongs to (#1308): a trade-scoped tag names
/// its trade ("materials (baker)" -> baker); the unscoped commons map to
/// the concern table where a craft clearly owns the matter (the bread,
/// the workshops, the fold). Taxes, writs, grief and the fevers belong
/// to no craft - the room hears those on seniority instead
pub fn issue_trades(issue: &str) -> Vec<&'static str> {
    if let Some((_, voc)) = issue.strip_suffix(')').and_then(|s| s.split_once(" (")) {
        return crate::game::vocation::vocation_named(voc)
            .map(|v| vec![crate::game::vocation::trade_word(v)]).unwrap_or_default();
    }
    match issue {
        "hunger" | "the empty pot" | "the sour gut" | "the bad bread" => trades_owning("concern:food").to_vec(),
        "tools" | "workplaces" | "materials" | "empty shops" | "apprenticeship" | "mastery" | "honest work's stink" => trades_owning("concern:wages").to_vec(),
        "the drowned pit" => vec!["miner"],
        "the wolves" | "the shrinking wood" => vec!["shepherd", "farmer"],
        "the true faith" => trades_owning("concern:faith").to_vec(),
        "housing" => trades_owning("concern:bridge").to_vec(),
        _ => vec![],
    }
}

/// How much `sp`'s word on a line about `concepts` weighs, by trade -
/// judged only on the concepts it shares with what it answers, `matter`
/// (a generic "aye" carries every concern, but speaks to one). Where no
/// trade owns the matter the village's memory carries it: an elder, or
/// one the organizer holds near (`tie` - kin or friend of the house)
pub fn counsel_weight(sp: &Speaker, tie: Relation, concepts: &[&str], matter: &[&str]) -> f32 {
    let on = |c: &&&str| matter.contains(*c);
    let mut owned = false;
    for c in concepts.iter().filter(on) {
        let trades = trades_owning(c);
        if trades.is_empty() { continue; }
        if sp.trade.is_some_and(|t| trades.contains(&t)) { return TRADE_FIT; }
        owned = true;
    }
    if owned {
        if sp.trade.is_some() { TRADE_MISFIT } else { 1.0 }
    } else if concepts.iter().filter(on).any(|c| c.starts_with("concern:"))
        && (sp.status.contains(&"elder") || matches!(tie, Relation::Kin | Relation::Friend)) {
        ELDER_FIT
    } else { 1.0 }
}

/// Floor statements the room can run without answering the matter before
/// the organizer calls it back (#1186)
const DRIFT_LIMIT: usize = 3;

/// Run a house meeting's talk: a rite opens it (the room's own - a
/// prayer, the old words, a verse, or the organizer's plain word), the
/// organizer puts the question, then each round the attendee whose best
/// bid weighs heaviest speaks - an answer to the open question, a
/// response to the last speaker (agree or push back, by temperament), or
/// a revival of an earlier statement that touches something they care
/// about. Whoever spoke last sits the round out. A send-off closes it.
/// Returns the full record, framing included.
pub fn run_meeting<'a>(
    organizer: &Speaker,
    attendees: &[&Speaker],
    ties: &[Relation],
    prompt: &MeetingPrompt,
    pools: &'a [&'a [MeetingLine]],
    rites: &MeetingRites,
    rng: &mut Rng,
) -> Vec<Statement> {
    // who offers the word: the room looks to a devout attendee for the
    // prayer, else a gregarious one brings a verse, else the organizer
    // does it themselves - framing, small and constant (#1185)
    let devout_ix: Vec<usize> = attendees.iter().enumerate()
        .filter(|(_, s)| devout(s)).map(|(i, _)| i).collect();
    let offering = if !devout_ix.is_empty() {
        devout_ix.clone()
    } else {
        attendees.iter().enumerate()
            .filter(|(_, s)| s.traits.contains(&Trait::Gregarious)).map(|(i, _)| i).collect()
    };
    let (rite_by, rite) = if offering.is_empty() {
        (usize::MAX, rites.plain_opens[rng.below(rites.plain_opens.len())])
    } else {
        let i = offering[rng.below(offering.len())];
        let s = attendees[i];
        let pool = if devout(s) {
            if old_ways(s) { rites.old_rites } else { rites.prayers }
        } else { rites.verses };
        (i, pool[rng.below(pool.len())])
    };
    // statement 0 is the rite, 1 the organizer's question: the bids below
    // hang off Q, and nobody replies to the rite - its "ritual" concept
    // meets nothing in a pool line
    const Q: usize = 1;
    let mut record: Vec<Statement> = vec![
        Statement {
            speaker: rite_by, // usize::MAX: the organizer isn't an attendee
            act: Act::Greet,
            text: rite.to_string(),
            concepts: vec!["ritual"],
            replying_to: None,
            kind: ReplyKind::Framing,
        },
        Statement {
            speaker: usize::MAX,
            act: Act::Question,
            text: format!("{}: {}", organizer.name, prompt.text),
            concepts: prompt.concepts.to_vec(),
            replying_to: None,
            kind: ReplyKind::Framing,
        },
    ];
    let mut used: HashSet<&'static str> = HashSet::default();
    let mut answers = 0usize;
    let mut rounds = pools.len().max(4) * 2;

    while rounds > 0 {
        rounds -= 1;
        let last_idx = record.len() - 1;
        let last_speaker = record[last_idx].speaker;
        let question_open = answers < QUESTION_PULL;

        // the counsel a statement's speaker carried on the matter - the
        // persuasion channel (#1308): a word from the trade the matter
        // belongs to pulls follow-ups (the room builds on it); a word
        // the matter doesn't belong to is let die. The organizer's own
        // calls pull at full strength, whoever made them
        let authority = |si: usize| -> f32 {
            let st = &record[si];
            if st.speaker == usize::MAX { return 1.0; }
            counsel_weight(attendees[st.speaker],
                ties.get(st.speaker).copied().unwrap_or(Relation::Stranger),
                &st.concepts, &prompt.concepts) * attendees[st.speaker].pull
        };

        // Bid: (speaker, target statement, line, kind, score)
        let mut bids: Vec<(usize, usize, &'a MeetingLine, ReplyKind, f32)> = vec![];
        for (i, sp) in attendees.iter().enumerate() {
            if i == last_speaker { continue; } // you can't answer yourself
            let pool = pools[i];
            // the room's own read on who to hear (#1249): a voice that
            // already carries bids heavier for the floor
            let score = sp.pull * bid_traits(sp);
            // What could they say, and what would it answer?
            for (si, s) in record.iter().enumerate() {
                if si == Q && !question_open { continue; }
                if s.speaker == i { continue; } // nor take up your own earlier point
                // a re-ask steers like the ask it re-opens - but only
                // while it's the fresh word, not as an old statement
                let ask = s.act == Act::Question && s.speaker == usize::MAX;
                let open_q = (si == Q && question_open) || (ask && si == last_idx);
                let pull = if open_q { QUESTION_SCORE }
                    else if si == last_idx { LAST_PULL * authority(si) }
                    else { OLD_PULL * authority(si) };
                for l in pool {
                    // Agreement or pushback can only reply to a fellow attendee's statement, not to an ask
                    if open_q && (l.act == Act::Agree || l.act == Act::PushBack) {
                        continue;
                    }
                    // Nobody says "aye, that's so" to a challenge aimed at their own point
                    let challenged = s.act == Act::PushBack
                        && s.replying_to.is_some_and(|t| record[t].speaker == i);
                    if challenged && l.act == Act::Agree {
                        continue;
                    }
                    let overlap = l.concepts.iter().any(|c| s.concepts.contains(c));
                    if !overlap && !open_q { continue; }
                    // an open question invites any contribution, but lines
                    // touching its concepts answer it best
                    let fit = if open_q {
                        if overlap { 1.5 } else { 0.5 }
                    } else { 1.0 };
                    if l.lines.iter().all(|x| used.contains(*x)) { continue; }
                    // the kind the bid would be (#1186): an answer to the
                    // matter, work on the point just made, or a tangent
                    let kind = if open_q {
                        if overlap { ReplyKind::Answer } else { ReplyKind::NewTopic }
                    } else if l.act == Act::PushBack { ReplyKind::Object } else { ReplyKind::BuildOn };
                    let mut s = score * pull * fit * counsel_weight(sp,
                        ties.get(i).copied().unwrap_or(Relation::Stranger), l.concepts, &s.concepts);
                    for (t, m) in l.bias {
                        if sp.traits.contains(t) { s *= m; }
                    }
                    bids.push((i, si, l, kind, s));
                }
            }
        }
        // a real reply outranks a tangent: a fresh topic only gets the
        // floor when there's nothing to answer (#1186)
        if bids.iter().any(|b| b.3 != ReplyKind::NewTopic) {
            bids.retain(|b| b.3 != ReplyKind::NewTopic);
        }
        if bids.is_empty() { break; }
        let weights: Vec<f32> = bids.iter().map(|b| b.4).collect();
        let (speaker, target, line, kind, _) = bids[rng.weighted(&weights)];
        let other_sp = if record[target].speaker == usize::MAX {
            organizer
        } else {
            attendees[record[target].speaker]
        };
        let Some((t, text)) = fresh_line(floor_lines(line, other_sp), attendees[speaker], other_sp, &used, &HashSet::default(), false, rng) else { continue };
        used.insert(t);
        if target == Q { answers += 1; }
        // A reply is about what it shares with its target: a stock "aye,
        // that's so" carries every concern tag, but on the record it's
        // about the point it agreed with, not the whole agenda
        let shared: Vec<&'static str> = line.concepts.iter().copied()
            .filter(|c| record[target].concepts.contains(c)).collect();
        let concepts = if shared.is_empty() { line.concepts.to_vec() } else { shared };
        record.push(Statement {
            speaker, act: line.act, text,
            concepts,
            replying_to: Some(target),
            kind,
        });
        // the organizer owns the thread: the floor's run this long
        // without touching the matter, and they call it back (#1186)
        if drifted(&record) {
            answers = 0; // the matter's open again
            record.push(Statement {
                speaker: usize::MAX,
                act: Act::Question,
                text: rites.pull_backs[rng.below(rites.pull_backs.len())].to_string(),
                concepts: prompt.concepts.to_vec(),
                replying_to: Some(Q),
                kind: ReplyKind::Framing,
            });
        }
    }
    // the send-off: a believer gives the benediction - a different voice
    // than opened if the room has one - else the organizer sends them home
    let others: Vec<usize> = devout_ix.iter().copied().filter(|&i| i != rite_by).collect();
    let closing = if !others.is_empty() { others } else { devout_ix };
    let (close_by, close) = if closing.is_empty() {
        (usize::MAX, rites.plain_closes[rng.below(rites.plain_closes.len())])
    } else {
        let i = closing[rng.below(closing.len())];
        let pool = if old_ways(attendees[i]) { rites.old_closes } else { rites.blessings };
        (i, pool[rng.below(pool.len())])
    };
    record.push(Statement {
        speaker: close_by,
        act: Act::Farewell,
        text: close.to_string(),
        concepts: vec!["ritual"],
        replying_to: None,
        kind: ReplyKind::Framing,
    });
    record
}

/// The floor's drifted when it's gone DRIFT_LIMIT statements without
/// either asking or answering the matter - the ask itself or an answer
/// to it re-anchors the room (#1186)
fn drifted(record: &[Statement]) -> bool {
    let ask = |s: &Statement| s.act == Act::Question && s.speaker == usize::MAX;
    let anchor = record.iter().rposition(|s| {
        ask(s) || s.replying_to.map(|t| ask(&record[t])).unwrap_or(false)
    }).unwrap_or(0);
    record.len() - 1 - anchor >= DRIFT_LIMIT
}

#[cfg(test)]
mod tests {
    /// E3-1 (#1423): the fact table splits by sensitivity - open facts
    /// fold under every prefix, hidden facts stay the speaker's own
    #[test]
    fn the_fact_table_splits_by_sensitivity() {
        let mut s = Speaker::new("A");
        s.facts.open = vec!["archetype:burned".into(), "value:safety".into()];
        s.facts.hidden = vec!["ghost:levy".into(), "tier:wound=3".into()];
        let other = Speaker::new("B");
        let f = Facts::for_roles(&s, &other, Relation::Stranger);
        // A's open facts read under self: and any:, never other:
        assert!(f.has("self:archetype:burned") && f.has("any:archetype:burned"));
        assert!(!f.has("other:archetype:burned"));
        assert!(f.has("self:ghost:levy"));
        assert!(!f.has("other:ghost:levy"));
        assert!(!f.has("any:ghost:levy"));
        assert!(f.has("self:tier:wound=3"));
    }
    use super::*;
    use crate::engine::det::HashMap;
    use crate::game::dialogue_trees::*;

    /// #1178: a line may live under exactly one (act, ending) - a line filed
    /// under two acts, or under both an ending and a continuing reply, shows
    /// the player a label that misdescribes it (and ends the talk wrongly).
    fn end_class(e: Option<Ending>) -> u8 {
        match e {
            None => 0,
            Some(Ending::Farewell) => 1,
            Some(Ending::WalksOff) => 2,
            Some(Ending::Exhausted) => 3,
            Some(Ending::LeavesEarly) => 4,
        }
    }

    /// Markers of an outright hostile line: a WalksOff reply may only hold
    /// lines like these, and a Farewell reply may hold none.
    const RUDE: &[&str] = &[
        "knave", "scoundrel", "Out of my way", "parley", "Breathe elsewhere",
        "nothing to say", "frightened", "Get out", "Hold {2:poss} tongue", "pox",
        "{2:poss} pride", "{2:poss} own yard", "fool",
        // cold refusals and hostile dismissals that end a talk on a hard note
        "Cur!", "break them", "shadow off my path", "Begone", "be gone",
        "before I shout", "call folk out", "not plague", "move on",
        "{2:speak} poison", "look to my door", "stretch our own store",
        "nothing spare", "bailiff watches", "barely has coals", "not from me",
    ];

    fn is_rude(line: &str) -> bool {
        RUDE.iter().any(|m| line.contains(m))
    }

    /// Every reachable (tree, exchange) pair in a library, deduped by the
    /// calls slice (shared consts collapse to one visit).
    fn all_exchanges(library: &[DialogueTree]) -> Vec<(&'static str, &'static Exchange)> {
        let mut out: Vec<(&'static str, &'static Exchange)> = vec![];
        let mut seen: HashSet<*const &'static str> = HashSet::default();
        for tree in library {
            let mut stack: Vec<&'static Exchange> = vec![tree.root];
            if let Some(rej) = tree.rejoin { stack.push(rej); }
            while let Some(ex) = stack.pop() {
                if !seen.insert(ex.calls.as_ptr()) { continue; }
                out.push((tree.name, ex));
                for r in ex.replies {
                    if let Some(then) = &r.then { stack.push(then); }
                }
            }
        }
        out
    }

    #[test]
    fn test_lines_file_under_one_act_and_ending() {
        let library = village_library();
        let mut owners: HashMap<&'static str, HashSet<(Option<Act>, u8)>> = HashMap::default();
        for (_tree, ex) in all_exchanges(&library) {
            for call in ex.calls {
                owners.entry(*call).or_default().insert((Some(ex.act), 0));
            }
            for r in ex.replies {
                for line in r.lines {
                    owners.entry(*line).or_default().insert((Some(r.act), end_class(r.end)));
                }
            }
        }
        let mut bad: Vec<String> = vec![];
        for (line, sigs) in &owners {
            if sigs.len() > 1 {
                bad.push(format!("{line:?} filed under {sigs:?}"));
            }
        }
        bad.sort();
        assert!(bad.is_empty(), "lines filed under more than one act/ending:\n{}", bad.join("\n"));
    }

    /// #1371: the village shopkeeper is no pedlar - merchant-keyed trees
    /// carry stall and tally talk, never pack-train or foreign-port
    /// material (the itinerant trader's lines live in visitor_trader)
    #[test]
    fn test_the_village_trader_speaks_of_the_stall_not_the_road() {
        let pedlar = ["mule", "pack horse", "packs", "highway", "overseas",
            "caravan", "exotic", "traveling", "Flanders", "sea port",
            "the ridge", "king's road", "from the city"];
        let mut bad: Vec<String> = vec![];
        for (tree, ex) in all_exchanges(&village_library()) {
            if tree != "merchant_trade" && tree != "the_merchants_ledger" { continue; }
            for line in ex.calls.iter().chain(ex.replies.iter().flat_map(|r| r.lines.iter())) {
                let low = line.to_lowercase();
                if let Some(w) = pedlar.iter().find(|w| low.contains(**w)) {
                    bad.push(format!("{tree}: {line:?} carries pedlar word {w:?}"));
                }
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("
"));
    }


    #[test]
    fn test_endings_match_their_lines() {
        let library = village_library();
        let mut bad: Vec<String> = vec![];
        for (tree, ex) in all_exchanges(&library) {
            for r in ex.replies {
                match r.end {
                    Some(Ending::Farewell) => {
                        if r.act != Act::Farewell {
                            bad.push(format!("{tree}: Farewell end under act {:?}", r.act));
                        }
                        if let Some(l) = r.lines.iter().find(|l| is_rude(l)) {
                            bad.push(format!("{tree}: rude line {l:?} ends in Farewell"));
                        }
                    }
                    Some(Ending::WalksOff) => {
                        if r.act != Act::PushBack {
                            bad.push(format!("{tree}: WalksOff end under act {:?}", r.act));
                        }
                        if let Some(l) = r.lines.iter().find(|l| !is_rude(l)) {
                            bad.push(format!("{tree}: polite line {l:?} ends in WalksOff"));
                        }
                    }
                    Some(other) => bad.push(format!("{tree}: reply ends in {other:?}")),
                    None => {}
                }
            }
        }
        bad.sort();
        assert!(bad.is_empty(), "ending/act/line mismatches:\n{}", bad.join("\n"));
    }

    #[test]
    fn test_farewell_options_only_appear_when_the_talk_is_dry() {
        // #1179: a farewell's the tail, not a per-turn option - it never
        // shares a choice list with live replies, but it still shows once
        // farewells are all that's left
        let library = village_library();
        let mut saw_dry_list = false;
        for seed in 0..60 {
            let mut rng = Rng::new(seed);
            let a = Speaker::new("Aldric");
            let b = Speaker::new("Eliza");
            let mut mixed = false;
            converse_with(&a, &b, Relation::Acquaintance, None, &library, None, 1, &mut |ch: &[Choice]| {
                let farewells = ch.iter().filter(|c| c.act == Act::Farewell).count();
                assert!(farewells == 0 || farewells == ch.len(),
                    "farewells shared a list with live replies: {:?}",
                    ch.iter().map(|c| (c.act, c.preview.clone())).collect::<Vec<_>>());
                if farewells == ch.len() { mixed = true; }
                Some(0)
            }, &mut rng);
            saw_dry_list |= mixed;
        }
        assert!(saw_dry_list, "some path should bottom out on farewells alone");
    }

    #[test]
    fn test_specific_tree_wins_over_generic() {
        let library = village_library();
        let mut rng = Rng::new(42);

        // A sick villager talking to a well villager
        let mut sick = Speaker::new("Eliza");
        sick.status.push("sick");
        let well = Speaker::new("Aldric");

        let tr = converse(&sick, &well, Relation::Acquaintance, None, &library, None, &mut rng);
        assert_eq!(tr.trees_walked[0], "the_sickness", "sick speaker leads with the_sickness tree");
        assert!(tr.turns.iter().any(|t| t.tree == "the_sickness"));

        // A well villager talking to a well villager leads with generic smalltalk
        let tr_well = converse(&well, &Speaker::new("Bram"), Relation::Acquaintance, None, &library, None, &mut rng);
        assert_eq!(tr_well.trees_walked[0], "village_smalltalk", "well speaker leads with smalltalk");
    }

    #[test]
    fn test_specificity_cascade_falls_through_to_smalltalk() {
        let library = village_library();
        let mut sick = Speaker::new("Eliza");
        sick.status.push("sick");
        let well = Speaker::new("Aldric");

        // Over multiple seeds, if the sickness tree doesn't end abruptly, it falls through to smalltalk
        let mut fell_through = false;
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let tr = converse(&sick, &well, Relation::Acquaintance, None, &library, None, &mut rng);
            if tr.trees_walked.contains(&"village_smalltalk") && tr.trees_walked[0] == "the_sickness" {
                fell_through = true;
                break;
            }
        }
        assert!(fell_through, "sickness tree should fall through to smalltalk on some paths");
    }

    #[test]
    fn test_helplessness_registers_the_same_issue_resigned() {
        // a cold speaker's trouble lands as resignation, woven onto the
        // ordinary beats - and a dedicated pool answers for them (#1269)
        let library = village_library();
        let mut cold_call = false;
        let mut cold_reply = false;
        for seed in 0..80 {
            let mut rng = Rng::new(seed);
            let mut cold = Speaker::new("Eliza");
            cold.status.push("thaw_helpless");
            cold.status.push("hungry");
            let warm = Speaker::new("Aldric");
            // calling out: the resigned register on their own trouble -
            // a hungry speaker leads hunger_lean_times, so the weave
            // lives where the issue lives
            let tr = converse(&cold, &warm, Relation::Acquaintance, None, &library, None, &mut rng);
            cold_call |= tr.turns.iter().any(|t| t.speaker == 0
                && (t.text.contains("nothing for it")
                    || t.text.contains("a thing to be")
                    || t.text.contains("same as all the rest")
                    || t.text.contains("had that before")
                    || t.text.contains("never asked us")));
            // answering: the dedicated pool's resignation
            let mut rng = Rng::new(seed);
            let tr = converse(&warm, &cold, Relation::Acquaintance, None, &library, None, &mut rng);
            cold_reply |= tr.turns.iter().any(|t| t.speaker == 1
                && (t.text.contains("What's to be done but bear it")
                    || t.text.contains("nothing we say moves the sky")
                    || t.text.contains("the pot's still bare")
                    || t.text.contains("always is what it is")));
        }
        assert!(cold_call, "a cold speaker's hunger lands resigned, not as a bare status");
        assert!(cold_reply, "the helpless pool answers smalltalk with resignation");
    }

    #[test]
    fn test_the_warm_rungs_dont_speak_resigned() {
        let library = village_library();
        let mut joined_word = false;
        for seed in 0..80 {
            let mut rng = Rng::new(seed);
            let mut warm = Speaker::new("Eliza");
            warm.status.push("thaw_joined");
            warm.status.push("hungry");
            let tr = converse(&warm, &Speaker::new("Aldric"), Relation::Acquaintance, None, &library, None, &mut rng);
            for t in &tr.turns {
                assert!(!t.text.contains("nothing for it")
                    && !t.text.contains("What's to be done but bear it"),
                    "a member doesn't speak resignation: {}", t.text);
                joined_word |= t.speaker == 0 && (t.text.contains("folk looking at it together")
                    || t.text.contains("what the meeting's for")
                    || t.text.contains("ask together"));
            }
        }
        assert!(joined_word, "a joined member carries the cause's register");
    }

    #[test]
    fn test_quip_exclusivity_prevents_duplicate_lines() {
        let library = village_library();
        for seed in 0..30 {
            let mut rng = Rng::new(seed);
            let a = Speaker::new("Aldric");
            let b = Speaker::new("Eliza");
            let tr = converse(&a, &b, Relation::Acquaintance, None, &library, None, &mut rng);

            let mut seen = HashSet::default();
            for turn in &tr.turns {
                assert!(seen.insert(turn.text.clone()), "Line was repeated: {:?}", turn.text);
            }
        }
    }

    #[test]
    fn test_personality_biases_reply_distribution() {
        let library = village_library();

        let mut hothead = Speaker::new("Corvin");
        hothead.traits.push(Trait::Hotheaded);

        let calm = Speaker::new("Aldric");

        let rival = Speaker::new("Wulfric");

        // Count pushbacks in old_scores for hothead vs calm
        let mut hot_pushbacks = 0;
        let mut calm_pushbacks = 0;
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            let tr_hot = converse(&rival, &hothead, Relation::Rival, None, &library, None, &mut rng);
            hot_pushbacks += tr_hot.turns.iter().filter(|t| t.speaker == 1 && t.act == Act::PushBack).count();

            let mut rng = Rng::new(seed);
            let tr_calm = converse(&rival, &calm, Relation::Rival, None, &library, None, &mut rng);
            calm_pushbacks += tr_calm.turns.iter().filter(|t| t.speaker == 1 && t.act == Act::PushBack).count();
        }
        assert!(hot_pushbacks > calm_pushbacks, "Hothead should push back more: hot={}, calm={}", hot_pushbacks, calm_pushbacks);
    }

    #[test]
    fn test_player_can_pick_options_or_leave_early() {
        let library = village_library();
        let a = Speaker::new("Aldric");
        let player = Speaker::new("Player");

        // Player leaves immediately (choose returns None)
        let mut rng = Rng::new(1);
        let tr_leave = converse_with(
            &a, &player, Relation::Acquaintance, None, &library, None, 1,
            &mut |_choices| None,
            &mut rng,
        );
        assert_eq!(tr_leave.ending, Some(Ending::LeavesEarly));
        assert_eq!(tr_leave.turns.len(), 1, "Only initial caller turn occurred before player left");

        // Player picks first option each time
        let mut rng = Rng::new(2);
        let mut choices_seen = 0;
        let tr_play = converse_with(
            &a, &player, Relation::Acquaintance, None, &library, None, 1,
            &mut |choices| {
                // two replies, or two replies plus the ask slot (#1189)
                assert!(!choices.is_empty() && choices.len() <= 3);
                choices_seen += 1;
                Some(0)
            },
            &mut rng,
        );
        assert!(choices_seen > 0);
        assert!(tr_play.turns.len() > 1);
        assert_ne!(tr_play.ending, Some(Ending::LeavesEarly));
    }

    /// #1238: floor statements (and lines) each speaker got on `prompt`, over many seeds
    fn floor_by(attendees: &[&Speaker], prompt: &MeetingPrompt, seeds: u64) -> (Vec<usize>, Vec<String>) {
        let pools: Vec<Vec<MeetingLine>> = attendees.iter().map(|s| meeting_pool(s)).collect();
        let refs: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
        let mut counts = vec![0; attendees.len()];
        let mut said = vec![];
        for seed in 0..seeds {
            let mut rng = Rng::new(seed);
            let rec = run_meeting(&Speaker::new("Reeve"), attendees, &[], prompt, &refs, &MEET_RITES, &mut rng);
            for st in rec.iter().filter(|s| s.speaker < attendees.len() && s.replying_to.is_some()) {
                if st.concepts.iter().any(|c| prompt.concepts.contains(c)) {
                    counts[st.speaker] += 1;
                    said.push(st.text.clone());
                }
            }
        }
        (counts, said)
    }

    #[test]
    fn test_a_trades_counsel_weighs_on_its_own_ground() {
        let mut farmer = Speaker::new("Hugh");
        farmer.trade = Some("farmer");
        let mut baker = Speaker::new("Alys");
        baker.trade = Some("baker");
        let laborer = Speaker::new("Wat");
        let harvest = ["concern:harvest"];
        assert_eq!(counsel_weight(&farmer, Relation::Stranger, &harvest, &harvest), TRADE_FIT);
        assert_eq!(counsel_weight(&baker, Relation::Stranger, &harvest, &harvest), TRADE_MISFIT, "the baker on the fields is heard as a baker");
        assert_eq!(counsel_weight(&laborer, Relation::Stranger, &harvest, &harvest), 1.0);
        assert_eq!(counsel_weight(&baker, Relation::Stranger, &["concern:food"], &["concern:food"]), TRADE_FIT);
        // off the matters entirely, nobody's trade counts
        assert_eq!(counsel_weight(&farmer, Relation::Stranger, &["smalltalk"], &["smalltalk"]), 1.0);
        // a generic "aye" carries every concern, but on the harvest it's
        // still the baker talking fields
        let aye = meeting_pool(&laborer).into_iter().find(|l| l.concepts.len() > 3).expect("a catch-all line").concepts;
        assert_eq!(counsel_weight(&baker, Relation::Stranger, aye, &harvest), TRADE_MISFIT);
        assert_eq!(counsel_weight(&farmer, Relation::Stranger, aye, &harvest), TRADE_FIT);

        let prompt = MeetingPrompt { text: "The harvest - what do we do?", concepts: &["concern:harvest"] };
        let (counts, said) = floor_by(&[&farmer, &baker, &laborer], &prompt, 60);
        assert!(counts[0] > counts[1], "the farmer holds the floor on the harvest: {:?}", counts);
        let counsel: Vec<&str> = MEET_COUNSEL.iter().filter(|(t, _)| *t == "farmer").flat_map(|(_, l)| l.lines.iter().copied()).collect();
        assert!(said.iter().any(|s| counsel.contains(&s.as_str())), "the farmer speaks with a farmer's authority");
        // and only the farmer has those words
        let baker_pool = meeting_pool(&baker);
        assert!(baker_pool.iter().all(|l| l.lines.iter().all(|x| !counsel.contains(x))));
    }

    #[test]
    fn test_an_elder_carries_the_matter_no_trade_owns() {
        let mut elder = Speaker::new("Old Maud");
        elder.status.extend(["elder", "grieving"]);
        let mut young = Speaker::new("Jack");
        young.status.push("grieving");
        assert_eq!(counsel_weight(&elder, Relation::Stranger, &["concern:grief"], &["concern:grief"]), ELDER_FIT);
        assert_eq!(counsel_weight(&young, Relation::Stranger, &["concern:grief"], &["concern:grief"]), 1.0);
        // where a trade owns it, age alone doesn't
        assert_eq!(counsel_weight(&elder, Relation::Stranger, &["concern:harvest"], &["concern:harvest"]), 1.0);
        let mut young2 = Speaker::new("Ivo");
        young2.status.push("grieving");
        let prompt = MeetingPrompt { text: "We've lost so many.", concepts: &["concern:grief"] };
        let (counts, _) = floor_by(&[&elder, &young, &young2], &prompt, 60);
        assert!(counts[0] > counts[1] && counts[0] > counts[2], "the village's memory speaks where no craft does: {:?}", counts);
    }

    #[test]
    fn test_issue_trades_map_the_campaigns_cause_to_a_craft() {
        // a trade-scoped tag names its trade through the vocation's
        // display name - the toolmaker speaks as a smith (#1308)
        assert_eq!(issue_trades("materials (baker)"), vec!["baker"]);
        assert_eq!(issue_trades("the overseers' cut (toolmaker)"), vec!["smith"]);
        assert_eq!(issue_trades("workplaces (cobbler)"), vec!["shoemaker"]);
        // the commons map to the crafts that own them
        assert!(issue_trades("hunger").contains(&"baker") && issue_trades("hunger").contains(&"farmer"));
        assert_eq!(issue_trades("the drowned pit"), vec!["miner"]);
        assert_eq!(issue_trades("the true faith"), vec!["priest"]);
        // and some causes belong to no craft - seniority speaks for those
        assert!(issue_trades("the taxes").is_empty());
        assert!(issue_trades("the grief").is_empty());
        assert!(issue_trades("the lord's letter").is_empty());
    }

    #[test]
    fn test_the_organizers_own_carry_the_unowned_matter() {
        // seniority's other half (#1308): where no craft owns the matter,
        // kin and the near speak with the elder's weight too
        let mut kin = Speaker::new("Cis");
        kin.status.push("grieving");
        let mut far = Speaker::new("Tom");
        far.status.push("grieving");
        let grief = ["concern:grief"];
        assert_eq!(counsel_weight(&kin, Relation::Kin, &grief, &grief), ELDER_FIT);
        assert_eq!(counsel_weight(&kin, Relation::Friend, &grief, &grief), ELDER_FIT);
        assert_eq!(counsel_weight(&far, Relation::Stranger, &grief, &grief), 1.0);
        assert_eq!(counsel_weight(&far, Relation::Rival, &grief, &grief), 1.0);
        // a craft matter doesn't care who your kin are
        let mut baker = Speaker::new("Alys");
        baker.trade = Some("baker");
        assert_eq!(counsel_weight(&baker, Relation::Kin, &["concern:harvest"], &["concern:harvest"]), TRADE_MISFIT);
    }

    /// #1308: how many follow-ups each speaker's statements drew, over
    /// seeds - a statement that moved the room is the one the room
    /// builds on, not the one it let die
    fn replies_drawn(attendees: &[&Speaker], prompt: &MeetingPrompt, seeds: u64) -> (Vec<usize>, Vec<usize>) {
        let pools: Vec<Vec<MeetingLine>> = attendees.iter().map(|s| meeting_pool(s)).collect();
        let refs: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
        let (mut drawn, mut made) = (vec![0; attendees.len()], vec![0; attendees.len()]);
        for seed in 0..seeds {
            let mut rng = Rng::new(seed);
            let rec = run_meeting(&Speaker::new("Reeve"), attendees, &[], prompt, &refs, &MEET_RITES, &mut rng);
            for (si, st) in rec.iter().enumerate() {
                if st.speaker < attendees.len() && st.act != Act::Greet {
                    made[st.speaker] += 1;
                    drawn[st.speaker] += rec.iter().filter(|r| r.replying_to == Some(si)).count();
                }
            }
        }
        (drawn, made)
    }

    #[test]
    fn test_the_room_builds_on_the_trade_the_matter_belongs_to() {
        let mut farmer = Speaker::new("Hugh");
        farmer.trade = Some("farmer");
        let mut baker = Speaker::new("Alys");
        baker.trade = Some("baker");
        let laborer = Speaker::new("Wat");
        let prompt = MeetingPrompt { text: "The harvest - what do we do?", concepts: &["concern:harvest"] };
        let (drawn, made) = replies_drawn(&[&farmer, &baker, &laborer], &prompt, 60);
        // per word said, the farmer's counsel pulls the room after it;
        // the baker's on the fields is let die
        let per_word = |i: usize| drawn[i] as f32 / made[i].max(1) as f32;
        assert!(per_word(0) > per_word(1),
            "drawn per word - farmer {:.2}, baker {:.2} of {:?}", per_word(0), per_word(1), made);
    }

    #[test]
    fn test_house_meeting_threading() {
        let organizer = Speaker::new("Reeve");
        let mut a = Speaker::new("Eliza");
        a.status.push("sick");
        let mut b = Speaker::new("Bram");
        b.status.push("hungry");
        let c = Speaker::new("Corvin");

        let pool_a = meeting_pool(&a);
        let pool_b = meeting_pool(&b);
        let pool_c = meeting_pool(&c);
        let pools: Vec<&[MeetingLine]> = vec![&pool_a, &pool_b, &pool_c];
        let attendees = [&a, &b, &c];

        let mut rng = Rng::new(77);
        let statements = run_meeting(&organizer, &attendees, &[], &MEET_PROMPT, &pools, &MEET_RITES, &mut rng);

        assert!(statements.len() >= 5, "Meeting should produce multiple statements");
        assert_eq!(statements[0].act, Act::Greet); // the rite that opens it
        assert_eq!(statements[0].replying_to, None);
        assert_eq!(statements[1].speaker, usize::MAX); // organizer question
        assert_eq!(statements[1].replying_to, None);
        assert_eq!(statements.last().unwrap().act, Act::Farewell); // the send-off

        // Every floor statement must reply to a valid earlier statement
        let floor = &statements[2..statements.len() - 1];
        for (idx, stmt) in floor.iter().enumerate() {
            let idx = idx + 2;
            let target = stmt.replying_to.expect("Attendee statement must reply to something");
            assert!(target < idx, "Target statement must be prior to this statement");
            // No attendee speaks twice in a row
            assert_ne!(stmt.speaker, statements[idx - 1].speaker, "Speaker cannot speak consecutively");
        }
    }

    #[test]
    fn test_meeting_replies_do_not_agree_with_opener() {
        let organizer = Speaker::new("Reeve");
        let a = Speaker::new("Eliza");
        let b = Speaker::new("Bram");
        let pool_a = meeting_pool(&a);
        let pool_b = meeting_pool(&b);
        let pools: Vec<&[MeetingLine]> = vec![&pool_a, &pool_b];
        let attendees = [&a, &b];

        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let statements = run_meeting(&organizer, &attendees, &[], &MEET_PROMPT, &pools, &MEET_RITES, &mut rng);
            // Any statement answering the open question (statement 1,
            // after the rite) must NOT be Agree or PushBack
            for stmt in &statements {
                if stmt.replying_to == Some(1) {
                    assert_ne!(stmt.act, Act::Agree, "Should not agree with an open question");
                    assert_ne!(stmt.act, Act::PushBack, "Should not push back on an open question without raising a point");
                }
            }
        }
    }

    #[test]
    fn test_sick_rival_falls_through_to_rivalry_then_smalltalk() {
        let library = village_library();
        let mut sick_rival = Speaker::new("Eliza");
        sick_rival.status.push("sick");
        let rival = Speaker::new("Corvin");

        // The cascade has [the_sickness, old_scores, village_smalltalk]
        let facts = Facts::for_roles(&sick_rival, &rival, Relation::Rival);
        let cascaded = cascade(&library, &facts);
        let names: Vec<&str> = cascaded.iter().map(|t| t.name).collect();
        assert_eq!(names, vec!["the_sickness", "old_scores", "village_smalltalk"]);

        // When the conversation runs, old_scores is preserved and walked via its rejoin
        let mut saw_rival_walked = false;
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let tr = converse(&sick_rival, &rival, Relation::Rival, None, &library, None, &mut rng);
            if tr.trees_walked.contains(&"old_scores") && tr.trees_walked[0] == "the_sickness" {
                saw_rival_walked = true;
                break;
            }
        }
        assert!(saw_rival_walked, "old_scores should be preserved and walked on fallback from sickness");
    }

    #[test]
    fn test_player_choices_no_duplicate_options() {
        let library = village_library();
        let a = Speaker::new("Aldric");
        let player = Speaker::new("Player");

        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            converse_with(
                &a, &player, Relation::Acquaintance, None, &library, None, 1,
                &mut |choices| {
                    if choices.len() == 2 {
                        assert_ne!(choices[0].preview, choices[1].preview, "Player choices must be distinct");
                    }
                    Some(0)
                },
                &mut rng,
            );
        }
    }

    fn meeting_records(seeds: std::ops::Range<u64>) -> Vec<Vec<Statement>> {
        let organizer = Speaker::new("Reeve");
        let mut a = Speaker::new("Hob");
        a.status.push("grieving");
        let mut b = Speaker::new("Tilda");
        b.status.push("hungry");
        b.traits.push(Trait::Hotheaded);
        let c = Speaker::new("Osric");
        let pools = [meeting_pool(&a), meeting_pool(&b), meeting_pool(&c)];
        let pools: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
        let attendees = [&a, &b, &c];
        seeds.map(|seed| run_meeting(&organizer, &attendees, &[], &MEET_PROMPT, &pools, &MEET_RITES, &mut Rng::new(seed)))
            .collect()
    }

    #[test]
    fn test_pull_carries_the_floor() {
        // #1249: all else equal, the voice the room already follows
        // bids heavier and takes more of it
        let organizer = Speaker::new("Reeve");
        let mut quiet_a = Speaker::new("Hob");
        quiet_a.pull = 0.4;
        let mut quiet_b = Speaker::new("Osric");
        quiet_b.pull = 0.4;
        let mut loud = Speaker::new("Tilda");
        loud.pull = 2.0;
        let pools = [meeting_pool(&quiet_a), meeting_pool(&quiet_b), meeting_pool(&loud)];
        let pools: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
        let attendees = [&quiet_a, &quiet_b, &loud];
        let mut floor = [0usize; 3];
        for seed in 0..60u64 {
            let rec = run_meeting(&organizer, &attendees, &[], &MEET_PROMPT, &pools, &MEET_RITES, &mut Rng::new(seed));
            for st in &rec { if st.speaker < 3 { floor[st.speaker] += 1; } }
        }
        assert!(floor[2] > floor[0] && floor[2] > floor[1],
            "the voice that carries takes the floor: {floor:?}");
    }

    #[test]
    fn test_meeting_attendees_never_reply_to_themselves() {
        for rec in meeting_records(0..100) {
            for stmt in &rec[2..rec.len() - 1] {
                if stmt.speaker == usize::MAX { continue; } // the organizer's pull-backs
                let target = stmt.replying_to.unwrap();
                assert_ne!(rec[target].speaker, stmt.speaker, "attendee replied to own statement: {:?}", stmt.text);
                if stmt.act == Act::Agree && rec[target].act == Act::PushBack {
                    let challenged = rec[target].replying_to.map(|t| rec[t].speaker);
                    assert_ne!(challenged, Some(stmt.speaker), "agreed with a challenge to own point");
                }
            }
        }
    }

    #[test]
    fn test_dialogue_expansion_exceeds_1000_responses_and_is_clean() {
        let library = village_library();
        let mut total_responses = 0;
        let mut visited_exchanges = HashSet::default();

        fn walk_ex(ex: &Exchange, count: &mut usize, visited: &mut HashSet<*const Exchange>) {
            let ptr = ex as *const Exchange;
            if !visited.insert(ptr) { return; }
            *count += ex.calls.len();
            for r in ex.replies {
                *count += r.lines.len();
                if let Some(next) = &r.then {
                    walk_ex(next, count, visited);
                }
            }
        }

        for tree in &library {
            walk_ex(tree.root, &mut total_responses, &mut visited_exchanges);
            if let Some(rej) = tree.rejoin {
                walk_ex(rej, &mut total_responses, &mut visited_exchanges);
            }
        }

        let test_speaker = Speaker::new("Aldric");
        let pool = meeting_pool(&test_speaker);
        for m in &pool {
            total_responses += m.lines.len();
        }

        assert!(
            total_responses >= 1000,
            "Dialogue library must contain at least 1000 responses, found {}",
            total_responses
        );

        // Verify zero sensitive / slur words across all visited exchanges and lines
        let forbidden = ["faggot", "retard", "nigger", "cunt", "whore", "slut", "bitch"];
        for ex_ptr in &visited_exchanges {
            let ex = unsafe { &**ex_ptr };
            for c in ex.calls {
                let lower = c.to_lowercase();
                for f in &forbidden {
                    assert!(!lower.contains(f), "Forbidden word '{}' found in call: {}", f, c);
                }
            }
            for r in ex.replies {
                for l in r.lines {
                    let lower = l.to_lowercase();
                    for f in &forbidden {
                        assert!(!lower.contains(f), "Forbidden word '{}' found in line: {}", f, l);
                    }
                }
            }
        }
    }

    #[test]
    fn test_meeting_replies_carry_only_shared_concepts() {
        for rec in meeting_records(0..100) {
            for stmt in &rec[2..rec.len() - 1] {
                if stmt.speaker == usize::MAX { continue; } // the organizer's pull-backs
                let target = stmt.replying_to.unwrap();
                if target == 1 { continue; }
                assert!(stmt.concepts.iter().all(|c| rec[target].concepts.contains(c)),
                    "reply {:?} carries concepts its target {:?} lacks", stmt.concepts, rec[target].concepts);
            }
        }
    }

    #[test]
    fn test_meeting_replies_stay_on_the_thread() {
        // the matter at hand: replies land on the point just made or on
        // an open ask - old threads and tangents lose (#1186)
        let mut close = 0;
        let mut back = 0;
        let mut tangents = 0;
        for rec in meeting_records(0..100) {
            for (i, stmt) in rec.iter().enumerate() {
                if stmt.kind == ReplyKind::Framing { continue; }
                if stmt.kind == ReplyKind::NewTopic { tangents += 1; }
                let t = stmt.replying_to.expect("floor statements all reply");
                assert!(t < i, "reply {:?} targets a later statement", stmt.text);
                let ask = rec[t].act == Act::Question && rec[t].speaker == usize::MAX;
                if i - t == 1 || ask { close += 1; } else { back += 1; }
            }
        }
        eprintln!("thread: {close} on the live point, {back} reaching back, {tangents} tangents");
        assert!(close > back * 4, "most replies should land on the live point");
        assert!(tangents * 5 < close, "tangents should be rare");
    }

    #[test]
    fn test_meeting_reply_kinds_mark_the_thread() {
        for rec in meeting_records(0..100) {
            for (i, stmt) in rec.iter().enumerate() {
                if i < 2 || i == rec.len() - 1 || stmt.kind == ReplyKind::Framing { continue; }
                let t = stmt.replying_to.unwrap();
                let ask = rec[t].act == Act::Question && rec[t].speaker == usize::MAX;
                if t == 1 || (ask && i - t == 1) {
                    // the question, or a pull-back while it's the fresh
                    // word: only an answer - or a tangent, when nothing
                    // real was on offer
                    assert!(matches!(stmt.kind, ReplyKind::Answer | ReplyKind::NewTopic),
                        "{:?} on an open ask", stmt.kind);
                } else {
                    // a stale pull-back is floor business like the rest
                    assert!(matches!(stmt.kind, ReplyKind::Object | ReplyKind::BuildOn),
                        "{:?} on floor business", stmt.kind);
                    assert_eq!(stmt.kind == ReplyKind::Object, stmt.act == Act::PushBack);
                }
            }
        }
    }

    #[test]
    fn test_meeting_organizer_steers_when_the_floor_drifts() {
        let frame = |act, speaker| Statement {
            speaker, act, text: "framing".into(), concepts: vec![],
            replying_to: None, kind: ReplyKind::Framing,
        };
        let floor = |to, kind| Statement {
            speaker: 0, act: Act::Remark, text: "chatter".into(), concepts: vec![],
            replying_to: Some(to), kind,
        };
        // rite, ask, an answer - then a run that leaves the matter alone
        let mut rec = vec![
            frame(Act::Greet, 0),
            frame(Act::Question, usize::MAX),
            floor(1, ReplyKind::Answer),
        ];
        for _ in 0..DRIFT_LIMIT - 1 {
            rec.push(floor(rec.len() - 1, ReplyKind::BuildOn));
            assert!(!drifted(&rec));
        }
        rec.push(floor(rec.len() - 1, ReplyKind::BuildOn));
        assert!(drifted(&rec), "three unanswered statements is a drift");
        // an answer to the ask re-anchors it, and so does a pull-back
        rec.push(floor(1, ReplyKind::Answer));
        assert!(!drifted(&rec));
        for _ in 0..DRIFT_LIMIT {
            rec.push(floor(rec.len() - 1, ReplyKind::BuildOn));
        }
        assert!(drifted(&rec));
        rec.push(Statement {
            replying_to: Some(1),
            ..frame(Act::Question, usize::MAX)
        });
        assert!(!drifted(&rec));
    }

    #[test]
    fn test_meeting_pull_backs_come_from_the_rites() {
        // when a drift does happen, the room hears the organizer's words
        let mut steers = 0;
        for rec in meeting_records(0..200) {
            for stmt in &rec[2..rec.len() - 1] {
                if stmt.speaker == usize::MAX {
                    steers += 1;
                    assert_eq!(stmt.kind, ReplyKind::Framing);
                    assert_eq!(stmt.replying_to, Some(1));
                    assert!(MEET_RITES.pull_backs.contains(&stmt.text.as_str()),
                        "pull-back: {}", stmt.text);
                }
            }
        }
        eprintln!("steers across 200 meetings: {steers}");
        assert!(steers > 0, "drift should be reachable in the corpus");
    }

    #[test]
    fn test_cleric_prayer_reachable_for_sick_and_grieving() {
        let library = village_library();
        let mut cleric = Speaker::new("Anselm");
        cleric.ideology = "clergy";
        for status in ["sick", "grieving"] {
            let mut flock = Speaker::new("Bram");
            flock.status.push(status);
            let prayed = (0..50).any(|seed| {
                let tr = converse(&cleric, &flock, Relation::Acquaintance, None, &library, None, &mut Rng::new(seed));
                tr.turns.iter().any(|t| t.speaker == 0 && t.act == Act::Question)
                    && tr.turns.iter().any(|t| t.speaker == 1 && t.act == Act::Answer)
            });
            assert!(prayed, "a {} parishioner never reaches the cleric's prayer", status);
        }
    }

    #[test]
    fn test_cleric_console_reachable_for_sick_and_grieving_parishioner() {
        // the_cleric_visits_the_sick and the_cleric_consoles tie on criterion
        // count when the listener is both sick and grieving. If they shared
        // CLERIC_PARISH as their rejoin, falling through to the second tied
        // tree asked the same generic parish question twice and the grief
        // thread was never reached. Each now rejoins into its own
        // topic-appropriate exchange (VISIT_CARE / CONDOLE_OFFER).
        let library = village_library();
        let mut cleric = Speaker::new("Anselm");
        cleric.ideology = "clergy";
        let mut flock = Speaker::new("Bram");
        flock.status.push("sick");
        flock.status.push("grieving");

        // "the_cleric_visits_the_sick" always wins the tie and supplies the
        // root; "the_cleric_consoles" can only ever appear via its own
        // rejoin (advance_to_rejoin skipping past the first tree). So any
        // turn tagged with that tree name must come from that rejoin
        // exchange - which must carry grief content, not the generic,
        // grief-less CLERIC_PARISH small talk.
        let mut saw_the_cleric_consoles_rejoin = false;
        for seed in 0..50 {
            let tr = converse(&cleric, &flock, Relation::Acquaintance, None, &library, None, &mut Rng::new(seed));
            // speaker 0 (the cleric) opens the rejoin exchange with its `calls`
            // line; that's the line whose concepts identify which exchange it is.
            for t in tr.turns.iter().filter(|t| t.tree == "the_cleric_consoles" && t.speaker == 0) {
                saw_the_cleric_consoles_rejoin = true;
                assert!(t.concepts.contains(&"concern:grief"),
                    "the_cleric_consoles' rejoin opened without grief content: {:?}", t);
            }
        }
        assert!(saw_the_cleric_consoles_rejoin, "the_cleric_consoles' rejoin was never reached across 50 seeds");
    }

    #[test]
    fn test_house_meeting_grief_participates_after_question_pull() {
        let organizer = Speaker::new("Reeve");
        let a = Speaker::new("Eliza");
        let mut b = Speaker::new("Bram");
        b.status.push("grieving");
        let pool_a = meeting_pool(&a);
        let pool_b = meeting_pool(&b);
        let pools: Vec<&[MeetingLine]> = vec![&pool_a, &pool_b];
        let attendees = [&a, &b];

        let mut saw_grief_after_q = false;
        for seed in 0..50 {
            let mut rng = Rng::new(seed);
            let statements = run_meeting(&organizer, &attendees, &[], &MEET_PROMPT, &pools, &MEET_RITES, &mut rng);
            // Check if any statement with index >= 4 (after initial answers) contains concern:grief
            for stmt in statements.iter().skip(4) {
                if stmt.concepts.contains(&"concern:grief") {
                    saw_grief_after_q = true;
                    break;
                }
            }
            if saw_grief_after_q { break; }
        }
        assert!(saw_grief_after_q, "Grieving attendee must be able to contribute concern:grief after question pull");
    }

    /// #1185: every meeting opens with a rite and closes with a send-off,
    /// and the floor never takes the rite up as a point of business.
    #[test]
    fn test_every_meeting_is_framed() {
        for rec in meeting_records(0..50) {
            assert_eq!(rec[0].act, Act::Greet, "no rite opened the meeting");
            assert!(rec[0].concepts.contains(&"ritual"));
            assert_eq!(rec[1].act, Act::Question, "the question should follow the rite");
            assert_eq!(rec.last().unwrap().act, Act::Farewell, "no send-off closed the meeting");
            assert!(rec[2..rec.len() - 1].iter().all(|s| s.replying_to != Some(0)),
                "the floor took the rite up - it should be framing, not a point");
        }
    }

    /// #1185: whose words the rite is - a pious attendee brings the
    /// church's prayer, old-ways folk their own, a gregarious one a
    /// verse; when the room offers nobody the organizer does it plain.
    #[test]
    fn test_the_room_gives_the_rite_in_its_own_words() {
        let organizer = Speaker::new("Reeve");
        let run = |attendees: &[&Speaker]| -> Vec<Statement> {
            let pools: Vec<Vec<MeetingLine>> = attendees.iter().map(|s| meeting_pool(s)).collect();
            let pools: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
            run_meeting(&organizer, attendees, &[], &MEET_PROMPT, &pools, &MEET_RITES, &mut Rng::new(7))
        };
        let mut pious = Speaker::new("Anselm");
        pious.traits.push(Trait::Pious);
        let mut old_pious = Speaker::new("Morag");
        old_pious.traits.push(Trait::Pious);
        old_pious.status.push("old_ways");
        let mut greg = Speaker::new("Hob");
        greg.traits.push(Trait::Gregarious);
        let plain = Speaker::new("Corvin");

        // a pious attendee brings the prayer and gives the benediction
        let rec = run(&[&pious, &plain]);
        assert_eq!(rec[0].speaker, 0);
        assert!(MEET_RITES.prayers.contains(&rec[0].text.as_str()), "rite: {}", rec[0].text);
        assert_eq!(rec.last().unwrap().speaker, 0);
        assert!(MEET_RITES.blessings.contains(&rec.last().unwrap().text.as_str()));

        // old-ways folk get their own words, opening and closing
        let rec = run(&[&old_pious, &plain]);
        assert!(MEET_RITES.old_rites.contains(&rec[0].text.as_str()), "rite: {}", rec[0].text);
        assert!(MEET_RITES.old_closes.contains(&rec.last().unwrap().text.as_str()),
            "close: {}", rec.last().unwrap().text);

        // nobody devout: a gregarious one offers a verse, but the
        // send-off falls to the organizer
        let rec = run(&[&greg, &plain]);
        assert_eq!(rec[0].speaker, 0);
        assert!(MEET_RITES.verses.contains(&rec[0].text.as_str()), "rite: {}", rec[0].text);
        assert_eq!(rec.last().unwrap().speaker, usize::MAX);
        assert!(MEET_RITES.plain_closes.contains(&rec.last().unwrap().text.as_str()));

        // a bare room: the organizer does it all
        let rec = run(&[&plain]);
        assert_eq!(rec[0].speaker, usize::MAX);
        assert!(MEET_RITES.plain_opens.contains(&rec[0].text.as_str()), "rite: {}", rec[0].text);
        assert!(MEET_RITES.plain_closes.contains(&rec.last().unwrap().text.as_str()));
    }

    #[test]
    fn test_relation_colors_the_same_talk() {
        let library = village_library();
        let mut sick = Speaker::new("Eliza");
        sick.status.push("sick");
        let other = Speaker::new("Corvin");

        // Same seed, same speakers, same leading tree - only the bond
        // changes. The register should differ wherever a voice is authored.
        let mut differed = 0;
        for seed in 0..20 {
            let mut ra = Rng::new(seed);
            let rival = converse(&sick, &other, Relation::Rival, None, &library, None, &mut ra);
            let mut fa = Rng::new(seed);
            let friend = converse(&sick, &other, Relation::Friend, None, &library, None, &mut fa);
            assert_eq!(rival.trees_walked[0], "the_sickness");
            assert_eq!(friend.trees_walked[0], "the_sickness");
            if rival.lines() != friend.lines() { differed += 1; }
        }
        assert_eq!(differed, 20, "rival and friend talks should read differently every time");
    }

    #[test]
    fn test_sick_rival_speaks_in_a_rivals_voice() {
        let library = village_library();
        let mut sick = Speaker::new("Eliza");
        sick.status.push("sick");
        let other = Speaker::new("Corvin");

        // The ticket's exact complaint: a sick rival used to open with the
        // same warm greeting a friend got. The sickness content stays but
        // the first beat is spoken in the register's own words.
        let base_calls = [
            "*cough* Good morrow, {other}.",
            "Forgive my rough voice - this chest cold won't quit.",
            "Morrow, {other}. Don't stand too close in the breeze.",
            "A heavy morning, neighbor. My limbs are like lead.",
        ];
        for seed in 0..10 {
            let mut ra = Rng::new(seed);
            let tr = converse(&sick, &other, Relation::Rival, None, &library, None, &mut ra);
            assert_eq!(tr.trees_walked[0], "the_sickness");
            let opening = &tr.turns[0].text;
            assert!(!base_calls.iter().any(|t| t.replace("{other}", "Corvin") == *opening),
                "a rival should not open with the base greeting: {opening}");
        }
    }

    #[test]
    fn test_voice_spent_falls_back_to_base_lines() {
        let voice = [Voice { criteria: &[Criterion::Has("pair:relation:rival")], lines: &["Cold line"] }];
        let mut rng = Rng::new(0);
        let empty: HashSet<&'static str> = HashSet::default();
        let a = Speaker::new("A");
        let b = Speaker::new("B");

        // Fresh voice wins over fresh base
        let rival = Facts::for_roles(&a, &b, Relation::Rival);
        let (t, _) = fresh_voiced(&["Base line"], &voice, &rival, &a, &b, &empty, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(t, "Cold line");

        // Spent voice hands the floor back to the base lines
        let mut used: HashSet<&'static str> = HashSet::default();
        used.insert("Cold line");
        let (t, _) = fresh_voiced(&["Base line"], &voice, &rival, &a, &b, &used, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(t, "Base line");

        // A voice for another register never speaks
        let friend = Facts::for_roles(&a, &b, Relation::Friend);
        let (t, _) = fresh_voiced(&["Base line"], &voice, &friend, &a, &b, &empty, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(t, "Base line");
    }

    #[test]
    fn test_rank_colors_the_same_talk() {
        let library = village_library();
        let mut lord = Speaker::new("Sir Aldric");
        lord.ideology = "lord";
        let mut merchant = Speaker::new("Marta");
        merchant.ideology = "merchant";
        let laborer = Speaker::new("Corvin");

        // The lord holds court where the laborer would smalltalk
        for seed in 0..5 {
            let mut ra = Rng::new(seed);
            let tr = converse(&lord, &laborer, Relation::Acquaintance, None, &library, None, &mut ra);
            assert_eq!(tr.trees_walked[0], "the_lords_levee");
            assert!(tr.turns.iter().any(|t| t.speaker == 1 && t.text.contains("m'lord")),
                "the tenant never deferred to rank: {:?}", tr.lines());
        }
        // The merchant keeps the ledger instead
        let mut ra = Rng::new(0);
        let tr = converse(&merchant, &laborer, Relation::Acquaintance, None, &library, None, &mut ra);
        assert_eq!(tr.trees_walked[0], "the_merchants_ledger");
        // And the way up still runs through the manor
        let mut ra = Rng::new(0);
        let tr = converse(&laborer, &lord, Relation::Acquaintance, None, &library, None, &mut ra);
        assert_eq!(tr.trees_walked[0], "lord_manor");
    }

    #[test]
    fn test_a_sick_lord_still_hears_his_title() {
        let library = village_library();
        let mut sick_lord = Speaker::new("Sir Aldric");
        sick_lord.ideology = "lord";
        sick_lord.status.push("sick");
        let tenant = Speaker::new("Corvin");

        // The sickness tree outranks the levee, but the tenant's replies
        // still carry the manor's address
        for seed in 0..10 {
            let mut ra = Rng::new(seed);
            let tr = converse(&sick_lord, &tenant, Relation::Acquaintance, None, &library, None, &mut ra);
            assert_eq!(tr.trees_walked[0], "the_sickness");
            assert!(tr.turns.iter().any(|t| t.speaker == 1 && t.text.contains("m'lord")),
                "no deference in the tenant's replies: {:?}", tr.lines());
        }
    }

    #[test]
    fn test_trade_slot_fills_from_the_speakers_work() {
        // #1181: {trade}/{other_trade} read each side's work-noun
        let mut smith = Speaker::new("Bruno");
        smith.status.push("smith");
        let mut fisher = Speaker::new("Wat");
        fisher.status.push("fisher");
        let hand = Speaker::new("Corvin");

        let mut rng = Rng::new(0);
        let used: HashSet<&'static str> = HashSet::default();
        let (_, text) = fresh_line(&["How goes it at {other_trade}, {other}?"],
            &smith, &fisher, &used, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(text, "How goes it at the weir, Wat?");
        let (_, text) = fresh_line(&["{trade} keeps my hands full."],
            &smith, &fisher, &used, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(text, "the forge keeps my hands full.");

        // No craft named: the day's wage fills the slot, not a raw "{trade}"
        let (_, text) = fresh_line(&["Hard at it down at {other_trade}?"],
            &hand, &fisher, &used, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(text, "Hard at it down at the weir?");
        let (_, text) = fresh_line(&["{trade} keeps my hands full."],
            &hand, &fisher, &used, &HashSet::default(), false, &mut rng).unwrap();
        assert_eq!(text, "the day's work keeps my hands full.");
    }

    #[test]
    fn test_a_smith_and_a_fisher_talk_different_work() {
        // #1181: same seed, different trades - the work beat names their own
        let library = village_library();
        let mut smith = Speaker::new("Bruno");
        smith.status.push("smith");
        let mut fisher = Speaker::new("Wat");
        fisher.status.push("fisher");
        let hand = Speaker::new("Corvin");

        let mut work_seen = false;
        for seed in 0..20 {
            let mut ra = Rng::new(seed);
            let tr = converse(&hand, &smith, Relation::Acquaintance, None, &library, None, &mut ra);
            if tr.turns.iter().any(|t| t.text.contains("forge")) {
                work_seen = true;
                break;
            }
            let mut ra = Rng::new(seed);
            let tr = converse(&hand, &fisher, Relation::Acquaintance, None, &library, None, &mut ra);
            if tr.turns.iter().any(|t| t.text.contains("weir")) {
                work_seen = true;
                break;
            }
        }
        assert!(work_seen, "the work beat never named a trade in 40 talks");

        // And the same seed's work-beat text differs by who holds the bench
        let mut diffs = 0;
        for seed in 0..20 {
            let mut ra = Rng::new(seed);
            let a = converse(&hand, &smith, Relation::Acquaintance, None, &library, None, &mut ra);
            let mut rb = Rng::new(seed);
            let b = converse(&hand, &fisher, Relation::Acquaintance, None, &library, None, &mut rb);
            if a.lines() != b.lines() { diffs += 1; }
        }
        assert!(diffs > 0, "smith and fisher talks were identical on every seed");
    }

    fn audition_talk(traits: &[Trait], rel: Relation, pass: bool) -> Transcript {
        let mut caller = Speaker::new("Hob");
        caller.traits = traits.to_vec();
        caller.status.push("sick"); // a status tree to lose, so depth shows
        let probe = probe_for(&caller);
        let you = Speaker::new("Wanderer");
        // the offered previews are realized text (E4-4): match them
        // against the pools realized with the pair's own patch
        let mut yours = you.clone();
        let mut theirs = caller.clone();
        yours.rel = rel;
        theirs.rel = rel;
        yours.tv = crate::game::text::tv_for(&yours, &theirs, rel,
            matches!(rel, Relation::Rival));
        let cx = crate::game::text::Cx::plain(&yours, &theirs, &[]);
        // at the audition - both options drawn from the probe's two
        // pools - pick the landing or the wrong note as asked
        converse_with(&caller, &you, rel, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                if let Some(p) = probe {
                    let in_pool = |c: &Choice, pool: &[&'static str]| pool.iter()
                        .any(|l| crate::game::text::realize(l, &cx,
                            &mut Rng::new(0)) == c.preview);
                    if ch.iter().all(|c| in_pool(c, p.pass.1) || in_pool(c, p.fail.1)) {
                        let wanted = if pass { p.pass.1 } else { p.fail.1 };
                        return ch.iter().position(|c| in_pool(c, wanted));
                    }
                }
                Some(0)
            }, &mut Rng::new(7))
    }

    #[test]
    fn test_the_audition_reads_the_answer() {
        // the same talk, answered two ways (#1187): the answer that
        // lands opens them up; the wrong note leaves it polite
        let safe = audition_talk(&[Trait::Stingy], Relation::Acquaintance, true);
        let miss = audition_talk(&[Trait::Stingy], Relation::Acquaintance, false);
        assert_eq!(safe.vibe, Some(Vibe::Safe), "{:?}", safe.lines());
        assert_eq!(miss.vibe, Some(Vibe::Mismatch), "{:?}", miss.lines());
        assert!(safe.trees_walked.contains(&PROBE_TREE) && miss.trees_walked.contains(&PROBE_TREE));
        // a miss leaves the talk on the polite tail - after the probe
        // beat only smalltalk remains
        let after = |tr: &Transcript| {
            let at = tr.turns.iter().position(|t| t.tree == PROBE_TREE).unwrap();
            tr.turns[at + 2..].iter().all(|t| t.tree == "village_smalltalk")
        };
        assert!(after(&miss), "a miss should leave only the tail: {:?}", miss.trees_walked);
        // and the safe answer's talk isn't confined to it
        assert!(safe.turns.iter().any(|t| t.tree != "village_smalltalk" && t.tree != PROBE_TREE),
            "a pass should keep deeper material: {:?}", safe.trees_walked);
    }

    #[test]
    fn test_the_probe_is_their_trait() {
        // each test in the book belongs to its trait, and a caller with
        // none of them doesn't probe at all
        for p in crate::game::dialogue_trees::PROBE_TESTS {
            let tr = audition_talk(&[p.who], Relation::Acquaintance, true);
            assert!(tr.trees_walked.contains(&PROBE_TREE), "{:?} never probed", p.who);
            let call = tr.turns.iter().find(|t| t.tree == PROBE_TREE && t.speaker == 0).unwrap();
            // the call on the record is realized text - compare like to
            // like with the caller's own pair patch (E4-4)
            let mut hob = Speaker::new("Hob");
            hob.traits = vec![p.who];
            hob.status.push("sick");
            let you = Speaker::new("Wanderer");
            hob.rel = Relation::Acquaintance;
            hob.tv = crate::game::text::tv_for(&hob, &you, Relation::Acquaintance, false);
            let cx = crate::game::text::Cx::plain(&hob, &you, &[]);
            let calls: Vec<String> = p.calls.iter()
                .map(|l| crate::game::text::realize(l, &cx, &mut Rng::new(0)))
                .collect();
            assert!(calls.contains(&call.text), "{:?}: {:?} not theirs", p.who, call.text);
        }
        let none = audition_talk(&[], Relation::Acquaintance, true);
        assert!(!none.trees_walked.contains(&PROBE_TREE));
        assert_eq!(none.vibe, None);
    }

    #[test]
    fn test_old_friends_dont_audition() {
        // the audition is for early talk: kin and friends are past it
        for rel in [Relation::Friend, Relation::Kin, Relation::Rival] {
            let tr = audition_talk(&[Trait::Pious], rel, true);
            assert!(!tr.trees_walked.contains(&PROBE_TREE), "{rel:?} still auditioned");
            assert_eq!(tr.vibe, None);
        }
        // and nobody probes a villager-to-villager talk
        let mut a = Speaker::new("Hob");
        a.traits = vec![Trait::Pious];
        let tr = converse(&a, &Speaker::new("Tilda"), Relation::Acquaintance, None,
            &village_library(), None, &mut Rng::new(7));
        assert!(!tr.trees_walked.contains(&PROBE_TREE));
    }

    #[test]
    fn test_a_hothead_wants_you_to_stand_up() {
        // the right answer to their prod is iron, not agreement - the
        // one probe where pushing back is the safe read
        let tr = audition_talk(&[Trait::Hotheaded], Relation::Acquaintance, true);
        assert_eq!(tr.vibe, Some(Vibe::Safe));
        let answered = tr.turns.iter().find(|t| t.tree == PROBE_TREE && t.speaker == 1).unwrap();
        assert_eq!(answered.act, Act::PushBack, "the safe answer is the stand-up");
        // and meek agreement is the wrong note
        let miss = audition_talk(&[Trait::Hotheaded], Relation::Acquaintance, false);
        assert_eq!(miss.vibe, Some(Vibe::Mismatch));
    }

    /// An acquaintance who'll share the personal layer (E3-2): warm
    /// enough for tier 2, three talks in, unheard
    fn warm_open() -> Open {
        Open { intimacy: 0.6, benevolence: 0.5, integrity: 0.4, safety: 0.5,
            mood: 0.0, talks: 3, earshot: false, shared: 1 }
    }

    fn ask_talk(status: &'static str, want: &[&'static str]) -> Transcript {
        // a talk where your picks follow the wanted previews, falling
        // to the first option when none of them's showing
        let mut caller = Speaker::new("Hob");
        caller.status.push(status);
        caller.open = warm_open();
        let you = Speaker::new("Wanderer");
        let mut queue: Vec<&'static str> = want.to_vec();
        converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                if let Some(&w) = queue.first() {
                    if let Some(i) = ch.iter().position(|c| c.preview == w) {
                        queue.remove(0);
                        return Some(i);
                    }
                }
                ch.iter().position(|c| c.act != Act::Farewell).or(Some(0))
            }, &mut Rng::new(7))
    }

    #[test]
    fn test_you_can_put_a_question_to_them() {
        // once the hello's answered, an ask-slot opens in the picks
        // (#1189): it opens a menu of real questions, their answer
        // lands, and the talk picks up where it stood
        let tr = ask_talk("sick", &["Put a question to them", "Ask about their sickness"]);
        assert_eq!(tr.asked, vec!["their sickness"], "{:?}", tr.lines());
        assert!(tr.trees_walked.contains(&ASK_TREE));
        let q = tr.turns.iter().find(|t| t.tree == ASK_TREE && t.speaker == 1).unwrap();
        assert_eq!(q.act, Act::Question);
        let a = tr.turns.iter().find(|t| t.tree == ASK_TREE && t.speaker == 0).unwrap();
        assert_eq!(a.act, Act::Answer);
        // and asking isn't a farewell - the talk goes on past the beat
        let at = tr.turns.iter().position(|t| t.tree == ASK_TREE).unwrap();
        assert!(at + 2 < tr.turns.len(), "the asking closed the talk: {:?}", tr.lines());
    }

    #[test]
    fn test_the_ask_slot_waits_past_the_hello() {
        // no question slot on the opening answer - the hello gets its
        // say first (#1189)
        let mut caller = Speaker::new("Hob");
        caller.status.push("sick");
        let you = Speaker::new("Wanderer");
        let mut first_seen = false;
        converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                if !first_seen {
                    first_seen = true;
                    assert!(ch.iter().all(|c| c.preview != "Put a question to them"),
                        "the ask slot showed on the hello: {:?}", ch.iter().map(|c| &c.preview).collect::<Vec<_>>());
                }
                Some(0)
            }, &mut Rng::new(7));
    }

    #[test]
    fn test_the_topics_seed_from_who_they_are() {
        // the menu carries the standing asks plus what their state
        // opens - and not what it doesn't (#1189)
        for (status, want, banned) in [
            ("sick", "Ask about their sickness", "Ask about the road"),
            ("grieving", "Ask about their grief", "Ask about their sickness"),
            ("hungry", "Ask about their hunger", "Ask about their grief"),
            ("visitor", "Ask about the road", "Ask about the village"),
            ("keeps_secret", "Ask about what weighs on them", "Ask about the trouble they told"),
            ("shared", "Ask about the trouble they told", "Ask about what weighs on them"),
        ] {
            let mut caller = Speaker::new("Hob");
            caller.status.push(status);
            caller.open = warm_open();
            let you = Speaker::new("Wanderer");
            // some status talks run dry before a second beat - try a
            // few seeds until one lives long enough to open the menu
            let mut menu: Vec<String> = vec![];
            for seed in 0..20 {
                converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
                    &mut |ch: &[Choice]| {
                        if ch.last().map(|c| c.preview == "Say nothing of it").unwrap_or(false) {
                            if menu.is_empty() {
                                menu = ch.iter().map(|c| c.preview.clone()).collect();
                            }
                            return ch.iter().position(|c| c.preview == "Say nothing of it");
                        }
                        if menu.is_empty() {
                            return ch.iter().position(|c| c.preview == "Put a question to them")
                                .or_else(|| ch.iter().position(|c| c.act != Act::Farewell))
                                .or(Some(0));
                        }
                        ch.iter().position(|c| c.act != Act::Farewell).or(Some(0))
                    }, &mut Rng::new(7 + seed));
                if !menu.is_empty() { break; }
            }
            assert!(menu.iter().any(|p| p == want), "{status}: {:?} lacks {want}", menu);
            assert!(menu.iter().all(|p| p != banned), "{status}: {banned} wrongly on {:?}", menu);
            // the standing asks never leave the table
            assert!(menu.iter().any(|p| p == "Ask about themselves"), "{status}: {:?}", menu);
        }
    }

    #[test]
    fn test_an_asked_topic_leaves_the_menu() {
        // each question's a beat, once - asked, it's off the menu for
        // the rest of the talk (#1189)
        let mut caller = Speaker::new("Hob");
        caller.status.push("sick");
        caller.open = warm_open();
        let you = Speaker::new("Wanderer");
        let mut menus: Vec<Vec<String>> = vec![];
        let mut took_sick = false;
        converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                if ch.last().map(|c| c.preview == "Say nothing of it").unwrap_or(false) {
                    menus.push(ch.iter().map(|c| c.preview.clone()).collect());
                    if !took_sick {
                        took_sick = true;
                        return ch.iter().position(|c| c.preview == "Ask about their sickness");
                    }
                    return ch.iter().position(|c| c.preview == "Say nothing of it");
                }
                ch.iter().position(|c| c.preview == "Put a question to them").or(Some(0))
            }, &mut Rng::new(7));
        assert!(menus.len() >= 2, "the menu never reopened: {:?}", menus);
        assert!(menus[0].iter().any(|p| p == "Ask about their sickness"));
        assert!(menus[1..].iter().all(|m| !m.iter().any(|p| p == "Ask about their sickness")),
            "a spent topic stayed on: {:?}", menus[1]);
    }

    // --- the hog: a villager who never trades, only talks (#1252) ---

    fn reply(act: Act) -> Reply {
        Reply { criteria: &[], weight: 1.0, bias: &[], act, respond: None,
            concepts: &[], lines: &["x"], voice: &[], then: None, end: None }
    }

    #[test]
    fn test_the_hog_never_asks_back() {
        let mut hog = Speaker::new("Nog");
        hog.traits.push(Trait::SelfAbsorbed);
        let plain = Speaker::new("Pell");
        // a question for you is a turn not about them - near never drawn
        assert!(draw_weight(&reply(Act::Question), &hog)
            < draw_weight(&reply(Act::Question), &plain) * 0.1,
            "the hog's ask-back is all but gone");
        // and whatever they'd answer turns to their own story
        for act in [Act::Remark, Act::Answer] {
            assert!(draw_weight(&reply(act), &hog) > draw_weight(&reply(act), &plain),
                "the hog's own telling draws heavier: {act:?}");
        }
    }

    #[test]
    fn test_the_hog_bids_past_the_room() {
        let mut hog = Speaker::new("Nog");
        hog.traits.push(Trait::SelfAbsorbed);
        assert!(bid_traits(&hog) > bid_traits(&Speaker::new("Pell")),
            "a held audience is the whole of their art");
    }

    #[test]
    fn test_the_room_pushes_back_on_the_floor_not_the_matter() {
        use crate::game::dialogue_trees::meeting_pool;
        let mut hog = Speaker::new("Nog");
        hog.traits.push(Trait::SelfAbsorbed);
        hog.traits.push(Trait::Gregarious);
        let plain = Speaker::new("Pell");
        // a push-back line from a plain pool, aimed at each
        let push = meeting_pool(&Speaker::new("Pell")).iter()
            .find(|l| l.act == Act::PushBack).copied().expect("a pool has a push-back");
        assert_eq!(floor_lines(&push, &hog), HOG_FLOOR,
            "pushing back on a hog talks about the floor");
        assert_eq!(floor_lines(&push, &plain), push.lines,
            "pushing back on anyone else talks about the matter");
        assert!(HOG_FLOOR.iter().any(|l| l.contains("{other_trade}")),
            "the flavor names their trade");
    }

    #[test]
    fn test_a_hog_brings_their_own_material() {
        use crate::game::dialogue_trees::meeting_pool;
        let mut hog = Speaker::new("Nog");
        hog.traits.push(Trait::SelfAbsorbed);
        let pool = meeting_pool(&hog);
        assert!(pool.iter().any(|l| l.bias.iter()
            .any(|(t, _)| *t == Trait::SelfAbsorbed)),
            "a hog carries lines only they would say");
        assert!(!meeting_pool(&Speaker::new("Pell")).iter()
            .any(|l| l.bias.iter().any(|(t, _)| *t == Trait::SelfAbsorbed)),
            "nobody else does");
    }

    #[test]
    fn test_an_early_goodbye_finds_a_livelier_tree() {
        // #1368: a parting before the talk's had its say is a brush-off -
        // early on, a pool of nothing but farewells falls through to the
        // next tree's rejoin instead of closing at the hello
        const BRUSHOFF: Exchange = Exchange {
            act: Act::Greet, concepts: &["greeting"],
            calls: &["Morrow to thee."], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Farewell,
                respond: None,
                concepts: &["farewell"], lines: &["I must be on my way."],
                voice: &[], then: None, end: Some(Ending::Farewell),
            }],
        };
        const CHAT: Exchange = Exchange {
            act: Act::Remark, concepts: &["weather"],
            calls: &["Looks like rain again."], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Remark,
                respond: None,
                concepts: &["weather"], lines: &["Aye, grey all week."],
                voice: &[], then: None, end: Some(Ending::Farewell),
            }],
        };
        const EARLY: DialogueTree = DialogueTree {
            name: "early_exit", criteria: &[], root: &BRUSHOFF, rejoin: None };
        const TAIL: DialogueTree = DialogueTree {
            name: "chattier", criteria: &[], root: &CHAT, rejoin: Some(&CHAT) };
        for seed in 0..10 {
            let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
                Relation::Acquaintance, None, &[EARLY, TAIL], None, &mut Rng::new(seed));
            assert_eq!(tr.turns[1].act, Act::Remark,
                "the talk moved to the livelier tree, not the early goodbye");
            assert!(tr.turns.iter().take(2).all(|t| t.act != Act::Farewell),
                "no farewell inside the hello: {:?}", tr.turns.iter().map(|t| &t.text).collect::<Vec<_>>());
        }
        // and with nowhere livelier to go the talk dries rather than
        // trade a hello for a goodbye
        for seed in 0..5 {
            let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
                Relation::Acquaintance, None, &[EARLY], None, &mut Rng::new(seed));
            assert_eq!(tr.turns.len(), 1, "the hello stands alone: {:?}", tr.turns.len());
            assert_eq!(tr.ending, Some(Ending::Exhausted));
        }
    }

    #[test]
    fn test_a_circling_talk_drifts_apart() {
        // #1368: a node whose material repeats without naming anything
        // new is the doldrums - six unadvanced turns and they part
        const LOOP: Exchange = Exchange {
            act: Act::Remark, concepts: &["weather"],
            calls: &["Looks like rain.", "Grey sky again.", "Cold wind today.",
                "The clouds hang low.", "Damp underfoot.", "Dark by four, I warrant.",
                "The lane's all mud.", "A raw morning.", "Wet through, this week.",
                "The thatch is dripping."],
            voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Remark,
                respond: None,
                concepts: &["weather"],
                lines: &["Aye, that it is.", "So it is.", "Truth in that.",
                    "Can't argue.", "Aye, neighbor.", "Indeed.", "Hmm, aye.",
                    "That's the way of it.", "Just so.", "Well seen."],
                voice: &[], then: None, end: None,
            }],
        };
        const LOOP_TREE: DialogueTree = DialogueTree {
            name: "loop", criteria: &[], root: &LOOP, rejoin: None };
        for seed in 0..10 {
            let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
                Relation::Acquaintance, None, &[LOOP_TREE], None, &mut Rng::new(seed));
            assert!(tr.turns.len() <= AMBIENT_MAX_TURNS,
                "the doldrums end it before the cap: {} turns", tr.turns.len());
            assert_eq!(tr.ending, Some(Ending::Exhausted),
                "a circling talk drifts apart");
            assert!(tr.turns.len() <= 8,
                "six unadvanced turns is the doldrums: {} turns", tr.turns.len());
        }
    }

    #[test]
    fn test_every_reply_follows_the_call() {
        // #1367: across seeds, relations and pairs, whatever the
        // responder says fits the act it answers - a question never
        // draws a goodbye while the filter stands
        let library = village_library();
        let names = ["Hob", "Tilda", "Aldric", "Marga", "Peryn", "Bram"];
        for seed in 0..40 {
            let mut rng = Rng::new(seed);
            for rel in [Relation::Stranger, Relation::Acquaintance,
                        Relation::Friend, Relation::Kin, Relation::Rival] {
                let a = Speaker::new(names[rng.below(names.len())]);
                let b = Speaker::new(names[rng.below(names.len())]);
                let tr = converse(&a, &b, rel, None, &library, None, &mut rng);
                assert!(tr.turns.len() <= AMBIENT_MAX_TURNS,
                    "an ambient talk stays inside its cap: {}", tr.turns.len());
                for w in tr.turns.windows(2) {
                    if w[0].speaker == 0 && w[1].speaker == 1 {
                        assert!(act_fit(w[0].act, w[1].act) > 0,
                            "seed {seed} {rel:?}: {:?} answered by {:?}: {:?} -> {:?}",
                            w[0].act, w[1].act, w[0].text, w[1].text);
                    }
                }
            }
        }
    }

    #[test]
    fn test_a_question_nobody_can_answer_dries_up() {
        // #1367: when every fork left can't follow the call, the
        // responder stays silent and the talk closes - no question
        // answered by a goodbye
        const NO_ANSWER: Exchange = Exchange {
            act: Act::Question, concepts: &[],
            calls: &["Hast thou seen the miller?"], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Farewell,
                respond: None,
                concepts: &[], lines: &["Fare thee well."], voice: &[],
                then: None, end: Some(Ending::Farewell),
            }],
        };
        const Q_TREE: DialogueTree = DialogueTree {
            name: "q_no_answer", criteria: &[], root: &NO_ANSWER, rejoin: None,
        };
        for seed in 0..10 {
            let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
                Relation::Acquaintance, None, &[Q_TREE], None, &mut Rng::new(seed));
            assert_eq!(tr.turns.len(), 1,
                "nobody could answer but a reply was spoken: {:?}",
                tr.turns.iter().map(|t| &t.text).collect::<Vec<_>>());
            assert_eq!(tr.ending, Some(Ending::Exhausted));
        }
        // a dodge still rides when it's the only non-answer left
        const DODGE: Exchange = Exchange {
            act: Act::Question, concepts: &[],
            calls: &["Hast thou seen the miller?"], voice: &[],
            replies: &[
                Reply {
                    criteria: &[], weight: 1.0, bias: &[], act: Act::Farewell,
                    respond: None,
                    concepts: &[], lines: &["Fare thee well."], voice: &[],
                    then: None, end: Some(Ending::Farewell),
                },
                Reply {
                    criteria: &[], weight: 1.0, bias: &[], act: Act::Remark,
                    respond: None,
                    concepts: &[], lines: &["The mill sees whom it sees."], voice: &[],
                    then: None, end: None,
                },
            ],
        };
        const D_TREE: DialogueTree = DialogueTree {
            name: "q_dodge", criteria: &[], root: &DODGE, rejoin: None,
        };
        for seed in 0..10 {
            let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
                Relation::Acquaintance, None, &[D_TREE], None, &mut Rng::new(seed));
            assert_eq!(tr.turns[1].act, Act::Remark,
                "the remark was the only answer on offer: {:?}", tr.turns[1].text);
        }
        // and a goodbye still closes a talk that's had its say - once
        // past the hello, a lone parting fork is the polite exit (#1368:
        // it's only early that a pool of nothing but partings falls
        // through to a livelier tree)
        const CLOSING: Exchange = Exchange {
            act: Act::Remark, concepts: &[],
            calls: &["A fair morning to thee."], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Farewell,
                respond: None,
                concepts: &[], lines: &["And to thee."], voice: &[],
                then: None, end: Some(Ending::Farewell),
            }],
        };
        const MID: Exchange = Exchange {
            act: Act::Remark, concepts: &[],
            calls: &["The lane's all mud."], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Remark,
                respond: None,
                concepts: &[], lines: &["Every year about now."], voice: &[],
                then: Some(CLOSING), end: None,
            }],
        };
        const OPENER: Exchange = Exchange {
            act: Act::Remark, concepts: &[],
            calls: &["Grey sky again."], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Remark,
                respond: None,
                concepts: &[], lines: &["Aye, all week."], voice: &[],
                then: Some(MID), end: None,
            }],
        };
        const C_TREE: DialogueTree = DialogueTree {
            name: "r_closing", criteria: &[], root: &OPENER, rejoin: None,
        };
        let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
            Relation::Acquaintance, None, &[C_TREE], None, &mut Rng::new(1));
        assert_eq!(tr.turns.last().unwrap().act, Act::Farewell);
        assert_eq!(tr.ending, Some(Ending::Farewell));
    }

    #[test]
    fn test_the_village_doesnt_say_the_same_thing_twice() {
        // #1369: a line anyone said lately is passed over for fresher
        // words - the deterministic floor is that across a run of talks
        // no content line repeats while fresher words remained
        let library = village_library();
        let names = ["Hob", "Tilda", "Aldric", "Marga", "Peryn", "Bram",
                     "Oswyn", "Gisela", "Warin", "Edith", "Lucan", "Rohese"];
        // the village's memory holds a few days of talk, not every word
        // ever said - emulate it with a sliding window over the run
        let mut said: Vec<HashSet<&'static str>> = vec![];
        let mut rng = Rng::new(11);
        let mut repeats = 0usize;
        for _ in 0..30 {
            let recent: HashSet<&'static str> =
                said.iter().flatten().copied().collect();
            let a = Speaker::new(names[rng.below(names.len())]);
            let b = Speaker::new(names[rng.below(names.len())]);
            let rel = [Relation::Stranger, Relation::Acquaintance,
                       Relation::Friend, Relation::Rival][rng.below(4)];
            let tr = converse(&a, &b, rel, None, &library, Some(&recent), &mut rng);
            for turn in &tr.turns {
                let Some(l) = turn.line else { continue };
                if recent.contains(l) && !crate::game::dialogue_trees::STOCK_LINES.contains(&l)
                    && !matches!(turn.act, Act::Greet | Act::Farewell) {
                    repeats += 1;
                }
            }
            said.push(tr.turns.iter().filter_map(|t| t.line).collect());
            if said.len() > 4 { said.remove(0); }
        }
        // a repeat is only the dry-pool fallback; inside the window it
        // should be all but unheard of
        assert!(repeats <= 2, "{repeats} repeated content lines inside the window");
    }

    #[test]
    fn test_a_tired_line_still_lands_when_the_pool_is_dry() {
        // #1369: when everything a fork can say is a repeat, the talk
        // repeats rather than starve before its time
        const WORN: Exchange = Exchange {
            act: Act::Remark, concepts: &[],
            calls: &["Grey sky again."], voice: &[],
            replies: &[Reply {
                criteria: &[], weight: 1.0, bias: &[], act: Act::Remark,
                respond: None,
                concepts: &[], lines: &["Aye, all week."], voice: &[],
                then: None, end: None,
            }],
        };
        const WORN_TREE: DialogueTree = DialogueTree {
            name: "worn", criteria: &[], root: &WORN, rejoin: None,
        };
        let recent: HashSet<&'static str> = ["Grey sky again.", "Aye, all week."].into_iter().collect();
        let tr = converse(&Speaker::new("Hob"), &Speaker::new("Tilda"),
            Relation::Acquaintance, None, &[WORN_TREE], Some(&recent), &mut Rng::new(3));
        assert_eq!(tr.turns.len(), 2,
            "a dry pool still speaks what it has: {:?}", tr.turns.iter().map(|t| &t.text).collect::<Vec<_>>());
    }
}

#[cfg(test)]
mod e3_2 {
    use super::*;
    use crate::game::dialogue_trees::village_library;

    /// The spec formula itself, and the rung of it a pair reaches
    #[test]
    fn the_formula_reads_the_parts() {
        let o = Open { intimacy: 0.6, benevolence: 0.4, integrity: 0.5,
            safety: 0.5, mood: 0.5, talks: 0, earshot: false, shared: 2 };
        // .30*.6 + .25*.4 + .20*.5 + .15*.5 + .10*.5 = .505
        assert!((o.reach() - 0.505).abs() < 1e-6, "{}", o.reach());
        // ears near cost the exposure term
        let x = Open { earshot: true, ..o };
        assert!((x.reach() - 0.305).abs() < 1e-6);
        // and the tiers stand where the spec puts them
        assert!(o.reached(2, false, false, false));
        assert!(!x.reached(2, false, false, false), "ears close the private layer");
        assert!(!o.reached(3, false, false, false));
        let deep = Open { intimacy: 1.0, benevolence: 1.0, integrity: 1.0,
            safety: 1.0, mood: 1.0, talks: 9, earshot: false, shared: 2 };
        assert!(deep.reached(3, false, false, false));
        assert!(!Open { earshot: true, ..deep }.reached(3, false, false, false),
            "the core never opens with ears near");
        // a guarded one asks more of the core; a talkative one meets sooner
        let shy = Open { intimacy: 0.6, benevolence: 1.0, integrity: 0.5,
            safety: 1.0, mood: 1.0, talks: 9, earshot: false, shared: 2 };
        assert!(shy.reached(3, false, false, false), "reach .78 opens the core");
        assert!(!shy.reached(3, false, true, false), "guarded raises the core bar");
        assert!(!Open::default().reached(1, false, false, false));
        assert!(Open { talks: 1, ..Open::default() }.reached(1, true, false, false),
            "a talkative one opens the personal after a single talk");
        assert!(Open { talks: 2, ..Open::default() }.reached(1, false, false, false));
    }

    /// the pair's openness lands on the asked-about's own facts, the
    /// criteria a topic's tier reads
    #[test]
    fn the_open_facts_say_what_reaches() {
        let mut s = Speaker::new("Hob");
        s.open = Open { intimacy: 0.6, benevolence: 0.5, integrity: 0.4,
            safety: 0.5, mood: 0.0, talks: 3, earshot: false, shared: 1 };
        let f = Facts::for_roles(&s, &Speaker::new("Wanderer"), Relation::Friend);
        assert!(f.has("self:open:heart:2"), "warm enough for the private layer");
        assert!(f.has("self:open:village:1"));
        assert!(f.has("self:open:village:0"), "the public layer never closes");
        assert!(f.has("self:closed:heart:3"), "the core stays shut");
        // a stranger's facts are all closed past public
        let cold = Facts::for_roles(&Speaker::new("Nog"), &Speaker::new("Pell"), Relation::Stranger);
        assert!(cold.has("self:open:village:0"), "public still lists for a stranger");
        assert!(cold.has("self:closed:trouble:2"));
        assert!(!cold.has("self:open:work:1"));
    }

    /// a topic above the pair's openness never fires (the done-when):
    /// their grief stays off the open menu for someone barely known -
    /// and the named-tell ask that can't carry deflects instead
    #[test]
    fn a_too_deep_topic_never_fires() {
        let mut caller = Speaker::new("Hob");
        caller.status.push("grieving");
        caller.open = Open { talks: 1, ..Open::default() };
        let you = Speaker::new("Wanderer");
        let tr = converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                if ch.last().map(|c| c.preview == "Say nothing of it").unwrap_or(false) {
                    return ch.iter().position(|c| c.preview == "Ask about their grief")
                        .or_else(|| ch.iter().position(|c| c.preview == "Say nothing of it"));
                }
                ch.iter().position(|c| c.preview == "Put a question to them")
                    .or_else(|| ch.iter().position(|c| c.act != Act::Farewell)).or(Some(0))
            }, &mut Rng::new(7));
        assert_eq!(tr.vibe, Some(Vibe::Mismatch), "{:?}", tr.lines());
        assert!(tr.forward >= 1);
        assert!(tr.asked.is_empty(), "a deflected ask never draws: {:?}", tr.asked);
        assert!(tr.turns.iter().any(|t| t.line.map(|l| DEFLECTED.contains(&l)).unwrap_or(false)),
            "no turn-aside on the record: {:?}", tr.lines());
    }

    /// naming a tell the asker saw earns the bonus: grief worn openly
    /// opens what the bare reach wouldn't
    #[test]
    fn a_named_tell_opens_the_tier() {
        // just under the private bar: .15 + .10 + .04 = .29 closed;
        // the .2 bonus carries it
        let mut caller = Speaker::new("Hob");
        caller.status.push("grieving");
        caller.open = Open { intimacy: 0.5, benevolence: 0.4, integrity: 0.2,
            safety: 0.0, mood: 0.0, talks: 3, earshot: false, shared: 1 };
        let you = Speaker::new("Wanderer");
        let tr = converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                if ch.last().map(|c| c.preview == "Say nothing of it").unwrap_or(false) {
                    return ch.iter().position(|c| c.preview == "Ask about their grief")
                        .or_else(|| ch.iter().position(|c| c.preview == "Say nothing of it"));
                }
                ch.iter().position(|c| c.preview == "Put a question to them")
                    .or_else(|| ch.iter().position(|c| c.act != Act::Farewell)).or(Some(0))
            }, &mut Rng::new(7));
        assert_eq!(tr.forward, 0, "{:?}", tr.lines());
        assert_eq!(tr.asked, vec!["their grief"], "{:?}", tr.lines());
    }

    /// nothing unseen opens the named ask: a closed area with no wear
    /// showing simply isn't on the menu at all
    #[test]
    fn no_tell_no_named_ask() {
        let mut caller = Speaker::new("Hob");
        caller.status.push("keeps_secret");
        caller.open = Open::default();
        let you = Speaker::new("Wanderer");
        let mut saw_menu = false;
        for seed in 0..20 {
            converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
                &mut |ch: &[Choice]| {
                    if ch.last().map(|c| c.preview == "Say nothing of it").unwrap_or(false) {
                        saw_menu = true;
                        assert!(ch.iter().any(|c| c.preview == "Ask about the village"
                            || c.preview == "Ask about the day's work"),
                            "the public layer didn't list: {:?}",
                            ch.iter().map(|c| &c.preview).collect::<Vec<_>>());
                        // the public layer lists for anyone; what weighs
                        // on them and the tier-3 secret ask are closed
                        // but the kept secret's own wear tells on them
                        // (E3-4); everything else closed and tell-less
                        // stays off
                        assert!(ch.iter().all(|c| c.preview == "Say nothing of it"
                            || c.preview == "Ask about the village"
                            || c.preview == "Ask about the day's work"
                            || c.preview == "Ask about what weighs on them"
                            || c.preview == "Ask about what they never told"),
                            "a closed tell-less ask listed: {:?}",
                            ch.iter().map(|c| &c.preview).collect::<Vec<_>>());
                        return ch.iter().position(|c| c.preview == "Say nothing of it");
                    }
                    ch.iter().position(|c| c.preview == "Put a question to them")
                        .or_else(|| ch.iter().position(|c| c.act != Act::Farewell)).or(Some(0))
                }, &mut Rng::new(7 + seed));
            if saw_menu { break; }
        }
        assert!(saw_menu, "the ask slot never opened");
    }
}

#[cfg(test)]
mod e3_2_tells {
    use super::*;
    use crate::game::dialogue_trees::village_library;

    /// a tell names only the deep asks: grief worn openly can't stand
    /// in for time spent - the stranger's "themselves" stays off the
    /// menu while the grief's own question lists
    #[test]
    fn a_tell_names_only_the_deep_asks() {
        let mut caller = Speaker::new("Hob");
        caller.status.push("grieving");
        caller.open = Open::default();
        let you = Speaker::new("Wanderer");
        let mut saw_menu = false;
        for seed in 0..30 {
            converse_with(&caller, &you, Relation::Stranger, None, &village_library(), None, 1,
                &mut |ch: &[Choice]| {
                    if ch.last().map(|c| c.preview == "Say nothing of it").unwrap_or(false) {
                        saw_menu = true;
                        assert!(ch.iter().all(|c| c.preview != "Ask about themselves"),
                            "a tell bought the personal layer: {:?}",
                            ch.iter().map(|c| &c.preview).collect::<Vec<_>>());
                        return ch.iter().position(|c| c.preview == "Say nothing of it");
                    }
                    ch.iter().position(|c| c.preview == "Put a question to them")
                        .or_else(|| ch.iter().position(|c| c.act != Act::Farewell)).or(Some(0))
                }, &mut Rng::new(11 + seed));
            if saw_menu { break; }
        }
        assert!(saw_menu, "the ask slot never opened");
    }
}

#[cfg(test)]
mod e3_3 {
    use super::*;
    use crate::game::dialogue_trees::village_library;

    /// a reply's tag resolves at the pick and the chat's warmth reads
    /// it (E3-3): care warms past reflection, a deflection chills what
    /// an agreeable answer would have warmed
    #[test]
    fn the_tags_say_how_the_answer_landed() {
        let tr = |tag: Option<Respond>| Transcript {
            speakers: ["A".into(), "B".into()], trees_walked: vec![],
            turns: vec![Turn { speaker: 0, act: Act::Question, respond: None,
                    text: "q".into(), concepts: vec![], tree: "t", line: None,
                    channel: crate::game::setting::Channel::Public },
                Turn { speaker: 1, act: Act::Answer, respond: tag,
                    text: "a".into(), concepts: vec![], tree: "t", line: None,
                    channel: crate::game::setting::Channel::Public }],
            ending: Some(Ending::Farewell), vibe: None, asked: vec![],
            forward: 0, evasions: vec![], worry_told: None, slips: vec![], jokes: vec![], shared: 0, overshot: false,
            setting: crate::game::setting::STREET };
        assert!(chat_temp(&tr(Some(Respond::Care))) > chat_temp(&tr(Some(Respond::Reflect))));
        assert!(chat_temp(&tr(Some(Respond::Deflect))) < chat_temp(&tr(None)),
            "a deflection reads colder than a bare answer");
        // and the untagged derive from the act
        assert_eq!(respond_of(Act::Answer), Respond::Reflect);
        assert_eq!(respond_of(Act::PushBack), Respond::Deflect);
        assert_eq!(respond_of(Act::Question), Respond::PivotAsk);
    }

    /// the gate wants reciprocation, not just warmth (E3-3): a bond
    /// warm but never told anything stalls at the personal layer
    #[test]
    fn warmth_without_sharing_stalls() {
        let warm = Open { intimacy: 0.6, benevolence: 0.5, integrity: 0.5,
            safety: 0.5, mood: 0.0, talks: 3, earshot: false, shared: 0 };
        assert!(!warm.reached(2, false, false, false),
            "the private layer wants something of yourself back");
        assert!(Open { shared: 1, ..warm }.reached(2, false, false, false));
        assert!(!Open { shared: 1, ..warm }.reached(3, false, false, false),
            "the core wants it mutual - two landings, not one");
        let deep = Open { shared: 2, ..warm };
        assert!(deep.reached(2, false, false, false));
    }

    /// answering their audition tells of yourself (E3-3): when the pair
    /// can hold it, the telling lands and the count rises
    #[test]
    fn an_answered_probe_counts_as_sharing() {
        let mut caller = Speaker::new("Hob");
        caller.traits.push(Trait::Stingy);
        caller.open = Open { intimacy: 0.6, benevolence: 0.5, integrity: 0.4,
            safety: 0.5, mood: 0.0, talks: 3, earshot: false, shared: 0 };
        let you = Speaker::new("Wanderer");
        let tr = converse_with(&caller, &you, Relation::Acquaintance, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                ch.iter().position(|c| matches!(c.act, Act::Agree | Act::Answer))
                    .or_else(|| ch.iter().position(|c| c.act != Act::Farewell)).or(Some(0))
            }, &mut Rng::new(7));
        assert!(tr.shared >= 1, "the told tale landed: {:?}", tr.lines());
        assert!(tr.turns.iter().any(|t| t.respond.is_some()),
            "the replies carry their tags: {:?}", tr.lines());
    }

    /// the same telling to a cold pair reads as strange (E3-3): a
    /// stranger has no warmth to hold it, so it overshoots
    #[test]
    fn a_telling_past_the_bond_overshoots() {
        let mut caller = Speaker::new("Hob");
        caller.traits.push(Trait::Stingy);
        caller.open = Open::default();
        let you = Speaker::new("Wanderer");
        let tr = converse_with(&caller, &you, Relation::Stranger, None, &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                ch.iter().position(|c| matches!(c.act, Act::Agree | Act::Answer))
                    .or_else(|| ch.iter().position(|c| c.act != Act::Farewell)).or(Some(0))
            }, &mut Rng::new(7));
        assert_eq!(tr.shared, 0, "nothing landed: {:?}", tr.lines());
        assert!(tr.overshot, "the stranger found it strange");
    }

    /// the fine bond rides the facts (#1372): a lover's pair facts carry
    /// `pair:kind:lover` beside the coarse `pair:relation:kin`, and a
    /// kind-less caller gets the coarse set alone
    #[test]
    fn the_finer_bond_rides_the_facts() {
        let a = Speaker::new("Hob");
        let b = Speaker::new("Nissa");
        let f = Facts::for_roles_kind(&a, &b, Relation::Kin, Some(RelKind::Lover));
        assert!(f.has("pair:kind:lover"));
        assert!(f.has("pair:relation:kin"), "the coarse fact stays for the old trees");
        assert!(!f.has("pair:kind:family"));
        let plain = Facts::for_roles_kind(&a, &b, Relation::Acquaintance, None);
        assert!(plain.has("pair:relation:acquaintance"));
        assert!(!plain.has("pair:kind:acquaintance"), "no kind, no kind fact");
    }

    /// a sweetheart opens on sweetheart material (#1372): the kind tree
    /// walks first for a lover pair and its words read as a lover's,
    /// not a friend's; the same pair kind-less never hears it
    #[test]
    fn a_sweetheart_speaks_as_a_sweetheart() {
        let a = Speaker::new("Mahelt");
        let b = Speaker::new("Terrice");
        let tr = converse(&a, &b, Relation::Kin, Some(RelKind::Lover),
            &village_library(), None, &mut Rng::new(11));
        assert_eq!(tr.trees_walked[0], "sweetheart_beat", "{:?}", tr.trees_walked);
        assert!(tr.turns.iter().any(|t| t.concepts.contains(&"bond:lover")),
            "the lover's beat landed: {:?}", tr.lines());
        let plain = converse(&a, &b, Relation::Kin, None,
            &village_library(), None, &mut Rng::new(11));
        assert!(!plain.trees_walked.contains(&"sweetheart_beat"),
            "kind-less pairs never hear it: {:?}", plain.trees_walked);
    }

    /// the same for the teacher and the learner (#1372): mentor and
    /// apprentice each get their register, not the acquaintance's
    #[test]
    fn a_mentor_speaks_as_a_mentor() {
        let a = Speaker::new("Old Pell");
        let b = Speaker::new("Young Nog");
        let tr = converse(&a, &b, Relation::Acquaintance, Some(RelKind::Mentor),
            &village_library(), None, &mut Rng::new(11));
        assert!(tr.trees_walked.contains(&"mentor_counsel"), "{:?}", tr.trees_walked);
        let tr = converse(&b, &a, Relation::Acquaintance, Some(RelKind::Apprentice),
            &village_library(), None, &mut Rng::new(11));
        assert!(tr.trees_walked.contains(&"apprentice_eager"), "{:?}", tr.trees_walked);
    }

    /// and blood reads as blood (#1372): family pairs get the kin beat
    /// that is theirs, distinct from a sweetheart's
    #[test]
    fn a_cousin_speaks_as_kin() {
        let a = Speaker::new("Hob");
        let b = Speaker::new("Nissa");
        let tr = converse(&a, &b, Relation::Kin, Some(RelKind::Family),
            &village_library(), None, &mut Rng::new(11));
        assert_eq!(tr.trees_walked[0], "blood_kin", "{:?}", tr.trees_walked);
    }

    /// the two transcripts (E5-9, #1446): the setting rides the facts -
    /// `pair:place:*` always, `pair:channel:*` picked by the place, the
    /// law's ear, and the bond's way in
    #[test]
    fn the_setting_rides_the_facts() {
        use crate::game::setting::{Place, Setting};
        let a = Speaker::new("Hob");
        let b = Speaker::new("Nissa");
        // the law's ear flips an invited backstage room public again
        let heard = Facts::for_roles_setting(&a, &b, Relation::Friend, None,
            &Setting { place: Place::Alehouse, law_earshot: true, invited: true });
        assert!(heard.has("pair:place:alehouse"));
        assert!(heard.has("pair:channel:public"), "the ear turns it public");
        assert!(heard.has("pair:earshot:law"));
        // the same room, no ear, the bond's bought the way in
        let hushed = Facts::for_roles_setting(&a, &b, Relation::Friend, None,
            &Setting { place: Place::Alehouse, law_earshot: false, invited: true });
        assert!(hushed.has("pair:channel:backstage"));
        // a stranger at the same table still hears the street voice
        let cold = Facts::for_roles_setting(&a, &b, Relation::Stranger, None,
            &Setting { place: Place::Alehouse, law_earshot: false, invited: false });
        assert!(cold.has("pair:channel:public"), "the invitation's the gate");
        // and the market's public whatever the bond
        let square = Facts::for_roles_setting(&a, &b, Relation::Kin, None,
            &Setting { place: Place::Market, law_earshot: false, invited: true });
        assert!(square.has("pair:place:market"));
        assert!(square.has("pair:channel:public"));
    }

    /// the bond itself is the invitation (E5-9, #1446): friends and kin
    /// are let backstage without the caller asking - the same alehouse
    /// keeps a stranger at the street voice
    #[test]
    fn the_bond_is_the_invitation() {
        use crate::game::setting::{Channel, Place, Setting};
        let a = Speaker::new("Hob");
        let b = Speaker::new("Nissa");
        let lib = village_library();
        let corner = Setting { place: Place::Alehouse, law_earshot: false, invited: false };
        let warm = converse_at(&a, &b, Relation::Friend, None, &lib, None,
            &corner, &mut Rng::new(3));
        assert_eq!(warm.setting.channel(), Channel::Backstage);
        assert!(warm.turns.iter().all(|t| t.channel == Channel::Backstage));
        let cold = converse_at(&a, &b, Relation::Stranger, None, &lib, None,
            &corner, &mut Rng::new(3));
        assert_eq!(cold.setting.channel(), Channel::Public,
            "a stranger gets the street voice even at the table");
    }

    /// the same pair, the same seed (E5-9, #1446): in the market the
    /// talk defers; invited backstage it bites - the hidden transcript
    /// reads differently and the turns carry which face each was said in
    #[test]
    fn the_place_picks_the_transcript() {
        use crate::game::setting::{Channel, Place, Setting};
        let a = Speaker::new("Hob");
        let b = Speaker::new("Nissa");
        let lib = village_library();
        let square = Setting { place: Place::Market, law_earshot: false, invited: false };
        // an acquaintance let backstage the other way - the safety
        // rung's Challenger, which the engine can't see from the rel
        let corner = Setting { place: Place::Alehouse, law_earshot: false, invited: true };
        let mut differ = 0;
        for seed in 1..10 {
            let front = converse_at(&a, &b, Relation::Acquaintance, None, &lib, None,
                &square, &mut Rng::new(seed));
            let back = converse_at(&a, &b, Relation::Acquaintance, None, &lib, None,
                &corner, &mut Rng::new(seed));
            assert_eq!(front.setting.channel(), Channel::Public);
            assert_eq!(back.setting.channel(), Channel::Backstage);
            assert!(front.turns.iter().all(|t| t.channel == Channel::Public));
            assert!(back.turns.iter().all(|t| t.channel == Channel::Backstage));
            if front.lines() != back.lines() { differ += 1; }
        }
        assert!(differ > 0, "the hidden transcript should read differently");
    }
}

#[cfg(test)]
mod e5_3 {
    use super::*;
    use crate::game::desires::{Problem, Reveal};
    use crate::game::dialogue_trees::village_library;

    /// Drive a player talk to its parting: take the worry menu's marked
    /// pick when the doorknob shows, the farewell once the talk's run
    /// dry, the first live reply otherwise
    fn drive_to_door(caller: &Speaker, you: &Speaker, pick: &str, seed: u64) -> Transcript {
        converse_with(caller, you, Relation::Acquaintance, None,
            &village_library(), None, 1,
            &mut |ch: &[Choice]| {
                ch.iter().position(|c| c.preview == pick)
                    .or_else(|| ch.iter().position(|c| c.act == Act::Farewell))
                    .or(Some(0))
            }, &mut Rng::new(seed))
    }

    /// the doorknob (E5-3): as the talk heads out, the caller's worry
    /// rides in ahead of the farewell line - one last question at the
    /// door
    #[test]
    fn a_worry_rides_in_ahead_of_the_goodbye() {
        let mut caller = Speaker::new("Hob");
        caller.worry = Some(Reveal::Problem(Problem::Taxed));
        let you = Speaker::new("Wanderer");
        let mut saw = false;
        for seed in 0..30 {
            let tr = drive_to_door(&caller, &you, "Answer plainly", seed);
            if let Some(wi) = tr.turns.iter().position(|t| t.act == Act::Worry) {
                saw = true;
                assert_eq!(tr.turns[wi].speaker, 0, "the caller's worry");
                assert_eq!(tr.turns[wi].tree, WORRY_TREE);
                let fi = tr.turns.iter().position(|t| t.act == Act::Farewell).unwrap();
                assert!(wi < fi, "the worry comes before the goodbye: {:?}", tr.lines());
            }
        }
        assert!(saw, "the doorknob never fired in 30 seeds");
    }

    /// the done-when (E5-3): asking why opens the tier whenever the
    /// pair's openness allows - the fear's told and lands for the
    /// session to hear
    #[test]
    fn asking_why_opens_the_tier_when_openness_allows() {
        let mut caller = Speaker::new("Hob");
        caller.worry = Some(Reveal::Problem(Problem::Taxed));
        caller.open = Open { intimacy: 0.7, benevolence: 0.6, integrity: 0.6,
            safety: 0.6, mood: 0.0, talks: 4, earshot: false, shared: 2 };
        let you = Speaker::new("Wanderer");
        let mut saw = false;
        for seed in 0..30 {
            let tr = drive_to_door(&caller, &you, "Ask why", seed);
            if tr.turns.iter().any(|t| t.act == Act::Worry) {
                saw = true;
                assert_eq!(tr.worry_told, Some(Reveal::Problem(Problem::Taxed)),
                    "{:?}", tr.lines());
                assert!(tr.evasions.is_empty(), "no turn-aside: {:?}", tr.lines());
                let why = tr.turns.iter().position(|t| t.line.map(|l| WORRY_WHY.contains(&l)).unwrap_or(false)).unwrap();
                assert!(tr.turns[why + 1..].iter().any(|t| t.speaker == 0
                    && t.act == Act::Answer && t.tree == WORRY_TREE),
                    "the truth came out: {:?}", tr.lines());
            }
        }
        assert!(saw, "the doorknob never fired in 30 seeds");
    }

    /// a door that stays shut is its own tell (E5-3 / E5-2): the why
    /// asked past the pair's openness gets turned aside, and the
    /// turn-aside counts on the ledger by the area it stood over
    #[test]
    fn a_shut_door_is_itself_a_tell() {
        let mut caller = Speaker::new("Hob");
        caller.worry = Some(Reveal::Problem(Problem::Taxed));
        caller.open = Open::default();
        let you = Speaker::new("Wanderer");
        let mut saw = false;
        for seed in 0..30 {
            let tr = drive_to_door(&caller, &you, "Ask why", seed);
            if tr.turns.iter().any(|t| t.act == Act::Worry) {
                saw = true;
                assert_eq!(tr.worry_told, None, "{:?}", tr.lines());
                assert!(tr.evasions.contains(&(0, Area::Trouble)),
                    "the evasion counts under its area: {:?}", tr.lines());
                assert!(tr.turns.iter().any(|t| t.act == Act::Evade
                    && t.tree == WORRY_TREE), "{:?}", tr.lines());
            }
        }
        assert!(saw, "the doorknob never fired in 30 seeds");
    }

    /// the plain answers (E5-3): answer plainly rides the talk neutral;
    /// reassure carries the care tag the chat's warmth reads
    #[test]
    fn an_answer_or_a_kind_word_closes_the_door() {
        let mut caller = Speaker::new("Hob");
        caller.worry = Some(Reveal::Want(crate::game::desires::Want::GoodFood));
        let you = Speaker::new("Wanderer");
        let (mut answered, mut soothed) = (false, false);
        for seed in 0..40 {
            let tr = drive_to_door(&caller, &you, "Answer plainly", seed);
            if tr.turns.iter().any(|t| t.act == Act::Worry) {
                answered = true;
                assert_eq!(tr.worry_told, None);
                assert!(tr.evasions.is_empty());
                assert!(tr.turns.iter().any(|t| t.speaker == 1
                    && t.act == Act::Answer && t.tree == WORRY_TREE), "{:?}", tr.lines());
            }
            let tr = drive_to_door(&caller, &you, "Reassure them", seed);
            if tr.turns.iter().any(|t| t.act == Act::Worry) {
                soothed = true;
                assert_eq!(tr.worry_told, None);
                assert!(tr.turns.iter().any(|t| t.speaker == 1
                    && t.act == Act::Agree && t.respond == Some(Respond::Care)),
                    "the kind word carries the warmth tag: {:?}", tr.lines());
            }
        }
        assert!(answered && soothed, "the doorknob never fired in 40 seeds");
    }

    /// nothing carried, nothing asked (E5-3): a caller with no worry
    /// keyed parts without a doorknob
    #[test]
    fn nothing_carried_no_doorknob() {
        let caller = Speaker::new("Hob");
        let you = Speaker::new("Wanderer");
        for seed in 0..20 {
            let tr = drive_to_door(&caller, &you, "Answer plainly", seed);
            assert!(tr.turns.iter().all(|t| t.act != Act::Worry), "{:?}", tr.lines());
            assert_eq!(tr.worry_told, None);
        }
    }
}

#[cfg(test)]
mod e5_8 {
    use super::*;
    use crate::game::dialogue_trees::village_library;

    fn slip_prone() -> Slip {
        Slip { prone: 1.0, dead: Some(Dead {
            kin: "brother", name: "Corvin".into(), trade: "carter" }) }
    }

    /// a leak the realizer produces (E5-8): a strong ghost under a
    /// high-strung temperament shows through the words - the line
    /// keeps its spent template, its text comes out corrupted, its
    /// act marks the slip
    #[test]
    fn a_strong_ghost_leaks_through_the_words() {
        let mut fired = 0;
        for seed in 0..60 {
            let mut caller = Speaker::new("Hob");
            caller.slip = Some(slip_prone());
            caller.slips_left = 1;
            let you = Speaker::new("Wanderer");
            let tr = converse(&caller, &you, Relation::Acquaintance, None,
                &village_library(), None, &mut Rng::new(seed));
            let slips: Vec<&Turn> = tr.turns.iter().filter(|t| t.act == Act::Slip).collect();
            assert!(slips.len() <= 1, "one leak a talk: {:?}", tr.lines());
            if let Some(&t) = slips.first() {
                fired += 1;
                assert_eq!(tr.slips.len(), 1, "the slip's on the record");
                assert!(t.line.is_some(), "the spent template's still marked");
                // the turn that slips is the ghost-carrier's own
                assert_eq!(t.speaker, 0);
            }
        }
        assert!(fired > 0, "no slip in 60 talks at full prone");
    }

    /// the budget is the caller's to enforce (E5-8): a speaker with no
    /// slip left in the arc never leaks, however prone
    #[test]
    fn a_spent_budget_never_slips() {
        for seed in 0..40 {
            let mut caller = Speaker::new("Hob");
            caller.slip = Some(slip_prone());
            caller.slips_left = 0;
            let you = Speaker::new("Wanderer");
            let tr = converse(&caller, &you, Relation::Acquaintance, None,
                &village_library(), None, &mut Rng::new(seed));
            assert!(tr.turns.iter().all(|t| t.act != Act::Slip), "{:?}", tr.lines());
            assert!(tr.slips.is_empty());
        }
    }

    /// no ghost under them, no leak (E5-8): a speaker carrying no
    /// slip profile says every word as written
    #[test]
    fn a_quiet_mind_says_the_line_as_written() {
        for seed in 0..30 {
            let mut caller = Speaker::new("Hob");
            caller.slips_left = 1;
            let you = Speaker::new("Wanderer");
            let tr = converse(&caller, &you, Relation::Acquaintance, None,
                &village_library(), None, &mut Rng::new(seed));
            assert!(tr.turns.iter().all(|t| t.act != Act::Slip), "{:?}", tr.lines());
        }
    }

    /// the three shapes (E5-8): the dead name stands in for the
    /// listener's own, the tense corrects itself mid-word, the public
    /// line cuts off where ears are near
    #[test]
    fn the_leak_takes_three_shapes() {
        let dead = Some(Dead { kin: "brother", name: "Corvin".into(), trade: "carter" });
        let mk = |text: &str, channel| Turn {
            speaker: 0, act: Act::Remark, respond: None, text: text.to_string(),
            concepts: vec![], tree: "t", line: Some("l"),
            channel };

        // the name slip: the listener called by the dead man's name
        let mut rng = Rng::new(0);
        let mut saw = false;
        for _ in 0..20 {
            let mut x = mk("Fair day, Tilda - the rye looks well.",
                crate::game::setting::Channel::Backstage);
            if slip_corrupt(&mut x, "Tilda", &dead, &mut rng) == Some(SlipKind::Name) {
                assert!(x.text.contains("Corvin") && !x.text.contains("Tilda"),
                    "the dead name stands in: {}", x.text);
                saw = true;
                break;
            }
        }
        assert!(saw, "the name slip never landed");

        // the tense slip: the kept thing surfaces self-corrected
        let mut rng = Rng::new(1);
        let mut saw = false;
        for _ in 0..20 {
            let mut x = mk("The rye comes in thin this year.",
                crate::game::setting::Channel::Backstage);
            if slip_corrupt(&mut x, "Tilda", &dead, &mut rng) == Some(SlipKind::Tense) {
                assert!(x.text.contains("is... was") && x.text.contains("Corvin"),
                    "the tense corrects mid-word: {}", x.text);
                saw = true;
                break;
            }
        }
        assert!(saw, "the tense slip never landed");

        // the cut-off: a public line truncates where ears are near
        let mut rng = Rng::new(2);
        let long = "The reeve's men were counting the sheepfolds again when I passed the lane.";
        let mut saw = false;
        for _ in 0..30 {
            let mut x = mk(long, crate::game::setting::Channel::Public);
            if slip_corrupt(&mut x, "Tilda", &dead, &mut rng) == Some(SlipKind::CutOff) {
                assert!(x.text.ends_with(" -") && x.text.len() < long.len(),
                    "the sentence dies halfway: {}", x.text);
                saw = true;
                break;
            }
        }
        assert!(saw, "the cut-off never landed");

        // backstage keeps the whole sentence - no cut-off there
        for _ in 0..30 {
            let mut y = mk(long, crate::game::setting::Channel::Backstage);
            let k = slip_corrupt(&mut y, "Tilda", &dead, &mut rng);
            assert_ne!(k, Some(SlipKind::CutOff), "no ears backstage");
        }
    }

    /// the persona gates the profile (E5-8): only a hard-held wound
    /// under a high-strung temperament carries a slip - and only a
    /// death ghost names a dead
    #[test]
    fn only_the_marked_and_high_strung_leak() {
        use crate::game::persona::{EventId, EventKind, Ghost, Belief, Fear, Need};
        let ghost = |kind, strength| Ghost {
            event: EventId(0), kind, hook: None,
            lie: Belief::LordAlwaysWins, strength,
            fear: Fear::BellTolls, need: Need::ToBeSeen, turned: false };
        let persona = |ghosts: Vec<Ghost>, n: f32| {
            crate::game::persona::Persona {
                seed: 7, sex: crate::game::villager::Sex::Male,
                archetype: crate::game::persona::Archetype::Burned,
                temperament: [0.5, 0.5, 0.5, 0.5, n],
                trust_propensity: 0.5, values: [0.5; 10],
                story: vec![], ghosts,
                signature: crate::game::persona::Signature::Habit("x"),
                idiolect: crate::game::idiolect::Idiolect { phrase: "x",
                    habit: crate::game::idiolect::Habit::OathSaints },
                hidden: crate::game::persona::Landmark::Skill("x"),
                contradiction: None,
            }
        };
        // strong death ghost + high N: profile with a dead
        let p = persona(vec![ghost(EventKind::Fever, 0.8)], 0.8);
        let s = slip_of(&p, None).expect("a marked, high-strung soul carries a slip");
        assert!(s.dead.is_some(), "a death ghost names a dead");
        assert!(s.prone > 0.5);
        // strong ghost, calm nerves: no leak
        let p = persona(vec![ghost(EventKind::Fever, 0.8)], 0.2);
        assert!(slip_of(&p, None).is_none(), "calm nerves don't slip");
        // high N, faint wound: no leak
        let p = persona(vec![ghost(EventKind::Fever, 0.2)], 0.9);
        assert!(slip_of(&p, None).is_none(), "a faint wound doesn't slip");
        // a non-death wound: prone but no dead to name
        let p = persona(vec![ghost(EventKind::Eviction, 0.8)], 0.8);
        let s = slip_of(&p, None).expect("a strong wound still slips");
        assert!(s.dead.is_none(), "an eviction keeps no dead");
        // a real dead name beats the invented pool
        let p = persona(vec![ghost(EventKind::Fever, 0.8)], 0.8);
        let s = slip_of(&p, Some("Old Nan".into())).unwrap();
        assert_eq!(s.dead.unwrap().name, "Old Nan");
    }
}

#[cfg(test)]
mod e5_6 {
    use super::*;
    use crate::game::dialogue_trees::village_library;
    use crate::game::persona::{Belief, EventId, EventKind, Fear, Ghost, Need};

    fn persona(ghosts: Vec<Ghost>, traits: &[Trait]) -> crate::game::persona::Persona {
        let _ = traits;
        crate::game::persona::Persona {
            seed: 7, sex: crate::game::villager::Sex::Male,
            archetype: crate::game::persona::Archetype::Burned,
            temperament: [0.5; 5],
            trust_propensity: 0.5, values: [0.5; 10],
            story: vec![], ghosts,
            signature: crate::game::persona::Signature::Habit("x"),
            idiolect: crate::game::idiolect::Idiolect { phrase: "x",
                habit: crate::game::idiolect::Habit::OathSaints },
            hidden: crate::game::persona::Landmark::Skill("x"),
            contradiction: None,
        }
    }

    fn ghost(lie: Belief, strength: f32) -> Ghost {
        Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
            lie, strength, fear: Fear::BellTolls, need: Need::ToBeSeen, turned: false }
    }

    fn funny(traits: &[Trait], lie: Belief) -> Speaker {
        let mut s = Speaker::new("Hob");
        s.traits = traits.to_vec();
        s.joke = Some(joke_of(&persona(vec![ghost(lie, 0.8)], traits), &s.traits));
        s
    }

    /// the done-when (E5-6): a barb at power waits for the hidden
    /// transcript - in a public channel the draw can't strike the
    /// lord however hot the head
    #[test]
    fn a_barb_at_power_waits_for_the_hidden_transcript() {
        let mut caller = funny(&[Trait::Hotheaded, Trait::Gregarious], Belief::LordAlwaysWins);
        // a fixed draw so the barb always wins the style roll - the
        // gate is the only thing that can stop it
        caller.joke.as_mut().unwrap().styles = [0.05, 0.05, 1.0, 0.05];
        for seed in 0..80 {
            let you = Speaker::new("Wenna");
            let tr = converse_at(&caller, &you, Relation::Acquaintance, None,
                &village_library(), None, &crate::game::setting::STREET,
                &mut Rng::new(seed));
            for b in &tr.jokes {
                assert!(!(b.style == Some(JokeStyle::Aggressive)
                    && b.target.is_some_and(JokeTarget::power)),
                    "a barb at power in the street: {:?}", tr.lines());
            }
        }
        // backstage it can land
        let mut struck = false;
        for seed in 0..80 {
            let you = Speaker::new("Wenna");
            let tr = converse_at(&caller, &you, Relation::Friend, None,
                &village_library(), None, &crate::game::setting::BACKSTAGE,
                &mut Rng::new(seed));
            if tr.jokes.iter().any(|b| b.style == Some(JokeStyle::Aggressive)
                && b.target == Some(JokeTarget::Lord)) { struck = true; }
        }
        assert!(struck, "the barb never landed backstage in 80 talks");
    }

    /// the hearer's fork (E5-6): every joke draws its receipt, and the
    /// transcript keeps who told what and how it landed
    #[test]
    fn a_joke_gets_its_receipt() {
        let mut kinds_seen: Vec<JokeReceipt> = vec![];
        for seed in 0..120 {
            let caller = funny(&[Trait::Gregarious], Belief::LordAlwaysWins);
            let you = Speaker::new("Wenna");
            let tr = converse(&caller, &you, Relation::Friend, None,
                &village_library(), None, &mut Rng::new(seed));
            for b in tr.jokes.iter().filter(|b| b.joker == 0) {
                assert!(b.style.is_some(), "the beat's jokes know their cell");
                if !kinds_seen.contains(&b.receipt) { kinds_seen.push(b.receipt); }
                // the turn after the joke is the receipt's
                let at = tr.turns.iter().position(|t|
                    t.act == Act::Joke && t.speaker == 0).unwrap();
                let next = &tr.turns[at + 1];
                assert_eq!(next.speaker, 1, "the hearer receipts");
            }
        }
        assert!(kinds_seen.len() >= 2, "receipts should vary: {kinds_seen:?}");
    }

    /// who laughs shows alignment (E5-6): a hearer carrying the same
    /// lie laughs at the barb where a stranger lets it pass
    #[test]
    fn the_shared_lie_laughs() {
        let mut aligned_laughs = 0;
        let mut plain_laughs = 0;
        for seed in 0..160 {
            let mut caller = funny(&[Trait::Hotheaded], Belief::LordAlwaysWins);
            caller.joke.as_mut().unwrap().styles = [0.0, 0.0, 1.0, 0.0];
            let mut mate = funny(&[], Belief::LordAlwaysWins);
            let mut other = funny(&[], Belief::MillOwnsUs);
            // backstage, so the barb can strike at all
            for (you, tally) in [(&mut mate, &mut aligned_laughs), (&mut other, &mut plain_laughs)] {
                let tr = converse_at(&caller, you, Relation::Friend, None,
                    &village_library(), None, &crate::game::setting::BACKSTAGE,
                    &mut Rng::new(seed));
                *tally += tr.jokes.iter().filter(|b| matches!(b.receipt,
                    JokeReceipt::Laughed | JokeReceipt::AddedTo)).count();
            }
        }
        assert!(aligned_laughs > plain_laughs,
            "the shared lie should laugh more: {aligned_laughs} vs {plain_laughs}");
    }

    /// the wound worn light (E5-6): a strong self-worth lie sharpens
    /// the self-defeating edge - the humor turns inward
    #[test]
    fn a_self_worth_lie_turns_the_humor_inward() {
        let j = joke_of(&persona(vec![ghost(Belief::KeepHidden, 0.9)], &[]), &[]);
        assert!(j.styles[3] > 1.0, "the self-defeating edge: {:?}", j.styles);
        assert_eq!(j.grievance.map(|g| g.target), Some(JokeTarget::Oneself),
            "the wound's barbs aim at the self");
        // and the talk shows it
        let mut inward = 0;
        let mut caller = Speaker::new("Hob");
        caller.joke = Some(j);
        for seed in 0..80 {
            let you = Speaker::new("Wenna");
            let tr = converse(&caller, &you, Relation::Friend, None,
                &village_library(), None, &mut Rng::new(seed));
            inward += tr.jokes.iter().filter(|b|
                b.style == Some(JokeStyle::SelfDefeating)).count();
        }
        assert!(inward > 0, "the self-worth lie never showed in 80 talks");
    }

    /// the player's fork (E5-6): a joke at them offers the four
    /// receipts as a menu, and the pick lands as the hearer's line
    #[test]
    fn the_player_gets_the_four_forks() {
        let caller = funny(&[Trait::Gregarious], Belief::LordAlwaysWins);
        let mut menus = 0;
        let mut landed = false;
        for seed in 0..120 {
            let you = Speaker::new("Wanderer");
            let mut forks_seen = 0;
            let tr = converse_with(&caller, &you, Relation::Friend, None,
                &village_library(), None, 1, &mut |ch: &[Choice]| {
                    if ch.len() == 4 && ch[0].act == Act::Agree
                        && ch[1].act == Act::Joke {
                        forks_seen += 1;
                        return Some(0);
                    }
                    Some(0)
                }, &mut Rng::new(seed));
            menus += forks_seen;
            landed |= tr.jokes.iter().any(|b| b.receipt == JokeReceipt::Laughed);
        }
        assert!(menus > 0, "the four-fork menu never showed in 120 talks");
        assert!(landed, "the laugh never landed");
    }
}
