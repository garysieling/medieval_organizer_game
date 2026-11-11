//! The browser client: the same game2-core sim compiled to wasm, so fixes
//! land in one place. Exposes a `WasmGame` - a Game, the renderer that
//! draws it, and its serialized UiModel scene.
//!
//! Build: `cargo build -p game2-web --target wasm32-unknown-unknown`
//! then serve index.html + the wasm + assets/ over HTTP.
//!
//! The page asks `asset_manifest()` for the files `Art::load` will want,
//! fetches them over HTTP and hands the bytes back with `provide_asset`;
//! `open_view()` then builds the same `View` the window runs. Each frame
//! `render()` produces the RGBA `Buffer` `frame_ptr`/`frame_len` point at,
//! and the canvas blits it - the desktop's draw code, not a JS rewrite.

use game2_client::{
    camera_follow, render_intro, render_picker, talk::digit_cmd, Art, EventCam, View, MAX_ZOOM,
    MIN_ZOOM,
};
use game2_core::session::layout::MAP_W;
use game2_core::session::{
    self, iso_to_screen_cam, journal, roster::RosterCard, Command, Game, PKind, Scenario,
    SceneCache, ServerMsg, UiModel,
};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// A seed off the browser's randomness (wasm has no clock to take one
/// from); seed 1 where there's none
fn fresh_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        let mut b = [0u8; 8];
        if getrandom::fill(&mut b).is_ok() {
            return u64::from_le_bytes(b) % 1_000_000;
        }
    }
    1
}

/// Everything about drawing that a page needs and both hosts share:
/// the fetched art bytes, the `View` once it's open, the last frame's
/// pixels, the camera state, and the model the last render drew.
struct RenderHost {
    /// bytes the page fetched for `Art::load`, keyed by asset-relative path
    assets: HashMap<String, Vec<u8>>,
    view: Option<View>,
    /// the last rendered frame, kept alive so `frame_ptr` stays valid
    frame: Option<game2_core::engine::render::Buffer>,
    /// the model the frame drew, kept for the digit keys' convo check
    model: Option<UiModel>,
    mouse: (i32, i32),
    /// camera state, mirroring the window's loop: where the player stood,
    /// a focus-jump's hold on the follow, and the event camera
    player_seen: Option<((f32, f32), f32)>,
    follow_hold: Option<(f32, f32)>,
    event_cam: EventCam,
    cam_centered: bool,
    focus_seeded: bool,
}

impl RenderHost {
    fn new() -> RenderHost {
        RenderHost {
            assets: HashMap::new(),
            view: None,
            frame: None,
            model: None,
            mouse: (0, 0),
            player_seen: None,
            follow_hold: None,
            event_cam: EventCam::default(),
            cam_centered: false,
            focus_seeded: false,
        }
    }

    fn provide_asset(&mut self, rel: &str, bytes: &[u8]) {
        self.assets.insert(rel.to_string(), bytes.to_vec());
    }

    /// Build the renderer once the art's in. Reuses the desktop's `View`;
    /// `seed` settles the same sprite variants the window would pick
    fn open_view(&mut self, seed: u64) {
        let map = std::mem::take(&mut self.assets);
        let fetch = move |rel: &str| map.get(rel).cloned();
        self.view = Some(View::new(Art::load(&fetch, seed)));
    }

    fn pan(&mut self, dx: i32, dy: i32) {
        if let Some(v) = self.view.as_mut() {
            v.cam_x += dx;
            v.cam_y += dy;
            self.event_cam.hold();
        }
    }

    fn set_zoom(&mut self, z: f32) {
        if let Some(v) = self.view.as_mut() {
            v.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
            self.event_cam.hold();
        }
    }

    /// The camera the last frame drew with - for `ClickMap`, which takes
    /// the map's click back to world tiles through it
    fn cam(&self) -> String {
        match &self.view {
            Some(v) => format!(
                "{{\"x\":{},\"y\":{},\"zoom\":{}}}",
                v.cam_x, v.cam_y, v.zoom
            ),
            None => "{\"x\":0,\"y\":0,\"zoom\":1}".to_string(),
        }
    }

    /// What a digit key presses this frame: the conversation's row if a
    /// talk or decision box is up, else the numbered villager - the same
    /// `digit_cmd` the window asks, so both hosts answer alike
    fn digit_command(&self, n: usize) -> String {
        let c = digit_cmd(self.model.as_ref().and_then(|m| m.convo.as_ref()), n);
        c.map(|c| serde_json::to_string(&c).unwrap_or_default())
            .unwrap_or_default()
    }

