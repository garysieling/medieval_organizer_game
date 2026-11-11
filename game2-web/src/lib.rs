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

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use game2_core::session::{self, Game, Scenario, Command, PKind};
use game2_client::{Art, View, EventCam, camera_follow, MIN_ZOOM, MAX_ZOOM};

/// A seed off the browser's randomness (wasm has no clock to take one
/// from); seed 1 where there's none
fn fresh_seed() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        let mut b = [0u8; 8];
        if getrandom::fill(&mut b).is_ok() { return u64::from_le_bytes(b) % 1_000_000; }
    }
    1
}

/// A wasm-held Game: the browser ticks it, feeds it the art it asks for,
/// and blits the frame it renders
#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
    /// bytes the page fetched for `Art::load`, keyed by asset-relative path
    assets: HashMap<String, Vec<u8>>,
    view: Option<View>,
    /// the last rendered frame, kept alive so `frame_ptr` stays valid
    frame: Option<game2_core::engine::render::Buffer>,
    mouse: (i32, i32),
    /// camera state, mirroring the window's loop: where the player stood,
    /// a focus-jump's hold on the follow, and the event camera
    player_seen: Option<((f32, f32), f32)>,
    follow_hold: Option<(f32, f32)>,
    event_cam: EventCam,
    cam_centered: bool,
    focus_seeded: bool,
}

impl WasmGame {
    fn built(game: Game) -> WasmGame {
        WasmGame {
            game,
            assets: HashMap::new(),
            view: None,
            frame: None,
            mouse: (0, 0),
            player_seen: None,
            follow_hold: None,
            event_cam: EventCam::default(),
            cam_centered: false,
            focus_seeded: false,
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
        let wanted = std::cell::RefCell::new(Vec::new());
        let fetch = |rel: &str| { wanted.borrow_mut().push(rel.to_string()); None };
        drop(Art::load(&fetch, self.game.seed));
        let mut names = wanted.into_inner();
        names.sort();
        names.dedup();
        serde_json::to_string(&names).unwrap_or_default()
    }

    /// Bytes for one manifest path; anything unprovided falls back to the
    /// placeholder look, same as a missing file does on the desktop
    pub fn provide_asset(&mut self, rel: &str, bytes: &[u8]) {
        self.assets.insert(rel.to_string(), bytes.to_vec());
    }

    /// Build the renderer once the art's in. Reuses the desktop's `View`:
    /// the game seed settles the same sprite variants the window would
    /// have picked
    pub fn open_view(&mut self) {
        let map = std::mem::take(&mut self.assets);
        let fetch = move |rel: &str| map.get(rel).cloned();
        self.view = Some(View::new(Art::load(&fetch, self.game.seed)));
    }

    /// Where the pointer sits, for the flyer's hover - the same `mouse`
    /// the window passes `View::render`
    pub fn set_mouse(&mut self, x: i32, y: i32) {
        self.mouse = (x, y);
    }

    /// A deliberate camera move (drag-pan): like the window's pans it
    /// mutes the event camera briefly rather than fighting it
    pub fn pan(&mut self, dx: i32, dy: i32) {
        if let Some(v) = self.view.as_mut() {
            v.cam_x += dx;
            v.cam_y += dy;
            self.event_cam.hold();
        }
    }

    /// Zoom toward the wheel's wish, clamped where the window clamps it.
    /// A zoom while an event holds the camera counts as the player
    /// taking it, same as a pan
    pub fn set_zoom(&mut self, z: f32) {
        if let Some(v) = self.view.as_mut() {
            v.zoom = z.clamp(MIN_ZOOM, MAX_ZOOM);
            self.event_cam.hold();
        }
    }

    /// Draw a frame: the camera runs exactly the window's sequence
    /// (sim-asked look, first-frame centre on the player, the comfort-box
    /// follow muted while a focus jump or a live event holds it, then the
    /// event camera), and `View::render` produces the pixels
    pub fn render(&mut self) {
        let Some(view) = self.view.as_mut() else { return; };
        let m = self.game.ui_model();

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
                self.follow_hold = m.people.iter()
                    .find(|p| p.kind == PKind::Player).map(|p| p.pos);
            }
        }

