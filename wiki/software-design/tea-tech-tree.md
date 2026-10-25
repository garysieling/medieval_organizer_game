# Tea History Tech Tree

**Category:** Tech Tree / Economy / Culture
**Source:** Backlog item — "Affects trade AND psychometric effects (cultural rituals, social bonding). Research tea cultivation, processing, trade routes, cultural significance." Implemented stub: `game2-core/src/game/tea.rs`.

## Description

Tea as both a *trade good* (the original global addiction commodity) and a *social technology* (the ritual that lubricates relationships). The tree's two outputs — economic value and social-bonding effects — make it unusual: it's a good that produces social capital.

## Historical skeleton

```
wild leaf → cultivation → processing (green→oxidized spectrum) → trade routes → ritual culture
```

- **Cultivation:** shade-grown vs open field; terroir (altitude, mist, soil) determines quality tier — maps to terrain/biome realism (`game/terrain.rs`).
- **Processing spectrum:** green (steamed/pan-fired, unoxidized) → oolong (partial) → black (full oxidation) → fermented (pu-erh). Oxidation level = shelf-stability vs delicacy tradeoff — *black tea travels, green tea doesn't*. This is the key trade mechanic: distant trade favors oxidized/processed forms, local culture keeps the delicate stuff.
- **Trade routes:** tea was *the* caravan good — brick tea as currency, Tea Horse Road, clipper races. Tea's value density + shelf life made it a money-good in exactly the way the economy sim can feel.
- **Ritual culture:** chanoyu, gongfu cha, afternoon tea, moroccan mint — preparation ritual is the social core, not the leaf itself.

## The psychometric mechanics (the interesting half)

- **Tea gathering as social tech:** shared tea preparation is a ritual frame — a bounded interaction where defenses lower. Mechanically: tea events produce outsized warmth/trust gains per unit time vs ordinary co-presence (couple to `relationship-modeling.md`; a tea ceremony is a 1-1, which couples to `sim/community.rs` organizing).
- **Ritual provides coherence:** regular tea rituals give characters predictable social structure — a mild stress outflow / cohesion boost (compartments, `qsp-theory-of-mind.md`).
- **Caffeine-lite simulation:** small alertness/productivity effect — noticeable, not a drug system.
- **Status signaling:** rare imported tea + ceremony skill = hospitality as soft power (diplomacy between rulers, gift economy).

## Mechanics to extract

- **Brick tea = commodity money:** compressed, shelf-stable, denominated — tea can literally become a currency in remote regions (couple to market sim).
- **Oxidation = transport range:** the further from origin, the more processed the tea that arrives — creates a local/distant good split the market sim reprices naturally.
- **Ceremony skill** as a social skill that improves ritual outcomes (skill system coupling).
- **Gathering places:** tea houses as a *building type* that hosts the ritual frame — a social infrastructure building (couple to `sim/community.rs` gathering mechanics).

## Implementation notes

- `game/tea.rs` holds the tree: cultivation→processing→grades; the ritual mechanic needs a "tea event" interaction that amplifies relationship-edge updates.
- Region-gated: tea plant doesn't grow everywhere — cultivation is climate/biome-gated, making it a real trade-driver (import or do without).

## Rating

The rare trade good that's also social infrastructure — the trade/ritual split makes it mechanically unique in the economy.
