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

## Round 2 calibration (2026-10-31, seeds 42/7, 200-300d, dev build)

- villagers.hungry re-thresholded to the sim's own seek line (<30); the old
  <40 read the bottom of the meal cycle as famine (~0.68 peak when nobody
  starved in 300d). villagers.starving (<=10) added as the hard red line.
- hunger_peak replaced by hunger_pressure = MEAN share <30 (band 0.05-0.30
  per balance review). Peak-share flagged survivable winter waves.
- price_cv dropped: prices sit on a fixed vocation table by design, no
  market gauge exists. Replaced by econ_gold_spread (top/mean purse ratio;
  measured 4.5-6.6, band 0.5-8, flag >15).
- classify() now skips command feedback ("Can't X", "You can't", "No one
  is near") - blocked-button spam was inflating beat_density to ~19/10d.
- Harness driver upgraded from the idle fallback to a light organizing
  policy: eats/works via drive_player, talks fresh contacts for influence
  (+0.05 each to 0.4), recruits at bond >= 0.5, calls meetings when 2+
  ready, and never presses a greyed-out button (respects action_blocker).
  Result: first org member ~day 41-43 across seeds.
- punishment_warn 0.14-0.25 FAIL/WARN across seeds - a real finding, not
  instrumentation: most negative beats (flu, writs, wolves) land with no
  same-thread warning event in the prior 45d. Candidate for a warning
  mechanic, not a band change.
- is_beat now requires valence: neutral chatter (gossip, forewarnings,
  warnings-as-context) isn't a beat. Density 9.8 -> 6.6 beats/10d on
  seed 42, so the band moved 2-6 -> 2-8 to fit the livelier village
  (births, migrants, festivals, rare events landed after the band was
  set). Evidence: seed 42/400d healthy run at 6.58.

### shaping_slope recalibrated 0.5-3.0 -> 0.5-5.0
Gauge now weights negatives per-capita (1/villagers.alive that day) - a growing village makes more negatives honestly. Evidence: seed 13 raw 5.14 -> per-capita 4.42; seed 7 3.30 -> ~2.4. The remaining ramp is org escalation + winter + aging - the game is meant hard, so a 5x per-head ceiling is the honest line.
