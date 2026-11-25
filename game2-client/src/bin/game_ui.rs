//! Game UI: windowed front-end for the village game.
//! The simulation and rendering live in game2_core::session - this shell
//! opens a window, feeds input to the Game (local) or the server
//! (--connect ws://host:port), and blits what View draws.

use std::io::Write;
use minifb::{Key, Window, WindowOptions, MouseMode, Scale, ScaleMode};
use game2_core::engine::render::Buffer;
use game2_client::{self as client, Art, EventCam, View};
use game2_client::remote::{connect_remote, save_token};
use game2_core::session::{
    self, ClientMsg, Command, ConvoPanelV, Game, UiModel, MAP_TILES,
};

const W: usize = session::W;
const H: usize = session::H;
const FPS: u32 = session::FPS;
const MIN_ZOOM: f32 = client::MIN_ZOOM;
const MAX_ZOOM: f32 = client::MAX_ZOOM;
/// The speed key stays while the flag's on; web builds and the server drop it

/// The report's context, snapped off the model the frame the key lets
/// go - the worker files with no live game to ask (#1083)
#[cfg(all(feature = "dev-voice", not(target_arch = "wasm32")))]
fn voice_ctx(m: Option<&UiModel>) -> client::voice::VoiceCtx {
    match m {
        Some(m) => client::voice::VoiceCtx {
            seed: m.seed, tick: m.tick, frame: m.frame,
            camera: m.focus.map(|(x, y, _)| (x, y)),
            // the model's log shows newest first; the report's tail
            // reads oldest to newest like the day did
            log_tail: m.log.iter().take(10).rev().map(|e| e.text.clone()).collect(),
        },
        None => client::voice::VoiceCtx { seed: 0, tick: 0, frame: 0, camera: None, log_tail: vec![] },
    }
}

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

