//! Game UI: windowed front-end for the village game.
//! The simulation and rendering live in game2_core::session - this shell
//! opens a window, feeds input to the Game (local) or the server
//! (--connect ws://host:port), and blits what View draws.

use std::sync::{Arc, Mutex};
use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::render::Buffer;
use game2_client::{self as client, Art, View};
use game2_core::session::{
    self, ClientKind, ClientMsg, Command, Game, SceneCache, ServerMsg, UiModel, MAP_TILES,
};

const W: usize = session::W;
const H: usize = session::H;
const FPS: u32 = session::FPS;
const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 2.0;
/// The speed key stays while the flag's on; web builds and the server drop it
const SPEED: bool = cfg!(feature = "speed-timer");

fn present(window: &mut Window, buf: &Buffer) {
    // RGBA -> 0xRRGGBB u32 pixels for minifb, into one buffer kept across
    // frames (a fresh 5.5MB Vec every frame was a measurable cost)
    thread_local! { static PX: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) }; }
    PX.with(|px| {
        let mut px = px.borrow_mut();
        px.clear();
        px.extend(buf.pixels.chunks_exact(4).map(|c| session::rgb(c[0], c[1], c[2])));
        window.update_with_buffer(&px, W, H).unwrap();
    });
}

/// A claim token kept on disk: relaunch and you're the same wanderer (the
/// browser keeps it in localStorage)
fn claim_token() -> Option<String> {
    std::fs::read_to_string("game2.claim").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
fn save_token(t: &str) { let _ = std::fs::write("game2.claim", t); }

/// A live server connection: the ws for sends, the latest scene for draws
struct Remote {
    ws: Mutex<tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>>,
    latest: Mutex<Option<UiModel>>,
    token_seen: Mutex<Option<String>>,
    /// The server turned us away (a wrong join secret, a full village)
    denied: Mutex<Option<String>>,
    /// The roster the Welcome dealt, for the character picker
    roster: Mutex<Option<Vec<session::roster::RosterCard>>>,
}

fn connect_remote(url: &str) -> Arc<Remote> {
    let (mut ws, _) = tungstenite::connect(url).expect("can't reach the server");
    ws.send(tungstenite::Message::Text(
        serde_json::to_string(&ClientMsg::Hello { token: claim_token(), kind: ClientKind::Human, name: None,
            secret: std::env::var("GAME2_JOIN_SECRET").ok().filter(|s| !s.is_empty()) }).unwrap().into()
    )).ok();
    let remote = Arc::new(Remote {
        ws: Mutex::new(ws), latest: Mutex::new(None), token_seen: Mutex::new(None),
        denied: Mutex::new(None), roster: Mutex::new(None),
    });
    {
        let remote = remote.clone();
        std::thread::spawn(move || {
            // scenes arrive without the map (it's sent apart, when it
            // changes): the cache puts the last one back in
            let mut cache = SceneCache::new();
            let mut welcomed = false;
            loop {
                let msg = remote.ws.lock().unwrap().read();
                match msg {
                    Ok(tungstenite::Message::Text(t)) => match serde_json::from_str::<ServerMsg>(&t) {
                        Ok(msg) => {
                            if let ServerMsg::Welcome { token, roster, .. } = &msg {
                                *remote.token_seen.lock().unwrap() = Some(token.clone());
                                *remote.roster.lock().unwrap() = Some(roster.clone());
                            }
                            // a Denied before the Welcome is a refusal (we're not
                            // in); after it, it's a warning ("slow down") and play goes on
                            if let ServerMsg::Welcome { .. } = &msg { welcomed = true; }
                            if let ServerMsg::Denied { reason } = &msg {
                                if welcomed { eprintln!("game_ui: the server says: {reason}"); }
                                else { *remote.denied.lock().unwrap() = Some(reason.clone()); }
                            }
                            if let Some(scene) = cache.apply(msg) { *remote.latest.lock().unwrap() = Some(scene); }
                        }
                        Err(_) => {}
                    },
                    _ => break,
                }
            }
        });
    }
    remote
}

/// What the find box's keys did this frame
enum SearchKey { None, Enter, Escape }

/// The letters the find box types, folded to lowercase on the way in
const KEY_CHARS: [(Key, char); 26] = [
    (Key::A, 'a'), (Key::B, 'b'), (Key::C, 'c'), (Key::D, 'd'), (Key::E, 'e'),
    (Key::F, 'f'), (Key::G, 'g'), (Key::H, 'h'), (Key::I, 'i'), (Key::J, 'j'),
    (Key::K, 'k'), (Key::L, 'l'), (Key::M, 'm'), (Key::N, 'n'), (Key::O, 'o'),
    (Key::P, 'p'), (Key::Q, 'q'), (Key::R, 'r'), (Key::S, 's'), (Key::T, 't'),
    (Key::U, 'u'), (Key::V, 'v'), (Key::W, 'w'), (Key::X, 'x'), (Key::Y, 'y'),
    (Key::Z, 'z'),
];

/// The find box's keys while it has focus: letters and space type,
/// Backspace rubs out, Enter jumps the camera, Escape lets go
fn read_search(window: &Window, q: &mut String) -> SearchKey {
    if window.is_key_pressed(Key::Backspace, minifb::KeyRepeat::Yes) { q.pop(); }
    if window.is_key_pressed(Key::Space, minifb::KeyRepeat::Yes) { q.push(' '); }
    let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
    for (k, c) in KEY_CHARS {
        if window.is_key_pressed(k, minifb::KeyRepeat::Yes) {
            q.push(if shift { c.to_ascii_uppercase() } else { c });
        }
    }
    if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) { return SearchKey::Enter; }
    if window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No) { return SearchKey::Escape; }
    SearchKey::None
}

