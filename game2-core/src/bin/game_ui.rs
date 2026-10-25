//! Game UI v2: real sprites + text. Isometric map, villager sprites,
//! labeled action buttons, inspector panel, event log.

use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::ecs::{Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::{draw_text, draw_text_scaled};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole, Pet};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest};
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
const TILE_W: i32 = 20;
const TILE_H: i32 = 10;

fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }

fn draw_rect(buf: &mut Buffer, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
    buf.rect(x, y, w, h, r, g, b);
}

/// Convert world tile coords to screen isometric position
fn iso_to_screen_cam(tx: f32, ty: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (i32, i32) {
    let tw = TILE_W as f32 * zoom;
    let th = TILE_H as f32 * zoom;
    let sx = MAP_W as i32 / 2 + ((tx - ty) * tw) as i32 + cam_x;
    let sy = 100 + ((tx + ty) * th) as i32 + cam_y;
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

fn main() {
    let mut window = Window::new("The Wanderer - Medieval Village", W, H, WindowOptions::default())
        .expect("window failed");

    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GuildFormationSystem);

    let terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 500);

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

    // Load building sprites
    let house_s = Sprite::load(&assets.path(&assets.building_sprite("house", "blue"))).ok();
    let mill_frames: Vec<Sprite> = (1..=4).map(|i|
        Sprite::load(&assets.path(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/mill/mill({}).png", i))).unwrap()
    ).collect();
    let blacksmith_s = Sprite::load(&assets.path(&assets.building_sprite("blacksmith", "green"))).ok();
    let tree_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/trees/tree(1).png")).ok();
    let wheat_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/wheat/wheat(1).png")).ok();
    let corn_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/corn/corn(1).png")).ok();
    let stone_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/stones/stone(1).png")).ok();
    let player_s = Sprite::load(&assets.path("MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png")).map(|s| s.crop(0, 0, 32, 32)).ok();
    let tower_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Towers/Wood towers/castle_tower_wood(1).png")).ok();
    let river_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Rivers/Spring/river(1).png")).ok();
    let wall_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Castle walls/castle_wall(1).png")).ok();
    let road_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Roads/Spring/road(1).png")).ok();
    let corn2_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/corn/corn(2).png")).ok();
    let church_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Towers/Blue towers/castle_tower_blue(1).png")).ok();
    let guildhall_s = Sprite::load(&assets.path(&assets.building_sprite("house", "red"))).ok();
    let smoke_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/smoke.png")).ok();
    let rival_house = Sprite::load(&assets.path(&assets.building_sprite("house", "red"))).ok();
    let rival_woodhouse = Sprite::load(&assets.path(&assets.building_sprite("woodhouse", "red"))).ok();

    // buildings placed on map
    let mut buildings: Vec<(f32, f32, &Option<Sprite>)> = vec![
        (5.0, 5.0, &house_s), (12.0, 4.0, &house_s), (7.0, 12.0, &house_s),
        (15.0, 8.0, &blacksmith_s),
        (3.0, 14.0, &tree_s), (18.0, 5.0, &tree_s), (9.0, 16.0, &tree_s), (20.0, 12.0, &tree_s),
        // farms: wheat + corn fields near the village
        (14.0, 14.0, &wheat_s), (15.0, 14.0, &wheat_s), (14.0, 15.0, &wheat_s), (15.0, 15.0, &wheat_s),
        (16.0, 15.0, &corn_s), (16.0, 16.0, &corn_s), (17.0, 15.0, &corn_s),
        (4.0, 18.0, &stone_s), (19.0, 16.0, &stone_s),
        // market plaza (packed dirt tiles around stalls)
        (7.0, 6.0, &road_s), (8.0, 6.0, &road_s), (9.0, 6.0, &road_s),
        // food market stalls (dim when food runs low)
        (7.0, 7.0, &corn2_s), (9.0, 7.0, &corn2_s),
        // main road through the village center
        (8.0, 8.0, &road_s), (9.0, 9.0, &road_s), (10.0, 10.0, &road_s), (11.0, 11.0, &road_s),
        (12.0, 8.0, &road_s), (13.0, 7.0, &road_s), (8.0, 12.0, &road_s), (9.0, 13.0, &road_s),
        // river along the east edge
        (24.0, 3.0, &river_s), (26.0, 5.0, &river_s), (28.0, 7.0, &river_s), (30.0, 9.0, &river_s),
        // castle towers at the far edge (the ruler overlooks the village)
        (26.0, 6.0, &tower_s), (28.0, 8.0, &tower_s), (27.0, 4.0, &tower_s),
        // rival village across the river (red-roofed foreign houses)
        (28.0, 13.0, &rival_house), (30.0, 15.0, &rival_woodhouse),
        // castle walls connecting the towers
        (25.0, 5.0, &wall_s), (27.0, 7.0, &wall_s), (29.0, 9.0, &wall_s),
        // church (religion center) + guild hall (merchant base)
        (2.0, 8.0, &church_s), (17.0, 10.0, &guildhall_s),
    ];
    let mut built_count = 0usize;
    let mut zoom = 1.0f32; // 0.5 - 2.0
    let mut toasts: Vec<(String, u64)> = vec![];  // (msg, shown until tick)
    let mut last_guild_count = 0usize;

    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4 + 3) as f32, (i%3 + 6) as f32);
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
    // relationships: some villagers are friends, others feud
    use game2_core::game::relationships::RelKind;
    world.add_component(villagers[0], Relationship { other: villagers[1], kind: RelKind::Friend, strength: 0.7, interactions: 8 });
    world.add_component(villagers[2], Relationship { other: villagers[3], kind: RelKind::Rival, strength: -0.5, interactions: 5 });
    world.add_component(villagers[0], Relationship { other: villagers[3], kind: RelKind::Acquaintance, strength: 0.4, interactions: 3 });

    // rival village: foreign settlement across the river
    let mut rivals = vec![];
    for i in 0..3 {
        let r = villager::spawn_villager(&mut world, (28.0 + i as f32 * 2.0), (12.0 + (i % 2) as f32 * 3.0));
        world.add_component(r, Health { condition: 0.9, immunity: 0.5 });
        rivals.push(r);
    }
    let rival_names = ["Wulfric","Sable","Miren"];

    let player = world.create_entity();
    world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
    let org = world.create_entity();
    world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);

    let tooltips = [
        "talk to selected villager",
        "recruit villager to your org",
        "spend gold for influence",
        "call a house meeting",
        "commission a new house",
        "watch the village",
    ];
    let buttons = vec![
        Button { label: "Talk",     key: 'T', x: UI_X as i32 + 10, y: 60,  w: 130, h: 36, color: (60,60,120) },
        Button { label: "Recruit",  key: 'R', x: UI_X as i32 + 10, y: 104, w: 130, h: 36, color: (60,120,60) },
        Button { label: "Invest",   key: 'I', x: UI_X as i32 + 10, y: 148, w: 130, h: 36, color: (120,60,60) },
        Button { label: "Organize", key: 'O', x: UI_X as i32 + 10, y: 192, w: 130, h: 36, color: (120,100,40) },
        Button { label: "Build",    key: 'B', x: UI_X as i32 + 10, y: 236, w: 130, h: 36, color: (80,60,120) },
        Button { label: "Wait",     key: 'W', x: UI_X as i32 + 10, y: 280, w: 130, h: 36, color: (60,60,60) },
    ];

    let mut tick = 0u64;
    let mut cam_x = 0i32;   // camera pan offset
    let mut cam_y = 0i32;
    let mut started = false; // title screen until Enter
    let mut paused = false;
    let mut speed = 1u32;   // 1=normal 2=fast 4=turbo
    let mut selected_target = 0usize;
    let mut log: Vec<String> = vec!["Welcome to the village.".into()];
    let mut rapport = [0.0f32; 5]; // relationship score per villager
    let mut inspector_open = true;
    let mut help_open = false;
    let mut dialog: Option<(usize, String, u64)> = None; // (villager idx, text, until tick)
    let mut frame_n = 0u32;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // TITLE SCREEN until Enter
        if !started {
            let mut tbuf = Buffer::new(W, H);
            tbuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
            draw_text_scaled(&mut tbuf, 300, 200, "THE WANDERER", 4, 255, 220, 130);
            draw_text_scaled(&mut tbuf, 260, 280, "an outsider builds power", 2, 200, 200, 200);
            draw_text(&mut tbuf, 350, 360, "press ENTER to begin", 255, 255, 255);
            draw_text(&mut tbuf, 310, 400, "organize the village. gain influence.", 150, 150, 170);
            let mut fb = vec![0u32; W * H];
            for i in 0..W * H {
                let idx = i * 4;
                fb[i] = rgb(tbuf.pixels[idx], tbuf.pixels[idx+1], tbuf.pixels[idx+2]);
            }
            window.update_with_buffer(&fb, W, H).unwrap();
            if window.is_key_down(Key::Enter) { started = true; }
            continue;
        }

        if !paused {
            for _ in 0..speed { tick += 1; sched.tick(&mut world); }
        }
        // idle villagers wander (village feels alive)
        if !paused {
            for v in villagers.iter() {
                let idle = world.get_component::<Villager>(*v).map(|x| x.activity == Activity::Idle).unwrap_or(false);
                if idle {
                    if let Some(p) = world.get_component_mut::<Pos>(*v) {
                        let wob = ((tick + *v as u64) % 8) as f32;
                        if wob < 1.0 { p.x += 0.2; } else if wob < 2.0 { p.y += 0.2; }
                        else if wob < 3.0 { p.x -= 0.2; } else if wob < 4.0 { p.y -= 0.2; }
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
                    toasts.push((format!("{} has died.", names[i]), tick + 60));
                    log.push(format!("t{}: {} died.", tick, names[i]));
                    world.remove_component::<Health>(*v);
                }
            }
        }

        // villagers socialize: neighbors form/drift bonds
        if tick % 25 == 0 && !paused {
            let a = (tick as usize / 25) % 5;
            let b = (tick as usize / 25 + 1) % 5;
            // they talk -> bond strengthens
            if let Some(rel) = world.get_component_mut::<Relationship>(villagers[a]) {
                if rel.other == villagers[b] {
                    rel.strength = (rel.strength + 0.08).min(1.0);
                    rel.interactions += 1;
                }
            } else {
                world.add_component(villagers[a], Relationship {
                    other: villagers[b], kind: RelKind::Acquaintance, strength: 0.1, interactions: 1,
                });
            }
        }

        // gossip: rumors spread villager-to-villager
        if tick % 40 == 0 {
            let rumors = [
                "word spreads of a stranger organizing",
                "the miller is skimming the flour",
                "someone saw the lord's tax collector",
                "the guild merchants grow bold",
                "an old wound reopens between families",
            ];
            let a = (tick as usize / 40) % 5;
            let b = (tick as usize / 40 + 2) % 5;
            log.push(format!("t{}: {} tells {} - {}", tick, names[a], names[b], rumors[tick as usize / 40 % 5]));
        }

        // detect events -> toasts
        let gc = world.query::<Guild>().len();
        if gc > last_guild_count {
            toasts.push(("A guild forms in the village!".into(), tick + 40));
            log.push(format!("t{}: The Merchant Guild formed!", tick));
        }
        last_guild_count = gc;


        // --- RENDER into Buffer ---
        let mut buf = Buffer::new(W, H);
        buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);

        // isometric terrain (season-tinted)
        let board = render_terrain(&terrain, MAP_W, H);
        let season = world.get_component::<climate::Climate>(climate).unwrap().season;
        let (tint_r, tint_g, tint_b) = match season {
            Season::Spring => (1.0, 1.1, 0.9),
            Season::Summer => (1.0, 1.0, 1.0),
            Season::Autumn => (1.15, 0.9, 0.7),
            Season::Winter => (0.85, 0.9, 1.15),
        };
        let wave_phase = (tick % 20) < 10;
        for y in 0..H { for x in 0..MAP_W {
            let idx = (y * MAP_W + x) * 4;
            if idx + 3 < board.pixels.len() {
                let (r, g, b) = (board.pixels[idx], board.pixels[idx+1], board.pixels[idx+2]);
                if !(r == 0 && g == 0 && b == 0) {
                    // water shimmer: blue pixels brighten slightly on alternate frames
                    let shimmer = if b > r && b > g && wave_phase { 18.0 } else { 0.0 };
                    buf.set(x as i32, y as i32,
                        (r as f32 * tint_r).min(255.0) as u8,
                        (g as f32 * tint_g).min(255.0) as u8,
                        ((b as f32 * tint_b) + shimmer).min(255.0) as u8);
                }
            }
        }}

        // buildings on map (sorted back-to-front by tile y)
        let mut b_sorted: Vec<_> = buildings.iter().collect();
        b_sorted.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());
        for (bx, by, sprite) in b_sorted {
            if let Some(s) = sprite {
                let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y, zoom);
                s.blit_center(&mut buf, sx, sy);
            }
        }

        // chimney smoke rises from houses
        if let Some(sm) = &smoke_s {
            for (i, (bx, by, _)) in buildings.iter().enumerate().take(3) {
                let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y, zoom);
                let drift = ((tick + i as u64 * 7) % 30) as i32;
                sm.blit(&mut buf, sx - 6, sy - 40 - drift);
            }
        }

        // windmill: animated blades (frame cycles every 10 ticks)
        let mill_frame = &mill_frames[((tick / 10) % 4) as usize];
        let (msx, msy) = iso_to_screen_cam(10.0, 9.0, cam_x, cam_y, zoom);
        mill_frame.blit_center(&mut buf, msx, msy);

        // food stock indicator at market
        let food_stock = world.query::<villager::FoodSource>().first()
            .and_then(|f| world.get_component::<villager::FoodSource>(*f))
            .map(|f| f.servings).unwrap_or(0);
        let (fsx, fsy) = iso_to_screen_cam(8.0, 7.0, cam_x, cam_y, zoom);
        // stock bar under the market
        buf.rect(fsx - 20, fsy + 10, 40, 4, 60, 40, 40);
        let fw = ((food_stock as f32 / 500.0) * 40.0) as i32;
        buf.rect(fsx - 20, fsy + 10, fw, 4, 80, 200, 80);
        draw_text(&mut buf, fsx - 20, fsy + 16, &format!("food: {}", food_stock), 200, 200, 150);

        // relationship lines between bonded villagers
        for (i, va) in villagers.iter().enumerate() {
            for (j, vb) in villagers.iter().enumerate() {
                if j > i {
                    if let (Some(pa), Some(pb)) = (world.get_component::<Pos>(*va), world.get_component::<Pos>(*vb)) {
                        if let Some(rel) = world.get_component::<Relationship>(*va) {
                            if rel.other == *vb {
                                let (ax, ay) = iso_to_screen_cam(pa.x, pa.y, cam_x, cam_y, zoom);
                                let (bx, by) = iso_to_screen_cam(pb.x, pb.y, cam_x, cam_y, zoom);
                                let b = rel.strength;
                                let (lr, lg, lb) = if b > 0.3 { (80, 220, 80) } else if b < -0.3 { (255, 80, 80) } else { (150, 150, 150) };
                                // thin line
                                let steps = ((bx - ax).abs().max((by - ay).abs())).max(1);
                                for step in 0..=steps {
                                    let lx = ax + (bx - ax) * step / steps;
                                    let ly = ay + (by - ay) * step / steps;
                                    buf.set(lx, ly, lr, lg, lb);
                                }
                            }
                        }
                    }
                }
            }
        }

        // villagers as sprites (sorted by y for depth)
        let mut v_sorted: Vec<(usize, f32, f32)> = villagers.iter().enumerate()
            .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (i, p.x, p.y)))
            .collect();
        v_sorted.sort_by(|a, b| (a.1 + a.2).partial_cmp(&(b.1 + b.2)).unwrap());
        for (i, wx, wy) in v_sorted {
            let (sx, sy) = iso_to_screen_cam(wx, wy, cam_x, cam_y, zoom);
            villager_sprites[i].blit_scaled(&mut buf, sx - 32, sy - 60, 2);
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
            draw_text(&mut buf, sx - 14, sy - 76, names[i], 255, 255, 255);
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
        if let Some(pet) = world.get_component::<Pet>(dog) {
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
        for (i, rv) in rivals.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*rv) {
                let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                // rival sprite tinted different
                if let Some(rs) = &player_s {
                    rs.blit_scaled(&mut buf, sx - 32, sy - 60, 2);
                }
                draw_text(&mut buf, sx - 14, sy - 76, rival_names[i], 255, 140, 140);
            }
        }

        // player as sprite (highlighted)
        let (px, py) = iso_to_screen_cam(10.0, 10.0, cam_x, cam_y, zoom);
        if let Some(ps) = &player_s {
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
        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        let members = world.get_component::<Organization>(org).unwrap().members.len();
        let guilds = world.query::<Guild>().len();
        draw_text(&mut buf, UI_X as i32 + 10, 30, &format!("Influence"), 200, 200, 200);
        draw_rect(&mut buf, UI_X as i32 + 90, 30, 140, 10, 40, 40, 60);
        let infl_w = (p.influence * 50.0).min(140.0) as i32;
        draw_rect(&mut buf, UI_X as i32 + 90, 30, infl_w, 10, 255, 200, 80);
        draw_text(&mut buf, UI_X as i32 + 10, 44, &format!("Org: {} | Guilds: {} | Gold: {:.0}", members, guilds, p.resources), 200, 200, 200);

        // buttons
        let (mxf, myf) = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let (mx, my) = (mxf as i32, myf as i32);
        for (i, b) in buttons.iter().enumerate() {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            let disabled = i == 1 && world.get_component::<PlayerCharacter>(player).unwrap().influence < 0.3;
            draw_button(&mut buf, b, hover, disabled);
            if hover && !disabled {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, tooltips[i], 220, 220, 160);
            }
        }

        // inspector panel for selected villager
        if inspector_open {
            draw_rect(&mut buf, UI_X as i32 + 10, 330, 270, 110, 45, 45, 65);
            let v = villagers[selected_target];
            villager_sprites[selected_target].blit(&mut buf, UI_X as i32 + 18, 340);
            draw_text(&mut buf, UI_X as i32 + 60, 340, names[selected_target], 255, 255, 255);
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
        let org_members = world.get_component::<Organization>(org).unwrap().members.clone();
        for (i, m) in org_members.iter().take(6).enumerate() {
            // find which villager index this is
            let vi = villagers.iter().position(|v| v == m).unwrap_or(0);
            villager_sprites[vi].blit(&mut buf, UI_X as i32 + 200 + i as i32 * 14, 396);
        }

        // event log
        draw_rect(&mut buf, UI_X as i32 + 10, 450, 270, 170, 25, 25, 40);
        draw_text(&mut buf, UI_X as i32 + 14, 454, "LOG", 150, 150, 180);
        for (i, entry) in log.iter().rev().take(9).enumerate() {
            let trunc: String = entry.chars().take(44).collect();
            // color by event type: deaths red, guilds gold, joins green, weather cyan, talk white
            let (r, g, b) = if entry.contains("died") || entry.contains("dead") { (255, 100, 100) }
                else if entry.contains("Guild") || entry.contains("guild") { (255, 215, 0) }
                else if entry.contains("joined") { (120, 255, 120) }
                else if entry.contains("storm") || entry.contains("rain") || entry.contains("flu") { (120, 200, 255) }
                else { (210, 210, 210) };
            draw_text(&mut buf, UI_X as i32 + 14, 470 + i as i32 * 16, &trunc, r, g, b);
        }

        // season change flash (every 150 ticks = season cycle)
        if tick % 150 < 6 && tick > 10 {
            let flash = (6 - (tick % 150)) as f32 / 6.0 * 60.0;
            for y in 0..H { for x in 0..MAP_W {
                let idx = (y * MAP_W + x) * 4;
                if idx + 2 < buf.pixels.len() {
                    buf.pixels[idx] = (buf.pixels[idx] as f32 + flash).min(255.0) as u8;
                    buf.pixels[idx+1] = (buf.pixels[idx+1] as f32 + flash).min(255.0) as u8;
                    buf.pixels[idx+2] = (buf.pixels[idx+2] as f32 + flash).min(255.0) as u8;
                }
            }}
        }

        // day/night tint: cycle every 60 ticks
        let day_phase = (tick % 60) as f32 / 60.0;
        let brightness = 0.7 + 0.3 * (day_phase * std::f32::consts::PI * 2.0).sin().abs();
        if brightness < 0.9 {
            for y in 0..H { for x in 0..MAP_W {
                let idx = (y * MAP_W + x) * 4;
                if idx + 2 < buf.pixels.len() {
                    buf.pixels[idx]   = (buf.pixels[idx]   as f32 * brightness) as u8;
                    buf.pixels[idx+1] = (buf.pixels[idx+1] as f32 * brightness) as u8;
                    buf.pixels[idx+2] = (buf.pixels[idx+2] as f32 * (brightness * 1.05).min(1.0)) as u8;
                }
            }}
            if day_phase > 0.5 { draw_text(&mut buf, 10, 24, "NIGHT", 160, 160, 220); }
        }

        // weather tint overlay on map region
        if let Some(cl) = world.get_component::<climate::Climate>(climate) {
            if cl.rainfall_drift > 0.5 {
                // rain: dark blue tint bands on map area
                for y in (0..H).step_by(6) {
                    for x in 0..MAP_W {
                        let idx = (y * MAP_W + x) * 4;
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
            let h = terrain.heights[my][mx];
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
        buf.rect(mm_x + 20, mm_y + 20, 2, 2, 255, 255, 255); // player

        // dialog bubble over speaking villager
        if let Some((vi, ref text, until)) = dialog {
            if tick < until {
                if let Some(p) = world.get_component::<Pos>(villagers[vi]) {
                    let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                    let bw = (text.len() as i32 * 6 + 12).max(60);
                    let bx = (sx - bw / 2).max(4);
                    let by = sy - 95;
                    buf.rect(bx, by, bw, 18, 240, 240, 220);   // cream bubble
                    buf.rect(bx, by + 16, 8, 6, 240, 240, 220); // tail
                    draw_text(&mut buf, bx + 6, by + 5, text, 40, 30, 20);
                }
            } else { dialog = None; }
        }

        // toasts (big moments)
        let mut ty = 40;
        for (msg, until) in toasts.iter() {
            if tick < *until {
                let tw = msg.len() as i32 * 6 + 16;
                buf.rect(MAP_W as i32 / 2 - tw / 2, ty, tw, 20, 20, 20, 35);
                buf.rect(MAP_W as i32 / 2 - tw / 2, ty, tw, 1, 255, 215, 0);
                draw_text(&mut buf, MAP_W as i32 / 2 - tw / 2 + 8, ty + 6, msg, 255, 220, 130);
                ty += 26;
            }
        }
        toasts.retain(|(_, until)| tick < *until);

        // ambient particles: snow (winter) / dust motes (else)
        let is_winter = matches!(season, Season::Winter);
        let (pr, pg, pb, drift_speed) = if is_winter { (230, 235, 255, 2) } else { (180, 170, 140, 1) };
        for i in 0..30 {
            let px = ((tick * drift_speed + i * 37) % MAP_W as u64) as i32;
            let py = ((tick + i * 23) % H as u64) as i32;
            buf.set(px, py, pr, pg, pb);
        }

        // season banner
        draw_text(&mut buf, 10, 8, &format!("{:?}  t{}{}", season, tick,
            if paused { " [PAUSED]" } else if speed > 1 { " [4x]" } else { "" }), 255, 255, 255);

        // help overlay
        if help_open {
            buf.rect(200, 140, 420, 300, 20, 20, 35);
            buf.rect(200, 140, 420, 2, 200, 200, 220);
            draw_text_scaled(&mut buf, 290, 155, "CONTROLS", 2, 255, 220, 130);
            let lines = [
                "click button / key: take action",
                "click villager: select + inspect",
                "1-5: select villager",
                "T talk  R recruit  I invest",
                "O organize  B build  W wait",
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
        if paused {
            buf.rect(280, 200, 260, 140, 20, 20, 35);
            buf.rect(280, 200, 260, 2, 200, 200, 220);
            draw_text_scaled(&mut buf, 340, 215, "PAUSED", 2, 255, 255, 255);
            draw_text(&mut buf, 300, 255, "Space: resume", 200, 200, 200);
            draw_text(&mut buf, 300, 275, "0: speed 1x/4x", 200, 200, 200);
            draw_text(&mut buf, 300, 295, "Esc: quit", 200, 200, 200);
            draw_text(&mut buf, 300, 315, "1-5: select villager", 200, 200, 200);
        }

        // --- INPUT ---
        if window.get_mouse_down(minifb::MouseButton::Left) {
            for (i, b) in buttons.iter().enumerate() {
                if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h {
                    let msg = match i {
                        0 => {
                            let r = act_talk(&mut world, player, villagers[selected_target]);
                            rapport[selected_target] = (rapport[selected_target] + 0.15).min(1.0);
                            let concerns = ["the harvest is thin this year", "the miller takes too much", "the lord's taxes are heavy", "we need more hands for the fields", "the old feud still burns", "winter is coming"];
                            let concern = concerns[(tick as usize + selected_target) % concerns.len()];
                            dialog = Some((selected_target, concern.to_string(), tick + 60));
                            r.description
                        }
                        1 => { let r = act_recruit(&mut world, player, org, villagers[selected_target]); if r.success { format!("{} joined!", names[selected_target]) } else { "They declined.".into() } }
                        2 => { let r = act_invest(&mut world, player, 10.0); r.description }
                        3 => "You call a house meeting.".into(),
                        4 => {
                            let bx = 4.0 + (built_count % 4) as f32 * 5.0;
                            let by = 15.0 + (built_count / 4) as f32 * 4.0;
                            buildings.push((bx, by, &house_s));
                            built_count += 1;
                            format!("A house rises at ({:.0},{:.0})", bx, by)
                        }
                        _ => "You wait and observe.".into(),
                    };
                    log.push(msg);
                }
            }
            // click on map selects nearest villager
            let mut best = selected_target;
            let mut best_d = 9999.0f32;
            for (i, v) in villagers.iter().enumerate() {
                if let Some(p) = world.get_component::<Pos>(*v) {
                    let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y, zoom);
                    let d = ((sx as f32 - mx as f32).powi(2) + (sy as f32 - my as f32).powi(2)).sqrt();
                    if d < 30.0 && d < best_d { best = i; best_d = d; }
                }
            }
            selected_target = best;
        }
        if window.is_key_down(Key::Key1) { selected_target = 0; }
        if window.is_key_down(Key::Key2) { selected_target = 1; }
        if window.is_key_down(Key::Key3) { selected_target = 2; }
        if window.is_key_down(Key::Key4) { selected_target = 3; }
        if window.is_key_down(Key::Key5) { selected_target = 4; }
        if window.is_key_down(Key::T) {
            let r = act_talk(&mut world, player, villagers[selected_target]);
            rapport[selected_target] = (rapport[selected_target] + 0.15).min(1.0);
            let concerns = ["the harvest is thin this year", "the miller takes too much", "the lord's taxes are heavy", "we need more hands for the fields", "the old feud still burns", "winter is coming"];
            dialog = Some((selected_target, concerns[(tick as usize + selected_target) % concerns.len()].to_string(), tick + 60));
            log.push(r.description);
        }
        if window.is_key_down(Key::R) { let r = act_recruit(&mut world, player, org, villagers[selected_target]); if r.success { rapport[selected_target] = 1.0; } log.push(if r.success { format!("{} joined!", names[selected_target]) } else { "They declined.".into() }); }
        if window.is_key_down(Key::I) { let r = act_invest(&mut world, player, 10.0); log.push(r.description); }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { paused = !paused; }
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) { help_open = !help_open; }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { zoom = (zoom + 0.02).min(2.0); }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { zoom = (zoom - 0.02).max(0.5); }
        if window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) { speed = if speed == 1 { 4 } else { 1 }; }
        // camera pan
        if window.is_key_down(Key::Left)  { cam_x += 6; }
        if window.is_key_down(Key::Right) { cam_x -= 6; }
        if window.is_key_down(Key::Up)    { cam_y += 6; }
        if window.is_key_down(Key::Down)  { cam_y -= 6; }

        if window.is_key_down(Key::F12) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", frame_n)).ok(); frame_n += 1; }

        // blit buffer to framebuffer
        let mut fb = vec![0u32; W * H];
        for i in 0..W * H {
            let idx = i * 4;
            if idx + 2 < buf.pixels.len() {
                fb[i] = rgb(buf.pixels[idx], buf.pixels[idx+1], buf.pixels[idx+2]);
            }
        }
        window.update_with_buffer(&fb, W, H).unwrap();

        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        if p.influence > 2.0 {
            // win overlay in-window
            buf.rect(200, 240, 400, 100, 20, 20, 35);
            buf.rect(200, 240, 400, 2, 255, 215, 0);
            draw_text_scaled(&mut buf, 260, 265, "RECOGNIZED LEADER", 2, 255, 215, 0);
            draw_text(&mut buf, 290, 300, "the village is yours", 200, 200, 200);
            let mut fb = vec![0u32; W * H];
            for i in 0..W * H {
                let idx = i * 4;
                fb[i] = rgb(buf.pixels[idx], buf.pixels[idx+1], buf.pixels[idx+2]);
            }
            window.update_with_buffer(&fb, W, H).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(3));
            // game over summary screen
            let mut obuf = Buffer::new(W, H);
            obuf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
            draw_text_scaled(&mut obuf, 300, 150, "VILLAGE WON", 4, 255, 215, 0);
            draw_text(&mut obuf, 300, 240, &format!("org members: {}", members), 200, 200, 200);
            draw_text(&mut obuf, 300, 260, &format!("influence: {:.2}", p.influence), 200, 200, 200);
            draw_text(&mut obuf, 300, 280, &format!("buildings built: {}", built_count), 200, 200, 200);
            draw_text(&mut obuf, 300, 300, &format!("ticks: {}", tick), 200, 200, 200);
            draw_text(&mut obuf, 300, 340, "the wanderer became a leader", 160, 160, 180);
            let mut fb = vec![0u32; W * H];
            for i in 0..W * H {
                let idx = i * 4;
                fb[i] = rgb(obuf.pixels[idx], obuf.pixels[idx+1], obuf.pixels[idx+2]);
            }
            window.update_with_buffer(&fb, W, H).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(5));
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(80)); // ~12fps
    }
}
