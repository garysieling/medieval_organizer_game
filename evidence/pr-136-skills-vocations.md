# PR #136 — Skills, class & vocations: a living town in the windowed game

- **Link:** https://github.com/garysieling/game2/pull/136
- **Branch:** feat/skills-vocations (worktree: C:/projects/game2-skills)
- **Plan:** wiki/skills-vocations-plan.md

## What changed
- 4 new model modules: vocation (classes/vocations/levels/craft), desires
  (secret wants/problems, reveal + issue ledger, 360d expiry), household
  (buildings as entities: occupants/worker/overseer/stockpile/derelict),
  livelihood (purses, satchels, work days, trade, odd jobs, tax round)
- game_ui: ~30-person seeded town, clickable buildings + rich inspectors,
  KNOWN ISSUES line, ctx actions, children, tax collector, minstrel,
  reveal rolls in talk & meetings with parallel-complaint boost
- Sim fixes: errand passability aborts, hunger-first errands, meeting
  errand drop, homeward corner-snap, target_food retarget to stocked store

## Quality gates
- cargo check -p game2-core: clean (-D warnings)
- cargo test -p game2-core --lib: 276 passed
- cargo test -p game2-core --bin game_ui: 76/76 passing
- Evidence: evidence/skills-town.bmp (day 40), skills-town-day240.bmp

## Status
- Created by Devin; awaiting human review. Not merged.
