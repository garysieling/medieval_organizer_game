# Lit Review: Balance Testing Methodology — How Studios & Researchers Actually Measure "Balance"

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Domain:** Game QA / Game Balance Science
**Series:** Balance-tester spike (user-requested). Feeds the automated balance-tester agent; complements [economist-tester](../agents/economist-tester.md), [exploratory-tester-data-scientist](../agents/exploratory-tester-data-scientist.md), [fun-inspector](../agents/fun-inspector.md), and software-design docs `1000-turn-testing.md`, `ai-player-beta-testing.md`, `telemetry-kpi.md`.
**Confidence:** High — core sources (Schreiber, Sirlin, Riot, Blizzard, King, Ubisoft) are primary documents; academic papers are peer-reviewed

## Summary

"Balance testing" in industry is not one practice — it is five layered ones, and the state of the art is to run all of them as a funnel: (a) **question-driven human playtests** catch what only humans see (confusion, fairness-feel, boredom); (b) **telemetry analytics** turn live or lab sessions into funnels, heatmaps, and cohort curves that locate *where* balance breaks; (c) **synthetic players** — heuristic bots, Monte Carlo self-play, learned "replicant" agents — convert balance questions into distributions over thousands of runs; (d) **analytical methods** (cost curves, intransitive/RPS design, matchup matrices, degenerate-strategy analysis, difficulty curves, control charts) turn the resulting data into *diagnoses*, not just alarms; (e) **published threshold frameworks** (Riot's balance framework, Blizzard's adjusted win-rate bands, Firaxis' handicap-knob difficulty table, King's 1000-run-per-level bot pipeline) show that the mature studios all converge on the same architecture: *named KPIs with pre-registered target bands, a baseline distribution, and a cadence at which out-of-band values trigger review.* The transferable lesson for this sim: balance testing = define the KPI bands first, generate the seed/policy ensemble second, and let control-chart violations drive the tuning backlog.

## Key Sources

### Ian Schreiber & Brenda Romero — *Game Balance* (CRC Press, 2021, 806pp) + free 10-week "Game Balance Concepts" course (gamebalanceconcepts.wordpress.com) — **A+**
The field's only book-length treatment and the canonical syllabus (10 weekly "levels": numeric relationships, cost curves, probability, situational balance, progression & pacing, metrics, intransitive mechanics, game economies). Key findings: every game object should sit on a **cost curve** — if two options give the same payoff for different cost, the curve is broken; **intransitive (RPS) relationships** are the design tool that prevents single-solution degeneracy; player-controlled economies find their own price at the supply/demand crossing *whether or not players see the curves*, and **small populations make prices fluctuate more** (directly relevant: one village ≈ Catan's 4-player volatility, not WoW's auction house). Spreadsheet Monte Carlo is presented as the working designer's instrument — you don't need ML to get a distribution.

### David Sirlin — *Playing to Win* (free book, sirlin.net) + "Balancing Multiplayer Games" Parts 1–3 + "Game Balance and Yomi" — **A**
The degenerate-strategy authority. Key findings: a game is **degenerate** when expert play collapses it to one tactic — "when a game doesn't hold up to expert play, it's degenerate in some way"; the fix is not to complain about players but to fix the game. Defines balance as *viable options* (many meaningful choices) + *fairness* (equal-skill ⇒ equal chance). Practical instruments: **tier lists** from testers (S/A/B/C/F — even a perfectly balanced game produces perceived tiers, that's fine, it's signal not defect), and **matchup charts** in N-notation (6-4 = win 6 of 10 vs. expert counterplay). Yomi's end-state: 210 matchups, *zero* worse than 7-3, point spread +6.5/−6 — that *is* what "balanced" looks like written down. Controversial-but-defensible: for matchup estimation, expert deliberation beats telemetry for speed/accuracy; use numbers to check, experts to propose.

### Joris Dormans — *Engineering Emergence* (PhD, UvA 2012) + Machinations framework / Machinations.io — **A**
Games-as-economies formalism. Six node types (pool, source, drain, converter, trader, gate) + resource/state connections model any internal economy; **feedback loops are the generators of emergence**, and the point of modeling is to find *missing sinks* and *runaway positive loops* before code. The commercial tool operationalizes it: Monte Carlo prediction runs over the diagram, and an **AI-Balancer** that sweeps influencer variables toward a target value — i.e., automated tuning as a search problem over the economy graph. For this repo: the grain/gold/trust/power economies can be sketched as Machinations diagrams and diffed against observed telemetry.

