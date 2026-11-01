# Goods & the Market Economy

**Status:** Design accepted-in-principle (Gary, 2026-11-01) — bones first, tuning later.
**Sources:** `sprint-3-medieval-economics.md`, `game-design-fun-supply-chain.md`,
`medieval-crafts-tools.md`, `craft-incense-production.md`, `sprint-4-textiles.md`,
`sprint-7-fermentation-trades.md`, `agriculture-food-systems.md`, plus fresh survey
of medieval village-market literature (Masschaele; Kalmar museum; manorial market
records) and Against the Storm devlogs/wiki.

## Principles (Gary's)

1. **Barter-first.** Goods trade for goods; coin is the ledger underneath, not the
   vocabulary of the deal.
2. **Intrinsic value, hidden price.** Every good has a worth (a gold-equivalent
   the sim uses to balance trades) but nobody *quotes* it. You hear "two roots for
   three eggs" — never "eggs are 1.7 gold." Fairness is felt, not read.
3. **Three tiers.** Staples everyone needs; manufactured goods; luxuries.
4. **A production web, not a chain** (Against the Storm): ingredient *slots*, not
   shopping lists — a baker works on any grain, so one missing good never stops a
   trade cold.
5. **Muted variation.** Events, experience and tech nudge output and worth, but
   this is not a supply-chain game — the web exists so the *social* system has
   delayed, second-order problems to solve.
6. **Cottage industries distribute their own goods.** The alewife pours at her
   bench, the farm wife sells eggs at her door and at market. Vendor-buyer contact
   is the relationship engine (suppliers and customers meet regularly).
7. **Skip steps freely.** Wool → garments is fine; we don't model yarn.

## What's already in the walls

- `vocation.rs` — fixed `price()` table, per-vocation `inputs()`/`output()`,
  `imported()`, `gatherable()`. ~35 named goods live.
- `livelihood.rs` — `Purse`, `Satchel`, `trade_person` (person-to-person, priced),
  `buy_at_shop`, `seller_for` (finds *any* holder — satchels included: cottage
  door-sales already work mechanically), `shop_for`.
- `household.rs` — `Building.stock`, `on_offer` (stock + price), stalls.
- `recipes.rs` — AtS-style `input_options` already modeled; used only for
  printmaking so far.
- `sim/economics.rs` — a `Market` with bounded scarcity drift (0.2–5.0× base)
  that is **not wired into the session** — demo bins only. Reusable math.
- Cottage production exists: hens → eggs, garden beds → marigold/feverfew,
  fold → milk/wool, apiary → honey/wax, cellar → cheese, smoke-house → fish,
  still → perfume/spirits, pedlar → ribbon/needle/lute_string.
- `Want`s gate demand (GoodFood→bread, Finery→garments, Drink→drink); `Problem`s
  and grievances carry the social consequences a trade creates.

## The big brainstorm (what a village could trade)

Raw list before filtering. Brackets = already in game.

### Tier 1 — staples (everyone needs some of this every week)

- **Field:** [wheat], rye, barley, oats, peas/beans, flax, hemp, hay, straw,
  roots (turnip/parsnip/carrot), potherbs (onion/leek/cabbage), apples,
  berries, nuts, [mushrooms/chanterelle]
- **Yard & byre:** [milk], [eggs], [butter], [cheese], hides, tallow, dung,
  horn, feathers/quills, [wool], pelts (coney/fox), [honey], [wax]
- **Water & wild:** [fish], eels, rushes, reeds, [driftwood], shellfish,
  samphire, [marigold], [lavender], [feverfew]
- **Fuel & light-makings:** faggots (firewood bundles), turf/peat, dung-cakes,
  tallow, rushes, charcoal
- **Earth:** fieldstone, [sand], clay, lime, salt
- **Fiber:** [wool], flax, hemp, tow, nettle-fibre

### Tier 2 — manufactured (somebody's craft turns staples into these)

- **Kitchen & cellar:** [bread], pottage, [ale], [beer], cider, mead, [wine],
  [vinegar], sausages, bacon, smoked fish, flour, malt
- **Cloth & clothes:** [cloth], linen, yarn, [garments], hose, felt, hats,
  leather, shoes/clogs/pattens, belts, gloves, lace
- **Light:** rushlights, tallow candles, wax tapers, fish-oil lamp oil,
  horn lanterns
- **Household:** treen (bowls/spoons/trenchers), crocks/jugs, baskets, sieves,
  besoms, sacks, rope, nets, eel-pots, soap, blankets, rush mats
- **Build & craft inputs:** timber, fieldstone, [blocks], wattle, daub, thatch,
  lime, bricks, charcoal, pitch, glue, resin, nails, [tools], [casks], querns
