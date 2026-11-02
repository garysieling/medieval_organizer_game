# Lit Review: Reward / Punishment Cycles — What Makes Reward Schedules Fun vs Cruel vs Boring

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Domain:** Behavioral Psychology / Game Design Ethics
**Series:** Balance-tester design basis — the REWARD STRUCTURE scoring stream
**Confidence:** High on the core science (Skinner, SDT, loss aversion, Gottman ratio are textbook-stable); Medium on telemetry mappings, which are new heuristics to be validated against bot/human runs.

**Time basis:** sim tick ≈ 1 day; `YEAR_DAYS = 360` (`game2-core/src/game/desires.rs:13`); weekly rhythms exist at `tick % 7` (`sim/religion.rs:225`), event gates at `tick % 100/300` (`sim/world_events.rs:32`). All numbers below are in ticks (days) unless noted.

## Summary

The design goal — "a bit challenging, requiring thought/planning, with rewards that occur," training the player without a manual — is a *reinforcement-schedule* problem. The science gives a clear answer:

- **Rewards must be earned but not perfectly predictable.** Variable schedules sustain engagement without the mechanical tedium of fixed ones; but contingency (action → consequence) matters more than raw variability. Reward that arrives regardless of effort teaches nothing — it trains superstition.
- **Punishment must be legible.** Every severe negative event needs a visible antecedent chain (warning → escalation → consequence), mirroring natural/logical-consequence parenting and the sim's existing "ladder of shame" design. Unpredictable punishment produces learned helplessness and rage-quits, not learning.
- **Losses hurt ~2× as much as equivalent gains feel good** (prospect theory). A 1:1 positive:negative event mix *feels* punishing; the felt-neutral point is ~2:1 and relationship research puts the *flourishing* band at 4–5:1.
- **Feedback must be immediate and specific.** Clicker practice shows a marker within ~1 s of the behavior is what makes the consequence attributable; praise must name the behavior, not the trait. In tick terms: consequence within ≤3 ticks, with cause-named text.
- **The casino profile is the anti-target.** Variable-ratio compulsion + near-miss framing + losses-disguised-as-wins produces engagement without satisfaction — addictive-but-hollow. We instrument against it explicitly (LDW count = 0).
- **The counterweight is SDT/flow.** Rewards that serve the player's own goals (autonomy), land just above demonstrated skill (competence/flow channel), and celebrate relationships (relatedness — the village itself) produce durable fun; extrinsic token-showers can *undermine* intrinsic motivation.

## Key Sources

- Ferster & Skinner, *Schedules of Reinforcement* (1957) — the four basic schedules and their cumulative-record signatures; still the reference dataset for how schedule shape drives behavior shape — **A+**
- Skinner (1948), "Superstition in the Pigeon" — non-contingent reinforcement produces ritual behavior; the foundational warning for "rewards that just happen" — **A**
- Partial-reinforcement extinction effect (PREE) literature; gambling persistence studies (Delfabbro et al. 2022 operant-gambling review; James et al. 2016 slot-perseverance experiment; random-ratio early-win analysis, *J. Gambling Stud.* 2008) — **A-/B+**
- Clark et al. (2009, *Neuron*) — near-misses recruit win circuitry, *only under personal control*; Barton et al. (2019, *J. Gambling Stud.*) — pigeon/human replications largely fail; effect is real-but-fragile and agency-gated — **A/B**
- Dixon et al. + systematic review (2017, *J. Gambling Stud.*) on losses disguised as wins (LDWs): players systematically overestimate wins because loss outcomes carry win audiovisuals — **A**
- Schüll, *Addiction by Design* (2012) — the "machine zone": slots engineer uninterrupted dissociation, not fun; the ethnographic account of hollow engagement — **A-**
- Kahneman & Tversky (1979) prospect theory; Tversky & Kahneman (1991) loss-aversion slope ratio ≈ 2:1 — **A+**
- Karen Pryor clicker-training canon (10 Laws of Shaping; timing glossary) + VCA/academic shaping literature + Smith & Davis (2008) basenji study (clicker group took significantly *more* trials to extinguish — marker-trained behavior is more durable) — **A-/B+**
- Gottman & Levenson 5:1 positive:negative ratio during conflict (marriages); parent-child adaptations ~4–5:1; lab study: baseline mother-child dyads ran ~1:1 and improved mostly by *adding positives* — **A/B**
- Dreikurs/Adler natural & logical consequences; CDC parenting guidance (warn once → always follow through); consistency-of-discipline extension literature — **A-/B**
- Mueller & Dweck (1998) — process praise ("you worked hard on that") vs person praise ("you're smart") and post-failure resilience — **A**
- Deci & Ryan SDT (2000); Deci, Koestner & Ryan (1999) meta-analysis of 128 experiments: expected tangible rewards undermine intrinsic motivation (d ≈ −0.28 to −0.40); positive informational feedback enhances it (d ≈ +0.33) — **A+**
- Csikszentmihalyi flow channel; Sweetser & Wyeth GameFlow model (clear goals + immediate feedback + challenge/skill balance + control) — **A**
- Swink, *Game Feel* — feedback immediacy and "traction": the first minutes of flailing need fast, legible response or players put the game down — **B**
- Zagal et al. (2013) dark patterns in games; Gray et al. (2024) dark-pattern ontology; King & Delfabbro predatory monetization; Peticca-Harris et al. predatory-technique taxonomy — **A-/B+**