### Ubisoft — GDC 2019 "Automated Testing: Using AI Controlled Players to Test The Division" (Paredes & Jones) + Massive's bot taxonomy — **A**
Two bot species with different jobs: **client bots** take over the actual player, obey every game rule, no cheats — used for mission completion checks, perf gathering, follow-bots so QC can test MP missions solo; **server bots** stress infrastructure. La Forge's framing: "super testers" play millions of games overnight hunting exploits and unstoppable combinations — the designer sets parameters in the evening, reads exploit reports in the morning. Key architectural rule: the bot plays *through the interface*, not around it — matches this repo's `ai-player-beta-testing.md` spec exactly.

### King — Gudmundsson et al., "Human-Like Playtesting with Deep Learning" (IEEE CoG 2018) + Asadi's AI pipeline talks — **A**
The best-documented automated-balance pipeline in production. Supervised CNN trained on real player state-action pairs picks the **most human move** (~94% top-1); the agent then plays every new level **1000+ times in minutes** vs. a 1-week human playtest cycle, and predicted success rates correlate with real player difficulty better than MCTS. Crucial design choice: *human-like, not optimal* play — optimal agents misjudge human difficulty. Now extended with an auto-tweak loop: bot insight → designer refinement. The "persona portfolio" this repo already specs (naive/goal/expert/griefer) is the heuristic version of King's learned personas.

### Riot Games — "/dev: Champion Balance Framework," "/dev: Life of a Patch," Quick Gameplay Thoughts — **A+**
The gold standard: *pre-registered numeric thresholds, published publicly.* Four audience bands (Average ≤Gold / Skilled Plat–GM / Elite D2+ / Pro) each with explicit nerf/buff trigger lines — e.g., Average: nerf if >54.5% WR at low ban rate down to >52.5% WR at >5× average ban rate; buff only if <49% WR; Pro: nerf at >90% presence or >80% across two consecutive patches. Biweekly "State of the Game" meeting reads live-patch data against the thresholds; **every change ships with a target win-rate delta**, then the next patch audits whether the target was hit. Balance as control engineering: sensor (WR by band) → threshold → intervention → verification.

### Blizzard — StarCraft II balance team (David Kim interviews, "The Balancing Act") + Hearthstone Arena micro-adjustments — **A**
Two techniques that matter here. (1) **Adjusted win percentage**: because matchmaking pushes everyone toward 50%, raw win rates *hide* imbalance — Blizzard removes the matchmaker's confound and factors player skill, yielding "true" racial win rates; **45–55% adjusted = no sign of imbalance**. Lesson: never read a raw aggregate when a confounder (here: seed difficulty, policy skill) is in the loop. (2) **Hearthstone Arena micro-adjustments**: a closed-loop algorithm assigns every card a draft weight 0.7–1.3 from observed win rates to equalize class power — *numeric self-balancing*, the direct ancestor of a tuner that perturbs sim constants toward KPI bands. Also: public **Balance Test Maps** = the community as a canary fleet, "feedback based on playtesting is the most helpful information."

### Firaxis — Soren Johnson-era difficulty philosophy + Civ V/VI "Mayhem Level" — **A−**
Difficulty is engineered as a **handicap table**, not smarter AI: production/research bonuses scale by level, "Noble and Prince are the difficulty levels where the AI's rules are closest to the human's." Closed elite beta fed AI iteration (Soren Johnson: direct input from "the world's best Civ players" made iteration faster). Ed Beach's **Mayhem Level**: a live meter of "is something going wrong in the world" — too little mayhem = boredom, the AI generates problems to keep the meter *inside a band*. This is literally the difficulty-band-occupancy KPI below, invented by a shipped game.

### Microsoft GRUR / Bungie — Halo 3 user research (John Hopson, GDC 2009) + heatmaps — **A−**
The scale ceiling of human testing: 35 lab studies, ~1000 testers, 2 years, one-on-one sessions from menu navigation to Scarab-battle comprehension. Their key reporting principles: data turned around in days not weeks, plotted *in designer vocabulary* (death locations on the map — the kill/death **heatmaps** Bungie.net then shipped publicly), and behavioral + self-report data synchronized because "aesthetic experience is based on how users play." Bungie's own caveat is the methodology rule: "simply seeing a choke point doesn't tell you if it's good or bad" — data narrows the search, designers decide.

