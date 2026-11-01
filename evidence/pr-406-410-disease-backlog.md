# PRs #406 + #410: Victorian disease backlog — waterborne & fever calendar

**Merged:** 2026-11-01 (squashes `fc6158a`, `5a13acb`)
**Research:** `wiki/literature-reviews/victorian-diseases.md` (PR #293)
**Earlier tranche:** PR #297 (DiseaseKind, names, incubation, per-kind immunity)

## PR #406 — Waterborne diseases: the reach carries the cholera

- `Route` enum on `DiseaseKind`: Contact / Water / Ingested; `contagious()` derives.
- New kinds: the cholera (1d incubation), the slow fever (typhoid, ~10d, 0.8
  immunity), the flux (dysentery) — all Route::Water; consumption → Contact.
- `land_tick`: homes within sniffing distance of effluent ≥ 0.5 roll weekly
  for waterborne cases; fouler water → fouler fevers; severity scales.
- Snow's loop: waterborne cases pour effluent back into the reach tile
  nearest home — outbreaks self-amplify.
- Effluent decays slowly → waves, not permanent plague.
- Tests: foul water sickens / clean water doesn't / sick foul the reach.

## PR #410 — The fever calendar: ague, crowding, and hunger's open door

- `DiseaseKind::Ague` ("the ague") — Route::Vector, seasonal window days
  240–310, 10d incubation, 0.05 immunity → effectively relapsing.
- `DiseaseKind::season()` — day-of-year window hook for the roster.
- `land_tick`: homes within 8 of river roll weekly during the ague's window.
- Crowding: exposure dose doubles for agents sharing a roof with a case.
- Malnutrition: `Hunger < 40` halves effective immunity (exposure resistance
  and severity clearance both).
- Tests: hungry catches first, same-roof doubles dose, autumn marsh ague.

## Quality gates

- `cargo build` clean; full `--lib` suite green except known pre-existing
  `test_villagers_stay_in_their_own_village` flake (fails on unmodified main).
- Real-game log evidence: "is taken with the cholera / the flux / the slow
  fever / the ague" entries appear organically.

## Remaining

Occupational trade-exposure, epidemic blame events, Vaccination Act /
Leicester Method organizing content — in `wiki/backlog.md`.
