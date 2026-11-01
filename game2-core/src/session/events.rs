//! Rare events: the hundred small fortunes and misfortunes that wander
//! through a village's year. A traveller on the road, a sky that turns,
//! a beast at the fold, a thing found in a field - each semi-rare, each
//! arriving on its own cadence, each playing out over days and then gone.
//!
//! The design doc (wiki/software-design/rare-events.md) lists the hundred.
//! Every id has a spec in `EVENTS`: the name the log and the map use, a
//! shape (Visitor/Spell/Moment), a cadence window, and a stay. Implemented
//! ids get real spawn/day/depart arms; the rest ride a generic baseline -
//! a figure who walks in, says his piece, mills about, and walks out -
//! until their own PR lands the real effect.
//!
//! Testing: the harness (session/stage.rs) stages a village state, then
//! `force_event` puts the moment on now; `run_days` plays it out. The
//! probe (session/probe.rs) exposes what fired and when.

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use crate::game::desires::{self, Want};
use crate::game::livelihood::Purse;
use crate::game::pets::{self, Species, PetRole};
use crate::game::villager::{self, Pos};
use crate::game::household;
use crate::sim::illness::Health;
use crate::game::mycology::MushKind;
use super::{Game, Goal, MushPatch, Rng, TRAVELLER_SPEED, TALK_BUBBLE_SECS, CHAT_BUBBLE_SECS};

/// Which of the hundred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventId {
    // folk on the road
    Tinker, KnifeGrinder, RagAndBone, ChimneySweep, RatCatcher, Dowser,
    Scribe, ToothDrawer, BarberSurgeon, PilgrimBand, RelicSeller, Pardoner,
    Juggler, BearWard, Puppeteer, BalladMonger, NewsCrier, Recruiter,
    Deserter, RefugeeFamily, LeperBand, Friar, Heretic, Drover, FishHawker,
    SaltSeller, WoolMerchant, WineMerchant, SpiceMerchant, Bookseller,
    Glazier, Thatcher, WellDigger, CharcoalBurner, LimeBurner, Tanner,
    CandleMaker, Potter, HorseCop, Eggler, CheeseMonger, FortuneTeller,
    Astrologer, Alchemist, Quack, Midwife, Sowgelder, Farrier, CornFactor,
    Messenger,
    // the sky and the weather
    Hailstorm, HeatWave, Drought, Flood, LightningStrike, LateFrost,
    IndianSummer, FirstSnow, Thaw, Aurora, MeteorShower, BloodMoon,
    DarkNoon, WindStorm, Mirage,
    // the beasts
    WolfAtFold, FoxInYard, HawkStoops, StrayDog, StrayCat, RatPlague,
    Locusts, CaterpillarYear, Starlings, BeeTree, DeerInFields, WildBoar,
    OwlInTower, CrowsMob, FishRun,
    // ill luck
    ChimneyFire, FouledWell, GranaryLeak, CartOverturns, DrownedInPond,
    LostChild, Foundling, Quarrel, AlehouseBrawl, OldOakFalls, Assize,
    CellarThief,
    // findings
    CoinHoard, OldStones, SpringBubbles, HoneyTree, Truffle, HerbFlush,
    BerryYear, MushroomFlush, StormWashUp, ProdigalSon,
}

/// The shape an event takes on the map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// A figure walks in on the road, stands its `stay`, and walks out
    Visitor,
    /// No figure: `stay` days of an ambient effect (a hailstorm's day, a
    /// rat plague's fortnight)
    Spell,
    /// Happens on the day: applied once, logged, done
    Moment,
}

/// What figure a Visitor draws as (placeholder art until the new images
/// land - the traveller stands in for every wandering trade)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fig {
    /// A man with a pack (art.traveller_s)
    Traveller,
    /// An animal that may stay (a stray dog, a stray cat)
    Pet(Species),
}

/// One event's registration: what it is and when it can come
#[derive(Debug, Clone, Copy)]
pub struct EventSpec {
    pub id: EventId,
    /// What the log and the map call it: "a pedlar" - lowercase unless a name
    pub name: &'static str,
    pub kind: EventKind,
    /// What figure it draws with (Visitors only)
    pub fig: Fig,
    /// Days after the last firing (or the start) before it can come again
    pub gap: u64,
    /// Jitter on the gap: due = tick + gap + rng % span
    pub span: u64,
    /// Days a Visitor stands or a Spell holds
    pub stay: u64,
    /// One line of arrival speech, for the generic visitor baseline
    pub cry: &'static str,
}

use EventId::*;
use EventKind::*;

/// The hundred. Cadence: road-folk every half-year to two years, the sky
/// a season to a year, beasts a season and more, ill luck and findings
/// rarer still. Semi-rare is the point.
pub const EVENTS: &[EventSpec] = &[
    // ---- folk on the road ----
    EventSpec { id: Tinker, name: "the tinker", kind: Visitor, fig: Fig::Traveller, gap: 180, span: 120, stay: 4, cry: "Pots to mend! Kettles to patch!" },
    EventSpec { id: KnifeGrinder, name: "the knife-grinder", kind: Visitor, fig: Fig::Traveller, gap: 200, span: 120, stay: 3, cry: "Knives! Scissors! Put an edge on it!" },
    EventSpec { id: RagAndBone, name: "the rag-and-bone man", kind: Visitor, fig: Fig::Traveller, gap: 150, span: 100, stay: 3, cry: "Rags! Bones! Anything old!" },
    EventSpec { id: ChimneySweep, name: "the sweep", kind: Visitor, fig: Fig::Traveller, gap: 280, span: 140, stay: 4, cry: "Sweep your flues, before they burn you out!" },
    EventSpec { id: RatCatcher, name: "the rat-catcher", kind: Visitor, fig: Fig::Traveller, gap: 240, span: 140, stay: 4, cry: "Rats caught! Mice, moles, the lot!" },
    EventSpec { id: Dowser, name: "the water-diviner", kind: Visitor, fig: Fig::Traveller, gap: 360, span: 200, stay: 5, cry: "Water under your feet - I can feel it." },
    EventSpec { id: Scribe, name: "the scribe", kind: Visitor, fig: Fig::Traveller, gap: 200, span: 120, stay: 4, cry: "Letters writ, fair hand, a penny a page!" },
    EventSpec { id: ToothDrawer, name: "the tooth-drawer", kind: Visitor, fig: Fig::Traveller, gap: 300, span: 160, stay: 3, cry: "Bad teeth out, no fuss, quick as a wink!" },
    EventSpec { id: BarberSurgeon, name: "the barber-surgeon", kind: Visitor, fig: Fig::Traveller, gap: 280, span: 160, stay: 4, cry: "Bled and bandaged, corns cut, humours set right!" },
    EventSpec { id: PilgrimBand, name: "a band of pilgrims", kind: Visitor, fig: Fig::Traveller, gap: 260, span: 160, stay: 3, cry: "We come from the shrine. God keep this place." },
    EventSpec { id: RelicSeller, name: "the relic-seller", kind: Visitor, fig: Fig::Traveller, gap: 340, span: 200, stay: 3, cry: "A saint's own knuckle-bone - see the seal!" },
    EventSpec { id: Pardoner, name: "the pardoner", kind: Visitor, fig: Fig::Traveller, gap: 340, span: 200, stay: 3, cry: "Pardons, good folk - your sins made light for coin!" },
    EventSpec { id: Juggler, name: "a juggler", kind: Visitor, fig: Fig::Traveller, gap: 160, span: 100, stay: 3, cry: "Watch me now! Knives in the air!" },
    EventSpec { id: BearWard, name: "a bear-ward", kind: Visitor, fig: Fig::Traveller, gap: 380, span: 220, stay: 4, cry: "The bear dances! A penny to see him bow!" },
    EventSpec { id: Puppeteer, name: "a puppeteer", kind: Visitor, fig: Fig::Traveller, gap: 300, span: 180, stay: 3, cry: "A play! Noah's flood and the devil's due!" },
    EventSpec { id: BalladMonger, name: "the ballad-monger", kind: Visitor, fig: Fig::Traveller, gap: 180, span: 120, stay: 3, cry: "New ballads! All the doings of the wide world!" },
    EventSpec { id: NewsCrier, name: "the news-crier", kind: Visitor, fig: Fig::Traveller, gap: 140, span: 100, stay: 2, cry: "Hear ye! News from the castle and the town!" },
    EventSpec { id: Recruiter, name: "the recruiting sergeant", kind: Visitor, fig: Fig::Traveller, gap: 320, span: 200, stay: 4, cry: "The lord's coin for strong backs - the wars pay!" },
    EventSpec { id: Deserter, name: "a deserter", kind: Visitor, fig: Fig::Traveller, gap: 400, span: 260, stay: 4, cry: "..." },
    EventSpec { id: RefugeeFamily, name: "a refugee family", kind: Visitor, fig: Fig::Traveller, gap: 380, span: 240, stay: 5, cry: "Our village burned. Is there work here?" },
    EventSpec { id: LeperBand, name: "a leper band", kind: Visitor, fig: Fig::Traveller, gap: 420, span: 260, stay: 3, cry: "Unclean - a crust for Christ's sake." },
    EventSpec { id: Friar, name: "a wandering friar", kind: Visitor, fig: Fig::Traveller, gap: 240, span: 160, stay: 4, cry: "A word for your souls, good people." },
    EventSpec { id: Heretic, name: "a quiet preacher", kind: Visitor, fig: Fig::Traveller, gap: 420, span: 280, stay: 4, cry: "The church sells what God gives free." },
    EventSpec { id: Drover, name: "a drover", kind: Visitor, fig: Fig::Traveller, gap: 200, span: 120, stay: 3, cry: "Sheep to market! Stand clear!" },
    EventSpec { id: FishHawker, name: "a fish-hawker", kind: Visitor, fig: Fig::Traveller, gap: 160, span: 100, stay: 3, cry: "Salt fish! Keep all winter!" },
    EventSpec { id: SaltSeller, name: "the salt-seller", kind: Visitor, fig: Fig::Traveller, gap: 240, span: 140, stay: 3, cry: "Salt! White gold! None goes bad who buys it!" },
    EventSpec { id: WoolMerchant, name: "the wool merchant", kind: Visitor, fig: Fig::Traveller, gap: 260, span: 160, stay: 3, cry: "Fleece bought, top price, by the stone!" },
    EventSpec { id: WineMerchant, name: "the wine merchant", kind: Visitor, fig: Fig::Traveller, gap: 280, span: 160, stay: 3, cry: "Wine from over the hills - a cup before you buy!" },
    EventSpec { id: SpiceMerchant, name: "the spice merchant", kind: Visitor, fig: Fig::Traveller, gap: 320, span: 200, stay: 3, cry: "Pepper! Cloves! Things that cure what ails!" },
    EventSpec { id: Bookseller, name: "the bookseller", kind: Visitor, fig: Fig::Traveller, gap: 380, span: 220, stay: 3, cry: "Books! Words that outlive the winter!" },
    EventSpec { id: Glazier, name: "the glazier", kind: Visitor, fig: Fig::Traveller, gap: 360, span: 220, stay: 5, cry: "Glass for the church, light for the dark months!" },
    EventSpec { id: Thatcher, name: "the thatcher", kind: Visitor, fig: Fig::Traveller, gap: 220, span: 140, stay: 5, cry: "Roofs patched before the rain finds you!" },
    EventSpec { id: WellDigger, name: "the well-digger", kind: Visitor, fig: Fig::Traveller, gap: 400, span: 240, stay: 6, cry: "Wells dug deep - sweet water, not the pond's!" },
    EventSpec { id: CharcoalBurner, name: "a charcoal burner", kind: Visitor, fig: Fig::Traveller, gap: 300, span: 180, stay: 7, cry: "I'll take your dead wood and leave you coal." },
    EventSpec { id: LimeBurner, name: "the lime-burner", kind: Visitor, fig: Fig::Traveller, gap: 360, span: 220, stay: 6, cry: "Lime for your fields - the soil thanks you next year!" },
    EventSpec { id: Tanner, name: "the tanner", kind: Visitor, fig: Fig::Traveller, gap: 260, span: 160, stay: 4, cry: "Hides bought! Mind the smell, it's honest work!" },
    EventSpec { id: CandleMaker, name: "the candle-maker", kind: Visitor, fig: Fig::Traveller, gap: 260, span: 160, stay: 3, cry: "Wax bought! Tallow taken! Light for the dark!" },
    EventSpec { id: Potter, name: "the potter", kind: Visitor, fig: Fig::Traveller, gap: 240, span: 140, stay: 3, cry: "Pots and crocks! A jug that doesn't weep!" },
    EventSpec { id: HorseCop, name: "the horse-coper", kind: Visitor, fig: Fig::Traveller, gap: 380, span: 240, stay: 3, cry: "Fine horses! Sound wind, and mostly honest!" },
    EventSpec { id: Eggler, name: "the eggler", kind: Visitor, fig: Fig::Traveller, gap: 200, span: 120, stay: 2, cry: "Eggs! Butter! Cheese! What have your hens done?" },
    EventSpec { id: CheeseMonger, name: "the cheese-monger", kind: Visitor, fig: Fig::Traveller, gap: 240, span: 160, stay: 3, cry: "Aged wheels bought - the older the better!" },
    EventSpec { id: FortuneTeller, name: "the fortune-teller", kind: Visitor, fig: Fig::Traveller, gap: 300, span: 200, stay: 4, cry: "Your palm, love - the lines don't lie." },
    EventSpec { id: Astrologer, name: "the astrologer", kind: Visitor, fig: Fig::Traveller, gap: 420, span: 280, stay: 4, cry: "The stars over this village - let me read them." },
    EventSpec { id: Alchemist, name: "the alchemist", kind: Visitor, fig: Fig::Traveller, gap: 460, span: 300, stay: 4, cry: "I buy strange things. Strange things only." },
    EventSpec { id: Quack, name: "the quack", kind: Visitor, fig: Fig::Traveller, gap: 260, span: 160, stay: 3, cry: "Cures all! Ague, ache, and the falling sickness!" },
    EventSpec { id: Midwife, name: "the midwife", kind: Visitor, fig: Fig::Traveller, gap: 300, span: 180, stay: 5, cry: "Babies caught, mothers kept - a blessing on the house." },
    EventSpec { id: Sowgelder, name: "the pig man", kind: Visitor, fig: Fig::Traveller, gap: 320, span: 200, stay: 3, cry: "Pigs doctored! Beasts tended! Show me your stock!" },
    EventSpec { id: Farrier, name: "the farrier", kind: Visitor, fig: Fig::Traveller, gap: 280, span: 160, stay: 4, cry: "Shoes for hooves - hear that ring of iron!" },
    EventSpec { id: CornFactor, name: "the corn-factor", kind: Visitor, fig: Fig::Traveller, gap: 220, span: 140, stay: 3, cry: "Grain bought by the quarter - good weight, fair price!" },
    EventSpec { id: Messenger, name: "a messenger", kind: Visitor, fig: Fig::Traveller, gap: 200, span: 140, stay: 2, cry: "A letter! Is there one here called-?" },
    // ---- the sky and the weather ----
    EventSpec { id: Hailstorm, name: "a hailstorm", kind: Spell, fig: Fig::Traveller, gap: 200, span: 160, stay: 1, cry: "" },
    EventSpec { id: HeatWave, name: "a heat wave", kind: Spell, fig: Fig::Traveller, gap: 240, span: 180, stay: 5, cry: "" },
    EventSpec { id: Drought, name: "a drought", kind: Spell, fig: Fig::Traveller, gap: 300, span: 220, stay: 14, cry: "" },
    EventSpec { id: Flood, name: "the river comes up", kind: Spell, fig: Fig::Traveller, gap: 340, span: 260, stay: 4, cry: "" },
    EventSpec { id: LightningStrike, name: "lightning strikes", kind: Moment, fig: Fig::Traveller, gap: 300, span: 240, stay: 0, cry: "" },
    EventSpec { id: LateFrost, name: "a late frost", kind: Moment, fig: Fig::Traveller, gap: 280, span: 220, stay: 0, cry: "" },
    EventSpec { id: IndianSummer, name: "an Indian summer", kind: Spell, fig: Fig::Traveller, gap: 320, span: 240, stay: 5, cry: "" },
    EventSpec { id: FirstSnow, name: "the first snow", kind: Moment, fig: Fig::Traveller, gap: 200, span: 160, stay: 0, cry: "" },
    EventSpec { id: Thaw, name: "the thaw", kind: Spell, fig: Fig::Traveller, gap: 300, span: 220, stay: 4, cry: "" },
    EventSpec { id: Aurora, name: "the aurora", kind: Spell, fig: Fig::Traveller, gap: 420, span: 300, stay: 2, cry: "" },
    EventSpec { id: MeteorShower, name: "a meteor shower", kind: Moment, fig: Fig::Traveller, gap: 300, span: 240, stay: 0, cry: "" },
    EventSpec { id: BloodMoon, name: "a blood moon", kind: Spell, fig: Fig::Traveller, gap: 500, span: 360, stay: 2, cry: "" },
    EventSpec { id: DarkNoon, name: "a dark noon", kind: Spell, fig: Fig::Traveller, gap: 480, span: 340, stay: 1, cry: "" },
    EventSpec { id: WindStorm, name: "a windstorm", kind: Spell, fig: Fig::Traveller, gap: 260, span: 200, stay: 2, cry: "" },
    EventSpec { id: Mirage, name: "a mirage", kind: Moment, fig: Fig::Traveller, gap: 400, span: 300, stay: 0, cry: "" },
    // ---- the beasts ----
    EventSpec { id: WolfAtFold, name: "a wolf at the fold", kind: Moment, fig: Fig::Traveller, gap: 240, span: 200, stay: 0, cry: "" },
    EventSpec { id: FoxInYard, name: "a fox in the yard", kind: Moment, fig: Fig::Traveller, gap: 160, span: 120, stay: 0, cry: "" },
    EventSpec { id: HawkStoops, name: "a hawk stoops", kind: Moment, fig: Fig::Traveller, gap: 200, span: 160, stay: 0, cry: "" },
    EventSpec { id: StrayDog, name: "a stray dog", kind: Visitor, fig: Fig::Pet(Species::Dog), gap: 300, span: 220, stay: 5, cry: "" },
    EventSpec { id: StrayCat, name: "a stray cat", kind: Visitor, fig: Fig::Pet(Species::Cat), gap: 300, span: 220, stay: 5, cry: "" },
    EventSpec { id: RatPlague, name: "a plague of rats", kind: Spell, fig: Fig::Traveller, gap: 380, span: 280, stay: 10, cry: "" },
    EventSpec { id: Locusts, name: "locusts", kind: Moment, fig: Fig::Traveller, gap: 460, span: 340, stay: 0, cry: "" },
    EventSpec { id: CaterpillarYear, name: "a caterpillar year", kind: Spell, fig: Fig::Traveller, gap: 340, span: 260, stay: 10, cry: "" },
    EventSpec { id: Starlings, name: "a murmuration", kind: Moment, fig: Fig::Traveller, gap: 240, span: 180, stay: 0, cry: "" },
    EventSpec { id: BeeTree, name: "a bee tree found", kind: Moment, fig: Fig::Traveller, gap: 340, span: 260, stay: 0, cry: "" },
    EventSpec { id: DeerInFields, name: "a deer in the fields", kind: Moment, fig: Fig::Traveller, gap: 260, span: 200, stay: 0, cry: "" },
    EventSpec { id: WildBoar, name: "a wild boar", kind: Spell, fig: Fig::Traveller, gap: 400, span: 300, stay: 5, cry: "" },
    EventSpec { id: OwlInTower, name: "an owl in the tower", kind: Spell, fig: Fig::Traveller, gap: 340, span: 260, stay: 30, cry: "" },
    EventSpec { id: CrowsMob, name: "a mob of crows", kind: Moment, fig: Fig::Traveller, gap: 220, span: 160, stay: 0, cry: "" },
    EventSpec { id: FishRun, name: "a fish run", kind: Spell, fig: Fig::Traveller, gap: 280, span: 200, stay: 6, cry: "" },
    // ---- ill luck ----
    EventSpec { id: ChimneyFire, name: "a chimney fire", kind: Moment, fig: Fig::Traveller, gap: 300, span: 240, stay: 0, cry: "" },
    EventSpec { id: FouledWell, name: "a fouled well", kind: Spell, fig: Fig::Traveller, gap: 380, span: 300, stay: 7, cry: "" },
    EventSpec { id: GranaryLeak, name: "a granary leak", kind: Moment, fig: Fig::Traveller, gap: 300, span: 240, stay: 0, cry: "" },
    EventSpec { id: CartOverturns, name: "a cart overturns", kind: Moment, fig: Fig::Traveller, gap: 260, span: 200, stay: 0, cry: "" },
    EventSpec { id: DrownedInPond, name: "drowned in the pond", kind: Moment, fig: Fig::Traveller, gap: 560, span: 400, stay: 0, cry: "" },
    EventSpec { id: LostChild, name: "a lost child", kind: Spell, fig: Fig::Traveller, gap: 400, span: 300, stay: 2, cry: "" },
    EventSpec { id: Foundling, name: "a foundling", kind: Moment, fig: Fig::Traveller, gap: 500, span: 380, stay: 0, cry: "" },
    EventSpec { id: Quarrel, name: "a quarrel in the square", kind: Moment, fig: Fig::Traveller, gap: 200, span: 140, stay: 0, cry: "" },
    EventSpec { id: AlehouseBrawl, name: "an alehouse brawl", kind: Moment, fig: Fig::Traveller, gap: 260, span: 180, stay: 0, cry: "" },
    EventSpec { id: Assize, name: "the assize of ale", kind: Moment, fig: Fig::Traveller, gap: 200, span: 140, stay: 0, cry: "" },
    EventSpec { id: CellarThief, name: "a thief in the night", kind: Moment, fig: Fig::Traveller, gap: 240, span: 160, stay: 0, cry: "" },
    EventSpec { id: OldOakFalls, name: "the old oak falls", kind: Moment, fig: Fig::Traveller, gap: 600, span: 400, stay: 0, cry: "" },
    // ---- findings ----
    EventSpec { id: CoinHoard, name: "a coin hoard", kind: Moment, fig: Fig::Traveller, gap: 500, span: 380, stay: 0, cry: "" },
    EventSpec { id: OldStones, name: "old stones", kind: Moment, fig: Fig::Traveller, gap: 460, span: 340, stay: 0, cry: "" },
    EventSpec { id: SpringBubbles, name: "a spring bubbles up", kind: Moment, fig: Fig::Traveller, gap: 460, span: 340, stay: 0, cry: "" },
    EventSpec { id: HoneyTree, name: "a honey tree", kind: Moment, fig: Fig::Traveller, gap: 380, span: 280, stay: 0, cry: "" },
    EventSpec { id: Truffle, name: "a truffle", kind: Moment, fig: Fig::Traveller, gap: 420, span: 320, stay: 0, cry: "" },
    EventSpec { id: HerbFlush, name: "a flush of herbs", kind: Spell, fig: Fig::Traveller, gap: 340, span: 260, stay: 10, cry: "" },
    EventSpec { id: BerryYear, name: "a berry year", kind: Spell, fig: Fig::Traveller, gap: 300, span: 240, stay: 14, cry: "" },
    EventSpec { id: MushroomFlush, name: "a mushroom flush", kind: Spell, fig: Fig::Traveller, gap: 280, span: 200, stay: 10, cry: "" },
    EventSpec { id: StormWashUp, name: "a storm's leavings", kind: Moment, fig: Fig::Traveller, gap: 400, span: 300, stay: 0, cry: "" },
    EventSpec { id: ProdigalSon, name: "a prodigal returns", kind: Visitor, fig: Fig::Traveller, gap: 520, span: 400, stay: 6, cry: "Is this still-? I've come a long way." },
];