### Pfau et al., "Dungeons & Replicants II" (IEEE Trans. on Games 2022) — **B+**
Deep Player Behavior Modeling: generative "replicant" agents trained on 213 real players' atomic actions (MMORPG Aion), then run against iteratively-scaled NPC opponents to expose per-class imbalance automatically. Key advance: balance testing needs a *population of playstyles*, not one optimal agent — same lesson as King's human-like model.

### Mahlmann, Togelius & Yannakakis, "Evolving Card Sets Towards Balancing Dominion" (IEEE CEC 2012) — **B+**
Genetic algorithm evolves card sets against three agent skill levels; fitness = **skill differentiation** (good card sets let skilled players win by larger margins). The transferable definition: in a healthy single-player/skill game, *outcome should correlate with player skill* — a balance KPI in its own right (see "skill gradient" KPI below).

### De Oliveira & Chaimowicz, "Investigating RL for Dynamic Difficulty Adjustment" (SBGames 2023) + "Automatic Difficulty Balance in Two-Player Games with DRL" (IEEE CoG 2023) — **B**
Game-Master agents that learn to spend handicap knobs to produce rubber-band tension; win-rate-constrained reward keeps the GM honest. For a single-player sim this maps to: the *world* (winters, writs, events) is the GM, and its knob budget can be tuned toward band occupancy.

### Playtesting methodology — Fullerton, *Game Design Workshop* ch.9 + practitioner guides (Kleenex testing, focused playtests) — **B+**
The human-side canon: every test starts from a *falsifiable question* ("does X happen before Y?"), not a topic; **Kleenex testers are single-use** — fresh eyes is a one-shot resource, never spend it on a known-broken build; the observer stays silent, *the struggle is the finding*; question families: comprehension, discoverability, difficulty/pacing, engagement, preference, robustness. Expert testers for fine-grained deltas; internal testers are fast, biased, and free.

### SPC / anomaly detection practice — Transcend control-chart diagnostics, SPEC ICPE perf-regression control charts, AWS Lookout for Metrics — **B**
Borrowed whole from manufacturing/liveops: per-KPI control chart, baseline = median + **3.5×MAD** limits (MAD over σ for skewed game metrics), signal rules (1 point out / 2-of-3 trend / runs below center) trigger investigation; SPEC's **violation ratio** (fraction of samples out of limit) converts a run into a single regression score. This is exactly how the balance regression gate should decide pass/fail.

### "Assessing Video Game Balance using Autonomous Agents" (IEEE GAS 2023) — **B**
Frames the whole problem the way this spike does: autonomous agents supplement playtesting to (1) compare difficulty across game *versions* (regression) and (2) separate **skill demand from luck demand** — which is precisely the seed-variance-vs-policy-variance decomposition in the KPI section.

## (a) Human Playtesting Practice

- **Question-first protocol.** "Test the tutorial" is a topic; "do players understand recruiting needs trust>0.5 before their second winter?" is a test. Practitioner consensus (Fullerton; gamedesign.gg) is that tests built on topics produce notes nobody acts on.
- **Fresh-eyes is single-use.** Kleenex testers see the game once, ever. Budget them like gold: vertical-slice or better, blind + think-aloud, 5–8 per iteration. Expert testers (repeat players) answer fine-grained questions — "is the 4s or 5s shield better?" — and are *graduated* Kleenex testers.
- **Observer silence.** The single most common protocol error is helping. If the player is stuck, *that is the finding* — record where and for how long.
- **The community is the biggest lab.** Blizzard ships Balance Test Maps; Firaxis ran closed elite betas for AI iteration. A headless bot fleet is this repo's version of that same move: synthetic fresh-eyes that never expire.

## (b) Analytics / Telemetry-Driven Balancing

