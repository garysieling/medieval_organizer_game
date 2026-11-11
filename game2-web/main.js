// The browser client for game2, in two modes:
//
//   /game2-web/?ws=host:8787   a client of game_server - the page is a
//                              thin seat: it Hellos the server (reclaiming
//                              its character by the saved token), plays the
//                              picker and the prologue card, and streams
//                              scenes into the wasm renderer. The shipped
//                              mode.
//   /game2-web/                the dev harness: the sim itself runs in
//                              wasm, ticked by the page (?seed=N or
//                              ?staged=famine picks the telling). Its
//                              journal save lives in IndexedDB, so a
//                              bare reload resumes the same village;
//                              ?fresh starts it over.
//
// Serve the repo root over HTTP and open /game2-web/:
//   cargo build -p game2-web --release --target wasm32-unknown-unknown
//   wasm-bindgen target/wasm32-unknown-unknown/release/game2_web.wasm \
//       --out-dir game2-web/pkg --target web
//   python -m http.server            # from the repo root
//
// The art loads the way the design doc says: the wasm asks for the asset
// paths Art::load wants, the page fetches each from assets/ (a sibling
// of this file's directory), and hands the bytes back.

import init, { WasmGame, WasmClient } from './pkg/game2_web.js';
import { SoundBoard } from './sound.js';
import { load as db_load, save as db_save } from './db.js';

const ASSET_BASE = new URL('../assets/', import.meta.url).href;
const params = new URLSearchParams(location.search);
const server = params.get('ws') || params.get('server');
const remote = !!server;

const boot = document.getElementById('boot');
const canvas = document.getElementById('screen');
const ctx = canvas.getContext('2d');
const sound = new SoundBoard();

function say(msg) {
  if (!boot.isConnected) document.getElementById('wrap').appendChild(boot);
  boot.textContent = msg;
}
function blit(g, wasm) {
  if (!g.frame_len()) return;
  const px = new Uint8ClampedArray(wasm.memory.buffer, g.frame_ptr(), g.frame_len());
  ctx.putImageData(new ImageData(px, g.frame_width(), g.frame_height()), 0, 0);
}

async function feed_assets(g) {
  const manifest = JSON.parse(g.asset_manifest());
  say(`fetching the art — ${manifest.length} files`);
  let done = 0;
  await Promise.all(manifest.map(async rel => {
    try {
      const r = await fetch(ASSET_BASE + encodeURI(rel));
      if (r.ok) g.provide_asset(rel, new Uint8Array(await r.arrayBuffer()));
    } catch (_) { /* a miss draws the placeholder look, as on the desktop */ }
    if (++done % 20 === 0) say(`fetching the art — ${done}/${manifest.length}`);
  }));
}

const wasm = await init();
// the embedded harness remembers: a bare visit resumes the last
// village off the shelf; a named telling (?seed, ?staged, ?fresh)
// starts the village it names. The remote seat asks the server, not
// the shelf.
let g;
if (remote) {
  g = new WasmClient();
} else {
  const named = params.get('seed') || params.get('staged') || params.get('fresh') !== null;
  const record = named ? null : await db_load('village');
  g = record ? WasmGame.load(record) : null;
  if (!g) {
    g = params.get('seed') ? WasmGame.with_seed(+params.get('seed'))
      : (params.get('staged') ? WasmGame.staged(params.get('staged')) : new WasmGame());
  }
}

// 'connect' waits on the Welcome; 'picker' deals the roster; 'intro'
// shows the prologue card; 'play' runs the loop. The embedded harness
// starts straight in play.
let phase = remote ? 'connect' : 'play';
let sel = 0;
let welcome = false;
let map_seen = false;

// the one send both modes share: remote wraps in ClientMsg::Command,
// the harness feeds its own sim
const send = cmd => {
  if (remote) ws.send(JSON.stringify({ Command: cmd }));
  else g.command(JSON.stringify(cmd));
};

let ws = null;
if (remote) {
  const url = /^wss?:/.test(server) ? server : `ws://${server}`;
  ws = new WebSocket(url);
  ws.onopen = () => ws.send(JSON.stringify({ Hello: {
    token: localStorage.getItem('game2.claim'),
    kind: 'Human', name: null, secret: null,
  }}));
  ws.onmessage = e => {
    const f = g.push_message(e.data);
    if (f & 16) map_seen = true;
    if (f & 2) {
      welcome = true;
      localStorage.setItem('game2.claim', g.claim_token());
    }
    if (f & 4) { phase = 'dead'; say(`the village turned you away: ${g.denied_reason()}`); }
  };
  ws.onerror = () => say(`couldn't reach ${server} - is game_server up?`);
  ws.onclose = () => { if (phase !== 'dead') say('the line to the village went quiet'); };
}

