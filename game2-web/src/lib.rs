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
use game2_core::session::{Game, Scenario, Command};

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
    }
}
