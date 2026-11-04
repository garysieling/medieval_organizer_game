# Architecture & the layering refactor

The game is growing past what one file can hold, and several agents work on it
at once. This doc describes the layers we're moving toward and the order
we're taking the split in. **When you add code, put it in the layer it belongs
to, even if the move hasn't reached that area yet.**

## The layers

```
             +-------------------------------------------------+
  test /     |  probe (omniscient): ProbeView, secret KPIs,    |  tests, balance
  harness    |  stage (staged starts), scenario tests          |  tester, bots'
             +-------------------------------------------------+  graders
                              | reads everything
             +-------------------------------------------------+
  sim        |  Game: world, seats, ticks, events, dialogue,   |  authoritative,
  (server)   |  routines. No art, no camera, no window.        |  deterministic
             +-------------------------------------------------+  per seed
                 | ui_model_at(seat)        | agent_obs(seat)
                 v                          v
             +----------------+     +----------------+
  public     |  UiModel       |     |  AgentObs      |   what a player / an
  views      |  (model.rs)    |     |  (net.rs)      |   AI agent may know.
             +----------------+     +----------------+   Serializable; sent
                 |  ^ Command                |  ^ AgentCmd    over the wire
                 v  |                        v  |
             +----------------+     +----------------+
  clients    |  View (render) |     |  agent bots,   |
             |  desktop / web |     |  LLM drivers   |
             +----------------+     +----------------+
```

Rules:

1. **The sim never draws.** `Game` owns no sprites, camera or window state.
2. **Clients only read their public view.** `View::render` takes a `UiModel`,
   never a `Game`. An agent decides from `AgentObs` alone. This is what
   lets the same frame be drawn locally, in a browser, or by a remote client,
   and it's what keeps players and AIs non-omniscient.
3. **Only tests and harnesses use the probe.** `session::probe` exposes the hidden
   truth: which mushroom kills, the org's real strength under the lord's guess,
   what people secretly want. Nothing in `net.rs`, `model.rs` or the
   agent drivers may depend on it. The probe listener is a separate socket
   from the game socket.
4. **KPIs come in two kinds.**
   - *Public KPIs* are derived only from `UiModel` / `AgentObs`. That's
     what a player or an AI could compute about how they're doing.
   - *Secret KPIs* (the tuning gauges, `Game::telemetry`) read the true state,
     belong with the probe, and are for tests, balance runs and dashboards.
5. **Perf telemetry is separate from game KPIs.** `GAME2_BENCH` and the section
   profiler measure the engine (update/model/render ms per section). They
   live with the renderer and must cost nothing when off.

## Where code lives (target)