say(remote ? 'reaching the village' : 'waking the village');
await feed_assets(g);
// the remote renderer's seed folds in the Map - wait for it so every
// client of a server draws the same village (ten seconds, then whatever
// seed an empty map gives)
if (remote) {
  const t0 = performance.now();
  while (!map_seen && phase !== 'dead' && performance.now() - t0 < 10000)
    await new Promise(r => setTimeout(r, 100));
}
say('hanging the sprites');
g.open_view();
canvas.width = 1440; canvas.height = 960;

// the pointer: hover feeds the flyer's mouse, a drag pans (and mutes the
// event camera, like the window's pans); a click is a real ClickMap
let drag = null, down_at = null;
const scale = () => canvas.width / canvas.getBoundingClientRect().width;
canvas.addEventListener('pointermove', e => {
  const s = scale();
  g.set_mouse(Math.round(e.offsetX * s), Math.round(e.offsetY * s));
  if (drag) {
    g.pan(Math.round((drag.x - e.clientX) * s), Math.round((drag.y - e.clientY) * s));
    drag = { x: e.clientX, y: e.clientY };
  }
});
canvas.addEventListener('pointerdown', e => {
  sound.resume();
  drag = { x: e.clientX, y: e.clientY };
  down_at = { x: e.clientX, y: e.clientY, t: performance.now() };
});
canvas.addEventListener('pointerup', e => {
  if (down_at && Math.hypot(e.clientX - down_at.x, e.clientY - down_at.y) < 4
      && performance.now() - down_at.t < 300 && phase === 'play') {
    const s = scale(), cam = JSON.parse(g.cam());
    send({ ClickMap: {
      mx: Math.round(e.offsetX * s), my: Math.round(e.offsetY * s),
      cam_x: cam.x, cam_y: cam.y, zoom: cam.zoom }});
  }
  drag = null; down_at = null;
});
let zoom = 1.0;
canvas.addEventListener('wheel', e => {
  zoom = Math.min(2, Math.max(0.5, zoom * (e.deltaY < 0 ? 1.1 : 1 / 1.1)));
  g.set_zoom(zoom);
  e.preventDefault();
}, { passive: false });

const held = {};
addEventListener('keydown', e => {
  sound.resume();
  held[e.code] = true;
  if (phase === 'picker') {
    const n = g.roster_len();
    if (e.code === 'ArrowUp' || e.code === 'KeyW') sel = (sel + n - 1) % n;
    else if (e.code === 'ArrowDown' || e.code === 'KeyS') sel = (sel + 1) % n;
    else if (e.code === 'Enter') { ws.send(JSON.stringify({ Pick: sel })); phase = g.intro_lines() !== '[]' ? 'intro' : 'play'; }
    else if (e.code === 'Escape') { ws.send(JSON.stringify('DeclinePick')); phase = g.intro_lines() !== '[]' ? 'intro' : 'play'; }
    e.preventDefault(); return;
  }
  if (phase === 'intro') {
    if (e.code === 'Enter' || e.code === 'Space' || e.code === 'Escape') phase = 'play';
    e.preventDefault(); return;
  }
  if (phase !== 'play') return;
  if (e.code === 'Space') { send('TogglePause'); e.preventDefault(); }
  else if (e.code === 'Escape') send('Unselect');
  else if (e.code === 'KeyX') send({ TalkPick: null });
  else if (/^Digit[1-8]$/.test(e.code)) {
    const c = g.digit_command(+e.code[5] - 1);
    if (c) send(JSON.parse(c));
  }
});
addEventListener('keyup', e => { held[e.code] = false; });

// the harness journals its village to IndexedDB, so a closed tab isn't
// a lost village: every ten seconds, and again as the page goes away
if (!remote) {
  const persist = () => db_save('village', g.save_json());
  setInterval(persist, 10000);
  addEventListener('visibilitychange', () => { if (document.hidden) persist(); });
  addEventListener('pagehide', persist);
}

// walk is a per-frame direction while the keys are held, like the window
const walk_vec = () => [
  (held.KeyD || held.ArrowRight ? 1 : 0) - (held.KeyA || held.ArrowLeft ? 1 : 0),
  (held.KeyS || held.ArrowDown ? 1 : 0) - (held.KeyW || held.ArrowUp ? 1 : 0),
];

function loop() {
  if (phase === 'connect' && welcome && g.roster_len() === 0) phase = 'play';
  if (phase === 'connect' && welcome && g.roster_len() > 0) phase = 'picker';
  if (phase !== 'connect' && boot.isConnected) boot.remove();

  if (phase === 'picker') {
    g.render_picker(sel);
  } else if (phase === 'intro') {
    g.render_intro();
  } else if (phase === 'play') {
    const [dx, dy] = walk_vec();
    if (dx || dy) send({ Walk: [dx, dy] });
    if (!remote) g.tick();
    g.render();
    sound.drain(JSON.parse(g.take_sfx()));
  }
  blit(g, wasm);
  requestAnimationFrame(loop);
}
say(remote ? 'listening for the village' : 'the first light');
if (!remote) boot.remove();
requestAnimationFrame(loop);
