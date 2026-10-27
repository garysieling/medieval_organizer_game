//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::ecs::{Entity, Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::{draw_text, draw_text_outlined, draw_text_scaled};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest, can_recruit, recruit_influence_needed};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::ui::isometric::render_terrain;
use game2_core::sim::climate::{self, SeasonSystem, Season};
use game2_core::sim::illness::{Health, Disease, Stage, IllnessSystem};
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::community::{Citizen, Organization};
use game2_core::game::traits::{self, Personality, personality_from_seed};
use game2_core::game::relationships::{self, Relationship};

const W: usize = 960;
const H: usize = 640;
const MAP_W: usize = 640;
const UI_X: usize = 660;
const TILE_W: i32 = 28;
const TILE_H: i32 = 14;
/// Influence at which the village recognizes the player as its leader (the win)
const WIN_INFLUENCE: f32 = 2.0;

fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }

fn draw_rect(buf: &mut Buffer, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
    buf.rect(x, y, w, h, r, g, b);
}

/// Convert world tile coords to screen isometric position
fn iso_to_screen_cam(tx: f32, ty: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (i32, i32) {
    let tw = TILE_W as f32 * zoom;
    let th = TILE_H as f32 * zoom;
    let sx = MAP_W as i32 / 2 + ((tx - ty) * tw) as i32 + cam_x;
    let sy = -60 + ((tx + ty) * th) as i32 + cam_y;
    (sx, sy)
}

/// Top of a character's head relative to its ground point. Characters are
/// 32px frames drawn 2x from (sx-32, sy-60), but the figure only fills the
/// frame's bottom rows (18-31), so the head is at about sy-24.
const SPRITE_HEAD: i32 = -24;
/// Font glyph width and height (ui::font)
const GLYPH_W: i32 = 6;
const GLYPH_H: i32 = 7;

/// Where to draw a character's name: centred over them, just above the head
fn name_label_pos(sx: i32, sy: i32, text: &str) -> (i32, i32) {
    (sx - text.len() as i32 * GLYPH_W / 2, sy + SPRITE_HEAD - GLYPH_H - 4)
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

/// Colour for a log entry: deaths red, guild events gold, joins green,
/// weather/illness cyan, idle gossip dimmed so real events stand out
fn log_colour(entry: &str) -> (u8, u8, u8) {
    if entry.contains("died") || entry.contains("dead") { (255, 100, 100) }
    else if entry.contains(" tells ") { (140, 140, 160) } // gossip, even when it mentions the guild
    else if entry.contains("Guild") || entry.contains("guild") { (255, 215, 0) }
    else if entry.contains("joined") { (120, 255, 120) }
    else if entry.contains("storm") || entry.contains("rain") || entry.contains("flu") { (120, 200, 255) }
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

// Minimap: the same isometric projection as the map, shrunk to fit a box in
// the map's top-right corner (so up on the minimap is up on screen)
const MM_W: i32 = 84;
const MM_H: i32 = 46;
const MM_LEFT: i32 = MAP_W as i32 - MM_W - 6;
const MM_TOP: i32 = 6;
const MM_K: f32 = 1.25; // minimap px per tile step along the x/y diagonal

/// Minimap position of a world tile
fn tile_to_minimap(tx: f32, ty: f32) -> (f32, f32) {
    (MM_LEFT as f32 + MM_W as f32 / 2.0 + (tx - ty) * MM_K, MM_TOP as f32 + 2.0 + (tx + ty) * MM_K / 2.0)
}

/// Minimap position of a point on screen (inverse of iso_to_screen_cam,
/// then tile_to_minimap) - used to outline what the screen is showing
fn screen_to_minimap(sx: f32, sy: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (f32, f32) {
    let diff = (sx - MAP_W as f32 / 2.0 - cam_x as f32) / (TILE_W as f32 * zoom); // tx - ty
    let sum = (sy + 60.0 - cam_y as f32) / (TILE_H as f32 * zoom);                 // tx + ty
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
/// Gold a new house costs
const BUILD_COST: f32 = 30.0;
/// Tile the windmill stands on (it's animated, so drawn apart from `buildings`)
const MILL_TILE: (f32, f32) = (10.0, 9.0);
/// How far from the player (in tiles) to look for a free building site
const BUILD_SEARCH: i32 = 8;

/// Day a traveller brings the flu to the village
const FLU_DAY: u64 = 120;

/// Keyboard shortcut for each action button, in button order (Talk, Recruit,
/// Invest, Organize, Build, Wait). The input loop and the buttons' key hints
/// both come from here so they can't disagree. W/A/S/D move the player, so
/// no action may use them.
const ACTION_KEYS: [(Key, char); 6] = [
    (Key::T, 'T'), (Key::R, 'R'), (Key::I, 'I'), (Key::O, 'O'), (Key::B, 'B'), (Key::Z, 'Z'),
];
/// Keys that move the player
const MOVE_KEYS: [char; 4] = ['W', 'A', 'S', 'D'];

const NAMES: [&str; 5] = ["Aldric","Eliza","Bram","Corvin","Dagny"];
const RIVAL_NAMES: [&str; 3] = ["Wulfric","Sable","Miren"];
const TOOLTIPS: [&str; 6] = [
    "talk to selected villager",
    "recruit villager to your org",
    "spend gold for influence",
    "call a house meeting",
    "commission a new house",
    "watch the village",
];

/// Every sprite the game draws, loaded once up front
#[derive(Default)]
struct Art {
    villager_sprites: Vec<Sprite>,
    house_s: Option<Sprite>,
    mill_frames: Vec<Sprite>,
    blacksmith_s: Option<Sprite>,
    tree_s: Option<Sprite>,
    stone_s: Option<Sprite>,
    player_s: Option<Sprite>,
    tower_s: Option<Sprite>,
    wall_s: Option<Sprite>,
    church_s: Option<Sprite>,
    guildhall_s: Option<Sprite>,
    smoke_s: Option<Sprite>,
    rival_house: Option<Sprite>,
    rival_woodhouse: Option<Sprite>,
    // flat ground features: just the top face of each ground block, laid on a tile
    road_top: Option<Sprite>,
    wheat_top: Option<Sprite>,
    corn_top: Option<Sprite>,
    corn2_top: Option<Sprite>,
    river_top: Option<Sprite>,
}

/// The top face of an isometric ground block (the pack draws roads, fields
/// and rivers as full raised blocks sized for a much bigger tile grid); laid
/// flat on a map tile it reads as a road or field instead of a tower
fn top_face(block: &Option<Sprite>) -> Option<Sprite> {
    block.as_ref().map(|b| b.crop(0, 0, b.width, b.width / 2 + 1))
}

impl Art {
    /// Load every sprite from the asset folder at `root` ("assets" when run
    /// from the repo root, as the game is)
    fn load(root: &str) -> Art {
        let assets = Assets::new(root);

        // Load villager sprites (crop single 32x32 frame from sheets)
        let vgenders = ["m", "f", "m", "m", "f"];
        let vroles = ["peasant", "worker", "noble", "elder", "peasant"]; // Corvin=noble, others vary
        let villager_sprites: Vec<Sprite> = (0..5).map(|i| {
            let path = assets.path(&assets.villager_sprite(vroles[i], vgenders[i]));
            match Sprite::load(&path) {
                Ok(s) => s.crop(0, 0, 32, 32),
                Err(e) => { println!("sprite load {}: {}", path.display(), e); Sprite { width: 1, height: 1, pixels: vec![255,0,255,255] } }
            }
        }).collect();

        let road_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Roads/Spring/road(1).png")).ok();
        let wheat_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/wheat/wheat(1).png")).ok();
        let corn_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/corn/corn(1).png")).ok();
        let corn2_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/corn/corn(2).png")).ok();
        let river_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Rivers/Spring/river(1).png")).ok();

        // Load building sprites
        Art {
            villager_sprites,
            road_top: top_face(&road_s),
            wheat_top: top_face(&wheat_s),
            corn_top: top_face(&corn_s),
            corn2_top: top_face(&corn2_s),
            river_top: top_face(&river_s),
            house_s: Sprite::load(&assets.path(&assets.building_sprite("house", "blue"))).ok(),
            mill_frames: (1..=4).map(|i|
                Sprite::load(&assets.path(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/mill/mill({}).png", i))).unwrap()
            ).collect(),
            blacksmith_s: Sprite::load(&assets.path(&assets.building_sprite("blacksmith", "green"))).ok(),
            tree_s: Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/trees/tree(1).png")).ok(),
            stone_s: Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/stones/stone(1).png")).ok(),
            player_s: Sprite::load(&assets.path("MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png")).map(|s| s.crop(0, 0, 32, 32)).ok(),
            tower_s: Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Towers/Wood towers/castle_tower_wood(1).png")).ok(),
            wall_s: Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Castle walls/castle_wall(1).png")).ok(),
            church_s: Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Towers/Blue towers/castle_tower_blue(1).png")).ok(),
            guildhall_s: Sprite::load(&assets.path(&assets.building_sprite("house", "red"))).ok(),
            smoke_s: Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/smoke.png")).ok(),
            rival_house: Sprite::load(&assets.path(&assets.building_sprite("house", "red"))).ok(),
            rival_woodhouse: Sprite::load(&assets.path(&assets.building_sprite("woodhouse", "red"))).ok(),
        }
    }
}

/// The whole game state. The windowed game and `--dump-frame` both drive
/// it through the same update() and render(), so a headless frame is the
/// frame a player would see.
struct Game<'a> {
    art: &'a Art,
    world: World,
    sched: Scheduler,
    terrain: Terrain,
    climate: Entity,
    villagers: Vec<Entity>,
    rivals: Vec<Entity>,
    rival_homes: Vec<(f32, f32)>, // where each rival lives, across the river
    player: Entity,
    org: Entity,
    dog: Entity,
    market_food: Entity, // our village's food store (the rivals have their own)
    buildings: Vec<(f32, f32, &'a Option<Sprite>)>,
    ground: Vec<(f32, f32, &'a Option<Sprite>)>, // flat tiles: roads, fields, river
    buttons: Vec<Button>,
    built_count: usize,
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
    dialog: Option<(Entity, String, u64)>, // (speaker, text, until tick)
    frame_n: u32,
    was_sick: Vec<bool>, // per villager, to log when someone falls ill or recovers
}

impl<'a> Game<'a> {
    fn new(art: &'a Art) -> Game<'a> {
        let mut world = World::new();
        let mut sched = Scheduler::new();
        sched.add_system(villager::HungerSystem);
        sched.add_system(villager::SeekFoodSystem);
        sched.add_system(pets::PetBondSystem);
        sched.add_system(SeasonSystem);
        sched.add_system(GuildFormationSystem);
        sched.add_system(IllnessSystem); // diseases progress, spread to neighbours, and heal

        let terrain = Terrain::generate(33, 0.7, 42);
        let climate = climate::spawn_climate(&mut world);
        let market_food = villager::spawn_food_source(&mut world, 16.0, 16.0, 500);

        // buildings placed on map
        let buildings: Vec<(f32, f32, &Option<Sprite>)> = vec![
            (5.0, 5.0, &art.house_s), (12.0, 4.0, &art.house_s), (7.0, 12.0, &art.house_s),
            (15.0, 8.0, &art.blacksmith_s),
            (3.0, 14.0, &art.tree_s), (18.0, 5.0, &art.tree_s), (9.0, 16.0, &art.tree_s), (20.0, 12.0, &art.tree_s),
            (4.0, 18.0, &art.stone_s), (19.0, 16.0, &art.stone_s),
            // castle towers at the far edge (the ruler overlooks the village)
            (26.0, 6.0, &art.tower_s), (28.0, 8.0, &art.tower_s), (27.0, 4.0, &art.tower_s),
            // rival village across the river (red-roofed foreign houses)
            (28.0, 13.0, &art.rival_house), (30.0, 15.0, &art.rival_woodhouse),
            // castle walls connecting the towers
            (25.0, 5.0, &art.wall_s), (27.0, 7.0, &art.wall_s), (29.0, 9.0, &art.wall_s),
            // church (religion center) + guild hall (merchant base)
            (2.0, 8.0, &art.church_s), (17.0, 10.0, &art.guildhall_s),
        ];

        // flat ground features, drawn on the terrain under everything that stands
        let ground: Vec<(f32, f32, &Option<Sprite>)> = vec![
            // farms: wheat + corn fields near the village
            (14.0, 14.0, &art.wheat_top), (15.0, 14.0, &art.wheat_top), (14.0, 15.0, &art.wheat_top), (15.0, 15.0, &art.wheat_top),
            (16.0, 15.0, &art.corn_top), (16.0, 16.0, &art.corn_top), (17.0, 15.0, &art.corn_top),
            // market plaza (packed dirt tiles around stalls)
            (7.0, 6.0, &art.road_top), (8.0, 6.0, &art.road_top), (9.0, 6.0, &art.road_top),
            // food market stalls
            (7.0, 7.0, &art.corn2_top), (9.0, 7.0, &art.corn2_top),
            // main road through the village center
            (8.0, 8.0, &art.road_top), (9.0, 9.0, &art.road_top), (10.0, 10.0, &art.road_top), (11.0, 11.0, &art.road_top),
            (12.0, 8.0, &art.road_top), (13.0, 7.0, &art.road_top), (8.0, 12.0, &art.road_top), (9.0, 13.0, &art.road_top),
            // river along the east edge
            (24.0, 3.0, &art.river_top), (26.0, 5.0, &art.river_top), (28.0, 7.0, &art.river_top), (30.0, 9.0, &art.river_top),
        ];

        let mut villagers = vec![];
        for (i, _n) in NAMES.iter().enumerate() {
            let v = villager::spawn_villager(&mut world, [10.0,16.0,20.0,14.0,22.0][i] as f32, [16.0,20.0,14.0,10.0,18.0][i] as f32);
            world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
            world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
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

        let buttons = vec![
            Button { label: "Talk",     key: ACTION_KEYS[0].1, x: UI_X as i32 + 10, y: 60,  w: 130, h: 36, color: (60,60,120) },
            Button { label: "Recruit",  key: ACTION_KEYS[1].1, x: UI_X as i32 + 10, y: 104, w: 130, h: 36, color: (60,120,60) },
            Button { label: "Invest",   key: ACTION_KEYS[2].1, x: UI_X as i32 + 10, y: 148, w: 130, h: 36, color: (120,60,60) },
            Button { label: "Organize", key: ACTION_KEYS[3].1, x: UI_X as i32 + 10, y: 192, w: 130, h: 36, color: (120,100,40) },
            Button { label: "Build",    key: ACTION_KEYS[4].1, x: UI_X as i32 + 10, y: 236, w: 130, h: 36, color: (80,60,120) },
            Button { label: "Wait",     key: ACTION_KEYS[5].1, x: UI_X as i32 + 10, y: 280, w: 130, h: 36, color: (60,90,100) }, // not grey: grey means disabled
        ];

        Game {
            art, world, sched, terrain, climate, villagers, rivals, player, org, dog, market_food, buildings, ground, buttons,
            built_count: 0,
            zoom: 1.0,
            rival_homes,
            toasts: vec![],
            last_guild_count: 0,
            tick: 0,
            cam_x: 0,
            cam_y: 0,
            paused: false,
            speed: 1,
            selected_target: 0,
            log: vec!["Welcome to the village.".into()],
            inspector_open: true,
            help_open: false,
            dialog: None,
            was_sick: vec![false; NAMES.len()],
            frame_n: 0,
        }
    }

    fn ticks_per_frame(&self) -> u32 {
        if self.speed == 1 { 60 } else { 15 } // ~5s/day normal, ~1.2s turbo
    }

    fn player_pos(&self) -> (f32, f32) {
        self.world.get_component::<Pos>(self.player).map(|p| (p.x, p.y)).unwrap_or((10.0, 10.0))
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
                || self.ground.iter().any(|(gx, gy, _)| on(*gx, *gy))
                || self.world.query::<villager::FoodSource>().iter()
                    .filter_map(|f| self.world.get_component::<Pos>(*f))
                    .any(|p| on(p.x, p.y))
        };
        let mut best: Option<((f32, f32), i32)> = None;
        for y in (cy - BUILD_SEARCH)..=(cy + BUILD_SEARCH) {
            for x in (cx - BUILD_SEARCH)..=(cx + BUILD_SEARCH) {
                if x < 1 || y < 1 || x as usize >= self.terrain.width - 1 || y as usize >= self.terrain.height - 1 { continue; }
                let h = self.terrain.get(x as usize, y as usize);
                if !(0.15..0.75).contains(&h) || taken(x, y) { continue; }
                let d = (x - cx).abs() + (y - cy).abs();
                if d >= 2 && best.map(|(_, bd)| d < bd).unwrap_or(true) { best = Some(((x as f32, y as f32), d)); }
            }
        }
        best.map(|(site, _)| site)
    }

    /// Why action button `i` can't be used right now, if it can't
    fn disabled_reason(&self, i: usize) -> Option<String> {
        // Talk/Recruit need the selected villager alive and within reach; a
        // house meeting (Organize) just needs someone nearby to come
        let target = self.villagers[self.selected_target];
        let in_reach = self.villagers_in_reach();
        let target_alive = self.world.get_component::<Health>(target).is_some();
        let recruit_infl_ok = can_recruit(self.world.get_component::<PlayerCharacter>(self.player).unwrap().influence);
        if (i == 0 || i == 1) && !target_alive { Some("they have died".into()) }
        else if (i == 0 || i == 1) && !in_reach.contains(&target) { Some("too far away".into()) }
        else if i == 1 && !recruit_infl_ok { Some(format!("need influence {:.2}", recruit_influence_needed())) }
        else if i == 3 && in_reach.is_empty() { Some("no one nearby".into()) }
        else { None }
    }

    /// One frame of game time: the sim ticks every few frames, and between
    /// ticks villagers keep strolling
    fn update(&mut self) {
        let ticks_per_frame = self.ticks_per_frame();
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
                    let push = (3.0 - d) * 0.08;
                    let nx = dx / d * push; let ny = dy / d * push;
                    if let Some(p) = world.get_component_mut::<Pos>(villagers[i]) { p.x -= nx; p.y -= ny; }
                    if let Some(p) = world.get_component_mut::<Pos>(villagers[j]) { p.x += nx; p.y += ny; }
                }
            }
        }

        // idle villagers wander (village feels alive); direction changes daily,
        // step is per frame so they stroll ~0.6 tiles/day instead of racing to the edge
        if !self.paused {
            let step = 0.6 / ticks_per_frame as f32;
            for v in villagers.iter() {
                let idle = world.get_component::<Villager>(*v).map(|x| x.activity == Activity::Idle).unwrap_or(false);
                if idle {
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

        // death check: health <= 0 -> grave + log
        for (i, v) in villagers.iter().enumerate() {
            if let Some(h) = world.get_component::<Health>(*v) {
                if h.condition <= 0.0 {
                    toasts.push((format!("{} has died.", NAMES[i]), tick + 60));
                    log.push(format!("t{}: {} died.", tick, NAMES[i]));
                    world.remove_component::<Health>(*v);
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
            let a = (tick as usize / 40) % 5;
            let b = (tick as usize / 40 + 2) % 5;
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
        if matches!(&self.dialog, Some((_, _, until)) if tick >= *until) { self.dialog = None; }
        toasts.retain(|(_, until)| tick < *until);
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

        let mut buf = Buffer::new(W, H);
        buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);

        // terrain rendered in the SAME projection as entities (unified map)
        let season = world.get_component::<climate::Climate>(self.climate).unwrap().season;
        let wave_phase = (tick % 40) < 20;
        let tw = (TILE_W as f32 * zoom) as i32;
        let th = (TILE_H as f32 * zoom) as i32;
        for ty in 0..self.terrain.height {
            for tx in 0..self.terrain.width {
                let h = self.terrain.get(tx, ty);
                let (r, g, mut b) = season_colour(h, season);
                if h < 0.15 && wave_phase { b = b.saturating_add(15); } // water shimmer
                let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, cam_x, cam_y, zoom);
                buf.diamond(sx, sy, tw, th, r, g, b);
                buf.diamond(sx, sy + th, tw, 1, r / 2, g / 2, b / 2);
            }
        }

        // flat ground tiles (roads, fields, river), scaled onto the tile diamond
        for (gx, gy, sprite) in self.ground.iter() {
            if let Some(s) = sprite {
                let (sx, sy) = iso_to_screen_cam(*gx, *gy, cam_x, cam_y, zoom);
                s.scaled((2 * tw) as usize, (2 * th) as usize).blit(&mut buf, sx - tw, sy - th);
            }
        }

        // buildings on map (sorted back-to-front by tile y)
        let mut b_sorted: Vec<_> = self.buildings.iter().collect();
        b_sorted.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());
        for (bx, by, sprite) in b_sorted {
            if let Some(s) = sprite {
                let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y, zoom);
                s.blit_bottom(&mut buf, sx, sy + (TILE_H as f32 * zoom) as i32);
            }
        }

        // chimney smoke rises from houses
        if let Some(sm) = &art.smoke_s {
            for (i, (bx, by, _)) in self.buildings.iter().enumerate().take(3) {
                let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y, zoom);
                let drift = ((tick + i as u64 * 7) % 30) as i32;
                sm.blit(&mut buf, sx - 6, sy - 40 - drift);
            }
        }

        // windmill: animated blades (frame cycles every 10 ticks)
        let mill_frame = &art.mill_frames[((tick / 10) % 4) as usize];
        let (msx, msy) = iso_to_screen_cam(MILL_TILE.0, MILL_TILE.1, cam_x, cam_y, zoom);
        mill_frame.blit_center(&mut buf, msx, msy);

        // market food stock: shown in the HUD (top-left), not on the market
        // tile, where villagers always gather to eat and covered it up
        let food_stock = world.get_component::<villager::FoodSource>(self.market_food)
            .map(|f| f.servings).unwrap_or(0);


        // villagers as sprites (sorted by y for depth)
        let mut v_sorted: Vec<(usize, f32, f32)> = villagers.iter().enumerate()
            .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (i, p.x, p.y)))
            .collect();
        v_sorted.sort_by(|a, b| (a.1 + a.2).partial_cmp(&(b.1 + b.2)).unwrap());
        // name labels are drawn after every sprite (so no sprite covers one)
        // and de-overlapped: (text, x, y, colour, hungry, selected)
        let mut labels: Vec<(&str, i32, i32, (u8, u8, u8), bool, bool)> = vec![];
        for (i, wx, wy) in v_sorted {
            let (sx, sy) = iso_to_screen_cam(wx, wy, cam_x, cam_y, zoom);
            art.villager_sprites[i].blit_scaled(&mut buf, sx - 32, sy - 60, 2);
            let (lx, ly) = name_label_pos(sx, sy, NAMES[i]);
            // grave if dead (no Health component)
            if world.get_component::<Health>(villagers[i]).is_none() {
                buf.rect(sx - 4, sy - 10, 8, 10, 120, 120, 130); // gravestone
                draw_text_outlined(&mut buf, sx - 6, sy - 20, "RIP", 200, 200, 210);
                continue;
            }
            let hungry = world.get_component::<Hunger>(villagers[i]).map(|h| h.value < 40.0).unwrap_or(false);
            labels.push((NAMES[i], lx, ly, (255, 255, 255), hungry, i == selected_target));
            // sick: a green cross on the other side of the head
            if is_sick(world, villagers[i]) {
                draw_text_outlined(&mut buf, sx - 16, sy + SPRITE_HEAD, "+", 140, 230, 110);
            }
            // activity marker beside the head
            if let Some(vil) = world.get_component::<Villager>(villagers[i]) {
                match vil.activity {
                    Activity::SeekingFood => { draw_text_outlined(&mut buf, sx + 10, sy + SPRITE_HEAD, "o", 255, 200, 100); }
                    Activity::Idle => {},
                    _ => { draw_text_outlined(&mut buf, sx + 10, sy + SPRITE_HEAD, "*", 180, 180, 255); }
                }
            }
        }

        // dog follows its owner's position
        if let Some(pet) = world.get_component::<Pet>(self.dog) {
            if let Some(owner_p) = world.get_component::<Pos>(villagers[0]) {
                let (dx, dy) = iso_to_screen_cam(owner_p.x + 0.8, owner_p.y + 0.5, cam_x, cam_y, zoom);
                buf.rect(dx - 3, dy - 6, 6, 4, 139, 90, 43); // brown dog body
                buf.rect(dx - 4, dy - 9, 3, 3, 139, 90, 43); // head
                if pet.bond_strength > 0.5 {
                    labels.push(("<3", dx - 6, dy - 16, (255, 120, 120), false, false)); // placed with the names
                }
            }
        }

        // rival villagers (foreign settlement)
        for (i, rv) in self.rivals.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*rv) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                // rival sprite tinted different
                if let Some(rs) = &art.player_s {
                    rs.blit_scaled(&mut buf, sx - 32, sy - 60, 2);
                }
                let (lx, ly) = name_label_pos(sx, sy, RIVAL_NAMES[i]);
                labels.push((RIVAL_NAMES[i], lx, ly, (255, 140, 140), false, false));
            }
        }

        // player sprite at their actual position
        let player_pos = self.player_pos();
        let (px, py) = iso_to_screen_cam(player_pos.0, player_pos.1, cam_x, cam_y, zoom);
        if let Some(ps) = &art.player_s {
            ps.blit_scaled(&mut buf, px - 32, py - 60, 2);
            // gold ring under player
            buf.rect(px - 12, py - 4, 24, 3, 255, 215, 0);
        } else {
            draw_rect(&mut buf, px - 4, py - 8, 8, 8, 255, 255, 255);
        }
        let (lx, ly) = name_label_pos(px, py, "YOU");
        labels.push(("YOU", lx, ly, (255, 255, 0), false, false));

        // all name labels, on top of every sprite, nudged apart where they'd overlap
        let boxes: Vec<(i32, i32, i32)> = labels.iter()
            .map(|(t, x, y, _, hungry, _)| (*x, *y, t.len() as i32 * GLYPH_W + if *hungry { GLYPH_W + 2 } else { 0 }))
            .collect();
        for ((text, x, _, (r, g, b), hungry, selected), y) in labels.iter().zip(place_labels(&boxes)) {
            draw_text_outlined(&mut buf, *x, y, text, *r, *g, *b);
            // hunger indicator, right after the name
            if *hungry { draw_text_outlined(&mut buf, x + text.len() as i32 * GLYPH_W + 2, y, "!", 255, 100, 100); }
            // selection: underline the selected villager's name
            if *selected { draw_rect(&mut buf, x - 1, y + GLYPH_H + 1, text.len() as i32 * GLYPH_W + 1, 2, 255, 255, 0); }
        }

        // --- UI PANEL ---
        draw_rect(&mut buf, UI_X as i32, 0, (W as i32 - UI_X as i32), H as i32, 32, 32, 48);
        draw_text_scaled(&mut buf, UI_X as i32 + 10, 8, "THE WANDERER", 2, 255, 220, 130);

        // status: influence bar, members, season, tick
        let p = world.get_component::<PlayerCharacter>(self.player).unwrap();
        let members = world.get_component::<Organization>(self.org).unwrap().members.len();
        let guilds = world.query::<Guild>().len();
        // influence bar runs to the win: reach WIN_INFLUENCE and the village
        // follows you; the tick marks where recruiting starts to work
        const BAR_W: i32 = 120;
        draw_text(&mut buf, UI_X as i32 + 10, 30, "Influence", 200, 200, 200);
        draw_rect(&mut buf, UI_X as i32 + 70, 30, BAR_W, 10, 40, 40, 60);
        let infl_w = ((p.influence / WIN_INFLUENCE).clamp(0.0, 1.0) * BAR_W as f32) as i32;
        draw_rect(&mut buf, UI_X as i32 + 70, 30, infl_w, 10, 255, 200, 80);
        let recruit_x = UI_X as i32 + 70 + ((recruit_influence_needed() / WIN_INFLUENCE) * BAR_W as f32) as i32;
        draw_rect(&mut buf, recruit_x, 28, 1, 14, 120, 255, 120);
        draw_text(&mut buf, UI_X as i32 + 70 + BAR_W + 6, 31, &format!("{:.2}/{:.1}", p.influence, WIN_INFLUENCE), 255, 220, 130);
        draw_text(&mut buf, UI_X as i32 + 10, 44, &format!("Org {} | Guilds {} | Gold {:.0} | lead at {:.1}", members, guilds, p.resources, WIN_INFLUENCE), 200, 200, 200);

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
            art.villager_sprites[selected_target].blit_scaled(&mut buf, UI_X as i32 + 6, 312, 2);
            draw_text(&mut buf, UI_X as i32 + 60, 340, NAMES[selected_target], 255, 255, 255);
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
                draw_text(&mut buf, UI_X as i32 + 60, 376, &format!("Style: {:?}", style), 180, 180, 255);
            }
            if let Some(vil) = world.get_component::<Villager>(v) {
                draw_text(&mut buf, UI_X as i32 + 60, 388, &format!("Doing: {}", activity_text(vil.activity)), 200, 160, 160);
            }
            draw_text(&mut buf, UI_X as i32 + 18, 426, "keys 1-5: select villager", 120, 120, 140);
        }

        // organization: who has joined, by name
        let org_members = world.get_component::<Organization>(self.org).unwrap().members.clone();
        let member_names: Vec<&str> = org_members.iter()
            .filter_map(|m| villagers.iter().position(|v| v == m).map(|i| NAMES[i]))
            .collect();
        let org_line = if member_names.is_empty() { "nobody yet - recruit!".to_string() } else { member_names.join(", ") };
        draw_text(&mut buf, UI_X as i32 + 18, 408, "Org:", 255, 215, 0);
        let org_line: String = org_line.chars().take(36).collect();
        draw_text(&mut buf, UI_X as i32 + 48, 408, &org_line, 220, 220, 180);

        // event log
        draw_rect(&mut buf, UI_X as i32 + 10, 450, 270, 170, 25, 25, 40);
        draw_text(&mut buf, UI_X as i32 + 14, 454, "LOG", 150, 150, 180);
        // newest first, word-wrapped to the box instead of cut off mid-word
        const LOG_CHARS: usize = 43;      // (box width - padding) / glyph width
        const LOG_ROWS: usize = 12;
        let mut row = 0;
        'log: for entry in self.log.iter().rev() {
            let (r, g, b) = log_colour(entry);
            for line in wrap_text(entry, LOG_CHARS) {
                if row == LOG_ROWS { break 'log; }
                draw_text(&mut buf, UI_X as i32 + 14, 468 + row as i32 * 12, &line, r, g, b);
                row += 1;
            }
        }

        // season change: subtle banner notification instead of flash
        if tick > 10 && tick % 90 < 3 {
            let sname = match season { Season::Spring => "Spring", Season::Summer => "Summer", Season::Autumn => "Autumn", Season::Winter => "Winter" };
            let banner = format!("{} arrives", sname);
            let bw = banner.len() as i32 * GLYPH_W + 16;
            buf.rect(MAP_W as i32 / 2 - bw / 2, 56, bw, 16, 20, 20, 35);
            draw_text(&mut buf, MAP_W as i32 / 2 - bw / 2 + 8, 60, &banner, 255, 220, 130);
        }


        // weather tint overlay on map region
        if let Some(cl) = world.get_component::<climate::Climate>(self.climate) {
            if cl.rainfall_drift > 0.5 {
                // rain: an even, cool dimming over the whole map (it used to
                // darken every 6th row, which read as scanline stripes). The
                // terrain runs right up to the panel, so wash to UI_X, not MAP_W
                for y in 0..H {
                    for x in 0..UI_X {
                        let idx = (y * W + x) * 4;
                        let (r, g, b) = rain_wash((buf.pixels[idx], buf.pixels[idx + 1], buf.pixels[idx + 2]));
                        buf.set(x as i32, y as i32, r, g, b);
                    }
                }
            }
        }

        // minimap (top-right of map area): the map in miniature, same projection
        // and colours, with villagers, the player and the visible area marked
        buf.rect(MM_LEFT - 2, MM_TOP - 2, MM_W + 4, MM_H + 4, 20, 20, 30);
        let (w_t, h_t) = (self.terrain.width as f32, self.terrain.height as f32);
        for my in 0..MM_H { for mx in 0..MM_W {
            // which tile is under this minimap pixel (inverse of tile_to_minimap)
            let (cx, top) = (MM_LEFT as f32 + MM_W as f32 / 2.0, MM_TOP as f32 + 2.0);
            let diff = (MM_LEFT + mx) as f32 + 0.5 - cx;
            let sum = ((MM_TOP + my) as f32 + 0.5 - top) * 2.0;
            let (tx, ty) = ((sum / MM_K + diff / MM_K) / 2.0, (sum / MM_K - diff / MM_K) / 2.0);
            if tx >= 0.0 && ty >= 0.0 && tx < w_t && ty < h_t {
                let (r, g, b) = season_colour(self.terrain.get(tx as usize, ty as usize), season);
                buf.set(MM_LEFT + mx, MM_TOP + my, r, g, b);
            }
        }}
        let mm_dot = |buf: &mut Buffer, tx: f32, ty: f32, r: u8, g: u8, b: u8| {
            let (x, y) = tile_to_minimap(tx, ty);
            buf.rect(x as i32 - 1, y as i32 - 1, 2, 2, r, g, b);
        };
        for v in villagers.iter() {
            if let Some(p) = world.get_component::<Pos>(*v) { mm_dot(&mut buf, p.x, p.y, 255, 80, 80); }
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

        // dialog bubble over speaking villager
        if let Some((speaker, ref text, until)) = self.dialog {
            if tick < until {
                if let Some(p) = world.get_component::<Pos>(speaker) {
                    let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                    let bw = (text.len() as i32 * 6 + 12).max(60);
                    let bx = (sx - bw / 2).max(4);
                    let by = sy + SPRITE_HEAD - GLYPH_H - 4 - 26; // just above the name label
                    buf.rect(bx, by, bw, 18, 240, 240, 220);   // cream bubble
                    buf.rect(bx, by + 16, 8, 6, 240, 240, 220); // tail
                    draw_text(&mut buf, bx + 6, by + 5, text, 40, 30, 20);
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

        // ambient particles only during weather (snow in winter, rain when wet)
        let is_winter = matches!(season, Season::Winter);
        let raining = world.get_component::<climate::Climate>(self.climate).map(|c| c.rainfall_drift > 0.5).unwrap_or(false);
        if is_winter {
            // snow: 2px flakes drifting down
            for i in 0..90u64 {
                let px = ((tick * 3 + i * 37 + i * i * 7) % MAP_W as u64) as i32;
                let py = ((tick * 2 + i * 23 + i * i * 3) % H as u64) as i32;
                buf.rect(px, py, 2, 2, 230, 235, 255);
            }
        } else if raining {
            // rain: short slanting streaks
            for i in 0..120u64 {
                let px = ((tick * 5 + i * 41 + i * i * 7) % MAP_W as u64) as i32;
                let py = ((tick * 9 + i * 29 + i * i * 3) % H as u64) as i32;
                for k in 0..4 { buf.set(px - k / 2, py + k, 150, 180, 230); }
            }
        }

        // season banner
        draw_text_outlined(&mut buf, 10, 8, &format!("{:?}  t{}  food {}{}{}", season, tick, food_stock,
            if raining && !is_winter { "  raining" } else { "" },
            if self.paused { " [PAUSED]" } else if self.speed > 1 { " [4x]" } else { "" }), 255, 255, 255);

        // help overlay
        if self.help_open {
            buf.rect(200, 140, 420, 300, 20, 20, 35);
            buf.rect(200, 140, 420, 2, 200, 200, 220);
            draw_text_scaled(&mut buf, 290, 155, "CONTROLS", 2, 255, 220, 130);
            let lines = [
                "click button / key: take action",
                "click villager: select + inspect",
                "1-5: select villager",
                "T talk  R recruit  I invest",
                "O organize  B build  Z wait",
                "WASD: move   arrows: pan   +/-: zoom",
                "Space: pause   0: speed 4x",
                "F12: screenshot   Esc: quit",
            ];
            for (i, l) in lines.iter().enumerate() {
                draw_text(&mut buf, 240, 200 + i as i32 * 22, l, 210, 210, 210);
            }
            draw_text(&mut buf, 300, 420, "press H to close", 150, 150, 170);
        }

        // pause menu overlay
        if self.paused {
            buf.rect(280, 200, 260, 140, 20, 20, 35);
            buf.rect(280, 200, 260, 2, 200, 200, 220);
            draw_text_scaled(&mut buf, 340, 215, "PAUSED", 2, 255, 255, 255);
            draw_text(&mut buf, 300, 255, "Space: resume", 200, 200, 200);
            draw_text(&mut buf, 300, 275, "0: speed 1x/4x", 200, 200, 200);
            draw_text(&mut buf, 300, 295, "Esc: quit", 200, 200, 200);
            draw_text(&mut buf, 300, 315, "1-5: select villager", 200, 200, 200);
        }

        buf
    }

    /// Take action button `i` (Talk, Recruit, Invest, Organize, Build, Wait)
    fn act(&mut self, i: usize) {
        let tick = self.tick;
        let selected_target = self.selected_target;
        let msg = if let Some(reason) = self.action_blocker(i) {
            format!("Can't {}: {}.", self.buttons[i].label.to_lowercase(), reason)
        } else {
            match i {
                0 => {
                    let r = act_talk(&mut self.world, self.player, self.villagers[selected_target]);
                    let concerns = ["the harvest is thin this year", "the miller takes too much", "the lord's taxes are heavy", "we need more hands for the fields", "the old feud still burns", "winter is coming"];
                    let concern = concerns[(tick as usize + selected_target) % concerns.len()];
                    self.dialog = Some((self.villagers[selected_target], concern.to_string(), tick + 60));
                    r.description
                }
                1 => {
                    let r = act_recruit(&mut self.world, self.player, self.org, self.villagers[selected_target]);
                    if r.success { format!("{} joined!", NAMES[selected_target]) } else { r.description }
                }
                2 => { let r = act_invest(&mut self.world, self.player, 10.0); r.description }
                3 => self.house_meeting(),
                4 => {
                    // action_blocker checked there's gold and a site
                    let (bx, by) = self.build_site().unwrap();
                    if let Some(p) = self.world.get_component_mut::<PlayerCharacter>(self.player) { p.resources -= BUILD_COST; }
                    self.buildings.push((bx, by, &self.art.house_s));
                    self.built_count += 1;
                    format!("t{}: A house rises at ({:.0},{:.0}) - {:.0} gold", tick, bx, by, BUILD_COST)
                }
                _ => "You wait and observe.".into(),
            }
        };
        self.log.push(msg);
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

    /// Organize: hold a house meeting with every living villager within reach.
    /// Those who already trust you join the org; everyone who comes gets to
    /// know you a little better (organizing::house_meeting).
    fn house_meeting(&mut self) -> String {
        let invited = self.villagers_in_reach();
        if invited.is_empty() { return "No one is near enough to come to your house meeting.".into(); }
        let members_before = self.world.get_component::<Organization>(self.org).map(|o| o.members.clone()).unwrap_or_default();
        let r = game2_core::game::organizing::house_meeting(&mut self.world, self.org, &invited);
        let joined: Vec<&str> = invited.iter()
            .filter(|v| !members_before.contains(v))
            .filter(|v| self.world.get_component::<Organization>(self.org).map(|o| o.members.contains(v)).unwrap_or(false))
            .filter_map(|v| self.villagers.iter().position(|x| x == v).map(|i| NAMES[i]))
            .collect();
        let outcome = if joined.is_empty() {
            "No one joins yet, but they'll remember the conversation.".to_string()
        } else {
            format!("{} joined{}!", joined.join(", "), if r.formed_committee { " and formed a committee" } else { "" })
        };
        format!("t{}: House meeting - {} came. {}", self.tick, invited.len(), outcome)
    }

    /// A left click on the map: rivals rebuff or greet you, villagers get selected
    fn click_map(&mut self, mx: i32, my: i32) {
        let (tick, cam_x, cam_y, zoom) = (self.tick, self.cam_x, self.cam_y, self.zoom);
        // click on a rival villager -> they rebuff you (foreign village is wary)
        for (i, rv) in self.rivals.iter().enumerate() {
            if let Some(p) = self.world.get_component::<Pos>(*rv) {
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
                    self.dialog = Some((*rv, msg, tick + 60));
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
        self.selected_target = best;
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
    let art = Art::load("assets");
    if let Some(target) = dump_at {
        run_headless(&art, target);
        return;
    }

    let mut window = Window::new("The Wanderer - Medieval Village", W, H, WindowOptions::default())
        .expect("window failed");
    let mut game = Game::new(&art);
    let mut started = false; // title screen until Enter
    let mut mouse_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // TITLE SCREEN until Enter
        if !started {
            let mut tbuf = Buffer::new(W, H);
            tbuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
            draw_text_scaled(&mut tbuf, 300, 200, "THE WANDERER", 4, 255, 220, 130);
            draw_text_scaled(&mut tbuf, 260, 280, "an outsider builds power", 2, 200, 200, 200);
            draw_text(&mut tbuf, 350, 360, "press ENTER to begin", 255, 255, 255);
            draw_text(&mut tbuf, 310, 400, "organize the village. gain influence.", 150, 150, 170);
            present(&mut window, &tbuf);
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
        if clicked { game.click_map(mx, my); }
        if window.is_key_down(Key::Key1) { game.selected_target = 0; }
        if window.is_key_down(Key::Key2) { game.selected_target = 1; }
        if window.is_key_down(Key::Key3) { game.selected_target = 2; }
        if window.is_key_down(Key::Key4) { game.selected_target = 3; }
        if window.is_key_down(Key::Key5) { game.selected_target = 4; }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { game.paused = !game.paused; }
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) { game.help_open = !game.help_open; }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { game.zoom = (game.zoom + 0.02).min(2.0); }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { game.zoom = (game.zoom - 0.02).max(0.5); }
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) { game.speed = if game.speed == 1 { 4 } else { 1 }; }
        // WASD moves the player character (iso directions)
        if let Some(pp) = game.world.get_component_mut::<Pos>(game.player) {
            let spd = 0.15;
            if window.is_key_down(Key::W) { pp.x -= spd; pp.y -= spd; }  // up
            if window.is_key_down(Key::S) { pp.x += spd; pp.y += spd; }  // down
            if window.is_key_down(Key::A) { pp.x -= spd; pp.y += spd; }  // left
            if window.is_key_down(Key::D) { pp.x += spd; pp.y -= spd; }  // right
            pp.x = pp.x.clamp(0.0, 31.0); pp.y = pp.y.clamp(0.0, 31.0);
        }
        // arrows pan the camera
        if window.is_key_down(Key::Left)  { game.cam_x += 6; }
        if window.is_key_down(Key::Right) { game.cam_x -= 6; }
        if window.is_key_down(Key::Up)    { game.cam_y += 6; }
        if window.is_key_down(Key::Down)  { game.cam_y -= 6; }

        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", game.frame_n)).ok(); }

        // blit buffer to framebuffer
        present(&mut window, &buf);

        let p = game.world.get_component::<PlayerCharacter>(game.player).unwrap();
        if p.influence > WIN_INFLUENCE {
            let members = game.world.get_component::<Organization>(game.org).unwrap().members.len();
            // win overlay in-window
            buf.rect(200, 240, 400, 100, 20, 20, 35);
            buf.rect(200, 240, 400, 2, 255, 215, 0);
            draw_text_scaled(&mut buf, 260, 265, "RECOGNIZED LEADER", 2, 255, 215, 0);
            draw_text(&mut buf, 290, 300, "the village is yours", 200, 200, 200);
            present(&mut window, &buf);
            std::thread::sleep(std::time::Duration::from_secs(3));
            // game over summary screen
            let mut obuf = Buffer::new(W, H);
            obuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
            draw_text_scaled(&mut obuf, 300, 150, "VILLAGE WON", 4, 255, 215, 0);
            draw_text(&mut obuf, 300, 240, &format!("org members: {}", members), 200, 200, 200);
            draw_text(&mut obuf, 300, 260, &format!("influence: {:.2}", p.influence), 200, 200, 200);
            draw_text(&mut obuf, 300, 280, &format!("buildings built: {}", game.built_count), 200, 200, 200);
            draw_text(&mut obuf, 300, 300, &format!("ticks: {}", game.tick), 200, 200, 200);
            draw_text(&mut obuf, 300, 340, "the wanderer became a leader", 160, 160, 180);
            present(&mut window, &obuf);
            std::thread::sleep(std::time::Duration::from_secs(5));
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(80)); // ~12fps
    }
}


/// Headless verification: play the game (no input) until tick N, then
/// render that frame through the same Game::render the window uses and save
/// it as a BMP.
fn run_headless(art: &Art, target_tick: u64) {
    let mut game = Game::new(art);
    while game.tick < target_tick { game.update(); }
    let buf = game.render((-1, -1)); // cursor off-screen: no hover
    buf.save_bmp(&format!("evidence/frame-check-{}.bmp", target_tick)).unwrap();
    println!("frame {} saved", target_tick);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ground_block_top_face_fits_one_tile() {
        // the pack's ground blocks are ~95x97: a raised cube for a far bigger grid
        let block = Some(Sprite { width: 95, height: 97, pixels: vec![255; 95 * 97 * 4] });
        let top = top_face(&block).unwrap();
        assert_eq!((top.width, top.height), (95, 48), "keep only the top diamond, not the sides");
        let tile = top.scaled((2 * TILE_W) as usize, (2 * TILE_H) as usize);
        assert_eq!((tile.width, tile.height), (56, 28), "a ground tile covers exactly one map tile");
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
        for (v, x, y) in [(eliza, 17.0, 16.0), (bram, 15.0, 17.0), (corvin, 2.0, 2.0), (aldric, 2.0, 4.0), (dagny, 4.0, 2.0)] {
            let p = game.world.get_component_mut::<Pos>(v).unwrap();
            p.x = x; p.y = y;
        }
        // Eliza already trusts the player; Bram is a stranger
        let rel = game.world.create_entity();
        game.world.add_component(rel, Relationship { from: game.player, other: eliza, kind: RelKind::Friend, strength: 0.7, interactions: 6 });

        assert_eq!(game.disabled_reason(3), None, "a house meeting needs someone nearby, not the selected villager");
        game.act(3);

        let members = game.world.get_component::<Organization>(game.org).unwrap().members.clone();
        assert_eq!(members, vec![eliza], "the trusted neighbour joins; the stranger and the absent don't");
        let last = game.log.last().unwrap();
        assert!(last.contains("House meeting - 2 came") && last.contains("Eliza joined"), "{}", last);
        assert!(relationships::find_relationship(&game.world, game.player, bram).is_some(), "Bram got to know the player");
        assert!(relationships::find_relationship(&game.world, game.player, corvin).is_none(), "Corvin wasn't there");
    }
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
            let h = game.terrain.get(x as usize, y as usize);
            assert!((0.15..0.75).contains(&h), "house on unbuildable ground at ({},{}) h={}", x, y, h);
            assert!((x - px).abs() + (y - py).abs() <= 2.0 * BUILD_SEARCH as f32, "house far from the player");
            let clear_of = |bx: f32, by: f32| (bx.round() - x).abs() > 1.0 || (by.round() - y).abs() > 1.0;
            assert!(game.buildings[..start].iter().all(|(bx, by, _)| clear_of(*bx, *by)), "house on or beside an existing building at ({},{})", x, y);
            assert!(clear_of(MILL_TILE.0, MILL_TILE.1), "house on the windmill at ({},{})", x, y);
            assert!((x - px.round()).abs() + (y - py.round()).abs() >= 2.0, "house right on top of the player at ({},{})", x, y);
            assert!(game.ground.iter().all(|(gx, gy, _)| (gx.round(), gy.round()) != (x, y)), "house on a road or field");
            assert!(new[..i].iter().all(|&(ox, oy)| clear_of(ox, oy)), "two houses on or beside each other at ({},{})", x, y);
        }
    }
}
