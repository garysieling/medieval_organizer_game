//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

pub mod model;
pub mod net;
pub mod driver;
pub mod events;
pub mod probe;
pub mod stage;
pub mod dialogue;
pub mod routine;
pub mod country;
pub mod quests;
pub mod roster;

pub mod journal;
pub mod kpi;
pub mod layout;
pub mod seat;
pub mod setup;
mod subsystems;
mod verbs;
pub mod view_model;
#[cfg(test)]
mod tests;

pub use model::*;
pub use net::*;
pub use driver::*;
pub use layout::*;
pub use seat::*;
pub use view_model::*;

use crate::engine::ecs::{Entity, Scheduler, World};
use crate::game::terrain::Terrain;
use crate::game::tilemap::{Layout, Tile, TileMap, WallPiece, NEIGHBOURS, noise};
use crate::game::villager::{self, Hunger, Villager, Activity, Pos, Smell};
use crate::game::pets::{self, Species, PetRole, Pet};
use crate::game::player::{PlayerCharacter, act_talk, act_recruit, TalkCount};
use crate::game::guilds::{GuildFormationSystem, Guild};
use crate::sim::climate::{self, SeasonSystem, Season};
use crate::sim::precipitation::{precip_kind, PrecipKind};
use crate::sim::illness::{self, Health, Disease, DiseaseKind, Stage, IllnessSystem};
#[allow(unused_imports)] // only the tests name attachment styles now
use crate::sim::psychology::{AttachmentHistory, AttachmentStyle};
use crate::sim::community::{Citizen, Organization};
use crate::game::organizing::{gossip_insult, meeting_remark, passing_remark, talk_response};
use crate::game::traits::personality_from_seed;
use crate::game::relationships;
use crate::game::vocation::{self, Craft, Kid, KidGrowth, Level, SocialClass, TownRole, Vocation};
use crate::game::desires::{self, InnerLife, Known, Organizer, Problem, Want};
use crate::game::household::{self, Building, BuildingKind};
use crate::game::goods;
use crate::game::livelihood::{self, OddJob, Purse, Satchel};
use crate::game::fire::Fire;
use crate::game::herbology::{self, Apothecary};
use crate::game::mycology::{self, AilPhase, MushEffect, MushKind, PendingAil};
use crate::game::livestock::{self, Livestock, LivestockKind};
use crate::game::apiary::{self, HiveKind};
use crate::game::sanctions::{self, Sanction};
use crate::sim::reputation_qsp::{ReputationState, RepAction, RepStage};
use crate::sim::trails::{self, Trail, TrailParams};
#[cfg(test)]
use crate::game::relationships::Relationship;

pub fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }


/// Which way a step of (dx, dy) on the map looks on screen: 0 up, 1
/// right, 2 down, 3 left (the walk sheet's row order)
fn facing_on_screen(dx: f32, dy: f32) -> u8 {
    let (sx, sy) = ((dx - dy) * TILE_W as f32, (dx + dy) * TILE_H as f32);
    if sx.abs() > sy.abs() { if sx > 0.0 { 1 } else { 3 } } else if sy > 0.0 { 2 } else { 0 }
}


/// Where to draw a character's name: centred over them, just above the head
/// The player's coins (PlayerCharacter.resources) and their Purse are one
/// purse: actions spend resources, trades move the Purse. `to_purse`
/// copies resources -> Purse (after an action), false the other way
/// (after a day's trading)
fn sync_purse(world: &mut World, player: Entity, to_purse: bool) {
    let (res, gold) = (world.get_component::<PlayerCharacter>(player).map(|p| p.resources),
        world.get_component::<Purse>(player).map(|p| p.gold));
    if let (Some(r), Some(g)) = (res, gold) {
        if to_purse { if r != g { world.get_component_mut::<Purse>(player).unwrap().gold = r; } }
        else if r != g { world.get_component_mut::<PlayerCharacter>(player).unwrap().resources = g; }
    }
}


/// How many log lines the session keeps at most. A line or three lands
/// each day, so uncapped the log grew without bound over a long game -
/// and every frame's model used to clone all of it. The window only
/// shows the recent end; harnesses and tests read days or weeks back,
/// far under this floor.
const LOG_MAX: usize = 16384;

/// Overflow trims down to this, so the trim is a rare memmove instead of
/// a per-line churn.
const LOG_TRIM_TO: usize = 12288;


/// Whether the crowd-separation pass may push this villager aside: not the
/// dead, and not someone on their way to food
fn makes_room(world: &World, v: Entity) -> bool {
    !villager::is_dead(world, v)
        && world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none()
        && world.get_component::<Villager>(v).map(|x| x.activity != Activity::SeekingFood).unwrap_or(true)
        && !routine::holds_slot(world, v)
}


/// The disease `who` is fighting, if any: (name, stage)
fn disease_of(world: &World, who: Entity) -> Option<(String, Stage)> {
    world.query::<Disease>().into_iter()
        .filter_map(|d| world.get_component::<Disease>(d))
        .find(|d| d.patient == who)
        .map(|d| (d.name.clone(), d.stage))
}

/// The name of the sickness `who` is showing, if any. The latent are
/// brewing it and don't feel it yet; the recovering are on the mend.
fn sickness_name(world: &World, who: Entity) -> Option<String> {
    disease_of(world, who)
        .filter(|(_, stage)| *stage != Stage::Recovering && *stage != Stage::Latent)
        .map(|(name, _)| name)
}

/// Is `who` ill right now - showing symptoms?
/// Write a beat into a villager's tale; when the tale first reads as a
/// story - a theme takes shape - the log names it once
fn note_story(
    tales: &mut crate::engine::det::HashMap<usize, Vec<crate::sim::story::StoryEvent>>,
    told: &mut crate::engine::det::HashSet<usize>,
    log: &mut Vec<String>,
    i: usize, tick: u64, kind: crate::sim::story::EventKind, desc: String,
) {
    let ev = crate::sim::story::StoryEvent { tick, kind, description: desc };
    let tale = tales.entry(i).or_default();
    tale.push(ev);
    if tale.len() > 12 { tale.remove(0); }
    if told.contains(&i) { return; }
    if let Some(st) = crate::sim::story::detect_story(tale) {
        told.insert(i);
        let word = match st.theme {
            crate::sim::story::StoryTheme::Survival => "a tale of survival",
            crate::sim::story::StoryTheme::Rise => "a tale of rising",
            crate::sim::story::StoryTheme::Fall => "a tale of falling",
            crate::sim::story::StoryTheme::Romance => "a tale of the heart",
            crate::sim::story::StoryTheme::Redemption => "a tale of redemption",
            crate::sim::story::StoryTheme::Conflict => "a tale of bad blood",
            crate::sim::story::StoryTheme::Community => "a tale of the commons",
        };
        log.push(format!("t{}: the village marks it - {}'s is {}.", tick, vill_name(i), word));
    }
}

fn is_sick(world: &World, who: Entity) -> bool {
    sickness_name(world, who).is_some()
}

pub struct Button {
    pub label: &'static str,
    pub key: char,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub color: (u8, u8, u8),
}


/// How close (in tiles) a villager must be to the player to talk, recruit,
/// or come to a house meeting
const MEETING_REACH: f32 = 8.0;
/// How far (tiles) an idle villager strays from the village food store
/// before heading back
const VILLAGE_RADIUS: f32 = 7.0;
/// Gold a new house costs
const BUILD_COST: f32 = 30.0;
/// Every fourth row and column through the market is a street: blocks
/// three plots deep, the houses fronting the street and a yard behind
const STREET_EVERY: i32 = 4;
/// The market square: open ground this far round the cross
const SQUARE_R: i32 = 2;
/// The plots nearest the square kept for the workshops
const TRADE_PLOTS: usize = 26;

/// The town's streets inside the wall: a grid through the market cross,
/// every fourth row and column - the church and the mill keep their own
/// tiles (the square round the cross is left open green, see town_plots)
fn town_streets(wall: (i32, i32, i32, i32), hub: (i32, i32), keep: [(i32, i32); 2]) -> Vec<(i32, i32)> {
    let (hx, hy) = hub;
    let mut out = vec![];
    for y in wall.1 + 1..wall.3 {
        for x in wall.0 + 1..wall.2 {
            let on_grid = (x - hx).rem_euclid(STREET_EVERY) == 0 || (y - hy).rem_euclid(STREET_EVERY) == 0;
            if on_grid && !keep.contains(&(x, y)) {
                out.push((x, y));
            }
        }
    }
    out
}

/// The building plots inside the wall: open cells fronting a street (the
/// block's inner cells stay yards), nearest the market square first
fn town_plots(map: &TileMap, wall: (i32, i32, i32, i32), hub: (i32, i32)) -> Vec<(i32, i32)> {
    let (hx, hy) = hub;
    let mut plots: Vec<(i32, i32)> = (wall.1 + 1..wall.3).flat_map(|y| (wall.0 + 1..wall.2).map(move |x| (x, y)))
        .filter(|&(x, y)| map.buildable(x, y)
            // the market square stays open: the stalls and the crowd's
            && ((x - hx).abs() > SQUARE_R || (y - hy).abs() > SQUARE_R)
            && NEIGHBOURS.iter().any(|(dx, dy)| map.get(x + dx, y + dy) == Some(Tile::Road)))
        .collect();
    plots.sort_by_key(|&(x, y)| ((x - hx).abs().max((y - hy).abs()), (x - hx).abs() + (y - hy).abs(), y, x));
    plots
}
/// Frames drawn per second
pub const FPS: u32 = 60;
/// How long a reply to Talk, and a remark in passing, stay up (seconds)
const TALK_BUBBLE_SECS: u32 = 5;

/// How close the player stands to someone to talk to them (tiles). Talk or
/// Recruit on someone farther off walks the player over first.
const TALK_DIST: f32 = 1.5;

/// Frames the player can make no headway before a walk over to talk gives up
const APPROACH_STUCK_FRAMES: u32 = FPS;

/// The courier's pace along the road (tiles a frame): a day's ride each way
const TRAVELLER_SPEED: f32 = 4.0 / FPS as f32;

