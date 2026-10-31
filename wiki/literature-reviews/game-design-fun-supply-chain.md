# Lit Review: What Makes Supply-Chain & Management Games Fun — Design Basis for the Balance Tester

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Domain:** Game Design Theory / Management-Sim Craft
**Purpose:** Ground an automated "balance tester" that scores headless sim runs of the Wanderer organizing game. Defines named fun-producing mechanisms in seven reference titles, proposes mechanics for this game, and specifies a computable KPI/OKR system over sim telemetry.
**Confidence:** High — the fun theory spine is peer-reviewed/founder-stated; per-game mechanics are documented in manuals, dev blogs, datamined wikis, and interviews.

## Summary

Fun in management/economy games is not a vibe; it is a small set of composable mechanisms, all of which are measurable in a time series:

1. **A visible throughput loop** (Factorio, Settlers 2): the economy is a flow the player can watch, and the current bottleneck is legible. Fun = diagnosing the constraint, fixing it, and watching the fix expose the *next* constraint — a self-generating problem queue.
2. **Nested-horizon anticipation** (Civilization): short/medium/long goals are never all satisfied at once, so the player is always "playing in the future." This is the literal mechanism behind "one more turn."
3. **Incentive-priced indirect control** (Majesty): you can't command anyone — you price outcomes and autonomous agents respond. This is *exactly* the Wanderer's fantasy ("you can't command anyone") and makes pricing/motivation itself the skill.
4. **A food-first survival stack with compounding morale** (Lords of the Realm): fed serfs → morale → population growth → more labor to allocate. Subsistence is the base layer every other system sits on.
5. **Wealth-scaled adaptive pressure** (RimWorld): threats are bought from a budget proportional to player strength, with a rubber-band adaptation factor (0.4–1.47) that damps the next threat after losses. Challenge tracks skill — the flow channel, mechanized.
6. **Interlocking legible clocks** (Stardew Valley): day/week/season/year deadlines the player can plan around, parallel goal strands so there is no dead end, and festivals that function as forced breathers.
7. **The theory spine**: Csikszentmihalyi's flow (clear goals, unambiguous feedback, challenge/skill balance); MDA (designers build Mechanics→Dynamics→Aesthetics, so "make it fun" must be decomposed to mechanics); Koster (fun = mastering patterns; boredom = exhausted pattern; variable feedback required); Salen & Zimmerman (meaningful play = outcomes *discernible* and *integrated*); and the pacing rule buildup→climax→breather.

The through-line for the balance tester: **every fun signal above reduces to a measurable property of a telemetry time series** — event density, drought length, pressure-vs-power ratio, recovery after shocks, bottleneck rotation, action→outcome latency. The KPI section operationalizes this.

## Key Sources

