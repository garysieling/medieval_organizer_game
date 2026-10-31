# The Oil-Boiler's Visit (Ink Making Event)

**Category:** Design / Events + World Systems
**Source:** Gary request — "a bunch of linseed oil in a cauldron and boil it as a precursor to ink making... dip bread in it... mix in something black... periodic event, outside the town gates, small chance of starting a fire"
**Research:** `wiki/literature-reviews/ink-oil-boiling.md`
**Date:** 2026-10-31

## Description

A travelling ink-maker walks the road in every year or so and pitches a
cauldron on the meadow **outside the gate** — because that's where
medieval law and common sense put a trade whose fire "would be glad to
get to" the oil. Over several days he boils linseed oil down to varnish,
fries bread crusts and onions in it to degrease, smothers the flames when
the vapours catch, drips it on slate until it's "sticky like thin glue,"
then grinds lampblack into it and sells `ink` before moving on. Most
visits are a spectacle and a supply drop; once in a long while the oil
takes and the meadow burns.

## Historical anchors (what the event teaches)

- The boil happens outdoors away from buildings — Encyclopedie: "choose a
  wide space such as a garden and not to be near a building"; Pistoia
  banished stench-crafts beyond the walls.
- Bread crusts + a dozen onions really went into the pot ("speed up the
  degreasing").
- Ignition was expected, not anomalous — vapours were let catch then
  smothered with wet linen; a red-hot poker was the lighter.
- The black is lampblack (soot), not ash — worth getting right.
- The product feeds the print shop: aqueous inks wouldn't wet metal
  type; oil ink was bought from specialist ink-makers.

## The event (phase machine)

Session field `ink_boiler: Option<InkBoiler>` + `next_ink_boiler: u64`,
on the Circus pattern (`spawn_`/`advance_`/`_tick`).

| Phase | Days | What happens |
|-------|------|--------------|
| Pitch | — | walks the `traveller_path`, sets up cauldron at a meadow spot `nearest_buildable(wall.0 + 6, wall.3 + 4)` |
| Boil | 2 | smoke over the meadow; per-day fire chance; nearby villagers get `Problem::Smelly` (acrolein) |
| Degrease | 1 | a villager brings bread crusts; they go in the pot; the bringer gets `Want::GoodFood`/`Joy` ("the crusts came back fried") |
| Test | 1 | the slate-drip reveal — "it's varnish now" |
| Grind | 1 | lampblack stirred in; `ink` stocked at the print shop (or his own pitch if no print shop) |
| Pack | — | he walks back down the road; `next_ink_boiler = tick + 250..450` |

Log lines in the established voice: "an ink-maker has come up the road -
he's to boil his oil on the meadow, where the law wants it."

## Fire spread (the real reason this is two systems)

New module `game2-core/src/game/fire.rs` — a small CA over the real
`TileMap` (rules proven in `sim/emergence.rs`):

- `Fire { burning: HashMap<(i32,i32), u8>, scorched: HashMap<(i32,i32), u64> }`
- Flammable: `Grass`, `Wheat`, `Corn`, `Hill` tiles that hold a `trees`
  entry, and `occupied` building cells (worst case, rare)
- Firebreaks: `Water`, `River`, `Road`, `Bridge`, `Rock`
- Per tick: each burning cell rolls per-neighbour ignition (e.g. 25%),
  decrements its burn clock, then becomes scorched; scorched tiles
  recover to grass after N days
- Ignition source: the cauldron — on a flare, the boiler's tile ignites
  (the pot "takes fire, as it will"); he smothers it with wet linen and
  it's usually just a scorched patch and a story. Sometimes it isn't.
- A fire that reaches a building consumes stock; villagers adjacent to
  burning tiles flee (existing `Goal` machinery) and pick up a grievance.

## Economic beats

- `ink` becomes locally sourced during/after a visit (stocked at the
  print shop) — first domestic source of an `imported` good; the trader
  stays the fallback.
- Cheap ink days: the printer's `NoInput("ink")` grievance clears.
- Stretch (later PR): leftover `varnish` feeds `Vocation::Painter` (oil
  painting); `lampblack` as an intermediate good if the boiler settles.

## Chunking (tiny PRs)

1. Research + this doc (docs only)
2. `game/fire.rs` — fire CA + unit tests, not wired to anything
3. `InkBoiler` event skeleton — spawn/walk/pitch/leave, no fire, no yield
4. The recipe phases — boil days, bread beat, slate test, ink yield
5. Fire integration — flare roll, ignition into `Fire`, fire_tick wiring
6. Villager reactions — smelly grievance, fried-crusts beat, flee/complain

## Related

- `writing-materials-bookmaking.md` (scribal inks), `sprint-4-printmaking.md`
  (the press), `sim/emergence.rs` (the CA proof), circus event in
  `session.rs` (the outside-the-walls template)

## Rating

A remembered-history event that's mechanically load-bearing: first local
`ink` source, first real fire on the map, and a story generator with a
fat tail.
