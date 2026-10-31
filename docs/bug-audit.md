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




- [ ] **W-005** `dungeon::found_release_campaign` is never called — no release campaign is ever founded; `ReleaseOf`/`ReleaseCampaign`-by-founding are dead state.
- [ ] **W-006** `dungeon::join` is never called — "the sympathetic sign on without an invite" can't happen; release efforts only grow by Petition clicks.
- [ ] **W-007** `campaign::set_cadence` is never called outside tests — meeting cadence is frozen at founding; the `due` flag can't be influenced.
- [ ] **W-008** `AgentCmd::Eat` may never actually eat — documented in `docs/multiplayer-round2.md` ("paths to the market but never explicitly eats"); verify hunger resolves for driven agents.
- [ ] **W-009** `game2-web` `WasmGame` exposes no input — `tick`/`scene`/`day`/`telemetry` only; no `command()` binding, so the browser client can only watch, never play.
- [ ] **W-010** `drive_agent` can't satisfy its own doc comment — only Eat/Idle/WorkDay; never `Gather`/`BuyFrom` for bench inputs, never `TalkTo` despite `near` in the obs.
- [ ] **W-011** `blocked: [Option<String>; 5]` covers only the five main buttons — ctx buttons (Meet-not-due, Delegate-nonmember, Petition with no prisoner) fail silently into the log instead of showing a disabled reason.
- [ ] **W-012** Duplicated comment block in `update_ctx_actions_at` (~line 6154, "the cause meets on its cadence…" twice) — merge artifact; check whether logic beside it was duplicated too.

## P1 — recently added features, dead on arrival

- [ ] **W-013** `actions::rally_driver` is never called — rally has no driver or entry point.
- [ ] **W-014** `organizing::village_concerns` is never called — the village's concerns are never aggregated for speech/meeting content.
- [ ] **W-015** `organizing::concern_speech` is never called — heard concerns never surface in what people say.
- [ ] **W-016** `organizing::meeting_sayings` is never called — meeting chatter text is never generated.
- [ ] **W-017** `campaign::campaign_with` is never called — no way to look up a campaign by issue.
- [ ] **W-018** `campaign::foe_backed` is never called — CommonFoe framing never checks whether anyone backs the named foe.
- [ ] **W-019** `campaign::of_the_spirit` is never called.
- [ ] **W-020** `campaign::reveals_of` is never called.
- [ ] **W-021** `campaign::framing_suits` is never called — no validation that a framing suits the issue.
- [ ] **W-022** `campaign::grumble_line` is never called — dissenters' grumbles are computed but never rendered anywhere.
- [ ] **W-023** `campaign::stake_of` is never called — `stake_for` is used, the `stake_of` variant is dead.
- [ ] **W-024** `campaign::vote_for` is only used inside `hold_issue_vote` — either make it private or wire member-level ballots into the UI.
- [ ] **W-025** `player::recruit_trust` is never called — recruiting never consults personal trust.
- [ ] **W-026** `desires::housed` is never called.
- [ ] **W-027** `org_skills::role_skill` is never called — seat-holders' competence at their role is never measured.
- [ ] **W-028** `fire::on_fire`/`is_scorched` are never called — `session.rs` reads `fire.burning`/`fire.scorched` maps directly; two sources of truth for fire state.
- [ ] **W-029** `organizing::one_on_one`/`harvest_together` are only reachable from the text game/`sim::community` — the GUI Talk path never builds ties through them (verify intended).
- [ ] **W-030** Posted-guard positions come from `player_pos_at`, not the demo site — and `self.guards` is per-Game not per-org, so two seats' crackdowns draw knights at the wrong places (MP).
- [ ] **W-031** `CtxAct::Petition` is gated behind `campaign_of().is_some()` — you can't petition for a prisoner when the org's campaign has lapsed or been spent.
- [ ] **W-032** `Sympathized` entries never decay — a moral shock marks a villager woke forever.
- [ ] **W-033** `Imprisoned` has no natural release path — only Petition frees; if the org lapses or dies it's a life sentence.
- [ ] **W-034** `actions::Demoralized` never expires on its own — a hollow member stays hollow until the organizer Talks to them; check intended vs stuck.
- [ ] **W-035** `NAMES[i]` indexing assumes `villagers.len() <= 31` — any spawn path that pushes past 31 villagers (migrants, births) panics on name lookup.
- [ ] **W-036** `action_menu` (text game) has none of the new ctx verbs — no Meet/March/Strike/Townhall/Petition/Delegate/Matchmake/PairUp/Errand; text UI and GUI have drifted apart.
- [ ] **W-037** `player::market_day` only fires in `bin/game.rs` — the market-day purse event has no GUI/session equivalent.
- [ ] **W-038** `balance_test` driver never exercises March/Petition/Delegate/etc. — balance runs can't see the newest verbs' effects.
- [ ] **W-039** Traveller/minstrel fire-fleeing — #203 claimed "travelers flee too"; verify the traveller entity actually flees burning tiles.
- [ ] **W-040** `next_minstrel` scheduling vs per-seat tick drift — verify the minstrel cadence is a sim tick, not wall/frame.

