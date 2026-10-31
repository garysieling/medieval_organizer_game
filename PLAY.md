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
collector takes his yearly cut from everyone - including you. Out past
the wall lives Willa the herb-wife: she keeps wild patches picked, brews
the pickings into remedies, and the sick walk out to her hut for a cure.
In the meadow below town a shepherd keeps the fold - sheep for wool at
shearing, goats for milk, a sheepdog at his heel, and the flock's smell
on his coat. Now and then a wolf comes in the night. A few yards keep
hens too; the household eats the eggs, and now and then a fox comes.
And one elder keeps an eye on lonely hearts - the heart-helper has a
word in both their ears, and now and then two folk come together.
Hungry folk gather mushrooms in the shade of the wood - some caps are
chanterelles, some are death, and a damp year brings Saint Anthony's
fire and blame for the baker. A beekeeper keeps skeps at the meadow's
edge - honey for the table, wax for the church's candles and the seals -
and when the candles run out the church goes dark. The larder keeps its
own score too: bread spoils, grain draws mice where no cat walks, and a
poor soul eats what shouldn't be eaten. And the talk splinters by trade:
a mason's grief isn't a baker's, so a mixed room only converges on the
troubles everyone shares - the taxes, the dark, the cursed bread. When a
master dies the bench passes - to the senior apprentice, else kin under
the roof, else a journeyman with no shop - and the lord takes his fifth
of what was left. And the kids grow up: errands run become bonds, and on
the day they come of age they take up the trade of whoever kept them
busiest. The map learns where people walk too - feet wear the ground
in, a well-trodden way is quicker underfoot, and the path to the
herb-wife's hut shows where the sick keep going.

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


## Multiplayer

One server owns the world in real time; clients are windows, browsers or
API agents. Connecting claims a character - the server hands back a token,
and presenting it on reconnect resumes the same one (the windowed client
keeps it in `game2.claim`; the browser will keep it in localStorage).

- `game_server [--port 8787] [--seed N] [--idle-secs 300]` - sim + WebSocket
- `game_ui --connect ws://host:port` - the same View renders the server's
  `Scene` (a serialized `UiModel`); input goes back as `Command`s
- Go quiet past `--idle-secs` and the fallback driver steers your wanderer
  until you come back; an agent's villager reverts to its own goals
- `game_bots ws://host:8787 N` - N heuristic villagers (eat, work, gather,
  talk), acting only when their errand finishes - quiet wire
- Speed/pause keys are solo-only (`speed-timer` cargo feature); the shared
  world runs at one pace
- `Dockerfile` + `k8s/deployment.yaml`: one pod, one world, port 8787
- `game2-web` (scaffold): wasm client - same core, WS in, canvas out
