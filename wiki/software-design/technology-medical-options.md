# Technology's Effect on Medical Options

**Category:** System Design / Tech Tree
**Source:** Backlog item — how medical technology changes what illness gameplay exists. Companion to `deep-illness-simulation.md` (QSP stages) and `game/herbology.rs` (apothecary remedies).

## Description

Medical tech doesn't just "unlock cures" — it unlocks *capabilities in sequence*: seeing illness (diagnostics), explaining it (theory), changing its course (treatment), preventing it (public health). Each tier changes what the illness sim's compartments mean to the player. Designed so most of the tech tree is *medieval-plausible* — the game sits mostly in the herbology-to-early-humoral era, with later tiers as reachable but exceptional.

## Progression tiers

| Tier | Capability | What it changes in the illness sim |
|---|---|---|
| **Folk/herbal** | remedies for symptoms | comfort + small rate shifts on mild illness; *belief in efficacy* matters as much as efficacy (placebo/culture hooks into religion) |
| **Diagnostic** | detect illness before symptoms | the asymptomatic compartment becomes *visible* — prognosis becomes a decision input; screening appears |
| **Theoretical** | explanatory framework (humoral, miasma, later germ theory) | determines *which treatments unlock*; a wrong-but-coherent theory still unlocks coherent treatments — theories are simulated, not judged by the engine |
| **Surgical/interventional** | direct intervention on late stages | shifts rates on compartments diagnostics can't help |
| **Preventive/public** | sanitation, quarantine, inoculation | acts on *population* rates, not individual trajectories — the epidemic layer's real counter |

## Design points

- **Visibility precedes cure:** the deep-illness model's asymptomatic phase is invisible until diagnostics — a healer with no diagnostics sees "sudden" death that a diagnostic healer saw coming for a year. Tech literally changes what can be *perceived*, which is rarer and more interesting than tech raising numbers.
- **Theory shapes treatment, not truth:** treatments unlock under whatever theory the culture holds — this lets medicine evolve along historically messy paths (and feeds the allegory: rulers may *fund* the wrong theory).
- **Herbology is the floor:** `game/herbology.rs` is tier 0 — always available, mostly palliative, occasionally useful; its remedies are the baseline every better tier replaces.
- **Erroneous tech matters:** some unlocks *harm* (bloodletting-tier interventions) — tech trees should include historically real dead ends.

## Couplings

- **Deep illness sim:** every tier above shifts one compartment's visibility or rate — the design contract is "tech = which compartment levers exist."
- **Economy:** healers + ingredients + facilities are production chains (`game/guilds.rs`, `sim/economics.rs`).
- **Religion/folk medicine:** efficacy partially runs on belief/community (ties to `sim/religion.rs` cohesion and the satirical thread — a healing cult that *works* socially).
- **Lifecycle:** mortality rates per age feed `game/lifecycle.rs` — medical tech is how the sim's lifespan distribution drifts over a campaign.

## Rating

The right granularity for this game: medical tech as *capability tiers*, not "+10% health."
