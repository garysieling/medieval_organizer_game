# Dynamic Magic & Story Generation from Text

**Category:** Design / Generative Systems
**Source:** Backlog item (High) — "Dynamic magic & story generation from text."
**Date:** 2026-10-25

## Description

Using language models to generate content at runtime from the world's *state*: spells, stories, descriptions, dialogs that adapt to what's actually happening — text as a render layer over the simulation, not as authored content. `sim/magic.rs` + `sim/emergence.rs` + the Ollama backend are the substrates.

## Two generative modes

| Mode | What it does | Use |
|---|---|---|
| **Magic generation** | spells/effects emerge from state: a ritual's *description* is generated from the caster's intent + materials + belief context — magic is real but its *expression* is textual | `sim/magic.rs`, `novel-magical-powers.md` |
| **Story generation** | the chronicle/dialog/descriptions rendered from the sim's actual events — the sim provides the *facts*, the model provides the *prose* | `emergent-storytelling.md`, `playthrough-video-generation.md` |

## The grounding contract

- **The sim is the truth; the text is the telling** — a generated spell description describes a real effect the sim actually does; a generated story recounts events that actually happened. Generated text can never *invent* state (no "a dragon appeared" the sim didn't spawn)
- **Prompt = world state** — generation inputs are structured: entities, events, relationships, traits, recent history (from telemetry/history) — the model narrates state, it doesn't author it
- **Style from corpus** — the Gutenberg corpus (`project-gutenberg-integration.md`) provides period voice; generated text *sounds* medieval because the reference material is

## Magic specifically

- A magic system where spells are *described* by the model but *executed* by the sim: the sim defines effect types (heal, blight, reveal, bind, transform) and constraints (cost, belief requirements, backlash); the model names/describes/resolves the flavor
- Novel magical powers (`novel-magical-powers.md`) — the spell is a sim effect + generated description; "new" spells are new *combinations* of real effects

## Architecture

```
sim state → prompt builder (structured facts) → local LLM (ollama) →
generated text → cache in agent_kb → render to UI/chronicle/dialog
```

- Cache aggressively: identical state → same description (determinism + cost)
- Fallback: templated text when the model is unavailable/slow
- The generation layer is *optional* — the sim works without it; it's a presentation enricher

## Rating

The killer differentiator — prose that *knows* the world.