/// Days the ink-maker's cauldron smokes on the meadow: the boil, the
/// crusts, the drip test, and the grinding
const INK_BOIL_DAYS: u64 = 5;

/// Ink he sells off before the road takes him
const INK_YIELD: u32 = 6;

/// Per-boiling-day chance the oil's vapours take fire - it happens, "as
/// will happen" says the Encyclopedie; most times he smothers it, but the
/// meadow keeps the scorch
const OIL_FLARE_PCT: i32 = 8;

const CHAT_BUBBLE_SECS: u32 = 2;
/// How near two people must be to chat in passing (tiles), how many days
/// before the same pair chat again, and how much a chat counts for (a fifth
/// of a conversation: +0.01 to the relationship)
const CHAT_RANGE: f32 = 1.5;
const CHAT_COOLDOWN: u64 = 20;
const CHAT_INTENSITY: f32 = 0.2;
/// How low the tavern's barrel runs before the keeper walks out to buy
pub const TAVERN_BAR_LOW: u32 = 3;
/// How many measures he lays in at a time
pub const TAVERN_BARREL: u32 = 4;
/// What he pays the trade that made it, as a share of the street price -
/// the rest is the bench, the fire and his living
pub const TAVERN_WHOLESALE: f32 = 0.7;
/// How close a drinker stands to count as at the table
pub const TAVERN_TABLE_REACH: f32 = 4.0;
/// How many the bench seats; the rest drink standing
pub const TAVERN_BENCH: usize = 6;
/// The keeper's opening float: enough barrels to get the trade turning,
/// since his is the one trade that buys its stock before it sells it
pub const TAVERN_FLOAT: f32 = 80.0;
/// Coin the keeper keeps back off the bar - rent, bread, and the lord's
/// share on the barrel all come due whether the bench fills or not
pub const TAVERN_RESERVE: f32 = 12.0;
/// The share of what's left over that he'll commit to a day's lay-in
pub const TAVERN_SPEND_SHARE: f32 = 0.5;
/// The chance a cup at the bench is the one too many - the same roll the
/// counter makes, since it's the same cup
pub const TAVERN_DRUNK_PCT: i32 = 15;
/// How far the player walks each frame, in tiles (7 tiles a second)
const WALK_SPEED: f32 = 7.0 / FPS as f32;
/// How much faster the player goes on a road, and slower through woods
const ROAD_SPEEDUP: f32 = 1.8;
const WOODS_SLOWDOWN: f32 = 0.6;
/// How far from the player (in tiles) to look for a free building site
const BUILD_SEARCH: i32 = 8;

/// Day a traveller brings la grippe to the village
const FLU_DAY: u64 = 120;

/// Keyboard shortcut for each action button, in button order (Talk, Recruit,
/// Build). The input loop and the buttons' key hints
/// both come from here so they can't disagree. W/A/S/D move the player, so
/// no action may use them.
const ACTION_KEYS: [char; 3] = ['T', 'R', 'B'];
/// Whether action button `i` is shown at all: Recruit only appears once
/// there's a campaign to recruit into (founded at a house meeting)
pub fn action_shown(m: &UiModel, i: usize) -> bool {
    i != 1 || m.panel.campaign.is_some()
}

/// The action panel's buttons, in ACTION_KEYS order
pub const ACTION_LABELS: [&str; 3] = ["Talk", "Recruit", "Build"];
/// The action panel's buttons, laid out the same for every player
pub fn action_buttons() -> [Button; 3] {
    [
        Button { label: ACTION_LABELS[0].into(), key: ACTION_KEYS[0], x: UI_X as i32 + 10, y: 60,  w: 130, h: 36, color: (60,60,120) },
        Button { label: ACTION_LABELS[1].into(), key: ACTION_KEYS[1], x: UI_X as i32 + 10, y: 104, w: 130, h: 36, color: (60,120,60) },
        Button { label: ACTION_LABELS[2].into(), key: ACTION_KEYS[2], x: UI_X as i32 + 10, y: 148, w: 130, h: 36, color: (80,60,120) },
    ]
}
/// The button a contextual action draws with
pub fn ctx_button(act: CtxAct) -> Button {
    match act {
        CtxAct::Work => Button { label: "Work".into(), key: ' ', x: UI_X as i32 + 10, y: 278, w: 130, h: 30, color: (90, 90, 140) },
        CtxAct::Errand => Button { label: "Errand".into(), key: ' ', x: UI_X as i32 + 10, y: 278, w: 130, h: 30, color: (90, 120, 90) },
        CtxAct::PairUp => Button { label: "Pair up".into(), key: ' ', x: UI_X as i32 + 10, y: 278, w: 130, h: 30, color: (120, 90, 140) },
        CtxAct::Matchmake => Button { label: "Matchmake".into(), key: ' ', x: UI_X as i32 + 10, y: 278, w: 130, h: 30, color: (160, 90, 110) },
        // Meet lives on the campaign's FIGHT line in the org section - the
        // 278 slot would run into the inspector; March beside it
        CtxAct::Meet => Button { label: "Meet".into(), key: ' ', x: UI_X as i32 + 150, y: 486, w: 60, h: 16, color: (140, 80, 80) },
        CtxAct::Cadence => Button { label: "Cadence".into(), key: ' ', x: UI_X as i32 + 150, y: 470, w: 60, h: 16, color: (100, 100, 130) },
        CtxAct::March => Button { label: "March".into(), key: ' ', x: UI_X as i32 + 216, y: 486, w: 60, h: 16, color: (150, 60, 60) },
        // Delegate and Strike act on the picked member - they live on the
        // bottom edge of their inspector card
        CtxAct::Delegate => Button { label: "Seat".into(), key: ' ', x: UI_X as i32 + 146, y: 438, w: 62, h: 16, color: (80, 80, 140) },
        CtxAct::Strike => Button { label: "Strike".into(), key: ' ', x: UI_X as i32 + 212, y: 438, w: 62, h: 16, color: (150, 60, 60) },
        // demands to the collector share the card's bottom row; the
        // prisoner's petition sits where the seat button would
        CtxAct::Townhall => Button { label: "Demands".into(), key: ' ', x: UI_X as i32 + 146, y: 422, w: 62, h: 16, color: (150, 60, 60) },
        CtxAct::Petition => Button { label: "Petition".into(), key: ' ', x: UI_X as i32 + 146, y: 438, w: 62, h: 16, color: (140, 80, 80) },
        // Gather sits where Organize used to - the meeting is earned now
        CtxAct::Gather => Button { label: "Gather".into(), key: ' ', x: UI_X as i32 + 10, y: 236, w: 130, h: 30, color: (120, 100, 40) },
        CtxAct::TakeJob => Button { label: "Take the job".into(), key: ' ', x: UI_X as i32 + 150, y: 278, w: 130, h: 30, color: (90, 110, 90) },
        CtxAct::Pact => Button { label: "Swear aid".into(), key: ' ', x: UI_X as i32 + 10, y: 470, w: 130, h: 16, color: (80, 120, 90) },
        CtxAct::PetCat => Button { label: "Pet her".into(), key: ' ', x: UI_X as i32 + 150, y: 236, w: 130, h: 30, color: (110, 90, 120) },
        // shares the row's right half with Pet her - pushed last, it
        // draws over her in the rare winter visit, and once the fight's
        // called its button's gone and she's pettable again
        CtxAct::Snowballs => Button { label: "Snowballs!".into(), key: ' ', x: UI_X as i32 + 150, y: 236, w: 130, h: 30, color: (130, 160, 220) },
        CtxAct::Recruit => Button { label: "Win them".into(), key: ' ', x: UI_X as i32 + 212, y: 470, w: 62, h: 16, color: (90, 110, 150) },
    }
}
#[cfg(test)] // only the tests check against it
/// Keys that move the player
const MOVE_KEYS: [char; 4] = ['W', 'A', 'S', 'D'];

/// The whole town: adults, the three children (26-28), the tax collector
/// (29) and the herb-wife (30) last. Tests and seeds lean on this fixed order.
pub const NAMES: [&str; 31] = [
    "Aldric","Eliza","Bram","Corvin","Dagny","Hob","Marga","Gisela",
    "Wystan","Aelfric","Tilda","Osric","Fenwick","Hilda","Cedric","Rowena",
    "Godric","Matilda","Leofric","Edith","Anselm","Bertha","Roderick","Winifred",
    "Osmund","Greta","Pip","Nell","Tom","Reeve","Willa",
];
/// The second roll, for the born and the come-lately: the village and
/// the country grow past the first thirty-one, and they shouldn't all
/// answer to "a newcomer"
const NEW_NAMES: [&str; 78] = [
    "Sible","Dunn","Hamelin","Avice","Colle","Merrick",
    "Sabyn","Tybalt","Imma","Peryn","Lettice","Hobkin",
    "Wymond","Tansy","Godelieve","Oswin","Milo","Sabine",
    "Eglantine","Ralf","Isolde","Baldwin","Clarice","Drogo",
    "Emeline","Fulke","Petronilla","Jocelin","Wymar","Beatrice",
    "Hugh","Agnes","Lucan","Philippa","Sturmy","Ysolde",
    "Garard","Denise","Thurstan","Rohese","Engelard","Conan",
    "Basilia","Warin","Melisende","Jocosa","Aldwin","Ermentrude",
    "Goscelin","Mahelt","Pernel","Simone","Adeline","Bartram",
    "Serle","Hawise","Jankin","Mabyn","Osbert","Peronell",
    "Quintin","Rohaise","Tibbets","Urian","Wymark","Erk",
    "Juliana","Miles","Nicolas","Orm","Petronel","Roget",
    "Sanchia","Terrice","Vincent","Wymund","Aveline","Bricot",
];
/// A villager's name by index - past the named roll, the second roll
/// keeps them distinct; past that, a newcomer stands in
pub fn vill_name(i: usize) -> &'static str {
    NAMES.get(i).copied()
        .or_else(|| NEW_NAMES.get(i - NAMES.len()).copied())
        .unwrap_or("a newcomer")
}
/// Which villager indices are the children, the tax collector and the herb-wife
const KID_IDXS: [usize; 3] = [26, 27, 28];
pub const COLLECTOR_IDX: usize = 29;
const HERB_IDX: usize = 30;
pub const RIVAL_NAMES: [&str; 3] = ["Wulfric","Sable","Miren"];