/// Map this frame's window input to Commands (shared by local and remote play)
fn read_input(window: &Window, mx: i32, my: i32, clicked: bool, cam: (i32, i32, f32), minimap_hit: bool, typing: bool) -> Vec<Command> {
    // the find box has the keys: no game input gets through while it types
    if typing { return vec![]; }
    let mut cmds: Vec<Command> = vec![];
    if clicked && !minimap_hit {
        let mut done = false;
        for (i, b) in session::action_buttons().iter().enumerate() {
            if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h { cmds.push(Command::Act(i)); done = true; }
        }
        if !done { cmds.push(Command::ClickMap { mx, my, cam_x: cam.0, cam_y: cam.1, zoom: cam.2 }); }
    }
    const KEYS: [Key; 3] = [Key::T, Key::R, Key::B];
    for (i, k) in KEYS.iter().enumerate() {
        if window.is_key_pressed(*k, minifb::KeyRepeat::No) { cmds.push(Command::Act(i)); }
    }
    for (n, k) in [Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5, Key::Key6, Key::Key7, Key::Key8].iter().enumerate() {
        if window.is_key_down(*k) { cmds.push(Command::Select(n)); }
    }
    let (mut dx, mut dy) = (0.0, 0.0);
    if window.is_key_down(Key::W) { dy -= 1.0; }
    if window.is_key_down(Key::S) { dy += 1.0; }
    if window.is_key_down(Key::A) { dx -= 1.0; }
    if window.is_key_down(Key::D) { dx += 1.0; }
    if dx != 0.0 || dy != 0.0 { cmds.push(Command::Walk(dx, dy)); }
    cmds
}

