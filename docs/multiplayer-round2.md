# Multiplayer — round 2 punch list

Shipped in this PR: sim extracted to `game2-core::session` (art-free `Game`,
`View` owns rendering), `UiModel` snapshot boundary (one render path, local
and remote), `Command` input seam, multi-seat `Game` (`Seat` + seat-0
`Deref`), `game_server` (WS, claims, real-time ticks, per-seat scenes, idle
fallback), `game_ui --connect`, agent API (`AgentObs`/`AgentCmd`),
`game_bots`, Dockerfile + k8s, `speed-timer` feature flag. 353 tests green;
claim/reclaim and command round-trip verified live.

## Known gaps to fix

- **`sync_purse` runs for seat 0 only** — a remote player's
  `PlayerCharacter.resources` never mirrors their `Purse`. Loop it per
  seat in `update()`.
- **`drive_agent` heuristic is thin** (Eat → WorkDay → Idle). Wants: craft
  input gathering (`BuyFrom`/`Gather` when the bench needs stock), sell
  surplus, rest at own home, train juniors, occasional `TalkTo` for ties —
  keep the village-first bias, add probabilities not just priorities.
- **Inspector on non-zero seats** — `building_inspector` reads seat 0's
  selection (never parameterized); `villager_inspector` is fine.
- **Scene bandwidth** — every broadcast serializes the whole `UiModel`
  (incl. `TileMap`). Fine at 1–5 players; if it matters, send the map once
  per claim and diffs after.
- **`AgentCmd::Eat`** paths to the market but never explicitly eats; the
  builtin hunger system handles it once the villager is idle. Verify.
- **No scripted multiplayer regression** — add a test: server + 1 human +
  1 bot, assert claims, commands land, scenes stream.
- **Bots don't reclaim** — `game_bots` sends `token: None` every run;
  accept a token file like the UI client for stable villager identity.
- **`free_villager`** assigns by index, not role — bots may claim the
  priest/collector; consider a villager eligibility filter.
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
