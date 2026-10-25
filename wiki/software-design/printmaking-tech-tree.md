# Detailed Printmaking Tech Tree

**Category:** Tech Tree / Craft / Culture
**Source:** Backlog item — detailed printmaking tech tree.

## Description

Printmaking as the information-technology craft of the medieval-to-early-modern world: how images and text became reproducible. It's a tree where *each tier changes what information can spread* — making it a coupling point for literacy, propaganda, religion, and commerce. Mechanically: a production tree whose output is *copies*, and copies are how ideas travel.

## Historical skeleton

| Tier | Technique | ~Era | What it unlocks |
|---|---|---|---|
| **Seal/stamp** | carved seals, signet rings | ancient | authentication, mark-making |
| **Rubbing** | inked rubbings of stone/metal | ancient (China) | first faithful text/image copies |
| **Woodblock print** | carved relief block | 8th-9th c. China; medieval Europe | devotional images, playing cards, cheap multiples |
| **Block book** | text+image carved per page | 15th c. Europe | mass-produced picture books |
| **Movable type** | Gutenberg-style press | mid-15th c. | text at scale — the information revolution |
| **Engraving/etching** | intaglio metal plate | 15th-16th c. | fine images, maps, currency engraving |
| **Lithography** | planographic stone | late 18th c. | cheap color posters, newspapers' illustrations |

## Why it's a real game system (not just flavor)

- **Information propagation as mechanic:** movable type is where *ideas* become goods — pamphlets, tracts, maps. Couples directly to belief-spread (`sim/fundamentalism.rs`, religion, propaganda) — a Reformation needs a printing press. The tech tier determines *how fast narratives spread*, not just what you can sell.
- **Literacy coupling:** cheap text → reading spreads → skilled-labor pool widens (skill/economy coupling). Print is the multiplier on the whole knowledge economy.
- **Maps & knowledge goods:** engraving unlocks map production — navigation/trade/fog-of-war knowledge becomes a produced good (couples to world gen + trade).
- **Counterfeit & authority:** seals→engraving → currency, charters, indulgences — authentication vs forgery is a real mechanic (contracts/legal framework coupling).
- **Art market:** prints make art *reproducible* — images as a trade good with a value tier (regional woodblock styles, artist reputation — textiles-style regionalism again).

## Tree structure sketch

```
carving craft ─┬─ seal/signet (authentication)
               ├─ rubbing (faithful copies)
               ├─ woodblock (cheap images, cards, block books)
               ├─ movable type (text at scale → literacy/propaganda multipliers)
               ├─ engraving/etching (fine images, maps, currency)
               └─ lithography (mass visual culture)
```

- Inputs: paper/parchment/pigments (paper mill tree is a prerequisite — couples to economy; rag paper vs parchment is itself a production choice).
- Each tier raises *copy throughput* and *fidelity* — the two numbers that determine what the print shop can sell.
- Printer/publisher as a character class (like the apothecary): workshop + reputation + *risk* — printing the wrong tract is how you get in trouble with rulers/factions (allegory coupling).

## Implementation notes

- Model prints as goods with a `content` property (devotional image, tract, map, currency plate) — content type is what couples outward (belief, knowledge, legal).
- Movable type is the designed mid-tree spike: its multiplier on text goods is large enough to visibly change sim dynamics (literacy rate, tract circulation) — a tier worth the buildup.

## Rating

The sleeper tech tree — it quietly gates how every idea in the game spreads.
