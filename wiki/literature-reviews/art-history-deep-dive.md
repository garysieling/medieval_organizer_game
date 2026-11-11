# Art-history deep dive: Bosch and Dürer (#804)

What the named-artist pass gives the pipeline: not paintings, but
reusable *direction* — palette, motif, and the handful of forms that
read as "the period" at 48–128 px.

## Bosch (c. 1450–1516)

- **Palette**: chalky greens and rose-pinks over deep earth browns;
  flesh tones almost porcelain; accents of cinnabar red and slate blue.
  Reads well at pixel scale because his contrasts are hue-driven, not
  value-driven — the weird thing stays legible at 16 px.
- **Motifs worth taking**: hybrid marginalia (a bird with a man's
  ears, a fish with legs, a knife walking on feet), hollowed eggshells
  with someone inside, ladder-against-everything, fire on a horizon.
  These are *event-flavor* shapes: the omen, the wandering wonder, the
  dream a villager reports.
- **Composition**: crowds in rings facing inward — exactly our feast
  and meeting layouts; worth keeping for crowd scenes and the maypole.
- **Mechanic echo**: marginalia monsters as omen flavor (the ballad,
  the black storm already do this) — a "strange sight" road event
  could use the hybrid shapes without new mechanics.

## Dürer (1471–1528)

- **Palette**: monochrome-first — his engravings are line and stipple;
  where color exists it's muted earth with one saturated accent
  (the famous Nemesis green, the Hare's brown).
- **Motifs**: the *study* as a genre — a single hare, a single clump
  of turf, a rhinoceros nobody had seen. For the game: single-object
  frames work — a goods sheet item, a bestiary page, a memorial relief.
- **Mechanic echo**: a printmaker tier — a master artisan producing
  sellable prints (small reproducible goods, reputation-bearing):
  the Dürer economy of the same woodblock sold a hundred times. Filed
  as a follow-on; the vocation's `Printmaker` already exists.
- **Composition**: strong diagonals, dense hatching that reads as
  texture at low resolution — useful for engraved-looking shop signs
  and the notice board's parchment.

## Prompt additions (in `generate_art_prompts.py`, group `art-history`)

- `decor-memento-mori` — a skull on a book: memorial/estate flavor.
- `decor-marginalia-hybrid` — Bosch hybrid creature for omen flavor.
- `decor-painted-hall` — a wall-hanging/res fresco for painted halls.
- `goods-print-sheet` — a Dürer-tier woodcut print inventory item.
- `sign-engraved` — hatched monochrome shop sign look.
- `wonder-eggshell` — the hollow eggshell with someone inside.
