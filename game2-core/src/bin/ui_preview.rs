//! Headless render of the game UI layout -> BMP for verification.
//! Same layout as game_ui.rs but renders one frame to file.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::{draw_text, draw_text_scaled};
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole};
use game2_core::game::player::PlayerCharacter;
use game2_core::game::guilds::{Merchant};
use game2_core::ui::isometric::render_terrain;
use game2_core::sim::climate;
use game2_core::sim::illness::Health;
use game2_core::sim::psychology::{AttachmentHistory, infer_style};
use game2_core::sim::community::{Citizen, Organization};

const W: usize = 960;
const H: usize = 640;
const MAP_W: usize = 640;
const UI_X: i32 = 660;
const TILE_W: i32 = 20;
const TILE_H: i32 = 10;

fn iso_to_screen(tx: f32, ty: f32) -> (i32, i32) {
    (MAP_W as i32 / 2 + ((tx - ty) * TILE_W as f32) as i32, 60 + ((tx + ty) * TILE_H as f32) as i32)
}

struct Button { label: &'static str, key: char, x: i32, y: i32, w: i32, h: i32, color: (u8,u8,u8) }

fn draw_button(buf: &mut Buffer, b: &Button) {
    buf.rect(b.x, b.y, b.w, b.h, b.color.0, b.color.1, b.color.2);
    buf.rect(b.x, b.y, b.w, 1, 255, 255, 255);
    buf.rect(b.x, b.y + b.h - 1, b.w, 1, 20, 20, 20);
    let tw = b.label.len() as i32 * 6;
    draw_text(buf, b.x + (b.w - tw) / 2, b.y + (b.h - 7) / 2, b.label, 255, 255, 255);
    let key = format!("{}", b.key);
    draw_text(buf, b.x + b.w - 10, b.y + 2, &key, 180, 180, 200);
}

fn main() {
    let mut world = World::new();
    let terrain = Terrain::generate(33, 0.7, 42);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 500);
    let assets = Assets::new("assets");

    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let vgenders = ["m", "f", "m", "m", "f"];
    let vroles = ["peasant", "worker", "peasant", "elder", "peasant"];
    let villager_sprites: Vec<Sprite> = (0..5).map(|i| {
        Sprite::load(&assets.path(&assets.villager_sprite(vroles[i], vgenders[i])))
            .unwrap().crop(0, 0, 32, 32)
    }).collect();

    let house_s = Sprite::load(&assets.path(&assets.building_sprite("house", "blue"))).ok();
    let mill_s = Sprite::load(&assets.path(&assets.building_sprite("mill", "wood"))).ok();
    let blacksmith_s = Sprite::load(&assets.path(&assets.building_sprite("blacksmith", "green"))).ok();
    let tree_s = Sprite::load(&assets.path("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Enviroument/Spring/trees/tree(1).png")).ok();

