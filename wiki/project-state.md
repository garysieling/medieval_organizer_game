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
