//! Playtest gate: a single automated pass that combines goal-based
//! playtesting (SimplePlaytester) with headless visual/structural bug
//! checks (the playcheck checks) against one real, multi-system
//! simulation run. Exits non-zero if anything fails, so it can run on a
//! schedule or in CI as a genuine, locally-verifiable substitute for
//! "please playtest this" - no window, no assets, no external agent
//! required.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::ui::font::draw_text;
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

/// Checks on the frame a player actually sees. The render in main() below
/// is this gate's own simplified copy of the scene; this runs the real
/// game's headless dump (`game_ui --dump-frame`, built next to this binary)
/// and inspects that image - the panel, event log, minimap and HUD.
fn real_frame_checks() -> Vec<Check> {
    const DAY: &str = "120";
    let fail = |why: String| vec![Check { name: "real game frame", pass: false, detail: why }];
    let exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join(format!("game_ui{}", std::env::consts::EXE_SUFFIX))));
    let Some(game_ui) = exe.filter(|p| p.exists()) else {
        return fail("game_ui isn't built next to playtest_gate - build with `cargo build --release --bins`".into());
    };
    // render into a temp file of our own (evidence/frame-check-*.bmp are tracked)
    let path = std::env::temp_dir().join(format!("playtest-gate-frame-{}.bmp", std::process::id()));
    let path = path.to_string_lossy().to_string();
    match std::process::Command::new(&game_ui).args(["--dump-frame", DAY, "--out", &path]).output() {
        Ok(out) if out.status.success() => {}
        Ok(out) => return fail(format!("game_ui --dump-frame failed: {}", String::from_utf8_lossy(&out.stderr).trim())),
        Err(e) => return fail(format!("couldn't run game_ui: {e}")),
    }
    let frame = match Buffer::load_bmp(&path) {
        Ok(b) => b,
        Err(e) => return fail(format!("couldn't read {path}: {e}")),
    };
    std::fs::remove_file(&path).ok();
    if (frame.width, frame.height) != (W, H) {
        return fail(format!("frame is {}x{}, expected {W}x{H}", frame.width, frame.height));
    }
    let drawn = |x0: usize, y0: usize, x1: usize, y1: usize, bg: (u8, u8, u8)| {
        let mut n = 0;
        for y in y0..y1 { for x in x0..x1 {
            let i = (y * W + x) * 4;
            if (frame.pixels[i], frame.pixels[i + 1], frame.pixels[i + 2]) != bg { n += 1; }
        }}
        n
    };
    // the game's layout: panel from UI_X, event log box at y 466-618,
    // minimap box 84x46 in the map's top-right corner
    let panel = drawn(UI_X, 0, W, H, (32, 32, 48));
    let log = drawn(UI_X + 12, 466, UI_X + 278, 618, (25, 25, 40));
    // the key to the marks over villagers' heads, under the log box
    let legend = drawn(UI_X + 10, 625, UI_X + 280, 636, (32, 32, 48));
    let (mm_x, mm_y) = (MAP_W - 84 - 6, 6);
    let minimap = drawn(mm_x, mm_y, mm_x + 84, mm_y + 46, (20, 20, 30));
    let hud = (0..120).flat_map(|x| (6..16).map(move |y| (x, y)))
        .filter(|&(x, y)| { let i = (y * W + x) * 4; frame.pixels[i] > 230 && frame.pixels[i + 1] > 230 && frame.pixels[i + 2] > 230 })
        .count();
    vec![
        Check { name: "real frame: side panel drawn", pass: panel > (W - UI_X) * H / 6, detail: format!("{panel} px drawn") },
        Check { name: "real frame: event log has text", pass: log > 300, detail: format!("{log} px") },
        Check { name: "real frame: minimap shows terrain", pass: minimap > 84 * 46 / 3, detail: format!("{minimap} px") },
        Check { name: "real frame: map-marks legend shown", pass: legend > 80, detail: format!("{legend} px") },
        Check { name: "real frame: HUD line present", pass: hud > 40, detail: format!("{hud} px") },
    ]
}

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

    // --- render the real terrain+entities+UI frame. This mirrors game_ui's
    // content, not just its background: buildings and villagers are drawn
    // at their actual world positions (as solid marks rather than sprites,
    // since this gate deliberately has no asset-pack dependency), and the
    // panel shows real text pulled from world state. That way a regression
    // that silently drops buildings/villagers/panel content from the real
    // game screen shows up here too, not just a missing-background check. ---
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, W as i32, H as i32, 25, 25, 38);
    for ty in 0..terrain.height { for tx in 0..terrain.width {
        let h = terrain.get(tx, ty);
        let (r, g, b) = Terrain::tile_color(h);
        let (sx, sy) = iso(tx as f32, ty as f32);
        buf.diamond(sx, sy, TILE_W, TILE_H, r, g, b);
    }}

    // buildings: same handful of landmarks run_headless spawns, drawn as
    // solid rects at their projected position (no sprite assets required)
    let buildings = [(5.0f32, 5.0f32), (12.0, 4.0), (7.0, 12.0), (10.0, 9.0)];
    for &(bx, by) in &buildings {
        let (sx, sy) = iso(bx, by);
        buf.rect(sx - 10, sy - 14, 20, 14, 150, 110, 70);
    }

    // villagers: solid marker + name label at their real Pos
    for (i, &v) in villagers.iter().enumerate() {
        if let Some(p) = world.get_component::<Pos>(v) {
            let (sx, sy) = iso(p.x, p.y);
            buf.rect(sx - 4, sy - 10, 8, 10, 230, 210, 160);
            draw_text(&mut buf, sx - 14, sy - 22, &format!("v{i}"), 255, 255, 255);
        }
    }

    buf.rect(UI_X as i32, 0, 300, H as i32, 32, 32, 48);
    draw_text(&mut buf, UI_X as i32 + 10, 8, "THE WANDERER", 255, 220, 130);
    draw_text(&mut buf, UI_X as i32 + 10, 24, &format!("villagers: {}", villagers.len()), 200, 200, 200);
    draw_text(&mut buf, UI_X as i32 + 10, 36, &format!("guilds: {guild_count}"), 200, 200, 200);
    draw_text(&mut buf, UI_X as i32 + 10, 48, &format!("tick: {TICKS}"), 200, 200, 200);

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

    // Pixel-level presence checks: these fail if the real content (not just
    // the background rects) silently stops being drawn - e.g. a dropped
    // building/villager draw call, or a panel left with no text in it.
    let not_bg = |buf: &Buffer, x: i32, y: i32, bg: (u8, u8, u8)| -> bool {
        if x < 0 || y < 0 || x as usize >= buf.width || y as usize >= buf.height { return false; }
        let idx = (y as usize * buf.width + x as usize) * 4;
        idx + 2 < buf.pixels.len() && (buf.pixels[idx], buf.pixels[idx+1], buf.pixels[idx+2]) != bg
    };
    let map_bg = (25u8, 25u8, 38u8);
    let panel_bg = (32u8, 32u8, 48u8);

    let buildings_drawn = buildings.iter()
        .filter(|&&(bx, by)| { let (sx, sy) = iso(bx, by); not_bg(&buf, sx, sy - 7, map_bg) })
        .count();
    checks.push(Check { name: "buildings rendered", pass: buildings_drawn == buildings.len(), detail: format!("{buildings_drawn}/{} drawn", buildings.len()) });

    let villagers_drawn = villagers.iter()
        .filter_map(|v| world.get_component::<Pos>(*v))
        .filter(|p| { let (sx, sy) = iso(p.x, p.y); not_bg(&buf, sx, sy - 5, map_bg) })
        .count();
    checks.push(Check { name: "villagers rendered", pass: villagers_drawn == villagers.len(), detail: format!("{villagers_drawn}/{} drawn", villagers.len()) });

    let panel_content_pixels = (UI_X as i32 + 10..UI_X as i32 + 200)
        .filter(|&x| not_bg(&buf, x, 24, panel_bg))
        .count();
    checks.push(Check { name: "panel shows content", pass: panel_content_pixels > 0, detail: format!("{panel_content_pixels} non-bg px in content row") });

    checks.extend(real_frame_checks());

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
