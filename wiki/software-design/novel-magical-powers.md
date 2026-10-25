# Novel Magical Powers

**Category:** Design / Magic System
**Source:** Backlog item (High) — "Novel magical powers: innovative spell systems."
**Date:** 2026-10-25

## Description

How magic works in game2 — grounded in the sim, powered by generated description (`dynamic-magic-story-text.md`), and designed to produce *interesting* effects rather than fantasy-RPG fireballs.

## The design philosophy

- **Magic is real but rare** — a village hedgewitch, a hermit, a cunning priest; magic is embedded in the society, not a combat system bolt-on
- **Effects are sim-grounded** — every spell does something the sim can do: heal a wound, blight a field, reveal a secret, bind a fate, curse a harvest. The *magic* is in the effect's narrative dressing, not its rule-breaking
- **Power has cost and consequence** — belief, materials, backlash, reputation, madness; magic trades something real
- **Belief makes it work** — magic's efficacy scales with the practitioner's conviction AND the community's belief (`sim/religion.rs`, `sim/fundamentalism.rs`); a hedgewitch in a skeptical village is weaker

## Spell effect types (the sim-grounded palette)

| Type | What it does in the sim |
|---|---|
| **Heal/mend** | accelerate recovery, cure a disease (`sim/illness.rs`) |
| **Blight/curse** | reduce yield, afflict illness, sour a relationship, curse a harvest |
| **Reveal** | expose hidden state (a secret, a thief, the truth of a rumor) |
| **Bind/compel** | influence a villager's disposition (charm, oath-binding, geas) |
| **Ward** | protective effect (against disease, blight, misfortune) |
| **Scry** | peek at distant/hidden state (the fog-of-war cheat) |
| **Transmute** | convert resources (grain→flour at unnatural rate, lesser metals→higher) |
| **Commune** | speak with spirits/ancestors — draws on `agent_kb` memory of the dead |

## Novelty through combination

New spells = new *combinations* of effect + cost + target + narrative — the model generates the description, the sim executes the effects. A "curse of the seventh son" and a "blight of forgetting" are different *stories* over the same underlying effect types — that's how magic stays fresh without a hand-authored spell list.

## Constraints & backlash

- Material components (herbs — `herbology-apothecary-tree.md` is the source), sacrifice, time
- Belief requirements — some effects need communal faith, others need only the caster's conviction
- Backlash — miscast consequences: rebound on the caster, a warped result, local suspicion (`witch hunt` as an emergent risk)
- Detection — magic leaves traces; a suspicious village investigates a witch

## Related

- `sim/magic.rs`, `dynamic-magic-story-text.md`, `sim/religion.rs`, `herbology-apothecary-tree.md`

## Rating

Magic as society — integrated, not bolted on.
