# Flexible Recipe System (Against the Storm Style)

**Category:** System Design
**Source:** Backlog item — "make one thing many ways, forcing creativity. Multiple input combinations for same output, resource substitution, improvisation." Implemented stub: `game2-core/src/game/recipes.rs`.

## Description

Recipes as *requirements on classes of inputs*, not fixed ingredient lists. Against the Storm's core insight: a recipe defines *what a production step needs in function* (food fuel, binding material, water), and many goods satisfy each function — "food" accepts grain OR mushrooms OR fish, each with different supply chains. Scarcity forces substitution; substitution is where the interesting decisions live.

## Model

```
Recipe:
  output: { good, qty }
  inputs: [ { role, accepts: [tag|good...], qty } ]
  facility: tag            # e.g. "hearth", "loom", "still"
  labor: { role, turns }
  quality?: fn(inputs)     # substituted inputs may shift quality/efficiency
```

- Goods carry **tags** (`grain`, `fuel`, `mushroom`, `preserved_food`, `dye_fixative`...). A recipe input binds to a *role* satisfied by any good with the right tag — that's the substitution surface.
- **Efficiency deltas:** substitutes aren't free — a recipe that runs on coarse fuel takes longer or yields less than on charcoal. Players substitute *knowingly trading* throughput/quality for availability.
- **Improvisation tier:** when no input satisfies a role, a recipe may allow a *forced substitute* at a penalty (or not — some roles are strict). This produces the "make do" moment RTS/base games love.

## Why this fits the medieval sim

- Medieval production was substitution-heavy — flour from whatever grain grew locally, dyes from whatever plants existed, fuel from wood/peat/charcoal. The recipe system *is* the regional-difference mechanism: same "bread" recipe, different grain tags per region → regional cuisine emerges for free.
- Couples to the textiles/tea/herbology trees (all substitution-heavy crafts): a dye recipe accepting `madder|weld|lichen` makes textiles depend on *what your region grows*, not what a fixed recipe demands.
- Couples to economy: substitution changes input *demand* — the market reprices when players/NPCs swap inputs (Machinations resource flows get interesting here — see `machinations-resource-flow.md`).

## Mechanics to extract

- **Recipe discovery vs known recipes:** some substitutes are known, others discovered (experimentation, trade secrets — couples to guilds and contracts).
- **Quality propagation:** substituted inputs shift output quality → downstream goods inherit it (cheap dye → fading cloth → lower textile value).
- **Batch/bookkeeping minimalism:** recipes must stay legible to the playtester agents — role+tags, not open-ended formulas.

## Implementation notes

- `game/recipes.rs` holds recipes; extend with tag-based input matching + a `quality`/`efficiency` modifier function evaluated at craft time.
- Evidence for testing: a sim run where the only available fuel tag is `peat` — does the baker still produce? At what efficiency? That's a one-line test setup under the component-testing framework.

## Rating

One of the highest-fun-per-complexity systems in the genre — substitution turns every scarcity into a decision.