## P2 — whole subsystems implemented but never invoked

- [ ] **W-041** `sim::epidemic` `spawn_epidemic`/`step_epidemic` never wired — no epidemic loop despite the illness system.
- [ ] **W-042** `sim::gossip` `spawn_rumor`/`spread_rumor` never wired — rumours can't start or travel.
- [ ] **W-043** `sim::stratification` `infer_class`/`spawn_position`/`update_status` never wired — class never updates.
- [ ] **W-044** `sim::world_events` `spawn_event`/`should_fire`/`event_effect` never wired — no world events ever fire.
- [ ] **W-045** `sim::economics` `base_price`/`set_base_price` never consulted — market prices are hardcoded elsewhere (verify which wins).
- [ ] **W-046** `sim::reputation_qsp` `spawn_reputation`/`transition_tick` never stepped — reputation QSP is dormant.
- [ ] **W-047** `sim::fundamentalism` `fundamentalism_risk`/`is_fundamentalist`/`fundamentalist_effects` never wired.
- [ ] **W-048** `sim::illness::apply_treatment` never called — no illness is ever treated in the live sim.
- [ ] **W-049** `herbology` `knows_cure`/`treats_illness`/`herb_tree` never called — the apothecary never cures a patient.
- [ ] **W-050** `jobs` `work_tick`/`sells_goods` never called — the jobs system never ticks (verify whether vocation/livelihood replaced it and jobs.rs should go).
- [ ] **W-051** `lifecycle::spawn_person` never called — nobody is ever born; `KidGrowth` only ages existing kids, so the village can only shrink.
- [ ] **W-052** `migration` `spawn_migrants`/`integrate`/`migration_effects` never called — no newcomers ever arrive (pilgrim-settling is text-game only).
- [ ] **W-053** `succession` `spawn_lineage`/`succession_crisis` never called — a succession crisis can never trigger.
- [ ] **W-054** `trade_routes` `route_should_form`/`spawn_route`/`spawn_caravan`/`trade_tick` never called — no caravans or routes ever form.
- [ ] **W-055** `warfare` `spawn_conflict`/`conflict_effects` never called — conflicts can't start or resolve.
- [ ] **W-056** `military::army_power` never called (`resolve_battle` has a single caller) — battles can't be fought or lost.
- [ ] **W-057** `tech_tree::can_research` never called — research is never gated; `next_researchable` has one caller, verify it's live.
- [ ] **W-058** `world::spawn_world` never called — an alternate world-gen path abandoned in place.
- [ ] **W-059** `terrain::spawn_terrain` never called — `Terrain::generate_country` won; remove or wire.
- [ ] **W-060** `factions::spawn_faction` never called — factions can never spawn.
- [ ] **W-061** `traits::spawn_personality` never called — `personality_from_seed` is used instead; two personality paths coexist.
- [ ] **W-062** `tea` `spawn_tea`/`drink_tea` never called — tea exists only in its module.
- [ ] **W-063** `textiles` `spawn_textile`/`comfort_level` never called.
- [ ] **W-064** `livestock::centroid` never called — flock centre is never computed; check the fold's drift/graze logic.
- [ ] **W-065** `household::stock_room` never called.
- [ ] **W-066** `livelihood::has_role` never called.
- [ ] **W-067** `vocation` `is_service`/`options_for`/`skill_gain` never called — skills may never grow through the intended path.
- [ ] **W-068** `recipes` `can_make`/`printmaking_recipes` never called — the recipe data is bypassed by hardcoded livelihood checks.
- [ ] **W-069** `ai_player::choose_action` never called — no autonomous rival/ruler ever acts.
- [ ] **W-070** `music` `infer_mood`/`target_layers` never called — the soundtrack can't follow game state (`recommend_mood`/`describe`/`spawn_music` have one caller each; verify it's live).
- [ ] **W-071** `economy::run_cycle` never called — the production-chain cycle never runs in a live session.
- [ ] **W-072** `sanctions::blame_worthy` never called.
- [ ] **W-073** `relationships::infer_kind` never called — relationship kinds are never re-inferred from interactions.
- [ ] **W-074** `pets::pet_mood_boost` never called — a bonded pet gives its owner nothing.
- [ ] **W-075** `tilemap::smooth_noise` never called.
- [ ] **W-076** `trails::movement_cost` never called — worn trails don't make walking cheaper.
- [ ] **W-077** `community::exert_influence` never called — org influence is never spent through the community module.
- [ ] **W-078** `climate::rain_effect` never called — rain has no effect on crops/soil through the intended hook.
- [ ] **W-079** `weather::health_effect` never called — weather doesn't touch health.
- [ ] **W-080** `contracts` `run`/`setup` never called — the contract engine never runs; `seal_aid_pact`/`breach` are text-game-only paths.

