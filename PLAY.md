# How to Play

## Quick Start

```bash
cargo build                    # build everything
cargo run --bin full_session   # the game in one run (300 ticks)
cargo run --bin game_loop      # unified game loop (500 ticks)
cargo run --bin showcase       # master showcase (5 evidence files)
cargo run --bin demo           # all systems demo + BMP renders
```

## The Game

You are **The Wanderer** - an outsider who arrives in a medieval village with nothing but the clothes on your back. You can't command anyone. You build influence by talking to people, organizing them, and earning trust.

### The Core Loop

1. **Talk** to villagers (1-1 meetings) - learn their concerns, build relationships
2. **Recruit** them to your organization (trust-gated)
3. **Organize** house meetings, committees, actions
4. **Invest** resources in your cause
5. **Build** your influence into actual power

### What Emerges

- Villagers have **attachment styles** (from relationship history) - affects how they respond
- **Guilds form** when merchants are vulnerable (they organize spontaneously)
- **Religion emerges** via homophily (devout cluster into congregations)
- **Illness spreads** through the village, apothecary can treat
- **Feuds escalate** - vengeance cycles, "can't get even without going one better"
- **Stratification** - economic hierarchy from who works/trades/produces
- **Weather events** - storms, droughts, frost affect food/health
- **World events** - rare large events that can help struggling players

### The Village

- Villagers get hungry, seek food, eat (hunger cycle)
- They age, get sick, form bonds, die
- Pets bond to owners (dog, cat, etc.)
- Seasons cycle (Summer→Autumn→Winter→Spring)
- Tech tree: printmaking track (paper mill is the gate)

### AI Villages

Other villages/castles are AI-controlled with different ruler personalities:
- **Diplomat** - talks, recruits, alliances
- **Merchant** - invests, trades
- **Zealot** - religious, orthodox
- **Tyrant** - military, control
- **Hermit** - isolationist
- **Builder** - infrastructure

### Player Actions

- **Talk** (`act_talk`) - build relationship, learn concerns
- **Recruit** (`act_recruit`) - try to get them to join (needs trust>0.5)
- **Invest** (`act_invest`) - spend gold for influence
- **Organize** - house meetings, committees
- **Contract** - formal agreements
- **Build** - commission structures

### Evidence

Every task produces software output:
- **BMP renders** - isometric board, minimap, character sheet, relationship graph, tech tree
- **Text streams** - sim logs, metrics, telemetry
- `evidence/` has all outputs

## For Developers

```bash
cargo test --lib              # run tests
cargo run --bin agent_task    # real Ollama AI task
cargo run --bin gary_review   # vision-alignment check
cargo run --bin playtest      # goal-oriented tests
cargo run --bin fun_score     # rubric scoring
```

## Research & Documentation

- `wiki/backlog.md` - research/mechanics backlog
- `wiki/project-state.md` - full inventory
- `wiki/evidence-index.md` - all software output catalog
- `wiki/literature-reviews/` - research (16 reviews)
- `wiki/agents/` - 20 agent specs
- `wiki/decision-rubric.md` - autonomous decision rules

## Notes

- Windows game, WSL available for tooling
- Some exes blocked by Application Control (env issue, code works)
- Ollama: gemma3:4b for fast agent loops, flint for depth
- Evidence = software output (BMPs, text streams), not desktop captures
