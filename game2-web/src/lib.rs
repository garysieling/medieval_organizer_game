//! The browser client: the same game2-core sim/client code compiled to
//! wasm, so fixes land in one place. Connects to game_server over WS,
//! renders UiModel scenes to a canvas, keeps the claim token in
//! localStorage.
//!
//! Build: `cargo build -p game2-web --target wasm32-unknown-unknown`
//! then serve index.html + the wasm + assets/ over HTTP.
//!
//! NOTE: scaffolded - the ws/canvas bindings need a browser runtime shim
//! (wasm-bindgen + web-sys or a hand-rolled extern). See README in this dir.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    // TODO(wasm): ws connect -> Hello{token: localStorage.game2_claim}
    //            -> Scene -> View::render into a canvas ImageData
}
