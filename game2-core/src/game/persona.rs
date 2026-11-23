//! Persona (#1406, E1-1; design `docs/plans/procedural-characters.md`
//! part 1): the deep model under a villager - seeded once, changed only
//! by recorded events. Six passes build it; the prehistory and household
//! passes land in #1419/#1420 and the ghost pass in #1421, so `generate`
//! takes what they'll hand down and the call itself doesn't churn.

use crate::game::vocation::Vocation;

/// A prehistory event's record (pass 1, #1419): an index into the
/// event log the thirty years before play writes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub u32);

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

impl Belief {
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
/// `hook` indexes `hooks::HOOKS` so the old wound carries its lines (#1421)
#[derive(Debug, Clone, PartialEq)]
pub struct Ghost {
    /// The event that left it
    pub event: EventId,
    /// Index into `hooks::HOOKS`, kept
    pub hook: usize,
    /// The belief it taught, with its strength: campaign outcomes move it
    pub lie: Belief,
    /// How hard they hold it, 0 to 1
    pub strength: f32,
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
    /// Secret passion, skill or past
    pub hidden: Landmark,
    /// A deliberate crack, when the thresholds collide
    pub contradiction: Option<Contradiction>,
}

impl Persona {
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
}

/// What the earlier passes hand down - near-empty today, filled when
/// prehistory and households land (#1419, #1420). Bundled so `generate`'s
/// signature doesn't churn as the passes come in
#[derive(Debug, Default, Clone)]
pub struct Past {
    /// Prehistory events this person was in (pass 1)
    pub story: Vec<EventId>,
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

/// The n-th draw off a seed, as a 0..1 fraction
fn draw(seed: u64, n: u64) -> f32 {
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
pub fn generate(world_seed: u64, slot: usize, past: &Past) -> Persona {
    let seed = mix(world_seed ^ (slot as u64).wrapping_add(1)
        .wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let d = |n: u64| draw(seed, n);

    // the biome first: passes 3 and 4 pull toward its region
    let archetype = ARCHETYPES[(d(0) * ARCHETYPES.len() as f32) as usize
        % ARCHETYPES.len()];
    let (pull, pole, trust_pull) = archetype.prior();

    // pass 3, temperament: seeded draws leaned toward the biome; a
    // child's starts at the parents' mean when the household ran
    let mut temperament = [0.0f32; 5];
    for i in 0..5 {
        let mut t = (d(10 + i as u64) * 0.7 + (0.5 + pull[i]) * 0.3).clamp(0.0, 1.0);
        // a child's starts at the parents' mean, plus noise (pass 2)
        if let Some(p) = past.parents {
            t = (p[i] + (d(20 + i as u64) - 0.5) * 0.3).clamp(0.0, 1.0);
        }
        temperament[i] = t;
    }

    // trust propensity: childhood events set it eventually; today a
    // seeded draw leaned by the biome
    let trust_propensity = (d(30) * 0.7 + (0.5 + trust_pull) * 0.3).clamp(0.0, 1.0);

    // pass 4, values: a direction on the circle, each value falling off
    // with angle so opposites come out low; the trade tilts its pole
    let dir = pole as f32 + (d(40) - 0.5) * 3.0;
    let mut values = [0.0f32; 10];
    for i in 0..10 {
        let gap = (i as f32 - dir) / 10.0 * std::f32::consts::TAU;
        values[i] = (0.35 + 0.45 * gap.cos() + (d(50 + i as u64) - 0.5) * 0.2)
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
    // Ambitious: Advancement or Standing among the top two values
    let contradiction = if temperament[1] < 0.25
        && [top.0, top.1].iter().any(|&i| i == 3 || i == 4) {
        Some(Contradiction::IdleStriver)
    } else {
        None
    };

    // pass 6, signature: a visible landmark no neighbor shares (the
    // cast check in #1413 enforces it), and a hidden one
    let signature = SIGNATURES[(d(70) * SIGNATURES.len() as f32) as usize
        % SIGNATURES.len()];
    let hidden = HIDDENS[(d(80) * HIDDENS.len() as f32) as usize % HIDDENS.len()];

    Persona {
        seed, archetype, temperament, trust_propensity, values,
        story: past.story.clone(),
        // pass 5 lands with #1421: the story's events roll into ghosts
        ghosts: Vec::new(),
        signature, hidden, contradiction,
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

    #[test]
    fn a_lazy_striver_is_tagged() {
        // Advancement OR Standing anywhere in the top two counts, not
        // only first - and a Lazy streak must ride against it
        let mut found = false;
        for slot in 0..400 {
            let p = generate(3, slot, &Past::default());
            let lazy = p.temperament[1] < 0.25;
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
}
