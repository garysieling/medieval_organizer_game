//! Headless frames and the render benchmark (GAME2_BENCH*): per-section timings for perf work.

use super::*;

/// Copy a Buffer to the window
/// Headless verification: play the game (no input) until tick N, then
/// render that frame through the same Game::render the window uses and save
/// it as a BMP.
pub fn run_headless(art: Art, target_tick: u64, out: Option<String>, zoom: f32, cam: Option<(i32, i32)>, weather: Option<&str>, seed: u64) {
    run_headless_focus(art, target_tick, out, zoom, cam, weather, seed, None)
}

/// The View's pass marks (render.ground, render.sprites, ...): perf telemetry,
/// recorded only while a bench runs
pub(crate) type Prof = game2_core::infra::perf::Marks;

/// The bench's audio host (I4): the deterministic offline renderer -
/// the same `render_block` the live pump's core runs, so its cost is
/// the device half's cost. A missing assets dir leaves the bank empty
/// and synthesis stands in.
fn bench_audio_host() -> game2_audio::host::AudioHost {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut h = game2_audio::host::AudioHost::offline(game2_audio::wav::SAMPLE_RATE);
        let root = ["assets/sound", "../assets/sound"].iter()
            .map(std::path::Path::new)
            .find(|p| p.is_dir())
            .unwrap_or_else(|| std::path::Path::new("assets/sound"));
        h.set_bank(game2_audio::sampler::load_dir(root));
        h
    }
    // no filesystem on wasm - an empty bank, synth stands in
    #[cfg(target_arch = "wasm32")]
    { game2_audio::host::AudioHost::offline(game2_audio::wav::SAMPLE_RATE) }
}

