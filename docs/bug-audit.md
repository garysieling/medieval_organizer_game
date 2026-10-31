# Wiring bug audit — candidate list

A couple days of fast merges left systems implemented in `game2-core/src/game/`,
`src/sim/`, `src/infra/` that the session (`session.rs`), the render model
(`session/model.rs`), the agent surface (`session/net.rs`), the text game
(`game/action_menu.rs` + `bin/game.rs`) and the wasm client (`game2-web`) may
not have wired up in unison. Method: `pub fn` usage census across the crate +
manual reads of the recent diffs.

Each item is a **candidate** — when its ticket is worked, verify first: the
"dead" function may be an intentional library seam (fix = make it private),
superseded code (fix = delete), or a genuine missing wire (fix = wire it).
Items are removed from this list as their PRs land.

## P0 — live gameplay bugs in recently landed features





## P1 — recently added features, dead on arrival

- [x] ~~**W-023** `campaign::stake_of` is never called — `stake_for` is used, the `stake_of` variant is dead.~~ (fixed — deleted; nothing needed the top stake)
- [x] ~~**W-028** `fire::on_fire`/`is_scorched` are never called — `session.rs` reads `fire.burning`/`fire.scorched` maps directly; two sources of truth for fire state.~~ (fixed — `fire.burns_near` added; session asks the fire)
- [x] ~~**W-032** `Sympathized` entries never decay — a moral shock marks a villager woke forever.~~ (fixed)
- [x] ~~**W-033** `Imprisoned` has no natural release path — only Petition frees; if the org lapses or dies it's a life sentence.~~ (fixed — pressure gathers a little every day; a prisoner walks alone in ~50 days)
- [x] ~~**W-034** `actions::Demoralized` never expires on its own — a hollow member stays hollow until the organizer Talks to them; check intended vs stuck.~~ (fixed)
- [x] ~~**W-037** `player::market_day` only fires in `bin/game.rs` — the market-day purse event has no GUI/session equivalent.~~ (fixed — `livelihood_tick` pays each seat's purse monthly per living org member)
- [x] ~~**W-039** Traveller/minstrel fire-fleeing — #203 claimed "travelers flee too"; verify the traveller entity actually flees burning tiles.~~ (fixed — `fire_at_feet` turns minstrel/pedlar/circus/ink-boiler for the road when flames are near)
- [x] ~~**W-040** `next_minstrel` scheduling vs per-seat tick drift — verify the minstrel cadence is a sim tick, not wall/frame.~~ (verified — `next_minstrel` is set from `self.tick`, a sim day, shared across seats)

## P2 — whole subsystems implemented but never invoked

- [x] ~~**W-045** `sim::economics` `base_price`/`set_base_price` never consulted — market prices are hardcoded elsewhere (verify which wins).~~ (verified — `vocation::price()` wins in the session; `Market::base_price` runs in the village_sim market)
- [x] ~~**W-048** `sim::illness::apply_treatment` never called — no illness is ever treated in the live sim.~~ (partially stale — live curing runs via `herbology::apply_efficacy`; dead `apply_treatment` deleted)
- [x] ~~**W-049** `herbology` `knows_cure`/`treats_illness`/`herb_tree` never called — the apothecary never cures a patient.~~ (stale — all wired transitively: `knows_cure`←`brew_from_stock`, `treats_illness`/`herb_tree`←`medicinal_herbs`, used by the SeekRemedy path)
- [x] ~~**W-050** `jobs` `work_tick`/`sells_goods` never called — the jobs system never ticks (verify whether vocation/livelihood replaced it and jobs.rs should go).~~ (stale — both run inside `JobSystem`, scheduled by the `game_loop` driver)
- [x] ~~**W-053** `succession` `spawn_lineage`/`succession_crisis` never called — a succession crisis can never trigger.~~ (fixed — orphaned `succession` module deleted; the session's own succession_tick handles the living one)
- [x] ~~**W-054** `trade_routes` `route_should_form`/`spawn_route`/`spawn_caravan`/`trade_tick` never called — no caravans or routes ever form.~~ (fixed — orphaned `trade_routes` module deleted; no routes ever formed)
- [x] ~~**W-055** `warfare` `spawn_conflict`/`conflict_effects` never called — conflicts can't start or resolve.~~ (fixed — orphaned `warfare` module deleted; no conflicts ever started)
- [x] ~~**W-056** `military::army_power` never called (`resolve_battle` has a single caller) — battles can't be fought or lost.~~ (fixed — orphaned `military` module deleted with `resolve_battle`'s only caller)
- [x] ~~**W-057** `tech_tree::can_research` never called — research is never gated; `next_researchable` has one caller, verify it's live.~~ (stale — `can_research` runs via `research`/`next_researchable`, exercised by the demo bins and the tech view)

- [x] ~~**W-058** `world::spawn_world` never called — an alternate world-gen path abandoned in place.~~ (fixed — deleted the abandoned `spawn_world` path)
- [x] ~~**W-059** `terrain::spawn_terrain` never called — `Terrain::generate_country` won; remove or wire.~~ (fixed — `spawn_terrain` deleted; `Terrain::generate_country` is the one path)
- [x] ~~**W-060** `factions::spawn_faction` never called — factions can never spawn.~~ (fixed — orphaned `factions` module deleted; nothing ever constructed a Faction)
- [x] ~~**W-061** `traits::spawn_personality` never called — `personality_from_seed` is used instead; two personality paths coexist.~~ (fixed — `spawn_personality` deleted; `personality_from_seed` is the one path)
- [x] ~~**W-062** `tea` `spawn_tea`/`drink_tea` never called — tea exists only in its module.~~ (fixed — orphaned `tea` module deleted; nothing consumed it)
- [x] ~~**W-063** `textiles` `spawn_textile`/`comfort_level` never called.~~ (fixed — orphaned `textiles` module deleted; nothing consumed it)
- [x] ~~**W-064** `livestock::centroid` never called — flock centre is never computed; check the fold's drift/graze logic.~~ (fixed — the graze point centres on the flock)
- [x] ~~**W-065** `household::stock_room` never called.~~ (stale — called by `add_stock` at household.rs:80)
- [x] ~~**W-066** `livelihood::has_role` never called.~~ (fixed — answers the two Herbologist checks in session.rs)
- [x] ~~**W-067** `vocation` `is_service`/`options_for`/`skill_gain` never called — skills may never grow through the intended path.~~ (partially stale — `skill_gain` runs via `train_day` in `livelihood`; deleted dead `is_service` (dup of `is_pmc`) and `options_for`)
- [x] ~~**W-068** `recipes` `can_make`/`printmaking_recipes` never called — the recipe data is bypassed by hardcoded livelihood checks.~~ (verified — the recipe table is a dormant alternative model; livelihood's own craft checks cover it in the live session)
- [x] ~~**W-070** `music` `infer_mood`/`target_layers` never called — the soundtrack can't follow game state (`recommend_mood`/`describe`/`spawn_music` have one caller each; verify it's live).~~ (partially stale — `infer_mood`/`target_layers` DO run inside `evolve`, driven by the `music_demo` bin; the session has no audio path for it to follow)
- [x] ~~**W-071** `economy::run_cycle` never called — the production-chain cycle never runs in a live session.~~ (stale — runs inside `ProductionSystem`, scheduled by the `village_sim` driver; the session's own livelihood model supersedes it)
- [x] ~~**W-072** `sanctions::blame_worthy` never called.~~ (fixed — deleted; the blame path adds `Problem::Blamed` directly)
- [x] ~~**W-073** `relationships::infer_kind` never called — relationship kinds are never re-inferred from interactions.~~ (stale — `record_interaction` re-infers on every interaction; only the decay part of RelationshipSystem is unscheduled)
- [x] ~~**W-074** `pets::pet_mood_boost` never called — a bonded pet gives its owner nothing.~~ (fixed — `comforting_pet` now comforts in `care_tick`)
- [x] ~~**W-075** `tilemap::smooth_noise` never called.~~ (stale — called by terrain gen at tilemap.rs:220)
- [x] ~~**W-076** `trails::movement_cost` never called — worn trails don't make walking cheaper.~~ (fixed)
- [x] ~~**W-077** `community::exert_influence` never called — org influence is never spent through the community module.~~ (stale — the session spends through `OrganizedPower`; `InfluenceSystem`/`exert_influence` is the bin-scoped community API)
- [x] ~~**W-078** `climate::rain_effect` never called — rain has no effect on crops/soil through the intended hook.~~ (stale — `rain_effect` runs via `climate_multiplier`, which the demo/sim bins consult for food)
- [x] ~~**W-079** `weather::health_effect` never called — weather doesn't touch health.~~ (fixed — `full_session` applies each active weather's health bite to every Health each tick)
- [x] ~~**W-080** `contracts` `run`/`setup` never called — the contract engine never runs; `seal_aid_pact`/`breach` are text-game-only paths.~~ (fixed — `ContractSystem` scheduled in the contracts_demo driver; a fulfilled contract now credits reputation live)

## P3 — dead infra/ui code and duplicated drift-risk paths

- [x] ~~**W-081** `infra::agent_kb` is unwired — the whole KB module (append/read/write/list/exists/with_root) has no consumer.~~ (dormant by design — agent-pipeline scaffolding with no in-crate consumer; not a gameplay bug)
- [x] ~~**W-082** `infra::pr_flow` is unwired — create/merge/review PR helpers with no consumer.~~ (dormant by design — PR-flow helpers scaffold the agent loop, no in-crate consumer)
- [x] ~~**W-083** `infra::task_queue` is unwired — eight queue functions, no consumer.~~ (dormant by design — task-queue helpers scaffold the agent loop, no in-crate consumer)
- [x] ~~**W-084** `infra::worktree` is unwired — create/list/remove/detect_conflicts with no consumer.~~ (dormant by design — worktree helpers scaffold the agent loop, no in-crate consumer)
- [x] ~~**W-085** `infra::ollama` is unwired — execute_task/list_models/extract helpers with no consumer.~~ (stale — `OllamaClient` is used by the `creativity` bin)
- [x] ~~**W-086** `infra::hot_reload::reload` never called — the hot-reload hook is dead.~~ (dormant by design — the reload hook is scaffold for a dev pipeline, never claimed by the game)
- [x] ~~**W-087** `infra::save_load` `checkpoint`/`replay_from` have a single caller each — the game never saves; world dies with the process.~~ (partially stale — `History`/`checkpoint`/`replay_from` run in the `save_load_demo` bin; the live session still never checkpoints, which is a feature gap not a dead API)
- [x] ~~**W-088** `ui::dialogs::generate_line` never called.~~ (fixed — deleted `ui::dialogs`; the Dialog seam was never constructed)
- [x] ~~**W-089** `ui::font::draw_char` never called — `draw_text*` is used instead; delete or wire.~~ (stale — called by `draw_text` at font.rs:133)
- [x] ~~**W-090** Divergent spawn paths: `new_seeded` vs `new_scenario` vs `action_menu`'s own world setup — new spawn-time components can land in one constructor and silently miss the others (audit all three for the newest components).~~ (verified — `new_scenario` delegates to `new_seeded` so parity is automatic; `action_menu` is a separate text-game world by design)

## P4 — agent/network surface gaps

- [x] ~~**W-091** `AgentObs` carries no fire/burning field — driven agents can walk into flames.~~ (fixed)
- [x] ~~**W-092** `AgentObs` carries no org/campaign/membership fields — bots can't see the political layer at all.~~ (fixed)
- [x] ~~**W-093** `AgentCmd` has no organizing verbs — a bot can never found an org, join, meet, march, or petition; agents can't reach the win state.~~ (fixed — `JoinOrg` signs on a woken or self-interested villager; `Press` writes letters for a prisoner)
- [x] ~~**W-094** `free_villager` assigns by index, not role — bots can claim the priest or the tax collector (documented in `docs/multiplayer-round2.md`).~~ (fixed)
- [x] ~~**W-095** `game_bots` sends `token: None` every run — bots can't reclaim their villager across restarts (documented).~~ (fixed)
- [ ] **W-096** Scene bandwidth: the whole `UiModel` (incl. `TileMap`) serializes per broadcast per seat (documented).
- [ ] **W-097** No scripted multiplayer regression test — server + human + bot round-trip is unverified in CI (documented).

## P5 — repo hygiene

- [x] ~~**W-098** `stash@{0}` ("WIP on fix/visual-flags") is already-merged work — drop the stash.~~ (fixed — both stashes dropped; their content is in main)
- [x] ~~**W-099** ~140 merged remote branches never pruned — `git remote prune origin` + delete stale local branches.~~ (fixed — 162 merged remote branches deleted, ~60 stale worktrees removed, locals pruned)
- [x] ~~**W-100** `docs/multiplayer-round2.md` lists already-fixed gaps (sync_purse seats, building inspector seat) as open — stale doc; `tasks/todo.txt` + `tasks/manager.log` are stale committed artifacts.~~ (fixed)