    /// The frame's audio cues, drained so a held frame doesn't replay
    /// them: `[kind, pan, gain]` with pan where the cue lands on screen
    /// (-1 left .. 1 right) through the same projection the view uses
    fn take_sfx(&mut self) -> String {
        let Some(m) = self.model.as_mut() else {
            return "[]".into();
        };
        let cues = std::mem::take(&mut m.sfx_v);
        let (cx, cy, zoom) = self
            .view
            .as_ref()
            .map(|v| (v.cam_x, v.cam_y, v.zoom))
            .unwrap_or((0, 0, 1.0));
        let half = MAP_W as f32 / 2.0;
        let out: Vec<(String, f32, f32)> = cues
            .iter()
            .map(|(kind, x, y, gain)| {
                let (sx, _) = iso_to_screen_cam(*x as f32, *y as f32, cx, cy, zoom);
                (
                    kind.clone(),
                    ((sx as f32 - half) / half).clamp(-1.0, 1.0),
                    *gain,
                )
            })
            .collect();
        serde_json::to_string(&out).unwrap_or_else(|_| "[]".into())
    }

    /// Draw a frame: the camera runs exactly the window's sequence
    /// (sim-asked look, first-frame centre on the player, the comfort-box
    /// follow muted while a focus jump or a live event holds it, then the
    /// event camera), and `View::render` produces the pixels
    fn render_model(&mut self, m: UiModel) {
        let Some(view) = self.view.as_mut() else {
            return;
        };

        if !self.focus_seeded {
            view.focus_seen.set(m.focus.map(|f| f.2));
            self.focus_seeded = true;
        }
        if let Some((fx, fy, ff)) = m.focus {
            if view.focus_seen.replace(Some(ff)) != Some(ff) {
                (view.cam_x, view.cam_y) = session::camera_on(fx, fy);
                view.zoom = view.zoom.max(1.5);
                // the jump wins outright: follow stays off until the
                // player character takes a step
                self.follow_hold = m
                    .people
                    .iter()
                    .find(|p| p.kind == PKind::Player)
                    .map(|p| p.pos);
            }
        }

        if !self.cam_centered {
            if let Some(p) = m.people.iter().find(|p| p.kind == PKind::Player) {
                (view.cam_x, view.cam_y) = session::camera_on(p.pos.0, p.pos.1);
                self.cam_centered = true;
            }
        }

        if let Some(hp) = self.follow_hold {
            let now = m
                .people
                .iter()
                .find(|p| p.kind == PKind::Player)
                .map(|p| p.pos);
            if now
                .map(|n| (n.0 - hp.0).abs() > 0.01 || (n.1 - hp.1).abs() > 0.01)
                .unwrap_or(true)
            {
                self.follow_hold = None;
            }
        }
        let muted = self.follow_hold.is_some() || m.player_event.is_some();
        if let Some(cam) = camera_follow(
            &m,
            (view.cam_x, view.cam_y),
            view.zoom,
            &mut self.player_seen,
        ) {
            if !muted {
                (view.cam_x, view.cam_y) = cam;
            }
        }

        let ((nx, ny), nz) =
            self.event_cam
                .step(m.player_event.as_ref(), (view.cam_x, view.cam_y), view.zoom);
        view.cam_x = nx;
        view.cam_y = ny;
        view.zoom = nz;

        self.frame = Some(view.render(&m, self.mouse));
        self.model = Some(m);
    }

    fn frame_ptr(&self) -> *const u8 {
        self.frame
            .as_ref()
            .map(|f| f.pixels.as_ptr())
            .unwrap_or(std::ptr::null())
    }

    fn frame_len(&self) -> usize {
        self.frame.as_ref().map(|f| f.pixels.len()).unwrap_or(0)
    }
}

/// The asset paths `Art::load` will ask for, as a JSON array - the
/// page fetches each over HTTP and feeds the bytes back with
/// `provide_asset`. Dry-running the load with a fetch that only records
/// names costs nothing: a miss never reaches decode, and the list is the
/// same whatever seed the variants settle to.
fn manifest(seed: u64) -> String {
    let wanted = std::cell::RefCell::new(Vec::new());
    let fetch = |rel: &str| {
        wanted.borrow_mut().push(rel.to_string());
        None
    };
    drop(Art::load(&fetch, seed));
    let mut names = wanted.into_inner();
    names.sort();
    names.dedup();
    serde_json::to_string(&names).unwrap_or_default()
}

