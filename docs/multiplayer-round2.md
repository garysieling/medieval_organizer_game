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
- **Scene bandwidth** — every broadcast serializes the whole `UiModel`
  (incl. `TileMap`). Fine at 1–5 players; if it matters, send the map once
  per claim and diffs after.
- **No scripted multiplayer regression** — add a test: server + 1 human +
  1 bot, assert claims, commands land, scenes stream.
- **Remote `click_map`** rebuilds cam-dependent selection server-side;
  camera mismatch between seats could mis-select — acceptable now.

## Bigger pieces for round 2

- **`game2-web` WASM client** — scaffold only. Needs wasm-bindgen +
  web-sys (ws, canvas, localStorage), asset fetch for `Art::load`, and a
  `wasm32` CI step so it doesn't rot.
- **Pause/speed retirement** — `speed-timer` feature stays off by default;
  web/remote never see it. Drop the key and `speed` field fully later.
- **Persistence** — the world dies with the process; `save_load` only has
  entity counts. Serialize `World`/components (serde-ify components) so a
  pod restart keeps the village.
- **Auth** — claims are unguessable tokens but anyone can mint seats; a
  shared secret or per-seat cap would harden it lightly.
- **Protocol docs** — `docs/protocol.md` with the JSON shapes + a sample
  agent session, for people writing bots.
- **Obs polish** — agent obs could carry shop stock, wants, and craft
  level so bots can plan, not just react.