- **Print & word:** [paper], vellum, [ink], [pamphlets], playing cards,
  ballad sheets
- **Colour:** [pigment], [paint], woad, madder
- **Services:** [security], [faith], remedies, a barber's razor, mending

### Tier 3 — luxuries (elastic, status, want-gated)

- **Scent & smoke:** [perfume], rosewater, pomanders, incense (imported resin +
  garden aromatics), scented soap
- **Sweet & spice:** pepper, saffron, cinnamon, ginger, sugar, marchpane,
  hippocras (spiced wine), figs/raisins, almonds
- **Finery:** silk, velvet, [ribbon], lace collars, fur trim (fox pelt),
  fine gloves, brooches/rings
- **Sight & art:** [glass], mirrors, spectacles, [painted halls], illuminations,
  psalters
- **Joy:** [lute], [lute strings], playing cards, dice, whirligigs, songbooks
- **Table:** fine wine, [spirits], aged cheese (the year-old wheel), marchpane,
  wafers
- **Faith-adjacent:** wax tapers, pilgrim badges, rosaries
- **Found:** amber & jet (strand finds — driftwood's prettier cousins), pearls

### Trope check (things in every RTS that stay out or get renamed)

- ~~wood~~ → **timber** (craft-wood) + **faggots** (fuel-wood): two goods, both
  named for what they *are*, and splitting them creates the fuel question a
  winter needs.
- ~~stone~~ → **fieldstone** (what you pick off the ground) vs the mason's
  **blocks** (already in game — the dressed product).
- ~~iron, ore, crystal, meat, vegetables, food, materials~~ — none enter.
- [sand] stays: it's specific (the glazier's shore sand). [fish] stays: the
  smoke-house and eel-pots make it a trade, not a resource icon.

## The filtered working set

Keeping every good already in game (~35) plus **~30 new goods** — the smallest
set that makes the three tiers legible and the webs real. New goods are starred.

| Tier | Goods |
|---|---|
| 1 staples | wheat, rye*, barley*, peas*, roots*, potherbs*, apples*, flax*, hay*, straw*, hides*, tallow*, rushes*, faggots*, turf*, dung*, salt*, milk, eggs, butter, cheese, fish, honey, wax, wool, mushrooms, marigold, lavender, feverfew, timber(was wood), fieldstone(was stone), sand, driftwood |
| 2 crafted | bread, pottage*, linen*, leather*, soap*, baskets*, treen*, crocks*, rope*, rushlights*, candles*, tapers*, charcoal*, cider*, mead*, sausages*, vellum*, playing-cards*, garments, cloth, tools, blocks, glass, paint, pamphlets, paper, ink, casks, lute, ale, beer, wine, vinegar, pigment, remedies |
| 3 luxuries | incense*, spice*, silk*, marchpane*, hippocras*, rosewater*, pomander*, spectacles*, mirrors*, fox-pelts*, amber*, whirligigs*, pilgrim-badges*, perfume, spirits, ribbon, lute-strings, aged-cheese |

Rationale for the cut: each new good must (a) be produced by an actor that
exists or is trivial (a household, not a new vocation), (b) fill a tier that
was thin, or (c) complete a web (rye/barley for the grain slot; flax for the
fiber slot; turf/charcoal for the fuel slot). Pigs, tanning-vats, full
metalworking, mills, salt-springs are documented above but not in the first wave.

## Mechanics

### M1. Good registry (`game/goods.rs`)

A spec table: name → `{ tier, base_worth, tags }`. Tags are the slots:
`grain`, `fibre`, `fuel`, `light`, `drink`, `staple_food`, `finery`, `incense`,
`sweet`, `perishable`. `vocation::price` remains as the base worth source so
existing callers keep working; the registry overrides where set.

### M2. Hidden worths (`Worths` component)

Each good's current worth = base_worth × scarcity multiplier, drifting like
`sim/economics.rs::Market` but bounded **0.6–1.8** (muted). Multipliers move on
stocked-vs-spent pressure and on events: failed harvest → grains dear, wolf
winter → wool dear, storm → fish and strand goods swing, feast days → luxuries
dear. Delayed, damped — second-order fodder for ISSUES, not a trading sim.

### M3. Barter (`barter_person`, and shop offers in barter terms)

`barter(buyer, seller, offer, want)` values both bundles at hidden worths.
Lopsided deals settle with gold "to boot" — a penny or two evens it, exactly
like period market practice. Wildly lopsided deals get refused or — better —
accepted at a grievance: *she saw what I did.* The log speaks goods, never
figures: "2 roots for 3 eggs, and a penny to boot." A fair swap warms the bond
a little; a skinned one leaves a mark.