/// A house being built: first its road is laid out from the village's
/// roads a tile every ROAD_TILE_FRAMES, then the house rises over
/// RISE_FRAMES. `frames` counts the (unpaused) frames it's been going.
pub struct Works {
    pub building: usize, // index into Game::buildings
    pub road: Vec<(i32, i32)>,
    pub frames: u32,
}
const ROAD_TILE_FRAMES: u32 = FPS / 2;
const RISE_FRAMES: u32 = 3 * FPS;

impl Works {
    /// How many of the road's tiles are laid so far
    fn road_laid(&self) -> usize { ((self.frames / ROAD_TILE_FRAMES) as usize).min(self.road.len()) }
    /// How much of the house stands, 0 to 1 (0 until the road is done)
    fn risen(&self) -> f32 {
        let start = self.road.len() as u32 * ROAD_TILE_FRAMES;
        (self.frames.saturating_sub(start) as f32 / RISE_FRAMES as f32).min(1.0)
    }
}

/// A house meeting under way (Organize). The invited walk to the venue,
/// talk it over in turns in thought bubbles, and only then does it
/// resolve (Game::resolve_meeting).
pub struct Meeting {
    pub label: String,                       // where it is: "Bram's house"
    pub invited: Vec<Entity>,
    pub spots: Vec<(i32, i32)>,              // where each invitee stands, round the venue
    pub paths: Vec<Option<Vec<(i32, i32)>>>, // each one's way there, next step first (None: no way)
    pub frames: u32,                         // how long since it was called, unpaused
    pub talk_from: Option<u32>,              // when the talking began
    pub lines: Vec<(Entity, String)>,        // the conversation, in order
    /// Its record on the TALK panel, once the talking starts
    pub talk_id: u64,
    /// The seat that called it - resolution belongs to their org
    pub seat: usize,
    /// Roles promised for this meeting: (snacks, notes, opener, song,
    /// closer) - the meeting runs better for each kept
    pub roles: (bool, bool, bool, bool, bool),
}

/// The founding call a finished house meeting left with its organizer:
/// the vote the room took (kept for the stakes - grumbles get re-read
/// against whichever framing the organizer picks), the record it was
/// said on, and the org's name. Held on the calling seat until they
/// answer it (Command::Decide), their next meeting's offer supersedes
/// it, or it lapses unanswered after FOUNDING_OFFER_EXPIRY_DAYS - a
/// client with no control wired to Decide (the browser's canvas shim)
/// would otherwise leave every seat at half pace (see ticks_per_frame) forever
pub struct PendingFounding {
    /// The meeting's talk record - the convo box keeps its tail showing
    pub record: u64,
    /// The founding vote the room can take, if one is on the table - a
    /// cause already founded leaves no vote, just the next steps
    pub vote: Option<crate::game::campaign::IssueVote>,
    pub org_name: String,
    /// Who'd come back for another meeting
    pub meet_again: Vec<Entity>,
    /// Whether "take it to the street" is on offer (a cause exists)
    pub street: bool,
    /// The day the call was left, so an unanswered one can time out
    pub offered_tick: u64,
}
/// How long an organizer's call goes unanswered before it lapses on its
/// own, same as if they'd let it rest
const FOUNDING_OFFER_EXPIRY_DAYS: u64 = 7;
/// How fast people walk to a meeting (tiles a frame), how long the host
/// waits for them, and how long each turn to speak lasts
const MEETING_WALK: f32 = 2.5 / FPS as f32;
const GATHER_FRAMES: u32 = 20 * FPS;
const LINE_FRAMES: u32 = 3 * FPS;
/// How long the knights hold the streets after a crackdown: a posting is
/// a tour, not a life sentence - they ride home in a month
const GUARD_TOUR: u64 = 30;
/// What the player says to open the meeting, between rounds, and to close it
const MEETING_PROMPTS: [&str; 3] = [
    "Thanks for coming. What's on your minds?",
    "So what would make the most difference?",
    "Then let's take it on together.",
];

/// The village's vocations and how commonly each is rolled
/// How fast feet wear the ground in, and how fast it greens over
const TRAIL_PARAMS: TrailParams = TrailParams { reinforce_rate: 0.06, decay_rate: 0.02, saturation: 0.85 };
/// How much a fully-worn trail cuts the cost of crossing it (the
/// trails::movement_cost dial): ~1.5x at saturation, matching what the
/// old ad-hoc speedup gave
const TRAIL_K: f32 = 0.375;

const VOCATION_ROLL: [(Vocation, i32); 10] = [
    (Vocation::Farmer, 22), (Vocation::Baker, 12), (Vocation::Mason, 10),
    (Vocation::Tailor, 9), (Vocation::Toolmaker, 8), (Vocation::Knight, 8),
    (Vocation::Priest, 8), (Vocation::Luthier, 6), (Vocation::Printmaker, 5),
    (Vocation::Glazier, 5),
];

/// The trades that keep a village drinking: exactly one is guaranteed
/// at spawn, rolled off the seed - an alehouse, a brewery, a vintner's
/// or a still. They never come up in the open roll, so no village gets
/// two taps and none goes dry
pub const DRINK_TRADES: [Vocation; 4] = [
    Vocation::Alewife, Vocation::Brewer, Vocation::Vintner, Vocation::Distiller,
];

/// Roll a vocation by its weighted table (shareable by player spawns)
fn roll_vocation(rng: &mut Rng) -> Vocation {
    let total: i32 = VOCATION_ROLL.iter().map(|(_, w)| w).sum();
    let mut pick = rng.below(total);
    for (v, w) in VOCATION_ROLL { if pick < w { return v; } pick -= w; }
    Vocation::Farmer
}

/// What a traveller is walking to do when he gets there
#[derive(Clone, Debug)]
pub enum Goal {
    /// Buy `good` from a person's satchel (inputs)
    BuyFrom(Entity, &'static str, u32),
    /// Buy `good` at a shop's stockpile (finished goods; maybe for a want)
    BuyAt(Entity, &'static str, Option<Want>),
    /// Gather `good` at a tile (wood, stone, sand)
    Gather(&'static str, (i32, i32)),
    /// The collector walks to collect the year's cut
    CollectTax(Entity),
    /// A kid runs an OddJob (component has hirer/good/from/to/pay)
    RunOddJob,
    /// An agent asked their villager to go stand and chat with someone
    TalkTo(Entity),
    /// An agent asked their villager to simply stand somewhere
    Wander,
    /// The herb-wife walks out to pick a herb patch (index into herb_patches)
    Forage(usize),
    /// Someone sick walks out to the herb hut for a cure
    SeekRemedy(Entity),
    /// The shepherd walks out to the flock
    Herd(Entity),
    /// A hungry or broke soul gathers mushrooms in the shade (index into mush_patches)
    ForageShroom(usize),
    /// Walk home at the end of the day (target: the building entity)
    HeadHome(Entity),
    /// Driven out: walk to the map's edge and don't come back
    Exile,
    /// Weed and pick a garden bed (index into gardens)
    TendGarden(usize),
    /// An agent asked their villager to eat at the market store - arrival
    /// takes a real serving, not just a stand nearby
    Eat,
    /// A town's cart on the road to the village market (site index)
    Cart(usize),
    /// A castle's steward riding to a town's granary (castle, town)
    Levy(usize, usize),
    /// The steward back up to the castle with the provisions
    Deliver(usize),
}

/// A herb patch on the map: a tuft of `herb` that regrows a season after
/// it's picked
#[derive(Clone)]
pub struct HerbPatch {
    pub tile: (i32, i32),
    pub herb: &'static str,
    /// Pickable from this day
    pub next_ready: u64,
}

/// A walk under way: tiles to step along, what for when they run out
#[derive(Clone)]
pub struct Travel {
    pub who: Entity,
    pub path: Vec<(i32, i32)>,
    pub goal: Goal,
}

/// The travelling minstrel: walks the road in, plays at the market a
/// while, and leaves
pub struct Minstrel {
    pub e: Entity,
    pub path: Vec<(i32, i32)>,
    /// Some(day) while he's playing at the market until
    pub playing_until: Option<u64>,
}

/// The travelling circus: a troupe camps on the meadow for a fortnight -
/// music and wonder every day, a lighter purse every night, and for the
/// bored and the unbelonging, the real question: why stay?
pub struct Circus {
    pub e: Entity,
    pub path: Vec<(i32, i32)>,
    /// Some(day) while it's camped until
    pub camped_until: Option<u64>,
    /// The meadow tile it pitches on
    pub camp: (i32, i32),
}

/// The ink-maker: once a year or so he walks the road in and boils his
/// linseed oil down to varnish on the meadow outside the gate - the law
/// and the fire both want that trade out there (Encyclopedie: "a wide
/// space such as a garden"). Then lampblack goes into the pot, and the
/// print shop gets ink the road used to have to bring.
pub struct InkBoiler {
    pub e: Entity,
    pub path: Vec<(i32, i32)>,
    /// Some(day) while his cauldron smokes on the meadow until
    pub working_until: Option<u64>,
    /// The meadow tile the cauldron stands on
    pub camp: (i32, i32),
}

/// The roving pedlar: walks the road in, pitches his stall at the
/// market with goods nothing local supplies, trades gossip both ways,
/// and leaves. He's the game's newspaper.
pub struct Pedlar {
    pub e: Entity,
    pub path: Vec<(i32, i32)>,
    /// Some(day) while he's selling at the market until
    pub selling_until: Option<u64>,
    /// His pitch while he's here
    pub stall: Option<Entity>,
    /// The river tale's told once per visit
    pub rumor_sold: bool,
}

/// The map tests and tooling use: Game::new is always this one
pub const MAP_SEED: u64 = 42;

/// The map is five times the original 49-tile side in both dimensions.
pub const MAP_SCALE: usize = 5;
pub const MAP_TILES: usize = 49 * MAP_SCALE;

/// The map the windowed game plays: GAME2_SEED pins one (for a screenshot
/// of a particular country); otherwise each launch draws a fresh one
pub fn run_seed() -> u64 {
    std::env::var("GAME2_SEED").ok().and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() % 1_000_000).unwrap_or(MAP_SEED))
}

/// A tiny seeded generator (splitmix64): one seed settles the whole map
pub struct Rng(pub u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    /// 0..n
    fn below(&mut self, n: i32) -> i32 { (self.next() % n.max(1) as u64) as i32 }
    /// -n..=n
    fn around(&mut self, n: i32) -> i32 { self.below(2 * n + 1) - n }
}

/// The buildable tile with water lapping it nearest `to`: where a fishing
/// hut goes. None when there's no shore within reach.
fn shore_site(map: &TileMap, to: (i32, i32)) -> Option<(i32, i32)> {
    let mut best: Option<((i32, i32), i32)> = None;
    for y in 1..map.height as i32 - 1 {
        for x in 1..map.width as i32 - 1 {
            let lapping = NEIGHBOURS.iter()
                .any(|(dx, dy)| matches!(map.get(x + dx, y + dy), Some(Tile::Water) | Some(Tile::River)));
            if !lapping || !map.buildable(x, y) { continue; }
            let d = (x - to.0).abs() + (y - to.1).abs();
            if best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some(((x, y), d)); }
        }
    }
    best.map(|(p, _)| p)
}

