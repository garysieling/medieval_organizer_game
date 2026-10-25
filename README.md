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
python scripts/fetch_creativity_corpus.py   # download the mythic corpus (~75 MB)
cargo run --release --bin creativity -- "I need a character name"   # 100 ideas
cargo test --lib                     # run tests
```

## Project Structure

```
game2-core/src/
├── engine/    ECS + headless renderer (BMP output)
├── agents/    20 agent workers (manager, engineer, playtesters, researchers, creativity generator)
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

- **20 agent specs** with JD, skills, Claude rules, KB structure
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
- `wiki/agents/creativity-generator.md` - creativity RAG: corpus, randomness, obviousness cut

## Sprint 4 Complete

The game now runs as a unified world, not just isolated systems.

**What's new:**
- `game_loop` (500 ticks), `full_session` (300 ticks), `full_game`, `mega_sim` - real unified runs
- Agent fleet live: playtester, fun inspector (3.5/5), manager sprint, Gary Agent, retrospective, pedantic review
- Player layer: outsider takes actions, builds influence, leads organization
- AI players: 6 ruler personalities + governance models
- Emergence at scale: guilds form, illness spreads, religion emerges, attachment varies
- 4 lit reviews: printmaking, textiles, tea+herbology
- New mechanics: weather, gossip, QSP reputation, 1-1 meetings, house meetings
- Video-style frame sequence (10 BMPs)
- 20+ binaries, 15+ evidence outputs, 30+ commits

Run `cargo run --bin full_session` for the complete game in one run.
