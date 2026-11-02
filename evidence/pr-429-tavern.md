# PR #429 — The tavern on the green

## What was there already
The village had drink *production* — `Alewife`/`Brewer`/`Vintner`/`Distiller`,
one rolled off the seed — plus `Want::Drink`, `Drunk`, the assize's
`DRINK_EXCISE`, the drink levy, and souring in `drink_tick`. What it had no
word for anywhere in the codebase was a **tavern** or a **taverner**: drink
was poured from the producer's own bench, and the only "taproom" was a test
helper. No `Vocation::Taverner`, no public house, no keeper.

## What changed
- `Vocation::Taverner` — brews nothing, buys everything. `inputs: &[]`,
  `output: "lodging"` (6 gold), `trade_name: "the tavern"`, three actions.
  Its shop is `Shop(Taverner)`, so no new `BuildingKind` was needed.
- Spawn: a taverner is forced into the roster (`swap8`) and his house into
  `must`, so every village keeps a tavern. It opens with 4 ale and 2 beds.
- `tavern_tick` → `tavern_restock` + `tavern_bench`, hooked after `drink_tick`.
- **`tavern_restock`** (even days): cheapest-first, one barrel a trip, the
  keeper buys off whichever trade makes the good *or the road's stall* for
  imported wine and spirits — which is historically what set a tavern apart
  from an alewife's bench. He pays `TAVERN_WHOLESALE` (0.7) of street price
  and keeps `TAVERN_RESERVE` back; that margin is his whole living. Baring a
  shelf nudges the good dear, same as any sale.
- **`tavern_bench`**: everyone within 4 tiles of a *wet* tavern (bench seats 6)
  buys a cup off the bar — the keeper's till — cheapest they can afford, drunk
  there and not carried home, with the counter's same 15% roll for `Drunk`.
  Then every pair at the bench records a `Conversation`, the keeper records one
  with each of them, and all of them get `Want::Joy`.
- `Want::Shelter` gets its first answer in the game: the roofless buy `lodging`
  at the tavern (gated on `home_of(v).is_none()`, and consumed not carried).
- `brawl_moment` (the existing `AlehouseBrawl` event) now looks for the tavern
  first — it's the village's taproom now.

## Deliberately not built
An emergent "grudge boils over at the bench" brawl. Measured across 8 seeds ×
400 days: of ~700 bonds per village the **minimum** strength is −0.08, and
`RelKind::Rival` needs < −0.3. Nothing in the sim sours a bond far enough, so
a second brawl mechanism keyed on bond sourness could never fire. The existing
`AlehouseBrawl` event fires ~1/400 days and now lands at the tavern, fed by the
drunks the bench makes.

## Measured (8 seeds × 400 days), before → after
| | lay-ins | keeper's purse |
|---|---|---|
| alewife villages | 4 | ~0 (deadlocked) | 
| after | 32–45 | solvent |
| brewer/vintner villages | 4 | ~0 |
| after | 8–31 | 7–146 gold |

Two cold-start deadlocks had to be broken for the trade to turn at all: the
keeper's `TAVERN_FLOAT` (his is the only trade that pays for stock before it
sells any) and the bench actually buying cups (his only income).

## Quality gates
- `cargo test --workspace`: 710/710 (10 new `tavern_tests`)
- `cargo check --workspace --all-targets`: no warnings

## Pre-existing bugs found, not fixed here
1. **The drink trades are capital-deadlocked.** A `Brewer` spawns with
   `NoInput("hops")` and `purse = 0.0`: the stall carries 12 hops at 4 gold, he
   can't afford them, so he never makes beer, so he never earns, so he stays
   broke — for all 600 days. Same shape for `Vintner` (grapes). Only the
   `Alewife` (wheat, grown locally) actually produces. Every existing drink test
   injects stock by hand, which is why this was never visible. The tavern works
   around it by buying off the road's stall; the underlying trade still needs
   working capital or credit.
2. **Two tests asserted things their code never guaranteed** and broke purely
   because adding a vocation shifts the seed stream. Both were tightened rather
   than re-tuned: `test_the_wolfs_in_the_fold` assumed `rng.below(3)` never
   rolls 0 (it now checks the logged tally equals the fold's shortfall), and
   `test_the_hens_day_pays` summed a whole household's gold across a simulated
   day to detect a 1-gold egg sale (it now measures the hand the eggler
   actually pays).

## Deferred
- A `taverner` dialogue tree is in (`taverner_bench`, 57 lines), but the live
  session doesn't yet feed trade statuses to the conversation system — the trade
  trees only fire in `dialogue_gen`/`conversation_demo`.
- Casks at the tavern (they already double wine's shelf life in `drink_tick`).
- Lodging for the pedlar, trader and circus folk, who have nowhere to sleep.