- **Funnels & cohorts**: the live-ops grammar — acquisition funnel steps, D1/D7/D30 cohort curves — maps onto *progression* here: the funnel is Talk→Recruit→Organize→Power; the cohort is a seed; the "retention curve" is villagers.alive over days.
- **Heatmaps**: Bungie's kill/death location plots are the canonical example — spatial density of events as a balance instrument. Here: trail heat already *is* a heatmap gauge (trails.heat); event-log events could be location-tagged for a village heatmap of starvation/sanction incidents.
- **Deconfounding is the job.** Blizzard's adjusted win-rate exists because the matchmaker corrupts the raw signal. Same trap here: a policy that plays well *masks* a broken economy; a harsh-winter seed inflates mortality that's actually a price bug. Every KPI below is computed *within* a (seed, policy) cell, never pooled across them.
- **Thresholds beat vibes.** Riot's framework works because the trigger lines are written down *before* the data arrives — pre-registration prevents moving goalposts. The bands in the KPI section are this repo's equivalent.
- **Closed-loop tuning exists.** Hearthstone Arena's micro-adjustment algorithm (per-card weights 0.7–1.3 from observed win rates) and Machinations' AI-Balancer both prove that "telemetry → perturb constants → re-measure" can be automated; humans should gate the *proposals*, not do the sweeping.

## (c) Automated / Synthetic Testing

