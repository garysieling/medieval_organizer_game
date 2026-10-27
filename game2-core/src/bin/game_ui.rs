//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::ecs::{Entity, Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::{draw_text, draw_text_scaled};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest, can_recruit};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::ui::isometric::render_terrain;
use game2_core::sim::climate::{self, SeasonSystem, Season};
use game2_core::sim::illness::Health;
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
    fn load() -> Art {
        let assets = Assets::new("assets");

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
    player: Entity,
    org: Entity,
    dog: Entity,
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

        let terrain = Terrain::generate(33, 0.7, 42);
        let climate = climate::spawn_climate(&mut world);
        villager::spawn_food_source(&mut world, 16.0, 16.0, 500);

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

        // rival village: foreign settlement across the river
        let mut rivals = vec![];
        for i in 0..3 {
            let r = villager::spawn_villager(&mut world, (28.0 + i as f32 * 2.0), (12.0 + (i % 2) as f32 * 3.0));
            world.add_component(r, Health { condition: 0.9, immunity: 0.5 });
            rivals.push(r);
        }

        let player = world.create_entity();
        world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        world.add_component(player, Pos { x: 10.0, y: 10.0 }); // player has a position
        let org = world.create_entity();
        world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });
        let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
        pets::adopt(&mut world, dog, villagers[0]);

        let buttons = vec![
            Button { label: "Talk",     key: 'T', x: UI_X as i32 + 10, y: 60,  w: 130, h: 36, color: (60,60,120) },
            Button { label: "Recruit",  key: 'R', x: UI_X as i32 + 10, y: 104, w: 130, h: 36, color: (60,120,60) },
            Button { label: "Invest",   key: 'I', x: UI_X as i32 + 10, y: 148, w: 130, h: 36, color: (120,60,60) },
            Button { label: "Organize", key: 'O', x: UI_X as i32 + 10, y: 192, w: 130, h: 36, color: (120,100,40) },
            Button { label: "Build",    key: 'B', x: UI_X as i32 + 10, y: 236, w: 130, h: 36, color: (80,60,120) },
            Button { label: "Wait",     key: 'W', x: UI_X as i32 + 10, y: 280, w: 130, h: 36, color: (60,60,60) },
        ];

        Game {
            art, world, sched, terrain, climate, villagers, rivals, player, org, dog, buildings, ground, buttons,
            built_count: 0,
            zoom: 1.0,
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
            frame_n: 0,
        }
    }

    fn ticks_per_frame(&self) -> u32 {
        if self.speed == 1 { 60 } else { 15 } // ~5s/day normal, ~1.2s turbo
    }

    fn player_pos(&self) -> (f32, f32) {
        self.world.get_component::<Pos>(self.player).map(|p| (p.x, p.y)).unwrap_or((10.0, 10.0))
    }

    /// Why action button `i` can't be used right now, if it can't
    fn disabled_reason(&self, i: usize) -> Option<&'static str> {
        // proximity: Talk/Recruit/Organize need a living selected villager within 8 tiles of player
        let player_pos = self.player_pos();
        let target = self.villagers[self.selected_target];
        let dist_ok = self.world.get_component::<Pos>(target)
            .map(|vp| { let dx = vp.x - player_pos.0; let dy = vp.y - player_pos.1; (dx*dx + dy*dy).sqrt() < 8.0 })
            .unwrap_or(false);
        let target_alive = self.world.get_component::<Health>(target).is_some();
        let recruit_infl_ok = can_recruit(self.world.get_component::<PlayerCharacter>(self.player).unwrap().influence);
        if (i == 0 || i == 1 || i == 3) && !target_alive { Some("they have died") }
        else if (i == 0 || i == 1 || i == 3) && !dist_ok { Some("too far away") }
        else if i == 1 && !recruit_infl_ok { Some("need more influence") }
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
        let (tint_r, tint_g, tint_b) = match season {
            Season::Spring => (1.0, 1.1, 0.9),
            Season::Summer => (1.0, 1.0, 1.0),
            Season::Autumn => (1.15, 0.9, 0.7),
            Season::Winter => (0.85, 0.9, 1.15),
        };
        let wave_phase = (tick % 40) < 20;
        let tw = (TILE_W as f32 * zoom) as i32;
        let th = (TILE_H as f32 * zoom) as i32;
        for ty in 0..self.terrain.height {
            for tx in 0..self.terrain.width {
                let h = self.terrain.get(tx, ty);
                let (mut r, mut g, mut b) = if h < 0.15 { (30, 60, 180) }
                    else if h < 0.35 { (60, 140, 60) }
                    else if h < 0.55 { (140, 110, 50) }
                    else if h < 0.75 { (110, 90, 60) }
                    else { (220, 220, 220) };
                r = (r as f32 * tint_r).min(255.0) as u8;
                g = (g as f32 * tint_g).min(255.0) as u8;
                b = (b as f32 * tint_b).min(255.0) as u8;
                if b > r && b > g && wave_phase { b = b.saturating_add(15); } // water shimmer
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
        let (msx, msy) = iso_to_screen_cam(10.0, 9.0, cam_x, cam_y, zoom);
        mill_frame.blit_center(&mut buf, msx, msy);

        // food stock indicator at market
        let food_stock = world.query::<villager::FoodSource>().first()
            .and_then(|f| world.get_component::<villager::FoodSource>(*f))
            .map(|f| f.servings).unwrap_or(0);
        let (fsx, fsy) = iso_to_screen_cam(16.0, 15.0, cam_x, cam_y, zoom);
        // stock bar under the market
        buf.rect(fsx - 20, fsy + 10, 40, 4, 60, 40, 40);
        let fw = ((food_stock as f32 / 500.0) * 40.0) as i32;
        buf.rect(fsx - 20, fsy + 10, fw, 4, 80, 200, 80);
        draw_text(&mut buf, fsx - 20, fsy + 16, &format!("food: {}", food_stock), 200, 200, 150);


        // villagers as sprites (sorted by y for depth)
        let mut v_sorted: Vec<(usize, f32, f32)> = villagers.iter().enumerate()
            .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (i, p.x, p.y)))
            .collect();
        v_sorted.sort_by(|a, b| (a.1 + a.2).partial_cmp(&(b.1 + b.2)).unwrap());
        for (i, wx, wy) in v_sorted {
            let (sx, sy) = iso_to_screen_cam(wx, wy, cam_x, cam_y, zoom);
            art.villager_sprites[i].blit_scaled(&mut buf, sx - 32, sy - 60, 2);
            // selection highlight
            if i == selected_target {
                draw_rect(&mut buf, sx - 14, sy - 66, 28, 2, 255, 255, 0);
            }
            // grave if dead (no Health component)
            if world.get_component::<Health>(villagers[i]).is_none() {
                buf.rect(sx - 4, sy - 10, 8, 10, 120, 120, 130); // gravestone
                draw_text(&mut buf, sx - 6, sy - 20, "RIP", 200, 200, 210);
                continue;
            }
            // name label
            draw_text(&mut buf, sx - 14, sy - 76, NAMES[i], 255, 255, 255);
            // hunger indicator
            if let Some(h) = world.get_component::<Hunger>(villagers[i]) {
                if h.value < 40.0 { draw_text(&mut buf, sx - 3, sy - 66, "!", 255, 100, 100); }
            }
            if let Some(vil) = world.get_component::<Villager>(villagers[i]) {
                match vil.activity {
                    Activity::SeekingFood => { draw_text(&mut buf, sx + 8, sy - 60, "o", 255, 200, 100); }
                    Activity::Idle => {},
                    _ => { draw_text(&mut buf, sx + 8, sy - 60, "*", 180, 180, 255); }
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
                    draw_text(&mut buf, dx - 3, dy - 16, "<3", 255, 120, 120);
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
                draw_text(&mut buf, sx - 14, sy - 76, RIVAL_NAMES[i], 255, 140, 140);
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
        draw_text(&mut buf, px - 12, py - 76, "YOU", 255, 255, 0);

        // --- UI PANEL ---
        draw_rect(&mut buf, UI_X as i32, 0, (W as i32 - UI_X as i32), H as i32, 32, 32, 48);
        draw_text_scaled(&mut buf, UI_X as i32 + 10, 8, "THE WANDERER", 2, 255, 220, 130);

        // status: influence bar, members, season, tick
        let p = world.get_component::<PlayerCharacter>(self.player).unwrap();
        let members = world.get_component::<Organization>(self.org).unwrap().members.len();
        let guilds = world.query::<Guild>().len();
        draw_text(&mut buf, UI_X as i32 + 10, 30, &format!("Influence"), 200, 200, 200);
        draw_rect(&mut buf, UI_X as i32 + 90, 30, 140, 10, 40, 40, 60);
        let infl_w = (p.influence * 50.0).min(140.0) as i32;
        draw_rect(&mut buf, UI_X as i32 + 90, 30, infl_w, 10, 255, 200, 80);
        draw_text(&mut buf, UI_X as i32 + 10, 44, &format!("Org: {} | Guilds: {} | Gold: {:.0}", members, guilds, p.resources), 200, 200, 200);

        // buttons
        for (i, b) in self.buttons.iter().enumerate() {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            let reason = self.disabled_reason(i);
            draw_button(&mut buf, b, hover, reason.is_some());
            if hover {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, reason.unwrap_or(TOOLTIPS[i]), 220, 220, 160);
            }
        }

        // inspector panel for selected villager
        if self.inspector_open {
            draw_rect(&mut buf, UI_X as i32 + 10, 330, 270, 110, 45, 45, 65);
            let v = villagers[selected_target];
            art.villager_sprites[selected_target].blit(&mut buf, UI_X as i32 + 18, 340);
            draw_text(&mut buf, UI_X as i32 + 60, 340, NAMES[selected_target], 255, 255, 255);
            if let Some(h) = world.get_component::<Hunger>(v) {
                draw_text(&mut buf, UI_X as i32 + 60, 352, &format!("Hunger: {:.0}", h.value), 200, 200, 100);
            }
            if let Some(he) = world.get_component::<Health>(v) {
                draw_text(&mut buf, UI_X as i32 + 60, 364, &format!("Health: {:.0}%", he.condition * 100.0), 100, 220, 100);
            }
            if let Some(a) = world.get_component::<AttachmentHistory>(v) {
                let style = infer_style(a);
                draw_text(&mut buf, UI_X as i32 + 60, 376, &format!("Style: {:?}", style), 180, 180, 255);
            }
            if let Some(vil) = world.get_component::<Villager>(v) {
                draw_text(&mut buf, UI_X as i32 + 60, 388, &format!("Doing: {:?}", vil.activity), 200, 160, 160);
            }
            draw_text(&mut buf, UI_X as i32 + 10, 400, &format!("(keys 1-5 select villager)"), 120, 120, 140);
        }

        // organization panel: member portraits + name
        draw_text(&mut buf, UI_X as i32 + 160, 404, &format!("ORG:"), 255, 215, 0);
        let org_members = world.get_component::<Organization>(self.org).unwrap().members.clone();
        for (i, m) in org_members.iter().take(6).enumerate() {
            // find which villager index this is
            let vi = villagers.iter().position(|v| v == m).unwrap_or(0);
            art.villager_sprites[vi].blit(&mut buf, UI_X as i32 + 200 + i as i32 * 14, 396);
        }

        // event log
        draw_rect(&mut buf, UI_X as i32 + 10, 450, 270, 170, 25, 25, 40);
        draw_text(&mut buf, UI_X as i32 + 14, 454, "LOG", 150, 150, 180);
        for (i, entry) in self.log.iter().rev().take(9).enumerate() {
            let trunc: String = entry.chars().take(42).collect();
            // color by event type: deaths red, guilds gold, joins green, weather cyan, talk white
            let (r, g, b) = if entry.contains("died") || entry.contains("dead") { (255, 100, 100) }
                else if entry.contains("Guild") || entry.contains("guild") { (255, 215, 0) }
                else if entry.contains("joined") { (120, 255, 120) }
                else if entry.contains("storm") || entry.contains("rain") || entry.contains("flu") { (120, 200, 255) }
                else { (210, 210, 210) };
            draw_text(&mut buf, UI_X as i32 + 14, 470 + i as i32 * 16, &trunc, r, g, b);
        }

        // season change: subtle banner notification instead of flash
        if tick > 10 && tick % 90 < 3 {
            let sname = match season { Season::Spring => "Spring", Season::Summer => "Summer", Season::Autumn => "Autumn", Season::Winter => "Winter" };
            draw_text(&mut buf, MAP_W as i32 / 2 - 30, 60, &format!("{} arrives", sname), 255, 220, 130);
        }


        // weather tint overlay on map region
        if let Some(cl) = world.get_component::<climate::Climate>(self.climate) {
            if cl.rainfall_drift > 0.5 {
                // rain: dark blue tint bands on map area
                for y in (0..H).step_by(6) {
                    for x in 0..MAP_W {
                        let idx = (y * W + x) * 4;
                        if idx + 2 < buf.pixels.len() {
                            buf.set(x as i32, y as i32,
                                buf.pixels[idx].saturating_sub(10),
                                buf.pixels[idx+1].saturating_sub(10),
                                buf.pixels[idx+2].saturating_add(30));
                        }
                    }
                }
                draw_text(&mut buf, 10, 24, "RAINING", 150, 180, 255);
            }
        }

        // minimap (top-right of map area) - dots for villagers/player/buildings
        let mm_x = MAP_W as i32 - 90;
        let mm_y = 10;
        buf.rect(mm_x - 4, mm_y - 4, 84, 84, 20, 20, 30);
        buf.rect(mm_x - 4, mm_y - 4, 84, 1, 120, 120, 140);
        buf.rect(mm_x - 4, mm_y + 80, 84, 1, 120, 120, 140);
        // terrain thumbnail
        for my in 0..33 { for mx in 0..33 {
            let h = self.terrain.heights[my][mx];
            let px = mm_x + (mx * 2) as i32;
            let py = mm_y + (my * 2) as i32;
            let (r, g, b) = if h < 0.35 { (40, 60, 160) } else if h < 0.6 { (60, 140, 60) } else { (160, 160, 160) };
            buf.set(px, py, r, g, b); buf.set(px+1, py, r, g, b);
            buf.set(px, py+1, r, g, b); buf.set(px+1, py+1, r, g, b);
        }}
        // entity dots
        for v in villagers.iter() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                buf.rect(mm_x + (p.x * 2.0) as i32, mm_y + (p.y * 2.0) as i32, 2, 2, 255, 80, 80);
            }
        }
        buf.rect(mm_x + (player_pos.0 * 2.0) as i32, mm_y + (player_pos.1 * 2.0) as i32, 2, 2, 255, 255, 255); // player

        // dialog bubble over speaking villager
        if let Some((speaker, ref text, until)) = self.dialog {
            if tick < until {
                if let Some(p) = world.get_component::<Pos>(speaker) {
                    let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                    let bw = (text.len() as i32 * 6 + 12).max(60);
                    let bx = (sx - bw / 2).max(4);
                    let by = sy - 95;
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
        if is_winter || raining {
            let (pr, pg, pb) = if is_winter { (230, 235, 255) } else { (120, 160, 220) };
            for i in 0..20 {
                let px = ((tick + i * 37) % MAP_W as u64) as i32;
                let py = ((tick * 2 + i * 23) % H as u64) as i32;
                if px < MAP_W as i32 { buf.set(px, py, pr, pg, pb); }
            }
        }

        // season banner
        draw_text(&mut buf, 10, 8, &format!("{:?}  t{}{}", season, tick,
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
                "O organize  B build  WASD move",
                "arrows: pan map   +/-: zoom",
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
        let msg = if let Some(reason) = self.disabled_reason(i) {
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
                3 => "You call a house meeting.".into(),
                4 => {
                    let bx = 4.0 + (self.built_count % 4) as f32 * 5.0;
                    let by = 15.0 + (self.built_count / 4) as f32 * 4.0;
                    self.buildings.push((bx, by, &self.art.house_s));
                    self.built_count += 1;
                    format!("A house rises at ({:.0},{:.0})", bx, by)
                }
                _ => "You wait and observe.".into(),
            }
        };
        self.log.push(msg);
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
    let art = Art::load();
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
        for (k, i) in [(Key::T, 0), (Key::R, 1), (Key::I, 2), (Key::O, 3), (Key::B, 4)] {
            if window.is_key_pressed(k, minifb::KeyRepeat::No) { action = Some(i); }
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
        if p.influence > 2.0 {
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
}
