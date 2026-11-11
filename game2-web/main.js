// The browser boot for game2: run the same sim and the same software
// renderer the desktop window runs, and blit the RGBA buffer it hands
// back onto a canvas.
//
// Serve the repo root over HTTP and open /game2-web/:
//   cargo build -p game2-web --release --target wasm32-unknown-unknown
//   wasm-bindgen target/wasm32-unknown-unknown/release/game2_web.wasm \
//       --out-dir game2-web/pkg --target web
//   python -m http.server            # from the repo root
//
// The art loads the way the design doc says: the wasm asks for the asset
// paths Art::load wants, the page fetches each from assets/ (a sibling
// of this file's directory, i.e. the repo root), and hands the bytes back.

import init, { WasmGame } from './pkg/game2_web.js';

const ASSET_BASE = new URL('../assets/', import.meta.url).href;
const boot = document.getElementById('boot');
const canvas = document.getElementById('screen');
const ctx = canvas.getContext('2d');

function say(msg) { boot.textContent = msg; }

async function feed_assets(g) {
  const manifest = JSON.parse(g.asset_manifest());
  say(`fetching the art — ${manifest.length} files`);
  let done = 0;
  const step = () => { done++; if (done % 20 === 0) say(`fetching the art — ${done}/${manifest.length}`); };
  await Promise.all(manifest.map(async rel => {
    try {
      const r = await fetch(ASSET_BASE + encodeURI(rel));
      if (r.ok) g.provide_asset(rel, new Uint8Array(await r.arrayBuffer()));
    } catch (_) { /* a miss draws the placeholder look, as on the desktop */ }
    step();
  }));
}

function bind_input(g) {
  // pointer: hover feeds the flyer's mouse, a drag pans (and mutes the
  // event camera, like the window's pans)
  let drag = null;
  canvas.addEventListener('pointermove', e => {
    const scale = canvas.width / canvas.getBoundingClientRect().width;
    g.set_mouse(Math.round(e.offsetX * scale), Math.round(e.offsetY * scale));
    if (drag) {
      g.pan(Math.round((drag.x - e.clientX) * scale), Math.round((drag.y - e.clientY) * scale));
      drag = { x: e.clientX, y: e.clientY };
    }
  });
  canvas.addEventListener('pointerdown', e => { drag = { x: e.clientX, y: e.clientY }; });
  addEventListener('pointerup', () => { drag = null; });
  let zoom = 1.0;
  canvas.addEventListener('wheel', e => {
    zoom = Math.min(2, Math.max(0.5, zoom * (e.deltaY < 0 ? 1.1 : 1 / 1.1)));
    g.set_zoom(zoom);
    e.preventDefault();
  }, { passive: false });
  // the one input worth wiring early: space pauses, through the same
  // Command the ws protocol carries
  addEventListener('keydown', e => {
    if (e.code === 'Space') { g.command('"TogglePause"'); e.preventDefault(); }
  });
}

const wasm = await init();
const g = new WasmGame();
await feed_assets(g);
say('hanging the sprites');
g.open_view();
bind_input(g);

say('the first light');
g.tick();
g.render();
canvas.width = g.frame_width();
canvas.height = g.frame_height();
boot.remove();

function frame() {
  g.tick();
  g.render();
  const px = new Uint8ClampedArray(wasm.memory.buffer, g.frame_ptr(), g.frame_len());
  ctx.putImageData(new ImageData(px, g.frame_width(), g.frame_height()), 0, 0);
  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