/// A wasm-held Game: the browser ticks it, feeds it the art it asks for,
/// and blits the frame it renders
#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
    host: RenderHost,
}

impl WasmGame {
    fn built(game: Game) -> WasmGame {
        WasmGame {
            game,
            host: RenderHost::new(),
        }
    }
}

#[wasm_bindgen]
impl WasmGame {
    /// A fresh country each page load, like the window's launch
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        Self::built(Game::new_seeded(fresh_seed()))
    }

    /// A particular country: the same seed draws the same map
    pub fn with_seed(seed: u32) -> WasmGame {
        Self::built(Game::new_seeded(seed as u64))
    }

    /// A saved scenario, mid-telling
    pub fn scenario(kind: &str) -> WasmGame {
        let s = match kind {
            "fever" => Scenario::FeverYear,
            "circus" => Scenario::CircusWinter,
            "widow" => Scenario::WidowsShop,
            _ => Scenario::FeverYear,
        };
        Self::built(Game::new_scenario(s))
    }

    /// The asset paths `Art::load` will ask for, as a JSON array - the
    /// page fetches each over HTTP and feeds the bytes back with
    /// `provide_asset`. Dry-running the load with a fetch that only
    /// records names costs nothing: a miss never reaches decode.
    pub fn asset_manifest(&self) -> String {
        manifest(self.game.seed)
    }

    /// Bytes for one manifest path; anything unprovided falls back to the
    /// placeholder look, same as a missing file does on the desktop
    pub fn provide_asset(&mut self, rel: &str, bytes: &[u8]) {
        self.host.provide_asset(rel, bytes);
    }

    /// Build the renderer once the art's in. Reuses the desktop's `View`:
    /// the game seed settles the same sprite variants the window would
    /// have picked
    pub fn open_view(&mut self) {
        self.host.open_view(self.game.seed);
    }

    /// Where the pointer sits, for the flyer's hover - the same `mouse`
    /// the window passes `View::render`
    pub fn set_mouse(&mut self, x: i32, y: i32) {
        self.host.mouse = (x, y);
    }

    /// A deliberate camera move (drag-pan): like the window's pans it
    /// mutes the event camera briefly rather than fighting it
    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.host.pan(dx, dy);
    }

    /// Zoom toward the wheel's wish, clamped where the window clamps it.
    /// A zoom while an event holds the camera counts as the player
    /// taking it, same as a pan
    pub fn set_zoom(&mut self, z: f32) {
        self.host.set_zoom(z);
    }

    /// Draw a frame: the camera runs exactly the window's sequence
    /// (sim-asked look, first-frame centre on the player, the comfort-box
    /// follow muted while a focus jump or a live event holds it, then the
    /// event camera), and `View::render` produces the pixels
    pub fn render(&mut self) {
        let m = self.game.ui_model();
        self.host.render_model(m);
    }

    /// The frame's pixels, RGBA row-major, valid until the next `render`:
    /// `new ImageData(new Uint8ClampedArray(wasm.memory.buffer,
    /// frame_ptr(), frame_len()), frame_width(), frame_height())`
    pub fn frame_ptr(&self) -> *const u8 {
        self.host.frame_ptr()
    }

    /// `frame_width` * `frame_height` * 4 - the span `frame_ptr` covers
    pub fn frame_len(&self) -> usize {
        self.host.frame_len()
    }

    pub fn frame_width(&self) -> usize {
        self.host.frame.as_ref().map(|f| f.width).unwrap_or(0)
    }

    pub fn frame_height(&self) -> usize {
        self.host.frame.as_ref().map(|f| f.height).unwrap_or(0)
    }

    /// The camera the last frame drew with - for `ClickMap`, which takes
    /// the map's click back to world tiles through it
    pub fn cam(&self) -> String {
        self.host.cam()
    }

    /// What a digit key presses this frame, as a serialized `Command`
    /// (empty when it presses nothing): the conversation's row if a talk
    /// or decision box is up, else the numbered villager
    pub fn digit_command(&self, n: usize) -> String {
        self.host.digit_command(n)
    }

    /// The frame's audio cues, drained so a held frame doesn't replay
    /// them: `[kind, pan, gain]` - pan where the cue lands on screen
    pub fn take_sfx(&mut self) -> String {
        self.host.take_sfx()
    }

    /// One frame of the village
    pub fn tick(&mut self) {
        self.game.update();
    }

    /// The scene, serialized - the canvas shim draws it
    pub fn scene(&self) -> String {
        serde_json::to_string(&self.game.ui_model()).unwrap_or_default()
    }

    /// The day's count, for the page's clock
    pub fn day(&self) -> u64 {
        self.game.tick
    }

    /// One input toward the player's character: a serialized `Command`,
    /// the same enum the websocket protocol carries - the page plays the
    /// same game the window and the ws client do
    pub fn command(&mut self, json: &str) -> bool {
        match serde_json::from_str::<Command>(json) {
            Ok(c) => {
                self.game.command(c);
                true
            }
            Err(_) => false,
        }
    }

    /// The secret gauges, for a debug readout (omniscient on purpose: the
    /// page runs the whole sim in the browser - a dev tool, not a player view)
    pub fn telemetry(&self) -> String {
        self.game.secret_kpi_lines().join("\n")
    }

    /// A staged start for the harness: "ordinary", "fever", "famine",
    /// "winter", "strong", "unrest", "flush", "bereft"
    pub fn staged(kind: &str) -> WasmGame {
        use session::stage::{staged, Stage};
        let s = match kind {
            "fever" => Stage::Fever,
            "famine" => Stage::Famine,
            "winter" => Stage::HardWinter,
            "strong" => Stage::StrongOrg,
            "unrest" => Stage::Unrest,
            "flush" => Stage::Flush,
            "bereft" => Stage::Bereft,
            _ => Stage::Ordinary,
        };
        Self::built(staged(s))
    }

    /// The probe: the secret state, serialized. A SEPARATE API from
    /// `scene()` on purpose - tests get what players never see
    pub fn probe(&self) -> String {
        serde_json::to_string(&self.game.probe()).unwrap_or_default()
    }

    /// Force the rare event now: "Tinker" or "the tinker", the harness's
    /// way onto the schedule
    pub fn force_event(&mut self, name: &str) -> bool {
        match session::events::event_id(name) {
            Some(id) => {
                self.game.force_event(id);
                true
            }
            None => false,
        }
    }

    /// Run `days` days of frames - the same path the window plays
    pub fn run_days(&mut self, days: u64) {
        session::stage::run_days(&mut self.game, days);
    }

    /// The day's record, for IndexedDB: the journal (seed + every input,
    /// tagged by frame), the frame it stopped at, and the world's
    /// fingerprint. The datomic save from `datomic-save-load.md` - `load`
    /// replays the record back to the same village, and a save whose
    /// replay drifts from its fingerprint is refused, not trusted.
    pub fn save_json(&self) -> String {
        serde_json::json!({
            "v": 1,
            "journal": self.game.journal,
            "frame": self.game.frame_n,
            "digest": self.game.state_digest(),
        })
        .to_string()
    }

    /// Rebuild a saved village: replay its record to the saved frame and
    /// check the fingerprint. `undefined` on a bad string or a drifted
    /// replay - the page starts a fresh village instead
    pub fn load(text: &str) -> Option<WasmGame> {
        #[derive(serde::Deserialize)]
        struct Doc {
            journal: journal::Journal,
            frame: u64,
            digest: u64,
        }
        let doc: Doc = serde_json::from_str(text).ok()?;
        let g = journal::replay(&doc.journal, doc.frame);
        (g.state_digest() == doc.digest).then(|| Self::built(g))
    }
}

