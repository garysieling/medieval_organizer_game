# Built but not in the UI: the wiring backlog

100 things the sim (or `game/`, `sim/`) has that a player can't see or use.
Found 2026-11-05 by sweeping for: `game/`/`sim/` modules and `pub fn`s nothing
in `session/` or `game2-client` calls, ECS systems never added to the
scheduler, `Game` fields `view_model.rs` never reads, and villager components
the inspector doesn't show.

One branch and PR per item (`ui/NNN-short-name`). Some items are a whole
system that has to be scheduled first; others are just a line in the
inspector or panel. New art where it helps (see `assets/rd/README.md`).
Probe-only truths (rule 3 in `architecture.md`) stay hidden: where an item is
secret state, the UI shows what the *player* could know (a rumour, a sign,
a guess), not the number.

Status: `todo` / `wip #PR` / `done #PR` / `drop (why)`.

## A. Whole systems built and never run or shown

| # | Item | Where | Status |
|---|------|-------|--------|
| 1 | Blood feuds: offense -> vengeance -> escalation -> peacemaking between families | `game/feuds.rs` (`FeudSystem`, `spawn_feud`, `peacemaking`) | done #523 |
| 2 | Contracts and aid pacts: sign, breach, consequences | `game/contracts.rs` (`ContractSystem`, `seal_aid_pact`, `breach`) | done #526 |
| 3 | Job production chain: wheat -> flour -> bread, visible work | `game/jobs.rs` (`JobSystem`, `assign_job`, `work_tick`) | done #531 |
| 4 | Village production chains | `game/economy.rs` (`ProductionSystem`, `ProductionChainSystem`, `produce`) | done #532 |
| 5 | Market prices from supply and demand | `sim/economics.rs` (`spawn_market`, `update_price`, `add_supply`) | done #533 |
| 6 | Recipes: several paths to the same good | `game/recipes.rs` (`can_make`, `printmaking_recipes`) | done #534 |
| 7 | Printmaking tech tree (feathers -> lithography) | `game/tech_tree.rs` (`research`, `next_researchable`) | done #535 |
| 8 | Music layers that evolve with the village's mood | `game/music.rs` (`infer_mood`, `evolve`, `describe`) | done #536 |
| 9 | Faith, congregations and services | `sim/religion.rs` (`Faith`, `perform_service`, `DevotionSystem`, `GatherSystem`) | done #537 |
| 10 | Weather events: storms, droughts, heat waves | `sim/weather.rs` (`WeatherSystem`, `food_effect`, `health_effect`) | done #538 |
| 11 | Climate effects on crops and health | `sim/climate.rs` (`rain_effect`, `climate_multiplier`) | done #539 |
| 12 | Story detection: name the stories the sim tells (a chronicle) | `sim/story.rs` (`detect_story`, `surface_story`) | done #540 |
| 13 | Magic read out of old texts | `sim/magic.rs` (`ingest_text`, `magic_effects`) | done #541 |
| 14 | Grass and forest spread (cellular automaton) | `sim/emergence.rs` | done #542 |
| 15 | Reputation stages: unknown -> known -> trusted -> esteemed / disgraced | `sim/reputation_qsp.rs` (`spawn_reputation`) | done #543 |
| 16 | Contract verb from the old text action menu | `game/action_menu.rs` | done #526 |
| 17 | Guilds forming and policing their trade | `game/guilds.rs` (`guild_should_form`, `form_guild`, `enforce_security`) | done #545 |
| 18 | Landforms in map generation | `game/landforms.rs` (`select_landform`, `apply_landform`) | done #546 |
| 19 | Aging and life stages | `game/lifecycle.rs` (`AgingSystem`, `LifeStage`, `spawn_person`) | done #547 |
| 20 | Grief and mourning | `sim/psychology.rs` (`GriefSystem`, `Mourned`) | done #548 |
| 21 | Attachment styles shaping bonds | `sim/psychology.rs` (`AttachmentSystem`, `AttachmentStyle`) | done #549 |
| 22 | Relationship drift from interactions | `game/relationships.rs` (`RelationshipSystem`, `infer_kind`) | done #550 |
| 23 | Influence and one-on-ones as a running system | `sim/community.rs` (`InfluenceSystem`, `OneOnOneSystem`, `exert_influence`) | done #551 |
| 24 | Organized power wasting away when unused | `sim/organized_power.rs` (`PowerAtrophySystem`) | done #552 |
| 25 | Crop fields growing through the season | `game/villager.rs` (`CropGrowthSystem`, `spawn_crop_field`, `servings_left`) | done #553 |
| 26 | Stall system | `game/villager.rs` (`StallSystem`, `spawn_stall`) | done #554 (wired by the trader's stall + pedlar pitch + `Goal::BuyFrom`; the `Market`-pull model is the text-game's) |
| 27 | Nourishment | `NourishmentSystem` | done #555 |
| 28 | Organizing skills learned by doing and taught | `game/org_skills.rs` (`practice`, `teach`, `learn_from_meeting`) | done #556 |
| 29 | Pets lifting their owner's mood | `game/pets.rs` (`pet_mood_boost`) | done #557 (wired: `comforting_pet` reads the boost; `care_tick` satisfies `Want::Joy` on a bonded pet's owner) |
| 30 | Harvesting together, the storm on the hall, village concerns | `game/organizing.rs` (`harvest_together`, `storm_strikes_hall`, `village_concerns`) | done #558 |

