# PR #212 — Balance tester: a new agent/test mode that marks long runs

https://github.com/garysieling/game2/pull/212

## What changed
- `game2-core/src/agents/balance_tester.rs` — RunRecord/DaySample capture, log-line `classify()` (valence + story thread), 19-KPI table across fun/balance/narrative/reward with target bands + red-flag edges, OKR rollup, text report renderer.
- `game2-core/src/bin/balance_test.rs` — headless harness: `Game::new_seeded(seed)`, `game.update()` per frame, `driver::drive_player` for the Wanderer seat, one DaySample per tick. Exits 1 on any red flag.
- `session.rs` — new `stock.food` telemetry gauge (FoodSource servings on market_food); the grain/bread building-stock reads underreported real food.
- `.claude/skills/balance-tester/SKILL.md`, `wiki/agents/balance-tester.md`, `.agents/balance-tester/kb/` — agent mode, doc, seed KB (scoring notes, run history, instrumentation gaps).
- `PLAY.md` — binary listed in the dev section.

## Quality gates
- `cargo build -p game2-core` clean (worktree-local target dir; `dlltool` had to be added to PATH for fresh dep builds — `rustlib/.../bin/self-contained`).
- `cargo test -p game2-core --lib` 504/504 incl. 4 new scorer tests.
- Live runs: `balance_test 200 {42,7,1234}` — reports produced, repeatable flags (price_cv=0, hunger_peak~0.7).

## Findings from first runs (logged to KB)
- `price_cv = 0.00` every seed — gauge prices static; instrumentation gap, not gameplay.
- `hunger_peak ≈ 0.68-0.71` vs `food_cover ≈ 15.5d` — stocked store, hungry village: access-vs-availability question.
- `punishment_warn ≈ 0.14-0.46` — negative beats mostly land without same-thread warning.
- `valence_ratio ≈ 7.4-13.7` — above band; likely classifier over-credit for ambient positives (severity-weighting is a documented next step).
- `time_to_first_growth` never fires under the idle driver — measures the passive-player experience until a real policy exists.

## Linked work
- Research basis: PRs #208-#211 (four lit reviews, 53 KPIs + 20 OKRs; first cut = 19 KPIs in code).
- KB notes: `.agents/balance-tester/kb/`.
