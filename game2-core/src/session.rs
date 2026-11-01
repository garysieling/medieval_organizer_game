//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

pub mod model;
pub mod net;
pub mod driver;
pub mod record;
pub mod events;
pub mod probe;
pub mod stage;

pub use model::*;
pub use net::*;
pub use driver::*;
pub use record::*;

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

/// Draw `sprite` with its top-left at (x, y), scaled by `scale`
fn draw_zoomed(buf: &mut Buffer, sprite: &Sprite, x: i32, y: i32, scale: f32) {
    if (scale - 1.0).abs() < 0.01 {
        sprite.blit(buf, x, y);
    } else if (scale - 2.0).abs() < 0.01 {
        sprite.blit_scaled(buf, x, y, 2);
    } else {
        let (w, h) = ((sprite.width as f32 * scale).round().max(1.0) as usize, (sprite.height as f32 * scale).round().max(1.0) as usize);
        sprite.scaled(w, h).blit(buf, x, y);
    }
}

/// The dog's coat
const DOG: (u8, u8, u8) = (139, 90, 43);

/// A little dog, facing left, as pixel art: o outline, b coat, n eye/nose,
/// . transparent. Drawn 2x like the villagers.
const DOG_ART: [&str; 7] = [
    "..oo.......",
    ".obbo....o.",
    "obnbboooobo",
    ".obbbbbbbo.",
    "..obbbbbbo.",
    "..ob.o.obo.",
    "..oo....oo.",
];

fn dog_sprite() -> Sprite {
    let (w, h) = (DOG_ART[0].len(), DOG_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in DOG_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c { 'o' => (45, 28, 16), 'b' => DOG, 'n' => (15, 10, 8), _ => continue };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// The cat's coat (a grey tabby)
const CAT: (u8, u8, u8) = (95, 95, 105);

/// A little cat, facing right, sitting with its tail curled round: o
/// outline, b coat, n eye/nose, . transparent. Drawn 2x like the dog.
const CAT_ART: [&str; 7] = [
    "..o.....o..",
    ".obo...obo.",
    ".obbbnbbo..",
    "oobbbbbbo..",
    "obbbbbbbo..",
    "ob..bb..o..",
    ".o..oo.....",
];

fn cat_sprite() -> Sprite {
    let (w, h) = (CAT_ART[0].len(), CAT_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in CAT_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c { 'o' => (40, 38, 45), 'b' => CAT, 'n' => (15, 10, 8), _ => continue };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// A hen scratching round the farmyard, facing left: o outline, w white
/// feathers, r comb, n eye/beak, l legs, . transparent
const CHICKEN_ART: [&str; 7] = [
    "....oo.....",
    "...orro....",
    "..owwno....",
    ".owwwwwo...",
    ".owwwwoo...",
    "..l..l.....",
    "..l..l.....",
];

fn chicken_sprite() -> Sprite {
    let (w, h) = (CHICKEN_ART[0].len(), CHICKEN_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in CHICKEN_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c {
                'o' => (60, 55, 50), 'w' => (235, 230, 220), 'r' => (200, 60, 50),
                'n' => (15, 10, 8), 'l' => (200, 150, 60), _ => continue,
            };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// A gull on the wing: just two white strokes and a grey under-shadow
const GULL_ART: [&str; 3] = [
    "w.....w",
    ".w...w.",
    "..g.g..",
];

fn gull_sprite() -> Sprite {
    let (w, h) = (GULL_ART[0].len(), GULL_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in GULL_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c { 'w' => (245, 248, 250), 'g' => (170, 180, 195), _ => continue };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// Where the panel portrait's feet are: a 32px frame drawn 2x from
/// (UI_X+6, 312), the figure centred in it and ending at its bottom row
const PORTRAIT_FOOT: (i32, i32) = (UI_X as i32 + 6 + 32, 322 + 62);

/// Villager name colours on the map: within MEETING_REACH of the player
/// (Talk, Recruit and Organize can reach them) or not
const NAME_IN_REACH: (u8, u8, u8) = (255, 255, 255);
const NAME_OUT_OF_REACH: (u8, u8, u8) = (150, 150, 165);

/// Gravestone colours
const GRAVE_STONE: (u8, u8, u8) = (135, 135, 145);
const GRAVE_EDGE: (u8, u8, u8) = (55, 55, 65);

/// A ready herb patch: a little spray of flowers low on the meadow tile
fn herb_tuft(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    // green sprigs, then petal dots above them
    for (dx, dy) in [(-3, 0), (0, -1), (3, 0), (-1, -2), (1, -2)] {
        buf.rect(sx + k(dx), sy + k(dy), 1.max(k(1)), 1.max(k(1)), 70, 145, 75);
    }
    for (dx, dy, c) in [(-3, -3, (240, 225, 245)), (1, -4, (205, 165, 235)), (3, -2, (245, 240, 200))] {
        buf.rect(sx + k(dx), sy + k(dy), 1.max(k(1)), 1.max(k(1)), c.0, c.1, c.2);
    }
}

/// A wild mushroom: a little cap on a pale stem
fn shroom_tuft(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    buf.rect(sx - k(2), sy - k(4), 1.max(k(5)), 1.max(k(3)), 195, 70, 55);
    buf.rect(sx - k(1), sy - k(1), 1.max(k(3)), 1.max(k(2)), 235, 225, 200);
}

/// A woven skep: a small domed hive with banding
fn draw_skep(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    buf.rect(sx - k(4), sy - k(7), 1.max(k(8)), 1.max(k(6)), 210, 185, 120);
    buf.rect(sx - k(3), sy - k(9), 1.max(k(6)), 1.max(k(2)), 220, 198, 140);
    for row in [4, 6] { buf.rect(sx - k(4), sy - k(row), 1.max(k(8)), 1.max(k(1)), 160, 135, 85); }
}

/// A swarm on the wing: a knot of dark specks
fn draw_swarm(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    for (dx, dy) in [(-3, -8), (0, -10), (3, -8), (-1, -6), (2, -11), (-4, -10)] {
        buf.rect(sx + k(dx), sy + k(dy), 1.max(k(2)), 1.max(k(1)), 45, 35, 20);
    }
}

/// A grazing animal: a wool or hair blob low on four legs
fn draw_beast(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32, sheep: bool) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    let (body, dark) = if sheep { ((235, 230, 212), (205, 198, 180)) } else { ((148, 132, 110), (104, 92, 75)) };
    buf.rect(sx - k(5), sy - k(8), 2.max(k(10)), 1.max(k(5)), body.0, body.1, body.2);
    buf.rect(sx + k(4), sy - k(9), 1.max(k(3)), 1.max(k(4)), dark.0, dark.1, dark.2);
    for lx in [-3, -1, 1, 3] { buf.rect(sx + k(lx), sy - k(3), 1.max(k(1)), 1.max(k(3)), dark.0, dark.1, dark.2); }
}

/// Draw a gravestone standing on screen point (sx, sy) at the current zoom;
/// returns the y of its top, for the name label
fn draw_grave(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) -> i32 {
    let z = |v: f32| (v * zoom).round() as i32;
    let (w, h) = (z(12.0).max(4), z(16.0).max(5));
    let (x, top) = (sx - w / 2, sy - h);
    // outline, then the stone with a rounded head, then a carved cross
    buf.rect(x - 1, top + 2, w + 2, h - 1, GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    buf.rect(x + 1, top, w - 2, 3, GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    buf.rect(x, top + 2, w, h - 2, GRAVE_STONE.0, GRAVE_STONE.1, GRAVE_STONE.2);
    buf.rect(x + 2, top + 1, w - 4, 2, GRAVE_STONE.0, GRAVE_STONE.1, GRAVE_STONE.2);
    buf.rect(sx - 1, top + z(3.0), 2, z(8.0).max(3), GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    buf.rect(sx - z(3.0), top + z(5.0), z(6.0).max(2), 2, GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    top
}

/// Draw a character sprite (a 32px frame drawn 2x at zoom 1) standing on
/// screen point (sx, sy), at the current zoom
fn draw_character(buf: &mut Buffer, sprite: &Sprite, sx: i32, sy: i32, zoom: f32) {
    draw_zoomed(buf, sprite, sx - (32.0 * zoom).round() as i32, sy - (60.0 * zoom).round() as i32, 2.0 * zoom);
}

/// Screen y of a character's head at the current zoom
fn head_y(sy: i32, zoom: f32) -> i32 { sy + (SPRITE_HEAD as f32 * zoom).round() as i32 }

/// The point to hand name_label_pos so a label sits just above a zoomed head
fn label_anchor(sy: i32, zoom: f32) -> i32 { head_y(sy, zoom) - SPRITE_HEAD }

/// How far below SPRITE_HEAD this sprite's head actually starts, in screen
/// pixels at zoom 1: some figures (the old man) have blank rows above
/// row 18, which left their name floating
fn head_drop(sprite: &Sprite) -> i32 {
    let w = sprite.width as usize;
    let top = (0..sprite.height as usize)
        .find(|&y| (0..w).any(|x| sprite.pixels[(y * w + x) * 4 + 3] > 0))
        .unwrap_or(18) as i32;
    (top - 18).max(0) * 2
}

/// label_anchor for a particular sprite, so its name sits on its own head
fn sprite_label_anchor(sprite: &Sprite, sy: i32, zoom: f32) -> i32 {
    label_anchor(sy, zoom) + (head_drop(sprite) as f32 * zoom).round() as i32
}

/// Height of a speech bubble
const BUBBLE_H: i32 = 18;

/// Where a speech bubble goes for a speaker standing on screen point
/// (sx, sy): just above their (zoomed) name label, kept inside the map view.
/// None when the speaker isn't on the map view at all.
/// `label_y` is where the speaker's name label actually ended up: place_labels
/// nudges crowded labels upward, and the bubble must clear the nudged one
fn bubble_rect(sx: i32, sy: i32, zoom: f32, text: &str, label_y: Option<i32>) -> Option<(i32, i32, i32)> {
    if sx < 0 || sx >= MAP_W as i32 || sy < 0 || sy >= H as i32 + 60 { return None; }
    let bw = (text.len() as i32 * GLYPH_W + 12).max(60).min(MAP_W as i32 - 8);
    let bx = (sx - bw / 2).clamp(4, MAP_W as i32 - 4 - bw);
    let label_top = (head_y(sy, zoom) - GLYPH_H - LABEL_GAP).min(label_y.map(|y| y - 4).unwrap_or(i32::MAX));
    let by = (label_top - BUBBLE_H - 8).max(4);
    Some((bx, by, bw))
}

/// Top of a character's head relative to its ground point. Characters are
/// 32px frames drawn 2x from (sx-32, sy-60), but the figure only fills the
/// frame's bottom rows (18-31), so the head is at about sy-24.
const SPRITE_HEAD: i32 = -24;
/// Font glyph width and height (ui::font)
const GLYPH_W: i32 = 6;
const GLYPH_H: i32 = 7;
/// Space between a name label and the head below it (the text outline
/// adds one more pixel)
const LABEL_GAP: i32 = 1;

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

fn name_label_pos(sx: i32, sy: i32, text: &str) -> (i32, i32) {
    (sx - text.len() as i32 * GLYPH_W / 2, sy + SPRITE_HEAD - GLYPH_H - LABEL_GAP)
}

/// Word-wrap `text` into lines of at most `max` characters, breaking at
/// spaces (a word longer than a whole line is split)
fn wrap_text(text: &str, max: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut line = String::new();
    for word in text.split_whitespace() {
        let mut word = word.to_string();
        while word.chars().count() > max {
            if !line.is_empty() { lines.push(std::mem::take(&mut line)); }
            let head: String = word.chars().take(max).collect();
            word = word.chars().skip(max).collect();
            lines.push(head);
        }
        if line.is_empty() {
            line = word;
        } else if line.chars().count() + 1 + word.chars().count() <= max {
            line.push(' ');
            line.push_str(&word);
        } else {
            lines.push(std::mem::replace(&mut line, word));
        }
    }
    if !line.is_empty() { lines.push(line); }
    lines
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

/// Base colour of a terrain tile by height - shared by the map and minimap
/// so they agree on what is water, grass, fields, hills and peaks
fn terrain_colour(h: f32) -> (u8, u8, u8) {
    if h < 0.15 { (30, 60, 180) }
    else if h < 0.35 { (60, 140, 60) }
    else if h < 0.55 { (140, 110, 50) }
    else if h < 0.75 { (110, 90, 60) }
    else { (220, 220, 220) }
}

/// A pixel under rain: dimmed and cooled, like light through rain cloud
fn rain_wash(c: (u8, u8, u8)) -> (u8, u8, u8) {
    blend(c, (40, 55, 80), 0.25)
}

/// Mix colour `a` toward `b` by `t` (0 = all a, 1 = all b)
fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

/// A tile's colour in the current season: snow lies on the land in winter
/// and the water turns cold, grass goes gold in autumn and fresh in spring,
/// summer is the base palette. Shared by the map and the minimap.
fn season_colour(h: f32, season: Season) -> (u8, u8, u8) {
    let base = terrain_colour(h);
    let water = h < 0.15;
    let peak = h >= 0.75;
    match season {
        Season::Summer => base,
        Season::Spring if water || peak => base,
        Season::Spring => blend(base, (70, 170, 70), 0.25),
        Season::Autumn if water || peak => base,
        Season::Autumn => blend(base, (185, 125, 45), 0.45),
        Season::Winter if water => blend(base, (60, 80, 120), 0.5),
        Season::Winter => blend(base, (228, 232, 240), 0.65),
    }
}

/// A map tile's colour in a season: the minimap's, and the map's own when
/// a tile's block art is missing. Open ground follows season_colour.
fn tile_colour(tile: Tile, season: Season) -> (u8, u8, u8) {
    let land = |c: (u8, u8, u8)| match season {
        Season::Winter => blend(c, (228, 232, 240), 0.65),
        _ => c,
    };
    match tile {
        Tile::Water | Tile::River => season_colour(0.05, season),
        Tile::Grass => season_colour(0.25, season),
        Tile::Hill => season_colour(0.45, season),
        Tile::Rock => season_colour(0.65, season),
        Tile::Wheat => land((220, 185, 80)),
        Tile::Corn => land((200, 160, 60)),
        Tile::Road => land((229, 143, 77)),
        Tile::Bridge => (229, 143, 77),
    }
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

/// Minimap dot colours: living villagers, and the graves of the dead
const MM_VILLAGER: (u8, u8, u8) = (255, 80, 80);
const MM_GRAVE: (u8, u8, u8) = (150, 150, 160);

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

/// Minimap position of a point on screen (inverse of iso_to_screen_cam,
/// then tile_to_minimap) - used to outline what the screen is showing
fn screen_to_minimap(sx: f32, sy: f32, cam_x: i32, cam_y: i32, zoom: f32, w_t: f32, h_t: f32) -> (f32, f32) {
    // undo the zoom about the view centre, then the zoom-1 projection
    let bx = ZOOM_PIVOT.0 + (sx - ZOOM_PIVOT.0) / zoom;
    let by = ZOOM_PIVOT.1 + (sy - ZOOM_PIVOT.1) / zoom;
    let diff = (bx - MAP_W as f32 / 2.0 - cam_x as f32) / TILE_W as f32; // tx - ty
    let sum = (by + 60.0 - cam_y as f32) / TILE_H as f32;                 // tx + ty
    tile_to_minimap((sum + diff) / 2.0, (sum - diff) / 2.0, mm_k(w_t, h_t))
}

/// Nudge name labels so none overlap: each label (x, y, width) keeps its x
/// and, if it would sit on a label already placed, moves up a line at a time
/// (at most a few lines) until it's clear. Labels nearer the bottom of the
/// screen (characters in front) are placed first and keep their spot.
fn place_labels(labels: &[(i32, i32, i32)]) -> Vec<i32> {
    const LINE: i32 = GLYPH_H + 3;
    let mut order: Vec<usize> = (0..labels.len()).collect();
    order.sort_by_key(|&i| -labels[i].1);
    let mut placed: Vec<(i32, i32, i32)> = vec![];
    let mut ys = vec![0; labels.len()];
    for i in order {
        let (x, mut y, w) = labels[i];
        let hits = |y: i32, placed: &[(i32, i32, i32)]| placed.iter().any(|&(px, py, pw)| {
            x < px + pw + 2 && px < x + w + 2 && y < py + LINE && py < y + LINE
        });
        for _ in 0..4 {
            if !hits(y, &placed) { break; }
            y -= LINE;
        }
        placed.push((x, y, w));
        ys[i] = y;
    }
    ys
}

/// Marks drawn by villagers on the map: (glyph, colour, meaning). The
/// panel's legend lists them, so the map and the key can't drift apart.
const MARK_HUNGRY: (&str, (u8, u8, u8), &str) = ("!", (255, 100, 100), "hungry");
const MARK_FOOD: (&str, (u8, u8, u8), &str) = ("o", (255, 200, 100), "finding food");
const MARK_BUSY: (&str, (u8, u8, u8), &str) = ("*", (180, 180, 255), "busy");
const MARK_SICK: (&str, (u8, u8, u8), &str) = ("+", (140, 230, 110), "sick");
const MAP_MARKS: [(&str, (u8, u8, u8), &str); 4] = [MARK_HUNGRY, MARK_FOOD, MARK_BUSY, MARK_SICK];
/// The legend's row, between the log box and the bottom of the window
const LEGEND_Y: i32 = 627;

/// Whether the crowd-separation pass may push this villager aside: not the
/// dead, and not someone on their way to food
fn makes_room(world: &World, v: Entity) -> bool {
    !villager::is_dead(world, v)
        && world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none()
        && world.get_component::<Villager>(v).map(|x| x.activity != Activity::SeekingFood).unwrap_or(true)
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

/// Green when high, amber in the middle, red when low (0..=1)
fn meter_colour(frac: f32) -> (u8, u8, u8) {
    if frac >= 0.6 { (90, 200, 90) } else if frac >= 0.3 { (230, 180, 60) } else { (230, 80, 70) }
}

/// A small labelled meter: label, bar filled to `frac`, then a note
fn draw_meter(buf: &mut Buffer, x: i32, y: i32, label: &str, frac: f32, note: &str) {
    const BAR_X: i32 = 44;
    const BAR_W: i32 = 70;
    draw_text(buf, x, y, label, 200, 200, 200);
    buf.rect(x + BAR_X, y, BAR_W, 6, 30, 30, 45);
    let (r, g, b) = meter_colour(frac);
    buf.rect(x + BAR_X, y, (frac.clamp(0.0, 1.0) * BAR_W as f32) as i32, 6, r, g, b);
    draw_text(buf, x + BAR_X + BAR_W + 6, y, note, r, g, b);
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

fn draw_button(buf: &mut Buffer, b: &Button, hover: bool, disabled: bool) {
    let (mut r, mut g, mut bl) = b.color;
    if disabled { r = 50; g = 50; bl = 55; }
    else if hover { r = r.saturating_add(40); g = g.saturating_add(40); bl = bl.saturating_add(40); }
    draw_rect(buf, b.x, b.y, b.w, b.h, r, g, bl);
    draw_rect(buf, b.x, b.y, b.w, 1, 255, 255, 255);
    draw_rect(buf, b.x, b.y + b.h - 1, b.w, 1, 20, 20, 20);
    // label centered
    let tw = b.label.len() as i32 * 6;
    draw_text(buf, b.x + (b.w - tw) / 2, b.y + (b.h - 7) / 2, b.label, 255, 255, 255);
    // key hint
    let key = format!("{}", b.key);
    draw_text(buf, b.x + b.w - 10, b.y + 2, &key, 180, 180, 200);
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

/// Every sprite the game draws, loaded once up front
#[derive(Default)]
pub struct Art {
    pub villager_sprites: Vec<Sprite>,
    /// The rivals' figures: the same pack's outlined look, so they read foreign
    pub rival_sprites: Vec<Sprite>,
    /// The cottages a home can be: colour, timber or plaster, and the door
    /// and gable in different spots (the pack's numbered variants)
    pub houses: Vec<Option<Sprite>>,
    pub mill_frames: Vec<Sprite>, // the sails only: they turn on mill_tower
    pub mill_tower: Option<Sprite>,
    pub blacksmith_s: Option<Sprite>,
    pub stones: Vec<Sprite>,        // boulders, a few shapes
    pub stone_grounds: Vec<Sprite>, // flat rocks
    pub player_s: Option<Sprite>,
    pub tower_s: Option<Sprite>,
    /// Town wall blocks: running along x, along y, and the two gatehouses
    pub walls: [Option<Sprite>; 4],
    /// The odd wall stretch with a postern door in it: along x, along y
    pub wall_posts: [Option<Sprite>; 2],
    pub church_s: Option<Sprite>,
    pub guildhall_s: Option<Sprite>,
    pub rival_house: Option<Sprite>,
    pub rival_woodhouse: Option<Sprite>,
    /// The watch at the town gates, from the humans pack
    pub guards: Vec<Sprite>,
    pub priest_s: Option<Sprite>,
    pub traveller_s: Option<Sprite>, // a courier riding the road between villages
    pub dog: Option<Sprite>,   // hand-drawn: the asset packs have no animals
    pub cat: Option<Sprite>,
    pub chicken: Option<Sprite>,
    pub gull: Option<Sprite>,
    /// Ground specks scattered on open grass, per season set
    pub details: [Vec<Sprite>; 3],
    /// Ground blocks for the green seasons, autumn and winter (season_tiles),
    /// and the same as top faces only, for inland tiles
    pub tiles: [TileSet; 3],
    pub faces: [TileSet; 3],
}

/// The pack's ground blocks for one season's look: one per kind of map
/// tile, and every numbered road and river piece, in the pack's order
#[derive(Default)]
pub struct TileSet {
    pub grass: Option<Sprite>,
    /// Sandy grass where the sea meets the land, and drier bare looks up
    /// on the hills and the bare tops
    pub shore: Option<Sprite>,
    pub hill: Option<Sprite>,
    pub rock: Option<Sprite>,
    pub water: Option<Sprite>,
    /// Pale water round the shore, darker out at sea
    pub shallow: Option<Sprite>,
    pub deep: Option<Sprite>,
    pub wheat: Vec<Sprite>,
    pub corn: Vec<Sprite>,
    pub roads: Vec<Sprite>,
    pub rivers: Vec<Sprite>,
    /// Winter only: iced-over river stretches
    pub rivers_ice: Vec<Sprite>,
    pub trees: Vec<Sprite>,
}

/// Which TileSet a season draws with
fn season_tiles(season: Season) -> usize {
    match season { Season::Autumn => 1, Season::Winter => 2, _ => 0 }
}

/// `block` with its greens recoloured by `f` (grass to water, or autumn gold)
fn recolour_greens(block: &Sprite, f: impl Fn((u8, u8, u8)) -> (u8, u8, u8)) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    for p in out.pixels.chunks_mut(4) {
        if p[3] > 0 && p[1] > p[0] && p[1] > p[2] {
            let (r, g, b) = f((p[0], p[1], p[2]));
            p[0] = r; p[1] = g; p[2] = b;
        }
    }
    out
}

/// The top face of the pack's 94x97 ground blocks: a diamond centred on
/// (47, 26.5), 47 across and 26.5 down from its centre to its corners
const FACE_CENTRE: (f32, f32, f32, f32) = (47.0, 26.5, 47.0, 26.5);
/// How far the top face's outline runs either side of its edge, as a
/// fraction of the way to the centre (measured on the grass block: 0.76
/// to 1.12 - it's thickest at the corners)
const FACE_OUTLINE: (f32, f32) = (0.25, 0.15);

/// `block` with the dark outline round its top face (and its soft,
/// half-dark edge) painted over in the colour just inside it
fn strip_face_outline(block: &Sprite) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    let (cx, cy, rx, ry) = FACE_CENTRE;
    let px = |x: i32, y: i32| -> [u8; 4] {
        let i = (y as usize * block.width + x as usize) * 4;
        [block.pixels[i], block.pixels[i + 1], block.pixels[i + 2], block.pixels[i + 3]]
    };
    let lum = |p: [u8; 4]| p[0] as u32 + p[1] as u32 + p[2] as u32;
    for y in 0..block.height.min(60) as i32 {
        for x in 0..block.width as i32 {
            let d = ((x as f32 + 0.5 - cx) / rx).abs() + ((y as f32 + 0.5 - cy) / ry).abs();
            let here = px(x, y);
            if d < 1.0 - FACE_OUTLINE.0 || d > 1.0 + FACE_OUTLINE.1 || here[3] == 0 { continue; }
            // the face's own colour, just inside the outline on the way to
            // the centre (the distance shrinks evenly along that line)
            let t = 1.0 - (1.0 - FACE_OUTLINE.0 - 0.04) / d;
            let (ix, iy) = ((x as f32 + (cx - x as f32) * t) as i32, (y as f32 + (cy - y as f32) * t) as i32);
            let fill = px(ix, iy);
            if fill[3] > 0 && lum(here) * 100 < lum(fill) * 85 {
                let i = (y as usize * block.width + x as usize) * 4;
                out.pixels[i..i + 4].copy_from_slice(&fill);
            }
        }
    }
    out
}

/// Just the top face of a ground block, its sides cut away: inland, where
/// the blocks in front cover the sides anyway, their edges only peeked
/// between the tiles as faint lines
fn face_only(block: &Sprite) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    let (cx, cy, rx, ry) = FACE_CENTRE;
    for y in 0..block.height {
        for x in 0..block.width {
            let d = ((x as f32 + 0.5 - cx) / rx).abs() + ((y as f32 + 0.5 - cy) / ry).abs();
            if d > 1.0 + FACE_OUTLINE.1 / 2.0 { out.pixels[(y * block.width + x) * 4 + 3] = 0; }
        }
    }
    out
}

/// The sea: the pack has no open-water block, so it's the grass block with
/// its grass turned to water, colder in winter. `shade` scales the blue
/// per channel: shallow water is paler, the open sea darker.
fn water_block(grass: &Sprite, winter: bool, shade: (f32, f32, f32)) -> Sprite {
    let k = if winter { 0.8 } else { 1.0 };
    recolour_greens(grass, |(_, g, _)| {
        let l = g as f32 * k;
        ((l * shade.0) as u8, (l * shade.1) as u8, (l * shade.2).min(255.0) as u8)
    })
}

/// Blend every drawn pixel of `block` toward `colour` by `t` (where
/// recolour_greens only touches the greens - snow has none to take)
fn tint(block: &Sprite, colour: (u8, u8, u8), t: f32) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    for p in out.pixels.chunks_mut(4) {
        if p[3] == 0 { continue; }
        let (r, g, b) = blend((p[0], p[1], p[2]), colour, t);
        p[0] = r; p[1] = g; p[2] = b;
    }
    out
}

/// A per-tile pick from a sprite list, stable for the tile (`salt` keeps
/// different lists from lining up on the same tiles)
fn pick(v: &[Sprite], x: i32, y: i32, salt: u32) -> Option<&Sprite> {
    if v.is_empty() { None } else { v.get((noise(x, y, salt) * v.len() as f32) as usize % v.len()) }
}

impl TileSet {
    /// Load the blocks for the green seasons (0), autumn (1: the green set
    /// gone gold) or winter (2)
    fn load(at: &dyn Fn(&str) -> Option<Sprite>, season: usize) -> TileSet {
        let load = |rel: &str| at(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/{}", rel));
        let winter = season == 2;
        let gold = |s: Sprite| if season == 1 { recolour_greens(&s, |c| blend(c, (185, 125, 45), 0.45)) } else { s };
        let pieces = |dir: &str, name: &str, n: usize| -> Vec<Sprite> {
            (1..=n).filter_map(|i| load(&format!("{}/{}({}).png", dir, name, i))).map(gold).collect()
        };
        let grass = load("Enviroument/Spring/grass.png");
        // fields lie under the snow in winter
        let snow = || load("Enviroument/Winter/snow.png");
        let water = |shade: (f32, f32, f32)| grass.as_ref().map(|g| water_block(g, winter, shade));
        let (water, shallow, deep) = (water((0.55, 0.8, 1.2)), water((0.8, 1.0, 1.35)), water((0.35, 0.55, 1.0)));
        let ground = if winter { snow() } else { grass.map(gold) };
        // the look of bare land this season: a sandy rim where the sea
        // meets it, parched hills and grey tops (snow has no greens to
        // recolour, so winter tints the whole block instead)
        let (shore, hill, rock) = if winter {
            (ground.as_ref().map(|s| tint(s, (200, 210, 230), 0.3)),
             ground.as_ref().map(|s| tint(s, (215, 195, 165), 0.3)),
             ground.as_ref().map(|s| tint(s, (155, 160, 175), 0.35)))
        } else {
            (ground.as_ref().map(|s| recolour_greens(s, |c| blend(c, (224, 204, 150), 0.55))),
             ground.as_ref().map(|s| recolour_greens(s, |c| blend(c, (170, 140, 85), 0.45))),
             ground.as_ref().map(|s| recolour_greens(s, |c| blend(c, (150, 145, 140), 0.55))))
        };
        // both growth stages of each crop, so a field isn't one plant repeated
        let crop = |name: &str| -> Vec<Sprite> {
            (1..=2).filter_map(|i| load(&format!("Buildings/{}/{}({}).png", name, name, i))).map(&gold).collect()
        };
        let set = TileSet {
            shore, hill, rock, water, shallow, deep,
            grass: ground,
            trees: (1..=4).filter_map(|i|
                load(&format!("Enviroument/{}/trees/tree({}).png", if winter { "Winter" } else { "Spring" }, i)).map(&gold)).collect(),
            wheat: if winter { snow().into_iter().collect() } else { crop("wheat") },
            corn: if winter { snow().into_iter().collect() } else { crop("corn") },
            roads: if winter {
                // the winter set numbers its two straight roads the other way round
                let mut r = pieces("Roads/Winter", "road_winter", 15);
                if r.len() > 2 { r.swap(1, 2); }
                r
            } else { pieces("Roads/Spring", "road", 15) },
            rivers: if winter { pieces("Rivers/Winter", "river_winter", 17) } else { pieces("Rivers/Spring", "river", 17) },
            rivers_ice: if winter { pieces("Rivers/Winter/Ice", "river_ice_winter", 17) } else { vec![] },
        };
        // without the dark outline round their top faces, so the ground
        // reads as one surface rather than a grid of tiles (lines inside a
        // tile, like a road's or a riverbank's edges, stay)
        set.map_blocks(strip_face_outline)
    }

    /// Every ground block put through `f` (the trees stay as they are)
    fn map_blocks(&self, f: impl Fn(&Sprite) -> Sprite) -> TileSet {
        TileSet {
            grass: self.grass.as_ref().map(&f),
            shore: self.shore.as_ref().map(&f),
            hill: self.hill.as_ref().map(&f),
            rock: self.rock.as_ref().map(&f),
            water: self.water.as_ref().map(&f),
            shallow: self.shallow.as_ref().map(&f),
            deep: self.deep.as_ref().map(&f),
            wheat: self.wheat.iter().map(&f).collect(),
            corn: self.corn.iter().map(&f).collect(),
            roads: self.roads.iter().map(&f).collect(),
            rivers: self.rivers.iter().map(&f).collect(),
            rivers_ice: self.rivers_ice.iter().map(&f).collect(),
            trees: self.trees.clone(),
        }
    }

    /// The block for map tile (x, y), joined up with its neighbours
    fn block(&self, map: &TileMap, x: i32, y: i32) -> Option<&Sprite> {
        let sea = |dx: i32, dy: i32| map.get(x + dx, y + dy) == Some(Tile::Water);
        let by_sea = || NEIGHBOURS.iter().any(|&(dx, dy)| sea(dx, dy));
        let by_land = || NEIGHBOURS.iter().any(|&(dx, dy)| !sea(dx, dy) && map.get(x + dx, y + dy) != Some(Tile::River));
        match map.get(x, y)? {
            // the shore wears sand, the hills go parched, the tops bare (the
            // pack's dirt block is orange and read as a stray bit of road)
            Tile::Grass if by_sea() => self.shore.as_ref().or(self.grass.as_ref()),
            Tile::Grass => self.grass.as_ref(),
            Tile::Hill => self.hill.as_ref().or(self.grass.as_ref()),
            Tile::Rock => self.rock.as_ref().or(self.hill.as_ref()).or(self.grass.as_ref()),
            Tile::Water if by_land() => self.shallow.as_ref().or(self.water.as_ref()),
            Tile::Water if noise(x, y, 9) < 0.35 => self.deep.as_ref().or(self.water.as_ref()),
            Tile::Water => self.water.as_ref(),
            Tile::Wheat => pick(&self.wheat, x, y, 5),
            Tile::Corn => pick(&self.corn, x, y, 6),
            Tile::Road => self.roads.get(map.piece(x, y)? as usize - 1),
            Tile::River => {
                let p = map.piece(x, y)? as usize - 1;
                // in winter stretches of the river freeze right over
                if noise(x, y, 13) < 0.45 { self.rivers_ice.get(p).or(self.rivers.get(p)) } else { self.rivers.get(p) }
            }
            Tile::Bridge => self.rivers.get(map.piece(x, y)? as usize - 1),
        }
    }
}

/// The pack's ground blocks are 94px wide; the top face inside its outline
/// is about 91x50, starting 1.5px down. A block is scaled so that face is
/// exactly one map tile (neighbours' outlines overlap, as in the pack).
const BLOCK_FACE: (f32, f32, f32) = (91.0, 50.0, 1.5);

/// Size and top-left of the block drawn for the tile centred on (sx, sy)
fn block_rect(block: &Sprite, sx: i32, sy: i32, zoom: f32) -> (usize, usize, i32, i32) {
    let kx = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
    let ky = 2.0 * TILE_H as f32 * zoom / BLOCK_FACE.1;
    let (w, h) = ((block.width as f32 * kx).round() as usize, (block.height as f32 * ky).round() as usize);
    (w, h, sx - w as i32 / 2, sy - (TILE_H as f32 * zoom + BLOCK_FACE.2 * ky).round() as i32)
}

/// Draw a building standing on the tile centred on (sx, sy): scaled so its
/// base spans exactly one tile, its front corner on the tile's front corner
fn draw_on_tile(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32) {
    let w = (2.0 * TILE_W as f32 * zoom).round() as usize;
    let h = (s.height as f32 * w as f32 / s.width as f32).round().max(1.0) as usize;
    s.scaled(w, h).blit(buf, sx - w as i32 / 2, sy + (TILE_H as f32 * zoom).round() as i32 - h as i32);
}

/// Draw a piece of the town wall standing on the tile centred on (sx, sy):
/// scaled like the ground blocks (the pack draws its castle pieces for the
/// same 91x50 block face), so the piece's footprint is exactly the tile face
/// and its base lands on the tile's edges instead of overhanging them
fn draw_castle(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32) {
    let kx = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
    let ky = 2.0 * TILE_H as f32 * zoom / BLOCK_FACE.1;
    let (w, h) = ((s.width as f32 * kx).round() as usize, (s.height as f32 * ky).round().max(1.0) as usize);
    s.scaled(w, h).blit(buf, sx - w as i32 / 2, sy + (TILE_H as f32 * zoom).round() as i32 - h as i32);
}

/// Where a windmill's sails turn: on the front of its tower, just below the
/// roof (MILL_HUB, in the tower sprite's pixels), for a tower drawn with
/// draw_on_tile on the tile centred on (sx, sy)
fn mill_hub(tower: &Sprite, sx: i32, sy: i32, zoom: f32) -> (i32, i32) {
    let k = 2.0 * TILE_W as f32 * zoom / tower.width as f32;
    let (w, h) = ((2.0 * TILE_W as f32 * zoom).round() as i32, (tower.height as f32 * k).round() as i32);
    let top = sy + (TILE_H as f32 * zoom).round() as i32 - h;
    (sx - w / 2 + (MILL_HUB.0 * k).round() as i32, top + (MILL_HUB.1 * k).round() as i32)
}
const MILL_HUB: (f32, f32) = (24.5, 44.0);
/// The sails' size against the tower's (the pack draws them about as tall)
const MILL_SAILS: f32 = 0.7;

/// The puffs of smoke over a home on the tile centred on (sx, sy), at frame
/// `frame`: (x, y, size, grey) for each, rising from the roof over
/// SMOKE_SECS, drifting right and shrinking as they go
fn smoke_puffs(frame: u32, sx: i32, sy: i32, zoom: f32) -> Vec<(i32, i32, i32, u8)> {
    let period = SMOKE_SECS * FPS;
    (0..3).map(|k| {
        let t = ((frame + k * period / 3) % period) as f32 / period as f32; // 0 at the chimney, 1 gone
        let x = sx as f32 + (10.0 + 8.0 * t) * zoom;
        let y = sy as f32 - (34.0 + 26.0 * t) * zoom;
        let size = ((4.0 - 2.5 * t) * zoom).round().max(1.0) as i32;
        (x as i32, y as i32, size, (150.0 + 70.0 * t) as u8)
    }).collect()
}
const SMOKE_SECS: u32 = 2;

/// Draw a tree or boulder standing in the middle of the tile centred on
/// (sx, sy), at the ground blocks' scale (the pack draws them for its blocks)
fn draw_prop(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32) {
    let k = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
    let (w, h) = ((s.width as f32 * k).round() as i32, (s.height as f32 * k).round() as i32);
    draw_zoomed(buf, s, sx - w / 2, sy + (3.0 * zoom) as i32 - h, k);
}

/// A villager sprite cropped to its standing frame (each sheet is a grid
/// of 32x32 cells)
fn figure(at: &dyn Fn(&str) -> Option<Sprite>, rel: &str) -> Sprite {
    match at(rel) {
        Some(s) => s.crop(0, 0, 32, 32),
        None => { println!("sprite load {}", rel); Sprite { width: 1, height: 1, pixels: vec![255, 0, 255, 255] } }
    }
}

/// The sprite for `houses[i]`, or none where nothing loaded
static NO_SPRITE: Option<Sprite> = None;

impl Art {
    /// Load every sprite through `fetch` (rel path -> png bytes): the file
    /// system on desktop, a bundled pack on the web.
    /// Load from an asset folder on disk ("assets" from the repo root).
    pub fn load_fs(root: &str, seed: u64) -> Art {
        let root = std::path::PathBuf::from(root);
        Self::load(&move |rel: &str| std::fs::read(root.join(rel)).ok(), seed)
    }

    /// The cottage `i` wraps round to (the house list cycles)
    fn house(&self, i: usize) -> &Option<Sprite> {
        self.houses.get(i % self.houses.len().max(1)).unwrap_or(&NO_SPRITE)
    }

    /// Load every sprite, `seed` settling which variant each single-choice
    /// building wears this map (the mill's colour, the corner towers, the
    /// church's keep) - the same map seed looks the same every run
    pub fn load(fetch: &dyn Fn(&str) -> Option<Vec<u8>>, seed: u64) -> Art {
        let at = |rel: &str| fetch(rel).and_then(|b| Sprite::decode(&b).ok());
        let assets = Assets::new("");
        let pack = |rel: &str| at(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/{}", rel));

        // one variant out of several, chosen by the seed (None if it won't
        // load, or the first that did)
        let chosen = |v: Vec<Option<Sprite>>| -> Option<Sprite> {
            if v.is_empty() { return None; }
            v.get(seed as usize % v.len()).and_then(|s| s.clone())
                .or_else(|| v.into_iter().find_map(|s| s))
        };

        // Load villager sprites (every look the villagers pack has, one
        // each): role and gender as villager_sprite knows them
        let vpeople: [(&str, &str); 8] = [
            ("peasant", "m"), ("peasant", "f"), ("elder", "m"), ("noble", "m"),
            ("princess", "f"), ("worker", "m"), ("elder", "f"), ("noble", "f"),
        ];
        let villager_sprites: Vec<Sprite> = vpeople.iter()
            .map(|&(role, gender)| figure(&at, &assets.villager_sprite(role, gender))).collect();
        // the rivals: same figures, the outlined look, and their matriarch
        // is the pack's queen - every villager sprite ends up in use
        let rival_sprites: Vec<Sprite> = ["MiniVillagerMan", "MiniWorker", "MiniQueen"].iter()
            .map(|n| figure(&at, &format!("MinifolksVillagers/MinifolksVillagers/Outline/{}.png", n))).collect();
        // the watch at the gates: soldiers from the humans pack
        let guards: Vec<Sprite> = ["MiniSpearMan", "MiniSwordMan", "MiniShieldMan", "MiniHalberdMan"].iter()
            .map(|n| figure(&at, &format!("MinifolksHumans/Without Outline/{}.png", n))).collect();

        // the cottages: plaster and timber fronts in every colour, with the
        // door and gable in different spots, so no two homes look alike
        let houses: Vec<Option<Sprite>> = [
            "Buildings/house/White houses/Blue house/white_blue_house(1).png",
            "Buildings/house/Wood houses/Blue house/wood_blue_house(1).png",
            "Buildings/house/White houses/Green house/white_green_house(3).png",
            "Buildings/house/Wood houses/Red house/wood_red_house(2).png",
            "Buildings/house/White houses/Wood house/white_wood_house(4).png",
            "Buildings/house/Wood houses/Green house/wood_green_house(2).png",
            "Buildings/house/White houses/Red house/white_red_house(3).png",
            "Buildings/house/Wood houses/Wood house/wood_house(4).png",
            "Buildings/house/Wood houses/Wood house/wood_house(2).png",
            "Buildings/house/White houses/Blue house/white_blue_house(4).png",
        ].iter().map(|rel| pack(rel)).collect();

        // ground specks for open grass: tufts and flowers, snow lumps in
        // winter, the green set gone gold in autumn
        let gold = |s: Sprite| recolour_greens(&s, |c| blend(c, (185, 125, 45), 0.45));
        let detail_set = |dir: &str, name: &str, gild: bool| -> Vec<Sprite> {
            (1..=9).filter_map(|i| pack(&format!("Enviroument/{}/{}({}).png", dir, name, i)).map(|s| if gild { gold(s) } else { s })).collect()
        };
        let details = [
            detail_set("Spring", "grass_detail", false),
            detail_set("Spring", "grass_detail", true),
            detail_set("Winter", "snow_detail", false),
        ];

        Art {
            villager_sprites,
            rival_sprites,
            houses,
            tiles: [TileSet::load(&at, 0), TileSet::load(&at, 1), TileSet::load(&at, 2)],
            faces: [0, 1, 2].map(|season| TileSet::load(&at, season).map_blocks(face_only)),
            mill_frames: (1..=4).map(|i|
                at(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/mill/mill({}).png", i)).unwrap()
            ).collect(),
            mill_tower: chosen(["wood", "blue", "green", "red"].iter()
                .map(|c| pack(&format!("Buildings/mill/mill_{}.png", c))).collect()),
            blacksmith_s: at(&assets.building_sprite("blacksmith", "green")),
            // stone(1) is a tiny pebble that's nearly all dark outline - on
            // the map it read as a black hole, so the boulders stand instead
            stones: (2..=4).filter_map(|i| pack(&format!("Enviroument/Spring/stones/stone({}).png", i))).collect(),
            stone_grounds: (1..=4).filter_map(|i| pack(&format!("Enviroument/Spring/stones/stone_ground ({}).png", i))).collect(),
            player_s: at("MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png").map(|s| s.crop(0, 0, 32, 32)),
            // the pack's corner towers: flat-top, capped or keeps - the
            // whole watch matches within a map, but varies between runs
            tower_s: chosen([("Wood towers", "castle_tower_wood", 1), ("Wood towers", "castle_tower_wood", 4),
                             ("Wood towers", "castle_tower_wood", 5), ("Blue towers", "castle_tower_blue", 4)]
                .iter().map(|(d, n, i)| pack(&format!("Towers/{}/{}({}).png", d, n, i))).collect()),
            // the pack's castle_wall(2) runs along x, (1) along y; (5) and (6) are gatehouses
            walls: [2, 1, 5, 6].map(|n| pack(&format!("Castle walls/castle_wall({}).png", n))),
            // wall(4) and wall(3): the same runs with a postern door in them
            wall_posts: [4, 3].map(|n| pack(&format!("Castle walls/castle_wall({}).png", n))),
            // the church is one of the pack's tall keeps
            church_s: chosen([7, 8, 9].iter().map(|i| pack(&format!("Towers/Blue towers/castle_tower_blue({}).png", i))).collect()),
            guildhall_s: at(&assets.building_sprite("blacksmith", "wood")),
            rival_house: at(&assets.building_sprite("house", "red")),
            rival_woodhouse: at(&assets.building_sprite("woodhouse", "red")),
            priest_s: Some(figure(&at, "MinifolksHumans/Without Outline/MiniMage.png")),
            traveller_s: Some(figure(&at, "MinifolksHumans/Without Outline/MiniHorseMan.png")),
            guards,
            dog: Some(dog_sprite()),
            cat: Some(cat_sprite()),
            chicken: Some(chicken_sprite()),
            gull: Some(gull_sprite()),
            details,
        }
    }
}

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
    /// The seat that called it - resolution belongs to their org
    pub seat: usize,
}
/// How fast people walk to a meeting (tiles a frame), how long the host
/// waits for them, and how long each turn to speak lasts
const MEETING_WALK: f32 = 2.5 / FPS as f32;
const GATHER_FRAMES: u32 = 20 * FPS;
const LINE_FRAMES: u32 = 2 * FPS;
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

/// Tiles a side of the map: the town keeps its size, the country around it
/// grew half again so the farmland and wilds have room
pub const MAP_TILES: usize = 49;

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

/// The ground layer in world space at one zoom, drawn once then blitted
/// each frame: `origin` is the cache pixel that lands on screen (0,0) when
/// the camera is at (0,0)
pub struct GroundCache {
    pub buf: Buffer,
    pub origin: (i32, i32),
    pub zoom: f32,
    pub season: Season, // the season it was drawn in (Spring and Summer share a tile set but not their fallback colours)
    pub rev: u64,       // map.rev() when it was drawn
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
}

impl Default for Seat {
    fn default() -> Seat {
        Seat { player: 0, org: 0, picked: false, approach: None, selected_target: 0,
               selected_building: None, ctx_actions: vec![], agent: false }
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
    /// The classic map (MAP_SEED): tests, dumps and tooling stay put
    #[cfg(test)]
    fn new() -> Game { Game::new_seeded(MAP_SEED) }

    /// A game on the map `seed` draws. The country is perturbed - the coast
    /// shifts, the river meanders a tile or two - and the castle suits it:
    /// the wall's east face keeps clear of the river's west bank. Outside it
    /// the map's grown a little townage of its own: a farm below the south
    /// gate, a woodcutter's clearing and a fisher's hut on the shore.
    pub fn new_seeded(seed: u64) -> Game {
        let mut rng = Rng(seed ^ 0x9E3779B9);
        let mut world = World::new();
        let mut sched = Scheduler::new();
        sched.add_system(villager::HungerSystem);
        sched.add_system(villager::SeekFoodSystem);
        sched.add_system(pets::PetBondSystem);
        sched.add_system(SeasonSystem);
        sched.add_system(GuildFormationSystem);
        sched.add_system(IllnessSystem); // diseases progress, spread to neighbours, and heal

        let mut terrain = Terrain::generate_country(MAP_TILES, 0.7, seed);
        let climate = climate::spawn_climate(&mut world);
        // the felt-worth ledger rides beside the climate - what the
        // shelves show leans what a swap is worth, quietly
        goods::spawn_worths(&mut world);
        // tell the sim where the tile edge is, or its fixed bound pinches
        // anyone homed past the old 33-tile map
        world.add_component(climate, villager::WorldBounds { w: MAP_TILES as f32, h: MAP_TILES as f32 });
        let market_food = villager::spawn_food_source(&mut world, 16.0, 16.0, 500);

        // the river winds south between the towns to the sea; each waypoint
        // on its course drifts a tile or two, so it meanders differently
        // every map (kept east of the wall and west of the rivals')
        let river: Vec<(i32, i32)> = [3, 9, 15, 21, 27, 33, 39, 45].iter()
            .map(|&y| (24 + rng.below(3), y)).collect();
        // the castle suits the river: the east face keeps a tile clear of
        // its west bank and the rest of the rectangle breathes a tile
        // either way (every building still sits well inside). Only the
        // waypoints level with the wall set it - the castle doesn't grow
        // just because the river now runs further south
        let wall = (8 + rng.around(1), 7 + rng.around(1),
            river.iter().filter(|w| w.1 <= 30).map(|w| w.0).min().unwrap() - 1, 24 + rng.around(1));
        // the walled fields run out of the south gate to a little farm
        let (gate_x, farm_y) = ((wall.0 + wall.2) / 2, wall.3 + 2);

        // fields beside the food store, and strips either side of the lane
        // that runs out of the south gate, down through the farm - most of
        // the land the map grew is worked
        let mut fields: Vec<(i32, i32, Tile)> = vec![
            (13, 14, Tile::Wheat), (14, 14, Tile::Wheat), (13, 15, Tile::Wheat), (14, 15, Tile::Wheat),
            (17, 14, Tile::Corn), (18, 14, Tile::Corn), (18, 15, Tile::Corn),
            (gate_x - 2, wall.3 + 1, Tile::Wheat), (gate_x - 1, wall.3 + 1, Tile::Wheat),
            (gate_x - 2, wall.3 + 2, Tile::Wheat), (gate_x - 1, wall.3 + 2, Tile::Corn),
            (gate_x, wall.3 + 2, Tile::Corn), (gate_x + 1, wall.3 + 2, Tile::Wheat),
        ];
        for dy in 2..10 {
            for &dx in &[-6i32, -5, -4, -3, 3, 4, 5, 6] {
                let (fx, fy) = (gate_x + dx, farm_y + dy);
                if noise(fx, fy, 17) < 0.15 { continue; } // a ragged field edge
                fields.push((fx, fy, if dy % 3 == 1 { Tile::Corn } else { Tile::Wheat }));
            }
        }

        // where each building wants to stand; the map moves one onto dry land if it must
        let planned: Vec<((i32, i32), BSprite)> = vec![
            // homes around the market (the first three have chimney smoke)
            ((13, 12), BSprite::House(0)), ((19, 12), BSprite::House(1)), ((12, 18), BSprite::House(2)),
            ((19, 19), BSprite::Blacksmith),
            // church (religion centre) + guild hall (merchant base)
            ((14, 9), BSprite::Church), ((17, 10), BSprite::Guildhall),
            // rival village across the river (red-roofed foreign houses)
            ((28, 13), BSprite::RivalHouse), ((30, 15), BSprite::RivalWoodhouse),
        ];
        let mut herb_patches: Vec<HerbPatch> = vec![];
        let mut layout = Layout {
            // our village and the rivals' each stand on dry land, the neck
            // between them never floods, and the farm below the south gate
            // gets a broad pocket of level field
            settlements: vec![(16, 16, 9), (29, 14, 4), (25, 15, 3), (gate_x, farm_y + 4, 6)],
            river: river.clone(),
            bridge_near: (river[2].0, 15),
            town_wall: Some(wall),
            fields,
            buildings: planned.iter().map(|(p, _)| *p).collect(),
            road_hub: (16, 16),
            // the lane out of the south gate runs down through the strips
            // to the farmstead, beside the road to the rivals' village
            road_ends: vec![(29, 14), (gate_x, farm_y + 8)],
        };
        layout.buildings.push((MILL_TILE.0 as i32, MILL_TILE.1 as i32));
        let mut map = TileMap::generate(&mut terrain, &layout);
        let mut buildings: Vec<(f32, f32, BSprite)> = planned.iter().zip(&map.buildings)
            .map(|((_, s), (x, y))| (*x as f32, *y as f32, *s)).collect();

        // out-of-town townage: a farmhand's cottage by the south fields and
        // a smallholder's at the far end of the strips, a woodcutter's
        // clearing at the woods' edge, a fisher's hut on the shore - each
        // gets its road run out to it
        for (site, sprite) in [
            (Some((gate_x + 2, farm_y)), BSprite::House(6)),
            (Some((gate_x - 3, farm_y + 9)), BSprite::House(9)),
            (shore_site(&map, (26, 30)), BSprite::House(7)),
            (woods_edge_site(&map, wall, (wall.0, (wall.1 + wall.3) / 2)), BSprite::House(8)),
        ] {
            let Some((bx, by)) = site.and_then(|(x, y)| map.nearest_buildable(x, y)) else { continue };
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, sprite));
        }

        // a fuller town: homes on a loose grid inside the wall (so ~30
        // people have somewhere to live), the odd cottage outside, and a
        // workshop for most of the vocations that get rolled - the ~85%
        // coverage leaves the odd trade without a shop to work out of
        for gy in (wall.1 + 3..wall.3 - 1).step_by(3) {
            for gx in (wall.0 + 3..wall.2 - 1).step_by(3) {
                if let Some((bx, by)) = map.nearest_buildable(gx + rng.around(1), gy + rng.around(1)) {
                    map.build_at(bx, by);
                    map.connect((bx, by));
                    buildings.push((bx as f32, by as f32, BSprite::House(rng.below(97) as usize)));
                }
            }
        }
        for &(x, y) in &[(wall.0 - 3, wall.3 + 3), (wall.2 + 4, wall.3 + 2), (wall.0 - 5, (wall.1 + wall.3) / 2)] {
            if let Some((bx, by)) = map.nearest_buildable(x, y) {
                map.build_at(bx, by);
                map.connect((bx, by));
                buildings.push((bx as f32, by as f32, BSprite::House(rng.below(97) as usize)));
            }
        }
        // the herb-wife's hut stands apart in the meadow below the west end
        // of the wall, nearer the wild herbs than the market square
        let herb_hut_site = map.nearest_buildable(wall.0 + 1, wall.3 + 3);
        if let Some((bx, by)) = herb_hut_site {
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(5)));
        }
        // the apiary: the beekeeper's yard at the meadow's edge, east of the fold
        let apiary_site = map.nearest_buildable(wall.0 + (wall.2 - wall.0) / 2 + 8, wall.3 + 5);
        if let Some((bx, by)) = apiary_site {
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(9)));
        }
        // the fold: the shepherd's pen in the meadow below the town
        let fold_site = map.nearest_buildable(wall.0 + (wall.2 - wall.0) / 2 + 3, wall.3 + 4);
        if let Some((bx, by)) = fold_site {
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(7)));
        }

        // ---- the people ----
        // vocations rolled by weight; knights and priests are PMC
        // workshops for the vocations that get rolled (~85% coverage);
        // shops go on buildable tiles inside the wall, farms just outside
        let mut vocs_present: Vec<Vocation> = vec![];
        let mut adult_vocs: Vec<Vocation> = (0..23).map(|_| roll_vocation(&mut rng)).collect();
        // the fold always has a keeper: one hand from a trade the village
        // has twice (never the last farmer) keeps the flock instead - a
        // later duplicate is taken so the shops roll lands the same
        let swap = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or_else(|| adult_vocs.iter().rposition(|&v| v != Vocation::Farmer).unwrap_or(5));
        adult_vocs[swap] = Vocation::Shepherd;
        // and the apiary: another doubled trade hands the bees over
        let swap2 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap);
        adult_vocs[swap2] = Vocation::Beekeeper;
        // and the loom: the wool needs somewhere to go
        let swap3 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap2);
        adult_vocs[swap3] = Vocation::Weaver;
        // and the still: the gardens' flowers need somewhere to go
        let swap4 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper && v != Vocation::Weaver
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap3);
        adult_vocs[swap4] = Vocation::Perfumer;
        // and the paint shop: pigment to clout
        let swap5 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper && v != Vocation::Weaver && v != Vocation::Perfumer
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap4);
        adult_vocs[swap5] = Vocation::Painter;
        // and the river's edge needs its fisher
        let swap6 = adult_vocs.iter().rposition(|&v| v != Vocation::Farmer && v != Vocation::Shepherd
                && v != Vocation::Beekeeper && v != Vocation::Weaver && v != Vocation::Perfumer
                && v != Vocation::Painter && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .unwrap_or(swap5);
        adult_vocs[swap6] = Vocation::Fisher;
        // and the drink trade: every village keeps one - this village's
        // is rolled off the seed (alehouse, brewery, vintner's or still).
        // A later duplicate of an unforced trade gives way first; failing
        // that, any unforced trade does - the forced benches keep theirs
        let drink_voc = DRINK_TRADES[rng.below(DRINK_TRADES.len() as i32) as usize];
        let forced = |v: Vocation| matches!(v,
            Vocation::Farmer | Vocation::Shepherd | Vocation::Beekeeper | Vocation::Weaver
            | Vocation::Perfumer | Vocation::Painter | Vocation::Fisher
            | Vocation::Alewife | Vocation::Brewer | Vocation::Vintner | Vocation::Distiller);
        let swap7 = adult_vocs.iter().rposition(|&v| !forced(v)
                && adult_vocs.iter().filter(|&&x| x == v).count() > 1)
            .or_else(|| adult_vocs.iter().rposition(|&v| !forced(v)))
            .unwrap_or(swap6);
        adult_vocs[swap7] = drink_voc;
        for &v in &adult_vocs { if !vocs_present.contains(&v) { vocs_present.push(v); } }
        let mut shop_kinds: Vec<BuildingKind> = vec![];
        for v in vocs_present {
            // the fold and the apiary stand apart, seeded on their own
            if v == Vocation::Shepherd || v == Vocation::Beekeeper { continue; }
            // the trades a feature hangs on always keep their house
            let must = matches!(v, Vocation::Weaver | Vocation::Perfumer | Vocation::Painter
                | Vocation::Fisher | Vocation::Alewife | Vocation::Brewer | Vocation::Vintner
                | Vocation::Distiller);
            if must || rng.below(100) < 85 { shop_kinds.push(household::kind_for(v)); }
        }
        // the smithy, church and guildhall already stand: shop for the
        // toolmaker, the priest's church, and the trader's stall
        for (x, y) in [(19, 19), (14, 9)] { let _ = (x, y); }
        for kind in shop_kinds.iter().copied() {
            // near the market for shops; farms by the south fields; the
            // fish-smoke house goes down to the water's edge
            let (cx, cy) = if kind == BuildingKind::Shop(Vocation::Farmer) { (gate_x + 2, farm_y + 1) }
                else if kind == BuildingKind::Shop(Vocation::Fisher) { (35, 18) }
                else { (16 + rng.around(4), 15 + rng.around(4)) };
            let Some((bx, by)) = map.nearest_buildable(cx, cy) else { continue };
            map.build_at(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, BSprite::House(rng.below(97) as usize)));
        }
        // every building gets an entity: homes hold occupants, workplaces
        // hold a worker and a stockpile
        let mut building_entities: Vec<Entity> = vec![];
        let rival_from = buildings.len().saturating_sub(buildings.len()); // marker, computed below
        let n_planned = 8; // the `planned` list
        for (i, &(bx, by, _)) in buildings.iter().enumerate() {
            let (kind, name, cap) = if i < 3 { (BuildingKind::Home, "a cottage", 20) }
                else if i == 3 { (BuildingKind::Shop(Vocation::Toolmaker), "the smithy", 20) }
                else if i == 4 { (BuildingKind::Church, "the church", 20) }
                else if i == 5 { (BuildingKind::TraderStall, "the trader's stall", 12) }
                else if i < n_planned { (BuildingKind::Home, "a foreign house", 20) }
                else if i >= buildings.len() - shop_kinds.len() {
                    // the freshly seeded workshops, in shop_kinds order
                    let k = shop_kinds[i - (buildings.len() - shop_kinds.len())];
                    (k, match k { BuildingKind::Shop(v) => vocation::trade_name(v), BuildingKind::Church => "the church", BuildingKind::WatchPost => "the watch house", _ => "a workshop" }, 20)
                }
                else { (BuildingKind::Home, "a house", 20) };
            building_entities.push(household::spawn_building(&mut world, kind, name, bx, by, cap));
        }
        let _ = rival_from;
        // the herb hut: the Home-kind building standing on the site picked
        // earlier (Pos-matched so shop-seeding order can't mislabel it)
        let herb_hut = herb_hut_site.and_then(|(hx, hy)| {
            let e = building_entities.iter().copied().find(|&e| {
                world.get_component::<Pos>(e)
                    .map(|p| p.x.round() as i32 == hx && p.y.round() as i32 == hy).unwrap_or(false)
            })?;
            if let Some(b) = world.get_component_mut::<Building>(e) { b.name = "the herb hut".into(); }
            Some(e)
        });
        // the apiary: the building on the site picked earlier, re-kinded
        let apiary = apiary_site.and_then(|(hx, hy)| {
            let e = building_entities.iter().copied().find(|&e| {
                world.get_component::<Pos>(e)
                    .map(|p| p.x.round() as i32 == hx && p.y.round() as i32 == hy).unwrap_or(false)
            })?;
            if let Some(b) = world.get_component_mut::<Building>(e) {
                b.kind = household::BuildingKind::Apiary;
                b.name = "the apiary".into();
            }
            Some(e)
        });
        // the cheese cellar: a dark room where wheels age - this
        // village's is a stone one, every village's cheese is its own
        let cellar_trait = "stone-cellar";
        let cellar = fold_site.and_then(|(fx, _)| {
            let b = building_entities.iter().copied().find(|&e| {
                world.get_component::<Building>(e).map(|bb|
                    bb.kind == household::BuildingKind::Home).unwrap_or(false)
                    && world.get_component::<Pos>(e)
                        .map(|p| (p.x.round() as i32 - fx).abs() < 6).unwrap_or(false)
            })?;
            if let Some(bb) = world.get_component_mut::<Building>(b) {
                bb.kind = household::BuildingKind::Cellar;
                bb.name = "the cheese cellar".into();
            }
            Some(b)
        });
        // the fold: the building on the site picked earlier, re-kinded to
        // a pasture (Pos-matched so shop-seeding order can't mislabel it)
        let fold = fold_site.and_then(|(hx, hy)| {
            let e = building_entities.iter().copied().find(|&e| {
                world.get_component::<Pos>(e)
                    .map(|p| p.x.round() as i32 == hx && p.y.round() as i32 == hy).unwrap_or(false)
            })?;
            if let Some(b) = world.get_component_mut::<Building>(e) {
                b.kind = household::BuildingKind::Pasture;
                b.name = "the fold".into();
            }
            Some(e)
        });
        // the trader's stall is the guildhall slot (index 5), stocked with imports
        let trader_stall = building_entities[5];
        for g in ["cloth", "paper", "ink", "pigment"] {
            world.get_component_mut::<Building>(trader_stall).unwrap().stock.insert(g.to_string(), 4);
        }

        let stones: Vec<(i32, i32)> = [(6, 19), (20, 23), (34, 30), (14, 38), (42, 18)].iter()
            .filter_map(|&(x, y)| map.nearest_buildable(x, y)).collect();
        for &(x, y) in &stones { map.occupy(x, y); }

        // the land keeps score: canopy a little thinned already, the pit
        // already wet, the reach already carrying the town's leavings
        let mut land = crate::sim::environment::LandScore {
            tree_cover: map.trees.iter().map(|&t| (t, 0.85f32)).collect(),
            pit_water: 0.5, effluent: Default::default(),
        };
        let pit = stones.first().copied();
        // the reach: river tiles below the south wall take what the town dumps
        let reach: Vec<(i32, i32)> = (0..map.height as i32).flat_map(|y| (0..map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| map.get(x, y) == Some(Tile::River) && y > wall.3).collect();
        for &t in &reach { land.effluent.insert(t, 0.15); }

        // kitchen gardens: a soil bed beside some of the homes
        let homes_for_gardens: Vec<Entity> = building_entities.iter().copied()
            .filter(|&b| world.get_component::<Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false))
            .take(6).collect();
        let mut gardens: Vec<(Entity, (i32, i32), f32, bool)> = vec![];
        for home in homes_for_gardens {
            if let Some(hp) = world.get_component::<Pos>(home).copied() {
                let (hx, hy) = (hp.x.round() as i32, hp.y.round() as i32);
                if let Some(bed) = [(hx + 2, hy), (hx - 2, hy), (hx, hy + 2), (hx, hy - 2),
                                    (hx + 2, hy + 1), (hx - 2, hy - 1)]
                    .iter().copied().find(|&(x, y)| map.passable(x, y)) {
                    // every other bed keeps a kitchen garden - roots and
                    // peas for the pot beside the herb-wife's flowers
                    let kitchen = gardens.len() % 2 == 1;
                    gardens.push((home, bed, (gardens.len() as f32 * 0.3) % 0.9, kitchen));
                }
            }
        }

        // herb patches: wild tufts out in the meadow past the wall, weighted
        // to what cures - the herb-wife keeps them picked over
        const PATCH_HERBS: [&str; 13] = ["willow_bark", "yarrow", "sage", "willow_bark",
            "yarrow", "sage", "chamomile", "lavender", "willow_bark", "yarrow",
            "peppermint", "mugwort", "feverfew"];
        let in_wall = |x: i32, y: i32| (wall.0..=wall.2).contains(&x) && (wall.1..=wall.3).contains(&y);
        let herb_sites: Vec<(i32, i32)> = (0..map.height as i32)
            .flat_map(|y| (0..map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| map.get(x, y) == Some(Tile::Grass) && map.passable(x, y) && !in_wall(x, y))
            .collect();
        let step = (herb_sites.len() / PATCH_HERBS.len().max(1)).max(1);
        for (pi, &herb) in PATCH_HERBS.iter().enumerate() {
            if let Some(&tile) = herb_sites.get(pi * step) {
                herb_patches.push(HerbPatch { tile, herb, next_ready: (pi * 7 % 45) as u64 });
            }
        }

        // the watch stands just inside each gate; the priest keeps by the
        // church; a courier rides the road to the rivals' and back; gulls
        // hang about the open sea
        let guards: Vec<(f32, f32)> = map.walls.iter()
            .filter(|&(_, piece)| matches!(piece, WallPiece::GateAlongX | WallPiece::GateAlongY))
            .map(|&((x, y), piece)| match piece {
                WallPiece::GateAlongX => (x as f32, y as f32 + if y == wall.1 { 0.9 } else { -0.9 }),
                _ => (x as f32 + if x == wall.0 { 0.9 } else { -0.9 }, y as f32),
            }).collect();
        let priest = (14.7, 9.6);
        let traveller_path = map.walk_path((16, 16), (29, 14)).unwrap_or_default();
        let mut gulls = vec![];
        for y in 0..map.height as i32 { for x in 0..map.width as i32 {
            if map.get(x, y) == Some(Tile::Water) && noise(x, y, 21) > 0.85 { gulls.push((x as f32, y as f32)); }
        }}
        gulls.truncate(4);

        let mut villagers = vec![];
        // 23 vocation adults (0..23), two overseers (23,24), the trader
        // (25), three children (26..29) and the tax collector (29)
        for i in 0..NAMES.len() {
            let v = villager::spawn_villager(&mut world, 14.0 + (i % 6) as f32 * 0.8, 14.0 + (i % 4) as f32 * 0.8);
            world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
            world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32 % 6, warmth_episodes: 4 - i as u32 % 5, episodes: vec![] });
            world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
            world.add_component(v, personality_from_seed(i as u64 * 7 + 3));
            world.add_component(v, Satchel::default());
            world.add_component(v, Known::default());
            if i < 23 {
                // a working adult: the level roll sets the skill band
                let skill = match rng.below(100) {
                    r if r < 55 => 0.05 + rng.below(30) as f32 / 100.0,
                    r if r < 85 => 0.40 + rng.below(30) as f32 / 100.0,
                    _ => 0.75 + rng.below(20) as f32 / 100.0,
                };
                world.add_component(v, vocation::craft(adult_vocs[i], skill));
            } else if i < 25 {
                world.add_component(v, TownRole::Overseer);
            } else if i == 25 {
                world.add_component(v, TownRole::Trader);
            } else if i < 29 {
                world.add_component(v, Kid);
                // staggered ages - they come of age one by one, not in a wave
                world.add_component(v, KidGrowth { age_days: (i as u64 * 180) % 700, errands: vec![] });
            } else if i == 29 {
                world.add_component(v, TownRole::TaxCollector);
            } else {
                // the herb-wife: wise to the hedgerow cures, out in the meadow
                world.add_component(v, TownRole::Herbologist);
                world.add_component(v, Apothecary {
                    known_herbs: herbology::medicinal_herbs().iter().map(|(n, _, _)| n.to_string()).collect(),
                    remedies: vec![], skill: 0.55 });
            }
            villagers.push(v);
        }
        for i in 0..3 {
            let m = world.create_entity();
            world.add_component(m, Merchant { name: format!("Merchant{}",i), wealth: 60.0, trade_routes: vec!["north_road".into()], vulnerable: i < 2 });
        }

        // rival village: foreign settlement across the river, with its own
        // granary so its people eat at home instead of crossing to our market
        villager::spawn_food_source(&mut world, 30.0, 13.0, 500);
        let mut rivals = vec![];
        let mut rival_homes = vec![];
        for i in 0..3 {
            let home = (28.0 + i as f32 * 2.0, 12.0 + (i % 2) as f32 * 3.0);
            let r = villager::spawn_villager(&mut world, home.0, home.1);
            world.add_component(r, Health { condition: 0.9, immunity: 0.5 });
            rivals.push(r);
            rival_homes.push(home);
        }

        let player = world.create_entity();
        world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        world.add_component(player, Pos { x: 10.0, y: 10.0 }); // player has a position
        let org = world.create_entity();
        world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });
        world.add_component(org, crate::sim::organized_power::OrganizedPower::new());
        let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
        pets::adopt(&mut world, dog, villagers[0]);
        let cat = pets::spawn_pet(&mut world, "Mog", Species::Cat, PetRole::Hunting);
        pets::adopt(&mut world, cat, villagers[1]);
        let mut pets = vec![(dog, Species::Dog, (0.8, 0.5)), (cat, Species::Cat, (0.5, 0.3))];

        // ---- settle the town ----
        // the player is just another character: a class (never the
        // landowner), a trade, gold to match, and an inner life
        let pvoc = roll_vocation(&mut rng);
        let pclass = match rng.below(10) { 0..=6 => SocialClass::Working, 7..=8 => SocialClass::PetitBourgeois, _ => SocialClass::ProfessionalMiddle };
        let pvoc = if pclass == SocialClass::ProfessionalMiddle && !vocation::is_pmc(pvoc) {
            if rng.below(2) == 0 { Vocation::Knight } else { Vocation::Priest }
        } else { pvoc };
        let pskill = if pclass == SocialClass::PetitBourgeois { 0.8 } else { 0.1 + rng.below(20) as f32 / 100.0 };
        world.add_component(player, vocation::craft(pvoc, pskill));
        world.add_component(player, Satchel::default());
        world.add_component(player, Purse { gold: livelihood::seed_gold(pclass, false, (rng.next() % 1000) as f32 / 1000.0) });
        world.add_component(player, Organizer::default());
        {
            let pwants = desires::sample_wants(pclass, Some(pvoc), false, &mut || rng.next());
            world.add_component(player, InnerLife { wants: pwants, grievances: vec![], satisfied: vec![] });
            world.get_component_mut::<PlayerCharacter>(player).unwrap().resources =
                world.get_component::<Purse>(player).unwrap().gold;
        }
        // beds: homes hold 1-4; the unlucky have NoHome
        let homes: Vec<Entity> = building_entities.iter().copied().filter(|&b| {
            world.get_component::<Building>(b)
                // the herb hut is hers alone - nobody else takes a bed in it
                .map(|bb| bb.kind == BuildingKind::Home && bb.name != "a foreign house" && bb.name != "the herb hut").unwrap_or(false)
        }).collect();
        for (i, &v) in villagers.iter().enumerate() {
            if i == HERB_IDX {
                // the herb-wife keeps her own roof, out past the wall
                if let Some(hut) = herb_hut {
                    world.get_component_mut::<Building>(hut).unwrap().occupants.push(v);
                }
                continue;
            }
            let prefer = if KID_IDXS.contains(&i) {
                // a child lodges with the adult a few places before him
                household::home_of(&world, villagers[(i + 20) % 23])
            } else { None };
            let mut placed = false;
            if let Some(h) = prefer {
                if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                    world.get_component_mut::<Building>(h).unwrap().occupants.push(v);
                    placed = true;
                }
            }
            for k in 0..homes.len().max(1) {
                if placed { break; }
                let h = homes[(i * 7 + k) % homes.len()];
                if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                    world.get_component_mut::<Building>(h).unwrap().occupants.push(v);
                    placed = true;
                }
            }
            let _ = placed;
        }
        // the player finds a bed too, like anyone else
        for k in 0..homes.len().max(1) {
            let h = homes[(k * 3) % homes.len()];
            if world.get_component::<Building>(h).map(|b| b.occupants.len() < 4).unwrap_or(false) {
                world.get_component_mut::<Building>(h).unwrap().occupants.push(player);
                break;
            }
        }
        // everyone sleeps where they live - standing beside the door (the
        // home tile itself is occupied, and would trap them there); the
        // unhomed get a clear tile off the market square
        for (i, &v) in villagers.iter().enumerate() {
            let stand = household::home_of(&world, v)
                .and_then(|h| world.get_component::<Pos>(h).copied())
                .map(|hp| (hp.x.round() as i32, hp.y.round() as i32))
                .unwrap_or((16 + (i % 3) as i32, 17 + (i % 2) as i32));
            let door = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (stand.0 + dx, stand.1 + dy)))
                .find(|&(x, y)| map.passable(x, y))
                .unwrap_or(stand);
            let p = world.get_component_mut::<Pos>(v).unwrap();
            p.x = door.0 as f32 + ((i % 3) as f32 - 1.0) * 0.3;
            p.y = door.1 as f32 + ((i % 2) as f32) * 0.3;
        }
        // workshops: one worker each; trades with no shop (or more hands
        // than benches) leave folk working the street
        for &v in &villagers {
            let Some(voc) = world.get_component::<Craft>(v).map(|c| c.vocation) else { continue };
            let want = household::kind_for(voc);
            if let Some(b) = building_entities.iter().copied().find(|&b| {
                world.get_component::<Building>(b).map(|bb| bb.kind == want && bb.worker.is_none()).unwrap_or(false)
            }) {
                world.get_component_mut::<Building>(b).unwrap().worker = Some(v);
                world.get_component_mut::<Craft>(v).unwrap().workplace = Some(b);
            }
        }
        // and if the player's trade has a bench free, it's theirs
        if world.get_component::<Craft>(player).unwrap().workplace.is_none() {
            let want = household::kind_for(pvoc);
            if let Some(b) = building_entities.iter().copied().find(|&b| {
                world.get_component::<Building>(b).map(|bb| bb.kind == want && bb.worker.is_none()).unwrap_or(false)
            }) {
                world.get_component_mut::<Building>(b).unwrap().worker = Some(player);
                world.get_component_mut::<Craft>(player).unwrap().workplace = Some(b);
            }
        }
        world.get_component_mut::<Building>(trader_stall).unwrap().worker = Some(villagers[25]);
        // a few carry a difference: the village walks a little slower
        // for them, and theirs check on them
        for (i, kind) in [(7usize, "a lame leg"), (11, "failing sight"), (15, "a mind that needs quiet")] {
            if i < villagers.len() {
                world.add_component(villagers[i], villager::Condition { kind });
            }
        }
        // and the recluse keeps to the cave at the map's edge
        let hermit_e = villagers.get(22).copied();
        if let Some(h) = hermit_e {
            world.add_component(h, TownRole::Hermit);
            world.add_component(h, Pos { x: 36.0, y: 33.0 });
        }
        // the heart-helper: one elder who keeps an eye on lonely hearts
        let matchmaker = villagers.get(27).copied();
        if let Some(mm) = matchmaker { world.add_component(mm, TownRole::Matchmaker); }
        // the shepherd keeps the fold, and a sheepdog keeps him
        let shepherd = villagers.iter().copied().find(|&v|
            world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Shepherd).unwrap_or(false));
        // the beekeeper keeps the apiary, and the skeps stand round it
        let beekeeper = villagers.iter().copied().find(|&v|
            world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Beekeeper).unwrap_or(false));
        let church = Some(building_entities[4]);
        let mut hives = vec![];
        if let (Some(ay), Some((ax, ay2))) = (apiary, apiary_site) {
            for i in 0..4 {
                let (dx, dy) = ((i % 2) as i32 * 2 - 1, (i / 2) as i32 * 2 - 1);
                let (hx, hy) = (1..=4).map(|r| (ax + dx * r, ay2 + dy * r))
                    .find(|&(x, y)| map.passable(x, y)).unwrap_or((ax + 1, ay2 + 1));
                hives.push(apiary::spawn_hive(&mut world, if i % 2 == 0 { HiveKind::Skep } else { HiveKind::Log }, ay, hx as f32 + 0.5, hy as f32 + 0.5));
            }
        }
        if let Some(sh) = shepherd {
            let d = pets::spawn_pet(&mut world, "the sheepdog", Species::Dog, PetRole::Herding);
            world.get_component_mut::<Pet>(d).unwrap().owner = Some(sh);
            pets.push((d, Species::Dog, (0.7, 0.5)));
        }
        // the flock: sheep in the main, a few goats, out round the fold
        let mut flock: Vec<Entity> = vec![];
        if let (Some(fold), Some((fx, fy))) = (fold, fold_site) {
            for a in 0..10 {
                let kind = if a < 7 { LivestockKind::Sheep } else { LivestockKind::Goat };
                let (dx, dy) = ((a % 4) as i32 - 1, (a / 4) as i32 - 1);
                let (ax, ay) = (1..=6).map(|r| (fx + dx * r, fy + dy * r))
                    .find(|&(x, y)| map.passable(x, y)).unwrap_or((fx + 1, fy + 1));
                flock.push(livestock::spawn_animal(&mut world, kind, fold, ax as f32 + 0.35, ay as f32 + 0.45));
            }
        }
        let graze_at = fold.and_then(|f| world.get_component::<Pos>(f).map(|p| (p.x, p.y))).unwrap_or((0.0, 0.0));
        // mushroom patches, each kind true to its ground: the wood's shade
        // keeps the edibles and their death-traps side by side, the pasture
        // keeps the little nipple-tipped caps (and the little brown killers
        // beside them), the fly agaric stays under birch, and the best
        // mushrooms grow in the under
        let mut mush_patches: Vec<MushPatch> = vec![];
        let wood_kinds = [MushKind::Chanterelle, MushKind::DeathCap, MushKind::FieldMushroom,
                          MushKind::JackOLantern, MushKind::Galerina, MushKind::Chanterelle];
        for (mi, &(tx, ty)) in map.trees.iter().step_by(7).take(6).enumerate() {
            if let Some(t) = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (tx + dx, ty + dy)))
                .find(|&(x, y)| map.passable(x, y)) {
                mush_patches.push(MushPatch { tile: t, kind: wood_kinds[mi % wood_kinds.len()], next_ready: (mi * 11) as u64 });
            }
        }
        // the fly agaric keeps to birch: a patch off a deeper tree
        if let Some(&(tx, ty)) = map.trees.iter().nth(11) {
            if let Some(t) = (-3..=3).flat_map(|dy| (-3..=3).map(move |dx| (tx + dx, ty + dy)))
                .find(|&(x, y)| map.passable(x, y)) {
                mush_patches.push(MushPatch { tile: t, kind: MushKind::FlyAgaric, next_ready: 30 });
            }
        }
        // liberty caps in the wet grassland by the fold - the shepherd's
        // secret - with a galerina a tile off: the little brown mushroom
        // that looks just like them is the one that kills
        let patch_near = |mush_patches: &mut Vec<MushPatch>, map: &TileMap,
                              (cx, cy): (i32, i32), kind: MushKind, ready: u64| {
            if let Some(t) = (-4..=4).flat_map(|dy| (-4..=4).map(move |dx| (cx + dx, cy + dy)))
                .find(|&(x, y)| map.passable(x, y) && !mush_patches.iter().any(|p| p.tile == (x, y))) {
                mush_patches.push(MushPatch { tile: t, kind, next_ready: ready });
            }
        };
        let graze_tile = (graze_at.0.round() as i32, graze_at.1.round() as i32);
        patch_near(&mut mush_patches, &map, graze_tile, MushKind::LibertyCap, 40);
        if let Some(lc) = mush_patches.iter().find(|p| p.kind == MushKind::LibertyCap).map(|p| p.tile) {
            patch_near(&mut mush_patches, &map, (lc.0 + 1, lc.1 + 1), MushKind::Galerina, 45);
        }
        // the cave's deep patch: the best mushrooms grow in the under -
        // pushed last so the wood's poisoned patches keep their numbers
        mush_patches.push(MushPatch { tile: (35, 33), kind: MushKind::UnderCap, next_ready: 0 });
        // hens scratch about a few of the cottages' yards
        let mut hens = vec![];
        let mut hen_houses: Vec<Entity> = vec![];
        for &h in homes.iter().step_by(5).take(4) {
            let Some(hp) = world.get_component::<Pos>(h).copied() else { continue };
            hen_houses.push(h);
            for a in 0..3 {
                let (ax, ay) = (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| (hp.x.round() as i32 + dx + a % 2, hp.y.round() as i32 + dy)))
                    .find(|&(x, y)| map.passable(x, y)).unwrap_or((hp.x.round() as i32 + 1, hp.y.round() as i32 + 1));
                hens.push(livestock::spawn_animal(&mut world, LivestockKind::Hen, h, ax as f32 + 0.4, ay as f32 + 0.5));
            }
        }
        // an overseer drives each farm and mason's yard
        let mut oi = 23usize;
        for &b in &building_entities {
            if oi >= 25 { break; }
            let drives = world.get_component::<Building>(b)
                .map(|bb| matches!(bb.kind, BuildingKind::Shop(Vocation::Farmer | Vocation::Mason))).unwrap_or(false);
            if drives {
                world.get_component_mut::<Building>(b).unwrap().overseer = Some(villagers[oi]);
                oi += 1;
            }
        }
        // masters pair juniors: half the journeymen/apprentices find a
        // same-vocation master to learn from
        for i in 0..23 {
            let v = villagers[i];
            let (voc, skill) = match world.get_component::<Craft>(v) { Some(c) => (c.vocation, c.skill), None => continue };
            if vocation::level_of(skill) == Level::Master || rng.below(100) >= 50 { continue; }
            let master = villagers[..23].iter().copied().find(|&m| {
                m != v && world.get_component::<Craft>(m)
                    .map(|c| c.vocation == voc && vocation::level_of(c.skill) == Level::Master).unwrap_or(false)
            });
            if let Some(m) = master {
                world.get_component_mut::<Craft>(v).unwrap().master = Some(m);
                relationships::add_relationship(&mut world, relationships::Relationship {
                    from: v, other: m, kind: relationships::RelKind::Apprentice, strength: 0.4, interactions: 3 });
            }
        }
        // purses and inner lives now that classes are settled
        let mut g = Rng(seed ^ 0xA11CE);
        for (i, &v) in villagers.iter().enumerate() {
            let class = desires::class_of(&world, v);
            let is_kid = KID_IDXS.contains(&i);
            let gold = livelihood::seed_gold(class, is_kid, (g.next() % 1000) as f32 / 1000.0);
            world.add_component(v, Purse { gold });
            let voc = world.get_component::<Craft>(v).map(|c| c.vocation);
            let wants = desires::sample_wants(class, voc, is_kid, &mut || g.next());
            world.add_component(v, InnerLife { wants, grievances: vec![], satisfied: vec![] });
        }
        // the dark keeps the hermit - their one big want, now that they
        // have an inner life to hold it in
        if let Some(h) = hermit_e {
            desires::add_grievance(&mut world, h, desires::Problem::TheDark, u64::MAX);
        }
        // the player's coins and their Purse are the same purse - see
        // sync_purse, called at the end of every day's business
        sync_purse(&mut world, player, true);
        // the collector starts his yearly round in a few weeks
        let collector = villagers[COLLECTOR_IDX];
        let herbologist = villagers[HERB_IDX];
        {
            let mut roster: Vec<Entity> = villagers.clone();
            roster.push(player);
            livelihood::tax_round(&mut world, collector, &roster, 25, 11);
        }

        Game {
            world, sched, map, climate, villagers, rivals,
            seats: vec![Seat { player, org, ..Seat::default() }], pets, market_food, buildings, stones,
            town_wall: wall,
            guards,
            posted: vec![],
            priest,
            traveller_path,
            gulls,
            chicken_at: (gate_x as f32 + 0.4, farm_y as f32 + 0.3),
            rng,
            built_count: 0,
            works: vec![],
            rival_homes,
            toasts: vec![],
            last_guild_count: 0,
            tick: 0,
            paused: false,
            speed: 1,
            log: vec!["Welcome to the village.".into(),
                format!("The village keeps {}.", vocation::trade_name(drink_voc))],
            log_dropped: 0,
            bubbles: vec![],
            meeting: None,
            near: vec![],
            last_chat: vec![],
            was_sick: vec![None; NAMES.len()],
            died_on: vec![None; NAMES.len()],
            frame_n: 0,
            precip: None,
            rainbow_from: None,
            building_entities,
            collector: Some(collector),
            trader_stall: Some(trader_stall),
            travels: vec![],
            minstrel: None,
            next_minstrel: 250 + seed % 200,
            pedlar: None,
            next_pedlar: 60 + seed % 60,
            circus: None,
            next_circus: 300 + seed % 180,
            ink_boiler: None,
            next_ink_boiler: 400 + seed % 200,
            fire: Fire::new(),
            derelict: Default::default(),
            voiced: vec![],
            herbologist: Some(herbologist),
            herb_hut,
            herb_patches,
            remedy_visits: vec![],
            fold,
            shepherd,
            flock,
            graze_at,
            next_graze: 0,
            hens,
            hen_houses,
            matchmaker,
            match_tried: vec![],
            mush_patches,
            shroom_breaks: vec![],
            shroom_lore: vec![],
            ailments: vec![],
            shroom_rng: Rng(seed ^ 0xCA45),
            shroom_disclaimed: false,
            apiary,
            beekeeper,
            hives,
            church,
            swarm: None,
            succeeded: vec![],
            trails: Default::default(),
            land,
            pit,
            reach,
            sanctioned: vec![],
            wolf_coming: false,
            takings: 0.0,
            fox_coming: false,
            gardens,
            cellar,
            cheeses: vec![],
            cellar_trait,
            lord_esteem: 0.0,
            numeracy: if seed % 10 < 6 { Numeracy::Tally } else { Numeracy::RomanBoard },
            translator: None,
            hermit: hermit_e,
            reckonings: 0,
            sky: None,
            pending_discovery: None,
            disclosure_made: false,
            cured: 0,
            repression_debt: 0,
            harsh_winter: false,
            painted: vec![],
            heirlooms: vec![],
            letters: vec![],
            cave_pos: Some((36, 33)),
            player_passed: false,
            martyr_grave: None,
            path_cache: std::collections::HashMap::new(),
            path_tick: u64::MAX,
            sfx: vec![],
            dungeon: vec![],
            rare: vec![],
            next_rare: events::initial_due(seed),
            rare_fired: vec![],
            rare_seed: seed ^ 0xAA55_5EED,
        }
    }

    /// A village already mid-story: one of the saved scenarios - the
    /// fever year, the circus winter, the widow's shop
    pub fn new_scenario(s: Scenario) -> Game {
        let mut g = Game::new_seeded(MAP_SEED);
        match s {
            Scenario::FeverYear => {
                // the scarlatina's already in three beds when you arrive
                for (i, &v) in g.villagers.iter().take(3).enumerate() {
                    let d = g.world.create_entity();
                    g.world.add_component(d,
                        Disease::new(DiseaseKind::ScarletFever, v, 0.6 - i as f32 * 0.1));
                }
                g.log.push(format!("t{}: the scarlatina year - it's already in three beds.", g.tick));
            }
            Scenario::CircusWinter => {
                // a hard winter, and the tents already pitched outside
                g.tick = 400; // inside the harsh window
                g.spawn_circus();
                g.log.push("t400: the circus winter - wonders, and thin stores.".to_string());
            }
            Scenario::WidowsShop => {
                // a shop stands derelict, a widow weighs the keeping of it
                let b = g.building_entities.iter().copied().find(|&b|
                    g.world.get_component::<Building>(b)
                        .map(|bb| bb.worker.is_some() && matches!(bb.kind, household::BuildingKind::Shop(_)))
                        .unwrap_or(false));
                if let Some(b) = b {
                    if let Some(w) = g.world.get_component::<Building>(b).and_then(|bb| bb.worker) {
                        g.world.add_component(w, villager::Dead);
                        g.succession_tick(1);
                        g.log.push("t1: the widow's shop - who's to keep it now?".to_string());
                    }
                }
            }
        }
        g
    }

    /// The tuning panel's gauges: what the village's innards read at
    /// this tick - trail heat, people power, the economy's pulse
    pub fn telemetry(&self) -> Vec<(&'static str, f64)> {
        let alive = self.villagers.iter().filter(|&&v| !villager::is_dead(&self.world, v)).count();
        // hungry = below the sim's own seek-food line (30); a villager between
        // 30-40 is just low on the eat cycle, not in trouble
        let hungry = self.villagers.iter().filter(|&&v|
            self.world.get_component::<Hunger>(v).map(|h| h.value < 30.0).unwrap_or(false)).count();
        // starving = the hard line: a real food crisis, not the bottom of a meal cycle
        let starving = self.villagers.iter().filter(|&&v|
            self.world.get_component::<Hunger>(v).map(|h| h.value <= 10.0).unwrap_or(false)).count();
        let trail_heat: f32 = self.trails.values().map(|t| t.strength).sum();
        let trail_max = self.trails.values().map(|t| t.strength).fold(0.0f32, f32::max);
        let power = self.world.get_component::<crate::sim::organized_power::OrganizedPower>(self.org)
            .map(|p| p.capacity).unwrap_or(0.0);
        let grain = self.world.get_component::<Building>(self.market_food)
            .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0) as f64;
        let bread = self.world.get_component::<Building>(self.market_food)
            .map(|b| b.stock.get("bread").copied().unwrap_or(0)).unwrap_or(0) as f64;
        let food = self.world.get_component::<villager::FoodSource>(self.market_food)
            .map(|f| f.servings).unwrap_or(0) as f64;
        // the village's gold: mean purse and the top/mean spread - the
        // economy's pulse while prices sit on a fixed table
        let purses: Vec<f32> = self.villagers.iter()
            .filter(|&&v| !villager::is_dead(&self.world, v))
            .filter_map(|&v| self.world.get_component::<Purse>(v).map(|p| p.gold))
            .collect();
        let gold_mean = if purses.is_empty() { 0.0 } else { purses.iter().sum::<f32>() / purses.len() as f32 };
        let gold_top = purses.iter().copied().fold(0.0f32, f32::max);
        let org_members = self.world.get_component::<Organization>(self.org)
            .map(|o| o.members.len()).unwrap_or(0);
        let sanctioned = self.sanctioned.len();
        let tree_cover = self.land.tree_cover.values().sum::<f32>() / self.land.tree_cover.len().max(1) as f32;
        vec![
            ("villagers.alive", alive as f64),
            ("villagers.hungry", hungry as f64),
            ("villagers.starving", starving as f64),
            ("econ.gold_mean", gold_mean as f64),
            ("econ.gold_spread", (gold_top / gold_mean.max(1.0)) as f64),
            ("org.members", org_members as f64),
            ("org.power", power as f64),
            ("org.lord_esteem", self.lord_esteem as f64),
            ("sanctioned", sanctioned as f64),
            ("trails.heat", trail_heat as f64),
            ("trails.max", trail_max as f64),
            ("stock.grain", grain),
            ("stock.bread", bread),
            ("stock.food", food),
            ("env.tree_cover", tree_cover as f64),
            ("price.grain", goods::worth_at(&self.world, "grain") as f64),
            ("price.wood", goods::worth_at(&self.world, "wood") as f64),
            ("price.fish", goods::worth_at(&self.world, "fish") as f64),
            ("winter.harsh", if self.harsh_winter { 1.0 } else { 0.0 }),
        ]
    }

    /// The gauges as overlay lines: "org.power 0.4" for the tuning panel
    pub fn telemetry_lines(&self) -> Vec<String> {
        self.telemetry().iter().map(|(k, v)| format!("{} {:.2}", k, v)).collect()
    }

    pub fn ticks_per_frame(&self) -> u32 {
        // frames per day: ~5s/day normal, ~1.2s turbo
        if self.speed == 1 { 5 * FPS } else { 5 * FPS / 4 }
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

    pub fn player_pos_at(&self, seat: usize) -> (f32, f32) {
        self.world.get_component::<Pos>(self.seats[seat].player).map(|p| (p.x, p.y)).unwrap_or((10.0, 10.0))
    }

    /// Where the courier is on the road between the villages: there and
    /// back along `traveller_path`, at a riding pace
    pub fn traveller_pos(&self) -> Option<(f32, f32)> {
        let n = self.traveller_path.len();
        if n < 2 || self.traveller_path.is_empty() { return None; }
        let leg = (n - 1) as f32;
        let mut s = (self.frame_n as f32 * TRAVELLER_SPEED) % (leg * 2.0);
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
        let member = self.world.get_component::<Organization>(self.seats[seat].org).map(|o| o.members.contains(&target)).unwrap_or(false);
        let held = self.world.get_component::<crate::game::dungeon::Imprisoned>(target).is_some()
            || self.world.get_component::<crate::game::dungeon::CastleHeld>(target).is_some();
        if (i == 0 || i == 1) && !target_alive { Some("they have died".into()) }
        else if (i == 0 || i == 1) && held { Some("in the dungeon".into()) }
        else if i == 1 && member { Some("already a member".into()) }
        else if i == 2 && p.1 < INVEST_GOLD { Some(format!("need {:.0} gold", INVEST_GOLD)) }
        else if i == 1 && !recruit_infl_ok { Some(format!("need influence {:.2}", recruit_influence_needed())) }
        else if i == 3 && self.meeting.is_some() { Some("a meeting is under way".into()) }
        else if i == 3 && in_reach.is_empty() { Some("no one nearby".into()) }
        else { None }
    }

    /// One frame of game time: the sim ticks every few frames, and between
    /// ticks villagers keep strolling
    pub fn update(&mut self) {
        let ticks_per_frame = self.ticks_per_frame();
        // the soundscape resets each frame: cues live exactly one read
        self.sfx.clear();
        // sim advance: one tick = one day, throttled by frame count
        self.frame_n += 1;
        let mut ticked = false;
        if !self.paused && self.frame_n % ticks_per_frame == 0 {
            self.tick += 1;
            self.sched.tick(&mut self.world);
            ticked = true;
        }
        // the town's business each day: work, errands, taxes
        if ticked { self.livelihood_tick(self.tick); }
        // where everyone stood, so no one ends the frame inside a wall or a house
        let people: Vec<Entity> = self.villagers.iter().chain(&self.rivals).copied().collect();
        let stood: Vec<Option<Pos>> = people.iter().map(|e| self.world.get_component::<Pos>(*e).copied()).collect();
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

        // idle villagers wander (village feels alive); direction changes daily,
        // step is per frame so they stroll ~0.6 tiles/day instead of racing to the edge
        if !self.paused {
            let step = 0.6 / ticks_per_frame as f32;
            for v in villagers.iter() {
                let idle = world.get_component::<Villager>(*v).map(|x| x.activity == Activity::Idle).unwrap_or(false);
                if idle && !crate::game::dungeon::unavailable(world, *v) {
                    if let Some(p) = world.get_component_mut::<Pos>(*v) {
                        let wob = ((tick + *v as u64) % 8) as f32;
                        if wob < 1.0 { p.x += step; } else if wob < 2.0 { p.y += step; }
                        else if wob < 3.0 { p.x -= step; } else if wob < 4.0 { p.y -= step; }
                        p.x = p.x.clamp(2.0, self.map.width as f32 - 2.0);
                        p.y = p.y.clamp(2.0, self.map.height as f32 - 2.0);
                    }
                }
            }
        }

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
            let market_has = world.get_component::<villager::FoodSource>(self.market_food)
                .map(|f| f.servings > 0).unwrap_or(false);
            for v in villagers.iter() {
                if crate::game::dungeon::unavailable(world, *v) { continue; }
                let vdata = world.get_component::<Villager>(*v);
                let seeking = vdata.map(|x| x.activity == Activity::SeekingFood).unwrap_or(false);
                let target_food = vdata.and_then(|x| x.target_food);
                if seeking && market_has && target_food != Some(self.market_food) {
                    if let Some(vill) = world.get_component_mut::<Villager>(*v) {
                        vill.target_food = Some(self.market_food);
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
                let Some(p) = world.get_component_mut::<Pos>(*v) else { continue };
                let Some((hx, hy)) = dest else { continue };
                let (dx, dy) = (hx - p.x, hy - p.y);
                let d = (dx * dx + dy * dy).sqrt();
                // a hungry villager more than eating distance away walks the
                // road to the store - the sim's straight steps can pen them
                // against the town wall or a building for good, so take the
                // way round (an idle stray comes home the same way)
                let heading_home = seeking && dx.abs() + dy.abs() > 0.5
                    || !seeking && d > VILLAGE_RADIUS;
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
            self.advance_meeting();
            self.advance_travels();
            self.advance_minstrel();
            self.advance_pedlar();
            self.advance_circus();
            self.advance_ink_boiler();
            // and the rare events walk their roads
            self.advance_rare();
            // the flock grazes: the animals drift about the day's grazing point
            for (ai, &a) in self.flock.iter().enumerate() {
                if let Some(p) = self.world.get_component_mut::<Pos>(a) {
                    let (ox, oy) = (((ai * 37) % 9) as f32 - 4.0, ((ai * 53) % 7) as f32 - 3.0);
                    p.x += (self.graze_at.0 + ox * 0.5 - p.x) * 0.004;
                    p.y += (self.graze_at.1 + oy * 0.5 - p.y) * 0.004;
                }
            }
            // the yard hens scratch a slow wander round their own door
            for (hi, &a) in self.hens.iter().enumerate() {
                let hp = self.world.get_component::<Livestock>(a)
                    .and_then(|l| self.world.get_component::<Pos>(l.fold).copied());
                let (Some(hp), Some(p)) = (hp, self.world.get_component_mut::<Pos>(a)) else { continue };
                let t = self.frame_n as f32 / 50.0 + hi as f32 * 1.7;
                p.x += (hp.x + t.sin() * 1.4 + (hi % 3) as f32 * 0.4 - p.x) * 0.01;
                p.y += (hp.y + t.cos() * 1.0 + (hi % 2) as f32 * 0.4 - p.y) * 0.01;
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
        let pe = self.seats[seat].player; act_talk(&mut self.world, pe, v);
        self.say(v, &said, TALK_BUBBLE_SECS);
        // a blow-off leaves them hollow: a real talk coaches them back
        let coached = if self.world.get_component::<crate::game::actions::Demoralized>(v).is_some()
            && crate::game::actions::coach(&mut self.world, pe, v, self.tick) {
            " You talk the helplessness out of them."
        } else { "" };
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
        let msg = if let Some(reason) = self.action_blocker_at(seat, i) {
            format!("Can't {}: {}.", ACTION_LABELS[i].to_lowercase(), reason)
        } else {
            match i {
                0 => match self.start_approach_at(seat, selected_target, 0) {
                    Some(walking) => walking,
                    None => self.talk_to_at(seat, selected_target),
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
        self.log.push(format!("t{}: {}", tick, msg));
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
    /// belongs to the seat that called it.
    pub fn resolve_meeting(&mut self, invited: &[Entity], seat: usize) -> String {
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
        let founded = self.maybe_found_campaign(seat).map(|s| format!(" {}", s)).unwrap_or_default();
        format!("House meeting - {} came. {}{}{}", invited.len(), outcome, found, founded)
    }

    /// After a house meeting: enough living members and a heard issue and
    /// the org can take the founding vote. The organizer's ledger supplies
    /// the candidates; the campaign is born framed as a shared grievance.
    /// Returns a log line when the campaign forms.
    pub fn maybe_found_campaign(&mut self, seat: usize) -> Option<String> {
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
        let c = campaign::found_campaign(&mut self.world, org, &vote,
            campaign::Framing::SharedGrievance, campaign::DecisionRule::Majority, self.tick);
        self.world.add_component(c, campaign::CampaignMeeting {
            cadence_days: 3, last_held: self.tick, held: 0, resolves: 0 });
        self.toasts.push((format!("{} take a vote", oname), self.tick + 40));
        let mut msg = format!("{} take a vote - they'll fight for {}", oname, vote.winner);
        // they came along, but when the framing sits wrong on a stake they
        // grumble - heard in the log, not just counted on the ballots
        let grumbler = vote.ballots.iter().find(|b| b.grumbles).map(|b| b.member);
        if let Some(d) = grumbler {
            let name = self.villagers.iter().position(|&v| v == d).map(|i| vill_name(i)).unwrap_or("someone");
            msg += &format!(" - {} {}", name,
                campaign::grumble_line(campaign::Framing::SharedGrievance));
        }
        Some(msg)
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
    /// by a passing remark on the very next frame.
    pub fn say_in_passing(&mut self, who: Entity, text: &str, secs: u32) {
        let frame = self.frame_n;
        let blocked = self.bubbles.iter().any(|(w, _, until, _, passing)| *w == who && !*passing && frame < *until);
        if blocked { return; }
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), frame + secs * FPS, false, true));
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
        self.meeting = Some(Meeting { label, invited, spots, paths, frames: 0, talk_from: None, lines: vec![], seat });
        msg
    }

    /// What's said at a meeting: the player opens, each villager there says
    /// what's on their mind - now and then a secret slips out, and every
    /// parallel complaint already voiced makes the next speaker likelier
    /// to share their own. Three voices in a family surface an issue.
    pub fn meeting_lines(&mut self, came: &[Entity]) -> Vec<(Entity, String)> {
        if came.is_empty() { return vec![]; }
        let mut lines = vec![(self.player, MEETING_PROMPTS[0].to_string())];
        self.voiced.clear();
        for round in 0..2 {
            for &v in came {
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
                        let pe = self.player;
                if let Some(issue) = desires::hear(&mut self.world, pe, v, r, self.tick) {
                            self.toasts.push((format!("Organizer discovers issue {}", issue), self.tick + 40));
                            self.log.push(format!("t{}: Organizer discovers issue {}", self.tick, issue));
                        }
                        lines.push((v, said));
                        shared = true;
                    }
                }
                if !shared { lines.push((v, meeting_remark(&self.world, v, vill_name(i), round))); }
            }
            lines.push((self.player, MEETING_PROMPTS[round + 1].to_string()));
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
                    m.lines = self.meeting_lines(&m.invited);
                    m.talk_from = Some(m.frames);
                    // show the opening line right away - otherwise the first
                    // "since" tick after talk_from is 1, not 0, and the
                    // player's opening prompt (lines[0]) is skipped
                    if let Some((who, text)) = m.lines.first().cloned() { self.think(who, &text, LINE_FRAMES); }
                }
                self.meeting = Some(m);
            }
            Some(start) => {
                let (since, k) = (m.frames - start, ((m.frames - start) / LINE_FRAMES) as usize);
                if since % LINE_FRAMES == 0 && k < m.lines.len() {
                    let (who, text) = m.lines[k].clone();
                    self.think(who, &text, LINE_FRAMES); // gone as the next one speaks
                }
                if k < m.lines.len() { self.meeting = Some(m); return; }
                let msg = if m.invited.is_empty() { format!("No one made it to the house meeting at {}.", m.label) }
                    else { self.resolve_meeting(&m.invited, m.seat) };
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
            if n % 3 == 0 { self.say_in_passing(a, passing_remark(n), CHAT_BUBBLE_SECS); }
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
            Goal::Forage(_) | Goal::ForageShroom(_) => return false, // forages go through send_forage/send_shroom
            Goal::RunOddJob => self.world.get_component::<OddJob>(who)
                .and_then(|j| self.world.get_component::<Pos>(j.from).copied())
                .map(|p| (p.x.round() as i32, p.y.round() as i32)),
            Goal::Eat => {
                // the village table while it's laid: folk eat at their
                // own market, and only a bare shelf sends them hunting
                // for the next source
                let market_laid = self.world.get_component::<villager::FoodSource>(self.market_food)
                    .map(|f| f.servings > 0).unwrap_or(false);
                if market_laid {
                    self.world.get_component::<Pos>(self.market_food)
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
        self.ailment_tick(tick);
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
                    && bb.name != "the herb hut" && bb.name != "a foreign house")
                .unwrap_or(false));
        match room {
            Some(h) => self.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(v),
            None => desires::add_grievance(&mut self.world, v, desires::Problem::NoHome, tick + 60),
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
                if let Some(priest) = self.villagers.iter().copied().find(|&v|
                    self.world.get_component::<Craft>(v)
                        .map(|c| c.vocation == Vocation::Priest).unwrap_or(false)
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
                && !self.shunned(v))
            .collect();
        let store = self.market_food;
        for &v in &gathered {
            // everyone comes to market for the day
            self.travels.retain(|t| t.who != v);
            self.send(v, Goal::HeadHome(store));
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
            let stride = MEETING_WALK / trails::movement_cost(1.0, worn, TRAIL_K) * slow;
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
                        if matches!(good, "bread" | "faith" | "security" | "pamphlets" | "garments") {
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
                // a grant carried home lightens the round for the cause's
                // own: half the cut back, and the taxing doesn't count -
                // wherever the grant was won, not just seat zero's fight
                let granted = self.seats.iter().any(|s|
                    self.world.get_component::<crate::game::reactions::LordGrant>(s.org)
                        .map(|g| tick <= g.until && g.issue == "the taxes")
                        .unwrap_or(false)
                    && self.world.get_component::<Organization>(s.org)
                        .map(|o| o.members.contains(&target)).unwrap_or(false));
                if granted {
                    let back = taken * 0.5;
                    if let Some(p) = self.world.get_component_mut::<Purse>(target) { p.gold += back; }
                    crate::game::desires::lift_grievance(&mut self.world, target, crate::game::desires::Problem::Taxed);
                }
                self.say(target, "The lord's share, if you please.", CHAT_BUBBLE_SECS);
                if target == self.player {
                    self.log.push(format!("t{}: the tax collector takes his cut from you ({:.0} gold).", tick, taken));
                }
                if granted {
                    let name = self.villagers.iter().position(|v| *v == target).map(|i| vill_name(i)).unwrap_or("them");
                    self.log.push(format!("t{}: the lord's grant covers {} - half the cut comes back.", tick, name));
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
        self.herbologist_tick(tick);
        // the fold grazes, yields, and now and then howls
        self.livestock_tick(tick);
        // the heart-helper walks a lonely heart toward someone
        self.matchmaker_tick(tick);
        // a poor soul gathers in the wood, and damp years bring the fire
        self.mushroom_tick(tick);
        // the skeps yield, the church burns wax, a swarm wanders off
        self.apiary_tick(tick);
        // the larder keeps its own score: spoilage, vermin, the cat's job
        self.spoilage_tick(tick);
        // the dead hand on: benches, purses, and the lord's fifth
        self.succession_tick(tick);
        // and the young grow into their trades
        self.lifecycle_tick(tick);
        // the worn paths fade toward green where no one walks
        self.trail_tick();
        // the land keeps score: canopy, pit water, the reach
        self.land_tick(tick);
        // the ladder of shame climbs one rung at a time
        self.sanction_tick(tick);
        // and the year has its feast days
        self.festival_tick(tick);
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
        self.garden_tick(tick);
        // and the kitchen pot: staples off the garden beds feed the
        // hungry at home, and a week's peas and roots become pottage
        self.kitchen_tick(tick);
        // and the circus, while it camps, works its pull
        self.circus_tick(tick);
        // the oil boils, the crusts fry, and the black goes in
        self.ink_boiler_tick(tick);
        // and the hundred small fortunes: what's due comes on
        self.events_tick(tick);
        // and whatever the day caught, the fire runs on
        self.fire_tick(tick);
        // milk becomes wheels in the dark, and a noble buys the old ones
        self.cheese_tick(tick);
        // ale sours, wine turns, and the still owes the wise woman
        self.drink_tick(tick);
        // the day's work leaves its smell - unless the rain washes it off
        self.smell_tick(tick);
        // power atrophies; the lord's estimate creeps toward the truth
        self.power_tick(tick);
        // and now and then something rides over the walls
        self.sky_tick(tick);
        // and the soundscape's cues rise with the days
        self.sfx_tick(tick);
        // and a truth sits unsaid somewhere, ripening
        self.discovery_tick(tick);
        // and the castle sends its writs on a season's turn
        self.edict_tick(tick);
        // and every other winter runs long and tests the village
        self.winter_tick(tick);
        // and the painter lays color on the village's halls
        self.painter_tick(tick);
        // and now and then a maker makes a thing to outlast them
        self.heirloom_tick(tick);
        // and the literate put word to those they think of
        self.letter_tick(tick);
        // and theirs check on the ones who need checking on
        self.care_tick(tick);
        // and the river gives up its catch, season on season
        self.fisher_tick(tick);
        // and if the player's gone, the work goes on in another pair of hands
        self.continuation_tick(tick);
        // and the martyr's grave keeps its vigil
        self.martyr_tick(tick);
        // and the castle's prisoners wait on the org's answer
        self.dungeon_tick(tick);
        // and the posted knights stand only their tour
        self.guards_tick(tick);
        // and the marks a cause leaves fade on their own clock
        self.marks_tick(tick);
        let mut rng = Rng(self.rng.next());
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
                    let at_store = self.world.get_component::<Pos>(self.market_food)
                        .zip(self.world.get_component::<Pos>(v))
                        .map(|(s, p)| (s.x - p.x).abs() + (s.y - p.y).abs() <= 1.0)
                        .unwrap_or(false);
                    if at_store { self.arrive(v, Goal::Eat); }
                    else { self.send(v, Goal::Eat); }
                }
                continue;
            }
            if self.travels.iter().any(|t| t.who == v) { continue; }
            if self.world.get_component::<Kid>(v).is_some() { continue; }
            if self.world.get_component::<TownRole>(v).is_some() { continue; }
            // last night's cup still has them - they sleep it off
            if self.world.get_component::<villager::Drunk>(v)
                .map(|d| tick < d.until).unwrap_or(false) { continue; }
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
                if let Some(g) = self.want_shopping(v, tick) { self.send(v, g); continue; }
            }
            // far from home and far from your work? the day's done - walk back
            if let Some(home) = household::home_of(&self.world, v) {
                let hp = self.world.get_component::<Pos>(home).copied();
                let post = self.world.get_component::<Craft>(v).and_then(|c| c.workplace)
                    .and_then(|b| self.world.get_component::<Pos>(b).copied());
                if let (Some(hp), Some(p)) = (hp, self.world.get_component::<Pos>(v).copied()) {
                    let far_home = (p.x - hp.x).abs() + (p.y - hp.y).abs() > 6.0;
                    let far_post = post.map(|pp| (p.x - pp.x).abs() + (p.y - pp.y).abs() > 6.0).unwrap_or(true);
                    if far_home && far_post && self.send(v, Goal::HeadHome(home)) { continue; }
                }
            }
            // out of raw goods: send a kid, or go get it, or sit with the problem
            if let Some(input) = livelihood::needs_input(&self.world, v) {
                if !self.errand_for_input(v, input, i) {
                    desires::add_grievance(&mut self.world, v, Problem::NoInput(input), tick + 3);
                }
                continue;
            }
            // a day at the craft
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
        const WANT_GOODS: [(Want, &str); 10] = [
            (Want::GoodFood, "bread"), (Want::Finery, "garments"), (Want::Faith, "faith"),
            (Want::Security, "security"), (Want::Instrument, "lute"), (Want::BetterTools, "tools"),
            // the everyday pour first - the dearer the drink, the fewer
            // the purses that reach it
            (Want::Drink, "ale"), (Want::Drink, "beer"), (Want::Drink, "wine"),
            (Want::Drink, "spirits"),
        ];
        let life = self.world.get_component::<InnerLife>(v)?;
        let wants = life.wants.clone();
        let pos = *self.world.get_component::<Pos>(v)?;
        let gold = self.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
        for (w, good) in WANT_GOODS {
            if !wants.contains(&w) || desires::is_satisfied(&self.world, v, w, tick) { continue; }
            if gold < goods::worth_at(&self.world, good) { continue; }
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
                let (made, up) = { let pe = self.seats[seat].player; livelihood::work_day(&mut self.world, pe, tick) };
                let what = self.world.get_component::<Craft>(self.seats[seat].player)
                    .and_then(|c| vocation::output(c.vocation)).unwrap_or("goods");
                match up {
                    Some(l) => format!("You work - and reach {}!", vocation::level_name(l)),
                    None if made > 0.0 => format!("You work - {:.0} {} made.", made, what),
                    None => "You put in the day, but make nothing yet.".into(),
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
                        let mut msg = format!("{} meet about the campaign: {}", out.attendees.len(), who.join(", "));
                        if out.drifted { msg += " - run by whoever pulled hardest, nobody asked"; }
                        if out.noted { msg += " - the notes keep the resolves"; }
                        if out.consent { msg += " - consent banked for the next action"; }
                        if !out.healed.is_empty() {
                            let healed: Vec<String> = out.healed.iter()
                                .filter_map(|a| self.villagers.iter().position(|v| v == a).map(|i| vill_name(i).to_string())).collect();
                            msg += &format!(" - {} heard out", healed.join(", "));
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
                    Some(out) => match out.response {
                        TargetResponse::Carries => {
                            self.toasts.push(("the Reeve takes your demands to the king".into(), tick + 40));
                            // the carried demand lands as a grant: the load
                            // the issue names eases for the cause's people,
                            // and this year's taxing stops counting - but
                            // only the issue fought for is answered
                            let oe = self.seats[seat].org;
                            let issue = self.world.get_component::<crate::game::campaign::Campaign>(c)
                                .map(|cc| cc.issue.clone()).unwrap_or_default();
                            self.world.add_component(oe, crate::game::reactions::LordGrant {
                                issue: issue.clone(), until: tick + crate::game::desires::YEAR_DAYS });
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
                        TargetResponse::Deflects =>
                            format!("The Reeve sends regards and a clerk. {} went home hollow.", out.turnout.len()),
                        TargetResponse::Refuses =>
                            format!("The doors stay shut on {} voices. They go home hollow.", out.turnout.len()),
                    },
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
        let p = world.get_component::<PlayerCharacter>(self.seats[seat].player).unwrap();
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
            tick: self.tick, frame: self.frame_n,
            map: self.map.clone(), town_wall: self.town_wall,
            season, precip: self.precip, rainbow_from: self.rainbow_from,
            paused: self.paused, speed: self.speed,
            stones: self.stones.clone(), buildings, people, bubbles,
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
                name: p.name.clone(), standing: crate::game::player::standing(p.influence).to_string(),
                influence: p.influence, gold: p.resources,
                org: org.name.clone(), members, issues, guilds, food, campaign,
            },
            won: p.influence > WIN_INFLUENCE,
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
            debug_lines: self.telemetry_lines(),
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
            lines.push(Line { x: 18, y: 440, text: format!("{} \u{00b7} {}{}", nature, activity_text(vil.activity), cause), rgb: (200, 160, 160) });
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


/// Everything a frame needs that isn't the game itself: the sprite pack,
/// the camera and UI toggles, and the caches repainting costs too much to
/// redo every frame. One View per client (or per local player).
pub struct View {
    pub art: Art,
    pub cam_x: i32,
    pub cam_y: i32,
    pub zoom: f32,
    pub inspector_open: bool,
    pub help_open: bool,
    /// The ground layer repainted only when stale
    ground: std::cell::RefCell<Option<GroundCache>>,
    last_zoom: std::cell::Cell<f32>,
}

impl View {
    /// A view of the game drawn with `art`, looking at the market to start
    pub fn new(art: Art) -> View {
        View {
            art,
            cam_x: camera_on(16.0, 16.0).0,
            cam_y: camera_on(16.0, 16.0).1,
            zoom: 1.0,
            inspector_open: true,
            help_open: false,
            ground: std::cell::RefCell::new(None),
            last_zoom: std::cell::Cell::new(f32::NAN),
        }
    }

    /// The sprite a building draws with
    fn sprite_of(&self, b: BSprite) -> &Option<Sprite> {
        match b {
            BSprite::House(i) => self.art.house(i),
            BSprite::Blacksmith => &self.art.blacksmith_s,
            BSprite::Church => &self.art.church_s,
            BSprite::Guildhall => &self.art.guildhall_s,
            BSprite::RivalHouse => &self.art.rival_house,
            BSprite::RivalWoodhouse => &self.art.rival_woodhouse,
        }
    }

    /// The frame: ground, everything standing on it (in depth order),
    /// then the side panel. Draws the model the sim produced - the same
    /// code a remote client runs on the frame it gets over the wire.
    pub fn render(&self, m: &UiModel, mouse: (i32, i32)) -> Buffer {
        let (mx, my) = mouse;
        let art = &self.art;
        let (tick, cam_x, cam_y, zoom) = (m.tick, self.cam_x, self.cam_y, self.zoom);

        let mut buf = Buffer::new(W, H);
        buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);

        // terrain rendered in the SAME projection as entities (unified map).
        // The ground is thousands of tile blits, so it's painted once into a
        // world-space layer and each frame just blits the window the camera
        // shows; it's redrawn only when the zoom, the season or the map
        // itself has changed since the last pass.
        let season = m.season;
        if self.last_zoom.get() != zoom {
            // a held zoom key changes zoom every frame: rebuilding the layer
            // each frame would draw the whole map per frame, so while it's
            // moving just paint the visible tiles like the pass always did;
            // the cache rebuilds once the zoom settles
            self.draw_ground(m, &mut buf, cam_x, cam_y, zoom);
        } else {
            let stale = match *self.ground.borrow() {
                Some(ref g) => g.zoom != zoom || g.season != season || g.rev != m.map.rev(),
                None => true,
            };
            if stale { *self.ground.borrow_mut() = Some(self.render_ground(m, zoom)); }
            let g = self.ground.borrow();
            let g = g.as_ref().unwrap();
            // the cache pixel under screen (x, y): the layer's world-space
            // origin shifted by the camera pan (cam offsets are unzoomed,
            // so on screen they move cam * zoom px)
            let (ox, oy) = ((cam_x as f32 * zoom).round() as i32, (cam_y as f32 * zoom).round() as i32);
            buf.blit_from(&g.buf, g.origin.0 - ox, g.origin.1 - oy, 0, 0, MAP_W as i32, H as i32);
        }
        self.last_zoom.set(zoom);

        // everything standing on the map, people included, drawn back to
        // front in one painter's-order pass: whatever is nearer covers what
        // is behind it (people used to be painted over the lot and the
        // occluders re-drawn on top, which flipped nearer pieces back behind
        // farther ones - glitched seams, e.g. on the town wall)
        #[derive(Clone, Copy)]
        enum Piece<'a> {
            Building(Option<&'a Sprite>, f32), // a house, and how much of it stands
            Prop(Option<&'a Sprite>),          // a tree or boulder in the middle of its tile
            Castle(Option<&'a Sprite>),        // a town-wall block: wall, gatehouse or tower
            Herb,                              // a wild herb tuft, flat on the meadow
            Shroom,                            // a wild mushroom in the shade
            Skep,                              // a woven beehive in the apiary's yard
            Swarm,                             // a swarm of bees on the wing
        }
        #[derive(Clone, Copy)]
        enum Who<'a> { P(&'a PersonV), Mill, Smoke(usize) }

        // a workplace nobody works stands dimmed - derelict
        let tint_cache: std::collections::HashMap<usize, Option<Sprite>> = m.buildings.iter().enumerate()
            .filter(|(_, b)| b.derelict)
            .map(|(i, b)| (i, self.sprite_of(b.sprite).as_ref().map(|sp| tint(sp, (40, 40, 55), 0.6))))
            .collect();
        let mut standing: Vec<(f32, f32, Piece)> = vec![];
        for (i, b) in m.buildings.iter().enumerate() {
            let s = self.sprite_of(b.sprite);
            let sprite = if b.derelict { tint_cache[&i].as_ref().or(s.as_ref()) } else { s.as_ref() };
            standing.push((b.x, b.y, Piece::Building(sprite, b.risen)));
        }
        // a wood is a mix of the pack's trees, stable per tile
        standing.extend(m.map.trees.iter().map(|&(x, y)| (x as f32, y as f32, Piece::Prop(pick(&art.tiles[season_tiles(season)].trees, x, y, 4)))));
        // boulders and flat rocks, a different one each time
        let rock_prop = |x: i32, y: i32| if noise(x, y, 14) < 0.5 {
            pick(&art.stones, x, y, 16) } else { pick(&art.stone_grounds, x, y, 16) };
        standing.extend(m.stones.iter().map(|&(x, y)| (x as f32, y as f32, Piece::Prop(rock_prop(x, y)))));
        // wild herb tufts, ready for the picking
        standing.extend(m.herbs.iter().map(|&(x, y)| (x, y, Piece::Herb)));
        standing.extend(m.mush.iter().map(|&(x, y)| (x, y, Piece::Shroom)));
        standing.extend(m.skeps.iter().map(|&(x, y)| (x, y, Piece::Skep)));
        if let Some((wx, wy)) = m.swarm { standing.push((wx, wy, Piece::Swarm)); }
        // the land's score shows: stumps where the canopy went, the foul
        // reach below town, the pit filled to a pond
        for &(tx, ty) in &m.stumps {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(1.0), sy - k(4.0), k(2.0), k(4.0), 95, 75, 50);
            buf.rect(sx - k(2.0), sy - k(5.0), k(4.0), k(1.0), 120, 95, 65);
        }
        for &(tx, ty) in &m.murk {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 85, 70, 45);
        }
        if let Some((tx, ty)) = m.pond {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 60, 90, 120);
        }
        // something riding over the walls: a bright head and a fading
        // tail across the top of the frame
        if let Some(kind) = m.sky.as_deref() {
            let (hx, hy) = (buf.width as i32 * 2 / 3, buf.height as i32 / 8);
            let head = if kind == "comet" { (255, 245, 200) } else { (180, 60, 50) };
            for t in 0..14 {
                let (x, y) = (hx - t * 6, hy - t * 3);
                let f = 1.0 - t as f32 / 14.0;
                let (r, g, b) = ((head.0 as f32 * f) as u8, (head.1 as f32 * f) as u8, (head.2 as f32 * f) as u8);
                if x >= 0 && y >= 0 { buf.rect(x - 2, y - 1, 4, 3, r, g, b); }
            }
            buf.rect(hx - 3, hy - 3, 6, 6, 255, 250, 230);
        }
        // the cave mouth: a dark hollow at the map's edge, desaturated
        if let Some((cx, cy)) = m.cave {
            let (sx, sy) = iso_to_screen_cam(cx, cy, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(6.0), sy - k(6.0), k(12.0), k(7.0), 18, 18, 22);
            buf.rect(sx - k(4.0), sy - k(4.0), k(8.0), k(5.0), 8, 8, 10);
            buf.rect(sx - k(7.0), sy - k(7.0), k(14.0), k(1.0), 40, 40, 45);
        }
        // the painted halls: a bright trim above the door
        for &(tx, ty) in &m.painted {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(9.0), k(8.0), k(2.0), 200, 160, 40);
            buf.rect(sx - k(4.0), sy - k(9.0), k(2.0), k(5.0), 160, 60, 140);
            buf.rect(sx + k(2.0), sy - k(9.0), k(2.0), k(5.0), 60, 100, 160);
        }
        // the garden beds: soil strip, greener as it fills in
        for &(tx, ty, g) in &m.gardens {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(3.0), sy - k(1.0), k(6.0), k(3.0), 110, 85, 55);
            if g > 0.4 {
                buf.rect(sx - k(2.0), sy - k(2.0), k(4.0), k(1.0), 80, (120.0 + 60.0 * g) as u8, 60);
            }
            if g >= 1.0 {
                buf.rect(sx - k(1.0), sy - k(3.0), k(2.0), k(1.0), 220, 180, 80);
            }
        }
        // the scorched ground: dark scars where the fire's been
        for &(tx, ty) in &m.scorched {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 30, 26, 24);
        }
        // and what's alight: a flame's worth of colour over the tile
        for &(tx, ty) in &m.burning {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            let lick = (m.frame % 9 < 5) as i32; // it gutters
            buf.rect(sx - k(4.0), sy - k(2.0 + lick as f32), k(8.0), k(5.0 + lick as f32), 160, 45, 20);
            buf.rect(sx - k(2.0), sy - k(3.0 + lick as f32), k(4.0), k(4.0), 235, 150, 40);
        }
        // the desire-lines: where feet keep wearing, the ground shows it
        for &(tx, ty, s) in &m.trails {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            let shade = (30.0 * s) as u8;
            buf.rect(sx - k(3.0), sy - k(1.0), k(6.0), k(2.0),
                95u8.saturating_sub(shade), 75u8.saturating_sub(shade), 55u8.saturating_sub(shade));
        }
        let rocks = (0..m.map.height as i32).flat_map(|y| (0..m.map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| m.map.get(x, y) == Some(Tile::Rock));
        standing.extend(rocks.map(|(x, y)| (x as f32, y as f32, Piece::Prop(rock_prop(x, y)))));
        standing.extend(m.map.walls.iter().map(|&((x, y), piece)| {
            // the odd stretch of wall has a postern door in it
            let postern = noise(x, y, 15) < 0.15;
            let sprite = match piece {
                WallPiece::AlongX => art.wall_posts[0].as_ref().filter(|_| postern).or(art.walls[0].as_ref()),
                WallPiece::AlongY => art.wall_posts[1].as_ref().filter(|_| postern).or(art.walls[1].as_ref()),
                WallPiece::GateAlongX => art.walls[2].as_ref(),
                WallPiece::GateAlongY => art.walls[3].as_ref(),
                WallPiece::Tower => art.tower_s.as_ref(),
            };
            (x as f32, y as f32, Piece::Castle(sprite))
        }));
        standing.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());
        let draw_piece = |buf: &mut Buffer, &(bx, by, piece): &(f32, f32, Piece)| {
            let (sx, sy) = iso_to_screen_cam(bx, by, cam_x, cam_y, zoom);
            match piece {
                Piece::Prop(sprite) => if let Some(s) = sprite { draw_prop(buf, s, sx, sy, zoom); },
                Piece::Herb => herb_tuft(buf, sx, sy, zoom),
                Piece::Shroom => shroom_tuft(buf, sx, sy, zoom),
                Piece::Skep => draw_skep(buf, sx, sy, zoom),
                Piece::Swarm => draw_swarm(buf, sx, sy, zoom),
                Piece::Castle(sprite) => if let Some(s) = sprite { draw_castle(buf, s, sx, sy, zoom); },
                Piece::Building(sprite, up) => if let Some(s) = sprite {
                    if up >= 1.0 { draw_on_tile(buf, s, sx, sy, zoom); }
                    else if up > 0.0 {
                        // going up: the bottom of the house, from the ground up
                        let rows = ((s.height as f32 * up).round() as usize).max(1);
                        draw_on_tile(buf, &s.crop(0, s.height - rows, s.width, rows), sx, sy, zoom);
                    }
                },
            }
        };

        // people share the pass, along with the windmill and the chimney
        // smoke, which stand where their tower and houses stand
        let mut people: Vec<(f32, f32, Who)> = vec![];
        if art.mill_tower.is_some() {
            if let Some((mx_, my_)) = m.mill { people.push((mx_, my_, Who::Mill)); }
        }
        if art.house(0).is_some() {
            for (i, b) in m.buildings.iter().enumerate().filter(|(_, b)| b.smoke).take(3) {
                people.push((b.x, b.y, Who::Smoke(i)));
            }
        }
        people.extend(m.people.iter().filter(|p| p.kind != PKind::Gull).map(|p| (p.pos.0, p.pos.1, Who::P(p))));
        people.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());

        // market food stock: shown in the HUD (top-left), not on the market
        // tile, where villagers always gather to eat and covered it up
        let food_stock = m.panel.food;


        // name labels are drawn after every sprite (so no sprite covers one)
        // and de-overlapped: (text, x, y, colour, hungry, selected)
        let mut labels: Vec<(String, i32, i32, (u8, u8, u8), bool, bool)> = vec![];
        let mut standing = standing.iter().peekable();
        for &(px, py, who) in &people {
            while let Some(st) = standing.next_if(|st| st.0 + st.1 <= px + py) { draw_piece(&mut buf, st); }
            match who {
                // windmill: the tower on its tile, its sails turning on the
                // tower's face (frame cycles every 10 ticks)
                Who::Mill => {
                    if let Some(tower) = &art.mill_tower {
                        let (msx, msy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                        draw_on_tile(&mut buf, tower, msx, msy, zoom);
                        if let Some(sails) = art.mill_frames.get(((tick / 10) % 4) as usize) {
                            let (hx, hy) = mill_hub(tower, msx, msy, zoom);
                            let k = MILL_SAILS * 2.0 * TILE_W as f32 * zoom / tower.width as f32;
                            let (w, h) = ((sails.width as f32 * k).round() as i32, (sails.height as f32 * k).round() as i32);
                            draw_zoomed(&mut buf, sails, hx - w / 2, hy - h / 2, k);
                        }
                    }
                },
                // chimney smoke: a few small puffs rising off each home's
                // roof, drifting with the wind and thinning out (the pack's
                // smoke sprite is a cluster of balls, which just read as
                // circles on the roof)
                Who::Smoke(i) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    for (x, y, size, grey) in smoke_puffs(m.frame + i as u32 * 37, sx, sy, zoom) {
                        buf.rect(x, y, size, size, grey, grey, grey + 6);
                    }
                },
                Who::P(pv) if let PKind::Villager(i) = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    // the dead get a gravestone where they fell, not their standing figure
                    if pv.dead {
                        let top = draw_grave(&mut buf, sx, sy, zoom);
                        let (lx, _) = name_label_pos(sx, sy, vill_name(i));
                        labels.push((vill_name(i).into(), lx, top - GLYPH_H - LABEL_GAP, (160, 160, 170), false, pv.selected));
                    } else {
                        let fig = &art.villager_sprites[i % art.villager_sprites.len().max(1)];
                        // children draw small and flagged: the pack has no
                        // figure for them, so they're a scaled villager
                        if pv.kid {
                            draw_kid(&mut buf, fig, sx, sy, zoom);
                        } else {
                            draw_character(&mut buf, fig, sx, sy, zoom);
                        }
                        let lbl = pv.label.clone().unwrap_or_else(|| vill_name(i).into());
                        let (lx, ly) = name_label_pos(sx, sprite_label_anchor(fig, sy, zoom), &lbl);
                        // bright when close enough to talk, recruit or meet; dim otherwise
                        let rgb = if pv.in_reach { NAME_IN_REACH } else { NAME_OUT_OF_REACH };
                        labels.push((lbl, lx, ly, rgb, pv.hungry, pv.selected));
                        // sick: a green cross on the other side of the head
                        if pv.sick {
                            let (mk, (r, g, b), _) = MARK_SICK;
                            draw_text_outlined(&mut buf, sx - 16, head_y(sy, zoom), mk, r, g, b);
                        }
                        // activity marker beside the head
                        if pv.act_mark > 0 {
                            let (mk, (r, g, b), _) = if pv.act_mark == 1 { MARK_FOOD } else { MARK_BUSY };
                            draw_text_outlined(&mut buf, sx + 10, head_y(sy, zoom), mk, r, g, b);
                        }
                    }
                },
                Who::P(pv) if matches!(pv.kind, PKind::Sheep | PKind::Goat) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    draw_beast(&mut buf, sx, sy, zoom, pv.kind == PKind::Sheep);
                },
                Who::P(pv) if let PKind::Pet(species) = pv.kind => {
                    let sprite = match species { Species::Dog => &self.art.dog, Species::Cat => &self.art.cat, _ => &NO_SPRITE };
                    let (dx, dy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    // standing on its spot, drawn at the villagers' scale
                    let h = match sprite {
                        Some(s) => {
                            let (w, h) = ((s.width as f32 * 2.0 * zoom) as i32, (s.height as f32 * 2.0 * zoom) as i32);
                            draw_zoomed(&mut buf, s, dx - w / 2, dy - h, 2.0 * zoom);
                            h
                        }
                        None => 0,
                    };
                    if pv.bond {
                        labels.push(("<3".into(), dx - 6, dy - h - GLYPH_H - LABEL_GAP, (255, 120, 120), false, false)); // placed with the names
                    }
                },
                // the watch: soldiers from the humans pack at the gates
                Who::P(pv) if let PKind::Guard(i) = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if !art.guards.is_empty() {
                        draw_character(&mut buf, &art.guards[i % art.guards.len()], sx, sy, zoom);
                    }
                },
                Who::P(pv) if let PKind::Priest = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(s) = &art.priest_s { draw_character(&mut buf, s, sx, sy, zoom); }
                },
                Who::P(pv) if let PKind::Traveller = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(s) = &art.traveller_s { draw_character(&mut buf, s, sx, sy, zoom); }
                },
                Who::P(pv) if let PKind::Chicken = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(c) = &art.chicken {
                        let (w, h) = ((c.width as f32 * 2.0 * zoom) as i32, (c.height as f32 * 2.0 * zoom) as i32);
                        draw_zoomed(&mut buf, c, sx - w / 2, sy - h, 2.0 * zoom);
                    }
                },
                // rival villagers (foreign settlement): the outlined figures
                Who::P(pv) if let PKind::Rival(i) = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    let anchor = match art.rival_sprites.get(i) {
                        Some(rs) => { draw_character(&mut buf, rs, sx, sy, zoom); sprite_label_anchor(rs, sy, zoom) }
                        None => label_anchor(sy, zoom),
                    };
                    let (lx, ly) = name_label_pos(sx, anchor, RIVAL_NAMES[i]);
                    labels.push((RIVAL_NAMES[i].into(), lx, ly, (255, 140, 140), false, false));
                },
                // player sprite at their actual position
                Who::P(pv) if let PKind::Player = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(ps) = &art.player_s {
                        draw_character(&mut buf, ps, sx, sy, zoom);
                        // gold ring under player
                        buf.rect(sx - 12, sy - 4, 24, 3, 255, 215, 0);
                    } else {
                        draw_rect(&mut buf, sx - 4, sy - 8, 8, 8, 255, 255, 255);
                    }
                    let anchor = art.player_s.as_ref().map(|ps| sprite_label_anchor(ps, sy, zoom)).unwrap_or(label_anchor(sy, zoom));
                    let (lx, ly) = name_label_pos(sx, anchor, "YOU");
                    labels.push(("YOU".into(), lx, ly, (255, 255, 0), false, false));
                },
                Who::P(_) => {},
            }
        }
        for st in standing { draw_piece(&mut buf, st); }

        // gulls wheel over the open sea
        if let Some(gull) = &art.gull {
            for (i, pv) in m.people.iter().filter(|p| p.kind == PKind::Gull).enumerate() {
                let (gx, gy) = pv.pos;
                let t = m.frame as f32 * 0.05 + i as f32 * 2.1;
                let (sx, sy) = iso_to_screen_cam(gx + t.cos() * 0.6, gy + t.sin() * 0.3, cam_x, cam_y, zoom);
                draw_zoomed(&mut buf, gull, sx - gull.width as i32, sy - (14.0 * zoom) as i32, 2.0 * zoom);
            }
        }

        // all name labels, on top of every sprite, nudged apart where they'd overlap
        let boxes: Vec<(i32, i32, i32)> = labels.iter()
            .map(|(t, x, y, _, hungry, _)| (*x, *y, t.len() as i32 * GLYPH_W + if *hungry { GLYPH_W + 2 } else { 0 }))
            .collect();
        let placed = place_labels(&boxes);
        // where each name finally sits, so a speech bubble can clear it
        let label_ys: Vec<(&String, i32)> = labels.iter().zip(&placed).map(|(l, y)| (&l.0, *y)).collect();
        for ((text, x, _, (r, g, b), hungry, selected), y) in labels.iter().zip(placed) {
            draw_text_outlined(&mut buf, *x, y, text, *r, *g, *b);
            // hunger indicator, right after the name
            if *hungry { let (m, (r, g, b), _) = MARK_HUNGRY; draw_text_outlined(&mut buf, x + text.len() as i32 * GLYPH_W + 2, y, m, r, g, b); }
            // selection: underline the selected villager's name
            if *selected { draw_rect(&mut buf, x - 1, y + GLYPH_H + 1, text.len() as i32 * GLYPH_W + 1, 2, 255, 255, 0); }
        }

        // --- UI PANEL ---
        draw_rect(&mut buf, UI_X as i32, 0, W as i32 - UI_X as i32, H as i32, 32, 32, 48);
        draw_text_scaled(&mut buf, UI_X as i32 + 10, 8, "THE WANDERER", 2, 255, 220, 130);

        // status: influence bar, members, season, tick
        let p = &m.panel;
        let guild = Some(m.panel.guilds).filter(|g| *g > 0);
        // influence bar runs to the win: reach WIN_INFLUENCE and the village
        // follows you; the tick marks where recruiting starts to work
        const BAR_W: i32 = 120;
        draw_text(&mut buf, UI_X as i32 + 10, 30, "Influence", 200, 200, 200);
        draw_rect(&mut buf, UI_X as i32 + 70, 30, BAR_W, 10, 40, 40, 60);
        let infl_w = ((p.influence / WIN_INFLUENCE).clamp(0.0, 1.0) * BAR_W as f32) as i32;
        draw_rect(&mut buf, UI_X as i32 + 70, 30, infl_w, 10, 255, 200, 80);
        let recruit_x = UI_X as i32 + 70 + ((recruit_influence_needed() / WIN_INFLUENCE) * BAR_W as f32) as i32;
        draw_rect(&mut buf, recruit_x, 28, 1, 14, 120, 255, 120);
        draw_text(&mut buf, UI_X as i32 + 70 + BAR_W + 6, 31, &format!("{:.2}/{:.1} lead", p.influence, WIN_INFLUENCE), 255, 220, 130);
        draw_text(&mut buf, UI_X as i32 + 10, 44, &status_line(&p.org, p.members.len(), p.gold, guild), 200, 200, 200);

        // buttons
        for (i, b) in action_buttons().iter().enumerate() {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            let reason = m.blocked[i].as_deref();
            draw_button(&mut buf, b, hover, reason.is_some());
            // a greyed-out button always says why; others explain themselves on hover
            if let Some(r) = &reason {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, r, 200, 140, 120);
            } else if hover {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, TOOLTIPS[i], 220, 220, 160);
            }
        }

        // inspector: a villager's card, or a building's when one's clicked
        // inspector: a villager's card, or a building's when one's clicked
        if self.inspector_open {
            draw_rect(&mut buf, UI_X as i32 + 10, 316, 270, 140, 45, 45, 65);
            if let Some(insp) = &m.inspector {
                if insp.dead {
                    draw_grave(&mut buf, PORTRAIT_FOOT.0, PORTRAIT_FOOT.1, 2.0);
                } else if let Some(fig_i) = insp.fig {
                    let fig = &art.villager_sprites[fig_i % art.villager_sprites.len().max(1)];
                    fig.blit_scaled(&mut buf, UI_X as i32 + 6, 322, 2);
                }
                for line in &insp.lines {
                    if line.text.starts_with('\u{1}') {
                        // "METER:name:frac:note" sentinel from the sim
                        let mut it = line.text[3..].split('\u{2}');
                        let (name, rest) = (it.next().unwrap_or(""), it.next().unwrap_or(""));
                        let (frac, note) = rest.split_once(':')
                            .map(|(a, b)| (a.parse::<f32>().unwrap_or(0.0), b))
                            .unwrap_or((0.0, ""));
                        draw_meter(&mut buf, UI_X as i32 + line.x, line.y, name, frac, note);
                    } else {
                        draw_text(&mut buf, UI_X as i32 + line.x, line.y, &line.text, line.rgb.0, line.rgb.1, line.rgb.2);
                    }
                }
            }
        }

        // contextual actions: after the inspector, so the ones that live on
        // the card's bottom edge (Seat/Strike) stay visible. Meet sits on
        // the campaign's FIGHT line, the rest under the main buttons
        // two passes: the buttons first, then any greyed reason on hover -
        // drawn last so a neighbouring button or status line can't cover it
        let ctx_btns: Vec<(Button, Option<&String>)> = m.ctx.iter().map(|a| ctx_button(*a))
            .zip(m.ctx_blocked.iter()).map(|(b, r)| (b, r.as_ref())).collect();
        for (b, reason) in &ctx_btns {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            draw_button(&mut buf, b, hover, reason.is_some());
        }
        for (b, reason) in &ctx_btns {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            if let Some(r) = reason.filter(|_| hover) {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 4, r, 255, 180, 140);
            }
        }

        // organization: who has joined, by name
        let member_names: Vec<&str> = m.panel.members.iter().map(|s| s.as_str()).collect();
        draw_text(&mut buf, UI_X as i32 + 18, 464, MEMBERS_LABEL, 255, 215, 0);
        draw_text(&mut buf, UI_X as i32 + 18 + (MEMBERS_LABEL.len() as i32 + 1) * GLYPH_W, 464, &members_line(&member_names), 220, 220, 180);

        // the issues the organizer has pieced together
        draw_text(&mut buf, UI_X as i32 + 18, 476, "ISSUES", 255, 215, 0);
        let issue_line = if m.panel.issues.is_empty() { "none heard yet".to_string() } else { m.panel.issues.join("; ") };
        for (n, line) in wrap_text(&issue_line, 42).iter().take(if m.panel.campaign.is_some() { 1 } else { 2 }).enumerate() {
            draw_text(&mut buf, UI_X as i32 + 18 + (7 * GLYPH_W), 476 + n as i32 * 11, line, 220, 190, 140);
        }
        // the campaign the org fights for, once it's taken its vote - the
        // Meet button (a ctx action) sits to the right of the issue line
        if let Some(c) = &m.panel.campaign {
            draw_text(&mut buf, UI_X as i32 + 18, 488,
                &format!("FIGHT {}", c.issue), 255, 180, 120);
            let due = if c.due { "meet due" } else { "meet to come" };
            // cadence leads: it's the bit you set - the seats can trail off
            draw_text(&mut buf, UI_X as i32 + 18, 500,
                &format!("every {}d - {} - {}", c.cadence_days, due, c.seats.join(" ")), 120, 120, 140);
        } else {
            draw_text(&mut buf, UI_X as i32 + 18, 500, PANEL_HINT, 120, 120, 140);
        }

        // event log
        draw_rect(&mut buf, UI_X as i32 + 10, 512, 270, 110, 25, 25, 40);
        draw_text(&mut buf, UI_X as i32 + 14, 516, "LOG", 150, 150, 180);
        // newest first, word-wrapped to the box instead of cut off mid-word
        const LOG_CHARS: usize = 43;      // (box width - padding) / glyph width
        const LOG_ROWS: usize = 8;
        let mut row = 0;
        'log: for entry in &m.log {
            let (r, g, b) = log_colour(entry);
            // "day 120" in dim grey, then the message in its colour
            let (day, msg) = log_parts(entry);
            let text = match &day { Some(d) => format!("{} {}", d, msg), None => msg.to_string() };
            for (n, line) in wrap_text(&text, LOG_CHARS).iter().enumerate() {
                if row == LOG_ROWS { break 'log; }
                let y = 530 + row as i32 * 12;
                match &day {
                    Some(d) if n == 0 && line.starts_with(d.as_str()) => {
                        draw_text(&mut buf, UI_X as i32 + 14, y, d, 120, 120, 145);
                        draw_text(&mut buf, UI_X as i32 + 14 + (d.len() as i32 + 1) * GLYPH_W, y, line[d.len()..].trim_start(), r, g, b);
                    }
                    _ => { draw_text(&mut buf, UI_X as i32 + 14, y, line, r, g, b); }
                }
                row += 1;
            }
        }

        // what the marks over villagers' heads mean, under the log
        let mut mx = UI_X as i32 + 14;
        for (m, (r, g, b), word) in MAP_MARKS {
            draw_text(&mut buf, mx, LEGEND_Y, m, r, g, b);
            mx += (m.len() as i32 + 1) * GLYPH_W;
            draw_text(&mut buf, mx, LEGEND_Y, word, 150, 150, 170);
            mx += (word.len() as i32 + 2) * GLYPH_W;
        }

        // season change: subtle banner notification instead of flash
        if tick > 10 && tick % 90 < 3 {
            let sname = match season { Season::Spring => "Spring", Season::Summer => "Summer", Season::Autumn => "Autumn", Season::Winter => "Winter" };
            let banner = format!("{} arrives", sname);
            let bw = banner.len() as i32 * GLYPH_W + 16;
            buf.rect(MAP_W as i32 / 2 - bw / 2, 56, bw, 16, 20, 20, 35);
            draw_text(&mut buf, MAP_W as i32 / 2 - bw / 2 + 8, 60, &banner, 255, 220, 130);
        }


        // weather over the map: overcast, falling rain or snow, the rainbow after rain
        draw_weather(&mut buf, m.precip, m.frame as u64, m.rainbow_from.map(|f| (m.frame - f) as u64));

        // minimap (top-right of map area): the map in miniature, same projection
        // and colours, with villagers, the player and the visible area marked
        buf.rect(MM_LEFT - 2, MM_TOP - 2, MM_W + 4, MM_H + 4, 20, 20, 30);
        let (w_t, h_t) = (m.map.width as f32, m.map.height as f32);
        let k = mm_k(w_t, h_t);
        // the woods as a flat grid for the per-pixel lookup below (a
        // contains() on the Vec walked the whole woods for every pixel)
        let mut tree_at = vec![false; m.map.width * m.map.height];
        for &(tx, ty) in &m.map.trees { tree_at[ty as usize * m.map.width + tx as usize] = true; }
        for my in 0..MM_H { for mx in 0..MM_W {
            if let Some((tx, ty)) = minimap_tile((MM_LEFT + mx) as f32 + 0.5, (MM_TOP + my) as f32 + 0.5, w_t, h_t) {
                let (r, g, b) = m.map.get(tx as i32, ty as i32).map(|t| tile_colour(t, season)).unwrap_or((20, 20, 30));
                // the woods show darker
                let (r, g, b) = if tree_at[ty as usize * m.map.width + tx as usize] { blend((r, g, b), (20, 70, 35), 0.5) } else { (r, g, b) };
                buf.set(MM_LEFT + mx, MM_TOP + my, r, g, b);
            }
        }}
        let mm_dot = |buf: &mut Buffer, tx: f32, ty: f32, r: u8, g: u8, b: u8| {
            let (x, y) = tile_to_minimap(tx, ty, k);
            buf.rect(x as i32 - 1, y as i32 - 1, 2, 2, r, g, b);
        };
        for pv in &m.people {
            match pv.kind {
                PKind::Villager(_) => {
                    // the dead are graves now: grey, not a living red villager
                    let (r, g, b) = if !pv.dead { MM_VILLAGER } else { MM_GRAVE };
                    mm_dot(&mut buf, pv.pos.0, pv.pos.1, r, g, b);
                }
                PKind::Rival(_) => mm_dot(&mut buf, pv.pos.0, pv.pos.1, 255, 160, 60),
                PKind::Player => mm_dot(&mut buf, pv.pos.0, pv.pos.1, 255, 255, 0),
                _ => {}
            }
        }
        // outline of what the map view is showing, clipped to the minimap box
        let (vx0, vy0) = screen_to_minimap(0.0, 0.0, cam_x, cam_y, zoom, w_t, h_t);
        let (vx1, vy1) = screen_to_minimap(MAP_W as f32, H as f32, cam_x, cam_y, zoom, w_t, h_t);
        let (x0, x1) = ((vx0 as i32).max(MM_LEFT), (vx1 as i32).min(MM_LEFT + MM_W - 1));
        let (y0, y1) = ((vy0 as i32).max(MM_TOP), (vy1 as i32).min(MM_TOP + MM_H - 1));
        if x0 < x1 && y0 < y1 {
            buf.rect(x0, y0, x1 - x0 + 1, 1, 255, 255, 255);
            buf.rect(x0, y1, x1 - x0 + 1, 1, 255, 255, 255);
            buf.rect(x0, y0, 1, y1 - y0 + 1, 255, 255, 255);
            buf.rect(x1, y0, 1, y1 - y0 + 1, 255, 255, 255);
        }

        // speech bubbles over whoever is speaking
        for bv in &m.bubbles {
            let (text, thought) = (bv.text.as_str(), bv.thought);
            {
                {
                    let (sx, sy) = iso_to_screen_cam(bv.pos.0, bv.pos.1, cam_x, cam_y, zoom);
                    // off the map view (e.g. under the panel): the log already
                    // has what they said, so don't float a bubble over the panel
                    // the drawn label carries the marks - the plain name
                    // is its prefix, not the whole string
                    let label_y = bv.name.as_deref().and_then(|n| label_ys.iter().find(|(t, _)| t.starts_with(n)).map(|(_, y)| *y));
                    if let Some((bx, by, bw)) = bubble_rect(sx, sy, zoom, text, label_y) {
                        let tail_x = (sx - 4).clamp(bx + 2, bx + bw - 10);
                        if thought {
                            // a thought: pale blue, its corners rounded off, a
                            // trail of shrinking puffs down to the thinker
                            let c = (225, 235, 250);
                            buf.rect(bx + 2, by, bw - 4, BUBBLE_H, c.0, c.1, c.2);
                            buf.rect(bx, by + 2, bw, BUBBLE_H - 4, c.0, c.1, c.2);
                            buf.rect(tail_x + 2, by + BUBBLE_H + 1, 4, 4, c.0, c.1, c.2);
                            buf.rect(tail_x + 4, by + BUBBLE_H + 6, 2, 2, c.0, c.1, c.2);
                        } else {
                            buf.rect(bx, by, bw, BUBBLE_H, 240, 240, 220);        // cream bubble
                            buf.rect(tail_x, by + BUBBLE_H - 2, 8, 6, 240, 240, 220); // tail, under the speaker
                        }
                        draw_text(&mut buf, bx + 6, by + 5, text, 40, 30, 20);
                    }
                }
            }
        }

        // toasts (big moments)
        let mut ty = 40;
        for msg in m.toasts.iter() {
            {
                let tw = msg.len() as i32 * 6 + 16;
                buf.rect(MAP_W as i32 / 2 - tw / 2, ty, tw, 20, 20, 20, 35);
                buf.rect(MAP_W as i32 / 2 - tw / 2, ty, tw, 1, 255, 215, 0);
                draw_text(&mut buf, MAP_W as i32 / 2 - tw / 2 + 8, ty + 6, msg, 255, 220, 130);
                ty += 26;
            }
        }

        // season banner
        let hud = hud_text(season, tick, food_stock, m.precip.map(|k| k.label()), m.paused, m.speed);
        // on a dark strip so it reads over any terrain the camera shows
        buf.rect(4, 4, text_width(&hud, 1) + 12, GLYPH_H + 8, 20, 20, 32);
        draw_text_outlined(&mut buf, 10, 8, &hud, 255, 255, 255);

        // help overlay
        if self.help_open { draw_help(&mut buf); }

        // pause menu overlay
        if m.paused {
            draw_overlay(&mut buf, PAUSE_W, "PAUSED", (255, 255, 255), &PAUSE_LINES, PAUSE_ROW, None);
        }

        buf
    }
    /// The ground pass: every tile's block (or flat diamond where there's no
    /// art) drawn back to front so nearer blocks cover farther ones, plus the
    /// odd tuft or flower on open grass. Drawn into `buf` at camera
    /// (cam_x, cam_y) - called for the world-space cache layer, not per frame.
    /// The ground pass: every tile's block (or flat diamond where there's no
    /// art) drawn back to front so nearer blocks cover farther ones, plus the
    /// odd tuft or flower on open grass. Drawn into `buf` at camera
    /// (cam_x, cam_y) - called for the world-space cache layer, not per frame.
    /// The ground pass: every tile's block (or flat diamond where there's no
    /// art) drawn back to front so nearer blocks cover farther ones, plus the
    /// odd tuft or flower on open grass. Drawn into `buf` at camera
    /// (cam_x, cam_y) - called for the world-space cache layer, not per frame.
    pub fn draw_ground(&self, m: &UiModel, buf: &mut Buffer, cam_x: i32, cam_y: i32, zoom: f32) {
        let season = m.season;
        let tw = (TILE_W as f32 * zoom) as i32;
        let th = (TILE_H as f32 * zoom) as i32;
        // the sea is level with the land, or the shore blocks' sides showed
        // over it like a strip of grass. Blocks are scaled once per pass,
        // not once per tile.
        let set = &self.art.tiles[season_tiles(season)];
        let faces = &self.art.faces[season_tiles(season)];
        let mut scaled: Vec<(*const Sprite, Sprite)> = vec![];
        let (w_t, h_t) = (m.map.width as i32, m.map.height as i32);
        for d in 0..(w_t + h_t - 1) {
            for tx in (d - h_t + 1).max(0)..=d.min(w_t - 1) {
                let ty = d - tx;
                let Some(tile) = m.map.get(tx, ty) else { continue };
                let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, cam_x, cam_y, zoom);
                if sx < -2 * tw || sx > buf.width as i32 + 2 * tw || sy < -4 * th || sy > buf.height as i32 + 4 * th { continue; }
                // inland the blocks in front cover a tile's sides: just its face
                let inland = tx + 1 < w_t && ty + 1 < h_t;
                let set = if inland { faces } else { set };
                match set.block(&m.map, tx, ty) {
                    Some(block) => {
                        let (w, h, x, y) = block_rect(block, sx, sy, zoom);
                        let key = block as *const Sprite;
                        if !scaled.iter().any(|(k, _)| *k == key) { scaled.push((key, block.scaled(w, h))); }
                        scaled.iter().find(|(k, _)| *k == key).unwrap().1.blit(buf, x, y);
                    }
                    None => {
                        let (r, g, b) = tile_colour(tile, season);
                        buf.diamond(sx, sy, tw, th, r, g, b);
                        buf.diamond(sx, sy + th, tw, 1, r / 2, g / 2, b / 2);
                    }
                }
                // the odd tuft or flower on open grass outside the walls
                let in_town = (m.town_wall.0..=m.town_wall.2).contains(&tx)
                    && (m.town_wall.1..=m.town_wall.3).contains(&ty);
                if tile == Tile::Grass && !in_town && !m.map.is_occupied(tx, ty) && noise(tx, ty, 7) < 0.10 {
                    if let Some(d) = pick(&self.art.details[season_tiles(season)], tx, ty, 8) {
                        let k = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
                        let (dw, dh) = ((d.width as f32 * k).round() as i32, (d.height as f32 * k).round().max(1.0) as i32);
                        let (jx, jy) = (noise(tx, ty, 10) - 0.5, noise(tx, ty, 11) - 0.5);
                        draw_zoomed(buf, d,
                            sx + (jx * TILE_W as f32 * zoom) as i32 - dw / 2,
                            sy + ((jy - 0.6) * TILE_H as f32 * zoom) as i32 - dh, k);
                    }
                }
            }
        }
    }

    /// The ground layer drawn once into world space (camera at 0,0): a
    /// buffer covering every tile's block so the frame pass can blit the
    /// window of it the camera actually shows
    pub fn render_ground(&self, m: &UiModel, zoom: f32) -> GroundCache {
        let (w_t, h_t) = (m.map.width as i32, m.map.height as i32);
        let tw = (TILE_W as f32 * zoom) as i32;
        let th = (TILE_H as f32 * zoom) as i32;
        let season = m.season;
        let set = &self.art.tiles[season_tiles(season)];
        // the rectangle of world screen space the ground covers: the fullest
        // (edge-style) block each tile could draw, plus room for the specks
        // a tile may wear and a margin
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for ty in 0..h_t { for tx in 0..w_t {
            let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, 0, 0, zoom);
            let (bx0, by0, bx1, by1) = match set.block(&m.map, tx, ty) {
                Some(block) => { let (w, h, x, y) = block_rect(block, sx, sy, zoom); (x, y, x + w as i32, y + h as i32) }
                None => (sx - tw, sy - th, sx + tw, sy + 2 * th),
            };
            x0 = x0.min(bx0); y0 = y0.min(by0); x1 = x1.max(bx1); y1 = y1.max(by1);
        }}
        let pad = (40.0 * zoom).round() as i32 + 8;
        x0 -= tw + pad; x1 += tw + pad; y0 -= pad; y1 += 2 * th + pad;
        let mut buf = Buffer::new((x1 - x0).max(1) as usize, (y1 - y0).max(1) as usize);
        // the same dark the frame clears to, so the layer's edges blend away
        buf.rect(0, 0, buf.width as i32, buf.height as i32, 25, 25, 38);
        // the camera offsets draw_ground applies are in unzoomed space (the
        // projection scales them by zoom), so to shift content by -x0
        // screen px the cam to pass is -x0 / zoom
        let (kx, ky) = ((-x0 as f32 / zoom).round() as i32, (-y0 as f32 / zoom).round() as i32);
        self.draw_ground(m, &mut buf, kx, ky, zoom);
        let origin = ((kx as f32 * zoom).round() as i32, (ky as f32 * zoom).round() as i32);
        GroundCache { buf, origin, zoom, season, rev: m.map.rev() }
    }

}

/// Draw a child's figure - the pack has none, so they're a scaled-down
/// villager flagged "(child)" on the label
fn draw_kid(buf: &mut Buffer, sprite: &Sprite, sx: i32, sy: i32, zoom: f32) {
    draw_zoomed(buf, sprite, sx - (19.0 * zoom).round() as i32, sy - (36.0 * zoom).round() as i32, 1.2 * zoom);
}

#[cfg(test)]
mod stay_home_tests {
    use super::*;

    #[test]
    fn test_villagers_stay_in_their_own_village() {
        // on main, Aldric was at x 27.5 by day 1050 - the rivals' village -
        // and stayed there, eating from their store
        let mut game = Game::new();
        let home = *game.world.get_component::<Pos>(game.market_food).unwrap();
        // where each villager actually lives - an outskirts cottage is
        // legitimately far from the store, that's not straying
        let homes: Vec<Option<Pos>> = game.villagers.iter().map(|&v| {
            household::home_of(&game.world, v).and_then(|h| game.world.get_component::<Pos>(h)).map(|p| *p)
        }).collect();
        // and where they work - a farm or a mason's yard south of the wall
        // is a day's post, not wandering
        let posts: Vec<Option<Pos>> = game.villagers.iter().map(|&v| {
            game.world.get_component::<Craft>(v).and_then(|c| c.workplace)
                .and_then(|b| game.world.get_component::<Pos>(b)).map(|p| *p)
        }).collect();
        // a stray is being far from the store AND from your own house for
        // days on end - an errand out and back isn't one
        let mut run = vec![0u64; NAMES.len()];
        let (mut worst, mut who_was_it) = (0u64, 0usize);
        let mut last = 0;
        while game.tick < 1500 {
            game.update();
            if game.tick == last { continue; }
            last = game.tick;
            // the roster grows: births and newcomers take index 32, 33...
            run.resize(game.villagers.len(), 0);
            for (i, v) in game.villagers.iter().enumerate() {
                // the dead lie where they fell - a grave isn't a stray;
                // and the held are at the castle, not wandering - hauled
                // off mid-errand they stand where they were taken, but
                // they're in custody, not adrift
                if villager::is_dead(&game.world, *v) { run[i] = 0; continue; }
                if crate::game::dungeon::unavailable(&game.world, *v) { run[i] = 0; continue; }
                if game.travels.iter().any(|t| t.who == *v)
                    || game.world.get_component::<OddJob>(*v).is_some()
                    || Some(*v) == game.collector { run[i] = 0; continue; }
                let p = game.world.get_component::<Pos>(*v).unwrap();
                let d = |hp: Pos| ((p.x - hp.x).powi(2) + (p.y - hp.y).powi(2)).sqrt();
                // straying means leaving town: outside the wall (with room
                // for the fields just past it), not the far corner of the
                // market square, which is still home
                let (wx0, wy0, wx1, wy1) = game.town_wall;
                let outside = p.x < wx0 as f32 - 4.0 || p.x > wx1 as f32 + 4.0
                    || p.y < wy0 as f32 - 4.0 || p.y > wy1 as f32 + 4.0;
                // newcomers (born or settled mid-run) have no slot in the
                // snapshot - look theirs up live
                let home = homes.get(i).copied().flatten()
                    .or_else(|| household::home_of(&game.world, *v)
                        .and_then(|h| game.world.get_component::<Pos>(h)).map(|p| *p));
                let post = posts.get(i).copied().flatten()
                    .or_else(|| game.world.get_component::<Craft>(*v)
                        .and_then(|c| c.workplace)
                        .and_then(|b| game.world.get_component::<Pos>(b)).map(|p| *p));
                let adrift = outside
                    && home.map(|hp| d(hp) > 4.0).unwrap_or(true)
                    && post.map(|pp| d(pp) > 4.0).unwrap_or(true);
                run[i] = if adrift { run[i] + 1 } else { 0 };
                if run[i] > worst { (worst, who_was_it) = (run[i], i); }
            }
        }
        let sp = *game.world.get_component::<Pos>(game.villagers[who_was_it]).unwrap();
        let st = (sp.x.round() as i32, sp.y.round() as i32);
        let route = game.map.walk_path(st, (home.x.round() as i32, home.y.round() as i32)).map(|p| p.len());
        let around: Vec<String> = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| format!("{:?}", game.map.get(st.0 + dx, st.1 + dy))).collect();
        // a month adrift is a defection; a fortnight's foraging east is an
        // excursion that comes home (on main, Aldric stayed for good)
        assert!(worst <= 30, "{} stayed {} days away at {:?} (route home: {:?}) - neighbours {:?}",
            vill_name(who_was_it), worst, st, route, around);
    }
}

#[cfg(test)]
mod rival_tests {
    use super::*;

    #[test]
    fn test_rivals_stay_in_their_own_village() {
        let _art = Art::default(); // no sprites needed to run the game, only to draw it
        let mut game = Game::new();
        while game.tick < 600 { game.update(); }
        for (i, r) in game.rivals.iter().enumerate() {
            let p = game.world.get_component::<Pos>(*r).unwrap();
            assert!(p.x >= 24.0, "{} wandered into our village at ({:.1},{:.1})", RIVAL_NAMES[i], p.x, p.y);
        }
    }
}

#[cfg(test)]
mod illness_tests {
    use super::*;

    #[test]
    fn test_flu_arrives_spreads_through_the_log_and_passes() {
        let mut game = Game::new();
        while game.tick < FLU_DAY + 5 { game.update(); }
        // the herb-wife may already have seen to him - the arrival itself
        // is the log line, not whether he's still down with it
        assert!(game.log.iter().any(|l| l.contains("Bram is taken with la grippe")), "the log should say who fell ill");
        while game.tick < 600 { game.update(); }
        // la grippe itself runs its course - whatever else the village
        // caught meanwhile is another story
        assert!(game.world.query::<Disease>().iter().all(|&d|
            game.world.get_component::<Disease>(d).map(|dd| dd.kind != DiseaseKind::Influenza).unwrap_or(true)),
            "la grippe should have run its course by day 600");
        assert!(game.log.iter().any(|l| l.contains("recovered from la grippe")), "the log should say who recovered");
    }
}

#[cfg(test)]
mod herb_tests {
    use super::*;

    #[test]
    fn test_the_herb_wife_is_one_and_keeps_her_own_hut() {
        let game = Game::new();
        let hers = game.villagers.iter()
            .filter(|&&v| game.world.get_component::<TownRole>(v) == Some(&TownRole::Herbologist))
            .count();
        assert_eq!(hers, 1, "there is exactly one herb-wife");
        assert_eq!(game.herbologist, Some(game.villagers[HERB_IDX]));
        let hut = game.herb_hut.expect("the hut should stand");
        let b = game.world.get_component::<Building>(hut).unwrap();
        assert_eq!(b.name, "the herb hut");
        assert_eq!(b.kind, BuildingKind::Home);
        assert!(b.occupants.contains(&game.villagers[HERB_IDX]), "she lives in it");
    }

    #[test]
    fn test_herb_patches_grow_wild_outside_the_wall() {
        let game = Game::new();
        assert!(game.herb_patches.len() >= 8, "a dozen or so tufts");
        let (x0, y0, x1, y1) = game.town_wall;
        for p in &game.herb_patches {
            assert!(game.map.passable(p.tile.0, p.tile.1), "patch on a walkable tile");
            let inside = (x0..=x1).contains(&p.tile.0) && (y0..=y1).contains(&p.tile.1);
            assert!(!inside, "herbs grow wild, not in the streets");
        }
    }

    #[test]
    fn test_she_brews_what_the_hut_stocks() {
        let mut game = Game::new();
        let hut = game.herb_hut.unwrap();
        game.world.get_component_mut::<Building>(hut).unwrap().stock.insert("willow_bark".into(), 2);
        game.herbologist_tick(5);
        let her = game.herbologist.unwrap();
        assert!(herbology::remedies_on_hand(&game.world, her) >= 1, "a remedy on the shelf");
        assert_eq!(game.world.get_component::<Building>(hut).unwrap().stock["willow_bark"], 1, "one herb went in");
    }

    #[test]
    fn test_a_forage_trip_tops_up_the_hut() {
        let mut game = Game::new();
        let her = game.herbologist.unwrap();
        let pi = (0..game.herb_patches.len())
            .find(|&i| game.send_forage(her, i)).expect("a patch within her reach");
        let herb = game.herb_patches[pi].herb;
        for _ in 0..(16 * FPS) { game.update(); }
        let hut = game.herb_hut.unwrap();
        // two bundles picked; she may already have brewed some of them
        let got = game.world.get_component::<Building>(hut).unwrap().stock.get(herb).copied().unwrap_or(0) as usize
            + herbology::remedies_on_hand(&game.world, game.herbologist.unwrap());
        assert!(got >= 2, "the pickings get dried at the hut ({} stocked + brewed)", got);
        assert!(game.herb_patches[pi].next_ready > game.tick, "the patch needs a season to come back");
    }

    #[test]
    fn test_the_sick_walk_out_and_a_cure_turns_the_disease() {
        let mut game = Game::new();
        let v = game.villagers[5];
        let d = game.world.create_entity();
        game.world.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: v, stage: Stage::Moderate, severity: 0.45, contagious: false, incubation: 0 });
        // stock the shelf so a cure's in hand
        let hut = game.herb_hut.unwrap();
        game.world.get_component_mut::<Building>(hut).unwrap().stock.insert("yarrow".into(), 2);
        let before_gold = game.world.get_component::<Purse>(v).unwrap().gold;
        game.arrive(v, Goal::SeekRemedy(hut));
        let sev = game.world.query::<Disease>().iter()
            .filter_map(|d| game.world.get_component::<Disease>(*d))
            .find(|d| d.patient == v).map(|d| (d.severity, d.stage));
        let (sev, stage) = sev.unwrap_or((0.0, Stage::Recovering));
        assert!(sev < 0.45 || stage == Stage::Recovering, "the remedy bit into it");
        assert!(game.world.get_component::<Purse>(v).unwrap().gold < before_gold, "she doesn't work for free");
        assert!(game.remedy_visits.iter().any(|(e, _)| *e == v), "the call is remembered");
    }

    #[test]
    fn test_nothing_in_the_hedge_says_so() {
        let mut game = Game::new();
        let v = game.villagers[5];
        let d = game.world.create_entity();
        game.world.add_component(d, Disease { kind: DiseaseKind::Influenza, name: "flu".into(), patient: v, stage: Stage::Mild, severity: 0.2, contagious: false, incubation: 0 });
        let hut = game.herb_hut.unwrap();
        game.arrive(v, Goal::SeekRemedy(hut));
        assert!(game.bubbles.iter().any(|(e, t, _, _, _)| *e == game.herbologist.unwrap() && t.contains("Nothing")),
            "she says when she can't help");
    }
}

#[cfg(test)]
mod flock_tests {
    use super::*;

    #[test]
    fn test_a_shepherd_keeps_the_fold() {
        let g = Game::new();
        let sh = g.shepherd.expect("a shepherd keeps the fold");
        assert_eq!(g.world.get_component::<Craft>(sh).unwrap().vocation, Vocation::Shepherd);
        let fold = g.fold.unwrap();
        let b = g.world.get_component::<Building>(fold).unwrap();
        assert_eq!(b.kind, BuildingKind::Pasture);
        assert_eq!(b.name, "the fold");
        assert_eq!(b.worker, Some(sh), "the fold knows its shepherd");
        assert_eq!(g.world.get_component::<Craft>(sh).unwrap().workplace, Some(fold));
        assert!(g.flock.len() >= 8, "a decent flock, got {}", g.flock.len());
    }

    #[test]
    fn test_the_sheepdog_belongs_to_the_shepherd() {
        let g = Game::new();
        let sh = g.shepherd.unwrap();
        assert!(g.pets.iter().any(|(e, s, _)| *s == Species::Dog
            && g.world.get_component::<Pet>(*e).map(|p| p.owner == Some(sh)).unwrap_or(false)),
            "a herding dog at his heel");
    }

    #[test]
    fn test_milk_and_wool_come_off_the_fold() {
        let mut g = Game::new();
        let fold = g.fold.unwrap();
        let sh = g.shepherd.unwrap();
        for t in 0..3 { livelihood::work_day(&mut g.world, sh, t); }
        assert!(g.world.get_component::<Building>(fold).unwrap().stock.get("milk").copied().unwrap_or(0) > 0,
            "the goats give milk daily");
        g.livestock_tick(45); // shearing day, mid-season
        assert!(g.world.get_component::<Building>(fold).unwrap().stock.get("wool").copied().unwrap_or(0) > 0,
            "the sheep are shorn in season");
    }

    #[test]
    fn test_the_shepherd_smells_of_it() {
        let g = Game::new();
        let sh = g.shepherd.unwrap();
        assert!(desires::problems_of(&g.world, sh, 0).contains(&desires::Problem::Smelly));
    }

    #[test]
    fn test_a_wolf_night_losses_and_the_loss_grievance() {
        let mut g = Game::new();
        let fold = g.fold.unwrap();
        let before = livestock::flock(&g.world, fold).len();
        g.wolf_night(60);
        let after = livestock::flock(&g.world, fold).len();
        assert!(before - after <= 2, "wolves take a couple at most");
        if let Some(sh) = g.shepherd {
            assert!(desires::problems_of(&g.world, sh, 60).contains(&desires::Problem::Loss),
                "the loss sticks on him for the season");
        }
    }

    #[test]
    fn test_the_flock_stays_near_the_fold() {
        let mut g = Game::new();
        let fold = g.fold.unwrap();
        let fp = *g.world.get_component::<Pos>(fold).unwrap();
        for _ in 0..(10 * FPS) { g.update(); }
        for &a in &g.flock {
            let p = g.world.get_component::<Pos>(a).unwrap();
            let d = (p.x - fp.x).abs() + (p.y - fp.y).abs();
            assert!(d < 20.0, "an animal strayed too far ({:.0} tiles)", d);
        }
    }
}

#[cfg(test)]
mod hen_tests {
    use super::*;

    #[test]
    fn test_hens_scratch_about_a_few_yards() {
        let g = Game::new();
        assert!(g.hen_houses.len() >= 3, "a few yards keep hens");
        assert!(g.hens.len() >= 8, "a few hens a yard");
        for &a in &g.hens {
            assert_eq!(g.world.get_component::<Livestock>(a).unwrap().kind, LivestockKind::Hen);
            let home = g.world.get_component::<Livestock>(a).unwrap().fold;
            assert!(g.hen_houses.contains(&home), "each hen belongs to a yard");
        }
    }

    #[test]
    fn test_eggs_lay_and_a_hungry_house_eats_them() {
        let mut g = Game::new();
        let h = g.hen_houses[0];
        g.livestock_tick(1);
        let eggs = g.world.get_component::<Building>(h).unwrap().stock.get("eggs").copied().unwrap_or(0);
        assert!(eggs > 0, "a hen lays in the morning");
        // a hungry occupant takes one off the larder
        let occ = g.world.get_component::<Building>(h).unwrap().occupants[0];
        g.world.get_component_mut::<Hunger>(occ).unwrap().value = 20.0;
        g.livestock_tick(2);
        assert!(g.world.get_component::<Hunger>(occ).unwrap().value > 20.0, "eggs for breakfast");
    }

    #[test]
    fn test_eggs_do_not_pile_past_the_larder() {
        let mut g = Game::new();
        let h = g.hen_houses[0];
        for t in 0..10 { g.livestock_tick(t); }
        assert!(g.world.get_component::<Building>(h).unwrap().stock.get("eggs").copied().unwrap_or(0) <= 6);
    }

    #[test]
    fn test_the_fox_takes_a_hen_and_the_house_grieves() {
        let mut g = Game::new();
        let before = g.hens.len();
        g.fox_night(75);
        assert!(before - g.hens.len() <= 1, "a fox takes one, not the yard");
        assert!(g.log.iter().any(|l| l.contains("a fox")), "the yard says so");
    }
}

#[cfg(test)]
mod heart_tests {
    use super::*;

    #[test]
    fn test_the_heart_helper_is_one() {
        let g = Game::new();
        let mm = g.matchmaker.expect("a heart-helper watches the square");
        assert_eq!(g.world.get_component::<TownRole>(mm), Some(&TownRole::Matchmaker));
        assert_eq!(g.villagers.iter().filter(|&&v|
            g.world.get_component::<TownRole>(v) == Some(&TownRole::Matchmaker)).count(), 1);
    }

    #[test]
    fn test_lonely_hearts_excludes_the_spoken_for() {
        let mut g = Game::new();
        // nobody starts lonely - wants are sparse - so none qualify by default
        // give two villagers unmet love wants and they join the pool
        let (a, b) = (g.villagers[0], g.villagers[1]);
        for v in [a, b] {
            g.world.get_component_mut::<InnerLife>(v).unwrap().wants.push(desires::Want::Love);
        }
        let pool = g.lonely_hearts();
        assert!(pool.contains(&a) && pool.contains(&b), "the lonely are known");
        // a kid can't hold a heart to mend (no craft)
        assert!(!g.lonely_hearts().iter().any(|&v| g.world.get_component::<Craft>(v).is_none()));
    }

    #[test]
    fn test_a_match_kindles_a_lover_bond() {
        let mut g = Game::new();
        let (a, b) = (g.villagers[0], g.villagers[1]);
        for v in [a, b] {
            g.world.get_component_mut::<InnerLife>(v).unwrap().wants.push(desires::Want::Love);
        }
        // keep trying across the years until the spark catches
        for t in (0..900).step_by(31) { g.match_attempt(a, t, g.matchmaker, "the heart-helper"); }
        assert!(g.has_rel(relationships::RelKind::Lover, a), "a match was kindled");
        assert!(g.world.get_component::<InnerLife>(a).unwrap().satisfied.iter()
            .any(|(w, _)| *w == desires::Want::Love), "the ache is mended");
        assert!(g.log.iter().any(|l| l.contains("together")), "the village hears of it");
    }

    #[test]
    fn test_kin_are_not_paired() {
        let mut g = Game::new();
        let (a, b) = (g.villagers[0], g.villagers[1]);
        for v in [a, b] {
            g.world.get_component_mut::<InnerLife>(v).unwrap().wants.push(desires::Want::Love);
        }
        // a and b are kin: no match between them, ever
        let rel = g.world.create_entity();
        g.world.add_component(rel, relationships::Relationship {
            from: a, other: b, kind: relationships::RelKind::Family, strength: 0.8, interactions: 5 });
        for t in (0..40).step_by(31) { g.match_attempt(a, t, g.matchmaker, "the heart-helper"); }
        assert_eq!(g.rel_kind(a, b), Some(relationships::RelKind::Family), "kin stay kin");
    }
}

#[cfg(test)]
mod shroom_tests {
    use super::*;

    fn deadly_patch(g: &Game) -> usize {
        g.mush_patches.iter().position(|p| p.deadly()).unwrap()
    }

    fn kind_patch(g: &Game, kind: MushKind) -> usize {
        g.mush_patches.iter().position(|p| p.kind == kind).unwrap()
    }

    /// The roll `shroom_take` makes for `who` on day `t`
    fn read_roll(who: Entity, t: u64) -> u64 {
        (who as u64).wrapping_mul(2654435761).wrapping_add(t) % 1000
    }

    /// A day `who`'s eye reads true at `skill`
    fn true_read_day(who: Entity, skill: f32) -> u64 {
        (0..50000).find(|&t| read_roll(who, t) < mycology::read_chance(skill)).unwrap()
    }

    /// A day `who`'s eye fails at `skill`, folk rule or no
    fn misread_day(who: Entity, skill: f32, folk: bool) -> u64 {
        (0..50000).find(|&t| {
            let r = read_roll(who, t);
            r >= mycology::read_chance(skill) && (r % 10 < 3) == folk
        }).unwrap()
    }

    #[test]
    fn test_the_wood_keeps_each_kind_to_its_ground() {
        let g = Game::new();
        assert!(g.mush_patches.len() >= 5, "the wood keeps its patches");
        assert!(g.mush_patches.iter().any(|p| p.deadly()), "some caps are death");
        assert!(g.mush_patches.iter().any(|p| !p.deadly()), "some are good");
        // the families, each true to its habitat
        assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::LibertyCap),
            "the pasture keeps the little nipple-tipped caps");
        assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::Galerina),
            "the little brown killer keeps them company");
        assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::FlyAgaric),
            "the fly agaric under birch");
        assert!(g.mush_patches.iter().any(|p| p.kind == MushKind::UnderCap),
            "the under keeps its own");
    }

    #[test]
    fn test_the_lookalike_pairs_sit_the_way_the_research_put_them() {
        // galerina is the little brown mushroom that looks like the
        // liberty cap - the textbook trap for the dream-seeker
        assert_eq!(MushKind::Galerina.spec().lookalike, MushKind::LibertyCap);
        assert_eq!(MushKind::LibertyCap.spec().lookalike, MushKind::Galerina);
        assert!(MushKind::Galerina.is_deadly());
        assert!(MushKind::LibertyCap.believed_edible());
        // the death-cap wears the field mushroom's face; the jack wears
        // the chanterelle's - one kills, one just turns the gut
        assert_eq!(MushKind::DeathCap.spec().lookalike, MushKind::FieldMushroom);
        assert_eq!(MushKind::Chanterelle.spec().lookalike, MushKind::JackOLantern);
        assert!(MushKind::DeathCap.is_deadly());
        assert!(!MushKind::JackOLantern.is_deadly());
    }

    #[test]
    fn test_a_good_cap_fills_the_belly() {
        let mut g = Game::new();
        let who = g.villagers[4];
        let pi = kind_patch(&g, MushKind::Chanterelle);
        g.world.get_component_mut::<Hunger>(who).unwrap().value = 30.0;
        g.world.get_component_mut::<Craft>(who).unwrap().skill = 1.0;
        g.tick = true_read_day(who, 1.0);
        g.arrive(who, Goal::ForageShroom(pi));
        let h = g.world.get_component::<Hunger>(who).unwrap().value;
        assert!(h > 30.0, "chanterelles fill the belly, got {:.0}", h);
    }

    #[test]
    fn test_the_death_cap_keeps_its_secret_for_days() {
        // the amatoxin course: nothing, then the belly, a false mend,
        // then the liver - the delayed onset is the horror of it
        let mut g = Game::new();
        let who = g.villagers[4];
        let pi = deadly_patch(&g);
        g.world.get_component_mut::<Craft>(who).unwrap().skill = 0.0;
        g.tick = misread_day(who, 0.0, false);
        g.arrive(who, Goal::ForageShroom(pi));
        let diseases_of = |g: &Game| g.world.query::<Disease>().iter()
            .filter_map(|&d| g.world.get_component::<Disease>(d))
            .filter(|dd| dd.patient == who).map(|dd| (dd.name.clone(), dd.severity))
            .collect::<Vec<_>>();
        assert!(diseases_of(&g).is_empty(), "nothing at first - that's the horror of it");
        assert!(g.ailments.iter().any(|a| a.who == who && matches!(a.phase, AilPhase::ShroomBelly(_))),
            "the course is already under way");
        // the belly lands a day on, when the meal's half forgotten
        g.ailment_tick(g.tick + 1);
        assert!(diseases_of(&g).iter().any(|(n, _)| n == "a bad belly"), "the belly lands");
        assert!(g.lore_of(who).contains(&pi), "the belly teaches the patch");
        // the false mend - they seem to turn the corner
        g.ailment_tick(g.tick + 3);
        assert!(diseases_of(&g).iter().all(|(n, s)| n != "a bad belly" || *s < 0.2),
            "they seem to mend - they haven't");
        // then the liver gives out, days after the meal
        g.ailment_tick(g.tick + 7);
        assert!(diseases_of(&g).iter().any(|(n, _)| n == "the liver gives out"),
            "the liver gives out at the end");
    }

    #[test]
    fn test_the_learned_leave_death_be() {
        let mut g = Game::new();
        let who = g.villagers[4];
        let pi = deadly_patch(&g);
        g.world.get_component_mut::<Hunger>(who).unwrap().value = 30.0;
        g.learn_shroom(who, pi);
        g.arrive(who, Goal::ForageShroom(pi));
        let h = g.world.get_component::<Hunger>(who).unwrap().value;
        assert!(h <= 30.0, "the learned leave the death-cap standing");
        assert!(g.ailments.iter().all(|a| a.who != who), "no course begins for the learned");
    }

    #[test]
    fn test_the_herb_wife_teaches_the_wood() {
        let mut g = Game::new();
        let her = g.herbologist.unwrap();
        let who = g.villagers.iter().copied()
            .find(|&v| v != her && !villager::is_dead(&g.world, v)).unwrap();
        let hp = g.world.get_component::<Pos>(her).copied().unwrap();
        let p = g.world.get_component_mut::<Pos>(who).unwrap();
        p.x = hp.x; p.y = hp.y;
        g.teach_shrooms(g.tick);
        assert!(g.lore_of(who).iter().any(|&pi| g.mush_patches[pi].deadly()),
            "a neighbour learns which cap is death");
    }

    #[test]
    fn test_the_folk_rule_clears_a_death_cap() {
        // "a poisoned cap blackens silver" - the silver stayed bright,
        // the cap was death anyway: knowledge can be wrong
        let mut g = Game::new();
        let who = g.villagers[4];
        let pi = deadly_patch(&g);
        g.world.get_component_mut::<Craft>(who).unwrap().skill = 0.0;
        g.tick = misread_day(who, 0.0, true);
        g.arrive(who, Goal::ForageShroom(pi));
        assert!(g.ailments.iter().any(|a| a.who == who && matches!(a.phase, AilPhase::ShroomBelly(_))),
            "the silver stayed bright - they ate death anyway");
    }

    #[test]
    fn test_the_liberty_cap_gives_visions() {
        let mut g = Game::new();
        let who = g.villagers[4];
        let pi = kind_patch(&g, MushKind::LibertyCap);
        g.world.get_component_mut::<Craft>(who).unwrap().skill = 1.0;
        let mut saw_joy = false;
        let mut saw_turn = false;
        let mut day = 0u64;
        for _ in 0..30 {
            day = (day + 1..day + 50000)
                .find(|&t| read_roll(who, t) < mycology::read_chance(1.0)).unwrap();
            g.tick = day;
            g.arrive(who, Goal::ForageShroom(pi));
            saw_joy |= desires::is_satisfied(&g.world, who, Want::Joy, g.tick);
            saw_turn |= g.world.query::<Disease>().iter().any(|&d|
                g.world.get_component::<Disease>(d)
                    .map(|dd| dd.patient == who && dd.name == "a fearful turn").unwrap_or(false));
            if saw_joy && saw_turn { break; }
        }
        assert!(saw_joy, "the field speaks kindly");
        assert!(saw_turn, "now and then a dark turn");
    }

    #[test]
    fn test_the_fly_agarics_dream_is_wild_not_lethal() {
        let mut g = Game::new();
        let who = g.villagers[4];
        let pi = kind_patch(&g, MushKind::FlyAgaric);
        g.world.get_component_mut::<Craft>(who).unwrap().skill = 1.0;
        g.tick = true_read_day(who, 1.0);
        g.arrive(who, Goal::ForageShroom(pi));
        assert!(g.world.query::<Disease>().iter().any(|&d|
            g.world.get_component::<Disease>(d)
                .map(|dd| dd.patient == who && dd.name == "the red cap's dream").unwrap_or(false)),
            "the red cap's dream, not its death");
        assert!(g.ailments.iter().all(|a| a.who != who), "no amatoxin course from the fly agaric");
    }

    #[test]
    fn test_the_fire_blames_the_bread() {
        let mut g = Game::new();
        g.ergot(210);
        assert!(g.log.iter().any(|l| l.contains("in the bread")));
        // the fire takes the convulsive and the burning forms
        let n_fire = g.world.query::<Disease>().iter().filter(|&&d|
            g.world.get_component::<Disease>(d)
                .map(|dd| dd.name == "the convulsive fire" || dd.name == "the burning fire")
                .unwrap_or(false)).count();
        let want = 2 + if g.harvest_wetness(210) > 0.45 { 2 } else { 0 };
        assert_eq!(n_fire, want, "the wet year's toll");
        // somebody carries the blame - the baker if there is one
        let blamed = g.villagers.iter().copied().any(|v|
            desires::problems_of(&g.world, v, 210).contains(&desires::Problem::Blamed));
        assert!(blamed, "the village finds someone to blame");
        // the brothers of Saint Anthony make their rounds
        if g.church.is_some() {
            assert!(g.ailments.iter().any(|a| matches!(a.phase, AilPhase::AntonineCare)),
                "a hospice bed for the burned");
        }
    }
}

#[cfg(test)]
mod apiary_tests {
    use super::*;

    #[test]
    fn test_a_beekeeper_keeps_the_skeps() {
        let g = Game::new();
        let bk = g.beekeeper.expect("a beekeeper keeps the yard");
        assert_eq!(g.world.get_component::<Craft>(bk).unwrap().vocation, Vocation::Beekeeper);
        let ay = g.apiary.unwrap();
        let b = g.world.get_component::<Building>(ay).unwrap();
        assert_eq!(b.kind, BuildingKind::Apiary);
        assert_eq!(b.name, "the apiary");
        assert_eq!(b.worker, Some(bk));
        assert!(apiary::hives_of(&g.world, ay).len() >= 3, "a few skeps stand round it");
    }

    #[test]
    fn test_the_skeps_yield_in_season() {
        let mut g = Game::new();
        let ay = g.apiary.unwrap();
        g.apiary_tick(50); // midsummer
        let b = g.world.get_component::<Building>(ay).unwrap();
        assert!(b.stock.get("honey").copied().unwrap_or(0) > 0, "honey for the table");
        // the wax went straight to the church's candles
        let cb = g.world.get_component::<Building>(g.church.unwrap()).unwrap();
        assert!(cb.stock.get("candle").copied().unwrap_or(0) > 0, "wax for the candles");
    }

    #[test]
    fn test_a_dark_church_troubles_the_priest() {
        let mut g = Game::new();
        // burn day with no candles on the shelf
        g.apiary_tick(20);
        let priest = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
        if let Some(pr) = priest {
            assert!(desires::problems_of(&g.world, pr, 20).iter()
                .any(|p| matches!(p, desires::Problem::NoInput("wax"))), "no wax, dark church");
        }
    }
}

#[cfg(test)]
mod vermin_tests {
    use super::*;

    #[test]
    fn test_bread_spoils_and_grain_holds() {
        let mut g = Game::new();
        let be = g.building_entities[5]; // the trader's stall, well-stocked
        if let Some(b) = g.world.get_component_mut::<Building>(be) {
            household::add_stock(b, "bread", 4); household::add_stock(b, "wheat", 4);
        }
        g.spoilage_tick(3); // the weekly rot - tick 3 lands on the wheat's turn too
        g.spoilage_tick(10); // and the next week's
        let b = g.world.get_component::<Building>(be).unwrap();
        assert_eq!(b.stock.get("bread").copied().unwrap_or(0), 2, "bread goes in a week");
        assert_eq!(b.stock.get("wheat").copied().unwrap_or(0), 3, "wheat goes in a month");
    }

    #[test]
    fn test_a_hunting_cat_walks_the_yard() {
        let g = Game::new();
        // Mog hunts for villagers[1] - their home is watched
        let home = household::home_of(&g.world, g.villagers[1]);
        if let Some(h) = home {
            assert!(g.cat_watches(h), "the cat keeps the vermin off");
        }
        // and some building nowhere near a cat is not watched
        assert!(!g.cat_watches(g.building_entities[4]), "the church has no cat");
    }

    #[test]
    fn test_a_poor_soul_eats_what_shouldnt_be_eaten() {
        let mut g = Game::new();
        let v = g.villagers[6];
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 0.5;
        g.world.get_component_mut::<Hunger>(v).unwrap().value = 10.0;
        g.spoilage_tick(4);
        assert!(desires::problems_of(&g.world, v, 4).contains(&desires::Problem::SourGut),
            "a poor soul carries the sour gut");
    }
}

#[cfg(test)]
mod trade_issue_tests {
    use super::*;

    fn set_voc(g: &mut Game, v: Entity, voc: Vocation) {
        g.world.get_component_mut::<Craft>(v).unwrap().vocation = voc;
    }

    #[test]
    fn test_a_trades_grief_echoes_among_the_trade() {
        let mut g = Game::new();
        let pe = g.player;
        let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
        set_voc(&mut g, a, Vocation::Mason);
        set_voc(&mut g, b, Vocation::Mason);
        set_voc(&mut g, c, Vocation::Mason);
        let r = || desires::Reveal::Problem(desires::Problem::NoInput("stone"));
        assert!(desires::hear(&mut g.world, pe, a, r(), 10).is_none());
        assert!(desires::hear(&mut g.world, pe, b, r(), 10).is_none());
        let issue = desires::hear(&mut g.world, pe, c, r(), 10);
        assert_eq!(issue.as_deref(), Some("the masons' workshops starved of materials"),
            "three masons make one issue, named for the trade");
    }

    #[test]
    fn test_the_same_trouble_across_trades_splinters() {
        let mut g = Game::new();
        let pe = g.player;
        let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
        set_voc(&mut g, a, Vocation::Mason);
        set_voc(&mut g, b, Vocation::Mason);
        set_voc(&mut g, c, Vocation::Baker);
        let r = || desires::Reveal::Problem(desires::Problem::NoInput("stone"));
        desires::hear(&mut g.world, pe, a, r(), 10);
        desires::hear(&mut g.world, pe, b, r(), 10);
        // two masons and a baker say "no materials" - that's two voices
        // of one trade plus one of another, never three of a kind
        assert!(desires::hear(&mut g.world, pe, c, r(), 10).is_none(),
            "a mixed room doesn't converge on a trade's grief");
        assert!(g.world.get_component::<Organizer>(pe).unwrap().issues.is_empty());
    }

    #[test]
    fn test_everybodys_grief_converges_in_a_mixed_room() {
        let mut g = Game::new();
        let pe = g.player;
        let (a, b, c) = (g.villagers[0], g.villagers[1], g.villagers[2]);
        set_voc(&mut g, a, Vocation::Mason);
        set_voc(&mut g, b, Vocation::Baker);
        set_voc(&mut g, c, Vocation::Priest);
        let r = || desires::Reveal::Problem(desires::Problem::Taxed);
        desires::hear(&mut g.world, pe, a, r(), 10);
        desires::hear(&mut g.world, pe, b, r(), 10);
        let issue = desires::hear(&mut g.world, pe, c, r(), 10);
        assert_eq!(issue.as_deref(), Some("the tax collector's cut"),
            "the commons converge whatever the room's mix");
    }
}

#[cfg(test)]
mod succession_tests {
    use super::*;

    #[test]
    fn test_the_bench_passes_to_the_apprentice() {
        let mut g = Game::new();
        // a shop master and a junior bound to them
        let master = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v)
                .map(|c| c.workplace.is_some() && c.vocation != Vocation::Farmer).unwrap_or(false)).unwrap();
        let shop = g.world.get_component::<Craft>(master).unwrap().workplace.unwrap();
        let junior = g.villagers.iter().copied().find(|&v|
            v != master && g.world.get_component::<Kid>(v).is_none()
            && g.world.get_component::<Craft>(v).is_some()).unwrap();
        g.world.get_component_mut::<Craft>(junior).unwrap().master = Some(master);
        g.world.get_component_mut::<Craft>(junior).unwrap().workplace = None;
        g.world.get_component_mut::<Purse>(master).unwrap().gold = 10.0;
        let before = g.world.get_component::<Purse>(junior).unwrap().gold;
        g.world.add_component(master, villager::Dead);
        g.succession_tick(1);
        assert_eq!(g.world.get_component::<Craft>(junior).unwrap().workplace, Some(shop),
            "the apprentice takes the bench");
        assert_eq!(g.world.get_component::<Building>(shop).unwrap().worker, Some(junior));
        let after = g.world.get_component::<Purse>(junior).unwrap().gold;
        assert!((after - before - 8.0).abs() < 0.01, "four parts pass, the fifth is the lord's");
        assert!(desires::problems_of(&g.world, junior, 1).contains(&desires::Problem::DeathDuty),
            "the death-duty is grievance");
    }

    #[test]
    fn test_a_shop_with_no_heir_stands_shuttered() {
        let mut g = Game::new();
        // a master with no apprentice, no kin at home, and no journeyman
        let master = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v)
                .map(|c| c.workplace.is_some() && c.vocation != Vocation::Farmer).unwrap_or(false)).unwrap();
        let shop = g.world.get_component::<Craft>(master).unwrap().workplace.unwrap();
        // clear every possible heir path
        let mvoc = g.world.get_component::<Craft>(master).unwrap().vocation;
        for &v in g.villagers.clone().iter() {
            if let Some(c) = g.world.get_component_mut::<Craft>(v) {
                if c.master == Some(master) { c.master = None; }
                if c.vocation == mvoc && v != master { c.vocation = Vocation::Farmer; }
            }
        }
        if let Some(home) = household::home_of(&g.world, master) {
            for &o in g.world.get_component::<Building>(home).unwrap().occupants.clone().iter() {
                if o != master && g.world.get_component::<Kid>(o).is_none() {
                    g.world.add_component(o, villager::Dead);
                }
            }
        }
        g.world.add_component(master, villager::Dead);
        g.succession_tick(1);
        assert_eq!(g.world.get_component::<Building>(shop).unwrap().worker, None,
            "no heir - the bench stands empty");
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    #[test]
    fn test_a_kid_comes_of_age_and_takes_a_trade() {
        let mut g = Game::new();
        let kid = g.villagers[26];
        assert!(g.world.get_component::<Kid>(kid).is_some());
        g.world.get_component_mut::<KidGrowth>(kid).unwrap().age_days = vocation::COMING_OF_AGE - 1;
        g.lifecycle_tick(100);
        assert!(g.world.get_component::<Kid>(kid).is_none(), "they've grown");
        assert!(g.world.get_component::<Craft>(kid).is_some(), "they take a trade");
    }

    #[test]
    fn test_the_errand_hirer_becomes_the_master() {
        let mut g = Game::new();
        let kid = g.villagers[27];
        let master = g.villagers.iter().copied().find(|&v|
            v != kid && g.world.get_component::<Craft>(v).is_some()).unwrap();
        let mvoc = g.world.get_component::<Craft>(master).unwrap().vocation;
        {
            let grw = g.world.get_component_mut::<KidGrowth>(kid).unwrap();
            grw.age_days = vocation::COMING_OF_AGE - 1;
            grw.errands = vec![(master, 5)];
        }
        g.lifecycle_tick(100);
        let c = g.world.get_component::<Craft>(kid).unwrap();
        assert_eq!(c.vocation, mvoc, "they follow the trade they ran for");
        assert_eq!(c.master, Some(master), "the hirer stands as master");
        assert_eq!(c.tasked_by, Some(master));
    }
}

#[cfg(test)]
mod trail_tests {
    use super::*;

    #[test]
    fn test_feet_wear_the_ground_in() {
        let mut g = Game::new();
        // send someone out and back a few times - a desire-line
        // is a way walked over and over
        let who = g.villagers[0];
        let p = g.world.get_component::<Pos>(who).unwrap();
        let start = (p.x.round() as i32, p.y.round() as i32);
        for _ in 0..4 {
            let now = {
                let p = g.world.get_component::<Pos>(who).unwrap();
                (p.x.round() as i32, p.y.round() as i32)
            };
            let dest = if now == start { (start.0 + 4, start.1) } else { start };
            let path = g.map.walk_path(now, dest).unwrap_or_else(|| vec![dest]);
            g.travels.push(Travel { who, path, goal: Goal::Wander });
            for _ in 0..400 { g.advance_travels(); }
        }
        assert!(!g.trails.is_empty(), "feet leave a mark");
        assert!(g.trails.values().any(|t| t.strength > 0.2), "the way wears in");
    }

    #[test]
    fn test_an_unwalked_path_greens_over() {
        let mut g = Game::new();
        g.trails.insert((5, 5), Trail { strength: 0.4 });
        for _ in 0..30 { g.trail_tick(); }
        let s = g.trails.get(&(5, 5)).map(|t| t.strength).unwrap_or(0.0);
        assert!(s < 0.3, "the grass takes it back");
    }
}

#[cfg(test)]
mod land_tests {
    use super::*;

    #[test]
    fn test_gathering_thins_the_wood_and_the_undisturbed_regrow() {
        let mut g = Game::new();
        let who = g.villagers[0];
        g.world.get_component_mut::<Satchel>(who);
        let tile = g.map.trees[0];
        let before = g.land.tree_cover.get(&tile).copied().unwrap_or(0.85);
        g.arrive(who, Goal::Gather("wood", tile));
        let after = g.land.tree_cover.get(&tile).copied().unwrap();
        assert!(after < before, "a gather takes canopy");
        // undisturbed ground creeps back
        g.land_tick(5);
        let regrown = g.land.tree_cover.get(&tile).copied().unwrap();
        assert!(regrown > after, "undisturbed canopy regrows");
        // trampled ground does not
        g.trails.insert(tile, Trail { strength: 0.5 });
        let c = g.land.tree_cover.get(&tile).copied().unwrap();
        g.land_tick(12);
        assert_eq!(g.land.tree_cover.get(&tile).copied().unwrap(), c, "a worn tile stays bare");
    }

    #[test]
    fn test_the_pit_drowns_and_the_mason_knows_it() {
        let mut g = Game::new();
        let who = g.villagers[0];
        let t = g.pit.unwrap_or((6, 19));
        for _ in 0..20 { g.arrive(who, Goal::Gather("stone", t)); }
        assert!(g.land.pit_water >= 0.7, "the pit fills as it's dug");
        let mason = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Mason).unwrap_or(false));
        if let Some(m) = mason {
            g.land_tick(2);
            assert!(desires::problems_of(&g.world, m, 2).contains(&desires::Problem::DryPit),
                "the drowned pit lands on the trade that needs it");
        }
    }

    #[test]
    fn test_the_reach_carries_downstream_only() {
        let mut g = Game::new();
        for _ in 0..30 { g.land_tick(3); }
        let (wx0, _, _, wy1) = g.town_wall;
        // every river tile above the town stays clean
        let upstream_dirty = (0..g.map.height as i32).flat_map(|y| (0..g.map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| g.map.get(x, y) == Some(Tile::River) && y < wy1 - 4)
            .any(|t| g.land.effluent.get(&t).copied().unwrap_or(0.0) > 0.2);
        assert!(!upstream_dirty, "the stink never travels upriver");
        let _ = wx0;
    }

    #[test]
    fn test_foul_water_brings_the_waterborne_sickness() {
        let mut g = Game::new();
        // foul the water beside a home - the reach giving back what it got
        let v = g.villagers[0];
        let hp = household::home_of(&g.world, v)
            .and_then(|h| g.world.get_component::<Pos>(h).copied()).expect("villager 0 has a home");
        g.land.effluent.insert((hp.x.round() as i32 + 1, hp.y.round() as i32), 0.9);
        for _ in 0..300 { g.land_tick(2); }
        let waterborne = g.world.query::<Disease>().iter().any(|&d|
            g.world.get_component::<Disease>(d)
                .map(|dd| dd.kind.route() == illness::Route::Water).unwrap_or(false));
        assert!(waterborne, "foul water should sicken somebody");
    }

    #[test]
    fn test_clean_water_carries_no_sickness() {
        let mut g = Game::new();
        for _ in 0..100 { g.land_tick(2); }
        assert!(g.world.query::<Disease>().iter().all(|&d|
            g.world.get_component::<Disease>(d)
                .map(|dd| dd.kind.route() != illness::Route::Water).unwrap_or(true)),
            "clean water sicks no one");
    }

    #[test]
    fn test_the_water_sick_foul_the_reach_further() {
        let mut g = Game::new();
        let v = g.villagers[0];
        let home = household::home_of(&g.world, v)
            .and_then(|h| g.world.get_component::<Pos>(h).copied()).expect("villager 0 has a home");
        let d = g.world.create_entity();
        g.world.add_component(d, Disease::new(DiseaseKind::Cholera, v, 0.7));
        // lands on the foul-water check; no accretion day, so what climbs
        // is what the sick poured back
        g.land_tick(2);
        let nearest = g.reach.iter().copied()
            .min_by_key(|&(x, y)| ((x as f32 - home.x).abs() + (y as f32 - home.y).abs()) as i32)
            .expect("there's a reach");
        let e = g.land.effluent.get(&nearest).copied().unwrap_or(0.0);
        assert!(e > 0.15, "the sick pour it back fouler");
    }

    #[test]
    fn test_the_autumn_marsh_sends_the_ague() {
        let mut g = Game::new();
        // a home right on the river - the marsh's own doorstep
        let v = g.villagers[0];
        let river = (0..g.map.height as i32).flat_map(|y| (0..g.map.width as i32).map(move |x| (x, y)))
            .find(|&(x, y)| g.map.get(x, y) == Some(Tile::River)).expect("there's a river");
        let home = household::home_of(&g.world, v).expect("villager 0 has a home");
        if let Some(hp) = g.world.get_component_mut::<Pos>(home) {
            hp.x = river.0 as f32; hp.y = (river.1 - 1) as f32;
        }
        // out of season the marsh keeps its shakes
        for _ in 0..60 { g.land_tick(100); }
        assert!(g.world.query::<Disease>().iter().all(|&d|
            g.world.get_component::<Disease>(d).map(|dd| dd.kind != DiseaseKind::Ague).unwrap_or(true)),
            "the ague waits for autumn");
        // in season, it comes calling
        for _ in 0..400 { g.land_tick(247); }
        assert!(g.world.query::<Disease>().iter().any(|&d|
            g.world.get_component::<Disease>(d).map(|dd| dd.kind == DiseaseKind::Ague).unwrap_or(false)),
            "the autumn marsh should send the ague");
    }

    #[test]
    fn test_sickness_abroad_finds_a_scapegoat() {
        let mut g = Game::new();
        // two down with la grippe, symptoms showing
        for &v in g.villagers.iter().take(2) {
            let d = g.world.create_entity();
            let mut dis = Disease::new(DiseaseKind::Influenza, v, 0.6);
            dis.incubation = 0; dis.stage = Stage::Moderate;
            g.world.add_component(d, dis);
        }
        for _ in 0..30 { g.land_tick(2); }
        let blamed = g.villagers.iter().copied().any(|v|
            desires::problems_of(&g.world, v, 2).contains(&desires::Problem::Blamed));
        assert!(blamed, "when sickness is abroad, somebody takes the blame");
    }

    #[test]
    fn test_a_thin_wood_troubles_the_toolmaker() {
        let mut g = Game::new();
        for c in g.land.tree_cover.values_mut() { *c = 0.2; }
        let tool = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Toolmaker).unwrap_or(false));
        if let Some(t) = tool {
            g.land_tick(2);
            assert!(desires::problems_of(&g.world, t, 2).contains(&desires::Problem::ThinWood),
                "the shrinking wood lands on the wood trade");
        }
    }
}

#[cfg(test)]
mod sanction_tests {
    use super::*;

    fn grieve(g: &mut Game, v: Entity, n: usize, tick: u64) {
        let kinds = [desires::Problem::Blamed, desires::Problem::Broke,
            desires::Problem::Smelly, desires::Problem::SourGut,
            desires::Problem::NoHome, desires::Problem::Taxed];
        for i in 0..n {
            desires::add_grievance(&mut g.world, v, kinds[i % kinds.len()], tick + 60);
        }
    }

    #[test]
    fn test_the_ladder_climbs_one_rung_at_a_time() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        grieve(&mut g, v, 13, 4);
        g.sanction_tick(4);
        // thirteen grievances earn the stocks, but the ladder is climbed -
        // the first landing is only the talk
        assert!(g.sanctioned.iter().any(|(e, s, _)| *e == v && *s == Sanction::Gossip),
            "it starts with a bad word");
        assert!(!g.sanctioned.iter().any(|(e, s, _)| *e == v && *s > Sanction::Gossip),
            "nobody goes straight to the stocks");
    }

    #[test]
    fn test_the_boycott_dries_up_the_shop() {
        let mut g = Game::new();
        let shop = g.building_entities.iter().copied().find(|&b|
            g.world.get_component::<Building>(b).map(|b| b.worker.is_some()).unwrap_or(false)).unwrap();
        let worker = g.world.get_component::<Building>(shop).unwrap().worker.unwrap();
        assert!(!g.boycotted(shop));
        g.sanctioned.push((worker, Sanction::Boycott, 100));
        assert!(g.boycotted(shop), "a boycotted shop's trade dries up");
        assert!(g.shunned(worker), "the boycott is worse than the cold shoulder");
    }

    #[test]
    fn test_a_respected_soul_walks_free_of_the_stocks() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        g.world.add_component(v, ReputationState {
            stage: RepStage::Esteemed, score: 0.8, transition_rates: vec![] });
        let before = g.world.get_component::<ReputationState>(v).unwrap().score;
        g.apply_sanction(v, Sanction::Stocks, 10);
        let after = g.world.get_component::<ReputationState>(v).unwrap().score;
        assert!(after > before, "the crowd's sympathy lifts the wrongly shamed");
        assert!(g.log.iter().any(|l| l.contains("would have none of it")));
    }

    #[test]
    fn test_the_worst_walk_out() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        g.apply_sanction(v, Sanction::Exile, 10);
        assert!(g.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::Exile)),
            "the exiled take the road");
        // and at the world's edge they're dead to the village
        g.arrive(v, Goal::Exile);
        assert!(villager::is_dead(&g.world, v), "gone is gone");
    }
}

#[cfg(test)]
mod festival_tests {
    use super::*;

    #[test]
    fn test_the_feast_gathers_the_village() {
        let mut g = Game::new();
        g.festival_tick(45);
        assert!(g.log.iter().any(|l| l.contains("the village gathers")),
            "the log names the feast");
        let walkers = g.travels.iter()
            .filter(|t| matches!(t.goal, Goal::HeadHome(_))).count();
        assert!(walkers > 10, "the whole village walks to market");
        // and the collector comes uninvited
        let c = g.collector.unwrap();
        assert!(g.travels.iter().any(|t| t.who == c), "he always shows");
    }

    #[test]
    fn test_the_feast_feeds_the_wants() {
        let mut g = Game::new();
        // hungry and joyless before the feast, and the store stocked for it
        for &v in g.villagers.clone().iter() {
            if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 20.0; }
        }
        if let Some(b) = g.world.get_component_mut::<Building>(g.building_entities[5]) {
            household::add_stock(b, "bread", 20);
        }
        g.festival_tick(45);
        let fed = g.villagers.iter().filter(|&&v| {
            g.world.get_component::<Hunger>(v).map(|h| h.value > 30.0).unwrap_or(false)
        }).count();
        assert!(fed > 10, "the good stock feeds the crowd");
    }
}

#[cfg(test)]
mod garden_tests {
    use super::*;

    #[test]
    fn test_the_cottages_have_gardens() {
        let g = Game::new();
        assert!(g.gardens.len() >= 4, "some homes keep a bed");
        for (home, _, _, _) in &g.gardens {
            assert_eq!(g.world.get_component::<Building>(*home).unwrap().kind,
                household::BuildingKind::Home);
        }
    }

    #[test]
    fn test_a_full_bed_yields_into_the_house() {
        let mut g = Game::new();
        let gi = 0;
        g.gardens[gi].2 = 1.0;
        let home = g.gardens[gi].0;
        g.arrive(g.villagers[0], Goal::TendGarden(gi));
        let b = g.world.get_component::<Building>(home).unwrap();
        let picked = b.stock.get("marigold").copied().unwrap_or(0)
            + b.stock.get("feverfew").copied().unwrap_or(0);
        assert_eq!(picked, 1, "a full bed gives a bunch");
        assert_eq!(g.gardens[gi].2, 0.0, "the bed's picked clean");
    }

    #[test]
    fn test_the_bed_fills_in_over_the_weeks() {
        let mut g = Game::new();
        let g0 = g.gardens[0].2;
        g.garden_tick(5);
        assert!(g.gardens[0].2 > g0, "it grows between pickings");
    }

    #[test]
    fn test_a_kitchen_bed_grows_dinner_not_flowers() {
        let mut g = Game::new();
        // every other bed keeps a kitchen garden
        let gi = g.gardens.iter().position(|(_, _, _, k)| *k)
            .expect("some beds are kitchen beds");
        g.gardens[gi].2 = 1.0;
        let home = g.gardens[gi].0;
        g.tick = 0;
        g.arrive(g.villagers[0], Goal::TendGarden(gi));
        let b = g.world.get_component::<Building>(home).unwrap();
        let pot = b.stock.get("roots").copied().unwrap_or(0)
            + b.stock.get("potherbs").copied().unwrap_or(0)
            + b.stock.get("peas").copied().unwrap_or(0);
        assert_eq!(pot, 1, "a kitchen bed gives the pot, not the still");
        assert_eq!(b.stock.get("marigold").copied().unwrap_or(0)
            + b.stock.get("feverfew").copied().unwrap_or(0), 0);
    }

    #[test]
    fn test_the_pot_turns_three_staples_into_pottage() {
        let mut g = Game::new();
        let home = g.building_entities.iter().copied().find(|&b| {
            g.world.get_component::<Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)
        }).unwrap();
        {
            let b = g.world.get_component_mut::<Building>(home).unwrap();
            for good in ["peas", "roots", "potherbs"] { household::add_stock(b, good, 1); }
        }
        g.kitchen_tick(3); // pot day
        let b = g.world.get_component::<Building>(home).unwrap();
        assert_eq!(b.stock.get("pottage").copied().unwrap_or(0), 2);
        assert_eq!(b.stock.get("peas").copied().unwrap_or(0), 0);
    }
}

#[cfg(test)]
mod weaver_tests {
    use super::*;

    #[test]
    fn test_a_weaver_keeps_the_loom() {
        let g = Game::new();
        let w = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Weaver).unwrap_or(false));
        assert!(w.is_some(), "one weaver works the loom");
        let w = w.unwrap();
        let loom = g.world.get_component::<Craft>(w).unwrap().workplace;
        if let Some(l) = loom {
            assert_eq!(g.world.get_component::<Building>(l).unwrap().kind,
                household::BuildingKind::Shop(Vocation::Weaver));
        }
        assert_eq!(vocation::inputs(Vocation::Weaver), &["wool"]);
    }

    #[test]
    fn test_the_loom_turns_wool_to_cloth() {
        let mut g = Game::new();
        let w = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Weaver).unwrap_or(false)).unwrap();
        // a bolt of wool in the satchel, a day's work, cloth out
        if let Some(s) = g.world.get_component_mut::<Satchel>(w) {
            *s.goods.entry("wool".to_string()).or_insert(0) += 2;
        }
        let (made, _) = livelihood::work_day(&mut g.world, w, 10);
        assert!(made > 0.0, "the loom runs");
        let cloth = g.world.get_component::<Satchel>(w)
            .map(|s| s.goods.get("cloth").copied().unwrap_or(0)).unwrap_or(0)
            + g.world.get_component::<Craft>(w).and_then(|c| c.workplace)
                .and_then(|b| g.world.get_component::<Building>(b))
                .map(|b| b.stock.get("cloth").copied().unwrap_or(0)).unwrap_or(0);
        assert!(cloth > 0, "wool in, cloth out");
    }
}

#[cfg(test)]
mod cheese_tests {
    use super::*;

    #[test]
    fn test_the_cellar_ages_wheels() {
        let mut g = Game::new();
        assert!(g.cellar.is_some(), "a cellar stands by the fold");
        assert_eq!(g.world.get_component::<Building>(g.cellar.unwrap()).unwrap().kind,
            household::BuildingKind::Cellar);
        // milk in the fold makes wheels in the dark
        if let Some(b) = g.world.get_component_mut::<Building>(g.fold.unwrap()) {
            household::add_stock(b, "milk", 6);
        }
        for t in [6u64, 13, 20] { g.cheese_tick(t); }
        assert!(!g.cheeses.is_empty(), "milk down becomes wheels");
    }

    #[test]
    fn test_a_noble_pays_triple_for_an_aged_wheel() {
        let mut g = Game::new();
        g.cheeses = vec![0]; // a wheel laid down at the start of time
        let sh = g.shepherd.unwrap();
        g.world.get_component_mut::<Purse>(sh).unwrap().gold = 0.0;
        g.cheese_tick(75);
        let gold = g.world.get_component::<Purse>(sh).unwrap().gold;
        assert!(gold > vocation::price("cheese") * 2.0, "aged cheese pays triple");
        assert!(g.log.iter().any(|l| l.contains("stone-cellar")),
            "the village's cheese has a name");
    }
}

#[cfg(test)]
mod smell_tests {
    use super::*;

    #[test]
    fn test_the_fold_smells_of_the_fold() {
        let mut g = Game::new();
        let sh = g.shepherd.unwrap();
        for _ in 0..5 { g.smell_tick(1); }
        assert_eq!(g.smells_of(sh), Some("the fold"), "a week of the fold tells on a body");
        // the rain washes everyone clean
        g.precip = Some(crate::sim::precipitation::PrecipKind::Rain);
        g.smell_tick(2);
        assert!(g.smells_of(sh).is_none(), "a good rain settles it");
    }

    #[test]
    fn test_the_herb_wife_carries_bitter_herbs() {
        let mut g = Game::new();
        let w = g.herbologist.unwrap();
        for _ in 0..5 { g.smell_tick(1); }
        assert_eq!(g.smells_of(w), Some("bitter herbs"));
        // the tanner's clean by comparison - a farmer smells of nothing much
        let farmer = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Farmer).unwrap_or(false)).unwrap();
        assert!(g.smells_of(farmer).is_none(), "a farmer smells of nothing much");
    }
}

#[cfg(test)]
mod power_tests {
    use super::*;

    fn capacity(g: &Game) -> f32 {
        g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
            .map(|p| p.capacity).unwrap_or(0.0)
    }

    #[test]
    fn test_power_builds_as_its_spent_and_atrophies_idle() {
        let mut g = Game::new();
        assert_eq!(capacity(&g), 0.0, "you're nobody until you've shown power");
        g.record_power(crate::sim::organized_power::ActionQuality::Transformational);
        g.record_power(crate::sim::organized_power::ActionQuality::Transformational);
        let high = capacity(&g);
        assert!(high > 0.2, "meetings in the street build it");
        for t in (3..60).step_by(7) { g.power_tick(t); }
        assert!(capacity(&g) < high, "power atrophies when idle - unlike gold");
    }

    #[test]
    fn test_the_lords_estimate_lags_and_shifts_the_tax() {
        let mut g = Game::new();
        // a strong org takes weeks to register with the lord
        for _ in 0..10 { g.record_power(crate::sim::organized_power::ActionQuality::Transformational); }
        assert!(capacity(&g) > 1.0);
        for t in (3..200).step_by(7) { g.power_tick(t); }
        assert!(g.lord_esteem > 0.6, "eventually the lord learns");
        // and then the collector passes the player by
        g.world.get_component_mut::<Purse>(g.player).unwrap().gold = 100.0;
        g.arrive(g.collector.unwrap(), Goal::CollectTax(g.player));
        assert_eq!(g.world.get_component::<Purse>(g.player).unwrap().gold, 100.0,
            "a force is passed by");
    }
}

#[cfg(test)]
mod pedlar_tests {
    use super::*;

    #[test]
    fn test_the_pedlar_pitches_and_sells() {
        let mut g = Game::new();
        // two hearts for finery, whatever the want-roll dealt the village
        for &v in &g.villagers[..2] {
            if let Some(l) = g.world.get_component_mut::<InnerLife>(v) {
                if !l.wants.contains(&Want::Finery) { l.wants.push(Want::Finery); }
            }
        }
        g.spawn_pedlar();
        assert!(g.pedlar.is_some());
        // walk him in - the pitch happens once he's stood at market
        for _ in 0..400 {
            if g.pedlar.as_ref().map(|p| p.selling_until.is_some()).unwrap_or(false) { break; }
            g.advance_pedlar();
        }
        let m = g.pedlar.as_ref().unwrap();
        assert!(m.stall.is_some(), "he pitches a stall");
        let stall = m.stall.unwrap();
        let ribbon = g.world.get_component::<Building>(stall)
            .map(|b| b.stock.get("ribbon").copied().unwrap_or(0)).unwrap_or(0);
        assert!(ribbon > 0, "oddities nothing local supplies");
        assert!(g.travels.iter().any(|t| matches!(t.goal, Goal::BuyFrom(s, "ribbon", _)
            if s == stall)), "the word goes out to the finery-wanters");
    }

    #[test]
    fn test_the_road_takes_him() {
        let mut g = Game::new();
        g.spawn_pedlar();
        for _ in 0..400 {
            if g.pedlar.as_ref().map(|p| p.path.is_empty() && p.selling_until.is_some()).unwrap_or(false) { break; }
            g.advance_pedlar();
        }
        // run him past his selling days
        g.tick = 10;
        for _ in 0..900 {
            g.advance_pedlar();
            if g.pedlar.is_none() { break; }
        }
        assert!(g.pedlar.is_none(), "the road takes him");
        assert!(g.next_pedlar > g.tick, "another comes by one day");
    }
}

#[cfg(test)]
mod perfume_tests {
    use super::*;

    #[test]
    fn test_the_perfumer_keeps_the_still() {
        let g = Game::new();
        let p = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Perfumer).unwrap_or(false));
        assert!(p.is_some(), "a perfumer keeps the still");
        // Hungary water: garden flowers steeped in the water of life
        assert_eq!(vocation::inputs(Vocation::Perfumer), &["marigold", "lavender", "spirits"]);
        assert_eq!(vocation::output(Vocation::Perfumer), Some("perfume"));
    }

    #[test]
    fn test_a_dab_of_perfume_smells_of_roses() {
        let mut g = Game::new();
        let buyer = g.villagers[0];
        let shop = g.building_entities.iter().copied().find(|&b|
            g.world.get_component::<Building>(b).map(|bb| bb.worker.is_some()).unwrap_or(false)).unwrap();
        // stock the shelf, give the buyer a purse, and let them buy
        g.world.get_component_mut::<Building>(shop).unwrap()
            .stock.insert("perfume".to_string(), 2);
        g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 30.0;
        g.arrive(buyer, Goal::BuyFrom(shop, "perfume", 1));
        assert_eq!(g.smells_of(buyer), Some("roses"), "the buyer smells of roses");
        // and perfume is no smell at all to the heart-helper
        let smelly = [buyer].iter().any(|&v|
            g.smells_of(v).map(|k| k != "roses").unwrap_or(false));
        assert!(!smelly, "a dab of roses counts for a clean suitor");
    }
}

#[cfg(test)]
mod reckoning_tests {
    use super::*;

    #[test]
    fn test_the_reckoning_loses_a_little() {
        let mut g = Game::new();
        g.numeracy = Numeracy::Tally;
        let buyer = g.villagers[0];
        g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 50.0;
        let stall = g.trader_stall.unwrap();
        g.world.get_component_mut::<Building>(stall).unwrap()
            .stock.insert("cloth".to_string(), 5);
        g.arrive(buyer, Goal::BuyFrom(stall, "cloth", 1));
        // the price plus the foreign count's bite
        let gold = g.world.get_component::<Purse>(buyer).unwrap().gold;
        assert!(gold < 50.0 - vocation::price("cloth"), "the reckoning takes its bite");
        assert!(g.reckonings >= 1);
    }

    #[test]
    fn test_a_clerk_steps_up_for_the_tallies() {
        let mut g = Game::new();
        g.numeracy = Numeracy::Tally;
        let buyer = g.villagers[0];
        g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 500.0;
        let stall = g.trader_stall.unwrap();
        g.world.get_component_mut::<Building>(stall).unwrap()
            .stock.insert("cloth".to_string(), 20);
        for t in 0..10 { g.arrive(buyer, Goal::BuyFrom(stall, "cloth", 1)); let _ = t; }
        assert!(g.translator.is_some(), "enough bad reckonings and a clerk steps up");
        // and with a clerk, the count comes out right
        let before = g.world.get_component::<Purse>(buyer).unwrap().gold;
        g.arrive(buyer, Goal::BuyFrom(stall, "cloth", 1));
        let gold = g.world.get_component::<Purse>(buyer).unwrap().gold;
        assert!(gold >= before - vocation::price("cloth") - 0.01,
            "he keeps the tallies straight");
    }
}

#[cfg(test)]
mod sky_tests {
    use super::*;

    #[test]
    fn test_a_comet_rides_and_everyone_reads_it() {
        let mut g = Game::new();
        g.sky_tick(150);
        assert!(g.sky.is_some(), "a comet rides the walls");
        assert!(g.log.iter().any(|l| l.contains("rides over the walls")));
        let omens = g.bubbles.iter().filter(|b| b.0 != g.player).count();
        assert!(omens > 10, "a crowd of small opinions");
        // the priest reads it his way
        let priest = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false));
        if let Some(p) = priest {
            let said = g.bubbles.iter().find(|b| b.0 == p).map(|b| b.1.as_str());
            assert_eq!(said, Some("A sign. Pray."), "the priest sees warning");
        }
        // and it burns out
        g.sky_tick(160);
        assert!(g.sky.is_none(), "it burns for three nights, no more");
    }
}

#[cfg(test)]
mod disclosure_tests {
    use super::*;

    #[test]
    fn test_a_truth_ripens_and_comes_out() {
        let mut g = Game::new();
        let cleric = g.villagers[5];
        g.translator = Some(cleric);
        g.discovery_tick(20);
        assert!(g.pending_discovery.is_some(), "a truth sits unsaid");
        g.discovery_tick(50); // past the fortnight
        assert!(g.disclosure_made, "the word's out");
        assert!(g.log.iter().any(|l| l.contains("whispers it about")),
            "it comes out in a whisper");
        // weak org: the threatened move - blame lands on the discoverer
        let blame = desires::problems_of(&g.world, cleric, 50)
            .iter().filter(|p| **p == desires::Problem::Blamed).count();
        assert!(blame >= 1, "the ladder's there for a man who talks treason");
    }

    #[test]
    fn test_a_strong_org_stands_with_its_own() {
        let mut g = Game::new();
        let cleric = g.villagers[5];
        g.translator = Some(cleric);
        g.lord_esteem = 0.5;
        g.pending_discovery = Some((cleric, "the lord's fifth is really a quarter", 20));
        g.discovery_tick(50);
        assert!(g.disclosure_made);
        let blame = desires::problems_of(&g.world, cleric, 50)
            .iter().filter(|p| **p == desires::Problem::Blamed).count();
        assert_eq!(blame, 0, "the village stands with him");
        assert!(g.log.iter().any(|l| l.contains("stands with")));
    }
}

#[cfg(test)]
mod circus_tests {
    use super::*;

    #[test]
    fn test_the_circus_camps_and_tempts() {
        let mut g = Game::new();
        g.spawn_circus();
        // walk it in
        for _ in 0..600 {
            if g.circus.as_ref().map(|c| c.camped_until.is_some()).unwrap_or(false) { break; }
            g.advance_circus();
        }
        assert!(g.circus.as_ref().map(|c| c.camped_until.is_some()).unwrap_or(false),
            "the tents go up on the meadow");
        // a villager by the camp gets the wonder and feels the pull
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        let camp = g.circus.as_ref().unwrap().camp;
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: camp.1 as f32 };
        g.circus_tick(1);
        assert!(desires::problems_of(&g.world, v, 1).contains(&desires::Problem::Tempted),
            "the bored feel the road's pull");
    }

    #[test]
    fn test_the_loneliest_runs_off_with_it() {
        let mut g = Game::new();
        g.spawn_circus();
        for _ in 0..600 {
            if g.circus.as_ref().map(|c| c.camped_until.is_some()).unwrap_or(false) { break; }
            g.advance_circus();
        }
        // tempt someone, then let the tents fold
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        desires::add_grievance(&mut g.world, v, desires::Problem::Tempted, 100);
        let until = g.circus.as_ref().unwrap().camped_until.unwrap();
        g.tick = until;
        g.circus_tick(until);
        assert!(g.circus.is_none(), "the tents fold");
        if g.travels.iter().any(|t| matches!(t.goal, Goal::Exile)) {
            assert!(g.log.iter().any(|l| l.contains("run off with the circus")),
                "the road takes the one it tempted");
        }
    }
}

#[cfg(test)]
mod ink_boiler_tests {
    use super::*;

    #[test]
    fn test_he_pitches_outside_the_walls() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        assert!(g.ink_boiler.is_some());
        // walk him out to the meadow
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        let b = g.ink_boiler.as_ref().unwrap();
        assert!(b.working_until.is_some(), "the cauldron's set up");
        // the law wants a fire like his beyond the south wall
        assert!(b.camp.1 > g.town_wall.3, "he boils outside the gate, not in the street");
        assert!(!g.log.iter().any(|l| l.is_empty()));
    }

    #[test]
    fn test_the_road_takes_him_too() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.path.is_empty() && b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        // run him past his boiling days
        g.tick = 100;
        for _ in 0..2000 {
            g.advance_ink_boiler();
            if g.ink_boiler.is_none() { break; }
        }
        assert!(g.ink_boiler.is_none(), "the road takes him");
        assert!(g.next_ink_boiler > g.tick, "he'll be back in a year or so");
    }

    #[test]
    fn test_he_shows_up_on_the_map() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        let e = g.ink_boiler.as_ref().unwrap().e;
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.e == e && matches!(p.kind, PKind::Traveller)),
            "the ink-maker is on the map the moment he's spawned, not just his speech bubble");
    }

    #[test]
    fn test_the_recipe_walks_its_days() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        let pitched = g.ink_boiler.as_ref().unwrap().working_until.unwrap() - INK_BOIL_DAYS;
        for d in 0..INK_BOIL_DAYS { g.ink_boiler_tick(pitched + d); }
        assert!(g.log.iter().any(|l| l.contains("varnish")), "the slate says varnish");
        // the print shop - or the trader if there's no printer - is stocked
        let stocked = g.building_entities.iter().copied().find(|&e| {
            g.world.get_component::<Building>(e)
                .map(|b| matches!(b.kind, household::BuildingKind::Shop(Vocation::Printmaker)))
                .unwrap_or(false)
        }).or(g.trader_stall).unwrap();
        let ink = g.world.get_component::<Building>(stocked)
            .and_then(|b| b.stock.get("ink").copied()).unwrap_or(0);
        assert!(ink >= INK_YIELD, "there's ink where the road used to bring it");
    }

    #[test]
    fn test_the_crusts_come_back_fried() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        let (camp, be) = {
            let b = g.ink_boiler.as_ref().unwrap();
            (b.camp, b.e)
        };
        // someone walks out with the day's crusts
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 - 2) as f32 };
        let pitched = g.ink_boiler.as_ref().unwrap().working_until.unwrap() - INK_BOIL_DAYS;
        g.tick = pitched + 2;
        g.ink_boiler_tick(g.tick);
        assert!(g.travels.iter().any(|t| matches!(t.goal, Goal::TalkTo(e) if e == be)),
            "the crusts get carried out to the pot");
        // and the bearer gets the taste of them
        g.arrive(v, Goal::TalkTo(be));
        assert!(desires::is_satisfied(&g.world, v, Want::GoodFood, g.tick),
            "they come back fried");
    }

    #[test]
    fn test_the_meadow_can_burn() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        let camp = g.ink_boiler.as_ref().unwrap().camp;
        assert!(g.fire.ignite(&g.map, camp.0, camp.1), "the cauldron's tile can take fire");
        // it runs its course and leaves scorched ground behind
        let mut scorched_any = false;
        for t in 1..40 {
            g.fire_tick(t);
            if !g.fire.scorched.is_empty() { scorched_any = true; break; }
        }
        assert!(scorched_any, "it burned out to scorched ground");
    }

    #[test]
    fn test_the_smoke_reaches_the_downwind() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        let camp = g.ink_boiler.as_ref().unwrap().camp;
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 - 3) as f32 };
        let pitched = g.ink_boiler.as_ref().unwrap().working_until.unwrap() - INK_BOIL_DAYS;
        g.tick = pitched;
        g.ink_boiler_tick(g.tick);
        assert!(desires::problems_of(&g.world, v, g.tick).contains(&desires::Problem::Smelly),
            "the acrolein's sharp on the eyes");
    }

    #[test]
    fn test_folk_beside_the_flames_run_for_home() {
        let mut g = Game::new();
        g.spawn_ink_boiler();
        for _ in 0..600 {
            if g.ink_boiler.as_ref().map(|b| b.working_until.is_some()).unwrap_or(false) { break; }
            g.advance_ink_boiler();
        }
        let camp = g.ink_boiler.as_ref().unwrap().camp;
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 + 1) as f32 };
        g.fire.ignite(&g.map, camp.0, camp.1);
        g.fire_tick(1);
        assert!(g.travels.iter().any(|t| t.who == v && matches!(t.goal, Goal::HeadHome(_))),
            "she runs for home");
        // and mid-errand is no excuse to walk through flames
        let v2 = g.villagers.iter().copied().find(|&x| x != v
            && g.world.get_component::<TownRole>(x).is_none()).unwrap();
        *g.world.get_component_mut::<Pos>(v2).unwrap() = Pos { x: camp.0 as f32, y: (camp.1 + 1) as f32 };
        g.travels.push(Travel { who: v2, path: vec![(5, 5)], goal: Goal::Wander });
        g.fire_tick(2);
        assert!(g.travels.iter().all(|t| t.who != v2 || matches!(t.goal, Goal::HeadHome(_))),
            "the errand's dropped for the run home");
    }
}

#[cfg(test)]
mod edict_tests {
    use super::*;

    #[test]
    fn test_a_writ_levies_the_store() {
        let mut g = Game::new();
        if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
            household::add_stock(b, "grain", 20);
        }
        g.edict_tick(80);
        assert!(g.log.iter().any(|l| l.contains("bailiff's writ")), "a writ comes down");
        let grain = g.world.get_component::<Building>(g.market_food)
            .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0);
        assert!(grain < 20, "the levy takes its share");
        let grieved = g.villagers.iter().filter(|&&v|
            desires::problems_of(&g.world, v, 80).contains(&desires::Problem::WritBorne)).count();
        assert!(grieved > 0, "the writ's a thing to organize about");
    }

    #[test]
    fn test_a_strong_org_stands_against_the_summons() {
        let mut g = Game::new();
        g.lord_esteem = 0.8;
        let m = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        for _ in 0..4 { desires::add_grievance(&mut g.world, m, desires::Problem::Broke, 100); }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
        // summons day: tick 320 -> (320/120)%3 == 2
        g.edict_tick(320);
        assert!(g.log.iter().any(|l| l.contains("stands against")),
            "a strong org says no");
        assert!(g.repression_debt > 0, "the defiance will be paid for");
        assert!(!g.travels.iter().any(|t| t.who == m && matches!(t.goal, Goal::Exile)),
            "he stays - for now");
    }
}

#[cfg(test)]
mod winter_tests {
    use super::*;

    #[test]
    fn test_the_winter_runs_long_and_the_store_thins() {
        let mut g = Game::new();
        g.winter_tick(400);
        assert!(g.harsh_winter, "every other winter's a hard one");
        assert!(g.log.iter().any(|l| l.contains("sets in hard")));
        if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
            household::add_stock(b, "grain", 20);
        }
        for t in (400..410).step_by(5) { g.winter_tick(t); }
        let grain = g.world.get_component::<Building>(g.market_food)
            .map(|b| b.stock.get("grain").copied().unwrap_or(0)).unwrap_or(0);
        assert!(grain < 20, "the stores run thin");
    }

    #[test]
    fn test_a_weak_village_looks_for_who_to_blame() {
        let mut g = Game::new();
        g.lord_esteem = 0.0;
        let herb = g.herbologist.unwrap();
        for t in (400..500).step_by(30) { g.winter_tick(t); } // 400,430,460,490 - all %30==10 in the window
        assert!(desires::problems_of(&g.world, herb, 500).contains(&desires::Problem::Blamed),
            "the herb-wife's hut looks different in a hard winter");
    }

    #[test]
    fn test_an_org_worth_the_name_pulls_together() {
        let mut g = Game::new();
        g.lord_esteem = 0.6;
        for v in g.villagers.iter().take(6).copied().collect::<Vec<_>>() {
            if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 40.0; }
        }
        // tick 1150: 1150 % 730 = 420 (in the harsh window), 1150 % 60 = 10 (the share-out day)
        g.winter_tick(1150);
        assert!(g.log.iter().any(|l| l.contains("pulls together")),
            "an org shares what it has");
        // and the herb-wife takes no blame for a shared winter
        let herb = g.herbologist.unwrap();
        assert!(!desires::problems_of(&g.world, herb, 1150).contains(&desires::Problem::Blamed),
            "belonging is the counterweight");
    }
}

#[cfg(test)]
mod painter_tests {
    use super::*;

    #[test]
    fn test_a_hall_gets_its_colors() {
        let mut g = Game::new();
        let painter = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Painter).unwrap_or(false));
        let Some(p) = painter else { panic!("no painter") };
        let shop = g.world.get_component::<Craft>(p).unwrap().workplace.unwrap();
        g.world.get_component_mut::<Building>(shop).unwrap()
            .stock.insert("paint".to_string(), 2);
        g.painter_tick(7);
        assert!(!g.painted.is_empty(), "a hall gets its colors");
        assert!(g.log.iter().any(|l| l.contains("gets its colors")));
    }

    #[test]
    fn test_the_color_fades() {
        let mut g = Game::new();
        g.painted.push((g.building_entities[0], 10));
        g.painter_tick(400);
        assert!(g.painted.is_empty(), "the seasons take the color back");
    }
}

#[cfg(test)]
mod heirloom_tests {
    use super::*;

    #[test]
    fn test_a_maker_lays_down_a_masterwork() {
        let mut g = Game::new();
        // a sure pair of hands
        let maker = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c|
                matches!(c.vocation, Vocation::Luthier | Vocation::Glazier | Vocation::Toolmaker
                    | Vocation::Weaver | Vocation::Tailor)).unwrap_or(false)).unwrap();
        g.world.get_component_mut::<Craft>(maker).unwrap().skill = 0.9;
        g.heirloom_tick(60);
        assert!(!g.heirlooms.is_empty(), "a thing made to outlast its maker");
        let h = g.world.get_component::<villager::Heirloom>(g.heirlooms[0]).unwrap();
        assert_eq!(h.maker, maker);
        assert_eq!(h.owner, maker, "the maker keeps it first");
    }

    #[test]
    fn test_the_dead_pass_their_things_on() {
        let mut g = Game::new();
        let owner = g.villagers[0];
        let item = g.world.create_entity();
        g.world.add_component(item, villager::Heirloom {
            maker: owner, owner, name: "a lute", born_tick: 10, stories: vec![] });
        g.heirlooms.push(item);
        // someone under the same roof inherits
        let home = household::home_of(&g.world, owner).unwrap();
        let kin = g.world.get_component::<Building>(home).unwrap()
            .occupants.iter().copied().find(|&o| o != owner).unwrap();
        g.world.get_component_mut::<Health>(owner).unwrap().condition = 0.0;
        g.arrive(owner, Goal::Wander); // a tick to mark him dead? no - die directly
        g.world.add_component(owner, villager::Dead);
        g.heirloom_tick(11);
        let h = g.world.get_component::<villager::Heirloom>(item).unwrap();
        assert_eq!(h.owner, kin, "a bequest picks kin");
        assert!(!h.stories.is_empty(), "it gathers the story of the passing");
    }
}

#[cfg(test)]
mod letter_tests {
    use super::*;

    #[test]
    fn test_a_letter_comes_for_them() {
        let mut g = Game::new();
        let from = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).unwrap();
        let to = g.villagers[0];
        g.letters.push((from, to, 10));
        g.letter_tick(15);
        assert!(g.letters.is_empty(), "it arrived");
        assert!(g.log.iter().any(|l| l.contains("a letter comes")), "word travels");
    }

    #[test]
    fn test_the_illiterate_pay_the_priest_a_penny() {
        let mut g = Game::new();
        let priest = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).unwrap();
        let reader = g.villagers.iter().copied().find(|&v| v != priest && !g.literate(v)).unwrap();
        g.world.get_component_mut::<Purse>(reader).unwrap().gold = 10.0;
        g.world.get_component_mut::<Purse>(priest).unwrap().gold = 0.0;
        g.letters.push((priest, reader, 10));
        g.letter_tick(15);
        let gold = g.world.get_component::<Purse>(reader).unwrap().gold;
        let priest_gold = g.world.get_component::<Purse>(priest).unwrap().gold;
        assert_eq!(gold, 9.0, "a penny for the reading");
        assert_eq!(priest_gold, 1.0, "and the penny's his");
    }

    #[test]
    fn test_the_dead_dont_say_the_office() {
        let mut g = Game::new();
        let priest = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Priest).unwrap_or(false)).unwrap();
        let reader = g.villagers.iter().copied().find(|&v| v != priest && !g.literate(v)).unwrap();
        g.world.get_component_mut::<Purse>(reader).unwrap().gold = 10.0;
        g.world.get_component_mut::<Purse>(priest).unwrap().gold = 0.0;
        g.world.add_component(priest, villager::Dead);
        g.letters.push((priest, reader, 10));
        g.letter_tick(15);
        let gold = g.world.get_component::<Purse>(reader).unwrap().gold;
        let priest_gold = g.world.get_component::<Purse>(priest).unwrap().gold;
        assert_eq!(gold, 10.0, "no office read, no penny paid");
        assert_eq!(priest_gold, 0.0, "the dead take no fee");
        assert!(g.log.iter().any(|l| l.contains("a letter comes")), "the letter still lands");
    }
}

#[cfg(test)]
mod care_tests {
    use super::*;

    #[test]
    fn test_the_village_walks_slower_for_a_lame_leg() {
        let g = Game::new();
        let lame = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<villager::Condition>(v)
                .map(|c| c.kind == "a lame leg").unwrap_or(false));
        assert!(lame.is_some(), "someone carries a difference");
    }

    #[test]
    fn test_theirs_check_on_them() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
        let home = household::home_of(&g.world, v).unwrap();
        let kin = g.world.get_component::<Building>(home).unwrap()
            .occupants.iter().copied().find(|&o| o != v);
        if kin.is_some() {
            g.care_tick(2);
            // the visit warms them - Joy's satisfied
            assert!(g.world.get_component::<desires::InnerLife>(v)
                .map(|l| l.satisfied.iter().any(|(w, u)| *w == Want::Joy && *u > 2))
                .unwrap_or(false), "theirs check on them");
        }
    }

    #[test]
    fn test_the_grave_sends_no_visitors() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
        // no kin under the roof to take the visit
        if let Some(home) = household::home_of(&g.world, v) {
            g.world.get_component_mut::<Building>(home).unwrap()
                .occupants.retain(|&o| o == v);
        }
        // their best friend's gone: the strongest bond points the visit
        // at the grave - before the fix the dead still called round
        let dead_friend = g.villagers.iter().copied().find(|&x| x != v).unwrap();
        let rel = g.world.create_entity();
        g.world.add_component(rel, relationships::Relationship {
            from: dead_friend, other: v, kind: relationships::RelKind::Friend,
            strength: 0.9, interactions: 9 });
        g.world.add_component(dead_friend, villager::Dead);
        g.care_tick(2);
        let after = g.world.get_component::<relationships::Relationship>(rel).unwrap();
        assert_eq!(after.interactions, 9, "the grave sends no visitors");
        assert!(g.bubbles.iter().all(|(w, _, _, _, _)| *w != dead_friend),
            "no word from the grave");
    }

    #[test]
    fn test_the_dungeon_sends_no_visitors() {
        let mut g = Game::new();
        let v = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
        if let Some(home) = household::home_of(&g.world, v) {
            g.world.get_component_mut::<Building>(home).unwrap()
                .occupants.retain(|&o| o == v);
        }
        let held_friend = g.villagers.iter().copied().find(|&x| x != v).unwrap();
        let rel = g.world.create_entity();
        g.world.add_component(rel, relationships::Relationship {
            from: held_friend, other: v, kind: relationships::RelKind::Friend,
            strength: 0.9, interactions: 9 });
        g.world.add_component(held_friend,
            crate::game::dungeon::Imprisoned { since: 0, charge: "sedition".into() });
        g.care_tick(2);
        let after = g.world.get_component::<relationships::Relationship>(rel).unwrap();
        assert_eq!(after.interactions, 9, "the dungeon sends no visitors");
    }
}

#[cfg(test)]
mod fisher_tests {
    use super::*;

    #[test]
    fn test_the_fisher_works_the_river() {
        let mut g = Game::new();
        let fisher = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Fisher).unwrap_or(false));
        let Some(f) = fisher else { panic!("no fisher") };
        let hut = g.world.get_component::<Craft>(f).unwrap().workplace.unwrap();
        g.fisher_tick(4);
        let fish = g.world.get_component::<Building>(hut).unwrap()
            .stock.get("fish").copied().unwrap_or(0);
        assert!(fish > 0, "the river gives up its catch");
    }

    #[test]
    fn test_the_water_brings_up_something_strange() {
        let mut g = Game::new();
        g.fisher_tick(60);
        assert!(g.log.iter().any(|l| l.contains("pale stone")),
            "a pale stone, faintly lit");
    }

    #[test]
    fn test_the_gone_fisher_lands_nothing() {
        let mut g = Game::new();
        let fisher = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Fisher).unwrap_or(false));
        let Some(f) = fisher else { panic!("no fisher") };
        let hut = g.world.get_component::<Craft>(f).unwrap().workplace.unwrap();
        g.world.get_component_mut::<Building>(hut).unwrap()
            .stock.insert("fish".into(), 0);
        g.world.add_component(f, villager::Dead);
        g.fisher_tick(4);
        let fish = g.world.get_component::<Building>(hut).unwrap()
            .stock.get("fish").copied().unwrap_or(0);
        assert_eq!(fish, 0, "the river gives the dead no catch");
    }

    #[test]
    fn test_the_heir_works_the_river() {
        let mut g = Game::new();
        let fisher = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == Vocation::Fisher).unwrap_or(false));
        let Some(f) = fisher else { panic!("no fisher") };
        let hut = g.world.get_component::<Craft>(f).unwrap().workplace.unwrap();
        g.world.get_component_mut::<Building>(hut).unwrap()
            .stock.insert("fish".into(), 0);
        // the bench passes to living hands - the heir needn't carry
        // the vocation, the river only asks who's behind the bench
        let heir = g.villagers.iter().copied().find(|&v| v != f
            && !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Building>(hut).unwrap().worker = Some(heir);
        g.world.add_component(f, villager::Dead);
        g.fisher_tick(4);
        let fish = g.world.get_component::<Building>(hut).unwrap()
            .stock.get("fish").copied().unwrap_or(0);
        assert!(fish > 0, "the heir works the river");
    }
}

#[cfg(test)]
mod cave_tests {
    use super::*;

    #[test]
    fn test_the_hermit_keeps_to_the_dark() {
        let g = Game::new();
        let Some(h) = g.hermit else { panic!("no hermit") };
        assert_eq!(g.world.get_component::<TownRole>(h), Some(&TownRole::Hermit));
        assert!(desires::problems_of(&g.world, h, 1).contains(&desires::Problem::TheDark),
            "the dark keeps them");
        // and the deep patch is there for the picking
        assert!(g.mush_patches.iter().any(|p| p.tile == (35, 33) && p.kind == MushKind::UnderCap),
            "the best mushrooms grow in the under");
    }
}

#[cfg(test)]
mod continuation_tests {
    use super::*;

    #[test]
    fn test_the_work_goes_on() {
        let mut g = Game::new();
        let succ = g.villagers[5];
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(succ);
        let r = g.world.create_entity();
        g.world.add_component(r, relationships::Relationship {
            from: g.player, other: succ, kind: relationships::RelKind::Friend,
            strength: 0.9, interactions: 8 });
        g.world.add_component(g.player, villager::Dead);
        g.continuation_tick(1);
        assert_eq!(g.player, succ, "the deepest bond takes it up");
        assert!(g.log.iter().any(|l| l.contains("the work goes on")));
        assert!(g.world.get_component::<PlayerCharacter>(succ).is_some());
    }

    #[test]
    fn test_the_standing_passes_half_on() {
        let mut g = Game::new();
        let succ = g.villagers[5];
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(succ);
        g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().influence = 1.6;
        g.world.add_component(g.player, villager::Dead);
        g.continuation_tick(1);
        assert_eq!(g.player, succ);
        let p = g.world.get_component::<PlayerCharacter>(succ).unwrap();
        assert!((p.influence - 0.8).abs() < 1e-5,
            "the standing passes half on - got {}", p.influence);
    }

    #[test]
    fn test_no_hands_means_the_work_dies() {
        let mut g = Game::new();
        // no org, no kin under the roof
        if let Some(home) = household::home_of(&g.world, g.player) {
            g.world.get_component_mut::<Building>(home).unwrap().occupants.clear();
        }
        g.world.add_component(g.player, villager::Dead);
        g.continuation_tick(1);
        assert!(g.log.iter().any(|l| l.contains("the work dies with them")));
        assert!(g.player_passed);
    }
}

#[cfg(test)]
mod gibbet_tests {
    use super::*;

    #[test]
    fn test_the_third_defiance_is_a_gibbet() {
        let mut g = Game::new();
        g.lord_esteem = 0.8;
        let m = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        for p in [desires::Problem::Broke, desires::Problem::Blamed,
                  desires::Problem::WritBorne, desires::Problem::LongWinter,
                  desires::Problem::ShortChanged] {
            desires::add_grievance(&mut g.world, m, p, 100);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
        g.repression_debt = 2; // the third writ
        g.edict_tick(320); // summons day
        assert!(villager::is_dead(&g.world, m), "the bailiff comes back with knights");
        assert!(g.martyr_grave.is_some(), "a martyr's grave");
        assert!(g.log.iter().any(|l| l.contains("hanged for the defiance")));
    }

    #[test]
    fn test_the_vigil_keeps_the_grave() {
        let mut g = Game::new();
        g.martyr_grave = Some((15, 15));
        let before = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
            .map(|p| p.capacity).unwrap_or(0.0);
        g.martyr_tick(20);
        let after = g.world.get_component::<crate::sim::organized_power::OrganizedPower>(g.org)
            .map(|p| p.capacity).unwrap_or(0.0);
        assert!(after > before, "the remembering is a kind of organizing");
    }
}

#[cfg(test)]
mod scenario_tests {
    use super::*;

    #[test]
    fn test_the_fever_year_opens_in_three_beds() {
        let g = Game::new_scenario(Scenario::FeverYear);
        let fevered = g.world.query::<Disease>().iter()
            .filter(|&&d| g.world.get_component::<Disease>(d)
                .map(|dd| dd.name == "scarlatina").unwrap_or(false)).count();
        assert_eq!(fevered, 3, "the scarlatina's already in three beds");
        assert!(g.log.iter().any(|l| l.contains("the scarlatina year")));
    }

    #[test]
    fn test_the_circus_winter_opens_harsh() {
        let mut g = Game::new_scenario(Scenario::CircusWinter);
        assert!(g.circus.is_some(), "the tents are already pitched");
        g.winter_tick(g.tick);
        assert!(g.harsh_winter, "a hard winter");
    }

    #[test]
    fn test_the_widows_shop_stands_derelict() {
        let g = Game::new_scenario(Scenario::WidowsShop);
        assert!(g.log.iter().any(|l| l.contains("the widow's shop")));
        // and someone's weighing the keeping of it
        let any_derelict = g.building_entities.iter().any(|&b|
            g.world.get_component::<Building>(b).map(|bb| household::derelict(bb)).unwrap_or(false));
        assert!(any_derelict, "a shop stands derelict");
    }
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;

    #[test]
    fn test_the_tuning_panel_reads_the_innards() {
        let g = Game::new();
        let gauges = g.telemetry();
        assert!(gauges.iter().any(|(k, _)| *k == "org.power"), "the power matrix's there");
        assert!(gauges.iter().any(|(k, _)| *k == "trails.heat"), "the trails heat's there");
        assert!(gauges.iter().any(|(k, _)| *k == "env.tree_cover"), "the land's score's there");
        assert!(gauges.iter().any(|(k, _)| *k == "price.fish"), "the economy's there");
        let lines = g.telemetry_lines();
        assert!(lines.iter().any(|l| l.starts_with("villagers.alive")), "a line per gauge");
    }
}

#[cfg(test)]
mod perf_tests {
    use super::*;

    #[test]
    fn test_the_path_memo_holds_for_the_day() {
        let mut g = Game::new();
        let p1 = g.walk_memo((10, 10), (15, 15));
        assert!(p1.is_some(), "the walk's still walkable");
        assert!(g.path_cache.contains_key(&((10, 10), (15, 15))), "it's remembered");
        // a day older and it's stale
        g.tick = 1;
        g.walk_memo((10, 10), (15, 15));
        assert_eq!(g.path_tick, 1, "the cache turned over");
        assert_eq!(g.path_cache.len(), 1, "fresh, not accumulated");
    }
}

#[cfg(test)]
mod sfx_tests {
    use super::*;

    #[test]
    fn test_the_soundscape_raises_its_cues() {
        let mut g = Game::new();
        g.sfx_tick(45); // a feast day
        assert!(g.sfx.iter().any(|c| c.kind == "festival"), "the festival hums");
        // rain's a cue while it falls
        g.precip = Some(crate::sim::precipitation::PrecipKind::Rain);
        g.sfx.clear();
        g.sfx_tick(20);
        assert!(g.sfx.iter().any(|c| c.kind == "rain"), "the bed keeps the weather");
    }

    #[test]
    fn test_cues_reach_the_scene() {
        let mut g = Game::new();
        g.emit_sfx("wonder", 20.0, 5.0, 0.7);
        let m = g.ui_model();
        assert!(m.sfx.iter().any(|k| k == "wonder"), "the cue's on the scene for the audio layer");
        // and a frame later it's gone - cues live exactly one read
        g.update();
        assert!(g.sfx.is_empty(), "the queue resets");
    }
}

#[cfg(test)]
mod dungeon_tests {
    use super::*;

    #[test]
    fn test_taken_is_held_not_gone() {
        let mut g = Game::new();
        let m = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        g.dungeon.push((m, 10));
        g.arrive(m, Goal::Exile); // they reach the castle's edge
        assert!(!villager::is_dead(&g.world, m), "held, not gone");
        assert!(g.log.iter().any(|l| l.contains("held at the castle")));
    }

    #[test]
    fn test_the_org_pays_the_ransom() {
        let mut g = Game::new();
        let m = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
        g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().resources = 50.0;
        g.dungeon.push((m, 10));
        g.dungeon_tick(50); // 40 days held, a %15==5 day - inside the window
        assert!(g.dungeon.is_empty(), "the ransom came");
        assert!(g.log.iter().any(|l| l.contains("pays the ransom")));
        let res = g.world.get_component::<PlayerCharacter>(g.player).unwrap().resources;
        assert_eq!(res, 30.0, "the purse answers for its own");
    }

    #[test]
    fn test_the_ransomed_dont_walk_off_the_world() {
        let mut g = Game::new();
        let m = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        g.world.get_component_mut::<Organization>(g.org).unwrap().members.push(m);
        g.world.get_component_mut::<PlayerCharacter>(g.player).unwrap().resources = 50.0;
        g.dungeon.push((m, 10));
        assert!(g.send(m, Goal::Exile), "the writ marches them out");
        g.world.add_component(m, crate::game::dungeon::CastleHeld { since: 10 });
        // the ransom comes mid-march: the walk out is still live when the
        // cell's paid - the exile leg can't arrive a driving-out
        g.dungeon_tick(50);
        assert!(!villager::is_dead(&g.world, m), "the ransomed live");
        assert!(g.world.get_component::<crate::game::dungeon::CastleHeld>(m).is_none(),
            "the castle lets go");
        assert!(g.travels.iter().all(|t| t.who != m || !matches!(t.goal, Goal::Exile)),
            "the walk out is spent");
        // and the exile leg can't finish what the pardon stopped
        g.advance_travels();
        assert!(!villager::is_dead(&g.world, m), "the pardoned don't die at the edge");
    }

    #[test]
    fn test_no_ransom_means_the_ladder_climbs() {
        let mut g = Game::new();
        let m = g.villagers.iter().copied().find(|&v|
            g.world.get_component::<TownRole>(v).is_none()).unwrap();
        g.dungeon.push((m, 10));
        g.dungeon_tick(80); // 70 days - a %15==5 day, unanswered
        assert!(villager::is_dead(&g.world, m), "the ladder climbs to its top");
        assert!(g.log.iter().any(|l| l.contains("hanged at the castle")));
    }
}

#[cfg(test)]
mod season_tests {
    use super::*;

    fn brightness(c: (u8, u8, u8)) -> u32 { c.0 as u32 + c.1 as u32 + c.2 as u32 }

    #[test]
    fn test_winter_land_is_snowy_and_water_stays_water() {
        for h in [0.25, 0.45, 0.65] {
            let summer = season_colour(h, Season::Summer);
            let winter = season_colour(h, Season::Winter);
            assert!(brightness(winter) > brightness(summer) + 150, "winter land at {} should be snow-pale: {:?} vs {:?}", h, winter, summer);
        }
        let water = season_colour(0.05, Season::Winter);
        assert!(water.2 > water.0 && water.2 > water.1, "winter water should still read as water: {:?}", water);
    }

    #[test]
    fn test_autumn_grass_turns_gold_and_summer_is_the_base_palette() {
        let (r, g, _) = season_colour(0.25, Season::Autumn);
        assert!(r > 100 && r as i32 > g as i32 - 20, "autumn grass should turn golden: {:?}", (r, g));
        assert_eq!(season_colour(0.25, Season::Summer), terrain_colour(0.25));
    }
}

#[cfg(test)]
mod inspector_tests {
    use super::*;

    #[test]
    fn test_fullness_reads_the_right_way_round() {
        // Hunger stores fullness: a low number is a hungry villager
        assert_eq!(fullness_word(95.0), "full");
        assert_eq!(fullness_word(24.0), "hungry");
        assert_eq!(fullness_word(5.0), "starving");
        assert_eq!(meter_colour(0.24), (230, 80, 70), "a hungry villager's bar should be red");
        assert_eq!(meter_colour(0.95), (90, 200, 90));
    }

    #[test]
    fn test_activity_in_plain_words() {
        assert_eq!(activity_text(Activity::SeekingFood), "looking for food");
        assert_eq!(activity_text(Activity::Idle), "idle");
    }
}

#[cfg(test)]
mod rain_tests {
    use super::*;

    #[test]
    fn test_rain_dims_and_cools_evenly() {
        let grass = (60, 140, 60);
        let wet = rain_wash(grass);
        assert!(wet.1 < grass.1, "rain should dim the land: {:?}", wet);
        // the same pixel gets the same wash on every row - no stripes
        assert_eq!(rain_wash(grass), rain_wash(grass));
        let (r, _, b) = rain_wash((200, 200, 200));
        assert!(b > r, "rain light should be cool, not neutral: r={} b={}", r, b);
    }

    #[test]
    fn test_rainy_frame_has_no_row_banding() {
        // render a flat map region, apply the rain wash the way render() does,
        // and check neighbouring rows match
        let mut buf = Buffer::new(W, H);
        buf.rect(0, 0, MAP_W as i32, H as i32, 140, 110, 50);
        for y in 0..H { for x in 0..MAP_W {
            let idx = (y * W + x) * 4;
            let (r, g, b) = rain_wash((buf.pixels[idx], buf.pixels[idx + 1], buf.pixels[idx + 2]));
            buf.set(x as i32, y as i32, r, g, b);
        }}
        let row = |y: usize| { let i = (y * W + 100) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
        for y in 0..12 { assert_eq!(row(y), row(y + 1), "rows {} and {} differ - banding", y, y + 1); }
    }
}

#[cfg(test)]
mod key_tests {
    use super::*;

    #[test]
    fn test_every_button_hint_is_its_bound_key_and_none_move_the_player() {
        assert_eq!(action_buttons().len(), ACTION_KEYS.len(), "every action button needs a key");
        for (b, ch) in action_buttons().iter().zip(ACTION_KEYS.iter()) {
            assert_eq!(b.key, *ch, "{} shows key {}", b.label, b.key);
            assert!(!MOVE_KEYS.contains(ch), "{} uses {}, which moves the player", b.label, ch);
        }
    }
}

#[cfg(test)]
mod progression_tests {
    use super::*;

    fn influence(game: &Game) -> f32 {
        game.world.get_component::<PlayerCharacter>(game.player).unwrap().influence
    }

    /// Walk to each villager, hear them out, then win them over at house
    /// meetings instead of with Recruit; finally invest the gold.
    #[test]
    fn test_the_village_can_still_be_won_when_people_join_at_meetings() {
        let mut game = Game::new();
        for i in 0..game.villagers.len() {
            let v = game.villagers[i];
            // folk die over the years this takes - a corpse can't come to a
            // meeting, whatever the player says beside it
            if villager::is_dead(&game.world, v) { continue; }
            let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
            { let p = game.world.get_component_mut::<Pos>(game.player).unwrap(); p.x = vx + 0.5; p.y = vy; }
            game.selected_target = i;
            for _ in 0..8 { game.act(0); } // talk until there's nothing new to hear
            for _ in 0..40 {
                if game.world.get_component::<Organization>(game.org).unwrap().members.contains(&v)
                    || villager::is_dead(&game.world, v) { break; }
                // trust decays a little every day - a meeting a fortnight
                // apart won't climb past it unless the talking keeps up
                game.act(0);
                game.act(3);
                finish_meeting(&mut game);
            }
            // he may have died mid-courtship - the years this takes carry
            // folk off, and a corpse can't come to a meeting
            if villager::is_dead(&game.world, v) { continue; }
            if !game.world.get_component::<Organization>(game.org).unwrap().members.contains(&v) {
                let rel = relationships::find_relationship(&game.world, game.player, v)
                    .and_then(|r| game.world.get_component::<Relationship>(r)).map(|r| r.strength);
                let last: Vec<&String> = game.log.iter().rev().take(4).collect();
                panic!("{} never joined at a meeting (dead {}, trust {:?}, last log {:?})",
                    vill_name(i), villager::is_dead(&game.world, v), rel, last);
            }
        }
        while game.disabled_reason(2).is_none()
            && game.world.get_component::<PlayerCharacter>(game.player).unwrap().resources >= 10.0 {
            game.act(2);
        }
        assert!(influence(&game) > WIN_INFLUENCE,
            "everyone joined and all gold invested, yet influence is only {:.2} (win at {})", influence(&game), WIN_INFLUENCE);
    }
}

#[cfg(test)]
mod works_tests {
    use super::*;

    #[test]
    fn test_a_new_house_lays_its_road_a_tile_at_a_time_then_rises() {
        let mut game = Game::new();
        // somewhere a road has to be laid to reach - the town's lanes reach
        // everywhere now, so hunt for a spot the road hasn't got to yet
        let spot = [(8i32, 20), (4, 24), (28, 26), (30, 5), (4, 5), (26, 3)]
            .iter().copied().find(|&(cx, cy)| {
                (-4..=4).all(|dy| (-4..=4).all(|dx| game.map.get(cx + dx, cy + dy) != Some(Tile::Road)))
            }).expect("a spot out past the roads");
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: spot.0 as f32, y: spot.1 as f32 };
        // the Wanderer's starting gold is rolled by class and can come up
        // short - this test is about the works, not the purse
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 100.0;
        assert_eq!(game.action_blocker(4), None);
        game.act(4);
        let w = game.works.last().expect("building should start works");
        let road = w.road.clone();
        assert!(road.len() >= 1, "a house out here needs a road");
        assert!(road.iter().all(|&(x, y)| game.map.get(x, y) != Some(Tile::Road)), "the road isn't there at once");
        let mut laid_counts = vec![];
        for _ in 0..(road.len() as u32 * ROAD_TILE_FRAMES) {
            game.update();
            laid_counts.push(road.iter().filter(|&&(x, y)| game.map.get(x, y) == Some(Tile::Road)).count());
        }
        assert!(laid_counts.windows(2).all(|p| p[1] >= p[0] && p[1] - p[0] <= 1), "one tile at a time");
        assert_eq!(*laid_counts.last().unwrap(), road.len(), "the whole road is laid");
        assert!(!game.works.is_empty(), "the house is still going up");
        for _ in 0..RISE_FRAMES { game.update(); }
        assert!(game.works.is_empty(), "the house is finished");
    }
}

#[cfg(test)]
mod chat_tests {
    use super::*;

    fn trust(game: &Game, a: Entity, b: Entity) -> f32 {
        relationships::find_relationship(&game.world, a, b)
            .and_then(|r| game.world.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0)
    }

    #[test]
    fn test_talk_puts_their_reply_in_a_speech_bubble() {
        let mut game = Game::new();
        let v = game.villagers[1];
        let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
        game.selected_target = 1;
        game.act(0);
        let (who, said, until, _, _) = game.bubbles.iter().find(|b| b.0 == v).expect("no speech bubble over Eliza").clone();
        assert_eq!(who, v);
        assert!(game.log.last().unwrap().contains(&said), "the bubble says what the log reports");
        assert!(until > game.frame_n + FPS, "the bubble stays up for seconds, not a frame");
    }

    #[test]
    fn test_passing_someone_gets_a_remark_and_a_little_warmth() {
        let mut game = Game::new();
        let v = game.villagers[2];
        let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
        let before = trust(&game, game.player, v);
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 1.0, y: vy };
        // only some meetings get a bubble - pad the ledger so Bram's pair
        // lands on the remark roll whatever came before it
        while (game.last_chat.len() + v as usize + game.tick as usize) % 3 != 0 {
            game.last_chat.push(((v, v), 0));
        }
        game.chat_in_passing();
        assert!(game.bubbles.iter().any(|b| b.0 == v), "Bram should say something as you pass");
        assert!((trust(&game, game.player, v) - before - 0.01).abs() < 1e-4, "a chat in passing is worth +0.01");
        // standing about together doesn't keep generating chatter
        game.bubbles.clear();
        for _ in 0..10 { game.chat_in_passing(); }
        assert!(!game.bubbles.iter().any(|b| b.0 == v), "they already said hello");
    }
}

#[cfg(test)]
mod river_tests {
    use super::*;

    #[test]
    fn test_the_river_can_only_be_crossed_at_the_bridge() {
        let game = Game::new();
        let mut river = 0;
        for y in 0..game.map.height as i32 { for x in 0..game.map.width as i32 {
            match game.map.get(x, y) {
                Some(Tile::River) => { river += 1; assert!(!game.walkable(x as f32, y as f32), "walked onto the river at ({},{})", x, y); }
                Some(Tile::Bridge) => assert!(game.walkable(x as f32, y as f32), "the bridge is blocked"),
                _ => {}
            }
        }}
        assert!(river > 10);
    }
}

#[cfg(test)]
mod ground_cache_tests {
    use super::*;

    #[test]
    fn test_cached_ground_matches_drawing_it_straight() {
        // the world-space layer blitted each frame must be exactly what the
        // tile pass would have drawn - any tile dropped or shifted shows up
        // as bare background or a seam
        let game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        // exact comparison at integer zooms, where the layer's draw shift is
        // a whole number of pixels so it lands exactly where the culled pass
        // put it (at fractional zooms the layer can sit a sub-pixel off and
        // still be right)
        for &zoom in &[1.0f32, 2.0] {
            let mut direct = Buffer::new(W, H);
            direct.rect(0, 0, W as i32, H as i32, 25, 25, 38);
            view.draw_ground(&game.ui_model(), &mut direct, 0, 0, zoom);
            let g = view.render_ground(&game.ui_model(), zoom);
            let mut cached = Buffer::new(W, H);
            cached.rect(0, 0, W as i32, H as i32, 25, 25, 38);
            cached.blit_from(&g.buf, g.origin.0, g.origin.1, 0, 0, MAP_W as i32, H as i32);
            if std::env::var("DUMP_GROUND").is_ok() {
                direct.save_bmp(&format!("../evidence/ground-direct-{}.bmp", (zoom * 100.0) as i32)).ok();
                cached.save_bmp(&format!("../evidence/ground-cache-{}.bmp", (zoom * 100.0) as i32)).ok();
                g.buf.save_bmp(&format!("../evidence/ground-layer-{}.bmp", (zoom * 100.0) as i32)).ok();
            }
            // the layer can legitimately show MORE than the old culled pass
            // (edge blocks that used to pop in), but nothing it drew may be
            // missing or misplaced inside the map viewport
            let bg = [25u8, 25, 38, 255];
            let (mut missing, mut bounds) = (0, [i32::MAX, i32::MAX, i32::MIN, i32::MIN]);
            for y in 0..H { for x in 0..MAP_W {
                let i = (y * W + x) * 4;
                if direct.pixels[i..i + 4] != bg && direct.pixels[i..i + 4] != cached.pixels[i..i + 4] {
                    missing += 1;
                    bounds = [bounds[0].min(x as i32), bounds[1].min(y as i32), bounds[2].max(x as i32), bounds[3].max(y as i32)];
                }
            }}
            assert_eq!(missing, 0, "zoom {}: ground pixels dropped by the cache in {:?}", zoom, bounds);
        }
    }
}

#[cfg(test)]
mod depth_tests {
    use super::*;

    #[test]
    fn test_hiding_someone_doesnt_flip_the_wall_piece_in_front() {
        // someone standing just inside the town wall is hidden by it - but the
        // piece hiding them must not paint over the piece in FRONT of it (the
        // occluder re-draw used to flip the seam back to front, leaving a
        // glitched spot on the wall)
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        // a spot where the last wall piece and the front corner tower
        // overlap: the tower stands in front, so it owns the seam (the map
        // is perturbed per seed - find the corner, don't hardcode it)
        let ((cx, cy), _) = game.map.walls.iter()
            .filter(|(_, p)| *p == WallPiece::Tower)
            .max_by_key(|((x, y), _)| x + y).copied().expect("a corner tower");
        let (sx, sy) = iso_to_screen_cam(cx as f32, cy as f32, view.cam_x, view.cam_y, view.zoom);
        let seam = |buf: &Buffer| { let i = ((sy - 30) as usize * W + (sx - 17) as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
        let render_at = |game: &mut Game, x: f32, y: f32| {
            *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
            view.render(&game.ui_model(), (-1, -1))
        };
        let clear = render_at(&mut game, 16.0, 16.0);
        let hidden = render_at(&mut game, 21.0, 23.0);
        assert_eq!(seam(&clear), seam(&hidden), "hiding the player flipped the tower in front of the wall piece");
    }

    #[test]
    fn test_walking_behind_a_house_hides_the_player() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        let (bx, by, _) = game.buildings[0];
        // the player's yellow ring is drawn under them: behind the house it's covered
        let ring_px = |game: &Game| {
            let buf = view.render(&game.ui_model(), (-1, -1));
            let (px, py) = game.player_pos();
            let (sx, sy) = iso_to_screen_cam(px, py, view.cam_x, view.cam_y, view.zoom);
            (sx - 12..sx + 12).flat_map(|x| (sy - 4..sy - 1).map(move |y| (x, y)))
                .filter(|&(x, y)| x >= 0 && y >= 0 && (x as usize) < W && (y as usize) < H)
                .filter(|&(x, y)| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == (255, 215, 0) })
                .count()
        };
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: bx + 1.0, y: by + 1.0 };
        let in_front = ring_px(&game);
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: bx - 0.6, y: by - 0.6 };
        let behind = ring_px(&game);
        assert!(in_front > 20, "the ring shows in front of the house ({} px)", in_front);
        assert!(behind < in_front / 2, "behind the house the ring should be hidden ({} vs {} px)", behind, in_front);
    }
}

#[cfg(test)]
mod walk_tests {
    use super::*;

    fn put_player(game: &mut Game, x: f32, y: f32) {
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
    }

    #[test]
    fn test_the_player_cant_walk_into_a_building_but_can_go_round_behind() {
        let mut game = Game::new();
        // a building with open ground to the west and behind it - the town
        // is built up enough that the first cottage won't do
        let (bx, by, _) = *game.buildings.iter().find(|&&(bx, by, _)| {
            game.walkable(bx - 1.0, by) && game.walkable(bx - 2.0, by)
                && [(bx - 1.0, by), (bx, by - 1.0)].iter().any(|&(x, y)| game.walkable(x, y))
        }).expect("a building with open ground behind it");
        // walking straight at it (along +x) stops at the building's tile
        put_player(&mut game, bx - 2.0, by);
        for _ in 0..60 { game.walk(1.0, 0.0); }
        let (px, _) = game.player_pos();
        assert!(px.round() < bx, "walked into the building at {} (player at {})", bx, px);
        // but the tiles behind it (farther from the viewer) are open ground
        assert!([(bx - 1.0, by), (bx, by - 1.0)].iter().any(|&(x, y)| game.walkable(x, y)), "can't get behind the building");
    }

    #[test]
    fn test_the_player_cant_walk_into_the_sea() {
        let mut game = Game::new();
        put_player(&mut game, 16.0, 16.0);
        for _ in 0..600 { game.walk(1.0, 1.0); }
        let (px, py) = game.player_pos();
        assert!(!matches!(game.map.get(px.round() as i32, py.round() as i32), Some(Tile::Water) | Some(Tile::River) | None),
            "the player walked into the water at ({:.1},{:.1})", px, py);
    }

    #[test]
    fn test_a_still_walk_stick_leaves_the_player_where_they_stood() {
        // a zero Walk command (a held key let go, or noise on the wire)
        // used to divide by zero and NaN the player's position away
        let mut game = Game::new();
        let (x, y) = game.player_pos();
        game.walk(0.0, 0.0);
        let (nx, ny) = game.player_pos();
        assert!(nx.is_finite() && ny.is_finite(), "the player fell off the world ({},{})", nx, ny);
        assert_eq!((nx, ny), (x, y));
    }

    #[test]
    fn test_roads_are_quicker() {
        let mut game = Game::new();
        let road = (0..game.map.height as i32).flat_map(|y| (0..game.map.width as i32).map(move |x| (x, y)))
            .find(|&(x, y)| game.map.get(x, y) == Some(Tile::Road) && game.map.get(x + 1, y) == Some(Tile::Road)).unwrap();
        let grass = (0..game.map.height as i32).flat_map(|y| (0..game.map.width as i32).map(move |x| (x, y)))
            .find(|&(x, y)| game.map.get(x, y) == Some(Tile::Grass) && !game.map.trees.contains(&(x, y)) && game.walkable(x as f32 + 0.2, y as f32)).unwrap();
        let step = |game: &mut Game, (x, y): (i32, i32)| {
            put_player(game, x as f32 - 0.2, y as f32);
            game.walk(1.0, 0.0);
            game.player_pos().0 - (x as f32 - 0.2)
        };
        let (on_road, off_road) = (step(&mut game, road), step(&mut game, grass));
        assert!((on_road - WALK_SPEED * ROAD_SPEEDUP).abs() < 1e-4 && (off_road - WALK_SPEED).abs() < 1e-4, "road {} vs grass {}", on_road, off_road);
    }
}

#[cfg(test)]
mod talk_reach_tests {
    use super::*;

    #[test]
    fn test_talk_goes_to_whoever_is_beside_you_not_the_last_clicked() {
        // on main, Talk with Aldric selected (the default) said "too far
        // away" while the player stood right next to Bram
        let mut game = Game::new();
        let (px, py) = game.player_pos();
        for (i, v) in game.villagers.clone().into_iter().enumerate() {
            let far = if i == 2 { (px + 1.0, py) } else { (px + 20.0 + i as f32, py + 20.0) };
            *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: far.0, y: far.1 };
        }
        game.selected_target = 0;
        assert_eq!(game.disabled_reason(0), None, "someone is beside you, so Talk should be usable");
        game.act(0);
        assert_eq!(game.selected_target, 2, "Talk should turn to the villager in reach");
        let line = game.log.last().unwrap();
        assert!(line.contains(vill_name(2)) && !line.contains("Can't"), "{}", line);
    }

    /// A villager the player can walk to who's well out of talking distance
    fn far_villager(game: &Game) -> usize {
        (0..game.villagers.len())
            .filter(|&i| game.dist_to(i) > 4.0 && game.path_to(i).is_some())
            .max_by(|&a, &b| game.dist_to(a).partial_cmp(&game.dist_to(b)).unwrap())
            .expect("someone to walk over to")
    }

    #[test]
    fn test_talk_on_a_picked_villager_walks_over_then_talks() {
        let mut game = Game::new();
        while game.tick < 3 { game.update(); }
        let who = far_villager(&game);
        game.selected_target = who;
        game.picked = true; // clicked on them
        let v = game.villagers[who];
        let talks = |g: &Game| g.world.get_component::<TalkCount>(v).map(|c| c.value).unwrap_or(0);
        let before = talks(&game);
        let start = game.dist_to(who);

        game.act(0);
        assert!(game.log.last().unwrap().contains(&format!("You walk over to {}", vill_name(who))), "{}", game.log.last().unwrap());
        assert_eq!(talks(&game), before, "no talking from across the village");

        for _ in 0..(30 * FPS) {
            if game.approach.is_none() { break; }
            game.update();
        }
        assert!(game.approach.is_none(), "still walking to {} ({} tiles off, started {})", vill_name(who), game.dist_to(who), start);
        assert!(game.dist_to(who) <= TALK_DIST + 0.5, "stopped {} tiles from {}", game.dist_to(who), vill_name(who));
        assert_eq!(talks(&game), before + 1, "talked once on arrival");
        assert!(game.log.iter().any(|l| l.contains(vill_name(who)) && l.contains(": \"")), "no talk line: {:?}", game.log);
    }

    #[test]
    fn test_recruit_on_a_picked_villager_walks_over_then_recruits() {
        let mut game = Game::new();
        while game.tick < 3 { game.update(); }
        let who = far_villager(&game);
        game.selected_target = who;
        game.picked = true;
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().influence = 10.0;
        assert_eq!(game.disabled_reason(1), None, "Recruit walks over, so distance doesn't grey it out");

        game.act(1);
        assert!(game.log.last().unwrap().contains(&format!("You walk over to {}", vill_name(who))), "{}", game.log.last().unwrap());
        assert!(game.approach.as_ref().map(|a| a.action) == Some(1));

        for _ in 0..(30 * FPS) {
            if game.approach.is_none() { break; }
            game.update();
        }
        assert!(game.approach.is_none(), "still walking to {}", vill_name(who));
        assert!(game.dist_to(who) <= TALK_DIST + 0.5, "stopped {} tiles from {}", game.dist_to(who), vill_name(who));
        // the recruit attempt happened on arrival (joined or turned the player down)
        let line = game.log.last().unwrap();
        assert!(line.contains(vill_name(who)) && !line.contains("walk over") && !line.contains("Can't"), "{}", line);
    }

    #[test]
    fn test_talk_is_never_too_far_away() {
        let mut game = Game::new();
        let (px, py) = game.player_pos();
        let from = (px.round() as i32, py.round() as i32);
        // a tile the player can walk to, well out of reach
        let spot = (0..game.map.height as i32).flat_map(|y| (0..game.map.width as i32).map(move |x| (x, y)))
            .find(|&(x, y)| ((x as f32 - px).powi(2) + (y as f32 - py).powi(2)).sqrt() > MEETING_REACH + 2.0
                && game.map.walk_path(from, (x, y)).is_some())
            .expect("somewhere far but reachable");
        let v = game.villagers[2];
        *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: spot.0 as f32, y: spot.1 as f32 };
        game.selected_target = 2;
        game.picked = true;
        assert_eq!(game.disabled_reason(0), None, "Talk walks over, so distance doesn't grey it out");
        game.act(0);
        assert!(game.approach.is_some() && game.log.last().unwrap().contains("You walk over to"), "{}", game.log.last().unwrap());
    }
}

#[cfg(test)]
mod action_log_tests {
    use super::*;

    #[test]
    fn test_action_lines_are_dated_and_say_who_not_the_score() {
        let mut game = Game::new();
        while game.tick < 3 { game.update(); }
        // stand next to Eliza and select her
        let (ex, ey) = { let p = game.world.get_component::<Pos>(game.villagers[1]).unwrap(); (p.x, p.y) };
        { let p = game.world.get_component_mut::<Pos>(game.player).unwrap(); p.x = ex + 0.5; p.y = ey; }
        game.selected_target = 1;

        game.act(0);
        let talk = game.log.last().unwrap().clone();
        assert!(talk.starts_with("t3: Eliza ") && talk.contains(": \"") && !talk.contains("influence"), "{}", talk);

        game.act(1); // not enough influence yet
        let recruit = game.log.last().unwrap().clone();
        assert!(recruit.starts_with("t3: Can't recruit") && !recruit.contains("influence)"), "{}", recruit);

        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 20.0;
        game.act(2);
        let invest = game.log.last().unwrap().clone();
        assert!(invest.starts_with("t3: Invested 10 gold") && !invest.contains("influence"), "{}", invest);

    }

    #[test]
    fn test_influence_lines_are_not_coloured_as_illness() {
        // "influence" contains "flu"; it used to turn every influence line weather-cyan
        let illness = (120, 200, 255);
        assert_ne!(log_colour("t3: Invested 10 gold"), illness);
        assert_ne!(log_colour("t3: Can't recruit: need influence 0.40."), illness);
        assert_ne!(log_colour("t9: the grain store is low"), illness);
        assert_eq!(log_colour("t120: Bram is taken with la grippe"), illness);
        assert_eq!(log_colour("t200: it's raining"), illness);
    }
}

#[cfg(test)]
mod log_priority_tests {
    use super::*;

    #[test]
    fn test_old_gossip_does_not_push_events_out_of_the_log() {
        let mut log: Vec<String> = vec!["t1: The Merchant Guild formed!".into(), "t120: Bram is taken with la grippe".into()];
        for t in (160..=600).step_by(40) { log.push(format!("t{}: Aldric tells Bram - word spreads", t)); }
        let shown = visible_log(&log);
        assert!(shown.iter().any(|e| e.contains("la grippe")), "the grippe should still be in view: {:?}", shown);
        assert!(shown.iter().any(|e| e.contains("Guild formed")), "the guild should still be in view");
        assert_eq!(shown.iter().filter(|e| is_gossip(e)).count(), LOG_GOSSIP_SHOWN);
        // the important pin rides above the newest gossip - the flu's
        // still first; the word-of-mouth line's just under it
        assert!(shown[0].contains("la grippe"), "important pins top");
        assert_eq!(shown[1], &"t600: Aldric tells Bram - word spreads".to_string(), "newest of the rest next");
    }

    #[test]
    fn test_weather_changes_do_not_push_events_out_of_the_log() {
        let mut log: Vec<String> = vec!["t120: Bram is taken with la grippe".into()];
        for t in (130..=600).step_by(5) {
            log.push(format!("t{}: drizzle begins", t));
            log.push(format!("t{}: the rain stops - a rainbow!", t + 2));
        }
        let shown = visible_log(&log);
        assert!(shown.iter().any(|e| e.contains("la grippe")), "the grippe should still be in view: {:?}", shown);
        assert_eq!(shown.iter().filter(|e| is_weather_note(e)).count(), LOG_WEATHER_SHOWN);
    }

    #[test]
    fn test_flu_from_a_real_game_stays_in_the_log_by_day_600() {
        let mut game = Game::new();
        while game.tick < 600 { game.update(); }
        let shown: Vec<&String> = visible_log(&game.log).into_iter().take(8).collect();
        assert!(shown.iter().any(|e| e.contains("is taken with") || e.contains("recovered from")),
            "by day 600 the sickness has scrolled away: {:?}", shown);
    }

    #[test]
    fn test_the_log_is_bounded_over_a_long_game() {
        let mut game = Game::new();
        // a running tap of lines, far past the cap
        for t in 1..=(LOG_MAX + 5000) { game.log.push(format!("t{}: a line", t)); }
        game.update();
        assert!(game.log.len() <= LOG_MAX,
            "the log keeps only its recent window: {}", game.log.len());
        assert_eq!(game.log.last().unwrap(), &format!("t{}: a line", LOG_MAX + 5000),
            "and what it keeps is the newest end");
        assert_eq!(game.log_dropped + game.log.len(), LOG_MAX + 5000 + 2,
            "dropped + kept accounts for every line written");
    }
}

#[cfg(test)]
mod bubble_tests {
    use super::*;

    #[test]
    fn test_bubbles_stay_inside_the_map_view() {
        let text = "the lord's taxes are heavy";
        // near the right edge: pulled left so it doesn't spill into the panel
        let (bx, _, bw) = bubble_rect(MAP_W as i32 - 10, 300, 1.0, text, None).unwrap();
        assert!(bx + bw <= MAP_W as i32 - 4, "bubble runs into the panel: {}..{}", bx, bx + bw);
        // near the left and top edges: pushed back on screen
        let (bx, by, _) = bubble_rect(5, 20, 1.0, text, None).unwrap();
        assert!(bx >= 4 && by >= 4);
        // a speaker under the panel or off screen gets no bubble at all
        assert_eq!(bubble_rect(MAP_W as i32 + 40, 300, 1.0, text, None), None);
        assert_eq!(bubble_rect(-30, 300, 1.0, text, None), None);
    }

    #[test]
    fn test_bubble_clears_the_name_label_at_any_zoom() {
        for z in [0.5f32, 1.0, 2.0] {
            let (_, by, _) = bubble_rect(300, 400, z, "hello", None).unwrap();
            let (_, label_y) = name_label_pos(300, label_anchor(400, z), "hello");
            assert!(by + BUBBLE_H <= label_y, "zoom {}: bubble (bottom {}) overlaps the name label (top {})", z, by + BUBBLE_H, label_y);
        }
    }

    #[test]
    fn test_bubble_clears_a_label_nudged_up_by_the_crowd() {
        // place_labels can lift a crowded name up to 4 lines (40px); the
        // bubble used to sit 8px above the un-nudged spot, right on top of it
        let (_, home) = name_label_pos(300, label_anchor(400, 1.0), "Bram");
        for lift in [10, 20, 40] {
            let nudged = home - lift;
            let (_, by, _) = bubble_rect(300, 400, 1.0, "the lord's taxes are heavy", Some(nudged)).unwrap();
            // the bubble and its 4px tail end above the label
            assert!(by + BUBBLE_H + 4 <= nudged, "lift {}: bubble bottom {} covers the label at {}", lift, by + BUBBLE_H + 4, nudged);
        }
    }
}

#[cfg(test)]
mod zoom_tests {
    use super::*;

    #[test]
    fn test_zoom_keeps_the_view_centre_still() {
        // whatever sits at the middle of the map view stays there as you zoom
        let (cx, cy) = (ZOOM_PIVOT.0 as i32, ZOOM_PIVOT.1 as i32);
        let (tx, ty) = (11.4f32, 11.4f32); // a tile near the view centre at zoom 1, camera at rest
        let at1 = iso_to_screen_cam(tx, ty, 0, 0, 1.0);
        for z in [0.5, 1.5, 2.0] {
            let at = iso_to_screen_cam(tx, ty, 0, 0, z);
            let (d1, dz) = ((at1.0 - cx).abs() + (at1.1 - cy).abs(), (at.0 - cx).abs() + (at.1 - cy).abs());
            assert!(dz as f32 <= d1 as f32 * z.max(1.0) + 2.0, "zoom {} moved the centre tile from {:?} to {:?}", z, at1, at);
        }
    }

    #[test]
    fn test_sprites_scale_with_zoom() {
        let sprite = Sprite { width: 20, height: 10, pixels: vec![255; 20 * 10 * 4] };
        let count = |scale: f32| {
            let mut buf = Buffer::new(100, 100);
            draw_zoomed(&mut buf, &sprite, 10, 10, scale);
            buf.pixels.chunks(4).filter(|p| p[0] == 255).count()
        };
        assert_eq!(count(1.0), 200);
        assert_eq!(count(0.5), 50, "half zoom should draw a quarter of the pixels");
        assert_eq!(count(2.0), 800);
        // and labels follow a zoomed head
        assert_eq!(head_y(300, 2.0), 300 + 2 * SPRITE_HEAD);
        assert_eq!(name_label_pos(0, label_anchor(300, 0.5), "X").1 + GLYPH_H + LABEL_GAP, head_y(300, 0.5));
    }
}

#[cfg(test)]
mod legend_tests {
    use super::*;

    #[test]
    fn test_the_panel_explains_the_map_marks() {
        // the legend fits the panel and sits below the log box
        let chars: usize = MAP_MARKS.iter().map(|(m, _, w)| m.len() + 1 + w.len() + 2).sum();
        assert!(UI_X as i32 + 14 + chars as i32 * GLYPH_W <= W as i32 - 4, "legend runs off the panel");
        assert!(LEGEND_Y >= 450 + 170 && LEGEND_Y + GLYPH_H <= H as i32);

        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        while game.tick < 30 { game.update(); }
        let buf = view.render(&game.ui_model(), (-1, -1));
        // each mark's colour appears in the legend row
        for (m, rgb, word) in MAP_MARKS {
            let found = (LEGEND_Y..LEGEND_Y + GLYPH_H).any(|y| (UI_X..W).any(|x| {
                let i = (y as usize * W + x) * 4;
                (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == rgb
            }));
            assert!(found, "the legend doesn't show '{}' ({})", m, word);
        }
    }
}

#[cfg(test)]
mod log_day_tests {
    use super::*;

    #[test]
    fn test_log_entries_show_the_day_like_the_hud() {
        assert_eq!(log_parts("t120: Bram is taken with la grippe"), (Some("day 120".into()), "Bram is taken with la grippe"));
        assert_eq!(log_parts("Welcome to the village."), (None, "Welcome to the village."));
        assert_eq!(log_parts("tea: not a day"), (None, "tea: not a day"));
    }

    #[test]
    fn test_the_log_draws_the_day_dim_and_the_message_in_colour() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        game.log.push("t5: The Merchant Guild formed!".into());
        let buf = view.render(&game.ui_model(), (-1, -1));
        let row = |x0: i32, x1: i32, rgb: (u8, u8, u8)| (530..530 + GLYPH_H).any(|y| (x0..x1).any(|x| {
            let i = (y as usize * W + x as usize) * 4;
            (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == rgb
        }));
        let x = UI_X as i32 + 14;
        let msg_x = x + 6 * GLYPH_W; // after "day 5 "
        assert!(row(x, msg_x - GLYPH_W, (120, 120, 145)), "the day stamp should be dim");
        assert!(!row(x, msg_x - GLYPH_W, (255, 215, 0)), "the day stamp shouldn't take the message colour");
        assert!(row(msg_x, msg_x + 10 * GLYPH_W, (255, 215, 0)), "the guild message keeps its gold");
    }
}

#[cfg(test)]
mod food_reach_tests {
    use super::*;

    #[test]
    fn test_no_one_starves_beside_a_full_store() {
        // the crowd-separation push (3 tiles) used to hold hungry villagers
        // out of eating reach (1 tile) of the store: on main they sat at zero
        // hunger for up to 75 days in a row while it held ~480 servings
        let mut game = Game::new();
        let (mut last, mut run, mut worst, mut who) = (0, vec![0u64; NAMES.len()], 0u64, 0usize);
        while game.tick < 720 {
            game.update();
            if game.tick == last { continue; }
            last = game.tick;
            // the roster grows: births and newcomers take index 32, 33...
            run.resize(game.villagers.len(), 0);
            for (k, &v) in game.villagers.iter().enumerate() {
                // the dead keep their last hunger (zero, if they starved): this
                // test is about the living being able to reach the store
                if game.world.get_component::<Health>(v).is_none() { run[k] = 0; continue; }
                let empty = game.world.get_component::<Hunger>(v).map(|h| h.value <= 0.0).unwrap_or(false);
                run[k] = if empty { run[k] + 1 } else { 0 };
                if run[k] > worst { (worst, who) = (run[k], k); }
            }
        }
        let servings = game.world.get_component::<villager::FoodSource>(game.market_food).unwrap().servings;
        assert!(servings > 0, "the store ran out - this test is about reach, not supply");
        let v = game.villagers[who];
        let sp = *game.world.get_component::<Pos>(v).unwrap();
        let st = (sp.x.round() as i32, sp.y.round() as i32);
        let home = game.world.get_component::<Pos>(game.market_food).unwrap();
        let route = game.map.walk_path(st, (home.x.round() as i32, home.y.round() as i32)).map(|p| p.len());
        let act = game.world.get_component::<Villager>(v).map(|x| x.activity);
        let on_job = game.travels.iter().any(|t| t.who == v) || game.world.get_component::<OddJob>(v).is_some();
        assert!(worst <= 5, "{} went {} days at zero hunger at {:?} (passable {}, route {:?}, act {:?}, job {}) with {} servings in the store",
            vill_name(who), worst, st, game.map.passable(st.0, st.1), route, act, on_job, servings);
    }
}

#[cfg(test)]
mod help_tests {
    use super::*;

    #[test]
    fn test_the_panel_points_to_help() {
        assert!(PANEL_HINT.contains("H help"));
        assert!(UI_X as i32 + 18 + text_width(PANEL_HINT, 1) < W as i32 - 20, "hint runs off the panel card");
    }

    #[test]
    fn test_help_box_is_centred_on_the_map_and_fits_its_text() {
        let (x, y, w, h) = help_rect();
        assert!(((x + w / 2) - MAP_W as i32 / 2).abs() <= 1, "help box not centred on the map view");
        assert!(x >= 0 && x + w <= MAP_W as i32 && y >= 0 && y + h <= H as i32);
        let widest = HELP_LINES.iter().map(|l| text_width(l, 1)).max().unwrap();
        assert!(widest + 40 <= w, "help text too wide for its box");

        // in a real frame, the drawn text inside the box is balanced left/right
        let game = Game::new();
        let mut view = View::new(Art::load_fs("../assets", 0));
        view.help_open = true;
        let buf = view.render(&game.ui_model(), (-1, -1));
        let (mut lo, mut hi) = (i32::MAX, i32::MIN);
        for yy in y + 10..y + h - 5 { for xx in x + 4..x + w - 4 {
            let i = (yy as usize * W + xx as usize) * 4;
            if buf.pixels[i] > 140 { lo = lo.min(xx); hi = hi.max(xx); }
        }}
        let (left, right) = (lo - x, x + w - hi);
        assert!((left - right).abs() <= 12, "help text hugs one side: {}px left, {}px right", left, right);
    }
}

#[cfg(test)]
mod blocker_tests {
    use super::*;

    #[test]
    fn test_invest_is_greyed_out_with_no_gold() {
        let mut game = Game::new();
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 5.0;
        assert_eq!(game.action_blocker(2).as_deref(), Some("need 10 gold"));
        game.act(2);
        assert!(game.log.last().unwrap().ends_with("Can't invest: need 10 gold."), "{}", game.log.last().unwrap());
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 10.0;
        assert_eq!(game.action_blocker(2), None);
    }

    #[test]
    fn test_recruit_is_greyed_out_for_a_member() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        let v = game.villagers[2];
        game.world.get_component_mut::<Organization>(game.org).unwrap().members.push(v);
        // beside them, so Recruit doesn't turn to someone nearer (reach_target)
        let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 0.5, y: vy };
        game.selected_target = 2;
        assert_eq!(game.action_blocker(1).as_deref(), Some("already a member"));
        // the log line mustn't read as a success (log_colour greens "joined")
        game.act(1);
        assert_ne!(log_colour(game.log.last().unwrap()), (120, 255, 120), "{}", game.log.last().unwrap());
        // and the button is drawn disabled, with the reason beside it
        let buf = view.render(&game.ui_model(), (-1, -1));
        let b = &action_buttons()[1];
        let i = ((b.y + 4) as usize * W + (b.x + 4) as usize) * 4;
        assert_ne!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), b.color, "Recruit still drawn in its live colour");
    }
}

#[cfg(test)]
mod status_line_tests {
    use super::*;

    #[test]
    fn test_status_line_names_things_and_fits() {
        assert_eq!(status_line("The Harvesters", 5, 0.0, Some(3)), "Harvesters 5 | Gold 0 | Merchant Guild 3");
        assert_eq!(status_line("The Harvesters", 0, 100.0, None), "Harvesters 0 | Gold 100 | no guild yet");
        // the widest it gets still fits the panel
        let widest = status_line("The Harvesters", 5, 9999.0, Some(99));
        assert!(UI_X as i32 + 10 + text_width(&widest, 1) <= W as i32 - 6, "{} runs off the panel", widest);
        assert!(UI_X as i32 + 70 + 120 + 6 + text_width("1.99/2.0 lead", 1) <= W as i32 - 4);
    }
}

#[cfg(test)]
mod pause_box_tests {
    use super::*;

    #[test]
    fn test_pause_box_is_centred_with_balanced_text() {
        let (x, y, w, h) = pause_rect();
        assert!(((x + w / 2) - MAP_W as i32 / 2).abs() <= 1, "pause box not centred on the map view");
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        game.paused = true;
        let buf = view.render(&game.ui_model(), (-1, -1));
        let (mut lo, mut hi) = (i32::MAX, i32::MIN);
        for yy in y + 10..y + h - 4 { for xx in x + 4..x + w - 4 {
            let i = (yy as usize * W + xx as usize) * 4;
            if buf.pixels[i] > 140 { lo = lo.min(xx); hi = hi.max(xx); }
        }}
        let (left, right) = (lo - x, x + w - hi);
        // was ~20px left vs ~120px right, and the box centred at x 410 not 330
        assert!((left - right).abs() <= 12, "pause text hugs one side: {}px left, {}px right", left, right);
    }
}

#[cfg(test)]
mod dead_portrait_tests {
    use super::*;

    #[test]
    fn test_a_dead_villagers_card_shows_a_gravestone() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        while game.tick < 60 { game.update(); }
        let corvin = game.villagers[3];
        game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 3 { game.update(); }
        game.selected_target = 3;
        let buf = view.render(&game.ui_model(), (-1, -1));
        let (x0, y0) = (UI_X as i32 + 6, 312);
        let mut stone = 0;
        let fig: Vec<(u8, u8, u8)> = view.art.villager_sprites[3].pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
        let mut figure = 0;
        for y in y0..y0 + 64 { for x in x0..x0 + 64 {
            let i = (y as usize * W + x as usize) * 4;
            let px = (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]);
            if px == GRAVE_STONE { stone += 1; }
            else if fig.contains(&px) && px != GRAVE_EDGE { figure += 1; }
        }}
        assert!(stone > 200, "no gravestone in the card ({} stone px)", stone);
        assert!(figure < 20, "Corvin's living portrait is still drawn ({} px)", figure);
    }
}

#[cfg(test)]
mod member_line_tests {
    use super::*;

    #[test]
    fn test_member_line_reads_and_fits_the_card() {
        assert_eq!(members_line(&[]), "nobody yet - recruit!");
        assert_eq!(members_line(&["Dagny", "Bram"]), "Dagny, Bram");
        let all = members_line(&["Dagny", "Bram", "Aldric", "Corvin", "Eliza"]);
        assert_eq!(all, "Dagny, Bram, Aldric, Corvin, Eliza");
        // past the width it stops at a whole name instead of mid-word
        let long = members_line(&["Dagny", "Bram", "Aldric", "Corvin", "Eliza", "Wulfric"]);
        assert!(long.ends_with(" ..") && long.len() <= MEMBERS_CHARS, "{}", long);
        // the card box ends at UI_X + 280
        let x = UI_X as i32 + 18 + (MEMBERS_LABEL.len() as i32 + 1) * GLYPH_W;
        for line in [&all, &long] {
            assert!(x + text_width(line, 1) <= UI_X as i32 + 278, "{} runs off the card", line);
        }
    }

    #[test]
    fn test_build_log_doesnt_quote_tile_coordinates() {
        let mut game = Game::new();
        // the Wanderer's starting gold is rolled by class and can come up
        // short - this test is about the log line, not the purse
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 100.0;
        game.act(4);
        let line = game.log.last().unwrap();
        assert!(line.contains("new house") && !line.contains('('), "{}", line);
    }
}

#[cfg(test)]
mod reach_name_tests {
    use super::*;

    #[test]
    fn test_names_are_bright_only_within_reach() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        // one villager beside the player, one far across the village
        let (px, py) = game.player_pos();
        let (near, far) = (game.villagers[0], game.villagers[4]);
        *game.world.get_component_mut::<Pos>(near).unwrap() = Pos { x: px + 1.5, y: py };
        *game.world.get_component_mut::<Pos>(far).unwrap() = Pos { x: px + 6.0, y: py + 6.0 }; // 8.5 tiles: just out of reach, still on screen
        game.selected_target = 2;
        let buf = view.render(&game.ui_model(), (-1, -1));
        let name_colour = |v: Entity, i: usize| {
            let p = *game.world.get_component::<Pos>(v).unwrap();
            let (sx, sy) = iso_to_screen_cam(p.x, p.y, view.cam_x, view.cam_y, view.zoom);
            let (lx, ly) = name_label_pos(sx, sprite_label_anchor(&view.art.villager_sprites[i], sy, view.zoom), vill_name(i));
            // labels may be nudged up a line or two; look over that band
            let (mut bright, mut dim) = (0, 0);
            for y in (ly - 40).max(0)..ly + GLYPH_H { for x in lx.max(0)..(lx + vill_name(i).len() as i32 * GLYPH_W).min(MAP_W as i32) {
                let k = (y as usize * W + x as usize) * 4;
                match (buf.pixels[k], buf.pixels[k + 1], buf.pixels[k + 2]) {
                    c if c == NAME_IN_REACH => bright += 1,
                    c if c == NAME_OUT_OF_REACH => dim += 1,
                    _ => {}
                }
            }}
            (bright, dim)
        };
        let (b, d) = name_colour(near, 0);
        assert!(b > 10 && d == 0, "{} is in reach but not bright ({} bright, {} dim)", vill_name(0), b, d);
        let (_, d) = name_colour(far, 4);
        assert!(d > 10, "{} is out of reach but not dimmed", vill_name(4));
    }
}

#[cfg(test)]
mod nature_tests {
    use super::*;

    #[test]
    fn test_card_describes_nature_in_plain_words_that_fit() {
        use AttachmentStyle::*;
        for st in [Secure, Anxious, Avoidant, Disorganized] {
            let line = format!("Nature: {}", nature_word(st));
            assert!(!line.contains(&format!("{:?}", st)), "{} still uses the jargon", line);
            assert!(UI_X as i32 + 60 + text_width(&line, 1) <= UI_X as i32 + 278, "{} runs off the card", line);
        }
    }
}

#[cfg(test)]
mod panel_tests {
    use super::*;

    #[test]
    fn test_wrap_text_breaks_at_words() {
        let lines = wrap_text("t280: Bram tells Dagny - someone saw the lord's tax collector", 20);
        assert!(lines.iter().all(|l| l.chars().count() <= 20), "{:?}", lines);
        assert_eq!(lines.join(" "), "t280: Bram tells Dagny - someone saw the lord's tax collector");
        assert_eq!(lines[0], "t280: Bram tells");
    }

    #[test]
    fn test_wrap_text_splits_overlong_words() {
        assert_eq!(wrap_text("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
        assert!(wrap_text("", 10).is_empty());
    }

    #[test]
    fn test_gossip_about_the_guild_is_not_coloured_as_a_guild_event() {
        assert_eq!(log_colour("t1: The Merchant Guild formed!"), (255, 215, 0));
        assert_ne!(log_colour("t120: Corvin tells Aldric - the guild merchants grow bold"), (255, 215, 0));
        assert_eq!(log_colour("Eliza joined!"), (120, 255, 120));
    }
}

#[cfg(test)]
mod minimap_tests {
    use super::*;

    #[test]
    fn test_screen_and_tile_projections_agree_on_the_minimap() {
        // a tile drawn on screen lands at the same minimap spot as the tile itself
        let (w_t, h_t) = (MAP_TILES as f32, MAP_TILES as f32);
        for &(tx, ty, cam_x, cam_y, zoom) in &[(10.0f32, 16.0f32, 0, 0, 1.0f32), (28.0, 13.0, -120, 40, 1.0), (5.0, 2.0, 30, -10, 1.5)] {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let a = screen_to_minimap(sx as f32, sy as f32, cam_x, cam_y, zoom, w_t, h_t);
            let b = tile_to_minimap(tx, ty, mm_k(w_t, h_t));
            assert!((a.0 - b.0).abs() < 1.0 && (a.1 - b.1).abs() < 1.0, "{:?} vs {:?}", a, b);
        }
    }

    #[test]
    fn test_whole_map_fits_in_the_minimap_box() {
        let (w_t, h_t, n) = (MAP_TILES as f32, MAP_TILES as f32, MAP_TILES as f32 - 1.0);
        for &(tx, ty) in &[(0.0, 0.0), (n, 0.0), (0.0, n), (n, n)] {
            let (x, y) = tile_to_minimap(tx, ty, mm_k(w_t, h_t));
            assert!(x >= MM_LEFT as f32 && x <= (MM_LEFT + MM_W) as f32, "x {} for ({},{})", x, tx, ty);
            assert!(y >= MM_TOP as f32 && y <= (MM_TOP + MM_H) as f32, "y {} for ({},{})", y, tx, ty);
        }
    }

    #[test]
    fn test_minimap_and_map_share_terrain_colours() {
        // heights 0.15-0.35 are grass on the map; the old minimap drew them as water
        assert_eq!(terrain_colour(0.25), (60, 140, 60));
        assert_eq!(terrain_colour(0.05), (30, 60, 180));
    }
}

#[cfg(test)]
mod label_tests {
    use super::*;

    #[test]
    fn test_overlapping_labels_are_stacked_apart() {
        // three names on the same spot (villagers sharing the market tile)
        let ys = place_labels(&[(100, 200, 36), (104, 202, 30), (98, 199, 30)]);
        for a in 0..3 { for b in (a + 1)..3 {
            assert!((ys[a] - ys[b]).abs() >= GLYPH_H + 3, "labels {} and {} still overlap: {:?}", a, b, ys);
        }}
    }

    #[test]
    fn test_separate_labels_stay_where_they_are() {
        let ys = place_labels(&[(10, 100, 30), (200, 100, 30), (10, 300, 30)]);
        assert_eq!(ys, vec![100, 100, 300]);
    }

    #[test]
    fn test_front_label_keeps_its_spot() {
        // the lower label (character in front) stays put; the other moves up
        let ys = place_labels(&[(100, 200, 36), (100, 205, 36)]);
        assert_eq!(ys[1], 205);
        assert!(ys[0] < 200);
    }
}

/// Overcast, falling rain/snow and the post-rain rainbow over the map view.
/// `since_clear` is frames since rain stopped (drives the rainbow fade).
/// The terrain runs right up to the panel, so this covers to UI_X, not MAP_W.
fn draw_weather(buf: &mut Buffer, kind: Option<PrecipKind>, frame: u64, since_clear: Option<u64>) {
    // a clear sky draws nothing: don't scan a million pixels for a mask
    // only rain, snow and the rainbow ever use
    if kind.is_none() && since_clear.is_none() { return; }
    let mask = weather_fx::background_mask(buf, UI_X, (25, 25, 38));
    match kind {
        Some(k) => {
            if k.is_snow() {
                weather_fx::overcast(buf, UI_X, k);
            } else {
                // rain: an even, cool dimming, deeper the harder it falls
                let t = 0.4 + 0.6 * k.intensity();
                for y in 0..H {
                    for x in 0..UI_X {
                        let idx = (y * W + x) * 4;
                        let c = (buf.pixels[idx], buf.pixels[idx + 1], buf.pixels[idx + 2]);
                        let (r, g, b) = blend(c, rain_wash(c), t);
                        buf.pixels[idx] = r; buf.pixels[idx + 1] = g; buf.pixels[idx + 2] = b;
                    }
                }
            }
            weather_fx::draw_precip(buf, UI_X, &mask, k, frame);
        }
        None => if let Some(f) = since_clear {
            weather_fx::draw_rainbow(buf, UI_X, &mask, weather_fx::rainbow_alpha(f));
        },
    }
}

/// The top-left status line: season, day, the village food store, weather, speed
fn hud_text(season: Season, day: u64, food: impl std::fmt::Display, weather: Option<&str>, paused: bool, speed: u32) -> String {
    format!("{:?} | day {} | food store {}{}{}", season, day, food,
        weather.map(|w| format!(" | {}", w)).unwrap_or_default(),
        if paused { " [PAUSED]" } else if speed > 1 { " [4x]" } else { "" })
}

/// The panel's standing reminder of the keys - the only on-screen pointer
/// to the help overlay once the title screen is gone
const PANEL_HINT: &str = "1-8 select villager   H help";

const HELP_LINES: [&str; 9] = [
    "click button / key: take action",
    "click villager: select + inspect",
    "bright name: close enough to talk",
    "1-8: select villager",
    "T talk  R recruit  I invest",
    "O organize  B build",
    "WASD: move   arrows: pan   +/-: zoom",
    "Space: pause   0: speed 4x",
    "F12: screenshot   Esc: quit",
];
const HELP_W: i32 = 420;
const HELP_ROW: i32 = 22;

#[cfg(test)] // only the tests check against it
/// Where the help box sits: centred on the map view, as tall as its text
fn help_rect() -> (i32, i32, i32, i32) {
    overlay_rect(HELP_W, HELP_LINES.len(), HELP_ROW, true)
}

fn draw_help(buf: &mut Buffer) {
    draw_overlay(buf, HELP_W, "CONTROLS", (255, 220, 130), &HELP_LINES, HELP_ROW, Some("press H to close"));
}

const PAUSE_LINES: [&str; 4] = ["Space: resume", "0: speed 1x/4x", "Esc: quit", "1-8: select villager"];
const PAUSE_W: i32 = 260;
const PAUSE_ROW: i32 = 20;

#[cfg(test)]
fn pause_rect() -> (i32, i32, i32, i32) {
    overlay_rect(PAUSE_W, PAUSE_LINES.len(), PAUSE_ROW, false)
}

/// A modal box over the map (help, pause): centred on the map view and
/// sized to its lines, which are centred as a block under a scale-2 title
fn overlay_rect(w: i32, lines: usize, row: i32, footer: bool) -> (i32, i32, i32, i32) {
    let h = 55 + lines as i32 * row + if footer { 30 } else { 12 };
    (MAP_W as i32 / 2 - w / 2, H as i32 / 2 - h / 2, w, h)
}

fn draw_overlay(buf: &mut Buffer, w: i32, title: &str, title_rgb: (u8, u8, u8), lines: &[&str], row: i32, footer: Option<&str>) {
    let (x, y, w, h) = overlay_rect(w, lines.len(), row, footer.is_some());
    let cx = x + w / 2;
    buf.rect(x, y, w, h, 20, 20, 35);
    buf.rect(x, y, w, 2, 200, 200, 220);
    draw_centered_at(buf, cx, y + 15, title, 2, title_rgb);
    // the block of lines is centred as a whole, left-aligned within it
    let block = lines.iter().map(|l| text_width(l, 1)).max().unwrap_or(0);
    for (i, l) in lines.iter().enumerate() {
        draw_text(buf, cx - block / 2, y + 55 + i as i32 * row, l, 210, 210, 210);
    }
    if let Some(f) = footer { draw_centered_at(buf, cx, y + h - 22, f, 1, (150, 150, 170)); }
}

/// The panel's status line: your organization by name, your gold, and the
/// rival merchants' guild by size (it was "Org 5 | Guilds 1 | ... | lead at
/// 2.0" - a count of guild objects, and the goal the bar already shows)
fn status_line(org: &str, members: usize, gold: f32, guild: Option<usize>) -> String {
    let rival = match guild { Some(n) => format!("Merchant Guild {}", n), None => "no guild yet".into() };
    format!("{} {} | Gold {:.0} | {}", org.trim_start_matches("The "), members, gold, rival)
}

/// Label for who's in your organization, on the villager card ("Org:" never
/// said which, and the status line calls it the Harvesters)
const MEMBERS_LABEL: &str = "Joined";
/// Characters that fit after the label, inside the villager card
const MEMBERS_CHARS: usize = 36;

/// The card's member list, cut at a whole name with ".." when it won't fit
fn members_line(names: &[&str]) -> String {
    if names.is_empty() { return "nobody yet - recruit!".into(); }
    let all = names.join(", ");
    if all.len() <= MEMBERS_CHARS { return all; }
    let mut line = names[0].to_string();
    for n in &names[1..] {
        let next = format!("{}, {}", line, n);
        if next.len() + 3 > MEMBERS_CHARS { break; }
        line = next;
    }
    format!("{} ..", line)
}

/// Width in pixels of `text` at `scale`: each glyph is 5px plus 1px
/// spacing, and the last space doesn't count
fn text_width(text: &str, scale: usize) -> i32 {
    (text.chars().count() * 6 * scale) as i32 - scale as i32
}

/// Draw `text` horizontally centred on x = `cx` at row `y`
fn draw_centered_at(buf: &mut Buffer, cx: i32, y: i32, text: &str, scale: usize, (r, g, b): (u8, u8, u8)) {
    draw_text_scaled(buf, cx - text_width(text, scale) / 2, y, text, scale, r, g, b);
}

/// Draw `text` horizontally centred on the screen at row `y`
fn draw_centered(buf: &mut Buffer, y: i32, text: &str, scale: usize, rgb: (u8, u8, u8)) {
    draw_centered_at(buf, W as i32 / 2, y, text, scale, rgb);
}

/// The title screen shown until the player presses Enter: what the game
/// is, what winning means, the main keys, and how to start
pub fn render_title() -> Buffer {
    let mut tbuf = Buffer::new(W, H);
    tbuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
    draw_centered(&mut tbuf, 160, "THE WANDERER", 4, (255, 220, 130));
    draw_centered(&mut tbuf, 230, "an outsider builds power", 2, (200, 200, 200));
    draw_centered(&mut tbuf, 290, "Talk with the villagers, win their trust, organize.", 1, (190, 190, 205));
    draw_centered(&mut tbuf, 306, &format!("Reach {:.1} influence and the village is yours.", WIN_INFLUENCE), 1, (190, 190, 205));
    draw_centered(&mut tbuf, 340, "WASD move   T talk   R recruit   O organize   H help", 1, (140, 140, 160));
    draw_centered(&mut tbuf, 400, "press ENTER to begin", 2, (255, 220, 130));
    tbuf
}

/// Width of the win banner
const BANNER_W: i32 = 360;

/// The "you've won" banner drawn over the last frame, centred on the map
/// view (not straddling the side panel)
pub fn draw_win_banner(buf: &mut Buffer) {
    let cx = MAP_W as i32 / 2;
    buf.rect(cx - BANNER_W / 2, 240, BANNER_W, 100, 20, 20, 35);
    buf.rect(cx - BANNER_W / 2, 240, BANNER_W, 2, 255, 215, 0);
    draw_centered_at(buf, cx, 265, "RECOGNIZED LEADER", 2, (255, 215, 0));
    draw_centered_at(buf, cx, 302, "the village is yours", 1, (200, 200, 200));
}

/// The game-over summary screen after a win
pub fn render_summary(game: &Game) -> Buffer {
    let members = game.world.get_component::<Organization>(game.org).unwrap().members.len();
    let influence = game.world.get_component::<PlayerCharacter>(game.player).unwrap().influence;
    let mut obuf = Buffer::new(W, H);
    obuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
    draw_centered(&mut obuf, 150, "VILLAGE WON", 4, (255, 215, 0));
    draw_centered(&mut obuf, 240, &format!("led the village after {} days", game.tick), 1, (220, 220, 220));
    draw_centered(&mut obuf, 260, &format!("influence {:.2}   members {}   houses built {}", influence, members, game.built_count), 1, (200, 200, 200));
    draw_centered(&mut obuf, 300, "the wanderer became a leader", 1, (160, 160, 180));
    obuf
}

/// Copy a Buffer to the window
/// Headless verification: play the game (no input) until tick N, then
/// render that frame through the same Game::render the window uses and save
/// it as a BMP.
pub fn run_headless(art: Art, target_tick: u64, out: Option<String>, zoom: f32, cam: Option<(i32, i32)>, weather: Option<&str>, seed: u64) {
    let mut game = Game::new_seeded(seed);
    let mut view = View::new(art);
    view.zoom = zoom;
    if let Some(cam) = cam { (view.cam_x, view.cam_y) = cam; }
    while game.tick < target_tick { game.update(); }
    // force the sky for a screenshot of each kind of weather
    if let Some(w) = weather {
        game.precip = match w {
            "drizzle" => Some(PrecipKind::Drizzle), "light" => Some(PrecipKind::LightRain),
            "showers" => Some(PrecipKind::Showers), "rain" => Some(PrecipKind::Rain),
            "heavy" => Some(PrecipKind::HeavyRain), "flurries" => Some(PrecipKind::Flurries),
            "snow" => Some(PrecipKind::Snow), "blizzard" => Some(PrecipKind::HeavySnow),
            _ => None,
        };
        game.rainbow_from = (w == "rainbow").then(|| game.frame_n.saturating_sub(100));
    }
    let buf = view.render(&game.ui_model(), (-1, -1)); // cursor off-screen: no hover
    let path = out.unwrap_or_else(|| format!("evidence/frame-check-{}.bmp", target_tick));
    buf.save_bmp(&path).unwrap();
    println!("frame {} saved to {}", target_tick, path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_block_top_face_covers_one_tile() {
        // the pack's ground blocks are 94x97 cubes drawn for a bigger grid
        let block = Sprite { width: 94, height: 97, pixels: vec![255; 94 * 97 * 4] };
        for zoom in [0.5, 1.0, 2.0] {
            let (sx, sy) = (300, 200);
            let (w, h, x, y) = block_rect(&block, sx, sy, zoom);
            let face_w = BLOCK_FACE.0 / 94.0 * w as f32;
            let face_top = y as f32 + BLOCK_FACE.2 * h as f32 / 97.0;
            assert!((face_w - 2.0 * TILE_W as f32 * zoom).abs() <= 1.0, "zoom {}: face {} px wide", zoom, face_w);
            assert!((face_top - (sy as f32 - TILE_H as f32 * zoom)).abs() <= 1.0, "zoom {}: face starts at {}", zoom, face_top);
            assert!((x + w as i32 / 2 - sx).abs() <= 1, "zoom {}: block centred on its tile", zoom);
        }
    }

    #[test]
    fn test_a_building_stands_with_its_base_on_its_tile() {
        let mut buf = Buffer::new(W, H);
        let house = Sprite { width: 70, height: 67, pixels: vec![255; 70 * 67 * 4] };
        draw_on_tile(&mut buf, &house, 300, 200, 1.0);
        let lit: Vec<(usize, usize)> = (0..H).flat_map(|y| (0..W).map(move |x| (x, y)))
            .filter(|&(x, y)| buf.pixels[(y * W + x) * 4] == 255).collect();
        let (x0, x1) = (lit.iter().map(|p| p.0).min().unwrap(), lit.iter().map(|p| p.0).max().unwrap());
        let bottom = lit.iter().map(|p| p.1).max().unwrap();
        assert_eq!(x1 - x0 + 1, 2 * TILE_W as usize, "a building's base is one tile wide");
        assert_eq!(bottom as i32 + 1, 200 + TILE_H, "it stands on the tile's front corner");
    }

    #[test]
    fn test_name_label_sits_just_above_the_head_and_centred() {
        let (sx, sy) = (300, 200);
        let (lx, ly) = name_label_pos(sx, sy, "Aldric");
        let label_bottom = ly + GLYPH_H;
        let head = sy + SPRITE_HEAD;
        assert!(label_bottom <= head && head - label_bottom <= 6,
            "label should end just above the head (bottom {} vs head {})", label_bottom, head);
        let centre = lx + "Aldric".len() as i32 * GLYPH_W / 2;
        assert_eq!(centre, sx, "label should be centred over the character");
    }
}

#[cfg(test)]
mod real_frame_tests {
    //! Regression checks on the frame a player actually sees: real sprites,
    //! the real Game, Game::render(). (playtest_gate's UI checks use their own
    //! stripped-down renderer and passed even when the real panel was empty.)
    use super::*;

    /// Pixels in a box that differ from the given background colour
    fn drawn(buf: &Buffer, x0: usize, y0: usize, x1: usize, y1: usize, bg: (u8, u8, u8)) -> usize {
        let mut n = 0;
        for y in y0..y1 { for x in x0..x1 {
            let i = (y * W + x) * 4;
            if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) != bg { n += 1; }
        }}
        n
    }

    #[test]
    fn test_rocks_read_as_rocks_not_holes() {
        // the pack's stone(1) is a tiny pebble that's nearly all dark outline:
        // on the map it read as a black hole in the ground
        let art = Art::load_fs("../assets", 0);
        assert!(!art.stones.is_empty(), "stone sprites should load");
        for stone in art.stones.iter() {
            let opaque: Vec<&[u8]> = stone.pixels.chunks(4).filter(|p| p[3] > 128).collect();
            // every stone has a thick dark outline; what makes it read as a rock
            // is a visible light face (stone(1): 10% of its pixels, the boulders 25-33%)
            let light = opaque.iter().filter(|p| (p[0] as u32 + p[1] as u32 + p[2] as u32) > 300).count();
            assert!(light * 5 >= opaque.len(), "stone sprite shows almost no rock face ({} of {} px light)", light, opaque.len());
        }
    }

    #[test]
    fn test_real_frames_show_panel_log_minimap_and_hud() {
        // tests run from game2-core/, the art lives in the repo's assets/
        assert!(std::path::Path::new("../assets").exists(), "assets folder not found from {:?}", std::env::current_dir());
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        // day 1, la grippe (150), rain (200, from the weather sim) and deep winter (330)
        for day in [1u64, 150, 200, 330] {
            while game.tick < day { game.update(); }
            let buf = view.render(&game.ui_model(), (-1, -1));
            let panel = drawn(&buf, UI_X, 0, W, H, (32, 32, 48));
            assert!(panel > (W - UI_X) * H / 6, "day {}: side panel is mostly empty ({} px drawn)", day, panel);
            let log = drawn(&buf, UI_X + 12, 466, UI_X + 278, 618, (25, 25, 40));
            assert!(log > 300, "day {}: event log shows no text ({} px)", day, log);
            let minimap = drawn(&buf, MM_LEFT as usize, MM_TOP as usize, (MM_LEFT + MM_W) as usize, (MM_TOP + MM_H) as usize, (20, 20, 30));
            assert!(minimap > (MM_W * MM_H) as usize / 3, "day {}: minimap has no terrain ({} px)", day, minimap);
            // the HUD line (season, day, food) is text over the map's top-left
            let hud_text = (0..120).flat_map(|x| (6..16).map(move |y| (x, y)))
                .filter(|&(x, y)| { let i = (y * W + x) * 4; buf.pixels[i] > 230 && buf.pixels[i + 1] > 230 && buf.pixels[i + 2] > 230 })
                .count();
            assert!(hud_text > 40, "day {}: HUD text missing ({} white px)", day, hud_text);
        }
    }
}

#[cfg(test)]
mod organize_tests {
    use super::*;
    use crate::game::relationships::RelKind;

    #[test]
    fn test_organize_holds_a_house_meeting_with_nearby_villagers() {
        let mut game = Game::new();
        let (eliza, bram, corvin) = (game.villagers[1], game.villagers[2], game.villagers[3]);
        // the player stands among Eliza and Bram; Corvin is far away
        game.world.get_component_mut::<Pos>(game.player).unwrap().x = 16.0;
        game.world.get_component_mut::<Pos>(game.player).unwrap().y = 16.0;
        for (v, x, y) in [(eliza, 17.0, 16.0), (bram, 15.0, 17.0), (corvin, 2.0, 2.0)] {
            let p = game.world.get_component_mut::<Pos>(v).unwrap();
            p.x = x; p.y = y;
        }
        // everyone else is off across the fields, well out of reach
        for (i, v) in game.villagers.clone().into_iter().enumerate() {
            if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32 * 0.5, y: 3.0 }; }
        }
        // Eliza already trusts the player; Bram is a stranger
        let rel = game.world.create_entity();
        game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });

        assert_eq!(game.disabled_reason(3), None, "a house meeting needs someone nearby, not the selected villager");
        game.act(3);
        finish_meeting(&mut game);

        let members = game.world.get_component::<Organization>(game.org).unwrap().members.clone();
        assert_eq!(members, vec![eliza], "the trusted neighbour joins; the stranger and the absent don't");
        let last = game.log.last().unwrap();
        assert!(last.contains("House meeting - 2 came") && last.contains("Eliza joined"), "{}", last);
        assert!(relationships::find_relationship(&game.world, game.player, bram).is_some(), "Bram got to know the player");
        // a distant villager doesn't attend - the members list above
        // already says Eliza alone; Corvin's absence is no member and
        // no meeting-bond either (long sim windows make incidental bonds)
        assert!(!members.contains(&corvin), "Corvin wasn't there");
    }

    #[test]
    fn test_a_meeting_takes_time_people_gather_then_talk_then_it_ends() {
        let mut game = Game::new();
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 16.0, y: 16.0 };
        let (eliza, bram) = (game.villagers[1], game.villagers[2]);
        for (v, x, y) in [(eliza, 18.0, 16.0), (bram, 15.0, 19.0)] { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x, y }; }
        for (i, v) in game.villagers.clone().into_iter().enumerate() {
            if i != 1 && i != 2 { *game.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: 3.0 + i as f32, y: 3.0 }; }
        }
        let rel = game.world.create_entity();
        game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });
        game.act(3);
        assert!(game.log.last().unwrap().contains("You call a house meeting at"), "{}", game.log.last().unwrap());
        assert!(game.world.get_component::<Organization>(game.org).unwrap().members.is_empty(), "nothing's decided yet");
        assert_eq!(game.disabled_reason(3).as_deref(), Some("a meeting is under way"));
        // they walk over...
        let spots = game.meeting.as_ref().unwrap().spots.clone();
        let far = |game: &Game| [eliza, bram].iter().zip(&spots).map(|(v, s)| {
            let p = game.world.get_component::<Pos>(*v).unwrap(); (p.x - s.0 as f32).abs() + (p.y - s.1 as f32).abs()
        }).sum::<f32>();
        let start = far(&game);
        for _ in 0..FPS { game.update(); }
        assert!(far(&game) < start, "people walk to the meeting");
        // ...then talk it over in thought bubbles, and only then does it end
        let mut thoughts = vec![];
        let mut frames = 0;
        while game.meeting.is_some() {
            game.update();
            frames += 1;
            for b in &game.bubbles { if b.3 && !thoughts.contains(&(b.0, b.1.clone())) { thoughts.push((b.0, b.1.clone())); } }
            assert!(frames < 60 * FPS, "the meeting never ended");
        }
        assert!(thoughts.iter().any(|t| t.0 == eliza) && thoughts.iter().any(|t| t.0 == bram) && thoughts.iter().any(|t| t.0 == game.player),
            "everyone had their say: {:?}", thoughts);
        assert!(frames > 5 * LINE_FRAMES, "a meeting takes a while ({} frames)", frames);
        assert_eq!(game.world.get_component::<Organization>(game.org).unwrap().members, vec![eliza]);
        assert!(game.log.last().unwrap().contains("Eliza joined"), "{}", game.log.last().unwrap());
    }
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

#[cfg(test)]
mod build_tests {
    use super::*;

    #[test]
    fn test_build_costs_gold_and_uses_free_land_near_the_player() {
        let mut game = Game::new();
        let gold0 = game.world.get_component::<PlayerCharacter>(game.player).unwrap().resources;
        let start = game.buildings.len();
        let mut built = 0;
        while game.action_blocker(4).is_none() {
            game.act(4);
            built += 1;
            assert!(built < 100, "Build never stopped");
        }
        assert_eq!(built, (gold0 / BUILD_COST) as usize, "one house per {} gold", BUILD_COST);
        let gold = game.world.get_component::<PlayerCharacter>(game.player).unwrap().resources;
        assert!((gold0 - gold - built as f32 * BUILD_COST).abs() < 1e-3, "gold should drop by the cost of each house");
        assert_eq!(game.action_blocker(4), Some(format!("need {:.0} gold", BUILD_COST)));
        let (px, py) = game.player_pos();
        let new: Vec<_> = game.buildings[start..].iter().map(|(x, y, _)| (*x, *y)).collect();
        for (i, &(x, y)) in new.iter().enumerate() {
            let tile = game.map.get(x as i32, y as i32);
            assert!(matches!(tile, Some(Tile::Grass) | Some(Tile::Hill)), "house on {:?} at ({},{})", tile, x, y);
            assert!((x - px).abs() + (y - py).abs() <= 2.0 * BUILD_SEARCH as f32, "house far from the player");
            let clear_of = |bx: f32, by: f32| (bx.round() - x).abs() > 1.0 || (by.round() - y).abs() > 1.0;
            assert!(game.buildings[..start].iter().all(|(bx, by, _)| clear_of(*bx, *by)), "house on or beside an existing building at ({},{})", x, y);
            assert!(clear_of(MILL_TILE.0, MILL_TILE.1), "house on the windmill at ({},{})", x, y);
            assert!((x - px.round()).abs() + (y - py.round()).abs() >= 2.0, "house right on top of the player at ({},{})", x, y);
            assert!(new[..i].iter().all(|&(ox, oy)| clear_of(ox, oy)), "two houses on or beside each other at ({},{})", x, y);
        }
    }
}

#[cfg(test)]
mod grave_tests {
    use super::*;

    #[test]
    fn test_a_grave_stays_where_the_villager_died() {
        let mut game = Game::new();
        let bram = game.villagers[2];
        game.world.get_component_mut::<Health>(bram).unwrap().condition = 0.0;
        while game.tick < 5 { game.update(); } // the death check turns them into a grave
        assert!(villager::is_dead(&game.world, bram));
        let at = { let p = game.world.get_component::<Pos>(bram).unwrap(); (p.x, p.y) };
        while game.tick < 200 { game.update(); }
        let now = { let p = game.world.get_component::<Pos>(bram).unwrap(); (p.x, p.y) };
        assert_eq!(now, at, "the dead don't wander or get jostled");
    }
}

#[cfg(test)]
mod death_tests {
    use super::*;

    #[test]
    fn test_the_dead_lie_under_a_gravestone_not_standing() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        while game.tick < 60 { game.update(); }
        let corvin = game.villagers[3];
        // a quiet spot away from the crowd, so no label or figure overlaps the grave
        *game.world.get_component_mut::<Pos>(corvin).unwrap() = Pos { x: 21.0, y: 21.5 };
        game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 2 { game.update(); }
        assert!(game.died_on[3].is_some(), "the day of death should be recorded");

        let buf = view.render(&game.ui_model(), (-1, -1));
        let p = *game.world.get_component::<Pos>(corvin).unwrap();
        let (sx, sy) = iso_to_screen_cam(p.x, p.y, view.cam_x, view.cam_y, view.zoom);
        let px = |x: i32, y: i32| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
        // the stone is there, beside the carved cross down its middle -
        // checked across a small patch so a passer-by's edge can't hide it
        let stone_shows = (sy - 7..sy - 1).any(|y| (sx - 6..sx + 1)
            .any(|x| px(x, y) == GRAVE_STONE));
        assert!(stone_shows, "no gravestone where Corvin fell");
        // and nothing is standing on it: the space where a living head would be
        // shows no sprite colours from Corvin's figure
        let fig = &view.art.villager_sprites[3];
        let figure_colours: Vec<(u8, u8, u8)> = fig.pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
        let head = head_y(sy, view.zoom) + 4;
        assert!(!figure_colours.contains(&px(sx, head)), "Corvin is still drawn standing over his grave");
    }
}

#[cfg(test)]
mod minimap_dead_tests {
    use super::*;

    #[test]
    fn test_the_dead_show_as_graves_on_the_minimap() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        while game.tick < 60 { game.update(); }
        let corvin = game.villagers[3];
        game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 2 { game.update(); }
        let buf = view.render(&game.ui_model(), (-1, -1));
        let px = |v: Entity| {
            let p = *game.world.get_component::<Pos>(v).unwrap();
            let (x, y) = tile_to_minimap(p.x, p.y, mm_k(game.map.width as f32, game.map.height as f32));
            let i = ((y as i32 - 1) as usize * W + (x as i32 - 1) as usize) * 4;
            (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2])
        };
        assert_eq!(px(corvin), MM_GRAVE, "dead Corvin still shows as a living villager on the minimap");
        assert_eq!(px(game.villagers[0]), MM_VILLAGER);
    }
}

#[cfg(test)]
mod dog_tests {
    use super::*;

    #[test]
    fn test_dog_goes_with_its_new_owner_when_aldric_dies() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        while game.tick < 60 { game.update(); }
        let aldric = game.villagers[0];
        // Aldric is close to Eliza, so she's who takes his dog in
        let rel = game.world.create_entity();
        game.world.add_component(rel, Relationship {
            from: aldric, other: game.villagers[1], kind: crate::game::relationships::RelKind::Friend, strength: 0.6, interactions: 5,
        });
        game.world.get_component_mut::<Health>(aldric).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 3 { game.update(); }
        let owner = game.world.get_component::<Pet>(game.pets[0].0).unwrap().owner.expect("someone should take the dog in");
        assert_ne!(owner, aldric, "the dog should be rehomed");

        // stand them on a clear tile - nothing taller around to hide the
        // dog (the town is built up enough that a fixed spot won't do)
        let clear = (2..game.map.width as i32 - 2).flat_map(|x| (2..game.map.height as i32 - 2).map(move |y| (x, y)))
            .find(|&(x, y)| game.map.passable(x, y)
                && !game.map.trees.iter().any(|&(tx, ty)| (tx - x).abs() <= 1 && (ty - y).abs() <= 1)
                && !game.stones.contains(&(x, y))
                && !game.map.walls.iter().any(|(w, _)| (w.0 - x).abs() <= 1 && (w.1 - y).abs() <= 1)
                && !game.buildings.iter().any(|&(bx, by, _)| (bx.round() as i32 - x).abs() <= 1 && (by.round() as i32 - y).abs() <= 1)
                && x > 12 && x < 30 && y > 12 && y < 28)
            .expect("a clear tile");
        *game.world.get_component_mut::<Pos>(owner).unwrap() = Pos { x: clear.0 as f32, y: clear.1 as f32 };
        let buf = view.render(&game.ui_model(), (-1, -1));
        let dog_at = |v: Entity| {
            let p = *game.world.get_component::<Pos>(v).unwrap();
            let (dx, dy) = iso_to_screen_cam(p.x + 0.8, p.y + 0.5, view.cam_x, view.cam_y, view.zoom);
            let mut coat = 0;
            for y in (dy - 16)..dy { for x in (dx - 12)..(dx + 12) {
                let i = (y as usize * W + x as usize) * 4;
                if (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) == DOG { coat += 1; }
            }}
            coat >= 20
        };
        assert!(dog_at(owner), "the dog isn't drawn beside its new owner");
        assert!(!dog_at(aldric), "the dog is still drawn at Aldric's grave");
    }
}

#[cfg(test)]
mod dog_sprite_tests {
    use super::*;

    #[test]
    fn test_the_dog_is_a_proper_little_sprite() {
        let dog = dog_sprite();
        assert_eq!((dog.width, dog.height), (11, 7));
        assert!(DOG_ART.iter().all(|r| r.len() == dog.width), "every row of the art is the same width");
        let coat = dog.pixels.chunks(4).filter(|p| p[3] > 0 && (p[0], p[1], p[2]) == DOG).count();
        // drawn 2x that's 4x as many on screen; the old two rectangles were 33 px
        assert!(coat * 4 >= 2 * 33, "only {} coat pixels ({} on screen)", coat, coat * 4);
    }
}

#[cfg(test)]
mod title_tests {
    use super::*;

    #[test]
    fn test_title_lines_are_centred_and_say_how_to_win() {
        let buf = render_title();
        let bg = (15u8, 15u8, 25u8);
        let lit = |x: usize, y: usize| { let i = (y * W + x) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) != bg };
        // find each band of text rows and check it's centred on the screen
        let mut bands = vec![];
        let mut y = 0;
        while y < H {
            if (0..W).any(|x| lit(x, y)) {
                let top = y;
                while y < H && (0..W).any(|x| lit(x, y)) { y += 1; }
                let (mut lo, mut hi) = (W, 0);
                for yy in top..y { for x in 0..W { if lit(x, yy) { lo = lo.min(x); hi = hi.max(x); } } }
                bands.push((top, lo, hi));
            }
            y += 1;
        }
        assert!(bands.len() >= 5, "expected title, subtitle, goal, keys and start lines; found {}", bands.len());
        for (top, lo, hi) in bands {
            let centre = (lo + hi) as i32 / 2;
            assert!((centre - W as i32 / 2).abs() <= 3, "line at y={} is centred at x={}, not {}", top, centre, W / 2);
        }
    }
}

#[cfg(test)]
mod end_screen_tests {
    use super::*;

    /// Horizontal centre of each band of drawn rows in [y0, y1)
    fn line_centres(buf: &Buffer, bg: (u8, u8, u8), x0: usize, x1: usize, y0: usize, y1: usize) -> Vec<(usize, i32)> {
        let lit = |x: usize, y: usize| { let i = (y * W + x) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) != bg };
        let mut out = vec![];
        let mut y = y0;
        while y < y1 {
            if (x0..x1).any(|x| lit(x, y)) {
                let top = y;
                let (mut lo, mut hi) = (x1, x0);
                while y < y1 && (x0..x1).any(|x| lit(x, y)) {
                    for x in x0..x1 { if lit(x, y) { lo = lo.min(x); hi = hi.max(x); } }
                    y += 1;
                }
                out.push((top, (lo + hi) as i32 / 2));
            }
            y += 1;
        }
        out
    }

    #[test]
    fn test_summary_lines_are_centred_and_speak_in_days() {
        let mut game = Game::new();
        while game.tick < 90 { game.update(); }
        let buf = render_summary(&game);
        let lines = line_centres(&buf, (15, 15, 25), 0, W, 0, H);
        assert!(lines.len() >= 4, "expected title and three lines, found {}", lines.len());
        for (y, c) in lines {
            assert!((c - W as i32 / 2).abs() <= 3, "summary line at y={} is centred at x={}", y, c);
        }
    }

    #[test]
    fn test_win_banner_is_centred_on_the_map_view() {
        let mut buf = Buffer::new(W, H);
        draw_win_banner(&mut buf);
        let (bx, bw) = (MAP_W as i32 / 2 - BANNER_W / 2, BANNER_W);
        assert!(bx >= 0 && bx + bw <= MAP_W as i32, "banner {}..{} spills off the map view", bx, bx + bw);
        for (y, c) in line_centres(&buf, (20, 20, 35), bx as usize, (bx + bw) as usize, 250, 340) {
            assert!((c - MAP_W as i32 / 2).abs() <= 3, "banner text at y={} is centred at x={}", y, c);
        }
    }
}

#[cfg(test)]
mod hud_tests {
    use super::*;

    #[test]
    fn test_hud_speaks_in_days_and_names_the_food_store() {
        let hud = hud_text(Season::Summer, 120, 495, Some("heavy rain"), false, 1);
        assert_eq!(hud, "Summer | day 120 | food store 495 | heavy rain");
        assert!(!hud.contains("t120"));
        assert!(hud_text(Season::Winter, 3, 0, None, true, 1).ends_with("[PAUSED]"));
    }

    #[test]
    fn test_hud_sits_on_a_dark_strip() {
        let mut game = Game::new();
        let view = View::new(Art::load_fs("../assets", 0));
        while game.tick < 30 { game.update(); }
        let buf = view.render(&game.ui_model(), (-1, -1));
        // just past the end of the text, still on the strip
        let hud = hud_text(Season::Summer, game.tick, 0, game.precip.map(|k| k.label()), false, 1);
        let x = 10 + text_width(&hud, 1) - 20;
        let i = ((GLYPH_H + 10) as usize * W + x as usize) * 4;
        assert_eq!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), (20, 20, 32), "no backing strip under the HUD");
    }
}


#[cfg(test)]
mod campaign_tests {
    use super::*;

    #[test]
    fn test_the_harvesters_take_a_vote_and_then_they_meet() {
        let mut g = Game::new();
        // three members sworn in, all taxed, and the issue heard - the
        // vote happens (an org with no stake in the cause can't found one)
        for &v in &g.villagers[..3] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..3].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        let msg = g.maybe_found_campaign(0).unwrap();
        assert!(msg.contains("they'll fight for the taxes"), "{}", msg);
        let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();

        // the panel shows the fight, its seats empty
        let m = g.ui_model();
        let cv = m.panel.campaign.as_ref().unwrap();
        assert_eq!(cv.issue, "the taxes");
        assert_eq!(cv.seats.len(), 3);
        assert!(cv.seats.iter().all(|s| s.ends_with("(empty)")), "{:?}", cv.seats);
        assert!(!cv.due, "just founded - the cadence still has to run");

        // and the Meet button runs the campaign meeting when it's due
        g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
            .unwrap().cadence_days = 0;
        g.seats[0].ctx_actions.push(CtxAct::Meet);
        let i = g.seats[0].ctx_actions.len() - 1;
        g.ctx_act_at(0, i);
        assert!(g.log.last().unwrap().contains("meet about the campaign"), "{:?}", g.log.last());
        assert!(g.ui_model().panel.campaign.unwrap().due);
    }

    #[test]
    fn test_the_organizer_sets_how_often_they_meet() {
        let mut g = Game::new();
        for &v in &g.villagers[..3] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..3].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
        assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
            .unwrap().cadence_days, 3, "the founding cadence");
        // the Cadence button steps the recommendation up, then wraps round
        g.seats[0].ctx_actions.push(CtxAct::Cadence);
        let i = g.seats[0].ctx_actions.len() - 1;
        g.ctx_act_at(0, i);
        assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
            .unwrap().cadence_days, 7);
        assert_eq!(g.ui_model().panel.campaign.unwrap().cadence_days, 7);
        for _ in 0..3 { g.ctx_act_at(0, i); }
        assert_eq!(g.world.get_component::<crate::game::campaign::CampaignMeeting>(c)
            .unwrap().cadence_days, 3, "the cycle wraps to the fight's rhythm");
    }

    #[test]
    fn test_a_ctx_button_that_cant_answer_says_why() {
        let mut g = Game::new();
        for &v in &g.villagers[..3] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..3].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        g.seats[0].ctx_actions.push(CtxAct::Meet);
        // the campaign just met at founding - Meet shows but can't answer yet
        let mi = g.ui_model().ctx.iter().position(|a| *a == CtxAct::Meet).unwrap();
        assert!(g.ui_model().ctx_blocked[mi].is_some(), "not due: the button should say so");
        let log_len = g.log.len();
        g.ctx_act_at(0, mi);
        assert!(g.log[log_len].contains("Can't"), "{:?}", g.log.last());
        // and when it IS due the button answers again
        g.tick += 4;
        assert!(g.ctx_blocker_at(0, CtxAct::Meet).is_none());
    }

    #[test]
    fn test_the_losers_grumble_as_the_cause_is_named() {
        let mut g = Game::new();
        for &v in &g.villagers[..3] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..3].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        let msg = g.maybe_found_campaign(0).unwrap();
        // a tax grievance under a shared-grievance banner loses something in
        // the framing - the log hears the dissent, not just the count
        assert!(msg.contains("mutters"), "{}", msg);
    }

    #[test]
    fn test_the_organizer_names_a_seat_and_takes_the_street() {
        let mut g = Game::new();
        for &v in &g.villagers[..3] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..3].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();

        // pick a member and hand them the turnout seat
        g.seats[0].selected_target = 0;
        g.seats[0].picked = true;
        g.seats[0].ctx_actions.push(CtxAct::Delegate);
        g.ctx_act_at(0, 0);
        assert!(g.log.last().unwrap().contains("turnout coordinator"), "{:?}", g.log.last());
        assert_eq!(crate::game::campaign::seat_holder(&g.world, c,
            crate::game::campaign::Role::TurnoutCoordinator), Some(g.villagers[0]));

        // a bonded member answers the march - the street reads the ties
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, g.villagers[1],
                crate::game::relationships::Interaction {
                    other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
        g.seats[0].ctx_actions.push(CtxAct::March);
        g.ctx_act_at(0, 1);
        assert!(g.log.last().unwrap().contains("march in the street"), "{:?}", g.log.last());
    }
}

#[cfg(test)]
mod campaign_dungeon_tests {
    use super::*;

    #[test]
    fn test_the_knights_take_the_loudest_and_the_letters_bring_them_home() {
        let mut g = Game::new();
        for &v in &g.villagers[..4] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..4].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        // name villagers[0] the turnout coordinator, bond the rest to him
        g.seats[0].selected_target = 0;
        g.seats[0].picked = true;
        g.seats[0].ctx_actions.push(CtxAct::Delegate);
        g.ctx_act_at(0, 0);
        for &v in &g.villagers[1..4] {
            for _ in 0..3 {
                crate::game::relationships::record_interaction(&mut g.world, v,
                    crate::game::relationships::Interaction {
                        other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
            }
        }
        // a real crowd draws the knights, and they take the loudest
        g.seats[0].ctx_actions.push(CtxAct::March);
        g.ctx_act_at(0, 1);
        let last = g.log.last().unwrap().clone();
        assert!(last.contains("knights come out") && last.contains("is taken"), "{}", last);
        // then the letters bring him home
        g.seats[0].ctx_actions.push(CtxAct::Petition);
        for _ in 0..4 { g.ctx_act_at(0, 2); }
        assert!(g.log.iter().any(|l| l.contains("walks free")), "{:?}", g.log);
    }

    #[test]
    fn test_the_cell_holds_them_they_dont_live_their_lives() {
        let mut g = Game::new();
        let v = g.villagers[0];
        let at = g.world.get_component::<Pos>(v).copied().unwrap();
        let hunger = g.world.get_component::<Hunger>(v).unwrap().value;
        g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
        // the held take no errands
        assert!(!g.send(v, Goal::Wander));
        // days pass: they don't stroll off, don't hunger in the cell
        for _ in 0..600 { g.update(); }
        let now = g.world.get_component::<Pos>(v).copied().unwrap();
        assert_eq!((at.x, at.y), (now.x, now.y), "the held stand where the guards left them");
        assert_eq!(g.world.get_component::<Hunger>(v).unwrap().value, hunger,
            "the castle feeds its prisoners - no starving where they stand");
    }

    #[test]
    fn test_the_held_walk_free_when_the_village_never_stops_asking() {
        let mut g = Game::new();
        let v = g.villagers[0];
        g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
        // no campaign, no petition - just the days passing and the
        // remembering that gathers a little every one of them
        for t in 1..55 { g.marks_tick(g.tick + t); }
        assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
            "fifty days of asking is a sentence served");
        assert!(g.log.iter().any(|l| l.contains("walks free")), "{:?}", g.log);
    }

    #[test]
    fn test_the_taken_get_a_release_campaign_and_their_friends_sign_on() {
        let mut g = Game::new();
        for &v in &g.villagers[..4] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..4].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        // the loudest leads, and a bystander loves him enough to feel the take
        for &v in &g.villagers[1..4] {
            for _ in 0..3 {
                crate::game::relationships::record_interaction(&mut g.world, v,
                    crate::game::relationships::Interaction {
                        other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
            }
        }
        let friend = g.villagers[6];
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, friend,
                crate::game::relationships::Interaction {
                    other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
        g.seats[0].ctx_actions.push(CtxAct::March);
        g.ctx_act_at(0, 0);
        let prisoner = g.villagers.iter().copied()
            .find(|&v| g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some())
            .expect("the crackdown takes the loudest");
        // the cause becomes a person: a campaign of their own
        let rc = g.world.get_component::<crate::game::dungeon::ReleaseOf>(prisoner)
            .map(|r| r.campaign).expect("a release campaign stands for the taken");
        assert!(g.world.get_component::<crate::game::campaign::Campaign>(rc).is_some());
        // and the ones who loved them sign on without an invite
        assert!(g.world.get_component::<Organization>(g.org).unwrap().members.contains(&friend),
            "the friend signs on for the release");
    }

    #[test]
    fn test_you_cant_talk_a_man_out_of_a_cell() {
        let mut g = Game::new();
        // stand him beside the player, then the knights take him
        let v = g.villagers[0];
        let (px, py) = g.player_pos_at(0);
        *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: px + 1.0, y: py };
        g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
        // he's not around to be reached, and the buttons say why
        assert!(!g.villagers_in_reach().contains(&v));
        g.seats[0].selected_target = 0;
        g.seats[0].picked = true;
        assert_eq!(g.disabled_reason(0).as_deref(), Some("in the dungeon"));
        assert_eq!(g.disabled_reason(1).as_deref(), Some("in the dungeon"));
        g.act(0);
        assert!(g.log.last().unwrap().contains("in the dungeon"), "{:?}", g.log.last());
    }

    #[test]
    fn test_the_knights_ride_home_when_the_tour_is_done() {
        let mut g = Game::new();
        g.world.add_component(g.org, crate::game::reactions::GuardsPosted { knights: 2, since: 10 });
        g.posted.push((g.org, 15.0, 16.0));
        g.posted.push((g.org, 14.0, 16.0));
        // another org's knights still stand their own tour
        let other = g.world.create_entity();
        g.posted.push((other, 20.0, 20.0));
        // mid-tour they stand their ground
        g.guards_tick(20);
        assert!(g.world.get_component::<crate::game::reactions::GuardsPosted>(g.org).is_some());
        assert_eq!(g.posted.len(), 3);
        // the tour done, they ride out - the posting and the street both
        // clear, but only theirs: another org's knights keep their own tour
        g.guards_tick(40);
        assert!(g.world.get_component::<crate::game::reactions::GuardsPosted>(g.org).is_none());
        assert_eq!(g.posted.len(), 1);
        assert_eq!(g.posted[0].0, other);
        // the gate watch, though, stands all year
        assert!(!g.guards.is_empty());
        assert!(g.log.iter().any(|l| l.contains("knights ride home")), "{:?}", g.log);
    }

    #[test]
    fn test_petition_shows_even_when_the_cause_is_spent() {
        let mut g = Game::new();
        // no campaign at all - and someone's inside
        g.seats[0].selected_target = 0;
        g.seats[0].picked = true;
        let v = g.villagers[0];
        g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
        g.update_ctx_actions_at(0);
        assert!(g.seats[0].ctx_actions.contains(&CtxAct::Petition),
            "the way out doesn't depend on a live campaign: {:?}", g.seats[0].ctx_actions);
    }

    #[test]
    fn test_the_cell_opens_when_the_knights_ride_home() {
        let mut g = Game::new();
        let v = g.villagers[0];
        g.world.get_component_mut::<Organization>(g.org).unwrap().members = vec![v];
        g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
        g.world.add_component(g.org, crate::game::reactions::GuardsPosted { knights: 2, since: 0 });
        // petitions unanswered and the tour still ends: the cell opens
        g.guards_tick(30);
        assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
            "no life sentence - the knights gone, the door stands open");
        assert!(g.log.iter().any(|l| l.contains("walks home")), "{:?}", g.log);
    }

    #[test]
    fn test_the_stint_is_the_stint_whatever_the_tours_do() {
        let mut g = Game::new();
        let v = g.villagers[0];
        g.world.add_component(v, crate::game::dungeon::Imprisoned { since: 0, charge: "sedition".into() });
        // a fresh crackdown every week keeps the tour going - the prisoner
        // still walks when their own month is done
        g.world.add_component(g.org, crate::game::reactions::GuardsPosted { knights: 2, since: 25 });
        g.guards_tick(31);
        assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
            "a restamped tour can't stretch the stint past a month");
    }
}

#[cfg(test)]
mod campaign_visual_tests {
    use super::*;

    #[test]
    fn test_the_woken_sign_on_without_being_convinced() {
        let mut g = Game::new();
        for &v in &g.villagers[..4] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..3].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        // a stranger saw the march and felt the cause wake in them - but
        // they don't know the organizer from a hole in the wall
        let stranger = g.villagers[5];
        g.world.add_component(stranger, crate::game::reactions::Sympathized {
            issues: vec![("the taxes".into(), 1)] });
        let before = g.world.get_component::<Organization>(g.org).unwrap().members.len();
        let (pe, oe) = (g.player, g.org);
        let r = crate::game::player::act_recruit(&mut g.world, pe, oe, stranger);
        assert!(r.success, "the woken sign on without a bond: {}", r.description);
        assert_eq!(g.world.get_component::<Organization>(g.org).unwrap().members.len(), before + 1);
        // and the map reads them over their heads
        let m = g.ui_model();
        let pv = m.people.iter().find(|p| p.e == stranger).unwrap();
        assert!(pv.label.as_deref().unwrap_or("").contains("(woke)"), "{:?}", pv.label);
        // and the card reads them too, once they're picked
        g.seats[0].selected_target = 5;
        g.seats[0].picked = true;
        let insp = g.villager_inspector_at(0);
        assert!(insp.lines.iter().any(|l| l.text.contains("woke")), "{:?}", insp.lines.iter().map(|l| &l.text).collect::<Vec<_>>());
    }
}

#[cfg(test)]
mod campaign_grant_tests {
    use super::*;

    #[test]
    fn test_carried_demands_lighten_the_tax_round() {
        let mut g = Game::new();
        let pe = g.player;
        for &v in &g.villagers[..4] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
            for _ in 0..3 {
                crate::game::relationships::record_interaction(&mut g.world, v,
                    crate::game::relationships::Interaction {
                        other: pe, kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
            }
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..4].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();

        // a man with his own taxing to complain of carries the demand;
        // the doors stay shut on the ones who don't share the fight
        let reeve = g.collector.unwrap();
        desires::add_grievance(&mut g.world, reeve, desires::Problem::Taxed, g.tick + 200);
        g.seats[0].selected_target = COLLECTOR_IDX;
        g.seats[0].picked = true;
        g.seats[0].ctx_actions.push(CtxAct::Townhall);
        g.ctx_act_at(0, 0);
        assert!(g.log.last().unwrap().contains("king will hear"), "{:?}", g.log.last());

        // the grant lands: the cause's people stop counting the taxing
        assert!(g.world.get_component::<crate::game::reactions::LordGrant>(g.org).is_some());
        assert!(!desires::problems_of(&g.world, g.villagers[1], g.tick)
            .contains(&desires::Problem::Taxed), "the grievance is answered");
    }
}

#[cfg(test)]
mod campaign_ban_tests {
    use super::*;

    #[test]
    fn test_a_rule_is_written_against_the_crowd_that_wont_stop() {
        let mut g = Game::new();
        let pe = g.player;
        for &v in &g.villagers[..5] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
            for _ in 0..3 {
                crate::game::relationships::record_interaction(&mut g.world, v,
                    crate::game::relationships::Interaction {
                        other: pe, kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
            }
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..5].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        g.seats[0].ctx_actions.push(CtxAct::March);

        // first crowd: the knights come out and take the loudest
        g.ctx_act_at(0, 0);
        assert!(g.log.last().unwrap().contains("knights come out"), "{:?}", g.log.last());
        // second crowd: the law is written
        g.ctx_act_at(0, 0);
        let last = g.log.last().unwrap().clone();
        assert!(last.contains("no more than two assembled"), "{}", last);
        assert!(g.world.get_component::<crate::game::reactions::AssemblyBan>(g.org).is_some());
        // and under the ban even a handful is broken up on sight
        g.ctx_act_at(0, 0);
        assert!(g.log.last().unwrap().contains("knights come out"), "{:?}", g.log.last());
    }
}

#[cfg(test)]
mod free_villager_tests {
    use super::*;

    #[test]
    fn test_knights_and_priests_stay_off_the_claim_list() {
        let mut g = Game::new();
        // villagers[0] is always a working adult with a Craft vocation
        let knight = g.villagers[0];
        g.world.get_component_mut::<Craft>(knight).unwrap().vocation = Vocation::Knight;
        let claimed = g.free_villager().map(|i| g.villagers[i]);
        assert_ne!(claimed, Some(knight), "a knight keeps their post");
    }

    #[test]
    fn test_kids_stay_off_the_claim_list() {
        let g = Game::new();
        // villagers[26..29] are the three seeded children
        let kids: Vec<Entity> = g.villagers[26..29].to_vec();
        for &k in &kids {
            assert!(g.world.get_component::<Kid>(k).is_some(), "setup: expected a child here");
        }
        let claimed = g.free_villager().map(|i| g.villagers[i]);
        assert!(claimed.map(|c| !kids.contains(&c)).unwrap_or(true), "a child can't take agent commands");
    }
}

#[cfg(test)]
mod agent_tests {
    use super::*;

    #[test]
    fn test_the_agent_who_asks_for_food_gets_a_meal_not_a_stand() {
        let mut g = Game::new();
        let who = g.villagers[0];
        g.world.get_component_mut::<Hunger>(who).unwrap().value = 20.0;
        let servings = g.world.get_component::<villager::FoodSource>(g.market_food).unwrap().servings;
        // Eat queues a real meal at the store, not just a stand beside it
        g.seats.push(Seat { player: who, ..Seat::default() });
        g.agent_command(1, AgentCmd::Eat).unwrap();
        assert!(g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::Eat)));
        // and on arrival - the walk done, standing at the table - a
        // serving goes down and the belly fills
        let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
        *g.world.get_component_mut::<Pos>(who).unwrap() = sp;
        g.arrive(who, Goal::Eat);
        assert_eq!(g.world.get_component::<Hunger>(who).unwrap().value, 100.0);
        assert_eq!(g.world.get_component::<villager::FoodSource>(g.market_food).unwrap().servings,
            servings - 1, "a real serving went down");
    }

    #[test]
    fn test_the_hungry_walk_to_the_store_not_just_stand_about() {
        let mut g = Game::new();
        let who = g.villagers[0];
        // low enough that a stray egg breakfast still leaves them hungry
        g.world.get_component_mut::<Hunger>(who).unwrap().value = 2.0;
        let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
        *g.world.get_component_mut::<Pos>(who).unwrap() = Pos { x: sp.x + 6.0, y: sp.y };
        g.livelihood_tick(1);
        assert!(g.travels.iter().any(|t| t.who == who && matches!(t.goal, Goal::Eat)),
            "a hungry villager is sent for the meal, not left standing");
    }

    #[test]
    fn test_a_dead_hirer_pays_no_coin() {
        let mut g = Game::new();
        let kid = g.villagers[26];
        let hirer = g.villagers[0];
        let to = g.building_entities[0];
        g.world.get_component_mut::<Satchel>(kid).unwrap().goods.insert("wheat".into(), 2);
        if g.world.get_component::<Purse>(kid).is_none() {
            g.world.add_component(kid, Purse { gold: 0.0 });
        }
        g.world.add_component(kid, OddJob {
            hirer, good: "wheat".into(), qty: 2, from: hirer, to, pay: 5.0, loaded: true });
        g.world.add_component(hirer, villager::Dead);
        let stock_before = g.world.get_component::<Building>(to).unwrap().stock.get("wheat").copied().unwrap_or(0);
        let kid_gold = g.world.get_component::<Purse>(kid).unwrap().gold;
        g.arrive(kid, Goal::RunOddJob);
        assert_eq!(g.world.get_component::<Purse>(kid).unwrap().gold, kid_gold,
            "a corpse pays no coin");
        assert_eq!(g.world.get_component::<Building>(to).unwrap().stock.get("wheat").copied().unwrap_or(0),
            stock_before + 2, "the goods land in the place's stock");
        assert!(g.world.get_component::<OddJob>(kid).is_none(), "the errand ends there");
    }

    #[test]
    fn test_the_dead_finish_no_errands() {
        let mut g = Game::new();
        let who = g.villagers[0];
        g.world.get_component_mut::<Hunger>(who).unwrap().value = 20.0;
        let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
        g.travels.push(Travel { who, path: vec![(sp.x.round() as i32, sp.y.round() as i32)], goal: Goal::Eat });
        g.world.add_component(who, villager::Dead);
        g.advance_travels();
        assert!(!g.travels.iter().any(|t| t.who == who), "the errand ends where they fell");
        assert_ne!(g.world.get_component::<Hunger>(who).unwrap().value, 100.0, "a corpse eats nothing");
    }

    #[test]
    fn test_no_conversation_is_had_with_the_dead() {
        let mut g = Game::new();
        let (who, e) = (g.villagers[0], g.villagers[1]);
        g.world.add_component(e, villager::Dead);
        g.arrive(who, Goal::TalkTo(e));
        assert!(g.world.query::<relationships::Relationship>().iter()
            .filter(|&&r| g.world.get_component::<relationships::Relationship>(r)
                .map(|rr| rr.from == who && rr.other == e).unwrap_or(false))
            .count() == 0, "a grave isn't a conversation");
    }

    #[test]
    fn test_the_hungry_at_the_store_eat_where_they_stand() {
        let mut g = Game::new();
        let who = g.villagers[0];
        g.world.get_component_mut::<Hunger>(who).unwrap().value = 2.0;
        let sp = g.world.get_component::<Pos>(g.market_food).copied().unwrap();
        *g.world.get_component_mut::<Pos>(who).unwrap() = Pos { x: sp.x, y: sp.y };
        g.livelihood_tick(1);
        assert_eq!(g.world.get_component::<Hunger>(who).unwrap().value, 100.0,
            "already at the store: the meal goes down now");
    }

    #[test]
    fn test_the_agent_who_the_cause_reached_signs_on_and_writes_for_the_held() {
        let mut g = Game::new();
        // a villager outside the taxed four: a stranger to the cause
        let who = g.villagers[6];
        g.seats.push(Seat { player: who, org: g.org, agent: true, ..Seat::default() });
        // a cause to sign on with
        for &v in &g.villagers[..4] {
            desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
        }
        g.world.get_component_mut::<Organization>(g.org).unwrap().members =
            g.villagers[..4].to_vec();
        g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
            .issues.push("the taxes".into());
        g.maybe_found_campaign(0).unwrap();
        // a stranger to it is refused
        assert!(g.agent_command(1, AgentCmd::JoinOrg).is_err());
        // but the woken sign on without being convinced
        g.world.add_component(who, crate::game::reactions::Sympathized {
            issues: vec![("the taxes".into(), g.tick)] });
        g.agent_command(1, AgentCmd::JoinOrg).unwrap();
        assert!(g.world.get_component::<Organization>(g.org).unwrap().members.contains(&who));
        // and letters reach a prisoner, whoever writes them
        let held = g.villagers[1];
        assert!(g.agent_command(1, AgentCmd::Press { prisoner: held }).is_err(), "no one inside yet");
        g.world.add_component(held, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
        g.agent_command(1, AgentCmd::Press { prisoner: held }).unwrap();
        assert!(g.world.get_component::<crate::game::dungeon::ReleaseCampaign>(held)
            .map(|r| r.pressure > 0.0).unwrap_or(false), "the letters pile up");
    }
}

#[cfg(test)]
mod growth_tests {
    use super::*;

    #[test]
    fn test_a_full_house_gets_a_child() {
        let mut g = Game::new();
        // a house with two grown folk: room fills out from there
        let home = g.building_entities.iter().copied().find(|&b|
            g.world.get_component::<Building>(b).map(|bb| bb.kind == BuildingKind::Home).unwrap_or(false)).unwrap();
        let two: Vec<Entity> = g.villagers[..2].to_vec();
        g.world.get_component_mut::<Building>(home).unwrap().occupants = two;
        let n = g.villagers.len();
        g.maybe_birth(40); // may or may not, it's a roll - keep calling until the roll lands
        for t in (40..400).step_by(40) { if g.villagers.len() > n { break; } g.maybe_birth(t); }
        assert!(g.villagers.len() > n, "a full house gets a child");
        let baby = *g.villagers.last().unwrap();
        assert!(g.world.get_component::<Kid>(baby).is_some());
        assert!(g.world.get_component::<KidGrowth>(baby).is_some());
        assert!(g.log.iter().any(|l| l.contains("born")), "{:?}", g.log.last());
    }

    #[test]
    fn test_a_stranger_settles_at_the_gate() {
        let mut g = Game::new();
        let n = g.villagers.len();
        g.maybe_migrant(30);
        assert_eq!(g.villagers.len(), n + 1);
        let m = *g.villagers.last().unwrap();
        // trade in hand, and a roof if one's going (a bed want otherwise)
        assert!(g.world.get_component::<Craft>(m).is_some());
        let housed = household::home_of(&g.world, m).is_some();
        let wants_bed = desires::problems_of(&g.world, m, g.tick).contains(&desires::Problem::NoHome);
        assert!(housed || wants_bed, "a spare bed or the want of one");
        assert!(g.log.iter().any(|l| l.contains("settles")), "{:?}", g.log.last());
    }
}

#[cfg(test)]
mod drink_tests {
    use super::*;
    use crate::game::villager::Drunk;

    /// The village's taproom: whichever of the four trades pours here
    fn drink_shop(g: &Game) -> Option<Entity> {
        g.building_entities.iter().copied().find(|&be|
            g.world.get_component::<Building>(be)
                .map(|b| matches!(b.kind, BuildingKind::Shop(
                    Vocation::Alewife | Vocation::Brewer | Vocation::Vintner | Vocation::Distiller))
                    && b.worker.is_some())
                .unwrap_or(false))
    }

    #[test]
    fn test_every_village_keeps_a_drink_trade() {
        let g = Game::new();
        assert!(DRINK_TRADES.iter().any(|&dv| g.villagers.iter().any(|&v|
            g.world.get_component::<Craft>(v).map(|c| c.vocation == dv).unwrap_or(false))),
            "one of the four pours here");
        assert!(drink_shop(&g).is_some(), "and the taproom's a real place");
        assert!(g.log.iter().any(|l| l.contains("The village keeps")));
    }

    #[test]
    fn test_small_ale_sours_and_the_spoil_sells() {
        let mut g = Game::new();
        let shop = drink_shop(&g).unwrap();
        {
            let b = g.world.get_component_mut::<Building>(shop).unwrap();
            b.stock.clear();
            b.stock.insert("ale".into(), 8);
        }
        g.drink_tick(2);
        let b = g.world.get_component::<Building>(shop).unwrap();
        assert!(b.stock.get("ale").copied().unwrap_or(0) < 8, "the ale's turning");
        assert!(b.stock.get("vinegar").copied().unwrap_or(0) > 0, "the spoil's vinegar");
        assert!(vocation::price("vinegar") > 0.0, "and vinegar still sells");
    }

    #[test]
    fn test_wine_turns_on_a_shelf_and_keeps_in_the_dark() {
        let mut g = Game::new();
        let shelf = g.building_entities.iter().copied().find(|&be|
            g.world.get_component::<Building>(be)
                .map(|b| b.kind != BuildingKind::Cellar).unwrap_or(false)).unwrap();
        {
            let b = g.world.get_component_mut::<Building>(shelf).unwrap();
            b.stock.clear();
            b.stock.insert("wine".into(), 4);
        }
        if let Some(cellar) = g.cellar {
            g.world.get_component_mut::<Building>(cellar).unwrap().stock.insert("wine".into(), 4);
        }
        g.drink_tick(12); // a wine day, not a casked one
        assert_eq!(g.world.get_component::<Building>(shelf).unwrap()
            .stock.get("wine").copied().unwrap_or(0), 3, "a shelf bottle turns");
        if let Some(cellar) = g.cellar {
            assert_eq!(g.world.get_component::<Building>(cellar).unwrap()
                .stock.get("wine").copied().unwrap_or(0), 4, "the dark keeps it");
        }
    }

    #[test]
    fn test_a_cask_buys_the_wine_time() {
        let mut g = Game::new();
        let mut bare = None;
        let mut casked = None;
        for &be in &g.building_entities {
            let is_cellar = g.world.get_component::<Building>(be)
                .map(|b| b.kind == BuildingKind::Cellar).unwrap_or(false);
            if is_cellar { continue; }
            if bare.is_none() { bare = Some(be); }
            else if casked.is_none() { casked = Some(be); break; }
        }
        let (bare, casked) = (bare.unwrap(), casked.unwrap());
        {
            let b = g.world.get_component_mut::<Building>(bare).unwrap();
            b.stock.clear(); b.stock.insert("wine".into(), 4);
        }
        {
            let b = g.world.get_component_mut::<Building>(casked).unwrap();
            b.stock.clear(); b.stock.insert("wine".into(), 4); b.stock.insert("casks".into(), 2);
        }
        g.drink_tick(12);
        assert_eq!(g.world.get_component::<Building>(bare).unwrap()
            .stock.get("wine").copied().unwrap_or(0), 3, "a bare bottle turns");
        assert_eq!(g.world.get_component::<Building>(casked).unwrap()
            .stock.get("wine").copied().unwrap_or(0), 4, "the cask holds it");
    }

    #[test]
    fn test_a_cup_goes_down_at_the_counter() {
        let mut g = Game::new();
        let shop = drink_shop(&g).unwrap();
        g.world.get_component_mut::<Building>(shop).unwrap().stock.insert("ale".into(), 5);
        let buyer = g.villagers[8];
        g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 20.0;
        g.arrive(buyer, Goal::BuyAt(shop, "ale", Some(Want::Drink)));
        assert!(desires::is_satisfied(&g.world, buyer, Want::Drink, g.tick),
            "the thirst's answered");
        assert_eq!(g.world.get_component::<Satchel>(buyer).unwrap()
            .goods.get("ale").copied().unwrap_or(0), 0, "nothing carried home");
        assert_eq!(g.world.get_component::<Building>(shop).unwrap().stock["ale"], 4);
    }

    #[test]
    fn test_enough_cups_and_the_morning_after_comes() {
        let mut g = Game::new();
        let shop = drink_shop(&g).unwrap();
        g.world.get_component_mut::<Building>(shop).unwrap().stock.insert("beer".into(), 40);
        let buyer = g.villagers[9];
        g.world.get_component_mut::<Purse>(buyer).unwrap().gold = 500.0;
        for _ in 0..40 {
            if g.world.get_component::<Drunk>(buyer).is_some() { break; }
            g.arrive(buyer, Goal::BuyAt(shop, "beer", Some(Want::Drink)));
        }
        assert!(g.world.get_component::<Drunk>(buyer).is_some(), "the morning after comes");
    }

    #[test]
    fn test_the_collector_counts_the_barrels() {
        let mut g = Game::new();
        let v = g.villagers[5];
        let shop = g.building_entities.iter().copied().find(|&be|
            g.world.get_component::<Building>(be)
                .map(|b| b.worker.is_none() && household::is_workplace(b.kind)).unwrap_or(false))
            .unwrap_or_else(|| drink_shop(&g).unwrap());
        {
            let b = g.world.get_component_mut::<Building>(shop).unwrap();
            b.stock.clear();
            b.stock.insert("beer".into(), 10); // 70 gold of drink
        }
        g.world.get_component_mut::<Craft>(v).unwrap().workplace = Some(shop);
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
        let c = g.collector.unwrap();
        let before = g.world.get_component::<Purse>(c).unwrap().gold;
        g.arrive(c, Goal::CollectTax(v));
        // the year's cut plus the ale excise: 5.0 + (70 * 0.15) = 5 + 11
        let after = g.world.get_component::<Purse>(c).unwrap().gold;
        assert!((after - before - 16.0).abs() < 0.5,
            "the purse cut and the barrel count both came ({})", after - before);
        assert!(g.log.iter().any(|l| l.contains("ale excise")));
    }

    #[test]
    fn test_the_still_owes_the_wise_woman() {
        let mut g = Game::new();
        let still = g.building_entities[10];
        {
            let b = g.world.get_component_mut::<Building>(still).unwrap();
            b.kind = BuildingKind::Shop(Vocation::Distiller);
            b.stock.insert("spirits".into(), 3);
        }
        let hut = g.herb_hut.expect("the herb hut stands on the classic map");
        g.world.get_component_mut::<Building>(hut).unwrap().stock.remove("spirits");
        g.drink_tick(10); // the fortnight's measure
        assert_eq!(g.world.get_component::<Building>(hut).unwrap()
            .stock.get("spirits").copied().unwrap_or(0), 1, "a measure for the tinctures");
        assert_eq!(g.world.get_component::<Building>(still).unwrap()
            .stock.get("spirits").copied().unwrap_or(0), 2);
    }

    #[test]
    fn test_a_watch_earns_its_keep_at_the_cellar() {
        let mut g = Game::new();
        let shop = drink_shop(&g).unwrap();
        {
            let b = g.world.get_component_mut::<Building>(shop).unwrap();
            b.stock.clear();
            b.stock.insert("spirits".into(), 8); // 200 gold - worth stealing
        }
        // post a watch - the classic village rolls none of its own
        g.world.add_component(g.villagers[3], vocation::craft(Vocation::Knight, 0.5));
        g.force_event(events::EventId::CellarThief);
        assert!(g.log.iter().any(|l| l.contains("catches a thief")),
            "a living watch catches him: {:?}", g.log.last());
        assert_eq!(g.world.get_component::<Building>(shop).unwrap()
            .stock.get("spirits").copied().unwrap_or(0), 8, "the good stock stays");
        // no watch, no lock: hang the knights and the drink walks
        for &v in &g.villagers.clone() {
            if g.world.get_component::<Craft>(v)
                .map(|c| c.vocation == Vocation::Knight).unwrap_or(false) {
                g.world.add_component(v, villager::Dead);
            }
        }
        g.force_event(events::EventId::CellarThief);
        assert_eq!(g.world.get_component::<Building>(shop).unwrap()
            .stock.get("spirits").copied().unwrap_or(0), 4, "half the drink walks off");
        assert!(g.log.iter().any(|l| l.contains("thief in the night")));
    }
}
