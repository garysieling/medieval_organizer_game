//! Persona (#1406, E1-1; design `docs/plans/procedural-characters.md`
//! part 1): the deep model under a villager - seeded once, changed only
//! by recorded events. Six passes build it; the prehistory and household
//! passes land in #1419/#1420 and the ghost pass in #1421, so `generate`
//! takes what they'll hand down and the call itself doesn't churn.

use crate::game::villager::Sex;
use crate::game::vocation::Vocation;

/// A prehistory event's record (pass 1, #1419): an index into the
/// event log the thirty years before play writes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub u32);

/// The record-sources the thirty years are written in (#1419). The
/// kinds live beside the persona because the ghost pass keys on them
/// (E2-3, #1421); `session::prehistory` re-exports this
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    Fire,
    Levy,
    DeathDuty,
    Fever,
    MillerDebt,
    BrokenBetrothal,
    OldFaith,
    Eviction,
    HarvestFeast,
}

impl EventKind {
    /// The event's slug for the fact table ("ghost:levy")
    pub fn tag(self) -> &'static str {
        match self {
            EventKind::Fire => "fire",
            EventKind::Levy => "levy",
            EventKind::DeathDuty => "death_duty",
            EventKind::Fever => "fever",
            EventKind::MillerDebt => "miller_debt",
            EventKind::BrokenBetrothal => "broken_betrothal",
            EventKind::OldFaith => "old_faith",
            EventKind::Eviction => "eviction",
            EventKind::HarvestFeast => "harvest_feast",
        }
    }
}

/// The archetypes are the biomes: a region of temperament and value
/// space with a voice register, an audition question and a tendency
/// in a campaign
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    OldGuard,         // Custom and Safety high, openness low
    Striver,          // Advancement high, conscientious
    SoftHeart,        // Care high, agreeable
    Firebrand,        // Fairness high, low agreeableness or high extraversion
    QuietHand,        // Care and Propriety, conscientious, introverted
    TongueOfTheWell,  // extraverted, Adventure
    Burned,           // neurotic, a deep ghost, deferent
    Doubter,          // Freedom high, low trust propensity
    Devout,           // Custom high, devout
    Merrymaker,       // Pleasure high, extraverted
    Climber,          // Standing high, Care low
}

/// All of them, in a fixed order
pub const ARCHETYPES: [Archetype; 11] = [
    Archetype::OldGuard, Archetype::Striver, Archetype::SoftHeart,
    Archetype::Firebrand, Archetype::QuietHand, Archetype::TongueOfTheWell,
    Archetype::Burned, Archetype::Doubter, Archetype::Devout,
    Archetype::Merrymaker, Archetype::Climber,
];

/// Temperament axes, in the order `temperament` stores them
pub const AXES: [&str; 5] = ["openness", "conscientiousness", "extraversion",
    "agreeableness", "neuroticism"];

/// The ten values on the Schwartz circle, under their village names -
/// neighbors ally, opposites sit five seats round
pub const VALUES: [&str; 10] = ["Freedom", "Adventure", "Pleasure", "Advancement",
    "Standing", "Safety", "Propriety", "Custom", "Care", "Fairness"];

impl Archetype {
    /// The biome's slug for the fact table ("archetype:old_guard")
    pub fn tag(self) -> &'static str {
        match self {
            Archetype::OldGuard => "old_guard",
            Archetype::Striver => "striver",
            Archetype::SoftHeart => "soft_heart",
            Archetype::Firebrand => "firebrand",
            Archetype::QuietHand => "quiet_hand",
            Archetype::TongueOfTheWell => "tongue_of_the_well",
            Archetype::Burned => "burned",
            Archetype::Doubter => "doubter",
            Archetype::Devout => "devout",
            Archetype::Merrymaker => "merrymaker",
            Archetype::Climber => "climber",
        }
    }

    /// What the village would call the sort
    pub fn name(self) -> &'static str {
        match self {
            Archetype::OldGuard => "the old guard",
            Archetype::Striver => "a striver",
            Archetype::SoftHeart => "a soft heart",
            Archetype::Firebrand => "a firebrand",
            Archetype::QuietHand => "a quiet hand",
            Archetype::TongueOfTheWell => "a tongue of the well",
            Archetype::Burned => "one of the burned",
            Archetype::Doubter => "a doubter",
            Archetype::Devout => "a devout one",
            Archetype::Merrymaker => "a merrymaker",
            Archetype::Climber => "a climber",
        }
    }

    /// How they sound before you know them
    pub fn voice(self) -> &'static str {
        match self {
            Archetype::OldGuard => "proverbs; as my father did",
            Archetype::Striver => "mastery and coin",
            Archetype::SoftHeart => "asks after others first",
            Archetype::Firebrand => "names wrongs, and names names",
            Archetype::QuietHand => "few words, practical",
            Archetype::TongueOfTheWell => "news and other people's business",
            Archetype::Burned => "hedges; it's not my place",
            Archetype::Doubter => "questions, dry wit",
            Archetype::Devout => "scripture and saints",
            Archetype::Merrymaker => "jokes, songs, ale",
            Archetype::Climber => "flatters upward",
        }
    }

    /// The question they're really asking you
    pub fn audition(self) -> &'static str {
        match self {
            Archetype::OldGuard => "Do you respect how things are done?",
            Archetype::Striver => "Can you get things done?",
            Archetype::SoftHeart => "Do you care, or want something?",
            Archetype::Firebrand => "Will you stand your ground?",
            Archetype::QuietHand => "Will you do the work?",
            Archetype::TongueOfTheWell => "What do you know?",
            Archetype::Burned => "Are you safe to be seen with?",
            Archetype::Doubter => "Are you who you say?",
            Archetype::Devout => "Do you keep faith?",
            Archetype::Merrymaker => "Are you good company?",
            Archetype::Climber => "What can you do for me?",
        }
    }

    /// (temperament pull per axis, the value their circle turns to,
    /// trust-propensity pull) - the biome's pull on passes 3 and 4
    fn prior(self) -> ([f32; 5], usize, f32) {
        // axes: O, C, E, A, N - pulls land on the seeded draw, 0.7 draw
        // to 0.3 pull so the region leans without flattening
        match self {
            Archetype::OldGuard => ([-0.3, 0.1, 0.0, 0.0, 0.0], 7, 0.0),   // Custom
            Archetype::Striver => ([0.0, 0.3, 0.0, 0.0, 0.0], 3, 0.0),    // Advancement
            Archetype::SoftHeart => ([0.0, 0.0, 0.0, 0.3, 0.0], 8, 0.1),  // Care
            Archetype::Firebrand => ([0.1, 0.0, 0.2, -0.2, 0.1], 9, 0.0), // Fairness
            Archetype::QuietHand => ([0.0, 0.3, -0.3, 0.2, 0.0], 6, 0.0), // Propriety
            Archetype::TongueOfTheWell => ([0.1, 0.0, 0.3, 0.0, 0.0], 1, 0.1), // Adventure
            Archetype::Burned => ([0.0, 0.0, -0.2, -0.1, 0.3], 5, -0.2),  // Safety
            Archetype::Doubter => ([0.2, 0.0, 0.0, -0.1, 0.1], 0, -0.3),  // Freedom
            Archetype::Devout => ([-0.1, 0.2, 0.0, 0.1, -0.1], 7, 0.1),   // Custom
            Archetype::Merrymaker => ([0.1, -0.1, 0.3, 0.1, -0.1], 2, 0.1), // Pleasure
            Archetype::Climber => ([0.0, 0.1, 0.1, -0.2, 0.0], 4, -0.1),  // Standing
        }
    }
}

