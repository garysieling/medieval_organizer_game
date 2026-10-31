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
herb-wife's hut shows where the sick keep going. The land keeps score:
the wood thins and isn't replanted where feet and hooves keep it bare,
the worked pit fills with water until it's a pond, and the reach below
town carries the stink of everything dumped in it - and all three become
troubles the whole village can agree on. And when blame piles up the
village climbs the ladder of shame: a bad word, the cold shoulder, a
boycott, a crowd at the door, the stocks - or the road out. A respected
soul in the stocks is a scandal, not a lesson. And the year has its
feast days: St. Swithin's, the Harvest Home, Candlemas, St. Martin's -
the village gathers at the market, the good stock comes out, the truth
comes out in the toasts, and the tax collector is never invited but
always shows. Some of the cottages keep a garden bed beside the door -
the soil strip fills in over the weeks with marigold and feverfew, and
when the herb-wife's own shelves run thin she tends the beds of folk
who don't begrudge her. A weaver keeps the loom house now - the
shepherd's wool in, village cloth out - and the tailor cuts local
cloth before he'd pay the trader's price for it. Down in the cheese
cellar by the fold the milk becomes wheels in the dark - this village's
is the stone-cellar cheese - and now and then a noble rides in and pays
triple for a wheel well past its year. And the day's work leaves its
smell: the shepherd smells of the fold, the beekeeper of smoke and wax,
the herb-wife of bitter herbs - a suitor's suit falls flat if they
haven't had a rain in a while, but the rain washes everyone clean.
Power builds as you spend it: a full kitchen meeting shows the street
you're somebody, and the lord's estimate of you lags behind the truth -
a stranger he squeezes, a power he negotiates, a force his collector
passes by. And every few weeks a pedlar comes up the road - pots, pins,
and word of the next village - pitches his stall of oddities at the
market, trades gossip both ways, and moves on. The perfumer keeps the
still too - marigold and lavender under it, scent out at twenty gold a
flask - and a dab of it smells of roses for days, no smell at all to a
suitor's nose. Trade with the road loses a little to the reckoning -
the outside counts in figures the village doesn't keep - until enough
bad tallies and a clerk steps up to keep them straight. And now and
then something rides over the walls - a comet, an eclipse - and the
whole village looks up, and each reads it by what they carry: the
priest sees warning, the herb-wife a gathering-night, the blamed sees
it burn for them. And a truth sits unsaid somewhere until it ripens -
the clerk's tallies show the lord's fifth is really a quarter, the old
cures work doctrine or no - and when it comes out, the power it
threatens answers: a strong org stands with its own, a weak one leaves
the discoverer to the ladder. And every year or so a circus camps on
the meadow by the gate - wonders and light fingers in the crowd - and
the bored and the unbelonging feel the pull; when the tents fold the
loneliest may go with them. And every season a bailiff's writ comes
down - a levy of grain, a fee in gold, or a summons for one of the
village's own. An org worth the name bargains it down; a strong one
stands against a summons - and pays for it when the next writ comes
double. And every other winter runs long: the snow keeps falling, the
stores thin, and the village either pulls together or pulls apart - an
org worth the name shares what it has, and a weak one goes looking for
who to blame. The herb-wife's hut looks very different in a hard
winter. The painter keeps the paint shop too - pigment and wood in,
color on the halls - and a painted hall is a power claim the street
can see, bright for a season until it fades and the work comes round
again. And now and then a sure pair of hands lays down a thing made to
outlast them - a lute, a window, a fine toolset - and it gathers
stories the way a person does, and passes to kin when the hands that
held it stop. The literate put word to those they think of - a letter
travels a week on the road and warms the bond it lands on - and the
ones who can't read pay the priest a penny to hear it. And a few carry
a difference - a lame leg, failing sight, a mind that needs quiet - the
village walks a little slower for them, and theirs check on them. And
the river gives: the fisher keeps the fish-smoke house down at the
water's edge - a catch for the smoke, thin in the ice, and now and then
the water brings up a pale stone, faintly lit, and the tale gets
around. And at the map's edge there's a cave, and in it a hermit - the
dark keeps them, they say, and the best mushrooms grow in the under.
Walk out and ask, and they'll tell you true: the pale one's the real
thing, where the light dies. The village has a voice with a flourish
on it - by Saint Crispin, a pox on this weather, the talk turns and
they call a body a knave and no mistake. And at the dark end of it
all: defy the castle a third time and the bailiff comes back with
knights - a hanging where all can see - and the grave they make of it
keeps a vigil no lord can stamp out. When the organizer's hands stop,
the work goes on - the member who knew them best takes it up. And there
are stories you can walk into mid-telling: the fever year, the circus
winter, the widow's shop. For the ones who tune the sim, the village
reads its own innards: trail heat, people power, the land's score. And
underneath it all the walks remember themselves - a path's the same
all day, so the BFS only pays once. And the whole village compiles to
wasm now - the same sim, one build away from a browser tab. And it has
a voice you can almost hear: the rain on the thatch, the smithy's
rhythm, the festival hum, a hush when something rides the sky - the
cues rise with the days for the audio layer to answer. And a summons
doesn't take you for good now - it takes you to the castle, and the
org has a month to pay the ransom before the ladder climbs to its top.

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
- Hungry villagers forage the wood's caps: good ones fill the belly, the
  little ones bring visions, and the pale lookalikes kill days later.
  **The caps' names and traits are story, not a foraging guide - never
  eat a wild mushroom on a game's say-so.**

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
cargo run --bin balance_test 400 42  # balance test: 400 days, seed 42, exit 1 on red flags
```

### Video recordings

The recorder plays the village headless - at computer speed, not watch
speed - and writes what the window would show to a video, for reviewing
test runs and long stretches of village life:

```bash
cargo run --release --bin record_video -- --play-time 60 --speed 2x --format mp4 --audio --out evidence/videos/minute.mp4
```

- `--speed realtime|1x|2x|5x|10x|60x` compresses play time to watch time
  (`--speedup N` for arbitrary values)
- `--format gif|apng|avi|mp4|webm|frames`
  - `gif`/`apng`/`frames` need no external tools (no sound)
  - `avi` is MJPEG+PCM written in pure Rust - the game's sfx cues
    synthesised into a soundtrack, no external tools
  - `mp4`/`webm`/`ffmpeg` pipe raw frames to an ffmpeg binary - best
    quality and size; `scripts/get-ffmpeg.ps1` fetches one into
    `.tools/ffmpeg/` (needs node, no admin), then pass `--ffmpeg-path`
  - `--audio` adds the synthesised soundtrack to mp4/webm
- `--script FILE` drives the player: `<frame> <verb> [args]` lines
  (`walk`, `stop`, `act`, `ctx`, `select`, `click`, `look`, `pause`,
  `speed`, `cancel`) - see `scripts/record-walkabout.txt`
- `--scale 0.5`, `--fps 12`, `--cam X,Y`, `--zoom F`, `--seed N`,
  `--game-speed 4` tune it further

## Research & Documentation

- `wiki/backlog.md` - research/mechanics backlog
- `wiki/project-state.md` - full inventory
- `wiki/evidence-index.md` - all software output catalog
- `wiki/literature-reviews/` - research (20 reviews)
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