/// A buildable tile beside a tree, outside `wall`, nearest `to`: where the
/// woodcutter's clearing goes. None when the woods aren't near.
fn woods_edge_site(map: &TileMap, wall: (i32, i32, i32, i32), to: (i32, i32)) -> Option<(i32, i32)> {
    let mut best: Option<((i32, i32), i32)> = None;
    for y in 1..map.height as i32 - 1 {
        for x in 1..map.width as i32 - 1 {
            if (wall.0..=wall.2).contains(&x) && (wall.1..=wall.3).contains(&y) { continue; }
            if !map.buildable(x, y)
                || !map.trees.iter().any(|&(tx, ty)| (tx - x).abs() + (ty - y).abs() == 1) { continue; }
            let d = (x - to.0).abs() + (y - to.1).abs();
            if best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some(((x, y), d)); }
        }
    }
    best.map(|(p, _)| p)
}


/// Which sprite a building draws with: the sim names the look and the view
/// resolves it to the art pack's pixels, so the sim itself never holds art.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BSprite {
    /// Index into the cottage set (wraps round)
    House(usize),
    /// A company town's manager's row: the grand look (the hall's art
    /// stands in until the manor set lands)
    Mansion,
    /// A terraced street: the index picks the one cottage the whole row
    /// shares, so the street reads as a single build
    Rowhouse(usize),
    /// The workers' quarters by the gate: the index picks the barracks
    /// variant (the watch-house set stands in)
    Barracks(usize),
    Blacksmith,
    Church,
    Guildhall,
    RivalHouse,
    RivalWoodhouse,
    CastleKeep,
    /// A workplace's own look; the index picks the variant (and the
    /// cottage that stands in when that look has no art)
    Work(WorkLook, usize),
}

/// The workplaces that wear their own look instead of a cottage's
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkLook { Tavern, Bakery, Granary, WatchHouse, Fold, Apiary, Cellar, Workshop, Factory }

impl WorkLook {
    pub const N: usize = 9;
    pub const ALL: [WorkLook; WorkLook::N] = [WorkLook::Tavern, WorkLook::Bakery, WorkLook::Granary,
        WorkLook::WatchHouse, WorkLook::Fold, WorkLook::Apiary, WorkLook::Cellar, WorkLook::Workshop,
        WorkLook::Factory];

    /// Its sprites' name under assets/rd/buildings (name1.png, name2.png)
    pub fn file(self) -> &'static str {
        match self {
            WorkLook::Tavern => "tavern", WorkLook::Bakery => "bakery", WorkLook::Granary => "granary",
            WorkLook::WatchHouse => "watch-house", WorkLook::Fold => "fold", WorkLook::Apiary => "apiary",
            WorkLook::Cellar => "cheese-cellar", WorkLook::Workshop => "workshop",
            WorkLook::Factory => "factory",
        }
    }

    /// The look a kind of building wears: None keeps a cottage (homes, the
    /// farmhouse) or a look of its own (the smithy, the church)
    pub fn of(kind: BuildingKind) -> Option<WorkLook> {
        Some(match kind {
            BuildingKind::Shop(Vocation::Taverner | Vocation::Alewife | Vocation::Brewer) => WorkLook::Tavern,
            BuildingKind::Shop(Vocation::Baker) => WorkLook::Bakery,
            BuildingKind::Shop(Vocation::Farmer | Vocation::Toolmaker) => return None,
            BuildingKind::Shop(_) => WorkLook::Workshop,
            BuildingKind::Granary => WorkLook::Granary,
            BuildingKind::WatchPost => WorkLook::WatchHouse,
            BuildingKind::Pasture => WorkLook::Fold,
            BuildingKind::Apiary => WorkLook::Apiary,
            BuildingKind::Cellar => WorkLook::Cellar,
            BuildingKind::Home | BuildingKind::Church | BuildingKind::Mill | BuildingKind::TraderStall => return None,
        })
    }
}


/// Big moments announced at the top of the map: (text, shown until
/// tick, where it happened). `push` is the plain banner; `push_at` says
/// where it happened, so the client can jump the camera there on a click
#[derive(Debug, Default)]
pub struct Toasts(pub Vec<(String, u64, Option<(f32, f32)>)>);

