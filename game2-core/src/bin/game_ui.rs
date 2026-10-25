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
fn iso_to_screen_cam(tx: f32, ty: f32, cam_x: i32, cam_y: i32) -> (i32, i32) {
    let sx = MAP_W as i32 / 2 + ((tx - ty) * TILE_W as f32) as i32 + cam_x;
    let sy = 100 + ((tx + ty) * TILE_H as f32) as i32 + cam_y;
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
    let vroles = ["peasant", "worker", "peasant", "elder", "peasant"];
    let villager_sprites: Vec<Sprite> = (0..5).map(|i| {
        let path = assets.path(&assets.villager_sprite(vroles[i], vgenders[i]));
        match Sprite::load(&path) {
            Ok(s) => s.crop(0, 0, 32, 32),
            Err(e) => { println!("sprite load {}: {}", path.display(), e); Sprite { width: 1, height: 1, pixels: vec![255,0,255,255] } }
        }
    }).collect();

    // Load building sprites
    let house_s = Sprite::load(&assets.path(&assets.building_sprite("house", "blue"))).ok();
    let mill_s = Sprite::load(&assets.path(&assets.building_sprite("mill", "wood"))).ok();
    let blacksmith_s = Sprite::load(&assets.path(&assets.building_sprite("blacksmith", "green"))).ok();
    let tree_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/trees/tree(1).png")).ok();

    // buildings placed on map
    let buildings: Vec<(f32, f32, &Option<Sprite>)> = vec![
        (5.0, 5.0, &house_s), (12.0, 4.0, &house_s), (7.0, 12.0, &house_s),
        (10.0, 9.0, &mill_s), (15.0, 8.0, &blacksmith_s),
        (3.0, 14.0, &tree_s), (18.0, 5.0, &tree_s), (9.0, 16.0, &tree_s), (20.0, 12.0, &tree_s),
    ];

    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4 + 3) as f32, (i%3 + 6) as f32);
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
        world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        villagers.push(v);
    }
    for i in 0..3 {
        let m = world.create_entity();
        world.add_component(m, Merchant { name: format!("Merchant{}",i), wealth: 60.0, trade_routes: vec!["north_road".into()], vulnerable: i < 2 });
    }
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
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

    let mut tick = 0u64;
    let mut cam_x = 0i32;   // camera pan offset
    let mut cam_y = 0i32;
    let mut started = false; // title screen until Enter
    let mut paused = false;
    let mut speed = 1u32;   // 1=normal 2=fast 4=turbo
    let mut selected_target = 0usize;
    let mut log: Vec<String> = vec!["Welcome to the village.".into()];
    let mut inspector_open = true;
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
        for y in 0..H { for x in 0..MAP_W {
            let idx = (y * MAP_W + x) * 4;
            if idx + 3 < board.pixels.len() {
                let (r, g, b) = (board.pixels[idx], board.pixels[idx+1], board.pixels[idx+2]);
                if !(r == 0 && g == 0 && b == 0) {
                    buf.set(x as i32, y as i32,
                        (r as f32 * tint_r) as u8,
                        (g as f32 * tint_g) as u8,
                        (b as f32 * tint_b) as u8);
                }
            }
        }}

        // buildings on map (sorted back-to-front by tile y)
        let mut b_sorted: Vec<_> = buildings.iter().collect();
        b_sorted.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());
        for (bx, by, sprite) in b_sorted {
            if let Some(s) = sprite {
                let (sx, sy) = iso_to_screen_cam(*bx, *by, cam_x, cam_y);
                s.blit_center(&mut buf, sx, sy);
            }
        }

        // villagers as sprites (sorted by y for depth)
        let mut v_sorted: Vec<(usize, f32, f32)> = villagers.iter().enumerate()
            .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (i, p.x, p.y)))
            .collect();
        v_sorted.sort_by(|a, b| (a.1 + a.2).partial_cmp(&(b.1 + b.2)).unwrap());
        for (i, wx, wy) in v_sorted {
            let (sx, sy) = iso_to_screen_cam(wx, wy, cam_x, cam_y);
            villager_sprites[i].blit_scaled(&mut buf, sx - 32, sy - 60, 2);
            // selection highlight
            if i == selected_target {
                draw_rect(&mut buf, sx - 14, sy - 66, 28, 2, 255, 255, 0);
            }
            // name label
            draw_text(&mut buf, sx - 14, sy - 76, names[i], 255, 255, 255);
            // hunger indicator
            if let Some(h) = world.get_component::<Hunger>(villagers[i]) {
                if h.value < 40.0 {
                    draw_text(&mut buf, sx - 3, sy - 66, "!", 255, 100, 100);
                }
            }
        }

        // player marker on map
        let (px, py) = iso_to_screen_cam(10.0, 10.0, cam_x, cam_y);
        draw_rect(&mut buf, px - 4, py - 8, 8, 8, 255, 255, 255);
        draw_text(&mut buf, px - 10, py - 20, "YOU", 255, 255, 0);

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
            draw_button(&mut buf, b, hover, false);
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

        // event log
        draw_rect(&mut buf, UI_X as i32 + 10, 450, 270, 170, 25, 25, 40);
        draw_text(&mut buf, UI_X as i32 + 14, 454, "LOG", 150, 150, 180);
        for (i, entry) in log.iter().rev().take(9).enumerate() {
            let trunc: String = entry.chars().take(44).collect();
            draw_text(&mut buf, UI_X as i32 + 14, 470 + i as i32 * 16, &trunc, 210, 210, 210);
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

        // season banner
        draw_text(&mut buf, 10, 8, &format!("{:?}  t{}{}", season, tick,
            if paused { " [PAUSED]" } else if speed > 1 { " [4x]" } else { "" }), 255, 255, 255);

        // --- INPUT ---
        if window.get_mouse_down(minifb::MouseButton::Left) {
            for (i, b) in buttons.iter().enumerate() {
                if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h {
                    let msg = match i {
                        0 => { let r = act_talk(&mut world, player, villagers[selected_target]); r.description }
                        1 => { let r = act_recruit(&mut world, player, org, villagers[selected_target]); if r.success { format!("{} joined!", names[selected_target]) } else { "They declined.".into() } }
                        2 => { let r = act_invest(&mut world, player, 10.0); r.description }
                        3 => "You call a house meeting.".into(),
                        4 => "You commission a building.".into(),
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
                    let (sx, sy) = iso_to_screen_cam(p.x, p.y, cam_x, cam_y);
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
        if window.is_key_down(Key::T) { let r = act_talk(&mut world, player, villagers[selected_target]); log.push(r.description); }
        if window.is_key_down(Key::R) { let r = act_recruit(&mut world, player, org, villagers[selected_target]); log.push(if r.success { format!("{} joined!", names[selected_target]) } else { "They declined.".into() }); }
        if window.is_key_down(Key::I) { let r = act_invest(&mut world, player, 10.0); log.push(r.description); }
        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { paused = !paused; }
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
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(80)); // ~12fps
    }
}
