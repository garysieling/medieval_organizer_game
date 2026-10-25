# Sprint 4 Planning - "The Game Starts Playing"

Sprint 3 built all the systems. Sprint 4 makes them a real game.

## Goals
- A playable/simulatable game loop, not just demos
- Agent fleet does real work via Ollama
- Evidence: video or long-form sim log of actual gameplay

## Workstreams

### W1 - Unified World (the big one)
- Single WorldState holding all components on shared entities
- Game loop: tick -> all systems run -> metrics/render
- Save/load real state (Datomic checkpoints serialize world)

### W2 - Agent Fleet Live
- OllamaClient executes real tasks: ImplementationEngineer writes code, SimplePlaytester runs sims, FunInspector scores
- Manager Bot builds sprint plans, delegates via capability matching
- Retrospective agent analyzes outputs
- Gary Agent: vision-alignment checks

### W3 - Real Gameplay
- Player actions: build, trade, recruit, organize (via org/community systems)
- Win/lose conditions or meaningful progression
- Villagers live: eat, form bonds, get sick, attend services, join orgs

### W4 - Polish & Evidence
- Isometric board shows real entity state
- Video of a long sim run (sprint-end)
- Fix remaining env/test issues
- Character sheets show real data

## Exit Criteria
- [ ] Full game loop runs 1000+ ticks with all systems
- [ ] 3+ agents complete real Ollama tasks
- [ ] Player can take meaningful actions (not just watch)
- [ ] Video or detailed sim log of a full playthrough
- [ ] All tests passing (or environment documented)

## Risk Notes
- W1 is the hardest - proper component wiring, not just demos
- Agent fleet needs prompt engineering for reliable task output
- Rendering real game state (not canned data) is the payoff