impl Toasts {
    pub fn push(&mut self, t: (String, u64)) { self.0.push((t.0, t.1, None)); }
    pub fn push_at(&mut self, t: (String, u64), at: Option<(f32, f32)>) { self.0.push((t.0, t.1, at)); }
    pub fn retain(&mut self, f: impl FnMut(&(String, u64, Option<(f32, f32)>)) -> bool) { self.0.retain(f) }
}
impl std::ops::Deref for Toasts {
    type Target = [(String, u64, Option<(f32, f32)>)];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for Toasts {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

/// Where `e` stands this frame, for a toast the player can click to visit
fn toast_spot(world: &World, e: Entity) -> Option<(f32, f32)> {
    world.get_component::<Pos>(e).map(|p| (p.x, p.y))
}

/// The whole game state. The windowed game and `--dump-frame` both drive
/// it through the same update() and render(), so a headless frame is the
/// frame a player would see. This is the authoritative simulation: it owns
/// no art, no camera and no window state - the View draws it.
pub struct Game {
    pub world: World,
    pub sched: Scheduler,
    /// What each tile is (water, grass, road, river...), laid out on the terrain
    pub map: TileMap,
    pub climate: Entity,
    pub villagers: Vec<Entity>,
    /// Each villager's last position, facing and whether they moved this
    /// frame (track_poses): the walk cycles read it
    pub villager_poses: crate::engine::det::HashMap<Entity, ((f32, f32), u8, bool)>,
    pub rivals: Vec<Entity>,
    pub rival_homes: Vec<(f32, f32)>, // where each rival lives, across the river
    /// One seat per claimed connection (seat 0 in solo play)
    pub seats: Vec<Seat>,
    /// Each pet, its species (the view picks the sprite), and how it stands off its owner
    pub pets: Vec<(Entity, Species, (f32, f32))>,
    pub market_food: Entity, // our village's food store (the rivals have their own)
    pub buildings: Vec<(f32, f32, BSprite)>,
    pub stones: Vec<(i32, i32)>, // boulders out in the fields
    /// The town wall's rectangle this map (it breathes with the seed) - for
    /// keeping the grass specks off the town streets
    pub town_wall: (i32, i32, i32, i32),
    /// The market cross this seed's village grew round (tiles)
    pub village: (i32, i32),
    /// Where the windmill ended up (it's animated, so drawn apart from `buildings`)
    pub mill: (f32, f32),
    /// The watch standing just inside each gate
    pub guards: Vec<(f32, f32)>,
    /// Knights posted to the streets after a crackdown, tagged with the
    /// org they stand over - gone when that org's tour is done, unlike
    /// the gate watch that stands all year
    pub posted: Vec<(Entity, f32, f32)>,
    /// The priest by the church
    pub priest: (f32, f32),
    /// The courier's road to the rivals' village and back (empty if there is none)
    pub traveller_path: Vec<(i32, i32)>,
    /// Where the gulls hang about (open-sea tiles)
    pub gulls: Vec<(f32, f32)>,
    /// The hen's farmyard
    pub chicken_at: (f32, f32),
    /// The map's own generator: settled at new_seeded, then handing out
    /// sprites (e.g. which cottage each new build gets)
    pub rng: Rng,
    /// The seed this world was dealt: the character roster derives from
    /// it, so a server and its clients deal the same hand
    pub seed: u64,
    pub built_count: usize,
    pub works: Vec<Works>, // houses still being built
    pub toasts: Toasts,  // (msg, shown until tick, where it happened)
    pub last_guild_count: usize,
    pub tick: u64,
    pub paused: bool,
    pub speed: u32,   // 1=normal 2=fast 4=turbo
    pub log: Vec<String>,
    /// Lines trimmed off the log's front so far - a cursor into "every
    /// line ever pushed" is `log_dropped + index into log`, so readers
    /// that pace the log (the balance harness) survive a trim
    pub log_dropped: usize,

    pub bubbles: Vec<(Entity, String, u32, bool, bool)>, // speech bubbles: (speaker, text, until frame, a thought?, ambient passing remark?)
    pub meeting: Option<Meeting>,
    /// The day a snowball fight was called (u64::MAX = never)
    pub snowball_day: u64,
    /// Whether today's fight has raised its hubbub yet
    pub snowball_cued: bool,
    /// Where today's fight is being made, for the sound's pan
    pub snowball_at: (f32, f32),
    /// Snowballs in the air this frame
    pub snowballs: Vec<routine::Snowball>,
    /// Pairs of people standing near each other, and the day each pair last chatted
    pub near: Vec<(Entity, Entity)>,
    pub last_chat: Vec<((Entity, Entity), u64)>,
    /// Every conversation heard - villagers in passing, the player's Talk,
    /// house meetings - for the TALK panel (session::dialogue)
    pub talk_log: Vec<dialogue::TalkRecord>,
    pub talk_next_id: u64,
    pub frame_n: u32,

    pub precip: Option<PrecipKind>, // what is falling today
    pub rainbow_from: Option<u32>,   // frame the last rain stopped (rainbow fades from there)
    pub was_sick: Vec<Option<String>>, // per villager, the disease they're showing - to log when someone falls ill or recovers
    /// The village's tune: the evolving-music state the day's world
    /// metrics steer
    pub music: Entity,
    /// The weather event rolling through: storm, dry spell, heat or
    /// hard frost - fields and bones feel it while it lasts
    pub weather: Entity,
    pub died_on: Vec<Option<u64>>, // per villager, the day they died
    /// The Building entity behind each `buildings` entry (parallel)
    pub building_entities: Vec<Entity>,

    /// The tax collector's entity (villagers[COLLECTOR_IDX])
    pub collector: Option<Entity>,
    /// The trader's stall building at the market
    pub trader_stall: Option<Entity>,
    /// Walks under way (errands, shopping, the tax round, odd jobs)
    pub travels: Vec<Travel>,
    /// The travelling minstrel, when one's about
    pub minstrel: Option<Minstrel>,
    /// Day the next minstrel is due
    pub next_minstrel: u64,
    /// The roving pedlar, when one's about
    pub pedlar: Option<Pedlar>,
    /// Day the next pedlar is due
    pub next_pedlar: u64,
    /// The travelling circus, when it's camped
    pub circus: Option<Circus>,
    /// Day the next circus is due
    pub next_circus: u64,
    /// The ink-maker, when he's boiling on the meadow
    pub ink_boiler: Option<InkBoiler>,
    /// Day the next ink-maker is due
    pub next_ink_boiler: u64,
    /// Whatever's alight on the map, and the ground it scorched
    pub fire: Fire,
    /// Which buildings currently stand derelict (the view darkens them)
    pub derelict: crate::engine::det::HashSet<usize>,
    /// (who, complaint family) already voiced at the meeting under way -
    /// parallel complaints make the next speaker likelier to share
    pub voiced: Vec<(Entity, desires::IssueTag)>,
    /// The herb-wife (villagers[HERB_IDX]) and her hut out in the meadow
    pub herbologist: Option<Entity>,
    pub herb_hut: Option<Entity>,
    /// The herb tufts growing wild on the map
    pub herb_patches: Vec<HerbPatch>,
    /// (who, the day they last called at the hut) - so the sick don't besiege her
    pub remedy_visits: Vec<(Entity, u64)>,
    /// The fold out in the meadow and the shepherd who keeps it
    pub fold: Option<Entity>,
    pub shepherd: Option<Entity>,
    /// The living animals, and where the day's grazing point sits
    pub flock: Vec<Entity>,
    pub graze_at: (f32, f32),
    /// Each animal's own spot of grass and the frame it picks the next
    /// one: the flock grazes as individuals, not in lockstep
    pub flock_wander: crate::engine::det::HashMap<Entity, (f32, f32, u32)>,
    pub next_graze: u64,
    /// Yard hens and the homes they scratch about
    pub hens: Vec<Entity>,
    pub hen_houses: Vec<Entity>,
    /// The heart-helper, and the pairings she's already tried this month
    pub matchmaker: Option<Entity>,
    pub match_tried: Vec<(Entity, Entity, u64)>,
    /// Wild mushroom patches in the shade of the wood
    pub mush_patches: Vec<MushPatch>,
    /// (who, the day they last gathered) - so the wood isn't stripped by one soul
    pub shroom_breaks: Vec<(Entity, u64)>,
    /// (who, the patch indices they've learned are death) - the wood
    /// teaches its lessons the hard way, and the herb-wife passes them on
    pub shroom_lore: Vec<(Entity, Vec<usize>)>,
    /// Ailments that keep their own calendar: the amatoxin course lands
    /// in phases days after the meal, and Saint Anthony's brothers make
    /// their rounds once an outbreak's named
    pub ailments: Vec<PendingAil>,
    /// The tavern on the green: the one table the whole village drinks
    /// at. Its keeper brews nothing - he lays in the drink trades' barrels
    /// and pours them by the cup
    pub tavern: Option<Entity>,
    /// Who sat at the tavern's table on which day - the common table's
    /// register, so a night's drinking is a night's company
    pub tavern_table: Vec<(Entity, u64)>,
    /// The apiary, its keeper, its skeps, and the church it keeps in candles
    pub apiary: Option<Entity>,
    pub beekeeper: Option<Entity>,
    pub hives: Vec<Entity>,
    pub church: Option<Entity>,
    /// (who, the day the swarm leaves) - a swarm takes a liking to someone
    pub swarm: Option<(Entity, u64)>,
    /// The dead whose affairs are settled - succession runs once per soul
    pub succeeded: Vec<Entity>,
    /// Where feet have worn the ground: tile -> trail strength
    pub trails: crate::engine::det::HashMap<(i32, i32), Trail>,
    /// The land's memory: canopy left, water in the pit, stink downstream
    pub land: crate::sim::environment::LandScore,
    /// The worked pit's tile, and the reach below town that takes the leavings
    pub pit: Option<(i32, i32)>,
    pub reach: Vec<(i32, i32)>,
    /// (who, the rung they're on, the day it ends) - the ladder of shame
    pub sanctioned: Vec<(Entity, Sanction, u64)>,
    /// The worst state each feud was announced at, so the log tells its
    /// turns once each
    pub feud_told: crate::engine::det::HashMap<Entity, crate::game::feuds::FeudStatus>,
    /// (villager -> (day, units)) - what each one made at the bench today,
    /// so the card can say the day's work plainly
    pub produced: crate::engine::det::HashMap<Entity, (u64, f32)>,
    /// (workshop -> (day, good -> net units)) - what flowed through a
    /// shop today: inputs out of stock, outputs in, so the shop's card
    /// can show the chain it worked
    pub shop_flow: crate::engine::det::HashMap<Entity, (u64, Vec<(String, i32)>)>,
    /// The pack's been seen: a wolf night is coming once the roll lands
    pub wolf_coming: bool,
    /// The hens are restless: a fox night is coming once the roll lands
    pub fox_coming: bool,
    /// What the collector owes the lord: levies and excise in hand,
    /// separate from his own purse - it rides out on the remittance
    pub takings: f32,
    /// (home, bed tile, growth, kitchen-bed) - kitchen gardens beside
    /// the cottages: herbs on one, roots and peas for the pot on the next
    pub gardens: Vec<(Entity, (i32, i32), f32, bool)>,
    /// The cheese cellar, the wheels aging in it (start ticks), its character
    pub cellar: Option<Entity>,
    pub cheeses: Vec<u64>,
    pub cellar_trait: &'static str,
    /// The lord's estimate of the org's power - lags the truth, updated
    /// only by what he sees
    pub lord_esteem: f32,
    /// How the village keeps its accounts - the outside counts in
    /// positional numerals, so trade loses a little to the reckoning
    pub numeracy: Numeracy,
    /// The clerk who keeps the tallies straight, when one's emerged
    pub translator: Option<Entity>,
    /// Reckonings that didn't add up - enough and a clerk steps up
    pub reckonings: u32,
    /// A sky event rides over the walls: ("comet"/"eclipse", the day it goes)
    pub sky: Option<(&'static str, u64)>,
    /// The disclosure: (who found it, what they found, the day it was found)
    /// while it stays unsaid
    pub pending_discovery: Option<(Entity, &'static str, u64)>,
    /// Whether the discovery's been made public
    pub disclosure_made: bool,
    /// How many the herb-wife's turned - proof the old cures work
    pub cured: u32,
    /// What the castle's owed after a defied writ - the next one lands double
    pub repression_debt: u32,
    /// Whether the winter's running long this year - every other winter
    /// is a harsh one, and the village either pulls together or apart
    pub harsh_winter: bool,
    /// (building, the day it was painted) - color is clout that fades
    pub painted: Vec<(Entity, u64)>,
    /// The things made to outlast their makers - item entities
    pub heirlooms: Vec<Entity>,
    /// Letters in transit: (from, to, the day they arrive)
    pub letters: Vec<(Entity, Entity, u64)>,
    /// The cave mouth at the map's edge - the monochrome under
    pub cave_pos: Option<(i32, i32)>,
    /// The hermit who keeps to it
    pub hermit: Option<Entity>,
    /// Whether the player's first body's gone - the work goes on
    pub player_passed: bool,
    /// The martyr's grave - where they hanged the one who said no
    pub martyr_grave: Option<(i32, i32)>,
    /// Path memo: (from,to) -> the walk, cleared each day - the map
    /// barely changes mid-day, and the BFS is the game's hot line
    pub path_cache: crate::engine::det::HashMap<((i32, i32), (i32, i32)), Option<Vec<(i32, i32)>>>,
    /// The tick the cache was born on - a day older and it's stale
    pub path_tick: u64,
    /// The audio cue queue: drained per frame into the UiModel
    pub sfx: Vec<SfxCue>,
    /// The castle's prisoners: (who, the day they were taken) - the org
    /// ransoms them or the ladder climbs
    pub dungeon: Vec<(Entity, u64)>,
    /// Rare events afoot: the visitors standing their pitch, the spells
    /// holding their days (the hundred are in session/events.rs)
    pub rare: Vec<events::RareEvent>,
    /// The rare-event schedule: (which, the day it's due)
    pub next_rare: Vec<(events::EventId, u64)>,
    /// What's already fired: (which, the day) - the probe's record
    pub rare_fired: Vec<(events::EventId, u64)>,
    /// The events' own rng stream - scheduling rolls off it, never off
    /// `rng`, so the events don't shift what the village would have rolled
    pub rare_seed: u64,
    /// The country's towns and castles, and who keeps each
    pub sites: Vec<country::Site>,
    /// The wood's own rng stream - visions, dark turns and the fire's
    /// blame roll off it, never off `rng`, so the wood doesn't shift
    /// what the village would have rolled either
    pub shroom_rng: Rng,
    /// The safety note's told its piece: real cap traits in-world carry
    /// the disclaimer once, where folk will read it
    pub shroom_disclaimed: bool,
    /// The odd jobs villagers have asked the Wanderer to run, waiting
    /// for a taker (session::quests)
    pub pending_quests: Vec<quests::PendingQuest>,
    /// Giver -> the day they can ask again: a job settles a while
    pub quest_rest: crate::engine::det::HashMap<Entity, u64>,
    /// The village's research: the printmaking track the print shop's
    /// work feeds (game::tech_tree)
    /// When each dead villager fell - the day their home's still theirs
    /// for the heirs before the bed goes free
    pub dead_since: crate::engine::det::HashMap<Entity, u64>,
    /// The first harvest got its line in the log
    pub crop_noted: bool,
    /// The village's crop fields - stocked sources like the granary,
    /// standing at the edge where the ground allowed them
    pub crop_fields: Vec<Entity>,
    /// Each villager's tale as the days write it - the story events the
    /// chronicle reads for themes
    pub tales: crate::engine::det::HashMap<usize, Vec<crate::sim::story::StoryEvent>>,
    /// Villager indexes whose formed tale was already named in the log
    pub tales_told: crate::engine::det::HashSet<usize>,
    /// The village's wood: a living stand that grows, spreads, burns
    /// and regrows - the fuel commons its coverage leans on
    pub forest: crate::sim::emergence::Automata,
    /// The ground the village stands on: the landform's name, when one
    /// shaped the heightfield
    pub landform: Option<String>,
    /// The wood's own chance-stream: a lightning roll mustn't shift the
    /// village's other draws
    pub forest_rng: Rng,
    /// The parish congregation: every villager a member, the priest its
    /// leader - holy-day services feed devotion, bonds and the poor box
    pub congregation: Entity,
    /// What the priest's old books have taught: the parish's reading,
    /// collected
    pub grimoire: crate::sim::magic::Grimoire,
    pub tech: Vec<crate::game::tech_tree::Tech>,
}

/// An audio cue: the sim raises these, the audio layer maps them to
/// sound - a kind, a place, an intensity
#[derive(Debug, Clone)]
pub struct SfxCue {
    pub kind: &'static str,
    pub x: f32,
    pub y: f32,
    pub intensity: f32,
}

/// A saved scenario: a village already mid-story when you walk in
#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    /// The year the scarlatina took hold - sickness already in the walls
    FeverYear,
    /// The year the circus came in a hard winter - wonders and thin stores
    CircusWinter,
    /// A shop stands derelict, a widow weighs the keeping of it
    WidowsShop,
}

/// A letter, traveling - durable word that crosses distance
#[derive(Debug)]
pub struct Letter {
    pub from: Entity,
    pub to: Entity,
    pub due: u64,
}

/// How a village keeps its accounts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Numeracy {
    Tally,
    RomanBoard,
    Positional,
}

/// A wild mushroom patch: what it is stays hidden until it's in the pan.
/// Some caps are death; a skilled eye reads them, a hungry one doesn't.
#[derive(Debug, Clone, Copy)]
pub struct MushPatch {
    pub tile: (i32, i32),
    /// the lookalike problem: which of the wood's kinds it really is
    pub kind: MushKind,
    /// the day it flushes again
    pub next_ready: u64,
}

impl MushPatch {
    /// The lookalike problem in a word: this one's death
    pub fn deadly(&self) -> bool { self.kind.is_deadly() }
}

/// seat 0 is the local/only player in solo play: `game.selected_target` and
/// friends resolve through here so single-seat code needs no seat index.
impl std::ops::Deref for Game {
    type Target = Seat;
    fn deref(&self) -> &Seat { &self.seats[0] }
}
impl std::ops::DerefMut for Game {
    fn deref_mut(&mut self) -> &mut Seat { &mut self.seats[0] }
}

impl Game {
    pub fn ticks_per_frame(&self) -> u32 {
        // frames per day: ~21s/day normal, ~3.8s turbo; a house meeting (or
        // the call it leaves the organizer) slows the calendar so the
        // room's talk can be read as it's said. Speeds past turbo (tests
        // fast-forwarding weeks at a time) thin the frames further: the
        // per-frame strides scale by ticks_per_frame, so a day still
        // covers the same ground, just sampled less often
        let base = if self.speed == 1 { 21 * FPS } else { (15 * FPS / self.speed.max(4)).max(1) };
        if self.meeting.is_some() || self.seats.iter().any(|s| s.pending_founding.is_some()) { base * 2 } else { base }
    }

    /// A frame's share of a day's walking, whatever the watch speed. The
    /// stroll constants are per-frame at walking pace (a day is 21s of
    /// frames then); scaling by it keeps a day's stride the same at turbo
    /// and beyond, the way the pull-home steps already do
    pub fn walk_pace(&self) -> f32 {
        21.0 * FPS as f32 / self.ticks_per_frame() as f32
    }

    /// Whether the player can stand at (x, y): not in the water or the
    /// river (the bridge is fine), and not inside a building - walking
    /// round behind one is fine
    pub fn walkable(&self, x: f32, y: f32) -> bool {
        let (tx, ty) = (x.round() as i32, y.round() as i32);
        let built = self.map.is_occupied(tx, ty)
            || self.buildings.iter().any(|(bx, by, _)| bx.round() as i32 == tx && by.round() as i32 == ty);
        !built && !matches!(self.map.get(tx, ty), None | Some(Tile::Water) | Some(Tile::River))
    }


    /// Each seat's facing and walking, from how far its character moved
    /// since last frame (walking, an approach, a meeting - any movement)
    fn track_poses(&mut self) {
        for i in 0..self.seats.len() {
            let now = self.player_pos_at(i);
            let seat = &mut self.seats[i];
            let (dx, dy) = seat.last_pos.map(|(x, y)| (now.0 - x, now.1 - y)).unwrap_or((0.0, 0.0));
            seat.walking = dx.abs() + dy.abs() > 1e-4;
            if seat.walking {
                seat.facing = facing_on_screen(dx, dy);
            }
            seat.last_pos = Some(now);
        }
        // villagers, the rivals and the travelling figures (minstrel,
        // pedlar, circus, ink-boiler, the rare visitors) all walk a cycle
        let movers: Vec<Entity> = self.villagers.iter().chain(&self.rivals).copied()
            .chain(self.minstrel.as_ref().map(|m| m.e))
            .chain(self.pedlar.as_ref().map(|m| m.e))
            .chain(self.circus.as_ref().map(|c| c.e))
            .chain(self.ink_boiler.as_ref().map(|b| b.e))
            .chain(self.rare.iter().filter_map(|r| r.e)).collect();
        self.villager_poses.retain(|e, _| movers.contains(e));
        for &v in &movers {
            let Some(p) = self.world.get_component::<Pos>(v).map(|p| (p.x, p.y)) else { continue };
            let pose = self.villager_poses.entry(v).or_insert((p, 2, false));
            let (dx, dy) = (p.0 - pose.0.0, p.1 - pose.0.1);
            pose.2 = dx.abs() + dy.abs() > 1e-4;
            if pose.2 { pose.1 = facing_on_screen(dx, dy); }
            pose.0 = p;
        }
    }


    /// Where the courier is on the road between the villages: there and
    /// back along `traveller_path`, at a riding pace
    pub fn traveller_pos(&self) -> Option<(f32, f32)> { self.traveller_pos_at(self.frame_n) }

    /// Where the courier is at frame `frame`
    fn traveller_pos_at(&self, frame: u32) -> Option<(f32, f32)> {
        let n = self.traveller_path.len();
        if n < 2 || self.traveller_path.is_empty() { return None; }
        let leg = (n - 1) as f32;
        let mut s = (frame as f32 * TRAVELLER_SPEED) % (leg * 2.0);
        if s > leg { s = leg * 2.0 - s; }
        let (i, f) = (s as usize, s.fract());
        let (a, b) = (self.traveller_path[i], self.traveller_path[(i + 1).min(n - 1)]);
        Some((a.0 as f32 + (b.0 - a.0) as f32 * f, a.1 as f32 + (b.1 - a.1) as f32 * f))
    }


    /// One frame of game time: the sim ticks every few frames, and between
    /// ticks villagers keep strolling
    pub fn update(&mut self) {
        // the whole update, poses and facing included, so it matches the bench's update stage
        let _frame = crate::infra::perf::scope("total.frame");
        self.advance_frame();
        // after every move this frame (the player's command, the sim, the
        // strolling), so the model's poses match its positions
        let _poses = crate::infra::perf::scope("total.poses");
        self.track_poses();
        // and those standing for something turn to face it
        self.face_routines();
    }

    fn advance_frame(&mut self) {
        let ticks_per_frame = self.ticks_per_frame();
        let walk_pace = 21.0 * FPS as f32 / ticks_per_frame as f32;
        // the soundscape resets each frame: cues live exactly one read
        self.sfx.clear();
        // sim advance: one tick = one day, throttled by frame count
        self.frame_n += 1;
        let mut ticked = false;
        if !self.paused && self.frame_n % ticks_per_frame == 0 {
            self.tick += 1;
            crate::timed!("total.ecs", self.sched.tick(&mut self.world));
            ticked = true;
        }
        // the town's business each day: work, errands, taxes
        if ticked { crate::timed!("total.day", self.livelihood_tick(self.tick)); }
        // where everyone stood, so no one ends the frame inside a wall or a house
        let people: Vec<Entity> = self.villagers.iter().chain(&self.rivals).copied().collect();
        let stood: Vec<Option<Pos>> = people.iter().map(|e| self.world.get_component::<Pos>(*e).copied()).collect();
        // the country's folk keep to their own granary and their own
        // ground (a carter at the market eats there): None for the village's
        let country_homes: Vec<Option<(Entity, Option<(f32, f32)>)>> = self.villagers.iter().map(|&v|
            self.site_of(v).map(|_| (self.store_of(v), self.home_ground(v)))).collect();
        let Game { world, villagers, log, toasts, .. } = self;
        let tick = self.tick;
        // weather: log when rain or snow starts and stops; a rainbow follows rain
        let now = world.get_component::<climate::Climate>(self.climate).and_then(precip_kind);
        if now != self.precip {
            match (self.precip, now) {
                (_, Some(k)) => log.push(format!("t{}: {} begins", tick, k.label())),
                (Some(k), None) if !k.is_snow() => {
                    log.push(format!("t{}: the rain stops - a rainbow!", tick));
                    self.rainbow_from = Some(self.frame_n);
                }
                (Some(_), None) => log.push(format!("t{}: the snow stops", tick)),
                (None, None) => {}
            }
            if now.is_some() { self.rainbow_from = None; }
            self.precip = now;
        }
        // villagers separate if too close (no clustering). Positions and
        // who-may-yield are snapshotted once for the whole pair pass - the
        // roster keeps every grave, so per-pair component lookups grew
        // quadratically over a long game. Pushes write back into the
        // snapshot too, so later pairs read exactly what the old per-pair
        // lookups would have - the pass is faster, not different.
        let _sep = crate::infra::perf::scope("sim.separate");
        let mut pos_now: Vec<Option<Pos>> = villagers.iter().map(|v| world.get_component::<Pos>(*v).copied()).collect();
        let yields: Vec<bool> = villagers.iter().map(|v| makes_room(world, *v)).collect();
        for i in 0..villagers.len() {
            for j in (i+1)..villagers.len() {
                // two fixtures (graves, the held) never yield to each other
                if !yields[i] && !yields[j] { continue; }
                let (Some(a), Some(b)) = (pos_now[i], pos_now[j]) else { continue };
                let dx = b.x - a.x; let dy = b.y - a.y;
                let d2 = dx*dx + dy*dy;
                if d2 < 9.0 && d2 > 0.001 {
                    let d = d2.sqrt();
                    let push = (3.0 - d) * 0.4 / FPS as f32 * walk_pace; // eased apart over a few frames
                    let nx = dx / d * push; let ny = dy / d * push;
                    // a grave stays put; the living step around it. Nor is
                    // anyone shoved who's going for food: the push reaches 3
                    // tiles but eating needs 1, so two hungry villagers at
                    // the store pushed each other out of reach and starved
                    if yields[i] {
                        if let Some(p) = world.get_component_mut::<Pos>(villagers[i]) { p.x -= nx; p.y -= ny; }
                        if let Some(p) = &mut pos_now[i] { p.x -= nx; p.y -= ny; }
                    }
                    if yields[j] {
                        if let Some(p) = world.get_component_mut::<Pos>(villagers[j]) { p.x += nx; p.y += ny; }
                        if let Some(p) = &mut pos_now[j] { p.x += nx; p.y += ny; }
                    }
                }
            }
        }

        // idle villagers keep to the day's plan (session::routine::amble,
        // after the errands walk) - no more drifting a step each way and back
        drop(_sep);

        // rivals keep to their own village: idle drift is a random walk, so
        // without a pull home they'd wander across the river into ours
        if !self.paused {
            for (r, (hx, hy)) in self.rivals.iter().zip(&self.rival_homes) {
                if let Some(p) = world.get_component_mut::<Pos>(*r) {
                    let (dx, dy) = (hx - p.x, hy - p.y);
                    let d = (dx * dx + dy * dy).sqrt();
                    if d > 3.0 {
                        let step = 0.6 / ticks_per_frame as f32; // as fast as villagers stroll
                        p.x += dx / d * step;
                        p.y += dy / d * step;
                    }
                }
            }
        }

        // and so do ours: left to the random walk, a villager drifted east
        // until the rivals' food store was nearer than ours, then ate there
        // and lived in their village, off the edge of the view
        let _pull = crate::infra::perf::scope("sim.pull_home");
        if !self.paused {
            let village_home = world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y));
            // our own store keeps them home: while it still serves, no one
            // should be setting off for the rivals' granary just because
            // wandering took them far enough east that it was the nearer one
            for (vi, v) in villagers.iter().enumerate() {
                if crate::game::dungeon::unavailable(world, *v) { continue; }
                let (store, village_home) = country_homes.get(vi).copied().flatten()
                    .unwrap_or((self.market_food, village_home));
                // the village table is the granary AND the fields: a village
                // villager's store is whichever stocked one stands closest,
                // while a countryman's is still his own settlement's granary
                let vpos = world.get_component::<Pos>(*v).map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
                let (store, market_has) = if store == self.market_food {
                    let stocked = |f: Entity| world.get_component::<villager::FoodSource>(f)
                        .map(|fs| fs.servings > 0).unwrap_or(false);
                    let nearest = std::iter::once(self.market_food).chain(self.crop_fields.iter().copied())
                        .filter(|&f| stocked(f))
                        .min_by_key(|&f| world.get_component::<Pos>(f)
                            .map(|fp| ((fp.x - vpos.0).abs() + (fp.y - vpos.1).abs()) as i32).unwrap_or(i32::MAX));
                    (nearest.unwrap_or(store), nearest.is_some())
                } else {
                    (store, world.get_component::<villager::FoodSource>(store)
                        .map(|f| f.servings > 0).unwrap_or(false))
                };
                let vdata = world.get_component::<Villager>(*v);
                let seeking = vdata.map(|x| x.activity == Activity::SeekingFood).unwrap_or(false);
                let target_food = vdata.and_then(|x| x.target_food);
                if seeking && market_has && target_food != Some(store) {
                    if let Some(vill) = world.get_component_mut::<Villager>(*v) {
                        vill.target_food = Some(store);
                    }
                }
                // a seeking villager heads for the food source SeekFoodSystem
                // actually picked (which may be the rival granary once ours
                // runs dry), not always the village store - otherwise this
                // wall-avoidance pass fights the daily system's choice and
                // walks a hungry villager back and forth, never arriving
                let dest = if seeking {
                    if market_has { village_home } else {
                        target_food.and_then(|f| world.get_component::<Pos>(f)).map(|p| (p.x, p.y))
                            .or(village_home)
                    }
                } else {
                    // the herb-wife's pull is her own hut in the meadow,
                    // not the market square
                    if livelihood::has_role(world, *v, TownRole::Herbologist) {
                        self.herb_hut.and_then(|h| world.get_component::<Pos>(h)).map(|p| (p.x, p.y)).or(village_home)
                    } else {
                        village_home
                    }
                };
                let food_tile = dest.map(|(hx, hy)| (hx.round() as i32, hy.round() as i32));
                // the day's plan and the errands walk their own roads (the
                // creek, the stones, the flock): the pull is only for
                // strays with neither
                let planned = world.get_component::<routine::Routine>(*v)
                    .map(|r| r.day + 1 >= tick && r.spot != routine::Spot::Here).unwrap_or(false)
                    || self.travels.iter().any(|t| t.who == *v);
                let Some(p) = world.get_component_mut::<Pos>(*v) else { continue };
                let Some((hx, hy)) = dest else { continue };
                let (dx, dy) = (hx - p.x, hy - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                // a hungry villager more than eating distance away walks the
                // road to the store - the sim's straight steps can pen them
                // against the town wall or a building for good, so take the
                // way round (an idle stray comes home the same way)
                let heading_home = seeking && dx.abs() + dy.abs() > 0.5
                    || !seeking && !planned && d > VILLAGE_RADIUS;
                if heading_home {
                    // past every errand's reach the nudge below would walk
                    // them home a tile a day for months: once a day, put a
                    // stray this far out on the road itself - the same
                    // walk_path the errands take, just longer. Not one
                    // already walking: a second path would drag them back
                    // toward its stale waypoints every frame
                    if dx.abs() + dy.abs() > 30.0 && self.frame_n % ticks_per_frame == 0
                        && !self.travels.iter().any(|t| t.who == *v) {
                        if let Some(t) = food_tile {
                            let from = (p.x.round() as i32, p.y.round() as i32);
                            if let Some(mut path) = self.map.walk_path(from, t) {
                                path.push(t);
                                self.travels.push(Travel { who: *v, path, goal: Goal::Wander });
                                continue;
                            }
                        }
                    }
                    let step = 1.0 / ticks_per_frame as f32;
                    let (nx, ny) = (p.x + dx / d * step, p.y + dy / d * step);
                    if self.map.passable(nx.round() as i32, ny.round() as i32) {
                        p.x = nx; p.y = ny;
                    } else if let Some(path) = food_tile.and_then(|t| self.map.walk_path((p.x.round() as i32, p.y.round() as i32), t)) {
                        if let Some(&(qx, qy)) = path.first() {
                            let (ddx, ddy) = (qx as f32 - p.x, qy as f32 - p.y);
                            let dd = (ddx * ddx + ddy * ddy).sqrt().max(1e-4);
                            let (sx, sy) = (p.x + ddx / dd * step, p.y + ddy / dd * step);
                            // a diagonal step can clip a wall corner: the
                            // wall-revert then bounces them back every frame
                            // and they never come home - take one axis at a
                            // time so the way round actually clears
                            if self.map.passable(sx.round() as i32, sy.round() as i32) { p.x = sx; p.y = sy; }
                            else if self.map.passable(sx.round() as i32, p.y.round() as i32) { p.x = sx; }
                            else if self.map.passable(p.x.round() as i32, sy.round() as i32) { p.y = sy; }
                            // wedged in a corner where neither axis clears:
                            // step onto the path tile outright - walk_path
                            // only lays passable tiles, so it can't strand
                            else { p.x = qx as f32; p.y = qy as f32; }
                        }
                    } else {
                        // no way home from here at all (they ate somewhere
                        // hemmed in): step onto the passable tile nearest
                        // home if there is one; if not, keep walking anyway
                        // (they got in, they can get out the same way)
                        let (cx, cy) = (p.x.round() as i32, p.y.round() as i32);
                        let escape = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (cx + dx, cy + dy)))
                            .filter(|&q| q != (cx, cy) && self.map.passable(q.0, q.1))
                            .min_by_key(|&q| (q.0 - hx.round() as i32).abs() + (q.1 - hy.round() as i32).abs());
                        match escape {
                            Some((qx, qy)) => {
                                let (ddx, ddy) = (qx as f32 - p.x, qy as f32 - p.y);
                                let dd = (ddx * ddx + ddy * ddy).sqrt().max(1e-4);
                                p.x += ddx / dd * step; p.y += ddy / dd * step;
                            }
                            None => { p.x = nx; p.y = ny; }
                        }
                    }
                }
            }
        }

