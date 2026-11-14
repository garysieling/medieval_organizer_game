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

mod road;
mod sky;
mod beasts;
mod ill_luck;
mod findings;

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use crate::game::desires::{self, Want};
use crate::game::livelihood::Purse;
use crate::game::pets::{self, Pet, Species, PetRole};
use crate::game::villager::{self, Pos};
use crate::game::household;
use crate::sim::illness::Health;
use crate::game::mycology::MushKind;
use super::{Game, Goal, MushPatch, Rng, TRAVELLER_SPEED, TALK_BUBBLE_SECS, CHAT_BUBBLE_SECS, toast_spot};

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
    DarkNoon, WindStorm, Mirage, SeaMonster,
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
    // the moral knot: a man walks in carrying two ways to be wrong
    Dilemma,
    // the state's own answer to the pox: a traveling officer reads the
    // vaccination act on the square - once a save, the answer stands
    VaccinationAct,
}

/// The vaccination act's officer waiting on the square: the answer's
/// three ways - the scratch for the infants, the fine for the
/// refusers, or the village's own counter-institution
#[derive(Debug)]
pub struct VaxOffer {
    /// the officer riding the district
    pub authority: Entity,
}

/// The moral knot the burdened man brings: two ways to be wrong, and
/// a pick the player makes - the choice writes a debt that comes due
/// days later, when someone remembers
#[derive(Debug, Clone)]
pub struct MoralKnot {
    /// What the man asks - the two ways to be wrong
    pub text: String,
    /// Each branch: its words, and the debt it writes. Who carries the
    /// remembering is fixed when a seat answers - whoever's hands made
    /// the pick (#962)
    pub branches: Vec<DilemmaBranch>,
}

#[derive(Debug, Clone)]
pub struct DilemmaBranch {
    pub text: String,
    /// The debt it writes: who it wounds, and how hard
    pub debt: f32,
    pub victim: Entity,
}

/// A choice's weight carried home: it lands when the debt's day comes
#[derive(Debug, Clone)]
pub struct DilemmaDebt {
    pub branch: usize,
    pub due: u64,
    pub witness: Entity,
    pub victim: Entity,
    pub weight: f32,
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

use crate::session::{EvTag, Sev, Thread, Valence};
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
    EventSpec { id: Dilemma, name: "the burdened man", kind: Visitor, fig: Fig::Traveller, gap: 300, span: 150, stay: 4, cry: "Two ways to be wrong - who\'ll hear me?" },
    EventSpec { id: VaccinationAct, name: "the vaccination act", kind: Visitor, fig: Fig::Traveller, gap: 500, span: 350, stay: 12, cry: "The Act's officer rides through - the lancet waits on the square." },
    EventSpec { id: SeaMonster, name: "the deep one", kind: Moment, fig: Fig::Traveller, gap: 500, span: 360, stay: 0, cry: "" },
];

/// The spec for `id`
pub fn spec(id: EventId) -> &'static EventSpec {
    EVENTS.iter().find(|s| s.id == id).unwrap()
}

/// The visitor sprite slugs (#693's art batch, `rd/people/<slug>.png`
/// on the client) - the `PersonV.person` keys the road people draw
/// from. A visitor whose slug isn't here draws the generic courier.
pub const VISIT_SLUGS: &[&str] = &[
    "tinker", "knife-grinder", "rag-and-bone", "chimney-sweep", "rat-catcher",
    "dowser", "scribe", "tooth-drawer", "barber-surgeon", "pilgrims",
    "relic-seller", "pardoner", "juggler", "bear-ward", "puppeteer",
    "ballad-monger", "news-crier", "recruiter", "deserter", "refugee-family",
    "leper-band", "friar", "quiet-preacher", "drover", "fish-hawker",
    "salt-seller", "wool-merchant", "wine-merchant", "spice-merchant",
    "bookseller", "glazier", "thatcher", "well-digger", "charcoal-burner",
    "lime-burner", "tanner", "candle-maker", "potter", "horse-coper",
    "eggler", "cheese-monger", "fortune-teller", "astrologer", "alchemist",
    "quack", "midwife", "sowgelder", "farrier", "corn-factor", "messenger",
    "prodigal",
];

