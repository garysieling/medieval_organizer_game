//! Playtest gate: a single automated pass that combines goal-based
//! playtesting (SimplePlaytester) with headless visual/structural bug
//! checks (the playcheck checks) against one real, multi-system
//! simulation run. Exits non-zero if anything fails, so it can run on a
//! schedule or in CI as a genuine, locally-verifiable substitute for
//! "please playtest this" - no window, no assets, no external agent
//! required.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Pos, FoodSource};
use game2_core::game::pets::{self, Species, PetRole};
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::sim::climate::{self, SeasonSystem};

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
    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GuildFormationSystem);

    let terrain = Terrain::generate(33, 0.7, 42);
    let _climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 16.0, 16.0, 500);

    let spawn_pts = [(10.0f32, 16.0f32), (16.0, 20.0), (20.0, 14.0), (14.0, 10.0), (22.0, 18.0)];
    let mut villagers = vec![];
    for (x, y) in spawn_pts {
        villagers.push(villager::spawn_villager(&mut world, x, y));
    }
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);

    // 2 vulnerable merchants: guild formation requires this
    for i in 0..3 {
        let m = world.create_entity();
        world.add_component(m, Merchant {
            name: format!("Merchant{i}"), wealth: 60.0, trade_routes: vec!["r".into()], vulnerable: i < 2,
        });
    }

    let mut goals: Vec<Check> = vec![];

    // --- GOAL 1: simulation survives 150 ticks without panicking ---
    for _ in 0..150 { sched.tick(&mut world); }
    goals.push(Check { name: "survives 150 ticks", pass: true, detail: "no panic".into() });

    // --- GOAL 2: villagers actually eat (food source depletes) ---
    let food_entity = world.query::<FoodSource>().first().copied();
    let servings = food_entity.and_then(|e| world.get_component::<FoodSource>(e)).map(|f| f.servings);
    let ate = servings.map(|s| s < 500).unwrap_or(false);
    goals.push(Check { name: "villagers eat", pass: ate, detail: format!("food servings: {:?}", servings) });

    // --- GOAL 3: a guild emerges from vulnerable merchants ---
    let guild_count = world.query::<Guild>().len();
    goals.push(Check { name: "guild emerges", pass: guild_count > 0, detail: format!("{guild_count} guild(s)") });

    // --- render the real terrain+UI frame (same math as playcheck/game_ui) ---
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);
    for ty in 0..terrain.height { for tx in 0..terrain.width {
        let h = terrain.get(tx, ty);
        let (r, g, b) = if h < 0.15 { (30, 60, 180) } else if h < 0.35 { (60, 140, 60) }
            else if h < 0.55 { (140, 110, 50) } else if h < 0.75 { (110, 90, 60) } else { (220, 220, 220) };
        let (sx, sy) = iso(tx as f32, ty as f32);
        buf.diamond(sx, sy, TILE_W, TILE_H, r, g, b);
    }}
    buf.rect(UI_X as i32, 0, 300, H as i32, 32, 32, 48);

    let mut checks: Vec<Check> = vec![];

    // visual/structural checks (ported from playcheck.rs)
    let mut map_pixels = 0u32; let mut water_pixels = 0u32;
    for y in 0..H { for x in 0..MAP_W {
        let idx = (y * MAP_W + x) * 4;
        if idx + 2 < buf.pixels.len() {
            let (r, g, b) = (buf.pixels[idx], buf.pixels[idx+1], buf.pixels[idx+2]);
            if r > 50 || g > 50 || b > 50 { map_pixels += 1; }
            if b > r && b > g && b > 120 { water_pixels += 1; }
        }
    }}
    let water_pct = if map_pixels > 0 { water_pixels * 100 / map_pixels } else { 0 };
    checks.push(Check { name: "map has terrain", pass: map_pixels > 20000, detail: format!("{map_pixels}px") });
    checks.push(Check { name: "not all water", pass: water_pct < 70, detail: format!("{water_pct}% water") });

    let positions: Vec<(i32, i32)> = villagers.iter()
        .filter_map(|v| world.get_component::<Pos>(*v).map(|p| iso(p.x, p.y)))
        .collect();
    let mut overlaps = 0;
    for i in 0..positions.len() { for j in (i+1)..positions.len() {
        let dx = (positions[i].0 - positions[j].0).abs();
        let dy = (positions[i].1 - positions[j].1).abs();
        if dx < 16 && dy < 16 { overlaps += 1; }
    }}
    checks.push(Check { name: "no entity pile-up", pass: overlaps <= 1, detail: format!("{overlaps} overlaps") });

    let off_map = positions.iter().filter(|(sx, sy)| *sx < -40 || *sx > MAP_W as i32 || *sy < -40 || *sy > H as i32).count();
    checks.push(Check { name: "entities on screen", pass: off_map == 0, detail: format!("{off_map} off-map") });

    let on_water = villagers.iter()
        .filter_map(|v| world.get_component::<Pos>(*v))
        .filter(|p| terrain.get(p.x as usize, p.y as usize) < 0.15)
        .count();
    checks.push(Check { name: "villagers on land", pass: on_water == 0, detail: format!("{on_water} in water") });

    let panel_lit = buf.pixels.get(UI_X * 4).copied().unwrap_or(0) > 20;
    checks.push(Check { name: "UI panel drawn", pass: panel_lit, detail: "right panel".into() });

    println!("\n=== PLAYTEST GATE ===\n");
    println!("-- goals --");
    let mut fails = 0;
    for c in goals.iter().chain(checks.iter()) {
        let status = if c.pass { "OK " } else { fails += 1; "FAIL" };
        println!("  [{status}] {} ({})", c.name, c.detail);
    }
    println!("\n=== {} checks, {} failed ===\n", goals.len() + checks.len(), fails);
    if fails > 0 { std::process::exit(1); }
}