    let buildings: Vec<(f32, f32, &Option<Sprite>)> = vec![
        (5.0, 5.0, &house_s), (12.0, 4.0, &house_s), (7.0, 12.0, &house_s),
        (10.0, 9.0, &mill_s), (15.0, 8.0, &blacksmith_s),
        (3.0, 14.0, &tree_s), (18.0, 5.0, &tree_s), (9.0, 16.0, &tree_s), (20.0, 12.0, &tree_s),
    ];

    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4 + 3) as f32, (i%3 + 6) as f32);
        world.add_component(v, Hunger { value: 30.0 + i as f32 * 10.0 });
        world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
        world.add_component(v, AttachmentHistory { anxiety_episodes: i as u32, warmth_episodes: (4-i) as u32, episodes: vec![] });
        villagers.push(v);
    }
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 1.2, authority: 0.0, resources: 87.0, standing: 0.0 });
    let org = world.create_entity();
    world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![villagers[0], villagers[1]], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });

    // === RENDER ===
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);

    let board = render_terrain(&terrain, MAP_W, H);
    for y in 0..H { for x in 0..MAP_W {
        let idx = (y * MAP_W + x) * 4;
        if idx + 3 < board.pixels.len() {
            let (r, g, b) = (board.pixels[idx], board.pixels[idx+1], board.pixels[idx+2]);
            if !(r == 0 && g == 0 && b == 0) { buf.set(x as i32, y as i32, r, g, b); }
        }
    }}

    let mut b_sorted: Vec<_> = buildings.iter().collect();
    b_sorted.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());
    for (bx, by, sprite) in b_sorted {
        if let Some(s) = sprite {
            let (sx, sy) = iso_to_screen(*bx, *by);
            s.blit_center(&mut buf, sx, sy);
        }
    }

    let mut v_sorted: Vec<(usize, f32, f32)> = villagers.iter().enumerate()
        .filter_map(|(i, v)| world.get_component::<Pos>(*v).map(|p| (i, p.x, p.y)))
        .collect();
    v_sorted.sort_by(|a, b| (a.1 + a.2).partial_cmp(&(b.1 + b.2)).unwrap());
    for (i, wx, wy) in v_sorted {
        let (sx, sy) = iso_to_screen(wx, wy);
        villager_sprites[i].blit_scaled(&mut buf, sx - 32, sy - 60, 2);
        if i == 0 { buf.rect(sx - 14, sy - 66, 28, 2, 255, 255, 0); }
        draw_text(&mut buf, sx - 14, sy - 76, names[i], 255, 255, 255);
        if let Some(h) = world.get_component::<Hunger>(villagers[i]) {
            if h.value < 40.0 { draw_text(&mut buf, sx - 3, sy - 66, "!", 255, 100, 100); }
        }
    }

    let (px, py) = iso_to_screen(10.0, 10.0);
    buf.rect(px - 4, py - 8, 8, 8, 255, 255, 255);
    draw_text(&mut buf, px - 10, py - 20, "YOU", 255, 255, 0);

    // UI PANEL
    buf.rect(UI_X, 0, W as i32 - UI_X, H as i32, 32, 32, 48);
    draw_text_scaled(&mut buf, UI_X + 10, 8, "THE WANDERER", 2, 255, 220, 130);
    let p = world.get_component::<PlayerCharacter>(player).unwrap();
    let members = world.get_component::<Organization>(org).unwrap().members.len();
    draw_text(&mut buf, UI_X + 10, 30, "Influence", 200, 200, 200);
    buf.rect(UI_X + 90, 30, 140, 10, 40, 40, 60);
    buf.rect(UI_X + 90, 30, (p.influence * 50.0).min(140.0) as i32, 10, 255, 200, 80);
    draw_text(&mut buf, UI_X + 10, 44, &format!("Org: {} | Guilds: 1 | Gold: {:.0}", members, p.resources), 200, 200, 200);

    let buttons = vec![
        Button { label: "Talk",     key: 'T', x: UI_X+10, y: 60,  w: 130, h: 36, color: (60,60,120) },
        Button { label: "Recruit",  key: 'R', x: UI_X+10, y: 104, w: 130, h: 36, color: (60,120,60) },
        Button { label: "Invest",   key: 'I', x: UI_X+10, y: 148, w: 130, h: 36, color: (120,60,60) },
        Button { label: "Organize", key: 'O', x: UI_X+10, y: 192, w: 130, h: 36, color: (120,100,40) },
        Button { label: "Build",    key: 'B', x: UI_X+10, y: 236, w: 130, h: 36, color: (80,60,120) },
        Button { label: "Wait",     key: 'W', x: UI_X+10, y: 280, w: 130, h: 36, color: (60,60,60) },
    ];
    for b in buttons.iter() { draw_button(&mut buf, b); }

    // inspector
    buf.rect(UI_X + 10, 330, 270, 110, 45, 45, 65);
    let v = villagers[0];
    villager_sprites[0].blit(&mut buf, UI_X + 18, 340);
    draw_text(&mut buf, UI_X + 60, 340, names[0], 255, 255, 255);
    if let Some(h) = world.get_component::<Hunger>(v) {
        draw_text(&mut buf, UI_X + 60, 352, &format!("Hunger: {:.0}", h.value), 200, 200, 100);
    }
    if let Some(he) = world.get_component::<Health>(v) {
        draw_text(&mut buf, UI_X + 60, 364, &format!("Health: {:.0}%", he.condition * 100.0), 100, 220, 100);
    }
    if let Some(a) = world.get_component::<AttachmentHistory>(v) {
        draw_text(&mut buf, UI_X + 60, 376, &format!("Style: {:?}", infer_style(a)), 180, 180, 255);
    }
    if let Some(vil) = world.get_component::<Villager>(v) {
        draw_text(&mut buf, UI_X + 60, 388, &format!("Doing: {:?}", vil.activity), 200, 160, 160);
    }
    draw_text(&mut buf, UI_X + 10, 404, "(click villager to inspect)", 120, 120, 140);

    // log
    buf.rect(UI_X + 10, 450, 270, 170, 25, 25, 40);
    draw_text(&mut buf, UI_X + 14, 454, "LOG", 150, 150, 180);
    let log = [
        "t203: The Merchant Guild forms!",
        "t198: Eliza joined The Harvesters",
        "t195: You talked with Bram.",
        "t190: Autumn arrives. Harvest time.",
        "t185: Aldric joined The Harvesters",
        "t180: The flu spreads through the village.",
        "t172: You invested 10 gold.",
        "t160: Welcome to the village.",
    ];
    for (i, entry) in log.iter().enumerate() {
        draw_text(&mut buf, UI_X + 14, 470 + i as i32 * 18, entry, 210, 210, 210);
    }

    // minimap
    let mm_x = MAP_W as i32 - 90;
    let mm_y = 10;
    buf.rect(mm_x - 4, mm_y - 4, 84, 84, 20, 20, 30);
    buf.rect(mm_x - 4, mm_y - 4, 84, 1, 120, 120, 140);
    for my in 0..33 { for mx in 0..33 {
        let h = terrain.heights[my][mx];
        let (r, g, b) = if h < 0.35 { (40, 60, 160) } else if h < 0.6 { (60, 140, 60) } else { (160, 160, 160) };
        buf.set(mm_x + (mx * 2) as i32, mm_y + (my * 2) as i32, r, g, b);
        buf.set(mm_x + (mx * 2) as i32 + 1, mm_y + (my * 2) as i32, r, g, b);
        buf.set(mm_x + (mx * 2) as i32, mm_y + (my * 2) as i32 + 1, r, g, b);
        buf.set(mm_x + (mx * 2) as i32 + 1, mm_y + (my * 2) as i32 + 1, r, g, b);
    }}
    for v in villagers.iter() {
        if let Some(p) = world.get_component::<Pos>(*v) {
            buf.rect(mm_x + (p.x * 2.0) as i32, mm_y + (p.y * 2.0) as i32, 2, 2, 255, 80, 80);
        }
    }

    // toast
    let msg = "The Merchant Guild forms!";
    let tw = msg.len() as i32 * 6 + 16;
    buf.rect(MAP_W as i32 / 2 - tw / 2, 40, tw, 20, 20, 20, 35);
    buf.rect(MAP_W as i32 / 2 - tw / 2, 40, tw, 1, 255, 215, 0);
    draw_text(&mut buf, MAP_W as i32 / 2 - tw / 2 + 8, 46, msg, 255, 220, 130);

    draw_text(&mut buf, 10, 8, "Summer  t203", 255, 255, 255);

    buf.save_bmp("evidence/ui-preview.bmp").unwrap();
    println!("saved evidence/ui-preview.bmp");
}
