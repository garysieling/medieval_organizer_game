//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::ecs::{Entity, Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::{draw_text, draw_text_outlined, draw_text_scaled};
use game2_core::game::terrain::Terrain;
use game2_core::game::tilemap::{Layout, Tile, TileMap, WallPiece, NEIGHBOURS, noise};
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest, can_recruit, TalkCount, recruit_influence_needed, RECRUIT_INFLUENCE};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::sim::climate::{self, SeasonSystem, Season};
use game2_core::sim::precipitation::{precip_kind, PrecipKind};
use game2_core::ui::weather_fx;
use game2_core::sim::illness::{Health, Disease, Stage, IllnessSystem};
use game2_core::sim::psychology::{AttachmentHistory, AttachmentStyle, infer_style};
use game2_core::sim::community::{Citizen, Organization};
use game2_core::game::organizing::{meeting_remark, passing_remark, talk_response};
use game2_core::game::traits::personality_from_seed;
use game2_core::game::relationships;
#[cfg(test)]
use game2_core::game::relationships::Relationship;

const W: usize = 1440;
const H: usize = 960;
const MAP_W: usize = 1080;
const UI_X: usize = 1100;
const TILE_W: i32 = 28;
const TILE_H: i32 = 14;
/// Influence at which the village recognizes the player as its leader (the
/// win) - the same goal as the text game
const WIN_INFLUENCE: f32 = game2_core::game::player::LEAD_INFLUENCE;

fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }

fn draw_rect(buf: &mut Buffer, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
    buf.rect(x, y, w, h, r, g, b);
}

/// Convert world tile coords to screen isometric position
/// Point the zoom scales about: the centre of the map view, so zooming
/// keeps what you're looking at on screen
const ZOOM_PIVOT: (f32, f32) = (MAP_W as f32 / 2.0, H as f32 / 2.0);

