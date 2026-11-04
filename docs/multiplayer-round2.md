# Multiplayer — round 2 punch list

Shipped in this PR: sim extracted to `game2-core::session` (art-free `Game`,
`View` owns rendering), `UiModel` snapshot boundary (one render path, local
and remote), `Command` input seam, multi-seat `Game` (`Seat` + seat-0
`Deref`), `game_server` (WS, claims, real-time ticks, per-seat scenes, idle
fallback), `game_ui --connect`, agent API (`AgentObs`/`AgentCmd`),
`game_bots`, Dockerfile + k8s, `speed-timer` feature flag. 353 tests green;
claim/reclaim and command round-trip verified live.

## Known gaps — status

Fixed since this list was written:

- ~~`sync_purse` runs for seat 0 only~~ — done; the purse mirrors per seat.
- ~~Inspector on non-zero seats~~ — done; `building_inspector` is
  seat-parameterized.
- ~~`AgentCmd::Eat` never explicitly eats~~ — done (#232): the agent's
  Eat actually eats.
- ~~Bots don't reclaim~~ — done (#228): `game_bots` keeps a
  `game2-bot-{n}.claim` file like the UI client's `game2.claim`.
- ~~`free_villager` assigns by index~~ — done (#228): postings
  (`TownRole`, PMC `Craft`s) and kids are off the menu.
- ~~`AgentObs` thin~~ — improved (#230): carries `held`, `orgs`,
  `campaigns`, and nearby `burning` tiles.

Still open:

- **`drive_agent` heuristic is thin** (Eat → WorkDay → Idle). Wants: craft
  input gathering (`BuyFrom`/`Gather` when the bench needs stock), sell
  surplus, rest at own home, train juniors, occasional `TalkTo` for ties —
  keep the village-first bias, add probabilities not just priorities.
- ~~**Scene bandwidth**~~ — done: a scene was ~1 MB, 98% of it the
  `TileMap` (~10 MB/s per human seat). Scenes now go without the map (~22 KB);
  `ServerMsg::Map` follows the Welcome and goes again only when
  `TileMap::digest` moves. Clients rebuild frames with `SceneCache`.
- ~~**No scripted multiplayer regression**~~ — done: the server is a
  library (`game2-core/src/server.rs`, `serve(ServerConfig)`), and
  `game2-core/tests/multiplayer.rs` plays one over real sockets: a human and
  an agent claim seats, scenes/obs stream, commands land, ping/pong, a
  token reclaim resumes the character and retires the old socket, nothing
  secret on the wire, the probe socket demands its token.
- ~~**Unbounded outbox**~~ — done: scenes/obs/probe frames are snapshots,
  so a connection keeps only the newest unsent one; replies queue in order.
- **Remote `click_map`** rebuilds cam-dependent selection server-side;
  camera mismatch between seats could mis-select — acceptable now.

## Bigger pieces for round 2

- **`game2-web` WASM client** — scaffold only. Needs wasm-bindgen +
  web-sys (ws, canvas, localStorage), asset fetch for `Art::load`, and a
  `wasm32` CI step so it doesn't rot.
- **Pause/speed retirement** — `speed-timer` feature stays off by default;
  web/remote never see it. Drop the key and `speed` field fully later.
- ~~**Persistence**~~ — first cut done, via the journal
  (`session/journal.rs`): the server applies every input through
  `Game::apply_input` and notes it with its frame; the sim is deterministic,
  so seed + journal rebuild the world exactly. `game_server --save FILE`
  writes journal, claims and frame every `--save-every` seconds (30) and on
  stop, and a server started on the file replays it (checking a state
  digest) - tokens keep their characters across restarts. Limit: a restore
  replays every frame (~25s per hour of play), so long-lived servers will
  want periodic snapshots (serde-ified components) on top.
- ~~**Auth**~~ — done, lightly: `game_server --join-secret W` (or
  `GAME2_JOIN_SECRET`) makes every Hello carry the word (`Hello.secret`;
  `game_ui --connect` and `game_bots` read `GAME2_JOIN_SECRET`), and
  `--max-seats N` (default 64) caps the characters a server hands out.
  Reclaims with a token always work. Sockets have a 10s handshake/Hello
  deadline and at most 256 are open at once. Not real accounts: anyone with
  the word plays.
- ~~**Protocol docs**~~ — done: `docs/protocol.md`, with every message's
  JSON shape; its examples are parsed by `tests/protocol_doc.rs`.
- **Obs polish** — agent obs could carry shop stock, wants, and craft
  level so bots can plan, not just react.
