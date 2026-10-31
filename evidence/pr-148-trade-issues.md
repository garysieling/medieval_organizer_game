# PR #148 — The talk splinters by trade

https://github.com/garysieling/game2/pull/148

## What changed
- `IssueTag`: `&'static str` → `String`; `issue_tag(world, e, r)` scopes trade-flavored problems by the speaker's vocation — `"materials (mason)"` vs `"materials (baker)"`
- Scoped: `NoWorkplace`, `NoMaster`, `NoInput`, `DerelictNearby`, `OverseerCut`, `Smelly`, `Loss`, `Want::BetterTools`. Commons (Taxed, Broke, Blamed, SourGut, NoHome, wants) stay unscoped — the bridge issues
- `issue_name` parses the scope: "materials (mason)" → "the masons' workshops starved of materials"
- Meeting convergence unchanged in code but now naturally per-trade; a room that fails with ≥3 voices on the same *base* across different trades logs "the talk splinters by trade" and reports it in the meeting outcome

## Quality gates
- `cargo test --lib -p game2-core`: 386/386 (3 new trade_issue_tests)

## Design intent (from Gary)
Bridge issues are the organizer's real game — farm one trade for quick wins, but village-scale power needs the issues that cross trades. F42's environmental grievances are deliberately unscoped for this reason.
