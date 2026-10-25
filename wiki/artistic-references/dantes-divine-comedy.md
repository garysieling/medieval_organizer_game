# Dante's Divine Comedy — Medieval Mindset Study

**Type:** Thematic Reference
**Source:** Backlog item (High) — extract medieval humor/faith/fear/virtue, everyday Life contexts.
**Date:** 2026-10-25

## Description

The Divine Comedy (c. 1320) is the single densest window into the medieval *world-model*: what people feared, what made them laugh, how the cosmos was organized, how sin/virtue were categorized, how social roles were read morally. It's also inherently *spatial* — a world organized as concentric zones with rules, which maps naturally onto our regional/isometric world.

## What it gives the game

### The moral taxonomy (systems-ready)

Dante's categorization is *machine-like* — sins ranked and grouped by the disposition of love: perverted (pride, envy, wrath), deficient (sloth), excessive (avarice, gluttony, lust). That's a ready-made vice/virtue spectrum to seed villager moral dispositions and to make "sin" legible as *motivation gone wrong*, not badness — aligned with the psych model.

### Medieval humor

Real medieval comedy: brutal, earthy, bodily, status-based (tricksters mocking fat priests, peasants outwitting masters). Villager banter/dialog should carry this register — sharp and bawdy, not sanitized-fantasy nice.

### Faith as lived texture

Pilgrimage, relics, feast days, saints as neighbors you *petition*, purgatory as collective obligation (praying for the dead is how a village works). Church isn't a building, it's a *schedule and a ledger of obligations* — feeds `sim/religion.rs` events: processions, masses for the dead, shrine visits, fear of judgment day.

### Fear inventory (the era's anxieties)

- Judgment/hell — the background terror
- Famine & crop failure — death-level stakes, not inconvenience
- Sickness as *moral sign* (plague as divine or astrological) — feeds `sim/illness.rs` believability
- Chaos/war — local violence, not geopolitics
- Heresy/excommunication — spiritual exile as social death

### Virtue register

Medieval virtue is *relational and communal* — charity as obligation, hospitality, keeping faith, honoring custom — less "being a good person" than "playing your role well." Aligns with the multi-dim relationship model (`relationship-modeling.md`).

## The architectural gift

Dante's world is *concentric and traversed* — you move through zones with distinct rules/atmospheres. Our world's regions and the map's zones can borrow this: moral texture as spatial texture. (Also see `constellation-tech-trees.md` — Purgatorio's terraces as progression structure.)

## Applications

- `sim/religion.rs` event & virtue system
- Villager dialog/banter tone (dialog spec)
- Ruler/priest character archetypes for `regional-ruler-personality.md`
- Public-domain text source: Project Gutenberg (Doré illustrations for art reference)

## Tags

- era: high medieval
- theme: faith, fear, virtue, humor
- source: public domain
