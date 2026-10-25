# Project State - 2026-10-25

## What's Built (Working Code)

### Engine
- `engine/ecs.rs` - World, Entity, components, System trait, Scheduler (tested)
- `engine/render.rs` - headless pixel buffer, BMP encoder, diamond/rect/line primitives

### Infra
- `infra/task_queue.rs` - 9-field line format, dependencies, priorities, deadlines
- `infra/agent_kb.rs` - agent knowledge bases
- `infra/worktree.rs` - git worktree agent isolation
- `infra/ollama.rs` - OllamaClient (gemma3:4b verified working)
- `infra/telemetry.rs` - metrics/spans export
- `infra/save_load.rs` - Datomic checkpoint history
- `infra/hot_reload.rs` - component reload registry
- `infra/pr_flow.rs` - PR lifecycle + conflict detection

### Sim Domains
- economics (market/price formation), psychology (attachment emergence)
- illness (QSP compartments), religion (congregations/cohesion)
- community (influence/orgs/1-1s), climate (seasons/drift), magic (grimoire)
- fundamentalism (emergence conditions), emergence (CA fire spread)

### Game Systems
- world, terrain (fractal), economy, recipes, tech_tree, factions
- relationships, contracts, lifecycle, military, guilds
- pets, tea, textiles, herbology, music, villager

### UI (all render to BMP)
- isometric (2.5D board + entities), minimap (fog-of-war fade)
- character_sheet (panel layout), dialogs, graph (social bonds), tech_view (constellation)

### Agents (real code)
- manager_bot.rs (sprint plans, capability matching, exit criteria)
- implementation_engineer.rs (OllamaClient task execution)
- simple_playtester.rs, fun_inspector.rs (rubric scoring)

## Binaries / Demos
- `demo` - exercises all systems, 5 BMP renders + text output
- `sim500` - 500-turn village sim with telemetry
- `village_sim` - integrated multi-domain sim
- `mega_sim` - 6 systems interacting
- `showcase` - master showcase, 5 evidence files
- `agent_task` - real Ollama task execution
- `save_load_demo` - Datomic checkpoints/replay

## Evidence
- 8 BMP renders (isometric, minimap×2, char sheet, rel graph, tech tree, showcase world, showcase tech/rel/char)
- Text outputs: demo, sim500, village_sim, mega_sim, showcase, agent_task, save_load, tests
- 12+ PNG screenshots via screenshot.ps1

## Documentation
- 19 agent specs in wiki/agents/
- 4 lit reviews (medieval econ, QSP, community organizing, religion)
- Decision rubric + pending-decisions log
- Fun rubric, screenshot policy, sprint history KBs

## Agent Specs
All 19 defined: 3 engineering, 5 testing, 4 research/analysis, 2 consult, 4 orchestration, 1 video, 1 lit survey + Gary Agent (backlog)

## What's Next
- Unify world state (components on shared entities)
- Agent fleet executes real tasks via Ollama fleet
- Save/load + hot reload end-to-end
- Video of long sim run
- More emergent mechanics from lit reviews
- Fix filesystem tests (temp dirs)

## Sprint 4 Additions

### New Code
- `bin/game_loop.rs` - unified 500-tick loop
- `bin/full_session.rs` - all systems in one run
- `bin/full_game.rs` - all domains on shared entities
- `bin/mega_sim.rs` - 6 systems interacting
- `bin/production_chain.rs` - wheat->flour->bread
- `bin/guild_demo.rs` - emergent guild formation
- `bin/outbreak.rs` - illness spread + treatment
- `bin/contracts_demo.rs` - breach consequences
- `bin/religion_demo.rs` - homophily congregation
- `bin/player_demo.rs` - outsider->leader
- `bin/video_seq.rs` - frame sequence (video)
- `bin/playtest.rs` - goal-oriented tests
- `bin/fun_score.rs` - rubric scoring
- `bin/code_review.rs` - refactor proposals
- `bin/retrospective.rs` - action items
- `bin/gary_review.rs` - vision nudges
- `bin/manager_sprint.rs` - mini-sprint orchestration
- `bin/final_showcase.rs` - 4 renders
- `agents/gary_agent.rs` - vision alignment
- `agents/retrospective_agent.rs` - sprint analysis
- `agents/pedantic_senior_engineer.rs` - complexity review
- `agents/creativity_generator/` - mythic-corpus RAG
- `game/organizing.rs` - 1-1 meetings, house meetings
- `game/player.rs` - PlayerCharacter, actions
- `game/ai_player.rs` - AI rulers, governance
- `game/action_menu.rs` - text-based UI
- `game/guilds.rs` - emergent guild formation
- `sim/weather.rs` - storms/drought/frost
- `sim/gossip.rs` - rumor propagation
- `sim/reputation_qsp.rs` - QSP reputation model
- `sim/fundamentalism.rs` - extremism emergence
- `ui/graph.rs` - relationship bonds
- `ui/tech_view.rs` - tech constellation

### New Evidence
- 12+ BMP renders (world, relationships, tech, char, minimap, isometric)
- 10 frame sequence (video)
- 15+ text streams (all sims, reviews, scores)
- 4 lit reviews (printmaking, textiles, tea+herbology)

### Sprint 4 Status
All workstreams A-G substantially complete. Agent fleet live, unified world running, player layer working, real emergence at scale.
