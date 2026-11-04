//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

pub mod model;
pub mod net;
pub mod driver;
pub mod record;
pub mod events;
pub mod probe;
pub mod stage;
pub mod dialogue;
pub mod routine;
pub mod country;
pub mod render;
pub mod kpi;
pub mod setup;
#[cfg(test)]
mod tests;

pub use model::*;
pub use net::*;
pub use driver::*;
pub use record::*;
pub use render::*;

use crate::engine::ecs::{Entity, Scheduler, World};
use crate::engine::render::Buffer;
use crate::engine::sprite::{Sprite, Assets};
use crate::ui::font::{draw_text, draw_text_outlined, draw_text_scaled};
use crate::game::terrain::Terrain;
use crate::game::tilemap::{Layout, Tile, TileMap, WallPiece, NEIGHBOURS, noise};
use crate::game::villager::{self, Hunger, Villager, Activity, Pos, Smell};
use crate::game::pets::{self, Species, PetRole, Pet};
use crate::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest, can_recruit, TalkCount, recruit_influence_needed, RECRUIT_INFLUENCE};
use crate::game::guilds::{GuildFormationSystem, Merchant, Guild};
use crate::sim::climate::{self, SeasonSystem, Season};
use crate::sim::precipitation::{precip_kind, PrecipKind};
use crate::ui::weather_fx;
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

pub const W: usize = 1440;
pub const H: usize = 960;
pub const MAP_W: usize = 1080;
pub const UI_X: usize = 1100;
const TILE_W: i32 = 28;
const TILE_H: i32 = 14;
/// Influence at which the village recognizes the player as its leader (the
/// win) - the same goal as the text game
const WIN_INFLUENCE: f32 = crate::game::player::LEAD_INFLUENCE;

pub fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }

fn draw_rect(buf: &mut Buffer, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
    buf.rect(x, y, w, h, r, g, b);
}

/// Convert world tile coords to screen isometric position
/// Point the zoom scales about: the centre of the map view, so zooming
/// keeps what you're looking at on screen
const ZOOM_PIVOT: (f32, f32) = (MAP_W as f32 / 2.0, H as f32 / 2.0);

pub fn iso_to_screen_cam(tx: f32, ty: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (i32, i32) {
    // position at zoom 1, then scaled about the view centre
    let bx = MAP_W as f32 / 2.0 + (tx - ty) * TILE_W as f32 + cam_x as f32;
    let by = -60.0 + (tx + ty) * TILE_H as f32 + cam_y as f32;
    let sx = ZOOM_PIVOT.0 + (bx - ZOOM_PIVOT.0) * zoom;
    let sy = ZOOM_PIVOT.1 + (by - ZOOM_PIVOT.1) * zoom;
    (sx.round() as i32, sy.round() as i32)
}

/// Conservative tile bounds whose ground sprites can reach the viewport.
/// The screen margins match draw_ground's sprite-overhang cull, then one tile
/// is added to absorb rounding at fractional zooms.
fn visible_tile_bounds(cam_x: i32, cam_y: i32, zoom: f32, width: usize, height: usize,
    map_width: i32, map_height: i32) -> (i32, i32, i32, i32) {
    let (mx, my) = ((2.0 * TILE_W as f32 * zoom).ceil(), (4.0 * TILE_H as f32 * zoom).ceil());
    let corners = [(-mx, -my), (width as f32 + mx, -my), (-mx, height as f32 + my),
        (width as f32 + mx, height as f32 + my)];
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (sx, sy) in corners {
        let u = ((sx - ZOOM_PIVOT.0) / zoom - cam_x as f32) / TILE_W as f32;
        let v = ((sy - ZOOM_PIVOT.1) / zoom + 60.0 - cam_y as f32) / TILE_H as f32;
        let (tx, ty) = ((u + v) * 0.5, (v - u) * 0.5);
        min_x = min_x.min(tx); min_y = min_y.min(ty);
        max_x = max_x.max(tx); max_y = max_y.max(ty);
    }
    let (x0, y0, x1, y1) = ((min_x.floor() as i32 - 40).max(0),
        (min_y.floor() as i32 - 40).max(0), (max_x.ceil() as i32 + 40).min(map_width - 1),
        (max_y.ceil() as i32 + 40).min(map_height - 1));
    (x0, y0, x1, y1)
}

/// Can nothing standing on screen point (sx, sy) reach the map view? A
/// piece draws up from its tile (tall towers ~260px at zoom 1), a little
/// below it, and at most ~150px to either side
fn off_view(sx: i32, sy: i32, zoom: f32) -> bool {
    let z = |v: f32| (v * zoom).round() as i32;
    sx < -z(150.0) || sx > MAP_W as i32 + z(150.0) || sy < -z(40.0) || sy > H as i32 + z(300.0)
}


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


/// Gossip lines shown in the log at once; older gossip is hidden so it
/// can't push real events (illness, joins, deaths, guilds) out of view
const LOG_GOSSIP_SHOWN: usize = 2;

/// Is this log entry idle gossip ("X tells Y - ...")?
fn is_gossip(entry: &str) -> bool { entry.contains(" tells ") }

/// Weather changes shown in the log at once (the HUD names today's weather)
const LOG_WEATHER_SHOWN: usize = 1;

/// Is this log entry a routine weather change ("drizzle begins", "the rain stops")?
fn is_weather_note(entry: &str) -> bool {
    entry.ends_with(" begins") || entry.contains("the rain stops") || entry.contains("the snow stops")
}

/// Town business shown in the log at once: promotions, the minstrel, the
/// tax man's visits - nice to know, but not worth pushing illness and
/// deaths out of view
const LOG_TOWN_SHOWN: usize = 3;

/// How many log lines the session keeps at most. A line or three lands
/// each day, so uncapped the log grew without bound over a long game -
/// and every frame's model used to clone all of it. The window only
/// shows the recent end; harnesses and tests read days or weeks back,
/// far under this floor.
const LOG_MAX: usize = 16384;

/// Overflow trims down to this, so the trim is a rare memmove instead of
/// a per-line churn.
const LOG_TRIM_TO: usize = 12288;

/// Is this log entry routine town business?
fn is_town_note(entry: &str) -> bool {
    entry.contains(" is now a ") || entry.contains("has come to town")
        // the country's carts and stewards on the road
        || entry.contains("'s cart ") || entry.contains("to fetch provisions")
        || entry.contains("takes his cut") || entry.contains("Organizer discovers issue")
        || entry.contains("the fold") || entry.contains("a fox")
        || entry.contains("brought") || entry.contains(" treats ")
        || entry.contains("skeps") || entry.contains("swarm")
        || entry.contains("takes up") || entry.contains("shuttered")
        || entry.contains("the talk turns") || entry.contains("cold shoulder")
        || entry.contains("boycott") || entry.contains("a crowd at")
        || entry.contains("the village gathers") || entry.contains("pays triple")
        || entry.contains("a pedlar") || entry.contains("pedlar's moved on")
        || entry.contains("rides over the walls") || entry.contains("circus")
        || entry.contains("bailiff's writ") || entry.contains("a summons")
        || entry.contains("winter") || entry.contains("thin tempers")
        || entry.contains("gets its colors")
}

/// The log entries to show, newest first: every event, but only the latest
/// LOG_GOSSIP_SHOWN pieces of gossip and the latest weather change.
/// The moments that matter - sickness, death, an exile - pin to the top:
/// routine village business scrolls below them no matter how busy it gets.
fn visible_log(log: &[String]) -> Vec<&String> {
    let (mut gossip, mut weather, mut town) = (0, 0, 0);
    let filtered: Vec<&String> = log.iter().rev().filter(|e| {
        if is_gossip(e) { gossip += 1; return gossip <= LOG_GOSSIP_SHOWN; }
        if is_weather_note(e) { weather += 1; return weather <= LOG_WEATHER_SHOWN; }
        if is_town_note(e) { town += 1; return town <= LOG_TOWN_SHOWN; }
        true
    }).collect();
    // stable partition: important entries first (already newest-first),
    // then the rest - a quiet death still outranks a busy skep-day
    let (important, rest): (Vec<_>, Vec<_>) = filtered.into_iter()
        .partition(|e| is_important(e));
    important.into_iter().chain(rest).collect()
}

/// An entry the player should still see rows later: illness and loss,
/// the exile's road, the discovery told
fn is_important(entry: &str) -> bool {
    entry.contains("is taken with") || entry.contains("recovered") || entry.contains("died")
        || entry.contains("driven out") || entry.contains("run off")
        || entry.contains("whispers it about") || entry.contains("stocks")
        || entry.contains("born") || entry.contains("dead")
        || entry.contains("taken for questioning")
}

/// Split a log entry stored as "t120: message" into the day the player
/// sees ("day 120", matching the HUD) and the message
fn log_parts(entry: &str) -> (Option<String>, &str) {
    if let Some((stamp, msg)) = entry.split_once(": ") {
        if let Some(day) = stamp.strip_prefix('t').filter(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit())) {
            return (Some(format!("day {}", day)), msg);
        }
    }
    (None, entry)
}

/// Colour for a log entry: deaths red, guild events gold, joins green,
/// weather/illness cyan, idle gossip dimmed so real events stand out
fn log_colour(entry: &str) -> (u8, u8, u8) {
    if entry.contains("died") || entry.contains("dead") { (255, 100, 100) }
    else if is_gossip(entry) { (140, 140, 160) } // gossip, even when it mentions the guild
    else if entry.contains("Guild") || entry.contains("guild") { (255, 215, 0) }
    else if entry.contains("joined") { (120, 255, 120) }
    // whole words only: "influence" contains "flu" and "grain" contains "rain"
    else if entry.split(|c: char| !c.is_alphabetic()).any(|w| matches!(w, "storm" | "rain" | "raining" | "drizzle" | "showers" | "snow" | "rainbow" | "grippe" | "scarlatina" | "gripes" | "sweats" | "consumption" | "fever" | "cholera" | "ague")) { (120, 200, 255) }
    else { (210, 210, 210) }
}


// Minimap: the same isometric projection as the map, shrunk to fit a box at
// the bottom of the side panel (so up on the minimap is up on screen)
pub const MM_W: i32 = 320;
pub const MM_H: i32 = 160;
pub const MM_LEFT: i32 = UI_X as i32 + 10;
pub const MM_TOP: i32 = 660;

/// Minimap px per tile step along the x/y diagonal: as small as the box
/// needs so the whole map fits whatever its size
fn mm_k(w_t: f32, h_t: f32) -> f32 {
    let span = (w_t + h_t - 2.0).max(1.0);
    ((MM_W - 24) as f32 / span).min((MM_H - 16) as f32 * 2.0 / span)
}


/// Minimap position of a world tile
fn tile_to_minimap(tx: f32, ty: f32, k: f32) -> (f32, f32) {
    (MM_LEFT as f32 + MM_W as f32 / 2.0 + (tx - ty) * k, MM_TOP as f32 + 2.0 + (tx + ty) * k / 2.0)
}

/// The tile under minimap pixel (mx, my), if it's on the minimap's tiles
/// (inverse of tile_to_minimap)
/// Where a minimap click centres the camera, for any map (sim's or a model's)
pub fn minimap_look_map(map: &TileMap, mx: i32, my: i32) -> Option<(i32, i32)> {
    minimap_tile(mx as f32, my as f32, map.width as f32, map.height as f32)
        .map(|(tx, ty)| camera_on(tx, ty))
}

pub fn minimap_tile(mx: f32, my: f32, w_t: f32, h_t: f32) -> Option<(f32, f32)> {
    let k = mm_k(w_t, h_t);
    let (cx, top) = (MM_LEFT as f32 + MM_W as f32 / 2.0, MM_TOP as f32 + 2.0);
    let diff = mx - cx;
    let sum = (my - top) * 2.0;
    let (tx, ty) = ((sum / k + diff / k) / 2.0, (sum / k - diff / k) / 2.0);
    let inside = mx >= MM_LEFT as f32 && mx < (MM_LEFT + MM_W) as f32 && my >= MM_TOP as f32 && my < (MM_TOP + MM_H) as f32;
    (inside && tx >= 0.0 && ty >= 0.0 && tx < w_t && ty < h_t).then_some((tx, ty))
}

/// The camera offset that puts tile (tx, ty) in the middle of the map view
/// (at any zoom: the zoom pivots about the view's centre)
pub fn camera_on(tx: f32, ty: f32) -> (i32, i32) {
    (-((tx - ty) * TILE_W as f32).round() as i32, (ZOOM_PIVOT.1 + 60.0 - (tx + ty) * TILE_H as f32).round() as i32)
}


/// Whether the crowd-separation pass may push this villager aside: not the
/// dead, and not someone on their way to food
fn makes_room(world: &World, v: Entity) -> bool {
    !villager::is_dead(world, v)
        && world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none()
        && world.get_component::<Villager>(v).map(|x| x.activity != Activity::SeekingFood).unwrap_or(true)
        && !routine::holds_slot(world, v)
}

/// A villager's attachment style as a player would put it: the card showed
/// the psychology term ("Style: Disorganized"), which reads like a fault
fn nature_word(style: crate::sim::psychology::AttachmentStyle) -> &'static str {
    use crate::sim::psychology::AttachmentStyle::*;
    match style {
        Secure => "steady and trusting",
        Anxious => "anxious, needs reassurance",
        Avoidant => "keeps to themselves",
        Disorganized => "wary and unpredictable",
    }
}

/// How fed a villager is, in words. `Hunger` stores fullness (100 = just
/// ate, 0 = starving), so the raw number reads backwards as "hunger".
fn fullness_word(fullness: f32) -> &'static str {
    if fullness >= 70.0 { "full" }
    else if fullness >= 40.0 { "peckish" }
    else if fullness >= 15.0 { "hungry" }
    else { "starving" }
}

/// What a villager is doing, in plain words
fn activity_text(a: Activity) -> &'static str {
    match a {
        Activity::Idle => "idle",
        Activity::SeekingFood => "looking for food",
        Activity::Eating => "eating",
    }
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
/// Gold the Invest action spends
const INVEST_GOLD: f32 = 10.0;
/// Gold a new house costs
const BUILD_COST: f32 = 30.0;
/// Tile the windmill stands on (it's animated, so drawn apart from `buildings`)
const MILL_TILE: (f32, f32) = (10.0, 9.0);
/// The church: on the market square's north side, facing the cross
const CHURCH_TILE: (i32, i32) = (16, 13);
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
fn town_streets(wall: (i32, i32, i32, i32)) -> Vec<(i32, i32)> {
    let (hx, hy) = (16, 16);
    let mut out = vec![];
    for y in wall.1 + 1..wall.3 {
        for x in wall.0 + 1..wall.2 {
            let on_grid = (x - hx).rem_euclid(STREET_EVERY) == 0 || (y - hy).rem_euclid(STREET_EVERY) == 0;
            if on_grid && (x, y) != CHURCH_TILE
                && (x, y) != (MILL_TILE.0 as i32, MILL_TILE.1 as i32) {
                out.push((x, y));
            }
        }
    }
    out
}

/// The building plots inside the wall: open cells fronting a street (the
/// block's inner cells stay yards), nearest the market square first
fn town_plots(map: &TileMap, wall: (i32, i32, i32, i32)) -> Vec<(i32, i32)> {
    let (hx, hy) = (16, 16);
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

/// Walking over to someone to talk to or recruit them
pub struct Approach {
    pub who: usize,              // villager index
    pub action: usize,           // the action button to take on arrival (Talk or Recruit)
    pub path: Vec<(i32, i32)>,   // tiles still to walk, their tile last
    pub stuck: u32,              // frames without headway
}
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
/// Invest, Organize, Build). The input loop and the buttons' key hints
/// both come from here so they can't disagree. W/A/S/D move the player, so
/// no action may use them.
const ACTION_KEYS: [char; 5] = ['T', 'R', 'I', 'O', 'B'];
/// Whether action button `i` is shown at all: Recruit only appears once
/// there's a campaign to recruit into (founded at a house meeting)
pub fn action_shown(m: &UiModel, i: usize) -> bool {
    i != 1 || m.panel.campaign.is_some()
}

/// The action panel's buttons, in ACTION_KEYS order
pub const ACTION_LABELS: [&str; 5] = ["Talk", "Recruit", "Invest", "Organize", "Build"];
/// The action panel's buttons, laid out the same for every player
pub fn action_buttons() -> [Button; 5] {
    [
        Button { label: ACTION_LABELS[0].into(), key: ACTION_KEYS[0], x: UI_X as i32 + 10, y: 60,  w: 130, h: 36, color: (60,60,120) },
        Button { label: ACTION_LABELS[1].into(), key: ACTION_KEYS[1], x: UI_X as i32 + 10, y: 104, w: 130, h: 36, color: (60,120,60) },
        Button { label: ACTION_LABELS[2].into(), key: ACTION_KEYS[2], x: UI_X as i32 + 10, y: 148, w: 130, h: 36, color: (120,60,60) },
        Button { label: ACTION_LABELS[3].into(), key: ACTION_KEYS[3], x: UI_X as i32 + 10, y: 192, w: 130, h: 36, color: (120,100,40) },
        Button { label: ACTION_LABELS[4].into(), key: ACTION_KEYS[4], x: UI_X as i32 + 10, y: 236, w: 130, h: 36, color: (80,60,120) },
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
/// The second roll, for the born and the come-lately: the village grows
/// past the first thirty-one, and they shouldn't all answer to "a newcomer"
const NEW_NAMES: [&str; 12] = [
    "Sible","Dunn","Hamelin","Avice","Colle","Merrick",
    "Sabyn","Tybalt","Imma","Peryn","Lettice","Hobkin",
];
/// A villager's name by index - past the named roll, the second roll
/// keeps them distinct; past that, a newcomer stands in
fn vill_name(i: usize) -> &'static str {
    NAMES.get(i).copied()
        .or_else(|| NEW_NAMES.get(i - NAMES.len()).copied())
        .unwrap_or("a newcomer")
}
/// Which villager indices are the children, the tax collector and the herb-wife
const KID_IDXS: [usize; 3] = [26, 27, 28];
const COLLECTOR_IDX: usize = 29;
const HERB_IDX: usize = 30;
const RIVAL_NAMES: [&str; 3] = ["Wulfric","Sable","Miren"];
const TOOLTIPS: [&str; 5] = [
    "talk to someone nearby",
    "recruit villager to your org",
    "spend gold for influence",
    "call a house meeting",
    "commission a new house",
];


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
}

/// The founding call a finished house meeting left with its organizer:
/// the vote the room took (kept for the stakes - grumbles get re-read
/// against whichever framing the organizer picks), the record it was
/// said on, and the org's name. Held on the calling seat until they
/// answer it (Command::Decide) or their next meeting's offer supersedes it
pub struct PendingFounding {
    /// The meeting's talk record - the convo box keeps its tail showing
    pub record: u64,
    pub vote: crate::game::campaign::IssueVote,
    pub org_name: String,
}
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
    Blacksmith,
    Church,
    Guildhall,
    RivalHouse,
    RivalWoodhouse,
    CastleKeep,
}

/// What a seat tells the sim: the same actions the window's keys and clicks
/// map to, so a local loop, a remote client and a test all steer the same way.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Command {
    /// One of the five action buttons (Talk, Recruit, Invest, Organize, Build)
    Act(usize),
    /// A contextual action (Work, Pair up, Errand)
    CtxAct(usize),
    /// Held-direction walk, one frame's step
    Walk(f32, f32),
    /// Pick a villager by index (the 1-8 keys)
    Select(usize),
    /// Click the map through this camera (select villager or building)
    ClickMap { mx: i32, my: i32, cam_x: i32, cam_y: i32, zoom: f32 },
    TogglePause,
    ToggleSpeed,
    /// Drop any walk-over-to-talk in progress
    CancelApproach,
    /// Answer the villager mid-Talk with reply `n`, or None to take your leave
    TalkPick(Option<usize>),
    /// Answer the founding call a house meeting left open on the
    /// conversation box: pick `n` of the framings, or the last option
    /// (let it rest)
    Decide(usize),
}

/// A contextual action offered under the main buttons: what the player can
/// do depends on where they stand and who's picked.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CtxAct {
    /// Work the player's own craft (at their bench if they have one)
    Work,
    /// Pair the picked junior with a master of their trade
    PairUp,
    /// Send the picked child on a paid errand
    Errand,
    /// Put a word between two hearts the organizer knows are lonely
    Matchmake,
    /// Hold the campaign's meeting, on its cadence
    Meet,
    /// Take the campaign to the streets: a demonstration at the player
    March,
    /// Give the picked member a campaign seat (turnout, notes, chair)
    Delegate,
    /// Ballot the picked member's trade: nine of ten, or a public failure
    Strike,
    /// Put the campaign's demands to the collector, in public
    Townhall,
    /// Keep pressing for the prisoner's release
    Petition,
    /// Set how often the cause meets: the organizer's recommendation
    Cadence,
}

/// One player's hold on the game: which character they steer, what they've
/// picked, the walk-over-to-talk under way, their contextual buttons. A solo
/// game has exactly one seat; a server keeps one per claimed connection.
pub struct Seat {
    /// The seat's character entity (a wanderer for people, a villager for agents)
    pub player: Entity,
    /// The seat's organization entity
    pub org: Entity,
    /// They picked the selected villager themselves and haven't walked off
    /// since: Talk and Recruit go to them, however far
    pub picked: bool,
    /// A walk-over-to-act under way
    pub approach: Option<Approach>,
    /// Index into villagers[] of the villager currently selected
    pub selected_target: usize,
    /// A building the seat clicked, or None while a villager is selected
    pub selected_building: Option<usize>,
    /// Contextual actions that depend on where the player stands and who's picked
    pub ctx_actions: Vec<CtxAct>,
    /// An API-driven seat steers a villager, not a wanderer
    pub agent: bool,
    /// A Talk under way, waiting on the player's reply (session::dialogue)
    pub talk: Option<dialogue::PlayerTalk>,
    /// The founding call this seat's house meeting left open on the
    /// conversation box, waiting on their pick (Command::Decide). Each
    /// organizer holds their own - another seat's meeting can't erase it
    pub pending_founding: Option<PendingFounding>,
    /// Where the character stood last frame, which way they face (0 up,
    /// 1 right, 2 down, 3 left on screen) and whether they moved: drives
    /// the walk cycle
    pub last_pos: Option<(f32, f32)>,
    pub facing: u8,
    pub walking: bool,
}

impl Default for Seat {
    fn default() -> Seat {
        Seat { player: 0, org: 0, picked: false, approach: None, selected_target: 0,
               selected_building: None, ctx_actions: vec![], agent: false, talk: None,
               pending_founding: None, last_pos: None, facing: 2, walking: false }
    }
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
    pub villager_poses: std::collections::HashMap<Entity, ((f32, f32), u8, bool)>,
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
    pub built_count: usize,
    /// The village has named the player its leader (crossing WIN_INFLUENCE).
    /// It's a milestone, not the end: the game carries on
    pub leader_named: bool,
    pub works: Vec<Works>, // houses still being built
    pub toasts: Vec<(String, u64)>,  // (msg, shown until tick)
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
    pub derelict: std::collections::HashSet<usize>,
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
    pub flock_wander: std::collections::HashMap<Entity, (f32, f32, u32)>,
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
    pub trails: std::collections::HashMap<(i32, i32), Trail>,
    /// The land's memory: canopy left, water in the pit, stink downstream
    pub land: crate::sim::environment::LandScore,
    /// The worked pit's tile, and the reach below town that takes the leavings
    pub pit: Option<(i32, i32)>,
    pub reach: Vec<(i32, i32)>,
    /// (who, the rung they're on, the day it ends) - the ladder of shame
    pub sanctioned: Vec<(Entity, Sanction, u64)>,
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
    pub path_cache: std::collections::HashMap<((i32, i32), (i32, i32)), Option<Vec<(i32, i32)>>>,
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
        // frames per day: ~7s/day normal, ~1.2s turbo; a house meeting (or
        // the call it leaves the organizer) slows the calendar so the
        // room's talk can be read as it's said
        let base = if self.speed == 1 { 7 * FPS } else { 5 * FPS / 4 };
        if self.meeting.is_some() || self.seats.iter().any(|s| s.pending_founding.is_some()) { base * 2 } else { base }
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

    /// One frame's step of the player in direction (dx, dy) (tile axes),
    /// quicker along a road and slower through the woods. Blocked straight on, they slide along whatever
    /// is in the way where they can.
    pub fn walk_at(&mut self, seat: usize, dx: f32, dy: f32) {
        let (px, py) = self.player_pos_at(seat);
        let at = (px.round() as i32, py.round() as i32);
        // a well-worn path is quicker underfoot, whoever's feet wear it
        let worn = self.trails.get(&at).map(|t| t.strength).unwrap_or(0.0);
        let spd = WALK_SPEED / trails::movement_cost(1.0, worn, TRAIL_K) * match self.map.get(at.0, at.1) {
            Some(Tile::Road) | Some(Tile::Bridge) => ROAD_SPEEDUP,
            _ if self.map.trees.contains(&at) => WOODS_SLOWDOWN,
            _ => 1.0,
        };
        // the same pace whichever way they go, diagonals included
        let len = (dx * dx + dy * dy).sqrt();
        // a still stick or noise on the wire: no step, and never a NaN position
        if !(len.is_finite() && len > 0.0) { return; }
        let (nx, ny) = (px + dx / len * spd, py + dy / len * spd);
        let to = [(nx, ny), (nx, py), (px, ny)].into_iter().find(|&(x, y)| self.walkable(x, y));
        if let (Some((x, y)), Some(p)) = (to, self.world.get_component_mut::<Pos>(self.seats[seat].player)) {
            let ot = (p.x.round() as i32, p.y.round() as i32);
            p.x = x;
            p.y = y;
            // feet wear the ground the player walks, same as any villager's
            let nt = (p.x.round() as i32, p.y.round() as i32);
            if nt != ot {
                let tr = self.trails.entry(nt).or_insert(Trail::wild());
                trails::step_trail(tr, 1, 0.0, &TRAIL_PARAMS);
            }
        }
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

    pub fn player_pos_at(&self, seat: usize) -> (f32, f32) {
        self.world.get_component::<Pos>(self.seats[seat].player).map(|p| (p.x, p.y)).unwrap_or((10.0, 10.0))
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

    /// Why action button `i` can't be used right now, if it can't: Build's
    /// own rules (gold, a free site), otherwise disabled_reason
    pub fn action_blocker_at(&self, seat: usize, i: usize) -> Option<String> {
        if i == 4 {
            let gold = self.world.get_component::<PlayerCharacter>(self.seats[seat].player).map(|p| p.resources).unwrap_or(0.0);
            if gold < BUILD_COST { return Some(format!("need {:.0} gold", BUILD_COST)); }
            if self.build_site_at(seat).is_none() { return Some("no room nearby".into()); }
            return None;
        }
        self.disabled_reason_at(seat, i)
    }

    /// Why a ctx button can't answer right now, if it can't - shown greyed
    /// with the reason beside it, like the five main buttons' `blocked`
    pub fn ctx_blocker_at(&self, seat: usize, act: CtxAct) -> Option<String> {
        match act {
            CtxAct::Meet => {
                let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                let due = self.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
                    .map(|m| self.tick.saturating_sub(m.last_held) >= m.cadence_days).unwrap_or(false);
                if due { None } else { Some("not due to meet".into()) }
            }
            CtxAct::Townhall => {
                // the Reeve won't hear the same demands while he's honouring
                // a full grant for them; a part-grant or a hollowed one can
                // still be pressed
                let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)?.issue.clone();
                let g = self.world.get_component::<crate::game::reactions::LordGrant>(self.seats[seat].org)?;
                (g.issue == issue && g.active(self.tick) && !g.partial && !g.hollow)
                    .then(|| format!("the Reeve has answered - the grant stands till day {}", g.until))
            }
            CtxAct::Delegate => {
                let c = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org)?;
                let open = [crate::game::campaign::Role::TurnoutCoordinator,
                    crate::game::campaign::Role::NoteTaker,
                    crate::game::campaign::Role::Chair].iter().copied()
                    .any(|r| crate::game::campaign::seat_holder(&self.world, c, r).is_none());
                if open { None } else { Some("every seat's spoken for".into()) }
            }
            _ => None,
        }
    }

    /// The nearest free tile to the player where a house can go: dry,
    /// buildable land (not water or peaks), with no building (or the
    /// windmill) on it or next to it - a house sprite is ~2 tiles wide - not
    /// on a road, field or food store, and not right on top of the player
    pub fn build_site_at(&self, seat: usize) -> Option<(f32, f32)> {
        let (px, py) = self.player_pos_at(seat);
        let (cx, cy) = (px.round() as i32, py.round() as i32);
        let taken = |x: i32, y: i32| {
            let on = |bx: f32, by: f32| bx.round() as i32 == x && by.round() as i32 == y;
            let beside = |bx: f32, by: f32| (bx.round() as i32 - x).abs() <= 1 && (by.round() as i32 - y).abs() <= 1;
            self.buildings.iter().any(|(bx, by, _)| beside(*bx, *by))
                || beside(MILL_TILE.0, MILL_TILE.1)
                || self.world.query::<villager::FoodSource>().iter()
                    .filter_map(|f| self.world.get_component::<Pos>(*f))
                    .any(|p| on(p.x, p.y))
        };
        let mut best: Option<((f32, f32), i32)> = None;
        for y in (cy - BUILD_SEARCH)..=(cy + BUILD_SEARCH) {
            for x in (cx - BUILD_SEARCH)..=(cx + BUILD_SEARCH) {
                if x < 1 || y < 1 || x as usize >= self.map.width - 1 || y as usize >= self.map.height - 1 { continue; }
                if !self.map.buildable(x, y) || taken(x, y) { continue; }
                let d = (x - cx).abs() + (y - cy).abs();
                if d >= 2 && best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some(((x as f32, y as f32), d)); }
            }
        }
        best.map(|(site, _)| site)
    }

    /// Who Talk and Recruit go to: the selected villager if they're alive
    /// and in reach, otherwise the nearest living villager who is - the
    /// player who walks up to someone means them, not whoever was clicked
    /// last. With no one in reach it stays the selected villager.
    pub fn reach_target_at(&self, seat: usize) -> usize {
        let in_reach = self.villagers_in_reach_at(seat);
        if in_reach.contains(&self.villagers[self.seats[seat].selected_target]) { return self.seats[seat].selected_target; }
        let (px, py) = self.player_pos_at(seat);
        let dist = |v: &Entity| self.world.get_component::<Pos>(*v).map(|p| (p.x - px).powi(2) + (p.y - py).powi(2)).unwrap_or(f32::MAX);
        in_reach.iter()
            .min_by(|a, b| dist(a).partial_cmp(&dist(b)).unwrap())
            .and_then(|v| self.villagers.iter().position(|x| x == v))
            .unwrap_or(self.seats[seat].selected_target)
    }

    /// Why action button `i` can't be used right now for this seat, if it
    /// can't - the seat's own selection, reach, purse and org answer, not
    /// seat 0's
    pub fn disabled_reason_at(&self, seat: usize, i: usize) -> Option<String> {
        // Recruit is pointless on a member and Invest on an empty purse: say
        // so on the button, as Build does, rather than fail in the log
        // Talk and Recruit need the selected villager alive (they walk over
        // to them); a house meeting (Organize) just needs someone nearby to come
        let target = self.villagers[if (i == 0 || i == 1) && self.seats[seat].picked { self.seats[seat].selected_target } else { self.reach_target_at(seat) }];
        let in_reach = self.villagers_in_reach_at(seat);
        let target_alive = self.world.get_component::<Health>(target).is_some();
        let p = self.world.get_component::<PlayerCharacter>(self.seats[seat].player)
            .map(|p| (p.influence, p.resources)).unwrap_or((0.0, 0.0));
        let recruit_infl_ok = can_recruit(p.0);
        let campaign = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org);
        let member = self.world.get_component::<Organization>(self.seats[seat].org).map(|o| o.members.contains(&target)).unwrap_or(false);
        let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(target).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(target).is_some();
        if (i == 0 || i == 1) && !target_alive { Some("they have died".into()) }
        else if (i == 0 || i == 1) && held { Some("in the dungeon".into()) }
        else if i == 1 && member { Some("already a member".into()) }
        else if i == 1 && campaign.is_none() { Some("no campaign to join".into()) }
        else if i == 2 && p.1 < INVEST_GOLD { Some(format!("need {:.0} gold", INVEST_GOLD)) }
        else if i == 1 && !recruit_infl_ok { Some(format!("need influence {:.2}", recruit_influence_needed())) }
        else if i == 1 && !campaign.map(|c| crate::game::player::sympathizes(&self.world, c, target)).unwrap_or(false)
            && crate::game::player::rapport(&self.world, target) < crate::game::player::RECRUIT_RAPPORT_NEEDED { Some("talk with them more".into()) }
        else if i == 3 && self.meeting.is_some() { Some("a meeting is under way".into()) }
        else if i == 3 && in_reach.is_empty() { Some("no one nearby".into()) }
        else { None }
    }

    /// One frame of game time: the sim ticks every few frames, and between
    /// ticks villagers keep strolling
    pub fn update(&mut self) {
        // the whole update, poses and facing included, so it matches the bench's update stage
        let _frame = crate::infra::perf::scope("total.frame");
        self.advance_frame();
        // after every move this frame (the player's command, the sim, the
        // strolling), so the model's poses match its positions
        self.track_poses();
        // and those standing for something turn to face it
        self.face_routines();
    }

    fn advance_frame(&mut self) {
        let ticks_per_frame = self.ticks_per_frame();
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
        if ticked { self.note_leader(); }
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
                    let push = (3.0 - d) * 0.4 / FPS as f32; // eased apart over a few frames
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
        if !self.paused {
            let village_home = world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y));
            // our own store keeps them home: while it still serves, no one
            // should be setting off for the rivals' granary just because
            // wandering took them far enough east that it was the nearer one
            for (vi, v) in villagers.iter().enumerate() {
                if crate::game::dungeon::unavailable(world, *v) { continue; }
                let (store, village_home) = country_homes.get(vi).copied().flatten()
                    .unwrap_or((self.market_food, village_home));
                let market_has = world.get_component::<villager::FoodSource>(store)
                    .map(|f| f.servings > 0).unwrap_or(false);
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

        // death check: health <= 0 -> grave + log
        let died_on = &mut self.died_on;
        for (i, v) in villagers.iter().enumerate() {
            if let Some(h) = world.get_component::<Health>(*v) {
                if h.condition <= 0.0 {
                    toasts.push((format!("{} has died.", vill_name(i)), tick + 60));
                    log.push(format!("t{}: {} died.", tick, vill_name(i)));
                    world.remove_component::<Health>(*v);
                    world.add_component(*v, villager::Dead);
                    died_on[i] = Some(tick);
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
            toasts.push((format!("A traveller brings la grippe - {} falls ill", vill_name(2)), tick + 40));
        }
        // log who falls ill and who recovers
        let sick_now: Vec<Option<String>> = villagers.iter().map(|v| sickness_name(world, *v)).collect();
        for (i, (now, was)) in sick_now.iter().zip(self.was_sick.iter_mut()).enumerate() {
            match (now, was.as_ref()) {
                (Some(name), None) => log.push(format!("t{}: {} is taken with {}", tick, vill_name(i), name)),
                (None, Some(old)) => log.push(format!("t{}: {} recovered from {}", tick, vill_name(i), old)),
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
        toasts.retain(|(_, until)| tick < *until);
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
        if !self.paused {
            for w in self.works.iter_mut() {
                let laid = w.road_laid();
                w.frames += 1;
                for &(x, y) in &w.road[laid..w.road_laid()] { self.map.pave(x, y); }
            }
            self.works.retain(|w| w.risen() < 1.0);
            crate::timed!("sim.advance_meeting", self.advance_meeting());
            crate::timed!("sim.advance_player_talks", self.advance_player_talks());
            crate::timed!("sim.advance_travels", self.advance_travels());
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
        if !self.paused { self.chat_in_passing(); }
    }

    /// Talk to villager `who`: hear what they say. Returns the log line
    pub fn talk_to_at(&mut self, seat: usize, who: usize) -> String {
        let v = self.villagers[who];
        // Uncapped talk count - Rapport caps at 1.0 and would
        // freeze the concern/small-talk index after a few talks.
        let talks_before = self.world.get_component::<TalkCount>(v)
            .map(|c| c.value).unwrap_or(0);
        // a 1-1 has a small chance of drawing out one of their secrets:
        // a want, or the problem that's in their way
        // the fold's smell: folk hold back a touch with the shepherd
        let smelly = desires::problems_of(&self.world, v, self.tick).contains(&desires::Problem::Smelly);
        let revealed = if (self.rng.next() % 1000) < (desires::REVEAL_BASE * if smelly { 900.0 } else { 1000.0 }) as u64 {
            let pick = self.rng.next();
            desires::next_reveal(&self.world, v, self.tick, pick)
        } else { None };
        let (did, said) = match revealed {
            Some(r) => {
                let (d, s) = desires::reveal_speech(vill_name(who), &r);
                let pe = self.seats[seat].player;
                if let Some(issue) = desires::hear(&mut self.world, pe, v, r, self.tick) {
                    self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                    self.log.push(format!("t{}: Organizer discovers issue {}", self.tick, issue));
                }
                (d, s)
            }
            None => talk_response(&self.world, v, vill_name(who), talks_before),
        };
        let pe = self.seats[seat].player;
        // the bond as it stood before this talk: a first talk is with a stranger
        let rel = dialogue::relation_with_wanderer(self.rel_kind(pe, v));
        act_talk(&mut self.world, pe, v);
        self.say(v, &said, TALK_BUBBLE_SECS);
        // a blow-off leaves them hollow: a real talk coaches them back
        let coached = if self.world.get_component::<crate::game::actions::Demoralized>(v).is_some()
            && crate::game::actions::coach(&mut self.world, pe, v, self.tick) {
            " You talk the helplessness out of them."
        } else { "" };
        // the conversation itself goes on the TALK panel, the replies yours to pick
        self.open_player_talk(seat, v, rel, &said);
        if !coached.is_empty() { self.narrate_talk(seat, coached.trim()); }
        format!("{}: \"{}\"{}", did, said, coached)
    }

    /// Distance from the player to villager `who`, in tiles
    pub fn dist_to_at(&self, seat: usize, who: usize) -> f32 {
        let (px, py) = self.player_pos_at(seat);
        self.world.get_component::<Pos>(self.villagers[who])
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt()).unwrap_or(f32::MAX)
    }

    /// The tiles to walk to reach villager `who`, round walls and buildings
    pub fn path_to_at(&self, seat: usize, who: usize) -> Option<Vec<(i32, i32)>> {
        let (px, py) = self.player_pos_at(seat);
        let p = self.world.get_component::<Pos>(self.villagers[who])?;
        self.map.walk_path((px.round() as i32, py.round() as i32), (p.x.round() as i32, p.y.round() as i32))
    }

    /// Talk or Recruit (action button `action`) on someone who isn't beside
    /// the player: set off to walk over to them (the action happens on
    /// arrival; see advance_approach). None when they're already close enough
    /// to act on now.
    pub fn start_approach_at(&mut self, seat: usize, who: usize, action: usize) -> Option<String> {
        if self.dist_to_at(seat, who) <= TALK_DIST { return None; }
        match self.path_to_at(seat, who) {
            Some(path) => {
                self.seats[seat].approach = Some(Approach { who, action, path, stuck: 0 });
                Some(format!("You walk over to {}.", vill_name(who)))
            }
            None => Some(format!("Can't {}: there's no way to reach {}.",
                ACTION_LABELS[action].to_lowercase(), vill_name(who))),
        }
    }

    /// One frame of walking over to someone: follow the path (finding a new
    /// one if they've wandered off), and talk or recruit once beside them
    pub fn advance_approach_at(&mut self, seat: usize) {
        let Some(mut a) = self.seats[seat].approach.take() else { return };
        let who = a.who;
        // they died on the way: nothing to say; hauled off: no one to walk to
        if crate::game::dungeon::unavailable(&self.world, self.villagers[who]) { return; }
        if self.dist_to_at(seat, who) <= TALK_DIST {
            // beside them now: take the action on them (act checks it can
            // still be done - they may have joined, or influence slipped)
            self.seats[seat].selected_target = who;
            self.seats[seat].picked = true;
            self.act_at(seat, a.action);
            return;
        }
        let Some(&Pos { x: vx, y: vy }) = self.world.get_component::<Pos>(self.villagers[who]) else { return };
        let headed_off = a.path.last()
            .map(|&(x, y)| ((x as f32 - vx).powi(2) + (y as f32 - vy).powi(2)).sqrt() > TALK_DIST)
            .unwrap_or(true);
        if headed_off {
            match self.path_to_at(seat, who) {
                Some(path) => a.path = path,
                None => { self.log.push(format!("t{}: You can't find a way to {}.", self.tick, vill_name(who))); return; }
            }
        }
        let (px, py) = self.player_pos_at(seat);
        // drop tiles already reached; the last one is their tile, so head for them
        while a.path.len() > 1 {
            let (x, y) = a.path[0];
            if (x as f32 - px).abs() < 0.25 && (y as f32 - py).abs() < 0.25 { a.path.remove(0); } else { break; }
        }
        let (tx, ty) = match a.path.as_slice() { [(x, y), _, ..] => (*x as f32, *y as f32), _ => (vx, vy) };
        let (dx, dy) = (tx - px, ty - py);
        if dx.abs() + dy.abs() > 1e-3 { self.walk_at(seat, dx, dy); }
        a.stuck = if self.player_pos_at(seat) != (px, py) { 0 } else { a.stuck + 1 };
        if a.stuck > APPROACH_STUCK_FRAMES {
            self.log.push(format!("t{}: You can't get through to {}.", self.tick, vill_name(who)));
            return;
        }
        self.seats[seat].approach = Some(a);
    }

    /// Take action button `i` (Talk, Recruit, Invest, Organize, Build)
    pub fn act_at(&mut self, seat: usize, i: usize) {
        let tick = self.tick;
        // Talk and Recruit go to whoever the player picked; otherwise to
        // whoever is in reach
        if (i == 0 || i == 1) && !self.seats[seat].picked { self.seats[seat].selected_target = self.reach_target_at(seat); }
        let selected_target = self.seats[seat].selected_target;
        // a Talk's words go on the TALK panel, not the event log
        let mut talked = false;
        let msg = if let Some(reason) = self.action_blocker_at(seat, i) {
            format!("Can't {}: {}.", ACTION_LABELS[i].to_lowercase(), reason)
        } else {
            match i {
                0 => match self.start_approach_at(seat, selected_target, 0) {
                    Some(walking) => walking,
                    None => { talked = true; self.talk_to_at(seat, selected_target) }
                },
                1 => if let Some(walking) = self.start_approach_at(seat, selected_target, 1) { walking } else {
                    let (pe, oe, te) = (self.seats[seat].player, self.seats[seat].org, self.villagers[selected_target]);
                    let r = act_recruit(&mut self.world, pe, oe, te);
                    if r.success { format!("{} joined!", vill_name(selected_target)) }
                    else { format!("{}: {}", vill_name(selected_target), r.description.to_lowercase()) }
                }
                2 => { let pe = self.seats[seat].player; let r = act_invest(&mut self.world, pe, INVEST_GOLD); r.description }
                3 => self.call_meeting_at(seat),
                4 => {
                    // action_blocker checked there's gold and a site
                    let (bx, by) = self.build_site_at(seat).unwrap();
                    if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.seats[seat].player) { p.resources -= BUILD_COST; }
                    self.buildings.push((bx, by, BSprite::House(self.rng.below(97) as usize)));
                    self.map.build_at(bx as i32, by as i32);
                    // the road is laid a tile at a time, then the house goes up
                    let road = self.map.plan_road((bx as i32, by as i32));
                    self.works.push(Works { building: self.buildings.len() - 1, road, frames: 0 });
                    self.built_count += 1;
                    format!("Work starts on a new house beside you - {:.0} gold", BUILD_COST)
                }
                _ => unreachable!("no action button {}", i),
            }
        };
        // every action line is dated like the rest of the log. It doesn't
        // show the influence it earned: the log reads as what happened in
        // the village, not a score sheet (the panel's bar shows standing)
        if !talked { self.log.push(format!("t{}: {}", tick, msg)); }
        // gold spent here is the same purse the day's trading counts
        let pe = self.seats[seat].player; sync_purse(&mut self.world, pe, true);
    }

    /// Living villagers within MEETING_REACH of the player
    pub fn villagers_in_reach_at(&self, seat: usize) -> Vec<Entity> {
        let (px, py) = self.player_pos_at(seat);
        self.villagers.iter().copied().filter(|v| {
            let alive = self.world.get_component::<Health>(*v).is_some();
            let near = self.world.get_component::<Pos>(*v)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() < MEETING_REACH)
                .unwrap_or(false);
            // the shunned aren't asked to sit, and the held aren't around at all
            alive && near && !self.shunned(*v)
                && !crate::game::dungeon::unavailable(&self.world, *v)
        }).collect()
    }

    /// The end of a house meeting, once those who came have talked it over.
    /// Those who already trust you join the org; everyone who came gets to
    /// know you a little better (organizing::house_meeting). The meeting
    /// belongs to the seat that called it; `record` is its talk_log id, so
    /// a founding call can keep its tail showing on the conversation box.
    pub fn resolve_meeting(&mut self, invited: &[Entity], seat: usize, record: u64) -> String {
        let (oe, pe) = (self.seats[seat].org, self.seats[seat].player);
        let members_before = self.world.get_component::<Organization>(oe).map(|o| o.members.clone()).unwrap_or_default();
        let r = crate::game::organizing::house_meeting(&mut self.world, oe, invited);
        let joined: Vec<&str> = invited.iter()
            .filter(|v| !members_before.contains(v))
            .filter(|v| self.world.get_component::<Organization>(oe).map(|o| o.members.contains(v)).unwrap_or(false))
            .filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| vill_name(i)))
            .collect();
        // a member won at a meeting counts toward leading the village just
        // like one won with Recruit - otherwise every meeting recruit is
        // influence the player can never get back (they can't be Recruited)
        if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(pe) {
            p.influence += RECRUIT_INFLUENCE * joined.len() as f32;
        }
        let outcome = if joined.is_empty() {
            "No one joins yet, but they'll remember the conversation.".to_string()
        } else {
            format!("{} joined{}!", joined.join(", "), if r.formed_committee { " and formed a committee" } else { "" })
        };
        // a meeting also succeeds when it surfaces a pattern: three or more
        // voices with versions of the same trouble become an issue
        let mut tags: std::collections::HashMap<desires::IssueTag, Vec<Entity>> = Default::default();
        for (who, tag) in &self.voiced { tags.entry(tag.clone()).or_default().push(*who); }
        let mut found = String::new();
        for (tag, voices) in &tags {
            let mut vs = voices.clone();
            vs.sort_unstable();
            vs.dedup();
            if vs.len() >= 3 {
                if let Some(issue) = desires::found_issue(&mut self.world, pe, tag.clone()) {
                    self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                    self.log.push(format!("t{}: Organizer discovers issue {}", self.tick, issue));
                    found = format!(" Organizer discovers issue {}.", issue);
                }
            }
        }
        // three voices of one trade converge; the same trouble said by
        // three trades splinters - a mason's grief isn't a baker's
        if found.is_empty() {
            let mut bases: std::collections::HashMap<String, std::collections::HashSet<Entity>> = Default::default();
            for (tag, voices) in &tags {
                let base = tag.split_once(" (").map(|(b, _)| b).unwrap_or(tag.as_str());
                bases.entry(base.to_string()).or_default().extend(voices.iter().copied());
            }
            if bases.values().any(|vs| vs.len() >= 3) {
                self.log.push(format!("t{}: the talk splinters by trade.", self.tick));
                found = " The talk splinters by trade.".into();
            }
        }
        // a meeting in the street is power spent: the lord notices a
        // full kitchen - and a thin one
        self.record_power(if invited.len() >= 3 {
            crate::sim::organized_power::ActionQuality::Transformational
        } else { crate::sim::organized_power::ActionQuality::Overreach });
        let founded = self.founding_offer(seat, record).map(|s| format!(" {}", s)).unwrap_or_default();
        format!("House meeting - {} came. {}{}{}", invited.len(), outcome, found, founded)
    }

    /// The day any seat's influence first passes WIN_INFLUENCE the village
    /// names them its leader - a line in the log, and play goes on
    fn note_leader(&mut self) {
        if self.leader_named { return; }
        let led = self.seats.iter().any(|s| self.world.get_component::<PlayerCharacter>(s.player)
            .map(|p| p.influence > WIN_INFLUENCE).unwrap_or(false));
        if !led { return; }
        self.leader_named = true;
        self.toasts.push(("the village names you its leader".into(), self.tick + 60));
        self.log.push(format!("t{}: the village names you its leader - there's still work to do.", self.tick));
    }

    /// The founding vote, when the org can take one: enough living
    /// members and a heard issue. The organizer's ledger supplies the
    /// candidates; the room's leaning is read for a shared grievance
    /// (grumbles are re-read against the framing actually picked).
    /// No campaign is founded - that's the organizer's call (decide_at),
    /// or maybe_found_campaign's in tests
    fn founding_vote(&self, seat: usize) -> Option<(crate::game::campaign::IssueVote, String)> {
        use crate::game::campaign;
        let org = self.seats[seat].org;
        if campaign::campaign_of(&self.world, org).is_some() { return None; }
        let living = self.world.get_component::<Organization>(org)
            .map(|o| o.members.iter().filter(|&&m| !villager::is_dead(&self.world, m)).count())
            .unwrap_or(0);
        if living < 3 { return None; }
        let issues = self.world.get_component::<desires::Organizer>(self.seats[seat].player)
            .map(|o| o.issues.clone()).unwrap_or_default();
        if issues.is_empty() { return None; }
        let vote = campaign::hold_issue_vote(&self.world, org, &issues, campaign::Framing::SharedGrievance, self.tick)?;
        let oname = self.world.get_component::<Organization>(org).map(|o| o.name.clone()).unwrap_or_default();
        Some((vote, oname))
    }

    /// Found the campaign off a settled vote: the entity, its meeting
    /// cadence, the toast and the "they'll fight for" line for the log
    fn found_and_note(&mut self, seat: usize, vote: &crate::game::campaign::IssueVote,
                      framing: crate::game::campaign::Framing, rule: crate::game::campaign::DecisionRule) -> String {
        use crate::game::campaign;
        let org = self.seats[seat].org;
        let oname = self.world.get_component::<Organization>(org).map(|o| o.name.clone()).unwrap_or_default();
        let c = campaign::found_campaign(&mut self.world, org, vote, framing, rule, self.tick);
        self.world.add_component(c, campaign::CampaignMeeting {
            cadence_days: 3, last_held: self.tick, held: 0, resolves: 0 });
        self.toasts.push((format!("{} take a vote", oname), self.tick + 40));
        let mut msg = format!("{} take a vote - they'll fight for {}", oname, vote.winner);
        // they came along, but when the framing sits wrong on a stake they
        // grumble - heard in the log, not just counted on the ballots
        let grumbler = vote.ballots.iter().find(|b| b.grumbles).map(|b| b.member);
        if let Some(d) = grumbler {
            let name = self.villagers.iter().position(|&v| v == d).map(|i| vill_name(i)).unwrap_or("someone");
            msg += &format!(" - {} {}", name, campaign::grumble_line(framing));
        }
        msg
    }

    /// After a house meeting: enough living members and a heard issue and
    /// the org can take the founding vote. The room's leaning goes up on
    /// the conversation box - which cause and how it's put is the
    /// organizer's call to make (decide_at). Returns the log line's tail.
    fn founding_offer(&mut self, seat: usize, record: u64) -> Option<String> {
        let (vote, oname) = self.founding_vote(seat)?;
        let lean = format!("{} lean to {} - your call.", oname, desires::issue_name(vote.winner.clone()));
        self.seats[seat].pending_founding = Some(PendingFounding { record, vote, org_name: oname });
        Some(lean)
    }

    /// The organizer's answer to the founding call on the conversation
    /// box: take the room's leaning up, framed as picked - or let it rest
    /// (the next house meeting can put it again)
    pub fn decide_at(&mut self, seat: usize, pick: usize) {
        let Some(mut d) = self.seats[seat].pending_founding.take() else { return };
        use crate::game::campaign::{self, DecisionRule, Framing};
        let framing = match pick {
            0 => Framing::SharedGrievance,
            1 => Framing::CommonFoe,
            2 => Framing::MoralDuty,
            _ => {
                self.log.push(format!("t{}: The talk stays talk for now - no cause taken up.", self.tick));
                self.note_talk(d.record, None, "The talk stays talk for now - no cause taken up.");
                return;
            }
        };
        // who grumbles is re-read against the framing actually picked
        for b in &mut d.vote.ballots {
            b.grumbles = b.stake.as_ref().map(|s| !campaign::framing_suits(framing, s)).unwrap_or(false);
        }
        let msg = self.found_and_note(seat, &d.vote, framing, DecisionRule::Majority);
        self.log.push(format!("t{}: {}", self.tick, msg));
        self.note_talk(d.record, None, &msg);
    }

    /// After a house meeting: found the campaign on the vote straight
    /// away, without asking the organizer - kept for tests and for
    /// callers that don't offer the founding call
    pub fn maybe_found_campaign(&mut self, seat: usize) -> Option<String> {
        let (vote, _) = self.founding_vote(seat)?;
        Some(self.found_and_note(seat, &vote,
            crate::game::campaign::Framing::SharedGrievance, crate::game::campaign::DecisionRule::Majority))
    }

    /// Put a speech bubble over `who` for `secs` seconds (replacing any
    /// they already had)
    pub fn say(&mut self, who: Entity, text: &str, secs: u32) {
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), self.frame_n + secs * FPS, false, false));
    }

    /// Put a thought bubble over `who` for `frames` frames (at a meeting)
    pub fn think(&mut self, who: Entity, text: &str, frames: u32) {
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), self.frame_n + frames, true, false));
    }

    /// Ambient chatter as people pass each other: unlike `say`, this never
    /// overwrites a bubble that isn't itself passing chatter while that
    /// bubble is still up (e.g. a Talk reply), so a reply can't be clobbered
    /// by a passing remark on the very next frame. Returns whether the line
    /// was actually shown, so a caller spending a scripted turn (a
    /// conversation line) can tell it was blocked and hold the turn open.
    pub fn say_in_passing(&mut self, who: Entity, text: &str, secs: u32) -> bool {
        let frame = self.frame_n;
        let blocked = self.bubbles.iter().any(|(w, _, until, _, passing)| *w == who && !*passing && frame < *until);
        if blocked { return false; }
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), frame + secs * FPS, false, true));
        true
    }

    /// Organize: call a house meeting at the home nearest the player (or
    /// where they stand, if no home is in reach) for every living villager
    /// within reach. They walk over; see advance_meeting.
    pub fn call_meeting_at(&mut self, seat: usize) -> String {
        let mut invited = self.villagers_in_reach_at(seat);
        if invited.is_empty() { return "No one is near enough to come to your house meeting.".into(); }
        // a kitchen only holds so many: the eight nearest get the word
        let (px, py) = self.player_pos_at(seat);
        invited.sort_by_key(|&v| {
            self.world.get_component::<Pos>(v)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2) * 100.0) as i32).unwrap_or(i32::MAX)
        });
        invited.truncate(8);
        // errands are dropped: folk coming to a meeting stop mid-way home
        self.travels.retain(|t| !invited.contains(&t.who));
        let (px, py) = self.player_pos_at(seat);
        let dist = |(x, y): (i32, i32)| ((x as f32 - px).powi(2) + (y as f32 - py).powi(2)).sqrt();
        let home = self.buildings.iter().enumerate()
            .filter(|(_, (_, _, s))| matches!(s, BSprite::House(_)))
            .map(|(i, (x, y, _))| ((*x as i32, *y as i32), if i < 3 { format!("{}'s house", vill_name(i)) } else { "the new house".to_string() }))
            .filter(|(at, _)| dist(*at) < MEETING_REACH)
            .min_by(|a, b| dist(a.0).partial_cmp(&dist(b.0)).unwrap());
        let (venue, label) = home.unwrap_or(((px.round() as i32, py.round() as i32), "your spot".to_string()));
        // open ground round the venue, nearest first: one spot each
        let mut ground: Vec<(i32, i32)> = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (venue.0 + dx, venue.1 + dy)))
            .filter(|&at| at != venue && self.map.passable(at.0, at.1)).collect();
        ground.sort_by_key(|&(x, y)| (x - venue.0).abs() + (y - venue.1).abs());
        let spots: Vec<(i32, i32)> = (0..invited.len()).map(|i| *ground.get(i).unwrap_or(&venue)).collect();
        let paths = invited.iter().zip(&spots).map(|(v, &spot)| {
            let p = self.world.get_component::<Pos>(*v).copied()?;
            let from = (p.x.round() as i32, p.y.round() as i32);
            if from == spot { Some(vec![spot]) } else { self.map.walk_path(from, spot) }
        }).collect();
        let names: Vec<&str> = invited.iter().filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| vill_name(i))).collect();
        let msg = format!("You call a house meeting at {} - {} {} on the way.", label, names.join(", "), if names.len() == 1 { "is" } else { "are" });
        self.meeting = Some(Meeting { label, invited, spots, paths, frames: 0, talk_from: None, lines: vec![], talk_id: 0, seat });
        msg
    }

    /// What's said at a meeting: the player opens, each villager there says
    /// what's on their mind - now and then a secret slips out, and every
    /// parallel complaint already voiced makes the next speaker likelier
    /// to share their own. Three voices in a family surface an issue.
    pub fn meeting_lines(&mut self, came: &[Entity], seat: usize) -> Vec<(Entity, String)> {
        // the seat that called it hosts: its player asks, and hears what comes out
        let host_e = self.seats[seat].player;
        if came.is_empty() { return vec![]; }
        let mut lines = vec![(host_e, MEETING_PROMPTS[0].to_string())];
        self.voiced.clear();
        // what each attendee says when no secret comes out of them: the
        // meeting's talk from conversation::run_meeting - answers to the
        // question, agreement and pushback threaded on what's been said -
        // seeded off the meeting, not the game's rng
        let mut said = {
            use crate::game::conversation::{run_meeting, MeetingLine, Rng as TalkRng, Speaker};
            use crate::game::dialogue_trees::{meeting_pool, MEET_PROMPT};
            let speakers: Vec<Speaker> = came.iter()
                .map(|&v| dialogue::speaker_of(&self.world, v, &self.name_of(v), self.tick)).collect();
            let pools: Vec<Vec<MeetingLine>> = speakers.iter().map(meeting_pool).collect();
            let pool_refs: Vec<&[MeetingLine]> = pools.iter().map(|p| p.as_slice()).collect();
            let refs: Vec<&Speaker> = speakers.iter().collect();
            let host = self.world.get_component::<PlayerCharacter>(host_e).map(|p| p.name.clone()).unwrap_or_default();
            let seed = self.tick.wrapping_mul(0x51_7CC1) ^ came.iter().fold(0u64, |h, &v| h.rotate_left(7) ^ v as u64);
            let record = run_meeting(&Speaker::new(&host), &refs, &MEET_PROMPT, &pool_refs, &mut TalkRng::new(seed));
            // per attendee, in the order they said them: (statement index,
            // the statement it answers, the words)
            let mut q: Vec<Vec<(usize, Option<usize>, String)>> = vec![vec![]; came.len()];
            for (k, st) in record.into_iter().enumerate().skip(1) {
                if let Some(d) = q.get_mut(st.speaker) { d.push((k, st.replying_to, st.text)); }
            }
            q
        };
        // the meeting runs in rounds, not in run_meeting's order, so a
        // reply is only said once what it answers has been (statement 0 is
        // the host's question) - never an answer ahead of its question
        let mut aired: std::collections::HashSet<usize> = [0].into_iter().collect();
        for round in 0..2 {
            for (ci, &v) in came.iter().enumerate() {
                let Some(i) = self.villagers.iter().position(|x| *x == v) else { continue };
                let mine: std::collections::HashSet<desires::IssueTag> =
                    desires::unrevealed(&self.world, v, self.tick).iter()
                        .map(|r| desires::issue_tag(&self.world, v, r)).collect();
                let parallel = self.voiced.iter().filter(|(_, t)| mine.contains(t)).count();
                let mut chance = desires::REVEAL_BASE + desires::REVEAL_PARALLEL * parallel as f32;
                // the fold's smell hangs on a shepherd: a touch slower to share
                if desires::problems_of(&self.world, v, self.tick).contains(&desires::Problem::Smelly) { chance *= 0.9; }
                let roll = (self.rng.next() % 1000) as f32 / 1000.0;
                let mut shared = false;
                if roll < chance {
                    let pick = self.rng.next();
                    if let Some(r) = desires::next_reveal(&self.world, v, self.tick, pick) {
                        let (_, said) = desires::reveal_speech(vill_name(i), &r);
                        self.voiced.push((v, desires::issue_tag(&self.world, v, &r)));
                        let pe = host_e;
                if let Some(issue) = desires::hear(&mut self.world, pe, v, r, self.tick) {
                            self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                            self.log.push(format!("t{}: Organizer discovers issue {}", self.tick, issue));
                        }
                        lines.push((v, said));
                        shared = true;
                    }
                }
                if !shared {
                    let ready = said[ci].iter().position(|(_, to, _)| to.map_or(true, |t| aired.contains(&t)));
                    let line = match ready {
                        Some(j) => { let (k, _, text) = said[ci].remove(j); aired.insert(k); text }
                        None => meeting_remark(&self.world, v, vill_name(i), round),
                    };
                    lines.push((v, line));
                }
            }
            lines.push((host_e, MEETING_PROMPTS[round + 1].to_string()));
        }
        lines
    }

    /// One frame of a house meeting: while people gather, each walks a
    /// step towards their spot; once they're all there (or the host has
    /// waited GATHER_FRAMES) they talk in turns, then it resolves
    pub fn advance_meeting(&mut self) {
        let Some(mut m) = self.meeting.take() else { return };
        m.frames += 1;
        // those who've arrived stand at their spot (no strolling off mid-meeting);
        // a dead invitee's grave stays where they died, it doesn't keep walking
        for (i, v) in m.invited.iter().enumerate() {
            if crate::game::dungeon::unavailable(&self.world, *v) { continue; }
            let Some(p) = self.world.get_component_mut::<Pos>(*v) else { continue };
            match &mut m.paths[i] {
                Some(path) if !path.is_empty() => {
                    let (nx, ny) = (path[0].0 as f32, path[0].1 as f32);
                    let (dx, dy) = (nx - p.x, ny - p.y);
                    let d = (dx * dx + dy * dy).sqrt();
                    if d <= MEETING_WALK { p.x = nx; p.y = ny; path.remove(0); } else { p.x += dx / d * MEETING_WALK; p.y += dy / d * MEETING_WALK; }
                }
                Some(_) => { p.x = m.spots[i].0 as f32; p.y = m.spots[i].1 as f32; }
                None => {}
            }
        }
        // arrived at their spot, and alive - a dead invitee never counts as
        // having attended even if they got there before dying
        let arrived = |m: &Meeting, world: &World, i: usize| {
            !crate::game::dungeon::unavailable(world, m.invited[i]) && m.paths[i].as_ref().map(|p| p.is_empty()).unwrap_or(false)
        };
        match m.talk_from {
            None => {
                if (0..m.invited.len()).all(|i| arrived(&m, &self.world, i)) || m.frames >= GATHER_FRAMES {
                    // whoever didn't get there in time (or died on the way) isn't at the meeting
                    let came: Vec<usize> = (0..m.invited.len()).filter(|&i| arrived(&m, &self.world, i)).collect();
                    m.invited = came.iter().map(|&i| m.invited[i]).collect();
                    m.spots = came.iter().map(|&i| m.spots[i]).collect();
                    m.paths = came.iter().map(|&i| m.paths[i].clone()).collect();
                    m.lines = self.meeting_lines(&m.invited, m.seat);
                    m.talk_from = Some(m.frames);
                    // the TALK panel keeps the meeting's record as it's said
                    if !m.invited.is_empty() {
                        let host = self.seats[m.seat].player;
                        m.talk_id = self.open_meeting_record(host, &m.invited);
                    }
                    // show the opening line right away - otherwise the first
                    // "since" tick after talk_from is 1, not 0, and the
                    // player's opening prompt (lines[0]) is skipped
                    if let Some((who, text)) = m.lines.first().cloned() {
                        self.think(who, &text, LINE_FRAMES);
                        self.note_talk(m.talk_id, Some(who), &text);
                    }
                }
                self.meeting = Some(m);
            }
            Some(start) => {
                let (since, k) = (m.frames - start, ((m.frames - start) / LINE_FRAMES) as usize);
                if since % LINE_FRAMES == 0 && k < m.lines.len() {
                    let (who, text) = m.lines[k].clone();
                    self.think(who, &text, LINE_FRAMES); // gone as the next one speaks
                    if k > 0 { self.note_talk(m.talk_id, Some(who), &text); }
                }
                if k < m.lines.len() { self.meeting = Some(m); return; }
                let msg = if m.invited.is_empty() { format!("No one made it to the house meeting at {}.", m.label) }
                    else { self.resolve_meeting(&m.invited, m.seat, m.talk_id) };
                self.log.push(format!("t{}: {}", self.tick, msg));
            }
        }
    }

    /// When the player and a villager, or two villagers, come within
    /// CHAT_RANGE of each other, one of them passes a remark - at most once
    /// a CHAT_COOLDOWN days a pair - and the pair get on a little better
    pub fn chat_in_passing(&mut self) {
        let mut people: Vec<Entity> = self.villagers.iter().copied()
            .filter(|v| !crate::game::dungeon::unavailable(&self.world, *v)).collect();
        people.push(self.player);
        let pos = |w: &World, e: Entity| w.get_component::<Pos>(e).map(|p| (p.x, p.y));
        let mut near = vec![];
        for (i, &a) in people.iter().enumerate() {
            for &b in &people[i + 1..] {
                let (Some(pa), Some(pb)) = (pos(&self.world, a), pos(&self.world, b)) else { continue };
                if (pa.0 - pb.0).powi(2) + (pa.1 - pb.1).powi(2) < CHAT_RANGE * CHAT_RANGE { near.push((a, b)); }
            }
        }
        let tick = self.tick;
        for &(a, b) in &near {
            if self.near.contains(&(a, b)) { continue; } // already together: they've said hello
            let last = self.last_chat.iter().find(|(p, _)| *p == (a, b)).map(|(_, t)| *t);
            if last.map(|t| tick < t + CHAT_COOLDOWN).unwrap_or(false) { continue; }
            // the player is listed last, so a villager always does the talking.
            // Only some meetings get a bubble - every pair passing a remark was
            // a constant stream of chatter (the chat still counts either way)
            let n = self.last_chat.len() + (a as usize) + tick as usize;
            if n % 3 == 0 {
                // two villagers hold a whole conversation - on the record
                // for the TALK panel, not floated over the street
                // (session::dialogue); the player gets a word in passing
                if b == self.player { self.say_in_passing(a, passing_remark(n), CHAT_BUBBLE_SECS); }
                else { self.villagers_chat(a, b); }
            }
            let (pe, other) = (self.player, if b == self.player { a } else { b });
            relationships::record_interaction(&mut self.world, if b == pe { b } else { a }, relationships::Interaction {
                other, kind: relationships::InteractionKind::Conversation, intensity: CHAT_INTENSITY,
            });
            self.last_chat.retain(|(p, _)| *p != (a, b));
            self.last_chat.push(((a, b), tick));
        }
        self.near = near;
    }

    /// The camera centre for the map tile under minimap point (mx, my),
    /// if (mx, my) is on the minimap
    pub fn minimap_look(&self, mx: i32, my: i32) -> Option<(i32, i32)> {
        minimap_look_map(&self.map, mx, my)
    }

    /// A left click on the map at screen point (mx, my), seen through the
    /// caller's camera: rivals rebuff or greet you, villagers get selected
    pub fn click_map_at(&mut self, seat: usize, mx: i32, my: i32, cam_x: i32, cam_y: i32, zoom: f32) {
        let tick = self.tick;
        // click on a rival villager -> they rebuff you (foreign village is wary)
        for (i, rv) in self.rivals.clone().into_iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(rv) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                let d = ((sx as f32 - mx as f32).powi(2) + (sy as f32 - my as f32).powi(2)).sqrt();
                if mx < MAP_W as i32 && d < 24.0 {
                    let p_inf = self.world.get_component::<PlayerCharacter>(self.seats[seat].player).unwrap().influence;
                    let (msg, tag) = if p_inf > 1.5 {
                        (format!("{}: we've heard of you - the organizer.", RIVAL_NAMES[i]), "opened")
                    } else if p_inf > 0.8 {
                        (format!("{}: word travels. state your business.", RIVAL_NAMES[i]), "wary")
                    } else {
                        (format!("{}: we don't know you, outsider.", RIVAL_NAMES[i]), "rebuffed")
                    };
                    self.say(rv, &msg, TALK_BUBBLE_SECS);
                    self.log.push(format!("t{}: {} {} your approach.", tick, RIVAL_NAMES[i], tag));
                }
            }
        }
        // click on map selects nearest villager
        let on_map = mx < MAP_W as i32;
        let mut best = self.seats[seat].selected_target;
        let mut best_d = 9999.0f32;
        for (i, v) in self.villagers.iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(*v) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                let d = ((sx as f32 - mx as f32).powi(2) + (sy as f32 - my as f32).powi(2)).sqrt();
                if on_map && d < 30.0 && d < best_d { best = i; best_d = d; }
            }
        }
        // click on a building: the inspector shows what it's for, who
        // works or lives there, and what it has for offer
        let mut best_b: Option<usize> = None;
        let mut best_bd = 9999.0f32;
        for (i, (bx, by, _)) in self.buildings.iter().enumerate() {
            let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y, zoom);
            let d = ((sx as f32 - mx as f32).powi(2) + ((sy as f32 - 14.0 * zoom) - my as f32).powi(2)).sqrt();
            if on_map && d < 42.0 && d < best_bd { best_b = Some(i); best_bd = d; }
        }
        // a villager wins the click only if clearly nearer
        self.seats[seat].selected_building = if best_d + 8.0 < best_bd { None } else { best_b };
        if self.seats[seat].selected_building.is_some() { self.seats[seat].picked = false; return; }
        if best_d < 30.0 { self.seats[seat].picked = true; }
        self.seats[seat].selected_target = best;
    }

    /// Send `who` walking toward `goal` (a person, a shop or a tile).
    /// Returns whether a way was found - otherwise nothing happens
    pub fn send(&mut self, who: Entity, goal: Goal) -> bool {
        // the held take no errands - the jailer's rounds aren't theirs;
        // the dead take none either, however short the walk
        if crate::game::dungeon::unavailable(&self.world, who) { return false; }
        let Some(from) = self.world.get_component::<Pos>(who).copied() else { return false };
        let to = match &goal {
            Goal::SeekRemedy(e) | Goal::Herd(e) | Goal::HeadHome(e) => self.world.get_component::<Pos>(*e).copied()
                .and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32)),
            Goal::TendGarden(i) => self.gardens.get(*i).map(|(_, t, _, _)| *t),
            Goal::Exile => {
                // the nearest edge of the world
                self.world.get_component::<Pos>(who).copied().and_then(|from| {
                    let (w, h) = (self.map.width as i32, self.map.height as i32);
                    [(2, from.y.round() as i32), (w - 3, from.y.round() as i32),
                     (from.x.round() as i32, 2), (from.x.round() as i32, h - 3)]
                        .iter().copied()
                        .filter(|&(x, y)| self.map.passable(x, y))
                        .min_by_key(|&(x, y)| ((x as f32 - from.x).abs() + (y as f32 - from.y).abs()) as i32)
                })
            }
            Goal::BuyFrom(e, _, _) | Goal::BuyAt(e, _, _) | Goal::CollectTax(e) | Goal::TalkTo(e) =>
                self.world.get_component::<Pos>(*e).copied().map(|p| (p.x.round() as i32, p.y.round() as i32)),
            Goal::Wander => Some((from.x.round() as i32, from.y.round() as i32)),
            Goal::Gather(..) => return false, // gathers go through send_gather
            Goal::Cart(_) | Goal::Levy(..) | Goal::Deliver(_) => return false, // country walks go through send_far
            Goal::Forage(_) | Goal::ForageShroom(_) => return false, // forages go through send_forage/send_shroom
            Goal::RunOddJob => self.world.get_component::<OddJob>(who)
                .and_then(|j| self.world.get_component::<Pos>(j.from).copied())
                .map(|p| (p.x.round() as i32, p.y.round() as i32)),
            Goal::Eat => {
                // the village table while it's laid: folk eat at their
                // own market, and only a bare shelf sends them hunting
                // for the next source
                let own = self.store_of(who);
                let market_laid = self.world.get_component::<villager::FoodSource>(own)
                    .map(|f| f.servings > 0).unwrap_or(false);
                if market_laid {
                    self.world.get_component::<Pos>(own)
                        .map(|p| (p.x.round() as i32, p.y.round() as i32))
                } else {
                    self.world.query::<villager::FoodSource>().iter().copied()
                        .filter(|&f| self.world.get_component::<villager::FoodSource>(f)
                            .map(|fs| fs.servings > 0).unwrap_or(false))
                        .filter_map(|f| self.world.get_component::<Pos>(f)
                            .map(|fp| ((fp.x.round() as i32, fp.y.round() as i32),
                                (fp.x - from.x).abs() + (fp.y - from.y).abs())))
                        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                        .map(|(p, _)| p)
                }
            }
        };
        let Some(to) = to else { return false };
        // nobody hikes halfway to the next village on an errand (the tax
        // man's round reaches the outskirts - that's his job; so does a
        // sick walk out to the herb hut)
        let legs = ((to.0 as f32 - from.x).abs() + (to.1 as f32 - from.y).abs()) as i32;
        // urgent errands get the long reach: the sick, the hungry, the
        // flock, home - and a worker's inputs, or the craft starves
        let cap = match &goal { Goal::CollectTax(_) => 24,
            Goal::SeekRemedy(_) | Goal::Herd(_) | Goal::HeadHome(_) | Goal::Eat
            | Goal::BuyFrom(..) | Goal::BuyAt(..) => 30,
            Goal::Exile => 80, Goal::TendGarden(_) => 16, _ => 9 };
        if legs > cap { return false; }
        if to == (from.x.round() as i32, from.y.round() as i32) {
            self.travels.push(Travel { who, path: vec![to], goal });
            return true;
        }
        match self.walk_memo((from.x.round() as i32, from.y.round() as i32), to) {
            Some(mut path) => { path.push(to); self.travels.push(Travel { who, path, goal }); true }
            None => false,
        }
    }

    /// Raise an audio cue: a kind, a place, an intensity - the audio
    /// layer turns it into sound
    pub fn emit_sfx(&mut self, kind: &'static str, x: f32, y: f32, intensity: f32) {
        self.sfx.push(SfxCue { kind, x, y, intensity });
    }

    /// The BFS is the game's hot line - a path's the same all day, so
    /// remember it until the map might have moved
    fn walk_memo(&mut self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        if self.path_tick != self.tick {
            self.path_cache.clear();
            self.path_tick = self.tick;
        }
        if let Some(cached) = self.path_cache.get(&(from, to)) { return cached.clone(); }
        let p = self.map.walk_path(from, to);
        self.path_cache.insert((from, to), p.clone());
        p
    }

    /// A passable tile right next to a building's (occupied) one: where a
    /// caller at the door actually stands, south side preferred
    fn at_the_door(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        if self.map.passable(x, y) { return Some((x, y)); }
        [(0, 1), (-1, 0), (1, 0), (0, -1), (-1, 1), (1, 1), (-1, -1), (1, -1)]
            .iter().map(|(dx, dy)| (x + dx, y + dy))
            .find(|&(qx, qy)| self.map.passable(qx, qy))
    }

    /// Walk `who` to tile `t` carrying `goal` (shared by the forage sends)
    fn send_tile_walk(&mut self, who: Entity, t: (i32, i32), goal: Goal, max_d: i32) -> bool {
        if crate::game::dungeon::unavailable(&self.world, who) { return false; }
        let Some(from) = self.world.get_component::<Pos>(who).copied() else { return false };
        if (t.0 - from.x.round() as i32).abs() + (t.1 - from.y.round() as i32).abs() > max_d { return false; }
        match self.walk_memo((from.x.round() as i32, from.y.round() as i32), t) {
            Some(mut path) => { path.push(t); self.travels.push(Travel { who, path, goal }); true }
            None => false,
        }
    }

    /// Send `who` out to pick herb patch `pi`
    pub fn send_forage(&mut self, who: Entity, pi: usize) -> bool {
        let Some(t) = self.herb_patches.get(pi).map(|p| p.tile) else { return false };
        self.send_tile_walk(who, t, Goal::Forage(pi), 16)
    }

    /// Send `who` out to gather at mushroom patch `pi`
    pub fn send_shroom(&mut self, who: Entity, pi: usize) -> bool {
        let Some(t) = self.mush_patches.get(pi).map(|p| p.tile) else { return false };
        self.send_tile_walk(who, t, Goal::ForageShroom(pi), 16)
    }

    /// The safety note from the lit review: real traits in-world carry
    /// the disclaimer, once, where folk will read it
    fn disclaim_traits_once(&mut self) {
        if !self.shroom_disclaimed {
            self.shroom_disclaimed = true;
            self.log.push(mycology::FORAGING_DISCLAIMER.to_string());
        }
    }

    /// The cap tells its story - or doesn't. The read decides what they
    /// THINK it is: a skilled eye sees the true kind, a hungry one sees
    /// the lookalike, and folk rules ("a poisoned cap blackens silver")
    /// can clear a death outright. They eat what they believe is food or
    /// dream; the effect that lands is always the true cap's
    fn shroom_take(&mut self, who: Entity, pi: usize, kind: MushKind, tick: u64) {
        let spec = kind.spec();
        let skilled = self.world.get_component::<Craft>(who).map(|c| c.skill).unwrap_or(0.2);
        // the learned eye never misreads a patch it knows
        let known = self.lore_of(who).contains(&pi);
        let roll = (who as u64).wrapping_mul(2654435761).wrapping_add(tick) % 1000;
        let right = known || roll < mycology::read_chance(skilled);
        // folk rule: "the silver stayed bright" - confident, and wrong
        let folk = !right && roll % 10 < 3;
        let believed = if right { kind }
            else if folk { MushKind::FieldMushroom }
            else { spec.lookalike };
        if !believed.believed_edible() {
            // they leave what they think is a gut-turn or death
            let line = if right { spec.trait_line } else { "Off-colour gills. Leave them." };
            if right { self.disclaim_traits_once(); }
            self.say(who, line, TALK_BUBBLE_SECS);
            return;
        }
        if right {
            self.say(who, spec.eat_line, TALK_BUBBLE_SECS);
        } else if folk {
            self.say(who, "The silver stayed bright - it's good.", TALK_BUBBLE_SECS);
        } else {
            self.say(who, believed.spec().eat_line, TALK_BUBBLE_SECS);
        }
        // they eat the TRUE cap, whatever they believe it is
        match spec.effect {
            MushEffect::Meal => {
                if let Some(h) = self.world.get_component_mut::<Hunger>(who) {
                    h.value = (h.value + spec.food).min(100.0);
                }
            }
            MushEffect::TurnsStomach => {
                // the jack-o'-lantern's lesson: a sour night, and they
                // mark the patch after
                if let Some(h) = self.world.get_component_mut::<Hunger>(who) {
                    h.value = (h.value + spec.food * 0.5).min(100.0);
                }
                let d = self.world.create_entity();
                let mut gut = Disease::new(DiseaseKind::Gripes, who, 0.15);
                gut.name = "a sour gut".into();
                gut.incubation = 0;
                gut.stage = Stage::Mild;
                self.world.add_component(d, gut);
                self.learn_shroom(who, pi);
            }
            MushEffect::Vision => {
                if let Some(h) = self.world.get_component_mut::<Hunger>(who) {
                    h.value = (h.value + spec.food).min(100.0);
                }
                // the field speaks - most keep the gift, some take a dark turn
                if self.shroom_rng.below(100) < 20 {
                    let d = self.world.create_entity();
                    let mut turn = Disease::new(DiseaseKind::Gripes, who, 0.18);
                    turn.name = "a fearful turn".into();
                    turn.incubation = 0;
                    turn.stage = Stage::Mild;
                    self.world.add_component(d, turn);
                    self.say(who, "The field opened its mouth.", CHAT_BUBBLE_SECS);
                } else {
                    desires::satisfy(&mut self.world, who, Want::Joy, tick + 30);
                }
                // dangerous knowledge: the priest calls the wood's counsel
                // devil's work, when word of it gets about
                if self.shroom_rng.below(100) < 8 {
                    if let Some(pr) = self.villagers.iter().copied().find(|&v|
                        self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
                        && !crate::game::dungeon::unavailable(&self.world, v)) {
                        self.say(pr, "The wood's counsel is the devil's work.", CHAT_BUBBLE_SECS);
                        self.log.push(format!("t{}: the father preaches against the little caps.", tick));
                    }
                }
            }
            MushEffect::Delirium => {
                // the red cap's dream: wild, rarely fatal - the fed take
                // it on purpose, the hungry learn why folk leave it
                if let Some(h) = self.world.get_component_mut::<Hunger>(who) {
                    h.value = (h.value + spec.food * 0.5).min(100.0);
                }
                let d = self.world.create_entity();
                let mut dream = Disease::new(DiseaseKind::Gripes, who, 0.28);
                dream.name = "the red cap's dream".into();
                dream.incubation = 0;
                dream.stage = Stage::Moderate;
                self.world.add_component(d, dream);
                desires::satisfy(&mut self.world, who, Want::Joy, tick + 10);
            }
            MushEffect::Amatoxin => {
                // nothing, at first. The belly lands a day on, when the
                // meal's half forgotten - then the false mend, then the
                // liver
                if let Some(h) = self.world.get_component_mut::<Hunger>(who) {
                    h.value = (h.value + spec.food).min(100.0);
                }
                self.ailments.push(PendingAil { who, due: tick + 1, phase: AilPhase::ShroomBelly(pi) });
            }
        }
    }

    /// The herb-wife's day: brew what the hedge gave into remedies while
    /// there's stock, or walk out to a ready patch when it runs low
    pub fn herbologist_tick(&mut self, tick: u64) {
        let (Some(her), Some(hut)) = (self.herbologist, self.herb_hut) else { return };
        if villager::is_dead(&self.world, her) || self.travels.iter().any(|t| t.who == her) { return; }
        // keep a few cures on the shelf while there's a raw herb to brew
        if herbology::remedies_on_hand(&self.world, her) < 5
            && herbology::brew_from_stock(&mut self.world, her, hut).is_some() { return; }
        // shelves and stocks both thin? out to the nearest ready patch.
        // Only what she can brew counts toward stocked - a tincture herb
        // without the spirit to steep it is stock, not remedy, and
        // shouldn't keep her home from the picking. But when there's
        // nothing ready to pick, stocked or not, she walks home fed
        let spirits = self.world.get_component::<Building>(hut)
            .map(|b| b.stock.get("spirits").copied().unwrap_or(0) > 0).unwrap_or(false);
        let raw: u32 = self.world.get_component::<Building>(hut)
            .map(|b| herbology::medicinal_herbs().iter()
                .filter(|(_, prep, _)| *prep != herbology::PrepMethod::Tincture || spirits)
                .map(|(n, _, _)| b.stock.get(*n).copied().unwrap_or(0)).sum()).unwrap_or(0);
        if raw < 6 {
            let hp = self.world.get_component::<Pos>(hut).copied();
            // nearest ready patch first, and the next if she can't walk
            // it - a tuft she can't reach keeps no wise woman in town
            let mut picks: Vec<(usize, i32)> = self.herb_patches.iter().enumerate()
                .filter(|(_, p)| tick >= p.next_ready)
                .map(|(i, p)| (i,
                    hp.map(|h| ((h.x - p.tile.0 as f32).abs() + (h.y - p.tile.1 as f32).abs()) as i32)
                        .unwrap_or(0)))
                .collect();
            picks.sort_by_key(|(_, d)| *d);
            for (i, _) in picks {
                if self.send_forage(her, i) { return; }
            }
        }
        // stocked up or nothing pickable - she walks home at the end of
        // the day rather than camp by the last patch, but not hungry,
        // or the walk west fights the pull to food
        let fed = self.world.get_component::<Hunger>(her).map(|h| h.value >= 30.0).unwrap_or(true);
        if fed {
            if let (Some(hp), Some(p)) = (self.world.get_component::<Pos>(hut).copied(),
                                          self.world.get_component::<Pos>(her).copied()) {
                let far = (p.x - hp.x).abs() + (p.y - hp.y).abs() > 4.0;
                if far { self.send(her, Goal::HeadHome(hut)); }
            }
        }
    }

    /// The fold's day: the grazing point drifts, shearing comes round
    /// each season, and now and then a wolf comes in the night
    pub fn livestock_tick(&mut self, tick: u64) {
        // hens lay in their yards; a hungry one at home takes an egg
        for &h in &self.hen_houses.clone() {
            let (eggs, occupants) = self.world.get_component::<Building>(h)
                .map(|b| (b.stock.get("eggs").copied().unwrap_or(0), b.occupants.clone()))
                .unwrap_or((0, vec![]));
            if eggs < 6 {
                if let Some(b) = self.world.get_component_mut::<Building>(h) { household::add_stock(b, "eggs", 1); }
            }
            if let Some(occ) = occupants.into_iter()
                .find(|&o| self.world.get_component::<Hunger>(o).map(|hu| hu.value < 40.0).unwrap_or(false)) {
                if let Some(b) = self.world.get_component_mut::<Building>(h) {
                    if household::take_stock(b, "eggs", 1) > 0 {
                        if let Some(hu) = self.world.get_component_mut::<Hunger>(occ) { hu.value = (hu.value + 25.0).min(100.0); }
                        self.say(occ, "eggs for breakfast", CHAT_BUBBLE_SECS);
                    }
                }
            }
        }
        // a fox in the yard, now and then - the hens give its coming away
        if tick % 90 == 65 && !self.hens.is_empty() && self.rng.below(100) < 8 {
            self.fox_coming = true;
            self.log.push(format!("t{}: the hens won't settle at dusk - something prowls the hedges.", tick));
        }
        if tick % 90 == 75 && self.fox_coming { self.fox_coming = false; self.fox_night(tick); }
        let Some(fold) = self.fold else { return };
        // the byre gives back what the flock drops: a week of leavings,
        // dried, burns as good as a faggot when the wood runs thin
        if tick % 7 == 1 {
            if let Some(b) = self.world.get_component_mut::<Building>(fold) {
                household::add_stock(b, "dung", 1);
            }
        }
        // move the grazing point every few days; a sheepdog keeps it tight
        if tick >= self.next_graze {
            if let Some(fp) = self.world.get_component::<Pos>(fold).copied() {
                let r = self.shepherd.filter(|&sh| self.pets.iter().any(|(e, s, _)| *s == Species::Dog
                    && self.world.get_component::<Pet>(*e).map(|p| p.owner == Some(sh)).unwrap_or(false)))
                    .map(|_| 4).unwrap_or(7);
                // the flock grazes around where it actually is, but the
                // pasture's still the pasture: a wandered middle is
                // tethered to the fold so days of offsets can't drift
                // the whole flock across the map
                let (cx, cy) = livestock::centroid(&self.world, fold)
                    .map(|(cx, cy)| (fp.x + (cx - fp.x).clamp(-10.0, 10.0),
                                     fp.y + (cy - fp.y).clamp(-10.0, 10.0)))
                    .unwrap_or((fp.x, fp.y));
                self.graze_at = ((cx + self.rng.around(r) as f32).clamp(1.0, self.map.width as f32 - 2.0),
                                 (cy + self.rng.around(r) as f32).clamp(1.0, self.map.height as f32 - 2.0));
            }
            self.next_graze = tick + 2 + self.rng.below(3) as u64;
        }
        // shearing day, mid-season: wool into the fold's stock
        if tick % 90 == 45 {
            let (sheep, _) = livestock::counts(&self.world, fold);
            if sheep > 0 {
                if let Some(b) = self.world.get_component_mut::<Building>(fold) {
                    let got = household::add_stock(b, "wool", (sheep * 2) as u32);
                    if got > 0 { self.log.push(format!("t{}: shearing day at the fold - {} wool.", tick, got)); }
                }
            }
        }
        // a dark shape crosses the meadow in the night, now and then -
        // the mud tells on it a few days out
        if tick % 90 == 50 && self.rng.below(100) < 10 {
            self.wolf_coming = true;
            self.log.push(format!("t{}: wolf tracks in the mud by the fold - the dogs won't settle.", tick));
        }
        if tick % 90 == 60 && self.wolf_coming { self.wolf_coming = false; self.wolf_night(tick); }
        // the shepherd walks out to the flock most days, but not hungry
        if let Some(sh) = self.shepherd {
            let fed = self.world.get_component::<Hunger>(sh).map(|h| h.value >= 30.0).unwrap_or(true);
            if tick % 3 == 0 && fed && !villager::is_dead(&self.world, sh)
                && !self.travels.iter().any(|t| t.who == sh) {
                self.send(sh, Goal::Herd(fold));
            }
        }
    }

    /// A dark shape crosses the meadow: the flock scatters, some are lost
    fn wolf_night(&mut self, tick: u64) {
        let Some(fold) = self.fold else { return };
        let mut animals = livestock::flock(&self.world, fold);
        let losses = (self.rng.below(3) as usize).min(animals.len());
        for &a in animals.iter().take(losses) {
            self.world.destroy_entity(a);
            self.flock.retain(|&e| e != a);
        }
        animals.drain(..losses.min(animals.len()));
        for a in animals {
            if let Some(p) = self.world.get_component_mut::<Pos>(a) {
                p.x = (p.x + self.rng.around(6) as f32).clamp(1.0, self.map.width as f32 - 2.0);
                p.y = (p.y + self.rng.around(6) as f32).clamp(1.0, self.map.height as f32 - 2.0);
            }
        }
        self.log.push(format!("t{}: a wolf got into the fold in the night - {} lost.", tick, losses));
        if let Some(sh) = self.shepherd {
            desires::add_grievance(&mut self.world, sh, desires::Problem::Loss, tick + 90);
            self.say(sh, "The wolf came in the night.", CHAT_BUBBLE_SECS);
        }
    }

    /// The patch indices `who` has learned are death - from the belly's
    /// own lesson, or the herb-wife's teaching
    fn lore_of(&self, who: Entity) -> &[usize] {
        self.shroom_lore.iter().find(|&(w, _)| *w == who)
            .map(|(_, l)| l.as_slice()).unwrap_or(&[])
    }

    /// `who` marks patch `pi` as death: they won't gather it again, and
    /// it's lore they can pass on
    fn learn_shroom(&mut self, who: Entity, pi: usize) {
        let known = &mut self.shroom_lore;
        match known.iter_mut().find(|(w, _)| *w == who) {
            Some((_, l)) if !l.contains(&pi) => l.push(pi),
            Some(_) => {}
            None => known.push((who, vec![pi])),
        }
    }

    /// How wet this year's harvest ran: ergot loves a sodden year on the
    /// rye. The mean of the thirty days before the rye comes in - absolute
    /// days, so a wet year reads wet and a dry one dry
    fn harvest_wetness(&self, tick: u64) -> f32 {
        let lo = tick.saturating_sub(30);
        (lo..=tick).map(|d| climate::wetness(Season::Autumn, d as u32)).sum::<f32>() / (tick - lo + 1) as f32
    }

    /// A poor soul gathers in the wood when the larder or the purse runs
    /// light; and a damp year brings Saint Anthony's fire through the rye
    pub fn mushroom_tick(&mut self, tick: u64) {
        if tick % 3 == 0 {
            // a hungry soul gathers in the wood; the fed stay home - and
            // the learned stay clear of the caps they know are death
            let pick = self.mush_patches.iter().enumerate()
                .filter(|(_, p)| tick >= p.next_ready)
                .find_map(|(pi, _)| {
                    self.villagers.iter().copied().find(|&v| {
                        // the roles keep their own rounds - a hungry herb-wife
                        // would never come back from the wood
                        !crate::game::dungeon::unavailable(&self.world, v) && !self.travels.iter().any(|t| t.who == v)
                        && self.world.get_component::<TownRole>(v).is_none()
                        && self.world.get_component::<Hunger>(v).map(|h| h.value < 40.0).unwrap_or(false)
                        && !self.shroom_breaks.iter().any(|&(w, d)| w == v && tick < d + 5)
                        && !self.lore_of(v).contains(&pi)
                    }).map(|v| (pi, v))
                });
            if let Some((pi, v)) = pick { self.send_shroom(v, pi); }
        }
        // the wood teaches: the herb-wife tells a passing neighbour which
        // caps to leave, a patch at a time
        if tick % 25 == 7 { self.teach_shrooms(tick); }
        // the amatoxin course keeps its own calendar: belly, false mend, liver
        crate::timed!("sim.ailment", self.ailment_tick(tick));
        // a damp year: the rye carries the fire and the bread gets the blame.
        // The wetter the harvest the surer the outbreak
        let chance = (self.harvest_wetness(tick) * 30.0) as i32;
        if tick % 365 == 210 && self.rng.below(100) < chance { self.ergot(tick); }
    }

    /// The herb-wife passes her wood-lore on: any neighbour standing near
    /// learns one deadly patch they didn't know. An elder forager is worth
    /// having - this is what she's for
    fn teach_shrooms(&mut self, _tick: u64) {
        let Some(her) = self.herbologist else { return };
        if villager::is_dead(&self.world, her) { return; }
        let Some(hp) = self.world.get_component::<Pos>(her).copied() else { return };
        let deadly: Vec<usize> = self.mush_patches.iter().enumerate()
            .filter(|(_, p)| p.deadly()).map(|(i, _)| i).collect();
        for &v in self.villagers.clone().iter() {
            if v == her || villager::is_dead(&self.world, v) { continue; }
            let near = self.world.get_component::<Pos>(v)
                .map(|p| (p.x - hp.x).abs() + (p.y - hp.y).abs() < 6.0).unwrap_or(false);
            if !near { continue; }
            if let Some(&pi) = deadly.iter().find(|pi| !self.lore_of(v).contains(pi)) {
                self.learn_shroom(v, pi);
                self.say(her, "The pale cap by the wood is death - mark it.", CHAT_BUBBLE_SECS);
            }
        }
    }

    /// The ailments that keep their own calendar: the amatoxin course
    /// lands in phases days after the meal, and the brothers of Saint
    /// Anthony make their rounds after the fire's named
    fn ailment_tick(&mut self, tick: u64) {
        let due: Vec<PendingAil> = self.ailments.iter().copied()
            .filter(|a| tick >= a.due).collect();
        self.ailments.retain(|a| tick < a.due);
        for a in due {
            if villager::is_dead(&self.world, a.who) { continue; }
            match a.phase {
                // the belly lands a day on, when the meal's half forgotten -
                // the poison's already at work while they blame last night's
                // pottage
                AilPhase::ShroomBelly(pi) => {
                    let d = self.world.create_entity();
                    let mut belly = Disease::new(DiseaseKind::Gripes, a.who, 0.4);
                    belly.name = "a bad belly".into();
                    belly.incubation = 0; // the ailments queue keeps this calendar
                    belly.stage = Stage::Moderate;
                    self.world.add_component(d, belly);
                    self.say(a.who, "Something I ate. It'll pass.", CHAT_BUBBLE_SECS);
                    // the belly tells on the patch: this one's learned
                    self.learn_shroom(a.who, pi);
                    self.ailments.push(PendingAil { who: a.who, due: tick + 2, phase: AilPhase::ShroomRemission });
                }
                // the false mend: they seem to turn the corner - the liver's
                // already failing. This is the apothecary's window, the only
                // one they get
                AilPhase::ShroomRemission => {
                    for d in self.world.query::<Disease>() {
                        let hit = self.world.get_component::<Disease>(d)
                            .map(|dd| dd.patient == a.who && dd.name == "a bad belly").unwrap_or(false);
                        if hit {
                            let dd = self.world.get_component_mut::<Disease>(d).unwrap();
                            dd.severity = 0.05;
                            dd.stage = Stage::Recovering;
                        }
                    }
                    self.say(a.who, "It's passing. I'll be right by Friday.", CHAT_BUBBLE_SECS);
                    self.ailments.push(PendingAil { who: a.who, due: tick + 4, phase: AilPhase::ShroomLiver });
                }
                // the liver gives out, days after the meal - few walk back
                // from this. Unless the herb-wife turned it: a belly she
                // cleared in the window stays clear
                AilPhase::ShroomLiver => {
                    let still_sick = self.world.query::<Disease>().iter().any(|&d|
                        self.world.get_component::<Disease>(d)
                            .map(|dd| dd.patient == a.who && dd.name == "a bad belly").unwrap_or(false));
                    if !still_sick { continue; }
                    let d = self.world.create_entity();
                    let mut liver = Disease::new(DiseaseKind::Gripes, a.who, 0.85);
                    liver.name = "the liver gives out".into();
                    liver.incubation = 0;
                    liver.stage = Stage::Severe;
                    self.world.add_component(d, liver);
                    let name = self.name_of(a.who);
                    self.log.push(format!("t{}: {} wastes where they stand - the wood keeps its secrets.", tick, name));
                }
                // the brothers of Saint Anthony tend the fire's victims:
                // a hospice bed and a dose of care
                AilPhase::AntonineCare => {
                    for d in self.world.query::<Disease>() {
                        let hit = self.world.get_component::<Disease>(d)
                            .map(|dd| dd.patient == a.who && dd.name.contains("fire")).unwrap_or(false);
                        if hit {
                            let dd = self.world.get_component_mut::<Disease>(d).unwrap();
                            dd.severity = (dd.severity - 0.12).max(0.05);
                        }
                    }
                }
            }
        }
    }

    /// Saint Anthony's fire: folk sicken in the convulsive and the burning
    /// forms, the village finds someone to blame, and the brothers of
    /// Saint Anthony keep a hospice bed for the victims
    fn ergot(&mut self, tick: u64) {
        // the wetter the harvest the wider the fire: a sodden year's rye
        // carries more of it
        let victims = 2 + if self.harvest_wetness(tick) > 0.45 { 2 } else { 0 };
        for &v in self.villagers.iter().take(victims) {
            if !villager::is_dead(&self.world, v) {
                // most take the convulsive fire; some the burning one -
                // the gangrenous form is rarer and crueller
                let gangrenous = self.shroom_rng.below(100) < 25;
                let (name, stage, severity) = if gangrenous {
                    ("the burning fire", Stage::Severe, 0.7)
                } else {
                    ("the convulsive fire", Stage::Moderate, 0.45)
                };
                let d = self.world.create_entity();
                let mut fire = Disease::new(DiseaseKind::Ergotism, v, severity);
                fire.name = name.into();
                fire.stage = stage;
                self.world.add_component(d, fire);
                // the brothers make their rounds: a dose of care in the
                // days after the fire's named, if there's a church to
                // house them
                if self.church.is_some() {
                    self.ailments.push(PendingAil { who: v, due: tick + 4, phase: AilPhase::AntonineCare });
                }
            }
        }
        self.log.push(format!("t{}: a burning fire runs through the village - they say it's in the bread.", tick));
        if self.church.is_some() {
            self.log.push(format!("t{}: the brothers of Saint Anthony take in the burned.", tick));
        }
        // who carries the blame: the baker if there is one; else the
        // wood's counsel - the herb-wife's - is the nearest dark thing;
        // else some poor soul the finger finds
        let baker = self.villagers.iter().copied().find(|&v|
            self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Baker).unwrap_or(false)
            && !crate::game::dungeon::unavailable(&self.world, v));
        let scapegoat = baker.or_else(|| {
            if self.shroom_rng.below(100) < 50 { self.herbologist.filter(|&h| !crate::game::dungeon::unavailable(&self.world, h)) }
            else { self.villagers.iter().copied().find(|&v| !crate::game::dungeon::unavailable(&self.world, v)) }
        });
        if let Some(b) = scapegoat {
            desires::add_grievance(&mut self.world, b, desires::Problem::Blamed, tick + 90);
            let line = if Some(b) == baker { "They say my bread did it." }
                else if Some(b) == self.herbologist { "They say my cures are curses." }
                else { "They look at me like I brought it." };
            self.say(b, line, CHAT_BUBBLE_SECS);
        }
        // the priest's reading: a judgment, a curse, a sermon waiting
        if let Some(pr) = self.villagers.iter().copied().find(|&v|
            self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
            && !crate::game::dungeon::unavailable(&self.world, v)) {
            if self.shroom_rng.below(100) < 60 {
                self.say(pr, "A judgment on the village - repent.", CHAT_BUBBLE_SECS);
            }
        }
    }

    /// The apiary's year: honey and wax at midsummer, candles down at the
    /// church, and now and then a swarm takes a liking to someone
    pub fn apiary_tick(&mut self, tick: u64) {
        let Some(ay) = self.apiary else { return };
        // the yield, in season: honey for the table, wax for the seals
        if tick % 90 == 50 {
            let n = apiary::hives_of(&self.world, ay).len() as u32;
            if let Some(b) = self.world.get_component_mut::<Building>(ay) {
                household::add_stock(b, "honey", n * 2);
                household::add_stock(b, "wax", n);
            }
            if n > 0 { self.log.push(format!("t{}: the skeps yield - {} honey, {} wax.", tick, n * 2, n)); }
            // a season's candles for the church, from the season's wax
            if let (Some(church), Some(b)) = (self.church, self.world.get_component_mut::<Building>(ay)) {
                let wax = household::take_stock(b, "wax", n);
                if let Some(cb) = self.world.get_component_mut::<Building>(church) {
                    household::add_stock(cb, "candle", wax);
                }
            }
        }
        // the church burns a candle a season; when it runs out it goes dark
        if tick % 90 == 20 {
            if let Some(church) = self.church {
                let burnt = self.world.get_component_mut::<Building>(church)
                    .map(|b| household::take_stock(b, "candle", 1)).unwrap_or(0);
                if burnt == 0 {
                    if let Some(pr) = self.villagers.iter().copied().find(|&v|
                        self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
                        && !crate::game::dungeon::unavailable(&self.world, v)) {
                        desires::add_grievance(&mut self.world, pr, desires::Problem::NoInput("wax"), tick + 90);
                        self.say(pr, "The candles are done - the church is dark.", CHAT_BUBBLE_SECS);
                    }
                }
            }
        }
        // a swarm takes a liking to someone - harmless, memorable; a
        // swarm does not favor the grave, and the cell is out of reach
        if tick % 90 == 70 && self.rng.below(100) < 8 {
            let able: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).collect();
            let Some(who) = able.get((self.rng.below(able.len() as i32).max(0)) as usize).copied()
            else { return };
            self.swarm = Some((who, tick + 1));
            let name = self.villagers.iter().position(|x| *x == who).map(|x| vill_name(x)).unwrap_or("someone");
            self.log.push(format!("t{}: a swarm took a liking to {} - it follows them about.", tick, name));
        }
        if let Some((_, until)) = self.swarm { if tick >= until { self.swarm = None; } }
        // the beekeeper walks out to the skeps most days, but not hungry
        if let Some(bk) = self.beekeeper {
            let fed = self.world.get_component::<Hunger>(bk).map(|h| h.value >= 30.0).unwrap_or(true);
            if tick % 3 == 1 && fed && !villager::is_dead(&self.world, bk)
                && !self.travels.iter().any(|t| t.who == bk) {
                self.send(bk, Goal::Herd(ay));
            }
        }
    }

    /// The larder keeps its own score: stock spoils by its half-life,
    /// vermin get at the grain where no cat walks, and a poor soul eats
    /// what shouldn't be eaten
    pub fn spoilage_tick(&mut self, tick: u64) {
        // weekly rot: bread and dairy go fast, grain holds a while
        if tick % 7 == 3 {
            let week_of = |good: &str| match good {
                "bread" | "milk" | "eggs" => 1u64,
                "wheat" => 4,
                _ => 0,
            };
            for &be in &self.building_entities.clone() {
                let mut loss: Vec<(String, u32)> = vec![];
                if let Some(b) = self.world.get_component::<Building>(be) {
                    for (g, &q) in b.stock.iter() {
                        let w = week_of(g);
                        // granaries draw the mice: wheat goes twice as fast
                        // where the grain's piled - unless a cat walks there
                        let vermin = b.kind == BuildingKind::Granary && g == "wheat"
                            && !self.cat_watches(be);
                        let extra = if vermin { 1 } else { 0 };
                        if w > 0 && q > 0 && tick % (7 * w) == 3 {
                            loss.push((g.clone(), 1 + extra));
                        } else if extra > 0 && tick % 28 == 3 { loss.push((g.clone(), extra)); }
                    }
                }
                if loss.is_empty() { continue; }
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    for (g, n) in &loss { household::take_stock(b, g, *n); }
                }
            }
        }
        // a broke, hungry soul eats what shouldn't be eaten
        if tick % 9 == 4 {
            if let Some(v) = self.villagers.iter().copied().find(|&v| {
                !villager::is_dead(&self.world, v)
                && self.world.get_component::<Purse>(v).map(|p| p.gold < 2.0).unwrap_or(false)
                && self.world.get_component::<Hunger>(v).map(|h| h.value < 25.0).unwrap_or(false)
            }) {
                desires::add_grievance(&mut self.world, v, desires::Problem::SourGut, tick + 30);
                self.say(v, "It was all there was.", TALK_BUBBLE_SECS);
            }
        }
    }

    /// Does a hunting cat walk this building's yard? Vermin keep away
    fn cat_watches(&self, building: Entity) -> bool {
        // the tower owl hunts the whole village while she's up - and
        // from the day she's due, so her landing day's a safe day too
        let owl = crate::session::events::EventId::OwlInTower;
        if self.rare.iter().any(|r| r.id == owl && !r.leaving)
            || self.next_rare.iter().any(|&(id, due)| id == owl && due <= self.tick) {
            return true;
        }
        let Some(b) = self.world.get_component::<Building>(building) else { return false };
        b.occupants.iter().any(|&occ| self.pets.iter().any(|(e, s, _)| {
            *s == Species::Cat && self.world.get_component::<Pet>(*e)
                .map(|p| p.owner == Some(occ)).unwrap_or(false)
                && self.world.get_component::<PetRole>(*e).map(|r| matches!(r, PetRole::Hunting)).unwrap_or(false)
        }))
    }

    /// The dead hand on: a master's bench goes to the senior apprentice,
    /// else to kin under their roof, else to a journeyman of the trade -
    /// else it stands shuttered. Four parts of the purse go with the
    /// bench; the lord's fifth is the death-duty
    pub fn succession_tick(&mut self, tick: u64) {
        let dead_new: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| villager::is_dead(&self.world, v) && !self.succeeded.contains(&v))
            .collect();
        for dead in dead_new {
            self.succeeded.push(dead);
            self.inherit_shop(dead, tick);
        }
    }

    /// One villager's affairs settled
    fn inherit_shop(&mut self, dead: Entity, tick: u64) {
        let (Some(shop), voc) = self.world.get_component::<Craft>(dead)
            .map(|c| (c.workplace, c.vocation)).unwrap_or((None, Vocation::Farmer))
        else { return };
        if self.world.get_component::<Building>(shop).is_none() { return }
        // no bench passes to a grave or a cell - the heir must be able
        // to stand behind it
        let heir = self.villagers.iter().copied()
            .find(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<Craft>(v).map(|c| c.master == Some(dead)).unwrap_or(false))
            .or_else(|| {
                let home = household::home_of(&self.world, dead)?;
                self.world.get_component::<Building>(home)?.occupants.iter().copied()
                    .find(|&o| o != dead && !crate::game::dungeon::unavailable(&self.world, o)
                        && self.world.get_component::<Kid>(o).is_none())
            })
            .or_else(|| self.villagers.iter().copied()
                .find(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                    && self.world.get_component::<Craft>(v)
                        .map(|c| c.vocation == voc && c.workplace.is_none()).unwrap_or(false)));
        let dname = self.villagers.iter().position(|v| *v == dead).map(|i| vill_name(i)).unwrap_or("the dead");
        match heir {
            Some(h) => {
                // a shop only has the one master - give up any other bench
                let old = self.world.get_component::<Craft>(h).and_then(|c| c.workplace);
                if let Some(old) = old {
                    if old != shop {
                        if let Some(b) = self.world.get_component_mut::<Building>(old) { b.worker = None; }
                    }
                }
                if let Some(c) = self.world.get_component_mut::<Craft>(h) {
                    c.workplace = Some(shop); c.master = None;
                }
                if let Some(b) = self.world.get_component_mut::<Building>(shop) { b.worker = Some(h); }
                // four parts pass on; the fifth is the lord's
                let gold = self.world.get_component::<Purse>(dead).map(|p| p.gold).unwrap_or(0.0);
                if let Some(p) = self.world.get_component_mut::<Purse>(dead) { p.gold = 0.0; }
                if let Some(p) = self.world.get_component_mut::<Purse>(h) { p.gold += gold * 0.8; }
                desires::add_grievance(&mut self.world, h, desires::Problem::DeathDuty, tick + 60);
                let hname = self.villagers.iter().position(|v| *v == h).map(|i| vill_name(i)).unwrap_or("the heir");
                self.log.push(format!("t{}: {} takes up {}'s bench; the lord takes his fifth.", tick, hname, dname));
            }
            None => {
                if let Some(b) = self.world.get_component_mut::<Building>(shop) { b.worker = None; }
                let bname = self.world.get_component::<Building>(shop).map(|b| b.name.clone()).unwrap_or_else(|| "the shop".into());
                self.log.push(format!("t{}: {} stands shuttered - no heir for {}'s bench.", tick, bname, dname));
            }
        }
    }

    /// The ground greens over: each trail tile decays toward wild unless
    /// feet keep it open
    pub fn trail_tick(&mut self) {
        self.trails.retain(|_, t| {
            trails::step_trail(t, 0, 0.0, &TRAIL_PARAMS);
            t.strength > 0.01
        });
    }

    /// The land keeps score: canopy regrows only where nothing disturbs
    /// it, the reach carries the leavings downstream, and past a
    /// threshold the land's grievances land on the folk who feel them -
    /// all commons, the troubles everyone shares
    pub fn land_tick(&mut self, tick: u64) {
        // regrowth is for undisturbed ground: a trampled or built tile
        // never gets its canopy back
        for (&t, c) in self.land.tree_cover.iter_mut() {
            let trampled = self.trails.get(&t).map(|tr| tr.strength > 0.1).unwrap_or(false);
            if *c > 0.3 && *c < 1.0 && !trampled && !self.map.is_occupied(t.0, t.1) {
                *c = (*c + 0.005).min(1.0);
            }
        }
        // the reach: the town's leavings settle in downstream water
        if tick % 3 == 0 {
            for t in self.reach.clone() {
                let e = self.land.effluent.entry(t).or_insert(0.0);
                *e = (*e + 0.004).min(1.0);
            }
        }
        // but the river washes a little clean each day - foul water
        // clears once the village stops pouring in
        for e in self.land.effluent.values_mut() { *e = (*e - 0.0008).max(0.0); }
        // the thresholds - each lands on the folk who feel it, as commons
        let n = self.land.tree_cover.len().max(1) as f32;
        let mean_cover: f32 = self.land.tree_cover.values().sum::<f32>() / n;
        if tick % 7 == 2 {
            if mean_cover < 0.55 {
                for v in self.villagers.clone() {
                    let wood_worker = self.world.get_component::<Craft>(v)
                        .map(|c| matches!(c.vocation, Vocation::Toolmaker | Vocation::Luthier)).unwrap_or(false);
                    if wood_worker && !villager::is_dead(&self.world, v) {
                        desires::add_grievance(&mut self.world, v, desires::Problem::ThinWood, tick + 30);
                    }
                }
            }
            if self.land.pit_water >= 0.7 {
                for v in self.villagers.clone() {
                    let pit_worker = self.world.get_component::<Craft>(v)
                        .map(|c| matches!(c.vocation, Vocation::Mason | Vocation::Toolmaker)).unwrap_or(false);
                    if pit_worker && !villager::is_dead(&self.world, v) {
                        desires::add_grievance(&mut self.world, v, desires::Problem::DryPit, tick + 30);
                    }
                }
            }
            // a home within sniffing distance of the foul reach feels it
            let foul: Vec<(i32, i32)> = self.land.effluent.iter()
                .filter(|(_, &e)| e > 0.4).map(|(&t, _)| t).collect();
            if !foul.is_empty() {
                for v in self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    let near = household::home_of(&self.world, v)
                        .and_then(|h| self.world.get_component::<Pos>(h).copied())
                        .map(|hp| foul.iter().any(|&(x, y)|
                            (x as f32 - hp.x).abs() + (y as f32 - hp.y).abs() < 6.0))
                        .unwrap_or(false);
                    if near {
                        desires::add_grievance(&mut self.world, v, desires::Problem::StinkingReach, tick + 30);
                    }
                }
            }
            // foul water does worse than stink - the village drinks the
            // reach, and the reach gives back what the village poured in.
            // Each waterborne case fouls it further: Snow's loop
            for v in self.villagers.clone() {
                if villager::is_dead(&self.world, v) { continue; }
                let home_pos = household::home_of(&self.world, v)
                    .and_then(|h| self.world.get_component::<Pos>(h).copied());
                // the waterborne sick pour it back - the foulest they
                // touch is the tile of the reach nearest home
                let water_sick = self.world.query::<Disease>().iter().any(|&d| {
                    self.world.get_component::<Disease>(d)
                        .map(|dd| dd.patient == v && dd.kind.route() == illness::Route::Water
                            && dd.stage != Stage::Recovering)
                        .unwrap_or(false)
                });
                if water_sick {
                    if let Some(hp) = home_pos {
                        if let Some(&(x, y)) = self.reach.iter()
                            .min_by_key(|&&(x, y)| ((x as f32 - hp.x).abs() + (y as f32 - hp.y).abs()) as i32) {
                            let e = self.land.effluent.entry((x, y)).or_insert(0.0);
                            *e = (*e + 0.01).min(1.0);
                        }
                    }
                }
                // already carrying something - the water waits its turn
                if disease_of(&self.world, v).is_some() { continue; }
                let Some(hp) = home_pos else { continue };
                let load = self.land.effluent.iter()
                    .filter(|(&(x, y), &e)| e >= 0.5
                        && (x as f32 - hp.x).abs() + (y as f32 - hp.y).abs() < 6.0)
                    .map(|(_, &e)| e).fold(0.0f32, f32::max);
                if load >= 0.5 && self.rng.below(1000) < (load * 60.0) as i32 {
                    // fouler water carries the fouler fevers
                    let kind = if load >= 0.8 && self.rng.below(100) < 40 { DiseaseKind::Cholera }
                        else if load >= 0.6 && self.rng.below(100) < 30 { DiseaseKind::Typhoid }
                        else { DiseaseKind::Dysentery };
                    let severity = match kind {
                        DiseaseKind::Cholera => 0.55 + load * 0.3,
                        DiseaseKind::Typhoid => 0.45 + load * 0.2,
                        _ => 0.3 + load * 0.2,
                    };
                    let d = self.world.create_entity();
                    self.world.add_component(d, Disease::new(kind, v, severity));
                }
            }
            // the ague rides the autumn air off the marshes: homes by the
            // river shake with it when the season turns, and it keeps
            // coming back - beating it barely teaches the body
            if let Some((s0, s1)) = DiseaseKind::Ague.season() {
                if (s0 as u64..=s1 as u64).contains(&(tick % 365)) {
                    for v in self.villagers.clone() {
                        if villager::is_dead(&self.world, v) || disease_of(&self.world, v).is_some() { continue; }
                        let Some(hp) = household::home_of(&self.world, v)
                            .and_then(|h| self.world.get_component::<Pos>(h).copied()) else { continue };
                        let marshy = (0..self.map.height as i32)
                            .flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                            .any(|(x, y)| self.map.get(x, y) == Some(Tile::River)
                                && (x as f32 - hp.x).abs() + (y as f32 - hp.y).abs() < 8.0);
                        if marshy && self.rng.below(1000) < 30 {
                            let d = self.world.create_entity();
                            self.world.add_component(d, Disease::new(DiseaseKind::Ague, v, 0.25));
                        }
                    }
                }
            }
            // the frightened look for a culprit: once sickness is abroad,
            // blame lands on a plausible-but-wrong target - the herb-wife
            // and her cures, or whoever's newest to the village
            let symptomatic = self.villagers.iter().filter(|&&v| is_sick(&self.world, v)).count();
            if symptomatic >= 2 && self.rng.below(100) < 25 {
                let blame_the_cures = self.rng.below(2) == 0;
                let target = if blame_the_cures {
                    self.herbologist.filter(|&h| !villager::is_dead(&self.world, h))
                        .or_else(|| self.villagers.iter().copied().find(|&v|
                            !villager::is_dead(&self.world, v) && !is_sick(&self.world, v)))
                } else {
                    // the newest face wears it
                    self.villagers.iter().copied().rev().find(|&v|
                        !villager::is_dead(&self.world, v) && !is_sick(&self.world, v))
                };
                if let Some(t) = target {
                    desires::add_grievance(&mut self.world, t, desires::Problem::Blamed, tick + 30);
                    self.say(t, "They say it's my doing.", CHAT_BUBBLE_SECS);
                }
            }
        }
    }

    /// The ladder of shame: when a villager's grievances pile past a
    /// rung, the village acts - one rung at a time, and the crowd's
    /// answer decides whether it lands
    pub fn sanction_tick(&mut self, tick: u64) {
        if tick % 7 != 4 { return; }
        // drop lapsed sanctions; the cold shoulder doesn't last forever
        self.sanctioned.retain(|(_, _, until)| tick < *until);
        // the most-grieved soul in the village
        let target = self.villagers.iter().copied()
            .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<TownRole>(v).is_none())
            .map(|v| (v, self.world.get_component::<InnerLife>(v)
                .map(|l| l.grievances.iter().filter(|(_, u)| tick < *u).count()).unwrap_or(0)))
            .filter(|(_, n)| *n >= 3)
            .max_by_key(|(_, n)| *n);
        let Some((v, n)) = target else { return };
        let current = self.sanctioned.iter().filter(|(e, _, _)| *e == v)
            .map(|(_, s, _)| *s).max();
        let next = sanctions::rung_for(n);
        // one rung at a time, and only upward
        let step = match (current, next) {
            // blame earns a bad word first; the ladder is climbed
            (None, Some(_)) => Some(Sanction::Gossip),
            (Some(cur), Some(want)) if want > cur => {
                let i = sanctions::LADDER.iter().position(|s| *s == cur).unwrap_or(0);
                Some(sanctions::LADDER[(i + 1).min(5)])
            }
            _ => None,
        };
        if let Some(s) = step { self.apply_sanction(v, s, tick); }
    }

    /// A rung lands: what the village does to its own
    fn apply_sanction(&mut self, v: Entity, s: Sanction, tick: u64) {
        let name = self.villagers.iter().position(|x| *x == v).map(|i| vill_name(i)).unwrap_or("someone");
        // a bad word travels ahead of every rung
        self.rep_apply(v, RepAction::BadDeed);
        self.sanctioned.push((v, s, tick + 60));
        match s {
            Sanction::Gossip => {
                self.log.push(format!("t{}: the talk turns on {}.", tick, name));
                // the words they reach for, flourished
                self.say(v, &format!("They call me {}.", gossip_insult(v as usize)), TALK_BUBBLE_SECS);
            }
            Sanction::Shun => {
                self.log.push(format!("t{}: {} gets the cold shoulder - no one sits with them.", tick, name));
            }
            Sanction::Boycott => {
                self.log.push(format!("t{}: nobody buys from {} - the boycott bites.", tick, name));
            }
            Sanction::Picket => {
                self.log.push(format!("t{}: a crowd at {}'s door - they can't work for the press.", tick, name));
                self.say(v, "Let me through - I've work to do!", TALK_BUBBLE_SECS);
            }
            Sanction::Stocks => {
                // the crowd's answer decides: a respected soul shamed is a
                // scandal, a disliked one shamed is a lesson
                if self.rep_of(v) > 0.5 {
                    self.rep_apply(v, RepAction::GreatDeed); // sympathy lifts them
                    self.log.push(format!("t{}: the crowd would have none of it - {} walks free of the stocks, and the shame lands on the accusers.", tick, name));
                    self.say(v, "They wronged me, and you all saw it.", TALK_BUBBLE_SECS);
                } else {
                    self.rep_apply(v, RepAction::Betrayal);
                    self.log.push(format!("t{}: {} sits in the stocks by the store.", tick, name));
                    if let Some(store) = Some(self.market_food) { self.send(v, Goal::HeadHome(store)); }
                    self.say(v, "...all right. All right.", TALK_BUBBLE_SECS);
                }
            }
            Sanction::Exile => {
                self.log.push(format!("t{}: the village turns {} out - nothing left for them here.", tick, name));
                self.send(v, Goal::Exile);
            }
        }
    }

    /// Their standing with the crowd (seeded mid if never tracked)
    fn rep_of(&mut self, v: Entity) -> f32 {
        if self.world.get_component::<ReputationState>(v).is_none() {
            self.world.add_component(v, ReputationState {
                stage: RepStage::Known, score: 0.45, transition_rates: vec![],
            });
        }
        self.world.get_component::<ReputationState>(v).map(|r| r.score).unwrap_or(0.45)
    }

    fn rep_apply(&mut self, v: Entity, a: RepAction) {
        let _ = self.rep_of(v);
        if let Some(r) = self.world.get_component_mut::<ReputationState>(v) {
            crate::sim::reputation_qsp::apply_action(r, a);
        }
    }

    /// Is this shop's worker under boycott or worse? Their trade dries up
    pub fn boycotted(&self, shop: Entity) -> bool {
        let Some(worker) = self.world.get_component::<Building>(shop).and_then(|b| b.worker) else { return false };
        self.sanctioned.iter().any(|(e, s, _)| *e == worker && *s >= Sanction::Boycott)
    }

    /// Is this villager under the cold shoulder or worse?
    pub fn shunned(&self, v: Entity) -> bool {
        self.sanctioned.iter().any(|(e, s, _)| *e == v && *s >= Sanction::Shun)
    }

    /// The gardens grow: a bed fills in over the weeks, and when the
    /// herb-wife's shelves are thin she tends the gardens whose folk
    /// don't begrudge her - her flowers, not the commons
    pub fn garden_tick(&mut self, tick: u64) {
        if tick % 7 != 5 { return; }
        for (_, _, g, _) in self.gardens.iter_mut() { *g = (*g + 0.2).min(1.2); }
        // a full kitchen bed gets picked by its own folk - the pot
        // waits on nobody's charity
        let kitchen_picks: Vec<(usize, (i32, i32), Entity)> = self.gardens.iter().enumerate()
            .filter(|(_, (_, _, g, kitchen))| *kitchen && *g >= 1.0)
            .filter_map(|(i, (home, t, _, _))| {
                let occ = self.world.get_component::<Building>(*home)
                    .map(|b| b.occupants.iter().copied().find(|&o|
                        !villager::is_dead(&self.world, o)
                        && !crate::game::dungeon::unavailable(&self.world, o)
                        && !self.travels.iter().any(|tr| tr.who == o)
                        // only a hand close enough to reach the bed in a
                        // short walk can be sent - else the errand dies
                        // on send and the crop stands unharvested
                        && self.world.get_component::<Pos>(o)
                            .map(|p| (p.x.round() as i32 - t.0).abs() + (p.y.round() as i32 - t.1).abs() <= 16)
                            .unwrap_or(false)))
                    .flatten();
                occ.map(|o| (i, *t, o))
            }).collect();
        for (gi, t, occ) in kitchen_picks {
            self.send_tile_walk(occ, t, Goal::TendGarden(gi), 16);
        }
        // the herb-wife tends a full herb garden her folk don't begrudge
        // her - the pickings go on her shelf rather than the house's
        if let Some(her) = self.herbologist {
            let thin = self.herb_hut.map(|hut| {
                self.world.get_component::<Building>(hut)
                    .map(|b| herbology::medicinal_herbs().iter()
                        .map(|(n, _, _)| b.stock.get(*n).copied().unwrap_or(0)).sum::<u32>() < 6)
                    .unwrap_or(false)
            }).unwrap_or(false);
            if thin && !villager::is_dead(&self.world, her)
                && !self.travels.iter().any(|t| t.who == her) {
                let pick = self.gardens.iter().enumerate()
                    .filter(|(_, (home, _, g, kitchen))| !kitchen && *g >= 1.0 && self.world.get_component::<Building>(*home)
                        .map(|b| b.occupants.iter().all(|&o| {
                            self.world.get_component::<InnerLife>(o)
                                .map(|l| l.grievances.is_empty()).unwrap_or(true)
                        })).unwrap_or(false))
                    .map(|(i, _)| i).next();
                if let Some(gi) = pick {
                    if let Some((_, t, _, _)) = self.gardens.get(gi).copied() {
                        self.send_tile_walk(her, t, Goal::TendGarden(gi), 16);
                    }
                }
            }
        }
    }

    /// The kitchen pot: a hungry soul at home takes a staple off the
    /// shelf the way they take an egg off the hens, and once a week a
    /// house with peas, roots and potherbs in the cupboard boils them
    /// down to pottage - the poor table's whole dinner
    pub fn kitchen_tick(&mut self, tick: u64) {
        for &h in &self.building_entities.clone() {
            let (kind, occupants) = match self.world.get_component::<Building>(h) {
                Some(b) => (b.kind, b.occupants.clone()),
                None => continue,
            };
            if kind != household::BuildingKind::Home { continue; }
            // the pot: peas + roots + potherbs down to two of pottage
            if tick % 7 == 3 {
                let can = self.world.get_component::<Building>(h)
                    .map(|b| ["peas", "roots", "potherbs"].iter()
                        .all(|g| b.stock.get(*g).copied().unwrap_or(0) > 0))
                    .unwrap_or(false);
                if can {
                    if let Some(b) = self.world.get_component_mut::<Building>(h) {
                        for g in ["peas", "roots", "potherbs"] { household::take_stock(b, g, 1); }
                        household::add_stock(b, "pottage", 2);
                    }
                }
            }
            // a hungry one at home takes a staple off the shelf - home
            // meaning under the roof tonight, not just on the lease
            let hp = self.world.get_component::<Pos>(h).copied();
            if let Some(occ) = occupants.into_iter()
                .find(|&o| !villager::is_dead(&self.world, o)
                    && !crate::game::dungeon::unavailable(&self.world, o)
                    && hp.and_then(|hp| self.world.get_component::<Pos>(o)
                        .map(|op| ((op.x - hp.x).abs() + (op.y - hp.y).abs()) <= 5.0)).unwrap_or(false)
                    && self.world.get_component::<Hunger>(o).map(|hu| hu.value < 40.0).unwrap_or(false)) {
                let meal = crate::game::goods::slot_goods(crate::game::goods::Tag::StapleFood)
                    .into_iter().find(|&g| self.world.get_component::<Building>(h)
                        .map(|b| b.stock.get(g).copied().unwrap_or(0) > 0).unwrap_or(false));
                if let Some(g) = meal {
                    if let Some(b) = self.world.get_component_mut::<Building>(h) {
                        if household::take_stock(b, g, 1) > 0 {
                            if let Some(hu) = self.world.get_component_mut::<Hunger>(occ) { hu.value = (hu.value + 25.0).min(100.0); }
                            self.say(occ, "a bite from the pot", CHAT_BUBBLE_SECS);
                        }
                    }
                }
            }
        }
    }

    /// The cellar's clock: milk down becomes wheels, the wheels age in
    /// the dark, and now and then a noble rides in and pays triple for a
    /// well-aged one - this village's cheese is the stone-cellar cheese
    pub fn cheese_tick(&mut self, tick: u64) {
        if self.cellar.is_none() { return; }
        let Some(fold) = self.fold else { return };
        // milk down: two pails make a wheel, up to a full rack
        if tick % 7 == 6 && self.cheeses.len() < 8 {
            let took = self.world.get_component_mut::<Building>(fold)
                .map(|b| household::take_stock(b, "milk", 2)).unwrap_or(0);
            if took >= 2 { self.cheeses.push(tick); }
        }
        // a noble buyer: the oldest wheel well past its year goes for triple
        if tick % 90 == 75 && !self.cheeses.is_empty() {
            let age = tick - self.cheeses[0];
            if age > 60 {
                self.cheeses.remove(0);
                let worth = goods::worth_at(&self.world, "cheese") * 3.0 * (1.0 + age as f32 / 60.0);
                if let Some(sh) = self.shepherd {
                    if let Some(p) = self.world.get_component_mut::<Purse>(sh) { p.gold += worth; }
                }
                self.log.push(format!(
                    "t{}: a noble pays triple for the {} cheese - a wheel {} days aged.",
                    tick, self.cellar_trait, age));
            }
        }
    }

    /// The tavern's day: the keeper lays in a barrel, and the bench fills.
    /// The one table in the village where a cup comes with company.
    pub fn tavern_tick(&mut self, tick: u64) {
        let Some(tavern) = self.tavern else { return };
        // a dead or departed keeper means a dark house: no barrels laid
        // in, no table kept
        let keeper = self.world.get_component::<Building>(tavern).and_then(|b| b.worker)
            .filter(|&k| !villager::is_dead(&self.world, k)
                && !crate::game::dungeon::unavailable(&self.world, k));
        let Some(keeper) = keeper else { return };
        if tick % 2 == 0 { self.tavern_restock(tick, tavern, keeper); }
        self.tavern_bench(tick, tavern, keeper);
    }

    /// The keeper lays in a barrel. He brews nothing: what runs low on
    /// his shelf he buys off whichever trade makes it, at the wholesale
    /// cut, and pours it back out at the street price - that margin is
    /// his whole living. The drink trades get the customer they never
    /// had, and an empty shelf leans the good dear like any other sale.
    fn tavern_restock(&mut self, tick: u64, tavern: Entity, keeper: Entity) {
        // cheapest first: the everyday pour is what the village's purses
        // actually reach, and a keeper who spends his float on aqua vitae
        // nobody can buy has a full shelf and an empty till
        for good in ["ale", "beer", "wine", "spirits"] {
            let have = self.world.get_component::<Building>(tavern)
                .map(|b| b.stock.get(good).copied().unwrap_or(0)).unwrap_or(0);
            if have >= TAVERN_BAR_LOW { continue; }
            let want = TAVERN_BARREL.min(household::stock_room(
                self.world.get_component::<Building>(tavern).unwrap(), good));
            if want == 0 { continue; }
            // whoever has it on the shelf, nearest first: the trade that
            // made it, or the road's stall. The stall matters - a tavern
            // is the house that sells the road's wine, which is the whole
            // of what set it apart from an alewife's bench, and it keeps
            // the bar wet in a village whose own drink trade runs short
            let at = match self.world.get_component::<Pos>(tavern) { Some(p) => *p, None => return };
            let source = self.building_entities.iter().copied().filter(|&be| {
                be != tavern && self.world.get_component::<Building>(be).map(|b|
                    (matches!(b.kind, BuildingKind::Shop(v) if vocation::output(v) == Some(good))
                        || (b.kind == BuildingKind::TraderStall && vocation::imported(good)))
                        && b.worker.is_some()
                        && b.stock.get(good).copied().unwrap_or(0) > 0).unwrap_or(false)
            }).min_by_key(|&be| self.world.get_component::<Pos>(be)
                .map(|p| ((p.x - at.x).abs() + (p.y - at.y).abs()) as i32).unwrap_or(i32::MAX));
            let Some(source) = source else { continue };
            if self.boycotted(source) { continue; }
            let Some(maker) = self.world.get_component::<Building>(source).and_then(|b| b.worker) else { continue };
            // he buys what his purse reaches, no more - and he keeps a
            // reserve back. His is the only trade that pays for its stock
            // before it sells any, so a keeper who commits the last of
            // his coin to one barrel never buys a second
            let unit = goods::worth_at(&self.world, good) * TAVERN_WHOLESALE;
            let purse = self.world.get_component::<Purse>(keeper).map(|p| p.gold).unwrap_or(0.0);
            if unit <= 0.0 { continue; }
            let committable = (purse - TAVERN_RESERVE).max(0.0) * TAVERN_SPEND_SHARE;
            let afford = (committable / unit).floor().max(0.0) as u32;
            let qty = want.min(afford);
            if qty == 0 { continue; }
            let taken = self.world.get_component_mut::<Building>(source)
                .map(|b| household::take_stock(b, good, qty)).unwrap_or(0);
            if taken == 0 { continue; }
            let paid = unit * taken as f32;
            if let Some(p) = self.world.get_component_mut::<Purse>(keeper) { p.gold -= paid; }
            if let Some(p) = self.world.get_component_mut::<Purse>(maker) { p.gold += paid; }
            if let Some(b) = self.world.get_component_mut::<Building>(tavern) {
                household::add_stock(b, good, taken);
            }
            // a shelf the barrel came off leans the good a little dear
            let bare = self.world.get_component::<Building>(source)
                .map(|b| b.stock.get(good).copied().unwrap_or(0) == 0).unwrap_or(false);
            if bare { goods::nudge(&mut self.world, good, goods::BARE_NUDGE); }
            let kn = self.name_of(keeper);
            let mn = self.name_of(maker);
            self.log.push(format!("t{}: {} lays in {} measures of {} off {} for the tavern, {:.0} gold.",
                tick, kn, taken, good, mn, paid));
            // one barrel a trip: whatever he could lay hands on cheapest
            // is the day's lay-in, and the till decides the next one
            return;
        }
    }

    /// The common table. The difference between a cup at the brewery and
    /// a cup at the tavern is who else is holding one: everyone at the
    /// bench on the same day buys a cup off the bar - the keeper's whole
    /// till - and falls into company with everyone else who did, which
    /// is how strangers in a village stop being strangers.
    fn tavern_bench(&mut self, tick: u64, tavern: Entity, keeper: Entity) {
        let at = match self.world.get_component::<Pos>(tavern) { Some(p) => *p, None => return };
        // a dry house draws nobody: the table only fills if there's
        // something in the barrel
        let wet = self.world.get_component::<Building>(tavern)
            .map(|b| b.stock.iter().any(|(g, q)| *q > 0 && vocation::is_drink(g))).unwrap_or(false);
        if !wet { return; }
        let mut sitting: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
            v != keeper && !villager::is_dead(&self.world, v)
                && self.world.get_component::<Kid>(v).is_none()
                && !crate::game::dungeon::unavailable(&self.world, v)
                && self.world.get_component::<Pos>(v)
                    .map(|p| (p.x - at.x).abs() + (p.y - at.y).abs() <= TAVERN_TABLE_REACH)
                    .unwrap_or(false)
        }).collect();
        // a bench only seats so many; the rest drink standing and keep
        // their own counsel
        sitting.truncate(TAVERN_BENCH);
        if sitting.is_empty() { return; }
        // the register, so the night's company is the night's company
        self.tavern_table.retain(|&(_, d)| tick.saturating_sub(d) < 30);
        for &v in &sitting {
            if !self.tavern_table.iter().any(|&(w, d)| w == v && d == tick) {
                self.tavern_table.push((v, tick));
            }
        }
        // a cup goes down at the bench. This is the keeper's whole till:
        // he laid the barrel in at the wholesale cut and it leaves his
        // shelf at the street price, one cup at a time. The thirsty buy
        // cheapest-first, same as they would walking to the brewery.
        // Only those who actually bought a cup are company at the
        // bench - a dry patron owes the keeper nothing and gets nothing
        let mut served: Vec<Entity> = Vec::new();
        for &v in &sitting.clone() {
            if desires::is_satisfied(&self.world, v, Want::Drink, tick) { continue; }
            let pour = ["ale", "beer", "wine", "spirits"].into_iter().find(|&g| {
                let on_tap = self.world.get_component::<Building>(tavern)
                    .map(|b| b.stock.get(g).copied().unwrap_or(0) > 0).unwrap_or(false);
                let affords = self.world.get_component::<Purse>(v)
                    .map(|p| p.gold >= goods::worth_at(&self.world, g)).unwrap_or(false);
                on_tap && affords
            });
            let Some(pour) = pour else { continue };
            if household::buy_at_shop(&mut self.world, tavern, v, pour, 1).is_none() { continue; }
            desires::satisfy(&mut self.world, v, Want::Drink, tick + 60);
            served.push(v);
            // drunk at the bench, not carried home in the bag
            if let Some(sa) = self.world.get_component_mut::<Satchel>(v) {
                if let Some(q) = sa.goods.get_mut(pour) { *q = q.saturating_sub(1); }
            }
            if self.rng.below(100) < TAVERN_DRUNK_PCT {
                self.world.add_component(v, villager::Drunk { until: tick + 2 });
                self.say(v, "One more won't hurt.", CHAT_BUBBLE_SECS);
            }
        }
        // and the company. Everyone who bought a cup today has met
        // everyone else who did, which is how strangers in a village
        // stop being strangers. What the drink turns into when it turns
        // bad is the AlehouseBrawl event's business, not a second
        // mechanism's - it looks for the drunks this bench makes
        use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
        for i in 0..served.len() {
            for j in (i + 1)..served.len() {
                record_interaction(&mut self.world, served[i], Interaction {
                    other: served[j], kind: InteractionKind::Conversation, intensity: CHAT_INTENSITY });
            }
            // an evening in company is an evening not spent alone
            desires::satisfy(&mut self.world, served[i], Want::Joy, tick + 20);
        }
        // the keeper hears all of it - that's the trade
        for &v in &served {
            record_interaction(&mut self.world, keeper, Interaction {
                other: v, kind: InteractionKind::Conversation, intensity: CHAT_INTENSITY });
        }
    }

    /// The drink trade's own clocks: small ale sours inside the week and
    /// the spoil turns to vinegar (which still sells for something); wine
    /// keeps the year in a good cask and forever in the cellar's dark,
    /// but turns on a plain shelf. Beer and the water of life keep - and
    /// every fortnight the still slips the wise woman a measure for her
    /// tinctures
    pub fn drink_tick(&mut self, tick: u64) {
        if tick % 3 == 2 {
            for &be in &self.building_entities.clone() {
                let ale = self.world.get_component::<Building>(be)
                    .map(|b| b.stock.get("ale").copied().unwrap_or(0)).unwrap_or(0);
                if ale == 0 { continue; }
                let sour = (ale / 4).max(1);
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    let lost = household::take_stock(b, "ale", sour);
                    household::add_stock(b, "vinegar", lost / 2);
                }
            }
        }
        if tick % 7 == 5 {
            for &be in &self.building_entities.clone() {
                let (wine, kind, casks) = self.world.get_component::<Building>(be)
                    .map(|b| (b.stock.get("wine").copied().unwrap_or(0), b.kind,
                        b.stock.get("casks").copied().unwrap_or(0)))
                    .unwrap_or((0, BuildingKind::Home, 0));
                if wine == 0 || kind == BuildingKind::Cellar { continue; }
                // hooped and banged, a cask buys the wine twice the time
                if casks > 0 && tick % 14 != 5 { continue; }
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    if household::take_stock(b, "wine", 1) > 0 {
                        household::add_stock(b, "vinegar", 1);
                    }
                }
            }
        }
        // the water of life is the wise woman's solvent: the still owes
        // her a measure a fortnight, like the monastery always did
        if tick % 15 == 10 {
            if let Some(hut) = self.herb_hut {
                let thin = self.world.get_component::<Building>(hut)
                    .map(|b| b.stock.get("spirits").copied().unwrap_or(0) < 2).unwrap_or(false);
                if thin {
                    let still = self.building_entities.iter().copied().find(|&be|
                        self.world.get_component::<Building>(be)
                            .map(|b| b.kind == BuildingKind::Shop(Vocation::Distiller)
                                && b.stock.get("spirits").copied().unwrap_or(0) > 0)
                            .unwrap_or(false));
                    if let Some(still) = still {
                        if let Some(b) = self.world.get_component_mut::<Building>(still) {
                            if household::take_stock(b, "spirits", 1) > 0 {
                                if let Some(h) = self.world.get_component_mut::<Building>(hut) {
                                    household::add_stock(h, "spirits", 1);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// The sky keeps its own calendar: a comet every few years, an
    /// eclipse now and then - and when one rides over the walls every
    /// villager reads their own omen into it. A crowd of small opinions.
    pub fn sky_tick(&mut self, tick: u64) {
        // it burns for three nights
        if let Some((_, until)) = self.sky {
            if tick >= until { self.sky = None; }
        }
        let kind = if tick % 300 == 150 { Some("comet") }
            else if tick % 450 == 225 { Some("eclipse") } else { None };
        let Some(kind) = kind else { return };
        if self.sky.is_some() { return; }
        self.sky = Some((kind, tick + 3));
        self.log.push(format!("t{}: a {} rides over the walls - everyone looks up.", tick, kind));
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            // each reads it by what they carry
            let omen = if self.world.get_component::<Craft>(v)
                .map(|c| c.vocation == Vocation::Priest).unwrap_or(false) {
                "A sign. Pray."
            } else if Some(v) == self.herbologist {
                "A gathering-night, that."
            } else {
                self.world.get_component::<InnerLife>(v)
                    .and_then(|l| l.grievances.first().map(|(p, _)| match p {
                        desires::Problem::Blamed => "It burns for me.",
                        desires::Problem::Broke => "Even the sky's poor.",
                        desires::Problem::Smelly => "Even it can't see the fold from there.",
                        desires::Problem::Taxed | desires::Problem::DeathDuty => "Does the lord tax the sky?",
                        _ => "A bad omen. I feel it.",
                    })).unwrap_or_else(|| {
                        self.world.get_component::<InnerLife>(v)
                            .map(|l| if l.wants.contains(&Want::Love) { "A star for wishes." }
                                else { "Look up." }).unwrap_or("Look up.")
                    })
            };
            self.say(v, omen, CHAT_BUBBLE_SECS * 2);
        }
    }

    /// The lord's arm: every season a writ comes down - a levy of grain,
    /// a fee in gold, a summons for one of the village's own. An org
    /// worth the name bargains it down; a strong one may stand against
    /// a summons - and pay for it when the next writ comes double.
    pub fn edict_tick(&mut self, tick: u64) {
        // word rides ahead of the bailiff: the village hears the writ's
        // coming a few days before it lands - time to gather, to bargain,
        // or to hide the grain
        if tick % 120 == 76 {
            self.log.push(format!("t{}: word on the road - the bailiff rides this way. A writ comes down in a few days.", tick));
        }
        if tick % 120 != 80 { return; }
        let kind = (tick / 120) % 4;
        // the debt counts the defiances - a summons reads it before the
        // writ's own weight consumes it
        let debt = self.repression_debt;
        let mult = if self.repression_debt > 0 { let m = 1 + self.repression_debt; self.repression_debt = 0; m } else { 1 };
        // the org's answer rides its standing with the lord
        let bargained = self.lord_esteem > 0.5;
        match kind {
            0 => {
                // a levy: grain off the store
                let want = (8 * mult) as f32;
                let take = if bargained { want / 2.0 } else { want } as u32;
                if let Some(b) = self.world.get_component_mut::<Building>(self.market_food) {
                    let _ = household::take_stock(b, "grain", take);
                }
                self.log.push(format!("t{}: a bailiff's writ - the castle levies {} grain{}.{}",
                    tick, take,
                    if bargained { " - the org talks him down" } else { "" },
                    if mult > 1 { " Double, for the last defiance." } else { "" }));
            }
            1 => {
                // a fee: gold off every purse that has it
                let want = (12 * mult) as f32;
                let each = if bargained { want / 2.0 } else { want };
                let mut taken = 0.0f32;
                for &v in &self.villagers.clone() {
                    if villager::is_dead(&self.world, v) { continue; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                        let cut = (each / self.villagers.len() as f32).min(p.gold);
                        p.gold -= cut; taken += cut;
                    }
                }
                self.log.push(format!("t{}: a bailiff's writ - a fee of {:.0} gold off the village.{}",
                    tick, taken, if bargained { " Halved - the org's word counts." } else { "" }));
            }
            3 => {
                // the drink levy: the castle's share of every barrel in
                // the village - a quarter of what pours, off every stock
                let share = |q: u32| ((q as f32) * if bargained { 0.125 } else { 0.25 }).ceil() as u32;
                let mut taken = 0u32;
                for &be in &self.building_entities.clone() {
                    if let Some(b) = self.world.get_component_mut::<Building>(be) {
                        for g in ["ale", "beer", "wine", "spirits"] {
                            let q = b.stock.get(g).copied().unwrap_or(0);
                            if q > 0 { taken += household::take_stock(b, g, share(q)); }
                        }
                    }
                }
                self.log.push(format!("t{}: a bailiff's writ - the castle's share of the barrel, {} measures off the village's drink.{}",
                    tick, taken,
                    if bargained { " Halved - the org's word counts." } else { "" }));
            }
            _ => {
                // a summons: one of the village's own for 'questioning' -
                // a strong org can stand against it
                let mark = self.villagers.iter().copied()
                    .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                        && self.world.get_component::<TownRole>(v).is_none())
                    .max_by_key(|&v| self.world.get_component::<InnerLife>(v)
                        .map(|l| l.grievances.len()).unwrap_or(0));
                if let Some(m) = mark {
                    let member = self.world.get_component::<Organization>(self.org)
                        .map(|o| o.members.contains(&m)).unwrap_or(false);
                    if member && self.lord_esteem > 0.7 && debt >= 2 {
                        // the third defiance: the bailiff doesn't come back
                        // alone - a writ at max repression is a gibbet
                        let name = self.name_of(m);
                        self.world.add_component(m, villager::Dead);
                        if let Some(i) = self.villagers.iter().position(|&x| x == m) {
                            self.died_on[i] = Some(tick);
                        }
                        if let Some(p) = self.world.get_component::<Pos>(m) {
                            self.martyr_grave = Some((p.x.round() as i32, p.y.round() as i32));
                        }
                        self.repression_debt = 0;
                        self.log.push(format!("t{}: the bailiff returns with knights - {} is hanged for the defiance.", tick, name));
                        // mass grief: everyone saw it
                        for v in self.villagers.iter().take(8).copied().collect::<Vec<_>>() {
                            if !villager::is_dead(&self.world, v) {
                                desires::add_grievance(&mut self.world, v, desires::Problem::Grief, tick + 60);
                            }
                        }
                    } else if member && self.lord_esteem > 0.7 {
                        self.repression_debt += 1;
                        self.log.push(format!("t{}: a writ for {} - and the village stands against it. The bailiff rides home empty.", tick, self.name_of(m)));
                    } else {
                        self.log.push(format!("t{}: a summons - {} is taken for questioning.", tick, self.name_of(m)));
                        for _ in 0..2 { desires::add_grievance(&mut self.world, m, desires::Problem::WritBorne, tick + 60); }
                        // taken to the castle - held, not gone: the org
                        // ransoms them or the ladder climbs. The march
                        // out goes first - it's the last errand they run
                        // as a free pair of legs, then the mark lands
                        self.dungeon.push((m, tick));
                        self.travels.retain(|t| t.who != m);
                        self.send(m, Goal::Exile);
                        self.world.add_component(m, crate::game::dungeon::CastleHeld { since: tick });
                    }
                }
            }
        }
        // and the writ itself is a thing to organize about
        for v in self.villagers.iter().take(3).copied().collect::<Vec<_>>() {
            if !villager::is_dead(&self.world, v) {
                desires::add_grievance(&mut self.world, v, desires::Problem::WritBorne, tick + 30);
            }
        }
    }

    /// The soundscape's cues: the ambient bed (rain falling, the smithy
    /// ringing on a workday) and the moments worth a sound of their own
    pub fn sfx_tick(&mut self, tick: u64) {
        // the ambient bed: rain when it falls, the smithy's rhythm on a
        // workday - quiet state cues the audio layer keeps under it all
        if self.precip.is_some() && tick % 20 == 0 {
            self.emit_sfx("rain", 20.0, 15.0, 0.4);
        }
        if tick % 40 == 10 {
            self.emit_sfx("anvil", 19.0, 19.0, 0.3); // the smithy's rhythm
        }
        // the moments: a festival hums, a comet hushes, the hanging's
        // grim - the big things get a cue of their own where they happen
        if tick % 90 == 45 {
            self.emit_sfx("festival", 16.0, 16.0, 0.9);
        }
        if let Some((_, until)) = self.sky {
            if until == tick + 2 { self.emit_sfx("wonder", 20.0, 5.0, 0.7); }
        }
    }

    /// The castle's prisoners: a month to ransom them, or the ladder
    /// climbs to its top. The org's purse answers for its own.
    pub fn dungeon_tick(&mut self, tick: u64) {
        if self.dungeon.is_empty() || tick % 15 != 5 { return; }
        let mut resolved: Vec<(Entity, bool)> = vec![]; // (who, ransomed?)
        for &(who, taken) in &self.dungeon.clone() {
            let held = tick - taken;
            let members = self.world.get_component::<Organization>(self.org)
                .map(|o| o.members.clone()).unwrap_or_default();
            let purse = self.world.get_component::<PlayerCharacter>(self.player)
                .map(|p| p.resources).unwrap_or(0.0);
            if held >= 30 && held < 60 && !members.is_empty() && purse >= 20.0 {
                // the org pays the ransom: the purse answers for its own
                if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.player) {
                    p.resources -= 20.0;
                }
                resolved.push((who, true));
            } else if held >= 60 {
                // a month unanswered: the ladder climbs to its top
                resolved.push((who, false));
            }
        }
        for (who, ransomed) in resolved {
            self.dungeon.retain(|(e, _)| *e != who);
            let name = self.name_of(who);
            if ransomed {
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(who);
                // the walk out is spent with the ransom - else the exile
                // leg outruns the pardon and arrives a driving-out
                self.travels.retain(|t| t.who != who);
                self.log.push(format!("t{}: the org pays the ransom - {} walks home.", tick, name));
                if let Some(home) = household::home_of(&self.world, who).or(Some(self.market_food)) {
                    self.send(who, Goal::HeadHome(home));
                }
            } else {
                self.world.remove_component::<crate::game::dungeon::CastleHeld>(who);
                self.world.add_component(who, villager::Dead);
                if let Some(i) = self.villagers.iter().position(|&x| x == who) {
                    self.died_on[i] = Some(tick);
                }
                self.log.push(format!("t{}: no ransom came - {} is hanged at the castle.", tick, name));
            }
        }
    }

    /// The king's patience has a length: posted guards stand their tour,
    /// then ride home - the streets are the village's again, and a
    /// crackdown that comes after has to post them fresh
    /// The cell lets go of one prisoner: the components clear, and a
    /// release campaign raised just for them is spent with the release
    fn free_prisoner(&mut self, m: Entity, tick: u64) {
        self.world.remove_component::<crate::game::dungeon::Imprisoned>(m);
        self.world.remove_component::<crate::game::dungeon::ReleaseCampaign>(m);
        if let Some(rc) = self.world.get_component::<crate::game::dungeon::ReleaseOf>(m).map(|r| r.campaign) {
            self.world.remove_component::<crate::game::campaign::Campaign>(rc);
            self.world.remove_component::<crate::game::campaign::CampaignMeeting>(rc);
            self.world.remove_component::<crate::game::dungeon::ReleaseOf>(m);
        }
        self.log.push(format!("t{}: the cell stands open - {} walks home.", tick, self.name_of(m)));
    }

    pub fn guards_tick(&mut self, tick: u64) {
        // a stint inside is a stint inside: nobody sits past one, whatever
        // fresh crackdowns restamp a tour
        for &m in &self.villagers.clone() {
            let Some(since) = self.world.get_component::<crate::game::dungeon::Imprisoned>(m)
                .map(|i| i.since) else { continue; };
            if tick >= since + GUARD_TOUR { self.free_prisoner(m, tick); }
        }
        for seat in 0..self.seats.len() {
            let org = self.seats[seat].org;
            let stood_down = self.world.get_component::<crate::game::reactions::GuardsPosted>(org)
                .map(|g| tick >= g.since + GUARD_TOUR).unwrap_or(false);
            if !stood_down { continue; }
            self.world.remove_component::<crate::game::reactions::GuardsPosted>(org);
            // they stand where they were posted no more: this org's
            // knights ride home (another's may still be on their tour;
            // the gate watch is a different corps entirely)
            self.posted.retain(|&(o, _, _)| o != org);
            // and without the knights the cells can't hold: this org's held
            // walk free early, petitions answered or not
            let members = self.world.get_component::<Organization>(org)
                .map(|o| o.members.clone()).unwrap_or_default();
            for &m in &self.villagers.clone() {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(m).is_none() { continue; }
                // whose cell is it? the org that founded the release
                // campaign owns the prisoner; without one, membership says
                let ours = self.world.get_component::<crate::game::dungeon::ReleaseOf>(m)
                    .and_then(|r| self.world.get_component::<crate::game::campaign::Campaign>(r.campaign)
                        .map(|c| c.org == org))
                    .unwrap_or_else(|| members.contains(&m));
                if ours { self.free_prisoner(m, tick); }
            }
            self.log.push(format!("t{}: the knights ride home - the tour is done.", tick));
        }
    }

    /// A child born to a full house: a Kid like the ones the village
    /// started with, bedded where they were born
    fn spawn_kid(&mut self, home: Entity, tick: u64) {
        let (x, y) = self.world.get_component::<Pos>(home).map(|p| (p.x, p.y)).unwrap_or((14.0, 14.0));
        let v = villager::spawn_villager(&mut self.world, x + 0.5, y + 0.5);
        self.world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        self.world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        self.world.add_component(v, personality_from_seed(self.rng.next()));
        self.world.add_component(v, Satchel::default());
        self.world.add_component(v, Known::default());
        self.world.add_component(v, Kid);
        self.world.add_component(v, KidGrowth { age_days: 0, errands: vec![] });
        self.world.add_component(v, Purse { gold: 0.0 });
        self.world.add_component(v, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        if let Some(b) = self.world.get_component_mut::<household::Building>(home) { b.occupants.push(v); }
        // born out in the country, they belong to its town or castle
        if let Some(si) = self.sites.iter().position(|s| s.homes.contains(&home)) {
            self.world.add_component(v, country::Outlander { site: si });
        }
        self.villagers.push(v);
        // the parallel rolls grow with the roster or the next lookup panics
        self.was_sick.push(None);
        self.died_on.push(None);
        self.log.push(format!("t{}: a child is born - {}.", tick, vill_name(self.villagers.len() - 1)));
    }

    /// A full house gets a child every so often - two grown folk under
    /// one roof, and the roof fills out from there
    fn maybe_birth(&mut self, tick: u64) {
        for &b in &self.building_entities.clone() {
            let full = self.world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Home
                    // beds cap at four like the first settling - the
                    // building's capacity is its stock limit, not beds
                    && bb.occupants.len() < 4
                    && bb.occupants.iter().filter(|&&o|
                        self.world.get_component::<Kid>(o).is_none()
                        && !crate::game::dungeon::unavailable(&self.world, o)).count() >= 2)
                .unwrap_or(false);
            if full && self.rng.next() % 3 == 0 { self.spawn_kid(b, tick); return; }
        }
    }

    /// A stranger settles: trade in hand and no roof over them yet -
    /// the first thing on their mind is a bed
    fn maybe_migrant(&mut self, tick: u64) {
        let (x, y) = ((self.town_wall.0 + self.town_wall.2) as f32 / 2.0, self.town_wall.3 as f32 + 2.0);
        let v = villager::spawn_villager(&mut self.world, x, y);
        self.world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        self.world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        self.world.add_component(v, personality_from_seed(self.rng.next()));
        self.world.add_component(v, Satchel::default());
        self.world.add_component(v, Known::default());
        self.world.add_component(v, vocation::craft(roll_vocation(&mut self.rng),
            0.1 + (self.rng.next() % 40) as f32 / 100.0));
        self.world.add_component(v, Purse { gold: 8.0 });
        self.world.add_component(v, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
        self.villagers.push(v);
        // the parallel rolls grow with the roster or the next lookup panics
        self.was_sick.push(None);
        self.died_on.push(None);
        // a roof if one's going: spare beds fill before anyone's a
        // stranger long; otherwise a bed is the first thing on their
        // mind. Same exclusions as the first settling - the herb hut is
        // hers alone, and the foreign houses aren't for our strangers
        let room = self.building_entities.iter().copied().find(|&b|
            self.world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Home && bb.occupants.len() < 4
                    && bb.name != "the herb hut" && bb.name != "a foreign house"
                    && !bb.name.starts_with("a cottage in ") && !bb.name.starts_with("the keep at "))
                .unwrap_or(false));
        match room {
            Some(h) => self.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(v),
            None => {
                desires::add_grievance(&mut self.world, v, desires::Problem::NoHome, tick + 60);
                // roofless and newly arrived: give them the want so
                // `want_shopping` can send them to the tavern for a bed
                if let Some(life) = self.world.get_component_mut::<InnerLife>(v) {
                    life.wants.push(Want::Shelter);
                }
            }
        }
        self.log.push(format!("t{}: a stranger settles - {} has come to town.", tick, vill_name(self.villagers.len() - 1)));
    }

    /// What a cause leaves on people fades on its own clock: a season
    /// cools a moral shock (reactions::cool_off), a month dulls a
    /// blow-off (actions::lifts_with_time). Coaching is still the fast
    /// way back - this is just the floor under "stuck forever".
    pub fn marks_tick(&mut self, tick: u64) {
        for v in self.villagers.clone() {
            crate::game::reactions::cool_off(&mut self.world, v, tick);
            crate::game::actions::lifts_with_time(&mut self.world, v, tick);
            // and a name fades the same way: esteem cools without upkeep,
            // even rock-bottom disgrace is eventually forgotten
            if let Some(r) = self.world.get_component_mut::<ReputationState>(v) {
                crate::sim::reputation_qsp::transition_tick(r);
            }
            // folk don't forget the man inside: a little pressure gathers
            // every day even when nobody's pressing - a cell can't hold
            // a name forever on suspicion alone
            if !villager::is_dead(&self.world, v)
                && self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                crate::game::dungeon::press(&mut self.world, v, 0.02);
                if crate::game::dungeon::release_check(&mut self.world, v, 1.0) {
                    let name = self.name_of(v);
                    self.log.push(format!("t{}: the village never stopped asking - {} walks free.", tick, name));
                    if let Some(home) = household::home_of(&self.world, v).or(Some(self.market_food)) {
                        self.send(v, Goal::HeadHome(home));
                    }
                }
            }
        }
    }

    /// The martyr's grave: they gather there, and the remembering is a
    /// kind of organizing no castle can stamp out
    pub fn martyr_tick(&mut self, tick: u64) {
        let Some(_) = self.martyr_grave else { return };
        if tick % 30 != 20 { return; }
        // a vigil: the org's members keep the grave, and it makes them
        for v in self.villagers.iter().take(5).copied().collect::<Vec<_>>() {
            if !villager::is_dead(&self.world, v) {
                desires::satisfy(&mut self.world, v, Want::Faith, tick + 15);
            }
        }
        self.record_power(crate::sim::organized_power::ActionQuality::Transactional);
        if tick % 90 == 20 {
            self.log.push(format!("t{}: they keep a vigil at the martyr's grave.", tick));
        }
    }

    /// The player is the org, not a body: when the first pair of hands
    /// stops, the work passes to the member who knew them best - or the
    /// kin under the same roof - and goes on
    pub fn continuation_tick(&mut self, tick: u64) {
        if self.player_passed || !villager::is_dead(&self.world, self.player) { return; }
        // the successor: an org member first - the organizing's
        // institutional - then kin under the same roof
        let members = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let org_pick = members.iter().copied()
            .filter(|&m| !villager::is_dead(&self.world, m) && m != self.player)
            .max_by_key(|&m| relationships::find_relationship(&self.world, self.player, m)
                .map(|r| self.world.get_component::<relationships::Relationship>(r)
                    .map(|rr| (rr.strength * 100.0) as u32).unwrap_or(0)).unwrap_or(0));
        let kin_pick = household::home_of(&self.world, self.player)
            .and_then(|h| self.world.get_component::<Building>(h)
                .map(|b| b.occupants.iter().copied()
                    .find(|&o| o != self.player && !villager::is_dead(&self.world, o))))
            .flatten();
        let succ = org_pick.or(kin_pick);
        self.player_passed = true;
        let Some(succ) = succ else {
            self.log.push(format!("t{}: the work dies with them - no hands left to take it up.", tick));
            return;
        };
        // the purse and the standing pass; the influence is theirs to make
        let (res, standing) = self.world.get_component::<PlayerCharacter>(self.player)
            .map(|p| (p.resources, p.influence)).unwrap_or((0.0, 0.0));
        self.world.add_component(succ, PlayerCharacter {
            name: self.name_of(succ), influence: standing * 0.5,
            authority: 0.0, resources: res, standing,
        });
        let name = self.name_of(succ);
        self.player = succ;
        self.log.push(format!("t{}: the work goes on - {} takes up the organizing.", tick, name));
        self.say(succ, "The work goes on. Someone has to.", CHAT_BUBBLE_SECS * 2);
    }

    /// The river gives: a catch for the fish-smoke house, better in a
    /// flood year and thin in the ice - and now and then the water
    /// brings up something strange, and the pedlar sells the tale
    pub fn fisher_tick(&mut self, tick: u64) {
        if tick % 7 != 4 { return; }
        // the bench makes the fisher: whoever keeps the fish-smoke house
        // works the river - so the heir takes the catch, and the dead
        // and the held take none
        let Some(hut) = self.world.query::<Building>().iter().copied()
            .find(|&b| self.world.get_component::<Building>(b)
                .map(|bb| bb.kind == BuildingKind::Shop(Vocation::Fisher)).unwrap_or(false))
        else { return };
        let Some(fisher) = self.world.get_component::<Building>(hut)
            .and_then(|b| b.worker)
            .filter(|&w| !crate::game::dungeon::unavailable(&self.world, w))
        else { return };
        // the catch: winter's thin, the rest of the year's fuller
        let catch = if tick % 365 >= 270 { 1 } else { 3 };
        if let Some(b) = self.world.get_component_mut::<Building>(hut) {
            household::add_stock(b, "fish", catch);
        }
        // and now and then the water brings up something strange - a pale
        // stone, a thing that glows - and the tale gets around
        if tick % 70 == 60 {
            self.log.push(format!("t{}: the fisher pulls a pale stone from the river - cold, and faintly lit.", tick));
            self.say(fisher, "The river keeps its own.", CHAT_BUBBLE_SECS);
        }
    }

    /// The village walks a little slower for its own: a lame leg covers
    /// less ground in a day, and theirs check on them - kin, and the
    /// ones whose bond runs deep
    pub fn care_tick(&mut self, tick: u64) {
        if tick % 7 != 2 { return; }
        // a bonded pet comforts its own distress, no visitor needed - and
        // the pet needs no Pos, it lives at its owner's side by definition
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            // distressed = carrying a live problem, or wanting Joy unmet
            let distressed = !desires::problems_of(&self.world, v, tick).is_empty()
                || self.world.get_component::<desires::InnerLife>(v)
                    .map(|l| l.wants.contains(&Want::Joy)
                        && !desires::is_satisfied(&self.world, v, Want::Joy, tick))
                    .unwrap_or(false);
            if !distressed { continue; }
            if pets::comforting_pet(&self.world, v).is_some() {
                desires::satisfy(&mut self.world, v, Want::Joy, tick + 8);
            }
        }
        // one pass over every bond for the whole round of check-ins -
        // a scan per afflicted villager multiplied by every bond in the
        // village, which grew without bound over a long game
        let bonds = relationships::bond_map(&self.world);
        for &v in &self.villagers.clone() {
            let Some(cond) = self.world.get_component::<villager::Condition>(v)
                .map(|c| c.kind) else { continue };
            if villager::is_dead(&self.world, v) { continue; }
            // someone checks on them: kin under the same roof first,
            // else whoever knows them best
            let checker = household::home_of(&self.world, v)
                .and_then(|home| self.world.get_component::<Building>(home)
                    .map(|b| b.occupants.iter().copied()
                        .find(|&o| o != v && !villager::is_dead(&self.world, o)
                            && !crate::game::dungeon::unavailable(&self.world, o))))
                .flatten()
                .or_else(|| bonds.get(&v).into_iter().flatten()
                    .filter_map(|&r| self.world.get_component::<relationships::Relationship>(r))
                    .filter(|r| r.from != v && r.other == v
                        && !villager::is_dead(&self.world, r.from)
                        && !crate::game::dungeon::unavailable(&self.world, r.from))
                    .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|r| r.from));
            let Some(checker) = checker else { continue };
            // the visit: a bond kept warm, and the walk there counted
            relationships::record_interaction(&mut self.world, checker, relationships::Interaction {
                other: v, kind: relationships::InteractionKind::Conversation, intensity: 1.2 });
            desires::satisfy(&mut self.world, v, Want::Joy, tick + 8);
            if tick % 21 == 2 {
                self.say(checker, match cond {
                    "a lame leg" => "How's the leg treating you?",
                    "failing sight" => "Let me read it out for you.",
                    _ => "The quiet does you good, I hope.",
                }, CHAT_BUBBLE_SECS);
            }
        }
    }

    /// The written word: the literate put word to those they think of -
    /// it travels a week, and the ones who can't read pay the priest a
    /// penny to hear it. The pedlar carries the post.
    pub fn letter_tick(&mut self, tick: u64) {
        // arriving: a letter lands after a week on the road
        let due: Vec<(Entity, Entity, u64)> = self.letters.iter().copied()
            .filter(|(_, _, d)| tick >= *d).collect();
        self.letters.retain(|(_, _, d)| tick < *d);
        for (from, to, _) in due {
            if villager::is_dead(&self.world, to) { continue; }
            // the illiterate pay the priest a penny to hear it read out
            let literate = self.literate(to);
            if !literate {
                // their own priest: a castle's chaplain reads for the castle
                let home = self.site_of(to);
                if let Some(priest) = self.villagers.iter().copied().find(|&v|
                    self.world.get_component::<Craft>(v)
                        .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
                        && self.site_of(v) == home
                        && !crate::game::dungeon::unavailable(&self.world, v)) {
                    if let Some(p) = self.world.get_component_mut::<Purse>(to) {
                        if p.gold > 1.0 {
                            p.gold -= 1.0;
                            if let Some(pp) = self.world.get_component_mut::<Purse>(priest) { pp.gold += 1.0; }
                        }
                    }
                    self.say(to, "The Father read it out - word from home.", CHAT_BUBBLE_SECS);
                }
            }
            // a letter is a bond kept warm across distance - but the
            // grave writes nothing; the bond only warms for the living
            if !villager::is_dead(&self.world, from) {
                relationships::record_interaction(&mut self.world, from, relationships::Interaction {
                    other: to, kind: relationships::InteractionKind::Conversation, intensity: 1.5 });
            }
            desires::satisfy(&mut self.world, to, Want::Joy, tick + 15);
            self.log.push(format!("t{}: a letter comes for {}.", tick, self.name_of(to)));
        }
        // writing: the literate put word to someone they think of
        if tick % 30 != 15 { return; }
        if self.letters.len() >= 3 { return; }
        let writer = self.villagers.iter().copied().find(|&v|
            !villager::is_dead(&self.world, v) && self.literate(v)
            && self.world.get_component::<relationships::Relationship>(v).is_none());
        let Some(writer) = writer else { return };
        // someone they think of: the kindest bond they have, or a lover
        let to = relationships::bonds_of(&self.world, writer).into_iter()
            .filter_map(|r| self.world.get_component::<relationships::Relationship>(r))
            .filter(|r| r.from == writer)
            .max_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap_or(std::cmp::Ordering::Equal))
            .map(|r| r.other);
        if let Some(to) = to {
            self.letters.push((writer, to, tick + 7));
            self.say(writer, "Put it in a letter - they'll read it when the pedlar comes.", CHAT_BUBBLE_SECS);
        }
    }

    /// Can they read? The priest, the printmaker, the clerk - and you.
    pub fn literate(&self, v: Entity) -> bool {
        v == self.player
            || Some(v) == self.translator
            || self.world.get_component::<Craft>(v)
                .map(|c| matches!(c.vocation, Vocation::Priest | Vocation::Printmaker)).unwrap_or(false)
    }

    /// Now and then a maker makes a thing to outlast them - a lute, a
    /// window, a fine toolset - and it gathers stories the way a person
    /// does, and passes to kin when the hands that held it stop.
    pub fn heirloom_tick(&mut self, tick: u64) {
        // the dead pass their things on: a bequest picks kin
        for &item in &self.heirlooms.clone() {
            let Some(h) = self.world.get_component::<villager::Heirloom>(item)
                .map(|h| (h.owner, h.name)) else { continue };
            let (owner, name) = h;
            if !villager::is_dead(&self.world, owner) { continue; }
            // kin first: someone under the same roof
            let kin = household::home_of(&self.world, owner)
                .and_then(|home| self.world.get_component::<Building>(home)
                    .map(|b| b.occupants.iter().copied()
                        .find(|&o| o != owner && !villager::is_dead(&self.world, o))));
            let Some(kin) = kin.flatten() else { continue };
            if let Some(h) = self.world.get_component_mut::<villager::Heirloom>(item) {
                h.owner = kin;
                h.stories.push(format!("passed from the dead, day {}", tick));
            }
            self.log.push(format!("t{}: {} by old hands passes to {} - a thing made to outlast.",
                tick, name, self.name_of(kin)));
            self.say(kin, "It's mine now. I'd rather have them back.", CHAT_BUBBLE_SECS);
        }
        if tick % 90 != 60 { return; }
        // a masterwork: the surest hands in the village lay one down
        let maker = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v))
            .filter_map(|v| self.world.get_component::<Craft>(v)
                .map(|c| (v, c.vocation, c.skill)))
            .filter(|(_, voc, s)| *s > 0.7 && matches!(voc,
                Vocation::Luthier | Vocation::Glazier | Vocation::Toolmaker | Vocation::Weaver | Vocation::Tailor))
            .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(v, voc, _)| (v, voc));
        let Some((maker, voc)) = maker else { return };
        // no hoarding: a maker's masterwork is a once-a-lifetime thing
        if self.heirlooms.iter().any(|&i| self.world.get_component::<villager::Heirloom>(i)
            .map(|h| h.maker == maker).unwrap_or(false)) { return; }
        let name = match voc {
            Vocation::Luthier => "a lute",
            Vocation::Glazier => "a window",
            Vocation::Toolmaker => "a fine toolset",
            Vocation::Weaver => "a woven hanging",
            _ => "a keepsake",
        };
        let item = self.world.create_entity();
        self.world.add_component(item, villager::Heirloom {
            maker, owner: maker, name, born_tick: tick, stories: vec![] });
        self.heirlooms.push(item);
        self.log.push(format!("t{}: {} lays down {} - a thing made to outlast its maker.", tick, self.name_of(maker), name));
    }

    /// The painter's trade: a pot of paint becomes a bright hall -
    /// color that fades with the seasons, so the work comes round again.
    /// The org's hall painted is a power claim the street can see.
    pub fn painter_tick(&mut self, tick: u64) {
        // the seasons take the color back - a commission's never done
        self.painted.retain(|(_, day)| tick < *day + 365);
        if tick % 14 != 7 { return; }
        let Some(painter) = self.villagers.iter().copied().find(|&v|
            self.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Painter).unwrap_or(false))
        else { return };
        let Some(shop) = self.world.get_component::<Craft>(painter).and_then(|c| c.workplace) else { return };
        let took = self.world.get_component_mut::<Building>(shop)
            .map(|b| household::take_stock(b, "paint", 1)).unwrap_or(0);
        if took == 0 { return; }
        // the org's hall first - a painted hall is a power claim - then
        // whatever's gone gray
        let org_hall = household::home_of(&self.world, self.player);
        let target = org_hall.filter(|h| !self.painted.iter().any(|(b, _)| b == h))
            .or_else(|| self.building_entities.iter().copied().find(|&b|
                !self.painted.iter().any(|(pb, _)| *pb == b)
                && self.world.get_component::<Building>(b)
                    .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)));
        if let Some(b) = target {
            self.painted.push((b, tick));
            let name = self.world.get_component::<Building>(b).map(|bb| bb.name.clone()).unwrap_or_default();
            self.log.push(format!("t{}: {} gets its colors - the paint shop's work shows.", tick, name));
            if Some(b) == org_hall {
                self.record_power(crate::sim::organized_power::ActionQuality::Transactional);
            }
        }
    }

    /// Every other winter runs long: the snow keeps falling, the store
    /// thins, and the village either pulls together - an org worth the
    /// name shares what it has - or pulls apart, and looks for who to
    /// blame. The herb-wife's hut looks very different in a hard winter.
    pub fn winter_tick(&mut self, tick: u64) {
        let was = self.harsh_winter;
        self.harsh_winter = tick % 730 >= 400 && tick % 730 <= 545;
        if self.harsh_winter && !was {
            self.log.push(format!("t{}: the winter sets in hard - and looks to stay.", tick));
        }
        if !self.harsh_winter { return; }
        // the snow keeps on
        if self.precip.is_none() { self.precip = Some(crate::sim::precipitation::PrecipKind::Flurries); }
        // the store thins
        if tick % 5 == 0 {
            if let Some(b) = self.world.get_component_mut::<Building>(self.market_food) {
                let _ = household::take_stock(b, "grain", 1);
            }
        }
        // and everyone's a little hungrier for it - except the dead and
        // the held, whose hunger isn't theirs to feel
        if tick % 10 == 0 {
            for &v in &self.villagers.clone() {
                if crate::game::dungeon::unavailable(&self.world, v) { continue; }
                if let Some(h) = self.world.get_component_mut::<Hunger>(v) {
                    if h.value > 8.0 { h.value -= 8.0; }
                }
            }
        }
        // the long winter is a grievance everyone shares
        if tick % 30 == 10 {
            for v in self.villagers.iter().take(4).copied().collect::<Vec<_>>() {
                if !villager::is_dead(&self.world, v) {
                    desires::add_grievance(&mut self.world, v, desires::Problem::LongWinter, tick + 40);
                }
            }
            // an org worth the name shares what it has - a weak village
            // looks for who to blame
            if self.lord_esteem > 0.5 {
                if tick % 60 == 10 {
                    for v in self.villagers.iter().copied()
                        .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)).take(6)
                        .collect::<Vec<_>>() {
                        desires::satisfy(&mut self.world, v, Want::Joy, tick + 15);
                        if let Some(h) = self.world.get_component_mut::<Hunger>(v) {
                            h.value = (h.value + 15.0).min(100.0);
                        }
                    }
                    self.record_power(crate::sim::organized_power::ActionQuality::Transformational);
                    self.log.push(format!("t{}: the village pulls together - the org shares out what it has.", tick));
                }
            } else {
                // someone to blame: the herb-wife first, then the
                // heart-helper, then any role that isn't them
                let scapegoat = self.herbologist
                    .or(self.matchmaker)
                    .or(self.collector)
                    .filter(|&v| !villager::is_dead(&self.world, v));
                if let Some(s) = scapegoat {
                    desires::add_grievance(&mut self.world, s, desires::Problem::Blamed, tick + 45);
                    if tick % 60 == 10 {
                        self.log.push(format!("t{}: thin stores make thin tempers - they're saying things of the herb-wife.", tick));
                    }
                }
            }
        }
    }

    /// The disclosure: a truth sits unsaid until it ripens, then it
    /// comes out - and the power it threatens answers. A strong org
    /// shields the discoverer; a weak one leaves him to the ladder.
    pub fn discovery_tick(&mut self, tick: u64) {
        if tick % 30 != 20 || self.disclosure_made { return; }
        if let Some((who, found, day)) = self.pending_discovery {
            // it ripens a fortnight, then comes out in a whisper
            if tick >= day + 14 {
                self.pending_discovery = None;
                self.disclosure_made = true;
                let name = self.name_of(who);
                self.log.push(format!("t{}: {} whispers it about - {}. The word's out now.", tick, name, found));
                if self.lord_esteem > 0.4 {
                    // an org worth the name stands with its own
                    self.log.push(format!("t{}: the village stands with {} - the truth was theirs to say.", tick, name));
                    self.say(who, "I said what I saw. That's all I did.", CHAT_BUBBLE_SECS * 2);
                } else {
                    // no one to stand with him: the threatened move - and
                    // the ladder's there for a man who talks treason
                    for _ in 0..3 {
                        desires::add_grievance(&mut self.world, who, desires::Problem::Blamed, tick + 90);
                    }
                    self.say(who, "I only said what I saw.", CHAT_BUBBLE_SECS * 2);
                }
            }
            return;
        }
        // what there is to find: the clerk's tallies, the old cures
        let found = if let Some(t) = self.translator {
            Some((t, "the lord's fifth is really a quarter"))
        } else if self.cured >= 3 {
            self.herbologist.map(|h| (h, "the old cures work, doctrine or no"))
        } else { None };
        if let Some((who, found)) = found {
            self.pending_discovery = Some((who, found, tick));
        }
    }

    /// People power: it builds as the org spends it and atrophies when
    /// idle - and the lord's estimate lags behind the truth, for he only
    /// learns what the street shows him
    pub fn power_tick(&mut self, tick: u64) {
        if tick % 7 != 3 { return; }
        let capacity = {
            if let Some(p) = self.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(self.org) {
                crate::sim::organized_power::step_idle(p,
                    &crate::sim::organized_power::PowerParams { atrophy_rate: 0.02, ..Default::default() });
                p.capacity
            } else { 0.0 }
        };
        // the lord's estimate creeps toward what's shown - perceived, not raw
        self.lord_esteem += (capacity - self.lord_esteem) * 0.08;
    }

    /// An org action in the street: power builds as it's spent
    pub fn record_power(&mut self, q: crate::sim::organized_power::ActionQuality) {
        let org = self.org;
        crate::sim::organized_power::record_action(&mut self.world, org, q);
    }

    /// The day's work leaves its smell: the fold, the smoke, the cut.
    /// The rain washes everyone clean.
    pub fn smell_tick(&mut self, tick: u64) {
        let _ = tick;
        let raining = self.precip.map(|p| matches!(p,
            crate::sim::precipitation::PrecipKind::Rain
            | crate::sim::precipitation::PrecipKind::HeavyRain
            | crate::sim::precipitation::PrecipKind::Showers)).unwrap_or(false);
        for &v in &self.villagers.clone() {
            if villager::is_dead(&self.world, v) { continue; }
            if raining {
                if let Some(s) = self.world.get_component_mut::<Smell>(v) { s.intensity = 0.0; }
                continue;
            }
            let kind = self.world.get_component::<Craft>(v).and_then(|c| match c.vocation {
                Vocation::Shepherd => Some("the fold"),
                Vocation::Beekeeper => Some("smoke and wax"),
                Vocation::Mason => Some("the cut"),
                Vocation::Weaver => Some("the dye"),
                Vocation::Alewife | Vocation::Brewer | Vocation::Distiller => Some("the mash"),
                Vocation::Vintner => Some("the press"),
                Vocation::Cooper => Some("the pitch"),
                _ => None,
            });
            // and the herb-wife always carries her bitter herbs
            let kind = kind.or_else(|| if Some(v) == self.herbologist { Some("bitter herbs") } else { None });
            if let Some(kind) = kind {
                if let Some(s) = self.world.get_component_mut::<Smell>(v) {
                    // perfume lingers under the work-smell but doesn't grow
                    if s.kind != "roses" {
                        s.intensity = (s.intensity + 0.15).min(1.0);
                        s.kind = kind;
                    }
                } else {
                    self.world.add_component(v, Smell { intensity: 0.15, kind });
                }
            } else if let Some(s) = self.world.get_component_mut::<Smell>(v) {
                if s.kind == "roses" { s.intensity = (s.intensity - 0.02).max(0.0); }
            }
        }
    }

    /// What they smell of, if anything worth mentioning
    pub fn smells_of(&self, v: Entity) -> Option<&'static str> {
        self.world.get_component::<Smell>(v)
            .filter(|s| s.intensity > 0.6).map(|s| s.kind)
    }

    /// The year's feast days: one mid-season each quarter, the village
    /// gathers at the market, the good stock comes out, and for a day
    /// the wants that never quite get fed are fed
    pub fn festival_tick(&mut self, tick: u64) {
        if tick % 90 != 45 { return; }
        const FEASTS: [&str; 4] = ["St. Swithin's day", "the Harvest Home", "Candlemas", "St. Martin's day"];
        let feast = FEASTS[(tick / 90 % 4) as usize];
        self.log.push(format!("t{}: {} - the village gathers at the market.", tick, feast));
        // the good stock comes out: bread, honey, whatever the stalls held
        let mut loaves = 0u32;
        for &be in &self.building_entities.clone() {
            if let Some(b) = self.world.get_component_mut::<Building>(be) {
                loaves += household::take_stock(b, "bread", 3);
                loaves += household::take_stock(b, "honey", 2);
            }
        }
        // and the barrel's tapped: whatever the village pours, the feast pours
        let mut poured = 0u32;
        for &be in &self.building_entities.clone() {
            if let Some(b) = self.world.get_component_mut::<Building>(be) {
                for d in ["ale", "beer", "wine", "spirits"] {
                    poured += household::take_stock(b, d, 2);
                }
            }
        }
        let gathered: Vec<Entity> = self.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&self.world, v)
                && self.world.get_component::<TownRole>(v).is_none()
                // the village's feast: the country keeps its own
                && self.site_of(v).is_none()
                && !self.shunned(v))
            .collect();
        // everyone comes to market for the day, in a ring round the cross
        let store = self.market_food;
        for &v in &gathered { self.travels.retain(|t| t.who != v); }
        self.gather_feast(tick, &gathered);
        for &v in &gathered {
            // the day feeds the wants that never quite get fed
            desires::satisfy(&mut self.world, v, Want::Joy, tick + 90);
            desires::satisfy(&mut self.world, v, Want::Music, tick + 90);
            if loaves > 0 {
                desires::satisfy(&mut self.world, v, Want::GoodFood, tick + 30);
                if let Some(h) = self.world.get_component_mut::<Hunger>(v) { h.value = (h.value + 25.0).min(100.0); }
            }
            if poured > 0 {
                desires::satisfy(&mut self.world, v, Want::Drink, tick + 30);
            }
        }
        // bonds grow between the co-attendees: a pair or two each
        for (i, &v) in gathered.iter().enumerate() {
            for &o in gathered.iter().skip(i + 1).take(2) {
                relationships::record_interaction(&mut self.world, v, relationships::Interaction {
                    other: o, kind: relationships::InteractionKind::Conversation, intensity: 1.5 });
            }
        }
        // a few toasts surface what folk carry - the truth comes out in drink
        for &v in gathered.iter().take(4) {
            let line = self.world.get_component::<InnerLife>(v)
                .and_then(|l| l.grievances.first().map(|(p, _)| *p))
                .map(|p| match p {
                    desires::Problem::Blamed => "To my bread - whatever they say of it!",
                    desires::Problem::Taxed | desires::Problem::DeathDuty => "To what's left after the fifth!",
                    desires::Problem::Broke => "To better days!",
                    desires::Problem::Smelly => "To the fold - it smells of honest work!",
                    _ => "To the village!",
                }).unwrap_or("To the harvest!");
            self.say(v, line, CHAT_BUBBLE_SECS * 2);
        }
        // the tax collector is never invited but always shows
        if let Some(c) = self.collector {
            if !villager::is_dead(&self.world, c) {
                self.travels.retain(|t| t.who != c);
                self.send(c, Goal::HeadHome(store));
                self.say(c, "Uninvited. As ever.", CHAT_BUBBLE_SECS * 2);
            }
        }
    }

    /// The young grow: each day the kids age, and on the day they come of
    /// age they take up a trade - the master they ran the most errands
    /// for, else whatever trade the village can use
    pub fn lifecycle_tick(&mut self, tick: u64) {
        for &v in self.villagers.clone().iter() {
            if villager::is_dead(&self.world, v) { continue; }
            let Some(g) = self.world.get_component_mut::<KidGrowth>(v) else { continue };
            g.age_days += 1;
            if g.age_days < vocation::COMING_OF_AGE { continue; }
            let errands = g.errands.clone();
            self.world.remove_component::<Kid>(v);
            self.world.remove_component::<KidGrowth>(v);
            // the hirer they ran for most is their likeliest master
            let master = errands.iter().max_by_key(|(_, n)| *n).map(|(e, _)| *e)
                .filter(|&m| !villager::is_dead(&self.world, m)
                    && self.world.get_component::<Craft>(m).is_some());
            let voc = master.and_then(|m| self.world.get_component::<Craft>(m).map(|c| c.vocation))
                .unwrap_or_else(|| roll_vocation(&mut self.rng));
            let mut c = vocation::craft(voc, 0.15);
            c.master = master;
            c.tasked_by = master;
            self.world.add_component(v, c);
            let name = self.villagers.iter().position(|x| *x == v).map(|i| vill_name(i)).unwrap_or("a youngster");
            let trade = vocation::vocation_name(voc).to_lowercase();
            self.log.push(format!("t{}: {} takes up the {}.", tick, name, trade));
            self.toasts.push((format!("{} comes of age - a {}!", name, trade), tick + 30));
        }
    }

    /// The heart-helper's rounds: she keeps her corner of the market, and
    /// now and then a lonely heart is nudged to come and sit with her
    pub fn matchmaker_tick(&mut self, tick: u64) {
        let Some(mm) = self.matchmaker else { return };
        if tick % 4 != 0 || villager::is_dead(&self.world, mm) { return; }
        // a heart behind bars can't answer the heart-helper - skip it
        // for one that can, or the lock-up wastes the day's match
        if let Some(&a) = self.lonely_hearts().iter().find(|&&a| {
            a != mm && !crate::game::dungeon::unavailable(&self.world, a)
            && self.world.get_component::<Hunger>(a).map(|h| h.value >= 30.0).unwrap_or(true)
            && !self.travels.iter().any(|t| t.who == a)
        }) { self.send(a, Goal::TalkTo(mm)); }
    }

    /// Villagers carrying an unmet Want::Love and no sweetheart yet
    pub fn lonely_hearts(&self) -> Vec<Entity> {
        self.villagers.iter().copied().filter(|&v| {
            // the grave and the lock-up take no suitors - no bonds
            // written to a cell, no heart sent courting from one
            !crate::game::dungeon::unavailable(&self.world, v)
            // the roles keep their own rounds; a heart called to market
            // would never wander home again
            && self.world.get_component::<TownRole>(v).is_none()
            // and no one courts the shunned
            && !self.shunned(v)
            // grown, not a kid: kids have no craft
            && self.world.get_component::<Craft>(v).is_some()
            && self.world.get_component::<InnerLife>(v)
                .map(|l| l.wants.contains(&desires::Want::Love)
                    && !l.satisfied.iter().any(|(w, u)| *w == desires::Want::Love && *u > self.tick))
                .unwrap_or(false)
            && !self.has_rel(relationships::RelKind::Lover, v)
        }).collect()
    }

    /// Does `v` stand in any relationship of `kind`?
    pub fn has_rel(&self, kind: relationships::RelKind, v: Entity) -> bool {
        self.world.find::<relationships::Relationship>(|_, r| {
            r.kind == kind && (r.from == v || r.other == v)
        }).is_some()
    }

    /// The relationship standing between `a` and `b`, if any
    fn rel_kind(&self, a: Entity, b: Entity) -> Option<relationships::RelKind> {
        relationships::find_relationship(&self.world, a, b)
            .and_then(|e| self.world.get_component::<relationships::Relationship>(e))
            .map(|r| r.kind)
    }

    fn name_of(&self, e: Entity) -> String {
        self.villagers.iter().position(|&x| x == e).map(|i| vill_name(i).to_string())
            .unwrap_or_else(|| "someone".into())
    }

    /// Try to kindle something between `a` and another lonely heart.
    /// `broker` pockets the fee if a match catches; `who` is who the log
    /// credits - "the heart-helper", or "you" when the player brokers it
    pub fn match_attempt(&mut self, a: Entity, tick: u64, broker: Option<Entity>, who: &str) {
        let tried = |x: Entity, y: Entity| self.match_tried.iter()
            .any(|&(p, q, until)| ((p == x && q == y) || (p == y && q == x)) && until > tick);
        let bs: Vec<Entity> = self.lonely_hearts().into_iter()
            .filter(|&b| b != a && !tried(a, b)
                && !matches!(self.rel_kind(a, b), Some(relationships::RelKind::Family | relationships::RelKind::Lover)))
            .collect();
        let Some(&b) = bs.first() else {
            self.say(a, "No heart left to mend - not today.", TALK_BUBBLE_SECS);
            return;
        };
        // the roll: shared wants draw folks together; feuded blood works against
        let shared = self.world.get_component::<InnerLife>(a).map(|la| la.wants.iter()
            .filter(|w| self.world.get_component::<InnerLife>(b).map(|lb| lb.wants.contains(w)).unwrap_or(false))
            .count()).unwrap_or(0);
        let feuded = self.rel_kind(a, b) == Some(relationships::RelKind::Rival);
        // and smell tells on a suitor: "she smells of the fold" - a dab
        // of perfume, though, is no smell at all
        let smelly = [a, b].iter().any(|&v|
            self.smells_of(v).map(|k| k != "roses").unwrap_or(false));
        if smelly { self.say(a, "...you smell of the work, friend.", CHAT_BUBBLE_SECS); }
        let spark = 0.5 + shared as f32 * 0.1 - if feuded { 0.15 } else { 0.0 }
            - if smelly { 0.15 } else { 0.0 };
        // a deterministic spark per pair and month - no rng spent, so the
        // same village always courts the same way
        let roll = ((a as u64).wrapping_mul(73856093) ^ (b as u64).wrapping_mul(19349663)
            ^ (tick / 30).wrapping_mul(83492791)) % 1000;
        if (roll as f32) / 1000.0 < spark {
            for (x, y) in [(a, b), (b, a)] {
                relationships::add_relationship(&mut self.world, relationships::Relationship {
                    from: x, other: y, kind: relationships::RelKind::Lover, strength: 0.6, interactions: 1 });
            }
            desires::satisfy(&mut self.world, a, desires::Want::Love, tick + 365);
            desires::satisfy(&mut self.world, b, desires::Want::Love, tick + 365);
            if let Some(pe) = broker {
                if let Some(p) = self.world.get_component_mut::<Purse>(pe) { p.gold += 2.0; }
            }
            let (an, bn) = (self.name_of(a), self.name_of(b));
            self.log.push(format!("t{}: {} brought {} and {} together{}.", tick, who, an, bn,
                if feuded { " - a match the families won't like" } else { "" }));
            // a brokered match is power shown in the street
            if let Some(pe) = broker {
                if pe == self.player { self.record_power(crate::sim::organized_power::ActionQuality::Transactional); }
            }
            self.say(a, "Come, I want you to meet someone.", TALK_BUBBLE_SECS);
        } else {
            self.match_tried.push((a, b, tick + 30));
            self.say(a, "Perhaps another heart.", TALK_BUBBLE_SECS);
        }
    }

    /// A fox in the yard: a hen gone in the night, and the household grieves it
    fn fox_night(&mut self, tick: u64) {
        let hens_at = |h| livestock::flock(&self.world, h).iter()
            .any(|&a| self.world.get_component::<Livestock>(a).map(|l| l.kind == LivestockKind::Hen).unwrap_or(false));
        let Some(&h) = self.hen_houses.iter().find(|&&h| hens_at(h)) else { return };
        let Some(&a) = livestock::flock(&self.world, h).first() else { return };
        self.world.destroy_entity(a);
        self.hens.retain(|&e| e != a);
        let name = self.world.get_component::<Building>(h).map(|b| b.name.clone()).unwrap_or_else(|| "the yard".into());
        self.log.push(format!("t{}: a fox got into {} - a hen gone in the night.", tick, name));
        // the house grieves her through whoever can feel it - the
        // held and the dead take no grief
        if let Some(&occ) = self.world.get_component::<Building>(h)
            .and_then(|b| b.occupants.iter().find(|&&o|
                !crate::game::dungeon::unavailable(&self.world, o))) {
            desires::add_grievance(&mut self.world, occ, desires::Problem::Loss, tick + 90);
            self.say(occ, "The fox took her. Best hen I had.", CHAT_BUBBLE_SECS);
        }
    }

    /// Send `who` to gather `good` at tile `t`
    pub fn send_gather(&mut self, who: Entity, good: &'static str, t: (i32, i32)) -> bool {
        // the held take no errands - a forage is no freer a walk than
        // the others
        if crate::game::dungeon::unavailable(&self.world, who) { return false; }
        let Some(from) = self.world.get_component::<Pos>(who).copied() else { return false };
        // gathers get the forage-errand reach: wood and stone have no
        // other source, and the craft can't wait on a near tree
        if (t.0 - from.x.round() as i32).abs() + (t.1 - from.y.round() as i32).abs() > 16 { return false; }
        match self.map.walk_path((from.x.round() as i32, from.y.round() as i32), t) {
            Some(mut path) => { path.push(t); self.travels.push(Travel { who, path, goal: Goal::Gather(good, t) }); true }
            None => false,
        }
    }

    /// One frame of every walk under way: step along the path, and do the
    /// errand when it runs out
    pub fn advance_travels(&mut self) {
        let mut arrived: Vec<(Entity, Goal)> = vec![];
        // the country's carters and stewards go by cart and horse
        let riders: Vec<Entity> = self.sites.iter().filter_map(|s| s.carrier).collect();
        for t in self.travels.iter_mut() {
            if t.path.is_empty() { continue; }
            // the dead finish no errands - the errand ends where they fell
            if villager::is_dead(&self.world, t.who) { t.path.clear(); continue; }
            // a well-worn path is quicker underfoot - trails::movement_cost
            // is the one place the worn-way discount is worked out
            let worn = self.trails.get(&t.path[0]).map(|tr| tr.strength).unwrap_or(0.0);
            // a lame leg covers less ground in a day
            let slow = self.world.get_component::<villager::Condition>(t.who)
                .map(|c| if c.kind == "a lame leg" { 0.6 } else { 1.0 }).unwrap_or(1.0);
            let ride = if riders.contains(&t.who) { country::RIDE_PACE } else { 1.0 };
            let stride = MEETING_WALK / trails::movement_cost(1.0, worn, TRAIL_K) * slow * ride;
            let Some(p) = self.world.get_component_mut::<Pos>(t.who) else { t.path.clear(); continue };
            let ot = (p.x.round() as i32, p.y.round() as i32);
            let (nx, ny) = (t.path[0].0 as f32, t.path[0].1 as f32);
            let (dx, dy) = (nx - p.x, ny - p.y);
            let d = (dx * dx + dy * dy).sqrt();
            let (stepx, stepy) = if d <= stride { (nx, ny) } else { (p.x + dx / d * stride, p.y + dy / d * stride) };
            // never step somewhere they can't stand: a wall in the way
            // aborts the errand rather than leaving them walking into it
            // (and starving on the spot) forever - an exile aborting at
            // the rocks still takes the road, else he stands banished
            // and breathing at the border until hunger does it
            if !self.map.passable(stepx.round() as i32, stepy.round() as i32) {
                if matches!(t.goal, Goal::Exile) { arrived.push((t.who, t.goal.clone())); }
                t.path.clear(); continue;
            }
            p.x = stepx; p.y = stepy;
            let nt = (p.x.round() as i32, p.y.round() as i32);
            if nt != ot {
                let tr = self.trails.entry(nt).or_insert(Trail::wild());
                trails::step_trail(tr, 1, 0.0, &TRAIL_PARAMS);
            }
            if d <= stride { t.path.remove(0); }
            if t.path.is_empty() { arrived.push((t.who, t.goal.clone())); }
        }
        self.travels.retain(|t| !t.path.is_empty());
        for (who, goal) in arrived { self.arrive(who, goal); }
    }

    /// What happens when a walk reaches its end: the trade, the
    /// collection, the gather, the delivery
    pub fn arrive(&mut self, who: Entity, goal: Goal) {
        let tick = self.tick;
        match goal {
            Goal::BuyFrom(seller, good, qty) => {
                // seller may be a person (satchel) or a building (the stall)
                let mut paid = if self.world.get_component::<Building>(seller).is_some() {
                    household::buy_at_shop(&mut self.world, seller, who, good, qty)
                } else {
                    livelihood::trade_person(&mut self.world, who, seller, good, qty)
                };
                // short of coin, a neighbour still swaps: the buyer offers
                // out of the satchel toward what the ask is worth, and a
                // few coppers to boot squares it
                if paid.is_none() && self.world.get_component::<Building>(seller).is_none() {
                    let want_worth = goods::worth_at(&self.world, good) * qty as f32;
                    let offer = livelihood::pick_offer(&self.world, who, want_worth, good);
                    if !offer.is_empty() {
                        let offer_refs: Vec<(&str, u32)> = offer.iter()
                            .map(|(g, q)| (g.as_str(), *q)).collect();
                        if let Some(deal) = livelihood::barter_person(
                            &mut self.world, who, seller, &[(good, qty)], &offer_refs) {
                            paid = Some(deal.boot);
                            let an = self.villagers.iter().position(|v| *v == who).map(vill_name);
                            let bn = self.villagers.iter().position(|v| *v == seller).map(vill_name);
                            if let (Some(a), Some(b)) = (an, bn) {
                                self.log.push(format!("t{}: {} swaps {} with {}.", tick, a,
                                    livelihood::barter_terms(&offer, &[(good.into(), qty)], deal.boot), b));
                            }
                            // the deal lands on the bond: fair swaps warm a
                            // little, a short sack leaves a mark, generosity
                            // reads as aid
                            use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
                            let (kind, who_records) = if deal.fairness < 0.85 {
                                (InteractionKind::Conflict, seller)
                            } else if deal.fairness > 1.15 {
                                (InteractionKind::Aid, who)
                            } else {
                                (InteractionKind::Conversation, who)
                            };
                            record_interaction(&mut self.world, who_records,
                                Interaction { other: if who_records == who { seller } else { who },
                                    kind, intensity: (1.0 - deal.fairness).abs().clamp(0.2, 1.0) });
                        }
                    }
                }
                // trade with the road's accounts loses a little to the
                // reckoning - unless the village has a clerk for it
                if Some(seller) == self.trader_stall && vocation::imported(good)
                    && self.numeracy != Numeracy::Positional && self.translator.is_none() {
                    self.reckonings += 1;
                    // the tally leans on what was actually paid - a shelf
                    // emptied just now doesn't swell the reckoning's cut
                    let skim = paid.unwrap_or(0.0) * 0.1;
                    if let Some(p) = self.world.get_component_mut::<Purse>(who) {
                        p.gold = (p.gold - skim).max(0.0);
                    }
                    if self.reckonings % 3 == 0 {
                        desires::add_grievance(&mut self.world, who, desires::Problem::ShortChanged, tick + 40);
                        if self.reckonings == 3 {
                            self.log.push(format!("t{}: the reckoning didn't add up - he counted it his way.", tick));
                        }
                    }
                    // enough of these and a clerk steps up to keep the
                    // tallies straight - the dragoman problem solved
                    if self.reckonings >= 8 {
                        let clerk = self.villagers.iter().copied().find(|&v|
                            self.world.get_component::<Craft>(v)
                                .map(|c| c.vocation == Vocation::Printmaker || c.vocation == Vocation::Priest)
                                .unwrap_or(false)
                            && !crate::game::dungeon::unavailable(&self.world, v));
                        if let Some(c) = clerk {
                            self.translator = Some(c);
                            let name = self.name_of(c);
                            self.log.push(format!("t{}: {} keeps the tallies straight now - no more foreign reckonings.", tick, name));
                        }
                    }
                }
                // a dab of perfume: the buyer smells of roses for days
                if good == "perfume" {
                    if let Some(s) = self.world.get_component_mut::<Smell>(who) {
                        s.kind = "roses"; s.intensity = 0.8;
                    } else {
                        self.world.add_component(who, Smell { kind: "roses", intensity: 0.8 });
                    }
                }
            }
            Goal::BuyAt(shop, good, want) => {
                if household::buy_at_shop(&mut self.world, shop, who, good, 1).is_some() {
                    if let Some(w) = want {
                        desires::satisfy(&mut self.world, who, w, tick + 60);
                        // consumable goods are enjoyed, not kept
                        if matches!(good, "bread" | "faith" | "security" | "pamphlets" | "garments"
                            | "lodging") {
                            if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                                if let Some(q) = s.goods.get_mut(good) { *q = q.saturating_sub(1); }
                            }
                        }
                    }
                    if vocation::is_drink(good) {
                        // a cup goes down at the counter, not home in the
                        // bag: company, a small lift - and sometimes the
                        // morning after
                        desires::satisfy(&mut self.world, who, Want::Drink, tick + 60);
                        desires::satisfy(&mut self.world, who, Want::Joy, tick + 15);
                        if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                            if let Some(q) = s.goods.get_mut(good) { *q = q.saturating_sub(1); }
                        }
                        // the keeper hears everyone's business - a cup
                        // buys a chat, and the alehouse earns its name
                        if let Some(k) = self.world.get_component::<Building>(shop)
                            .and_then(|b| b.worker).filter(|&k| k != who) {
                            relationships::record_interaction(&mut self.world, who, relationships::Interaction {
                                other: k, kind: relationships::InteractionKind::Conversation,
                                intensity: CHAT_INTENSITY,
                            });
                        }
                        if self.rng.below(100) < 15 {
                            self.world.add_component(who, villager::Drunk { until: tick + 2 });
                            self.say(who, "One more won't hurt.", CHAT_BUBBLE_SECS);
                        }
                    }
                }
            }
            Goal::Gather(good, tile) => {
                // the land keeps score: the canopy thins, the pit fills
                let qty = match good {
                    "wood" => {
                        let cover = self.land.tree_cover.get(&tile).copied().unwrap_or(0.6);
                        if let Some(c) = self.land.tree_cover.get_mut(&tile) { *c = (*c - 0.12).max(0.0); }
                        crate::sim::environment::wood_yield(cover)
                    }
                    "stone" => {
                        let y = crate::sim::environment::stone_yield(self.land.pit_water);
                        self.land.pit_water = (self.land.pit_water + 0.03).min(1.0);
                        if self.land.pit_water >= 1.0 && self.pit == Some(tile) {
                            self.log.push(format!("t{}: the pit's drowned - no stone comes out of a pond.", tick));
                        }
                        y
                    }
                    _ => 2,
                };
                if qty > 0 {
                    if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                        *s.goods.entry(good.to_string()).or_insert(0) += qty;
                    }
                }
            }
            Goal::CollectTax(target) => {
                let Some(c) = self.collector else { return };
                if villager::is_dead(&self.world, target) { return; }
                // the lord's reply turns on his estimate of the org:
                // a stranger he squeezes, a power he negotiates, a force
                // he passes by
                if target == self.player && self.lord_esteem > 0.6 {
                    self.log.push(format!("t{}: the collector passes you by - the lord knows an organized man when he sees one.", tick));
                    return;
                }
                let cut = livelihood::collect_tax(&mut self.world, c, target, tick);
                // the levy rides the books as the lord's, not the man's
                self.takings += cut;
                let taken = cut
                    * if target == self.player && self.lord_esteem > 0.3 {
                        self.log.push(format!("t{}: the lord learns to tread lightly - the cut is halved.", tick));
                        0.5
                    } else { 1.0 };
                // the assize's share on the barrel: a shop stocked with
                // drink pays a cut on what it pours - the alehouse, the
                // brewery and the still are the lord's best takings
                let excise = self.world.get_component::<Craft>(target)
                    .and_then(|cr| cr.workplace)
                    .and_then(|b| self.world.get_component::<Building>(b))
                    .map(|b| b.stock.iter()
                        .filter(|(g, _)| vocation::is_drink(g))
                        .map(|(g, q)| goods::worth_at(&self.world, g) * *q as f32).sum::<f32>())
                    .map(|v| (v * livelihood::DRINK_EXCISE).round())
                    .unwrap_or(0.0);
                if excise > 0.0 {
                    // he takes what the purse can bear, no more - the
                    // lord can't coin what isn't there
                    let paid = self.world.get_component::<Purse>(target)
                        .map(|p| p.gold.min(excise)).unwrap_or(0.0);
                    if let Some(p) = self.world.get_component_mut::<Purse>(target) { p.gold -= paid; }
                    if let Some(p) = self.world.get_component_mut::<Purse>(c) { p.gold += paid; }
                    self.takings += paid;
                    let name = self.villagers.iter().position(|v| *v == target)
                        .map(|i| vill_name(i)).unwrap_or("you");
                    self.log.push(format!("t{}: the collector counts {}'s barrels - the ale excise comes to {:.0} gold.", tick, name, excise));
                }
                // a grant nobody kept watch over is clawed back: if the
                // campaign that won it has lapsed, the collector takes the
                // full cut though the grant still stands on paper
                let seat_orgs: Vec<Entity> = self.seats.iter().map(|s| s.org).collect();
                for o in seat_orgs {
                    if let Some(camp) = crate::game::campaign::campaign_of(&self.world, o) {
                        if crate::game::reactions::hollow_if_unguarded(&mut self.world, camp, tick) {
                            self.log.push(format!("t{}: the lord's grant still stands on paper - but nobody kept watch, and the collector takes the full cut. Meet again and put your demands to the Reeve to win it back.", tick));
                        }
                    }
                }
                // a grant carried home lightens the round for the cause's
                // own - wherever the grant was won, not just seat zero's
                // fight: half the cut back and the taxing doesn't count;
                // a partial grant gives a quarter back and the grievance
                // stands; a hollowed one gives nothing
                let relief = self.seats.iter()
                    .filter(|s| self.world.get_component::<Organization>(s.org)
                        .map(|o| o.members.contains(&target)).unwrap_or(false))
                    .filter_map(|s| self.world.get_component::<crate::game::reactions::LordGrant>(s.org))
                    .filter(|g| g.issue == "the taxes")
                    .map(|g| g.relief(tick))
                    .fold(0.0f32, f32::max);
                let granted = relief > 0.0;
                if granted {
                    let back = taken * relief;
                    if let Some(p) = self.world.get_component_mut::<Purse>(target) { p.gold += back; }
                    if relief >= crate::game::reactions::FULL_RELIEF {
                        crate::game::desires::lift_grievance(&mut self.world, target, crate::game::desires::Problem::Taxed);
                    }
                }
                self.say(target, "The lord's share, if you please.", CHAT_BUBBLE_SECS);
                if target == self.player {
                    self.log.push(format!("t{}: the tax collector takes his cut from you ({:.0} gold).", tick, taken));
                }
                if granted {
                    let name = self.villagers.iter().position(|v| *v == target).map(|i| vill_name(i)).unwrap_or("them");
                    if relief >= crate::game::reactions::FULL_RELIEF {
                        self.log.push(format!("t{}: the lord's grant covers {} - half the cut comes back.", tick, name));
                    } else {
                        self.log.push(format!("t{}: the lord's part-grant covers {} - a quarter of the cut comes back.", tick, name));
                    }
                }
            }
            Goal::TalkTo(e) => {
                // whoever they meant to greet is gone, behind bars, or
                // down the road and dusted - no conversation at any of them
                if crate::game::dungeon::unavailable(&self.world, e)
                    || self.world.get_component::<Pos>(e).is_none() {
                    self.say(who, "Oh.", TALK_BUBBLE_SECS);
                } else if Some(e) == self.matchmaker && who != e {
                    // a lonely heart comes to sit with the heart-helper
                    self.match_attempt(who, tick, Some(e), "the heart-helper");
                } else if self.ink_boiler.as_ref().map(|b| b.e) == Some(e) && who != e {
                    // crusts for the pot, and a fried taste for carrying them
                    desires::satisfy(&mut self.world, who, Want::GoodFood, tick + 30);
                    self.say(who, "The crusts come back fried.", CHAT_BUBBLE_SECS);
                } else {
                    // stand and chat: a conversation, a step toward a tie
                    relationships::record_interaction(&mut self.world, who, relationships::Interaction {
                        other: e, kind: relationships::InteractionKind::Conversation, intensity: CHAT_INTENSITY,
                    });
                    self.say(who, "Good day to you.", TALK_BUBBLE_SECS);
                }
            }
            Goal::Wander => {}
            Goal::Eat => {
                // a real meal off the nearest table - if the day's hunger
                // system already fed them mid-errand, no second helping
                let still_hungry = self.world.get_component::<Hunger>(who)
                    .map(|h| h.value < 100.0).unwrap_or(false);
                let table = self.world.get_component::<Pos>(who).copied()
                    .map(|p| self.world.query::<villager::FoodSource>().iter().copied()
                        .filter(|&f| self.world.get_component::<villager::FoodSource>(f)
                            .map(|fs| fs.servings > 0).unwrap_or(false))
                        .filter(|&f| self.world.get_component::<Pos>(f)
                            .map(|fp| (fp.x - p.x).abs() + (fp.y - p.y).abs() < 3.0).unwrap_or(false))
                        .next());
                if still_hungry {
                    if let Some(Some(t)) = table {
                        let served = self.world.get_component_mut::<villager::FoodSource>(t)
                            .map(|f| { f.servings -= 1; true }).unwrap_or(false);
                        if served {
                            if let Some(h) = self.world.get_component_mut::<Hunger>(who) { h.value = 100.0; }
                        }
                    }
                }
            }
            Goal::Herd(_) => {}
            Goal::HeadHome(_) => {}
            Goal::Cart(si) => self.arrive_cart(who, si),
            Goal::Levy(castle, town) => self.arrive_levy(who, castle, town),
            Goal::Deliver(castle) => self.arrive_deliver(who, castle),
            Goal::TendGarden(gi) => {
                // weeding and picking: the bed yields into the house's
                // stock, and the tending itself is a small joy
                if let Some((home, _, g, kitchen)) = self.gardens.get_mut(gi) {
                    if *g >= 1.0 {
                        *g = 0.0;
                        // a kitchen bed comes up roots, potherbs and peas
                        // in turn; a herb bed keeps its flowers
                        let good = if *kitchen {
                            ["roots", "potherbs", "peas"][(tick / 7 % 3) as usize]
                        } else if tick % 2 == 0 { "marigold" } else { "feverfew" };
                        if let Some(b) = self.world.get_component_mut::<Building>(*home) {
                            household::add_stock(b, good, 1);
                        }
                        let home = *home;
                        if let Some(b) = self.world.get_component::<Building>(home) {
                            for &o in &b.occupants.clone() {
                                desires::satisfy(&mut self.world, o, Want::Joy, tick + 20);
                            }
                        }
                    }
                }
            }
            Goal::Exile => {
                // at the world's edge they don't come back - unless the
                // castle's keeping them: the dungeon's a held state, and
                // the org may yet ransom them home
                let name = self.villagers.iter().position(|v| *v == who).map(|i| vill_name(i)).unwrap_or("someone");
                if self.dungeon.iter().any(|(e, _)| *e == who) {
                    self.log.push(format!("t{}: {} is held at the castle - the org may yet ransom them.", tick, name));
                } else {
                    self.world.add_component(who, villager::Dead);
                    self.log.push(format!("t{}: {} is driven out - the road takes them.", tick, name));
                }
            }
            Goal::ForageShroom(pi) => {
                self.shroom_breaks.push((who, tick));
                if let Some(p) = self.mush_patches.get_mut(pi) { p.next_ready = tick + 60; }
                let Some(kind) = self.mush_patches.get(pi).map(|p| p.kind) else { return };
                self.shroom_take(who, pi, kind, tick);
            }
            Goal::Forage(pi) => {
                // picked clean for a season; two bundles dry on the hut's beams
                if let Some(p) = self.herb_patches.get_mut(pi) { p.next_ready = tick + 90; }
                let herb = self.herb_patches.get(pi).map(|p| p.herb);
                if let (Some(herb), Some(hut)) = (herb, self.herb_hut) {
                    if let Some(b) = self.world.get_component_mut::<Building>(hut) {
                        household::add_stock(b, herb, 2);
                    }
                }
            }
            Goal::SeekRemedy(_) => {
                if villager::is_dead(&self.world, who) { return; }
                self.remedy_visits.push((who, tick));
                let Some(her) = self.herbologist else { return };
                if villager::is_dead(&self.world, her) { return; }
                let illness = disease_of(&self.world, who).map(|(n, _)| n);
                let hut = self.herb_hut.unwrap_or(0);
                let efficacy = herbology::best_remedy(&mut self.world, her)
                    .or_else(|| herbology::brew_from_stock(&mut self.world, her, hut));
                match efficacy {
                    Some(eff) => {
                        herbology::apply_efficacy(&mut self.world, who, eff);
                        let fee = (2.0 + eff * 8.0).min(8.0)
                            .min(self.world.get_component::<Purse>(who).map(|p| p.gold).unwrap_or(0.0));
                        if let Some(p) = self.world.get_component_mut::<Purse>(who) { p.gold -= fee; }
                        if let Some(p) = self.world.get_component_mut::<Purse>(her) { p.gold += fee; }
                        let patient = self.villagers.iter().position(|v| *v == who).map(|i| vill_name(i)).unwrap_or("someone");
                        self.say(her, "Steep it and rest - it'll turn.", CHAT_BUBBLE_SECS);
                        self.cured += 1;
                        self.log.push(format!("t{}: {} treats {}'s {}.", tick, vill_name(HERB_IDX), patient, illness.unwrap_or_else(|| "illness".into())));
                    }
                    None => { self.say(her, "Nothing in the hedge for that today.", CHAT_BUBBLE_SECS); }
                }
            }
            Goal::RunOddJob => {
                let Some(job) = self.world.get_component_mut::<OddJob>(who)
                    .map(|j| OddJob { hirer: j.hirer, good: j.good.clone(), qty: j.qty, from: j.from, to: j.to, pay: j.pay, loaded: j.loaded })
                else { return };
                if !job.loaded {
                    // at the source: the hirer already paid for it - goods
                    // into the kid's satchel, then walk them to the hirer
                    let took = if let Some(b) = self.world.get_component_mut::<Building>(job.from) {
                        household::take_stock(b, &job.good, job.qty)
                    } else {
                        self.world.get_component_mut::<Satchel>(job.from)
                            .and_then(|s| s.goods.get_mut(&job.good).map(|q| { let n = (*q).min(job.qty); *q -= n; n }))
                            .unwrap_or(0)
                    };
                    if took == 0 { self.world.remove_component::<OddJob>(who); return; }
                    if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                        *s.goods.entry(job.good.clone()).or_insert(0) += took;
                    }
                    if let Some(j) = self.world.get_component_mut::<OddJob>(who) { j.loaded = true; }
                    // walk the second leg to the hirer
                    if let (Some(from_p), Some(to_p)) = (
                        self.world.get_component::<Pos>(who).copied(),
                        self.world.get_component::<Pos>(job.to).copied()) {
                        let to = (to_p.x.round() as i32, to_p.y.round() as i32);
                        if let Some(mut path) = self.map.walk_path((from_p.x.round() as i32, from_p.y.round() as i32), to) {
                            path.push(to);
                            self.travels.push(Travel { who, path, goal: Goal::RunOddJob });
                        }
                    }
                } else {
                    // delivered: goods to the hirer, pay to the kid. A hirer
                    // who died waiting gets no satchel and pays no coin - the
                    // goods land in the place's stock if there is one
                    let qty = self.world.get_component_mut::<Satchel>(who)
                        .and_then(|s| s.goods.get_mut(&job.good).map(|q| { let n = (*q).min(job.qty); *q -= n; n }))
                        .unwrap_or(0);
                    if villager::is_dead(&self.world, job.hirer) {
                        if let Some(b) = self.world.get_component_mut::<Building>(job.to) {
                            *b.stock.entry(job.good.clone()).or_insert(0) += qty;
                        } else {
                            // no place to leave them: the kid keeps what they carried
                            if let Some(s) = self.world.get_component_mut::<Satchel>(who) {
                                *s.goods.entry(job.good.clone()).or_insert(0) += qty;
                            }
                        }
                        self.say(who, "...he's gone.", CHAT_BUBBLE_SECS);
                        self.world.remove_component::<OddJob>(who);
                        return;
                    }
                    if let Some(s) = self.world.get_component_mut::<Satchel>(job.hirer) {
                        *s.goods.entry(job.good).or_insert(0) += qty;
                    }
                    if let Some(h) = self.world.get_component_mut::<Purse>(job.hirer) { h.gold -= job.pay.min(h.gold); }
                    if let Some(k) = self.world.get_component_mut::<Purse>(who) { k.gold += job.pay; }
                    // an errand run is a bond: the hirer they run for
                    // most is their likeliest master
                    if let Some(g) = self.world.get_component_mut::<KidGrowth>(who) {
                        if let Some((_, n)) = g.errands.iter_mut().find(|(e, _)| *e == job.hirer) { *n += 1; }
                        else { g.errands.push((job.hirer, 1)); }
                    }
                    self.say(who, "All delivered!", CHAT_BUBBLE_SECS);
                    self.world.remove_component::<OddJob>(who);
                }
            }
        }
    }

    /// The town's business each day: work, errands, the tax round, the
    /// trader's restock, the odd minstrel, a tune in the street
    pub fn livelihood_tick(&mut self, tick: u64) {
        // the felt worths slide a little back toward ordinary each day
        goods::drift(&mut self.world);
        // market day, once a month: the org's members sell their work and
        // the purse sees its share of the takings
        if tick % 30 == 20 {
            for seat in 0..self.seats.len() {
                let earned = crate::game::player::market_day(&mut self.world,
                    self.seats[seat].player, self.seats[seat].org);
                if earned > 0.0 {
                    self.log.push(format!("t{}: market day - the org sells its members' work; {:.0} gold to the purse.", tick, earned));
                }
            }
        }
        // the trader's stall restocks an exotic every couple of days
        if tick % livelihood::TRADER_RESTOCK_DAYS == 0 {
            if let Some(s) = self.trader_stall { livelihood::restock_trader(&mut self.world, s, tick); }
        }
        // workplaces nobody works stand dimmed - the derelicts of the street
        self.derelict.clear();
        for (i, e) in self.building_entities.iter().enumerate() {
            if self.world.get_component::<Building>(*e).map(household::derelict).unwrap_or(false) {
                self.derelict.insert(i);
            }
        }
        // a minstrel rides in every so often
        if self.minstrel.is_none() && tick >= self.next_minstrel { self.spawn_minstrel(); }
        // and the pedlar comes by every few weeks - the game's newspaper
        if self.pedlar.is_none() && tick >= self.next_pedlar { self.spawn_pedlar(); }
        // and the village grows the way it shrinks - one at a time: a
        // child in a full house, a stranger at the gate
        if tick > 0 && tick % 40 == 0 { self.maybe_birth(tick); }
        if tick > 0 && tick % 60 == 30 { self.maybe_migrant(tick); }
        // and a circus camps outside the walls every year or so
        if self.circus.is_none() && tick >= self.next_circus { self.spawn_circus(); }
        // and the ink-maker boils his oil where the law wants it: outside
        if self.ink_boiler.is_none() && tick >= self.next_ink_boiler { self.spawn_ink_boiler(); }
        // the collector walks his round
        if let Some(c) = self.collector {
            if !self.travels.iter().any(|t| t.who == c) {
                if let Some(target) = livelihood::tax_due(&self.world, c, tick) {
                    self.send(c, Goal::CollectTax(target));
                }
            }
            // and rides out with the takings: the lord's cut is the
            // lord's - his books hand it over, whatever his own purse
            // holds, and what's his stays his
            if tick % 60 == 45 && self.takings > 0.0 {
                let remit = self.takings;
                if let Some(p) = self.world.get_component_mut::<Purse>(c) { p.gold -= remit.min(p.gold); }
                self.takings = 0.0;
                self.log.push(format!("t{}: the collector rides to the castle - {:.0} gold of the lord's cut goes over the wall.", tick, remit));
            }
        }
        // the herb-wife keeps her shelf brewed and her patches picked
        crate::timed!("sim.herbologist", self.herbologist_tick(tick));
        // the fold grazes, yields, and now and then howls
        crate::timed!("sim.livestock", self.livestock_tick(tick));
        // the heart-helper walks a lonely heart toward someone
        crate::timed!("sim.matchmaker", self.matchmaker_tick(tick));
        // a poor soul gathers in the wood, and damp years bring the fire
        crate::timed!("sim.mushroom", self.mushroom_tick(tick));
        // the skeps yield, the church burns wax, a swarm wanders off
        crate::timed!("sim.apiary", self.apiary_tick(tick));
        // the larder keeps its own score: spoilage, vermin, the cat's job
        crate::timed!("sim.spoilage", self.spoilage_tick(tick));
        // the dead hand on: benches, purses, and the lord's fifth
        crate::timed!("sim.succession", self.succession_tick(tick));
        // and the young grow into their trades
        crate::timed!("sim.lifecycle", self.lifecycle_tick(tick));
        // the worn paths fade toward green where no one walks
        crate::timed!("sim.trail", self.trail_tick());
        // the land keeps score: canopy, pit water, the reach
        crate::timed!("sim.land", self.land_tick(tick));
        // the ladder of shame climbs one rung at a time
        crate::timed!("sim.sanction", self.sanction_tick(tick));
        // and the year has its feast days
        crate::timed!("sim.festival", self.festival_tick(tick));
        // winter hearths burn a bundle every few days - faggots, turf,
        // the dry leavings, charcoal off the road; a house with nothing
        // to burn eats cold, and that's a trouble the village can hear
        if tick % 360 >= 270 && tick % 3 == 0 {
            for &h in &self.building_entities.clone() {
                let Some(b) = self.world.get_component::<Building>(h) else { continue };
                if b.kind != household::BuildingKind::Home { continue; }
                let occupants = b.occupants.clone();
                let fuel = crate::game::goods::slot_goods(crate::game::goods::Tag::Fuel)
                    .into_iter().find(|&g| b.stock.get(g).copied().unwrap_or(0) > 0);
                // else the house burns what its folk carry home - a
                // bundle in the satchel keeps the hearth same as a stack
                let hp = self.world.get_component::<Pos>(h).copied();
                let carried = if fuel.is_none() {
                    crate::game::goods::slot_goods(crate::game::goods::Tag::Fuel).into_iter()
                        .find_map(|g| occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                            // and actually home - the bundle has to be
                            // under the roof to warm it
                            && hp.and_then(|hp| self.world.get_component::<Pos>(o)
                                .map(|op| (op.x - hp.x).abs() + (op.y - hp.y).abs() <= 7.0)).unwrap_or(false)
                            && self.world.get_component::<Satchel>(o)
                                .map(|s| s.goods.get(g).copied().unwrap_or(0) > 0).unwrap_or(false))
                            .map(|o| (o, g)))
                } else { None };
                match fuel.or(carried.map(|(_, g)| g)) {
                    Some(g) => {
                        if let Some((o, _)) = carried {
                            if let Some(s) = self.world.get_component_mut::<Satchel>(o) {
                                *s.goods.get_mut(g).unwrap() -= 1;
                            }
                        } else if let Some(b) = self.world.get_component_mut::<Building>(h) {
                            household::take_stock(b, g, 1);
                        }
                    }
                    None => {
                        for &o in &occupants {
                            if !villager::is_dead(&self.world, o) {
                                desires::add_grievance(&mut self.world, o,
                                    desires::Problem::ColdHearth, tick + 9);
                            }
                        }
                        // and a cold house does something about it: a free
                        // hand goes out for a bundle tomorrow - never a
                        // villager a seat steers, that errand's theirs to give
                        if let Some(hand) = occupants.iter().copied().find(|&o|
                            !villager::is_dead(&self.world, o)
                            && !crate::game::dungeon::unavailable(&self.world, o)
                            && !self.travels.iter().any(|t| t.who == o)
                            && !self.seats.iter().any(|s| s.player == o)) {
                            if let Some(op) = self.world.get_component::<Pos>(hand).copied() {
                                if let Some(t) = self.gather_site("faggots", op) {
                                    self.send_gather(hand, "faggots", t);
                                }
                            }
                        }
                    }
                }
            }
        }
        // the gardens fill in a little more each week
        crate::timed!("sim.garden", self.garden_tick(tick));
        // and the kitchen pot: staples off the garden beds feed the
        // hungry at home, and a week's peas and roots become pottage
        crate::timed!("sim.kitchen", self.kitchen_tick(tick));
        // and the circus, while it camps, works its pull
        crate::timed!("sim.circus", self.circus_tick(tick));
        // the oil boils, the crusts fry, and the black goes in
        crate::timed!("sim.ink_boiler", self.ink_boiler_tick(tick));
        // and the hundred small fortunes: what's due comes on
        crate::timed!("sim.events", self.events_tick(tick));
        // and whatever the day caught, the fire runs on
        crate::timed!("sim.fire", self.fire_tick(tick));
        // milk becomes wheels in the dark, and a noble buys the old ones
        crate::timed!("sim.cheese", self.cheese_tick(tick));
        // ale sours, wine turns, and the still owes the wise woman
        crate::timed!("sim.drink", self.drink_tick(tick));
        // the keeper lays in a barrel, and the bench fills
        crate::timed!("sim.tavern", self.tavern_tick(tick));
        // the day's work leaves its smell - unless the rain washes it off
        crate::timed!("sim.smell", self.smell_tick(tick));
        // power atrophies; the lord's estimate creeps toward the truth
        crate::timed!("sim.power", self.power_tick(tick));
        // and now and then something rides over the walls
        crate::timed!("sim.sky", self.sky_tick(tick));
        // and the soundscape's cues rise with the days
        crate::timed!("sim.sfx", self.sfx_tick(tick));
        // and a truth sits unsaid somewhere, ripening
        crate::timed!("sim.discovery", self.discovery_tick(tick));
        // and the castle sends its writs on a season's turn
        crate::timed!("sim.edict", self.edict_tick(tick));
        // and every other winter runs long and tests the village
        crate::timed!("sim.winter", self.winter_tick(tick));
        // and the painter lays color on the village's halls
        crate::timed!("sim.painter", self.painter_tick(tick));
        // and now and then a maker makes a thing to outlast them
        crate::timed!("sim.heirloom", self.heirloom_tick(tick));
        // and the literate put word to those they think of
        crate::timed!("sim.letter", self.letter_tick(tick));
        // and theirs check on the ones who need checking on
        crate::timed!("sim.care", self.care_tick(tick));
        // and the river gives up its catch, season on season
        crate::timed!("sim.fisher", self.fisher_tick(tick));
        // and if the player's gone, the work goes on in another pair of hands
        crate::timed!("sim.continuation", self.continuation_tick(tick));
        // and the martyr's grave keeps its vigil
        crate::timed!("sim.martyr", self.martyr_tick(tick));
        // and the castle's prisoners wait on the org's answer
        crate::timed!("sim.dungeon", self.dungeon_tick(tick));
        // and the posted knights stand only their tour
        crate::timed!("sim.guards", self.guards_tick(tick));
        // and the marks a cause leaves fade on their own clock
        crate::timed!("sim.marks", self.marks_tick(tick));
        let mut rng = Rng(self.rng.next());
        // the country's carts and stewards, and its granaries
        self.country_tick(tick);
        // the holy day calls the faithful before anyone's planned the rest
        self.routine_dawn(tick, &mut rng);
        let villagers = self.villagers.clone();
        for i in 0..villagers.len() {
            let v = villagers[i];
            if crate::game::dungeon::unavailable(&self.world, v) { continue; }
            // hunger comes first: a starving person abandons any errand
            // to look for food - even one already under way. And looking
            // means walking: the store has the day's meal - except the
            // exile's road, which no belly's free to abandon
            let hungry = self.world.get_component::<Hunger>(v).map(|h| h.value < 30.0).unwrap_or(false);
            if hungry {
                self.travels.retain(|t| t.who != v || matches!(t.goal, Goal::Exile));
                // an exile walks on, belly and all - the road's the
                // only errand his sentence leaves him
                if !self.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::Exile)) {
                    let at_store = self.world.get_component::<Pos>(self.store_of(v))
                        .zip(self.world.get_component::<Pos>(v))
                        .map(|(s, p)| (s.x - p.x).abs() + (s.y - p.y).abs() <= 1.0)
                        .unwrap_or(false);
                    if at_store { self.arrive(v, Goal::Eat); }
                    else if !self.send(v, Goal::Eat) && self.site_of(v).is_some() {
                        // out on the country's roads: the nearest granary
                        // with food in it, however far
                        let sp = self.world.get_component::<Pos>(self.store_of(v)).copied();
                        if let Some(d) = sp.and_then(|p| self.at_the_door(p.x.round() as i32, p.y.round() as i32)) {
                            self.send_far(v, d, Goal::Eat);
                        }
                    }
                }
                continue;
            }
            if self.travels.iter().any(|t| t.who == v) { continue; }
            // a carter at the market, a steward on the road: the country's
            // business has them
            if self.is_away(v) { continue; }
            // today's rite or feast already has them
            let gathered = self.world.get_component::<routine::Routine>(v)
                .map(|r| r.day == tick && matches!(r.mode, routine::Mode::Worshipping | routine::Mode::Feasting))
                .unwrap_or(false);
            if gathered { continue; }
            if self.world.get_component::<Kid>(v).is_some() {
                // a child off the errand list plays
                if self.world.get_component::<OddJob>(v).is_none() { self.plan_play(v, tick, &mut rng); }
                continue;
            }
            if self.world.get_component::<TownRole>(v).is_some() { continue; }
            // a seat's own villager answers only its commands - never a
            // routine walk sent behind its back
            if self.seats.iter().any(|s| s.player == v) { continue; }
            // last night's cup still has them - they sleep it off
            if self.world.get_component::<villager::Drunk>(v)
                .map(|d| tick < d.until).unwrap_or(false) { self.plan_rest(v, tick); continue; }
            // at a meeting they're not off working
            if self.meeting.as_ref().map(|m| m.invited.contains(&v)).unwrap_or(false) { continue; }
            // the sick call on the herb-wife: out to her hut for a remedy,
            // a call at most every fortnight
            if is_sick(&self.world, v) {
                if let (Some(hut), Some(her)) = (self.herb_hut, self.herbologist) {
                    let seen = self.remedy_visits.iter().find(|(e, _)| *e == v).map(|(_, d)| *d).unwrap_or(0);
                    if !villager::is_dead(&self.world, her) && tick >= seen + 15 && self.send(v, Goal::SeekRemedy(hut)) { continue; }
                }
            }
            // a shopping day, now and then: something they want is for sale
            if rng.below(100) < 12 {
                if let Some(g) = self.want_shopping(v, tick) {
                    let good = match &g { Goal::BuyAt(_, good, _) => *good, _ => "something" };
                    if self.send(v, g) { self.plan_errand(v, tick, routine::Spot::Errand(good)); continue; }
                }
            }
            // the day's mode: work, a stall on market day, a stroll, a day in
            match self.plan_day(v, tick, &mut rng) {
                routine::DayPlan::Work => {}
                routine::DayPlan::Sell | routine::DayPlan::Free => continue,
            }
            // out of raw goods: send a kid, or go get it, or sit with the problem
            if let Some(input) = livelihood::needs_input(&self.world, v) {
                if self.errand_for_input(v, input, i) {
                    // a kid may have taken the errand: then they wait at the bench
                    if self.travels.iter().any(|t| t.who == v) {
                        self.plan_errand(v, tick, routine::Spot::Supplies(input));
                    } else { self.plan_bench(v, tick); }
                } else {
                    desires::add_grievance(&mut self.world, v, Problem::NoInput(input), tick + 3);
                    self.plan_bench(v, tick);
                }
                continue;
            }
            // a day at the craft, at the bench
            self.plan_bench(v, tick);
            let (made, up) = livelihood::work_day(&mut self.world, v, tick);
            let _ = made;
            if let Some(level) = up {
                let voc = self.world.get_component::<Craft>(v).map(|c| vocation::vocation_name(c.vocation)).unwrap_or("trade");
                self.toasts.push((format!("{} is now a {}!", vill_name(i), vocation::level_name(level)), tick + 30));
                self.log.push(format!("t{}: {} is now a {} {}.", tick, vill_name(i), vocation::level_name(level).to_lowercase(), voc));
            }
        }
        // a tune in the street: anyone with a lute plays, and it lifts the
        // want for music and for joy around them
        for (i, &v) in villagers.iter().enumerate() {
            if villager::is_dead(&self.world, v) { continue; }
            let has_lute = self.world.get_component::<Satchel>(v)
                .map(|s| s.goods.get("lute").copied().unwrap_or(0) > 0).unwrap_or(false);
            if has_lute && rng.below(8) == 0 {
                self.say(v, "\u{266a}", CHAT_BUBBLE_SECS);
                if let Some(p) = self.world.get_component::<Pos>(v).copied() {
                    let pe = self.player;
                   for &l in self.villagers.iter().chain(std::iter::once(&pe)) {
                        let near = self.world.get_component::<Pos>(l)
                            .map(|lp| (lp.x - p.x).abs() + (lp.y - p.y).abs() < 3.0).unwrap_or(false);
                        if near {
                            desires::satisfy(&mut self.world, l, Want::Music, tick + 60);
                            desires::satisfy(&mut self.world, l, Want::Joy, tick + 30);
                        }
                    }
                }
                let _ = i;
            }
        }
        // every seat's purse mirrors the day's trading, not just seat 0's;
        // agent seats have no PlayerCharacter and sync_purse no-ops on them
        for s in 0..self.seats.len() {
            let pe = self.seats[s].player; sync_purse(&mut self.world, pe, false);
        }
    }

    /// What this person might walk out to buy today: a good that feeds an
    /// unsatisfied want, stocked by some working shop
    pub fn want_shopping(&mut self, v: Entity, tick: u64) -> Option<Goal> {
        const WANT_GOODS: [(Want, &str); 11] = [
            (Want::GoodFood, "bread"), (Want::Finery, "garments"), (Want::Faith, "faith"),
            (Want::Security, "security"), (Want::Instrument, "lute"), (Want::BetterTools, "tools"),
            // the everyday pour first - the dearer the drink, the fewer
            // the purses that reach it
            (Want::Drink, "ale"), (Want::Drink, "beer"), (Want::Drink, "wine"),
            (Want::Drink, "spirits"),
            // a bed at the tavern is no roof of one's own, but it's a
            // night out of the weather while they wait for one
            (Want::Shelter, "lodging"),
        ];
        let life = self.world.get_component::<InnerLife>(v)?;
        let wants = life.wants.clone();
        let pos = *self.world.get_component::<Pos>(v)?;
        let gold = self.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
        for (w, good) in WANT_GOODS {
            if !wants.contains(&w) || desires::is_satisfied(&self.world, v, w, tick) { continue; }
            if gold < goods::worth_at(&self.world, good) { continue; }
            // a bed at the tavern is for whoever hasn't one of their own;
            // a soul with a roof who still wants one wants a better roof,
            // and the keeper can't sell them that
            if good == "lodging" && household::home_of(&self.world, v).is_some() { continue; }
            if let Some(shop) = livelihood::shop_for(&self.world, good, pos) {
                return Some(Goal::BuyAt(shop, good, Some(w)));
            }
        }
        None
    }

    /// Find `input` for worker `v`: send a nearby idle kid to fetch it for
    /// pay when one's about, else walk for it - a seller's satchel, or a
    /// gather site when the land provides
    pub fn errand_for_input(&mut self, v: Entity, input: &'static str, _i: usize) -> bool {
        let pos = match self.world.get_component::<Pos>(v) { Some(p) => *p, None => return false };
        // a kid can run it: any child about - nearby, idle, not already
        // on a job. The village grows, so it's the Kid mark that counts,
        // not whichever three were born first
        if self.rng.below(100) < 40 {
            let kids: Vec<Entity> = self.villagers.iter().copied()
                .filter(|&k| self.world.get_component::<Kid>(k).is_some()).collect();
            for k in kids {
                if villager::is_dead(&self.world, k) || self.travels.iter().any(|t| t.who == k)
                    || self.world.get_component::<OddJob>(k).is_some() { continue; }
                let near = self.world.get_component::<Pos>(k)
                    .map(|p| (p.x - pos.x).abs() + (p.y - pos.y).abs() < 7.0).unwrap_or(false);
                if !near { continue; }
                if let Some(from) = livelihood::seller_for(&self.world, input, pos)
                    .filter(|&s| !self.boycotted(s)) {
                    let pay = livelihood::ODD_JOB_MIN + self.rng.below(4) as f32;
                    self.world.add_component(k, OddJob {
                        hirer: v, good: input.to_string(), qty: 2, from, to: v, pay, loaded: false });
                    self.send(k, Goal::RunOddJob);
                    return true;
                }
            }
        }
        // go themselves: a seller's satchel, the stall, or gather it
        if let Some(s) = livelihood::seller_for(&self.world, input, pos)
            .filter(|&s| !self.boycotted(s)) {
            return self.send(v, Goal::BuyFrom(s, input, 2));
        }
        if vocation::gatherable(input) {
            if let Some(t) = self.gather_site(input, pos) {
                return self.send_gather(v, input, t);
            }
        }
        false
    }

    /// Where to gather `good` nearest `at`: wood off a tree tile, stone
    /// off a rock or boulder, sand off a tile lapping water
    pub fn gather_site(&self, good: &str, at: Pos) -> Option<(i32, i32)> {
        let dist = |x: i32, y: i32| ((x as f32 - at.x).abs() + (y as f32 - at.y).abs()) as i32;
        match good {
            "wood" => self.map.trees.iter().copied()
                .min_by_key(|&(x, y)| dist(x, y)),
            "stone" => self.stones.iter().copied()
                .chain((0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                    .filter(|&(x, y)| self.map.get(x, y) == Some(Tile::Rock)))
                .min_by_key(|&(x, y)| dist(x, y)),
            "sand" => (0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                .filter(|&(x, y)| self.map.passable(x, y)
                    && NEIGHBOURS.iter().any(|(dx, dy)| matches!(self.map.get(x + dx, y + dy), Some(Tile::Water) | Some(Tile::River))))
                .min_by_key(|&(x, y)| dist(x, y)),
            // faggots come off the same trees as the craft-wood - the
            // leavings and deadfall, not the timber
            "faggots" => self.map.trees.iter().copied()
                .min_by_key(|&(x, y)| dist(x, y)),
            // turf cuts off the wet ground by the river, same ground the
            // marsh fevers rise off
            "turf" => (0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
                .filter(|&(x, y)| self.map.get(x, y) == Some(Tile::Grass) && self.map.passable(x, y)
                    && NEIGHBOURS.iter().any(|(dx, dy)| matches!(self.map.get(x + dx, y + dy), Some(Tile::Water) | Some(Tile::River))))
                .min_by_key(|&(x, y)| dist(x, y)),
            _ => None,
        }
    }

    /// The travelling minstrel rides in
    pub fn spawn_minstrel(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        let path = self.map.walk_path(start, (16, 16)).unwrap_or_default();
        self.minstrel = Some(Minstrel { e, path, playing_until: None });
        self.log.push(format!("t{}: a travelling minstrel has come to town.", self.tick));
    }

    /// The circus comes up the road and pitches on the meadow by the gate
    pub fn spawn_circus(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        // a meadow spot just outside the south wall
        let camp = self.map.nearest_buildable(self.town_wall.0 + 4, self.town_wall.3 + 3)
            .unwrap_or((20, 30));
        let path = self.map.walk_path(start, camp).unwrap_or_default();
        self.circus = Some(Circus { e, path, camped_until: None, camp });
        self.log.push(format!("t{}: a circus comes up the road - tents on the meadow by the gate.", self.tick));
    }

    /// Flames at a traveller's feet or on the road just ahead - reason
    /// enough for any of them to turn for home
    fn fire_at_feet(&self, e: Entity, path: &[(i32, i32)]) -> bool {
        if !self.fire.active() { return false; }
        let near = self.world.get_component::<Pos>(e)
            .map(|p| self.fire.burns_near(p.x.round() as i32, p.y.round() as i32, 2))
            .unwrap_or(false);
        near || path.first().map(|&(x, y)| self.fire.on_fire(x, y)).unwrap_or(false)
    }

    /// One frame of the circus: walk in, camp a fortnight of wonder and
    /// light fingers, then leave - and maybe not alone
    pub fn advance_circus(&mut self) {
        let Some(mut c) = self.circus.take() else { return };
        // the meadow's burning or the road in is: the tents fold early
        if self.fire_at_feet(c.e, &c.path) {
            c.camped_until = Some(self.tick.saturating_sub(1));
            c.path.clear();
        }
        if c.camped_until.is_none() {
            let Some(p) = self.world.get_component_mut::<Pos>(c.e) else { return };
            if c.path.is_empty() {
                c.camped_until = Some(self.tick + 10);
                self.say(c.e, "Come see! Wonders from over the hill!", TALK_BUBBLE_SECS * 2);
            } else {
                let (nx, ny) = (c.path[0].0 as f32, c.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; c.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.circus = Some(c);
        } else {
            self.circus = Some(c);
        }
    }

    /// Each day the circus camps: wonder for those who come, a lighter
    /// purse for the same, and the pull grows on the ones the days here
    /// have worn thin - until the tents fold, and someone may go with them
    pub fn circus_tick(&mut self, tick: u64) {
        let camped = self.circus.as_ref().and_then(|c| c.camped_until);
        let Some(until) = camped else { return };
        let camp = self.circus.as_ref().map(|c| c.camp).unwrap();
        let org = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.clone()).unwrap_or_default();
        if tick < until {
            // a night of wonders: joy for the ones who walk out, and a
            // few coins lighter walking home
            for &v in &self.villagers.clone() {
                if villager::is_dead(&self.world, v) { continue; }
                let near = self.world.get_component::<Pos>(v)
                    .map(|p| (p.x.round() as i32 - camp.0).abs() + (p.y.round() as i32 - camp.1).abs() < 8)
                    .unwrap_or(false);
                if !near { continue; }
                desires::satisfy(&mut self.world, v, Want::Joy, tick + 10);
                desires::satisfy(&mut self.world, v, Want::Music, tick + 10);
                if let Some(p) = self.world.get_component_mut::<Purse>(v) {
                    if p.gold > 1.0 { p.gold -= 1.0; } // light fingers in the crowd
                }
                // the bored and the unbelonging feel the pull - belonging
                // is the counterweight
                if !org.contains(&v) && self.world.get_component::<TownRole>(v).is_none() {
                    desires::add_grievance(&mut self.world, v, desires::Problem::Tempted, until);
                }
            }
        } else {
            // the tents fold - and the road takes the one it most tempted
            let mut bond_n: std::collections::HashMap<Entity, usize> = std::collections::HashMap::new();
            self.world.iter::<relationships::Relationship>(|_, r| {
                *bond_n.entry(r.from).or_default() += 1;
                if r.other != r.from { *bond_n.entry(r.other).or_default() += 1; }
            });
            let mark = self.villagers.iter().copied()
                .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v) && !org.contains(&v)
                    && self.world.get_component::<TownRole>(v).is_none()
                    && desires::problems_of(&self.world, v, tick).contains(&desires::Problem::Tempted))
                .min_by_key(|&v| {
                    // the loneliest goes: fewest bonds is the lightest tie
                    bond_n.get(&v).copied().unwrap_or(0)
                });
            if let Some(m) = mark {
                let name = self.name_of(m);
                self.log.push(format!("t{}: {} has run off with the circus!", tick, name));
                // the road takes them whole - whatever errand they were
                // on is spent
                self.travels.retain(|t| t.who != m);
                self.send(m, Goal::Exile);
            }
            self.circus = None;
            self.next_circus = tick + 300 + self.rng.next() % 200;
            self.log.push(format!("t{}: the circus folds its tents and goes.", tick));
        }
    }

    /// The ink-maker comes up the road and pitches his cauldron on the
    /// meadow outside the gate - where the law wants a fire like his
    pub fn spawn_ink_boiler(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        // a clear spot beyond the south wall, off the circus's patch
        let camp = self.map.nearest_buildable(self.town_wall.0 + 8, self.town_wall.3 + 4)
            .unwrap_or((24, 30));
        let path = self.map.walk_path(start, camp).unwrap_or_default();
        self.ink_boiler = Some(InkBoiler { e, path, working_until: None, camp });
        self.log.push(format!("t{}: an ink-maker comes up the road - his cauldron will smoke on the meadow, where such fires belong.", self.tick));
    }

    /// One frame of the ink-maker's visit: walk out to the meadow, work
    /// the cauldron a few days, then the road takes him back
    pub fn advance_ink_boiler(&mut self) {
        let Some(mut b) = self.ink_boiler.take() else { return };
        // flames come for a pot of boiling oil too: he packs and goes
        if self.fire_at_feet(b.e, &b.path) {
            b.working_until = Some(self.tick.saturating_sub(1));
            b.path.clear();
        }
        if b.working_until.is_none() {
            let Some(p) = self.world.get_component_mut::<Pos>(b.e) else { return };
            if b.path.is_empty() {
                b.working_until = Some(self.tick + INK_BOIL_DAYS);
                self.say(b.e, "The oil takes its time - bring your crusts when it spits.", TALK_BUBBLE_SECS * 2);
            } else {
                let (nx, ny) = (b.path[0].0 as f32, b.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; b.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.ink_boiler = Some(b);
        } else if self.tick <= b.working_until.unwrap() {
            self.ink_boiler = Some(b); // the pot's on
        } else {
            // the varnish's made: pack up and walk off
            if let Some(p) = self.world.get_component_mut::<Pos>(b.e).copied() {
                let back = self.traveller_path.last().copied().unwrap_or((29, 14));
                if b.path.is_empty() {
                    b.path = self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back).unwrap_or_default();
                }
                if b.path.is_empty() {
                    self.world.destroy_entity(b.e);
                    self.next_ink_boiler = self.tick + 250 + self.rng.next() % 200;
                    self.log.push(format!("t{}: the ink-maker's moved on, smelling of his trade.", self.tick));
                    return;
                }
                let p = self.world.get_component_mut::<Pos>(b.e).unwrap();
                let (nx, ny) = (b.path[0].0 as f32, b.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; b.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.ink_boiler = Some(b);
        }
    }

    /// Each day the cauldron's on, the recipe walks its stages: two days
    /// of boil, the crusts and onions that take the grease out, the slate
    /// drip that says it's varnish, and the black going in - then the
    /// print shop's stocked with ink the road used to have to bring
    pub fn ink_boiler_tick(&mut self, tick: u64) {
        let Some(b) = &self.ink_boiler else { return };
        let Some(until) = b.working_until else { return };
        let (camp, be) = (b.camp, b.e);
        match tick - (until - INK_BOIL_DAYS) {
            2 => {
                // day three: a pound of crusts and a dozen onions go in
                // to degrease - someone carries them out and comes back
                // smelling of frying oil
                self.say(be, "A pound of crusts and an onion or two - they take the grease out.", TALK_BUBBLE_SECS);
                let candidates: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                    !villager::is_dead(&self.world, v)
                        && self.world.get_component::<TownRole>(v).is_none()
                        && !self.travels.iter().any(|t| t.who == v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - camp.0).abs() + (p.y.round() as i32 - camp.1).abs() <= 8)
                            .unwrap_or(false)
                }).collect();
                // ask until someone can actually make the walk
                for v in candidates {
                    if self.send(v, Goal::TalkTo(be)) {
                        self.log.push(format!("t{}: a villager carries the day's crusts out to the cauldron.", tick));
                        break;
                    }
                }
            }
            3 => {
                // day four: dripped on the slate it hangs like thin glue
                self.say(be, "See it hang off the slate? That's varnish now.", TALK_BUBBLE_SECS);
                self.log.push(format!("t{}: the oil's boiled down to varnish - sticky as thin glue off the slate.", tick));
            }
            4 => {
                // day five: the lampblack goes in - soot, not ash - and
                // it's ink; the print shop's stocked till he comes again
                let shop = self.building_entities.iter().copied().find(|&e| {
                    self.world.get_component::<Building>(e)
                        .map(|b| matches!(b.kind, household::BuildingKind::Shop(Vocation::Printmaker)))
                        .unwrap_or(false)
                }).or(self.trader_stall);
                if let Some(shop) = shop {
                    if let Some(b) = self.world.get_component_mut::<Building>(shop) {
                        household::add_stock(b, "ink", INK_YIELD);
                    }
                }
                self.say(be, "The black goes in - there's your ink.", TALK_BUBBLE_SECS);
                self.log.push(format!("t{}: lampblack into the varnish - the ink-maker sells {} ink before he goes.", tick, INK_YIELD));
            }
            // days one and two: the oil just boils, watched and smoking -
            // and now and then the vapours take fire, as they will; and
            // the acrolein reaches whoever's downwind - sharp on the eyes
            0 | 1 => {
                if self.rng.below(100) < OIL_FLARE_PCT && self.fire.ignite(&self.map, camp.0, camp.1) {
                    self.say(be, "She's caught - keep back!", TALK_BUBBLE_SECS);
                    self.log.push(format!("t{}: the oil takes fire on the meadow.", tick));
                }
                let near: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                    !villager::is_dead(&self.world, v)
                        && self.world.get_component::<Pos>(v)
                            .map(|p| (p.x.round() as i32 - camp.0).abs() + (p.y.round() as i32 - camp.1).abs() <= 6)
                            .unwrap_or(false)
                }).collect();
                for v in near {
                    desires::add_grievance(&mut self.world, v, desires::Problem::Smelly, tick + 30);
                }
            }
            _ => {}
        }
    }

    /// Whatever's alight on the map runs its course: spreading a tile a
    /// day, burning out to scorched ground, and gutting what it reaches
    fn fire_tick(&mut self, tick: u64) {
        if !self.fire.active() && self.fire.scorched.is_empty() { return; }
        let (lit, out) = self.fire.tick(&mut self.map, tick);
        // a building the fire holds loses what it holds - it burns while
        // it stands alight, restocking only once it's cold
        for &(x, y) in self.fire.burning.keys() {
            if let Some(&be) = self.building_entities.iter().find(|&&be| {
                self.world.get_component::<Pos>(be)
                    .map(|p| p.x.round() as i32 == x && p.y.round() as i32 == y).unwrap_or(false)
            }) {
                if let Some(b) = self.world.get_component_mut::<Building>(be) {
                    b.stock.clear();
                }
            }
        }
        // and the canopy score forgets the trees the fire took
        for &t in &out {
            if let Some(c) = self.land.tree_cover.get_mut(&t) { *c = 0.0; }
        }
        // folk beside the flames run for home - errands and all
        if self.fire.active() {
            let fleeing: Vec<Entity> = self.villagers.iter().copied().filter(|&v| {
                !villager::is_dead(&self.world, v)
                    && self.world.get_component::<Pos>(v)
                        .map(|p| self.fire.burns_near(p.x.round() as i32, p.y.round() as i32, 2))
                        .unwrap_or(false)
            }).collect();
            for v in fleeing {
                // but not toward the flames: home if it stands clear,
                // else the market store
                let safe = |e: Entity| self.world.get_component::<Pos>(e)
                    .map(|p| !self.fire.burns_near(p.x.round() as i32, p.y.round() as i32, 4))
                    .unwrap_or(false);
                let home = household::home_of(&self.world, v).filter(|&h| safe(h))
                    .or(Some(self.market_food).filter(|&h| safe(h)));
                if let Some(h) = home {
                    if self.send(v, Goal::HeadHome(h)) {
                        self.say(v, "Fire! Fire on the meadow!", CHAT_BUBBLE_SECS);
                        // the errand's off: the homeward way's the one that stays
                        self.travels.retain(|t| t.who != v || matches!(t.goal, Goal::HeadHome(_)));
                    }
                }
            }
        }
        if !lit.is_empty() || !out.is_empty() {
            self.log.push(format!("t{}: fire on the meadow - {} tiles alight, {} burnt out.", tick, lit.len(), out.len()));
        }
    }

    /// The roving pedlar comes up the road
    pub fn spawn_pedlar(&mut self) {
        let e = self.world.create_entity();
        let start = self.traveller_path.last().copied().unwrap_or((29, 14));
        self.world.add_component(e, Pos { x: start.0 as f32, y: start.1 as f32 });
        let path = self.map.walk_path(start, (16, 16)).unwrap_or_default();
        self.pedlar = Some(Pedlar { e, path, selling_until: None, stall: None, rumor_sold: false });
        self.log.push(format!("t{}: a pedlar comes up the road - pots, pins, and word of the next village.", self.tick));
    }

    /// One frame of the pedlar's visit: walk in, pitch the stall, trade
    /// gossip and oddities a few days, then the road takes him
    pub fn advance_pedlar(&mut self) {
        let Some(mut m) = self.pedlar.take() else { return };
        // no stall is worth a burning pitch: he shoulders his pack
        if self.fire_at_feet(m.e, &m.path) {
            m.selling_until = Some(self.tick.saturating_sub(1));
            m.path.clear();
        }
        if m.selling_until.is_none() {
            // walking to the market
            let Some(p) = self.world.get_component_mut::<Pos>(m.e) else { return };
            if m.path.is_empty() {
                // pitched: a stall of oddities nothing local supplies
                let (px, py) = (p.x, p.y);
                let stall = self.world.create_entity();
                self.world.add_component(stall, Pos { x: px, y: py });
                let mut stock = std::collections::HashMap::new();
                stock.insert("ribbon".to_string(), 4);
                stock.insert("needle".to_string(), 6);
                stock.insert("lute_string".to_string(), 2);
                self.world.add_component(stall, household::Building {
                    kind: household::BuildingKind::TraderStall,
                    name: "the pedlar's pitch".into(),
                    worker: Some(m.e), stock, occupants: vec![],
                    capacity: 1, overseer: None,
                });
                m.stall = Some(stall);
                m.selling_until = Some(self.tick + 3);
                // and the word goes out: a few with a want for finery come
                let buyers: Vec<Entity> = self.villagers.iter().copied()
                    .filter(|&v| !crate::game::dungeon::unavailable(&self.world, v)
                        && !self.travels.iter().any(|t| t.who == v)
                        && self.world.get_component::<InnerLife>(v)
                            .map(|l| l.wants.contains(&Want::Finery)).unwrap_or(false))
                    .take(4).collect();
                for v in buyers { self.send(v, Goal::BuyFrom(stall, "ribbon", 1)); }
                // and he trades in the only currency he makes: news
                const RUMOURS: [&str; 4] = [
                    "Word from the next village - their harvest failed. Bread's dear there.",
                    "Their fisher drowned his boat last month. No fish on their market.",
                    "The lord raised the fifth to a quarter there. Hard times.",
                    "A wolf took a shepherd on their common. They say it walks upright.",
                ];
                let rumour = RUMOURS[(self.tick / 60 % 4) as usize];
                self.say(m.e, rumour, TALK_BUBBLE_SECS * 2);
            } else {
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.pedlar = Some(m);
        } else if self.tick <= m.selling_until.unwrap() {
            // the river tale: he sells it once, and it gets around -
            // true or not is a world parameter
            if !m.rumor_sold {
                m.rumor_sold = true;
                self.say(m.e, "They say a thing glows under the river - a pale stone, older than the walls.", TALK_BUBBLE_SECS * 2);
                // someone who hears it carries it on
                if let Some(&v) = self.villagers.iter().find(|&&v|
                    !villager::is_dead(&self.world, v)) {
                    self.say(v, "The pedlar swears he saw it glow.", CHAT_BUBBLE_SECS);
                }
            }
            self.pedlar = Some(m); // selling at the market
        } else {
            // the road calls: pack up and walk off
            if let Some(p) = self.world.get_component_mut::<Pos>(m.e).copied() {
                let back = self.traveller_path.last().copied().unwrap_or((29, 14));
                if m.path.is_empty() {
                    m.path = self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back).unwrap_or_default();
                }
                if m.path.is_empty() {
                    if let Some(s) = m.stall { self.world.destroy_entity(s); }
                    self.world.destroy_entity(m.e);
                    self.next_pedlar = self.tick + 60 + self.rng.next() % 60;
                    self.log.push(format!("t{}: the pedlar's moved on.", self.tick));
                    return;
                }
                let p = self.world.get_component_mut::<Pos>(m.e).unwrap();
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.pedlar = Some(m);
        }
    }

    /// One frame of the minstrel's visit: ride to the market, play a
    /// while, then ride off - and the town wants music again next season
    pub fn advance_minstrel(&mut self) {
        let Some(mut m) = self.minstrel.take() else { return };
        // the song's no good to anyone with the thatch alight: he turns
        // his horse for the road home
        if self.fire_at_feet(m.e, &m.path) {
            m.playing_until = Some(self.tick.saturating_sub(1));
            m.path.clear();
        }
        if m.playing_until.is_none() {
            // riding to the market
            let Some(p) = self.world.get_component_mut::<Pos>(m.e) else { return };
            if m.path.is_empty() {
                m.playing_until = Some(self.tick + 2);
            } else {
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.minstrel = Some(m);
        } else if self.tick <= m.playing_until.unwrap() {
            // playing at the market: everyone nearby gets their fill of it
            if let Some(p) = self.world.get_component::<Pos>(m.e).copied() {
                for &l in self.villagers.iter().chain(std::iter::once(&self.seats[0].player)) {
                    let near = self.world.get_component::<Pos>(l)
                        .map(|lp| (lp.x - p.x).abs() + (lp.y - p.y).abs() < 5.0).unwrap_or(false);
                    if near {
                        desires::satisfy(&mut self.world, l, Want::Music, self.tick + 90);
                        desires::satisfy(&mut self.world, l, Want::Joy, self.tick + 45);
                    }
                }
            }
            self.minstrel = Some(m);
        } else {
            // the road calls: ride off, and another comes by one day
            if let Some(p) = self.world.get_component_mut::<Pos>(m.e).copied() {
                let back = self.traveller_path.last().copied().unwrap_or((29, 14));
                if m.path.is_empty() {
                    m.path = self.map.walk_path((p.x.round() as i32, p.y.round() as i32), back).unwrap_or_default();
                }
                if m.path.is_empty() {
                    self.world.destroy_entity(m.e);
                    self.next_minstrel = self.tick + 250 + self.rng.next() % 150;
                    return; // he's gone
                }
                let p = self.world.get_component_mut::<Pos>(m.e).unwrap();
                let (nx, ny) = (m.path[0].0 as f32, m.path[0].1 as f32);
                let (dx, dy) = (nx - p.x, ny - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d <= TRAVELLER_SPEED { p.x = nx; p.y = ny; m.path.remove(0); }
                else { p.x += dx / d * TRAVELLER_SPEED; p.y += dy / d * TRAVELLER_SPEED; }
            }
            self.minstrel = Some(m);
        }
    }

    /// The contextual actions that apply right now: work at the player's
    /// shop, pair a picked junior with a master, send a kid on an errand
    pub fn update_ctx_actions_at(&mut self, seat: usize) {
        self.seats[seat].ctx_actions.clear();
        let (px, py) = self.player_pos_at(seat);
        // Work: the player's own craft, at the bench if they have one
        if let Some(c) = self.world.get_component::<Craft>(self.seats[seat].player) {
            let near = c.workplace
                .and_then(|b| self.world.get_component::<Pos>(b))
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() < 4.0)
                .unwrap_or(true);
            if near {
                self.seats[seat].ctx_actions.push(CtxAct::Work);
            }
        }
        if self.seats[seat].selected_building.is_none() {
            let v = self.villagers[self.seats[seat].selected_target];
            if self.world.get_component::<Kid>(v).is_some() {
                self.seats[seat].ctx_actions.push(CtxAct::Errand);
            } else if let Some(c) = self.world.get_component::<Craft>(v) {
                let junior = vocation::level_of(c.skill) < Level::Master && c.master.is_none();
                let has_master = junior && self.villagers.iter().any(|&m| {
                    m != v && self.world.get_component::<Craft>(m)
                        .map(|mc| mc.vocation == c.vocation && vocation::level_of(mc.skill) == Level::Master).unwrap_or(false)
                });
                if has_master {
                    self.seats[seat].ctx_actions.push(CtxAct::PairUp);
                }
            }
            // you know two hearts that are looking: put a word between them
            let pe = self.seats[seat].player;
            let lonely_known = self.world.get_component::<desires::Organizer>(pe)
                .map(|o| o.heard.iter()
                    .filter(|(t, _, u)| *t == "loneliness" && *u > self.tick)
                    .map(|(_, e, _)| *e).collect::<std::collections::HashSet<_>>().len())
                .unwrap_or(0);
            if lonely_known >= 2 {
                self.seats[seat].ctx_actions.push(CtxAct::Matchmake);
            }
        }
        // the cause meets on its cadence and marches on the streets: the
        // buttons show once a campaign exists; seats go to the picked member
        let cause = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org).is_some();
        if cause {
            self.seats[seat].ctx_actions.push(CtxAct::Meet);
            self.seats[seat].ctx_actions.push(CtxAct::Cadence);
            self.seats[seat].ctx_actions.push(CtxAct::March);
        }
        if self.seats[seat].selected_building.is_none() && self.seats[seat].picked {
            let v = self.villagers[self.seats[seat].selected_target];
            if !crate::game::villager::is_dead(&self.world, v) {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                    // someone's inside: the way out is pressure - the
                    // letters go on whether the cause still marches or not
                    self.seats[seat].ctx_actions.push(CtxAct::Petition);
                } else if !crate::game::dungeon::unavailable(&self.world, v) && cause {
                    let is_member = self.world.get_component::<Organization>(self.seats[seat].org)
                        .map(|o| o.members.contains(&v)).unwrap_or(false);
                    if is_member {
                        self.seats[seat].ctx_actions.push(CtxAct::Delegate);
                    }
                    if self.world.get_component::<Craft>(v).is_some() {
                        self.seats[seat].ctx_actions.push(CtxAct::Strike);
                    }
                    if Some(v) == self.collector {
                        self.seats[seat].ctx_actions.push(CtxAct::Townhall);
                    }
                }
            }
        }
    }

    /// Take a contextual action (entry `i` of ctx_actions)
    pub fn ctx_act_at(&mut self, seat: usize, i: usize) {
        let Some(&act) = self.seats[seat].ctx_actions.get(i) else { return };
        let tick = self.tick;
        if let Some(reason) = self.ctx_blocker_at(seat, act) {
            self.log.push(format!("t{}: Can't {}: {}.", tick, ctx_button(act).label.to_lowercase(), reason));
            return;
        }
        let msg = match act {
            CtxAct::Work => {
                let pe = self.seats[seat].player;
                let (voc, workplace, tasked_by) = self.world.get_component::<Craft>(pe)
                    .map(|c| (Some(c.vocation), c.workplace, c.tasked_by)).unwrap_or((None, None, None));
                // nothing comes of a day without the makings: say what's
                // short and how to come by it, not just "nothing yet"
                let missing: Vec<&'static str> = voc.map(|v| vocation::inputs(v).iter().copied()
                    .filter(|g| {
                        let held = self.world.get_component::<Satchel>(pe)
                            .map(|s| s.goods.get(*g).copied().unwrap_or(0)).unwrap_or(0);
                        let stocked = workplace
                            .and_then(|b| self.world.get_component::<Building>(b))
                            .map(|b| b.stock.get(*g).copied().unwrap_or(0)).unwrap_or(0);
                        held + stocked < 1
                    }).collect()).unwrap_or_default();
                if !missing.is_empty() {
                    let how: Vec<String> = missing.iter().map(|g|
                        if vocation::gatherable(g) { format!("{} - gather some", g) }
                        else { format!("{} - buy some", g) }).collect();
                    format!("You can't work: no {} to hand.", how.join(", "))
                } else {
                    let what = voc.and_then(vocation::output).unwrap_or("goods");
                    // where the day's work actually lands is read off the
                    // before-and-after: a full shelf spills to the satchel,
                    // and a full satchel loses it
                    let held_before = self.world.get_component::<Satchel>(pe)
                        .map(|s| s.goods.get(what).copied().unwrap_or(0)).unwrap_or(0);
                    let stocked_before = workplace
                        .and_then(|b| self.world.get_component::<Building>(b))
                        .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0);
                    // a tasked junior's share goes through the senior's shop
                    // and comes back as pay - neither is lost work
                    let senior_shop = tasked_by
                        .and_then(|s| self.world.get_component::<Craft>(s))
                        .and_then(|c| c.workplace);
                    let senior_before = senior_shop
                        .and_then(|b| self.world.get_component::<Building>(b))
                        .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0);
                    let gold_before = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0);
                    let (made, up) = livelihood::work_day(&mut self.world, pe, tick);
                    match up {
                        Some(l) => format!("You work - and reach {}!", vocation::level_name(l)),
                        None if made > 0.0 => {
                            let held = self.world.get_component::<Satchel>(pe)
                                .map(|s| s.goods.get(what).copied().unwrap_or(0)).unwrap_or(0) as i32 - held_before as i32;
                            let stocked = workplace
                                .and_then(|b| self.world.get_component::<Building>(b))
                                .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0) as i32 - stocked_before as i32;
                            // an heir keeps the senior's bench: when it's the
                            // same shelf, its take is already counted above
                            let senior = if senior_shop == workplace { 0 } else {
                                senior_shop.and_then(|b| self.world.get_component::<Building>(b))
                                    .map(|b| b.stock.get(what).copied().unwrap_or(0)).unwrap_or(0) as i32 - senior_before as i32
                            };
                            let gold = self.world.get_component::<Purse>(pe).map(|p| p.gold).unwrap_or(0.0) - gold_before;
                            let shop = workplace.and_then(|b| self.world.get_component::<Building>(b))
                                .map(|b| b.name.as_str()).unwrap_or("the shop");
                            let mut parts: Vec<String> = vec![];
                            if stocked > 0 { parts.push(format!("{} stocked at {}", stocked, shop)); }
                            if held > 0 { parts.push(format!("{} in your satchel", held)); }
                            if senior > 0 {
                                let sname = tasked_by.and_then(|s| self.villagers.iter().position(|v| *v == s))
                                    .map(|i| vill_name(i)).unwrap_or("your senior");
                                parts.push(format!("{} through {}'s shop", senior, sname));
                            }
                            if gold > 0.0 { parts.push(format!("{:.0}g earned", gold)); }
                            let lost = made as i32 - stocked - held - senior;
                            if lost > 0 { parts.push(format!("{} spoiled - no room for it", lost)); }
                            let went = if parts.is_empty() { String::new() } else { format!(" - {}", parts.join(", ")) };
                            format!("You work - {:.0} {} made{}.", made, what, went)
                        }
                        None => "You put in the day, but make nothing yet.".into(),
                    }
                }
            }
            CtxAct::PairUp => {
                let v = self.villagers[self.seats[seat].selected_target];
                let voc = self.world.get_component::<Craft>(v).map(|c| c.vocation);
                let master = voc.and_then(|voc| self.villagers.iter().copied().find(|&m| {
                    m != v && self.world.get_component::<Craft>(m)
                        .map(|mc| mc.vocation == voc && vocation::level_of(mc.skill) == Level::Master).unwrap_or(false)
                        && !crate::game::dungeon::unavailable(&self.world, m)
                }));
                match master {
                    Some(m) => {
                        self.world.get_component_mut::<Craft>(v).unwrap().master = Some(m);
                        relationships::add_relationship(&mut self.world, relationships::Relationship {
                            from: v, other: m, kind: relationships::RelKind::Apprentice, strength: 0.3, interactions: 1 });
                        let mi = self.villagers.iter().position(|x| *x == m).unwrap();
                        format!("You pair {} with {} - he'll learn faster now.", vill_name(self.seats[seat].selected_target), vill_name(mi))
                    }
                    None => "No master of their trade to pair them with.".into(),
                }
            }
            CtxAct::Matchmake => {
                let v = self.villagers[self.seats[seat].selected_target];
                self.match_attempt(v, tick, Some(self.seats[seat].player), "You");
                "You have a word in both their ears.".into()
            }
            CtxAct::Errand => {
                let k = self.villagers[self.seats[seat].selected_target];
                let need = livelihood::needs_input(&self.world, self.seats[seat].player);
                let pos = *self.world.get_component::<Pos>(self.seats[seat].player).unwrap();
                let job = need.and_then(|g| livelihood::seller_for(&self.world, g, pos)
                    .filter(|&s| !self.boycotted(s))
                    .map(|from| (g, from)));
                match job {
                    Some((g, from)) => {
                        self.world.add_component(k, OddJob {
                            hirer: self.seats[seat].player, good: g.to_string(), qty: 2, from, to: self.seats[seat].player,
                            pay: 3.0, loaded: false });
                        // a fresh errand supersedes the day's walk
                        self.travels.retain(|t| t.who != k);
                        self.send(k, Goal::RunOddJob);
                        format!("You send {} to fetch {}.", vill_name(self.seats[seat].selected_target), g)
                    }
                    None => "Nothing you need fetched right now.".into(),
                }
            }
            CtxAct::Meet => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
                    return;
                };
                match crate::game::campaign::hold_campaign_meeting(&mut self.world, c, tick) {
                    Some(out) => {
                        let who: Vec<String> = out.attendees.iter()
                            .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string()))
                            .collect();
                        // the room talks it through first: what it decided is
                        // what the notes and the consent report
                        let d = crate::game::campaign_dialogue::deliberate_meeting(&mut self.world, c, &out, tick);
                        let decided = d.as_ref().map(|d| d.decided).unwrap_or(true);
                        let consent = d.as_ref().map(|d| d.consent).unwrap_or(out.consent);
                        let mut msg = format!("{} meet about the campaign: {}", out.attendees.len(), who.join(", "));
                        if out.drifted { msg += " - run by whoever pulled hardest, nobody asked"; }
                        if out.noted && decided { msg += " - the notes keep the resolves"; }
                        if !decided { msg += " - nothing settled"; }
                        if consent { msg += " - consent banked for the next action"; }
                        if !out.healed.is_empty() {
                            let healed: Vec<String> = out.healed.iter()
                                .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string())).collect();
                            msg += &format!(" - {} heard out", healed.join(", "));
                        }
                        // the log keeps the outcome; what the room said goes
                        // on the TALK panel, in the order it was said
                        if let Some(d) = d {
                            let mut with = vec![self.seats[seat].player];
                            with.extend(out.attendees.iter().copied());
                            self.record_campaign_talk(with, &d);
                        }
                        msg
                    }
                    None => "The campaign isn't due to meet yet.".into(),
                }
            }
            CtxAct::Cadence => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else {
                    return;
                };
                // the organizer's recommendation: the meeting steps up or
                // down a notch - weekly for the long haul, daily for the fight
                let now = self.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
                    .map(|m| m.cadence_days).unwrap_or(3);
                let next = match now { 0..=3 => 7, 4..=7 => 14, 8..=14 => 30, _ => 3 };
                crate::game::campaign::set_cadence(&mut self.world, c, next, tick);
                format!("The cause will meet every {} days now - the organizer's word.", next)
            }
            CtxAct::Delegate => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let v = self.villagers[self.seats[seat].selected_target];
                let name = vill_name(self.seats[seat].selected_target);
                use crate::game::campaign::Role;
                let role_name = |r: Role| match r {
                    Role::TurnoutCoordinator => "turnout coordinator",
                    Role::NoteTaker => "note taker",
                    Role::Chair => "chair",
                };
                // a seated member doesn't get re-seated silently - their
                // old seat would empty behind the new one
                let already = self.world.get_component::<crate::game::campaign::Campaign>(c)
                    .and_then(|c| c.seats.iter().find(|s| s.holder == v)).map(|s| s.role);
                match already {
                    Some(r) => format!("{} already holds the {} seat.", name, role_name(r)),
                    // the first seat standing empty: turnout, notes, then chair
                    None => {
                        let open = [Role::TurnoutCoordinator, Role::NoteTaker, Role::Chair].iter().copied()
                            .find(|&r| crate::game::campaign::seat_holder(&self.world, c, r).is_none());
                        match open {
                            Some(r) if crate::game::campaign::delegate_role(&mut self.world, c, r, v) =>
                                format!("You name {} the campaign's {}.", name, role_name(r)),
                            Some(_) => format!("{} can't hold a seat - they have to be a living member.", name),
                            None => "Every seat is spoken for - someone would have to step down.".into(),
                        }
                    }
                }
            }
            CtxAct::March => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let pe = self.seats[seat].player;
                match crate::game::actions::demonstrate(&mut self.world, c, pe, tick) {
                    Some(out) => {
                        let n = out.turnout.len();
                        // guards already posted stay posted - a second
                        // crackdown doesn't pile more knights into the frame
                        let posted_before = self.world
                            .get_component::<crate::game::reactions::GuardsPosted>(self.seats[seat].org).is_some();
                        let r = crate::game::reactions::react(&mut self.world, c, &out.turnout, tick);
                        // everyone who saw it and felt the cause wake in them
                        let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                            .map(|cc| cc.issue.clone()).unwrap_or_default();
                        let members: std::collections::HashSet<Entity> =
                            self.world.get_component::<Organization>(self.seats[seat].org)
                                .map(|o| o.members.iter().copied().collect()).unwrap_or_default();
                        let mut woke = 0;
                        for &v in &self.villagers {
                            // a march is seen, not invited to: shunned or
                            // not, whoever shares the stake or the friend
                            // can feel the cause wake
                            if !members.contains(&v) {
                                woke += crate::game::reactions::witness(&mut self.world, v, &issue, &out.turnout, tick) as usize;
                            }
                        }
                        let mut msg = match r {
                            Some(crate::game::reactions::Reaction::Crackdown) => {
                                self.toasts.push(("knights posted to the streets".into(), tick + 40));
                                format!("{} march in the street - and the knights come out.", n)
                            }
                            Some(crate::game::reactions::Reaction::Watched) =>
                                format!("{} march in the street - the names get taken.", n),
                            _ => format!("{} march in the street - beneath notice, for now.", n),
                        };
                        if woke > 0 { msg += &format!(" {} watching feel the cause wake.", woke); }
                        // a crackdown takes the loudest - and the cause
                        // becomes a prisoner
                        if r == Some(crate::game::reactions::Reaction::Crackdown) {
                            // the knights stand in the streets where everyone
                            // can see them - once; posted guards stay posted
                            if !posted_before {
                                let (px, py) = self.player_pos_at(seat);
                                let o = self.seats[seat].org;
                                self.posted.push((o, px + 1.5, py));
                                self.posted.push((o, px - 1.5, py));
                            }
                            // a campaign that keeps drawing the knights gets
                            // a rule written against it - the movement
                            // mattered enough to be outlawed
                            if posted_before
                                && self.world.get_component::<crate::game::reactions::AssemblyBan>(self.seats[seat].org).is_none() {
                                self.world.add_component(self.seats[seat].org,
                                    crate::game::reactions::AssemblyBan { cap: 2 });
                                msg += " The Reeve proclaims it law: no more than two assembled.";
                                self.toasts.push(("the ban is proclaimed".into(), tick + 40));
                            }
                            // every crackdown can still take someone
                            if let Some(taken) = crate::game::dungeon::arrest(&mut self.world, c, tick) {
                                // hauled off mid-errand: whatever they were doing is dropped
                                self.travels.retain(|t| t.who != taken);
                                let tn = self.villagers.iter().position(|v| *v == taken).map(|i| vill_name(i))
                                    .unwrap_or("someone");
                                msg += &format!(" {} is taken.", tn);
                                // the release effort is a campaign of its own:
                                // the issue is a person now, not a grievance
                                let rc = crate::game::dungeon::found_release_campaign(&mut self.world,
                                    self.seats[seat].org, taken, tick);
                                // those who watched *them* hauled off - a friend
                                // of the taken, not just anyone in the crowd -
                                // sign on for their release, no organizing needed
                                let scene = [taken];
                                let mut signed = 0usize;
                                for &v in &self.villagers.clone() {
                                    if crate::game::reactions::witness(&mut self.world, v,
                                            &"the prisoners".to_string(), &scene, tick)
                                        && crate::game::dungeon::join(&mut self.world, rc, v) {
                                        signed += 1;
                                    }
                                }
                                if signed > 0 { msg += &format!(" {} sign on to free them.", signed); }
                                self.toasts.push((format!("{} is taken", tn), tick + 40));
                            }
                        }
                        // the campaign talks over what the street brought
                        // (after any arrest, so the taken are counted)
                        let ev = crate::game::campaign_arc::ArcEvent::March(
                            r.unwrap_or(crate::game::reactions::Reaction::Ignored));
                        if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick) {
                            let mut with = vec![pe];
                            with.extend(out.turnout.iter().copied());
                            self.record_campaign_talk(with, &d);
                        }
                        msg
                    }
                    None => "You call a march - and nobody comes. The street stays empty.".into(),
                }
            }
            CtxAct::Strike => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let v = self.villagers[self.seats[seat].selected_target];
                let Some(voc) = self.world.get_component::<Craft>(v).map(|c| c.vocation) else {
                    return;
                };
                match crate::game::actions::strike_vote(&mut self.world, c, voc, tick) {
                    Some(out) => {
                        let ev = crate::game::campaign_arc::ArcEvent::Strike { called: out.called };
                        if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick) {
                            self.record_campaign_talk(vec![self.seats[seat].player, v], &d);
                        }
                        format!("{} of {} say yes{}. {}", out.yes, out.yes + out.no,
                            if out.called { " - the strike is on" } else { "" },
                            if out.called { "The trade stands together." } else { "Less than nine of ten: it fails in public." })
                    }
                    None => "Nobody of the trade to ballot.".into(),
                }
            }
            CtxAct::Townhall => {
                let Some(c) = crate::game::campaign::campaign_of(&self.world, self.seats[seat].org) else { return };
                let Some(target) = self.collector else { return };
                use crate::game::actions::{townhall, TargetResponse};
                match townhall(&mut self.world, c, target, tick) {
                    Some(out) => { let msg = match out.response {
                        TargetResponse::Carries => {
                            self.toasts.push(("the Reeve takes your demands to the king".into(), tick + 40));
                            // the carried demand lands as a grant: the load
                            // the issue names eases for the cause's people,
                            // and this year's taxing stops counting - but
                            // only the issue fought for is answered
                            let oe = self.seats[seat].org;
                            let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                                .map(|cc| cc.issue.clone()).unwrap_or_default();
                            self.world.add_component(oe, crate::game::reactions::LordGrant::full(
                                &issue, tick, tick + crate::game::desires::YEAR_DAYS));
                            // the tag the tax fight is voted under
                            if issue == "the taxes" {
                                if let Some(o) = self.world.get_component::<Organization>(oe).map(|o| o.members.clone()) {
                                    for m in o {
                                        crate::game::desires::lift_grievance(&mut self.world, m, crate::game::desires::Problem::Taxed);
                                    }
                                }
                            }
                            format!("{} face the Reeve - and he says the king will hear of this.", out.turnout.len())
                        }
                        TargetResponse::Concedes => {
                            // it touches him, but the room was thin: part of
                            // it, for half a year - the grievance stands
                            self.toasts.push(("the Reeve grants a part of it".into(), tick + 40));
                            let oe = self.seats[seat].org;
                            let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                                .map(|cc| cc.issue.clone()).unwrap_or_default();
                            // a part already granted and honoured isn't
                            // granted again: pressing a thin room twice
                            // doesn't stretch it
                            if out.nothing_new {
                                format!("{} face the Reeve - he grants no more than he already has.", out.turnout.len())
                            } else {
                                self.world.add_component(oe, crate::game::reactions::LordGrant::partial(
                                    &issue, tick, tick + crate::game::desires::YEAR_DAYS / 2));
                                format!("{} face the Reeve - he grants a part of it, and no more.", out.turnout.len())
                            }
                        }
                        TargetResponse::Deflects =>
                            format!("The Reeve sends regards and a clerk. {} went home hollow.", out.turnout.len()),
                        TargetResponse::Refuses =>
                            format!("The doors stay shut on {} voices. They go home hollow.", out.turnout.len()),
                    };
                    // the answer at the door is the arc's outcome (read
                    // after any grant above has landed)
                    let ev = crate::game::campaign_arc::ArcEvent::Townhall(out.response);
                    if let Some(d) = crate::game::campaign_arc::deliberate_arc(&mut self.world, c, ev, tick) {
                        let mut with = vec![self.seats[seat].player, target];
                        with.extend(out.turnout.iter().copied());
                        self.record_campaign_talk(with, &d);
                    }
                    msg },
                    None => "Nobody turns out for the townhall.".into(),
                }
            }
            CtxAct::Petition => {
                let v = self.villagers[self.seats[seat].selected_target];
                crate::game::dungeon::press(&mut self.world, v, 0.3);
                // the hold: what the state spends to keep them inside
                if crate::game::dungeon::release_check(&mut self.world, v, 1.0) {
                    let name = vill_name(self.seats[seat].selected_target);
                    self.toasts.push((format!("{} walks free", name), tick + 40));
                    format!("The letters don't stop - and the cell opens. {} walks free.", name)
                } else {
                    let p = self.world.get_component::<crate::game::dungeon::ReleaseCampaign>(v)
                        .map(|r| r.pressure).unwrap_or(0.0);
                    format!("Letters, vigils, a line out the door ({:.0}% there).", p * 100.0)
                }
            }
        };
        self.log.push(format!("t{}: {}", tick, msg));
    }

    /// The frame a client draws: the authoritative sim's whole view of the
    /// world this seat should see. One per player seat per broadcast.
    pub fn ui_model_at(&self, seat: usize) -> UiModel {
        let mut m = self.ui_model_parts_at(seat);
        // the debug readout shows what this seat could know, never the
        // secret gauges: the model goes over the wire to remote clients
        m.debug_lines = kpi::kpi_lines(&kpi::public_kpis(&m));
        m
    }

    /// The seat's frame, all but the lines derived from the frame itself
    fn ui_model_parts_at(&self, seat: usize) -> UiModel {
        let world = &self.world;
        let season = world.get_component::<climate::Climate>(self.climate).unwrap().season;
        let in_reach = self.villagers_in_reach_at(seat);
        let mut people: Vec<PersonV> = vec![];
        // the fight this org is in - woke only marks someone whose shock
        // matches it (a stranger woken to a different cause isn't your
        // recruit)
        let fight = crate::game::campaign::campaign_of(world, self.seats[seat].org)
            .and_then(|c| world.get_component::<crate::game::campaign::Campaign>(c))
            .map(|c| c.issue.clone());
        for (i, v) in self.villagers.iter().enumerate() {
            let Some(p) = world.get_component::<Pos>(*v) else { continue };
            // what the cause has done to them, read over their head:
            // taken (in the dungeon), hollow (a blow-off to coach),
            // woke (moved by the cause - recruitable)
            let cause_mark = if world.get_component::<crate::game::dungeon::Imprisoned>(*v).is_some() {
                " (taken)"
            } else if world.get_component::<crate::game::actions::Demoralized>(*v).is_some() {
                " (hollow)"
            } else if fight.as_ref().map(|issue|
                world.get_component::<crate::game::reactions::Sympathized>(*v)
                    .map(|s| s.issues.iter().any(|(i, _)| i == issue)).unwrap_or(false))
                .unwrap_or(false) {
                " (woke)"
            } else { "" };
            let kid = world.get_component::<Kid>(*v).is_some();
            // the marks go by the state, not the roll: a child grown
            // drops the (child), and whoever holds the herb hut is the herb-wife
            let role = if Some(*v) == self.herbologist { " (herb-wife)" }
                else if kid { " (child)" } else { "" };
            people.push(PersonV {
                e: *v, kind: PKind::Villager(i), pos: (p.x, p.y),
                label: Some(format!("{}{}{}", vill_name(i), role, cause_mark)),
                dead: villager::is_dead(world, *v),
                kid,
                selected: self.seats[seat].selected_target == i && self.seats[seat].picked,
                in_reach: in_reach.contains(v),
                hungry: world.get_component::<Hunger>(*v).map(|h| h.value < 40.0).unwrap_or(false),
                sick: is_sick(world, *v),
                act_mark: match world.get_component::<Villager>(*v).map(|v| v.activity) {
                    Some(Activity::SeekingFood) => 1, Some(Activity::Idle) | None => 0, _ => 2 },
                bond: false,
            });
        }
        for (i, v) in self.rivals.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                people.push(PersonV { e: *v, kind: PKind::Rival(i), pos: (p.x, p.y),
                    label: Some(RIVAL_NAMES[i].to_string()), dead: false, kid: false,
                    selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        let (px, py) = self.player_pos_at(seat);
        people.push(PersonV { e: self.seats[seat].player, kind: PKind::Player, pos: (px, py),
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        for (i, (x, y)) in self.guards.iter().copied()
            .chain(self.posted.iter().map(|p| (p.1, p.2))).enumerate() {
            people.push(PersonV { e: 0, kind: PKind::Guard(i), pos: (x, y),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        }
        people.push(PersonV { e: 0, kind: PKind::Priest, pos: self.priest,
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        if let Some((tx, ty)) = self.traveller_pos() {
            people.push(PersonV { e: 0, kind: PKind::Traveller, pos: (tx, ty),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        }
        if let Some(m) = &self.minstrel {
            if let Some(p) = world.get_component::<Pos>(m.e) {
                people.push(PersonV { e: m.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        if let Some(m) = &self.pedlar {
            if let Some(p) = world.get_component::<Pos>(m.e) {
                people.push(PersonV { e: m.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        if let Some(c) = &self.circus {
            if let Some(p) = world.get_component::<Pos>(c.e) {
                people.push(PersonV { e: c.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the circus".into()), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        if let Some(b) = &self.ink_boiler {
            if let Some(p) = world.get_component::<Pos>(b.e) {
                people.push(PersonV { e: b.e, kind: PKind::Traveller, pos: (p.x, p.y),
                    label: Some("the ink-boiler".into()), dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        // the rare events' figures: wandering trades drawn as travellers,
        // strays drawn as the beasts they are - all labelled by their errand
        for r in &self.rare {
            let Some(e) = r.e else { continue };
            let Some(p) = world.get_component::<Pos>(e) else { continue };
            let spec = events::spec(r.id);
            let kind = match spec.fig {
                events::Fig::Traveller => PKind::Traveller,
                events::Fig::Pet(s) => PKind::Pet(s),
            };
            people.push(PersonV { e, kind, pos: (p.x, p.y),
                label: Some(spec.name.into()), dead: false, kid: false, selected: false,
                in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        }
        for &(x, y) in &self.gulls {
            people.push(PersonV { e: 0, kind: PKind::Gull, pos: (x, y),
                label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        }
        let hen_t = (self.frame_n / 12) as f32;
        people.push(PersonV { e: 0, kind: PKind::Chicken,
            pos: (self.chicken_at.0 + hen_t.cos() * 0.5, self.chicken_at.1 + (hen_t * 1.7).sin() * 0.4),
            label: None, dead: false, kid: false, selected: false, in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
        for (pet_e, species, (ox, oy)) in &self.pets {
            if let Some(pet) = world.get_component::<Pet>(*pet_e) {
                if let Some(p) = world.get_component::<Pos>(pet.owner.unwrap_or(self.villagers[0])) {
                    people.push(PersonV { e: *pet_e, kind: PKind::Pet(*species),
                        pos: (p.x + ox, p.y + oy), label: None, dead: false, kid: false,
                        selected: false, in_reach: false, hungry: false, sick: false,
                        act_mark: 0, bond: pet.bond_strength > 0.5 });
                }
            }
        }
        for &a in &self.flock {
            if let (Some(l), Some(p)) = (world.get_component::<Livestock>(a), world.get_component::<Pos>(a)) {
                people.push(PersonV { e: a, kind: if l.kind == LivestockKind::Sheep { PKind::Sheep } else { PKind::Goat },
                    pos: (p.x, p.y), label: None, dead: false, kid: false, selected: false,
                    in_reach: false, hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        for &a in &self.hens {
            if let Some(p) = world.get_component::<Pos>(a) {
                people.push(PersonV { e: a, kind: PKind::Chicken, pos: (p.x, p.y), label: None,
                    dead: false, kid: false, selected: false, in_reach: false,
                    hungry: false, sick: false, act_mark: 0, bond: false });
            }
        }
        people.sort_by(|a, b| (a.pos.0 + a.pos.1).partial_cmp(&(b.pos.0 + b.pos.1)).unwrap());

        let risen = |i: usize| self.works.iter().find(|w| w.building == i).map(|w| w.risen()).unwrap_or(1.0);
        let buildings: Vec<BldgV> = self.buildings.iter().enumerate()
            .map(|(i, (x, y, s))| BldgV { x: *x, y: *y, sprite: *s, risen: risen(i),
                derelict: self.derelict.contains(&i), smoke: i < 3 })
            .collect();
        let bubbles: Vec<BubbleV> = self.bubbles.iter().filter_map(|(e, text, _, thought, _)| {
            // the dead don't talk - a bubble over a grave is ghost chatter
            if villager::is_dead(world, *e) { return None; }
            let p = world.get_component::<Pos>(*e)?;
            let name = self.villagers.iter().position(|v| *v == *e).map(|i| vill_name(i).to_string())
                .or_else(|| self.rivals.iter().position(|r| *r == *e).map(|i| RIVAL_NAMES[i].to_string()));
            Some(BubbleV { e: *e, pos: (p.x, p.y), text: text.clone(), thought: *thought, name })
        }).collect();
        let org = world.get_component::<Organization>(self.seats[seat].org).unwrap();
        let members: Vec<String> = org.members.iter()
            .filter_map(|m| self.villagers.iter().position(|v| v == m).map(|i| {
                let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(*m).is_some();
                format!("{}{}", vill_name(i), if held { " (held)" } else { "" })
            }))
            .collect();
        let issues: Vec<String> = world.get_component::<Organizer>(self.seats[seat].player)
            .map(|o| o.issues.iter().map(|t| desires::issue_name(t.clone())).collect()).unwrap_or_default();
        // a player seat drives a PlayerCharacter; an agent seat drives a plain
        // villager, whose panel is their own name and purse
        let me = self.seats[seat].player;
        let (my_name, my_influence, my_gold) = match world.get_component::<PlayerCharacter>(me) {
            Some(p) => (p.name.clone(), p.influence, p.resources),
            None => (self.name_of(me), 0.0,
                world.get_component::<Purse>(me).map(|p| p.gold).unwrap_or(0.0)),
        };
        let food = world.get_component::<villager::FoodSource>(self.market_food).map(|f| f.servings).unwrap_or(0);
        let guilds = world.query::<Guild>().len();
        let name_of = |e: Entity| {
            if e == self.seats[seat].player { return "you".to_string(); }
            self.villagers.iter().position(|v| *v == e).map(|i| vill_name(i).to_string())
                .or_else(|| self.rivals.iter().position(|r| *r == e).map(|i| RIVAL_NAMES[i].to_string()))
                .unwrap_or_else(|| "someone".to_string())
        };
        let campaign = crate::game::campaign::campaign_of(world, self.seats[seat].org).map(|e| {
            use crate::game::campaign::{Campaign, CampaignMeeting, Framing, Role};
            let c = world.get_component::<Campaign>(e).unwrap();
            let mtg = world.get_component::<CampaignMeeting>(e);
            let seats = [Role::TurnoutCoordinator, Role::NoteTaker, Role::Chair].iter().map(|&r| {
                let label = match r { Role::TurnoutCoordinator => "turnout", Role::NoteTaker => "notes", Role::Chair => "chair" };
                let holder = c.seats.iter().find(|s| s.role == r).map(|s| name_of(s.holder))
                    .unwrap_or_else(|| "(empty)".to_string());
                format!("{} {}", label, holder)
            }).collect();
            CampaignV {
                issue: c.issue.clone(),
                framing: match c.framing { Framing::SharedGrievance => "shared grievance",
                    Framing::CommonFoe => "common foe", Framing::MoralDuty => "moral duty" }.to_string(),
                dissenters: c.dissenters.len(), seats,
                resolves: mtg.map(|m| m.resolves).unwrap_or(0),
                due: mtg.map(|m| self.tick.saturating_sub(m.last_held) >= m.cadence_days).unwrap_or(false),
                cadence_days: mtg.map(|m| m.cadence_days).unwrap_or(0),
            }
        });
        let blocked = [self.action_blocker_at(seat, 0), self.action_blocker_at(seat, 1), self.action_blocker_at(seat, 2),
                       self.action_blocker_at(seat, 3), self.action_blocker_at(seat, 4)];
        let inspector = if self.seats[seat].selected_building.filter(|_| !self.seats[seat].picked).is_some() {
            self.building_inspector_at(seat)
        } else {
            Some(self.villager_inspector_at(seat))
        };
        UiModel {
            player_pose: (self.seats[seat].facing, self.seats[seat].walking),
            // the courier (entity 0 in people) faces the way it rides
            poses: self.villager_poses.iter().map(|(&e, &(_, f, w))| (e, f, w))
                .chain(self.traveller_pos().zip(self.traveller_pos_at(self.frame_n.wrapping_sub(1)))
                    .map(|(now, was)| (0, facing_on_screen(now.0 - was.0, now.1 - was.1), true))).collect(),
            tick: self.tick, frame: self.frame_n,
            map: self.map.clone(), town_wall: self.town_wall,
            season, precip: self.precip, rainbow_from: self.rainbow_from,
            paused: self.paused, speed: self.speed,
            stones: self.stones.clone(), buildings, people, bubbles,
            castle_keeps: self.map.castle_keeps.iter().map(|&(x, y)| (x as f32, y as f32)).collect(),
            settlement_houses: self.map.settlement_houses.iter().map(|&(x, y)|
                (x as f32, y as f32, BSprite::House((noise(x, y, 41) * 97.0) as usize))).collect(),
            toasts: self.toasts.iter().filter(|(_, until)| self.tick < *until).map(|(m, _)| m.clone()).collect(),
            // only the lines the window would show - cloning the whole
            // history every frame was the model's biggest copy
            log: visible_log(&self.log).into_iter().cloned().collect(),
            sel: Selection { target: self.seats[seat].selected_target, picked: self.seats[seat].picked, building: self.seats[seat].selected_building },
            reach: in_reach.iter().filter_map(|v| self.villagers.iter().position(|x| x == v)).collect(),
            reach_target: self.reach_target_at(seat),
            blocked, ctx: self.seats[seat].ctx_actions.clone(),
            ctx_blocked: self.seats[seat].ctx_actions.iter()
                .map(|a| self.ctx_blocker_at(seat, *a)).collect(),
            inspector,
            panel: PanelV {
                name: my_name, standing: crate::game::player::standing(my_influence).to_string(),
                influence: my_influence, gold: my_gold,
                org: org.name.clone(), members, issues, guilds, food, campaign,
            },
            mill: Some((MILL_TILE.0, MILL_TILE.1)),
            herbs: self.herb_patches.iter().filter(|p| self.tick >= p.next_ready)
                .map(|p| (p.tile.0 as f32, p.tile.1 as f32)).collect(),
            mush: self.mush_patches.iter().filter(|p| self.tick >= p.next_ready)
                .map(|p| (p.tile.0 as f32, p.tile.1 as f32)).collect(),
            skeps: self.hives.iter().filter_map(|&h| world.get_component::<Pos>(h)
                .map(|p| (p.x, p.y))).collect(),
            swarm: self.swarm.and_then(|(w, _)| world.get_component::<Pos>(w).map(|p| (p.x, p.y))),
            trails: self.trails.iter().filter(|(_, t)| t.strength > 0.15)
                .map(|(&(x, y), t)| (x as f32, y as f32, t.strength)).collect(),
            stumps: self.land.tree_cover.iter().filter(|(_, &c)| c < 0.3)
                .map(|(&(x, y), _)| (x as f32, y as f32)).collect(),
            murk: self.land.effluent.iter().filter(|(_, &e)| e > 0.4)
                .map(|(&(x, y), _)| (x as f32, y as f32)).collect(),
            pond: if self.land.pit_water >= 0.7 {
                self.pit.map(|(x, y)| (x as f32, y as f32))
            } else { None },
            burning: self.fire.burning.keys().map(|&(x, y)| (x as f32, y as f32)).collect(),
            scorched: self.fire.scorched.keys().map(|&(x, y)| (x as f32, y as f32)).collect(),
            gardens: self.gardens.iter()
                .map(|&(_, (x, y), g, _)| (x as f32, y as f32, g.min(1.0))).collect(),
            sky: self.sky.map(|(k, _)| k.to_string()),
            painted: self.painted.iter()
                .filter_map(|&(b, _)| self.world.get_component::<Pos>(b).map(|p| (p.x, p.y))).collect(),
            cave: self.cave_pos.map(|(x, y)| (x as f32, y as f32)),
            talk: self.talk_panel_at(seat),
            convo: self.convo_panel_at(seat),
            debug_lines: vec![],
            sfx: self.sfx.iter().map(|c| c.kind.to_string()).collect(),
        }
    }

    /// The inspector card for the building this seat clicked
    fn building_inspector_at(&self, seat: usize) -> Option<InspectorV> {
        let bi = self.seats[seat].selected_building?;
        let be = *self.building_entities.get(bi)?;
        let world = &self.world;
        let b = world.get_component::<Building>(be)?;
        let mut lines: Vec<Line> = vec![];
        let kind_word = match b.kind {
            BuildingKind::Home => "Home",
            BuildingKind::Shop(_) => "Workshop",
            BuildingKind::Church => "Church",
            BuildingKind::WatchPost => "Watch",
            BuildingKind::Mill => "Mill",
            BuildingKind::Granary => "Granary",
            BuildingKind::TraderStall => "Trader's stall",
            BuildingKind::Pasture => "Pasture",
            BuildingKind::Apiary => "Apiary",
            BuildingKind::Cellar => "Cheese cellar",
        };
        lines.push(Line { x: 18, y: 326, text: b.name.clone(), rgb: (255, 255, 255) });
        lines.push(Line { x: 18, y: 340, text: kind_word.into(), rgb: (200, 200, 230) });
        let worker = b.worker.and_then(|w| self.villagers.iter().position(|v| *v == w))
            .map(|i| vill_name(i).to_string())
            .or_else(|| b.worker.map(|_| "you".to_string()));
        let who = worker.map(|w| format!("worked by {}", w)).unwrap_or_else(|| "stands empty".to_string());
        let col = if household::derelict(b) { (230, 140, 90) } else { (190, 190, 210) };
        lines.push(Line { x: 18, y: 354, text: who, rgb: col });
        if let Some(o) = b.overseer {
            let on = self.villagers.iter().position(|v| *v == o).map(|i| vill_name(i)).unwrap_or("someone");
            lines.push(Line { x: 18, y: 368, text: format!("overseer: {}", on), rgb: (200, 160, 120) });
        }
        if b.kind == BuildingKind::Home {
            let names: Vec<&str> = b.occupants.iter()
                .filter_map(|o| self.villagers.iter().position(|v| *v == *o).map(|i| vill_name(i)))
                .collect();
            lines.push(Line { x: 18, y: 368,
                text: format!("lives here: {}", if names.is_empty() { "no one".into() } else { names.join(", ") }),
                rgb: (190, 190, 210) });
        }
        let offers = household::on_offer_at(&self.world, b);
        let stock_line = offers.iter().take(3).map(|(g, q, _)| format!("{} {}", g, q))
            .collect::<Vec<_>>().join(" \u{00b7} ");
        lines.push(Line { x: 18, y: 390,
            text: if stock_line.is_empty() { "stock: empty".into() } else { format!("stock: {}", stock_line) },
            rgb: (180, 190, 210) });
        if let Some((g, q, pr)) = offers.first() {
            lines.push(Line { x: 18, y: 404, text: format!("for offer: {} {} @ {:.0}g", q, g, pr), rgb: (150, 210, 150) });
        }
        Some(InspectorV { fig: None, dead: false, died_day: None, lines })
    }

    /// The inspector card for the selected villager
    fn villager_inspector_at(&self, seat: usize) -> InspectorV {
        let world = &self.world;
        let i = self.seats[seat].selected_target;
        let v = self.villagers[i];
        let mut lines: Vec<Line> = vec![];
        lines.push(Line { x: 60, y: 330, text: vill_name(i).into(), rgb: (255, 255, 255) });
        if let Some(day) = self.died_on[i] {
            lines.push(Line { x: 60, y: 344, text: format!("Died on day {}", day), rgb: (230, 90, 80) });
            lines.push(Line { x: 60, y: 358, text: "rest in peace".into(), rgb: (150, 150, 165) });
            return InspectorV { fig: None, dead: true, died_day: Some(day), lines };
        }
        let org_members = world.get_component::<Organization>(self.seats[seat].org)
            .map(|o| o.members.clone()).unwrap_or_default();
        let occ = desires::occupation_of(world, v).unwrap_or_else(|| "Villager".into());
        lines.push(Line { x: 60, y: 342,
            text: format!("{} \u{00b7} {}", occ, vocation::class_name(desires::class_of(world, v))), rgb: (200, 200, 230) });
        if livelihood::has_role(&self.world, v, TownRole::Herbologist) {
            let n = herbology::remedies_on_hand(world, v);
            lines.push(Line { x: 60, y: 354,
                text: format!("knows the cures \u{00b7} {} remedies", n), rgb: (150, 210, 150) });
        }
        if let Some(c) = world.get_component::<Craft>(v) {
            lines.push(Line { x: 60, y: 354,
                text: format!("\u{1}Craft\u{2}{}{:.0}%", c.skill, c.skill * 100.0), rgb: (0, 0, 0) });
        }
        if let Some(p) = world.get_component::<Purse>(v) {
            lines.push(Line { x: 60, y: 366, text: format!("Gold {:.0}", p.gold), rgb: (240, 210, 120) });
        }
        if let Some(h) = world.get_component::<Hunger>(v) {
            lines.push(Line { x: 60, y: 378,
                text: format!("\u{1}Fed\u{2}{}{}", h.value / 100.0, fullness_word(h.value)), rgb: (0, 0, 0) });
        }
        if let Some(he) = world.get_component::<Health>(v) {
            let note = disease_of(world, v).map(|(name, stage)| format!("{} ({:?})", name, stage))
                .unwrap_or_else(|| format!("{:.0}%", he.condition * 100.0));
            lines.push(Line { x: 60, y: 390, text: format!("\u{1}Health\u{2}{}{}", he.condition, note), rgb: (0, 0, 0) });
        }
        let home = household::home_of(world, v)
            .and_then(|h| world.get_component::<Building>(h)).map(|b| b.name.clone());
        let shop = world.get_component::<Craft>(v).and_then(|c| c.workplace)
            .and_then(|s| world.get_component::<Building>(s)).map(|b| b.name.clone());
        let roost = match (&home, &shop) {
            (Some(h), Some(s)) => format!("{} \u{00b7} {}", h, s),
            (Some(h), None) => h.clone(),
            (None, Some(s)) => format!("no home \u{00b7} {}", s),
            (None, None) => "no home".into(),
        };
        lines.push(Line { x: 18, y: 404, text: roost, rgb: (190, 190, 210) });
        if let Some(k) = world.get_component::<Known>(v) {
            for (n, r) in k.revealed.iter().take(2).enumerate() {
                let text = match r {
                    desires::Reveal::Want(w) => desires::want_label(*w).to_string(),
                    desires::Reveal::Problem(p) => desires::problem_label(p),
                };
                lines.push(Line { x: 18, y: 416 + n as i32 * 11, text, rgb: (210, 170, 140) });
            }
        }
        if let Some(vil) = world.get_component::<Villager>(v) {
            let nature = world.get_component::<AttachmentHistory>(v)
                .map(|a| nature_word(crate::sim::psychology::infer_style(a)))
                .unwrap_or("just folk");
            // where they stand with the cause: inside it, woken by it,
            // hollowed out by a blow-off, or taken for it
            let fight = crate::game::campaign::campaign_of(world, self.seats[seat].org)
                .and_then(|c| world.get_component::<crate::game::campaign::Campaign>(c))
                .map(|c| c.issue.clone());
            let cause = if world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some() {
                " \u{00b7} taken"
            } else if world.get_component::<crate::game::actions::Demoralized>(v).is_some() {
                " \u{00b7} hollow"
            } else if fight.as_ref().map(|issue|
                world.get_component::<crate::game::reactions::Sympathized>(v)
                    .map(|s| s.issues.iter().any(|(i, _)| i == issue)).unwrap_or(false))
                .unwrap_or(false) {
                " \u{00b7} woke"
            } else if org_members.contains(&v) { " \u{00b7} with you" } else { "" };
            // what they're about: the day's plan, unless hunger has them
            let bench = world.get_component::<Craft>(v).and_then(|c| c.workplace)
                .and_then(|b| world.get_component::<Building>(b)).map(|b| b.name.clone());
            let doing = (vil.activity == Activity::Idle)
                .then(|| routine::routine_text(world, v, bench.as_deref())).flatten()
                .unwrap_or_else(|| activity_text(vil.activity).to_string());
            lines.push(Line { x: 18, y: 440, text: format!("{} \u{00b7} {}{}", nature, doing, cause), rgb: (200, 160, 160) });
        }
        InspectorV { fig: Some(i), dead: false, died_day: None, lines }
    }

    /// One input from this seat: what a key press or click means to the sim.
    pub fn command_at(&mut self, seat: usize, cmd: Command) {
        match cmd {
            Command::Act(i) => self.act_at(seat, i),
            Command::CtxAct(i) => self.ctx_act_at(seat, i),
            Command::Walk(dx, dy) => { self.seats[seat].approach = None; self.seats[seat].picked = false; self.walk_at(seat, dx, dy); }
            Command::Select(i) => {
                // an index off the list is refused, not stored - a client
                // that sends one can't crash the next scene
                if i < self.villagers.len() {
                    self.seats[seat].selected_target = i;
                    self.seats[seat].picked = true;
                }
            }
            Command::ClickMap { mx, my, cam_x, cam_y, zoom } => self.click_map_at(seat, mx, my, cam_x, cam_y, zoom),
            Command::TogglePause => self.paused = !self.paused,
            Command::ToggleSpeed => self.speed = if self.speed == 1 { 4 } else { 1 },
            Command::CancelApproach => self.seats[seat].approach = None,
            Command::TalkPick(n) => self.talk_pick_at(seat, n),
            Command::Decide(i) => self.decide_at(seat, i),
        }
    }

    // ---------- multiplayer: seats, claims and the agent surface ----------

    /// A new wanderer joins at the market: the same spawn as the first
    /// player, with a class, a trade, a purse and an inner life.
    pub fn add_seat(&mut self) -> usize {
        let player = self.world.create_entity();
        let gold = livelihood::seed_gold(SocialClass::Working, false, self.rng.next() as f32 / u64::MAX as f32);
        self.world.add_component(player, PlayerCharacter {
            name: "A Wanderer".into(), influence: 0.1, authority: 0.0, resources: gold, standing: 0.0 });
        let mp = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x + 2.0, p.y + 1.0)).unwrap_or((10.0, 10.0));
        self.world.add_component(player, Pos { x: mp.0, y: mp.1 });
        self.world.add_component(player, vocation::craft(roll_vocation(&mut self.rng), 0.2));
        self.world.add_component(player, Satchel::default());
        self.world.add_component(player, Purse { gold });
        self.world.add_component(player, Organizer::default());
        let org = self.world.create_entity();
        self.world.add_component(org, Organization {
            name: "The Free Folk".into(), members: vec![], leader: player,
            influence_pool: 0.0, resources: 0.0, charter: "Newcomers".into() });
        self.seats.push(Seat { player, org, ..Seat::default() });
        self.seats.len() - 1
    }

    /// An API agent takes the reins of a villager: the seat's character is
    /// the villager entity itself; no PlayerCharacter, no org of their own.
    pub fn add_agent_seat(&mut self, villager: Entity) -> usize {
        let org = self.world.create_entity();
        self.world.add_component(org, Organization {
            name: "-".into(), members: vec![], leader: villager,
            influence_pool: 0.0, resources: 0.0, charter: "-".into() });
        self.seats.push(Seat { player: villager, org, agent: true, ..Seat::default() });
        self.seats.len() - 1
    }

    /// The first living villager nobody drives yet. A posting isn't up
    /// for the taking - a bot can claim a free hand, not the collector,
    /// the herb-wife, the knight or the priest (postings ride either
    /// TownRole or a PMC Craft); and a child is nobody's worker. The
    /// town's duties stay the town's.
    pub fn free_villager(&self) -> Option<usize> {
        // an agent needs a villager who can answer - the grave and the
        // lock-up keep their counsel
        self.villagers.iter().position(|v| !crate::game::dungeon::unavailable(&self.world, *v)
            && !self.seats.iter().any(|s| s.player == *v)
            && self.world.get_component::<TownRole>(*v).is_none()
            && self.world.get_component::<Kid>(*v).is_none()
            && !self.world.get_component::<Craft>(*v)
                .map(|c| vocation::is_pmc(c.vocation)).unwrap_or(false))
    }

    /// What an agent asks its villager to do. Maps to the same travel/goal
    /// machinery the village's own errands use, so nothing happens until the
    /// walking is done - arrival triggers the trade, the chat, the gather.
    pub fn agent_command(&mut self, seat: usize, cmd: AgentCmd) -> Result<(), String> {
        let who = self.seats.get(seat).map(|s| s.player).ok_or_else(|| "no such seat".to_string())?;
        // the held can only stand down - walks, errands and gathers all
        // pass the jailer first
        if (self.world.get_component::<crate::game::dungeon::Imprisoned>(who).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(who).is_some())
            && !matches!(cmd, AgentCmd::Idle) {
            return Err("held - the cell allows no errands".into());
        }
        match cmd {
            AgentCmd::Idle => { self.travels.retain(|t| t.who != who); Ok(()) }
            AgentCmd::GoTo { x, y } => {
                let Some(p) = self.world.get_component::<Pos>(who) else { return Err("you aren't in the world".into()) };
                let from = (p.x.round() as i32, p.y.round() as i32);
                match self.map.walk_path(from, (x, y)) {
                    Some(mut path) => {
                        path.push((x, y));
                        self.travels.retain(|t| t.who != who);
                        self.travels.push(Travel { who, path, goal: Goal::Wander });
                        Ok(())
                    }
                    None => Err("no way there".into()),
                }
            }
            AgentCmd::TalkTo { e } => {
                // a fresh command supersedes the walk underway
                self.travels.retain(|t| t.who != who);
                if self.send(who, Goal::TalkTo(e)) { Ok(()) } else { Err("no way to them".into()) }
            }
            AgentCmd::BuyFrom { seller, good, qty } => {
                let g: &'static str = Box::leak(good.into_boxed_str());
                self.travels.retain(|t| t.who != who);
                if self.send(who, Goal::BuyFrom(seller, g, qty)) { Ok(()) } else { Err("no way to them".into()) }
            }
            AgentCmd::BuyAt { shop, good } => {
                let g: &'static str = Box::leak(good.into_boxed_str());
                self.travels.retain(|t| t.who != who);
                if self.send(who, Goal::BuyAt(shop, g, None)) { Ok(()) } else { Err("no way to that shop".into()) }
            }
            AgentCmd::Gather { good, x, y } => {
                let g: &'static str = Box::leak(good.into_boxed_str());
                self.travels.retain(|t| t.who != who);
                if self.send_gather(who, g, (x, y)) { Ok(()) } else { Err("nothing to gather there".into()) }
            }
            AgentCmd::WorkDay => {
                let (made, up) = livelihood::work_day(&mut self.world, who, self.tick);
                if made <= 0.0 && up.is_none() { Err("no craft to work".into()) } else { Ok(()) }
            }
            AgentCmd::Eat => {
                // step to the market store; the hunger logic does the rest
                let Some(p) = self.world.get_component::<Pos>(self.market_food).map(|p| (p.x.round() as i32, p.y.round() as i32)) else {
                    return Err("no store".into()) };
                match self.map.walk_path(
                    self.world.get_component::<Pos>(who).map(|p| (p.x.round() as i32, p.y.round() as i32)).unwrap_or(p), p)
                {
                    // already standing at the store: eat now, no walk to take
                    Some(path) if path.is_empty() => { self.arrive(who, Goal::Eat); Ok(()) }
                    Some(path) => { self.travels.retain(|t| t.who != who); self.travels.push(Travel { who, path, goal: Goal::Eat }); Ok(()) }
                    None => Err("no way to the store".into()),
                }
            }
            AgentCmd::JoinOrg => {
                let oe = self.seats[seat].org;
                if self.world.get_component::<Organization>(oe)
                    .map(|o| o.members.contains(&who)).unwrap_or(false) { return Ok(()); }
                // you sign on when the cause has woken you, or the fight's
                // your own - either way the issue itself organizes you
                let issue = crate::game::campaign::campaign_of(&self.world, oe)
                    .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c))
                    .map(|c| c.issue.clone());
                let Some(issue) = issue else { return Err("no cause to sign on with".into()) };
                let woken = self.world.get_component::<crate::game::reactions::Sympathized>(who)
                    .map(|s| s.issues.iter().any(|(i, _)| *i == issue)).unwrap_or(false);
                let own = crate::game::campaign::stake_for(&self.world, who, &issue, self.tick).is_some();
                if !(woken || own) { return Err("the cause hasn't reached you - come to a meeting first".into()); }
                if let Some(o) = self.world.get_component_mut::<Organization>(oe) { o.members.push(who); }
                Ok(())
            }
            AgentCmd::Press { prisoner } => {
                if self.world.get_component::<crate::game::dungeon::Imprisoned>(prisoner).is_none() {
                    return Err("no one inside by that name".into());
                }
                // a letter home, a vigil, a name kept spoken
                crate::game::dungeon::press(&mut self.world, prisoner, 0.1);
                Ok(())
            }
        }
    }

    /// What an agent sees: itself, its purse, its belly, who's near and
    /// what's under way. Compact - the client decides between observations.
    pub fn agent_obs(&self, seat: usize) -> Option<AgentObs> {
        let who = self.seats.get(seat)?.player;
        let p = self.world.get_component::<Pos>(who)?;
        let gold = self.world.get_component::<Purse>(who).map(|x| x.gold).unwrap_or(0.0);
        let hunger = self.world.get_component::<Hunger>(who).map(|x| x.value).unwrap_or(100.0);
        let busy = if self.travels.iter().any(|t| t.who == who) {
            Some("travelling".to_string())
        } else {
            self.world.get_component::<Villager>(who).map(|v| format!("{:?}", v.activity))
        };
        let mut near: Vec<ObsPerson> = self.villagers.iter().enumerate()
            .filter_map(|(i, v)| {
                if *v == who { return None; }
                let q = self.world.get_component::<Pos>(*v)?;
                let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
                (d < 8.0).then(|| ObsPerson { e: *v, name: vill_name(i).to_string(), dist: d,
                    dead: villager::is_dead(&self.world, *v) })
            }).collect();
        for s in self.seats.iter().filter(|s| !s.agent) {
            if s.player == who { continue; }
            if let Some(q) = self.world.get_component::<Pos>(s.player) {
                let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
                if d < 8.0 { near.push(ObsPerson { e: s.player, name: "a wanderer".into(), dist: d, dead: false }); }
            }
        }
        let near_buildings: Vec<ObsBuilding> = self.building_entities.iter().filter_map(|be| {
            let q = self.world.get_component::<Pos>(*be)?;
            let d = ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
            if d >= 8.0 { return None; }
            let b = self.world.get_component::<Building>(*be)?;
            let kind = match b.kind {
                BuildingKind::Home => "home", BuildingKind::Shop(_) => "shop",
                BuildingKind::Church => "church", BuildingKind::WatchPost => "watch",
                BuildingKind::Mill => "mill", BuildingKind::Granary => "granary",
                BuildingKind::TraderStall => "stall", BuildingKind::Pasture => "pasture",
            BuildingKind::Apiary => "apiary",
            BuildingKind::Cellar => "cellar",
            };
            Some(ObsBuilding { e: *be, name: b.name.clone(), kind: kind.into(), dist: d })
        }).collect();
        let satchel = self.world.get_component::<Satchel>(who)
            .map(|s| s.goods.iter().map(|(g, q)| (g.clone(), *q)).collect()).unwrap_or_default();
        let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(who).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(who).is_some();
        // the political layer: every banner you march under (membership
        // or leadership - a villager can hold more than one), and what
        // each fights for. "-" marks a seat's placeholder org, not a cause
        let mut orgs: Vec<String> = vec![];
        let mut campaigns: Vec<String> = vec![];
        for e in self.world.query::<Organization>().iter() {
            let Some(o) = self.world.get_component::<Organization>(*e) else { continue };
            if o.name == "-" || !(o.members.contains(&who) || o.leader == who) { continue; }
            orgs.push(o.name.clone());
            if let Some(issue) = crate::game::campaign::campaign_of(&self.world, *e)
                .and_then(|c| self.world.get_component::<crate::game::campaign::Campaign>(c))
                .map(|c| c.issue.clone()) {
                campaigns.push(issue);
            }
        }
        // the fire on the wind: burning tiles close enough to matter
        let burning: Vec<(f32, f32)> = self.fire.burning.keys()
            .filter(|&&(x, y)| ((x as f32 - p.x).powi(2) + (y as f32 - p.y).powi(2)).sqrt() < 12.0)
            .map(|&(x, y)| (x as f32, y as f32)).collect();
        Some(AgentObs {
            tick: self.tick, you: who, pos: (p.x, p.y), gold, hunger, busy,
            near, near_buildings, satchel,
            walking: self.travels.iter().any(|t| t.who == who),
            held, orgs, campaigns, burning,
        })
    }

    // ---- seat-0 conveniences: the solo game and tests drive seat 0 ----
    pub fn act(&mut self, i: usize) { self.act_at(0, i) }
    pub fn ctx_act(&mut self, i: usize) { self.ctx_act_at(0, i) }
    pub fn walk(&mut self, dx: f32, dy: f32) { self.walk_at(0, dx, dy) }
    pub fn click_map(&mut self, mx: i32, my: i32, cam_x: i32, cam_y: i32, zoom: f32) { self.click_map_at(0, mx, my, cam_x, cam_y, zoom) }
    pub fn command(&mut self, cmd: Command) { self.command_at(0, cmd) }
    pub fn ui_model(&self) -> UiModel { self.ui_model_at(0) }
    pub fn action_blocker(&self, i: usize) -> Option<String> { self.action_blocker_at(0, i) }
    pub fn disabled_reason(&self, i: usize) -> Option<String> { self.disabled_reason_at(0, i) }
    pub fn villagers_in_reach(&self) -> Vec<Entity> { self.villagers_in_reach_at(0) }
    pub fn reach_target(&self) -> usize { self.reach_target_at(0) }
    pub fn player_pos(&self) -> (f32, f32) { self.player_pos_at(0) }
    pub fn update_ctx_actions(&mut self) { self.update_ctx_actions_at(0) }
    pub fn advance_approach(&mut self) { self.advance_approach_at(0) }
    pub fn call_meeting(&mut self) -> String { self.call_meeting_at(0) }
    pub fn talk_to(&mut self, i: usize) -> String { self.talk_to_at(0, i) }
    pub fn dist_to(&self, who: usize) -> f32 { self.dist_to_at(0, who) }
    pub fn path_to(&self, who: usize) -> Option<Vec<(i32, i32)>> { self.path_to_at(0, who) }
    pub fn start_approach(&mut self, who: usize, action: usize) -> Option<String> { self.start_approach_at(0, who, action) }
    pub fn build_site(&self) -> Option<(f32, f32)> { self.build_site_at(0) }


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

