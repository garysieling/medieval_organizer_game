# Sprint 6: "The Game Plays Itself" - Depth + Autonomy

**Theme:** The UI is built. Now make the game deep enough to be interesting + autonomous enough to run itself. Focus on simulation depth, emergent content, and the agent workforce producing real game work.

**Status:** Planned

## Workstream A: Simulation Depth (the game gets richer)

- [ ] A1: Real economy tick — market prices move from supply/demand, villagers buy/sell
- [ ] A2: Production chains live — farms produce wheat -> mill -> bakery -> bread, visible on map
- [ ] A3: Tech progresses — the village discovers techs over time, unlocking buildings
- [ ] A4: Skills/jobs — villagers specialize (farmer/miller/baker/soldier), work produces goods
- [ ] A5: Needs beyond hunger — rest, social, safety needs drive behavior
- [ ] A6: Aging + lifecycle — villagers age visibly (sprite changes), die, children born
- [ ] A7: Villager memory — they remember events (feud, kindness, betrayal)

## Workstream B: Emergent Content (the world writes itself)

- [ ] B1: Villager personalities — each villager gets traits (lazy/ambitious/kind/stingy) affecting behavior
- [ ] B2: Villager relationships — friendships/rivalries form, shown in graph
- [ ] B3: Rumor spread — gossip travels villager-to-villager about events
- [ ] B4: Festivals/events — the village holds festivals when prosperous
- [ ] B5: Villager projects — they propose/do projects (build shrine, hold feast)
- [ ] B6: Named buildings — guild hall gets a name, houses get owners

## Workstream C: More Game Systems Visible

- [ ] C1: Contracts as gameplay — offer work contracts to villagers for pay
- [ ] C2: Crime/punishment — theft, laws, justice visible (stocks, trial)
- [ ] C3: Religion visible — congregation gathers at the church building
- [ ] C4: Feuds visible — rival families marked, vengeance shown
- [ ] C5: Trade caravans — merchant caravan travels to/from map edge

## Workstream D: Player Depth

- [ ] D1: More actions — Scheme, Gather, Negotiate, Forage, Pray, Train
- [ ] D2: Skill tree — player's abilities unlock (Charisma, Cunning, Labor, Faith)
- [ ] D3: Reputation tags — player known as "kind" or "cunning" based on actions
- [ ] D4: Player home — own house the player can build/upgrade
- [ ] D5: Player equipment — tools/clothes that grant bonuses

## Workstream E: The Village Evolves

- [ ] E1: Village grows — new migrants arrive, buildings expand over time
- [ ] E2: Buildings age/upgrade — houses improve, mill upgrades
- [ ] E3: Territory expands — map grows as village prospers
- [ ] E4: Named villager roles — Mayor, Sheriff, Priest positions emerge

## Workstream F: Agent Fleet Produces Game Content

- [ ] F1: Ollama agents write game content — quest text, villager names, event descriptions
- [ ] F2: Creativity Generator generates new buildings/mechanics from ideas
- [ ] F3: Fun Inspector reviews the game + suggests improvements
- [ ] F4: Playtester runs the game + reports balance issues

## Workstream G: Persistence & Saves

- [ ] G1: Save/load the whole game state to file
- [ ] G2: Autosave every 100 ticks
- [ ] G3: Load menu to pick a save

## Workstream H: Evidence & Polish

- [ ] H1: Full play session recorded (frame sequence -> video)
- [ ] H2: Update PLAY.md + README with the real game
- [ ] H3: Screenshots of gameplay for evidence

## Exit Criteria

1. The game runs 1000+ ticks with visible life: villagers work, eat, die, feud, organize
2. Economy moves (prices, production), tech unlocks buildings
3. Village physically grows (new buildings appear, population rises)
4. Agent fleet produces real game content via Ollama
5. Save/load works; a full session is documented