- **Hunicke, LeBlanc & Zubek, "MDA: A Formal Approach to Game Design" (2004)** — **Rating: A+.** The field-standard framework. Key finding: designers create mechanics that produce dynamics that produce aesthetics; players experience it in reverse. "Make it more fun" is an aesthetics-layer note that says nothing about which mechanic to change — the direct justification for scoring runs on *dynamics-level* telemetry rather than declaring fun as a verdict.
- **Csikszentmihalyi's flow, as synthesized in game studies (IJISAM 2014; 43-study meta-analysis; 252-study scoping review; see also `sprint-4-psychology-of-play.md`)** — **Rating: A.** Three testable conditions: clear goals, unambiguous feedback, challenge/skill balance. Video games are ideal flow machines because feedback is immediate and difficulty manipulable. Gives us `action_yield`, `pressure_index`, and goal-stack KPIs directly.
- **Raph Koster, *A Theory of Fun* + "Atomic Theory of Fun" (2012) + "Core of Fun" (2007)** — **Rating: A-.** Fun is the neurochemical reward for pattern mastery; a game atom = input→model→feedback→mastery. Boredom is a *solved* pattern — hence a novelty KPI. "Low-risk activity for high reward is bad for fun" — hence grind must cost something and pay out. Games are "deliberate practice machines": clear goals, repeated attempts, continuous feedback, mentally demanding.
- **Salen & Zimmerman, *Rules of Play* — meaningful play** — **Rating: A.** An action is meaningful when its outcome is *discernible* (player can see what it did) and *integrated* (it matters to the larger game). Directly yields the `action_yield` and `price_legibility` KPIs.
- **Sid Meier interviews (Ars Technica 2019; GameSpot GDC keynote; ACM "Interesting Complexity")** — **Rating: A.** "A game is a series of interesting decisions." "One more turn" mechanism named by its inventor: players hold short/medium/long goals simultaneously and "are playing in the future — and that future is just one more turn ahead." "The first 15 minutes of any game must be compelling." Quantized turns shrink the perceived distance to payoff — foreshadowing as retention tech.
- **Factorio — FFF dev blogs (esp. #376), Design Codex teardown, main-bus community literature** — **Rating: A-.** "The core of what makes Factorio good is that the horrible huge grind… can be mitigated by automation" — grind exists *to be deleted*, which is its payoff. Main bus is "a tool to stop you screwing yourself": bootstrap structure, then outgrow it. Ratios are "legible math with infinite depth." Every solved constraint reveals the next downstream — the self-generating problem queue.
- **RimWorld — wiki/datamined raid math + Tynan Sylvester forum posts + Design Lab teardown** — **Rating: A-.** Raid points = `(wealth_points + pawn_points) × threat_scale × starting_factor × adaptation_factor`, clamped [35, 10,000]. Buildings count at half value. Adaptation counter clamps [−60,+100], maps to multiplier 0.4–1.47, drops 30 on a colonist death — a literal rubber band. Threat scale presets: 0.10 Peaceful → 1.00 baseline → 2.20 "Losing is Fun". Storytellers differ in *cadence* not force: Cassandra ~4.6 days on / 6 off, Phoebe 8/8, Randy memoryless. Design intent: "story generator," not simulation for its own sake.
- **The Settlers II — official manual + settlers2.net road guides + contemporaneous reviews** — **Rating: B+.** "Transportation is the most important concept in Settlers II." Flags cap at 8 items; carriers are a fixed-throughput resource; road length trades per-item latency against segment throughput (2-node roads maximize throughput, 3-node move items faster); flag overflow visibly stalls production; heavily used roads auto-pave — the world annotates its own hotspots.
- **Stardew Valley — ConcernedApe dev updates + systems teardowns** — **Rating: B+.** A day is 14 real minutes; a season 28 days; festivals freeze the clock and force rest. All deadlines are in-sim (nothing decays while closed) — anticipation without FOMO. Parallel goal strands (farm/mine/fish/social) mean no dead end. Design target stated by the dev: make sure players "feel like they have direction and purpose" early — hence the quest log. Calendar arithmetic is a hard inequality: a season must be long enough to grow what it offers.
- **Majesty: The Fantasy Kingdom Sim — manual, Ars Technica review (2000), retrospectives** — **Rating: B.** Heroes are autonomous with per-class dispositions; the *only* lever is priced reward flags, whose attractiveness scales with bounty ÷ (distance × danger × hero-suitability). Economy is auto-run (tax collectors); frustration when incentives are mispriced *is* the gameplay — pricing is the skill.
- **Lords of the Realm (1994) — contemporaneous reviews + MobyGames synthesis** — **Rating: B.** "Happy, well-fed peasants are more productive" — food is the base layer; morale compounds into population; labor allocation across grain/cattle/sheep/industry; inter-county wagons have real travel latency making logistics a planning problem; conscription is a legible opportunity cost (army strength bought with workforce + happiness).

## Per-Game Deep Dives — Named Mechanisms

### Factorio — "the self-generating problem queue" + "visible throughput"
Every production line is a flow you can *watch*: belts carry a visible items/sec, so the bottleneck is diagnosable by eye. Fixing it raises throughput until the next stage downstream becomes the constraint — the game never runs out of problems because your own success manufactures them. Ratios (N miners → M furnaces → K assemblers) are legible optimization math. The **spaghetti → main bus → trains/bots** arc is a *mastery ladder* in Koster's sense: each phase teaches a new pattern, and the moment the pattern is solved the game hands you a new pattern-space. Crucially, grind is structured as **labor awaiting deletion**: hand-crafting exists only so automation can erase it — the payoff for the grind is that the grind disappears (FFF-376: "the grind… can be mitigated by automation").
*Transferable insight:* an economy game is fun when (a) the flow is visible, (b) exactly one constraint binds at a time, (c) the binding constraint *rotates* as a consequence of the player's fixes.

### Civilization V — "nested-horizon anticipation" (a.k.a. one-more-turn)
Meier's own account: the player holds short-term (unit arrives next turn), medium-term (wonder in 8 turns), and long-term (victory) goals that are *never simultaneously resolved*, so there is always a near-future payoff pending — "you're playing in the future." Turn quantization makes that future feel close and discrete. Eureka/boost mechanics reward incidental play with progress — learning-by-doing converted into anticipation ("do X to finish Y sooner"). Era pacing stages the mastery ladder: each era adds systems once the previous pattern is chunked. Meier's **first-15-minutes rule**: the early game must be compelling or nothing else matters.
*Transferable insight:* retention = ≥2 pending clocks at all times; reward early; stage complexity so each mastered pattern unlocks a new one.

### Majesty — "incentive-priced indirect control"
The player never orders a unit. Heroes are autonomous agents with class dispositions (rangers explore, warriors hunt, rogues get killed early); the only lever is the **priced flag** — gold on a target, weighted against distance and danger. Underprice and nothing happens; overprice and the treasury bleeds. Attachment emerges anyway: heroes level, acquire names, die doing what you paid them to do. Reviews note it was "a little frustrating, but also very addictive" — the frustration *is* the skill expression, because control-through-pricing is a learnable pattern.
*Transferable insight:* this is the Wanderer's exact fantasy. Indirect control works when (a) agents have legible dispositions, (b) the price/response function is learnable, (c) failure modes are comic or instructive rather than silent.

### Lords of the Realm — "the food-first feudal stack"
Subsistence is the base of a strict dependency pyramid: fed serfs → morale → population growth → assignable labor → surplus → industry (stone, wood, iron, arms) → castles and armies. Seasons gate sowing/harvest, so food planning has a calendar horizon. Inter-county wagon latency makes logistics a *planning* problem, not a reflex one. Conscription is a legible trade: each soldier is peasants-not-working plus happiness-lost. Reviews praised that orders show visible results (fields sown, troops marching) — feedback is diegetic, not spreadsheet.
*Transferable insight:* the KPI stack must put subsistence first — a village starving is a village that can't be organized; every elite-tier gain must trace to a subsistence cost someone paid.

### RimWorld — "the wealth-scaled rubber band" (the AI Storyteller)
Threats aren't scripted; they're *purchased* from a point budget scaled to colony wealth (items full, buildings half, pawns double) and difficulty multiplier. The **adaptation factor** is the genius bit: a hidden counter climbs while you prosper, drops hard on downed/dead colonists, and maps to a 0.4–1.47 threat multiplier — losses literally buy you a quieter next chapter, and prosperity buys danger. This is the flow channel implemented as a feedback controller, plus a built-in comeback mechanic. The three storytellers prove pacing is separable from pressure: same budget math, different cadence (Cassandra's regular 4.6-on/6-off drumbeat vs. Phoebe's slow 8/8 vs. Randy's memoryless chaos). Sylvester's explicit goal: generate *tellable moments* — near-losses are the product, not failures.
*Transferable insight:* pressure should be a function of player strength, not wall-clock; after a shock the system should visibly ease; near-losses are content.

### Stardew Valley — "interlocking calm clocks"
Four timescales interlock without competing: the day (fixed time/energy budget → gentle prioritization), the week (gift caps, shop schedules), the season (28 days, crops gated → forced replanning 4×/year), the year (festivals, Grandpa's evaluation). Festivals freeze the clock — engineered breathers. Parallel self-directed goal strands mean the player "always has three small things to do next" and can never hit a dead end. All deadlines are in-sim — a save left a year is exactly where it was; anticipation replaces FOMO. Dev-state goal: early direction and purpose via an unobtrusive checklist.
*Transferable insight:* relaxed grind works when every chore has a visible near payoff and every season has a visible deadline; breathers should be *scheduled*, not hoped for.

### The Settlers 2 — "the legible bottleneck"
The economy is literally a transport graph: buildings are nodes, road segments are edges, carriers are the fixed-throughput resource, flags are buffers capped at 8 items. Overflowing flags visibly stall upstream production — you diagnose the economy by *watching little people walk*. Road-length is a real tradeoff (short segments maximize throughput; longer ones reduce per-item latency); double roads and donkeys are the throughput upgrades; heavily-used roads auto-pave, so the world annotates its own hotspots. The fun is that your grand plan's failure mode is always *findable*.
*Transferable insight:* every resource flow in our sim should have a visible carrier/buffer pair; a stalled economy should be diagnosable from telemetry alone; degradation should leave tracks (trails.heat is already this).

### The theory spine
- **Flow (Csikszentmihalyi):** clear goals + unambiguous feedback + challenge≈skill. All three are computable (pending goals count, action→delta latency, threat/power ratio).
- **MDA:** score the *dynamics* (telemetry), fix the *mechanics* (constants), aim at the *aesthetics* (engagement). The tester reads the middle layer.
- **Koster:** fun = mastering a not-yet-solved pattern. Boredom = exhausted pattern → novelty metric. Variable feedback keeps atoms alive → outcome-variance metric.
- **Meaningful choice (Salen & Zimmerman / Meier):** choices must be discernible (action produces visible delta) and integrated (delta matters later) → action_yield + consequence-lag metrics.
- **Pacing:** buildup→climax→breather alternation → window-classification metric.
- **Grind needs a payoff:** the payoff is either deletion of the grind (Factorio) or a compounding asset (Lords' morale→population) — never a flat line.

### Cross-title synthesis: mechanism → Wanderer analog → governing KPI

| Reference mechanism | Analog in this sim | KPI(s) that police it |
|---|---|---|
| Factorio's self-generating problem queue | Talk→trust→recruit→organize pipeline; village troubles rotate | K10 bottleneck_rotation, K11 org_growth_shape |
| Settlers 2's legible bottleneck | Flag-like buffers (stock.grain, stock.bread), trails.heat as worn-path throughput | K8 grain_cover_ticks, K9 price_legibility, K18 consequence_visibility |
| Civ's one-more-turn horizon stack | Writ/festival/winter countdowns + open ISSUES | K17 multi_horizon_stack, K3 event_drought_max |
| Civ's eureka / first-15-minutes | Three-voices→issue unlock; early trust gains | K2 time_to_first_reward |
| Majesty's priced indirect control | Pledges/bounties motivating autonomous villagers | K13 action_yield (pricing must produce visible response) |
| RimWorld's wealth-scaled rubber band | Threats ∝ org.power; easing after losses | K5 threat_rate, K6 recovery_ratio, K16 winter_survival_band |
| Stardew's interlocking calm clocks | Feast-day and writ calendar; harsh-winter cadence | K14 pacing_alternation, K7 hungry_fraction |
| Koster's novelty / flow's feedback | Event-type variety; action→delta latency | K4 novelty_ratio, K1 decision_density, K13 action_yield |
| Meier's interesting decisions; S&Z's meaningful play | Every action discernible + integrated | K13 action_yield, K18 consequence_visibility |
| PLAY.md's lagging lord_esteem | Dramatic-irony window for the org | K12 esteem_lag_gap |
| RimWorld's story-generator goal | Deaths, scandals, feasts, successions | K15 story_event_density |

The table is the bridge the balance tester implements: **each row is one measurable promise the sim makes to the player.**

## Game Mechanic Proposals

### Mechanic 1: The pledge board (Majesty's priced flags, mapped to organizing)
**Basis:** Majesty's incentive-priced indirect control + PLAY.md's "you can't command anyone." The org posts **pledges** on shared troubles (bread price, the wolf, the writ): gold, favors, or meeting-commitment offered to whichever villager acts. Response scales with `pledge ÷ (risk × distance × disposition-fit)`.
**Variables (4):** `pledge_value`, `issue_urgency`, `member_disposition_fit`, `response_threshold`.
**Fun factor:** makes *motivating people* the skill rather than clicking them; underpriced pledges failing publicly is comedy + information (learnable price curve, per Koster).

### Mechanic 2: The organizing pipeline as a supply chain (Factorio/Settlers)
**Basis:** visible throughput + legible bottleneck. Talk → trust → recruit → member → organizer is a literal conversion pipeline with ratios (e.g., ~3 talks per trust-gate, 3 voices per issue per PLAY.md). Render each stage's conversion rate; surface the starving stage as the current "trouble."
**Variables (5):** `talk→trust_rate`, `trust→recruit_rate`, `recruit→member_rate`, `meeting_throughput`, `organizer_capacity`.
**Fun factor:** the self-generating problem queue applied to people — fix recruiting, then meetings become the bottleneck, then logistics of feeding members.

### Mechanic 3: The pressure director (RimWorld's storyteller, village-flavored)
**Basis:** wealth-scaled threats + adaptation rubber band. Threat budget ∝ `org.power + village prosperity`; spent on writs, levies, blame cascades, wolves, hard winters. An adaptation counter climbs on org wins and drops on member losses/sanctions, mapping to a 0.5–1.4 pressure multiplier.
**Variables (4):** `threat_budget`, `adaptation_counter`, `cadence_phase`, `shock_severity`.
**Fun factor:** challenge tracks strength (flow channel); comebacks are engineered, not lucky; cadence can be tuned separately from force.

### Mechanic 4: The feast-and-writ calendar (Stardew clocks + Lords' seasons)
**Basis:** interlocking legible deadlines. Existing feast days (St. Swithin's, Harvest Home, Candlemas, St. Martin's) and seasonal bailiff writs become visible countdowns the player plans around: organize before the writ, feast to convert contacts to members, winter as the stress-test climax.
**Variables (4):** `ticks_to_festival`, `ticks_to_writ`, `ticks_to_winter`, `stock_cover_days`.
**Fun factor:** mid-horizon goals that bridge talk-loop (short) and power-arc (long) — the Civ nested-horizon stack made seasonal; festivals are scheduled breathers.

### Mechanic 5: Troubles surface the bottleneck (Settlers 2's legible stall + existing ISSUES line)
**Basis:** PLAY.md already has "three voices on one trouble → the organizer sees the issue." Make the *village's* current binding constraint (grain cover, trail congestion, dark church, stinking reach) emit complaints proportional to severity; the ISSUES line becomes the economy's bottleneck indicator.
**Variables (4):** `constraint_severity`, `complaint_rate`, `issue_threshold` (=3 voices), `issue_visibility_lag`.
**Fun factor:** the world tells you what to fix next — Settlers' "watch the flags" via gossip instead of goods; also the eureka payoff (insight unlocked by accumulated observation).

### Mechanic 6: Spend power to be seen (Lords' visible orders + "power builds as you spend it")
**Basis:** PLAY.md: "a full kitchen meeting shows the street you're somebody, and the lord's estimate of you lags behind the truth." `org.power` is latent; `lord_esteem` and street-standing update only on *visible displays* (meetings, feasts sponsored, painted halls, standing against a writ) with a decay.
**Variables (4):** `power` (latent), `esteem` (perceived, lagging), `display_magnitude`, `esteem_decay`.
**Fun factor:** dramatic irony is a resource — you can act inside the lag window; visible flex is the grind's payoff; creates the squeeze/negotiate/pass-by phase transitions in lord response.

### Mechanic 7: Grind-to-deletion (Factorio's automation payoff)
**Basis:** repetitive org chores (running messages, collecting dues, scheduling meetings) should be *delegatable* to recruited members — the player's reward for recruiting is that the chore leaves their action list.
**Variables (3):** `chore_load`, `delegation_threshold` (member skill), `chore_residual`.
**Fun factor:** every promotion visibly shrinks the todo list — the clearest possible "grind has a payoff" signal.

## Scoring / KPI / OKR System

All KPIs are computable from the telemetry gauges (`villagers.alive`, `villagers.hungry`, `org.members`, `org.power`, `org.lord_esteem`, `sanctioned`, `trails.heat`, `stock.grain`, `stock.bread`, `env.tree_cover`, `price.grain`, `price.wood`, `price.fish`, `winter.harsh`) plus a per-tick event log. "Event" = a non-routine log line (deaths, recruitments, meetings, writs, sanctions, shortages, feasts, feuds, weather, births, wolf/fox, world events); routine per-villager eat/sleep/walk lines are excluded.

### KPIs

Each KPI below gives: **name**, **computed as** (formula over the time series), **target band**, **red-flag threshold**, and **basis** (the research the number traces to). Bands are priors — calibrate per Next Steps §3.

**K1. `decision_density`**
- Computed as: meaningful event-log lines per 10-tick rolling window (routine per-villager eat/sleep/walk lines excluded).
- Target: **2–6 per 10 ticks**. Red flag: **<0.5 sustained 50 ticks**.
- Basis: Meier's "series of interesting decisions" made countable; flow needs steady input; existing rubric flags 10+ dead ticks. Upper bound guards event spam = noise, not engagement.

**K2. `time_to_first_reward`**
- Computed as: ticks from t=0 to the first milestone-tagged event (first trust gain, first recruit, first meeting held, first gold earned).
- Target: **≤40 ticks**. Red flag: **>80 ticks**.
- Basis: Meier's "first 15 minutes must be compelling," compressed to sim ticks; flow's unambiguous-feedback condition requires early payoff.

**K3. `event_drought_max`**
- Computed as: longest consecutive run of ticks with zero event-log lines in a run.
- Target: **≤60 ticks**. Red flag: **>100 ticks**.
- Basis: the fun rubric's own "no new events for 100+ ticks" red flag; Stardew's "always three small things to do next."

**K4. `novelty_ratio`**
- Computed as: distinct event types ÷ total events, per 200-tick window.
- Target: **≥0.25**. Red flag: **<0.10 in two consecutive windows**.
- Basis: Koster — boredom is a *solved* pattern; once the player has chunked all event types, the sim stops teaching and stops being fun.

**K5. `threat_rate`**
- Computed as: threat-class events (writ, levy, sanction, blame cascade, illness outbreak, wolf/fox, harsh weather) per 50 ticks, compared to expectation `E = 1 + org.power/50`.
- Target: **measured within 0.5–1.5× of E**. Red flag: **0 for 100+ ticks** (no stakes) or **>2× E sustained 50 ticks** (overwhelm).
- Basis: RimWorld raid points ∝ colony wealth — pressure scales to strength, mechanizing the flow channel; threat_scale 1.0 is the intended baseline, ±50% is the human-tolerable band.

**K6. `recovery_ratio`**
- Computed as: for each shock event, `gauge(t+150) ÷ gauge(pre-shock)` on whichever gauge the shock hit (stock.grain, org.members, villagers.alive).
- Target: **≥0.8**. Red flag: **<0.5, or monotone decline for 150 ticks post-shock**.
- Basis: RimWorld's adaptation factor (0.4–1.47) is an engineered rubber band; the rubric demands comeback potential. A healthy sim bends, it doesn't spiral.

**K7. `hungry_fraction`**
- Computed as: `villagers.hungry ÷ villagers.alive`, rolling 10-tick mean.
- Target: **<0.15 normal; <0.40 while `winter.harsh=1`**. Red flag: **>0.50 for 30 ticks**.
- Basis: Lords of the Realm — food is the base layer of the dependency pyramid; mass hunger is a death spiral, not a challenge, and crowds out every other loop.

**K8. `grain_cover_ticks`**
- Computed as: `(stock.grain + stock.bread) ÷ |d(stock)/dt|` — ticks until empty at the current burn rate (use 999 if stocks are rising).
- Target: **≥30 ticks**. Red flag: **<10 outside winter; <5 during `winter.harsh`**.
- Basis: Settlers' flag-buffer legibility; Stardew's calendar arithmetic — stores must be able to cover the season or the calm breaks into frustration.

**K9. `price_legibility`**
- Computed as: coefficient of variation (σ/μ) of `price.grain` per 100-tick window; also applied to `price.wood`, `price.fish`.
- Target: **CV 0.05–0.35**. Red flag: **>0.60 with no cause event within 20 ticks**, or **<0.02 for 200 ticks** (dead economy).
- Basis: Salen & Zimmerman — outcomes must be discernible *and* caused; Settlers' whole appeal is that price/flow signals trace to visible bottlenecks.

**K10. `bottleneck_rotation`**
- Computed as: per 50-tick window, identify the dominant constraint (argmin of normalized covers: grain_cover, member-growth slope, `trails.heat` headroom, `sanctioned` headroom); count identity changes per 500 ticks.
- Target: **≥3 distinct bottlenecks per 500 ticks**. Red flag: **same constraint >400 ticks** (stuck grind) or **no detectable constraint >100 ticks** (no tension).
- Basis: Factorio's self-generating problem queue — fun is the bottleneck *migrating* because the player fixed the last one; a static bottleneck is grind.

**K11. `org_growth_shape`**
- Computed as: `org.members` time series segmented into slope regimes; require (a) `time_to_first_member ≤80`, (b) a compounding mid-phase (positive second derivative), (c) plateau ≥25% of `villagers.alive`.
- Target: **full S-curve present**. Red flag: **`org.members = 0` at tick 150**, or membership never exceeds 3.
- Basis: the progression fantasy is a compounding asset (Lords' morale→population flywheel; Factorio's production curve). A flat or absent curve means the core loop never pays off.

**K12. `esteem_lag_gap`**
- Computed as: `org.power − org.lord_esteem` (normalized) sampled during power-rise windows; plus ticks for esteem to close 80% of a step-change in power.
- Target: **gap ∈ [0.05, 0.5]; catch-up latency 20–80 ticks**. Red flag: **gap ≤0 sustained** (no irony window) or **esteem > power sustained** (incoherent detection).
- Basis: PLAY.md — "the lord's estimate of you lags behind the truth." The lag *is* the gameplay window (act while underestimated); Civ's foreshadowing principle — anticipation requires a visible pending change.

**K13. `action_yield`**
- Computed as: fraction of logged player/bot actions followed by a measurable delta in any gauge within 20 ticks.
- Target: **≥0.6**. Red flag: **<0.3**.
- Basis: flow's unambiguous-feedback condition + the *discernible* half of meaningful play. Dead clicks are the objective signature of grind.

**K14. `pacing_alternation`**
- Computed as: classify each 10-tick window as peak (≥3 events), normal (1–2), or breather (0); track longest runs and class shares.
- Target: **longest peak-run ≤40 ticks; longest breather-run ≤60 ticks; each class ≥10% of windows**. Red flag: **>80% of windows in one class**.
- Basis: the rubric's buildup→climax→breather pacing rule; RimWorld Cassandra's 4.6-on/6-off drumbeat; Stardew festivals as scheduled breathers.

**K15. `story_event_density`**
- Computed as: log lines tagged death/feud/scandal/feast/hanging/omen/succession/betrayal/rescue per 200 ticks.
- Target: **≥2 per 200 ticks**. Red flag: **0 in 300 ticks**.
- Basis: RimWorld's design goal — the product is tellable moments; rubric checks "something unexpected happened" and "I cared about an outcome."

**K16. `winter_survival_band`**
- Computed as: `Δ villagers.alive` across `winter.harsh=1` windows as a fraction of pre-winter population, plus stock drawdown check.
- Target: **0–15% population loss, with visible stock drawdown**. Red flag: **>25% loss** (wipe) or **0% loss AND zero stock drawdown** (no stakes).
- Basis: threats must bite without annihilating — RimWorld's ~1.0 baseline tuning; PLAY.md's harsh winter is the designed stress-climax, so it must cost something.

**K17. `multi_horizon_stack`**
- Computed as: count of simultaneously-pending horizons at each sample tick: open ISSUES + countdowns (writ, festival, winter) + in-progress org projects.
- Target: **2–4 pending**. Red flag: **0 for >50 ticks** (nothing to anticipate) or **>6 sustained** (overwhelm).
- Basis: Meier's "multiple irons in the fire" — one-more-turn is literally ≥2 unclosed horizons; >6 exceeds working-memory comfort.

**K18. `consequence_visibility`**
- Computed as: fraction of large environmental drifts (`env.tree_cover` −10%+, `trails.heat` doubling) followed within 50 ticks by a social/economic signal (price move, complaint, new ISSUE).
- Target: **≥0.7**. Red flag: **<0.3**.
- Basis: the *integrated* half of meaningful play — world changes must matter to people. PLAY.md promises the land's score becomes shared troubles; this KPI verifies it.

### OKRs

**O1 — Every headless run sustains a flow channel (goals + feedback + balanced pressure).**
- KR1.1: `decision_density` ∈ [2,6] per 10 ticks in ≥70% of rolling windows across ≥10 seeded runs.
- KR1.2: `event_drought_max` ≤60 ticks in ≥90% of runs.
- KR1.3: `action_yield` ≥0.6 for the scripted organizer bot; `time_to_first_reward` ≤40 ticks in ≥80% of runs.

**O2 — Pressure scales to strength and always leaves a way back (the RimWorld director principle).**
- KR2.1: `threat_rate` within 0.5–1.5× of power-scaled expectation for ≥70% of ticks.
- KR2.2: `recovery_ratio` ≥0.8 after ≥75% of shock events.
- KR2.3: <5% of runs reach village extinction before tick 500; <10% reach tick 500 with zero threat events (both failure modes bounded).

**O3 — The economy is legible and its bottleneck rotates (the Factorio/Settlers principle).**
- KR3.1: `bottleneck_rotation` ≥3 distinct constraints per 500 ticks in ≥70% of runs.
- KR3.2: `price_legibility` CV ∈ [0.05, 0.35] for ≥80% of windows.
- KR3.3: `consequence_visibility` ≥0.7 — the land's score visibly moves people and prices.

**O4 — The grind compounds into visible power (the payoff principle).**
- KR4.1: `time_to_first_member` ≤80 ticks in ≥80% of runs.
- KR4.2: `org_growth_shape` shows ≥3 slope regimes (S-curve) in ≥60% of runs.
- KR4.3: `esteem_lag_gap` >0 during ≥40% of power-rise windows — the dramatic-irony window reliably exists.

**O5 — The world keeps generating stories and open horizons (the retention proxy).**
- KR5.1: `story_event_density` ≥2 per 200 ticks in ≥80% of runs.
- KR5.2: `multi_horizon_stack` ∈ [2,4] at ≥80% of sampled ticks after tick 100 — there's always a next thing.
- KR5.3: `pacing_alternation` passes (peak ≤40, breather ≤60, all classes present) in ≥70% of runs.

### Scoring note
Aggregate per run: `fun_score = weighted_pass_fraction` over the 18 KPIs (weights: O1 25%, O2 25%, O3 20%, O4 15%, O5 15%), with **hard vetoes**: any red flag sustained past its threshold forces review regardless of score, mirroring the rubric's auto-flag convention. Vetoes outrank averages because a single sustained dead stretch (Koster's solved pattern) ruins a run no matter the mean.

## Next Steps

1. **Telemetry contract:** define the event-log taxonomy (which log lines count as "events," which tags map to threat/story/milestone classes) — every KPI above depends on it. Proposed tags: `milestone`, `threat`, `story`, `economy`, `social`, `world`.
2. **Implement the scorer:** a `fun_score` consumer that ingests gauge time series + event log, computes the 18 KPIs in rolling windows, and emits per-OKR pass/fail plus vetoes (there is already a `cargo run --bin fun_score` stub per PLAY.md — extend it).
3. **Calibrate bands on real runs:** the bands above are research-derived priors, not truths. Run ≥20 seeded sims, plot each KPI's distribution, and tighten bands so a deliberately-starved run and a deliberately-blessed run land on opposite sides of every red flag.
4. **Mechanic pilots in priority order:** Mechanic 3 (pressure director) and Mechanic 4 (calendar countdowns) are the cheapest high-leverage wins — both are pure telemetry/event-layer work. Mechanic 1 (pledge board) is the deepest fit to the fantasy but needs the disposition model.
5. **Cross-feed:** hand the bottleneck-identity list (KPI 10) to the ISSUES pipeline (Mechanic 5) so the village's gossip literally narrates whatever the balance tester sees as the binding constraint — dev instrumentation doubling as a diegetic feature.
