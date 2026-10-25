# Sprint 4 Planning — "The Game Starts Playing"

Sprint 3 built all the systems in isolation. Sprint 4 wires them into a real, playable game and gets the agent fleet doing real work. Same ambitious scale as Sprint 3 (~40 tasks, parallel workstreams A-G).

## Workstream A — Unified World & Game Loop (the foundation)

- A1: `WorldState` — single struct holding all domains, proper component wiring on shared entities
- A2: Game loop — `Game::tick()` runs all systems in order, records metrics, renders state
- A3: Entity archetypes — `Villager` gets full stack (Pos, Hunger, Health, Attachment, Faith, Citizen, Trader, Relations, Age)
- A4: Real serialization — world state → text file → reload (Datomic checkpoints actually save/restore)
- A5: Event system — emergent events fire (famine, festival, plague, conflict) with conditions
- A6: Time system — ticks → days → seasons → years, life events on schedule
- A7: `game_loop.rs` binary — the real game running, not just demos

## Workstream B — Agent Fleet Live

- B1: ImplementationEngineer completes real code tasks via gemma3:4b (multiple, varied)
- B2: SimplePlaytester runs goal playtests ("survive 100 ticks", "feed 5 villagers") and reports
- B3: FunInspector scores playtest output against rubric, files assessment
- B4: RetrospectiveAgent analyzes sprint outputs, files findings to its KB
- B5: Manager Bot orchestrates a mini-sprint: builds plan → delegates → tracks → reports
- B6: Agent KBs update from real work (patterns-learned, task-history written by actual runs)
- B7: Gary Agent v1 — reads project-state + recent work, gives on-track nudges (vision alignment)

## Workstream C — Real Gameplay Depth

- C1: Production economy — farm→wheat→mill→flour→bakery→bread chain working end-to-end
- C2: 1-1 organizing meetings — player talks to citizen → reveals concerns/networks → recruit → org grows
- C3: House meetings — gather identified citizens → shared agenda emerges → committee forms
- C4: Guild formation live — merchants spontaneously organize when trade is risky
- C5: Religion emergence — congregations form via homophily; fundamentalism risk from conditions
- C6: Illness outbreak — disease spreads through village, treatments affect outcome
- C7: Contracts + reputation — sign contract → breach → reputation damage → trust consequences

## Workstream D — Player Layer

- D1: Player character — an outsider entity with influence (not authority), goals, resources
- D2: Player actions — talk, recruit, invest, organize, contract, build (each maps to a system call)
- D3: Action menu — text-based player interface (choose actions each turn)
- D4: Win/lose or progression — authority milestones (influence→recognized→formal power)
- D5: AI players — heuristic agents play villages concurrently via same action interface

## Workstream E — Evidence & Polish

- E1: Full game loop run — 500+ ticks, all systems, metrics logged to evidence/
- E2: Video or frame-sequence of a real playthrough (assemble BMPs into a clip or long-form log)
- E3: Character sheet shows real entity data pulled from world
- E4: Isometric board shows live entity positions + terrain from actual sim
- E5: Relationship graph evolves over time (bonds form/strengthen/rivalries emerge)
- E6: Screenshot suite — 8+ real outputs across all systems
- E7: Fix remaining env issues (WSL fallback for blocked exes)

## Workstream F — Research & Emergence

- F1: Lit review — printmaking history detail (for the detailed tech tree Gary wants)
- F2: Lit review — textiles/fabric history + sensory processing (autism-informed)
- F3: Lit review — tea history: trade + psychometric effects + rituals
- F4: Lit review — herbology/apothecary historical accuracy
- F5: Emergent mechanic — weather events (storms, droughts) affecting food/health
- F6: Emergent mechanic — gossip/rumor propagation through relationships
- F7: QSP compartment model applied to reputation (states + transition rates)

## Workstream G — Process & Quality

- G1: PedanticEngineer review run — audit codebase, file findings, propose 2+ refactors
- G2: Test suite green — all tests pass or documented env blocks
- G3: PR flow live — one real worktree→PR→review→merge cycle
- G4: Conflict resolution live — one real merge conflict resolved
- G5: Sprint retrospective + Sprint 5 plan

## Exit Criteria

- [ ] `game_loop` binary runs 500+ ticks: villagers live, economy flows, seasons change, events fire
- [ ] 3+ agents complete real Ollama tasks with logged output (not simulated)
- [ ] Player can take 5+ meaningful actions that visibly change world state
- [ ] Guild + congregation + org form emergently during a run (not spawned)
- [ ] Video or compiled evidence doc showing a real playthrough
- [ ] Lit reviews produce 3+ concrete mechanics added to backlog
- [ ] Retrospective agent files real findings; Manager Bot runs a mini-sprint solo

## Evidence Targets

- Video/frame-sequence of a real game session
- Full game loop metrics log (500+ ticks, all systems)
- Agent task outputs from real Ollama runs (code written, playtests scored)
- Emergent events logged (guild formed at tick N, congregation emerged, disease spread)
- Player action → world state change examples
- Refactor proposals from PedanticEngineer review

## Risks & Mitigations

- **W1 is the hard part** — unified WorldState vs separate demos. Mitigate: start there, it's the foundation
- **Ollama reliability** — gemma3 output needs parsing/validation. Mitigate: structured prompts + retry loop
- **Emergence tuning** — orgs may not form without tuning. Mitigate: adjust thresholds from sim outputs
- **Context/time** — huge scope again. Mitigate: parallel workstreams, blockers logged not stalled
