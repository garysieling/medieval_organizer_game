# Fabric/Textiles Tech Tree

**Category:** Tech Tree / Economy
**Source:** Backlog item — "Affects trade AND texture sensitivity (autism representation). Research historical textile production, materials, economic impact." Implemented stub: `game2-core/src/game/textiles.rs`.

## Description

Textiles as a deep production tree and a *sensory* system at once: economically, cloth was the second great medieval trade good after food; experientially, fabric texture is a real axis of comfort for many autistic people — making "what cloth touches your skin" a character-level mechanic is a quiet piece of representation the genre never touches.

## Historical skeleton (the tree)

```
fiber → yarn → cloth → dye/finish → garment
```

| Stage | Real progression | Game note |
|---|---|---|
| **Fiber** | flax (linen), wool, hemp, nettle; cotton/silk via trade | regional availability drives the whole tree — flax regions vs wool regions produce genuinely different economies |
| **Processing** | retting/breaking/scutching (flax); shearing/carding (wool) | labor-heavy bottleneck — mechanization (fulling mill, spinning wheel, later spinning jenny) is the *real* tech |
| **Spinning** | drop spindle → spinning wheel (medieval) → jenny (early modern) | each step is a throughput multiplier |
| **Weaving** | warp-weighted loom → horizontal treadle loom | loom tech gates cloth width/quality |
| **Dyeing/finishing** | woad/madder/weld/lichen; fulling; shearing | dyes are the value-add — regional dye goods create trade specialization (madder regions export red) |
| **Garment** | homespun → tailored → decorated | quality tiers feed status/fashion |

## Economic realism (what the tree buys us)

- **The classic multiplier industry:** textiles were *the* proto-industrial sector — a tree with real labor bottlenecks (processing >> weaving) means mechanization tech actually feels like an industrial revolution in miniature.
- **Trade specialization:** raw fiber is heavy; dyed finished cloth is light and valuable → cloth regions import fiber, export value. The market sim (`sim/economics.rs`) sees emergent specialization.
- **Substitution coupling:** dyes and fibers feed the flexible recipe system — "red dye" is a role madder, kermes, or brazilwood can all fill (see `flexible-recipe-system.md`).

## The texture-sensitivity mechanic (autism representation)

Done respectfully as *simulation*, not a trait checkbox:

- Characters can have **tactile sensitivity** (a trait on the character component, weighted toward some autistic characters — and some non-autistic ones).
- Garment comfort = f(fiber, weave quality, finish). Coarse homespun wool vs soft finished linen registers differently on the comfort dimension — it affects mood/stress the same way hunger or cold does (ties to `qsp-theory-of-mind.md` compartments).
- This is *normalizing representation*: texture sensitivity is modeled like warmth/hunger — a real sensory need some characters have more strongly, not a "disability" flag.
- Emergent story: the sensitive child who thrives only when the family can afford soft linen — class, economy, and sensory needs intersecting. That's the kind of emergent narrative this game is for.

## Implementation notes

- `game/textiles.rs` holds the tree; wire dyes/fibers into `game/recipes.rs` tag-substitution and `game/tech_tree.rs` nodes (mechanization steps).
- Comfort contribution feeds the mood/stress compartments.

## Rating

A tree that's economically deep *and* does quiet representation work — rare combination worth keeping.
