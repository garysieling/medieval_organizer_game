# game2 - AI-Driven Medieval Fantasy Simulation

An emergent storytelling game with deep social/economic/psychological simulation, built by an AI agent workforce. Rust + custom ECS engine.

## Quick Start

```bash
cargo build                          # build everything
cargo run --bin demo                 # all systems demo + 5 BMP renders
cargo run --bin showcase             # master showcase (5 evidence files)
cargo run --bin sim500               # 500-turn sim with telemetry
cargo run --bin village_sim          # integrated multi-domain sim
cargo run --bin mega_sim             # 6 systems interacting
cargo run --bin agent_task           # real Ollama AI task (needs ollama+gemma3:4b)
cargo test --lib                     # run tests
```

## Project Structure

```
game2-core/src/
├── engine/    ECS + headless renderer (BMP output)
├── agents/    19 agent workers (manager, engineer, playtesters, researchers)
├── game/      game systems (villager, terrain, economy, tech_tree, factions, pets, tea, textiles, herbology, music, guilds, military, contracts, lifecycle, relationships, recipes, world)
├── sim/       sim domains (economics, psychology, illness, religion, community, climate, magic, fundamentalism, emergence)
├── infra/     task_queue, agent_kb, worktree, ollama, telemetry, save_load, hot_reload, pr_flow
└── ui/        isometric, minimap (fog fade), character_sheet, dialogs, graph, tech_view

wiki/          research wiki, agent specs, lit reviews, backlog, sprint docs
evidence/      software-generated output (BMP renders, text streams, test logs)
screenshots/   PNG evidence captures
.agents/       agent knowledge bases
```

## Key Achievements

- **19 agent specs** with JD, skills, Claude rules, KB structure
- **Full simulation**: villagers with hunger/health/attachment, market economics, illness, religion, community organizing, climate, magic
- **Emergent behavior**: attachment styles, guild formation, fundamentalism, CA fire spread - all from simple rules
- **Real AI execution**: OllamaClient + gemma3:4b executes tasks end-to-end
- **Headless rendering**: isometric board, minimap, character sheet, relationship graph, tech constellation - all render to BMP
- **Evidence pipeline**: every task produces software output (BMPs, text streams, telemetry)

## Sprint Status

- Sprint 0: Infrastructure (task queue, manager bot, agent KB)
- Sprint 1: Agent specs + capability matching + dependencies + worktrees + ECS + Ollama
- Sprint 2: Inference + villager + lit reviews + screenshots
- Sprint 3: Town to City - all systems built and integrated (this commit)

## Docs
- `wiki/project-state.md` - full inventory
- `wiki/sprint-3-summary.md` - what landed this sprint
- `wiki/backlog.md` - research/mechanics backlog
- `wiki/ai-agent-workforce.md` - agent architecture