fn iso_to_screen_cam(tx: f32, ty: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (i32, i32) {
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
const PORTRAIT_FOOT: (i32, i32) = (UI_X as i32 + 6 + 32, 312 + 62);

/// Villager name colours on the map: within MEETING_REACH of the player
/// (Talk, Recruit and Organize can reach them) or not
const NAME_IN_REACH: (u8, u8, u8) = (255, 255, 255);
const NAME_OUT_OF_REACH: (u8, u8, u8) = (150, 150, 165);

/// Gravestone colours
const GRAVE_STONE: (u8, u8, u8) = (135, 135, 145);
const GRAVE_EDGE: (u8, u8, u8) = (55, 55, 65);

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

/// The log entries to show, newest first: every event, but only the latest
/// LOG_GOSSIP_SHOWN pieces of gossip and the latest weather change
fn visible_log(log: &[String]) -> Vec<&String> {
    let (mut gossip, mut weather) = (0, 0);
    log.iter().rev().filter(|e| {
        if is_gossip(e) { gossip += 1; return gossip <= LOG_GOSSIP_SHOWN; }
        if is_weather_note(e) { weather += 1; return weather <= LOG_WEATHER_SHOWN; }
        true
    }).collect()
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
    else if entry.split(|c: char| !c.is_alphabetic()).any(|w| matches!(w, "storm" | "rain" | "raining" | "drizzle" | "showers" | "snow" | "rainbow" | "flu")) { (120, 200, 255) }
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
const MM_W: i32 = 320;
const MM_H: i32 = 160;
const MM_LEFT: i32 = UI_X as i32 + 10;
const MM_TOP: i32 = 660;
const MM_K: f32 = 4.7; // minimap px per tile step along the x/y diagonal

/// Minimap dot colours: living villagers, and the graves of the dead
const MM_VILLAGER: (u8, u8, u8) = (255, 80, 80);
const MM_GRAVE: (u8, u8, u8) = (150, 150, 160);

/// Minimap position of a world tile
fn tile_to_minimap(tx: f32, ty: f32) -> (f32, f32) {
    (MM_LEFT as f32 + MM_W as f32 / 2.0 + (tx - ty) * MM_K, MM_TOP as f32 + 2.0 + (tx + ty) * MM_K / 2.0)
}

/// The tile under minimap pixel (mx, my), if it's on the minimap's tiles
/// (inverse of tile_to_minimap)
fn minimap_tile(mx: f32, my: f32, w_t: f32, h_t: f32) -> Option<(f32, f32)> {
    let (cx, top) = (MM_LEFT as f32 + MM_W as f32 / 2.0, MM_TOP as f32 + 2.0);
    let diff = mx - cx;
    let sum = (my - top) * 2.0;
    let (tx, ty) = ((sum / MM_K + diff / MM_K) / 2.0, (sum / MM_K - diff / MM_K) / 2.0);
    let inside = mx >= MM_LEFT as f32 && mx < (MM_LEFT + MM_W) as f32 && my >= MM_TOP as f32 && my < (MM_TOP + MM_H) as f32;
    (inside && tx >= 0.0 && ty >= 0.0 && tx < w_t && ty < h_t).then_some((tx, ty))
}

/// The camera offset that puts tile (tx, ty) in the middle of the map view
/// (at any zoom: the zoom pivots about the view's centre)
fn camera_on(tx: f32, ty: f32) -> (i32, i32) {
    (-((tx - ty) * TILE_W as f32).round() as i32, (ZOOM_PIVOT.1 + 60.0 - (tx + ty) * TILE_H as f32).round() as i32)
}

/// Minimap position of a point on screen (inverse of iso_to_screen_cam,
/// then tile_to_minimap) - used to outline what the screen is showing
fn screen_to_minimap(sx: f32, sy: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (f32, f32) {
    // undo the zoom about the view centre, then the zoom-1 projection
    let bx = ZOOM_PIVOT.0 + (sx - ZOOM_PIVOT.0) / zoom;
    let by = ZOOM_PIVOT.1 + (sy - ZOOM_PIVOT.1) / zoom;
    let diff = (bx - MAP_W as f32 / 2.0 - cam_x as f32) / TILE_W as f32; // tx - ty
    let sum = (by + 60.0 - cam_y as f32) / TILE_H as f32;                 // tx + ty
    tile_to_minimap((sum + diff) / 2.0, (sum - diff) / 2.0)
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
        && world.get_component::<Villager>(v).map(|x| x.activity != Activity::SeekingFood).unwrap_or(true)
}

/// A villager's attachment style as a player would put it: the card showed
/// the psychology term ("Style: Disorganized"), which reads like a fault
fn nature_word(style: AttachmentStyle) -> &'static str {
    match style {
        AttachmentStyle::Secure => "steady and trusting",
        AttachmentStyle::Anxious => "anxious, needs reassurance",
        AttachmentStyle::Avoidant => "keeps to themselves",
        AttachmentStyle::Disorganized => "wary and unpredictable",
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

/// Is `who` ill right now (a disease not yet beaten)?
fn is_sick(world: &World, who: Entity) -> bool {
    disease_of(world, who).map(|(_, stage)| stage != Stage::Recovering).unwrap_or(false)
}

struct Button {
    label: &'static str, key: char,
    x: i32, y: i32, w: i32, h: i32,
    color: (u8, u8, u8),
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
const FPS: u32 = 60;
/// How long a reply to Talk, and a remark in passing, stay up (seconds)
const TALK_BUBBLE_SECS: u32 = 5;

/// How close the player stands to someone to talk to them (tiles). Talk or
/// Recruit on someone farther off walks the player over first.
const TALK_DIST: f32 = 1.5;

/// Frames the player can make no headway before a walk over to talk gives up
const APPROACH_STUCK_FRAMES: u32 = FPS;

/// The courier's pace along the road (tiles a frame): a day's ride each way
const TRAVELLER_SPEED: f32 = 4.0 / FPS as f32;

/// Walking over to someone to talk to or recruit them
struct Approach {
    who: usize,              // villager index
    action: usize,           // the action button to take on arrival (Talk or Recruit)
    path: Vec<(i32, i32)>,   // tiles still to walk, their tile last
    stuck: u32,              // frames without headway
}
const CHAT_BUBBLE_SECS: u32 = 3;
/// How near two people must be to chat in passing (tiles), how many days
/// before the same pair chat again, and how much a chat counts for (a fifth
/// of a conversation: +0.01 to the relationship)
const CHAT_RANGE: f32 = 1.5;
const CHAT_COOLDOWN: u64 = 10;
const CHAT_INTENSITY: f32 = 0.2;
/// How far the player walks each frame, in tiles (7 tiles a second)
const WALK_SPEED: f32 = 7.0 / FPS as f32;
/// How much faster the player goes on a road, and slower through woods
const ROAD_SPEEDUP: f32 = 1.8;
const WOODS_SLOWDOWN: f32 = 0.6;
/// How far from the player (in tiles) to look for a free building site
const BUILD_SEARCH: i32 = 8;

/// Day a traveller brings the flu to the village
const FLU_DAY: u64 = 120;

/// Keyboard shortcut for each action button, in button order (Talk, Recruit,
/// Invest, Organize, Build). The input loop and the buttons' key hints
/// both come from here so they can't disagree. W/A/S/D move the player, so
/// no action may use them.
const ACTION_KEYS: [(Key, char); 5] = [
    (Key::T, 'T'), (Key::R, 'R'), (Key::I, 'I'), (Key::O, 'O'), (Key::B, 'B'),
];
#[cfg(test)] // only the tests check against it
/// Keys that move the player
const MOVE_KEYS: [char; 4] = ['W', 'A', 'S', 'D'];

const NAMES: [&str; 8] = ["Aldric","Eliza","Bram","Corvin","Dagny","Hob","Marga","Gisela"];
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
struct Art {
    villager_sprites: Vec<Sprite>,
    /// The rivals' figures: the same pack's outlined look, so they read foreign
    rival_sprites: Vec<Sprite>,
    /// The cottages a home can be: colour, timber or plaster, and the door
    /// and gable in different spots (the pack's numbered variants)
    houses: Vec<Option<Sprite>>,
    mill_frames: Vec<Sprite>, // the sails only: they turn on mill_tower
    mill_tower: Option<Sprite>,
    blacksmith_s: Option<Sprite>,
    stones: Vec<Sprite>,        // boulders, a few shapes
    stone_grounds: Vec<Sprite>, // flat rocks
    player_s: Option<Sprite>,
    tower_s: Option<Sprite>,
    /// Town wall blocks: running along x, along y, and the two gatehouses
    walls: [Option<Sprite>; 4],
    /// The odd wall stretch with a postern door in it: along x, along y
    wall_posts: [Option<Sprite>; 2],
    church_s: Option<Sprite>,
    guildhall_s: Option<Sprite>,
    rival_house: Option<Sprite>,
    rival_woodhouse: Option<Sprite>,
    /// The watch at the town gates, from the humans pack
    guards: Vec<Sprite>,
    priest_s: Option<Sprite>,
    traveller_s: Option<Sprite>, // a courier riding the road between villages
    dog: Option<Sprite>,   // hand-drawn: the asset packs have no animals
    cat: Option<Sprite>,
    chicken: Option<Sprite>,
    gull: Option<Sprite>,
    /// Ground specks scattered on open grass, per season set
    details: [Vec<Sprite>; 3],
    /// Ground blocks for the green seasons, autumn and winter (season_tiles),
    /// and the same as top faces only, for inland tiles
    tiles: [TileSet; 3],
    faces: [TileSet; 3],
}

/// The pack's ground blocks for one season's look: one per kind of map
/// tile, and every numbered road and river piece, in the pack's order
#[derive(Default)]
struct TileSet {
    grass: Option<Sprite>,
    /// Sandy grass where the sea meets the land, and drier bare looks up
    /// on the hills and the bare tops
    shore: Option<Sprite>,
    hill: Option<Sprite>,
    rock: Option<Sprite>,
    water: Option<Sprite>,
    /// Pale water round the shore, darker out at sea
    shallow: Option<Sprite>,
    deep: Option<Sprite>,
    wheat: Vec<Sprite>,
    corn: Vec<Sprite>,
    roads: Vec<Sprite>,
    rivers: Vec<Sprite>,
    /// Winter only: iced-over river stretches
    rivers_ice: Vec<Sprite>,
    trees: Vec<Sprite>,
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
    fn load(assets: &Assets, season: usize) -> TileSet {
        let load = |rel: &str| Sprite::load(&assets.path(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/{}", rel))).ok();
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
fn figure(assets: &Assets, rel: &str) -> Sprite {

    let path = assets.path(rel);
    match Sprite::load(&path) {
        Ok(s) => s.crop(0, 0, 32, 32),
        Err(e) => { println!("sprite load {}: {}", path.display(), e); Sprite { width: 1, height: 1, pixels: vec![255, 0, 255, 255] } }
    }
}

/// The sprite at `rel`, if it loads
fn block(assets: &Assets, rel: &str) -> Option<Sprite> {
    Sprite::load(&assets.path(rel)).ok()
}

/// The sprite for `houses[i]`, or none where nothing loaded
static NO_SPRITE: Option<Sprite> = None;

impl Art {
    /// Load every sprite from the asset folder at `root` ("assets" when run
    /// from the repo root, as the game is), with the classic look
    #[cfg(test)]
    fn load(root: &str) -> Art { Self::load_seeded(root, 0) }

    /// The cottage `i` wraps round to (the house list cycles)
    fn house(&self, i: usize) -> &Option<Sprite> {
        self.houses.get(i % self.houses.len().max(1)).unwrap_or(&NO_SPRITE)
    }

    /// Load every sprite, `seed` settling which variant each single-choice
    /// building wears this map (the mill's colour, the corner towers, the
    /// church's keep) - the same map seed looks the same every run
    fn load_seeded(root: &str, seed: u64) -> Art {
        let assets = Assets::new(root);
        let pack = |rel: &str| block(&assets, &format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/{}", rel));

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
            .map(|&(role, gender)| figure(&assets, &assets.villager_sprite(role, gender))).collect();
        // the rivals: same figures, the outlined look, and their matriarch
        // is the pack's queen - every villager sprite ends up in use
        let rival_sprites: Vec<Sprite> = ["MiniVillagerMan", "MiniWorker", "MiniQueen"].iter()
            .map(|n| figure(&assets, &format!("MinifolksVillagers/MinifolksVillagers/Outline/{}.png", n))).collect();
        // the watch at the gates: soldiers from the humans pack
        let guards: Vec<Sprite> = ["MiniSpearMan", "MiniSwordMan", "MiniShieldMan", "MiniHalberdMan"].iter()
            .map(|n| figure(&assets, &format!("MinifolksHumans/Without Outline/{}.png", n))).collect();

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
            tiles: [TileSet::load(&assets, 0), TileSet::load(&assets, 1), TileSet::load(&assets, 2)],
            faces: [0, 1, 2].map(|season| TileSet::load(&assets, season).map_blocks(face_only)),
            mill_frames: (1..=4).map(|i|
                Sprite::load(&assets.path(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/mill/mill({}).png", i))).unwrap()
            ).collect(),
            mill_tower: chosen(["wood", "blue", "green", "red"].iter()
                .map(|c| pack(&format!("Buildings/mill/mill_{}.png", c))).collect()),
            blacksmith_s: Sprite::load(&assets.path(&assets.building_sprite("blacksmith", "green"))).ok(),
            // stone(1) is a tiny pebble that's nearly all dark outline - on
            // the map it read as a black hole, so the boulders stand instead
            stones: (2..=4).filter_map(|i| pack(&format!("Enviroument/Spring/stones/stone({}).png", i))).collect(),
            stone_grounds: (1..=4).filter_map(|i| pack(&format!("Enviroument/Spring/stones/stone_ground ({}).png", i))).collect(),
            player_s: Sprite::load(&assets.path("MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png")).map(|s| s.crop(0, 0, 32, 32)).ok(),
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
            guildhall_s: Sprite::load(&assets.path(&assets.building_sprite("blacksmith", "wood"))).ok(),
            rival_house: Sprite::load(&assets.path(&assets.building_sprite("house", "red"))).ok(),
            rival_woodhouse: Sprite::load(&assets.path(&assets.building_sprite("woodhouse", "red"))).ok(),
            priest_s: Some(figure(&assets, "MinifolksHumans/Without Outline/MiniMage.png")),
            traveller_s: Some(figure(&assets, "MinifolksHumans/Without Outline/MiniHorseMan.png")),
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
struct Works {
    building: usize, // index into Game::buildings
    road: Vec<(i32, i32)>,
    frames: u32,
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
struct Meeting {
    label: String,                       // where it is: "Bram's house"
    invited: Vec<Entity>,
    spots: Vec<(i32, i32)>,              // where each invitee stands, round the venue
    paths: Vec<Option<Vec<(i32, i32)>>>, // each one's way there, next step first (None: no way)
    frames: u32,                         // how long since it was called, unpaused
    talk_from: Option<u32>,              // when the talking began
    lines: Vec<(Entity, String)>,        // the conversation, in order
}
/// How fast people walk to a meeting (tiles a frame), how long the host
/// waits for them, and how long each turn to speak lasts
const MEETING_WALK: f32 = 2.5 / FPS as f32;
const GATHER_FRAMES: u32 = 20 * FPS;
const LINE_FRAMES: u32 = 2 * FPS;
/// What the player says to open the meeting, between rounds, and to close it
const MEETING_PROMPTS: [&str; 3] = [
    "Thanks for coming. What's on your minds?",
    "So what would make the most difference?",
    "Then let's take it on together.",
];

/// The map tests and tooling use: Game::new is always this one
const MAP_SEED: u64 = 42;

/// The map the windowed game plays: GAME2_SEED pins one (for a screenshot
/// of a particular country); otherwise each launch draws a fresh one
fn run_seed() -> u64 {
    std::env::var("GAME2_SEED").ok().and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() % 1_000_000).unwrap_or(MAP_SEED))
}

/// A tiny seeded generator (splitmix64): one seed settles the whole map
struct Rng(u64);
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

/// The whole game state. The windowed game and `--dump-frame` both drive
/// it through the same update() and render(), so a headless frame is the
/// frame a player would see.
struct Game<'a> {
    art: &'a Art,
    world: World,
    sched: Scheduler,
    /// What each tile is (water, grass, road, river...), laid out on the terrain
    map: TileMap,
    climate: Entity,
    villagers: Vec<Entity>,
    rivals: Vec<Entity>,
    rival_homes: Vec<(f32, f32)>, // where each rival lives, across the river
    player: Entity,
    org: Entity,
    /// Each pet and the sprite it draws with, and how it stands off its owner
    pets: Vec<(Entity, &'a Option<Sprite>, (f32, f32))>,
    market_food: Entity, // our village's food store (the rivals have their own)
    buildings: Vec<(f32, f32, &'a Option<Sprite>)>,
    stones: Vec<(i32, i32)>, // boulders out in the fields
    /// The town wall's rectangle this map (it breathes with the seed) - for
    /// keeping the grass specks off the town streets
    town_wall: (i32, i32, i32, i32),
    /// The watch standing just inside each gate
    guards: Vec<(f32, f32)>,
    /// The priest by the church
    priest: (f32, f32),
    /// The courier's road to the rivals' village and back (empty if there is none)
    traveller_path: Vec<(i32, i32)>,
    /// Where the gulls hang about (open-sea tiles)
    gulls: Vec<(f32, f32)>,
    /// The hen's farmyard
    chicken_at: (f32, f32),
    /// The map's own generator: settled at new_seeded, then handing out
    /// sprites (e.g. which cottage each new build gets)
    rng: Rng,
    buttons: Vec<Button>,
    built_count: usize,
    works: Vec<Works>, // houses still being built
    zoom: f32, // 0.5 - 2.0
    toasts: Vec<(String, u64)>,  // (msg, shown until tick)
    last_guild_count: usize,
    tick: u64,
    cam_x: i32,   // camera pan offset
    cam_y: i32,
    paused: bool,
    speed: u32,   // 1=normal 2=fast 4=turbo
    selected_target: usize,
    log: Vec<String>,
    inspector_open: bool,
    help_open: bool,
    bubbles: Vec<(Entity, String, u32, bool, bool)>, // speech bubbles: (speaker, text, until frame, a thought?, ambient passing remark?)
    meeting: Option<Meeting>,
    /// Pairs of people standing near each other, and the day each pair last chatted
    near: Vec<(Entity, Entity)>,
    last_chat: Vec<((Entity, Entity), u64)>,
    frame_n: u32,
    /// The player picked the selected villager themselves (clicked them or
    /// pressed 1-5) and hasn't walked off since: Talk and Recruit go to them,
    /// however far
    picked: bool,
    approach: Option<Approach>,
    precip: Option<PrecipKind>, // what is falling today
    rainbow_from: Option<u32>,   // frame the last rain stopped (rainbow fades from there)
    was_sick: Vec<bool>, // per villager, to log when someone falls ill or recovers
    died_on: Vec<Option<u64>>, // per villager, the day they died
}

impl<'a> Game<'a> {
    /// The classic map (MAP_SEED): tests, dumps and tooling stay put
    #[cfg(test)]
    fn new(art: &'a Art) -> Game<'a> { Game::new_seeded(art, MAP_SEED) }

    /// A game on the map `seed` draws. The country is perturbed - the coast
    /// shifts, the river meanders a tile or two - and the castle suits it:
    /// the wall's east face keeps clear of the river's west bank. Outside it
    /// the map's grown a little townage of its own: a farm below the south
    /// gate, a woodcutter's clearing and a fisher's hut on the shore.
    fn new_seeded(art: &'a Art, seed: u64) -> Game<'a> {
        let mut rng = Rng(seed ^ 0x9E3779B9);
        let mut world = World::new();
        let mut sched = Scheduler::new();
        sched.add_system(villager::HungerSystem);
        sched.add_system(villager::SeekFoodSystem);
        sched.add_system(pets::PetBondSystem);
        sched.add_system(SeasonSystem);
        sched.add_system(GuildFormationSystem);
        sched.add_system(IllnessSystem); // diseases progress, spread to neighbours, and heal

        let mut terrain = Terrain::generate_country(33, 0.7, seed);
        let climate = climate::spawn_climate(&mut world);
        let market_food = villager::spawn_food_source(&mut world, 16.0, 16.0, 500);

        // the river winds south between the towns to the sea; each waypoint
        // on its course drifts a tile or two, so it meanders differently
        // every map (kept east of the wall and west of the rivals')
        let river: Vec<(i32, i32)> = [3, 9, 15, 21, 27].iter()
            .map(|&y| (24 + rng.below(3), y)).collect();
        // the castle suits the river: the east face keeps a tile clear of
        // its west bank and the rest of the rectangle breathes a tile
        // either way (every building still sits well inside)
        let wall = (8 + rng.around(1), 7 + rng.around(1),
            river.iter().map(|w| w.0).min().unwrap() - 1, 24 + rng.around(1));
        // the walled fields run out of the south gate to a little farm
        let (gate_x, farm_y) = ((wall.0 + wall.2) / 2, wall.3 + 2);

        // where each building wants to stand; the map moves one onto dry land if it must
        let planned: Vec<((i32, i32), &Option<Sprite>)> = vec![
            // homes around the market (the first three have chimney smoke)
            ((13, 12), art.house(0)), ((19, 12), art.house(1)), ((12, 18), art.house(2)),
            ((19, 19), &art.blacksmith_s),
            // church (religion centre) + guild hall (merchant base)
            ((14, 9), &art.church_s), ((17, 10), &art.guildhall_s),
            // rival village across the river (red-roofed foreign houses)
            ((28, 13), &art.rival_house), ((30, 15), &art.rival_woodhouse),
        ];
        let mut layout = Layout {
            // our village and the rivals' each stand on dry land, the neck
            // between them never floods, and the farm below the south gate
            // gets its own pocket of field
            settlements: vec![(16, 16, 9), (29, 14, 4), (25, 15, 3), (gate_x, farm_y, 3)],
            river: river.clone(),
            bridge_near: (river[2].0, 15),
            town_wall: Some(wall),
            // fields beside the food store, and the farm's strips outside
            // the south gate
            fields: vec![
                (13, 14, Tile::Wheat), (14, 14, Tile::Wheat), (13, 15, Tile::Wheat), (14, 15, Tile::Wheat),
                (17, 14, Tile::Corn), (18, 14, Tile::Corn), (18, 15, Tile::Corn),
                (gate_x - 2, wall.3 + 1, Tile::Wheat), (gate_x - 1, wall.3 + 1, Tile::Wheat),
                (gate_x - 2, wall.3 + 2, Tile::Wheat), (gate_x - 1, wall.3 + 2, Tile::Corn),
                (gate_x, wall.3 + 2, Tile::Corn), (gate_x + 1, wall.3 + 2, Tile::Wheat),
            ],
            buildings: planned.iter().map(|(p, _)| *p).collect(),
            road_hub: (16, 16),
            road_ends: vec![(29, 14)],
        };
        layout.buildings.push((MILL_TILE.0 as i32, MILL_TILE.1 as i32));
        let mut map = TileMap::generate(&mut terrain, &layout);
        let mut buildings: Vec<(f32, f32, &Option<Sprite>)> = planned.iter().zip(&map.buildings)
            .map(|((_, s), (x, y))| (*x as f32, *y as f32, *s)).collect();

        // out-of-town townage: a farmhand's cottage by the south fields, a
        // woodcutter's clearing at the woods' edge, a fisher's hut on the
        // shore - each gets its road run out to it
        for (site, sprite) in [
            (Some((gate_x + 2, farm_y)), art.house(6)),
            (shore_site(&map, (26, 30)), art.house(7)),
            (woods_edge_site(&map, wall, (wall.0, (wall.1 + wall.3) / 2)), art.house(8)),
        ] {
            let Some((bx, by)) = site.and_then(|(x, y)| map.nearest_buildable(x, y)) else { continue };
            map.occupy(bx, by);
            map.connect((bx, by));
            buildings.push((bx as f32, by as f32, sprite));
        }

        let stones: Vec<(i32, i32)> = [(6, 19), (20, 23)].iter()
            .filter_map(|&(x, y)| map.nearest_buildable(x, y)).collect();
        for &(x, y) in &stones { map.occupy(x, y); }

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
        for (i, _n) in NAMES.iter().enumerate() {
            let (vx, vy) = [(10.0,16.0),(16.0,20.0),(20.0,14.0),(14.0,10.0),(22.0,18.0),(15.0,21.0),(21.0,15.0),(17.0,12.0)][i];
            let v = villager::spawn_villager(&mut world, vx, vy);
            world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
            world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: 4 - i as u32 % 5, episodes: vec![] });
            world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
            world.add_component(v, personality_from_seed(i as u64 * 7 + 3));
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
        let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
        pets::adopt(&mut world, dog, villagers[0]);
        let cat = pets::spawn_pet(&mut world, "Mog", Species::Cat, PetRole::Companion);
        pets::adopt(&mut world, cat, villagers[1]);
        let pets = vec![(dog, &art.dog, (0.8, 0.5)), (cat, &art.cat, (0.5, 0.3))];

        let buttons = vec![
            Button { label: "Talk",     key: ACTION_KEYS[0].1, x: UI_X as i32 + 10, y: 60,  w: 130, h: 36, color: (60,60,120) },
            Button { label: "Recruit",  key: ACTION_KEYS[1].1, x: UI_X as i32 + 10, y: 104, w: 130, h: 36, color: (60,120,60) },
            Button { label: "Invest",   key: ACTION_KEYS[2].1, x: UI_X as i32 + 10, y: 148, w: 130, h: 36, color: (120,60,60) },
            Button { label: "Organize", key: ACTION_KEYS[3].1, x: UI_X as i32 + 10, y: 192, w: 130, h: 36, color: (120,100,40) },
            Button { label: "Build",    key: ACTION_KEYS[4].1, x: UI_X as i32 + 10, y: 236, w: 130, h: 36, color: (80,60,120) },
        ];

        Game {
            art, world, sched, map, climate, villagers, rivals, player, org, pets, market_food, buildings, stones, buttons,
            town_wall: wall,
            guards,
            priest,
            traveller_path,
            gulls,
            chicken_at: (gate_x as f32 + 0.4, farm_y as f32 + 0.3),
            rng,
            built_count: 0,
            works: vec![],
            zoom: 1.0,
            rival_homes,
            toasts: vec![],
            last_guild_count: 0,
            tick: 0,
            // looking at the market to start
            cam_x: camera_on(16.0, 16.0).0,
            cam_y: camera_on(16.0, 16.0).1,
            paused: false,
            speed: 1,
            selected_target: 0,
            log: vec!["Welcome to the village.".into()],
            inspector_open: true,
            help_open: false,
            bubbles: vec![],
            meeting: None,
            near: vec![],
            last_chat: vec![],
            was_sick: vec![false; NAMES.len()],
            died_on: vec![None; NAMES.len()],
            frame_n: 0,
            picked: false,
            approach: None,
            precip: None,
            rainbow_from: None,
        }
    }

    fn ticks_per_frame(&self) -> u32 {
        // frames per day: ~5s/day normal, ~1.2s turbo
        if self.speed == 1 { 5 * FPS } else { 5 * FPS / 4 }
    }

    /// Whether the player can stand at (x, y): not in the water or the
    /// river (the bridge is fine), and not inside a building - walking
    /// round behind one is fine
    fn walkable(&self, x: f32, y: f32) -> bool {
        let (tx, ty) = (x.round() as i32, y.round() as i32);
        let built = self.map.is_occupied(tx, ty)
            || self.buildings.iter().any(|(bx, by, _)| bx.round() as i32 == tx && by.round() as i32 == ty);
        !built && !matches!(self.map.get(tx, ty), None | Some(Tile::Water) | Some(Tile::River))
    }

    /// One frame's step of the player in direction (dx, dy) (tile axes),
    /// quicker along a road and slower through the woods. Blocked straight on, they slide along whatever
    /// is in the way where they can.
    fn walk(&mut self, dx: f32, dy: f32) {
        let (px, py) = self.player_pos();
        let at = (px.round() as i32, py.round() as i32);
        let spd = WALK_SPEED * match self.map.get(at.0, at.1) {
            Some(Tile::Road) | Some(Tile::Bridge) => ROAD_SPEEDUP,
            _ if self.map.trees.contains(&at) => WOODS_SLOWDOWN,
            _ => 1.0,
        };
        // the same pace whichever way they go, diagonals included
        let len = (dx * dx + dy * dy).sqrt();
        let (nx, ny) = (px + dx / len * spd, py + dy / len * spd);
        let to = [(nx, ny), (nx, py), (px, ny)].into_iter().find(|&(x, y)| self.walkable(x, y));
        if let (Some((x, y)), Some(p)) = (to, self.world.get_component_mut::<Pos>(self.player)) {
            p.x = x;
            p.y = y;
        }
    }

    fn player_pos(&self) -> (f32, f32) {
        self.world.get_component::<Pos>(self.player).map(|p| (p.x, p.y)).unwrap_or((10.0, 10.0))
    }

    /// Where the courier is on the road between the villages: there and
    /// back along `traveller_path`, at a riding pace
    fn traveller_pos(&self) -> Option<(f32, f32)> {
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
    fn action_blocker(&self, i: usize) -> Option<String> {
        if i == 4 {
            let gold = self.world.get_component::<PlayerCharacter>(self.player).map(|p| p.resources).unwrap_or(0.0);
            if gold < BUILD_COST { return Some(format!("need {:.0} gold", BUILD_COST)); }
            if self.build_site().is_none() { return Some("no room nearby".into()); }
            return None;
        }
        self.disabled_reason(i)
    }

    /// The nearest free tile to the player where a house can go: dry,
    /// buildable land (not water or peaks), with no building (or the
    /// windmill) on it or next to it - a house sprite is ~2 tiles wide - not
    /// on a road, field or food store, and not right on top of the player
    fn build_site(&self) -> Option<(f32, f32)> {
        let (px, py) = self.player_pos();
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
    fn reach_target(&self) -> usize {
        let in_reach = self.villagers_in_reach();
        if in_reach.contains(&self.villagers[self.selected_target]) { return self.selected_target; }
        let (px, py) = self.player_pos();
        let dist = |v: &Entity| self.world.get_component::<Pos>(*v).map(|p| (p.x - px).powi(2) + (p.y - py).powi(2)).unwrap_or(f32::MAX);
        in_reach.iter()
            .min_by(|a, b| dist(a).partial_cmp(&dist(b)).unwrap())
            .and_then(|v| self.villagers.iter().position(|x| x == v))
            .unwrap_or(self.selected_target)
    }

    /// Why action button `i` can't be used right now, if it can't
    fn disabled_reason(&self, i: usize) -> Option<String> {
        // Recruit is pointless on a member and Invest on an empty purse: say
        // so on the button, as Build does, rather than fail in the log
        // Talk and Recruit need the selected villager alive (they walk over
        // to them); a house meeting (Organize) just needs someone nearby to come
        let target = self.villagers[if (i == 0 || i == 1) && self.picked { self.selected_target } else { self.reach_target() }];
        let in_reach = self.villagers_in_reach();
        let target_alive = self.world.get_component::<Health>(target).is_some();
        let p = self.world.get_component::<PlayerCharacter>(self.player).unwrap();
        let recruit_infl_ok = can_recruit(p.influence);
        let member = self.world.get_component::<Organization>(self.org).map(|o| o.members.contains(&target)).unwrap_or(false);
        if (i == 0 || i == 1) && !target_alive { Some("they have died".into()) }
        else if i == 1 && member { Some("already a member".into()) }
        else if i == 2 && p.resources < INVEST_GOLD { Some(format!("need {:.0} gold", INVEST_GOLD)) }
        else if i == 1 && !recruit_infl_ok { Some(format!("need influence {:.2}", recruit_influence_needed())) }
        else if i == 3 && self.meeting.is_some() { Some("a meeting is under way".into()) }
        else if i == 3 && in_reach.is_empty() { Some("no one nearby".into()) }
        else { None }
    }

    /// One frame of game time: the sim ticks every few frames, and between
    /// ticks villagers keep strolling
    fn update(&mut self) {
        let ticks_per_frame = self.ticks_per_frame();
        // where everyone stood, so no one ends the frame inside a wall or a house
        let people: Vec<Entity> = self.villagers.iter().chain(&self.rivals).copied().collect();
        let stood: Vec<Option<Pos>> = people.iter().map(|e| self.world.get_component::<Pos>(*e).copied()).collect();
        let Game { world, villagers, log, toasts, .. } = self;

        // sim advance: one tick = one day, throttled by frame count
        self.frame_n += 1;
        let mut ticked = false;
        if !self.paused && self.frame_n % ticks_per_frame == 0 {
            self.tick += 1;
            self.sched.tick(world);
            ticked = true;
        }
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
        // villagers separate if too close (no clustering)
        for i in 0..villagers.len() {
            for j in (i+1)..villagers.len() {
                let (ax, ay, bx, by) = {
                    let pa = world.get_component::<Pos>(villagers[i]);
                    let pb = world.get_component::<Pos>(villagers[j]);
                    match (pa, pb) { (Some(a), Some(b)) => (a.x, a.y, b.x, b.y), _ => continue }
                };
                let dx = bx - ax; let dy = by - ay;
                let d2 = dx*dx + dy*dy;
                if d2 < 9.0 && d2 > 0.001 {
                    let d = d2.sqrt();
                    let push = (3.0 - d) * 0.4 / FPS as f32; // eased apart over a few frames
                    let nx = dx / d * push; let ny = dy / d * push;
                    // a grave stays put; the living step around it. Nor is
                    // anyone shoved who's going for food: the push reaches 3
                    // tiles but eating needs 1, so two hungry villagers at
                    // the store pushed each other out of reach and starved
                    if makes_room(world, villagers[i]) {
                        if let Some(p) = world.get_component_mut::<Pos>(villagers[i]) { p.x -= nx; p.y -= ny; }
                    }
                    if makes_room(world, villagers[j]) {
                        if let Some(p) = world.get_component_mut::<Pos>(villagers[j]) { p.x += nx; p.y += ny; }
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
                if idle && !villager::is_dead(world, *v) {
                    if let Some(p) = world.get_component_mut::<Pos>(*v) {
                        let wob = ((tick + *v as u64) % 8) as f32;
                        if wob < 1.0 { p.x += step; } else if wob < 2.0 { p.y += step; }
                        else if wob < 3.0 { p.x -= step; } else if wob < 4.0 { p.y -= step; }
                        p.x = p.x.clamp(2.0, 30.0);
                        p.y = p.y.clamp(2.0, 30.0);
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
            let home = world.get_component::<Pos>(self.market_food).map(|p| (p.x, p.y));
            let food_tile = home.map(|(hx, hy)| (hx.round() as i32, hy.round() as i32));
            for v in villagers.iter() {
                if villager::is_dead(world, *v) { continue; }
                let seeking = world.get_component::<Villager>(*v)
                    .map(|x| x.activity == Activity::SeekingFood).unwrap_or(false);
                let Some(p) = world.get_component_mut::<Pos>(*v) else { continue };
                let Some((hx, hy)) = home else { continue };
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
                            p.x += ddx / dd * step; p.y += ddy / dd * step;
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
                    toasts.push((format!("{} has died.", NAMES[i]), tick + 60));
                    log.push(format!("t{}: {} died.", tick, NAMES[i]));
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
            log.push(format!("t{}: {} tells {} - {}", tick, NAMES[a], NAMES[b], rumors[tick as usize / 40 % 5]));
        }

        // a traveller brings the flu; it's contagious, so neighbours may catch it
        if ticked && tick == FLU_DAY {
            let d = world.create_entity();
            world.add_component(d, Disease { name: "flu".into(), patient: villagers[2], stage: Stage::Moderate, severity: 0.45, contagious: true });
            toasts.push((format!("A traveller brings the flu - {} falls ill", NAMES[2]), tick + 40));
        }
        // log who falls ill and who recovers
        let sick_now: Vec<bool> = villagers.iter().map(|v| is_sick(world, *v)).collect();
        for (i, (&now, was)) in sick_now.iter().zip(self.was_sick.iter_mut()).enumerate() {
            if now && !*was { log.push(format!("t{}: {} has the flu", tick, NAMES[i])); }
            if !now && *was { log.push(format!("t{}: {} recovered from the flu", tick, NAMES[i])); }
            *was = now;
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
        }
        self.advance_approach();
        let frame = self.frame_n;
        self.bubbles.retain(|(_, _, until, _, _)| frame < *until);
        // movement is still processed while paused, but relationships and
        // last_chat shouldn't change while the day isn't advancing
        if !self.paused { self.chat_in_passing(); }
    }

    /// Draw the whole frame: map, villagers, overlays and the side panel.
    /// `mouse` is the cursor position, for button hover and tooltips.
    fn render(&self, mouse: (i32, i32)) -> Buffer {
        let (mx, my) = mouse;
        let world = &self.world;
        let art = self.art;
        let (tick, cam_x, cam_y, zoom) = (self.tick, self.cam_x, self.cam_y, self.zoom);
        let villagers = &self.villagers;
        let selected_target = self.selected_target;
        let in_reach = self.villagers_in_reach();

        let mut buf = Buffer::new(W, H);
        buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);

        // terrain rendered in the SAME projection as entities (unified map)
        let season = world.get_component::<climate::Climate>(self.climate).unwrap().season;
        let tw = (TILE_W as f32 * zoom) as i32;
        let th = (TILE_H as f32 * zoom) as i32;
        // each tile is its block from the pack, drawn back to front so nearer
        // blocks overlap farther ones (the sea is level with the land, or the
        // shore blocks' sides showed over it like a strip of grass).
        // Blocks are scaled once per frame, not once per tile.
        let set = &art.tiles[season_tiles(season)];
        let faces = &art.faces[season_tiles(season)];
        let mut scaled: Vec<(*const Sprite, Sprite)> = vec![];
        let (w_t, h_t) = (self.map.width as i32, self.map.height as i32);
        for d in 0..(w_t + h_t - 1) {
            for tx in (d - h_t + 1).max(0)..=d.min(w_t - 1) {
                let ty = d - tx;
                let Some(tile) = self.map.get(tx, ty) else { continue };
                let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, cam_x, cam_y, zoom);
                if sx < -2 * tw || sx > MAP_W as i32 + 2 * tw || sy < -4 * th || sy > H as i32 + 4 * th { continue; }
                // inland the blocks in front cover a tile's sides: just its face
                let inland = tx + 1 < w_t && ty + 1 < h_t;
                let set = if inland { faces } else { set };
                match set.block(&self.map, tx, ty) {
                    Some(block) => {
                        let (w, h, x, y) = block_rect(block, sx, sy, zoom);
                        let key = block as *const Sprite;
                        if !scaled.iter().any(|(k, _)| *k == key) { scaled.push((key, block.scaled(w, h))); }
                        scaled.iter().find(|(k, _)| *k == key).unwrap().1.blit(&mut buf, x, y);
                    }
                    None => {
                        let (r, g, b) = tile_colour(tile, season);
                        buf.diamond(sx, sy, tw, th, r, g, b);
                        buf.diamond(sx, sy + th, tw, 1, r / 2, g / 2, b / 2);
                    }
                }
                // the odd tuft or flower on open grass outside the walls
                let in_town = (self.town_wall.0..=self.town_wall.2).contains(&tx)
                    && (self.town_wall.1..=self.town_wall.3).contains(&ty);
                if tile == Tile::Grass && !in_town && !self.map.is_occupied(tx, ty) && noise(tx, ty, 7) < 0.10 {
                    if let Some(d) = pick(&art.details[season_tiles(season)], tx, ty, 8) {
                        let k = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
                        let (dw, dh) = ((d.width as f32 * k).round() as i32, (d.height as f32 * k).round().max(1.0) as i32);
                        let (jx, jy) = (noise(tx, ty, 10) - 0.5, noise(tx, ty, 11) - 0.5);
                        draw_zoomed(&mut buf, d,
                            sx + (jx * TILE_W as f32 * zoom) as i32 - dw / 2,
                            sy + ((jy - 0.6) * TILE_H as f32 * zoom) as i32 - dh, k);
                    }
                }
            }
        }

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
        }
        #[derive(Clone, Copy)]
        enum Who { Vill(usize), Rival(usize), Player, Pet(usize), Mill, Smoke(usize), Guard(usize), Priest, Traveller, Chicken }

        let risen = |i: usize| self.works.iter().find(|w| w.building == i).map(|w| w.risen()).unwrap_or(1.0);
        let mut standing: Vec<(f32, f32, Piece)> = self.buildings.iter().enumerate()
            .map(|(i, (x, y, s))| (*x, *y, Piece::Building(s.as_ref(), risen(i)))).collect();
        // a wood is a mix of the pack's trees, stable per tile
        standing.extend(self.map.trees.iter().map(|&(x, y)| (x as f32, y as f32, Piece::Prop(pick(&set.trees, x, y, 4)))));
        // boulders and flat rocks, a different one each time
        let rock_prop = |x: i32, y: i32| if noise(x, y, 14) < 0.5 {
            pick(&art.stones, x, y, 16) } else { pick(&art.stone_grounds, x, y, 16) };
        standing.extend(self.stones.iter().map(|&(x, y)| (x as f32, y as f32, Piece::Prop(rock_prop(x, y)))));
        let rocks = (0..self.map.height as i32).flat_map(|y| (0..self.map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| self.map.get(x, y) == Some(Tile::Rock));
        standing.extend(rocks.map(|(x, y)| (x as f32, y as f32, Piece::Prop(rock_prop(x, y)))));
        standing.extend(self.map.walls.iter().map(|&((x, y), piece)| {
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
        if art.mill_tower.is_some() { people.push((MILL_TILE.0, MILL_TILE.1, Who::Mill)); }
        if art.house(0).is_some() {
            for i in 0..self.buildings.len().min(3) {
                people.push((self.buildings[i].0, self.buildings[i].1, Who::Smoke(i)));
            }
        }
        people.extend(villagers.iter().enumerate()
            .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (p.x, p.y, Who::Vill(i)))));
        people.extend(self.rivals.iter().enumerate()
            .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (p.x, p.y, Who::Rival(i)))));
        let player_pos = self.player_pos();
        people.push((player_pos.0, player_pos.1, Who::Player));
        // the watch stands its post at the gates; the priest keeps by the
        // church; the courier rides the road between the villages; the hen
        // wanders its farmyard
        people.extend(self.guards.iter().enumerate().map(|(i, &(x, y))| (x, y, Who::Guard(i))));
        people.push((self.priest.0, self.priest.1, Who::Priest));
        if let Some((tx, ty)) = self.traveller_pos() { people.push((tx, ty, Who::Traveller)); }
        let hen_t = self.frame_n as f32 * 0.012;
        people.push((self.chicken_at.0 + hen_t.cos() * 0.5, self.chicken_at.1 + (hen_t * 1.7).sin() * 0.4, Who::Chicken));
        // each pet keeps by its owner - whoever that is now: PetBondSystem
        // rehomes them when their owner dies; a stray lingers where it was
        // first kept, by Aldric
        for (i, (pet, _, (ox, oy))) in self.pets.iter().enumerate() {
            if let Some(p) = world.get_component::<Pet>(*pet)
                .and_then(|pet| world.get_component::<Pos>(pet.owner.unwrap_or(villagers[0]))) {
                people.push((p.x + ox, p.y + oy, Who::Pet(i)));
            }
        }
        people.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());

        // market food stock: shown in the HUD (top-left), not on the market
        // tile, where villagers always gather to eat and covered it up
        let food_stock = world.get_component::<villager::FoodSource>(self.market_food)
            .map(|f| f.servings).unwrap_or(0);

        // name labels are drawn after every sprite (so no sprite covers one)
        // and de-overlapped: (text, x, y, colour, hungry, selected)
        let mut labels: Vec<(&str, i32, i32, (u8, u8, u8), bool, bool)> = vec![];
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
                    for (x, y, size, grey) in smoke_puffs(self.frame_n + i as u32 * 37, sx, sy, zoom) {
                        buf.rect(x, y, size, size, grey, grey, grey + 6);
                    }
                },
                Who::Vill(i) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    // the dead get a gravestone where they fell, not their standing figure
                    if world.get_component::<Health>(villagers[i]).is_none() {
                        let top = draw_grave(&mut buf, sx, sy, zoom);
                        let (lx, _) = name_label_pos(sx, sy, NAMES[i]);
                        labels.push((NAMES[i], lx, top - GLYPH_H - LABEL_GAP, (160, 160, 170), false, i == selected_target));
                    } else {
                        draw_character(&mut buf, &art.villager_sprites[i], sx, sy, zoom);
                        let (lx, ly) = name_label_pos(sx, sprite_label_anchor(&art.villager_sprites[i], sy, zoom), NAMES[i]);
                        let hungry = world.get_component::<Hunger>(villagers[i]).map(|h| h.value < 40.0).unwrap_or(false);
                        // bright when close enough to talk, recruit or meet; dim otherwise
                        let rgb = if in_reach.contains(&villagers[i]) { NAME_IN_REACH } else { NAME_OUT_OF_REACH };
                        labels.push((NAMES[i], lx, ly, rgb, hungry, i == selected_target));
                        // sick: a green cross on the other side of the head
                        if is_sick(world, villagers[i]) {
                            let (m, (r, g, b), _) = MARK_SICK;
                            draw_text_outlined(&mut buf, sx - 16, head_y(sy, zoom), m, r, g, b);
                        }
                        // activity marker beside the head
                        if let Some(vil) = world.get_component::<Villager>(villagers[i]) {
                            match vil.activity {
                                Activity::Idle => {},
                                a => {
                                    let (m, (r, g, b), _) = if a == Activity::SeekingFood { MARK_FOOD } else { MARK_BUSY };
                                    draw_text_outlined(&mut buf, sx + 10, head_y(sy, zoom), m, r, g, b);
                                }
                            }
                        }
                    }
                },
                Who::Pet(i) => {
                    let (pet, sprite, _) = self.pets[i];
                    if let Some(pet) = world.get_component::<Pet>(pet) {
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
                        if pet.bond_strength > 0.5 {
                            labels.push(("<3", dx - 6, dy - h - GLYPH_H - LABEL_GAP, (255, 120, 120), false, false)); // placed with the names
                        }
                    }
                },
                // the watch: soldiers from the humans pack at the gates
                Who::Guard(i) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if !art.guards.is_empty() {
                        draw_character(&mut buf, &art.guards[i % art.guards.len()], sx, sy, zoom);
                    }
                },
                Who::Priest => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(s) = &art.priest_s { draw_character(&mut buf, s, sx, sy, zoom); }
                },
                Who::Traveller => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(s) = &art.traveller_s { draw_character(&mut buf, s, sx, sy, zoom); }
                },
                Who::Chicken => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(c) = &art.chicken {
                        let (w, h) = ((c.width as f32 * 2.0 * zoom) as i32, (c.height as f32 * 2.0 * zoom) as i32);
                        draw_zoomed(&mut buf, c, sx - w / 2, sy - h, 2.0 * zoom);
                    }
                },
                // rival villagers (foreign settlement): the outlined figures
                Who::Rival(i) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    let anchor = match art.rival_sprites.get(i) {
                        Some(rs) => { draw_character(&mut buf, rs, sx, sy, zoom); sprite_label_anchor(rs, sy, zoom) }
                        None => label_anchor(sy, zoom),
                    };
                    let (lx, ly) = name_label_pos(sx, anchor, RIVAL_NAMES[i]);
                    labels.push((RIVAL_NAMES[i], lx, ly, (255, 140, 140), false, false));
                },
                // player sprite at their actual position
                Who::Player => {
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
                    labels.push(("YOU", lx, ly, (255, 255, 0), false, false));
                },
            }
        }
        for st in standing { draw_piece(&mut buf, st); }

        // gulls wheel over the open sea
        if let Some(gull) = &art.gull {
            for (i, &(gx, gy)) in self.gulls.iter().enumerate() {
                let t = self.frame_n as f32 * 0.05 + i as f32 * 2.1;
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
        let label_ys: Vec<(&str, i32)> = labels.iter().zip(&placed).map(|(l, y)| (l.0, *y)).collect();
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
        let p = world.get_component::<PlayerCharacter>(self.player).unwrap();
        let org = world.get_component::<Organization>(self.org).unwrap();
        let guild = world.query::<Guild>().first().and_then(|g| world.get_component::<Guild>(*g)).map(|g| g.members.len());
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
        draw_text(&mut buf, UI_X as i32 + 10, 44, &status_line(&org.name, org.members.len(), p.resources, guild), 200, 200, 200);

        // buttons
        for (i, b) in self.buttons.iter().enumerate() {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            let reason = self.action_blocker(i);
            draw_button(&mut buf, b, hover, reason.is_some());
            // a greyed-out button always says why; others explain themselves on hover
            if let Some(r) = &reason {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, r, 200, 140, 120);
            } else if hover {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, TOOLTIPS[i], 220, 220, 160);
            }
        }

        // inspector panel for selected villager
        if self.inspector_open {
            draw_rect(&mut buf, UI_X as i32 + 10, 330, 270, 110, 45, 45, 65);
            let v = villagers[selected_target];
            // 2x portrait: the figure fills the frame's bottom rows, so this
            // puts it level with the stats beside it
            // the dead get their gravestone, as on the map, not a standing figure
            if self.died_on[selected_target].is_some() {
                draw_grave(&mut buf, PORTRAIT_FOOT.0, PORTRAIT_FOOT.1, 2.0);
            } else {
                art.villager_sprites[selected_target].blit_scaled(&mut buf, UI_X as i32 + 6, 312, 2);
            }
            draw_text(&mut buf, UI_X as i32 + 60, 340, NAMES[selected_target], 255, 255, 255);
            if let Some(day) = self.died_on[selected_target] {
                draw_text(&mut buf, UI_X as i32 + 60, 356, &format!("Died on day {}", day), 230, 90, 80);
                draw_text(&mut buf, UI_X as i32 + 60, 372, "rest in peace", 150, 150, 165);
            } else {
            // meters with plain words (Hunger stores fullness, so "Hunger: 24"
            // used to mean very hungry)
            if let Some(h) = world.get_component::<Hunger>(v) {
                draw_meter(&mut buf, UI_X as i32 + 60, 352, "Fed", h.value / 100.0, fullness_word(h.value));
            }
            if let Some(he) = world.get_component::<Health>(v) {
                let note = disease_of(world, v).map(|(name, stage)| format!("{} ({:?})", name, stage))
                    .unwrap_or_else(|| format!("{:.0}%", he.condition * 100.0));
                draw_meter(&mut buf, UI_X as i32 + 60, 364, "Health", he.condition, &note);
            }
            if let Some(a) = world.get_component::<AttachmentHistory>(v) {
                let style = infer_style(a);
                draw_text(&mut buf, UI_X as i32 + 60, 376, &format!("Nature: {}", nature_word(style)), 180, 180, 255);
            }
            if let Some(vil) = world.get_component::<Villager>(v) {
                draw_text(&mut buf, UI_X as i32 + 60, 388, &format!("Doing: {}", activity_text(vil.activity)), 200, 160, 160);
            }
            }
            draw_text(&mut buf, UI_X as i32 + 18, 426, PANEL_HINT, 120, 120, 140);
        }

        // organization: who has joined, by name
        let org_members = world.get_component::<Organization>(self.org).unwrap().members.clone();
        let member_names: Vec<&str> = org_members.iter()
            .filter_map(|m| villagers.iter().position(|v| v == m).map(|i| NAMES[i]))
            .collect();
        draw_text(&mut buf, UI_X as i32 + 18, 408, MEMBERS_LABEL, 255, 215, 0);
        draw_text(&mut buf, UI_X as i32 + 18 + (MEMBERS_LABEL.len() as i32 + 1) * GLYPH_W, 408, &members_line(&member_names), 220, 220, 180);

        // event log
        draw_rect(&mut buf, UI_X as i32 + 10, 450, 270, 170, 25, 25, 40);
        draw_text(&mut buf, UI_X as i32 + 14, 454, "LOG", 150, 150, 180);
        // newest first, word-wrapped to the box instead of cut off mid-word
        const LOG_CHARS: usize = 43;      // (box width - padding) / glyph width
        const LOG_ROWS: usize = 12;
        let mut row = 0;
        'log: for entry in visible_log(&self.log) {
            let (r, g, b) = log_colour(entry);
            // "day 120" in dim grey, then the message in its colour
            let (day, msg) = log_parts(entry);
            let text = match &day { Some(d) => format!("{} {}", d, msg), None => msg.to_string() };
            for (n, line) in wrap_text(&text, LOG_CHARS).iter().enumerate() {
                if row == LOG_ROWS { break 'log; }
                let y = 468 + row as i32 * 12;
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
        draw_weather(&mut buf, self.precip, self.frame_n as u64, self.rainbow_from.map(|f| (self.frame_n - f) as u64));

        // minimap (top-right of map area): the map in miniature, same projection
        // and colours, with villagers, the player and the visible area marked
        buf.rect(MM_LEFT - 2, MM_TOP - 2, MM_W + 4, MM_H + 4, 20, 20, 30);
        let (w_t, h_t) = (self.map.width as f32, self.map.height as f32);
        for my in 0..MM_H { for mx in 0..MM_W {
            if let Some((tx, ty)) = minimap_tile((MM_LEFT + mx) as f32 + 0.5, (MM_TOP + my) as f32 + 0.5, w_t, h_t) {
                let (r, g, b) = self.map.get(tx as i32, ty as i32).map(|t| tile_colour(t, season)).unwrap_or((20, 20, 30));
                // the woods show darker
                let (r, g, b) = if self.map.trees.contains(&(tx as i32, ty as i32)) { blend((r, g, b), (20, 70, 35), 0.5) } else { (r, g, b) };
                buf.set(MM_LEFT + mx, MM_TOP + my, r, g, b);
            }
        }}
        let mm_dot = |buf: &mut Buffer, tx: f32, ty: f32, r: u8, g: u8, b: u8| {
            let (x, y) = tile_to_minimap(tx, ty);
            buf.rect(x as i32 - 1, y as i32 - 1, 2, 2, r, g, b);
        };
        for v in villagers.iter() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                // the dead are graves now: grey, not a living red villager
                let (r, g, b) = if world.get_component::<Health>(*v).is_some() { MM_VILLAGER } else { MM_GRAVE };
                mm_dot(&mut buf, p.x, p.y, r, g, b);
            }
        }
        for rv in self.rivals.iter() {
            if let Some(p) = world.get_component::<Pos>(*rv) { mm_dot(&mut buf, p.x, p.y, 255, 160, 60); }
        }
        mm_dot(&mut buf, player_pos.0, player_pos.1, 255, 255, 0);
        // outline of what the map view is showing, clipped to the minimap box
        let (vx0, vy0) = screen_to_minimap(0.0, 0.0, cam_x, cam_y, zoom);
        let (vx1, vy1) = screen_to_minimap(MAP_W as f32, H as f32, cam_x, cam_y, zoom);
        let (x0, x1) = ((vx0 as i32).max(MM_LEFT), (vx1 as i32).min(MM_LEFT + MM_W - 1));
        let (y0, y1) = ((vy0 as i32).max(MM_TOP), (vy1 as i32).min(MM_TOP + MM_H - 1));
        if x0 < x1 && y0 < y1 {
            buf.rect(x0, y0, x1 - x0 + 1, 1, 255, 255, 255);
            buf.rect(x0, y1, x1 - x0 + 1, 1, 255, 255, 255);
            buf.rect(x0, y0, 1, y1 - y0 + 1, 255, 255, 255);
            buf.rect(x1, y0, 1, y1 - y0 + 1, 255, 255, 255);
        }

        // speech bubbles over whoever is speaking
        for (speaker, text, _, thought, _) in &self.bubbles {
            let (speaker, text, thought) = (*speaker, text.as_str(), *thought);
            {
                if let Some(p) = world.get_component::<Pos>(speaker) {
                    let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                    // off the map view (e.g. under the panel): the log already
                    // has what they said, so don't float a bubble over the panel
                    let name = self.villagers.iter().position(|v| *v == speaker).map(|i| NAMES[i])
                        .or_else(|| self.rivals.iter().position(|r| *r == speaker).map(|i| RIVAL_NAMES[i]));
                    let label_y = name.and_then(|n| label_ys.iter().find(|(t, _)| *t == n).map(|(_, y)| *y));
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
        for (msg, until) in self.toasts.iter() {
            if tick < *until {
                let tw = msg.len() as i32 * 6 + 16;
                buf.rect(MAP_W as i32 / 2 - tw / 2, ty, tw, 20, 20, 20, 35);
                buf.rect(MAP_W as i32 / 2 - tw / 2, ty, tw, 1, 255, 215, 0);
                draw_text(&mut buf, MAP_W as i32 / 2 - tw / 2 + 8, ty + 6, msg, 255, 220, 130);
                ty += 26;
            }
        }

        // season banner
        let hud = hud_text(season, tick, food_stock, self.precip.map(|k| k.label()), self.paused, self.speed);
        // on a dark strip so it reads over any terrain the camera shows
        buf.rect(4, 4, text_width(&hud, 1) + 12, GLYPH_H + 8, 20, 20, 32);
        draw_text_outlined(&mut buf, 10, 8, &hud, 255, 255, 255);

        // help overlay
        if self.help_open { draw_help(&mut buf); }

        // pause menu overlay
        if self.paused {
            draw_overlay(&mut buf, PAUSE_W, "PAUSED", (255, 255, 255), &PAUSE_LINES, PAUSE_ROW, None);
        }

        buf
    }

    /// Talk to villager `who`: hear what they say. Returns the log line
    fn talk_to(&mut self, who: usize) -> String {
        let v = self.villagers[who];
        // Uncapped talk count - Rapport caps at 1.0 and would
        // freeze the concern/small-talk index after a few talks.
        let talks_before = self.world.get_component::<TalkCount>(v)
            .map(|c| c.value).unwrap_or(0);
        let (did, said) = talk_response(&self.world, v, NAMES[who], talks_before);
        act_talk(&mut self.world, self.player, v);
        self.say(v, &said, TALK_BUBBLE_SECS);
        format!("{}: \"{}\"", did, said)
    }

    /// Distance from the player to villager `who`, in tiles
    fn dist_to(&self, who: usize) -> f32 {
        let (px, py) = self.player_pos();
        self.world.get_component::<Pos>(self.villagers[who])
            .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt()).unwrap_or(f32::MAX)
    }

    /// The tiles to walk to reach villager `who`, round walls and buildings
    fn path_to(&self, who: usize) -> Option<Vec<(i32, i32)>> {
        let (px, py) = self.player_pos();
        let p = self.world.get_component::<Pos>(self.villagers[who])?;
        self.map.walk_path((px.round() as i32, py.round() as i32), (p.x.round() as i32, p.y.round() as i32))
    }

    /// Talk or Recruit (action button `action`) on someone who isn't beside
    /// the player: set off to walk over to them (the action happens on
    /// arrival; see advance_approach). None when they're already close enough
    /// to act on now.
    fn start_approach(&mut self, who: usize, action: usize) -> Option<String> {
        if self.dist_to(who) <= TALK_DIST { return None; }
        match self.path_to(who) {
            Some(path) => {
                self.approach = Some(Approach { who, action, path, stuck: 0 });
                Some(format!("You walk over to {}.", NAMES[who]))
            }
            None => Some(format!("Can't {}: there's no way to reach {}.",
                self.buttons[action].label.to_lowercase(), NAMES[who])),
        }
    }

    /// One frame of walking over to someone: follow the path (finding a new
    /// one if they've wandered off), and talk or recruit once beside them
    fn advance_approach(&mut self) {
        let Some(mut a) = self.approach.take() else { return };
        let who = a.who;
        // they died on the way: nothing to say
        if villager::is_dead(&self.world, self.villagers[who]) { return; }
        if self.dist_to(who) <= TALK_DIST {
            // beside them now: take the action on them (act checks it can
            // still be done - they may have joined, or influence slipped)
            self.selected_target = who;
            self.picked = true;
            self.act(a.action);
            return;
        }
        let Some(&Pos { x: vx, y: vy }) = self.world.get_component::<Pos>(self.villagers[who]) else { return };
        let headed_off = a.path.last()
            .map(|&(x, y)| ((x as f32 - vx).powi(2) + (y as f32 - vy).powi(2)).sqrt() > TALK_DIST)
            .unwrap_or(true);
        if headed_off {
            match self.path_to(who) {
                Some(path) => a.path = path,
                None => { self.log.push(format!("t{}: You can't find a way to {}.", self.tick, NAMES[who])); return; }
            }
        }
        let (px, py) = self.player_pos();
        // drop tiles already reached; the last one is their tile, so head for them
        while a.path.len() > 1 {
            let (x, y) = a.path[0];
            if (x as f32 - px).abs() < 0.25 && (y as f32 - py).abs() < 0.25 { a.path.remove(0); } else { break; }
        }
        let (tx, ty) = match a.path.as_slice() { [(x, y), _, ..] => (*x as f32, *y as f32), _ => (vx, vy) };
        let (dx, dy) = (tx - px, ty - py);
        if dx.abs() + dy.abs() > 1e-3 { self.walk(dx, dy); }
        a.stuck = if self.player_pos() != (px, py) { 0 } else { a.stuck + 1 };
        if a.stuck > APPROACH_STUCK_FRAMES {
            self.log.push(format!("t{}: You can't get through to {}.", self.tick, NAMES[who]));
            return;
        }
        self.approach = Some(a);
    }

    /// Take action button `i` (Talk, Recruit, Invest, Organize, Build)
    fn act(&mut self, i: usize) {
        let tick = self.tick;
        // Talk and Recruit go to whoever the player picked; otherwise to
        // whoever is in reach
        if (i == 0 || i == 1) && !self.picked { self.selected_target = self.reach_target(); }
        let selected_target = self.selected_target;
        let msg = if let Some(reason) = self.action_blocker(i) {
            format!("Can't {}: {}.", self.buttons[i].label.to_lowercase(), reason)
        } else {
            match i {
                0 => match self.start_approach(selected_target, 0) {
                    Some(walking) => walking,
                    None => self.talk_to(selected_target),
                },
                1 => if let Some(walking) = self.start_approach(selected_target, 1) { walking } else {
                    let r = act_recruit(&mut self.world, self.player, self.org, self.villagers[selected_target]);
                    if r.success { format!("{} joined!", NAMES[selected_target]) }
                    else { format!("{}: {}", NAMES[selected_target], r.description.to_lowercase()) }
                }
                2 => { let r = act_invest(&mut self.world, self.player, INVEST_GOLD); r.description }
                3 => self.call_meeting(),
                4 => {
                    // action_blocker checked there's gold and a site
                    let (bx, by) = self.build_site().unwrap();
                    if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.player) { p.resources -= BUILD_COST; }
                    self.buildings.push((bx, by, self.art.house(self.rng.below(97) as usize)));
                    self.map.occupy(bx as i32, by as i32);
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
    }

    /// Living villagers within MEETING_REACH of the player
    fn villagers_in_reach(&self) -> Vec<Entity> {
        let (px, py) = self.player_pos();
        self.villagers.iter().copied().filter(|v| {
            let alive = self.world.get_component::<Health>(*v).is_some();
            let near = self.world.get_component::<Pos>(*v)
                .map(|p| ((p.x - px).powi(2) + (p.y - py).powi(2)).sqrt() < MEETING_REACH)
                .unwrap_or(false);
            alive && near
        }).collect()
    }

    /// The end of a house meeting, once those who came have talked it over.
    /// Those who already trust you join the org; everyone who came gets to
    /// know you a little better (organizing::house_meeting).
    fn resolve_meeting(&mut self, invited: &[Entity]) -> String {
        let members_before = self.world.get_component::<Organization>(self.org).map(|o| o.members.clone()).unwrap_or_default();
        let r = game2_core::game::organizing::house_meeting(&mut self.world, self.org, invited);
        let joined: Vec<&str> = invited.iter()
            .filter(|v| !members_before.contains(v))
            .filter(|v| self.world.get_component::<Organization>(self.org).map(|o| o.members.contains(v)).unwrap_or(false))
            .filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| NAMES[i]))
            .collect();
        // a member won at a meeting counts toward leading the village just
        // like one won with Recruit - otherwise every meeting recruit is
        // influence the player can never get back (they can't be Recruited)
        if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.player) {
            p.influence += RECRUIT_INFLUENCE * joined.len() as f32;
        }
        let outcome = if joined.is_empty() {
            "No one joins yet, but they'll remember the conversation.".to_string()
        } else {
            format!("{} joined{}!", joined.join(", "), if r.formed_committee { " and formed a committee" } else { "" })
        };
        format!("House meeting - {} came. {}", invited.len(), outcome)
    }

    /// Put a speech bubble over `who` for `secs` seconds (replacing any
    /// they already had)
    fn say(&mut self, who: Entity, text: &str, secs: u32) {
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), self.frame_n + secs * FPS, false, false));
    }

    /// Put a thought bubble over `who` for `frames` frames (at a meeting)
    fn think(&mut self, who: Entity, text: &str, frames: u32) {
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), self.frame_n + frames, true, false));
    }

    /// Ambient chatter as people pass each other: unlike `say`, this never
    /// overwrites a bubble that isn't itself passing chatter while that
    /// bubble is still up (e.g. a Talk reply), so a reply can't be clobbered
    /// by a passing remark on the very next frame.
    fn say_in_passing(&mut self, who: Entity, text: &str, secs: u32) {
        let frame = self.frame_n;
        let blocked = self.bubbles.iter().any(|(w, _, until, _, passing)| *w == who && !*passing && frame < *until);
        if blocked { return; }
        self.bubbles.retain(|(w, _, _, _, _)| *w != who);
        self.bubbles.push((who, text.to_string(), frame + secs * FPS, false, true));
    }

    /// Organize: call a house meeting at the home nearest the player (or
    /// where they stand, if no home is in reach) for every living villager
    /// within reach. They walk over; see advance_meeting.
    fn call_meeting(&mut self) -> String {
        let invited = self.villagers_in_reach();
        if invited.is_empty() { return "No one is near enough to come to your house meeting.".into(); }
        let (px, py) = self.player_pos();
        let dist = |(x, y): (i32, i32)| ((x as f32 - px).powi(2) + (y as f32 - py).powi(2)).sqrt();
        let home = self.buildings.iter().enumerate()
            .filter(|(_, (_, _, s))| self.art.houses.iter().any(|h| std::ptr::eq(*s, h)))
            .map(|(i, (x, y, _))| ((*x as i32, *y as i32), if i < 3 { format!("{}'s house", NAMES[i]) } else { "the new house".to_string() }))
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
        let names: Vec<&str> = invited.iter().filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| NAMES[i])).collect();
        let msg = format!("You call a house meeting at {} - {} {} on the way.", label, names.join(", "), if names.len() == 1 { "is" } else { "are" });
        self.meeting = Some(Meeting { label, invited, spots, paths, frames: 0, talk_from: None, lines: vec![] });
        msg
    }

    /// What's said at a meeting: the player opens, each villager there says
    /// what's on their mind (organizing::meeting_remark), twice round, with
    /// the player drawing them on between, and the player closes
    fn meeting_lines(&self, came: &[Entity]) -> Vec<(Entity, String)> {
        if came.is_empty() { return vec![]; }
        let mut lines = vec![(self.player, MEETING_PROMPTS[0].to_string())];
        for round in 0..2 {
            for v in came {
                let Some(i) = self.villagers.iter().position(|x| x == v) else { continue };
                lines.push((*v, meeting_remark(&self.world, *v, NAMES[i], round)));
            }
            lines.push((self.player, MEETING_PROMPTS[round + 1].to_string()));
        }
        lines
    }

    /// One frame of a house meeting: while people gather, each walks a
    /// step towards their spot; once they're all there (or the host has
    /// waited GATHER_FRAMES) they talk in turns, then it resolves
    fn advance_meeting(&mut self) {
        let Some(mut m) = self.meeting.take() else { return };
        m.frames += 1;
        // those who've arrived stand at their spot (no strolling off mid-meeting);
        // a dead invitee's grave stays where they died, it doesn't keep walking
        for (i, v) in m.invited.iter().enumerate() {
            if villager::is_dead(&self.world, *v) { continue; }
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
            !villager::is_dead(world, m.invited[i]) && m.paths[i].as_ref().map(|p| p.is_empty()).unwrap_or(false)
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
                    else { self.resolve_meeting(&m.invited) };
                self.log.push(format!("t{}: {}", self.tick, msg));
            }
        }
    }

    /// When the player and a villager, or two villagers, come within
    /// CHAT_RANGE of each other, one of them passes a remark - at most once
    /// a CHAT_COOLDOWN days a pair - and the pair get on a little better
    fn chat_in_passing(&mut self) {
        let mut people: Vec<Entity> = self.villagers.iter().copied().filter(|v| !villager::is_dead(&self.world, *v)).collect();
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
            // the player is listed last, so a villager always does the talking
            let n = self.last_chat.len() + (a as usize) + tick as usize;
            self.say_in_passing(a, passing_remark(n), CHAT_BUBBLE_SECS);
            relationships::record_interaction(&mut self.world, if b == self.player { b } else { a }, relationships::Interaction {
                other: if b == self.player { a } else { b }, kind: relationships::InteractionKind::Conversation, intensity: CHAT_INTENSITY,
            });
            self.last_chat.retain(|(p, _)| *p != (a, b));
            self.last_chat.push(((a, b), tick));
        }
        self.near = near;
    }

    /// The mouse held on the minimap: look at that part of the map.
    /// Returns whether (mx, my) was on the minimap.
    fn look_at_minimap(&mut self, mx: i32, my: i32) -> bool {
        match minimap_tile(mx as f32, my as f32, self.map.width as f32, self.map.height as f32) {
            Some((tx, ty)) => { (self.cam_x, self.cam_y) = camera_on(tx, ty); true }
            None => false,
        }
    }

    /// A left click on the map: rivals rebuff or greet you, villagers get selected
    fn click_map(&mut self, mx: i32, my: i32) {
        let (tick, cam_x, cam_y, zoom) = (self.tick, self.cam_x, self.cam_y, self.zoom);
        // click on a rival villager -> they rebuff you (foreign village is wary)
        for (i, rv) in self.rivals.clone().into_iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(rv) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                let d = ((sx as f32 - mx as f32).powi(2) + (sy as f32 - my as f32).powi(2)).sqrt();
                if mx < MAP_W as i32 && d < 24.0 {
                    let p_inf = self.world.get_component::<PlayerCharacter>(self.player).unwrap().influence;
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
        let mut best = self.selected_target;
        let mut best_d = 9999.0f32;
        for (i, v) in self.villagers.iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(*v) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                let d = ((sx as f32 - mx as f32).powi(2) + (sy as f32 - my as f32).powi(2)).sqrt();
                if on_map && d < 30.0 && d < best_d { best = i; best_d = d; }
            }
        }
        if best_d < 30.0 { self.picked = true; }
        self.selected_target = best;
    }
}

#[cfg(test)]
mod stay_home_tests {
    use super::*;

    #[test]
    fn test_villagers_stay_in_their_own_village() {
        // on main, Aldric was at x 27.5 by day 1050 - the rivals' village -
        // and stayed there, eating from their store
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        let home = *game.world.get_component::<Pos>(game.market_food).unwrap();
        let mut worst: f32 = 0.0;
        let mut last = 0;
        while game.tick < 1500 {
            game.update();
            if game.tick == last { continue; }
            last = game.tick;
            for v in &game.villagers {
                let p = game.world.get_component::<Pos>(*v).unwrap();
                worst = worst.max(((p.x - home.x).powi(2) + (p.y - home.y).powi(2)).sqrt());
            }
        }
        assert!(worst < VILLAGE_RADIUS + 3.0, "a villager strayed {:.1} tiles from the village store", worst);
    }
}

#[cfg(test)]
mod rival_tests {
    use super::*;

    #[test]
    fn test_rivals_stay_in_their_own_village() {
        let art = Art::default(); // no sprites needed to run the game, only to draw it
        let mut game = Game::new(&art);
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
        let art = Art::default();
        let mut game = Game::new(&art);
        while game.tick < FLU_DAY + 5 { game.update(); }
        assert!(is_sick(&game.world, game.villagers[2]), "Bram should have the flu just after it arrives");
        assert!(game.log.iter().any(|l| l.contains("Bram has the flu")), "the log should say who fell ill");
        while game.tick < 600 { game.update(); }
        assert!(game.villagers.iter().all(|v| !is_sick(&game.world, *v)), "the flu should have run its course by day 600");
        assert!(game.log.iter().any(|l| l.contains("recovered from the flu")), "the log should say who recovered");
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
        let art = Art::default();
        let game = Game::new(&art);
        assert_eq!(game.buttons.len(), ACTION_KEYS.len(), "every action button needs a key");
        for (b, (key, ch)) in game.buttons.iter().zip(ACTION_KEYS.iter()) {
            assert_eq!(b.key, *ch, "{} shows key {} but is bound to {:?}", b.label, b.key, key);
            assert_eq!(format!("{:?}", key), ch.to_string(), "{}: hint {} doesn't match key {:?}", b.label, ch, key);
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
        let art = Art::default();
        let mut game = Game::new(&art);
        for i in 0..game.villagers.len() {
            let v = game.villagers[i];
            let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
            { let p = game.world.get_component_mut::<Pos>(game.player).unwrap(); p.x = vx + 0.5; p.y = vy; }
            game.selected_target = i;
            for _ in 0..8 { game.act(0); } // talk until there's nothing new to hear
            for _ in 0..40 {
                if game.world.get_component::<Organization>(game.org).unwrap().members.contains(&v) { break; }
                game.act(3);
                finish_meeting(&mut game);
            }
            assert!(game.world.get_component::<Organization>(game.org).unwrap().members.contains(&v),
                "{} never joined at a meeting", NAMES[i]);
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
        let art = Art::default();
        let mut game = Game::new(&art);
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: 8.0, y: 20.0 };
        assert_eq!(game.action_blocker(4), None);
        game.act(4);
        let w = game.works.last().expect("building should start works");
        let road = w.road.clone();
        assert!(road.len() >= 2, "a house out here needs a road");
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
        let art = Art::default();
        let mut game = Game::new(&art);
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
        let art = Art::default();
        let mut game = Game::new(&art);
        let v = game.villagers[2];
        let (vx, vy) = { let p = game.world.get_component::<Pos>(v).unwrap(); (p.x, p.y) };
        let before = trust(&game, game.player, v);
        *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x: vx + 1.0, y: vy };
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
        let art = Art::default();
        let game = Game::new(&art);
        let mut river = 0;
        for y in 0..33 { for x in 0..33 {
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
mod depth_tests {
    use super::*;

    #[test]
    fn test_hiding_someone_doesnt_flip_the_wall_piece_in_front() {
        // someone standing just inside the town wall is hidden by it - but the
        // piece hiding them must not paint over the piece in FRONT of it (the
        // occluder re-draw used to flip the seam back to front, leaving a
        // glitched spot on the wall)
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        // a spot where the last wall piece and the front corner tower
        // overlap: the tower stands in front, so it owns the seam (the map
        // is perturbed per seed - find the corner, don't hardcode it)
        let ((cx, cy), _) = game.map.walls.iter()
            .filter(|(_, p)| *p == WallPiece::Tower)
            .max_by_key(|((x, y), _)| x + y).copied().expect("a corner tower");
        let (sx, sy) = iso_to_screen_cam(cx as f32, cy as f32, game.cam_x, game.cam_y, game.zoom);
        let seam = |buf: &Buffer| { let i = ((sy - 30) as usize * W + (sx - 17) as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
        let render_at = |game: &mut Game, x: f32, y: f32| {
            *game.world.get_component_mut::<Pos>(game.player).unwrap() = Pos { x, y };
            game.render((-1, -1))
        };
        let clear = render_at(&mut game, 16.0, 16.0);
        let hidden = render_at(&mut game, 21.0, 23.0);
        assert_eq!(seam(&clear), seam(&hidden), "hiding the player flipped the tower in front of the wall piece");
    }

    #[test]
    fn test_walking_behind_a_house_hides_the_player() {
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        let (bx, by, _) = game.buildings[0];
        // the player's yellow ring is drawn under them: behind the house it's covered
        let ring_px = |game: &Game| {
            let buf = game.render((-1, -1));
            let (px, py) = game.player_pos();
            let (sx, sy) = iso_to_screen_cam(px, py, game.cam_x, game.cam_y, game.zoom);
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
        let art = Art::default();
        let mut game = Game::new(&art);
        let (bx, by, _) = game.buildings[0];
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
        let art = Art::default();
        let mut game = Game::new(&art);
        put_player(&mut game, 16.0, 16.0);
        for _ in 0..600 { game.walk(1.0, 1.0); }
        let (px, py) = game.player_pos();
        assert!(!matches!(game.map.get(px.round() as i32, py.round() as i32), Some(Tile::Water) | Some(Tile::River) | None),
            "the player walked into the water at ({:.1},{:.1})", px, py);
    }

    #[test]
    fn test_roads_are_quicker() {
        let art = Art::default();
        let mut game = Game::new(&art);
        let road = (0..33).flat_map(|y| (0..33).map(move |x| (x, y)))
            .find(|&(x, y)| game.map.get(x, y) == Some(Tile::Road) && game.map.get(x + 1, y) == Some(Tile::Road)).unwrap();
        let grass = (0..33).flat_map(|y| (0..33).map(move |x| (x, y)))
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
        let art = Art::default();
        let mut game = Game::new(&art);
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
        assert!(line.contains(NAMES[2]) && !line.contains("Can't"), "{}", line);
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
        let art = Art::default();
        let mut game = Game::new(&art);
        while game.tick < 3 { game.update(); }
        let who = far_villager(&game);
        game.selected_target = who;
        game.picked = true; // clicked on them
        let v = game.villagers[who];
        let talks = |g: &Game| g.world.get_component::<TalkCount>(v).map(|c| c.value).unwrap_or(0);
        let before = talks(&game);
        let start = game.dist_to(who);

        game.act(0);
        assert!(game.log.last().unwrap().contains(&format!("You walk over to {}", NAMES[who])), "{}", game.log.last().unwrap());
        assert_eq!(talks(&game), before, "no talking from across the village");

        for _ in 0..(30 * FPS) {
            if game.approach.is_none() { break; }
            game.update();
        }
        assert!(game.approach.is_none(), "still walking to {} ({} tiles off, started {})", NAMES[who], game.dist_to(who), start);
        assert!(game.dist_to(who) <= TALK_DIST + 0.5, "stopped {} tiles from {}", game.dist_to(who), NAMES[who]);
        assert_eq!(talks(&game), before + 1, "talked once on arrival");
        assert!(game.log.iter().any(|l| l.contains(NAMES[who]) && l.contains(": \"")), "no talk line: {:?}", game.log);
    }

    #[test]
    fn test_recruit_on_a_picked_villager_walks_over_then_recruits() {
        let art = Art::default();
        let mut game = Game::new(&art);
        while game.tick < 3 { game.update(); }
        let who = far_villager(&game);
        game.selected_target = who;
        game.picked = true;
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().influence = 10.0;
        assert_eq!(game.disabled_reason(1), None, "Recruit walks over, so distance doesn't grey it out");

        game.act(1);
        assert!(game.log.last().unwrap().contains(&format!("You walk over to {}", NAMES[who])), "{}", game.log.last().unwrap());
        assert!(game.approach.as_ref().map(|a| a.action) == Some(1));

        for _ in 0..(30 * FPS) {
            if game.approach.is_none() { break; }
            game.update();
        }
        assert!(game.approach.is_none(), "still walking to {}", NAMES[who]);
        assert!(game.dist_to(who) <= TALK_DIST + 0.5, "stopped {} tiles from {}", game.dist_to(who), NAMES[who]);
        // the recruit attempt happened on arrival (joined or turned the player down)
        let line = game.log.last().unwrap();
        assert!(line.contains(NAMES[who]) && !line.contains("walk over") && !line.contains("Can't"), "{}", line);
    }

    #[test]
    fn test_talk_is_never_too_far_away() {
        let art = Art::default();
        let mut game = Game::new(&art);
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
        let art = Art::default();
        let mut game = Game::new(&art);
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
        assert_eq!(log_colour("t120: Bram has the flu"), illness);
        assert_eq!(log_colour("t200: it's raining"), illness);
    }
}

#[cfg(test)]
mod log_priority_tests {
    use super::*;

    #[test]
    fn test_old_gossip_does_not_push_events_out_of_the_log() {
        let mut log: Vec<String> = vec!["t1: The Merchant Guild formed!".into(), "t120: Bram has the flu".into()];
        for t in (160..=600).step_by(40) { log.push(format!("t{}: Aldric tells Bram - word spreads", t)); }
        let shown = visible_log(&log);
        assert!(shown.iter().any(|e| e.contains("has the flu")), "the flu should still be in view: {:?}", shown);
        assert!(shown.iter().any(|e| e.contains("Guild formed")), "the guild should still be in view");
        assert_eq!(shown.iter().filter(|e| is_gossip(e)).count(), LOG_GOSSIP_SHOWN);
        assert_eq!(shown[0], "t600: Aldric tells Bram - word spreads", "newest first");
    }

    #[test]
    fn test_weather_changes_do_not_push_events_out_of_the_log() {
        let mut log: Vec<String> = vec!["t120: Bram has the flu".into()];
        for t in (130..=600).step_by(5) {
            log.push(format!("t{}: drizzle begins", t));
            log.push(format!("t{}: the rain stops - a rainbow!", t + 2));
        }
        let shown = visible_log(&log);
        assert!(shown.iter().any(|e| e.contains("has the flu")), "the flu should still be in view: {:?}", shown);
        assert_eq!(shown.iter().filter(|e| is_weather_note(e)).count(), LOG_WEATHER_SHOWN);
    }

    #[test]
    fn test_flu_from_a_real_game_stays_in_the_log_by_day_600() {
        let art = Art::default();
        let mut game = Game::new(&art);
        while game.tick < 600 { game.update(); }
        let shown: Vec<&String> = visible_log(&game.log).into_iter().take(8).collect();
        assert!(shown.iter().any(|e| e.contains("flu")), "by day 600 the flu has scrolled away: {:?}", shown);
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

        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        while game.tick < 30 { game.update(); }
        let buf = game.render((-1, -1));
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
        assert_eq!(log_parts("t120: Bram has the flu"), (Some("day 120".into()), "Bram has the flu"));
        assert_eq!(log_parts("Welcome to the village."), (None, "Welcome to the village."));
        assert_eq!(log_parts("tea: not a day"), (None, "tea: not a day"));
    }

    #[test]
    fn test_the_log_draws_the_day_dim_and_the_message_in_colour() {
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        game.log.push("t5: The Merchant Guild formed!".into());
        let buf = game.render((-1, -1));
        let row = |x0: i32, x1: i32, rgb: (u8, u8, u8)| (468..468 + GLYPH_H).any(|y| (x0..x1).any(|x| {
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        let (mut last, mut run, mut worst) = (0, [0u64; NAMES.len()], 0u64);
        while game.tick < 720 {
            game.update();
            if game.tick == last { continue; }
            last = game.tick;
            for (k, &v) in game.villagers.iter().enumerate() {
                // the dead keep their last hunger (zero, if they starved): this
                // test is about the living being able to reach the store
                if game.world.get_component::<Health>(v).is_none() { run[k] = 0; continue; }
                let empty = game.world.get_component::<Hunger>(v).map(|h| h.value <= 0.0).unwrap_or(false);
                run[k] = if empty { run[k] + 1 } else { 0 };
                worst = worst.max(run[k]);
            }
        }
        let servings = game.world.get_component::<villager::FoodSource>(game.market_food).unwrap().servings;
        assert!(servings > 0, "the store ran out - this test is about reach, not supply");
        assert!(worst <= 5, "a villager went {} days at zero hunger with {} servings in the store", worst, servings);
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        game.help_open = true;
        let buf = game.render((-1, -1));
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 5.0;
        assert_eq!(game.action_blocker(2).as_deref(), Some("need 10 gold"));
        game.act(2);
        assert!(game.log.last().unwrap().ends_with("Can't invest: need 10 gold."), "{}", game.log.last().unwrap());
        game.world.get_component_mut::<PlayerCharacter>(game.player).unwrap().resources = 10.0;
        assert_eq!(game.action_blocker(2), None);
    }

    #[test]
    fn test_recruit_is_greyed_out_for_a_member() {
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
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
        let buf = game.render((-1, -1));
        let b = &game.buttons[1];
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        game.paused = true;
        let buf = game.render((-1, -1));
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        while game.tick < 60 { game.update(); }
        let corvin = game.villagers[3];
        game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 3 { game.update(); }
        game.selected_target = 3;
        let buf = game.render((-1, -1));
        let (x0, y0) = (UI_X as i32 + 6, 312);
        let mut stone = 0;
        let fig: Vec<(u8, u8, u8)> = art.villager_sprites[3].pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        // one villager beside the player, one far across the village
        let (px, py) = game.player_pos();
        let (near, far) = (game.villagers[0], game.villagers[4]);
        *game.world.get_component_mut::<Pos>(near).unwrap() = Pos { x: px + 1.5, y: py };
        *game.world.get_component_mut::<Pos>(far).unwrap() = Pos { x: px + 6.0, y: py + 6.0 }; // 8.5 tiles: just out of reach, still on screen
        game.selected_target = 2;
        let buf = game.render((-1, -1));
        let name_colour = |v: Entity, i: usize| {
            let p = *game.world.get_component::<Pos>(v).unwrap();
            let (sx, sy) = iso_to_screen_cam(p.x, p.y, game.cam_x, game.cam_y, game.zoom);
            let (lx, ly) = name_label_pos(sx, sprite_label_anchor(&art.villager_sprites[i], sy, game.zoom), NAMES[i]);
            // labels may be nudged up a line or two; look over that band
            let (mut bright, mut dim) = (0, 0);
            for y in (ly - 40).max(0)..ly + GLYPH_H { for x in lx.max(0)..(lx + NAMES[i].len() as i32 * GLYPH_W).min(MAP_W as i32) {
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
        assert!(b > 10 && d == 0, "{} is in reach but not bright ({} bright, {} dim)", NAMES[0], b, d);
        let (_, d) = name_colour(far, 4);
        assert!(d > 10, "{} is out of reach but not dimmed", NAMES[4]);
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
        for &(tx, ty, cam_x, cam_y, zoom) in &[(10.0f32, 16.0f32, 0, 0, 1.0f32), (28.0, 13.0, -120, 40, 1.0), (5.0, 2.0, 30, -10, 1.5)] {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let a = screen_to_minimap(sx as f32, sy as f32, cam_x, cam_y, zoom);
            let b = tile_to_minimap(tx, ty);
            assert!((a.0 - b.0).abs() < 1.0 && (a.1 - b.1).abs() < 1.0, "{:?} vs {:?}", a, b);
        }
    }

    #[test]
    fn test_whole_map_fits_in_the_minimap_box() {
        for &(tx, ty) in &[(0.0, 0.0), (33.0, 0.0), (0.0, 33.0), (33.0, 33.0)] {
            let (x, y) = tile_to_minimap(tx, ty);
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
                        buf.set(x as i32, y as i32, r, g, b);
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
fn render_title() -> Buffer {
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
fn draw_win_banner(buf: &mut Buffer) {
    let cx = MAP_W as i32 / 2;
    buf.rect(cx - BANNER_W / 2, 240, BANNER_W, 100, 20, 20, 35);
    buf.rect(cx - BANNER_W / 2, 240, BANNER_W, 2, 255, 215, 0);
    draw_centered_at(buf, cx, 265, "RECOGNIZED LEADER", 2, (255, 215, 0));
    draw_centered_at(buf, cx, 302, "the village is yours", 1, (200, 200, 200));
}

/// The game-over summary screen after a win
fn render_summary(game: &Game) -> Buffer {
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
fn present(window: &mut Window, buf: &Buffer) {
    let mut fb = vec![0u32; W * H];
    for i in 0..W * H {
        let idx = i * 4;
        if idx + 2 < buf.pixels.len() {
            fb[i] = rgb(buf.pixels[idx], buf.pixels[idx+1], buf.pixels[idx+2]);
        }
    }
    window.update_with_buffer(&fb, W, H).unwrap();
}

fn main() {
    // headless verification: --dump-frame N renders tick N to BMP and exits
    let args: Vec<String> = std::env::args().collect();
    let dump_at: Option<u64> = args.iter().position(|a| a == "--dump-frame")
        .and_then(|i| args.get(i+1)).and_then(|v| v.parse().ok());
    // optional --out PATH for the dump (default evidence/frame-check-N.bmp)
    let out: Option<String> = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).cloned();
    // optional --zoom Z for the dump (default 1)
    let zoom: f32 = args.iter().position(|a| a == "--zoom").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(1.0);
    // optional --cam X,Y to pan the dump
    let cam: Option<(i32, i32)> = args.iter().position(|a| a == "--cam").and_then(|i| args.get(i + 1))
        .and_then(|v| v.split_once(',')).and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
    // optional --seed N: which perturbed map to draw (default: a fresh one
    // each run; GAME2_SEED does the same for launches)
    let seed = args.iter().position(|a| a == "--seed").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or_else(run_seed);
    let art = Art::load_seeded("assets", seed);
    if let Some(target) = dump_at {
        // --weather <drizzle|light|showers|rain|heavy|flurries|snow|blizzard|rainbow|clear>
        let weather = args.iter().position(|a| a == "--weather").and_then(|i| args.get(i + 1)).cloned();
        run_headless(&art, target, out, zoom, cam, weather.as_deref(), seed);
        return;
    }

    let mut window = Window::new("The Wanderer - Medieval Village", W, H, WindowOptions::default())
        .expect("window failed");
    let mut game = Game::new_seeded(&art, seed);
    let mut started = false; // title screen until Enter
    let mut mouse_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // TITLE SCREEN until Enter
        if !started {
            present(&mut window, &render_title());
            if window.is_key_down(Key::Enter) { started = true; }
            continue;
        }

        game.update();

        // --- RENDER into Buffer ---
        let (mxf, myf) = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let (mx, my) = (mxf as i32, myf as i32);
        let mut buf = game.render((mx, my));

        // --- INPUT ---
        // actions fire once per click / key press, not every frame the input is held
        let mouse_down = window.get_mouse_down(minifb::MouseButton::Left);
        let clicked = mouse_down && !mouse_was_down;
        mouse_was_down = mouse_down;
        let mut action: Option<usize> = None;
        if clicked {
            for (i, b) in game.buttons.iter().enumerate() {
                if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h { action = Some(i); }
            }
        }
        for (i, (k, _)) in ACTION_KEYS.iter().enumerate() {
            if window.is_key_pressed(*k, minifb::KeyRepeat::No) { action = Some(i); }
        }
        if let Some(i) = action { game.act(i); }
        // holding the mouse on the minimap moves the view there
        if !(mouse_down && game.look_at_minimap(mx, my)) && clicked { game.click_map(mx, my); }
        if window.is_key_down(Key::Key1) { game.selected_target = 0; game.picked = true; }
        if window.is_key_down(Key::Key2) { game.selected_target = 1; game.picked = true; }
        if window.is_key_down(Key::Key3) { game.selected_target = 2; game.picked = true; }
        if window.is_key_down(Key::Key4) { game.selected_target = 3; game.picked = true; }
        if window.is_key_down(Key::Key5) { game.selected_target = 4; game.picked = true; }
        if window.is_key_down(Key::Key6) { game.selected_target = 5; game.picked = true; }
        if window.is_key_down(Key::Key7) { game.selected_target = 6; game.picked = true; }
        if window.is_key_down(Key::Key8) { game.selected_target = 7; game.picked = true; }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { game.paused = !game.paused; }
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) { game.help_open = !game.help_open; }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { game.zoom = (game.zoom + 0.006).min(2.0); }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { game.zoom = (game.zoom - 0.006).max(0.5); }
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) { game.speed = if game.speed == 1 { 4 } else { 1 }; }
        // WASD walk along the grid, so the player can follow a road or a
        // row of tiles: W/S up-right and down-left, A/D up-left and down-right
        let (mut dx, mut dy) = (0.0, 0.0);
        if window.is_key_down(Key::W) { dy -= 1.0; }
        if window.is_key_down(Key::S) { dy += 1.0; }
        if window.is_key_down(Key::A) { dx -= 1.0; }
        if window.is_key_down(Key::D) { dx += 1.0; }
        if dx != 0.0 || dy != 0.0 {
            // the player is walking themselves: drop any walk over to talk
            game.approach = None;
            game.picked = false;
            game.walk(dx, dy);
        }
        // arrows pan the camera
        if window.is_key_down(Key::Left)  { game.cam_x += 2; }
        if window.is_key_down(Key::Right) { game.cam_x -= 2; }
        if window.is_key_down(Key::Up)    { game.cam_y += 2; }
        if window.is_key_down(Key::Down)  { game.cam_y -= 2; }

        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", game.frame_n)).ok(); }

        // blit buffer to framebuffer
        present(&mut window, &buf);

        let p = game.world.get_component::<PlayerCharacter>(game.player).unwrap();
        if p.influence > WIN_INFLUENCE {
            // win banner over the village, then the summary screen
            draw_win_banner(&mut buf);
            present(&mut window, &buf);
            std::thread::sleep(std::time::Duration::from_secs(3));
            present(&mut window, &render_summary(&game));
            std::thread::sleep(std::time::Duration::from_secs(5));
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1000 / FPS as u64));
    }
}


/// Headless verification: play the game (no input) until tick N, then
/// render that frame through the same Game::render the window uses and save
/// it as a BMP.
fn run_headless(art: &Art, target_tick: u64, out: Option<String>, zoom: f32, cam: Option<(i32, i32)>, weather: Option<&str>, seed: u64) {
    let mut game = Game::new_seeded(art, seed);
    game.zoom = zoom;
    if let Some(cam) = cam { (game.cam_x, game.cam_y) = cam; }
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
    let buf = game.render((-1, -1)); // cursor off-screen: no hover
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
        let art = Art::load("../assets");
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        // day 1, the flu (150), rain (200, from the weather sim) and deep winter (330)
        for day in [1u64, 150, 200, 330] {
            while game.tick < day { game.update(); }
            let buf = game.render((-1, -1));
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
    use game2_core::game::relationships::RelKind;

    #[test]
    fn test_organize_holds_a_house_meeting_with_nearby_villagers() {
        let art = Art::default();
        let mut game = Game::new(&art);
        let (eliza, bram, corvin) = (game.villagers[1], game.villagers[2], game.villagers[3]);
        // the player stands among Eliza and Bram; Corvin is far away
        game.world.get_component_mut::<Pos>(game.player).unwrap().x = 16.0;
        game.world.get_component_mut::<Pos>(game.player).unwrap().y = 16.0;
        let (aldric, dagny) = (game.villagers[0], game.villagers[4]);
        for (v, x, y) in [(eliza, 17.0, 16.0), (bram, 15.0, 17.0), (corvin, 2.0, 2.0), (aldric, 2.0, 4.0), (dagny, 4.0, 2.0),
                          (game.villagers[5], 2.0, 6.0), (game.villagers[6], 6.0, 2.0), (game.villagers[7], 3.0, 5.0)] {
            let p = game.world.get_component_mut::<Pos>(v).unwrap();
            p.x = x; p.y = y;
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
        assert!(relationships::find_relationship(&game.world, game.player, corvin).is_none(), "Corvin wasn't there");
    }

    #[test]
    fn test_a_meeting_takes_time_people_gather_then_talk_then_it_ends() {
        let art = Art::default();
        let mut game = Game::new(&art);
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
        let art = Art::default();
        let mut game = Game::new(&art);
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
        let art = Art::default();
        let mut game = Game::new(&art);
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        while game.tick < 60 { game.update(); }
        let corvin = game.villagers[3];
        // a quiet spot away from the crowd, so no label or figure overlaps the grave
        *game.world.get_component_mut::<Pos>(corvin).unwrap() = Pos { x: 21.0, y: 21.5 };
        game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 2 { game.update(); }
        assert!(game.died_on[3].is_some(), "the day of death should be recorded");

        let buf = game.render((-1, -1));
        let p = *game.world.get_component::<Pos>(corvin).unwrap();
        let (sx, sy) = iso_to_screen_cam(p.x, p.y, game.cam_x, game.cam_y, game.zoom);
        let px = |x: i32, y: i32| { let i = (y as usize * W + x as usize) * 4; (buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]) };
        // the stone is there, beside the carved cross down its middle
        assert_eq!(px(sx - 3, sy - 4), GRAVE_STONE, "no gravestone where Corvin fell");
        // and nothing is standing on it: the space where a living head would be
        // shows no sprite colours from Corvin's figure
        let fig = &art.villager_sprites[3];
        let figure_colours: Vec<(u8, u8, u8)> = fig.pixels.chunks(4).filter(|c| c[3] > 128).map(|c| (c[0], c[1], c[2])).collect();
        let head = head_y(sy, game.zoom) + 4;
        assert!(!figure_colours.contains(&px(sx, head)), "Corvin is still drawn standing over his grave");
    }
}

#[cfg(test)]
mod minimap_dead_tests {
    use super::*;

    #[test]
    fn test_the_dead_show_as_graves_on_the_minimap() {
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        while game.tick < 60 { game.update(); }
        let corvin = game.villagers[3];
        game.world.get_component_mut::<Health>(corvin).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 2 { game.update(); }
        let buf = game.render((-1, -1));
        let px = |v: Entity| {
            let p = *game.world.get_component::<Pos>(v).unwrap();
            let (x, y) = tile_to_minimap(p.x, p.y);
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        while game.tick < 60 { game.update(); }
        let aldric = game.villagers[0];
        // Aldric is close to Eliza, so she's who takes his dog in
        let rel = game.world.create_entity();
        game.world.add_component(rel, Relationship {
            from: aldric, other: game.villagers[1], kind: game2_core::game::relationships::RelKind::Friend, strength: 0.6, interactions: 5,
        });
        game.world.get_component_mut::<Health>(aldric).unwrap().condition = 0.0;
        let t = game.tick;
        while game.tick < t + 3 { game.update(); }
        let owner = game.world.get_component::<Pet>(game.pets[0].0).unwrap().owner.expect("someone should take the dog in");
        assert_ne!(owner, aldric, "the dog should be rehomed");

        // out in the open by the market, so nothing in front hides the dog
        *game.world.get_component_mut::<Pos>(owner).unwrap() = Pos { x: 16.0, y: 17.0 };
        let buf = game.render((-1, -1));
        let dog_at = |v: Entity| {
            let p = *game.world.get_component::<Pos>(v).unwrap();
            let (dx, dy) = iso_to_screen_cam(p.x + 0.8, p.y + 0.5, game.cam_x, game.cam_y, game.zoom);
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
        let art = Art::default();
        let mut game = Game::new(&art);
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
        let art = Art::load("../assets");
        let mut game = Game::new(&art);
        while game.tick < 30 { game.update(); }
        let buf = game.render((-1, -1));
        // just past the end of the text, still on the strip
        let hud = hud_text(Season::Summer, game.tick, 0, game.precip.map(|k| k.label()), false, 1);
        let x = 10 + text_width(&hud, 1) - 20;
        let i = ((GLYPH_H + 10) as usize * W + x as usize) * 4;
        assert_eq!((buf.pixels[i], buf.pixels[i + 1], buf.pixels[i + 2]), (20, 20, 32), "no backing strip under the HUD");
    }
}
