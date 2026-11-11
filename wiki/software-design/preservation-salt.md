# Preservation & the Salt Economy

**Status:** Proposal (per #951's grill decisions — directions (c) + (d) combined)
**Sources:** `wiki/literature-reviews/food-preservation.md` (the deep dive),
`goods-market-economy.md`, `agriculture-food-systems.md`
**Implements:** nothing yet — a follow-on ticket is filed from this doc.

## Principles

1. **Salt is the choke point.** Every preservation chain worth modeling runs
   on one imported input. Make that input real and scarcity does the rest.
2. **Preservation is a season, not a recipe.** One calendar day — the
   Martinmas slaughter — converts the year's surplus into winter stores, as
   communal work with a feast attached.
3. **Bones first, tuning later.** Reuse `spoilage_tick`, `festival_tick`,
   `goods.rs`, and the pedlar/caravan supply lines. No new trade machinery.

## What's already in the walls

- `economy.rs::spoilage_tick` — weekly rot (bread/milk/eggs weekly, wheat
  monthly, granary wheat extra to vermin), `Problem::SourGut` for the
  desperate. **It ignores the `Perishable` tag** — a hardcoded `week_of`,
  so tagged `fish`/`ale`/`butter`/`pottage`/`mushrooms` never rot.
- `economy.rs::cheese_tick` / `drink_tick` — working precedent for
  convert-perishable-into-keeper (milk→cheese, ale→vinegar, cellar wine).
- `economy.rs::fisher_tick` — the fisher's workplace is literally named
  "the fish-smoke house" (`vocation.rs:357`) but stocks raw `fish`.
- `goods.rs` — `salt` exists as a dead good: worth 3.0, `Staple` tag, no
  producer, no consumer. `sausages` is the only non-perishable prepared
  food. `Perishable` tag exists on bread/milk/eggs/fish/mushrooms/ale/
  butter/pottage.
- `society.rs::festival_tick` — the feast machinery: fixed calendar days
  (May Day 81, Midsummer 172, Harvest Home 263, Yule 355), stock pulled
  from building stores, village gathers at the market, visitors drawn.
  **Day ~314 is empty** — exactly where Martinmas historically sat.
- `country.rs` caravans and the pedlar already import goods; `Purse`,
  `Satchel`, `trade_person`, `seller_for` price and move any named good.
- `routine.rs::local_celebration` / `assemble` — a ring of neighbours
  with a log line and want-satisfaction, reusable for the bee's social beat.

## Proposal

### 1. The salt economy

`salt` becomes a real staple that gates preservation:

- **Supply:** the pedlar and the town caravan carry it (it is imported —
  historically coastal or mined, never village-made). Possibly a rare
  `Goal::Cart` run too. Small unit size, meaningful price — 3.0 base is
  plausible; scarcity-drift via the existing market math if wired later.
- **Demand:** the salting-bee consumes it in bulk (below); household
  salting-tubs smoke it in small amounts year-round if the smokehouse
  vocation is added later.
- **The grievance lever (optional, cheap):** the lord may claim a salt
  *devoir* — a small weekly levy of salt or coin — that reads exactly like
  the existing `edict`/`levy` mechanics. Literature says this is *the*
  historical grievance engine; the org can campaign on it like any carried
  demand. Keep it off by default until the bee lands.

### 2. The Martinmas salting-bee

A fifth feast in `festival_tick`, day **314** (Nov 11; `tick % 365 == 314`):

- **The work:** every household's killable stock and the fishers' catch is
  pulled once: beasts/meat and `fish` plus `salt` convert in place —
  `meat + salt → salt-pork`, `fish + salt → smoked-fish`, both non-perishable
  goods added to `goods.rs` (Staple, worth ~4.0/~5.0, never rot).
- **The arithmetic:** 1 salt + 2 perishable → 2 preserved (roughly the
  historical 20 lb meat : 1 lb salt, scaled to unit sizes). Households
  short on salt simply preserve less — partial conversion, no cliff.
- **The feast:** what wouldn't keep is eaten the same day — the bee
  gathers at the market like Harvest Home, consumes a little
  bread/ale, satisfies `Want::GoodFood`/`Want::Music`, and warms
  relationships via the `assemble` ring. `player_event` kind 2 already
  puts the gathering on the camera.
- **The organizing beat:** the bee is a dense-contact day like a meeting —
  a named place where grievances air. The org holding salt before
  Martinmas is a power position; a village that can't cover the salt bill
  gets a shared grievance, not twelve private ones.

### 3. Wire the `Perishable` tag

`spoilage_tick`'s `week_of` moves to the tag: `Perishable` goods rot on
their weekly schedule (keeps today's cadence for bread/milk/eggs; `fish`,
`ale`, `butter`, `pottage`, `mushrooms` start actually perishing).
Non-tagged goods never rot; `salt-pork`/`smoked-fish` get no tag. This is
the honest implementation of what's already documented — and it makes the
bee *matter*, because fish and meat now visibly go bad.

## Explicitly out of scope

- No new vocation (a `Charcutier`/salter) and no new building (a dedicated
  smokehouse) in the first pass — the fisher's existing smoke house does
  the smoking. A later pass could add a salting shed; each costs an RD
  art-pipeline slot per `assets/rd/README.md`.
- No temperature/storage-quality model — cellar effects stay as they are.
- No ergot/poisoning event — noted in the lit review as a later event-card
  candidate (`St Anthony's Fire` is a strong `events.rs` entry).

## Alternates considered (from the grill pass)

- **(a) Processing chain** (butcher→salting tub→smokehouse as buildings):
  deeper but adds a building + vocation + art before the social payoff.
  Rejected as the first step; a natural follow-on.
- **(b) Storage-quality model** (cellars, vermin, temperature): real, but
  it's tuning on a mechanic nobody can act on yet. After the bee, worth
  revisiting if winter feels too flat.

## Test shape

- `cargo test -p game2-core --lib` — session tests in
  `game2-core/src/session/tests/` (a new `preservation.rs` or additions to
  an economy-adjacent file): the bee fires on day 314, converts
  `fish+salt → smoked-fish`, partial conversion when salt runs short,
  `Perishable` fish rots while `smoked-fish` doesn't, the gathering
  reaches `player_event` (kind 2), and a salt-short village takes a shared
  grievance.
- `game_ui --dump-frame` on a Martinmas tick to see the market gathering.