/// A client of `game_server`: no sim in the page at all - the browser
/// owns the websocket, feeds each `ServerMsg` JSON through `push_message`,
/// and the same `RenderHost` draws the cached scenes. This is the
/// shipped mode per the grill; the embedded `WasmGame` stays as the
/// dev/test harness
#[wasm_bindgen]
pub struct WasmClient {
    cache: SceneCache,
    host: RenderHost,
    /// the Welcome's payload: my entity, my claim token, the roster the
    /// picker deals, and the prologue card's facts
    you: u32,
    token: String,
    roster: Vec<RosterCard>,
    intro: Vec<String>,
    denied: Option<String>,
}

impl WasmClient {
    /// What `open_view` seeds `Art::load` with. The wire never carries the
    /// sim's seed, so every client folds the tilemap it was sent into one:
    /// the same village picks the same cottage colours on every client.
    fn art_seed(&self) -> u64 {
        let Some(map) = self.cache.map() else {
            return 0;
        };
        let json = serde_json::to_string(map).unwrap_or_default();
        // FNV-1a over the serialized map: deterministic, cheap, shared
        let mut h: u64 = 0xcbf29ce484222325;
        for b in json.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

#[wasm_bindgen]
impl WasmClient {
    /// A seat watching `ws://host:8787` - the page connects first and
    /// feeds frames through `push_message`
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmClient {
        WasmClient {
            cache: SceneCache::new(),
            host: RenderHost::new(),
            you: 0,
            token: String::new(),
            roster: Vec::new(),
            intro: Vec::new(),
            denied: None,
        }
    }

    /// One text frame off the websocket. Returns a flag word:
    /// 1 a full scene's cached (map merged in), 2 the Welcome landed
    /// (roster/intro/claim ready), 4 the server said no (`denied_reason`),
    /// 8 a Ping answered, 16 the Map landed (the renderer's seed is set)
    pub fn push_message(&mut self, text: &str) -> u32 {
        let Ok(msg) = serde_json::from_str::<ServerMsg>(text) else {
            return 0;
        };
        let mut out = 0;
        match &msg {
            ServerMsg::Welcome {
                you,
                token,
                roster,
                intro,
                ..
            } => {
                self.you = *you;
                self.token = token.clone();
                self.roster = roster.clone();
                self.intro = intro.clone();
                out |= 2;
            }
            ServerMsg::Denied { reason } => {
                self.denied = Some(reason.clone());
                return 4;
            }
            ServerMsg::Pong(_) => return 8,
            ServerMsg::Map(_) => out |= 16,
            _ => {}
        }
        if let Some(m) = self.cache.apply(msg) {
            self.host.model = Some(m);
            out |= 1;
        }
        out
    }

    /// Why the server turned the seat away, once `push_message` flags 4
    pub fn denied_reason(&self) -> String {
        self.denied.clone().unwrap_or_default()
    }

    /// The claim token the Welcome dealt - the page keeps it
    /// (localStorage) and sends it in the next Hello to take the same
    /// character back
    pub fn claim_token(&self) -> String {
        self.token.clone()
    }

    /// How many roster cards the picker deals (0: no pick needed)
    pub fn roster_len(&self) -> usize {
        self.roster.len()
    }

    /// The prologue card's facts, as a JSON array of lines
    pub fn intro_lines(&self) -> String {
        serde_json::to_string(&self.intro).unwrap_or_else(|_| "[]".into())
    }

    /// Draw the roster picker with `sel` lit - the same card the window
    /// draws, through the client's own `render_picker`
    pub fn render_picker(&mut self, sel: usize) {
        let Some(view) = self.host.view.as_ref() else {
            return;
        };
        self.host.frame = Some(render_picker(&view.art, &self.roster, sel));
    }

    /// Draw the prologue card - the same `render_intro` the window runs
    /// after the pick
    pub fn render_intro(&mut self) {
        if self.intro.is_empty() {
            return;
        }
        self.host.frame = Some(render_intro(&self.intro));
    }

    /// The asset paths `Art::load` asks for - same surface as WasmGame
    pub fn asset_manifest(&self) -> String {
        manifest(self.art_seed())
    }

    pub fn provide_asset(&mut self, rel: &str, bytes: &[u8]) {
        self.host.provide_asset(rel, bytes);
    }

    /// Build the renderer once the art's in - the map's hash seeds the
    /// sprite variants so every client of a server sees the same cottages
    pub fn open_view(&mut self) {
        self.host.open_view(self.art_seed());
    }

    pub fn set_mouse(&mut self, x: i32, y: i32) {
        self.host.mouse = (x, y);
    }

    pub fn pan(&mut self, dx: i32, dy: i32) {
        self.host.pan(dx, dy);
    }

    pub fn set_zoom(&mut self, z: f32) {
        self.host.set_zoom(z);
    }

    /// Draw the newest cached scene (the camera runs the window's
    /// sequence); a no-op till both a Scene and a Map have landed and the
    /// view's open
    pub fn render(&mut self) {
        if self.host.view.is_none() {
            return;
        }
        let Some(m) = self.host.model.take() else {
            return;
        };
        self.host.render_model(m);
    }

    pub fn frame_ptr(&self) -> *const u8 {
        self.host.frame_ptr()
    }

    pub fn frame_len(&self) -> usize {
        self.host.frame_len()
    }

    pub fn frame_width(&self) -> usize {
        self.host.frame.as_ref().map(|f| f.width).unwrap_or(0)
    }

    pub fn frame_height(&self) -> usize {
        self.host.frame.as_ref().map(|f| f.height).unwrap_or(0)
    }

    /// The camera the last frame drew with - for `ClickMap`
    pub fn cam(&self) -> String {
        self.host.cam()
    }

    /// What a digit key presses this frame, as a serialized `Command`
    /// (empty when it presses nothing)
    pub fn digit_command(&self, n: usize) -> String {
        self.host.digit_command(n)
    }

    /// The newest scene's audio cues, drained: `[kind, pan, gain]`
    pub fn take_sfx(&mut self) -> String {
        self.host.take_sfx()
    }

    /// Which entity is me, from the Welcome - for finding my figure in
    /// the people list when the camera looks for it
    pub fn me(&self) -> u32 {
        self.you
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    // The page constructs a WasmGame and ticks it - nothing to boot here
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_page_can_play_not_just_watch() {
        let mut g = WasmGame::new();
        assert!(!g.game.paused);
        assert!(g.command("\"TogglePause\""), "a real Command parses");
        assert!(g.game.paused, "the input reached the sim");
        assert!(!g.command("\"NotACommand\""), "garbage is refused");
        // a parseable-but-wrong index is refused by the sim, not stored up
        // to crash the next frame
        g.command("{\"Select\": 999}");
        g.tick();
        g.scene();
    }

    /// The test API rides the same wasm: stage a village, force the rare
    /// event on, run the days, and read both surfaces - the scene the
    /// canvas draws and the probe only tests see
    #[test]
    fn test_the_harness_rides_the_wasm_too() {
        let mut g = WasmGame::staged("famine");
        let pv: serde_json::Value = serde_json::from_str(&g.probe()).unwrap();
        let food = pv["food_servings"].as_u64().unwrap_or(999);
        assert!(
            food < 20,
            "the staged famine's store is thin (got {})",
            food
        );

        assert!(g.force_event("the tinker"), "by its map name");
        assert!(!g.force_event("no such event"), "bad names are refused");
        let pv: serde_json::Value = serde_json::from_str(&g.probe()).unwrap();
        assert!(
            pv["rare_fired"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f[0] == "Tinker"),
            "the firing's on the probe's record"
        );

        g.run_days(3);
        let scene: serde_json::Value = serde_json::from_str(&g.scene()).unwrap();
        assert!(scene["tick"].as_u64().unwrap() >= 3, "the days ran");
        // and the secret stays out of the scene: the canvas can't draw it
        let scene_json = g.scene();
        assert!(
            !scene_json.contains("grievances"),
            "the scene carries no inner lives"
        );
    }

    /// The page's boot sequence: ask for the manifest, feed bytes back,
    /// open the view, and a frame comes out the desktop renderer's size.
    /// Assets the page can't fetch draw as placeholders, same as the
    /// window's missing-file path
    #[test]
    fn test_the_frame_pipeline_draws_a_frame() {
        let mut g = WasmGame::with_seed(7);
        let manifest: Vec<String> = serde_json::from_str(&g.asset_manifest()).unwrap();
        assert!(!manifest.is_empty(), "the art asks for something");
        assert_eq!(
            manifest.len(),
            manifest
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            "no dupes in the manifest"
        );

        // the page hands bytes back for whatever it fetched
        g.provide_asset(&manifest[0], b"not a sprite");
        g.open_view();
        g.tick();
        g.render();

        assert_eq!(
            g.frame_width() * g.frame_height() * 4,
            g.frame_len(),
            "the buffer's RGBA-full"
        );
        assert!(g.frame_width() > 0, "a frame came out");
        assert!(!g.frame_ptr().is_null(), "the pointer's valid");
    }

    /// Camera plumbing: the first render centres on the player's
    /// character, and a page pan is honoured (and held against the event
    /// camera, like the window's)
    #[test]
    fn test_the_camera_centres_and_the_pan_sticks() {
        let mut g = WasmGame::with_seed(7);
        g.open_view();
        g.tick();
        g.render();
        let view = g.host.view.as_ref().unwrap();
        let (cx, cy) = (view.cam_x, view.cam_y);
        let m = g.game.ui_model();
        let p = m.people.iter().find(|p| p.kind == PKind::Player).unwrap();
        assert_eq!(
            (cx, cy),
            session::camera_on(p.pos.0, p.pos.1),
            "the opening frame centres on the player"
        );
        g.pan(40, -20);
        let view = g.host.view.as_ref().unwrap();
        assert_eq!(
            (view.cam_x, view.cam_y),
            (cx + 40, cy - 20),
            "the pan landed"
        );
    }

    /// The ws client's wire walk: a Welcome keeps the claim and deals the
    /// picker's cards, a Map then a Scene become a drawable frame, and a
    /// Denied names why
    #[test]
    fn test_the_ws_client_plays_the_wire() {
        let mut c = WasmClient::new();
        let g = Game::new_seeded(7);
        let model = g.ui_model_lean_at(0);
        let card = RosterCard {
            name: "Sabyn".into(),
            backstory: "a walker".into(),
            work: "baker".into(),
            kid: false,
            look: 3,
        };
        let welcome = ServerMsg::Welcome {
            you: 5,
            token: "claim1".into(),
            model: Box::new(model),
            roster: vec![card],
            intro: vec!["the market towns".into()],
        };
        // Welcome flags 2 (and not 1: no map has come to fill the scene)
        assert_eq!(c.push_message(&serde_json::to_string(&welcome).unwrap()), 2);
        assert_eq!(c.claim_token(), "claim1");
        assert_eq!(c.me(), 5);
        assert_eq!(c.roster_len(), 1);
        assert!(c.intro_lines().contains("market towns"));
        // a scene with no map yet is not a frame
        c.render();
        assert_eq!(c.frame_len(), 0, "a scene without its map draws nothing");

        // the map lands, then scenes are whole
        let map = ServerMsg::Map(Box::new(g.map.clone()));
        // Map flags 16: the render seed's in before the first scene
        assert_eq!(c.push_message(&serde_json::to_string(&map).unwrap()), 16);
        let scene = ServerMsg::Scene(Box::new(g.ui_model_lean_at(0)));
        assert_eq!(c.push_message(&serde_json::to_string(&scene).unwrap()), 1);
        c.open_view();
        c.render();
        assert!(c.frame_len() > 0, "the remote frame drew");

        // the picker and the prologue draw through the same cards
        c.render_picker(0);
        assert!(c.frame_len() > 0);
        c.render_intro();
        assert!(c.frame_len() > 0);

        // a refusal names itself
        assert_eq!(
            c.push_message("{\"Denied\":{\"reason\":\"the village is full\"}}"),
            4
        );
        assert_eq!(c.denied_reason(), "the village is full");
        assert_eq!(c.push_message("{\"Pong\":7}"), 8);
        assert_eq!(c.push_message("not json"), 0);
    }

    /// The shared digit-key surface: no convo open, a digit selects the
    /// numbered villager - the same `digit_cmd` the window asks
    #[test]
    fn test_digit_keys_pick_or_talk() {
        let mut c = WasmClient::new();
        let g = Game::new_seeded(7);
        c.host.model = Some(g.ui_model_lean_at(0));
        let c0 = c.digit_command(0);
        assert!(c0.contains("Select"), "digit one selects, got {c0}");
    }

    /// IndexedDB's round-trip: play a while, save, reload - the same
    /// village comes back, and a record that doesn't replay to its own
    /// fingerprint is refused
    #[test]
    fn test_the_save_remembers_the_village() {
        let mut g = WasmGame::with_seed(3);
        g.command(&serde_json::to_string(&Command::Walk(1.0, 0.0)).unwrap());
        for _ in 0..8 {
            g.tick();
        }
        let saved = g.save_json();
        let r = WasmGame::load(&saved).expect("the save loads");
        assert_eq!(
            r.game.state_digest(),
            g.game.state_digest(),
            "the reloaded village drifted from the played one"
        );

        assert!(WasmGame::load("not json").is_none(), "gibberish refuses");
        // a record that runs longer than it played fails the fingerprint
        let tampered = saved.replace("\"frame\":8", "\"frame\":40");
        assert!(
            WasmGame::load(&tampered).is_none(),
            "a drifted replay refuses"
        );
    }
}