- **Layered pyramid** (industry-standard tiering, cf. CAREC CI docs, Andrew Altimit's QA layer-cake): unit → determinism/replay → scenario gates → **soak** → synthetic players. Cheap layers run per-PR; soak and synthetic balance runs run nightly/scheduled. This repo already has the bottom tiers (`playtest_gate` = scenario gate + frame checks; `sim500`/`mega_sim`/`hard_winter` = soak harnesses).
- **Bot faithfulness spectrum.** Ubisoft's lesson: client bots that play *through the interface* with no cheats answer "does the game work for a player"; the moment a bot reads hidden state, its balance findings stop transferring to humans.
- **Human-like > optimal.** King's central result — difficulty estimates only correlate with real players when the agent plays like a player (supervised on human data, ~94% human-move rate), not when it plays optimally. For this sim: the persona portfolio in `ai-player-beta-testing.md` (naive / goal-oriented / experienced / griefer) is the correct structure; a single "rational" agent would misestimate difficulty in both directions.
- **Monte Carlo is the measurement unit.** Mahlmann et al. evolve toward balanced card sets; Blizzard/King/Ubisoft all count in *distributions*. Sample-size math: for a binary outcome (survived year 1), SE = √(p(1−p)/N) → **N=400 runs for ±5% CI at p=0.5; N=100 for ±10%**; smoke checks can ship at N=20. Multi-seed parallelism converts calendar time into core count.
- **Soak testing = long-horizon balance.** The wiki's own `1000-turn-testing.md` already encodes the industry's reason: degenerate equilibria, population crashes, runaway inflation, and "a world that just settles" only appear at horizon. Industry adds: leaks, float-precision drift, counter overflow — all invisible below the horizon.
- **Regression gates.** playtest_gate exits non-zero on check failure — already the right shape; the balance extension is comparing *KPI deltas against baseline control limits* instead of fixed thresholds, so tuning that shifts a KPI trips review automatically.

## (d) Analytical Techniques

- **Cost curves** (Schreiber): price every player option on one cost→payoff curve; points off the curve are balance bugs. Here: Talk/Recruit/Invest/Organize/Build/Wait each have a cost (time, gold, trust) and a payoff (members, power, esteem) — the tester should fit and publish the curve, then flag any action >2σ above it.
- **Transitive vs intransitive balance** (Schreiber): rock-paper-scissors cycles prevent single-solution degeneracy. In a single-player sim the "cycle" is between *strategies and threats*: e.g., hoarding grain ↔ harsh winter rewards it but tax levy punishes it; high visibility (full meetings) ↔ builds esteem but invites bailiff attention. The audit question: does every strong loop have a counter-pressure?
- **Dominant/degenerate strategy detection** (Sirlin): run an optimizer probe — if a degenerate policy (e.g., Invest-every-spare-gold, never-Organize) reaches top-quartile outcomes, the game is degenerate *at that horizon* even if it feels rich to a naive player. Degeneracy is a property of the cost/payoff surface, not of taste.
- **Matchup matrices** (Sirlin): N-notation grid of A-vs-B. Recast for single-player: **policy × scenario** matrix (persona × seed class: normal/harsh winter, early writ, flu year). A balanced sim has no 10-0 columns (no guaranteed-loss scenario) and no 10-0 row (no always-wins policy). Yomi's "0 matchups ≥7-3" is the acceptance bar shape.
- **Difficulty curves / flow channel** (Csikszentmihalyi; Firaxis mayhem band): challenge should track capability inside a band — too far under = grind (fun-rubric red flags), too far over = collapse. Firaxis literally meters "mayhem" and generates problems to keep it in-band; we meter it and *report* it.
- **Statistical process control**: median + 3.5×MAD control limits per KPI over the baseline ensemble; a new build is scored by its **violation ratio** (fraction of (KPI, window) cells out of limits). Turns "did balance regress?" into a number.

## (e) How Specific Studios Do It

| Studio | Instrument | Transferable piece |
|---|---|---|
| **Riot (LoL)** | Published 4-band threshold framework, biweekly State-of-Game cadence, per-change WR-delta targets audited next patch | Pre-register KPI bands; every tuning PR declares its expected KPI delta and is audited |
| **Blizzard (SC2)** | Adjusted win% (skill/matchmaker confound removed), 45–55% band, public balance-test maps | Deconfound by (seed, policy) cell; ship a "test build" the balance fleet runs before tuning lands |
| **Blizzard (Hearthstone Arena)** | Per-card draft weights 0.7–1.3 auto-computed from win rates | Closed-loop tuner: telemetry → proposed constant deltas → human gate |
| **Firaxis (Civ)** | Handicap-knob difficulty table (Noble/Prince = parity), elite closed beta, Mayhem Level | Difficulty knobs as a swept table; mayhem-band occupancy as the master KPI |
| **King (Candy Crush)** | DL human-like playtester, 1000+ runs/level, auto-tweak assist | Persona realism; massive-sample playtests as the unit of evidence |
| **Ubisoft (Division)** | Client bots through real interface; overnight exploit hunts; La Forge commit-assistant | Interface-faithful bots; "designer sets knobs, reads report in the morning" cadence |
| **DeepMind × Blizzard (SC2)** | AlphaStar as anonymous ladder canary | A strong external agent as a live probe for degenerate strategies |
| **MS GRUR / Bungie (Halo 3)** | 35 lab studies, kill/death heatmaps, designer-vocabulary reports | Behavioral data + self-report synchronized; report in the team's language |

## Field Guide: Which Technique Answers Which Question

The balance tester's decision matrix — the research converges on *different instruments for different questions*, and using the wrong layer is the classic failure (using telemetry to answer "is it fun?", using humans to answer "is the price curve stable"):

| Balance question | Right instrument | Why not the others |
|---|---|---|
| Is the economy stable? | Telemetry KPIs + Machinations source/drain audit | Humans can't feel a slow inflation until it breaks |
| Does a dominant strategy exist? | Optimizer/exploit persona, Monte Carlo ensemble | Naive players never find it (Sirlin: scrubs don't explore extremities) |
| Is it too hard for a real player? | Human Kleenex tests + human-like learned persona | Optimal agents misestimate difficulty (King's central result) |
| Did this patch change balance? | Control-chart violation ratio vs baseline ensemble | Single before/after runs drown in seed variance |
| Is the world alive or settling? | Event-type entropy + stuck-fraction KPIs over 1000-tick soak | Short runs can't see equilibrium death |
| Is the challenge curve right? | Band-occupancy KPI + Firaxis knob sweep | "Feels hard" isn't actionable; band % is |
| Are prices fair / is trade degenerate? | Price CV + arbitrage-exploit sub-KPI | Aggregate means hide oscillation and spikes |
| Do catch-up mechanics work? | Conditional-probability KPIs on the event log | The mechanic existing in code ≠ it firing in losing runs |

## Game Mechanic Proposals

### Mechanic 1: The balance tester is a Game Master reading gauges, not a judge of runs
Per Firaxis' Mayhem Level and the CoG Game-Master papers: the tester doesn't ask "was this run good?" — it asks "how much of this run was spent in the challenge band?" Band occupancy (KPI-6 below) is the single number that encodes this game's "purposely hard but fair" design goal.

### Mechanic 2: Cost-curve audit for the six player verbs
Each of Talk/Recruit/Invest/Organize/Build/Wait gets a measured (cost, payoff) point per persona; publish the curve in `balance-tuning.md` (economist-tester convention). Any verb >2σ above the fitted curve is a Sirlin-degenerate candidate; any verb >2σ below is dead content.

### Mechanic 3: Policy × scenario matchup matrix
Personas (naive, goal-oriented, experienced, exploit) × scenario classes (seed cluster: normal winter, harsh winter, early levy, flu year) — the single-player recast of Sirlin's matchup chart. Acceptance: no column 10-0 (unwinnable scenario), no row 10-0 (dominant strategy), interior cells within [3-7, 7-3].

### Mechanic 4: Machinations-style source/drain audit
Sketch the sim's four economies (food, gold, trust→members, power→esteem) as pools/sources/drains; the balance report must name *every pool's drains*. Missing drain = guaranteed inflation (e.g., if trust has no decay source, esteem inflation is a math fact, not a bug).

### Mechanic 5: Rubber-band verification
PLAY.md promises "world events that can help struggling players" — the tester must verify they *fire*: measure P(recovery event | run below survival floor) and P(recovery | event). A catch-up mechanic that doesn't measurably trigger is decoration.

### Mechanic 6: Difficulty-knob response surface
Firaxis method: enumerate the handicap knobs (winter severity, levy size, lord suspicion rate, price elasticity), sweep each ±25%, fit the response surface of band-occupancy and mortality. Output = the tuning table a designer uses to *aim* difficulty rather than guess it.

### Mechanic 7: Synthetic fresh-eyes
The Kleenex principle, synthetically: a persona with no prior on the sim (pure heuristic, no tuned weights) is the permanent fresh-eyes tester — every build gets its "first session" for free. Reserve learned/tuned personas for the expert-tester role.

## Scoring / KPI / OKR System

All KPIs are computable from per-tick gauge samples (`session.rs::telemetry()` emits all 15 gauges; one tick = one day; `YEAR_DAYS = 360`) plus the typed event log (`self.log` entries carry `t{tick}:` prefixes — needs a parse pass into event categories: death[starvation|illness|wolf|...], birth, recruitment, sanction, writ, meeting, recovery-event, market-trade). Sample rate: every tick for gauges, full event log. Every KPI is computed **per (seed, policy) cell** — never pooled (Blizzard adjusted-win-rate lesson).

### KPI table

| # | KPI | Formula | Target band | Red flag |
|---|---|---|---|---|
| 1 | **Price stability** | CV = σ/μ of price.grain/wood/fish per sliding 90-tick season window | 0.05–0.35 | CV>0.6 sustained 30+t (runaway inflation/collapse); CV<0.02 sustained 90+t (dead market). *Justification: Schreiber — small populations fluctuate more; flat = broken pricing.* |
| 2 | **Starvation mortality** | starvation-death events ÷ mean villagers.alive, per 360-tick year | 1–8%/yr normal; ≤25% harsh winter | 0% across all seeds (no stakes — fun-rubric violation); >30% (extinction spiral) |
| 3 | **Hunger pressure** | mean(villagers.hungry / villagers.alive) per season | 0.05–0.30 non-winter; ≤0.60 harsh winter | >0.5 for 2 consecutive non-winter seasons (food system broken); =0 always (hunger not modeled in practice) |
| 4 | **Progress velocity** | tick at which org.members first ≥5 (critical mass), median policy | day 60–180 | >240 (progression gate too hard — violates "grind has payoff"); <30 under naive policy (trust gate decorative) |
| 5 | **Snowball check** | org.members / villagers.alive at day 360 | 0.05–0.40 | >0.5 (org absorbs village — dominant snowball); org.members=0 at day 200 for goal policy |
| 6 | **Difficulty-band occupancy** | fraction of ticks where hunger pressure ∈[0.10,0.50] OR sanctioned>0 OR winter.harsh=1, while villagers.alive ratio ≥0.9 — "challenged but not dying" | 40–65% | <25% (too easy — game is "purposely hard by design"); >75% (relentless — fun-rubric "breather" missing). *Justification: flow channel + Firaxis mayhem band; 40–65 chosen over textbook 30–50 because the design brief is hard.* |
| 7 | **Stuck-state fraction** | fraction of 20-tick windows with (a) all-gauge normalized entropy ≈0 AND (b) zero event-log entries | <5% | any 30+ tick zero-entropy/zero-event run = stuck attractor. *Justification: fun-rubric flags 10-t no-change / 100-t no-event; window tightened because entropy test is stronger.* |
| 8 | **Event cadence / narrative density** | mean events per 30-tick window; distinct event types per year | ≥0.25 events/tick mean; ≥12 distinct types/yr | event-type count declining year-over-year (world settling — the emergent-health rubric's "dead world") |
| 9 | **Stock buffer coverage** | (stock.grain + stock.bread) ÷ villagers.alive ÷ daily-consumption estimate → days of supply | 5–60 days; <5 only entering harsh winter | >90 sustained (food glut trivializes survival); =0 for 30+t outside harsh winter |
| 10 | **Population envelope** | villagers.alive(year N) ÷ villagers.alive(start), for N∈{1,2,3} | 0.75–1.25 per year | >1.5 (unchecked growth → economy can't feed); <0.6 by year 2 (demographic collapse). *Justification: 1000-turn-testing doc's stability checks; medieval demographic range.* |
| 11 | **Ecology trajectory** | env.tree_cover at year 3 ÷ initial | 0.25–0.75 | monotone→0 across all seeds (irreversible depletion, no feedback loop); flat 1.0 (land pressure never binds) |
| 12 | **Esteem lag** | cross-correlation peak lag of org.power→org.lord_esteem | 30–180 ticks | lag≈0 (lord omniscient — contradicts "his estimate lags the truth"); lag>360 or no correlation (esteem system dead) |
| 13 | **Sanction resolution** | mean and max of sanctioned series; sanction events must resolve | mean <0.5·org.members; max <3 concurrent | sanctioned monotonically increasing (ladder of shame becomes a ratchet — absorbing state) |
| 14 | **Skill gradient** | corr(persona skill rank, terminal outcome score) across personas × seeds | ρ ∈ [0.4, 0.9] | ρ≤0 (skill doesn't matter — luck-only game); ρ≈1 AND naive>0.9 success (game trivial). *Justification: Mahlmann skill-differentiation fitness + GAS'23 skill-vs-luck split.* |
| 15 | **Seed lottery** | CV across ≥20 seeds of (day-360 org.members; year-2 alive ratio), per persona | 0.15–0.80 | >1.0 (run outcome is the seed, not the play — unfair-by-design); <0.1 (seeds don't matter — flat world). *Blizzard confound rule: compute within policy.* |
| 16 | **Determinism / replay** | same seed, same build, two runs → per-tick series identical | 0 divergence | any divergence = nondeterminism bug; blocks all other KPIs (replay-test doctrine) |
| 17 | **Exploit margin** | best exploit-policy outcome ÷ median goal-policy outcome | ≤2.0 | >2× = Sirlin degenerate; exploit sub-KPIs: arbitrage gold/tick, recruit-spam members/tick. *Economist-tester's exploit-log feeds this.* |
| 18 | **Rubber-band efficacy** | P(recovery event fires \| alive<floor ∧ hungry>0.4); Δsurvival after event | event P≥0.6 in-floor; post-event survival lift ≥10pp | events never fire in losing runs (mechanic dead); lift ≈0 (fires but doesn't help — decoration) |

### Worked example (one KPI, end to end)

KPI-1, price.grain, seed 42, goal policy, summer window t∈[1,90]: samples p₁…p₉₀ → μ=4.8, σ=1.9 → CV=0.40 → **out of band** (>0.35), and 34 consecutive ticks with CV>0.6 in the prior window → red flag fires. Triage path per the (d) methods: check stock.grain coverage (KPI-9) — if coverage ≈0 for the same window, the price spike is a real shortage (working as intended, maybe tuned); if coverage >60 days while price spikes, the price-formation code is the bug (Schreiber: price should track the supply/demand crossing). The KPI doesn't say what to fix; it says *where to look and why* — the Bungie rule.

### How KPIs combine into the run score

Per run: score = the outcome taxonomy label, not a scalar. {extinction: alive ratio <0.6 by year 2 OR starvation mortality >30%}, {stagnation: stuck fraction ≥5% OR event-type count declining}, {snowball: org share >0.5 OR exploit margin >2×}, else {challenged-sustainable} — and challenged-sustainable runs get a sub-band grade from KPI-6 occupancy (40–65% = on-target). A scalar score would average away exactly the failures that matter (a run can't be "70% extinct").

### SPC / regression rule (per build)

Baseline = the reference ensemble (≥30 seeds × each persona) frozen per release. For each KPI cell: control limits = baseline median ± 3.5×MAD (MAD over σ — skewed sim metrics, per Transcend/SPEC practice). A build's **violation ratio** = fraction of (KPI × window) cells outside limits. **Gate: violation ratio >10% ⇒ balance review; >25% ⇒ regression.** Determinism (KPI-16) is a hard gate — it fails, nothing else counts.

### OKRs

**O1 — Stand up the telemetry-to-KPI harness.**
- KR1.1: headless batch runner executes ≥20 seeds overnight, emitting all 15 gauges + parsed event categories per tick as JSON (extends `telemetry.rs` export; schema versioned).
- KR1.2: deterministic replay verified — same seed twice produces bit-identical series on 100% of seeds tested (KPI-16 green).
- KR1.3: 1000-tick soak run completes without panic on all seeds; run wall-time budgeted and reported.

**O2 — Baseline the village.**
- KR2.1: ≥30-seed baseline ensemble produces median+MAD control limits for all 18 KPIs.
- KR2.2: every run classified into outcome taxonomy — {extinction, stagnation, challenged-sustainable, snowball} — with ensemble proportions published (target: challenged-sustainable ≥50% under goal policy; extinction <20%).
- KR2.3: `balance-baseline.json` committed; the diff-target for all future tuning PRs.

**O3 — Hunt degenerate strategies.**
- KR3.1: ≥4 personas × ≥10 seeds each run in the matchup matrix (M3); matrix published with N-notation cells.
- KR3.2: zero policy beats median outcome by >2× (KPI-17); every exploit found gets a repro + fix proposal in the economist's `exploit-log.md` convention.
- KR3.3: cost-curve table for the six verbs published to `balance-tuning.md`; ≤1 verb >2σ off curve.

**O4 — Pin the difficulty band.**
- KR4.1: median policy spends ≥40% of ticks in the challenged band (KPI-6); stuck fraction <5% (KPI-7).
- KR4.2: starvation mortality inside [1%,25%]/yr in ≥80% of runs (KPI-2); harsh-winter runs measurably harder but survivable by goal policy ≥60% of seeds.
- KR4.3: difficulty-knob response surface (M6) delivered: each knob's ±25% sweep mapped to band-occupancy Δ.

**O5 — Close the regression loop.**
- KR5.1: `playtest_gate` (or sibling `balance_gate`) emits per-KPI deltas vs baseline control limits; exit code honors the violation-ratio gate.
- KR5.2: gate false-positive rate <10% on unchanged-code reruns; every tuning PR carries a before/after KPI table (Riot's per-change-target rule).
- KR5.3: KPI dashboard artifact per nightly run — one page, designer vocabulary, median ± band, violators highlighted.

## Limitations & Caveats

- **Survey bias toward multiplayer PvP.** Most published methodology (Riot, Blizzard, Sirlin, matchup matrices) is written for symmetric competitive games; the single-player recasts here (policy×scenario matrix, skill gradient, band occupancy) are this document's synthesis, not lifted practice. The closest shipped analogs are Firaxis difficulty tables and King's level pipeline — both single-player, both support the recast.
- **Bot-to-human transfer is unproven for this genre.** King's correlation result is for match-3 levels with millions of human samples; a systemic-sim bot's difficulty estimate has no human baseline to validate against yet. The persona portfolio mitigates but doesn't eliminate this — the first human playtests should calibrate which bot-KPI bands match felt difficulty.
- **Bands are hypotheses.** Every numeric target above is a first guess justified by analogy (marked in-line); Riot itself revises thresholds when they stop matching design intent. Expect O2's baseline run to invalidate several bands — that *is* the calibration process working.
- **Event-log parsing is unbuilt.** KPIs 2, 8, 13, 18 assume typed event categories that don't exist yet; until the parser lands those KPIs read as "data unavailable," not failures.

## Next Steps

- Extend `telemetry.rs` export to JSON + event-log parser into typed categories (starvation-death vs illness-death vs sanction vs recovery-event) — KPIs 2/8/13/18 all depend on typed events.
- Add `balance_gate` binary beside `playtest_gate`: ensemble runner + KPI scorer + control-limit comparator; reuse `session::telemetry()` wholesale.
- Persona spec unification: `ai-player-beta-testing.md` already names the five roles; the balance tester needs each as a concrete policy function with a declared skill rank (feeds KPI-14).
- Baseline ensemble run on current build → first `balance-baseline.json` and the outcome-taxonomy proportions; expect the first run to expose which KPI bands are miscailbrated — bands are hypotheses, not scripture (Riot revises thresholds when they stop matching intent).
- Open question for the designer: where is the win/lose boundary? KPI bands currently infer "success" from sustained growth + survival; if the game gets an explicit victory condition (e.g., org reaches N power, or survives M years), re-derive progress-velocity targets from it.