## (a) Operant Conditioning: The Four Schedules and What Each Feels Like

Ferster & Skinner's cumulative records remain the cleanest behavior-shaping dataset ever produced:

| Schedule | Rule | Response pattern | Felt experience in a game |
|---|---|---|---|
| Fixed Ratio (FR) | reward every N actions | high rate + **post-reinforcement pause**; "ratio strain" at high N | grind wall: "do 10 talks, get 1 recruit" — bursts then fatigue, then abandonment |
| Variable Ratio (VR) | reward after random ~N actions | highest, steadiest rate; minimal pausing; most extinction-resistant | slot-machine compulsion — powerful, dangerous (see b) |
| Fixed Interval (FI) | first response after fixed time pays | **scalloped**: long pause, then accelerating near the deadline | calendar events: taxman, feast day — dead time, then an anticipation spike |
| Variable Interval (VI) | first response after random ~T pays | moderate, **steady** rate, no scallop | the texture of a living world: pedlar, comet, noble cheese-buyer, pale stone |

Two nuances the table hides:

- **FR post-reinforcement pause** is why "collect N widgets → reward" quests feel like work: the organism literally rests after each payoff. Crafting/fetch loops inherit this automatically — acceptable in small doses, fatal as the core loop.
- **FI scalloping** is a feature for dread/anticipation: the yearly taxman produces exactly the accelerating-preparation curve a good debt deadline should. Fixed schedules are the right tool for *obligations*, the wrong tool for *delights*.

**Extinction curves & PREE:** continuously reinforced behavior dies quickly when rewards stop; intermittently reinforced behavior persists far longer — Smith & Davis (2008) found clicker-conditioned dogs needed significantly more trials to hit the extinction criterion than treat-only controls (the *marker* itself had become reinforcing). Double edge: intermittent reward makes a *good* strategy resilient to a bad patch — but also makes a *dead* strategy hard to abandon. That persistence-under-losing is the grind trap, and it is the mechanism casinos monetize.

**Timing window:** reinforcement delay degrades conditioning steeply — the reinforcer must land while the behavior is still attributable. Trainers solve the gap with a *bridging stimulus* (the click marks the exact instant; the treat follows). Game analog: a cause-named event ("the kitchen meeting impressed Maren — she joins") delivered ≤3 ticks after the action is our click.

**Extinction burst:** when a previously reinforced behavior stops paying, the organism *escalates* it — more lever presses, more intensity — before quitting. Expect a player whose strategy just stopped working to hammer it harder; the design should route that energy (see M5), not punish it.

**Superstition:** Skinner's pigeons, fed on a pure timer, developed ritual dances — whatever they happened to be doing at feeding got "reinforced." Non-contingent reward doesn't teach "the world is generous"; it teaches ritual. This is the formal reason `contingency_lift` (K2) exists.

**Game lesson:** build the reward layer as **FI skeleton + VI flesh** — the calendar (feasts ~90-tick spacing, taxman 360, writs ~seasonal, hard winter every-other-year) gives rhythm and dread; VI story beats keep tick-to-tick texture alive. Keep *player-skill* rewards contingent; keep *world-texture* events free.