### M4. Recipe web (extend `recipes.rs` to vocations)

`inputs(v)` gains slot alternatives — same `input_options` shape, resolved
against the registry's tags:

- Baker: `[grain]` → bread (wheat | rye | barley)
- Alewife: `[grain]` → ale | apples → cider | honey → mead
- Brewer: `[grain]` + hops → beer
- Weaver: wool → cloth | flax → linen
- Printmaker: (paper | vellum) + ink → pamphlets
- Toolmaker/Cooper/Painter/Luthier: timber (as now)
- Mason: fieldstone → blocks (as now)
- Perfumer: marigold|lavender + spirits → perfume | roses → rosewater
- Vintner: grapes → wine | wine + spice → hippocras
- Households (cottage): peas+roots+potherbs → pottage; milk → butter;
  rushes+tallow → rushlights; tallow+ash → soap; timber → treen; straw → baskets

`needs_input` asks the slot question ("any grain in hand?"), not the good question.

### M5. Cottage distribution

Households already produce (hens, garden). Add one **cottage craft** per
household — a specialty drawn from their make-up (a spinner's house makes
linen-lean; a shore house gathers rushes and boils tallow). Sold from the door
(`seller_for` already finds satchels) and at market stalls. This is the
relationship engine: the same faces trade with each other every week.

### M6. Market day

Feast days already gather the village at the market. Add the ordinary weekly-ish
market: anyone holding ≥3 of a saleable good pitches a basket; buyers prefer
stalls; standing at the market is a gossip opportunity (bonds form across the
egg basket). The pedlar remains the import channel.

### M7. What's never shown

No price labels anywhere a player reads: offers show goods and their barter
terms, words like "dear," "going cheap," "a fair ask." Gold stays on purses,
taxes, the reckoning — the layer the village *doesn't* talk in.

## Implementation backlog (preparatory steps first)

Each lands as its own PR from a worktree.

1. **Design doc + backlog commit** (this).
2. **`game/goods.rs` registry** — Tier enum, spec table, tags, `worth()` shim
   over `vocation::price`. Pure addition + tests.
3. **`Worths` scarcity drift** — session component; `trade_person`,
   `buy_at_shop`, `on_offer`, commission pricing read through it. Default
   multipliers 1.0 → behaviour identical until events move them.
4. **Barter** — `barter_person`, to-boot gold, barter-terms log strings,
   fair/skinny deal → bond/grievance hooks.
5. **Garden staples** — roots, potherbs, peas as goods; garden beds yield them;
   pottage recipe on households.
6. **Fuel slot** — faggots, turf, dung, charcoal; gatherer/collier yields;
   winter hearth need hooks into hard-winter.
7. **Light goods** — rushes→rushlights, tallow→candles, wax→tapers; church-dark
   mechanic already exists to consume tapers.
8. **Grain web** — rye, barley fields; baker/alewife/brewer slot inputs.
9. **Fiber web** — flax field, weaver linen option, tailor accepts cloth|linen.
10. **Drink web** — apples→cider, honey→mead on the alewife; barley beer.
11. **Cottage crafts** — baskets, besoms/treen, soap, rushlights by household
    specialty; door-sale already works.
12. **Market day stalls** — weekly market at the square, basket pitches, gossip.
13. **Hides & leather** — butcher/fold yields hides; cobbler absent → leather +
    shoes sell via cottages/pedlar until a vocation exists.
14. **Import luxuries** — spice, silk, incense resin, marchpane on pedlar/trader.
15. **Luxury recipes** — hippocras (vintner), simnel cake (baker, feast),
    rosewater/pomander (perfumer).
16. **Sight & glass luxuries** — spectacles (ties to failing-sight difference),
    mirrors on the glazier.
17. **Hidden-price UI pass** — barter terms + dear/cheap words in every place a
    price string prints.
18. **De-trope renames** — wood→timber/faggots, stone→fieldstone (mechanical,
    touches every producer — deliberately late).
19. **Strand finds** — amber/jet alongside driftwood after storms.
20. **Economy review** — economist-tester + balance-test long-run pass;
    PLAY.md and project-state updates.

## Open questions for Gary

- Should **salt** be a road import (leverage for the trader) or a cottage
  salt-pan on the reach? Currently pencilled as import — it gives the pedlar
  a staple, not just frippery.
- **Meat**: no butcher vocation exists; sausages are drafted as an autumn
  household preserve using fish/fold culls. A pig in the yard is the natural
  next livestock — flag or skip?
- Worths drift bounded 0.6–1.8 — is that "muted" enough, or should event shocks
  read stronger (say 0.4–2.5) so a failed harvest is *felt*?
