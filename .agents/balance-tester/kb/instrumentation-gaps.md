# Instrumentation Gaps - Balance Tester KB

Signals the KPI table wants that the harness can't yet see.

## Known gaps (2026-10-31)

1. **Typed events** - the log is strings (`t{n}: ...`); classify() is a
   keyword read of the village's own phrasing. The reward and balance
   reviews want valence/severity/cause fields. Severity-weighted
   valence_ratio, starvation-vs-illness mortality, and
   consequence_visibility all wait on this.
2. **Price discovery** - `price.grain|wood|fish` read `vocation::price()`
   which is static; per-entity markets (economics.rs, spawn_market) move
   but aren't gauged. price_cv flags every run until a market gauge lands.
3. **Player actions** - the idle driver talks/works/invests; it never
   recruits, so org-growth KPIs measure the passive experience only. A
   real policy (or the persona matrix from balance-methodology O3) is the
   unlock for contingency_lift / action_yield / skill gradient KPIs.
4. **Warning links** - punishment_warn infers warning by same-thread
   prior events; a real warning->consequence link (writ id -> crackdown)
   would be sharper.
5. **Arc shapes** - resolution_rate is a proxy (neg beat -> pos beat
   within 30d). The arcs review proposes thread-grouped phase-state
   machines (setup->escalation->climax->resolution, Freytag +
   kishotenketsu) - richer but unbuilt.