/// The one window every screen shares: windowed by default, F11 swaps in
/// minifb's borderless screen-fill (0.29 has no real fullscreen API)
fn make_window(fullscreen: bool) -> Window {
    let options = if fullscreen {
        WindowOptions {
            borderless: true,
            scale: Scale::FitScreen,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        }
    } else {
        WindowOptions::default()
    };
    Window::new("The Wanderer - Medieval Village", W, H, options).expect("window failed")
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

/// Two T presses this close together count as a double-tap (#944)
const TALK_DBL: std::time::Duration = std::time::Duration::from_millis(350);

/// Map this frame's window input to Commands (shared by local and remote play).
/// `talk_armed` marks the last lone T press: a second one inside TALK_DBL
/// with nothing between them is a double-tap - let the pick go, don't talk
fn read_input(window: &Window, mx: i32, my: i32, clicked: bool, cam: (i32, i32, f32), minimap_hit: bool, typing: bool, talk_armed: &mut Option<std::time::Instant>, convo: Option<&ConvoPanelV>, leave: bool) -> Vec<Command> {
    // the find box has the keys: no game input gets through while it types
    if typing { return vec![]; }
    let mut cmds: Vec<Command> = vec![];
    if clicked && !minimap_hit {
        // the panel's buttons answer for themselves where the model's in
        // scope - a miss here is a map click they may still convert
        cmds.push(Command::ClickMap { mx, my, cam_x: cam.0, cam_y: cam.1, zoom: cam.2 });
    }
    // a click or a held key between the presses breaks the double-tap
    if !cmds.is_empty() { *talk_armed = None; }
    const KEYS: [Key; 1] = [Key::T];
    for (i, k) in KEYS.iter().enumerate() {
        if window.is_key_pressed(*k, minifb::KeyRepeat::No) {
            if i == 0 && talk_armed.map(|t| t.elapsed() < TALK_DBL).unwrap_or(false) {
                *talk_armed = None;
                cmds.push(Command::Unselect);
            } else {
                if i == 0 { *talk_armed = Some(std::time::Instant::now()); }
                cmds.push(Command::Act(i));
            }
        }
    }
    // the conversation box's rows are numbered: while it's open the
    // digits press them (a reply, or the call's framing) and a digit
    // with no row is swallowed, not a villager pick under the box. A
    // press, not a hold - a held key would talk the whole way through
    for (n, k) in [Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5, Key::Key6, Key::Key7, Key::Key8, Key::Key9, Key::Key0].iter().enumerate() {
        let hit = if convo.is_some() { window.is_key_pressed(*k, minifb::KeyRepeat::No) }
            else { window.is_key_down(*k) };
        if hit {
            if let Some(c) = client::talk::digit_cmd(convo, n) { cmds.push(c); }
        }
    }
    // a talk on offer takes X or Escape as your leave - the same pick
    // the "x)" row clicks through
    if leave && (window.is_key_pressed(Key::X, minifb::KeyRepeat::No)
        || window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No)) {
        cmds.push(Command::TalkPick(None));
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
    game2_core::infra::crash::install();
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
    // scene capture (#1493): --record-scenes path.jsonl appends each
    // frame's UiModel, the static map on the first line only
    let record_scenes: Option<String> = args.iter().position(|a| a == "--record-scenes")
        .and_then(|i| args.get(i + 1)).cloned();
    // --no-audio (#1494): never opens a device
    let no_audio = args.iter().any(|a| a == "--no-audio");

    if let Some(target) = dump_at {
        let art = Art::load_fs("assets", seed);
        client::run_headless_focus(art, target, out, zoom, cam, weather.as_deref(), seed, focus.as_deref());
        return;
    }

    let art = Art::load_fs("assets", seed);
    // F11 swaps the window for a borderless screen-fill and back; the
    // buffer and every screen's state ride through the recreation
    let mut fullscreen = false;
    let mut window = make_window(fullscreen);
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
    // a saved day's own record: --load replays the journal to the same
    // village, the autosave keeps it against a crash
    let load_path = args.iter().position(|a| a == "--load").and_then(|i| args.get(i + 1)).cloned();
    let mut last_saved_day = 0u64;
    let mut game = if remote.is_none() {
        if let Some(ref lp) = load_path {
            if let Ok(j) = std::fs::read_to_string(lp)
                .and_then(|s| serde_json::from_str::<game2_core::session::journal::Journal>(&s)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))) {
                // a journal from before the action set changed can't
                // replay faithfully - its Act(2)s would silently no-op.
                // Starting fresh instead would let the autosave write
                // over the saved village, so say so and stand down
                if j.version == game2_core::session::journal::JOURNAL_VERSION {
                    let frames = j.entries.last().map(|e| e.0).unwrap_or(0);
                    Some(game2_core::session::journal::replay(&j, frames))
                } else {
                    eprintln!("{lp}: a journal of version {}, but this build replays {} - the save is left alone",
                        j.version, game2_core::session::journal::JOURNAL_VERSION);
                    std::process::exit(2);
                }
            } else { Some(Game::new_seeded_cast(seed)) }
        } else { Some(Game::new_seeded_cast(seed)) }
    } else { None };
    // the prologue card stands between the picker and the game: the
    // seed's own facts, till any key lets the day begin. A remote seat's
    // facts ride the Welcome - the card shows once it lands
    let mut intro: Option<Vec<String>> = game.as_ref().map(|g| g.intro_facts()).filter(|f| !f.is_empty());
    let mut intro_wanted = remote.is_some();
    // --record-scenes: the JSONL file and its line count
    let mut scenes_out = record_scenes.map(|p|
        std::io::BufWriter::new(std::fs::File::create(&p).expect("can't create the scenes file")));
    let mut scenes_wrote = 0u64;
    // the audio host (#1494): the sink owns the device stream so it
    // must outlive the host; a failed probe or --no-audio goes silent
    #[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
    let (_audio_sink, mut audio_host) = {
        let settings = game2_audio::host::HostSettings::default();
        let mut sink = if no_audio { None } else {
            client::audio::CpalSink::probe().map_err(|e| {
                eprintln!("audio: {e} - running silent");
                e
            }).ok()
        };
        let mut host = client::audio::open_host(no_audio, &settings,
            sink.as_mut().map(|s| s as &mut dyn game2_audio::host::AudioSink));
        // A4's ambience batch: whatever assets/sound holds (missing
        // patches voice silence until the files land)
        host.set_bank(game2_audio::sampler::load_dir(std::path::Path::new("assets/sound")));
        (sink, host)
    };
    #[cfg(not(all(feature = "audio", not(target_arch = "wasm32"))))]
    let mut audio_host = { let _ = no_audio; game2_audio::host::AudioHost::silent() };
    // the persisted audio settings (#1505): game2.settings.json beside
    // game2.claim, applied now, saved a second after a change, flushed
    // on quit. The Sound menu lands with U2; nothing else writes yet.
    let mut settings_store = client::audio::SettingsStore::load(
        std::path::Path::new("game2.settings.json"));
    audio_host.apply_settings(&settings_store.settings);
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
    // the last lone T press: T-T inside it is a double-tap, an unselect
    let mut talk_armed: Option<std::time::Instant> = None;
    // where the player stood and the zoom last frame, for the camera's follow
    let mut player_seen: Option<((f32, f32), f32)> = None;
    // a sim-asked look holds the follow camera off till the player takes
    // a step - the introduction's name gets more than a frame (#1059)
    let mut follow_hold: Option<(f32, f32)> = None;
    // the event camera: which civic event has it, and the eased centre
    let mut event_cam = EventCam::default();
    // push-to-talk dev feedback (#1083): hold F9, say the bug or the
    // idea, let go - the take lands as a dev-report issue
    #[cfg(all(feature = "dev-voice", not(target_arch = "wasm32")))]
    let mut voice = client::voice::Voice::new();

    // a Talk on offer draws "x) Take your leave": Escape presses it too,
    // ahead of the pause menu - the cancel layers sit before the door
    let mut leave_offered: bool;
    // the pause menu (Esc): layered last - the find box's release, the
    // picker's decline, the intro's dismiss and the talk's leave all
    // take their Escape first. `quit` is the menu's own way out
    let mut pause_menu = false;
    let mut quit = false;

    while window.is_open() && !quit {
        let frame_at = std::time::Instant::now();
        // a refusal (a wrong join secret, a full village) ends the session
        // with the server's reason, not a silent title card
        if let Some(r) = &remote {
            if let Some(why) = r.denied.lock().unwrap().take() {
                eprintln!("game_ui: the server turned us away: {why}");
                std::process::exit(2);
            }
        }
        // F11 swaps the window for a borderless screen-fill and back on
        // every screen - the picker, title and intro cards share this one
        // window. It trips on release, not press: a fresh window can't
        // tell a still-held key from a new press and would swap straight
        // back
        if window.is_key_released(Key::F11) {
            fullscreen = !fullscreen;
            window = make_window(fullscreen);
        }
        // persist the claim token as soon as it lands: a picker or the
        // prologue can sit between it and the gameplay loop for as long
        // as the player reads - closing there shouldn't lose the seat
        if let Some(r) = &remote {
            if let Some(t) = r.token_seen.lock().unwrap().take() { save_token(&t); }
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
                // since. Solo takes the same path so the seat's settled
                // and the day's record carries it
                if let Some(r) = &remote {
                    let msg = serde_json::to_string(&ClientMsg::DeclinePick).unwrap();
                    let _ = r.ws.lock().unwrap().send(tungstenite::Message::Text(msg.into()));
                } else if let Some(g) = game.as_mut() {
                    g.decline_pick_at(0);
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
        // the prologue card, once the Welcome's dealt it (a remote seat
        // may still be waiting on it when the picker's done)
        if intro.is_none() && intro_wanted {
            if let Some(r) = &remote {
                if let Some(f) = r.intro.lock().unwrap().take() {
                    if !f.is_empty() { intro = Some(f); }
                    intro_wanted = false;
                }
            } else { intro_wanted = false; }
        }
        if let Some(facts) = &intro {
            present(&mut window, &client::render_intro(facts));
            let down = window.get_mouse_down(minifb::MouseButton::Left);
            if !window.get_keys_pressed(minifb::KeyRepeat::No).is_empty()
                || (down && !mouse_was_down) {
                intro = None;
            }
            mouse_was_down = down;
            continue;
        }

        let (mxf, myf) = window.get_mouse_pos(MouseMode::Clamp).unwrap_or((0.0, 0.0));
        let (mx, my) = (mxf as i32, myf as i32);
        let mouse_down = window.get_mouse_down(minifb::MouseButton::Left);
        let clicked = mouse_down && !mouse_was_down;
        mouse_was_down = mouse_down;

        // the frame to draw: the server's latest scene, or the local game's
        // the day's own record goes down at the boundary: the autosave
        // file writes atomic, a .prev keeps the day before it
        if let Some(g) = game.as_ref() {
            let day = g.tick;
            if day != last_saved_day {
                last_saved_day = day;
                std::fs::create_dir_all("saves").ok();
                let json = serde_json::to_string(&g.journal).unwrap_or_default();
                std::fs::rename("saves/autosave.json", "saves/autosave.prev.json").ok();
                std::fs::write("saves/autosave.tmp", json).ok()
                    .and_then(|_| std::fs::rename("saves/autosave.tmp", "saves/autosave.json").ok());
            }
        }
        let cur_model: Option<UiModel> = match (&remote, &mut game) {
            (Some(r), _) => r.latest.lock().unwrap().clone(),
            // the pause menu holds a local game still; a remote seat's
            // world runs on either way
            (None, Some(g)) => { if !pause_menu { g.update(); } Some(g.ui_model()) }
            (None, None) => None,
        };
        // --record-scenes: one JSON line per frame; the map (98% of a
        // model) rides the first line only, like the wire protocol's
        // lean scenes. sound_dump --scenes replays the file.
        if let (Some(w), Some(m)) = (&mut scenes_out, &cur_model) {
            let line = if scenes_wrote == 0 {
                serde_json::to_string(m).unwrap_or_default()
            } else {
                let mut lean = m.clone();
                lean.map = Default::default();
                serde_json::to_string(&lean).unwrap_or_default()
            };
            let _ = writeln!(w, "{line}");
            scenes_wrote += 1;
        }
        // the frame's audio intent goes down to the host at the real
        // camera - manual pans and the event cam's holds steer the ear
        // the same way they steer the view. M toggles View.mute during
        // play (never while the find box has the keys)
        if let Some(m) = &cur_model {
            let (tx, ty) = session::cam_tile(view.cam_x, view.cam_y);
            let ear = game2_audio::soundscape::Listener::at(tx, ty, view.zoom);
            let frame = game2_audio::host::AudioFrame::from_model_with(m, &ear);
            // the same scan feeds the captions (#1508): what the ear
            // hears, the eye reads
            view.captions.update(&frame.emitters, &ear, m.frame,
                settings_store.settings.captions);
            audio_host.push_frame(&frame);
        }
        audio_host.set_muted(view.mute);
        // #1505: a blurred window fades out when the settings say so,
        // and a pending settings write lands once the second's out
        audio_host.set_focused(window.is_active());
        settings_store.due(std::time::Instant::now());
        // a talk under way offers leave on either box it shows on -
        // the convo box's "x)", or the TALK panel's when a call's
        // standing over the talk
        leave_offered = cur_model.as_ref().map(|m|
            m.convo.as_ref().map(|p| p.leave).unwrap_or(false) || m.talk.in_talk).unwrap_or(false);

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
                    // the jump wins outright: follow stays off until the
                    // player character takes a step
                    follow_hold = m.people.iter()
                        .find(|p| p.kind == session::PKind::Player).map(|p| p.pos);
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

        // the walking character: inside the frame's comfort box they're
        // left alone, drifting past it leans the camera down their stride.
        // An event or a sim-asked look mutes the follow rather than losing
        // its bearings - the camera resumes eased from where it was left,
        // and a pan while they stand still is never fought (#1059)
        if let Some(m) = &cur_model {
            if let Some(hp) = follow_hold {
                let now = m.people.iter().find(|p| p.kind == session::PKind::Player).map(|p| p.pos);
                if now.map(|n| (n.0 - hp.0).abs() > 0.01 || (n.1 - hp.1).abs() > 0.01).unwrap_or(true) {
                    follow_hold = None;
                }
            }
            let muted = follow_hold.is_some() || m.player_event.is_some();
            if let Some(cam) = client::camera_follow(m, (view.cam_x, view.cam_y), view.zoom, &mut player_seen) {
                if !muted { (view.cam_x, view.cam_y) = cam; }
            }
        }

        // an event the seat's part of has the camera: a new one jumps
        // and frames itself, a live one tracks till the player takes it
        if let Some(m) = &cur_model {
            let ((nx, ny), nz) = event_cam.step(m.player_event.as_ref(),
                (view.cam_x, view.cam_y), view.zoom);
            (view.cam_x, view.cam_y) = (nx, ny);
            view.zoom = nz;
        }

        // minimap look is a local camera move in both modes
        let mut manual_cam = false;
        let mut on_minimap = false;
        if mouse_down {
            if let Some(m) = &cur_model {
                if let Some((cx, cy)) = session::minimap_look_map(&m.map, mx, my) { (view.cam_x, view.cam_y) = (cx, cy); on_minimap = true; manual_cam = true; }
            }
        }

        // the find box: clicking it takes the keys, clicking away lets go -
        // and that click picks no one. Under the pause menu a click is
        // the menu's alone
        let mut swallow_click = false;
        if clicked && !pause_menu {
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
                            manual_cam = true;
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
        let convo_cover = cur_model.as_ref().and_then(|m| m.convo.as_ref().map(|p| (m.frame, p))).map(|(frame, p)| {
            let (_, h) = client::talk::convo_layout(p, &mut view.convo_pin.borrow_mut(), frame);
            mx >= client::talk::CONVO_X && mx < client::talk::CONVO_X + client::talk::CONVO_W
                && my >= client::talk::CONVO_Y && my < client::talk::CONVO_Y + h
        }).unwrap_or(false);
        if !view.search_focus && pointer_in && !convo_cover && !pause_menu {
            let (ex, ey) = session::edge_scroll(mx, my);
            if ex != 0.0 || ey != 0.0 {
                edge_cam.0 += ex / view.zoom;
                edge_cam.1 += ey / view.zoom;
                let (dx, dy) = (edge_cam.0.trunc() as i32, edge_cam.1.trunc() as i32);
                edge_cam.0 -= dx as f32;
                edge_cam.1 -= dy as f32;
                view.cam_x += dx;
                view.cam_y += dy;
                manual_cam = true;
            }
        }

        let mut buf = match &cur_model {
            Some(m) => view.render(m, (mx, my)),
            None => { present(&mut window, &client::render_title()); continue; }
        };
        if pause_menu {
            client::draw_pause_menu(&mut buf, (mx, my),
                if remote.is_some() { "MENU" } else { "PAUSED" });
            if view.sound_menu {
                client::draw_sound_menu(&mut buf, (mx, my),
                    &settings_store.settings, view.sound_row);
            }
        }
        present(&mut window, &buf);

        // the menu owns the input while it's up: Esc or Enter resumes,
        // Q or the Quit row lets the day go - the claim token keeps.
        // The frame's Escape is read once, here, for both menu
        // decisions: a press edge stays visible across the present,
        // so a second read next frame would fire the same press twice
        // (#1324)
        let esc_now = window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No);
        if pause_menu {
            // the Sound overlay rides the menu (#1506): while it's up it
            // owns the keys, and Esc peels back to the menu rather than
            // resuming the day. Every edit lands on the live settings,
            // applies to the host, and marks the file dirty for the
            // debounced write
            if view.sound_menu {
                use client::{SndKey, SoundAct};
                let k = if esc_now { Some(SndKey::Esc) }
                    else if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)
                        || window.is_key_pressed(Key::W, minifb::KeyRepeat::No) { Some(SndKey::Up) }
                    else if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)
                        || window.is_key_pressed(Key::S, minifb::KeyRepeat::No) { Some(SndKey::Down) }
                    else if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No)
                        || window.is_key_pressed(Key::A, minifb::KeyRepeat::No) { Some(SndKey::Left) }
                    else if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No)
                        || window.is_key_pressed(Key::D, minifb::KeyRepeat::No) { Some(SndKey::Right) }
                    else if window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) { Some(SndKey::Enter) }
                    else { None };
                let act = k.map(|k| client::sound_menu_key(&mut settings_store.settings, view.sound_row, k))
                    .or_else(|| if clicked {
                        client::sound_menu_click(mx, my, &mut settings_store.settings)
                    } else { None });
                if let Some((row, act)) = act {
                    view.sound_row = row;
                    match act {
                        SoundAct::Changed(key) => {
                            audio_host.apply_settings(&settings_store.settings);
                            settings_store.touch(std::time::Instant::now());
                            // a moved Effects slider previews itself
                            if key == "effects" {
                                audio_host.push_frame(&game2_audio::host::AudioFrame {
                                    cues: vec![game2_audio::host::CueKind::Bell { pan: 0.0, gain: 0.8 }],
                                    emitters: vec![],
                                    seed: 0,
                                    listener: game2_audio::soundscape::Listener::at(0.0, 0.0, 1.0),
                                    intent: game2_audio::director::Intent::default(),
                                });
                            }
                        }
                        SoundAct::Reset => {
                            audio_host.apply_settings(&settings_store.settings);
                            settings_store.touch(std::time::Instant::now());
                        }
                        SoundAct::Close => view.sound_menu = false,
                        SoundAct::Move => {}
                    }
                }
            } else {
                if esc_now
                    || window.is_key_pressed(Key::Enter, minifb::KeyRepeat::No) { pause_menu = false; }
                if window.is_key_pressed(Key::Q, minifb::KeyRepeat::No) { quit = true; }
                if window.is_key_pressed(Key::V, minifb::KeyRepeat::No) {
                    view.sound_menu = true;
                    view.sound_row = 0;
                }
                if clicked {
                    match client::pause_menu_click(mx, my) {
                        Some(client::MenuPick::Resume) => pause_menu = false,
                        Some(client::MenuPick::Sound) => {
                            view.sound_menu = true;
                            view.sound_row = 0;
                        }
                        // the discoverable F11 (#1601): same swap the key
                        // does, menu stays up either way
                        Some(client::MenuPick::Fullscreen) => {
                            fullscreen = !fullscreen;
                            window = make_window(fullscreen);
                        }
                        Some(client::MenuPick::Quit) => quit = true,
                        None => {}
                    }
                }
            }
            view.recycle(buf);
            let spent = frame_at.elapsed();
            if spent < frame_len { std::thread::sleep(frame_len - spent); }
            continue;
        }
        // one Escape, one errand on the same edge read_input sees: the
        // find box's release is still ahead on this press, a Talk's
        // leave goes out through it, the board's postings close, a
        // conversation box that isn't a Talk just holds it, and only a
        // bare press opens the menu
        let mut esc_cmd = None;
        if esc_now && !view.search_focus {
            match cur_model.as_ref().map(client::esc_route) {
                Some(client::EscRoute::Board) => esc_cmd = Some(Command::Board(false)),
                Some(client::EscRoute::Pause) => pause_menu = true,
                // TalkLeave goes out through read_input; Swallow holds it
                _ => {}
            }
        }
        if pause_menu {
            view.recycle(buf);
            let spent = frame_at.elapsed();
            if spent < frame_len { std::thread::sleep(frame_len - spent); }
            continue;
        }

        // input -> commands -> local game or the wire
        let mut cmds = read_input(&window, mx, my, clicked, (view.cam_x, view.cam_y, view.zoom), on_minimap || swallow_click, view.search_focus, &mut talk_armed,
            cur_model.as_ref().and_then(|m| m.convo.as_ref()), leave_offered);
        // the board's Esc: a set, not a toggle - the seat's panel state
        // can't desync from what the frame showed
        if let Some(c) = esc_cmd { cmds.push(c); }
        if clicked && !on_minimap && !swallow_click {
            if let Some(m) = &cur_model {
                // a toast that happened somewhere: the click jumps there -
                // but not one hidden under the conversation box, which
                // draws on top now (convo_cover above, #1600)
                if let Some(i) = if convo_cover { None } else { view.toast_hit(m, mx, my) } {
                    if let Some(at) = m.toasts[i].at {
                        (view.cam_x, view.cam_y) = session::camera_on(at.0, at.1);
                        manual_cam = true;
                        cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    }
                }
                // a log line that knows where it happened or who it's
                // about: the click jumps there and selects them (#645)
                if let Some(i) = view.log_hit(m, mx, my) {
                    let entry = &m.log[i];
                    cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    if let Some(at) = entry.at {
                        (view.cam_x, view.cam_y) = session::camera_on(at.0, at.1);
                        manual_cam = true;
                    }
                    if let Some(who) = entry.who {
                        cmds.push(Command::Select(who));
                    }
                }
                // the same layout the draw uses - one source of truth, so
                // a click can never land on a different rect than it sees
                let ctx_specs: Vec<session::Button> = session::ctx_buttons(&m.ctx);
                // one click, one button: overlapping rects (a shared slot
                // like Pet her / Snowballs!) answer only the topmost
                // a button answer takes the ClickMap's place at the head,
                // not the tail - a Walk collected first would clear the
                // picked villager the act means to reach
                let click_slot = |cmds: &mut Vec<Command>, cmd: Command| {
                    match cmds.iter().position(|x| matches!(x, Command::ClickMap { .. })) {
                        Some(i) => cmds[i] = cmd,
                        None => cmds.push(cmd),
                    }
                };
                if let Some(i) = ctx_specs.iter().enumerate().rev()
                    .find(|(_, b)| mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h)
                    .map(|(i, _)| i) {
                    click_slot(&mut cmds, Command::CtxAct(i));
                }
                // Talk rides the band's next cell (#1599): the draw and
                // this test read the same talk_button rect
                let tb = session::talk_button(m.ctx.len());
                if mx >= tb.x && mx < tb.x + tb.w && my >= tb.y && my < tb.y + tb.h {
                    click_slot(&mut cmds, Command::Act(0));
                }
                // a reply on the TALK panel
                if let Some(pick) = client::talk::talk_click(&m.talk, mx, my) {
                    cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    cmds.push(Command::TalkPick(pick));
                }
                // the mystery panel's asker name: select them and jump
                // there, same as a positioned log line (#646)
                if let Some(i) = view.mystery_hit(m, mx, my) {
                    cmds.retain(|x| !matches!(x, Command::ClickMap { .. }));
                    if let Some(mz) = &m.panel.mystery {
                        if let Some(at) = mz.asker_at {
                            (view.cam_x, view.cam_y) = session::camera_on(at.0, at.1);
                            manual_cam = true;
                        }
                    }
                    cmds.push(Command::Select(i));
                }
                // the conversation box: a speaker's line selects them, a
                // choice button answers it (a reply, leave, or the call)
                if let Some(c) = &m.convo {
                    if let Some(click) = client::talk::convo_click(c, mx, my, &mut view.convo_pin.borrow_mut(), m.frame) {
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
                if !view.search_focus && window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) { g.command(Command::TogglePause); }
            }
        }
        if !view.search_focus && (window.is_key_pressed(Key::H, minifb::KeyRepeat::No)) { view.help_open = !view.help_open; }
        if !view.search_focus && (window.is_key_pressed(Key::F3, minifb::KeyRepeat::No)) { view.ledger_open = !view.ledger_open; }
        // M mutes: the host reads View.mute each frame (#1494), and
        // the press says so on the toast feed (#1505)
        if !view.search_focus && (window.is_key_pressed(Key::M, minifb::KeyRepeat::No)) {
            view.toggle_mute(cur_model.as_ref().map(|m| m.frame).unwrap_or(0));
        }
        // V opens the Sound overlay off the pause menu (#1506) - only on
        // a bare map, where Esc itself would open the menu, so a Talk or
        // the board keeps its own keys. The menu comes up with it so
        // Esc still backs out to it
        if !view.search_focus && window.is_key_pressed(Key::V, minifb::KeyRepeat::No)
            && cur_model.as_ref().map(client::esc_route) == Some(client::EscRoute::Pause) {
            pause_menu = true;
            view.sound_menu = true;
            view.sound_row = 0;
        }
        if let Some((_, scroll_y)) = window.get_scroll_wheel() {
            // Scale multiplicatively so a wheel notch feels consistent at any zoom,
            // while trackpads can still make smaller, smooth adjustments.
            view.zoom = (view.zoom * 1.1_f32.powf(scroll_y)).clamp(MIN_ZOOM, MAX_ZOOM);
            if scroll_y != 0.0 { manual_cam = true; }
        }
        if window.is_key_down(Key::Equal) || window.is_key_down(Key::NumPadPlus) { view.zoom = (view.zoom + 0.006).min(MAX_ZOOM); manual_cam = true; }
        if window.is_key_down(Key::Minus) || window.is_key_down(Key::NumPadMinus) { view.zoom = (view.zoom - 0.006).max(MIN_ZOOM); manual_cam = true; }
        if !view.search_focus {
            if window.is_key_pressed(Key::Left, minifb::KeyRepeat::No)  { view.cam_x += 2; manual_cam = true; }
            if window.is_key_pressed(Key::Right, minifb::KeyRepeat::No) { view.cam_x -= 2; manual_cam = true; }
            if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No)    { view.cam_y += 2; manual_cam = true; }
            if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No)  { view.cam_y -= 2; manual_cam = true; }
        }
        // a manual pan or zoom holds the event camera off briefly - pan
        // wins for a moment, then the event has it back
        if manual_cam { event_cam.hold(); }

        if window.is_key_pressed(Key::F12, minifb::KeyRepeat::No) { buf.save_bmp(&format!("evidence/ui-shot-{}.bmp", cur_model.as_ref().map(|m| m.frame).unwrap_or(0))).ok(); }
        // hold F9, talk about a bug or an idea, let go - the local
        // whisper-cli transcribes the clip and it's filed dev-report
        // (#1083); the work rides a worker thread, the game doesn't wait
        #[cfg(all(feature = "dev-voice", not(target_arch = "wasm32")))]
        {
            if window.is_key_pressed(Key::F9, minifb::KeyRepeat::No) { voice.press(); }
            if window.is_key_released(Key::F9) { voice.release(voice_ctx(cur_model.as_ref())); }
        }
        // a survey answer landed this frame: the shot goes into the
        // bundle the sim just wrote under feedback/ (#1085)
        if let Some(stem) = cur_model.as_ref().and_then(|m| m.feedback_capture.clone()) {
            buf.save_bmp(&format!("feedback/{}.bmp", stem)).ok();
        }
        // the frame's done with: the next render draws into it
        view.recycle(buf);

        let spent = frame_at.elapsed();
        if spent < frame_len { std::thread::sleep(frame_len - spent); }
    }

    // the settings file lands on the way out (#1505) - pending changes
    // written, a corrupt one rewritten
    settings_store.flush();

    let _ = MAP_TILES;
}