| Area | Path | Notes |
|------|------|-------|
| Session state & frame loop | `session.rs` (~1.4k lines) | `Game` struct and its types, `update`/`advance_frame` |
| Tests | `session/tests/<subsystem>.rs` | one file per subsystem; `use crate::session::*;` |
| Render (client) | `game2-client/src/` (its own crate) | `View`, `Art`, `TileSet`, draw helpers, HUD, overlays; uses only `session`'s public API |
| Screen layout | `session/layout.rs` | window/tile sizes, camera maths, minimap box: shared by the client and click hit-testing |
| Perf bench | `game2-client/src/bench.rs` | `GAME2_BENCH*`, section profiler, headless dumps |
| World generation | `session/setup.rs` | `Game::new_seeded` |
| Journal & replay | `session/journal.rs` | `Input`, `Game::apply_input` (the server's only way to change the world), `replay`, `state_digest` |
| Server (host) | `server.rs` | `serve(ServerConfig)`: WS seats, claims, the world loop, the probe socket; `game_server` is its CLI; tested in `tests/multiplayer.rs` |
| Seats & commands | `session/seat.rs` | `Seat`, `Command`, `CtxAct`, `command_at`, agent API, solo wrappers |
| Player verbs | `session/verbs.rs` | `act_at`, `ctx_act_at`, blockers, approach-then-act |
| Public views | `session/model.rs`, `session/net.rs` (types); `session/view_model.rs` (`ui_model_at`, `agent_obs`, log filters) | what a seat may know is decided in `view_model.rs` |
| KPIs | `session/kpi/public.rs`, `session/kpi/secret.rs` | public: from a view only; secret: `Game::secret_kpis` |
| Omniscient | `session/probe.rs` | `ProbeView` for tests and the probe socket |
| Sim subsystems | `session/subsystems/<family>.rs` | the `*_tick` families: wildlife, wood, economy, society, visitors, travel |
| Rare events | `session/events/` | `mod.rs` (ids, table, scheduler, dispatch) + one file per family |
| Pure rules | `game/`, `sim/` | ECS components & systems, no `Game` |

The client is its own crate (`game2-client`), so the compiler enforces rules 1 and 2;
rule 3 (probe use) is still enforced by review and tests.

## The order we're taking it in

Each step is its own PR and moves code without changing behaviour. Every test
on main must still be listed afterwards (`cargo test -p game2-core --lib -- --list`,
module paths aside). New tests the step adds on purpose (layering guards, perf
stats) are fine: name them in the PR.

1. ~~Tests out of `session.rs` into `session/tests/`.~~ (#457)
2. ~~Render layer: `View`, `Art`, `TileSet`, draw helpers, HUD and the bench
   runners into `session/render/`.~~ `tests/layering.rs` fails if the render
   files mention `Game`/`World`.
3. ~~Perf bench: `infra/perf.rs` sections across sim/ECS/render, p50/p95/max, `GAME2_BENCH_JSON`.~~
3b. ~~Determinism: sim code uses `engine::det::{HashMap, HashSet}` (fixed-key SipHash), so a
   seed replays exactly, across processes too. Guarded by `tests/determinism.rs` (two runs
   compared each day; `GAME2_DET_TICKS=N` runs it longer) and `tests/layering.rs`.~~
4. ~~World generation (`new`, `new_seeded`, `new_scenario`) into `setup.rs`.~~
5. ~~Seats, commands, the agent API (`seat.rs`), the player's verbs (`verbs.rs`) and the
   public projections `ui_model_at` / `agent_obs` (`view_model.rs`).~~
6. ~~Public vs secret KPIs: `session/kpi/{public,secret}.rs`.~~ `public_kpis(&UiModel)` /
   `agent_kpis(&AgentObs)` can't see the sim by construction (layering test). `Game::secret_kpis`
   is the old `telemetry()`. This closed two leaks: `UiModel.debug_lines` carried the secret
   gauges to every client, and an agent's `Welcome` carried seat 0's frame.
7. ~~Sim tick families into `session/subsystems/`~~: visitors, animals, wood, people (#465); economy,
   society, land, travel, meetings and the day (`livelihood_tick`). `session.rs` now holds the
   `Game` struct, its types and the frame loop (`update`/`advance_frame`).
8. ~~`events.rs` split by category~~: `events/mod.rs` keeps the ids, the `EVENTS` table, the
   scheduler and four thin dispatchers (`event_day/began/moment/end` -> `group(id)`); each
   family's arms live in `events/{road,sky,beasts,ill_luck,findings}.rs`; tests in `session/tests/events.rs`.
   A new event's handler goes in its family's file, in that family's `<family>_<handler>` fn.

9. Crates, in stages:
   - ~~3a: the client moves out of `session` to `crate::client` and uses only `session`'s
     public API. Shared screen geometry is public in `session/layout.rs`; `session.rs` no
     longer imports any drawing types.~~ The TALK and conversation panels' layout, drawing and
     click mapping moved from `session/dialogue.rs` to `client/talk.rs`; `tests/layering.rs`
     fails if `session` draws (`session_never_draws`) or names the client.
   - 3b-1: ~~the tests that draw move from `session/tests/` to `client/tests/` and use only the
     public API (58 tests; 22 files whole, 10 split).~~
   - 3b-2: ~~`client/` + `game_ui` + `record_video` become the `game2-client` crate on top of the
     sim. `game2-core` drops minifb/gif/jpeg-encoder; the server image builds only the sim.~~

### Working alongside the split

Moving code conflicts with branches that edit the old spot. If your branch
touches a moved area, rebase onto main and re-apply your change in the
new file. The moved code is byte-identical apart from indentation and
`super::` paths, so the diff of your change still applies by hand.