**Recommended schedule assignment for existing sim systems:**

| Sim system | Schedule | Why |
|---|---|---|
| Feast days, taxman, seasonal writs | **FI** | obligations *should* scallop — anticipation and dread are the point |
| Pedlar, circus, noble buyer, comet, pale stone, letter | **VI** | world texture; steady low-level engagement, no exploitable timing |
| Recruit/talk success | **contingent, thinned** | early CRF (acquisition) → leaner as the player's skill is proven |
| Economy payoffs (sell, trade, cheese aging, perfumer flasks) | **FR-ish, small N** | honest work-loop reward; keep N small so post-reinforcement pauses stay short |
| Ladder of shame / writ escalation | **FI-like, player-triggered** | punishment must be *more* predictable than reward — see (d) |
| Truth ripening, grave-vigil, masterwork objects | **VI, long mean** | rare epoch-level beats; their scarcity is their weight |
| Rescue/world-events for struggling players | **VI, ramped** | helps without becoming a readable pattern |

## (b) Casino Design: The Addictive-but-Hollow Profile to AVOID

The casino is the perfected reinforcement machine — and precisely the wrong goal. Its lessons, inverted:

- **VR schedules** are why slots sustain play: maximum persistence per unit reward. Random-ratio research adds that *early wins* disproportionately set the persistence prior — the first-session experience writes the expectation.
- **Near-miss effect:** near-misses are subjectively unpleasant yet increase desire to play, recruiting win-circuitry (insula/striatum) — *only when the player has personal control* (Clark 2009). Replications are mixed (Barton 2019). Read: a near-miss on a **skill** outcome ("recruit almost landed — trust 0.47/0.5") is honest proximity information and probably motivates; a near-miss on a **pure-RNG** outcome is casino dressing and should not exist in our log.
- **Losses disguised as wins (LDWs):** multiline slots celebrate net losses with win audiovisuals; players reliably overestimate their win rate. In our terms: any positive-flagged event whose net gauge delta is negative is an LDW. The "bargain" that costs more than it saves but plays a victory sting is one. Instrument to zero (K8).
- **Loss aversion:** losses loom ~2× larger than gains (Tversky & Kahneman 1991). Even honest design needs ≥2:1 positive:negative felt-valence before a run reads as "rewards occur." A villager death needs roughly two genuine celebration events to net out emotionally.
- **The machine zone** (Schüll): what slots sell is not excitement but *uninterrupted dissociation* — the removal of friction, decision, and consequence. That is the exact opposite of "requiring thought/planning." The tell-tale: engagement persists while reported fun craters.
- **Predatory structure:** King & Delfabbro's catalog — disguised true costs, commitment-then-price-reveal, play-to-skip-grind — all share the signature of *designing against the player's informed interest*.

**Game lesson:** steal the *strength* of variable schedules (steady engagement, drought resilience) and refuse the *tricks* — no celebrated losses, no near-miss theater on non-skill rolls, no absence-punishing FOMO timers, no cost that hides itself until the player is committed. Early positive experience is legitimate design (traction, first-session prior), not a hook — the difference is that ours is *earned* and *honest*.

## (c) Dog Training Practice: How to Actually Train Without a Manual

Applied animal training is operant conditioning that has survived a hundred years of contact with reality:

- **Marker timing:** the click must land at the instant of the criterion behavior — "take a picture of the rear touching the ground." The treat follows fast; the *click* does the attribution work. Without the marker, a delayed treat reinforces whatever the dog was doing last — usually the wrong thing.
- **Positive reinforcement dominance:** aversive-heavy training produces fear, avoidance of the *trainer*, and suppression of exploration. Punishment suppresses behavior in the punisher's presence; it never teaches the alternative. For a game: harsh mechanics teach players to avoid *the game*, not to avoid the mistake.
- **CRF for acquisition, intermittent for maintenance:** reward *every* successful approximation while the behavior is being learned (early Talks should almost always yield something — a want, a secret, a bond nudge, a gossip thread), then thin the schedule once the loop is established. The modern shaping guidance (Pryor's revised laws) explicitly dropped early schedule-thinning: get the behavior fluent first.
- **Criterion raising in small steps:** raise the requirement ~10–20% at a time; each raise briefly depresses performance. Jump criteria too fast and you get extinction bursts, then quitting. Raise too slow and you get a plateaued animal that never develops.
- **Extinction bursts are expected, even usable:** trainers know the post-criterion-raise flare of effort ("surfing the burst" is now considered ethically dicey, but the *expectation* is not). When failures cluster, the handler's job is to find an adjacent criterion the animal *can* hit and pay it.
- **Jackpots and session ends:** occasional outsized rewards keep affect high; end sessions on success — a session ending in failure leaves avoidance conditioning attached to the whole enterprise.
- **Consistency:** a cue that pays randomly *and* punishes randomly is noise, not a schedule. Poisoned cues (same stimulus predicting both reward and aversive) produce animals that refuse the cue entirely — the game-design version is an action that unpredictably helps or harms with no legible difference.

**Game lesson:** the player is the dog, the sim is the trainer. Early game = CRF acquisition (every Talk yields info); mid game = thinned schedules + raised criteria (recruit a skeptic, bargain a writ, survive a summons); always = fast, cause-named, honestly-signed feedback. Failure spikes after the player levels up are *expected* — route them, don't read them as a difficulty bug.

**Minimum feedback signature per verb** (Swink's "nothing is ever silent"):

| Player action | Minimum immediate response | Earned payoff (contingent) |
|---|---|---|
| Talk | response text + bond/secret nudge this tick | want/problem revealed; issue converges after 3 voices |
| Recruit | explicit trust readout + success/fail text | member joins — crowd-visible at next meeting |
| Invest | gold debit + counterparty reaction | influence/esteem tick up; contract offer later |
| Organize | attendee list renders; meeting mood | issue crystallizes; committee; action unlocked |
| Build | construction marker + progress per tick | shop/hall opens; power claim the street can see |
| Wait | world visibly moves (walkers, prices, weather) | — deliberately unrewarded; waiting is not a strategy |

## (d) Child-Rearing Evidence: Fair Consequences

- **Gottman 5:1:** stable marriages run ≥5 positive:negative interactions *during conflict*; ≤1:1 predicts dissolution. Parent-child adaptations suggest ~4–5:1. Tellingly, observed dyads sit near 1:1 at baseline, and those coached toward 5:1 got there mostly by *adding positives* — not by suppressing negatives. For our scorer: `reward_ratio` targets [2.5, 5.0] (loss aversion sets the floor at ~2:1; Gottman sets the flourish band), and the tuning lever is *more positive beats*, not softer winters.
- **Specific vs generic praise (Mueller & Dweck 1998):** process praise ("you brought food before the snow — the stores held") preserves motivation and strategy-seeking after failure; person praise ("you're a natural leader") makes failure feel diagnostic of the self and collapses effort. Generic praise ("good job!") is barely better than nothing — it carries no attribution information. Our event text must name the *act and its consequence*.
- **Natural/logical consequences (Dreikurs/Adler):** effective consequences are *related* to the act, *revealed in advance*, *reasonable* in scale, and delivered without humiliation. The child who won't wear a coat gets cold (natural — the world punishes); the child who breaks the rule loses the *related* privilege (logical — the parent enforces a pre-announced, thematically-linked consequence). Miss dinner for undone homework fails the test: unrelated, and it isn't a consequence the child caused.
- **Predictable consequences & follow-through:** CDC guidance — once a warning is issued, *always* follow through. Inconsistency teaches that rules are noise. Worse: intermittently enforced prohibitions intermittently reinforce the misbehavior — the single most extinction-resistant pattern in the entire literature, accidentally applied to the behavior you wanted least.
- **The escalation logic:** consequences should ladder — reminder, warning, related consequence, escalating repeat — so that severity is always *earned visibly*. The sim's own "ladder of shame" (bad word → cold shoulder → boycott → crowd → stocks → the road) is already this pattern, which is why it reads as fair: each rung is a warning for the next.

**Game lesson:** punishment in the sim should work like good discipline — the writ ladder (levy → double levy → knights → hanging) is revealed, escalates through visible rungs, always follows through, and is *related* to the player's defiance. The yearly taxman is a legible FI punishment: annoying, anticipated, fair — never arbitrary. The design sin is not harshness; it is *unheralded* harshness.

## (e) The Counterweight: SDT, Flow, Juice, and Dark-Pattern Hygiene

Everything above is mechanism; this section is the conscience.

- **SDT (Deci & Ryan):** durable motivation feeds three needs — **autonomy** (my choices matter), **competence** (I can get better at this), **relatedness** (I belong / I matter to someone). The Deci-Koestner-Ryan meta-analysis (128 experiments) found expected tangible rewards *undermine* intrinsic motivation (d ≈ −0.28 to −0.40); positive informational feedback *enhances* it (d ≈ +0.33). Translation: rewards that *inform competence* ("your org held the winter; the village remembers") beat rewards that merely *pay out* (gold shower). A reward economy built on payouts can hollow out the very motivation it was meant to sustain — the overjustification effect.
- **Flow channel:** keep challenge just above demonstrated skill; below → boredom, above → anxiety. Player-level proxy for our scorer: action success rate held in ~0.4–0.7. The channel is *diagonal* — difficulty must climb as capability climbs (K6's `shaping_slope`).
- **Juice / feedback latency / traction:** Swink — the first minutes of any game are pure flailing; the player is owed *immediate, clear, useful* feedback or they put the game down and never return. First earned success early; every action produces a visible response within 1–3 ticks; nothing the player does is ever fully silent. Our six verbs (Talk/Recruit/Invest/Organize/Build/Wait) must each have a minimum feedback signature — even Wait shows the village moving.
- **Ethical engagement vs dark patterns:** Zagal's game taxonomy + Gray's ontology + the predatory-monetization literature converge on one test: does the pattern serve the *player's* informed interest, or does it subvert choice to extract attention/money/time? Our blacklist, with detection:
  - *Grinding for grind's sake* — long FR walls with no variability → detected by K1 mean spikes + low CV.
  - *Manufactured FOMO* — timers that punish *absence* rather than reward presence → design audit, not telemetry.
  - *LDWs* — celebrated net-losses → K8, must be 0.
  - *Near-miss theater on non-skill outcomes* → event-text audit: proximity framing allowed only when the outcome was skill-contingent.
  - *Hidden cost / commitment traps* — invest mechanics must disclose terms up front.
  - *Non-contingent reward noise* — free wins that train superstition → K2 lift <1.2 flags it.
- **The intrinsic-alignment test:** ask of every reward, "does this serve the player's goal, or the retention metric?" A village sim has no monetization pressure — every reward can be honest. That is a design luxury; the scoring system's job is to keep it that way as mechanics accumulate.

**Game lesson:** the deepest reward is watching the village *respond* — people power made visible (relatedness + competence), the lord's estimate lagging then conceding (competence + autonomy), a truth ripening and the org standing with its own. Extrinsic resource rewards are seasoning; competence-information and belonging are the meal.

## Sim Event Valence Taxonomy (for the scorers)

Concrete events from PLAY.md, classified for valence-tagging. Severity: 1 = minor, 2 = significant, 3 = run-defining.

| Event | Valence | Severity | Typically earned? |
|---|---|---|---|
| pedlar arrives / gossip both ways | + | 1 | no (VI texture) |
| feast day (St. Swithin's, Harvest Home, Candlemas, St. Martin's) | + | 2 | no (FI anchor) |
| recruit succeeds / member joins | + | 2 | yes — action |
| truth ripens and org stands with its own | + | 3 | yes — org strength |
| writ bargained down / summons resisted | + | 2–3 | yes — org power |
| noble pays triple for aged cheese / letter lands | + | 1 | partial — economy |
| comet/eclipse read favorably, pale stone found | + | 1–2 | no (world) |
| child comes of age into the trade/org | + | 2 | partial — long investment |
| hard winter survived with stores shared | + | 3 | yes — planning |
| taxman takes his fifth | − | 1 | n/a — FI obligation |
| bailiff's writ (levy/fee/summons) | − | 2 | caused by standing |
| writ doubles after defiance | − | 2 | caused — warning built in |
| knights / hanging after third defiance | − | 3 | caused — fully laddered |
| spoiled bread, mice in grain, cursed bread blame | − | 1–2 | partial — stocking neglect |
| wolf/fox raid, Saint Anthony's fire | − | 1–2 | partial — world |
| stocks / boycott / the road out (ladder of shame) | − | 2–3 | caused — blame visible |
| starvation/illness death | − | 2–3 | caused — stock/hunger traceable |
| loneliest leave with the circus | − | 1 | partial — neglect signal |
| sanctioned status | − | 2 | caused |

## Game Mechanic Proposals

### M1: FI skeleton + VI flesh reward calendar
Calendar events (feasts ~90-tick spacing, taxman 360, writs ~seasonal, hard winter every-other-year) are fixed-interval anchors; all positive texture events are jittered variable-interval. **Variables:** `vi_mean_gap` (15–45 ticks across all positive beats), `vi_cv` (0.5–1.2), `min_gap` (no two positive beats <5 ticks — clumping reads as noise), `early_win_guarantee` (first earned positive ≤60 ticks), `calendar_anchor_count` (4–6 FI anchors/year).

### M2: Successive-approximation skill ladder (shaping)
Organizing skill tiers: talk → first recruit → house meeting → committee → bargain a writ → defy the castle. Each tier unlocks on demonstrated success at the prior; each tier's first attempts run ~CRF (informative success likely), then thin. **Variables:** `tier_success_gate` (1 success to unlock early tiers, 2 late), `step_difficulty_delta` (≤20% harder per step), `acquisition_yield` (early-tier success ≈0.7–0.9), `maintenance_yield` (0.4–0.6), `burst_assist` (after N consecutive failures, surface an adjacent reachable success).

### M3: Legible escalation ladder for punishment
All severe negatives carry a warning chain: rumor/grievance event → gauge trend (lord_esteem falling, blame accumulating) → consequence. Writs escalate levy → double → knights with bargaining windows between. **Variables:** `warning_lead_ticks` (7–45), `escalation_steps` (3–5 before severity-3), `bargain_window_ticks` (≥14 — a real decision space), `severity_cap_per_step`, `warning_salience` (warning must appear in the event log, not just gauge drift).

### M4: Celebration moments / honest juice
Positive events render same-or-next-tick with cause-named text and a visible world response (crowd at a full kitchen meeting; painted hall as a power claim that fades; toasts at feasts where truths slip out). Magnitude scales with rarity. **Variables:** `feedback_latency_ticks` (≤3), `specificity` (fraction of reward events naming their cause — target ≥0.9), `celebration_tiers` (3: nudge / event / epoch), `rarity_magnitude_coupling` (rare events get big presentation).

### M5: Drought-breaker ("end on success")
When positive-event drought exceeds threshold, raise the probability of *reachable-opportunity* events — a villager approaches the player, pedlar gossip reveals a live want, a struggling-player world event fires (already in design). Rescue gives *options/information*, never free wins, and itself arrives on VI so it can't be gamed. **Variables:** `drought_threshold` (~90 ticks), `rescue_prob_per_tick` (ramps after threshold), `rescue_type` (info/option only — never directly gauge-positive), `rescue_max_per_year` (≤3 — reliable rescue is just a fixed schedule in disguise).

### M6: Contingency integrity (anti-superstition)
Baseline windfall rate kept low; the majority of positive events must be traceable to a player action within K ticks. Skinner's superstitious pigeons are the warning: free reward trains ritual, not skill. (Rumors/omens/comets are exempt — they're world-texture; villagers *should* be superstitious even if the player shouldn't be.) **Variables:** `earned_fraction` (≥0.7 of positive events action-attributable), `windfall_rate` (≤3/year unearned), `attribution_window` (K ≈ 5 ticks).

## Scoring / KPI / OKR System

All metrics computable from tick telemetry + event log. Assumed log schema: each event has `{tick, type, valence(+/-/0), severity(1–3), cause(ref to action/event id | null)}`. Gauges sampled per tick: `villagers.alive`, `villagers.hungry`, `org.members`, `org.power`, `org.lord_esteem`, `sanctioned`, `stock.grain`, `stock.bread`, `price.*`, `winter.harsh`. **If the current log lacks valence/severity/cause fields, add them first** — every scorer below depends on attribution.

### KPI table

| # | Metric | Formula | Target band | Red flag |
|---|---|---|---|---|
| K1 | `reward_interval_distribution` | intervals `i_k` between consecutive valence+ events; report `mean(i)` and `CV = std(i)/mean(i)` | mean ∈ [15, 45] ticks (≈8–24 beats/yr); CV ∈ [0.5, 1.2] | mean >90 (<4 beats/yr — starvation); CV <0.2 (metronome FI — scalloped boredom); CV >2.0 (feast-famine) |
| K2 | `contingency_lift` | `P(valence+ within K=5 ticks after a player action) / P(valence+ in a random 5-tick window)`; companion `action_yield` = fraction of player actions producing *any* logged consequence within K | lift ∈ [1.5, 4.0]; yield ≥0.6 | lift <1.2 → rewards arrive regardless of effort (superstition/meaningless); yield <0.4 → actions are dead buttons |
| K3 | `reward_ratio` | `count(valence+) / count(valence−)` per game-year | [2.5, 5.0] (loss-aversion floor ~2:1; Gottman flourish ~5:1) | <2.0 → feels punishing even when "balanced"; >8 → no stakes, wallpaper positivity |
| K4 | `punishment_predictability` | `warned_severe / total_severe`: fraction of severity≥2 negatives preceded by a warning event or relevant gauge trend within W=30 ticks | ≥0.8 for severity-3 (death/hanging/exile); ≥0.6 for all negatives | severity-3 with no antecedent in >15% of runs → arbitrary → helplessness/rage-quit |
| K5 | `extinction_risk` | `drought_max` = longest valence+ gap; `drought_p95`; `first_win_tick` = first *action-attributable* valence+ | drought_max ≤90; drought_p95 ≤60; first_win_tick ≤60 | drought >120 (fun-rubric's 100-tick dead-air flag is the floor); no positive by tick 90 → quit zone |
| K6 | `shaping_slope` | per 60-tick window: `success_rate` = successful actions / attempts; difficulty index = neg-event rate + writ severity + `winter.harsh`; `slope = Δdifficulty / Δcapability` where capability = org.members + gold accumulation rate | success_rate ∈ [0.4, 0.7] in ≥80% of windows; slope >0 | success <0.2 for ≥2 consecutive windows (impossible wall); >0.9 (treadmill); slope ≤0 while capability grows (flat difficulty = grind) |
| K7 | `fairness_index` | `attributable_neg / total_neg`: negatives with causal antecedent in log or gauge deltas (starvation death ← stock.grain low + hungry rising; crackdown ← defy count; spoiled stock ← mice/damp flag) | ≥0.8 | >20% exogenous/unexplained negatives → "the game cheats" perception |
| K8 | `ldw_count` | count of valence+ events whose net gauge delta is negative (celebrated loss) OR valence+ with `cause=null` and no player action in window and no world-event flag | **= 0** | >0 → dark-pattern leak; fix before tuning anything else |
| K9 | `warning_lead_time` | median ticks between warning event and its consequence | [7, 45] | <3 → no decision space (agency theater); >90 → warning forgotten; consequence reads as unrelated |

**Derivation notes:**

- *Classifying events:* use the valence taxonomy table above; where the log can't carry valence, derive it from gauge deltas (Δorg.members >0 → +; Δvillagers.alive <0 → −; Δlord_esteem sign) with event-type overrides for texture (feasts are + regardless of delta).
- *Detecting warnings:* a warning is any logged event of type rumor/grievance/writ-notice/blame that shares a causal-thread id or target with the subsequent negative, OR a monotonic gauge trend (lord_esteem falling ≥5 of last 10 ticks; stock.grain below subsistence threshold while hungry rising).
- *Statistics:* need ≥8 valence+ intervals per run for a meaningful CV; report KPI distributions across ≥10 seeded runs (median + P90), not single-run values — the schedules themselves are stochastic, so the scorer must be too.
- *Weighting:* positive events are not equal (a comet ≠ a recruit). v1 scores counts; v2 should weight by severity. Flag for iteration.
- *Reference implementation sketch:*

```python
events = [e for e in log if e.valence in ("+","-")]
pos   = [e.tick for e in events if e.valence == "+"]
gaps  = diff(pos)                       # K1: mean, std/mean
warned = [e for e in events if e.valence=="-" and e.severity>=2
          and any(w.is_warning and w.thread == e.thread
                  and 0 < e.tick - w.tick <= 30 for w in log)]   # K4
drought = max(gaps)                     # K5
lift = rate(pos_after_action, K=5) / baseline_rate(pos, window=5)  # K2
ldw  = [e for e in events if e.valence=="+"
        and net_gauge_delta(e) < 0]     # K8 must be []
```

- *Failure-mode map* — what each red flag is actually diagnosing:

| Red flag fires | Diagnosis | Literature anchor |
|---|---|---|
| K1 CV <0.2 | metronome rewards — predictable, scalloped, boring | FI scallop (Ferster & Skinner) |
| K1 mean >90 / K5 drought >120 | reward starvation — cruelty by omission | extinction curves; fun-rubric dead-air flag |
| K2 lift <1.2 | superstition economy — ritual, not skill | Skinner 1948 |
| K4 <0.8 | arbitrary punishment — helplessness, rage-quit | natural/logical consequence canon |
| K7 <0.8 | illegible causality — "the game cheats" | attribution theory via consequence literature |
| K8 >0 | celebrated losses — casino DNA in the build | LDW systematic review |
| K3 <2.0 | net-negative felt valence — "a bit challenging" became "a bit hostile" | loss aversion 2:1 + Gottman 5:1 |
| K6 success <0.2 or >0.9 | anxiety wall / boredom treadmill | flow channel |

**Interpretation:** K1+K5 together separate "variable" (moderate mean, CV~1, bounded max) from "fixed" (CV~0) and "cruel" (long right tail). K2+K8 are the honesty pair — rewards must be earned and honestly signed. K4+K7+K9 are the fairness triad — punishment must be preceded, caused, and actionable. K3+K6 hold felt-valence and challenge in the flow band.

### OKRs

- **O1 — Rewards occur, variably.** KR1.1: `reward_interval_distribution` in band (mean 15–45, CV 0.5–1.2) in ≥9/10 seeded bot runs. KR1.2: ≥8 valence+ events per game-year.
- **O2 — Punishment is fair.** KR2.1: `punishment_predictability` ≥0.8 on severity-3; `fairness_index` ≥0.8; `warning_lead_time` ∈ [7,45].
- **O3 — Early traction.** KR3.1: `first_win_tick` ≤60 in ≥9/10 runs. KR3.2: `contingency_lift` ≥1.5; `action_yield` ≥0.6.
- **O4 — The flow band holds.** KR4.1: action `success_rate` ∈ [0.4, 0.7] in ≥80% of 60-tick windows. KR4.2: `drought_p95` ≤60 ticks.
- **O5 — Zero dark patterns.** KR5.1: `ldw_count` = 0 in every run. KR5.2: `reward_ratio` ∈ [2.5, 5.0]. KR5.3: design audit — no event that fires *only* because the player was idle/absent ships (no FOMO penalties).

## Next Steps

- **Instrumentation:** extend the event log with `valence`, `severity`, `cause` fields if absent — every scorer depends on attribution. Add a causal-thread id so warning→consequence chains are first-class.
- **Calibration:** run `playtest`/`game_bots` seeded runs; compute K1–K9; locate the current sim. Expected gaps: K1 mean probably too high before first recruit; K4 likely low if warnings aren't logged as events; K2 unmeasurable until `cause` exists.
- **Validate targets against humans:** bot runs tune the schedule; a short human playtest confirms whether "fair + variable" reads as fun. The rubric's "would you play 10 more minutes" is the ground truth the KPIs proxy.
- **Cross-link:** pairs with `fun-rubric.md` (K5 inherits its 100-tick dead-air flag), `sprint-4-psychology-of-play.md` (flow targets), `systems-thinking-dynamics.md` (fairness = legible causal structure), and `sprint-4-attachment.md` (villager bonds are the relatedness reward).
- **Open question:** severity-weighted `reward_ratio` (a comet is not a recruit) and a `valence_authenticity` check on event text (does the copy name the cause?) are v2 metrics once severity tagging lands.