        if !self.cam_centered {
            if let Some(p) = m.people.iter().find(|p| p.kind == PKind::Player) {
                (view.cam_x, view.cam_y) = session::camera_on(p.pos.0, p.pos.1);
                self.cam_centered = true;
            }
        }

        if let Some(hp) = self.follow_hold {
            let now = m.people.iter().find(|p| p.kind == PKind::Player).map(|p| p.pos);
            if now.map(|n| (n.0 - hp.0).abs() > 0.01 || (n.1 - hp.1).abs() > 0.01).unwrap_or(true) {
                self.follow_hold = None;
            }
        }
        let muted = self.follow_hold.is_some() || m.player_event.is_some();
        if let Some(cam) = camera_follow(&m, (view.cam_x, view.cam_y), view.zoom, &mut self.player_seen) {
            if !muted { (view.cam_x, view.cam_y) = cam; }
        }

        let ((nx, ny), nz) = self.event_cam.step(m.player_event.as_ref(),
            (view.cam_x, view.cam_y), view.zoom);
        view.cam_x = nx;
        view.cam_y = ny;
        view.zoom = nz;

        self.frame = Some(view.render(&m, self.mouse));
    }

    /// The frame's pixels, RGBA row-major, valid until the next `render`:
    /// `new ImageData(new Uint8ClampedArray(wasm.memory.buffer,
    /// frame_ptr(), frame_len()), frame_width(), frame_height())`
    pub fn frame_ptr(&self) -> *const u8 {
        self.frame.as_ref().map(|f| f.pixels.as_ptr()).unwrap_or(std::ptr::null())
    }

    /// `frame_width` * `frame_height` * 4 - the span `frame_ptr` covers
    pub fn frame_len(&self) -> usize {
        self.frame.as_ref().map(|f| f.pixels.len()).unwrap_or(0)
    }

    pub fn frame_width(&self) -> usize {
        self.frame.as_ref().map(|f| f.width).unwrap_or(0)
    }

    pub fn frame_height(&self) -> usize {
        self.frame.as_ref().map(|f| f.height).unwrap_or(0)
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
            Ok(c) => { self.game.command(c); true }
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
        use session::stage::{Stage, staged};
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
            Some(id) => { self.game.force_event(id); true }
            None => false,
        }
    }

    /// Run `days` days of frames - the same path the window plays
    pub fn run_days(&mut self, days: u64) {
        session::stage::run_days(&mut self.game, days);
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
        assert!(food < 20, "the staged famine's store is thin (got {})", food);

        assert!(g.force_event("the tinker"), "by its map name");
        assert!(!g.force_event("no such event"), "bad names are refused");
        let pv: serde_json::Value = serde_json::from_str(&g.probe()).unwrap();
        assert!(pv["rare_fired"].as_array().unwrap().iter()
            .any(|f| f[0] == "Tinker"), "the firing's on the probe's record");

        g.run_days(3);
        let scene: serde_json::Value = serde_json::from_str(&g.scene()).unwrap();
        assert!(scene["tick"].as_u64().unwrap() >= 3, "the days ran");
        // and the secret stays out of the scene: the canvas can't draw it
        let scene_json = g.scene();
        assert!(!scene_json.contains("grievances"), "the scene carries no inner lives");
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
        assert_eq!(manifest.len(), manifest.iter().collect::<std::collections::HashSet<_>>().len(),
            "no dupes in the manifest");

        // the page hands bytes back for whatever it fetched
        g.provide_asset(&manifest[0], b"not a sprite");
        g.open_view();
        g.tick();
        g.render();

        assert_eq!(g.frame_width() * g.frame_height() * 4, g.frame_len(),
            "the buffer's RGBA-full");
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
        let view = g.view.as_ref().unwrap();
        let (cx, cy) = (view.cam_x, view.cam_y);
        let m = g.game.ui_model();
        let p = m.people.iter().find(|p| p.kind == PKind::Player).unwrap();
        assert_eq!((cx, cy), session::camera_on(p.pos.0, p.pos.1),
            "the opening frame centres on the player");
        g.pan(40, -20);
        let view = g.view.as_ref().unwrap();
        assert_eq!((view.cam_x, view.cam_y), (cx + 40, cy - 20), "the pan landed");
    }
}
