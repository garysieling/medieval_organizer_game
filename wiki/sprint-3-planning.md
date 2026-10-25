# Sprint 3 Planning — "Town to City"

Super-ambitious sprint. Goal: fill in the town with working systems across every workstream. 10x Sprint 2 scope (~40 tasks, parallel workstreams A-F).

## Workstream A — Complete Agent Specs (3 specs)
- A1: Aesthetic Consultant spec
- A2: Gameplay Consultant spec
- A3: Literature Survey Agent spec
- A4: Wire all 19 AgentTypes into Manager Bot spec compliance check

## Workstream B — Sim Domains (7 systems, the deepest)
- B1: `sim/economics.rs` — price formation, supply/demand, merchants
- B2: `sim/psychology.rs` — attachment emergence via anxiety/warmth episodes (Petters' model)
- B3: `sim/illness.rs` — health states, disease progression, simple QSP-style compartments
- B4: `sim/religion.rs` — faith traits, gathering events, emergent cohesion effects
- B5: `sim/community.rs` — influence networks, 1-1 conversations, power accumulation
- B6: `sim/climate.rs` — seasonal pressure + long-term climate change threat
- B7: `sim/magic.rs` — magic trait stub + text-ingestion hook for Gutenberg

## Workstream C — Game Systems (10 systems)
- C1: `game/world.rs` — map grid, regions, settlement placement
- C2: `game/terrain.rs` — midpoint-displacement procedural terrain (from lit review)
- C3: `game/economy.rs` — resources, production chains, market
- C4: `game/recipes.rs` — flexible recipes (multiple inputs → same output, Against the Storm style)
- C5: `game/tech_tree.rs` — constellation-style tech progression, printmaking track
- C6: `game/factions.rs` — asymmetric factions, ruler personality variation
- C7: `game/relationships.rs` — bonds forming between villagers (attachment-informed)
- C8: `game/contracts.rs` — contract creation, breach detection, reputation effects
- C9: `game/lifecycle.rs` — birth→aging→death, life events, lifecycle hooks
- C10: `game/military.rs` — simple abstracted battles (not a war sim)

## Workstream D — UI Foundation (4 systems)
- D1: `ui/isometric.rs` — 2.5D isometric board, tile rendering, camera
- D2: `ui/minimap.rs` — minimap + fog-of-war memory fade
- D3: `ui/character_sheet.rs` — traits, skills, health, relationships display
- D4: `ui/dialogs.rs` — character-to-character conversation framework

## Workstream E — Infrastructure (8 items)
- E1: `infra/telemetry.rs` — KPI recording (simplified OTel-style spans/metrics)
- E2: `infra/save_load.rs` — Datomic-style checkpoint history + replay
- E3: `infra/hot_reload.rs` — component reload stubs for dev loop
- E4: Real Ollama execution — gemma3:4b agent loop, one real task run end-to-end
- E5: `agents/` stubs → real modules for 4 most-used agents (Impl Eng, Manager, Playtester, Fun Insp)
- E6: Manager Bot: full sprint-plan builder + exit criteria checker
- E7: Task queue: priority field + deadline + assignee affinity
- E8: PR agent + conflict resolution basic flows via git worktrees

## Workstream F — Research & Playtesting (8 items)
- F1: Lit review — Medieval markets + guild economics
- F2: Lit review — QSP/ADME applied to game systems
- F3: Lit review — Community organizing tactics → mechanics
- F4: Lit review — Religion emergent effects in history
- F5: Full-game sim — 500-turn villager+food+economy run with metrics
- F6: Fun rubric draft — grind/stimulation balance rubric for Fun Inspector
- F7: Emergent effects proof — CA-based grass/forest spread test
- F8: Screenshot suite — 5+ PNGs across all workstreams + sprint-end video

## Exit Criteria
- [ ] All 19 agent specs complete in `wiki/agents/`
- [ ] All 7 sim domains have working systems (not just stubs)
- [ ] All 10 game systems have working components
- [ ] All 4 UI modules render something (even ASCII/PNG output)
- [ ] 1 real Ollama agent task completed end-to-end
- [ ] 500-turn sim run with telemetry output
- [ ] 4 lit reviews committed
- [ ] 5+ screenshots + 1 sprint video
- [ ] Retrospective + Sprint 4 plan

## Risks
- **Huge scope** — intentional; parallel workstreams let us land partial value in every area
- **UI complexity** — isometric is the hardest; keep it minimal (tile + camera, no animation)
- **Ollama speed** — flint 39.8B too slow; use gemma3:4b for agent loops
- **Context limits** — workstreams are independent; blockers logged, tasks pulled from queue

## Screenshot Targets
1. Isometric board render (ASCII or PNG)
2. Minimap with fog-of-war fade states
3. Character sheet layout
4. 500-turn sim metrics dashboard (text render)
5. Economy price-formation chart
6. Relationship graph emerging between villagers
7. Tech tree constellation layout
8. Sprint-end: full project structure tree