        drop(_pull);

        // death check: health <= 0 -> grave + log
        let _checks = crate::infra::perf::scope("sim.frame_checks");
        let died_on = &mut self.died_on;
        for (i, v) in villagers.iter().enumerate() {
            if let Some(h) = world.get_component::<Health>(*v) {
                if h.condition <= 0.0 {
                    toasts.push_at((format!("{} has died.", vill_name(i)), tick + 60), toast_spot(world, *v));
                    log.push(format!("t{}: {} died.", tick, vill_name(i)));
                    world.remove_component::<Health>(*v);
                    world.add_component(*v, villager::Dead);
                    died_on[i] = Some(tick);
                    note_story(&mut self.tales, &mut self.tales_told, log, i, tick,
                        crate::sim::story::EventKind::Death, format!("{} died.", vill_name(i)));
                }
            }
        }


        // gossip: rumors spread villager-to-villager
        if ticked && tick % 40 == 0 {
            let rumors = [
                "word spreads of a stranger organizing",
                "the miller is skimming the flour",
                "someone saw the lord's tax collector",
                "the guild merchants grow bold",
                "an old wound reopens between families",
            ];
            let a = (tick as usize / 40) % NAMES.len();
            let b = (tick as usize / 40 + 2) % NAMES.len();
            log.push(format!("t{}: {} tells {} - {}", tick, vill_name(a), vill_name(b), rumors[tick as usize / 40 % 5]));
        }

