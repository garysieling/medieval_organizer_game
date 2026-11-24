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
import { load as db_load, save as db_save } from './db.js';

const ASSET_BASE = new URL('../assets/', import.meta.url).href;
const params = new URLSearchParams(location.search);
const server = params.get('ws') || params.get('server');
const remote = !!server;

const boot = document.getElementById('boot');
const canvas = document.getElementById('screen');
const ctx = canvas.getContext('2d');
// the page's audio lives inside the wasm (game2-audio); the first
// gesture opens its AudioContext
const audio_start = () => { if (g) g.audio_start(); };

// the Sound dialog (#1507): the same AudioSettings the desktop's
// overlay writes, bound through set_audio_setting/audio_settings and
// parked in localStorage["game2.audio"]. The saved rows land on the
// wasm before audio starts; the fresh device inherits them.
const SND_KEY = 'game2.audio';
const snd_dialog = document.getElementById('sound-dialog');
const snd_rows = [...document.querySelectorAll('#sound-dialog [data-key]')];
const audio_persist = () => localStorage.setItem(SND_KEY, g.audio_settings());
function audio_load_saved() {
  try {
    const saved = JSON.parse(localStorage.getItem(SND_KEY) || 'null');
    if (saved) for (const [k, v] of Object.entries(saved)) {
      if (k !== 'version') g.set_audio_setting(k, v);
    }
  } catch (_) { /* a corrupt store is just defaults */ }
}
function snd_populate() {
  const s = JSON.parse(g.audio_settings());
  for (const el of snd_rows) {
    const k = el.dataset.key;
    if (!(k in s)) continue;
    if (el.type === 'checkbox') el.checked = s[k]; else el.value = s[k];
    if (el.dataset.out) document.getElementById(el.dataset.out).textContent = `${s[k]}%`;
  }
}
document.getElementById('sound-gear').addEventListener('click', () => {
  audio_start();            // the click's a gesture: the context may run
  snd_populate();
  snd_dialog.showModal();   // modal keeps the canvas inert beneath it
});
for (const el of snd_rows) {
  const apply = () => {
    const v = el.type === 'checkbox' ? el.checked
      : el.tagName === 'SELECT' ? el.value : +el.value;
    if (g.set_audio_setting(el.dataset.key, v)) {
      audio_persist();
      if (el.dataset.out) document.getElementById(el.dataset.out).textContent = `${el.value}%`;
    }
  };
  // ranges report as they're dragged; selects and checks on change
  el.addEventListener(el.type === 'range' ? 'input' : 'change', apply);
}
document.getElementById('snd-reset').addEventListener('click', () => {
  g.reset_audio_settings();
  audio_persist();
  snd_populate();
});
// mute-in-background follows the tab, like the desktop follows its window
addEventListener('visibilitychange', () => { if (g) g.audio_focused(!document.hidden); });

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

// the saved sound rows land before the first gesture can open audio
audio_load_saved();

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
  audio_start();
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
canvas.addEventListener('wheel', e => {
  // the live zoom's the base: the event camera may have set it (a
  // meeting opens at max) since the last manual step
  const zoom = JSON.parse(g.cam()).zoom;
  g.set_zoom(Math.min(2, Math.max(0.5, zoom * (e.deltaY < 0 ? 1.1 : 1 / 1.1))));
  e.preventDefault();
}, { passive: false });

const held = {};
addEventListener('keydown', e => {
  audio_start();
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
  }
  blit(g, wasm);
  requestAnimationFrame(loop);
}
say(remote ? 'listening for the village' : 'the first light');
if (!remote) boot.remove();
requestAnimationFrame(loop);
