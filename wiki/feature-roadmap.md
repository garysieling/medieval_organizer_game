---
created: 2026-10-30
status: ordered backlog — work top-down, one feature per PR
---
# Feature Roadmap — Missing Features, Spec'd for Incremental Work

A living list of features the game doesn't have yet, specced so each one
can be implemented as a single incremental PR that leaves the game a bit
better and all tests green. Ordered: earlier items are higher-value,
lower-risk, and lay groundwork later items reuse.

## How to work the list

- **One feature = one PR.** Don't batch; each merge must stand alone.
- **Additive modules.** New behavior goes in a `game/` (or `sim/`) module;
  wire into `game_ui.rs` without disturbing the text game or `jobs.rs`
  users — same rule as `skills-vocations-plan.md`.
- **Check overlap first.** Many `feat/*` remote branches are pending or
  squash-merged under different names (e.g. `feat/apothecary-prepares-remedies`,
  `feature/trail-strength-feedback-loop`, `feature/contagion-spreads-by-proximity`).
  Before starting an item, `git fetch` and grep the pending branches for
  its topic; skip or re-scope what's already landed.
- **Verify in repo style:** unit tests per module; `cargo test --lib`
  green; `cargo run --bin game_ui -- --dump-frame N` evidence BMPs where
  the feature is visible; update `PLAY.md` when controls/cards change.
- **Traceability.** Each item notes the lit review or brainstorm line it
  draws on, so the spec can be deepened on demand.

---

## Wave 1 — The cottage economy (animals, plants, and the people who tend them)



## Wave 5 — Meta & quality (work in when convenient)

- **F36. Perf/batch pass** — flock + villagers > 40 entities: batch
  separation pass, sprite cache, path memo (planned in
  skills-vocations-plan's phase 5).
- **F37. Debug/telemetry overlay** — trails heatmap, power matrix,
  economy prices over time; the tuning panel lit reviews ask for.
- **F40. Sound** — ambient loop + event chimes; last per backlog priority.

---

## Suggested opening run — in order

1. ~~F1 Herbologist~~ — **done, PR #141.**
2. ~~F2 Shepherds/flocks~~ — **done, PR #142.**
3. ~~F3 Chickens~~ — **done, PR #143.**
4. ~~F41 Heart Helper~~ — **done, PR #144.**
5. ~~F5 Foraging/mushrooms~~ — **done, PR #145.**
6. ~~F43 Issues belong to trades~~ — **done, PR #148.**
   grievance richer; do it before the roster of scoped problems grows.
7. ~~F7 Vermin/cats~~ — **done, PR #147.**
   now cats do too.
8. ~~F12 Shop succession~~ — **done, PR #149.**
9. ~~F11 Children grow up~~ — **done, PR #150.**
10. ~~F18 Footfall trails~~ — **done, PR #151.**
    already written.
11. ~~F42 The land keeps score~~ — **done, PR #152.**
    issues F43 makes matter; needs a few features' worth of economy to
    bite.
12. ~~F13 Sanction ladder~~ — **done, PR #153.**
    ergot paths converge here.
13. ~~F15 Festivals~~ — **done, PR #154.**
    the first thing that feels like a *village*, not a sim.
