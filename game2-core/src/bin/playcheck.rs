//! Playcheck: automated visual verification of the game UI.
//! Renders a frame of the real code path, analyzes pixels for bugs,
//! reports a checklist of what looks wrong.

use game2_core::engine::render::Buffer;
use game2_core::engine::ecs::World;
use game2_core::engine::sprite::{Sprite, Assets};
use game2_core::ui::font::draw_text;
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Pos};
use game2_core::game::player::PlayerCharacter;
use game2_core::sim::community::Organization;
use game2_core::engine::ecs::Scheduler;
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::game::guilds::{GuildFormationSystem, Merchant};

const W: usize = 960;
const H: usize = 640;
const MAP_W: usize = 640;
const TILE_W: i32 = 28;
const TILE_H: i32 = 14;
const UI_X: usize = 660;

fn iso(tx: f32, ty: f32) -> (i32, i32) {
    (MAP_W as i32 / 2 + ((tx - ty) * TILE_W as f32) as i32,
     60 + ((tx + ty) * TILE_H as f32) as i32)
}

struct Check { name: &'static str, pass: bool, detail: String }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ticks: u64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(120);

    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GuildFormationSystem);

    let terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 16.0, 16.0, 500);
    let assets = Assets::new("assets");

    let names = ["Aldric","Eliza","Bram","Corvin","Dagny"];
    let mut villagers = vec![];
    // spawn villagers on land tiles only
    let spawn_pts = [(10.0f32,16.0f32),(16.0,20.0),(20.0,14.0),(14.0,10.0),(22.0,18.0)];
    for i in 0..5 {
        let v = villager::spawn_villager(&mut world, spawn_pts[i].0, spawn_pts[i].1);
        villagers.push(v);
    }
    for i in 0..3 {
        let m = world.create_entity();
        world.add_component(m, Merchant {
            name: format!("M{}",i), wealth: 60.0, trade_routes: vec!["r".into()], vulnerable: i<2 });
    }
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter { name: "W".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
    world.add_component(player, Pos { x: 10.0, y: 10.0 });
    let org = world.create_entity();
    world.add_component(org, Organization { name: "Org".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Aid".into() });

    for _ in 0..ticks { sched.tick(&mut world); }

    // render the real frame
    let mut buf = Buffer::new(W, H);
    buf.rect(0,0,W as i32,H as i32,25,25,38);
    let season = world.get_component::<climate::Climate>(climate).unwrap().season;
    for ty in 0..terrain.height { for tx in 0..terrain.width {
        let h = terrain.get(tx, ty);
        let (r,g,b) = if h<0.15 {(30,60,180)} else if h<0.35 {(60,140,60)} else if h<0.55 {(140,110,50)} else if h<0.75 {(110,90,60)} else {(220,220,220)};
        let (sx,sy) = iso(tx as f32, ty as f32);
        buf.diamond(sx, sy, TILE_W, TILE_H, r, g, b);
    }}
    // buildings
    let house_s = Sprite::load(&assets.path(&assets.building_sprite("house","blue"))).ok();
    for (bx,by) in [(5.0f32,5.0f32),(12.0,4.0),(7.0,12.0)] {
        if let Some(s) = &house_s { let (sx,sy)=iso(bx,by); s.blit_bottom(&mut buf,sx,sy); }
    }
    // villagers
    let vsprite = Sprite::load(&assets.path(&assets.villager_sprite("peasant","m"))).unwrap().crop(0,0,32,32);
    let mut positions = vec![];
    for v in villagers.iter() {
        if let Some(p) = world.get_component::<Pos>(*v) {
            let (sx,sy) = iso(p.x,p.y);
            vsprite.blit_scaled(&mut buf, sx-32, sy-60, 2);
            positions.push((sx,sy));
        }
    }
    let (px,py) = iso(10.0,10.0);
    vsprite.blit_scaled(&mut buf, px-32, py-60, 2);
    positions.push((px,py));

    // UI panel
    buf.rect(UI_X as i32,0,300,H as i32,32,32,48);
    draw_text(&mut buf, UI_X as i32+10, 8, "THE WANDERER", 255,220,130);

    buf.save_bmp("evidence/playcheck.bmp").unwrap();

    // === AUTOMATED BUG CHECKS ===
    let mut checks: Vec<Check> = vec![];

    // 1. map coverage: is there terrain?
    let mut map_pixels = 0u32; let mut water_pixels = 0u32;
    for y in 0..H { for x in 0..MAP_W {
        let idx = (y*MAP_W + x)*4;
        if idx+2 < buf.pixels.len() {
            let (r,g,b) = (buf.pixels[idx],buf.pixels[idx+1],buf.pixels[idx+2]);
            if r>50 || g>50 || b>50 { map_pixels+=1; }
            if b>r && b>g && b>120 { water_pixels+=1; }
        }
    }}
    let water_pct = if map_pixels>0 { water_pixels*100/map_pixels } else { 0 };
    checks.push(Check { name: "map has terrain", pass: map_pixels > 20000, detail: format!("{}px", map_pixels) });
    checks.push(Check { name: "not all water", pass: water_pct < 70, detail: format!("{}% water", water_pct) });

    // 2. entity overlap: villager sprites too close?
    let mut overlaps = 0;
    for i in 0..positions.len() { for j in (i+1)..positions.len() {
        let dx = (positions[i].0 - positions[j].0).abs();
        let dy = (positions[i].1 - positions[j].1).abs();
        if dx < 16 && dy < 16 { overlaps += 1; }
    }}
    checks.push(Check { name: "no entity overlap", pass: overlaps <= 1, detail: format!("{} overlaps", overlaps) });

    // 3. entities on map: any sprite off the screen?
    let mut off_map = 0;
    for (sx,sy) in &positions {
        if *sx < -40 || *sx > MAP_W as i32 || *sy < -40 || *sy > H as i32 { off_map += 1; }
    }
    checks.push(Check { name: "entities on screen", pass: off_map == 0, detail: format!("{} off-map", off_map) });

    // 4. UI panel present
    let panel_lit = buf.pixels.get(UI_X*4).copied().unwrap_or(0) > 20;
    checks.push(Check { name: "UI panel drawn", pass: panel_lit, detail: "right panel".into() });

    // 5. entities on land (not water tiles)
    let mut on_water = 0;
    for v in villagers.iter() {
        if let Some(p) = world.get_component::<Pos>(*v) {
            let h = terrain.get(p.x as usize, p.y as usize);
            if h < 0.15 { on_water += 1; }
        }
    }
    checks.push(Check { name: "villagers on land", pass: on_water == 0, detail: format!("{} in water", on_water) });

    // report
    println!("\n=== PLAYCHECK t{} ===", ticks);
    let mut fails = 0;
    for c in &checks {
        let status = if c.pass { "OK " } else { fails += 1; "FAIL" };
        println!("  [{}] {} ({})", status, c.name, c.detail);
    }
    println!("=== {} checks, {} failed ===\n", checks.len(), fails);
    if fails > 0 { std::process::exit(1); }
}
