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
| 14 | Grass and forest spread (cellular automaton) | `sim/emergence.rs` | todo |
| 15 | Reputation stages: unknown -> known -> trusted -> esteemed / disgraced | `sim/reputation_qsp.rs` (`spawn_reputation`) | todo |
| 16 | Contract verb from the old text action menu | `game/action_menu.rs` | todo |
| 17 | Guilds forming and policing their trade | `game/guilds.rs` (`guild_should_form`, `form_guild`, `enforce_security`) | todo |
| 18 | Landforms in map generation | `game/landforms.rs` (`select_landform`, `apply_landform`) | todo |
| 19 | Aging and life stages | `game/lifecycle.rs` (`AgingSystem`, `LifeStage`, `spawn_person`) | todo |
| 20 | Grief and mourning | `sim/psychology.rs` (`GriefSystem`, `Mourned`) | todo |
| 21 | Attachment styles shaping bonds | `sim/psychology.rs` (`AttachmentSystem`, `AttachmentStyle`) | todo |
| 22 | Relationship drift from interactions | `game/relationships.rs` (`RelationshipSystem`, `infer_kind`) | todo |
| 23 | Influence and one-on-ones as a running system | `sim/community.rs` (`InfluenceSystem`, `OneOnOneSystem`, `exert_influence`) | todo |
| 24 | Organized power wasting away when unused | `sim/organized_power.rs` (`PowerAtrophySystem`) | todo |
| 25 | Crop fields growing through the season | `game/villager.rs` (`CropGrowthSystem`, `spawn_crop_field`, `servings_left`) | todo |
| 26 | Stall system | `game/villager.rs` (`StallSystem`, `spawn_stall`) | todo |
| 27 | Nourishment | `NourishmentSystem` | todo |
| 28 | Organizing skills learned by doing and taught | `game/org_skills.rs` (`practice`, `teach`, `learn_from_meeting`) | todo |
| 29 | Pets lifting their owner's mood | `game/pets.rs` (`pet_mood_boost`) | todo |
| 30 | Harvesting together, the storm on the hall, village concerns | `game/organizing.rs` (`harvest_together`, `storm_strikes_hall`, `village_concerns`) | todo |

## B. The sim runs it, the player never sees it

