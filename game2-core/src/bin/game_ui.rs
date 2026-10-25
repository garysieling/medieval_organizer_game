//! Game UI: windowed app - isometric map + clickable action panel.
//! The map renders live, buttons take actions, state updates.

use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::ecs::{Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::ui::isometric::{render_terrain, render_entity};
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::sim::illness::Health;
use game2_core::sim::psychology::AttachmentHistory;
use game2_core::sim::community::{Citizen, Organization};

const W: usize = 900;
const H: usize = 600;
const MAP_W: usize = 560;
const UI_X: usize = 580;

fn rgb(r: u8, g: u8, b: u8) -> u32 { ((r as u32) << 16) | ((g as u32) << 8) | b as u32 }

struct Button { label: &'static str, x: usize, y: usize, w: usize, h: usize, color: u32 }

fn draw_rect(buf: &mut Vec<u32>, x: usize, y: usize, w: usize, h: usize, color: u32) {
    for dy in 0..h { for dx in 0..w {
        let px = x + dx; let py = y + dy;
        if px < W && py < H { buf[py * W + px] = color; }
    }}
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

    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let mut villagers = vec![];
    for (i, n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, (i%3) as f32);
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
        Button { label: "Talk",     x: UI_X+10, y: 50,  w: 140, h: 40, color: rgb(60,60,120) },
        Button { label: "Recruit",  x: UI_X+10, y: 100, w: 140, h: 40, color: rgb(60,120,60) },
        Button { label: "Invest",   x: UI_X+10, y: 150, w: 140, h: 40, color: rgb(120,60,60) },
        Button { label: "Organize", x: UI_X+10, y: 200, w: 140, h: 40, color: rgb(120,100,40) },
        Button { label: "Build",    x: UI_X+10, y: 250, w: 140, h: 40, color: rgb(80,60,120) },
        Button { label: "Wait",     x: UI_X+10, y: 300, w: 140, h: 40, color: rgb(60,60,60) },
    ];

    let mut tick = 0u64;
    let mut selected_target = 0usize;
    let mut log: Vec<String> = vec!["Welcome to the village.".into()];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        tick += 1;
        sched.tick(&mut world);

        // --- RENDER ---
        let mut fb = vec![0u32; W * H];
        draw_rect(&mut fb, 0, 0, W, H, rgb(20,20,30));
        let board = render_terrain(&terrain, MAP_W, H);
        for y in 0..H { for x in 0..MAP_W {
            if x < board.width && y < board.height {
                let idx = (y * board.width + x) * 4;
                if idx + 3 < board.pixels.len() {
                    let r = board.pixels[idx];
                    let g = board.pixels[idx+1];
                    let b = board.pixels[idx+2];
                    if !(r == 0 && g == 0 && b == 0) {
                        fb[y * W + x] = rgb(r, g, b);
                    }
                }
            }
        }}
        let colors = [(255,80,80),(80,255,80),(80,80,255),(255,255,80),(255,80,255)];
        for (i, v) in villagers.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                let c = colors[i];
                let mut ent_buf = Buffer::new(MAP_W, H);
                render_entity(&mut ent_buf, p.x, p.y, c.0, c.1, c.2);
                for y in 0..H { for x in 0..MAP_W {
                    let idx = (y * MAP_W + x) * 4;
                    if idx + 3 < ent_buf.pixels.len() {
                        let r = ent_buf.pixels[idx];
                        let g = ent_buf.pixels[idx+1];
                        let b = ent_buf.pixels[idx+2];
                        if !(r == 0 && g == 0 && b == 0) {
                            fb[y * W + x] = rgb(r, g, b);
                        }
                    }
                }}
            }
        }

        // --- UI PANEL ---
        draw_rect(&mut fb, UI_X, 0, W - UI_X, H, rgb(30,30,45));
        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        let members = world.get_component::<Organization>(org).unwrap().members.len();
        let guilds = world.query::<Guild>().len();
        let cl = world.get_component::<climate::Climate>(climate).unwrap();
        draw_rect(&mut fb, UI_X+10, 10, 120, 10, rgb(40,40,60));
        let infl_w = (p.influence * 40.0).min(120.0) as usize;
        draw_rect(&mut fb, UI_X+10, 10, infl_w, 10, rgb(255,200,80));
        for i in 0..members.min(10) {
            draw_rect(&mut fb, UI_X+10+i*12, 30, 8, 8, rgb(80,255,80));
        }
        if guilds > 0 {
            draw_rect(&mut fb, UI_X+10, 380, 10, 10, rgb(255,215,0));
        }

        for b in buttons.iter() {
            draw_rect(&mut fb, b.x, b.y, b.w, b.h, b.color);
            draw_rect(&mut fb, b.x, b.y, b.w, 2, rgb(255,255,255));
            draw_rect(&mut fb, b.x, b.y+b.h-2, b.w, 2, rgb(255,255,255));
        }

        if let Some(p) = world.get_component::<Pos>(villagers[selected_target]) {
            let tx = MAP_W / 2 + ((p.x - p.y) * 4.0) as usize;
            let ty = (MAP_W / 4 + ((p.x + p.y) * 2.0) as usize).min(H - 20);
            draw_rect(&mut fb, tx, ty, 12, 4, rgb(255,255,0));
        }

        if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
            if window.get_mouse_down(minifb::MouseButton::Left) {
                for (i, b) in buttons.iter().enumerate() {
                    let (mxu, myu) = (mx as usize, my as usize);
                    if mxu >= b.x && mxu < b.x + b.w && myu >= b.y && myu < b.y + b.h {
                        let msg = match i {
                            0 => { let r = act_talk(&mut world, player, villagers[selected_target]); r.description }
                            1 => { let r = act_recruit(&mut world, player, org, villagers[selected_target]); if r.success { "They joined!".into() } else { "They declined.".into() } }
                            2 => { let r = act_invest(&mut world, player, 10.0); r.description }
                            3 => "You call a house meeting.".into(),
                            4 => "You commission a building.".into(),
                            _ => "You wait and observe.".into(),
                        };
                        log.push(format!("tick {}: {}", tick, msg));
                    }
                }
            }
            if window.get_mouse_down(minifb::MouseButton::Right) {
                selected_target = (mx as usize / 100) % 5;
            }
        }
        if window.is_key_down(Key::Key1) { selected_target = 0; }
        if window.is_key_down(Key::Key2) { selected_target = 1; }
        if window.is_key_down(Key::Key3) { selected_target = 2; }
        if window.is_key_down(Key::Key4) { selected_target = 3; }
        if window.is_key_down(Key::Key5) { selected_target = 4; }

        for (i, entry) in log.iter().rev().take(4).enumerate() {
            let w = (entry.len() as usize * 2).min(280);
            draw_rect(&mut fb, UI_X+10, 360+i*20, w, 3, rgb(200,200,200));
        }

        window.update_with_buffer(&fb, W, H).unwrap();

        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        if p.influence > 2.0 {
            println!("You are now the recognized leader!");
            break;
        }
    }
}
