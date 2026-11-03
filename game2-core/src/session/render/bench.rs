//! Headless frames and the render benchmark (GAME2_BENCH*): per-section timings for perf work.

use crate::session::*;

/// Copy a Buffer to the window
/// Headless verification: play the game (no input) until tick N, then
/// render that frame through the same Game::render the window uses and save
/// it as a BMP.
pub fn run_headless(art: Art, target_tick: u64, out: Option<String>, zoom: f32, cam: Option<(i32, i32)>, weather: Option<&str>, seed: u64) {
    run_headless_focus(art, target_tick, out, zoom, cam, weather, seed, None)
}

thread_local! {
    /// Per-section render time, summed over bench frames (bench_frames)
    static PROF: std::cell::RefCell<Vec<(&'static str, f64)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Whether render's section timers record (only while bench_frames runs)
pub(in crate::session) static PROF_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Marks section boundaries in render; only bench_frames reads them
pub(in crate::session) struct Prof(std::time::Instant);

impl Prof {
    pub(in crate::session) fn new() -> Prof { Prof(std::time::Instant::now()) }
    pub(in crate::session) fn mark(&mut self, name: &'static str) {
        if !PROF_ON.load(std::sync::atomic::Ordering::Relaxed) { return; }
        let now = std::time::Instant::now();
        let ms = (now - self.0).as_secs_f64() * 1e3;
        self.0 = now;
        PROF.with(|p| {
            let mut p = p.borrow_mut();
            match p.iter_mut().find(|e| e.0 == name) { Some(e) => e.1 += ms, None => p.push((name, ms)) }
        });
    }
}

/// Time `n` frames of what the window does each frame: update the game,
/// build the model, render it. Prints the average and worst per stage.
pub fn bench_frames(game: &mut Game, view: &mut View, n: u32) {
    // GAME2_BENCH_PAN=1: the camera slides every frame, as while panning
    // (the ground can't use its cache then)
    let pan = std::env::var("GAME2_BENCH_PAN").is_ok_and(|v| !v.is_empty());
    // GAME2_BENCH_ZOOM=1: the zoom drifts every frame, as while scrolling
    let zooming = std::env::var("GAME2_BENCH_ZOOM").is_ok_and(|v| !v.is_empty());
    // this run's sections only, and off again after (see the end)
    PROF.with(|p| p.borrow_mut().clear());
    PROF_ON.store(true, std::sync::atomic::Ordering::Relaxed);
    use std::time::Instant;
    let (mut tu, mut tm, mut tr) = (vec![], vec![], vec![]);
    for _ in 0..n.max(1) {
        if pan { view.cam_x += 3; view.cam_y += 1; }
        if zooming { view.zoom = (view.zoom * 1.003).min(2.0); }
        let t0 = Instant::now();
        game.update();
        let t1 = Instant::now();
        let m = game.ui_model();
        let t2 = Instant::now();
        let buf = view.render(&m, (-1, -1));
        let t3 = Instant::now();
        std::hint::black_box(&buf);
        view.recycle(buf);
        tu.push((t1 - t0).as_secs_f64() * 1e3);
        tm.push((t2 - t1).as_secs_f64() * 1e3);
        tr.push((t3 - t2).as_secs_f64() * 1e3);
    }
    PROF.with(|p| {
        let p = p.borrow();
        let line: Vec<String> = p.iter().map(|(k, v)| format!("{} {:.1}", k, v / n.max(1) as f64)).collect();
        println!("render sections (ms/frame): {}", line.join(", "));
    });
    PROF_ON.store(false, std::sync::atomic::Ordering::Relaxed);
    let stat = |v: &Vec<f64>| (v.iter().sum::<f64>() / v.len() as f64, v.iter().cloned().fold(0.0, f64::max));
    let ((au, wu), (am, wm), (ar, wr)) = (stat(&tu), stat(&tm), stat(&tr));
    println!("bench weather: precip {:?}, rainbow {:?}, season {:?}", game.precip, game.rainbow_from.map(|f| game.frame_n.saturating_sub(f)), game.ui_model().season);
    println!("bench {} frames: update {:.2}ms (max {:.1}), model {:.2}ms (max {:.1}), render {:.2}ms (max {:.1}), total {:.2}ms/frame",
        n, au, wu, am, wm, ar, wr, au + am + ar);
}

/// The camera offset that puts map point (tx, ty) at the zoom pivot (the
/// view centre), so it stays centred at any zoom
pub fn cam_centred_on(tx: f32, ty: f32) -> (i32, i32) {
    let cx = ZOOM_PIVOT.0 - MAP_W as f32 / 2.0 - (tx - ty) * TILE_W as f32;
    let cy = ZOOM_PIVOT.1 + 60.0 - (tx + ty) * TILE_H as f32;
    (cx.round() as i32, cy.round() as i32)
}

/// Does a scene figure answer to `--focus <name>`? (dog, cat, chicken,
/// sheep, goat, gull, guard, priest, traveller, player, rival, villager)
pub(in crate::session) fn focus_matches(kind: PKind, name: &str) -> bool {
    match (kind, name) {
        (PKind::Pet(Species::Dog), "dog") | (PKind::Pet(Species::Cat), "cat") => true,
        (PKind::Chicken, "chicken" | "hen") | (PKind::Sheep, "sheep") | (PKind::Goat, "goat") => true,
        (PKind::Gull, "gull") | (PKind::Guard(_), "guard") | (PKind::Priest, "priest") => true,
        (PKind::Traveller, "traveller") | (PKind::Player, "player") => true,
        (PKind::Rival(_), "rival") | (PKind::Villager(_), "villager") => true,
        _ => false,
    }
}

/// run_headless, optionally centring the camera on the first figure of a
/// kind (`--focus sheep`): screenshot checks for new art land on it
/// without hunting for --cam coordinates
#[allow(clippy::too_many_arguments)]
pub fn run_headless_focus(art: Art, target_tick: u64, out: Option<String>, zoom: f32, cam: Option<(i32, i32)>, weather: Option<&str>, seed: u64, focus: Option<&str>) {
    let mut game = Game::new_seeded(seed);
    let mut view = View::new(art);
    view.zoom = zoom;
    if let Some(cam) = cam { (view.cam_x, view.cam_y) = cam; }
    while game.tick < target_tick { game.update(); }
    // GAME2_DUMP_WALK=dx,dy: walk the player that way for the last few
    // frames, so a dump shows the walk cycle mid-stride
    if let Some((dx, dy)) = std::env::var("GAME2_DUMP_WALK").ok().as_deref()
        .and_then(|v| v.split_once(',')).and_then(|(x, y)| Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))) {
        for _ in 0..8 { game.command(Command::Walk(dx, dy)); game.update(); }
        println!("walked ({}, {}): pose {:?}", dx, dy, game.ui_model().player_pose);
    }
    // map features too (`--focus tree|stone|wall|gate|church|smithy|guildhall|rivals|mill`), for props
    // and buildings: the first of each, a tree in a wood rather than a
    // lone one
    let m = game.ui_model();
    let woods = |&&(x, y): &&(i32, i32)| m.map.trees.iter().filter(|&&(tx, ty)| (tx - x).abs() + (ty - y).abs() <= 2).count() >= 5;
    let feature = match focus {
        Some("tree") => m.map.trees.iter().find(woods).map(|&(x, y)| (x as f32, y as f32)),
        Some("stone") => m.stones.first().map(|&(x, y)| (x as f32, y as f32)),
        Some("wall") => m.map.walls.iter().find(|w| w.1 == WallPiece::Tower).map(|&((x, y), _)| (x as f32, y as f32)),
        Some("gate") => m.map.walls.iter().find(|w| matches!(w.1, WallPiece::GateAlongX | WallPiece::GateAlongY))
            .map(|&((x, y), _)| (x as f32, y as f32)),
        Some("church") => m.buildings.iter().find(|b| b.sprite == BSprite::Church).map(|b| (b.x, b.y)),
        Some("smithy") => m.buildings.iter().find(|b| b.sprite == BSprite::Blacksmith).map(|b| (b.x, b.y)),
        Some("guildhall") => m.buildings.iter().find(|b| b.sprite == BSprite::Guildhall).map(|b| (b.x, b.y)),
        Some("rivals") => m.buildings.iter().find(|b| b.sprite == BSprite::RivalHouse).map(|b| (b.x, b.y)),
        Some("mill") => m.mill,
        _ => None,
    };
    if let Some((x, y)) = feature {
        (view.cam_x, view.cam_y) = cam_centred_on(x, y);
        println!("focus {}: tile ({:.1}, {:.1}) cam {},{}", focus.unwrap_or(""), x, y, view.cam_x, view.cam_y);
    } else if let Some(name) = focus {
        match game.ui_model().people.iter().find(|p| focus_matches(p.kind, name)) {
            Some(p) => {
                (view.cam_x, view.cam_y) = cam_centred_on(p.pos.0, p.pos.1);
                println!("focus {}: tile ({:.1}, {:.1}) cam {},{}", name, p.pos.0, p.pos.1, view.cam_x, view.cam_y);
            }
            None => eprintln!("focus {}: nothing of that kind on the map at tick {}", name, target_tick),
        }
    }
    // force the sky for a screenshot of each kind of weather
    if let Some(w) = weather {
        game.precip = match w {
            "drizzle" => Some(PrecipKind::Drizzle), "light" => Some(PrecipKind::LightRain),
            "showers" => Some(PrecipKind::Showers), "rain" => Some(PrecipKind::Rain),
            "heavy" => Some(PrecipKind::HeavyRain), "flurries" => Some(PrecipKind::Flurries),
            "snow" => Some(PrecipKind::Snow), "blizzard" => Some(PrecipKind::HeavySnow),
            _ => None,
        };
        game.rainbow_from = (w == "rainbow").then(|| game.frame_n.saturating_sub(100));
    }
    // GAME2_BENCH=N: time N more frames (sim, model, render) and report
    if let Some(n) = std::env::var("GAME2_BENCH").ok().and_then(|v| v.parse::<u32>().ok()) {
        bench_frames(&mut game, &mut view, n);
    }
    let buf = view.render(&game.ui_model(), (-1, -1)); // cursor off-screen: no hover
    let path = out.unwrap_or_else(|| format!("evidence/frame-check-{}.bmp", target_tick));
    buf.save_bmp(&path).unwrap();
    println!("frame {} saved to {}", target_tick, path);
}
