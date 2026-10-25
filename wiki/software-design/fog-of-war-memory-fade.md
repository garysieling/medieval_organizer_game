# Fog of War with Memory Fade

**Category:** System Design / UI-Sim
**Source:** Backlog item (Medium) — "If you haven't seen an area for a long time, it fades (not just static explored/unexplored). Memory decay, outdated information, rediscovery." Implemented: `game2-core/src/ui/minimap.rs` (fog-of-war fade).

## Description

Standard fog of war is binary or ternary: unexplored (black), explored (dimmed snapshot), visible (live). This design adds *decay*: explored areas' information *stales* — the snapshot you hold is from when you last looked, and its reliability fades with time since contact.

## States

```
unexplored ──see──> visible ──lose contact──> stale(t₀) ──decay──> stale(t₁) ──> ... ──> effectively-unknown
       ^                                          │
       └─────────────re-observe───────────────────┘
```

- **Visible:** live data.
- **Stale(age):** last-seen snapshot, rendered dimmer with age; the *data* is stale — a village may have grown, been raided, burned.
- **Effectively-unknown:** after long decay, rendered as unexplored but cheaper to re-explore (the world still has the terrain; the *information* is what decayed).

## Design decisions

- **What fades:** remembered *content* (buildings, units, ownership, resources), not terrain shape. Terrain is knowledge; contents are intelligence.
- **Fade rate as information design:** linear decay is legible ("old intel"); make the rate tunable per-thing — a mountain's content decays slowly (things persist), a war camp's fast (things move).
- **Presentation:** minimap fade is implemented; on the main board, stale areas render at reduced saturation + an "age" treatment so staleness is *felt*, not just dimmed.
- **Rediscovery:** re-observing refreshes the timestamp; a *diff* may be notable ("that farm became a fort") — diff-notice is the emergence hook: the sim changed the world while you weren't looking, and noticing is information.

## Emergent hooks

- Fog-of-war fade makes *reconnaissance* a real system: scouts produce refreshed intel that decays on a clock → patrol routes, lookout towers, informants become gameplay, not flavor.
- Multiplayer/AI-agent angle: each observer has *its own* stale map — shared maps (a faction's pooled knowledge) become a component: agents' beliefs, not a single truth. Feeds "different rulers see different worlds" (personality system).
- Drop-in character mode: a character knows only what *it* saw — same decay, personal scale.

## Implementation notes

- Store per-tile/per-entity: `last_observed_tick` + `last_observed_state`; render fade = f(current_tick − last_observed). `ui/minimap.rs` does the fade; the *belief* layer (stale state vs true state) is the next step — beliefs are queryable by AI agents so NPCs can act on outdated intel too.
- Telemetry: track "intel age" per agent — a measurable input to decision quality for the Data Scientist tester.

## Rating

Cheap to build, high design yield — one timestamp field turns a static convenience into an information economy.