        // a traveller brings la grippe; it brews a while, then the
        // neighbours may catch it
        if ticked && tick == FLU_DAY {
            let d = world.create_entity();
            world.add_component(d, Disease::new(DiseaseKind::Influenza, villagers[2], 0.45));
            toasts.push_at((format!("A traveller brings la grippe - {} falls ill", vill_name(2)), tick + 40),
                toast_spot(world, villagers[2]));
        }
        // log who falls ill and who recovers
        let sick_now: Vec<Option<String>> = villagers.iter().map(|v| sickness_name(world, *v)).collect();
        for (i, (now, was)) in sick_now.iter().zip(self.was_sick.iter_mut()).enumerate() {
            match (now, was.as_ref()) {
                (Some(name), None) => {
                    log.push(format!("t{}: {} is taken with {}", tick, vill_name(i), name));
                    note_story(&mut self.tales, &mut self.tales_told, log, i, tick,
                        crate::sim::story::EventKind::Illness, format!("{} is taken with {}", vill_name(i), name));
                }
                (None, Some(old)) => {
                    // a disease gone because it took them isn't a recovery
                    if self.died_on.get(i).and_then(|d| *d).is_none() {
                        log.push(format!("t{}: {} recovered from {}", tick, vill_name(i), old));
                        note_story(&mut self.tales, &mut self.tales_told, log, i, tick,
                            crate::sim::story::EventKind::Recovery, format!("{} recovered from {}", vill_name(i), old));
                    }
                }
                _ => {}
            }
            *was = now.clone();
        }

