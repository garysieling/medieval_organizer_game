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
| Session state & frame loop | `session.rs` (shrinking) | `Game` struct, `update`, glue |
| Tests | `session/tests/<subsystem>.rs` | one file per subsystem; `use crate::session::*;` |
| Render (client) | `session/render/` | `View`, `Art`, `TileSet`, draw helpers, HUD, overlays |
| Perf bench | `session/render/bench.rs` | `GAME2_BENCH*`, section profiler, headless dumps |
| World generation | `session/setup.rs` | `Game::new_seeded` |
| Seats & commands | `session/seat.rs` | `Seat`, `Command`, `command_at`, ctx actions |
| Public views | `session/model.rs`, `session/view_model.rs`, `session/net.rs` | `UiModel`, `AgentObs` |
| Omniscient | `session/probe.rs`, `session/kpi.rs` | probe + secret/public KPIs |
| Sim subsystems | `session/sim/<subsystem>.rs` | the `*_tick` families: wildlife, wood, economy, society, visitors, travel |
| Rare events | `session/events/` | split by category |
| Pure rules | `game/`, `sim/` | ECS components & systems, no `Game` |

Later, once these seams hold, the layers get promoted to crates
(`game2-sim`, `game2-client`) so the compiler enforces rules 1-3.

## The order we're taking it in

Each step is its own PR and moves code without changing behaviour. The test
list must be identical before and after (`cargo test -p game2-core --lib -- --list`).

1. Tests out of `session.rs` into `session/tests/`.
2. Render layer: `View`, `Art`, `TileSet`, draw helpers into `session/render/`.
3. Perf bench & headless runners into `render/bench.rs`; add p50/p95/max.
4. World generation (`new_seeded`) into `setup.rs`.
5. Seats, commands, the agent API and `ui_model_at` into their own files.
6. Public vs secret KPIs (`kpi.rs`), plus a test that the agent/player views
   carry nothing from the probe.
7. Sim tick families into `session/sim/`, a few per PR.
8. `events.rs` split by category.

### Working alongside the split

Moving code conflicts with branches that edit the old spot. If your branch
touches a moved area, rebase onto main and re-apply your change in the
new file. The moved code is byte-identical apart from indentation and
`super::` paths, so the diff of your change still applies by hand.
