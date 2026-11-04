//! Game UI: windowed front-end for the village game.
//! The simulation and rendering live in game2_core::session - this shell
//! opens a window, feeds input to the Game (local) or the server
//! (--connect ws://host:port), and blits what View draws.

use std::sync::{Arc, Mutex};
use minifb::{Key, Window, WindowOptions, MouseMode};
use game2_core::engine::render::Buffer;
use game2_client::{self as client, Art, View};
use game2_core::session::{
    self, ClientKind, ClientMsg, Command, Game, ServerMsg, UiModel, MAP_TILES,
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
}

fn connect_remote(url: &str) -> Arc<Remote> {
    let (mut ws, _) = tungstenite::connect(url).expect("can't reach the server");
    ws.send(tungstenite::Message::Text(
        serde_json::to_string(&ClientMsg::Hello { token: claim_token(), kind: ClientKind::Human, name: None }).unwrap().into()
    )).ok();
    let remote = Arc::new(Remote {
        ws: Mutex::new(ws), latest: Mutex::new(None), token_seen: Mutex::new(None),
    });
    {
        let remote = remote.clone();
        std::thread::spawn(move || loop {
            let msg = remote.ws.lock().unwrap().read();
            match msg {
                Ok(tungstenite::Message::Text(t)) => match serde_json::from_str::<ServerMsg>(&t) {
                    Ok(ServerMsg::Scene(m)) => { *remote.latest.lock().unwrap() = Some(*m); }
                    Ok(ServerMsg::Welcome { token, .. }) => { *remote.token_seen.lock().unwrap() = Some(token); }
                    _ => {}
                },
                _ => break,
            }
        });
    }
    remote
}

/// Map this frame's window input to Commands (shared by local and remote play)
fn read_input(window: &Window, mx: i32, my: i32, clicked: bool, cam: (i32, i32, f32), minimap_hit: bool) -> Vec<Command> {
    let mut cmds: Vec<Command> = vec![];
    if clicked && !minimap_hit {
        let mut done = false;
        for (i, b) in session::action_buttons().iter().enumerate() {
            if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h { cmds.push(Command::Act(i)); done = true; }
        }
        if !done { cmds.push(Command::ClickMap { mx, my, cam_x: cam.0, cam_y: cam.1, zoom: cam.2 }); }
    }
    const KEYS: [Key; 5] = [Key::T, Key::R, Key::I, Key::O, Key::B];
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
    let mut mouse_was_down = false;
    let frame_len = std::time::Duration::from_millis(1000 / FPS as u64);

    // remote mode: the server owns the world; this window sends Commands and
    // renders the Scenes it gets back. Same View, same render path.
    let remote = connect.map(|url| connect_remote(&url));
    let mut game = if remote.is_none() { Some(Game::new_seeded(seed)) } else { None };
    let mut view = View::new(art);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_at = std::time::Instant::now();
        if !started {
            present(&mut window, &client::render_title());
            if window.is_key_down(Key::Enter) { started = true; }
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

        // minimap look is a local camera move in both modes
        let mut on_minimap = false;
        if mouse_down {
            if let Some(m) = &cur_model {
                if let Some((cx, cy)) = session::minimap_look_map(&m.map, mx, my) { (view.cam_x, view.cam_y) = (cx, cy); on_minimap = true; }
            }
        }

        let buf = match &cur_model {
            Some(m) => view.render(m, (mx, my)),
            None => { present(&mut window, &client::render_title()); continue; }
        };
        present(&mut window, &buf);

        // input -> commands -> local game or the wire
        let mut cmds = read_input(&window, mx, my, clicked, (view.cam_x, view.cam_y, view.zoom), on_minimap);
        // a hidden button (Recruit, before there's a campaign) can't be pressed
        if let Some(m) = &cur_model { cmds.retain(|c| !matches!(c, Command::Act(i) if !session::action_shown(m, *i))); }
        if clicked && !on_minimap {
            if let Some(m) = &cur_model {
                let ctx_specs: Vec<session::Button> = m.ctx.iter().map(|a| session::ctx_button(*a)).collect();
                for (i, b) in ctx_specs.iter().enumerate() {
                    if mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h {
                        cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                        cmds.push(Command::CtxAct(i));
                    }
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
                if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { g.paused = !g.paused; }
                if SPEED && window.is_key_pressed(Key::Key0, minifb::KeyRepeat::No) { g.speed = if g.speed == 1 { 4 } else { 1 }; }
            }
        }
        if window.is_key_pressed(Key::H, minifb::KeyRepeat::No) { view.help_open = !view.help_open; }
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            // Scale multiplicatively so a wheel notch feels consistent at any zoom,
            // while trackpads can still make smaller, smooth adjustments.
            view.zoom = (view.zoom * 1.1_f32.powf(scroll_y)).clamp(MIN_ZOOM, MAX_ZOOM);
        }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { view.zoom = (view.zoom + 0.006).min(MAX_ZOOM); }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { view.zoom = (view.zoom - 0.006).max(MIN_ZOOM); }
        if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No)  { view.cam_x += 2; }
        if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) { view.cam_x -= 2; }
        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)    { view.cam_y += 2; }
        if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)  { view.cam_y -= 2; }

        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", cur_model.as_ref().map(|m| m.frame).unwrap_or(0))).ok(); }
        // the frame's done with: the next render draws into it
        view.recycle(buf);

        let spent = frame_at.elapsed();
        if spent < frame_len { std::thread::sleep(frame_len - spent); }
    }

    let _ = MAP_TILES;
}