| # | Item | Where | Status |
|---|------|-------|--------|
| 31 | What the lord thinks the org's strength is (show as his mood / rumours, not the number) | `Game::lord_esteem` | todo |
| 32 | The castle's grudge: the next writ lands double | `Game::repression_debt` | todo |
| 33 | A harsh winter coming / under way | `Game::harsh_winter` | todo |
| 34 | The collector's takings riding out to the lord | `Game::takings` | todo |
| 35 | Numerals and reckonings, and the clerk who emerges | `Game::numeracy`, `reckonings`, `translator` | todo |
| 36 | Letters on the road | `Game::letters` | todo |
| 37 | Heirlooms and who holds them | `Game::heirlooms` | todo |
| 38 | Cheese wheels aging in the cellar, and its character | `Game::cheeses`, `cellar_trait` | todo |
| 39 | Mushroom lore: who's learned which patch kills | `Game::shroom_lore` | todo |
| 40 | Ailments on their own calendar (amatoxin phases, St Anthony's brothers) | `Game::ailments` | todo |
| 41 | The shame ladder: who's on which rung | `Game::sanctioned` | todo |
| 42 | The matchmaker's tried pairings | `Game::matchmaker`, `match_tried` | todo |
| 43 | The herb-wife's tally of cures, and who's called at the hut | `Game::cured`, `remedy_visits` | todo |
| 44 | Signs before a wolf / fox night | `Game::wolf_coming`, `fox_coming` | todo |
| 45 | An unspoken discovery waiting to be disclosed | `Game::pending_discovery` | todo |
| 46 | The hermit at the cave | `Game::hermit` | todo |
| 47 | The martyr's grave | `Game::martyr_grave` | todo |
| 48 | The rivals' homes across the river | `Game::rival_homes` | todo |
| 49 | Towns and castles: names, keepers, on the minimap | `Game::sites` | todo |
| 50 | Word that a minstrel / pedlar / circus / ink-maker is due | `Game::next_*` | todo |
| 51 | Who drank at the tavern table | `Game::tavern_table` | todo |
| 52 | The swarm that took a liking to someone | `Game::swarm` | todo |
| 53 | Succession: who inherited from the dead | `Game::succeeded` | todo |
| 54 | Named leader milestone | `Game::leader_named` | todo |
| 55 | Buildings raised, buildings gone derelict | `Game::built_count`, `derelict` | todo |
| 56 | Complaints voiced at the meeting under way | `Game::voiced` | todo |
| 57 | The flock's grazing ground | `Game::graze_at`, `flock_wander` | todo |
| 58 | Which homes keep hens | `Game::hen_houses` | todo |
| 59 | The log silently dropping lines | `Game::log_dropped` | todo |
| 60 | The player's first body passing, the work going on | `Game::player_passed` | todo |
| 61 | Castle grants, guards posted, assembly bans as standing status | `game/reactions.rs` (`LordGrant`, `GuardsPosted`, `AssemblyBan`) | todo |
| 62 | Seasons' effects on work and food (HUD) | `sim/climate.rs` `ClimateEffect` | todo |
| 63 | The org's real power vs its parameters (public part: turnout, discipline) | `sim/organized_power.rs` | todo |
| 64 | Daily rare-event roll: what's in the air (omens) | `Game::rare_fired`, `next_rare` | todo |
| 65 | Trader's stall stock | `Game::trader_stall` | todo |

## C. Villager inner life the inspector doesn't show

| # | Item | Where | Status |
|---|------|-------|--------|
| 66 | Traits (lazy, pious, gregarious...) | `game/traits.rs` | todo |
| 67 | How traits change what they do (tooltip) | `traits::modifies` | todo |
| 68 | Craft level and progress | `game/vocation.rs` (`Level`, `skill_gain`, `train_day`) | todo |
| 69 | Social class | `vocation::SocialClass` | todo |
| 70 | A child's growth toward a trade | `vocation::KidGrowth` | todo |
| 71 | Faith and its strength | `religion::Faith` | todo |
| 72 | Reputation stage | `reputation_qsp` | todo |
| 73 | Memorable episodes (what happened to them) | `psychology::Episode`, `record_episode` | todo |
| 74 | Kind of tie between two people (friend, rival, kin...) | `relationships::RelKind` | todo |
| 75 | Wants the player has learned (`Known`) as a list | `desires::InnerLife`, `Known` | todo |
| 76 | Rapport and how often you've talked | `player::Rapport`, `TalkCount` | todo |
| 77 | Organizing skills per person | `org_skills::Organizing` | todo |
| 78 | Livestock they own | `livestock::Livestock` | todo |
| 79 | Their pet and its bond | `pets::Pet` | todo |
| 80 | Illness stage, route, treatment | `illness::Disease`, `Stage`, `Treatment` | todo |
| 81 | Immunity after recovery, relapses | `illness::immunity_gain`, `relapse` | todo |
| 82 | Life stage and vitality | `lifecycle::Vitality` | todo |
| 83 | Odd jobs and deals in their purse | `livelihood::OddJob`, `Deal` | todo |
| 84 | Quality tier of goods in a satchel | `goods::tier` | todo |
| 85 | Backstory | `dialogue_trees::backstory` | todo |
| 86 | Personality (attachment) | `psychology::Personality` | todo |
| 87 | Whether a home is also a workplace / up for sale | `household::is_workplace`, `on_offer` | todo |
| 88 | Who's mourning whom | `psychology::Mourned` | todo |

## D. Helpers and smaller pieces never called

| # | Item | Where | Status |
|---|------|-------|--------|
| 89 | Herb-wife's remedies: which herb treats what, preparing them | `herbology` (`knows_cure`, `prepare_remedy`, `treats_illness`, `herb_tree`) | todo |
| 90 | Wild herbs spawning | `herbology::spawn_herb` | todo |
| 91 | Fire risk: what's flammable near a fire | `fire::flammable`, `is_scorched` | todo |
| 92 | Recruit odds shown before you ask | `player::recruit_trust` | todo |
| 93 | Campaign elite shadowing / mass ties | `campaign::shadow_elite`, `tie_mass` | todo |
| 94 | Campaign history: recent beats and who consented | `campaign_arc` (`recent_beats`, `record_consent`, `arc_facts`) | todo |
| 95 | Personal facts feeding meeting dialogue | `campaign_dialogue` (`personal_facts`, `meeting_facts`) | todo |
| 96 | Conversations picked by role | `conversation::for_roles` | todo |
| 97 | Market prices settable (lord's fixed price) | `economics::set_base_price` | todo |
| 98 | `UiModel::debug_lines` built and never drawn | `session/model.rs` | todo |
| 99 | `UiModel::reach_target` never read by the client | `session/model.rs` | todo |
| 100 | Life events (births, marriages, deaths) as a family record | `lifecycle::LifeEvent` | todo |
