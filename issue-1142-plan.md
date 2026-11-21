## Diagnosis — reading (a), with the gauge already half-healed

Reproduced on current main (seed 42, 400d): `food_cover_days` now reads **4.24** (idle) / **4.28** (organizer) — the reported 0.41 no longer reproduces, because the market now gets refilled (surplus-flow carts). The *inconsistency* is upstream-fixed, but the gauge's hole is still there:

**The stock census is one pile.** `stock.food` reads only `market_food`'s `FoodSource.servings`. Villagers eat from "the village table" — `Goal::Eat` prefers `market_food + crop_fields`, nearest stocked — and only a bare table sends them hunting the next pile over (rivals' granary, site granaries, capped at 30 legs). On the idle probe at day 400 the world holds ~4,800 servings across 16 pools; the gauge sees 412.

**`hunger_pressure = 0.01` is true, not a gauge lie.** `HungerSystem` runs once per day (scheduler ticks on the day boundary) at -1/day; a meal resets to 100. So a villager eats ~once per 70 days and spends ~1-3 days below the seek line — a mechanically ~0.01-0.04 share while supply works. It can only reach the 0.05-0.30 band when food actually runs out and souls sit below 30 unfed. The WARN correctly reports "a food system that never bites" — demand is ~0.44 meals/day village-wide. That's a gameplay tuning signal, not instrumentation.

## Plan (instrumentation, not gameplay — per the issue's option a)

1. `secret.rs`: `stock.food` = the village table — `market_food + Σ crop_fields` (matching `Goal::Eat`'s own first-preference set, per `travel.rs`'s "the village table while it's laid"). *Not* all pools: counting unreachable far granaries would lie the other way — a village starving beside a full store it can't walk to would read as covered.
2. Update `m_food_cover`/`m_stockout_share` docs to name the village-table scope.
3. Regression test: market bare + a stocked crop field → `stock.food` counts the field (old code → 0); market+fields bare with full site granaries → `stock.food` = 0 (pins the scope: other settlements' larders are fallback, not cover).
4. Re-run `balance_test 400 42` across personas; expect cover ~5-6 and stockout still near 0, with `hunger_pressure` honestly WARN — filed as the residual finding (the 70-day meal cycle is why food never bites).