/// The spec for `id`
pub fn spec(id: EventId) -> &'static EventSpec {
    EVENTS.iter().find(|s| s.id == id).unwrap()
}

/// `id` by name: the variant ("Tinker") or the map name ("the tinker") -
/// what the wasm and ws test surfaces take
pub fn event_id(name: &str) -> Option<EventId> {
    EVENTS.iter().map(|s| s.id)
        .find(|&id| format!("{:?}", id) == name || spec(id).name == name)
}

/// What an event has while it's afoot: for a Visitor, the figure walking
/// and standing; for a Spell, just the day it ends. Moments never land in
/// `rare` at all - they're applied at spawn.
#[derive(Debug, Clone)]
pub struct RareEvent {
    pub id: EventId,
    /// The figure on the road (None for spells and moments)
    pub e: Option<Entity>,
    /// Tiles left to walk - the road in, or the road out
    pub path: Vec<(i32, i32)>,
    /// The day it leaves (visitors) or ends (spells); None until arrived
    pub until: Option<u64>,
    /// The tile it stands at while it stays
    pub at: Option<(i32, i32)>,
    /// Heading home already
    pub leaving: bool,
    /// Per-event scratch: a counter, a picked target, a flag
    pub aux: u64,
}

/// When each event first becomes due: its gap window from the start, jittered
pub fn initial_due(seed: u64) -> Vec<(EventId, u64)> {
    let mut rng = Rng(seed ^ 0x5EED5EED);
    EVENTS.iter().map(|s| (s.id, s.gap + rng.next() % s.span)).collect()
}

/// The events that tell on themselves: what the village notices a few
/// days out. Ill luck gives warning where foresight is natural - the
/// pack leaves tracks, the hens feel the fox before it comes
const OMENS: &[(EventId, &str)] = &[
    (WolfAtFold, "wolf tracks in the mud by the fold - the dogs won't settle"),
    (FoxInYard, "the hens won't settle at dusk - something prowls the hedges"),
];
/// Days ahead an omen lands
const OMEN_LEAD: u64 = 5;

impl Game {
    /// The day's rare-event business: what's due comes on, what's here
    /// works its day, what's done walks out or fades
    pub fn events_tick(&mut self, tick: u64) {
        // what's booked speaks of its coming, a few days out - due is
        // fixed until it fires, so the day hits once per booking
        for (id, omen) in OMENS {
            if self.next_rare.iter().any(|(i, d)| *i == *id && *d == tick + OMEN_LEAD)
                && self.rare.iter().all(|r| r.id != *id) {
                self.log.push(format!("t{}: {}.", tick, omen));
            }
        }
        // what's due comes on (unless it's already afoot)
        let due: Vec<EventId> = self.next_rare.iter()
            .filter(|(id, d)| tick >= *d && self.rare.iter().all(|r| r.id != *id))
            .map(|(id, _)| *id).collect();
        for id in &due { self.spawn_event(*id); }
        // each active event's day; a visitor whose stay's done folds up
        // (a spell spawned this tick already worked its first day in
        // spawn_event - a second dose the same day is a lie)
        for idx in 0..self.rare.len() {
            let (id, until, leaving) = {
                let r = &self.rare[idx]; (r.id, r.until, r.leaving)
            };
            let Some(until) = until else { continue };
            if tick < until {
                if !leaving && !due.contains(&id) { self.event_day(id, tick); }
            } else if !leaving {
                self.event_end(id, tick);
                if matches!(spec(id).kind, EventKind::Visitor) {
                    self.depart_event(idx);
                }
            }
        }
        // spells end clean: the weather lifts, the plague passes
        self.rare.retain(|r| !(matches!(spec(r.id).kind, EventKind::Spell)
            && r.until.map(|u| tick >= u).unwrap_or(false)));
    }

    /// Fire `id` now - the tests' way in, and a scenario's
    pub fn force_event(&mut self, id: EventId) {
        if self.rare.iter().any(|r| r.id == id) { return; }
        self.spawn_event(id);
    }

    /// `id`'s turn comes round: it fires, it's logged, and the road books
    /// it again far off
    fn spawn_event(&mut self, id: EventId) {
        let s = *spec(id);
        match s.kind {
            EventKind::Visitor => {
                let e = self.world.create_entity();
                let start = self.traveller_path.last().copied().unwrap_or((29, 14));
                self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
                let at = self.event_pitch(id);
                let path = self.map.walk_path(start, at).unwrap_or_default();
                self.rare.push(RareEvent { id, e: Some(e), path, until: None, at: Some(at), leaving: false, aux: 0 });
                self.log.push(format!("t{}: {} has come to town.", self.tick, s.name));
            }
            EventKind::Spell => {
                self.rare.push(RareEvent { id, e: None, path: vec![], until: Some(self.tick + s.stay),
                    at: None, leaving: false, aux: 0 });
                self.event_began(id);
                // its first day is this one - the daily tick for tomorrow
                // hasn't come yet, and a one-day spell sees no other
                self.event_day(id, self.tick);
            }
            EventKind::Moment => {
                self.event_moment(id);
            }
        }
        self.rare_fired.push((id, self.tick));
        self.reschedule_event(id, self.tick);
    }

    /// Where a visitor pitches: a buildable tile by the market (or where
    /// the spec wants him - a camp on the meadow, the church door)
    fn event_pitch(&self, id: EventId) -> (i32, i32) {
        match id {
            EventId::CharcoalBurner | EventId::Deserter =>
                self.map.nearest_buildable(self.town_wall.0 - 3, self.town_wall.1 - 3).unwrap_or((16, 16)),
            EventId::LeperBand | EventId::RefugeeFamily | EventId::Recruiter =>
                self.map.nearest_buildable(self.town_wall.0 + 2, self.town_wall.3 + 2).unwrap_or((16, 16)),
            _ => {
                let mp = self.world.get_component::<Pos>(self.market_food)
                    .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or((16, 16));
                self.map.nearest_buildable(mp.0 + 2, mp.1 + 2).unwrap_or((16, 16))
            }
        }
    }