/// Which visitor sprite a `Fig::Traveller` figure draws with - the
/// variant kebab-cased (`KnifeGrinder` -> `knife-grinder`), hand-mapped
/// where the art prompt used a different name. `None` for non-visitors
/// and for a visitor with no sprite yet.
pub fn visit_slug(id: EventId) -> Option<&'static str> {
    let s = spec(id);
    if !matches!(s.kind, Visitor) || !matches!(s.fig, Fig::Traveller) { return None; }
    let slug = match id {
        PilgrimBand => "pilgrims".to_string(),
        Heretic => "quiet-preacher".to_string(),
        HorseCop => "horse-coper".to_string(),
        ProdigalSon => "prodigal".to_string(),
        _ => {
            let mut k = String::new();
            for c in format!("{:?}", id).chars() {
                if c.is_uppercase() && !k.is_empty() { k.push('-'); }
                k.push(c.to_ascii_lowercase());
            }
            k
        }
    };
    VISIT_SLUGS.iter().find(|&&x| x == slug).copied()
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
    /// Per-event scratch: a counter, a picked target, a flag. For the
    /// stray cat it's the petting cooldown - the frame_n deadline
    pub aux: u64,
    /// The stray cat's strokes, per player: when she chooses, the kindest
    /// hand keeps her
    pub petters: crate::engine::det::HashMap<Entity, u32>,
}

/// When each event first becomes due: the opening months are full of
/// strangers and wonders - every id gets a first date inside the first
/// season, jittered - then the road settles to each spec's own gap
pub fn initial_due(seed: u64) -> Vec<(EventId, u64)> {
    let mut rng = Rng(seed ^ 0x5EED5EED);
    EVENTS.iter().map(|s| (s.id, 10 + rng.next() % 120)).collect()
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

/// What's in the air when a booked day draws close: the event's own
/// sign if it has one, else the family's - every due day gets a word
fn omen_of(id: EventId) -> &'static str {
    if let Some((_, o)) = OMENS.iter().find(|(i, _)| *i == id) { return o; }
    match group(id) {
        Group::Road => "strangers on the road - someone new's for the village",
        Group::Sky => "the sky's got a strange look to it",
        Group::Beasts => "the beasts are restless",
        Group::IllLuck => "a bad feeling about the week",
        Group::Findings => "a lucky wind's stirring",
    }
}

/// Petting the stray cat: each stroke warms her a little, then she needs
/// a moment before she'll take more - the cooldown rides in the event's
/// `aux` as the frame_n deadline
pub const STRAY_CAT_PET_BOND: f32 = 0.1;
/// Frames between strokes (two seconds at 60fps)
pub const STRAY_CAT_PET_COOLDOWN: u64 = 120;
/// Warmth at which the cat chooses the hand that's been kind to her
pub const STRAY_CAT_TAME_BOND: f32 = 0.6;
/// How close the player must stand to stroke her
pub const STRAY_CAT_PET_DIST: f32 = 2.0;

/// Which family an event belongs to: its handlers live in that family's file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group { Road, Sky, Beasts, IllLuck, Findings }