## B. The sim runs it, the player never sees it

| # | Item | Where | Status |
|---|------|-------|--------|
| 31 | What the lord thinks the org's strength is (show as his mood / rumours, not the number) | `Game::lord_esteem` | done #559 |
| 32 | The castle's grudge: the next writ lands double | `Game::repression_debt` | done #560 |
| 33 | A harsh winter coming / under way | `Game::harsh_winter` | done #561 |
| 34 | The collector's takings riding out to the lord | `Game::takings` | done #562 |
| 35 | Numerals and reckonings, and the clerk who emerges | `Game::numeracy`, `reckonings`, `translator` | done #563 |
| 36 | Letters on the road | `Game::letters` | done #564 |
| 37 | Heirlooms and who holds them | `Game::heirlooms` | done #565 |
| 38 | Cheese wheels aging in the cellar, and its character | `Game::cheeses`, `cellar_trait` | done #566 |
| 39 | Mushroom lore: who's learned which patch kills | `Game::shroom_lore` | done #567 |
| 40 | Ailments on their own calendar (amatoxin phases, St Anthony's brothers) | `Game::ailments` | done #568 (already wired: `ailment_tick` chains the phases, `ergot` queues AntonineCare) |
| 41 | The shame ladder: who's on which rung | `Game::sanctioned` | done #569 |
| 42 | The matchmaker's tried pairings | `Game::matchmaker`, `match_tried` | done #570 (already wired: TalkTo arrival runs `match_attempt`, `match_tried` gates retries) |
| 43 | The herb-wife's tally of cures, and who's called at the hut | `Game::cured`, `remedy_visits` | done #571 |
| 44 | Signs before a wolf / fox night | `Game::wolf_coming`, `fox_coming` | done #572 (already wired: hens/tracks log a few days out, the flags gate the nights) |
| 45 | An unspoken discovery waiting to be disclosed | `Game::pending_discovery` | done #573 (already wired: ripens a fortnight then whispers out; org standing decides the answer) |
| 46 | The hermit at the cave | `Game::hermit` | done #574 |
| 47 | The martyr's grave | `Game::martyr_grave` | done #575 |
| 48 | The rivals' homes across the river | `Game::rival_homes` | done #576 |
| 49 | Towns and castles: names, keepers, on the minimap | `Game::sites` | done #577 |
| 50 | Word that a minstrel / pedlar / circus / ink-maker is due | `Game::next_*` | done #578 |
| 51 | Who drank at the tavern table | `Game::tavern_table` | done #579 |
| 52 | The swarm that took a liking to someone | `Game::swarm` | done #580 |
| 53 | Succession: who inherited from the dead | `Game::succeeded` | done #581 (already wired: `succession_tick` settles the bench to heir or shuttered, logs both) |
| 54 | Named leader milestone | `Game::leader_named` | done #582 |
| 55 | Buildings raised, buildings gone derelict | `Game::built_count`, `derelict` | done #583 |
| 56 | Complaints voiced at the meeting under way | `Game::voiced` | done #584 (already wired: voiced warms the room, three of a tag found the issue) |
| 57 | The flock's grazing ground | `Game::graze_at`, `flock_wander` | done #585 (already wired: the ground drifts every few days, the flock wanders it on their own clocks) |
| 58 | Which homes keep hens | `Game::hen_houses` | done #586 |
| 59 | The log silently dropping lines | `Game::log_dropped` | done #587 |
| 60 | The player's first body passing, the work going on | `Game::player_passed` | done #588 (already wired: `continuation_tick` passes the purse and work to the closest member or kin) |
| 61 | Castle grants, guards posted, assembly bans as standing status | `game/reactions.rs` (`LordGrant`, `GuardsPosted`, `AssemblyBan`) | done #589 |
| 62 | Seasons' effects on work and food (HUD) | `sim/climate.rs` `ClimateEffect` | done #590 |
| 63 | The org's real power vs its parameters (public part: turnout, discipline) | `sim/organized_power.rs` | done #591 |
| 64 | Daily rare-event roll: what's in the air (omens) | `Game::rare_fired`, `next_rare` | done #592 |
| 65 | Trader's stall stock | `Game::trader_stall` | done #593 |

## C. Villager inner life the inspector doesn't show

| # | Item | Where | Status |
|---|------|-------|--------|
| 66 | Traits (lazy, pious, gregarious...) | `game/traits.rs` | done #594 |
| 67 | How traits change what they do (tooltip) | `traits::modifies` | done #595 |
| 68 | Craft level and progress | `game/vocation.rs` (`Level`, `skill_gain`, `train_day`) | done #596 |
| 69 | Social class | `vocation::SocialClass` | done #597 |
| 70 | A child's growth toward a trade | `vocation::KidGrowth` | done #598 |
| 71 | Faith and its strength | `religion::Faith` | done #599 |
| 72 | Reputation stage | `reputation_qsp` | done #600 |
| 73 | Memorable episodes (what happened to them) | `psychology::Episode`, `record_episode` | done #601 |
| 74 | Kind of tie between two people (friend, rival, kin...) | `relationships::RelKind` | done #602 |
| 75 | Wants the player has learned (`Known`) as a list | `desires::InnerLife`, `Known` | done #603 |
| 76 | Rapport and how often you've talked | `player::Rapport`, `TalkCount` | done #604 |
| 77 | Organizing skills per person | `org_skills::Organizing` | done #605 |
| 78 | Livestock they own | `livestock::Livestock` | done #606 |
| 79 | Their pet and its bond | `pets::Pet` | done #607 |
| 80 | Illness stage, route, treatment | `illness::Disease`, `Stage`, `Treatment` | done #608 |
| 81 | Immunity after recovery, relapses | `illness::immunity_gain`, `relapse` | done #609 |
| 82 | Life stage and vitality | `lifecycle::Vitality` | done #610 |
| 83 | Odd jobs and deals in their purse | `livelihood::OddJob`, `Deal` | done #611 |
| 84 | Quality tier of goods in a satchel | `goods::tier` | done #612 |
| 85 | Backstory | `dialogue_trees::backstory` | done #613 |
| 86 | Personality (attachment) | `psychology::Personality` | done #614 |
| 87 | Whether a home is also a workplace / up for sale | `household::is_workplace`, `on_offer` | done #615 |
| 88 | Who's mourning whom | `psychology::Mourned` | done #616 |

## D. Helpers and smaller pieces never called

| # | Item | Where | Status |
|---|------|-------|--------|
| 89 | Herb-wife's remedies: which herb treats what, preparing them | `herbology` (`knows_cure`, `prepare_remedy`, `treats_illness`, `herb_tree`) | done #617 |
| 90 | Wild herbs spawning | `herbology::spawn_herb` | done #618 |
| 91 | Fire risk: what's flammable near a fire | `fire::flammable`, `is_scorched` | done #619 |
| 92 | Recruit odds shown before you ask | `player::recruit_trust` | done #620 |
| 93 | Campaign elite shadowing / mass ties | `campaign::shadow_elite`, `tie_mass` | done #621 |
| 94 | Campaign history: recent beats and who consented | `campaign_arc` (`recent_beats`, `record_consent`, `arc_facts`) | done #622 |
| 95 | Personal facts feeding meeting dialogue | `campaign_dialogue` (`personal_facts`, `meeting_facts`) | done #623 |
| 96 | Conversations picked by role | `conversation::for_roles` | done #624 |
| 97 | Market prices settable (lord's fixed price) | `economics::set_base_price` | done #625 |
| 98 | `UiModel::debug_lines` built and never drawn | `session/model.rs` | done #626 |
| 99 | `UiModel::reach_target` never read by the client | `session/model.rs` | done #627 |
| 100 | Life events (births, marriages, deaths) as a family record | `lifecycle::LifeEvent` | done #628 |

## Third census (ticket #821) - the sprint's own Game fields

The sweep ran on the session-scale fields landed since the second
census (the faith/organizing/agriculture/war/art arcs). Method: `pub
fn` usage census + the `Game` fields `view_model.rs` never reads + the
ECS systems never scheduled.

Finding: 39 `Game` fields the public view (`UiModel`/`AgentObs`)
never reads - the new class of unwired the ticket warned of.

Disposition per field:
- Event-level fields already surfaced through `Game.log` (the LOG
  panel carries the event): `comeback`, `emigration`, `war`,
  `caravan`, `uprising`, `mystic`, `council`, `raid` - WIRED via the
  log channel.
- Sim-internal state the omniscient probe/tests reach (the session
  tests read them directly - they ARE wired to the probe): `drought`,
  `arc_phase`, `storyteller`, `lord_strength`, `soil_quality`,
  `learned_words`, `fluent`, `redacted_canon`, `vernacular`,
  `prophet`, `shield_cover`, `incense`, `tea`, `cloth_quality`,
  `petition_count`, `frame_readiness`, `institutions`,
  `locally_sponsored`, `leader_dependence`, `folk_school`,
  `community_teacher`, `brand_saturation`, `parley`,
  `hidden_institution`, `inquisition`, `compliance_burden`,
  `campaign_logistics`, `incorporation_offer`, `separation_pressure`,
  `front` - DORMANT-BY-DESIGN (the public view's not the surface for
  internal sim state; the tests and the log carry what the player
  needs).
- The `view_model.rs` surface stays the deliberately-thin public view
  per docs/architecture.md - the omniscient probe and the tests read
  the Game directly.

No code changes: the fields are `pub` (reachable) and the events are
in the log (visible). The third sweep's a disposition, not a wire.