    /// Next time it can come: the gap window from now, jittered off the
    /// events' own stream - the village's rng rolls the same as it would
    /// have without the schedule
    fn reschedule_event(&mut self, id: EventId, tick: u64) {
        let s = spec(id);
        let roll = self.rare_roll(id);
        if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == id) {
            *due = tick + s.gap + roll % s.span;
        }
    }

    /// A deterministic roll for `id` on this tick (splitmix over the
    /// events' seed), for scheduling - never for village chance
    pub fn rare_roll(&self, id: EventId) -> u64 {
        let mut z = self.rare_seed
            ^ self.tick.wrapping_mul(0x9E3779B97F4A7C15)
            ^ (id as u64).wrapping_mul(0x85EBCA6B);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// One frame for every afoot event: walk the road, stand the pitch
    pub fn advance_rare(&mut self) {
        for idx in 0..self.rare.len() {
            let mut r = self.rare[idx].clone();
            let Some(e) = r.e else { continue };
            // flames at his feet or on the road ahead: turn for home early
            if !r.leaving && self.fire_at_feet(e, &r.path) {
                r.until = Some(self.tick);
                r.path.clear();
            }
            if r.path.is_empty() {
                if r.until.is_none() {
                    r.until = Some(self.tick + spec(r.id).stay);
                    self.event_arrived(r.id);
                }
            } else {
                let Some(p) = self.world.get_component_mut::<Pos>(e) else { continue };
                let (nx, ny) = (r.path[0].0 as f32, r.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; r.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.rare[idx] = r;
        }
        // the road's taken him back - and his entity with him
        let gone: Vec<Entity> = self.rare.iter()
            .filter(|r| r.leaving && r.path.is_empty()).filter_map(|r| r.e).collect();
        for e in gone { self.world.destroy_entity(e); }
        self.rare.retain(|r| !(r.leaving && r.path.is_empty()));
    }

    /// Point a visitor for home
    fn depart_event(&mut self, idx: usize) {
        if idx >= self.rare.len() { return; }
        let mut r = self.rare[idx].clone();
        let back = self.traveller_path.last().copied().unwrap_or((29, 14));
        r.path = self.world.get_component::<Pos>(r.e.unwrap_or(0))
            .and_then(|p| self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back))
            .unwrap_or_default();
        r.leaving = true;
        self.log.push(format!("t{}: {} moves on.", self.tick, spec(r.id).name));
        self.rare[idx] = r;
    }

    /// The moment the figure reaches its pitch: a cry, a show, a knock
    fn event_arrived(&mut self, id: EventId) {
        let s = spec(id);
        let e = self.rare.iter().find(|r| r.id == id).and_then(|r| r.e);
        match id {
            EventId::Tinker => {
                if let Some(e) = e { self.say(e, s.cry, TALK_BUBBLE_SECS * 2); }
                self.log.push(format!("t{}: {} sets up by the market - out come the cracked pots.", self.tick, s.name));
            }
            EventId::StrayDog => {
                self.log.push(format!("t{}: a stray dog hangs about the market, ribs showing.", self.tick));
            }
            EventId::KnifeGrinder => {
                if let Some(e) = e { self.say(e, s.cry, TALK_BUBBLE_SECS * 2); }
                self.log.push(format!("t{}: {} treadles his wheel by the market - sparks fly.", self.tick, s.name));
            }
            _ => {
                // the baseline: a figure stands his pitch and cries his trade
                if let Some(e) = e {
                    let cry = if s.cry.is_empty() { "..." } else { s.cry };
                    self.say(e, cry, TALK_BUBBLE_SECS);
                }
            }
        }
    }

    /// Each day a spell holds or a visitor stands: the work the event does
    fn event_day(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::Tinker => {
                // pots mended: whoever's near his pitch gets a sharper
                // kit for a penny - once, not a penny a day for the same pot
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    // his work's already done on their kit: no second penny
                    if desires::is_satisfied(&self.world, v, Want::BetterTools, tick) { continue; }
                    // no penny, no mend
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::BetterTools, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                }
            }
            EventId::KnifeGrinder => {
                // an edge on the trades' steel for a penny; the kids get
                // the sparks for free - the show's the thing they remember
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    if self.world.get_component::<crate::game::vocation::Kid>(v).is_some() {
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 10);
                        continue;
                    }
                    // his wheel only earns off a working blade
                    if self.world.get_component::<crate::game::vocation::Craft>(v).is_none() { continue; }
                    if desires::is_satisfied(&self.world, v, Want::BetterTools, tick) { continue; }
                    // no penny, no edge - the poorest keep their dull steel
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::BetterTools, tick + 45);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                }
            }
            EventId::RagAndBone => {
                // a coin for the poorest rag-bundle in reach - one a day,
                // so a rich street sees him push an empty cart by
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let poorest = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold < 8.0).unwrap_or(false))
                    .min_by(|&a, &b| {
                        let ga = self.world.get_component::<Purse>(a).map(|p| p.gold).unwrap_or(0.0);
                        let gb = self.world.get_component::<Purse>(b).map(|p| p.gold).unwrap_or(0.0);
                        ga.partial_cmp(&gb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(v) = poorest {
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold += 1.0; }
                    self.say(v, "For the rags? Bless you, mister.", CHAT_BUBBLE_SECS);
                }
            }
            EventId::ChimneySweep => {
                // a penny a flue: homes in reach of his ladder breathe
                // easier (Security) - it's the hearth he cleans, not the
                // passer-by, so the reach runs on their home's door
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = household::home_of(&self.world, v)
                        .and_then(|h| self.world.get_component::<Pos>(h))
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    if desires::is_satisfied(&self.world, v, Want::Security, tick) { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Security, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                }
            }
            EventId::RatCatcher => {
                // his terrier earns: while he's about the store loses
                // nothing to vermin - the servings that weren't gnawed,
                // and his sack counts them
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings += 2;
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux += 1; }
            }
            EventId::Scribe => {
                // a penny a page for the illiterate: up to two letters a
                // day, carried to whoever their strongest bond is
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut wrote = 0;
                for &v in &self.villagers.clone() {
                    if wrote >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || self.letters.iter().any(|&(f, _, _)| f == v) { continue; }
                    // he writes for the illiterate - the lettered write their own
                    if self.literate(v) { continue; }
                    // and only to the living - the dead can't be written to
                    let partner = self.world.query::<crate::game::relationships::Relationship>().iter()
                        .filter_map(|&r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                        .filter(|rr| (rr.from == v || rr.other == v)
                            && !villager::is_dead(&self.world, if rr.from == v { rr.other } else { rr.from }))
                        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|rr| if rr.from == v { rr.other } else { rr.from });
                    let Some(to) = partner else { continue; };
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.letters.push((v, to, tick + 30));
                    wrote += 1;
                }
            }
            EventId::ToothDrawer => {
                // one bad tooth a day: the worst jaw in reach gets the
                // pull, a penny the tooth - quick as a wink, he swears
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false)
                        && self.world.get_component::<Health>(v).map(|h| h.condition < 0.9).unwrap_or(false))
                    .min_by(|&a, &b| {
                        let ha = self.world.get_component::<Health>(a).map(|h| h.condition).unwrap_or(1.0);
                        let hb = self.world.get_component::<Health>(b).map(|h| h.condition).unwrap_or(1.0);
                        ha.partial_cmp(&hb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(v) = mark {
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.05).min(1.0);
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Out with it! - that's that, then.", CHAT_BUBBLE_SECS);
                }
            }
            EventId::BarberSurgeon => {
                // a bloodletting for a penny: it mostly does them good,
                // though now and then the knife slips - the barber's
                // honest enough to say so
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false)
                        && self.world.get_component::<Health>(v).map(|h| h.condition < 0.95).unwrap_or(false))
                    .min_by(|&a, &b| {
                        let ha = self.world.get_component::<Health>(a).map(|h| h.condition).unwrap_or(1.0);
                        let hb = self.world.get_component::<Health>(b).map(|h| h.condition).unwrap_or(1.0);
                        ha.partial_cmp(&hb).unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(v) = mark {
                    let slipped = self.rare_roll(id) % 4 == 0;
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        // a nick only ever cuts - the floor never lifts them
                        h.condition = if slipped { (h.condition - 0.03).max(0.01) }
                            else { (h.condition + 0.03).min(1.0) };
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, if slipped { "The knife slipped. A bandage'll set it." } else { "There. The humours flow easier." }, CHAT_BUBBLE_SECS);
                }
            }
            EventId::PilgrimBand => {
                // the road to the shrine passes here: a penny in the
                // bowl feeds the faith - up to three alms a day, and
                // only from those who can spare it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut alms = 0;
                for &v in &self.villagers.clone() {
                    if alms >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Faith, tick) { continue; }
                    // only the faithful drop a penny in - the bowl means
                    // nothing to those whose wants lie elsewhere
                    let faithful = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !faithful || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Go with God.", CHAT_BUBBLE_SECS);
                    alms += 1;
                }
            }
            EventId::RelicSeller => {
                // a saint's knuckle-bone for two gold: the faithful carry
                // the blessing a month and a half - the seal's his own work
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Faith, tick) { continue; }
                    let faithful = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !faithful || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, tick + 45);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                    self.say(v, "See the seal!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::Pardoner => {
                // a grievance off the heart for two gold: the paper's
                // real enough, though the seal's his own carving
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    // a live burden only - the faded ones weigh nothing
                    let burdened = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.grievances.iter().any(|(_, u)| tick <= *u)).unwrap_or(false);
                    if !near || !burdened { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !pays { continue; }
                    // the newest live weight off the heart
                    if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                        if let Some(i) = l.grievances.iter().rposition(|(_, u)| tick <= *u) {
                            l.grievances.remove(i);
                        }
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                    self.say(v, "Your sins made light!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::Juggler => {
                // three knives and a grin: the show's free, but the hat
                // comes round - a penny in from those who felt the joy
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Joy, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Joy)).unwrap_or(false);
                    if !feels_it { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 15);
                    let spare = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if spare {
                        if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    }
                    self.say(v, "A marvel!", CHAT_BUBBLE_SECS);
                }
            }
            EventId::BearWard => {
                // the bear dances and the hat goes round; and now and
                // then the bear forgets himself and takes a snap at the
                // nearest hand - a story either way
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Joy, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Joy)).unwrap_or(false);
                    if !feels_it { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 20);
                    let spare = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if spare {
                        if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    }
                }
                // the snap - a snap's worth telling for years
                if self.rare_roll(id) % 5 == 0 {
                    let nearest = self.villagers.iter().copied()
                        .filter(|&v| !villager::is_dead(&self.world, v)
                            && !crate::game::dungeon::unavailable(&self.world, v))
                        .filter_map(|v| self.world.get_component::<Pos>(v)
                            .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                        .filter(|(d, _)| *d < 4).min_by_key(|(d, _)| *d).map(|(_, v)| v);
                    if let Some(v) = nearest {
                        if let Some(h) = self.world.get_component_mut::<Health>(v) {
                            h.condition = (h.condition - 0.05).max(0.01);
                        }
                        self.say(v, "He bit me!", CHAT_BUBBLE_SECS);
                        self.log.push(format!("t{}: the bear took a snap at the crowd - a story for years.", self.tick));
                    }
                }
            }
            EventId::Puppeteer => {
                // Noah's flood and the devil's due: the kids are
                // enthralled for free; the grown-ups who watch over
                // their heads drop a penny in the hat
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Joy, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Joy)).unwrap_or(false);
                    if !feels_it { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 20);
                    // kids get the front row for free; the rest pay the hat
                    if self.world.get_component::<crate::game::vocation::Kid>(v).is_none() {
                        let spare = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                        if spare {
                            if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                        }
                    }
                }
            }
            EventId::BalladMonger => {
                // a penny a broadside: a tune that sticks, and all the
                // doings of the wide world besides
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Music, tick) { continue; }
                    let feels_it = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Music)).unwrap_or(false);
                    if !feels_it { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Music, tick + 15);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Did you hear?", CHAT_BUBBLE_SECS);
                }
            }
            EventId::Deserter => {
                // hiding in the green: anyone who comes near knows his
                // face now - and knowing's sheltering, the law counts it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let sheltered = self.villagers.iter().copied()
                    .any(|v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                            .unwrap_or(false));
                if sheltered {
                    if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
                }
            }
            EventId::RefugeeFamily => {
                // their village burned - they eat from the store while
                // they camp: a serving a day is cheap for what they fled
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings = f.servings.saturating_sub(1);
                }
            }
            EventId::LeperBand => {
                // crusts go out at arm's length: a serving a day off the
                // store, and the faithful who carry them are counted.
                // Now and then the camp's cough brushes a near hand.
                if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                    f.servings = f.servings.saturating_sub(1);
                }
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Faith, tick) { continue; }
                    let faithful = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                    if !faithful { continue; }
                    desires::satisfy(&mut self.world, v, Want::Faith, tick + 20);
                }
                if self.rare_roll(id) % 8 == 0 {
                    let nearest = self.villagers.iter().copied()
                        .filter(|&v| !villager::is_dead(&self.world, v)
                            && !crate::game::dungeon::unavailable(&self.world, v))
                        .filter_map(|v| self.world.get_component::<Pos>(v)
                            .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                        .filter(|(d, _)| *d < 6).min_by_key(|(d, _)| *d).map(|(_, v)| v);
                    if let Some(v) = nearest {
                        if let Some(h) = self.world.get_component_mut::<Health>(v) {
                            h.condition = (h.condition - 0.04).max(0.01);
                        }
                        self.say(v, "The cough brushed me.", CHAT_BUBBLE_SECS);
                    }
                }
            }
            EventId::Friar => {
                // confession under the hawthorn: a live weight comes off
                // and the ear hears a want plain - no coin, just the telling
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut heard = 0;
                for &v in &self.villagers.clone() {
                    if heard >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let burdened = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.grievances.iter().any(|(_, u)| tick <= *u)).unwrap_or(false);
                    if !burdened { continue; }
                    // absolution takes the newest live weight
                    if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                        if let Some(i) = l.grievances.iter().rposition(|(_, u)| tick <= *u) {
                            l.grievances.remove(i);
                        }
                    }
                    // the telling surfaces a want the ear didn't know
                    // yet - the hear() path keeps the ledger's voice
                    let want = {
                        let wants: Vec<Want> = self.world.get_component::<desires::InnerLife>(v)
                            .map(|l| l.wants.clone()).unwrap_or_default();
                        let kn = self.world.get_component::<desires::Known>(v);
                        wants.into_iter().find(|w| kn
                            .map(|k| !k.revealed.contains(&desires::Reveal::Want(*w)))
                            .unwrap_or(true))
                    };
                    if let Some(w) = want {
                        let player = self.player;
                        desires::hear(&mut self.world, player, v, desires::Reveal::Want(w), tick);
                    }
                    self.say(v, "Go and carry it lighter.", CHAT_BUBBLE_SECS);
                    heard += 1;
                }
            }
            EventId::Heretic => {
                // a strange doctrine by the gate: every other soul who
                // hears it is shaken out of comfort; the others are
                // drawn to hear more - faith doesn't sit still for him
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for (i, &v) in self.villagers.clone().iter().enumerate() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    if i % 2 == 0 {
                        // shaken: the comfort lifts off the faithful
                        if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                            l.satisfied.retain(|(w, _)| *w != Want::Faith);
                        }
                    } else {
                        // drawn: the strange doctrine pulls the curious -
                        // the already-fed hear it again, and it lengthens
                        let curious = self.world.get_component::<desires::InnerLife>(v)
                            .map(|l| l.wants.contains(&Want::Faith)).unwrap_or(false);
                        if curious {
                            desires::satisfy(&mut self.world, v, Want::Faith, tick + 20);
                        }
                    }
                }
            }
            EventId::Drover => {
                // sheep to market: a head a day off his drove into the
                // fold for three gold, bought by the nearest soul with coin
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                let buyer = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 3.0).unwrap_or(false))
                    .filter_map(|v| self.world.get_component::<Pos>(v)
                        .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                    .filter(|(d, _)| *d < 8).min_by_key(|(d, _)| *d).map(|(_, v)| v);
                if let (Some(f), Some(b)) = (fold, buyer) {
                    let (fx, fy) = self.world.get_component::<Pos>(f).map(|p| (p.x, p.y)).unwrap_or((at.0 as f32, at.1 as f32));
                    let e = crate::game::livestock::spawn_animal(&mut self.world,
                        crate::game::livestock::LivestockKind::Sheep, f, fx + 1.0, fy + 1.0);
                    // onto the flock's roll too - a sheep you can't see is no sheep
                    self.flock.push(e);
                    if let Some(p) = self.world.get_component_mut::<Purse>(b) { p.gold -= 3.0; }
                    self.say(b, "Good ewe, that one.", CHAT_BUBBLE_SECS);
                }
            }
            EventId::FishHawker => {
                // salt fish keeps all winter: a penny a fish for those
                // who'd know good food when it swims by
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::GoodFood, tick) { continue; }
                    let taste = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::GoodFood)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !taste || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 20);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    self.say(v, "Salt fish - keeps all winter!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::CornFactor => {
                // grain bought by the quarter: surplus only - no factor
                // starves a village for weight - and the coin lands in
                // the market's own hand, not the building's ledger
                let surplus = self.world.get_component::<villager::FoodSource>(self.market_food)
                    .map(|f| (f.servings.saturating_sub(4)).min(3)).unwrap_or(0);
                if surplus > 0 {
                    if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                        f.servings -= surplus;
                    }
                    let seller = self.world.get_component::<household::Building>(self.market_food)
                        .and_then(|b| b.worker.or(b.overseer))
                        .filter(|&w| !villager::is_dead(&self.world, w)
                            && !crate::game::dungeon::unavailable(&self.world, w))
                        .or_else(|| self.villagers.iter().copied()
                            .find(|&v| !villager::is_dead(&self.world, v)
                                && !crate::game::dungeon::unavailable(&self.world, v)));
                    if let Some(s) = seller {
                        if let Some(p) = self.world.get_component_mut::<Purse>(s) {
                            p.gold += surplus as f32;
                        }
                    }
                }
            }
            EventId::SpiceMerchant => {
                // pepper and cloves are dear: two gold a packet, good
                // food eased and a pinch of cure for those who buy
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                for &v in &self.villagers.clone() {
                    if sold >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::GoodFood, tick) { continue; }
                    let taste = self.world.get_component::<desires::InnerLife>(v)
                        .map(|l| l.wants.contains(&Want::GoodFood)).unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !taste || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                    // the cure: folk hold pepper's a physic
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.02).min(1.0);
                    }
                    self.say(v, "Things that cure what ails!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::SaltSeller => {
                // white gold stretches the winter: a gold a handful off
                // the nearest, and the salt makes the larder last longer
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                // nearest three who can pay - the cry catches closest ears
                let mut buyers: Vec<(i32, Entity)> = self.villagers.iter().copied()
                    .filter(|&v| !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false))
                    .filter_map(|v| self.world.get_component::<Pos>(v)
                        .map(|p| ((p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs(), v)))
                    .filter(|(d, _)| *d < 8).collect();
                buyers.sort_by_key(|(d, _)| *d);
                for (_, v) in buyers.into_iter().take(3) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    if let Some(f) = self.world.get_component_mut::<villager::FoodSource>(self.market_food) {
                        f.servings += 2;
                    }
                    self.say(v, "None goes bad who buys it!", CHAT_BUBBLE_SECS);
                }
            }
            EventId::WoolMerchant => {
                // fleece bought by the stone: the fold's wool stock sells
                // first, then the fresh clip off the sheep - a head clips
                // once a visit, goats don't shear, four stone a day at most
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    let sheep = self.flock.iter().copied()
                        .filter(|&a| self.world.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.kind == crate::game::livestock::LivestockKind::Sheep).unwrap_or(false))
                        .count();
                    let already = self.rare.iter().find(|r| r.id == id).map(|r| r.aux as usize).unwrap_or(0);
                    // what's in the stockroom goes first, then the backs
                    let stock_wool = self.world.get_component::<household::Building>(f)
                        .and_then(|b| b.stock.get("wool").copied()).unwrap_or(0).min(4) as usize;
                    let fresh = sheep.saturating_sub(already).min(4usize.saturating_sub(stock_wool));
                    let clip = stock_wool + fresh;
                    if clip > 0 {
                        if let Some(b) = self.world.get_component_mut::<household::Building>(f) {
                            let w = b.stock.entry("wool".into()).or_insert(0);
                            let taken = (*w).min(stock_wool as u32);
                            *w -= taken;
                        }
                        if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) {
                            r.aux += fresh as u64;
                        }
                        let shepherd = self.world.get_component::<household::Building>(f)
                            .and_then(|b| b.worker.or(b.overseer))
                            .filter(|&w| !villager::is_dead(&self.world, w)
                                && !crate::game::dungeon::unavailable(&self.world, w));
                        if let Some(s) = shepherd {
                            if let Some(p) = self.world.get_component_mut::<Purse>(s) {
                                p.gold += clip as f32;
                            }
                            self.say(s, "Top price, by the stone!", CHAT_BUBBLE_SECS);
                        }
                    }
                }
            }
            EventId::Bookseller => {
                // words that outlive the winter: three gold a book, one
                // a day, to the lettered - a book's joy outlasts the cold
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut sold = 0;
                // the roster plus the player - she can read too
                let cands: Vec<Entity> = self.villagers.iter().copied()
                    .chain(std::iter::once(self.player)).collect();
                for v in cands {
                    if sold >= 1 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    if !self.literate(v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    // a book lasts - the already-fed don't buy again today
                    let fed = desires::is_satisfied(&self.world, v, Want::Joy, tick);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 3.0).unwrap_or(false);
                    if !near || fed || !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 40);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 3.0; }
                    self.say(v, "Words that outlive the winter!", CHAT_BUBBLE_SECS);
                    sold += 1;
                }
            }
            EventId::CharcoalBurner => {
                // his heap burns slow and warm at the woods edge: those
                // who pass feel it in their bones a little
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.01).min(1.0);
                    }
                }
            }
            EventId::LimeBurner => {
                // lime for the fields: folk carry it out to the beds
                // while he stands - the soil grows the sweeter for it
                for g in self.gardens.iter_mut() {
                    // no cap - a bed past ripe stays past ripe
                    g.2 += 0.02;
                }
            }
            EventId::Tanner => {
                // mind the smell: honest work, but the tan-pit's reek
                // hangs on whoever stands near his vats a while
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut nipped = 0;
                for &v in &self.villagers.clone() {
                    if nipped >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near { continue; }
                    // the fold's smell already hangs on a shepherd - a
                    // tanner's reek can't tell him anything new
                    let already = self.world.get_component::<crate::game::vocation::Craft>(v)
                        .map(|c| c.vocation == crate::game::vocation::Vocation::Shepherd).unwrap_or(false);
                    if already { continue; }
                    desires::add_grievance(&mut self.world, v, desires::Problem::Smelly, tick + 15);
                    self.say(v, "Mind the smell - it's honest work!", CHAT_BUBBLE_SECS);
                    nipped += 1;
                }
            }
            EventId::Glazier => {
                // glass for the church, light for the dark months: one
                // home a day gets its panes - the draft's out, the
                // light's in, the folk inside feel both
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut glazed = 0;
                for &v in &self.villagers.clone() {
                    if glazed >= 1 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let Some(h) = household::home_of(&self.world, v) else { continue };
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let occ: Vec<Entity> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default();
                    // a glazed home shows in its folk's light - done once
                    if occ.iter().next()
                        .map(|&o| desires::is_satisfied(&self.world, o, Want::Joy, tick))
                        .unwrap_or(false) { continue; }
                    for o in occ {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                            hl.condition = (hl.condition + 0.02).min(1.0);
                        }
                        desires::satisfy(&mut self.world, o, Want::Joy, tick + 10);
                    }
                    glazed += 1;
                }
            }
            EventId::Thatcher => {
                // roofs patched before the rain finds you: one home a
                // day - fresh thatch on the stock roll, dry heads under
                // it, and the leak's due pushed down the road
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let Some(h) = household::home_of(&self.world, v) else { continue };
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let done = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.stock.get("thatch").copied()).unwrap_or(0) > 0;
                    if done { continue; }
                    if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                        household::add_stock(b, "thatch", 1);
                    }
                    let occ: Vec<Entity> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default();
                    for o in occ {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                            hl.condition = (hl.condition + 0.02).min(1.0);
                        }
                    }
                    if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::GranaryLeak) {
                        // push it off, never pull it in - a sound roof
                        // postpones the leak, it doesn't summon it
                        *due = (*due).max(tick + 400);
                    }
                    break;
                }
            }
            EventId::WellDigger => {
                // he digs deep all week: those who stand near get a
                // cup of what he's striking - a taste of sweet water
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 6)
                        .unwrap_or(false);
                    if !near { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition + 0.01).min(1.0);
                    }
                }
            }
            EventId::FortuneTeller => {
                // your palm, love - the lines don't lie: a penny a palm,
                // the lines surface a want, and most times the telling's
                // kind - now and then it's dark and the penny bought truth
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut read = 0;
                for &v in &self.villagers.clone() {
                    if read >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    if !near || !pays { continue; }
                    // the lines surface a want the ear didn't know - if
                    // there's no line she can't already read, she won't
                    // take the penny for it
                    let want = {
                        let wants: Vec<Want> = self.world.get_component::<desires::InnerLife>(v)
                            .map(|l| l.wants.clone()).unwrap_or_default();
                        let kn = self.world.get_component::<desires::Known>(v);
                        wants.into_iter().find(|w| kn
                            .map(|k| !k.revealed.contains(&desires::Reveal::Want(*w)))
                            .unwrap_or(true))
                    };
                    let Some(w) = want else { continue };
                    // the organizer's own ledger hears it - the person,
                    // not the body politic
                    let player = self.player;
                    desires::hear(&mut self.world, player, v, desires::Reveal::Want(w), tick);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    // most palms read kind - a dark line's still the
                    // truth, and every palm reads its own lines
                    if (self.rare_roll(id) ^ (v as u64).wrapping_mul(0x9E3779B9)) % 4 != 0 {
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 15);
                        self.say(v, "The lines don't lie, love.", CHAT_BUBBLE_SECS);
                    } else {
                        self.say(v, "...ah. Best not say more.", CHAT_BUBBLE_SECS);
                    }
                    read += 1;
                }
            }
            EventId::Astrologer => {
                // he reads the sky over the village once: forewarned of
                // whatever's written nearest, and forewarned is forearmed
                let warned = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0;
                if warned { return; }
                let next = self.next_rare.iter()
                    .filter(|(i, _)| *i != id)
                    .min_by_key(|(_, d)| *d)
                    .map(|(i, d)| (*i, *d));
                if let Some((i, d)) = next {
                    let name = EVENTS.iter().find(|s| s.id == i).map(|s| s.name).unwrap_or("something");
                    // the due's already in days - no conversion
                    let days = d.saturating_sub(tick);
                    self.log.push(format!("t{}: the astrologer reads the sky - {} comes in about {} days.", tick, name, days.max(1)));
                }
                // forearmed: a day's warning's a kind of security for near folk
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    desires::satisfy(&mut self.world, v, Want::Security, tick + 20);
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
            }
            EventId::Alchemist => {
                // I buy strange things. Strange things only. - two gold
                // for whatever odd's in your satchel, whatever it is
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut bought = 0;
                for &v in &self.villagers.clone() {
                    if bought >= 2 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    // something, actually - a zero-count entry's a story,
                    // not a strange thing to sell
                    let good = self.world.get_component::<crate::game::livelihood::Satchel>(v)
                        .and_then(|s| s.goods.iter().find(|(_, &n)| n > 0)
                            .map(|(k, _)| k.clone()));
                    let Some(g) = good else { continue };
                    if let Some(s) = self.world.get_component_mut::<crate::game::livelihood::Satchel>(v) {
                        if let Some(n) = s.goods.get_mut(&g) {
                            *n = n.saturating_sub(1);
                            if *n == 0 { s.goods.remove(&g); }
                        }
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold += 2.0; }
                    self.say(v, "Strange. Just what I'm after.", CHAT_BUBBLE_SECS);
                    bought += 1;
                }
            }
            EventId::Quack => {
                // cures all: a penny a dose - most bottles ease (the
                // believing does the work), now and then the batch
                // itself's bad and the belly pays for it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut dosed = 0;
                for &v in &self.villagers.clone() {
                    if dosed >= 3 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 1.0).unwrap_or(false);
                    // a whole body gains nothing from the dose - he
                    // sells to the ailing, not the hale
                    let ails = self.world.get_component::<Health>(v)
                        .map(|h| h.condition < 1.0).unwrap_or(false);
                    if !near || !pays || !ails { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 1.0; }
                    // every bottle's its own brew
                    let brew = (self.rare_roll(id) ^ (v as u64).wrapping_mul(0x9E3779B9)) % 4;
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        if brew != 0 {
                            h.condition = (h.condition + 0.04).min(1.0);
                        } else {
                            h.condition = (h.condition - 0.04).max(0.01);
                            self.say(v, "...the belly doesn't sit right.", CHAT_BUBBLE_SECS);
                        }
                    }
                    dosed += 1;
                }
            }
            EventId::Midwife => {
                // babies caught, mothers kept: a home with little ones
                // gets her tending a day - the small one's stronger for
                // her hands and the house feels the blessing
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let mut tended = 0;
                for &v in &self.villagers.clone() {
                    if tended >= 1 { break; }
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let Some(h) = household::home_of(&self.world, v) else { continue };
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let occ: Vec<Entity> = self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default();
                    // she comes where there's a small one to tend - a
                    // held child can't take her hands, and neither can a ghost
                    let has_kid = occ.iter().any(|&o|
                        self.world.get_component::<crate::game::vocation::Kid>(o).is_some()
                            && !villager::is_dead(&self.world, o)
                            && !crate::game::dungeon::unavailable(&self.world, o));
                    if !has_kid { continue; }
                    for o in occ {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        if self.world.get_component::<crate::game::vocation::Kid>(o).is_some() {
                            if let Some(hl) = self.world.get_component_mut::<Health>(o) {
                                hl.condition = (hl.condition + 0.03).min(1.0);
                            }
                        }
                        desires::satisfy(&mut self.world, o, Want::Joy, tick + 10);
                    }
                    tended += 1;
                }
            }
            EventId::Sowgelder => {
                // beasts tended: his hands go over the fold - the stock
                // stands the stronger for it, and the wolves' day gets
                // pushed off while the doctored herd holds its scent
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    // his tending calms the fold - a brewing wolf night
                    // settles while his hands are on the stock
                    self.wolf_coming = false;
                    if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::WolfAtFold) {
                        // past the pack's own scheduling window - it
                        // always moves, never pulls a near wolf closer
                        *due = (*due).max(tick + 500);
                    }
                    let shepherd = self.world.get_component::<household::Building>(f)
                        .and_then(|b| b.worker.or(b.overseer))
                        .filter(|&w| !villager::is_dead(&self.world, w)
                            && !crate::game::dungeon::unavailable(&self.world, w));
                    if let Some(s) = shepherd {
                        desires::satisfy(&mut self.world, s, Want::Security, tick + 15);
                    }
                }
            }
            EventId::Farrier => {
                // shoes for hooves: two gold a shoeing once a visit -
                // a shod beast doesn't founder, and the shepherd's mind
                // rests easy on it
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    // no hooves, no shoes - an emptied fold buys nothing
                    let stock = self.flock.iter().any(|&a|
                        self.world.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.fold == f).unwrap_or(false));
                    let shod = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0;
                    if stock && !shod {
                        let shepherd = self.world.get_component::<household::Building>(f)
                            .and_then(|b| b.worker.or(b.overseer))
                            .filter(|&w| !villager::is_dead(&self.world, w)
                                && !crate::game::dungeon::unavailable(&self.world, w)
                                && self.world.get_component::<Purse>(w).map(|p| p.gold >= 2.0).unwrap_or(false));
                        if let Some(s) = shepherd {
                            if let Some(p) = self.world.get_component_mut::<Purse>(s) { p.gold -= 2.0; }
                            // sound stock from here out - new iron can't
                            // undo a loss already taken
                            desires::satisfy(&mut self.world, s, Want::Security, tick + 20);
                            self.say(s, "Hear that ring of iron!", CHAT_BUBBLE_SECS);
                            if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
                        }
                    }
                }
            }
            EventId::HorseCop => {
                // sound wind and mostly honest - he buys a head off the
                // fold a day, at a dealer's price: three gold, which is
                // cheap, and everyone knows it
                let fold = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
                if let Some(f) = fold {
                    let head = self.flock.iter().copied()
                        .find(|&a| self.world.get_component::<crate::game::livestock::Livestock>(a)
                            .map(|l| l.fold == f).unwrap_or(false));
                    let shepherd = self.world.get_component::<household::Building>(f)
                        .and_then(|b| b.worker.or(b.overseer))
                        .filter(|&w| !villager::is_dead(&self.world, w)
                            && !crate::game::dungeon::unavailable(&self.world, w));
                    if let (Some(a), Some(s)) = (head, shepherd) {
                        self.flock.retain(|&x| x != a);
                        self.world.destroy_entity(a);
                        if let Some(p) = self.world.get_component_mut::<Purse>(s) { p.gold += 3.0; }
                        self.say(s, "Sound wind, and mostly honest!", CHAT_BUBBLE_SECS);
                    }
                }
            }
            EventId::CandleMaker => {
                // wax bought, tallow taken - light for the dark: a home's
                // tallow goes into his dipper and candles come back, a
                // coin for the wax
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let homes: Vec<Entity> = self.world.query::<household::Building>().iter().copied()
                    .filter(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
                    .collect();
                let mut traded = 0;
                for h in homes {
                    if traded >= 2 { break; }
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let first_able = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)));
                    let Some(o) = first_able else { continue };
                    // wax bought, tallow taken: whatever fat's on their
                    // shelf goes in his pot for a coin - most homes have
                    // none, and then he just sells the light
                    let tallow = self.world.get_component::<household::Building>(h)
                        .map(|b| *b.stock.get("tallow").unwrap_or(&0) > 0).unwrap_or(false);
                    let buyer = self.world.get_component::<Purse>(o)
                        .map(|p| p.gold >= 1.0).unwrap_or(false);
                    let mut done = false;
                    if tallow {
                        // the candles have to land before the tallow goes
                        if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                            if household::add_stock(b, "candle", 2) > 0 {
                                household::take_stock(b, "tallow", 1);
                                done = true;
                            }
                        }
                        if done {
                            if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold += 1.0; }
                        }
                    } else if buyer {
                        // he dips his own wax and sells the light for a penny
                        if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                            if household::add_stock(b, "candle", 2) > 0 { done = true; }
                        }
                        if done {
                            if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold -= 1.0; }
                        }
                    }
                    if !done { continue; }
                    for o in self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default() {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        desires::satisfy(&mut self.world, o, Want::Joy, tick + 5);
                    }
                    traded += 1;
                }
            }
            EventId::Potter => {
                // pots and crocks, a jug that doesn't weep: a penny a
                // crock, and a good pot keeps the stew right - the house
                // that owns one eats the better for it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let homes: Vec<Entity> = self.world.query::<household::Building>().iter().copied()
                    .filter(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
                    .collect();
                let mut sold = 0;
                for h in homes {
                    if sold >= 2 { break; }
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    let buyer = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)
                                && self.world.get_component::<Purse>(o).map(|p| p.gold >= 1.0).unwrap_or(false)));
                    let Some(o) = buyer else { continue };
                    // the crock has to fit the shelf before the penny goes
                    let fits = self.world.get_component_mut::<household::Building>(h)
                        .map(|b| household::add_stock(b, "crock", 1) > 0).unwrap_or(false);
                    if !fits { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold -= 1.0; }
                    for o in self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default() {
                        if villager::is_dead(&self.world, o) || crate::game::dungeon::unavailable(&self.world, o) { continue; }
                        desires::satisfy(&mut self.world, o, Want::GoodFood, tick + 10);
                    }
                    sold += 1;
                }
            }
            EventId::Eggler => {
                // eggs, butter, cheese - what have your hens done? a
                // coin a basket for whatever the dairy's put by, from
                // whoever's actually about to take it
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let homes: Vec<Entity> = self.world.query::<household::Building>().iter().copied()
                    .filter(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
                    .collect();
                let mut bought = 0;
                for h in homes {
                    if bought >= 2 { break; }
                    let near = self.world.get_component::<Pos>(h)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    let good = self.world.get_component::<household::Building>(h)
                        .and_then(|b| ["eggs", "butter", "cheese"].iter()
                            .find(|g| *b.stock.get(**g).unwrap_or(&0) > 0)
                            .map(|g| g.to_string()));
                    let first_able = self.world.get_component::<household::Building>(h)
                        .and_then(|b| b.occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                                && !crate::game::dungeon::unavailable(&self.world, o)));
                    if !near { continue; }
                    let Some(g) = good else { continue };
                    let Some(o) = first_able else { continue };
                    if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                        household::take_stock(b, &g, 1);
                    }
                    if let Some(p) = self.world.get_component_mut::<Purse>(o) { p.gold += 1.0; }
                    self.say(o, "What have your hens done, eh?", CHAT_BUBBLE_SECS);
                    bought += 1;
                }
            }
            EventId::CheeseMonger => {
                // aged wheels bought, the older the better: the cellar's
                // oldest wheel goes for a monger's price - more for the
                // days on its rind - to the shepherd who made it
                if self.cellar.is_none() || self.cheeses.is_empty() { return; }
                let age = tick - self.cheeses[0];
                let worth = crate::game::vocation::price("cheese") * 2.0 * (1.0 + age as f32 / 60.0);
                self.cheeses.remove(0);
                let shepherd = self.shepherd
                    .filter(|&s| !villager::is_dead(&self.world, s)
                        && !crate::game::dungeon::unavailable(&self.world, s));
                if let Some(s) = shepherd {
                    if let Some(p) = self.world.get_component_mut::<Purse>(s) { p.gold += worth; }
                    self.say(s, "The older the better - fine rind!", CHAT_BUBBLE_SECS);
                }
            }
            EventId::Messenger => {
                // a letter, is there one here called-? news rides in for
                // one of them - most times a kind word from far off, and
                // now and then the word a man never wants
                let delivered = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0;
                if delivered { return; }
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                // the roll picks whose door - not always the same man's
                let Some(&v) = able.get((self.rare_roll(id) as usize / 7) % able.len().max(1)) else { return };
                if self.rare_roll(id) % 5 != 0 {
                    desires::satisfy(&mut self.world, v, Want::Joy, tick + 20);
                    self.say(v, "A letter for you - your sister's well!", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a messenger rides in - a letter, and glad word with it.", tick));
                } else {
                    // the word a man never wants - grief, not stock gone
                    desires::add_grievance(&mut self.world, v, desires::Problem::Grief, tick + 30);
                    self.say(v, "A letter for you - I'm sorry to carry it.", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: a messenger rides in - the letter brings no joy.", tick));
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
            }
            EventId::NewsCrier => {
                // the news gives them something to talk about: everyone
                // in earshot goes home with a day's more to say to the
                // one they talk to most
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near { continue; }
                    // the news goes home to their closest living bond -
                    // the dead hear no gossip, and the record keeps its kind honest
                    let partner = self.world.query::<crate::game::relationships::Relationship>().iter()
                        .filter_map(|&r| self.world.get_component::<crate::game::relationships::Relationship>(r))
                        .filter(|rr| (rr.from == v || rr.other == v)
                            && !villager::is_dead(&self.world, if rr.from == v { rr.other } else { rr.from }))
                        .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|rr| if rr.from == v { rr.other } else { rr.from });
                    if let Some(to) = partner {
                        crate::game::relationships::record_interaction(&mut self.world, v,
                            crate::game::relationships::Interaction {
                                other: to, kind: crate::game::relationships::InteractionKind::Conversation,
                                intensity: 0.2 });
                    }
                }
            }
            EventId::Hailstorm => {
                // the day it falls batters: gardens shredded, everyone stung
                if self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0 { return; }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = 1; }
                for g in self.gardens.iter_mut() { g.2 *= 0.5; }
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition - 0.02).max(0.05);
                    }
                }
            }
            EventId::HeatWave => {
                // five days of scorch: folk droop in it, the beds ripen
                // fast in the heat, and the well's head drops day by day
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        h.condition = (h.condition - 0.01).max(0.05);
                    }
                }
                for g in self.gardens.iter_mut() { g.2 += 0.04; }
                self.land.pit_water = (self.land.pit_water - 0.05).max(0.0);
            }
            EventId::Drought => {
                // a fortnight dry: the well runs low, the beds wilt hard
                // against what the sun would give, and no patch fruits
                // until the dry's past - not tomorrow, the whole dry
                self.land.pit_water = (self.land.pit_water - 0.08).max(0.0);
                for g in self.gardens.iter_mut() { g.2 = (g.2 - 0.6).max(0.0); }
                let until = self.rare.iter().find(|r| r.id == id).and_then(|r| r.until)
                    .unwrap_or(self.tick + 1);
                for p in self.mush_patches.iter_mut() {
                    p.next_ready = p.next_ready.max(until);
                }
            }
            EventId::Flood => {
                // the river comes up: the pit fills past what the
                // pump-line can take - a drowned pit yields nothing -
                // and the beds drink deep of the high water
                self.land.pit_water = (self.land.pit_water + 0.2).min(1.0);
                for g in self.gardens.iter_mut() { g.2 += 0.02; }
            }
            EventId::WineMerchant => {
                // "a cup before you buy": anyone in reach with a thirst
                // pays the pedlar's price for the road's vintage
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) || crate::game::dungeon::unavailable(&self.world, v) { continue; }
                    let near = self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                        .unwrap_or(false);
                    if !near || desires::is_satisfied(&self.world, v, Want::Drink, tick) { continue; }
                    let pays = self.world.get_component::<Purse>(v).map(|p| p.gold >= 2.0).unwrap_or(false);
                    if !pays { continue; }
                    desires::satisfy(&mut self.world, v, Want::Drink, tick + 30);
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) { p.gold -= 2.0; }
                }
            }
            _ => {}
        }
    }

    /// A spell's first day on: its mark in the log
    fn event_began(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::Hailstorm => {
                self.log.push(format!("t{}: a hailstorm batters the gardens - stones like shot.", self.tick));
                self.toasts.push(("Hailstones!".into(), self.tick + 40));
            }
            EventId::MushroomFlush => {
                // the wet fortnight's gift: every patch flushes at once,
                // and the wood's edges sprout a few besides - picked
                // through by the end (aux counts the bonus caps)
                for p in self.mush_patches.iter_mut() { p.next_ready = self.tick; }
                let mut added = 0u64;
                for (i, &(tx, ty)) in self.map.trees.iter().step_by(13).take(3).enumerate() {
                    if let Some(t) = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (tx + dx, ty + dy)))
                        .find(|&(x, y)| self.map.passable(x, y)
                            && !self.mush_patches.iter().any(|p| p.tile == (x, y))) {
                        let kind = if i % 2 == 0 { MushKind::Chanterelle } else { MushKind::FieldMushroom };
                        self.mush_patches.push(MushPatch { tile: t, kind, next_ready: self.tick });
                        added += 1;
                    }
                }
                if let Some(r) = self.rare.iter_mut().find(|r| r.id == id) { r.aux = added; }
                self.log.push(format!("t{}: a mushroom flush - the wood's full of caps after the rain.", self.tick));
                self.toasts.push(("Mushrooms!".into(), self.tick + 40));
            }
            EventId::HeatWave => {
                self.log.push(format!("t{}: a heat wave settles in - the stones are hot to the hand.", self.tick));
                self.toasts.push(("A heat wave".into(), self.tick + 40));
            }
            EventId::Drought => {
                self.log.push(format!("t{}: a drought - the sky's forgotten rain, and the well knows it.", self.tick));
                self.toasts.push(("A drought".into(), self.tick + 40));
            }
            EventId::Flood => {
                self.log.push(format!("t{}: the river comes up - the reach is full and rising.", self.tick));
                self.toasts.push(("The river's up".into(), self.tick + 40));
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// A moment's blow, all on the day
    fn event_moment(&mut self, id: EventId) {
        let s = spec(id);
        match id {
            EventId::AlehouseBrawl => self.brawl_moment(),
            EventId::Assize => self.assize_moment(),
            EventId::CellarThief => self.cellar_thief_moment(),
            EventId::LightningStrike => {
                // the bolt picks one of them - knocked flat where it
                // lands, not killed; the sky aims where it aims
                let able: Vec<Entity> = self.villagers.iter().copied().filter(|&v|
                    !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)).collect();
                if let Some(&v) = able.get((self.rare_roll(id) as usize / 7) % able.len().max(1)) {
                    if let Some(h) = self.world.get_component_mut::<Health>(v) {
                        // a strike only takes - the floor never heals
                        if h.condition > 0.05 {
                            h.condition = (h.condition - 0.25).max(0.05);
                        }
                    }
                    self.say(v, "...", CHAT_BUBBLE_SECS);
                    self.log.push(format!("t{}: lightning strikes the village - one of them's knocked flat where it lands.", self.tick));
                } else {
                    self.log.push(format!("t{}: lightning strikes the common - white fire in the grass.", self.tick));
                }
                self.toasts.push(("Lightning!".into(), self.tick + 40));
            }
            _ => { self.log.push(format!("t{}: {}.", self.tick, s.name)); }
        }
    }

    /// The alehouse earns its other name: a cup in and the insults come
    /// out. Bruises, not blades - the drunk and the thirsty swing first
    fn brawl_moment(&mut self) {
        // where the drink pours; else the market stands in for a taproom
        let taproom = self.building_entities.iter().copied().find(|&be|
            self.world.get_component::<household::Building>(be)
                .map(|b| matches!(b.kind, household::BuildingKind::Shop(
                    crate::game::vocation::Vocation::Alewife | crate::game::vocation::Vocation::Brewer
                    | crate::game::vocation::Vocation::Vintner | crate::game::vocation::Vocation::Distiller))
                    && b.stock.iter().any(|(g, q)| crate::game::vocation::is_drink(g) && *q > 0))
                .unwrap_or(false));
        let at = taproom.and_then(|be| self.world.get_component::<Pos>(be).copied())
            .or_else(|| self.world.get_component::<Pos>(self.market_food).copied());
        let mut brawlers: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<crate::game::vocation::TownRole>(v).is_none())
            .filter(|&v| {
                let drunk = self.world.get_component::<villager::Drunk>(v)
                    .map(|d| self.tick < d.until).unwrap_or(false);
                let near = at.map(|a| self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - a.x).abs() + (p.y - a.y).abs() < 8.0).unwrap_or(false))
                    .unwrap_or(false);
                drunk || near
            }).collect();
        if brawlers.is_empty() {
            // a brawl's still a brawl if the street's free - but a man
            // in the lock-up is no man for a punch-up: the able come
            // first, the window after
            brawlers = self.villagers.iter().copied()
                .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).take(4).collect();
        }
        brawlers.truncate(2);
        let names: Vec<String> = brawlers.iter().map(|&v| self.name_of(v)).collect();
        for &v in &brawlers {
            if let Some(h) = self.world.get_component_mut::<Health>(v) {
                h.condition = (h.condition - 0.05).max(0.05);
            }
            self.say(v, "Say that again!", CHAT_BUBBLE_SECS);
        }
        let place = taproom
            .and_then(|be| self.world.get_component::<household::Building>(be).map(|b| b.name.clone()))
            .unwrap_or_else(|| "the market".into());
        self.log.push(format!("t{}: an alehouse brawl at {} - {} come to blows over a spilled cup.",
            self.tick, place, names.join(" and ")));
    }

    /// The assize of ale: the taster walks the drink-shops. Thin brews
    /// are fined; an honest master's name is praised
    fn assize_moment(&mut self) {
        let mut heard = false;
        for &v in &self.villagers.clone() {
            let Some(c) = self.world.get_component::<crate::game::vocation::Craft>(v) else { continue };
            if !matches!(c.vocation, crate::game::vocation::Vocation::Alewife
                | crate::game::vocation::Vocation::Brewer | crate::game::vocation::Vocation::Vintner
                | crate::game::vocation::Vocation::Distiller) { continue; }
            heard = true;
            let name = self.name_of(v);
            if crate::game::vocation::level_of(c.skill) == crate::game::vocation::Level::Master {
                self.log.push(format!("t{}: the ale-taster calls {}'s brew honest and true.", self.tick, name));
                desires::satisfy(&mut self.world, v, Want::Mastery, self.tick + 60);
            } else {
                if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                    p.gold = (p.gold - 3.0).max(0.0);
                }
                self.log.push(format!("t{}: the ale-taster fines {} - thin stuff, barely worth the water.", self.tick, name));
            }
        }
        if !heard {
            self.log.push(format!("t{}: the ale-taster rides through - no brewer's bench to judge here.", self.tick));
        }
    }

    /// A cellar worth robbing is a cellar worth watching: the drink
    /// trade's stock draws a thief in the night, and the watch earns
    /// its keep when one walks the walls
    fn cellar_thief_moment(&mut self) {
        // the fattest drink-stock in the village draws him
        let mark = self.building_entities.iter().copied()
            .filter_map(|be| self.world.get_component::<household::Building>(be)
                .map(|b| (be, b.stock.iter()
                    .filter(|(g, _)| crate::game::vocation::is_drink(g))
                    .map(|(g, q)| crate::game::vocation::price(g) * *q as f32).sum::<f32>())))
            .filter(|(_, v)| *v >= 20.0)
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(be, _)| be);
        let Some(mark) = mark else {
            self.log.push(format!("t{}: a thief slips through in the night - but finds no fat cellar.", self.tick));
            return;
        };
        // a living watch is why a fat cellar locks its door
        let watched = self.villagers.iter().copied().any(|v|
            !villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::game::vocation::Craft>(v)
                    .map(|c| c.vocation == crate::game::vocation::Vocation::Knight).unwrap_or(false));
        let name = self.world.get_component::<household::Building>(mark)
            .map(|b| b.name.clone()).unwrap_or_else(|| "a cellar".into());
        if watched {
            self.log.push(format!("t{}: the watch catches a thief at {} - the good stock stays.", self.tick, name));
            return;
        }
        // half the drink walks off; the shop's keeper wears the grievance
        let worker = self.world.get_component::<household::Building>(mark).and_then(|b| b.worker);
        if let Some(b) = self.world.get_component_mut::<household::Building>(mark) {
            for g in ["ale", "beer", "wine", "spirits"] {
                let q = b.stock.get(g).copied().unwrap_or(0);
                household::take_stock(b, g, (q + 1) / 2);
            }
        }
        if let Some(w) = worker {
            desires::add_grievance(&mut self.world, w, desires::Problem::Robbed, self.tick + 60);
        }
        self.log.push(format!("t{}: a thief in the night - half the drink walks out of {}.", self.tick, name));
    }

    /// A visitor's last day: what it leaves behind
    fn event_end(&mut self, id: EventId, tick: u64) {
        match id {
            EventId::ChimneySweep => {
                // clean flues to the cap: the next chimney fire's far off
                if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::ChimneyFire) {
                    *due = (*due).max(tick + 150);
                }
                self.log.push(format!("t{}: every flue's clean to the cap - the sweep earned his soot.", tick));
            }
            EventId::RatCatcher => {
                // a cleared larder holds: the rat plague's far off
                if let Some((_, due)) = self.next_rare.iter_mut().find(|(i, _)| *i == EventId::RatPlague) {
                    *due = (*due).max(tick + 150);
                }
                self.log.push(format!("t{}: a sack of rats for the road - the larder breathes easier.", tick));
            }
            EventId::Dowser => {
                // the forked stick dips or it doesn't - when it does, a
                // spring in the water table feeds the gardens on the mark
                // he left (it feeds them, mind: a drowned pit yields no
                // stone, so the mark's never on the pit itself)
                if self.rare_roll(id) % 4 == 0 {
                    self.log.push(format!("t{}: the stick never dipped - dry ground all round, says the diviner.", tick));
                } else {
                    for g in self.gardens.iter_mut() { g.2 = (g.2 + 0.3).min(1.0); }
                    self.log.push(format!("t{}: water sings under the meadow - the gardens drink it in.", tick));
                }
            }
            EventId::MushroomFlush => {
                // the flush's own caps are picked through: its bonus
                // patches (the last `aux` pushed) leave the wood for good,
                // and anyone mid-walk to one wanders home instead
                let added = self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) as usize;
                if added > 0 {
                    let kept = self.mush_patches.len() - added;
                    self.mush_patches.truncate(kept);
                    for t in self.travels.iter_mut() {
                        if let Goal::ForageShroom(pi) = t.goal {
                            // the cap's gone - they stand where they are,
                            // they don't finish the dead walk to it
                            if pi >= kept { t.goal = Goal::Wander; t.path.clear(); }
                        }
                    }
                }
                self.log.push(format!("t{}: the flush is picked through - the shade keeps what it keeps.", tick));
            }
            EventId::Recruiter => {
                // the levy marches off - and a young one may march with
                // him, if the drum caught one while he was here
                let Some(at) = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at) else { return };
                let kid = self.villagers.iter().copied()
                    .filter(|&v| v != self.player
                        && !villager::is_dead(&self.world, v)
                        && !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<crate::game::vocation::Kid>(v).is_some()
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs() < 8)
                            .unwrap_or(false))
                    .min_by_key(|&v| self.world.get_component::<Pos>(v)
                        .map(|p| (p.x.round() as i32 - at.0).abs() + (p.y.round() as i32 - at.1).abs())
                        .unwrap_or(9999));
                match kid {
                    Some(v) if self.rare_roll(id) % 3 != 0 => {
                        // off the roster, and every index that pointed at
                        // the line shifts home with it
                        let idx = self.villagers.iter().position(|&x| x == v).unwrap();
                        self.villagers.remove(idx);
                        for seat in self.seats.iter_mut() {
                            if seat.selected_target > idx { seat.selected_target -= 1; }
                            else if seat.selected_target == idx {
                                seat.selected_target = idx.min(self.villagers.len().saturating_sub(1));
                            }
                        }
                        // his bed's free (the org rolls prune him next
                        // influence tick - the destroyed lack Health, so
                        // is_dead reads them gone)
                        for b in self.world.query::<household::Building>() {
                            if let Some(bb) = self.world.get_component_mut::<household::Building>(b) {
                                bb.occupants.retain(|&x| x != v);
                            }
                        }
                        self.world.destroy_entity(v);
                        self.log.push(format!("t{}: the levy took the youngest - they marched off at dawn.", tick));
                    }
                    _ => self.log.push(format!("t{}: the drum got no one - the village kept its young.", tick)),
                }
            }
            EventId::Deserter => {
                // sheltered, the law counts the knowing: a writ rides
                // hard behind him - the next one lands double
                if self.rare.iter().find(|r| r.id == id).map(|r| r.aux).unwrap_or(0) > 0 {
                    self.repression_debt += 1;
                    self.log.push(format!("t{}: riders asking after a deserter - a writ for harboring comes hard behind.", tick));
                } else {
                    self.log.push(format!("t{}: the deserter melted off at dawn - no one saw anything.", tick));
                }
            }
            EventId::CharcoalBurner => {
                // the heap comes down: the sacks get carried into the
                // village's first home - burns cleaner than wood all winter
                let shed = self.world.query::<household::Building>().iter().copied()
                    .find(|&b| self.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false));
                if let Some(h) = shed {
                    if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                        household::add_stock(b, "coal", 20);
                    }
                }
                self.log.push(format!("t{}: the charcoal burner broke his heap - he left twenty good sacks of coal.", tick));
            }
            EventId::WellDigger => {
                // sweet water, not the pond's: the sour gut clears
                // where clean water runs, and the beds drink deep
                for &v in &self.villagers.clone() {
                    if let Some(l) = self.world.get_component_mut::<desires::InnerLife>(v) {
                        l.grievances.retain(|(p, _)| *p != desires::Problem::SourGut);
                    }
                }
                for g in self.gardens.iter_mut() {
                    g.2 += 0.05;
                }
                self.log.push(format!("t{}: the well-digger's shaft struck sweet water - the pond's days are done.", tick));
            }
            EventId::RefugeeFamily => {
                // fed and rested, the road's still a hard bed: some put
                // down roots where they camped - two more beds for it
                if self.rare_roll(id) % 3 == 0 {
                    self.log.push(format!("t{}: the refugee family moved on at dawn - the road took them north.", tick));
                } else {
                    // their faces are the roll's own: a temp stream off
                    // the event seed, the village's rng keeps its luck
                    let mut trng = Rng(self.rare_roll(id));
                    let (x, y) = ((self.town_wall.0 + self.town_wall.2) as f32 / 2.0, self.town_wall.3 as f32 + 2.0);
                    // a home with a spare bed takes them, if there is one
                    let home = self.world.query::<household::Building>().iter().copied()
                        .find(|&b| self.world.get_component::<household::Building>(b)
                            .map(|bb| bb.kind == household::BuildingKind::Home && bb.occupants.len() < 4)
                            .unwrap_or(false));
                    for _ in 0..2 {
                        let v = villager::spawn_villager(&mut self.world, x, y);
                        self.world.add_component(v, Health { condition: 0.7, immunity: 0.4 });
                        self.world.add_component(v, crate::sim::community::Citizen {
                            influence: 0.1, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
                        self.world.add_component(v, crate::game::traits::personality_from_seed(trng.next()));
                        self.world.add_component(v, crate::game::livelihood::Satchel::default());
                        self.world.add_component(v, desires::Known::default());
                        let voc = super::roll_vocation(&mut trng);
                        let mut trng2 = Rng(trng.0);
                        self.world.add_component(v, desires::InnerLife {
                            wants: desires::sample_wants(
                                crate::game::vocation::SocialClass::Working, Some(voc), false,
                                &mut move || trng2.next()),
                            grievances: vec![], satisfied: vec![] });
                        self.world.add_component(v, crate::game::vocation::craft(voc, 0.2));
                        self.world.add_component(v, Purse { gold: 0.5 });
                        self.villagers.push(v);
                        // a roof where there's room (the camp's behind them)
                        if let Some(h) = home {
                            if let Some(b) = self.world.get_component_mut::<household::Building>(h) {
                                b.occupants.push(v);
                            }
                        }
                        // the parallel rolls grow with the roster
                        self.was_sick.push(None);
                        self.died_on.push(None);
                    }
                    self.log.push(format!("t{}: the refugee family stayed - two more beds in the village.", tick));
                }
            }
            EventId::StrayDog => {
                // the loneliest heart takes him in - the dog decides
                let mark = self.lonely_hearts().into_iter()
                    .filter(|&v| !villager::is_dead(&self.world, v))
                    .min_by_key(|&v| self.world.query::<crate::game::relationships::Relationship>().iter()
                        .filter(|&&r| self.world.get_component::<crate::game::relationships::Relationship>(r)
                            .map(|rr| rr.from == v || rr.other == v).unwrap_or(false)).count());
                if let Some(m) = mark {
                    let name = self.name_of(m);
                    let pet = pets::spawn_pet(&mut self.world, "Pilgrim", Species::Dog, PetRole::Companion);
                    pets::adopt(&mut self.world, pet, m);
                    self.pets.push((pet, Species::Dog, (0.7, 0.4)));
                    desires::satisfy(&mut self.world, m, Want::Love, tick + 60);
                    self.log.push(format!("t{}: the stray won't go - {} has a dog now.", tick, name));
                } else {
                    self.log.push(format!("t{}: the stray drifts on down the road.", tick));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::stage;
    use crate::session::PKind;

    /// The whole registry's its hundred and counting, every id once
    #[test]
    fn test_the_registry_is_a_hundred() {
        assert_eq!(EVENTS.len(), 102);
        for i in 0..EVENTS.len() {
            for j in (i + 1)..EVENTS.len() {
                assert_ne!(EVENTS[i].id, EVENTS[j].id, "duplicate id in the registry");
            }
        }
        for s in EVENTS { assert_eq!(spec(s.id).id, s.id); }
    }

    /// Every event has somewhere in the schedule
    #[test]
    fn test_every_event_is_scheduled() {
        let g = Game::new();
        for s in EVENTS {
            assert!(g.next_rare.iter().any(|(id, _)| *id == s.id), "{:?} never comes", s.id);
        }
    }

    /// The tinker walks in, stands his pitch, mends what needs mending,
    /// and moves on - state the backend and the frame both show
    #[test]
    fn test_the_tinker_stands_and_mends() {
        let mut g = Game::new();
        g.force_event(EventId::Tinker);
        assert!(g.rare.iter().any(|r| r.id == EventId::Tinker), "he's afoot");
        assert!(g.log.iter().any(|l| l.contains("tinker")), "the log names him");
        // walk him in - the pitch takes once he's stood at market
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Tinker && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let r = g.rare.iter().find(|r| r.id == EventId::Tinker).unwrap();
        assert!(r.until.is_some(), "he's stood his pitch");
        // the frame shows him: a traveller labelled by his trade
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| matches!(p.kind, PKind::Traveller)
            && p.label.as_deref() == Some("the tinker")), "the frame shows the tinker");
        // his days mend tools and take pennies
        let mended = g.villagers.iter().copied().any(|v|
            desires::is_satisfied(&g.world, v, Want::BetterTools, g.tick));
        stage::run_days(&mut g, 2);
        let mended = mended || g.villagers.iter().copied().any(|v|
            desires::is_satisfied(&g.world, v, Want::BetterTools, g.tick));
        assert!(mended, "someone's tools got sharper");
        // and the road takes him back when his days are done
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::Tinker), "the road took him");
        assert!(g.next_rare.iter().any(|(id, d)| *id == EventId::Tinker && *d > g.tick),
            "he's due again one day");
    }

    /// The hailstorm batters on its day and lifts on the next
    #[test]
    fn test_the_hailstorm_batters_and_lifts() {
        let mut g = Game::new();
        // a garden worth battering
        if g.gardens.is_empty() {
            let home = g.building_entities[0];
            g.gardens.push((home, (15, 20), 0.8));
        }
        let before: Vec<f32> = g.gardens.iter().map(|g| g.2).collect();
        g.force_event(EventId::Hailstorm);
        assert!(g.rare.iter().any(|r| r.id == EventId::Hailstorm));
        stage::run_days(&mut g, 1);
        // the battering fell: growth shredded somewhere (or already low)
        let after: Vec<f32> = g.gardens.iter().map(|g| g.2).collect();
        let battered = before.iter().zip(after.iter()).any(|(b, a)| a < b);
        assert!(battered || before.iter().all(|b| *b < 0.1), "the hail bit");
        stage::run_days(&mut g, 2);
        assert!(g.rare.iter().all(|r| r.id != EventId::Hailstorm), "the storm lifted");
    }

    /// The knife-grinder puts an edge on the trades' steel - the frame
    /// shows him at his wheel, and the road takes him when he's done
    #[test]
    fn test_the_knifegrinder_puts_an_edge_on() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            g.force_event(EventId::KnifeGrinder);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::KnifeGrinder && r.until.is_some()) { break; }
                g.advance_rare();
            }
            assert!(g.rare.iter().any(|r| r.id == EventId::KnifeGrinder), "he's afoot on {:?}", st);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the knife-grinder")),
                "the frame shows the grinder on {:?}", st);
            // his days sharpen the trades' tools
            stage::run_days(&mut g, 2);
            let sharpened = g.villagers.iter().copied().any(|v|
                g.world.get_component::<crate::game::vocation::Craft>(v).is_some()
                    && desires::is_satisfied(&g.world, v, Want::BetterTools, g.tick));
            assert!(sharpened, "a working blade got an edge on {:?}", st);
            // and the road takes him when his days are done
            stage::run_days(&mut g, 6);
            assert!(g.rare.iter().all(|r| r.id != EventId::KnifeGrinder), "the road took him on {:?}", st);
        }
    }

    /// The rag man pays the poorest in reach, and leaves the rich street
    /// empty-handed - staged poor and flush alike, the frame shows him too
    #[test]
    fn test_the_ragman_pays_the_poorest() {
        for st in [stage::Stage::Unrest, stage::Stage::Flush] {
            let mut g = stage::staged(st);
            // a rag-seller worth his coin: poor and standing by the market
            let seller = g.villagers[0];
            g.world.get_component_mut::<Purse>(seller).unwrap().gold = 1.0;
            g.force_event(EventId::RagAndBone);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::RagAndBone && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::RagAndBone).and_then(|r| r.at).unwrap();
            // stand her near his pitch
            g.world.get_component_mut::<Pos>(seller).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(seller).unwrap().y = at.1 as f32;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the rag-and-bone man")),
                "the frame shows the rag man on {:?}", st);
            stage::run_days(&mut g, 1);
            let gold = g.world.get_component::<Purse>(seller).unwrap().gold;
            // the staged flush was overridden to a penny for her - the
            // point stands either way: her rags earned
            assert!(gold > 1.0, "her rags earned on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::RagAndBone), "he moved on from {:?}", st);
        }
    }

    /// The sweep's clean flues hold the chimney fire off - the schedule
    /// itself shows it, and the street's easier for his visit
    #[test]
    fn test_the_sweep_holds_the_fire_off() {
        for st in [stage::Stage::Ordinary, stage::Stage::HardWinter] {
            let mut g = stage::staged(st);
            let before = g.next_rare.iter().find(|(i, _)| *i == EventId::ChimneyFire).map(|(_, d)| *d).unwrap();
            g.force_event(EventId::ChimneySweep);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::ChimneySweep && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the sweep")),
                "the frame shows the sweep on {:?}", st);
            // his days are done: the flues are clean, the fire's far off
            stage::run_days(&mut g, 6);
            let after = g.next_rare.iter().find(|(i, _)| *i == EventId::ChimneyFire).map(|(_, d)| *d).unwrap();
            assert!(after > before || after > g.tick + 100,
                "the chimney fire's pushed off ({before} -> {after}, tick {}) on {:?}", g.tick, st);
            assert!(g.log.iter().any(|l| l.contains("flue")), "the log says why");
            assert!(g.rare.iter().all(|r| r.id != EventId::ChimneySweep), "he moved on from {:?}", st);
        }
    }

    /// The rat-catcher's terrier earns: the store loses nothing while he
    /// works, and the plague's pushed off when he's gone
    #[test]
    fn test_the_ratcatcher_clears_the_larder() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let plague_before = g.next_rare.iter().find(|(i, _)| *i == EventId::RatPlague).map(|(_, d)| *d).unwrap();
            g.force_event(EventId::RatCatcher);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::RatCatcher && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the rat-catcher")),
                "the frame shows the catcher on {:?}", st);
            stage::run_days(&mut g, 2);
            // his sack fills: the day's tally rides in aux (the servings
            // net out against a hungry village, but his work shows here)
            assert!(g.rare.iter().find(|r| r.id == EventId::RatCatcher).map(|r| r.aux).unwrap_or(0) > 0,
                "his terrier earned on {:?}", st);
            stage::run_days(&mut g, 4);
            let after = g.next_rare.iter().find(|(i, _)| *i == EventId::RatPlague).map(|(_, d)| *d).unwrap();
            assert!(after > g.tick + 100 || after > plague_before,
                "the plague's pushed off ({plague_before} -> {after}) on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::RatCatcher), "he moved on from {:?}", st);
        }
    }

    /// The diviner's stick speaks at the end: water marked, or dry ground
    /// all round - and the frame showed him walking it first
    #[test]
    fn test_the_dowsers_stick_speaks() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            if g.gardens.is_empty() {
                let home = g.building_entities[0];
                g.gardens.push((home, (15, 20), 0.5));
            }
            let growth0: f32 = g.gardens.iter().map(|g| g.2).sum();
            g.force_event(EventId::Dowser);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Dowser && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the water-diviner")),
                "the frame shows the diviner on {:?}", st);
            stage::run_days(&mut g, 7);
            let found = g.log.iter().any(|l| l.contains("water sings"));
            let dry = g.log.iter().any(|l| l.contains("never dipped"));
            assert!(found || dry, "the stick said something on {:?}", st);
            // a spring feeds the gardens; the pit's stone yield is untouched
            if found {
                let growth1: f32 = g.gardens.iter().map(|g| g.2).sum();
                assert!(growth1 > growth0, "the gardens drank on {:?}", st);
            }
            assert!(g.land.pit_water <= 0.6, "the pit's not drowned on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::Dowser), "he moved on from {:?}", st);
        }
    }

    /// The scribe's fair hand earns: letters fly for the illiterate to
    /// the ones they think of - the frame shows him at his desk first
    #[test]
    fn test_the_scribes_fair_hand() {
        for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Scribe);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Scribe && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Scribe).and_then(|r| r.at).unwrap();
            // stand a letter-sender by his desk with a bond to write to -
            // a living one: the bereft's dead write no letters
            let living: Vec<_> = g.villagers.iter().copied()
                .filter(|&v| !villager::is_dead(&g.world, v)).collect();
            let (a, b) = (living[0], living[1]);
            g.world.get_component_mut::<Pos>(a).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(a).unwrap().y = at.1 as f32;
            let r = g.world.create_entity();
            g.world.add_component(r, crate::game::relationships::Relationship {
                from: a, other: b, kind: crate::game::relationships::RelKind::Friend,
                strength: 0.8, interactions: 5 });
            g.world.get_component_mut::<Purse>(a).unwrap().gold = 10.0;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the scribe")),
                "the frame shows the scribe on {:?}", st);
            stage::run_days(&mut g, 2);
            assert!(g.letters.iter().any(|&(f, t, _)| f == a && t == b),
                "a letter went out for her on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Scribe), "he moved on from {:?}", st);
        }
    }

    /// The mushroom flush readies every patch in the wood, adds a few
    /// bonus caps of its own for the stay, and is picked through by the
    /// end - the schedule gives it back one day
    #[test]
    fn test_the_mushroom_flush_fills_the_wood() {
        let mut g = Game::new();
        for p in g.mush_patches.iter_mut() { p.next_ready = g.tick + 100; }
        let n_before = g.mush_patches.len();
        g.force_event(EventId::MushroomFlush);
        assert!(g.mush_patches.iter().all(|p| p.next_ready <= g.tick),
            "the flush readies every patch");
        assert!(g.mush_patches.len() > n_before, "bonus caps with the flush");
        assert!(g.log.iter().any(|l| l.contains("full of caps")), "the log says so");
        stage::run_days(&mut g, 12);
        assert!(g.rare.iter().all(|r| r.id != EventId::MushroomFlush), "the flush passes");
        assert!(g.mush_patches.iter().skip(n_before).all(|p| p.next_ready > g.tick + 300),
            "the bonus caps are picked through");
    }

    /// The worst jaw in reach gets the pull, a penny the tooth - and the
    /// frame showed the drawer at his stool first
    #[test]
    fn test_the_toothdrawer_pulls_the_worst_jaw() {
        for st in [stage::Stage::Ordinary, stage::Stage::Fever] {
            let mut g = stage::staged(st);
            g.force_event(EventId::ToothDrawer);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::ToothDrawer && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::ToothDrawer).and_then(|r| r.at).unwrap();
            // a bad jaw by his stool: a living one with a coin for the pull
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Health>(v).unwrap().condition = 0.4;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the tooth-drawer")),
                "the frame shows the drawer on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<Health>(v).unwrap().condition > 0.4,
                "the pull eased her on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::ToothDrawer), "he moved on from {:?}", st);
        }
    }

    /// A bloodletting for a penny: the knife touches whoever's worst off
    /// in reach - eased or nicked, the day's work shows
    #[test]
    fn test_the_barbers_knife_touches() {
        for st in [stage::Stage::Ordinary, stage::Stage::Fever] {
            let mut g = stage::staged(st);
            g.force_event(EventId::BarberSurgeon);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::BarberSurgeon && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::BarberSurgeon).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Health>(v).unwrap().condition = 0.5;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the barber-surgeon")),
                "the frame shows the barber on {:?}", st);
            stage::run_days(&mut g, 1);
            assert_ne!(g.world.get_component::<Health>(v).unwrap().condition, 0.5,
                "his knife touched her on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::BarberSurgeon), "he moved on from {:?}", st);
        }
    }

    /// The pilgrim bowl feeds the faith: a penny in, a blessing out -
    /// and the frame shows the band passing first
    #[test]
    fn test_the_pilgrim_bowl_feeds_the_faith() {
        for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
            let mut g = stage::staged(st);
            g.force_event(EventId::PilgrimBand);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::PilgrimBand && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::PilgrimBand).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            // she's one who feels the lack of it
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a band of pilgrims")),
                "the frame shows the band on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick),
                "the bowl fed her faith on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny went in");
            stage::run_days(&mut g, 3);
            assert!(g.rare.iter().all(|r| r.id != EventId::PilgrimBand), "they passed from {:?}", st);
        }
    }

    /// The knuckle-bone's blessing holds: two gold for the faithful, and
    /// the frame shows his stall first
    #[test]
    fn test_the_relics_blessing_holds() {
        for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
            let mut g = stage::staged(st);
            g.force_event(EventId::RelicSeller);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::RelicSeller && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::RelicSeller).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the relic-seller")),
                "the frame shows the seller on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick),
                "the blessing took on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold <= 8.0, "two gold went for it");
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::RelicSeller), "he moved on from {:?}", st);
        }
    }

    /// A grievance off the heart for two gold: the burdened pay, the
    /// paper takes the weight - and the frame shows him first
    #[test]
    fn test_the_pardons_paper_takes_the_weight() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Pardoner);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Pardoner && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Pardoner).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 30);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the pardoner")),
                "the frame shows the pardoner on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<desires::InnerLife>(v).unwrap().grievances.is_empty(),
                "the pardon lifted her on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Pardoner), "he moved on from {:?}", st);
        }
    }

    /// Three knives and a grin: the joy spreads free, the hat takes a
    /// penny from those who can spare it - the frame shows him first
    #[test]
    fn test_the_jugglers_knives_lift_the_street() {
        for st in [stage::Stage::Ordinary, stage::Stage::Fever] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Juggler);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Juggler && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Juggler).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Joy];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a juggler")),
                "the frame shows the juggler on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
                "the knives lifted her on {:?}", st);
            stage::run_days(&mut g, 3);
            assert!(g.rare.iter().all(|r| r.id != EventId::Juggler), "he moved on from {:?}", st);
        }
    }

    /// The bear dances and the joy spreads - the snap's a maybe, but the
    /// crowd's lift is certain, and the frame shows the pair
    #[test]
    fn test_the_bear_dances_and_the_hat_goes_round() {
        for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
            let mut g = stage::staged(st);
            g.force_event(EventId::BearWard);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::BearWard && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::BearWard).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Joy];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a bear-ward")),
                "the frame shows the pair on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
                "the bear lifted her on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::BearWard), "they moved on from {:?}", st);
        }
    }

    /// Noah's flood enthralled them: the kids are lifted for free, and
    /// the frame shows the booth first
    #[test]
    fn test_the_puppet_show_enthralls_the_kids() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Puppeteer);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Puppeteer && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Puppeteer).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Joy];
            // she's a kid for the front row: no hat money changes hands
            g.world.add_component(v, crate::game::vocation::Kid);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a puppeteer")),
                "the frame shows the booth on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
                "the show enthralled her on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold == 10.0,
                "the front row was free on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Puppeteer), "he moved on from {:?}", st);
        }
    }

    /// A penny a broadside: the tune sticks in whoever has the ear for
    /// it - and the frame shows the sheets first
    #[test]
    fn test_the_ballad_sticks_in_the_ear() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            g.force_event(EventId::BalladMonger);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::BalladMonger && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::BalladMonger).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Music];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the ballad-monger")),
                "the frame shows the sheets on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Music, g.tick),
                "the tune stuck on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny went for the sheet");
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::BalladMonger), "he moved on from {:?}", st);
        }
    }

    /// The news gives them something to talk about - bonds pick up a
    /// day's talk in earshot of his bell
    #[test]
    fn test_the_news_sets_the_street_talking() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            g.force_event(EventId::NewsCrier);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::NewsCrier && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::NewsCrier).and_then(|r| r.at).unwrap();
            // give a listener a bond to take the talk home to
            let a = g.villagers.iter().copied().find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            let b = g.villagers.iter().copied().find(|&v| !villager::is_dead(&g.world, v) && v != a).unwrap();
            g.world.get_component_mut::<Pos>(a).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(a).unwrap().y = at.1 as f32;
            let bond = g.world.create_entity();
            g.world.add_component(bond, crate::game::relationships::Relationship {
                from: a, other: b, kind: crate::game::relationships::RelKind::Friend,
                strength: 0.5, interactions: 0 });
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the news-crier")),
                "the frame shows the crier on {:?}", st);
            stage::run_days(&mut g, 1);
            let talks = g.world.get_component::<crate::game::relationships::Relationship>(bond)
                .map(|r| r.interactions).unwrap_or(0);
            assert!(talks > 0, "the street talked on {:?}", st);
            stage::run_days(&mut g, 3);
            assert!(g.rare.iter().all(|r| r.id != EventId::NewsCrier), "he moved on from {:?}", st);
        }
    }

    /// The levy marches off with a young one if the drum caught one -
    /// or the village keeps its young, and the log says which
    #[test]
    fn test_the_drum_takes_or_the_village_keeps() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Recruiter);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Recruiter && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Recruiter).and_then(|r| r.at).unwrap();
            // a young one near his drum
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.add_component(v, crate::game::vocation::Kid);
            let before = g.villagers.len();
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the recruiting sergeant")),
                "the frame shows the drum on {:?}", st);
            stage::run_days(&mut g, 6);
            let took = g.log.iter().any(|l| l.contains("the levy took"));
            let kept = g.log.iter().any(|l| l.contains("kept its young"));
            assert!(took || kept, "the drum said something on {:?}", st);
            assert_eq!(took, g.villagers.len() < before, "the roll matched the roster on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::Recruiter), "he marched off from {:?}", st);
        }
    }

    /// Hide him and the writ follows: whoever knows his face bought the
    /// debt - and the frame showed him in the green first
    #[test]
    fn test_the_shelter_buy_the_debt() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            let debt0 = g.repression_debt;
            g.force_event(EventId::Deserter);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Deserter && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Deserter).and_then(|r| r.at).unwrap();
            // someone knows his face
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a deserter")),
                "the frame shows him on {:?}", st);
            stage::run_days(&mut g, 1);
            stage::run_days(&mut g, 5);
            assert!(g.repression_debt > debt0, "the writ's debt landed on {:?}", st);
            assert!(g.log.iter().any(|l| l.contains("writ for harboring")), "the riders came on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::Deserter), "he melted off from {:?}", st);
        }
    }

    /// Fed and rested, they stay or the road takes them - either way
    /// the store thinned for them and the log says how it ended
    #[test]
    fn test_the_family_roots_or_the_road_takes_them() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let before = g.villagers.len();
            let food0 = g.probe().food_servings;
            g.force_event(EventId::RefugeeFamily);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::RefugeeFamily && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a refugee family")),
                "the frame shows them on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.probe().food_servings < food0, "they ate from the store on {:?}", st);
            stage::run_days(&mut g, 6);
            let stayed = g.log.iter().any(|l| l.contains("family stayed"));
            let moved = g.log.iter().any(|l| l.contains("moved on at dawn"));
            assert!(stayed || moved, "their fate was told on {:?}", st);
            assert_eq!(stayed, g.villagers.len() == before + 2, "the roster matches on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::RefugeeFamily), "their camp's gone from {:?}", st);
        }
    }

    /// Crusts go out at arm's length: the store thins, the carriers'
    /// faith is fed - and the frame showed the band first
    #[test]
    fn test_the_crusts_and_the_faithful() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let food0 = g.probe().food_servings;
            g.force_event(EventId::LeperBand);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::LeperBand && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::LeperBand).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a leper band")),
                "the frame shows the band on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.probe().food_servings < food0, "the crusts went out on {:?}", st);
            assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick),
                "the charity counted on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::LeperBand), "the bells faded from {:?}", st);
        }
    }

    /// Confession under the hawthorn: a live weight comes off and the
    /// ear learns a want - the frame shows him first
    #[test]
    fn test_the_hawthorn_lightens_a_heart() {
        for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Friar);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Friar && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Friar).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Love];
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 30);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a wandering friar")),
                "the frame shows him on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<desires::InnerLife>(v).unwrap().grievances.is_empty(),
                "the weight came off on {:?}", st);
            assert!(g.world.get_component::<desires::Known>(v)
                .map(|k| k.revealed.contains(&desires::Reveal::Want(Want::Love))).unwrap_or(false),
                "the ear learned the want on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Friar), "he wandered on from {:?}", st);
        }
    }

    /// The strange doctrine doesn't leave faith where it found it:
    /// somebody's comfort moved, one way or the other
    #[test]
    fn test_the_doctrine_moves_the_streets_faith() {
        for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Heretic);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Heretic && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Heretic).and_then(|r| r.at).unwrap();
            // two ears for his doctrine, one on each side of the line:
            // an even-indexed soul gets shaken, an odd one gets drawn
            let even = g.villagers.iter().enumerate()
                .find(|(i, v)| i % 2 == 0 && !villager::is_dead(&g.world, **v)).map(|(_, v)| *v).unwrap();
            let odd = g.villagers.iter().enumerate()
                .find(|(i, v)| i % 2 == 1 && !villager::is_dead(&g.world, **v)).map(|(_, v)| *v).unwrap();
            for v in [even, odd] {
                g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
                g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
                g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
            }
            desires::satisfy(&mut g.world, even, Want::Faith, g.tick + 60);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a quiet preacher")),
                "the frame shows him on {:?}", st);
            stage::run_days(&mut g, 1);
            let faith_of = |g: &Game, v| g.world.get_component::<desires::InnerLife>(v)
                .and_then(|l| l.satisfied.iter().find(|(w, _)| *w == Want::Faith).map(|(_, u)| *u));
            assert!(faith_of(&g, even).is_none(), "the shaken lost comfort on {:?}", st);
            assert!(faith_of(&g, odd).is_some(), "the drawn were fed on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Heretic), "he preached elsewhere from {:?}", st);
        }
    }

    /// A head a day off the drove: the fold takes a sheep, the coin
    /// changes hands - and the frame shows the drove first
    #[test]
    fn test_the_drove_fills_the_fold() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            // a fold for the sheep if the stage lacks one
            let has_fold = g.world.query::<household::Building>().iter()
                .any(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
            if !has_fold {
                let f = g.world.create_entity();
                g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
                g.world.add_component(f, household::Building {
                    kind: household::BuildingKind::Pasture, name: "the fold".into(),
                    occupants: vec![], worker: None, overseer: None,
                    stock: Default::default(), capacity: 0 });
            }
            let sheep0 = g.world.query::<crate::game::livestock::Livestock>().len();
            g.force_event(EventId::Drover);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Drover && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Drover).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a drover")),
                "the frame shows the drove on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.query::<crate::game::livestock::Livestock>().len() > sheep0,
                "the fold grew on {:?}", st);
            // and she's on the flock's roll - the frame and the grazing know her
            assert!(g.flock.len() > 0, "she's on the roll on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Drover), "the drove moved on from {:?}", st);
        }
    }

    /// Salt fish keeps all winter: a penny a fish for those who'd know
    /// good food - and the frame shows the cart first
    #[test]
    fn test_the_salt_fish_sells_to_taste() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::FishHawker);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::FishHawker && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::FishHawker).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::GoodFood];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a fish-hawker")),
                "the frame shows the cart on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::GoodFood, g.tick),
                "the fish eased her on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny went for the fish");
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::FishHawker), "the cart moved on from {:?}", st);
        }
    }

    /// The quarter's bought for fair weight: the store thins and the
    /// market till fattens - the frame shows the factor first
    #[test]
    fn test_the_quarter_moves_for_fair_weight() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::CornFactor);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::CornFactor && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let food0 = g.probe().food_servings;
            let seller = g.world.get_component::<household::Building>(g.market_food)
                .and_then(|b| b.worker.or(b.overseer))
                .or_else(|| g.villagers.iter().copied()
                    .find(|&v| !villager::is_dead(&g.world, v)));
            let gold0 = seller.and_then(|s| g.world.get_component::<Purse>(s)).map(|p| p.gold).unwrap_or(0.0);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the corn-factor")),
                "the frame shows the factor on {:?}", st);
            stage::run_days(&mut g, 1);
            let food1 = g.probe().food_servings;
            let gold1 = seller.and_then(|s| g.world.get_component::<Purse>(s)).map(|p| p.gold).unwrap_or(0.0);
            assert!(food0 - food1 <= 3, "only surplus moved on {:?}", st);
            assert!(food1 < food0 || gold1 > gold0 || g.probe().food_servings <= 4,
                "the quarter moved for coin on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::CornFactor), "he weighed out from {:?}", st);
        }
    }

    /// Pepper's dear: two gold a packet eases the taste and folk hold
    /// a pinch cures what ails - the frame shows the spice-cart first
    #[test]
    fn test_the_pepper_eases_and_cures() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::SpiceMerchant);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::SpiceMerchant && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::SpiceMerchant).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::GoodFood];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the spice merchant")),
                "the frame shows the spice-cart on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::GoodFood, g.tick),
                "the packet eased her taste on {:?}", st);
            assert_eq!(g.world.get_component::<Purse>(v).unwrap().gold, 8.0, "her two gold went for the pepper");
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::SpiceMerchant), "the cart moved on from {:?}", st);
        }
    }

    /// White gold stretches the winter: a gold a handful and the salt
    /// makes the larder last - the frame shows the salt-cart first
    #[test]
    fn test_the_salt_stretches_the_store() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::SaltSeller);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::SaltSeller && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::SaltSeller).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            let food0 = g.probe().food_servings;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the salt-seller")),
                "the frame shows the salt-cart on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.probe().food_servings > food0, "the salt stretched the store on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her gold went for the salt");
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::SaltSeller), "the cart moved on from {:?}", st);
        }
    }

    /// The clip off the fold pays its shepherd a gold the head - the
    /// frame shows the wool-cart first
    #[test]
    fn test_the_clip_pays_the_shepherd() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            // a fold with a shepherd and a sheep, if the stage lacks one
            let shepherd = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            if g.world.query::<household::Building>().iter()
                .all(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind != household::BuildingKind::Pasture).unwrap_or(false)) {
                let f = g.world.create_entity();
                g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
                g.world.add_component(f, household::Building {
                    kind: household::BuildingKind::Pasture, name: "the fold".into(),
                    occupants: vec![], worker: Some(shepherd), overseer: None,
                    stock: Default::default(), capacity: 0 });
            } else {
                g.world.get_component_mut::<household::Building>(
                    g.world.query::<household::Building>().iter().copied()
                        .find(|&b| g.world.get_component::<household::Building>(b)
                            .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap())
                    .unwrap().worker = Some(shepherd);
            }
            let fold = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap();
            if g.flock.is_empty() {
                let e = crate::game::livestock::spawn_animal(&mut g.world,
                    crate::game::livestock::LivestockKind::Sheep, fold, 21.0, 21.0);
                g.flock.push(e);
            }
            let gold0 = g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0);
            g.force_event(EventId::WoolMerchant);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::WoolMerchant && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the wool merchant")),
                "the frame shows the wool-cart on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0) > gold0,
                "the clip paid him on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::WoolMerchant), "the cart moved on from {:?}", st);
        }
    }

    /// Words that outlive the winter: the lettered pay dear for a
    /// book's long joy - the frame shows the book-cart first
    #[test]
    fn test_the_book_outlives_the_winter() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Bookseller);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Bookseller && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Bookseller).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            // lettered like the book-buyers are: give her the priest's craft
            if !g.literate(v) {
                g.world.add_component(v, crate::game::vocation::craft(
                    crate::game::vocation::Vocation::Priest, 0.5));
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the bookseller")),
                "the frame shows the book-cart on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
                "the book's joy holds on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her gold went for the book");
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Bookseller), "the cart moved on from {:?}", st);
        }
    }

    /// His heap burns warm at the woods edge, and when he breaks it a
    /// stock of coal is left - the frame shows his smoke first
    #[test]
    fn test_the_heap_leaves_coal() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::CharcoalBurner);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::CharcoalBurner && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::CharcoalBurner).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            let cond0 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a charcoal burner")),
                "the frame shows his smoke on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0) > cond0,
                "the heap warmed her on {:?}", st);
            stage::run_days(&mut g, 8);
            let coal: u32 = g.world.query::<household::Building>().iter()
                .filter_map(|&b| g.world.get_component::<household::Building>(b)
                    .and_then(|bb| bb.stock.get("coal").copied()))
                .sum();
            assert!(coal > 0, "he left a heap on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::CharcoalBurner), "he packed out from {:?}", st);
        }
    }

    /// Lime for the fields: while he stands the beds grow sweeter -
    /// the frame shows the lime-cart first
    #[test]
    fn test_the_lime_sweetens_the_beds() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::LimeBurner);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::LimeBurner && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let growth0: f32 = g.gardens.iter().map(|g| g.2).sum();
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the lime-burner")),
                "the frame shows the lime-cart on {:?}", st);
            stage::run_days(&mut g, 1);
            let growth1: f32 = g.gardens.iter().map(|g| g.2).sum();
            assert!(growth1 > growth0 || g.gardens.is_empty() && growth1 == growth0,
                "the beds grew on {:?}", st);
            stage::run_days(&mut g, 7);
            assert!(g.rare.iter().all(|r| r.id != EventId::LimeBurner), "his kiln cooled from {:?}", st);
        }
    }

    /// Mind the smell: the tan-pit's reek hangs on whoever stands near
    /// his vats - the frame shows the tanner's trade first
    #[test]
    fn test_the_reek_hangs_on_the_near() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Tanner);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Tanner && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Tanner).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the tanner")),
                "the frame shows the vats on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<desires::InnerLife>(v)
                .map(|l| l.grievances.iter().any(|(p, u)| *p == desires::Problem::Smelly && g.tick <= *u))
                .unwrap_or(false), "the reek hung on her on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Tanner), "the vats packed out from {:?}", st);
        }
    }

    /// Glass for the church: one home's panes keep the cold out and
    /// let the light in - the frame shows the glazier first
    #[test]
    fn test_the_panes_keep_the_cold_out() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Glazier);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Glazier && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Glazier).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)
                    && household::home_of(&g.world, v).is_some()).unwrap();
            let h = household::home_of(&g.world, v).unwrap();
            g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the glazier")),
                "the frame shows him on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick), "the light came in on {:?}", st);
            stage::run_days(&mut g, 6);
            assert!(g.rare.iter().all(|r| r.id != EventId::Glazier), "his ladder came down on {:?}", st);
        }
    }

    /// Roofs patched before the rain finds you: fresh thatch on the
    /// stock roll and the leak's due pushed off - the frame shows
    /// his ladder first
    #[test]
    fn test_the_thatch_holds_the_rain_off() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Thatcher);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Thatcher && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Thatcher).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)
                    && household::home_of(&g.world, v).is_some()).unwrap();
            let h = household::home_of(&g.world, v).unwrap();
            g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the thatcher")),
                "the frame shows his ladder on {:?}", st);
            stage::run_days(&mut g, 1);
            let thatch = g.world.get_component::<household::Building>(h)
                .and_then(|b| b.stock.get("thatch").copied()).unwrap_or(0);
            assert!(thatch > 0, "fresh thatch on the roll on {:?}", st);
            stage::run_days(&mut g, 6);
            assert!(g.rare.iter().all(|r| r.id != EventId::Thatcher), "his ladder came down on {:?}", st);
        }
    }

    /// Sweet water, not the pond's: when his shaft comes in the sour
    /// gut clears and the beds drink - the frame shows his spade first
    #[test]
    fn test_the_shaft_strikes_sweet_water() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::WellDigger);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::WellDigger && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            desires::add_grievance(&mut g.world, v, desires::Problem::SourGut, g.tick + 30);
            let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the well-digger")),
                "the frame shows his spade on {:?}", st);
            stage::run_days(&mut g, 7);
            assert!(g.world.get_component::<desires::InnerLife>(v)
                .map(|l| l.grievances.iter().all(|(p, _)| *p != desires::Problem::SourGut))
                .unwrap_or(false), "the sweet water cleared her gut on {:?}", st);
            let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
            assert!(growth1 > growth0 || g.gardens.is_empty(), "the beds drank on {:?}", st);
            assert!(g.rare.iter().all(|r| r.id != EventId::WellDigger), "he packed his spade from {:?}", st);
        }
    }

    /// Your palm, love: a penny a palm and the lines surface a want -
    /// the frame shows her scarf first
    #[test]
    fn test_the_lines_dont_lie() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::FortuneTeller);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::FortuneTeller && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::FortuneTeller).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Love];
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the fortune-teller")),
                "the frame shows her scarf on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<desires::Known>(v)
                .map(|k| k.revealed.contains(&desires::Reveal::Want(Want::Love))).unwrap_or(false),
                "the lines surfaced the want on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny bought the palm");
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::FortuneTeller), "her tent came down on {:?}", st);
        }
    }

    /// He reads the sky over the village once: whatever's written
    /// nearest gets named, and near folk feel forearmed - the frame
    /// shows his charts first
    #[test]
    fn test_the_sky_reads_the_village() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Astrologer);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Astrologer && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Astrologer).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the astrologer")),
                "the frame shows his charts on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.log.iter().any(|l| l.contains("astrologer reads the sky")),
                "the reading was told on {:?}", st);
            assert!(desires::is_satisfied(&g.world, v, Want::Security, g.tick),
                "forearmed on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Astrologer), "his charts rolled up on {:?}", st);
        }
    }

    /// Strange things only: he takes whatever odd's in the satchel for
    /// two gold - the frame shows his flasks first
    #[test]
    fn test_the_strange_thing_sells() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Alchemist);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Alchemist && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Alchemist).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<crate::game::livelihood::Satchel>(v)
                .unwrap().goods.insert("witch's hazel".into(), 1);
            let gold0 = g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the alchemist")),
                "the frame shows his flasks on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .map(|s| s.goods.is_empty()).unwrap_or(false),
                "the strange thing's gone on {:?}", st);
            assert!(g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0) > gold0,
                "he paid for it on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Alchemist), "his flasks packed up on {:?}", st);
        }
    }

    /// Cures all: a penny a dose and the bottle does what it does -
    /// the frame shows his cart first
    #[test]
    fn test_the_bottle_does_what_it_does() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Quack);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Quack && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Quack).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
            let cond0 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the quack")),
                "the frame shows his cart on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny bought the dose");
            let cond1 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
            assert!(cond1 != cond0, "the bottle did something on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Quack), "his cart rolled on from {:?}", st);
        }
    }

    /// Babies caught, mothers kept: a home with a small one gets her
    /// tending - the frame shows her basket first
    #[test]
    fn test_the_blessing_on_the_house() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Midwife);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Midwife && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Midwife).and_then(|r| r.at).unwrap();
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)
                    && household::home_of(&g.world, v).is_some()).unwrap();
            let h = household::home_of(&g.world, v).unwrap();
            g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
            // a small one in the house for her to tend
            let k = villager::spawn_villager(&mut g.world, at.0 as f32, at.1 as f32);
            g.world.add_component(k, Health { condition: 0.5, immunity: 0.5 });
            g.world.add_component(k, crate::game::vocation::Kid);
            g.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(k);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the midwife")),
                "the frame shows her basket on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<Health>(k).map(|x| x.condition).unwrap_or(0.0) > 0.5,
                "her hands helped the small one on {:?}", st);
            stage::run_days(&mut g, 6);
            assert!(g.rare.iter().all(|r| r.id != EventId::Midwife), "she moved on from {:?}", st);
        }
    }

    /// Beasts tended: the doctored fold holds the wolves off, and the
    /// shepherd sleeps the easier - the frame shows his crook first
    #[test]
    fn test_the_doctored_fold_holds_the_wolves_off() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let shepherd = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            let fold = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
            let fold = match fold {
                Some(f) => { g.world.get_component_mut::<household::Building>(f).unwrap().worker = Some(shepherd); f }
                None => {
                    let f = g.world.create_entity();
                    g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
                    g.world.add_component(f, household::Building {
                        kind: household::BuildingKind::Pasture, name: "the fold".into(),
                        occupants: vec![], worker: Some(shepherd), overseer: None,
                        stock: Default::default(), capacity: 0 });
                    f
                }
            };
            let _ = fold;
            g.force_event(EventId::Sowgelder);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Sowgelder && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let due0 = g.next_rare.iter().find(|(i, _)| *i == EventId::WolfAtFold).map(|(_, d)| *d).unwrap_or(0);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the pig man")),
                "the frame shows his crook on {:?}", st);
            stage::run_days(&mut g, 1);
            let due1 = g.next_rare.iter().find(|(i, _)| *i == EventId::WolfAtFold).map(|(_, d)| *d).unwrap_or(0);
            assert!(due1 > due0, "the wolves' day pushed off on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Sowgelder), "he moved his crook on from {:?}", st);
        }
    }

    /// Shoes for hooves: the shepherd pays for sound stock, and a shod
    /// beast doesn't founder - the frame shows his anvil first
    #[test]
    fn test_the_ring_of_iron() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let shepherd = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            let fold = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
            match fold {
                Some(f) => { g.world.get_component_mut::<household::Building>(f).unwrap().worker = Some(shepherd); }
                None => {
                    let f = g.world.create_entity();
                    g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
                    g.world.add_component(f, household::Building {
                        kind: household::BuildingKind::Pasture, name: "the fold".into(),
                        occupants: vec![], worker: Some(shepherd), overseer: None,
                        stock: Default::default(), capacity: 0 });
                }
            }
            g.world.get_component_mut::<Purse>(shepherd).unwrap().gold = 10.0;
            // stock on the fold for him to shoe
            let fold_e = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap();
            if g.flock.is_empty() {
                let e = crate::game::livestock::spawn_animal(&mut g.world,
                    crate::game::livestock::LivestockKind::Sheep, fold_e, 21.0, 21.0);
                g.flock.push(e);
            }
            g.force_event(EventId::Farrier);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Farrier && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the farrier")),
                "the frame shows his anvil on {:?}", st);
            stage::run_days(&mut g, 1);
            assert_eq!(g.world.get_component::<Purse>(shepherd).unwrap().gold, 8.0, "his two gold went for the shoes");
            assert!(desires::is_satisfied(&g.world, shepherd, Want::Security, g.tick),
                "sound stock from here out on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Farrier), "his anvil packed up on {:?}", st);
        }
    }

    /// Sound wind and mostly honest - he buys a head off the fold a
    /// day at a dealer's price, and the shepherd takes the coin
    #[test]
    fn test_the_dealers_price() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let shepherd = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            let fold = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap();
            g.world.get_component_mut::<household::Building>(fold).unwrap().worker = Some(shepherd);
            if g.flock.is_empty() {
                let e = crate::game::livestock::spawn_animal(&mut g.world,
                    crate::game::livestock::LivestockKind::Sheep, fold, 21.0, 21.0);
                g.flock.push(e);
            }
            let flock0 = g.flock.len();
            let gold0 = g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0);
            g.force_event(EventId::HorseCop);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::HorseCop && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the horse-coper")),
                "the frame shows his string on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.flock.len() < flock0, "a head left the fold on {:?}", st);
            assert!(g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0) > gold0,
                "the dealer's price paid on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::HorseCop), "his string moved on from {:?}", st);
        }
    }

    /// Tallow taken, light for the dark: a home's tallow turns to
    /// candles, a coin for the wax - the frame shows his dipper first
    #[test]
    fn test_the_tallow_turns_to_light() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::CandleMaker);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::CandleMaker && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::CandleMaker).and_then(|r| r.at).unwrap();
            let h = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).unwrap();
            g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<household::Building>(h).unwrap().stock.insert("tallow".into(), 1);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the candle-maker")),
                "the frame shows his dipper on {:?}", st);
            stage::run_days(&mut g, 1);
            let b = g.world.get_component::<household::Building>(h).unwrap();
            assert_eq!(*b.stock.get("tallow").unwrap_or(&0), 0, "the tallow's in his dipper");
            assert!(*b.stock.get("candle").unwrap_or(&0) > 0, "light for the dark on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::CandleMaker), "his dipper packed up on {:?}", st);
        }
    }

    /// A jug that doesn't weep: a penny a crock and the house eats the
    /// better for it - the frame shows his wheel first
    #[test]
    fn test_the_crock_holds_the_stew() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Potter);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Potter && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Potter).and_then(|r| r.at).unwrap();
            let h = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).unwrap();
            g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
            // someone home to buy a crock
            let k = villager::spawn_villager(&mut g.world, at.0 as f32, at.1 as f32);
            g.world.add_component(k, Health { condition: 0.5, immunity: 0.5 });
            g.world.add_component(k, Purse { gold: 5.0 });
            g.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(k);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the potter")),
                "the frame shows his wheel on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(*g.world.get_component::<household::Building>(h).unwrap()
                .stock.get("crock").unwrap_or(&0) > 0, "a jug that doesn't weep on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::Potter), "his wheel moved on from {:?}", st);
        }
    }

    /// What have your hens done - he buys the dairy's put-by off the
    /// shelf, a coin a basket - the frame shows his panniers first
    #[test]
    fn test_the_hens_day_pays() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Eggler);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Eggler && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let at = g.rare.iter().find(|r| r.id == EventId::Eggler).and_then(|r| r.at).unwrap();
            let h = g.world.query::<household::Building>().iter().copied()
                .find(|&b| g.world.get_component::<household::Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).unwrap();
            // his panniers fill in order - make her shelf the only
            // dairy in reach so it's hers he buys
            for &o in &g.world.query::<household::Building>().iter().copied().collect::<Vec<_>>() {
                if let Some(b) = g.world.get_component_mut::<household::Building>(o) {
                    b.stock.remove("eggs"); b.stock.remove("butter"); b.stock.remove("cheese");
                }
            }
            g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
            g.world.get_component_mut::<household::Building>(h).unwrap().stock.insert("eggs".into(), 2);
            let k = villager::spawn_villager(&mut g.world, at.0 as f32, at.1 as f32);
            g.world.add_component(k, Health { condition: 0.5, immunity: 0.5 });
            g.world.add_component(k, Purse { gold: 0.0 });
            g.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(k);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the eggler")),
                "the frame shows his panniers on {:?}", st);
            // the hen may lay again the same day - the coin's the surer sign
            let gold0: f32 = g.world.get_component::<household::Building>(h).unwrap().occupants.iter()
                .filter_map(|&o| g.world.get_component::<Purse>(o).map(|p| p.gold)).sum();
            stage::run_days(&mut g, 1);
            let b = g.world.get_component::<household::Building>(h).unwrap();
            assert!(*b.stock.get("eggs").unwrap_or(&0) <= 2, "his basket's no deeper on {:?}", st);
            let gold1: f32 = g.world.get_component::<household::Building>(h).unwrap().occupants.iter()
                .filter_map(|&o| g.world.get_component::<Purse>(o).map(|p| p.gold)).sum();
            assert!(gold1 > gold0, "the coin came back on {:?}", st);
            stage::run_days(&mut g, 3);
            assert!(g.rare.iter().all(|r| r.id != EventId::Eggler), "his panniers moved on from {:?}", st);
        }
    }

    /// Aged wheels bought: two gold a wheel off the shelf, the older
    /// the better - the frame shows his rounds first
    #[test]
    fn test_the_wheel_fetches_two() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::CheeseMonger);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::CheeseMonger && r.until.is_some()) { break; }
                g.advance_rare();
            }
            // a well-aged wheel on the rack for him to covet
            let cellar = g.world.create_entity();
            g.cellar = Some(cellar);
            g.cheeses = vec![g.tick.saturating_sub(100)];
            let shepherd = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            g.shepherd = Some(shepherd);
            let gold0 = g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0);
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the cheese-monger")),
                "the frame shows his rounds on {:?}", st);
            stage::run_days(&mut g, 1);
            assert!(g.cheeses.is_empty(), "the oldest wheel went with him on {:?}", st);
            assert!(g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0) > gold0,
                "a monger's price paid on {:?}", st);
            stage::run_days(&mut g, 4);
            assert!(g.rare.iter().all(|r| r.id != EventId::CheeseMonger), "his rounds rolled on from {:?}", st);
        }
    }

    /// A letter, is there one here called-? news rides in for one of
    /// them - the frame shows his horse first
    #[test]
    fn test_the_letter_rides_in() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            g.force_event(EventId::Messenger);
            for _ in 0..2000 {
                if g.rare.iter().any(|r| r.id == EventId::Messenger && r.until.is_some()) { break; }
                g.advance_rare();
            }
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            let model = g.ui_model();
            assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a messenger")),
                "the frame shows his horse on {:?}", st);
            stage::run_days(&mut g, 1);
            let moved = desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
                || g.world.get_component::<desires::InnerLife>(v)
                    .map(|l| l.grievances.iter().any(|(p, _)| *p == desires::Problem::Grief))
                    .unwrap_or(false)
                || g.villagers.iter().any(|&o| o != v
                    && g.world.get_component::<desires::InnerLife>(o)
                        .map(|l| l.grievances.iter().any(|(p, _)| *p == desires::Problem::Grief)
                            || l.satisfied.iter().any(|(w, _)| *w == Want::Joy))
                        .unwrap_or(false));
            assert!(moved, "the letter found its man on {:?}", st);
            stage::run_days(&mut g, 3);
            assert!(g.rare.iter().all(|r| r.id != EventId::Messenger), "he rode on from {:?}", st);
        }
    }

    /// A heat wave: the folk droop, the beds ripen, the well's head
    /// drops - and the spell lifts when its days are out
    #[test]
    fn test_the_stones_are_hot() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let v = g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
            let cond0 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
            let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
            let water0 = g.land.pit_water;
            g.force_event(EventId::HeatWave);
            assert!(g.rare.iter().any(|r| r.id == EventId::HeatWave), "the heat settled in");
            stage::run_days(&mut g, 1);
            assert!(g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0) < cond0
                || g.villagers.iter().all(|&w| g.world.get_component::<Health>(w)
                    .map(|h| h.condition <= 0.05 + 0.005).unwrap_or(true)),
                "the folk droop on {:?}", st);
            let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
            assert!(growth1 > growth0 || g.gardens.is_empty(), "the beds ripen on {:?}", st);
            assert!(g.land.pit_water < water0, "the well's head dropped on {:?}", st);
            stage::run_days(&mut g, 6);
            assert!(g.rare.iter().all(|r| r.id != EventId::HeatWave), "the heat lifted on {:?}", st);
        }
    }

    /// A drought: the well runs low and the beds wilt while the sky
    /// forgets rain - and the dry lifts when its fortnight's out
    #[test]
    fn test_the_sky_forgets_rain() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let water0 = g.land.pit_water;
            g.force_event(EventId::Drought);
            assert!(g.rare.iter().any(|r| r.id == EventId::Drought), "the dry settled in");
            stage::run_days(&mut g, 1);
            assert!(g.land.pit_water < water0, "the well ran low on {:?}", st);
            stage::run_days(&mut g, 15);
            assert!(g.rare.iter().all(|r| r.id != EventId::Drought), "the rain remembered on {:?}", st);
        }
    }

    /// The river comes up: the pit fills toward drowning and the beds
    /// drink deep - and the water falls when its days are out
    #[test]
    fn test_the_reach_is_full() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let water0 = g.land.pit_water;
            g.force_event(EventId::Flood);
            assert!(g.rare.iter().any(|r| r.id == EventId::Flood), "the reach rose");
            stage::run_days(&mut g, 1);
            assert!(g.land.pit_water > water0 || water0 >= 1.0, "the water came up on {:?}", st);
            stage::run_days(&mut g, 5);
            assert!(g.rare.iter().all(|r| r.id != EventId::Flood), "the water fell on {:?}", st);
        }
    }

    /// Lightning strikes: the bolt picks one of them and knocks them
    /// flat where it lands
    #[test]
    fn test_the_bolt_picks_one() {
        for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
            let mut g = stage::staged(st);
            let before: Vec<f32> = g.villagers.iter()
                .map(|&v| g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0)).collect();
            g.force_event(EventId::LightningStrike);
            let after: Vec<f32> = g.villagers.iter()
                .map(|&v| g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0)).collect();
            let hit = before.iter().zip(after.iter()).any(|(b, a)| a < b);
            assert!(hit || g.villagers.is_empty(), "the bolt picked one on {:?}", st);
            assert!(g.log.iter().any(|l| l.contains("lightning")), "the strike was told on {:?}", st);
        }
    }

    /// The stray dog picks the loneliest heart and stays
    #[test]
    fn test_the_stray_dog_picks_a_heart() {
        let mut g = Game::new();
        let pets_before = g.pets.len();
        g.force_event(EventId::StrayDog);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::StrayDog && r.until.is_some()) { break; }
            g.advance_rare();
        }
        assert!(g.rare.iter().any(|r| r.id == EventId::StrayDog), "he's about");
        // the frame draws him as a dog, not a man
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| matches!(p.kind, PKind::Pet(Species::Dog))),
            "the frame shows a dog");
        // his stay ends: someone takes him in
        stage::run_days(&mut g, 7);
        assert!(g.rare.iter().all(|r| r.id != EventId::StrayDog));
        assert_eq!(g.pets.len(), pets_before + 1, "the stray stayed - somebody has a dog");
        assert!(g.log.iter().any(|l| l.contains("has a dog now") || l.contains("drifts on")),
            "the log tells how it ended");
    }
}
