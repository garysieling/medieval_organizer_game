# PR #297: Victorian diseases — DiseaseKind roster, period names, real incubation

**Merged:** 2026-10-31 (squash, `1ebb424`)
**Branch:** feat/victorian-diseases (deleted)
**Companion:** research survey PR #293 (`wiki/literature-reviews/victorian-diseases.md`)

## What changed

- `DiseaseKind` enum in `game2-core/src/sim/illness.rs` — each kind carries display
  name, incubation days, contagiousness, and acquired-immunity gain.
  `Disease::new(kind, patient, severity)` is now the spawn path.
- Names per the survey naming table: "flu"→"la grippe", "the fever"→"scarlatina",
  "bad belly"→"the gripes", "saint anthony's fire" and "the sweats" kept,
  "consumption" already period-correct.
- `Stage::Latent` now functional: new cases incubate silently (no damage, no
  shedding); transmitted cases re-incubate → wave-like outbreaks.
- Per-kind acquired immunity (poisons 0, influenza 0.15, scarlatina 0.9) replaced
  the flat `ACQUIRED_IMMUNITY` constant.
- `is_sick` = symptomatic only; latent carriers don't seek the herb-wife yet.
- Log strings name the real disease ("Bram is taken with la grippe"); pin and
  colour heuristics extended; balance_tester illness keywords updated.

## Quality gates

- `cargo build` — clean
- `cargo test` — 538/539 pass; `test_villagers_stay_in_their_own_village` fails
  identically on unmodified main (pre-existing sim flake, villager wanders >30d)
- New tests: latent incubates silently then strikes + doesn't shed;
  scarlatina vs la grippe differential immunity

## Deferred to backlog (wiki/backlog.md)

Waterborne spread via existing effluent/pit_water tiles (Broad Street mechanic),
seasonal outbreak table, occupational trade-exposure, crowding/class modifiers,
epidemic blame events, Vaccination Act / Leicester Method organizing content.