/// A belief a ghost teaches - a lie, mostly, though not every lesson is
/// one: `StrengthOfUs` and `TruthComesOut` are the true ones
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Belief {
    LordAlwaysWins,    // "the lord's men always win"
    SomeoneToBlame,    // "the village always finds someone to blame"
    WarsTakeSons,      // "the lord's wars take our sons"
    LordTakesFromDead, // "the lord takes even from the dead"
    GodSends,          // "God sends what God sends"
    MillOwnsUs,        // "the mill owns us"
    PromisesForKeepers,// "promises are for those who can keep them"
    KeepHidden,        // "keep your true self hidden"
    NeverCrossReeve,   // "never cross the reeve"
    StrongTogether,    // "we were strong together once"
    TruthComesOut,     // "the truth comes out if someone says it"
}

/// All beliefs, in a fixed order (cast check groups wounds by index)
pub const BELIEFS: [Belief; 11] = [
    Belief::LordAlwaysWins, Belief::SomeoneToBlame, Belief::WarsTakeSons,
    Belief::LordTakesFromDead, Belief::GodSends, Belief::MillOwnsUs,
    Belief::PromisesForKeepers, Belief::KeepHidden, Belief::NeverCrossReeve,
    Belief::StrongTogether, Belief::TruthComesOut,
];

impl Belief {
    /// The wound's slug for the fact table ("lie:lord_always_wins")
    pub fn tag(self) -> &'static str {
        match self {
            Belief::LordAlwaysWins => "lord_always_wins",
            Belief::SomeoneToBlame => "someone_to_blame",
            Belief::WarsTakeSons => "wars_take_sons",
            Belief::LordTakesFromDead => "lord_takes_from_dead",
            Belief::GodSends => "god_sends",
            Belief::MillOwnsUs => "mill_owns_us",
            Belief::PromisesForKeepers => "promises_for_keepers",
            Belief::KeepHidden => "keep_hidden",
            Belief::NeverCrossReeve => "never_cross_reeve",
            Belief::StrongTogether => "strong_together",
            Belief::TruthComesOut => "truth_comes_out",
        }
    }

    /// The lesson as they'd say it
    pub fn text(self) -> &'static str {
        match self {
            Belief::LordAlwaysWins => "the lord's men always win",
            Belief::SomeoneToBlame => "the village always finds someone to blame",
            Belief::WarsTakeSons => "the lord's wars take our sons",
            Belief::LordTakesFromDead => "the lord takes even from the dead",
            Belief::GodSends => "God sends what God sends",
            Belief::MillOwnsUs => "the mill owns us",
            Belief::PromisesForKeepers => "promises are for those who can keep them",
            Belief::KeepHidden => "keep your true self hidden",
            Belief::NeverCrossReeve => "never cross the reeve",
            Belief::StrongTogether => "we were strong together once",
            Belief::TruthComesOut => "the truth comes out if someone says it",
        }
    }
}

/// What the ghost leaves them watching for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fear {
    BailiffKnowsMyFace, // "the bailiff knows my face"
    FireReturns,        // "the thatch will go again, and this time us"
    BuriedLikeHim,      // "a ditch in a field, like the boy they took"
    WhispersTurn,       // "the whisper comes for me next"
    CollectorAtTheDoor, // "the miller's man at the door"
    AloneInIt,          // "no one to stand beside me when it counts"
    BellTolls,          // "the bell tolling for one more of mine"
    StonesRemember,     // "the stones remember what I swore off"
}

/// What the campaign can give - the need under the wound
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Need {
    ToBelong,     // a bench to sit on with the rest
    ToBeSafe,     // a door that holds
    ToSetItRight, // the wrong named, and made good
    ToBeBelieved, // one soul who takes their word
    ToProvide,    // winter stores, whatever it takes
    ToLayItDown,  // leave to stop carrying it
    ToBeSeen,     // for someone to finally see them
}

/// A deep mark a prehistory event left: wound, lie, fear and need.
/// `hook` indexes `hooks::HOOKS` where the wound carries a talk status
/// (#1421) - events without a HOOKS row keep the ghost as a record only
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ghost {
    /// The event that left it
    pub event: EventId,
    /// The event's kind - the fact table names the wound by it (#1423)
    pub kind: EventKind,
    /// Index into `hooks::HOOKS`, when one carries it
    pub hook: Option<usize>,
    /// The belief it taught, with its strength: campaign outcomes move it
    pub lie: Belief,
    /// How hard they hold it, 0 to 1
    pub strength: f32,
    /// The lie's been disproved enough to let go (E8-2, #1444): set by
    /// `beliefs::tested_by` crossing TURN_FLOOR; tells and showing stops
    /// counting it
    pub turned: bool,
    /// What it leaves them watching for
    pub fear: Fear,
    /// What the campaign can give
    pub need: Need,
}

