# Scoring Notes - Balance Tester KB

## Band provenance

All KPI bands are research-anchored priors from the four lit reviews
(game-design-fun-supply-chain, balance-testing-methodology,
narrative-storytelling-arcs, reward-punishment-conditioning). The reviews
explicitly call for calibration over a baseline ensemble; expect movement.

## First calibration run (2026-10-31, seed 42, 200 days, dev build)

- price_cv = 0.00 FAIL - `vocation::price()` is static; per-entity markets
  exist (economics.rs) but the gauge goods never move. Instrumentation gap,
  not gameplay. Keep the KPI; the flag is honest.
- hunger_peak = 0.68 FAIL while food_cover_days = 15.5 OK - the store is
  stocked but villagers stay hungry: SeekFood is gated by more than
  servings. Real balance question (access vs availability), needs a probe.
- punishment_warn = 0.29 WARN - negatives land without same-thread
  warning; matches the writ->knights pattern in reward-punishment review.
- valence_ratio = 7.43 WARN above 5 - classifier may over-credit ambient
  positives (letters, rainbows) vs weighted negatives; consider
  severity-weighted ratio (reward review, Next Steps).
- time_to_first_growth = 200 (never) WARN - expected under the idle
  driver; this KPI measures the passive-player experience until a real
  policy exists.
