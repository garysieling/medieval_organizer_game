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
use game2_core::game::villager::{self, Pos, Hunger};
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
     -60 + ((tx + ty) * TILE_H as f32) as i32)
}

struct Check { name: &'static str, pass: bool, detail: String }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(42);

    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GuildFormationSystem);

    let terrain = Terrain::generate(33, 0.7, seed);
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

    const TICKS: u64 = 500;
    let mut goals: Vec<Check> = vec![];

    // --- GOAL 1: simulation survives TICKS ticks without panicking.
    // Tracks, per tick, which villagers' hunger was just reset to full by
    // SeekFoodSystem (the only place after spawn that sets it to exactly
    // 100.0), so GOAL 2 below can tell "everyone ate" from "someone ate a
    // lot" instead of inferring it from the aggregate serving count. ---
    let mut ate_at_least_once = vec![false; villagers.len()];
    // villager-ticks spent standing in water: movement isn't terrain-aware,
    // so a villager can wade through a lake and be back on land by the end
    let mut water_ticks = 0u32;
    for _ in 0..TICKS {
        sched.tick(&mut world);
        for (i, &v) in villagers.iter().enumerate() {
            if world.get_component::<Hunger>(v).map(|h| h.value >= 100.0).unwrap_or(false) {
                ate_at_least_once[i] = true;
            }
            if world.get_component::<Pos>(v).map(|p| terrain.get(p.x as usize, p.y as usize) < 0.15).unwrap_or(false) {
                water_ticks += 1;
            }
        }
    }
    goals.push(Check { name: "survives 500 ticks", pass: true, detail: "no panic".into() });

    // --- GOAL 2: every villager eats at least once (not just some
    // villagers eating repeatedly while another starves) ---
    let fed_count = ate_at_least_once.iter().filter(|&&ate| ate).count();
    let ate = fed_count == villagers.len();
    goals.push(Check { name: "every villager eats at least once", pass: ate, detail: format!("{fed_count}/{} villagers ate", villagers.len()) });

    // --- GOAL 3: a guild emerges from vulnerable merchants ---
    let guild_count = world.query::<Guild>().len();
    goals.push(Check { name: "guild emerges", pass: guild_count > 0, detail: format!("{guild_count} guild(s)") });

    // --- GOAL 4: hunger values stay within their documented 0-100 range,
    // never NaN - catches underflow/overflow regressions in HungerSystem ---
    let bad_hunger = villagers.iter()
        .filter_map(|v| world.get_component::<Hunger>(*v))
        .filter(|h| !h.value.is_finite() || h.value < 0.0 || h.value > 100.0)
        .count();
    goals.push(Check { name: "hunger stays in range", pass: bad_hunger == 0, detail: format!("{bad_hunger} out-of-range") });

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
        let idx = (y * W + x) * 4;
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

    checks.push(Check { name: "villagers stay on land", pass: water_ticks == 0, detail: format!("{water_ticks} villager-ticks in water") });

    let panel_lit = buf.pixels.get(UI_X * 4).copied().unwrap_or(0) > 20;
    checks.push(Check { name: "UI panel drawn", pass: panel_lit, detail: "right panel".into() });

    println!("\n=== PLAYTEST GATE (seed {seed}) ===\n");
    println!("-- goals --");
    let mut fails = 0;
    for c in goals.iter().chain(checks.iter()) {
        let status = if c.pass { "OK " } else { fails += 1; "FAIL" };
        println!("  [{status}] {} ({})", c.name, c.detail);
    }
    println!("\n=== {} checks, {} failed ===\n", goals.len() + checks.len(), fails);
    if fails > 0 { std::process::exit(1); }
}