/// The family an event belongs to
pub fn group(id: EventId) -> Group {
    match id {
        Dilemma | VaccinationAct => Group::Road,
        Tinker | KnifeGrinder | RagAndBone | ChimneySweep | RatCatcher | Dowser | Scribe | ToothDrawer | BarberSurgeon | PilgrimBand | RelicSeller | Pardoner | Juggler | BearWard | Puppeteer | BalladMonger | NewsCrier | Recruiter | Deserter | RefugeeFamily | LeperBand | Friar | Heretic | Drover | FishHawker | SaltSeller | WoolMerchant | WineMerchant | SpiceMerchant | Bookseller | Glazier | Thatcher | WellDigger | CharcoalBurner | LimeBurner | Tanner | CandleMaker | Potter | HorseCop | Eggler | CheeseMonger | FortuneTeller | Astrologer | Alchemist | Quack | Midwife | Sowgelder | Farrier | CornFactor | Messenger => Group::Road,
        Hailstorm | HeatWave | Drought | Flood | LightningStrike | LateFrost | IndianSummer | FirstSnow | Thaw | Aurora | MeteorShower | BloodMoon | DarkNoon | WindStorm | Mirage | SeaMonster => Group::Sky,
        WolfAtFold | FoxInYard | HawkStoops | StrayDog | StrayCat | RatPlague | Locusts | CaterpillarYear | Starlings | BeeTree | DeerInFields | WildBoar | OwlInTower | CrowsMob | FishRun => Group::Beasts,
        ChimneyFire | FouledWell | GranaryLeak | CartOverturns | DrownedInPond | LostChild | Foundling | Quarrel | AlehouseBrawl | OldOakFalls | Assize | CellarThief => Group::IllLuck,
        CoinHoard | OldStones | SpringBubbles | HoneyTree | Truffle | HerbFlush | BerryYear | MushroomFlush | StormWashUp | ProdigalSon => Group::Findings,
    }
}

