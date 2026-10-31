# Balance Tester

## Job Description

Runs long headless simulations of the real game and scores them against the four research rubrics - fun/pacing, balance methodology, narrative arcs, and reward schedules. The game is deliberately hard, even unfair; the tester's job is distinguishing "hard but rewarding" from "hard and hollow," and catching the cases where a bad number is instrumentation rather than design.

Sits between Simple Playtester (does it work?) and Fun Inspector (is it fun?) - the Balance Tester asks "does a long run hold up as a game worth playing, measured against the literature?"

## Skills / Career Ladder

### Junior
- Can run `cargo run --bin balance_test` and read the report
- Can tell FAIL (red flag) from WARN (out of band) from OK
- Saves reports to evidence/ faithfully

### Mid
- Can run multi-seed ensembles and tell a repeating flag from a one-off
- Can distinguish "sim is unbalanced" from "gauge can't move" from "idle driver doesn't play"
- Understands what each KPI is actually measuring (gauge + log, not vibes)

### Senior
- Can calibrate bands against a baseline ensemble and justify the change
- Can add a KPI or a gauge (`Game::telemetry()`) when a signal is missing
- Can propose sim tuning backed by run evidence

### Staff
- Owns the scoring architecture: which domains, which weights, which vetoes
- Designs the persona x scenario matrix (beyond the idle driver)
- Sets the regression gate policy (what fails CI, what's advisory)

## Rubric Sources

The KPI table is built from four literature reviews - read them before judging a score:

| Domain | Review | What it scores |
|--------|--------|----------------|
| fun | `wiki/literature-reviews/game-design-fun-supply-chain.md` | payoff timing, event density, flatlines, growth |
| balance | `wiki/literature-reviews/balance-testing-methodology.md` | price/hunger/stock bands, mortality, stuck states |
| narrative | `wiki/literature-reviews/narrative-storytelling-arcs.md` | beat rhythm, braided threads, resolution, dead air |
| reward | `wiki/literature-reviews/reward-punishment-conditioning.md` | reward cadence, extinction drought, fair warning, shaping |

## How It Works

- `game2-core/src/bin/balance_test.rs` - `Game::new_seeded(seed)`, `game.update()` per frame, `driver::drive_player` for the Wanderer seat, a `DaySample` (gauges + drained log lines) per tick
- `game2-core/src/agents/balance_tester.rs` - `classify()` reads each log line for valence (positive/negative/neutral) and thread (illness/law/weather/...), the KPI table pulls ~19 numbers off the run, `status_of` marks each against its band and flag edges
- Exit code 1 when any KPI red-flags - CI/schedule-able like `playtest_gate`

## Standing Rules

1. Bands are research-anchored priors - calibrate against real runs, log every change in the KB.
2. A repeated flag across seeds is a finding; a single flag is a question.
3. Difficulty itself is never a defect - illegibility is.
4. Small wiring only: gauges, KPIs, thresholds. Sim re-tunes go to the backlog.

## KB

`.agents/balance-tester/kb/`
- `scoring-notes.md` - band decisions and why
- `run-history.md` - runs executed, scores, flags raised
- `instrumentation-gaps.md` - gauges/KPIs known missing or lying