        // detect events -> toasts
        let gc = world.query::<Guild>().len();
        if gc > self.last_guild_count {
            toasts.push(("A guild forms in the village!".into(), tick + 40));
            log.push(format!("t{}: The Merchant Guild formed!", tick));
        }
        self.last_guild_count = gc;

        // expire speech bubbles and toasts
        toasts.retain(|(_, until, _)| tick < *until);
        for (e, before) in people.iter().zip(stood) {
            let (Some(before), Some(now)) = (before, self.world.get_component::<Pos>(*e).copied()) else { continue };
            let blocked = |p: &Pos| self.map.is_occupied(p.x.round() as i32, p.y.round() as i32);
            if !blocked(&now) || blocked(&before) { continue; }
            // sliding beats freezing: pressed against a building or a wall,
            // keep the part of the step that isn't blocked (a seeker frozen
            // against the town wall used to starve there for good)
            let slide = [Pos { x: now.x, y: before.y }, Pos { x: before.x, y: now.y }]
                .into_iter().find(|p| !blocked(p));
            *self.world.get_component_mut::<Pos>(*e).unwrap() = slide.unwrap_or(before);
        }
        drop(_checks);

        if !self.paused {
            let _tail = crate::infra::perf::scope("sim.frame_tail");
            for w in self.works.iter_mut() {
                let laid = w.road_laid();
                w.frames += 1;
                for &(x, y) in &w.road[laid..w.road_laid()] { self.map.pave(x, y); }
            }
            self.works.retain(|w| w.risen() < 1.0);
            crate::timed!("sim.advance_meeting", self.advance_meeting());
            crate::timed!("sim.advance_player_talks", self.advance_player_talks());
            crate::timed!("sim.advance_travels", self.advance_travels());
            crate::timed!("sim.snowball_frame", self.snowball_frame());
            // and the jobs folk asked of the Wanderer: picked up where
            // they lie, handed over where they're bound
            self.quest_frame();
            // and those with nowhere to walk keep to the day's plan
            self.amble();
            crate::timed!("sim.advance_minstrel", self.advance_minstrel());
            crate::timed!("sim.advance_pedlar", self.advance_pedlar());
            crate::timed!("sim.advance_circus", self.advance_circus());
            crate::timed!("sim.advance_ink_boiler", self.advance_ink_boiler());
            // and the rare events walk their roads
            crate::timed!("sim.advance_rare", self.advance_rare());
            // the flock grazes: each animal drifts to a spot of its own
            // picking round the day's grazing point, at its own pace, and
            // only onto ground it can stand on - no walls, no water
            for &a in self.flock.clone().iter() {
                let Some(p) = self.world.get_component::<Pos>(a).copied() else { continue };
                let (tx, ty, due) = self.flock_wander.get(&a).copied().unwrap_or((p.x, p.y, 0));
                let arrived = (tx - p.x).powi(2) + (ty - p.y).powi(2) < 0.06;
                if arrived || self.frame_n >= due {
                    // a fresh spot of grass: keep it to the meadow, off the
                    // walls and buildings (a couple of tries, then it stands)
                    for _ in 0..8 {
                        let (cx, cy) = (self.graze_at.0 + self.rng.around(8) as f32 * 0.5,
                                        self.graze_at.1 + self.rng.around(8) as f32 * 0.5);
                        if self.walkable(cx, cy) {
                            let next = self.frame_n + FPS + self.rng.below(6 * FPS as i32) as u32;
                            self.flock_wander.insert(a, (cx, cy, next));
                            break;
                        }
                    }
                }
                let (tx, ty, _) = self.flock_wander.get(&a).copied().unwrap_or((p.x, p.y, 0));
                let (dx, dy) = (tx - p.x, ty - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d < 0.02 { continue; }
                // each ambles its own pace; a wall in the way ends the wander
                let step = (0.005 + (a % 5) as f32 * 0.002 + self.rng.below(4) as f32 * 0.001).min(d);
                let (nx, ny) = (p.x + dx / d * step, p.y + dy / d * step);
                if self.walkable(nx, ny) {
                    if let Some(pp) = self.world.get_component_mut::<Pos>(a) { pp.x = nx; pp.y = ny; }
                } else {
                    self.flock_wander.insert(a, (p.x, p.y, 0));
                }
            }
            // the yard hens scratch a slow wander round their own door -
            // on ground they can stand on, not through the walls
            for (hi, &a) in self.hens.iter().enumerate() {
                let hp = self.world.get_component::<Livestock>(a)
                    .and_then(|l| self.world.get_component::<Pos>(l.fold).copied());
                let (Some(hp), Some(p)) = (hp, self.world.get_component::<Pos>(a).copied()) else { continue };
                let t = self.frame_n as f32 / 50.0 + hi as f32 * 1.7;
                let (nx, ny) = (p.x + (hp.x + t.sin() * 1.4 + (hi % 3) as f32 * 0.4 - p.x) * 0.01,
                                p.y + (hp.y + t.cos() * 1.0 + (hi % 2) as f32 * 0.4 - p.y) * 0.01);
                if self.walkable(nx, ny) {
                    if let Some(pp) = self.world.get_component_mut::<Pos>(a) { pp.x = nx; pp.y = ny; }
                }
            }
            drop(_tail);
        }
        for seat in 0..self.seats.len() {
            self.update_ctx_actions_at(seat);
            self.advance_approach_at(seat);
        }
        let frame = self.frame_n;
        self.bubbles.retain(|(_, _, until, _, _)| frame < *until);
        // the log is a rolling window - the oldest lines drop in blocks
        if self.log.len() > LOG_MAX {
            let dropped = self.log.len() - LOG_TRIM_TO;
            self.log.drain(..dropped);
            self.log_dropped += dropped;
        }
        // movement is still processed while paused, but relationships and
        // last_chat shouldn't change while the day isn't advancing
        if !self.paused { crate::timed!("sim.chat_in_passing", self.chat_in_passing()); }
    }


    // ---------- multiplayer: seats, claims and the agent surface ----------


}


/// Run the game until the house meeting under way is over
#[cfg(test)]
fn finish_meeting(game: &mut Game) {
    for _ in 0..120 * FPS {
        if game.meeting.is_none() { return; }
        game.update();
    }
    panic!("the meeting never ended");
}