fn main() {
    // headless verification: --dump-frame N renders tick N to BMP and exits
    let args: Vec<String> = std::env::args().collect();
    let dump_at: Option<u64> = args.iter().position(|a| a == "--dump-frame")
        .and_then(|i| args.get(i+1)).and_then(|v| v.parse().ok());
    let out: Option<String> = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).cloned();
    let zoom: f32 = args.iter().position(|a| a == "--zoom").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(1.0);
    let cam: Option<(i32, i32)> = args.iter().position(|a| a == "--cam").and_then(|i| args.get(i + 1))
        .and_then(|v| v.split_once(',')).and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?)));
    let weather = args.iter().position(|a| a == "--weather").and_then(|i| args.get(i + 1)).cloned();
    // --focus sheep: centre the dump on the first figure of that kind
    let focus = args.iter().position(|a| a == "--focus").and_then(|i| args.get(i + 1)).cloned();
    let seed = args.iter().position(|a| a == "--seed").and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok()).unwrap_or_else(session::run_seed);
    // remote play: --connect ws://host:port
    let connect: Option<String> = args.iter().position(|a| a == "--connect").and_then(|i| args.get(i + 1)).cloned();

    if let Some(target) = dump_at {
        let art = Art::load_fs("assets", seed);
        client::run_headless_focus(art, target, out, zoom, cam, weather.as_deref(), seed, focus.as_deref());
        return;
    }

    let art = Art::load_fs("assets", seed);
    let mut window = Window::new("The Wanderer - Medieval Village", W, H, WindowOptions::default())
        .expect("window failed");
    let mut started = false;
    // the character picker's cards and cursor, between the title and play
    let mut picker: Option<(Vec<session::roster::RosterCard>, usize)> = None;
    let mut mouse_was_down = false;
    let frame_len = std::time::Duration::from_millis(1000 / FPS as u64);

    // remote mode: the server owns the world; this window sends Commands and
    // renders the Scenes it gets back. Same View, same render path.
    let remote = connect.map(|url| connect_remote(&url));
    // a fresh remote seat picks once the Welcome's roster lands; the
    // server only deals one to a seat it minted, so an empty roster
    // means a returning claim (or an older server) - no picker
    let mut want_remote_pick = remote.is_some();
    let mut game = if remote.is_none() { Some(Game::new_seeded(seed)) } else { None };
    let mut view = View::new(art);
    // the camera opens on the player, wherever this seed's village stands
    let mut cam_centered = false;
    // whether the first model's camera ask has been taken as the baseline:
    // whatever the world already held was asked before we were watching,
    // so a reconnect mustn't replay its zoom
    let mut focus_seeded = false;
    // which match Enter visits next in the find box
    let mut search_at = 0usize;
    // leftover edge-scroll drift too small for the whole-px camera
    let mut edge_cam = (0.0f32, 0.0f32);
    // where the player stood and the zoom last frame, for the camera's follow
    let mut player_seen: Option<((f32, f32), f32)> = None;

    // Escape is a tap, not a hold: while the find box has the keys its
    // Escape lets go of them instead of quitting
    while window.is_open() && !(window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No) && !view.search_focus && picker.is_none()) {
        let frame_at = std::time::Instant::now();
        // a refusal (a wrong join secret, a full village) ends the session
        // with the server's reason, not a silent title card
        if let Some(r) = &remote {
            if let Some(why) = r.denied.lock().unwrap().take() {
                eprintln!("game_ui: the server turned us away: {why}");
                std::process::exit(2);
            }
        }
        // the character picker stands between the title and the game
        if picker.as_ref().is_some_and(|(c, _)| c.is_empty()) { picker = None; }
        if let Some((cards, sel)) = picker.as_mut() {
            present(&mut window, &client::render_picker(&view.art, cards, *sel));
            if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)
                || window.is_key_pressed(Key::W, minifb::KeyRepeat::No) {
                *sel = (*sel + cards.len() - 1) % cards.len();
            }
            if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)
                || window.is_key_pressed(Key::S, minifb::KeyRepeat::No) {
                *sel = (*sel + 1) % cards.len();
            }
            if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) {
                let idx = *sel;
                match &remote {
                    Some(r) => {
                        let msg = serde_json::to_string(&ClientMsg::Pick(idx)).unwrap();
                        let _ = r.ws.lock().unwrap().send(tungstenite::Message::Text(msg.into()));
                    }
                    None => { if let Some(g) = game.as_mut() { g.pick_character_at(0, idx); } }
                }
                picker = None;
                started = true;
            }
            if window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No) {
                // keeping the wanderer is a decision too: tell the server,
                // or a reclaim would offer the picker over a purse earned
                // since
                if let Some(r) = &remote {
                    let msg = serde_json::to_string(&ClientMsg::DeclinePick).unwrap();
                    let _ = r.ws.lock().unwrap().send(tungstenite::Message::Text(msg.into()));
                }
                picker = None;
                started = true;
            }
            continue;
        }
        // a fresh remote seat picks once the Welcome's roster lands -
        // after the title's Enter, whenever that arrives
        if started && want_remote_pick {
            if let Some(r) = &remote {
                match r.roster.lock().unwrap().clone() {
                    Some(cards) if !cards.is_empty() => {
                        picker = Some((cards, 0));
                        want_remote_pick = false;
                        continue;
                    }
                    // the Welcome came and dealt no cards: play on
                    Some(_) => want_remote_pick = false,
                    None => {}
                }
            } else { want_remote_pick = false; }
        }
        if !started {
            present(&mut window, &client::render_title());
            if window.is_key_down(Key::Enter) {
                if remote.is_some() { started = true; }
                else {
                    let cards: Vec<session::roster::RosterCard> = session::roster::roster(seed)
                        .iter().map(session::roster::RosterCard::from).collect();
                    picker = Some((cards, 0));
                }
            }
            continue;
        }

        // persist the claim token once the server hands it over
        if let Some(r) = &remote {
            if let Some(t) = r.token_seen.lock().unwrap().take() { save_token(&t); }
        }

        let (mxf, myf) = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let (mx, my) = (mxf as i32, myf as i32);
        let mouse_down = window.get_mouse_down(minifb::MouseButton::Left);
        let clicked = mouse_down && !mouse_was_down;
        mouse_was_down = mouse_down;

        // the frame to draw: the server's latest scene, or the local game's
        let cur_model: Option<UiModel> = match (&remote, &mut game) {
            (Some(r), _) => r.latest.lock().unwrap().clone(),
            (None, Some(g)) => { g.update(); Some(g.ui_model()) }
            (None, None) => None,
        };

        // the sim asked the camera somewhere - an introduction's name:
        // centre on them and lean in, once per ask
        if let Some(m) = &cur_model {
            if !focus_seeded {
                view.focus_seen.set(m.focus.map(|f| f.2));
                focus_seeded = true;
            }
            if let Some((fx, fy, ff)) = m.focus {
                if view.focus_seen.replace(Some(ff)) != Some(ff) {
                    (view.cam_x, view.cam_y) = session::camera_on(fx, fy);
                    view.zoom = view.zoom.max(1.5);
                }
            }
        }

        if !cam_centered {
            if let Some(m) = &cur_model {
                if let Some(p) = m.people.iter().find(|p| p.kind == session::PKind::Player) {
                    (view.cam_x, view.cam_y) = session::camera_on(p.pos.0, p.pos.1);
                    cam_centered = true;
                }
            }
        }

        // the character walking clean off the map view: the camera
        // follows them back to centre, though only while they move -
        // a deliberate pan or jump with them standing still isn't fought
        if let Some(m) = &cur_model {
            if let Some(cam) = client::camera_follow(m, (view.cam_x, view.cam_y), view.zoom, &mut player_seen) {
                (view.cam_x, view.cam_y) = cam;
            }
        }

        // minimap look is a local camera move in both modes
        let mut on_minimap = false;
        if mouse_down {
            if let Some(m) = &cur_model {
                if let Some((cx, cy)) = session::minimap_look_map(&m.map, mx, my) { (view.cam_x, view.cam_y) = (cx, cy); on_minimap = true; }
            }
        }

        // the find box: clicking it takes the keys, clicking away lets go -
        // and that click picks no one
        let mut swallow_click = false;
        if clicked {
            if session::search_box_at(mx, my) { view.search_focus = true; swallow_click = true; }
            else if view.search_focus { view.search_focus = false; swallow_click = true; }
        }
        // its keys while it has them: Enter walks the camera to the next
        // who fits the query
        if view.search_focus {
            let typed = view.search_q.clone();
            match read_search(&window, &mut view.search_q) {
                SearchKey::Enter => {
                    if let Some(m) = &cur_model {
                        let hits = client::search_people(&m.people, &view.search_q);
                        if !hits.is_empty() {
                            let p = &m.people[hits[search_at % hits.len()]];
                            search_at += 1;
                            (view.cam_x, view.cam_y) = session::camera_on(p.pos.0, p.pos.1);
                        }
                    }
                }
                SearchKey::Escape => { view.search_focus = false; view.search_q.clear(); }
                SearchKey::None => {}
            }
            // a new query starts its matches over at the first
            if view.search_q != typed { search_at = 0; }
        }

        // a mouse hovering at the map view's rim drifts the camera that
        // way - gently at first, quicker the deeper it sits. The drift
        // is screen px, so it slows relative to the world as you zoom
        // in; fractions collect in edge_cam until they make a whole px.
        // Clamp pins a pointer that left the window to the rim, where it
        // would scroll forever, so the drift asks Discard where the
        // pointer honestly is; and a conversation box over the rim is
        // UI, not map, so it doesn't scroll either
        let pointer_in = window.get_mouse_pos(MouseMode::Discard).is_some();
        let convo_cover = cur_model.as_ref().and_then(|m| m.convo.as_ref()).map(|p| {
            let (_, h) = client::talk::convo_layout(p);
            mx >= client::talk::CONVO_X && mx < client::talk::CONVO_X + client::talk::CONVO_W
                && my >= client::talk::CONVO_Y && my < client::talk::CONVO_Y + h
        }).unwrap_or(false);
        if !view.search_focus && pointer_in && !convo_cover {
            let (ex, ey) = session::edge_scroll(mx, my);
            if ex != 0.0 || ey != 0.0 {
                edge_cam.0 += ex / view.zoom;
                edge_cam.1 += ey / view.zoom;
                let (dx, dy) = (edge_cam.0.trunc() as i32, edge_cam.1.trunc() as i32);
                edge_cam.0 -= dx as f32;
                edge_cam.1 -= dy as f32;
                view.cam_x += dx;
                view.cam_y += dy;
            }
        }

        let buf = match &cur_model {
            Some(m) => view.render(m, (mx, my)),
            None => { present(&mut window, &client::render_title()); continue; }
        };
        present(&mut window, &buf);

        // input -> commands -> local game or the wire
        let mut cmds = read_input(&window, mx, my, clicked, (view.cam_x, view.cam_y, view.zoom), on_minimap || swallow_click, view.search_focus);
        // a hidden button (Recruit, before there's a campaign) can't be pressed
        if let Some(m) = &cur_model { cmds.retain(|c| !matches!(c, Command::Act(i) if !session::action_shown(m, *i))); }
        if clicked && !on_minimap && !swallow_click {
            if let Some(m) = &cur_model {
                // a toast that happened somewhere: the click jumps there
                if let Some(i) = view.toast_hit(m, mx, my) {
                    if let Some(at) = m.toasts[i].at {
                        (view.cam_x, view.cam_y) = session::camera_on(at.0, at.1);
                        cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    }
                }
                let ctx_specs: Vec<session::Button> = m.ctx.iter().map(|a| session::ctx_button(*a)).collect();
                // one click, one button: overlapping rects (a shared slot
                // like Pet her / Snowballs!) answer only the topmost
                if let Some(i) = ctx_specs.iter().enumerate().rev()
                    .find(|(_, b)| mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h)
                    .map(|(i, _)| i) {
                    cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    cmds.push(Command::CtxAct(i));
                }
                // a reply on the TALK panel
                if let Some(pick) = client::talk::talk_click(&m.talk, mx, my) {
                    cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    cmds.push(Command::TalkPick(pick));
                }
                // the conversation box: a speaker's line selects them, a
                // choice button answers it (a reply, leave, or the call)
                if let Some(c) = &m.convo {
                    if let Some(click) = client::talk::convo_click(c, mx, my) {
                        cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                        match click {
                            client::talk::ConvoClick::Speaker(i) => cmds.push(Command::Select(i)),
                            client::talk::ConvoClick::Reply(n) => cmds.push(Command::TalkPick(n)),
                            client::talk::ConvoClick::Decide(i) => cmds.push(Command::Decide(i)),
                        }
                    }
                }
            }
        }
        match (&remote, &mut game) {
            (Some(r), _) => {
                for c in cmds {
                    r.ws.lock().unwrap().send(tungstenite::Message::Text(
                        serde_json::to_string(&ClientMsg::Command(c)).unwrap().into())).ok();
                }
            }
            (None, Some(g)) => for c in cmds { g.command(c); },
            (None, None) => {}
        }
        // pause and speed are solo-only: the shared world runs at one pace
        if remote.is_none() {
            if let Some(g) = &mut game {
                if !view.search_focus && window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { g.paused = !g.paused; }
                if SPEED && !view.search_focus && window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) { g.speed = if g.speed == 1 { 4 } else { 1 }; }
            }
        }
        if !view.search_focus && (window.is_key_pressed(Key::H, minifb::KeyRepeat::No)) { view.help_open = !view.help_open; }
        if !view.search_focus && (window.is_key_pressed(Key::F3, minifb::KeyRepeat::No)) { view.ledger_open = !view.ledger_open; }
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            // Scale multiplicatively so a wheel notch feels consistent at any zoom,
            // while trackpads can still make smaller, smooth adjustments.
            view.zoom = (view.zoom * 1.1_f32.powf(scroll_y)).clamp(MIN_ZOOM, MAX_ZOOM);
        }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { view.zoom = (view.zoom + 0.006).min(MAX_ZOOM); }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { view.zoom = (view.zoom - 0.006).max(MIN_ZOOM); }
        if !view.search_focus {
            if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No)  { view.cam_x += 2; }
            if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) { view.cam_x -= 2; }
            if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)    { view.cam_y += 2; }
            if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)  { view.cam_y -= 2; }
        }

        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", cur_model.as_ref().map(|m| m.frame).unwrap_or(0))).ok(); }
        // the frame's done with: the next render draws into it
        view.recycle(buf);

        let spent = frame_at.elapsed();
        if spent < frame_len { std::thread::sleep(frame_len - spent); }
    }

    let _ = MAP_TILES;
}