impl Game {
    /// The day's rare-event business: what's due comes on, what's here
    /// works its day, what's done walks out or fades
    pub fn events_tick(&mut self, tick: u64) {
        // what's booked speaks of its coming, a few days out - due is
        // fixed until it fires, so the day hits once per booking; two
        // same-sign bookings still carry the one word between them
        let mut said = crate::engine::det::HashSet::default();
        for (id, d) in self.next_rare.clone() {
            if d == tick + OMEN_LEAD && self.rare.iter().all(|r| r.id != id)
                && said.insert(omen_of(id)) {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", tick, omen_of(id)),
                );
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
                let start = self.road_far();
                self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
                // a man of the road carries his purse: the scribe's
                // pennies a page have to land somewhere
                self.world.add_component(e, Purse { gold: 0.0 });
                let at = self.event_pitch(id);
                let mut path = self.trail_path(start, at).unwrap_or_default();
                if path.is_empty() {
                    // no road to his pitch from the far end: he sets out
                    // from a nearer road, like the named travellers do -
                    // and if even that fails he comes overland, arriving
                    // where he pitched: a firing nobody sees is no visit
                    // at all (#1073)
                    path = self.trail_path(self.road_near(start), at).unwrap_or_default();
                    if path.is_empty() {
                        if let Some(p) = self.world.get_component_mut::<Pos>(e) {
                            *p = Pos { x: at.0 as f32, y: at.1 as f32 };
                        }
                    }
                }
                self.rare.push(RareEvent {
                    id,
                    e: Some(e),
                    path,
                    until: None,
                    at: Some(at),
                    leaving: false,
                    aux: 0,
                    petters: Default::default(),
                });
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Visitor, Valence::Positive, Sev::Minor),
                    format!("t{}: {} has come to town.", self.tick, s.name),
                    Some((at.0 as f32 + 0.5, at.1 as f32 + 0.5)),
                    None,
                );
            }
            EventKind::Spell => {
                self.rare.push(RareEvent { id, e: None, path: vec![], until: Some(self.tick + s.stay),
                    at: None, leaving: false, aux: 0, petters: Default::default() });
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
                self.map.nearest_buildable(self.town_wall.0 - 3, self.town_wall.1 - 3).unwrap_or(self.village),
            EventId::LeperBand | EventId::RefugeeFamily | EventId::Recruiter =>
                self.map.nearest_buildable(self.town_wall.0 + 2, self.town_wall.3 + 2).unwrap_or(self.village),
            _ => {
                let mp = self.world.get_component::<Pos>(self.market_food)
                    .map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or(self.village);
                self.map.nearest_buildable(mp.0 + 2, mp.1 + 2).unwrap_or(self.village)
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
        let tr_spd = TRAVELLER_SPEED * self.walk_pace();
        for idx in 0..self.rare.len() {
            let mut r = self.rare[idx].clone();
            let Some(e) = r.e else { continue };
            // a word holds the feet: a visitor you're talking to doesn't
            // walk on - the marker lifts when the talk closes (#1168)
            if self.world.get_component::<villager::InPlayerTalk>(e).is_some() { continue; }
            // flames at his feet or on the road ahead: turn for home early
            if !r.leaving && self.fire_at_feet(e, &r.path, tr_spd.ceil() as usize + 1) {
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
                Self::follow_path(p, &mut r.path, tr_spd);
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
    pub(in crate::session) fn depart_event(&mut self, idx: usize) {
        if idx >= self.rare.len() { return; }
        let mut r = self.rare[idx].clone();
        let back = self.road_far();
        r.path = self.world.get_component::<Pos>(r.e.unwrap_or(0))
            .and_then(|p| self.trail_path((p.x.round() as i32, p.y.round() as i32), back))
            .unwrap_or_default();
        r.leaving = true;
        let at = r.e.and_then(|e| toast_spot(&self.world, e));
        self.log.push_ev_at_who(
            EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
            format!("t{}: {} moves on.", self.tick, spec(r.id).name),
            at,
            None,
        );
        self.rare[idx] = r;
    }

    /// The moment the figure reaches its pitch: a cry, a show, a knock
    pub(in crate::session) fn event_arrived(&mut self, id: EventId) {
        let s = spec(id);
        let e = self.rare.iter().find(|r| r.id == id).and_then(|r| r.e);
        let pitch = self.rare.iter().find(|r| r.id == id).and_then(|r| r.at)
            .map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5));
        match id {
            EventId::Dilemma => {
                // the burdened man sets his knot down: the granary burns,
                // and there's one way to be wrong - the grain or the
                // children inside it
                let victim = *self.villagers.first().unwrap_or(&0);
                self.dilemma = Some(MoralKnot {
                    text: "the granary's caught - the grain feeds the village all winter, but two children are inside. Which do you carry out first?".into(),
                    branches: vec![
                        DilemmaBranch { text: "the grain - the village starves else".into(),
                            debt: 0.6, victim },
                        DilemmaBranch { text: "the children - let the grain burn".into(),
                            debt: 0.3, victim },
                    ] });
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: {} sets his knot down at the market - two ways to be wrong.",
                        self.tick, s.name
                    ),
                    pitch,
                    None,
                );
            }
            EventId::VaccinationAct => {
                if self.vax_done {
                    // the Act's been answered once - the officer's
                    // round is a formality now, the answer stands
                    self.log.push_ev_at_who(
                        EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                        format!(
                            "t{}: {} rides through - the Act's answer stands.",
                            self.tick, s.name
                        ),
                        pitch,
                        None,
                    );
                } else {
                    self.vax_offer = Some(VaxOffer { authority: e.unwrap_or(0) });
                    if let Some(e) = e { self.say(e, s.cry, TALK_BUBBLE_SECS * 2); }
                    self.log.push_ev_at_who(EvTag::new(Thread::Illness, Valence::Neutral, Sev::Notable), format!("t{}: {} reads the vaccination act on the square - the lancet's offered for the infants, the refusers pay the fine, and a third way's whispered in the crowd.", self.tick, s.name), pitch, None);
                    self.toasts.push_at(("The Vaccination Act".into(), self.tick + 40), pitch);
                }
            }
            EventId::Tinker => {
                if let Some(e) = e {
                    self.say(e, s.cry, TALK_BUBBLE_SECS * 2);
                }
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: {} sets up by the market - out come the cracked pots.",
                        self.tick, s.name
                    ),
                    pitch,
                    None,
                );
            }
            EventId::StrayDog => {
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: a stray dog hangs about the market, ribs showing.",
                        self.tick
                    ),
                    pitch,
                    None,
                );
            }
            EventId::StrayCat => {
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: a stray cat's under the market bench, watching the folk go by.",
                        self.tick
                    ),
                    pitch,
                    None,
                );
                // she's a pet to begin with - ownerless and bondless, but
                // pettable: a player who's kind to her keeps her. Hunting
                // like any village cat: vermin watch and a warm lap once
                // she's bond enough to give it
                if let Some(e) = e {
                    self.world.add_component(e, Pet {
                        name: "a stray".into(), species: Species::Cat, owner: None,
                        bond_strength: 0.0, mood_effect: 0.15,
                    });
                    self.world.add_component(e, PetRole::Hunting);
                }
            }
            EventId::ProdigalSon => {
                if let Some(e) = e { self.say(e, s.cry, TALK_BUBBLE_SECS * 2); }
                // the fatted calf: a rolled home feasts him home, and
                // the tales start before the soup's even cold
                let homes: Vec<Entity> = self.building_entities.iter().copied().filter(|&be|
                    self.world.get_component::<household::Building>(be)
                        .map(|bb| bb.kind == household::BuildingKind::Home
                            && bb.occupants.iter().any(|o| self.villagers.contains(o)))
                        .unwrap_or(false)).collect();
                let roll = self.rare_roll(id);
                let home = homes.get((roll as usize / 7) % homes.len().max(1)).copied();
                if let Some(h) = home {
                    for &o in self.world.get_component::<household::Building>(h)
                        .map(|b| b.occupants.clone()).unwrap_or_default().iter() {
                        if villager::is_dead(&self.world, o) { continue; }
                        desires::satisfy(&mut self.world, o, Want::Joy, self.tick + 30);
                    }
                }
                let home_at = home.and_then(|h| toast_spot(&self.world, h));
                self.log.push_ev_at_who(EvTag::new(Thread::Lifecycle, Valence::Positive, Sev::Minor), format!("t{}: a prodigal returns - he's come a long way, and the door that raised him opens first.", self.tick), home_at, None);
                self.toasts.push_at(("A prodigal returns".into(), self.tick + 40), home_at);
            }
            EventId::KnifeGrinder => {
                if let Some(e) = e {
                    self.say(e, s.cry, TALK_BUBBLE_SECS * 2);
                }
                self.log.push_ev_at_who(
                    EvTag::new(Thread::Visitor, Valence::Neutral, Sev::Minor),
                    format!(
                        "t{}: {} treadles his wheel by the market - sparks fly.",
                        self.tick, s.name
                    ),
                    pitch,
                    None,
                );
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
        match group(id) {
            Group::Road => self.road_day(id, tick),
            Group::Sky => self.sky_day(id, tick),
            Group::Beasts => self.beasts_day(id, tick),
            Group::IllLuck => self.ill_luck_day(id, tick),
            Group::Findings => self.findings_day(id, tick),
        }
    }

    /// A spell's first day on: its mark in the log
    fn event_began(&mut self, id: EventId) {
        let s = spec(id);
        match group(id) {
            Group::Sky => self.sky_began(id),
            Group::Beasts => self.beasts_began(id),
            Group::IllLuck => self.ill_luck_began(id),
            Group::Findings => self.findings_began(id),
            _ => {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", self.tick, s.name),
                );
            }
        }
    }

    /// A moment's blow, all on the day
    fn event_moment(&mut self, id: EventId) {
        let s = spec(id);
        match group(id) {
            Group::Sky => self.sky_moment(id),
            Group::Beasts => self.beasts_moment(id),
            Group::IllLuck => self.ill_luck_moment(id),
            Group::Findings => self.findings_moment(id),
            _ => {
                self.log.push_ev(
                    EvTag::new(Thread::Other, Valence::Neutral, Sev::Minor),
                    format!("t{}: {}.", self.tick, s.name),
                );
            }
        }
    }

    /// A visitor's last day: what it leaves behind
    fn event_end(&mut self, id: EventId, tick: u64) {
        match group(id) {
            Group::Road => self.road_end(id, tick),
            Group::Beasts => self.beasts_end(id, tick),
            Group::IllLuck => self.ill_luck_end(id, tick),
            Group::Findings => self.findings_end(id, tick),
            _ => {}
        }
    }
}