/// Time `n` frames of what the window does each frame: update the game,
/// build the model, render it. Prints the stage means and p95/max, the
/// render passes, and the slowest sections across sim, ECS and render.
/// GAME2_BENCH_JSON=path also writes the whole run as JSON for tooling.
/// `hold_weather` draws a `--weather` sky on every frame, so its cost shows
/// (the model's sky only: the sim's weather and its log run on untouched).
pub fn bench_frames(game: &mut Game, view: &mut View, n: u32, hold_weather: bool) {
    use game2_core::infra::perf;
    // GAME2_BENCH_PAN=1: the camera slides every frame, as while panning
    // (the ground can't use its cache then)
    let pan = std::env::var("GAME2_BENCH_PAN").is_ok_and(|v| !v.is_empty());
    // GAME2_BENCH_ZOOM=1: the zoom drifts every frame, as while scrolling
    let zooming = std::env::var("GAME2_BENCH_ZOOM").is_ok_and(|v| !v.is_empty());
    let start_tick = game.tick;
    let held = (game.precip, game.rainbow_from);
    // this run's sections only, and off again after (see the end)
    perf::start();
    // the audio side of a frame (I4): an offline host runs the same
    // scan/director/compose/render the live pump does, on this thread,
    // so the audio.* sections join the table wherever a bench runs -
    // no device needed. The ear sits on the camera like game_ui's.
    let mut audio = bench_audio_host();
    let mut ablock: Vec<f32> = vec![];
    use std::time::Instant;
    let (mut tu, mut tm, mut tr) = (vec![], vec![], vec![]);
    for _ in 0..n.max(1) {
        if pan { view.cam_x += 3; view.cam_y += 1; }
        if zooming { view.zoom = (view.zoom * 1.003).min(2.0); }
        let t0 = Instant::now();
        game.update();
        let t1 = Instant::now();
        let mut m = game.ui_model();
        if hold_weather { (m.precip, m.rainbow_from) = held; }
        let t2 = Instant::now();
        let buf = view.render(&m, (-1, -1));
        let t3 = Instant::now();
        std::hint::black_box(&buf);
        view.recycle(buf);
        {
            let (tx, ty) = cam_tile(view.cam_x, view.cam_y);
            audio.push_with(&m, &game2_audio::soundscape::Listener::at(tx, ty, view.zoom));
            let n = audio.frame_samples();
            ablock.clear();
            ablock.resize(n * 2, 0.0);
            audio.render_block(&mut ablock);
            std::hint::black_box(&ablock);
        }
        tu.push((t1 - t0).as_secs_f64() * 1e3);
        tm.push((t2 - t1).as_secs_f64() * 1e3);
        tr.push((t3 - t2).as_secs_f64() * 1e3);
        perf::frame_end();
    }
    let summary = perf::stop();
    let frames = n.max(1);
    let line: Vec<String> = summary.layer("render.")
        .map(|s| format!("{} {:.1}", &s.name["render.".len()..], s.mean_per_frame)).collect();
    sayln!("render sections (ms/frame): {}", line.join(", "));
    // audio is a per-block worker beside the frame - always name all
    // five sections so the table's top-N can't hide a cheap one
    let aline: Vec<String> = summary.layer("audio.")
        .map(|s| format!("{} {:.2}", &s.name["audio.".len()..], s.mean_per_frame)).collect();
    sayln!("audio sections (ms/frame): {}", aline.join(", "));
    let (su, sm, sr) = (perf::stats("update", &tu, frames), perf::stats("model", &tm, frames), perf::stats("render", &tr, frames));
    let (precip, rainbow) = if hold_weather { held } else { (game.precip, game.rainbow_from) };
    sayln!("bench weather: precip {:?}, rainbow {:?}, season {:?}", precip, rainbow.map(|f| game.frame_n.saturating_sub(f)), game.ui_model().season);
    sayln!("bench {} frames: update {:.2}ms (max {:.1}), model {:.2}ms (max {:.1}), render {:.2}ms (max {:.1}), total {:.2}ms/frame",
        n, su.mean_per_frame, su.max, sm.mean_per_frame, sm.max, sr.mean_per_frame, sr.max,
        su.mean_per_frame + sm.mean_per_frame + sr.mean_per_frame);
    sayln!("bench p50/p95: update {:.2}/{:.2}, model {:.2}/{:.2}, render {:.2}/{:.2} ms (ticks {}..{})",
        su.p50, su.p95, sm.p50, sm.p95, sr.p50, sr.p95, start_tick, game.tick);
    // total.* hold their sim.*/ecs.* children, so they don't add up with them
    sayln!("{}", summary.table(30));
    if let Some(path) = std::env::var("GAME2_BENCH_JSON").ok().filter(|p| !p.is_empty()) {
        let json = serde_json::json!({
            "frames": frames, "ticks": [start_tick, game.tick], "zoom": view.zoom, "pan": pan, "zooming": zooming,
            "stages": [su, sm, sr],
            "sections": summary.sections.iter().map(|s| {
                let mut v = serde_json::to_value(s).unwrap_or_default();
                v["display"] = perf::display_name(s.name).into();
                v
            }).collect::<Vec<_>>(),
        });
        match std::fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default()) {
            Ok(()) => sayln!("bench json written to {}", path),
            Err(e) => eprintln!("bench json: can't write {}: {}", path, e),
        }
    }
}

/// The camera offset that puts map point (tx, ty) at the zoom pivot (the
/// view centre), so it stays centred at any zoom
pub fn cam_centred_on(tx: f32, ty: f32) -> (i32, i32) {
    let cx = ZOOM_PIVOT.0 - MAP_W as f32 / 2.0 - (tx - ty) * TILE_W as f32;
    let cy = ZOOM_PIVOT.1 + 60.0 - (tx + ty) * TILE_H as f32;
    (cx.round() as i32, cy.round() as i32)
}

/// Does a scene figure answer to `--focus <name>`? (dog, cat, chicken,
/// sheep, goat, gull, guard, priest, traveller, player, rival, villager,
/// or any PersonV.person key - baker, tax-collector, tinker...)
pub(crate) fn focus_matches(pv: &PersonV, name: &str) -> bool {
    if pv.person.as_deref() == Some(name) { return true; }
    match (pv.kind, name) {
        (PKind::Pet(Species::Dog), "dog") | (PKind::Pet(Species::Cat), "cat") => true,
        (PKind::Chicken, "chicken" | "hen") | (PKind::Sheep, "sheep") | (PKind::Goat, "goat") => true,
        (PKind::Gull, "gull") | (PKind::Guard(_), "guard") | (PKind::Priest, "priest") => true,
        (PKind::Traveller, "traveller") | (PKind::Player, "player") => true,
        (PKind::Rival(_), "rival") | (PKind::Villager(_), "villager") => true,
        _ => false,
    }
}

