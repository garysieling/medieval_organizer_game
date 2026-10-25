# Herbology / Apothecary Medicine Tree

**Category:** Tech Tree / Economy / Folk Medicine
**Source:** Backlog item — "simple witchy apothecary-style medicine tree. Plants, natural remedies, pseudo-medical treatments." Implemented stub: `game2-core/src/game/herbology.rs`. Companion: `technology-medical-options.md` (this is its Tier 0).

## Description

The pre-modern medicine floor: plant gathering → preparation → remedies of *variable* efficacy. Deliberately witchy in flavor — a wise-woman's materia medica, not a hospital. The design tension that makes it interesting: most remedies are palliative or placebo, a few genuinely work, and the *culture's belief* in them runs on its own dynamics (couples to religion/community).

## Historical skeleton

```
forage/cultivate → identify → prepare (infusion/decoction/tincture/poultice/salve) → administer
```

- **Materia medica:** willow bark (real analgesic — salicin), foxglove (real cardiac — dangerous narrow window), chamomile/valerian (mild real sedatives), yarrow (wound), elderberry, honey (real antiseptic), plus aromatic/bitters that are mostly ritual.
- **Preparation changes effect:** infusion vs decoction vs tincture extract different compounds — preparation *is* the skill content. A practitioner skill (`skill-improvement-degradation.md`) that changes remedy quality.
- **Seasonality:** herbs are foraged on a calendar — winter limits to dried stocks; ties to seasons and storage (`sim/climate.rs`).

## The efficacy spectrum (the interesting mechanic)

```
remedy efficacy = real_effect(chemistry) + ritual_effect(belief, setting, practitioner)
```

- **Real effects:** a small set of remedies genuinely shift illness-compartment rates — willow bark on pain/fever, honey on wound infection, valerian on stress/sleep. The deep-illness sim feels these.
- **Ritual/placebo effects:** a remedy administered with confidence in a social ritual genuinely improves subjective outcomes — stress compartment drain, hope/morale bump. *Belief is mechanically real even when chemistry isn't.*
- **Harmful remedies exist:** some foraged things hurt (dose errors, misidentified plants) — practitioner skill lowers the floor, doesn't remove it. This is what makes the healer's reputation matter.
- **Interaction with belief:** a healer trusted by the community gets better outcomes on identical remedies — the *social* component compounds the chemical one (organizing/community coupling).

## Game design points

- **Apothecary as character class:** the wise-woman/herbalist is a legible role — garden + stillroom + foraging rounds + reputation economy. A satisfying specialized villager.
- **Knowledge as inventory:** a healer's *knowledge* (which plant does what, doses) is a tradable/inheritable asset — dies with her unless apprenticed (lifecycle + skill coupling). Knowledge loss is a real medieval dynamic the sim can dramatize.
- **Witchy without magic-real:** the vibe is folk-magic, the simulation is chemical + social; whether the culture *believes* it's magic is the religion layer's business (possible cross-labeling: a remedy treated as charm vs as medicine by different characters — belief modeling, `qsp-theory-of-mind.md`).
- **Bridge to real medicine:** medical tech (`technology-medical-options.md`) doesn't delete this tree — it *reframes* it; folk remedies persist as culture even after diagnostics arrive.

## Implementation notes

- `game/herbology.rs` holds plants/preparations; remedies become items with an efficacy vector (chemical vs belief) applied to illness/mood compartments on use.
- Forage spots on terrain (biome-tagged) → regional materia medica differences, same regionalism pattern as textiles/tea.

## Rating

High charm-per-code: small sim, strong theme, real couplings into illness/psychology/religion.