/// The conversation family a visitor belongs to: which dialogue tree
/// speaks for them when the player walks up and Talks (#1168). Every
/// traveller kind lands somewhere; the status carries on the Speaker as
/// `arch-*` so kind slugs stay free for per-kind overrides.
pub fn visitor_archetype(id: EventId) -> &'static str {
    use EventId::*;
    match id {
        Tinker | RagAndBone | SaltSeller | WoolMerchant | WineMerchant
        | SpiceMerchant | Bookseller | CornFactor | CheeseMonger
        | FishHawker | Eggler | HorseCop => "arch-trader",
        KnifeGrinder | ChimneySweep | RatCatcher | Dowser | Scribe
        | ToothDrawer | BarberSurgeon | Glazier | Thatcher | WellDigger
        | CharcoalBurner | LimeBurner | Farrier | Midwife | Sowgelder
        | Potter | CandleMaker | Tanner => "arch-craftsman",
        Juggler | BearWard | Puppeteer | BalladMonger | NewsCrier
        | Messenger => "arch-performer",
        PilgrimBand | RelicSeller | Pardoner | Friar => "arch-religious",
        RefugeeFamily | LeperBand => "arch-needy",
        Recruiter | Deserter => "arch-soldier",
        FortuneTeller | Astrologer | Alchemist | Quack => "arch-mystic",
        _ => "arch-caller",
    }
}