## P3 — dead infra/ui code and duplicated drift-risk paths

- [ ] **W-081** `infra::agent_kb` is unwired — the whole KB module (append/read/write/list/exists/with_root) has no consumer.
- [ ] **W-082** `infra::pr_flow` is unwired — create/merge/review PR helpers with no consumer.
- [ ] **W-083** `infra::task_queue` is unwired — eight queue functions, no consumer.
- [ ] **W-084** `infra::worktree` is unwired — create/list/remove/detect_conflicts with no consumer.
- [ ] **W-085** `infra::ollama` is unwired — execute_task/list_models/extract helpers with no consumer.
- [ ] **W-086** `infra::hot_reload::reload` never called — the hot-reload hook is dead.
- [ ] **W-087** `infra::save_load` `checkpoint`/`replay_from` have a single caller each — the game never saves; world dies with the process.
- [ ] **W-088** `ui::dialogs::generate_line` never called.
- [ ] **W-089** `ui::font::draw_char` never called — `draw_text*` is used instead; delete or wire.
- [ ] **W-090** Divergent spawn paths: `new_seeded` vs `new_scenario` vs `action_menu`'s own world setup — new spawn-time components can land in one constructor and silently miss the others (audit all three for the newest components).

## P4 — agent/network surface gaps

- [ ] **W-091** `AgentObs` carries no fire/burning field — driven agents can walk into flames.
- [ ] **W-092** `AgentObs` carries no org/campaign/membership fields — bots can't see the political layer at all.
- [ ] **W-093** `AgentCmd` has no organizing verbs — a bot can never found an org, join, meet, march, or petition; agents can't reach the win state.
- [ ] **W-094** `free_villager` assigns by index, not role — bots can claim the priest or the tax collector (documented in `docs/multiplayer-round2.md`).
- [ ] **W-095** `game_bots` sends `token: None` every run — bots can't reclaim their villager across restarts (documented).
- [ ] **W-096** Scene bandwidth: the whole `UiModel` (incl. `TileMap`) serializes per broadcast per seat (documented).
- [ ] **W-097** No scripted multiplayer regression test — server + human + bot round-trip is unverified in CI (documented).

## P5 — repo hygiene

- [ ] **W-098** `stash@{0}` ("WIP on fix/visual-flags") is already-merged work — drop the stash.
- [ ] **W-099** ~140 merged remote branches never pruned — `git remote prune origin` + delete stale local branches.
- [ ] **W-100** `docs/multiplayer-round2.md` lists already-fixed gaps (sync_purse seats, building inspector seat) as open — stale doc; `tasks/todo.txt` + `tasks/manager.log` are stale committed artifacts.
