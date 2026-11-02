//! The browser client: the same game2-core sim compiled to wasm, so fixes
//! land in one place. Exposes a `WasmGame` - a Game + its serialized
//! UiModel scene, for the canvas shim to draw.
//!
//! Build: `cargo build -p game2-web --target wasm32-unknown-unknown`
//! then serve index.html + the wasm + assets/ over HTTP.
//!
//! NOTE: the ws/canvas bindings still need a browser runtime shim - the
//! scene's a serialized UiModel; the canvas draws it.

use wasm_bindgen::prelude::*;
use game2_core::session::{self, Game, Scenario, Command};

/// A wasm-held Game: the browser ticks it and asks for the scene
#[wasm_bindgen]
pub struct WasmGame {
    game: Game,
}

#[wasm_bindgen]
impl WasmGame {
    /// The default village, mid-story
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame { game: Game::new_seeded(1) }
    }

    /// A saved scenario, mid-telling
    pub fn scenario(kind: &str) -> WasmGame {
        let s = match kind {
            "fever" => Scenario::FeverYear,
            "circus" => Scenario::CircusWinter,
            "widow" => Scenario::WidowsShop,
            _ => Scenario::FeverYear,
        };
        WasmGame { game: Game::new_scenario(s) }
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

    /// The tuning panel's lines, for a debug readout
    pub fn telemetry(&self) -> String {
        self.game.telemetry_lines().join("\n")
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
        WasmGame { game: staged(s) }
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
}
