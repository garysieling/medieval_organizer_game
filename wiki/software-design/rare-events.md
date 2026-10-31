# Rare Events

The hundred small fortunes and misfortunes that wander through a village's
year. Each is **semi-rare**: it arrives on its own cadence window, plays out
over days, and is gone again - with another due far down the road. They sit
on top of the ordinary sim (hunger, work, taxes) the way the minstrel, the
pedlar and the circus already do.

## Shapes

Every event has one of three shapes:

- **Visitor** - a figure walks in on `traveller_path`, stands its `stay` at a
  pitch (usually the market), works a daily effect, then walks out. Drawn as
  a Traveller (or Pet/Sheep where it fits - graphics are placeholder until
  the new images land) with a name label over it.
- **Spell** - no figure; `stay` days of an ambient effect (a hailstorm's day,
  a rat plague's fortnight). Surfaced through the log, weather/sky fields
  where they fit, and bubbles.
- **Moment** - happens on the day: applied once, logged, done.

## Machinery (`game2-core/src/session/events.rs`)

- `EventId` - the hundred ids; `EventSpec { id, name, kind, gap, span, stay }`
  - `EVENTS` is the registry; the next due day for each id lives in
  `Game.next_rare`, rescheduled `tick + gap + rng % span` after it fires.
- `Game.rare: Vec<RareEvent>` - what's afoot now (entity + path + until).
- `events_tick(tick)` (daily): spawns what's due, runs each active event's
  day, walks out what's done. `advance_rare()` (per-frame): the shared walk.
- `force_event(id)` - fires an event now, for tests and the staging harness.
- Unimplemented ids ride a generic baseline (a figure who walks in, says his
  piece, mills about, leaves) until their own PR lands the real effect.

## The test harness

- `session/stage.rs` - `Stage` (Ordinary, Fever, Famine, HardWinter,
  StrongOrg, Unrest, Flush, Bereft), `staged()`, `run_days()`, `force()`.
- `session/probe.rs` - `ProbeView`: the **secret** API. Everything the sim
  hides from players - the unsaid discovery, which mushroom is death, the
  org's hidden strength, every inner life, the event schedule. Exposed:
  - `Game::probe()` in-process (tests)
  - `game_server --probe-port N` - a second, read-only websocket listener.
    Player/agent connections never see it; `AgentObs` carries none of it.
  - `WasmGame::probe()` - separate from `scene()`.
- `game2-core/tests/rare_events.rs` - the sweep: every event x several
  stages, force it, run weeks of turns, assert the village still works.

## The hundred

### Folk on the road (50)

| # | id | name | shape | effect sketch |
|---|----|------|-------|----------------|
| 1 | Tinker | the tinker | Visitor | mends pots: BetterTools satisfied, coin changes hands |
| 2 | KnifeGrinder | the knife-grinder | Visitor | sharpens edges: BetterTools for the trades |
| 3 | RagAndBone | the rag-and-bone man | Visitor | buys rags: a coin to the poorest |
| 4 | ChimneySweep | the sweep | Visitor | flues cleaned: fire risk dips after |
| 5 | RatCatcher | the rat-catcher | Visitor | rats gone: spoilage dips |
| 6 | Dowser | the water-diviner | Visitor | wanders the meadow; may find a spring |
| 7 | Scribe | the scribe | Visitor | writes letters for the illiterate (letters fly) |
| 8 | ToothDrawer | the tooth-drawer | Visitor | pulls a bad tooth: one villager's health bumps |
| 9 | BarberSurgeon | the barber-surgeon | Visitor | bloodletting: small health bumps, small chance of harm |
| 10 | PilgrimBand | a band of pilgrims | Visitor | pray at the church: Faith satisfied |
| 11 | RelicSeller | the relic-seller | Visitor | a dubious relic: Faith for the devout, the priest grumbles |
| 12 | Pardoner | the pardoner | Visitor | sells indulgences: purses lighter, the priest objects |
| 13 | Juggler | a juggler | Visitor | tumbles at market: Joy for watchers |
| 14 | BearWard | a bear-ward | Visitor | the bear dances: Joy, purses lighter |
| 15 | Puppeteer | a puppeteer | Visitor | a mystery play: Faith + Joy |
| 16 | BalladMonger | the ballad-monger | Visitor | printed ballads: a secret sung aloud |
| 17 | NewsCrier | the news-crier | Visitor | word from the castle: mood + rumor seed |
| 18 | Recruiter | the recruiting sergeant | Visitor | young men Tempted to the wars; one may go |
| 19 | Deserter | a deserter | Visitor | hides out back: feeds him or doesn't (grievance) |
| 20 | RefugeeFamily | a refugee family | Visitor | begs at the gate; may settle (new villager) |
| 21 | LeperBand | a leper band | Visitor | begs outside the walls: charity vs grievance |
| 22 | Friar | a wandering friar | Visitor | preaches: devotion bump, grumbles at the ale |
| 23 | Heretic | a whispering heretic | Visitor | doubt spreads: the faithful shaken |
| 24 | Drover | a drover | Visitor | sheep to market: buys fodder, leaves muck |
| 25 | FishHawker | a fish-hawker | Visitor | salt fish: food for buyers |
| 26 | SaltSeller | the salt-seller | Visitor | salt into the market stock |
| 27 | WoolMerchant | the wool merchant | Visitor | buys fleece: the fold's keeper gains |
| 28 | WineMerchant | the wine merchant | Visitor | sells wine: Joy, purses lighter |
| 29 | SpiceMerchant | the spice merchant | Visitor | pepper + cures: the herb-wife eyes his stock |
| 30 | Bookseller | the bookseller | Visitor | a book sold: literate joy |
| 31 | Glazier | the glazier | Visitor | a church window: a small lasting joy |
| 32 | Thatcher | the thatcher | Visitor | roofs patched: Shelter satisfied |
| 33 | WellDigger | the well-digger | Visitor | deepens the well: water blessing |
| 34 | CharcoalBurner | a charcoal burner | Visitor | camps in the wood: canopy dips, forge stock up |
| 35 | LimeBurner | the lime-burner | Visitor | the kiln smokes: fields limed, food bump |
| 36 | Tanner | the tanner | Visitor | buys hides: keepers gain, a stink noted |
| 37 | CandleMaker | the candle-maker | Visitor | buys wax: the apiary gains |
| 38 | Potter | the potter | Visitor | sells pots: small comfort |
| 39 | HorseCop | the horse-coper | Visitor | horse talk: a purse shuffles, a laugh |
| 40 | Eggler | the eggler | Visitor | buys eggs: hen-keepers gain |
| 41 | CheeseMonger | the cheese-monger | Visitor | buys aged wheels: the cellar pays out |
| 42 | FortuneTeller | the fortune-teller | Visitor | reads palms: hope or dread in bubbles |
| 43 | Astrologer | the astrologer | Visitor | casts the village's stars: omen talk |
| 44 | Alchemist | the alchemist | Visitor | buys oddities; his pot may flare |
| 45 | Quack | the quack | Visitor | cure-alls: some believe, the herb-wife scowls |
| 46 | Midwife | the midwife | Visitor | passes through: the next birth safer |
| 47 | Sowgelder | the pig man | Visitor | tends the beasts: flock health tick |
| 48 | Farrier | the farrier | Visitor | shoes the horses: a day's ring of iron |
| 49 | CornFactor | the corn-factor | Visitor | buys surplus grain: purses gain, stock down |
| 50 | Messenger | a messenger | Visitor | a letter for someone (letters system) |

### The sky and the weather (15)

| # | id | name | shape | effect sketch |
|---|----|------|-------|----------------|
| 51 | Hailstorm | a hailstorm | Spell | gardens/crops hurt; everyone stung a little |
| 52 | HeatWave | a heat wave | Spell | days of thirst: tempers short, work slower |
| 53 | Drought | a drought | Spell | the pond falls; forage thin a spell |
| 54 | Flood | the river comes up | Spell | the reach fills, murk spreads |
| 55 | LightningStrike | lightning strikes | Moment | a fire kindles somewhere |
| 56 | LateFrost | a late frost | Moment | blossoms lost: forage hurt |
| 57 | IndianSummer | an Indian summer | Spell | warm autumn days: quiet joy |
| 58 | FirstSnow | the first snow | Moment | the children play: Joy |
| 59 | Thaw | the thaw | Spell | mud days: trails wear fast |
| 60 | Aurora | the aurora | Spell | night fires in the sky: omens read |
| 61 | MeteorShower | a meteor shower | Moment | wishes and dread |
| 62 | BloodMoon | a blood moon | Spell | dread bubbles; the priest prays |
| 63 | DarkNoon | a dark noon | Spell | gloom at midday: murk over the map |
| 64 | WindStorm | a windstorm | Spell | thatch torn: building damage |
| 65 | Mirage | a mirage | Moment | someone swears they saw towers |

### The beasts (15)

| # | id | name | shape | effect sketch |
|---|----|------|-------|----------------|
| 66 | WolfAtFold | a wolf at the fold | Moment | stock lost, or the shepherd stands |
| 67 | FoxInYard | a fox in the yard | Moment | a hen lost |
| 68 | HawkStoops | a hawk stoops | Moment | a chick snatched |
| 69 | StrayDog | a stray dog | Visitor | adopted by a lonely heart (new pet) |
| 70 | StrayCat | a stray cat | Visitor | if adopted: vermin down; else drifts on |
| 71 | RatPlague | a plague of rats | Spell | spoilage spike for a spell |
| 72 | Locusts | locusts | Moment | the gardens stripped |
| 73 | CaterpillarYear | a caterpillar year | Spell | garden growth suffers |
| 74 | Starlings | a murmuration | Moment | awe + seed loss |
| 75 | BeeTree | a bee tree found | Moment | wild honey: forage bonus |
| 76 | DeerInFields | a deer in the fields | Moment | venison for the quickest |
| 77 | WildBoar | a wild boar | Spell | tramples gardens; meat if caught |
| 78 | OwlInTower | an owl in the tower | Spell | an omen; the mice thin |
| 79 | CrowsMob | a mob of crows | Moment | ill-omen talk; seed loss |
| 80 | FishRun | a fish run | Spell | the river's thick: fish stock up |

### Ill luck (10)

| # | id | name | shape | effect sketch |
|---|----|------|-------|----------------|
| 81 | ChimneyFire | a chimney fire | Moment | a home scorches (fire kindled) |
| 82 | FouledWell | a fouled well | Spell | illness risk till it's cleared |
| 83 | GranaryLeak | a granary leak | Moment | grain stock lost |
| 84 | CartOverturns | a cart overturns | Moment | a villager hurt |
| 85 | DrownedInPond | drowned in the pond | Moment | a rare death by the water |
| 86 | LostChild | a lost child | Spell | the village searches (found) |
| 87 | Foundling | a foundling | Moment | a babe on the church step; taken in |
| 88 | Quarrel | a quarrel in the square | Moment | a feud seeds (grievances both ways) |
| 89 | AlehouseBrawl | an alehouse brawl | Moment | a hurt + a grievance |
| 90 | OldOakFalls | the old oak falls | Moment | the landmark falls: talk of endings |

### Findings (10)

| # | id | name | shape | effect sketch |
|---|----|------|-------|----------------|
| 91 | CoinHoard | a coin hoard | Moment | a field gives up a pot of coins; the lord's claim rumored |
| 92 | OldStones | old stones | Moment | a standing stone under a field: lore talk |
| 93 | SpringBubbles | a spring bubbles up | Moment | new water: a small blessing |
| 94 | HoneyTree | a honey tree | Moment | wild honey for the finding |
| 95 | Truffle | a truffle | Moment | a pig roots one out: sells dear |
| 96 | HerbFlush | a flush of herbs | Spell | the herb patches thicken |
| 97 | BerryYear | a berry year | Spell | forage good; food up a spell |
| 98 | MushroomFlush | a mushroom flush | Spell | extra patches - one poison |
| 99 | StormWashUp | a storm's leavings | Moment | the shore gives up driftwood |
| 100 | ProdigalSon | a prodigal returns | Visitor | a migrant's child comes home with coin |