/// The visible landmark - how you'd pick them out of a crowd. No two
/// villagers share one; the cast pass (#1413) enforces it
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signature {
    Thing(&'static str),
    Habit(&'static str),
    Phrase(&'static str),
    Place(&'static str),
}

impl Signature {
    /// The landmark as a phrase
    pub fn text(self) -> &'static str {
        match self {
            Signature::Thing(t) | Signature::Habit(t)
            | Signature::Phrase(t) | Signature::Place(t) => t,
        }
    }
}

/// The signatures in the village's wardrobe - wide enough that no two
/// villagers need share one
pub const SIGNATURES: &[Signature] = &[
    Signature::Thing("carries her grandmother's distaff everywhere"),
    Signature::Thing("a whittled hawk hung on a cord at his belt"),
    Signature::Thing("wears a key she swears fits nothing"),
    Signature::Thing("keeps a smooth river-stone in his sleeve"),
    Signature::Thing("a green ribbon at her wrist, frayed past mending"),
    Signature::Thing("carries a split hazel wand for finding water"),
    Signature::Thing("an old soldier's brass at his cap"),
    Signature::Thing("her mother's prayer-beads, worn smooth"),
    Signature::Habit("plaits straw while she listens"),
    Signature::Habit("counts the church bells under his breath"),
    Signature::Habit("touches the lintel twice going in"),
    Signature::Habit("spits over the bridge rail, every crossing"),
    Signature::Habit("ties and reties her apron strings"),
    Signature::Habit("whittles a spoon, never finishes one"),
    Signature::Habit("walks the long way past the church"),
    Signature::Habit("counts his change twice, then again"),
    Signature::Habit("salutes the magpies, one for sorrow"),
    Signature::Habit("stacks her tools by size before she sits"),
    Signature::Phrase("says 'as the mill grinds' to end an argument"),
    Signature::Phrase("greets the dog before its master"),
    Signature::Phrase("calls everyone 'my dear', high or low"),
    Signature::Phrase("ends every telling with 'so it goes'"),
    Signature::Phrase("says 'God send it' to anything promised"),
    Signature::Phrase("asks 'and what of it?' when folk complain"),
    Signature::Phrase("sings the last word of every greeting"),
    Signature::Phrase("answers a question with a proverb"),
    Signature::Place("always by the bakehouse wall at noon"),
    Signature::Place("sits the third bench, never the second"),
    Signature::Place("stands under the ash tree to talk"),
    Signature::Place("first to the well, last to leave it"),
    Signature::Place("keeps the same stall corner on market day"),
    Signature::Place("leans on the churchyard gate of an evening"),
    Signature::Place("walks the east ditch road, never the west"),
    Signature::Place("waits at the foot of the mill stair"),
    Signature::Thing("wears a mourning veil to every feast"),
    Signature::Thing("a carved spoon in his hatband"),
    Signature::Habit("will not sit with her back to a door"),
    Signature::Habit("blesses every loaf she hands over"),
    Signature::Phrase("says 'mind the stile' as a farewell"),
    Signature::Place("first through the church door on holy days"),
];

/// The hidden landmark - a secret passion, a skill or a past; the
/// organic leaders hide here as often as not
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Landmark {
    Passion(&'static str),
    Skill(&'static str),
    Past(&'static str),
}

impl Landmark {
    /// The landmark as a phrase
    pub fn text(self) -> &'static str {
        match self {
            Landmark::Passion(t) | Landmark::Skill(t) | Landmark::Past(t) => t,
        }
    }
}

pub const HIDDENS: &[Landmark] = &[
    Landmark::Passion("copies birdsong on a bone whistle"),
    Landmark::Passion("can taste rain an hour before it falls"),
    Landmark::Passion("keeps a hand-copied page of psalms"),
    Landmark::Passion("carves saints' faces in the rafters where none look"),
    Landmark::Passion("knows every old song the reeve hates"),
    Landmark::Passion("names every star that rises over the mill"),
    Landmark::Skill("can mend a harp string"),
    Landmark::Skill("reads a bee swarm's temper"),
    Landmark::Skill("splints a bone straight as a barber's"),
    Landmark::Skill("writes a fair hand, learned of a monk"),
    Landmark::Skill("calls the eels into a basket"),
    Landmark::Skill("finds lost things - a knack, not a trade"),
    Landmark::Past("once served at the bishop's table"),
    Landmark::Past("buried two husbands before the third"),
    Landmark::Past("walked the pilgrimage road and came back changed"),
    Landmark::Past("stood in the levy line and drew the long straw"),
    Landmark::Past("was fostered in the next valley over"),
    Landmark::Past("ran the market at Marlow before the debts"),
    Landmark::Past("learned letters keeping accounts for a woolman"),
    Landmark::Past("hid a priest in the thatch one winter"),
    Landmark::Past("is the one folk quietly bring their quarrels to"),
    Landmark::Past("took the pilgrimage scrip and never went"),
    Landmark::Passion("teaches the mason's boy his letters by lamplight"),
    Landmark::Skill("delivers lambs nobody else's hands can turn"),
];

/// A deliberate crack in the temperament: the thresholds collide and
/// the person is tagged with a contradiction that gets lines of its own
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Contradiction {
    /// Lazy plus Ambitious: low conscientiousness against a high
    /// Advancement value - `idle_striver` in the tags
    IdleStriver,
}

impl Contradiction {
    /// The tag the model carries it under
    pub fn tag(self) -> &'static str {
        match self {
            Contradiction::IdleStriver => "idle_striver",
        }
    }
}

/// The deep model, seeded once and changed only by recorded events
#[derive(Debug, Clone, PartialEq)]
pub struct Persona {
    /// hash(world_seed, slot)
    pub seed: u64,
    /// The body the persona draws (E1-3, #1417) - names and pronouns
    /// agree with it by construction
    pub sex: Sex,
    /// The biome: voice register and priors
    pub archetype: Archetype,
    /// O, C, E, A, N as in `sim::psychology`
    pub temperament: [f32; 5],
    /// Childhood events; Wary derives from it
    pub trust_propensity: f32,
    /// Schwartz circle, village names (`VALUES`)
    pub values: [f32; 10],
    /// Prehistory events this person was in
    pub story: Vec<EventId>,
    /// Wound, lie, fear and need (pass 5, #1421)
    pub ghosts: Vec<Ghost>,
    /// Visible landmark, unique in the village
    pub signature: Signature,
    /// How they sound (E4-7, #1416): a pet phrase and a speech habit,
    /// each unique in the village - the cast deals them without
    /// replacement like the signatures
    pub idiolect: crate::game::idiolect::Idiolect,
    /// Secret passion, skill or past
    pub hidden: Landmark,
    /// A deliberate crack, when the thresholds collide
    pub contradiction: Option<Contradiction>,
}

impl Persona {
    /// The n-th draw off this person's own stream, as a 0..1 fraction -
    /// names, sex and every later choice read the same seed (#1411)
    pub fn draw(&self, n: u64) -> f32 { draw(self.seed, n) }

    /// The two values that move them most - the start of a
    /// self-interest profile
    pub fn top_values(&self) -> (usize, usize) {
        let mut top = (0usize, 1usize);
        for i in 1..self.values.len() {
            if self.values[i] > self.values[top.0] { top = (i, top.0); }
            else if self.values[i] > self.values[top.1] { top = (top.0, i); }
        }
        top
    }

    /// The tier each fact-area reaches (E3-1, #1423): the surface at 1,
    /// kin/faith and the told past at 2, the wound at 3 - #1424's
    /// openness reads these to know how deep a topic can go
    pub fn tiers(&self) -> Vec<(&'static str, u8)> {
        let mut v = vec![("self", 1u8), ("work", 1), ("village", 1),
            ("kin", 2), ("faith", 2)];
        if !self.story.is_empty() { v.push(("past", 2)); }
        if !self.ghosts.is_empty() { v.push(("wound", 3)); }
        v
    }

    /// The persona's shadow on the conversation's fact table (E3-1,
    /// #1423): archetype and the top two values are voice-observable
    /// and go under every prefix; the wound, its lie and the tiers
    /// stay self-only until disclosure opens them
    pub fn fact_table(&self) -> crate::game::conversation::FactTable {
        let (a, b) = self.top_values();
        let open = vec![
            format!("archetype:{}", self.archetype.tag()),
            format!("value:{}", VALUES[a].to_lowercase()),
            format!("value:{}", VALUES[b].to_lowercase()),
        ];
        let mut hidden: Vec<String> = self.ghosts.iter()
            .flat_map(|g| [format!("ghost:{}", g.kind.tag()),
                format!("lie:{}", g.lie.tag())])
            .collect();
        for (area, t) in self.tiers() {
            hidden.push(format!("tier:{area}={t}"));
        }
        crate::game::conversation::FactTable { open, hidden }
    }

    /// The tell slugs the ghosts and lies leave on them (E3-4, #1425) -
    /// the visible habits: one per wound, one per lesson
    pub fn tells(&self) -> Vec<&'static str> {
        crate::game::fact_content::tells_of(self)
    }

    /// The two values that move them least - the "low" legs of the
    /// threshold table rank the way "high" does
    pub fn bottom_values(&self) -> (usize, usize) {
        let mut bot = (0usize, 1usize);
        for i in 1..self.values.len() {
            if self.values[i] < self.values[bot.0] { bot = (i, bot.0); }
            else if self.values[i] < self.values[bot.1] { bot = (bot.0, i); }
        }
        bot
    }

    /// E1-4 (#1412): both personality models as views of the persona -
    /// the Big Five component is the temperament axes verbatim, and the
    /// discrete traits come off the pass-3 threshold table, so the two
    /// views can never disagree
    pub fn personalities(&self)
        -> (crate::sim::psychology::Personality, crate::game::traits::Personality)
    {
        use crate::game::traits::Trait::*;
        let [o, c, e, a, n] = self.temperament;
        let (t1, t2) = self.top_values();
        let (b1, b2) = self.bottom_values();
        let top = |v: usize| v == t1 || v == t2;
        let low = |v: usize| v == b1 || v == b2;
        // the pass-3 table (design part 1), in TRAITS order so a probe's
        // first-match reads the same trait order the roll used to deal
        let mut traits = vec![];
        if c < 0.25 { traits.push(Lazy); }
        if top(3) || top(4) { traits.push(Ambitious); }
        if a > 0.70 { traits.push(Kind); }
        if top(5) && low(8) { traits.push(Stingy); }
        if top(7) { traits.push(Pious); }  // the devotion leg binds once faith's live
        if n > 0.65 && a < 0.40 { traits.push(Hotheaded); }
        if e > 0.70 { traits.push(Gregarious); }
        if top(4) && a < 0.50 { traits.push(Proud); }
        if c > 0.60 && n < 0.35 { traits.push(Steady); }
        if self.trust_propensity < 0.30 { traits.push(Wary); }
        if e < 0.25 && n < 0.40 { traits.push(Stolid); }
        if a < 0.15 { traits.push(Cruel); }
        if e > 0.65 && a < 0.30 { traits.push(SelfAbsorbed); }
        (
            crate::sim::psychology::Personality {
                openness: o, neuroticism: n, extraversion: e,
                agreeableness: a, conscientiousness: c,
            },
            crate::game::traits::Personality { traits },
        )
    }
}

/// What the earlier passes hand down - near-empty today, filled when
/// prehistory and households land (#1419, #1420). Bundled so `generate`'s
/// signature doesn't churn as the passes come in
/// A prehistory event the person was in, as pass 1 hands it down -
/// what the ghost pass rolls a mark from (E2-3, #1421)
#[derive(Debug, Clone)]
pub struct Mark {
    /// Its record in the event log
    pub id: EventId,
    /// What kind of happening it was
    pub kind: EventKind,
    /// The part they played in it
    pub part: &'static str,
    /// How hard it hit, 0 to 1 - the mark roll's severity leg
    pub severity: f32,
}

#[derive(Debug, Default, Clone)]
pub struct Past {
    /// Prehistory events this person was in (pass 1)
    pub story: Vec<EventId>,
    /// The events' own records for the ghost pass (E2-3): the kind,
    /// the part they played and how hard it hit
    pub marks: Vec<Mark>,
    /// Parents' mean temperament, when the household pass ran one
    /// (pass 2) - a child's starts there plus noise
    pub parents: Option<[f32; 5]>,
    /// The trade they came up in: tilts the value circle (pass 4)
    pub vocation: Option<Vocation>,
}

fn mix(mut x: u64) -> u64 {
    x ^= x >> 30; x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27; x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31; x
}

/// The cast slot a persona's seed was generated from - the roster's
/// index shifts home under departures but the seed never does (E4-8,
/// #1426). Newcomers generated past the founding cast land above the
/// scan and come back None
pub(crate) fn cast_slot(world_seed: u64, p: &Persona) -> Option<usize> {
    (0..64).find(|&slot| p.seed == mix(world_seed ^ (slot as u64)
        .wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15)))
}

/// The n-th draw off a seed, as a 0..1 fraction
pub(crate) fn draw(seed: u64, n: u64) -> f32 {
    (mix(seed ^ n.wrapping_mul(0x9E37_79B9_7F4A_7C15)) % 10_000) as f32 / 10_000.0
}

/// The value a trade leans toward, if it leans
fn vocation_pole(v: Vocation) -> Option<usize> {
    Some(match v {
        Vocation::Priest | Vocation::Bookbinder => 7,  // Custom
        Vocation::Knight => 4,                        // Standing
        Vocation::Brewer | Vocation::Distiller | Vocation::Taverner => 3, // Advancement
        Vocation::Farmer | Vocation::Shepherd | Vocation::Fisher
        | Vocation::Alewife | Vocation::Miner => 5,   // Safety
        Vocation::Painter | Vocation::Luthier | Vocation::Perfumer => 0, // Freedom
        _ => return None,
    })
}

/// Generate a villager's persona from the world seed - deterministic:
/// same seed, same slot, same past, same person. Passes 1-2 pass their
/// work in through `past`; pass 5's ghosts wait on #1421
/// `attempt` salts the draws of passes 3-6 (biome, temperament, values,
/// life story, signature) so a cast that fails a check rerolls just
/// those for the offending villager (E1-5, #1413): prehistory,
/// household and body stay fixed. attempt 0 is the natural roll
pub fn generate(world_seed: u64, slot: usize, past: &Past) -> Persona {
    generate_attempt(world_seed, slot, past, 0)
}

/// generate with the reroll salt for the cast validator (#1413)
pub fn generate_attempt(world_seed: u64, slot: usize, past: &Past, attempt: u32) -> Persona {
    let seed = mix(world_seed ^ (slot as u64).wrapping_add(1)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let d = |n: u64| draw(seed, n);
    let rseed = seed.wrapping_add((attempt as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    let r = |n: u64| draw(rseed, n);

    // the biome first: passes 3 and 4 pull toward its region
    let archetype = ARCHETYPES[(r(0) * ARCHETYPES.len() as f32) as usize
        % ARCHETYPES.len()];

    let (pull, pole, trust_pull) = archetype.prior();

    // pass 3, temperament: seeded draws leaned toward the biome; a
    // child's starts at the parents' mean when the household ran
    let mut temperament = [0.0f32; 5];
    for i in 0..5 {
        let mut t = (r(10 + i as u64) * 0.7 + (0.5 + pull[i]) * 0.3).clamp(0.0, 1.0);
        // a child's starts at the parents' mean, plus noise (pass 2)
        if let Some(p) = past.parents {
            t = (p[i] + (r(20 + i as u64) - 0.5) * 0.3).clamp(0.0, 1.0);
        }
        temperament[i] = t;
    }

    // trust propensity: childhood events set it eventually; today a
    // seeded draw leaned by the biome
    let trust_propensity = (r(30) * 0.7 + (0.5 + trust_pull) * 0.3).clamp(0.0, 1.0);

    // pass 4, values: a direction on the circle, each value falling off
    // with angle so opposites come out low; the trade tilts its pole
    let dir = pole as f32 + (r(40) - 0.5) * 3.0;
    let mut values = [0.0f32; 10];
    for i in 0..10 {
        let gap = (i as f32 - dir) / 10.0 * std::f32::consts::TAU;
        values[i] = (0.35 + 0.45 * gap.cos() + (r(50 + i as u64) - 0.5) * 0.2)
            .clamp(0.0, 1.0);
    }
    if let Some(v) = past.vocation.and_then(vocation_pole) {
        values[v] = (values[v] + 0.3).min(1.0);
    }

    // a deliberate crack, when the thresholds collide: low
    // conscientiousness against a high Advancement value tags the
    // idle_striver (design part 1, pass 3)
    let top = {
        let mut t = (0usize, 1usize);
        for i in 1..10 {
            if values[i] > values[t.0] { t = (i, t.0); }
            else if values[i] > values[t.1] { t = (t.0, i); }
        }
        t
    };
    // Ambitious: Advancement or Standing among the top two values.
    // "low" reads as the lower half here - the strict C < 0.25
    // collision only fired ~2% of the time, far under the design's
    // "~1 in 6" (#1413); the lazy streak against the ambitious pole
    // lands at about one cast member in five
    let contradiction = if temperament[1] < 0.5
        && [top.0, top.1].iter().any(|&i| i == 3 || i == 4) {
        Some(Contradiction::IdleStriver)
    } else {
        None
    };

    // pass 6, signature: a visible landmark no neighbor shares (the
    // cast check in #1413 enforces it), and a hidden one
    let signature = SIGNATURES[(r(70) * SIGNATURES.len() as f32) as usize
        % SIGNATURES.len()];
    let hidden = HIDDENS[(r(80) * HIDDENS.len() as f32) as usize % HIDDENS.len()];
    // the ear's landmark (E4-7, #1416): a first draw off the person's
    // own stream - the cast deal overwrites it for uniqueness
    let idiolect = crate::game::idiolect::Idiolect {
        phrase: crate::game::idiolect::PHRASES[
            (r(75) * crate::game::idiolect::PHRASES.len() as f32) as usize
                % crate::game::idiolect::PHRASES.len()],
        habit: crate::game::idiolect::HABITS[
            (r(85) * crate::game::idiolect::HABITS.len() as f32) as usize
                % crate::game::idiolect::HABITS.len()],
    };

    // pass 5, life story (E2-3, #1421): the events they were in roll
    // their marks - weighted by severity and neuroticism, and the
    // deepest two land as ghosts. Nothing simulates why: the lesson
    // comes off the event x disposition table
    let disp = Disposition::of(archetype, &values);
    let mut scored: Vec<(f32, &Mark)> = past.marks.iter()
        // a warm memory, not a ghost
        .filter(|m| m.kind != EventKind::HarvestFeast)
        .map(|m| (m.severity * (0.35 + 0.65 * temperament[4]) * r(110 + m.id.0 as u64), m))
        .filter(|(s, _)| *s > 0.35)
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let ghosts: Vec<Ghost> = scored.into_iter().take(2)
        .map(|(s, m)| {
            let (hook, lie, fear, need) = rationalize(m.kind, disp, m.part);
            Ghost { event: m.id, kind: m.kind, hook, lie, strength: s.min(1.0), fear, need, turned: false }
        }).collect();

    // the body (pass 0, E1-3): drawn off the person's own stream so
    // names and pronouns can agree with it
    let sex = if d(90) < 0.5 { Sex::Female } else { Sex::Male };

    Persona {
        seed, sex, archetype, temperament, trust_propensity, values,
        story: past.story.clone(), ghosts,
        signature, idiolect, hidden, contradiction,
    }
}

/// The stance the ghost table keys on (E2-3): who they are when it
/// happens to them. Archetypes cluster into four; a person with
/// Fairness in their top two takes the Just row instead - the same
/// fire teaches the fearful to keep their head down and them that the
/// truth comes out
#[derive(Debug, Clone, Copy)]
enum Disposition {
    /// Burned, Doubter: keep your head down
    Meek,
    /// OldGuard, Devout, QuietHand: bear it, keep the custom
    Steadfast,
    /// SoftHeart, Merrymaker: keep hold of one another
    Tender,
    /// Striver, Firebrand, TongueOfTheWell, Climber: someone pays for this
    Restless,
    /// Fairness in the top two values: the truth matters more
    Just,
}

impl Disposition {
    fn of(archetype: Archetype, values: &[f32; 10]) -> Self {
        let mut top = (usize::MAX, usize::MAX);
        let mut v = (f32::MIN, f32::MIN);
        for (i, &x) in values.iter().enumerate() {
            if x > v.0 { top = (i, top.0); v = (x, v.0); }
            else if x > v.1 { top.1 = i; v.1 = x; }
        }
        if top.0 == 9 || top.1 == 9 { return Disposition::Just; }
        match archetype {
            Archetype::Burned | Archetype::Doubter => Disposition::Meek,
            Archetype::OldGuard | Archetype::Devout | Archetype::QuietHand => Disposition::Steadfast,
            Archetype::SoftHeart | Archetype::Merrymaker => Disposition::Tender,
            _ => Disposition::Restless,
        }
    }
}

/// The rationalization table (E2-3): each event kind teaches each
/// disposition a lesson - a lie or a true belief, a fear, a need, and
/// the HOOKS row that carries it where one exists. The part played
/// picks the row where the authored wounds differ by side: the burned
/// out carry `burnt_barn`, a watcher who let the blame land carries
/// `blamed_neighbor`, and the scapegoat and the levy's survivors keep
/// records only - no authored line fits them yet. HarvestFeast never
/// reaches here: the feast is a warm memory, not a ghost
fn rationalize(kind: EventKind, d: Disposition, part: &str) -> (Option<usize>, Belief, Fear, Need) {
    let (hook, lie, fear, need) = rationalize_row(kind, d);
    let hook = match (kind, part) {
        (EventKind::Fire, "burned out") => Some(5),
        (EventKind::Fire, "watched it burn") => Some(6),
        (EventKind::Fire, "blamed") => None,
        // the one who remembers a taken brother carries it; the one who
        // served and came home keeps a survivor's wound, unwritten
        (EventKind::Levy, "served and came home") => None,
        _ => hook,
    };
    (hook, lie, fear, need)
}

/// The kind x disposition rows the rationalization reads
fn rationalize_row(kind: EventKind, d: Disposition) -> (Option<usize>, Belief, Fear, Need) {
    use Disposition::*;
    match kind {
        EventKind::Fire => match d {
            Meek => (Some(5), Belief::SomeoneToBlame, Fear::WhispersTurn, Need::ToBeSafe),
            Steadfast => (Some(6), Belief::SomeoneToBlame, Fear::FireReturns, Need::ToSetItRight),
            Tender => (Some(5), Belief::SomeoneToBlame, Fear::FireReturns, Need::ToBelong),
            Restless => (Some(6), Belief::SomeoneToBlame, Fear::WhispersTurn, Need::ToSetItRight),
            Just => (Some(6), Belief::TruthComesOut, Fear::WhispersTurn, Need::ToBeBelieved),
        },
        EventKind::Levy => match d {
            Meek => (Some(0), Belief::WarsTakeSons, Fear::BuriedLikeHim, Need::ToBeSafe),
            Steadfast => (Some(0), Belief::LordAlwaysWins, Fear::BuriedLikeHim, Need::ToLayItDown),
            Tender => (Some(0), Belief::WarsTakeSons, Fear::AloneInIt, Need::ToBelong),
            Restless => (Some(1), Belief::WarsTakeSons, Fear::BuriedLikeHim, Need::ToSetItRight),
            Just => (Some(1), Belief::LordAlwaysWins, Fear::AloneInIt, Need::ToSetItRight),
        },
        EventKind::DeathDuty => match d {
            Meek => (None, Belief::LordTakesFromDead, Fear::BailiffKnowsMyFace, Need::ToBeSafe),
            Steadfast => (None, Belief::LordTakesFromDead, Fear::BailiffKnowsMyFace, Need::ToProvide),
            Tender => (None, Belief::LordTakesFromDead, Fear::BellTolls, Need::ToBelong),
            Restless | Just => (None, Belief::LordTakesFromDead, Fear::BailiffKnowsMyFace, Need::ToSetItRight),
        },
        EventKind::Fever => match d {
            Meek => (Some(4), Belief::GodSends, Fear::BellTolls, Need::ToBeSafe),
            Steadfast => (Some(4), Belief::GodSends, Fear::BellTolls, Need::ToLayItDown),
            Tender => (Some(4), Belief::GodSends, Fear::BellTolls, Need::ToBelong),
            Restless => (Some(4), Belief::GodSends, Fear::AloneInIt, Need::ToBeSeen),
            Just => (Some(4), Belief::GodSends, Fear::BellTolls, Need::ToBeBelieved),
        },
        EventKind::MillerDebt => match d {
            Meek => (Some(2), Belief::MillOwnsUs, Fear::CollectorAtTheDoor, Need::ToBeSafe),
            Steadfast => (Some(2), Belief::MillOwnsUs, Fear::CollectorAtTheDoor, Need::ToProvide),
            Tender => (Some(2), Belief::MillOwnsUs, Fear::CollectorAtTheDoor, Need::ToBelong),
            Restless | Just => (Some(2), Belief::MillOwnsUs, Fear::CollectorAtTheDoor, Need::ToSetItRight),
        },
        EventKind::BrokenBetrothal => match d {
            Meek => (Some(7), Belief::PromisesForKeepers, Fear::WhispersTurn, Need::ToBeSeen),
            Steadfast => (Some(7), Belief::PromisesForKeepers, Fear::WhispersTurn, Need::ToLayItDown),
            Tender => (Some(7), Belief::PromisesForKeepers, Fear::AloneInIt, Need::ToBelong),
            Restless => (Some(7), Belief::PromisesForKeepers, Fear::AloneInIt, Need::ToBeSeen),
            Just => (Some(7), Belief::PromisesForKeepers, Fear::WhispersTurn, Need::ToSetItRight),
        },
        EventKind::OldFaith => match d {
            Meek => (Some(3), Belief::KeepHidden, Fear::StonesRemember, Need::ToBeSafe),
            Steadfast => (Some(3), Belief::KeepHidden, Fear::StonesRemember, Need::ToLayItDown),
            Tender => (Some(3), Belief::KeepHidden, Fear::StonesRemember, Need::ToBeSeen),
            Restless => (Some(3), Belief::KeepHidden, Fear::StonesRemember, Need::ToBeSeen),
            Just => (Some(3), Belief::KeepHidden, Fear::StonesRemember, Need::ToBeBelieved),
        },
        EventKind::Eviction => match d {
            Meek => (None, Belief::NeverCrossReeve, Fear::BailiffKnowsMyFace, Need::ToBeSafe),
            Steadfast => (None, Belief::NeverCrossReeve, Fear::CollectorAtTheDoor, Need::ToProvide),
            Tender => (None, Belief::NeverCrossReeve, Fear::AloneInIt, Need::ToBelong),
            Restless | Just => (None, Belief::NeverCrossReeve, Fear::BailiffKnowsMyFace, Need::ToSetItRight),
        },
        // unreachable in `generate` - the feast is filtered from the roll
        EventKind::HarvestFeast => (None, Belief::StrongTogether, Fear::AloneInIt, Need::ToBelong),
    }
}

/// A ghost this deep shows in how they live: about one villager in
/// eight carries the worst of it, so a buried child lands once in the
/// village and numbs in bulk - not everyone
const SEVERE: f32 = 0.7;

/// The cast's ghost caps (E2-4, #1422), run after the whole cast
/// generates: no wound held by more than two villagers - shared event
/// or not, the soldier-brother pile-up stays gone - and the severe
/// ghosts capped near one in eight. The weakest holders' ghosts demote
/// to light marks: the events stay in their story, the wound doesn't
/// form. #1413's validation pass can own this list when it lands
pub fn cap_cast_ghosts(cast: &mut [Option<Persona>]) {
    use std::cmp::Ordering;
    let cap = cast.len() / 8 + 1;
    // every wound in the cast, strongest holder first within a type
    let mut by_lie: Vec<(usize, f32, usize, EventId)> = vec![];
    for (s, p) in cast.iter().enumerate() {
        if let Some(p) = p {
            for g in &p.ghosts {
                by_lie.push((g.lie as usize, g.strength, s, g.event));
            }
        }
    }
    by_lie.sort_by(|a, b| a.0.cmp(&b.0)
        .then(b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal))
        .then(a.2.cmp(&b.2)).then(a.3.0.cmp(&b.3.0)));
    let mut drop: crate::engine::det::HashSet<(usize, EventId)> = Default::default();
    for run in by_lie.chunk_by(|a, b| a.0 == b.0) {
        for &(_, _, s, e) in run.iter().skip(2) {
            drop.insert((s, e));
        }
    }
    // then the severe cap, over what the duplicate pass left
    let mut severe: Vec<(f32, usize, EventId)> = by_lie.iter()
        .filter(|&(_, st, s, e)| *st >= SEVERE && !drop.contains(&(*s, *e)))
        .map(|&(_, st, s, e)| (st, s, e)).collect();
    severe.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal)
        .then(a.1.cmp(&b.1)).then(a.2.0.cmp(&b.2.0)));
    for &(_, s, e) in severe.iter().skip(cap) {
        drop.insert((s, e));
    }
    for (s, p) in cast.iter_mut().enumerate() {
        if let Some(p) = p {
            p.ghosts.retain(|g| !drop.contains(&(s, g.event)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_persona() {
        let a = generate(42, 7, &Past::default());
        let b = generate(42, 7, &Past::default());
        assert_eq!(a, b);
    }

    /// A departure shifts the roster's seats, never the seed's slot
    /// (E4-8, #1426): the story pass finds each soul by it
    #[test]
    fn the_seed_gives_back_the_slot() {
        for slot in 0..39 {
            let p = generate(77, slot, &Past::default());
            assert_eq!(cast_slot(77, &p), Some(slot));
        }
        // another world's seed claims no slot here
        let p = generate(78, 7, &Past::default());
        assert_eq!(cast_slot(77, &p), None);
    }

    #[test]
    fn different_slots_differ() {
        let past = Past::default();
        let mut diffs = 0;
        for slot in 0..39 {
            if generate(42, slot, &past) != generate(42, 0, &past) { diffs += 1; }
        }
        assert!(diffs >= 30, "slots should differ: {}", diffs);
    }

    #[test]
    fn different_worlds_differ() {
        assert_ne!(generate(42, 7, &Past::default()),
                   generate(43, 7, &Past::default()));
    }

    #[test]
    fn temperament_and_values_stay_in_range() {
        for slot in 0..39 {
            let p = generate(99, slot, &Past::default());
            assert!(p.temperament.iter().all(|&t| (0.0..=1.0).contains(&t)));
            assert!(p.values.iter().all(|&v| (0.0..=1.0).contains(&v)));
            assert!((0.0..=1.0).contains(&p.trust_propensity));
        }
    }

    #[test]
    fn a_trade_tilts_the_values() {
        // priests lean Custom more often than farmers across a cast
        let lean = |v: Vocation| {
            (0..40).filter(|&s| {
                let p = generate(7, s, &Past {
                    vocation: Some(v), ..Past::default() });
                let (a, b) = p.top_values();
                a == 7 || b == 7
            }).count()
        };
        assert!(lean(Vocation::Priest) > lean(Vocation::Miner));
    }

    #[test]
    fn the_household_pulls_the_child() {
        // a child of steady folks starts nearer steady than without
        let mean = |parents: Option<[f32; 5]>| {
            (0..40).map(|s| {
                generate(5, s, &Past { parents, ..Past::default() }).temperament[1] as f64
            }).sum::<f64>() / 40.0
        };
        assert!(mean(Some([0.9; 5])) > mean(None));
    }

    #[test]
    fn the_story_comes_through() {
        let past = Past { story: vec![EventId(3), EventId(9)], ..Past::default() };
        assert_eq!(generate(1, 0, &past).story, vec![EventId(3), EventId(9)]);
    }

    /// A Past with one of every kind - severe parts in several, so the
    //  mark roll lands
    fn storied_past() -> Past {
        Past {
            story: (0..5).map(EventId).collect(),
            marks: vec![
                Mark { id: EventId(0), kind: EventKind::Fire, part: "burned out", severity: 0.9 },
                Mark { id: EventId(1), kind: EventKind::Levy, part: "taken", severity: 0.9 },
                Mark { id: EventId(2), kind: EventKind::Fever, part: "buried their own", severity: 0.9 },
                Mark { id: EventId(3), kind: EventKind::HarvestFeast, part: "feasted", severity: 0.3 },
                Mark { id: EventId(4), kind: EventKind::MillerDebt, part: "owes the miller", severity: 0.6 },
            ],
            ..Past::default()
        }
    }

    #[test]
    fn every_ghost_cites_its_event() {
        // the done-when: ghosts cite the event that left them, capped
        // at two, and the feast never leaves one
        for s in 0..60 {
            let past = storied_past();
            let p = generate(11, s, &past);
            assert!(p.ghosts.len() <= 2, "slot {} carries {}", s, p.ghosts.len());
            for g in &p.ghosts {
                assert!(past.marks.iter().any(|m| m.id == g.event),
                    "ghost cites {:?} which touched nobody", g.event);
                assert!(p.story.contains(&g.event));
            }
            assert!(!p.ghosts.iter().any(|g|
                past.marks.iter().find(|m| m.id == g.event).unwrap().kind == EventKind::HarvestFeast));
        }
    }

    #[test]
    fn the_ghost_roll_replays() {
        // same seed, same past: the same ghosts
        let a = generate(13, 4, &storied_past());
        let b = generate(13, 4, &storied_past());
        assert_eq!(a.ghosts, b.ghosts);
        assert!(!a.ghosts.is_empty(), "a storied past leaves a mark somewhere");
    }

    #[test]
    fn the_event_and_the_disposition_set_the_lesson() {
        // the table, not simulation: fire teaches a Just person that
        // the truth comes out, a meek one to keep their head down
        let mut values = [0.3; 10];
        values[9] = 0.9; // Fairness top
        let lie_just = rationalize(EventKind::Fire, Disposition::Just, "burned out").1;
        assert_eq!(lie_just, Belief::TruthComesOut);
        let lie_meek = rationalize(EventKind::Fire, Disposition::Meek, "burned out").1;
        assert_eq!(lie_meek, Belief::SomeoneToBlame);
        // the part picks the wound's hook: a watcher who let blame land
        // carries `blamed_neighbor`; the scapegoat has no authored line
        assert_eq!(rationalize(EventKind::Fire, Disposition::Meek, "watched it burn").0, Some(6));
        assert_eq!(rationalize(EventKind::Fire, Disposition::Meek, "blamed").0, None);
        assert_eq!(rationalize(EventKind::Levy, Disposition::Meek, "served and came home").0, None);
        // and the disposition picks Just off the value circle
        assert!(matches!(Disposition::of(Archetype::QuietHand, &values), Disposition::Just));
        assert!(matches!(Disposition::of(Archetype::Burned, &[0.3; 10]), Disposition::Meek));
    }

    #[test]
    fn no_past_no_ghosts() {
        assert!(generate(1, 0, &Past::default()).ghosts.is_empty());
    }

    #[test]
    fn the_cast_caps_the_pile_up() {
        // E2-4 (#1422): five villagers carrying the same wound keep the
        // two deepest; the rest demote to light marks. And over a cast
        // the severe ghosts stay near one in eight
        let ghost = |event: u32, lie: Belief, strength: f32| Ghost {
            event: EventId(event), kind: EventKind::Fire, hook: None, lie, strength,
            fear: Fear::BuriedLikeHim, need: Need::ToBelong, turned: false,
        };
        let mut cast: Vec<Option<Persona>> = (0..16).map(|s| {
            let mut p = generate(3, s, &Past::default());
            p.ghosts = match s {
                // five holders of the soldier-brother wound
                0..=4 => vec![ghost(s as u32, Belief::WarsTakeSons, 0.5 + s as f32 * 0.02)],
                // the rest all severe, each wound rare enough to pass
                // the duplicate cap
                _ => vec![ghost(s as u32, [
                    Belief::MillOwnsUs, Belief::GodSends, Belief::KeepHidden,
                    Belief::LordAlwaysWins, Belief::SomeoneToBlame,
                    Belief::LordTakesFromDead,
                ][s % 6], 0.9 + s as f32 * 0.001)],
            };
            Some(p)
        }).collect();
        cap_cast_ghosts(&mut cast);
        let held: Vec<&Ghost> = cast.iter().flatten().flat_map(|p| p.ghosts.iter()).collect();
        assert_eq!(held.iter().filter(|g| g.lie == Belief::WarsTakeSons).count(), 2,
            "the pile-up keeps only its two deepest");
        // the two strongest (slots 3 and 4) keep it
        assert!(cast[3].as_ref().unwrap().ghosts.iter().any(|g| g.lie == Belief::WarsTakeSons));
        assert!(cast[4].as_ref().unwrap().ghosts.iter().any(|g| g.lie == Belief::WarsTakeSons));
        assert!(cast[0].as_ref().unwrap().ghosts.is_empty());
        // 16 villagers cap at 3 severe: the eleven deep wounds beyond
        // the pile-up demote to light marks
        assert_eq!(held.iter().filter(|g| g.strength >= SEVERE).count(), 3,
            "severe ghosts held at the cap");
    }

    #[test]
    fn a_lazy_striver_is_tagged() {
        // Advancement OR Standing anywhere in the top two counts, not
        // only first - and a lazy-ish streak must ride against it
        // ("low" is the lower half; C < 0.25 fired too rarely, #1413)
        let mut found = false;
        for slot in 0..400 {
            let p = generate(3, slot, &Past::default());
            let lazy = p.temperament[1] < 0.5;
            let (a, b) = p.top_values();
            let ambitious = a == 3 || a == 4 || b == 3 || b == 4;
            assert_eq!(p.contradiction == Some(Contradiction::IdleStriver),
                       lazy && ambitious, "slot {}", slot);
            found |= p.contradiction == Some(Contradiction::IdleStriver);
        }
        assert!(found, "someone in a big cast should earn the tag");
    }

    #[test]
    fn a_cast_spreads_across_the_biomes() {
        let mut seen = [false; 11];
        for slot in 0..39 { seen[ARCHETYPES.iter()
            .position(|&a| a == generate(11, slot, &Past::default()).archetype)
            .unwrap()] = true; }
        assert!(seen.iter().filter(|&&s| s).count() >= 5);
    }

    fn persona_with(temperament: [f32; 5], values: [f32; 10], trust: f32) -> Persona {
        Persona {
            seed: 1, sex: Sex::Male, archetype: Archetype::OldGuard,
            temperament, trust_propensity: trust, values,
            story: vec![], ghosts: vec![],
            signature: Signature::Habit("x"), hidden: Landmark::Skill("y"),
            idiolect: crate::game::idiolect::Idiolect {
                phrase: "x", habit: crate::game::idiolect::Habit::Echo },
            contradiction: None,
        }
    }

    #[test]
    fn the_big_five_is_the_axes_verbatim() {
        let (big5, _) = persona_with([0.1, 0.2, 0.3, 0.4, 0.5], [0.5; 10], 0.5).personalities();
        assert_eq!((big5.openness, big5.conscientiousness, big5.extraversion,
            big5.agreeableness, big5.neuroticism), (0.1, 0.2, 0.3, 0.4, 0.5));
    }

    #[test]
    fn the_traits_come_off_the_threshold_table() {
        use crate::game::traits::Trait::*;
        let flat = [0.5; 10];
        // O, C, E, A, N
        let kind = persona_with([0.5, 0.5, 0.5, 0.75, 0.5], flat, 0.5).personalities().1;
        assert_eq!(kind.traits, vec![Kind]);
        let cruel = persona_with([0.5, 0.5, 0.5, 0.10, 0.5], flat, 0.5).personalities().1;
        assert!(cruel.traits.contains(&Cruel) && !cruel.traits.contains(&Kind));
        let hot = persona_with([0.5, 0.5, 0.5, 0.30, 0.80], flat, 0.5).personalities().1;
        assert!(hot.traits.contains(&Hotheaded) && !hot.traits.contains(&Steady));
        let steady = persona_with([0.5, 0.65, 0.5, 0.5, 0.30], flat, 0.5).personalities().1;
        assert!(steady.traits.contains(&Steady) && !steady.traits.contains(&Lazy));
        let watcher = persona_with([0.5; 5], flat, 0.10).personalities().1;
        assert!(watcher.traits.contains(&Wary));
    }

    #[test]
    fn values_drive_the_rank_traits() {
        use crate::game::traits::Trait::*;
        // Standing and Safety top, Care bottoms: proud and stingy,
        // and ambitious for the standing
        let mut values = [0.4; 10];
        values[4] = 0.9; values[5] = 0.85; values[8] = 0.05;
        let (_, t) = persona_with([0.5, 0.5, 0.5, 0.40, 0.5], values, 0.5).personalities();
        assert!(t.traits.contains(&Ambitious) && t.traits.contains(&Proud)
            && t.traits.contains(&Stingy), "{:?}", t.traits);
        // Custom tops: pious
        let mut values = [0.4; 10];
        values[7] = 0.9;
        let (_, t) = persona_with([0.5; 5], values, 0.5).personalities();
        assert!(t.traits.contains(&Pious), "{:?}", t.traits);
    }

    #[test]
    fn lazy_and_ambitious_only_with_the_crack() {
        use crate::game::traits::Trait::*;
        // the table can only pair Lazy with Ambitious where generate
        // already tagged the idle_striver - the crack's the only door
        for slot in 0..400 {
            let p = generate(9, slot, &Past::default());
            let t = p.personalities().1.traits;
            // a strict Lazy+Ambitious pair is always a crack (the
            // crack's gate is wider, so the strict pair is inside it)
            let lazy_ambitious = t.contains(&Lazy) && t.contains(&Ambitious);
            assert!(!lazy_ambitious || p.contradiction == Some(Contradiction::IdleStriver),
                "slot {}", slot);
            // and opposites never share a body
            assert!(!(t.contains(&Kind) && t.contains(&Cruel)));
        }
    }
}