/// run_headless, optionally centring the camera on the first figure of a
/// kind (`--focus sheep`, or a visitor's slug: `--focus tinker`):
/// screenshot checks for new art land on it
/// without hunting for --cam coordinates
#[allow(clippy::too_many_arguments)]
pub fn run_headless_focus(art: Art, target_tick: u64, out: Option<String>, zoom: f32, cam: Option<(i32, i32)>, weather: Option<&str>, seed: u64, focus: Option<&str>) {
    let mut game = Game::new_seeded(seed);
    // GAME2_KILL=N: mark the first N villagers dying at the start, so a
    // dump can show the dead without waiting years for a natural one
    if let Some(n) = std::env::var("GAME2_KILL").ok().and_then(|v| v.parse::<usize>().ok()) {
        for &v in game.villagers.iter().take(n) {
            if let Some(h) = game.world.get_component_mut::<game2_core::sim::illness::Health>(v) {
                h.condition = 0.0;
            }
        }
    }
    let mut view = View::new(art);
    view.zoom = zoom;
    match cam {
        Some(cam) => (view.cam_x, view.cam_y) = cam,
        // no explicit camera and (below) no focus: start over this seed's
        // village instead of View::new's fixed corner, in case it moved
        None => if let Some((mx, my)) = game.ui_model().market { (view.cam_x, view.cam_y) = cam_centred_on(mx, my); },
    }
    while game.tick < target_tick { game.update(); }
    // GAME2_DUMP_EVENT=name: fire the rare event now - GAME2_DUMP_FRAMES
    // walks the figure in, --focus <slug> centres the camera on it
    if let Some(name) = std::env::var("GAME2_DUMP_EVENT").ok().filter(|v| !v.is_empty()) {
        match game2_core::session::events::event_id(&name) {
            Some(id) => { game.force_event(id); sayln!("event {} fired", name); }
            None => eprintln!("GAME2_DUMP_EVENT: no such event {}", name),
        }
    }
    // GAME2_DUMP_QUEST=name: hand the seat a parcel for the site named
    // (a country settlement, e.g. greenshaw) so a dump can show the job
    // waypoint and its edge arrow without playing to an offer
    if let Some(name) = std::env::var("GAME2_DUMP_QUEST").ok().filter(|v| !v.is_empty()) {
        use game2_core::session::quests::{PlayerQuest, QuestKind, QuestStage, QuestTarget};
        match game.sites.iter().position(|s| s.name.to_lowercase() == name.to_lowercase()) {
            Some(i) => {
                game.seats[0].quest = Some(PlayerQuest {
                    giver: game.villagers[0],
                    kind: QuestKind::Parcel,
                    target: QuestTarget::Site(i),
                    stage: QuestStage::ToCarry,
                    pickup: None,
                    expires: game.tick + 10,
                    errand: None,
                });
                let c = game.sites[i].centre;
                let cam = cam_centred_on(c.0 as f32, c.1 as f32);
                sayln!("quest: parcel to {} at {},{},--cam {},{}", game.sites[i].name, c.0, c.1, cam.0, cam.1);
            }
            None => eprintln!("GAME2_DUMP_QUEST: no site named {} (sites: {})", name,
                game.sites.iter().map(|s| s.name).collect::<Vec<_>>().join(", ")),
        }
    }
    // GAME2_KILL=N: report where the marked dead ended up, for shot checks
    if std::env::var("GAME2_KILL").is_ok() {
        eprintln!("art.tombstones={} art.graves={}", view.art.tombstones.len(), view.art.graves.len());
        let mm = game.ui_model();
        for p in mm.people.iter().filter(|p| p.dead) {
            sayln!("dead {:?} pos {:.1},{:.1} buried={} epitaph={:?}", p.kind, p.pos.0, p.pos.1, p.buried, p.epitaph);
        }
        sayln!("graveyard slots: {:?}", &mm.graveyard[..mm.graveyard.len().min(8)]);
    }
    // GAME2_DUMP_MARCH=castle|town|monastery: found a cause if need be
    // and send the column up the road to the nearest site of the kind -
    // GAME2_DUMP_FRAMES walks it out, --focus march centres on the head
    if let Some(kind) = std::env::var("GAME2_DUMP_MARCH").ok().filter(|v| !v.is_empty()) {
        use game2_core::game::campaign::{self, DecisionRule, Framing, IssueVote};
        use game2_core::session::country::SiteKind;
        let want = match kind.as_str() {
            "town" | "Town" => SiteKind::Town,
            "monastery" | "Monastery" | "abbey" | "Abbey" => SiteKind::Monastery,
            _ => SiteKind::Castle,
        };
        let org = game.seats[0].org;
        if campaign::campaign_of(&game.world, org).is_none() {
            let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
            campaign::found_campaign(&mut game.world, org, &vote,
                Framing::CommonFoe, DecisionRule::Majority, game.tick);
        }
        if let Some(o) = game.world.get_component_mut::<game2_core::sim::community::Organization>(org) {
            if o.members.is_empty() { o.members = game.villagers[..5.min(game.villagers.len())].to_vec(); }
        }
        let si = game.sites.iter().enumerate().filter(|(_, s)| s.kind == want)
            .min_by_key(|(_, s)| (s.centre.0 - game.village.0).pow(2) + (s.centre.1 - game.village.1).pow(2))
            .map(|(i, _)| i);
        match si {
            Some(si) => { let name = game.sites[si].name;
                sayln!("road march on {}: {}", name, game.force_road_march(0, si)); }
            None => eprintln!("GAME2_DUMP_MARCH: no {:?} in this seed's country", want),
        }
    }
    // GAME2_DUMP_MYSTERY=N[:K]: set mystery N doing the rounds with K
    // clues heard (default half), so the panel's dim marks can be shot
    if let Some(spec) = std::env::var("GAME2_DUMP_MYSTERY").ok().filter(|v| !v.is_empty()) {
        let mut parts = spec.split(':');
        if let Some(def) = parts.next().and_then(|v| v.parse::<usize>().ok()) {
            use game2_core::session::mystery::SeatMystery;
            let heard: u32 = parts.next().and_then(|v| v.parse().ok())
                .unwrap_or((game2_core::game::mysteries::MYSTERIES[def].clues.len() / 2) as u32);
            let mut known = 0u32;
            for i in 0..heard { known |= 1 << i; }
            game.seats[0].mystery = Some(SeatMystery { def, known, asker: game.villagers[0] });
        }
    }
    // GAME2_DUMP_FRAMES=N: play N more frames into the day, so a dump shows
    // folk where the day's plan took them (stalls up, a congregation seated)
    let more: u32 = std::env::var("GAME2_DUMP_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    for _ in 0..more { game.update(); }
    if more > 0 { let m = game.ui_model(); sayln!("day {} + {} frames: {} stalls, feast {}", m.tick, more, m.stalls.len(), m.feast); }
    // GAME2_DUMP_SELECT=N: open the inspector's page on villager N, as a
    // click on them would - screenshot checks for the card land on it
    if let Some(n) = std::env::var("GAME2_DUMP_SELECT").ok().and_then(|v| v.parse::<usize>().ok()) {
        game.seats[0].selected_target = n;
    }
    // GAME2_DUMP_TALK=N: a Talk open with villager N - GAME2_DUMP_SAYS=k
    // answers turns in first, so the box's pinned lines show grown (#1317)
    if let Some(n) = std::env::var("GAME2_DUMP_TALK").ok().and_then(|v| v.parse::<usize>().ok()) {
        use game2_core::game::villager::Pos as VPos;
        let v = game.villagers[n];
        let p = *game.world.get_component::<VPos>(v).unwrap();
        *game.world.get_component_mut::<VPos>(game.player).unwrap() = VPos { x: p.x + 0.5, y: p.y };
        game.seats[0].selected_target = n;
        game.seats[0].picked = true;
        game.command_at(0, Command::Act(0));
        let says: usize = std::env::var("GAME2_DUMP_SAYS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
        for _ in 0..says { game.command_at(0, Command::TalkPick(Some(0))); }
        sayln!("talk {}: open={} convo={} rows={}", n, game.seats[0].talk.is_some(),
            game.ui_model().convo.is_some(),
            game.ui_model().convo.map(|c| c.rows.len()).unwrap_or(0));
        // GAME2_DUMP_MOOD=1|2|3: force the open Talk's read, so the
        // icon's badge shows a state (#1320)
        if let Some(m) = std::env::var("GAME2_DUMP_MOOD").ok().and_then(|v| v.parse::<u8>().ok()) {
            use game2_core::session::dialogue::TalkMood;
            let mood = match m { 1 => Some(TalkMood::Warmed), 2 => Some(TalkMood::Guarded),
                3 => Some(TalkMood::Slighted), _ => None };
            if let (Some(mood), Some(t)) = (mood, &mut game.seats[0].talk) { t.mood = mood; }
        }
    }
    // GAME2_DUMP_BOARD=N: stand reading the market board with N long
    // postings up - the panel's wrap, cap and dismiss hint show (#1349)
    if let Some(n) = std::env::var("GAME2_DUMP_BOARD").ok().and_then(|v| v.parse::<usize>().ok()) {
        game.seats[0].notice_panel = true;
        for i in 0..n {
            game.notices.push(("writ".into(),
                format!("a fee of {} gold for every barrel the river brought, set by the steward on market day", i + 2),
                game.tick + 3));
        }
    }
    // GAME2_DUMP_WALK=dx,dy: walk the player that way for the last few
    // frames, so a dump shows the walk cycle mid-stride
    if let Some((dx, dy)) = std::env::var("GAME2_DUMP_WALK").ok().as_deref()
        .and_then(|v| v.split_once(',')).and_then(|(x, y)| Some((x.parse::<f32>().ok()?, y.parse::<f32>().ok()?))) {
        for _ in 0..8 { game.command(Command::Walk(dx, dy)); game.update(); }
        sayln!("walked ({}, {}): pose {:?}", dx, dy, game.ui_model().player_pose);
    }
    // map features too (`--focus tree|stone|wall|gate|church|smithy|guildhall|rivals|mill|town|castle|market|bridge`,
    // or a workplace's look: tavern|bakery|workshop|fold|apiary|cheese|granary|watch), for props
    // and buildings: the first of each, a tree in a wood rather than a
    // lone one
    let m = game.ui_model();
    // --focus march: centre on the column's head, wherever the road's
    // taken it
    if focus == Some("march") {
        match game.formation.as_ref().and_then(|f| f.members.first())
            .and_then(|&h| game.world.get_component::<game2_core::game::villager::Pos>(h).map(|p| (p.x, p.y))) {
            Some((x, y)) => {
                (view.cam_x, view.cam_y) = cam_centred_on(x, y);
                sayln!("focus march: tile ({:.1}, {:.1}) cam {},{}", x, y, view.cam_x, view.cam_y);
            }
            None => eprintln!("focus march: no column out at tick {}", target_tick),
        }
    }
    // --focus train: centre on the first freight wagon on the road
    if focus == Some("train") {
        match m.trains.first() {
            Some(&(tx, ty)) => {
                (view.cam_x, view.cam_y) = cam_centred_on(tx, ty);
                sayln!("focus train: tile ({:.1}, {:.1}) cam {},{}", tx, ty, view.cam_x, view.cam_y);
            }
            None => eprintln!("focus train: no freight on the road at tick {}", target_tick),
        }
    }
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
        // the country's first town or castle
        Some("town") => m.map.towns.first().map(|&(x, y, _)| (x as f32, y as f32)),
        Some("castle") => m.map.castles.first().map(|&(x, y, _)| (x as f32, y as f32)),
        Some("market") => m.market,
        Some("bridge") => m.map.bridges().next().map(|(x, y)| (x as f32, y as f32)),
        // a workplace by its look: tavern, bakery, workshop, fold, apiary...
        Some(look) => m.buildings.iter().find(|b| matches!(b.sprite, BSprite::Work(w, _) if w.file().starts_with(look)))
            .map(|b| (b.x, b.y)),
        _ => None,
    };
    if let Some((x, y)) = feature {
        (view.cam_x, view.cam_y) = cam_centred_on(x, y);
        sayln!("focus {}: tile ({:.1}, {:.1}) cam {},{}", focus.unwrap_or(""), x, y, view.cam_x, view.cam_y);
    } else if let Some(name) = focus.filter(|f| *f != "march" && *f != "train") {
        match game.ui_model().people.iter().find(|p| focus_matches(p, name)) {
            Some(p) => {
                (view.cam_x, view.cam_y) = cam_centred_on(p.pos.0, p.pos.1);
                sayln!("focus {}: tile ({:.1}, {:.1}) cam {},{} person={:?}", name, p.pos.0, p.pos.1, view.cam_x, view.cam_y, p.person);
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
            "haar" => Some(PrecipKind::Haar),
            _ => None,
        };
        game.rainbow_from = (w == "rainbow").then(|| game.frame_n.saturating_sub(100));
    }
    // GAME2_DUMP_FOLLOW=dx,dy,frames: after the camera is set, walk the
    // player and run the real follow camera each frame - the shot shows
    // the eased mid-walk framing, not a fixed snap. Prints where the
    // player lands on screen, for checks that can't read the pixels
    if let Some((dx, dy, n)) = std::env::var("GAME2_DUMP_FOLLOW").ok().as_deref()
        .and_then(|v| { let mut p = v.split(',');
            Some((p.next()?.parse::<f32>().ok()?, p.next()?.parse::<f32>().ok()?,
                p.next()?.parse::<u32>().ok()?)) }) {
        let mut fc = None;
        for _ in 0..n {
            game.command(Command::Walk(dx, dy));
            game.update();
            let m = game.ui_model();
            if let Some(c) = camera_follow(&m, (view.cam_x, view.cam_y), view.zoom, &mut fc) {
                (view.cam_x, view.cam_y) = c;
            }
        }
        let m = game.ui_model();
        if let Some(p) = m.people.iter().find(|p| p.kind == PKind::Player) {
            let (sx, sy) = iso_to_screen_cam(p.pos.0, p.pos.1, view.cam_x, view.cam_y, view.zoom);
            sayln!("follow: player tile ({:.1}, {:.1}) screen {sx},{sy} cam {},{}",
                p.pos.0, p.pos.1, view.cam_x, view.cam_y);
        }
    }
    // GAME2_BENCH=N: time N more frames (sim, model, render) and report
    if let Some(n) = std::env::var("GAME2_BENCH").ok().and_then(|v| v.parse::<u32>().ok()) {
        bench_frames(&mut game, &mut view, n, weather.is_some());
    }
    // sound captions (#1508): the same feed the window drives every
    // frame, primed once before the shot so a dump shows what live
    // play would - the persisted settings file decides if it's on
    {
        let s = crate::audio::SettingsStore::load(
            std::path::Path::new("game2.settings.json")).settings;
        let m = game.ui_model();
        let (tx, ty) = cam_tile(view.cam_x, view.cam_y);
        let ear = game2_audio::soundscape::Listener::at(tx, ty, view.zoom);
        let es = game2_audio::soundscape::scan(&m, &ear);
        view.captions.update(&es, &ear, m.frame, s.captions);
    }
    let mut buf = view.render(&game.ui_model(), (-1, -1)); // cursor off-screen: no hover
    // GAME2_DUMP_MENU=sound: draw the pause menu with the Sound overlay
    // over it (#1506) - screenshot checks for the panel land on the
    // dump, no window needed
    if std::env::var("GAME2_DUMP_MENU").ok().as_deref() == Some("sound") {
        draw_pause_menu(&mut buf, (-1, -1), "PAUSED");
        let s = game2_audio::settings::AudioSettings::default();
        draw_sound_menu(&mut buf, (-1, -1), &s, 0);
    }
    let path = out.unwrap_or_else(|| format!("evidence/frame-check-{}.bmp", target_tick));
    buf.save_bmp(&path).unwrap();
    sayln!("frame {} saved to {}", target_tick, path);
}