/// What "I'll take it" means with this visitor: the first fork of every
/// visitor tree is the deal, and this is what agreeing to it does
pub struct VisitorOffer {
    /// A satchel good they sell you
    pub good: Option<&'static str>,
    /// Gold your purse loses - a ware's price, a service, an alms
    pub gold_out: f32,
    /// Gold your purse gains - the recruiter's shilling
    pub gold_in: f32,
    /// Agreeing keeps them this much longer (a deserter hidden)
    pub stay_extra: u64,
    /// What goes on the talk's record when the deal lands
    pub note: &'static str,
}

/// No deal on offer: the talk is all the arc has
pub const NO_OFFER: VisitorOffer = VisitorOffer {
    good: None, gold_out: 0.0, gold_in: 0.0, stay_extra: 0, note: "",
};

/// A traveller's premium: the road charges more than the village bench
const TRAVELER_PREMIUM: f32 = 1.5;

/// What each kind's deal does. Priced wares carry the traveller's
/// premium over the same good's bench price (#1168)
pub fn visitor_offer(id: EventId) -> VisitorOffer {
    use EventId::*;
    let ware = |good: &'static str, base: f32, note: &'static str| VisitorOffer {
        good: Some(good), gold_out: base * TRAVELER_PREMIUM, gold_in: 0.0, stay_extra: 0, note,
    };
    let service = |price: f32, note: &'static str| VisitorOffer {
        good: None, gold_out: price, gold_in: 0.0, stay_extra: 0, note,
    };
    match id {
        Tinker => ware("tinware", 2.0, "You buy a mended pot from the tinker."),
        RagAndBone => ware("oddments", 1.0, "You pick a serviceable thing from the rag man's sack."),
        SaltSeller => ware("salt", 2.0, "You buy a measure of salt - dear, but the winter wants it."),
        WoolMerchant => ware("wool", 3.0, "You buy a fleece's worth of wool at the road's price."),
        WineMerchant => ware("wine", 4.0, "You buy a flask of wine - it costs what the road costs."),
        SpiceMerchant => ware("spice", 5.0, "You buy a twist of spice, dearer than sin."),
        Bookseller => ware("book", 5.0, "You buy a book with a cut spine and all its pages."),
        CornFactor => ware("grain", 2.0, "You buy a sack of grain at the factor's price."),
        CheeseMonger => ware("cheese", 2.0, "You buy a truckle of cheese from the monger."),
        FishHawker => ware("fish", 1.0, "You buy a string of fish, smelling of the coast."),
        Eggler => ware("eggs", 1.0, "You buy a basket of eggs, counted twice."),
        HorseCop => ware("nag", 4.0, "You take the coper's soundest nag - and his blessing with it."),
        CharcoalBurner => ware("charcoal", 2.0, "You buy a bag of charcoal, black-fingered and light."),
        LimeBurner => ware("lime", 2.0, "You buy a bushel of burned lime."),
        RelicSeller => ware("relic", 4.0, "You buy the saint's knuckle - a splinter, at least, of something."),
        Alchemist => ware("elixir", 3.0, "You buy the elixir - it smells of cloves and lies."),
        Quack => ware("remedy", 2.0, "You buy the remedy - it may even work."),
        KnifeGrinder => service(2.0, "He hones every blade in the house on the treadle wheel."),
        ChimneySweep => service(2.0, "His boy goes up the flue; the soot comes down in curtains."),
        RatCatcher => service(2.0, "His terriers work the walls - the rats have a bad afternoon."),
        Dowser => service(2.0, "His forked stick bends near the garden - water, he says, if you dig."),
        Scribe => service(2.0, "He writes your letter in a fair clerk's hand."),
        ToothDrawer => service(2.0, "The tooth comes out with the pliers; the ache goes with it."),
        BarberSurgeon => service(2.0, "He opens the vein and binds it neat - you feel the lighter for it."),
        Glazier => service(2.0, "He sets a new pane where the shutter broke."),
        Thatcher => service(2.0, "He promises to stop by your roof before he quits the parish."),
        WellDigger => service(2.0, "He tells you where your well should go and how deep."),
        Farrier => service(2.0, "He reshoes the plough horse at his travelling anvil."),
        Midwife => service(2.0, "She examines, advises, and takes her fee with both hands."),
        Sowgelder => service(2.0, "The sow squeals once; the gelder pockets his fee."),
        CandleMaker => ware("candles", 2.0, "You buy a bundle of dipped candles."),
        Potter => ware("pots", 2.0, "You buy two glazed pots, wrapped in straw."),
        Tanner => ware("leather", 3.0, "You buy a dressed hide, supple and smelling of oak bark."),
        Juggler | BearWard | Puppeteer | BalladMonger => service(1.0,
            "You toss a coin into the hat - the show's worth it."),
        NewsCrier => service(1.0, "You pay a penny for the full telling, names and all."),
        FortuneTeller => service(1.0, "She turns the cards and tells you what you half knew already."),
        Astrologer => service(2.0, "He casts your nativity on a scraped horn tablet."),
        PilgrimBand | Friar => service(1.0, "You press alms into a pilgrim's palm."),
        Pardoner => service(3.0, "You buy the pardon - sealed, stamped, and probably genuine."),
        RefugeeFamily | LeperBand => service(1.0, "You give what you can; it is not enough, and it is something."),
        Recruiter => VisitorOffer { good: None, gold_out: 0.0, gold_in: 2.0,
            stay_extra: 0, note: "The King's shilling weighs heavy in your palm." },
        Deserter => VisitorOffer { good: None, gold_out: 0.0, gold_in: 0.0,
            stay_extra: 2400, note: "You say nothing of him - he can breathe a while longer." },
        _ => NO_OFFER,
    }
}

/// Whether a rebuff sends this visitor down the road early - the kinds a
/// village can plausibly drive off with hard words (#1168)
pub fn visitor_dismissible(id: EventId) -> bool {
    matches!(id, EventId::Deserter | EventId::Heretic
        | EventId::LeperBand | EventId::RefugeeFamily)
}

/// A one-line read on a road visitor for the TALK panel - the speaker's
/// seed quirks already colour how he answers (#1168)
pub fn visitor_blurb(id: EventId) -> &'static str {
    match visitor_archetype(id) {
        "arch-trader" => "A trader off the road, selling at the road's price.",
        "arch-craftsman" => "A craftsman passing through, work for coin.",
        "arch-performer" => "A performer working the villages.",
        "arch-religious" => "A soul on God's own road.",
        "arch-needy" => "The road's own poor, asking what you'd spare.",
        "arch-soldier" => "The war's business, walking the villages.",
        "arch-mystic" => "A dealer in omens and doubtful sciences.",
        _ => "A traveller, resting a day off the road.",
    }
}
