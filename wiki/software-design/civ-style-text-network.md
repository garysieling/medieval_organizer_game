# Civ-Style Text & Network Dynamics

**Category:** Design / Information Systems
**Source:** Backlog item (High) — "Civ-style text/dialogue network dynamics."
**Date:** 2026-10-25

## Description

How information flows between entities — rumors, news, reports, diplomacy messages — modeled as a network with delay, distortion, and decay rather than instant global truth. A Civ-like information layer: you learn what your agents/allies report, when they report it, sometimes inaccurately.

## The model

- **Information is a fact with provenance** — `{what, where, when, source, freshness, reliability}` — same shape as the fog-of-war intel (`fog-of-war-memory-fade.md`)
- **Information travels** — along social/trade/political edges: a rumor reaches a village via a peddler, a report reaches a lord via a courier, news of a battle spreads mouth-to-mouth
- **Distortion on transmission** — each hop can amplify, mangle, or invert ("the lord is dead" → "the lord is dying" → "the lord's heir is dead")
- **Actor beliefs diverge** — different factions/individuals hold different "truths" — the player's map is one among many; propaganda and misinformation are *mechanics*, not flavor
- **Decisions run on beliefs** — villagers/rulers act on what they *believe*, which may be wrong; a famine rumor can trigger hoarding before any famine exists

## Message types

| Type | Content | Channel |
|---|---|---|
| Report | state observation (crop yield, army sighting) | agent → principal chain |
| Rumor | gossip, reputation, scandal | social graph |
| Diplomacy | treaty, threat, request | ruler ↔ ruler |
| Market | prices, goods available | trade network |
| News | births, deaths, battles, events | bells, heralds, pilgrims |

## Ties to other systems

- `fog-of-war-memory-fade.md` — intel freshness/decay (information entropy)
- `delegated-agency-characters.md` — the reporting channel is literally the delegation info flow
- `relationship-modeling.md` — information rides relationship edges
- `sim/emergence.rs` — misinformation cascades are an emergence source
- `world-generation-exploration.md` — discovery IS information acquisition

## Design rules

- **Truth is local** — no omniscient player view; the player's "facts" are only what reached them through someone's channel
- **Channels are game objects** — couriers, merchants, pilgrims, heralds are entities carrying information; intercepting a courier is a mechanic
- **The player can deceive** — planting false reports is a legitimate (if risky) tactic
- **Latency creates drama** — a war declared yesterday that the border hasn't heard about is a story machine

## Rating

Information asymmetry is one of the few dimensions that converts a sim into a *society* — high-value layer.
