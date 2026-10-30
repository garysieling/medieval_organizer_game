# How to Play

## Quick Start

```bash
cargo build                    # build everything
cargo run --bin game_ui        # THE GAME - windowed UI, real sprites
cargo run --bin game           # text-mode version
```

## The Window (game_ui)

```
+----------------------------------+----------+
|  isometric map: village,         |  THE     |
|  buildings, trees, villagers     | WANDERER |
|  walking around, name labels     |          |
|                                  | [Talk]   |
|  YOU = white marker              | [Recruit]|
|  click a villager to inspect     | [Invest] |
|                                  | [Organize]|
|                                  | [Build]  |
|                                  | [Wait]   |
|                                  |          |
|                                  | Inspector|
|                                  | (stats)  |
|                                  |          |
|                                  | LOG      |
+----------------------------------+----------+
```

## Controls

| Input | Action |
|-------|--------|
| **Click button** | Take action on selected villager |
| **Click villager** | Select/inspect them (class, vocation, skill, gold, home, shop, and any secrets they've let slip) |
| **Click building** | Inspect it: purpose, who works or lives there, stock and what's for offer |
| **1-5** | Select villager by number |
| **T/R/I/O/B/W** | Keyboard: Talk/Recruit/Invest/Organize/Build/Wait |
| **Arrows** | Pan the camera around the map |
| **Space** | Pause/unpause |
| **0** | Toggle 4x speed |
| **H** | Help overlay (all controls) |
| **Click minimap** | Jump camera to that region |
| **+/-** | Zoom map |
| **Enter** | Start (title screen) |
| **F12** | Save BMP screenshot to evidence/ |
| **Esc** | Quit |

## The Game

You are **The Wanderer** - an outsider who arrives with nothing. You can't command anyone. Build influence by talking, organizing, recruiting.

## The Game

You are **The Wanderer** - an outsider who arrives in a medieval village with nothing but the clothes on your back. You can't command anyone. You build influence by talking to people, organizing them, and earning trust.

### The Core Loop

1. **Talk** to villagers (1-1 meetings) - learn their concerns, build relationships
2. **Recruit** them to your organization (trust-gated)
3. **Organize** house meetings, committees, actions
4. **Invest** resources in your cause
5. **Build** your influence into actual power

### The Town

Everyone has a **class** (working, petit bourgeois, PMC - knights, priests,
overseers, the tax collector) and working folk a **vocation** (farmer,
baker, toolmaker, tailor, glazier, printmaker, mason, luthier...).
Skill climbs **journeyman -> apprentice -> master** with use - fast under
a paired master, slow alone. Workshops stockpile goods for sale; a trade
with no shop works at half rate, and a shop with no worker stands derelict.
Children (flagged "(child)") run odd jobs for a few gold. The tax
collector takes his yearly cut from everyone - including you.

Everyone carries secret **wants** and **problems**. Talk and house
meetings can draw them out; parallel complaints make people likelier to
share. Three voices on one trouble inside a year and the organizer sees
the issue - shown on the panel's ISSUES line.

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
