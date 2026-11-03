# AGENTS.md - Game2 Project Instructions & Learned Information

## Layout
- Read `docs/architecture.md` before adding code: sim vs public views (`UiModel`, `AgentObs`) vs probe (omniscient, tests only) vs render. The split of `session.rs` is in progress there; put new code in its target layer.
- Session tests live in `game2-core/src/session/tests/<subsystem>.rs` (start with `use crate::session::*;` when the file uses session items; it reaches private ones too). Add new ones to the matching file, or add a file and list it in `tests/mod.rs`. Don't put them back in `session.rs`.

## Build & Test Instructions

### Windows GNU Toolchain Note
On Windows with `x86_64-pc-windows-gnu`, crates with Windows import library generation (like `getrandom` under `rand`) require `dlltool.exe`. If missing from standard PATH, it is located in rustup's self-contained tool directory:
```bash
PATH="$PATH:$USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained"
```

### Running Tests
- Test library modules:
  ```bash
  cargo test -p game2-core --lib game::conversation
  cargo test -p game2-core --lib
  ```
- Build binaries:
  ```bash
  cargo check --bin conversation_demo --bin dialogue_gen
  cargo run --bin conversation_demo
  cargo run --bin dialogue_gen -- -n 10 -m 5 -o evidence/dialogue-gen-sample.md
  ```

## Conversation System Architecture
- **Rules & Specificity Cascade (`game/conversation.rs`)**:
  - Declarative criteria (`Has`, `Lacks`) on world facts (`self:*`, `other:*`, `pair:*`, `any:*`).
  - Ruskin's scoring: most criteria wins.
  - Cascades degrade gracefully from high-specificity trees (sickness, grief, rivalry, clergy) into the generic `village_smalltalk` tail.
- **Social Considerations & Weighting**:
  - Trait multipliers on reply weights bias choices probabilistically (Prom Week / Comme il Faut model) without hard-gating.
- **Discourse Acts & Quip Exclusivity**:
  - `Act` enums (`Greet`, `Remark`, `Question`, `Answer`, `Agree`, `PushBack`, `Farewell`).
  - Lines are spent upon utterance to avoid in-conversation repetition and allow natural material exhaustion.
- **House Meetings**:
  - Looser model where attendees bid on speaking based on concept overlap with the organizer's open question, the previous speaker, or prior statements.
- **In-game wiring (`session/dialogue.rs`)**:
  - Villagers meeting in passing run `converse()`; the transcript goes into `Game::talk_log` (no map bubbles). Kids only talk to kids, on the `children_at_play` tree.
  - The player's Talk opens a `PlayerTalk` that is replayed from its seed plus the picks so far (`converse_with`), so it can wait across frames for a reply (`Command::TalkPick`).
  - House meetings use `run_meeting` lines for attendees who don't reveal an issue, and are recorded as they're said.
  - The TALK panel (under the minimap) shows the picked villager's conversations, or the player's own. The event LOG holds only events.

## Villager Routines (`session/routine.rs`)
- Each day a villager gets a `Routine` (mode + named spot + anchor): Working (bench door, or fetching inputs), Selling (a stall round the market on market day), Wandering (notices, creek, woods, stones, tavern, a visitor), Resting (home door), Worshipping (holy day, every 7th), Feasting (festival ring). Kids play.
- Planning sits in the daily loop after hunger/sickness/meetings/errands; traits weight it (Lazy rests, Pious worships, Gregarious strolls, Ambitious works). No work on rest/worship/selling days.
- Walks use `Travel` paths; between walks `amble` drifts about the anchor, never onto a blocked tile, and paths back if carried off. Holding a slot (worship, feast, stall) means not yielding to the crowd push.
- Holy day: church rows face the priest at the door; the old faith (1 in 5) rings the stones or circles their elder in the grove. The call only reaches a day's walk (`CALL_REACH`), so the service fills before it ends.
- Routine-steered villagers carry `villager::Steered`, which turns off `SeekFoodSystem`'s blind idle drift (it hopped folk into houses).
- Tests that teleport a villager somewhere must also give them a plan there (e.g. `plan_visit`), or they'll walk back to today's.

## Art Pipeline (Retro Diffusion)
- Full recipe: `assets/rd/README.md`. Pick from `art-prompts/prompts.md` -> `scripts/rd_generate.py <id>` (use `--dry-run` first; `--max-usd` guard defaults to $2) -> `scripts/rd_slice.py` -> wire into `Art::load` with a fallback -> verify with `game_ui --dump-frame N --zoom 2 --focus <kind>`.
- API token: `$RETRODIFFUSION_API_KEY` or `~/.config/retrodiffusion/token`. Never commit it.
- `art-prompts/generated.json` is tracked: it's the ledger of paid generations; raw outputs live in `art-prompts/raw/<id>/`.
- Sync API calls reject an `Idempotency-Key` header (`idempotency_async_required`).
- Pack art is drawn 2x; RD sprites are box-resampled to a target height (`name@H`) and drawn at 1x.
- RD sprite pixel size is fixed (~25px objects, ~35px walk figures): request 256 sheets and resample; a smaller canvas only clips. `four_angle_walking` needs 48px (32 fails).
- Ground-block prompts use `ISO_GROUND`, never `ISO` (it draws a house on the block). Tile/building swaps need the whole set (roads/rivers carry their own grass).
- Ground tiles are composed (`scripts/rd_compose_tiles.py`), not generated per piece. Walk prompts: describe appearance positively (negatives like "no helmet" backfire).
- `four_angle_walking` only draws people: a rider or animal comes back as a walker. `rd_pro__isometric` sizes the object to the canvas (64px ~ a cottage); ask 96px for something that should tower over the roofs (the windmill).
- Animation dumps: `GAME2_DUMP_WALK=dx,dy game_ui --dump-frame N --focus player` walks the player 8 frames before the shot.

## Render performance
- Measure with `GAME2_BENCH=N game_ui --dump-frame T --zoom Z ...` (prints update/model/render ms, per-section render times and the weather); `GAME2_BENCH_PAN=1` slides the camera every frame (uncached ground), `GAME2_BENCH_ZOOM=1` drifts the zoom. Benchmark late ticks too (600, 1500): weather and a grown village only show up there. `--weather` doesn't survive bench frames.
- The bench also prints p50/p95 per stage and a table of the slowest sections across layers: `render.*` (View passes), `sim.*` (daily ticks, per-frame advances), `ecs.*` (scheduler systems), `total.*` (wrappers that hold their children, so they don't add up with them). `GAME2_BENCH_JSON=path` writes the run as JSON. Sim ticks only run on day frames, so check their p95/max, not their mean.
- New sim work: wrap it as `crate::timed!("sim.<name>", ...)` (or `infra::perf::scope`) so it shows in the table. It's free when no bench runs.
- Per-pixel full-frame passes (weather, tints) must not do float maths per pixel: a fixed per-channel mapping is a 256-entry table (`f32::round` is a libm call here).
- Never `sprite.scaled(w, h).blit(..)` in a per-frame path: use `blit_resized` (same pixels, no copy, clipped). Anything drawn per map piece must go through `off_view` culling.
- Generated buildings on a grass slab are exported with `rd_slice.py --flush-base`, or they float above their tile.
- Props and wall pieces that must sit on any ground (snow, rock): `--strip-base` (a tree's dirt patch) or `--flush-base --strip-slab` (a stone piece's grass slab); they draw at 1x via `draw_prop(k=1)` / `draw_standing`. One wall/gate block is generated and `--mirror`ed for the other axis.
