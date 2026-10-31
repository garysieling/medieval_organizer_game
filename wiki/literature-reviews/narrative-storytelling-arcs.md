# Lit Review: Storytelling Arcs — Design Basis for Scoring Narrative Quality in Sim Runs

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Domain:** Narratology / Interactive Narrative AI / Game Onboarding
**Series:** Balance-tester research spike; feeds arc detection in `game2-core/src/sim/story.rs` and run-scoring KPIs
**Confidence:** High on classical structures and drama-management literature; medium on quantitative pacing targets (few papers give hard numbers; RimWorld's datamined constants are the best empirical anchor)

## Summary

The sim already produces a rich event stream (births, deaths, illness, betrayal, recruitment, crackdowns, writs, hangings, vigils) but `story.rs::detect_story` only does *kind-membership* classification — "contains Illness + Recovery ⇒ Survival" — with no notion of *shape*. The research converges on the same fix from three directions:

1. **Classical narratology** says a story is a *trajectory*, not a bag of events: rising tension, a climax placed late (≈55–95% of duration per Save the Cat / Freytag), then release. Reagan et al. (2016) proved at corpus scale (1,327 novels) that emotional arcs collapse to **six shapes** — so "does this run have a shape?" is a computable question.
2. **Non-Western structures** (kishōtenketsu, soap braiding) free us from requiring *conflict*: a "ten" twist that recontextualizes (the sim's Discovery events — the lord's-fifth-is-really-a-quarter) is a legitimate arc engine, and braided threads at different phases are how serials sustain tension indefinitely.
3. **Interactive drama research** (Façade's beat sequencer, PaSSAGE's player model, Roberts & Isbell's desiderata) plus **RimWorld's shipped storyteller personas** show that dramatic pacing in a systemic game is literally a *scheduler problem*: fire weighted events on a variable-interval cycle with enforced valleys, scale threat size to player progress, and detect/complete arcs opportunistically.

For the balance tester: score runs on **arc completeness, beat-interval statistics, tension-curve shape, thread braiding, resolution rate, and novelty** — all computable over the existing event log. Full formulas in the Scoring section.

## Key Sources

- **Mateas & Stern, "Structuring Content in the Façade Interactive Drama Architecture" (AIIDE 2005)** — **A**. The canonical drama manager: a hidden agent monitors the sim and sequences *beats* annotated with preconditions and effects on story state; beats form a dynamically-linked, forward-flowing network (no cycles). Key insight: local agency lives *inside* beats; global coherence comes from sequencing. Directly applicable: our arc detector is a read-only drama manager.
- **Thue, Bulitko, Spetch & Wasylishen, PaSSAGE (AIIDE 2007)** — **A−**. Player-Specific Stories via Automatically Generated Events: learns a playstyle model at runtime and picks the *content* of the next event to maximize predicted enjoyment ("delayed authoring"). 101-participant study showed improved enjoyment for novices. Grounds the "teaching storyteller" persona.
- **Roberts & Isbell, "A Survey and Qualitative Analysis of Drama Management" (ITSSA 2008)** — **A**. Desiderata for drama managers; taxonomizes declarative-optimization, planning, and search-based architectures. Useful as a checklist our scheduler should satisfy.
- **RimWorld storyteller constants (datamined, corroborated by Ludeon dev blog)** — **A**. The best hard numbers in the literature: **Cassandra Classic** runs a 10.6-day cycle (4.6 days "on" with 1–2 major threats ≥1.9 days apart, then 6.0 days "off"; first on-phase days 11–15.6; ≈8.5 major threats per 60-day year). **Phoebe Chillax** runs a 16-day cycle (8 on / 8 off, exactly one threat). **Randy Random** has no cycle — flat-interval dice. Threat *size* scales with colony wealth (points budget); personas differ only in *when*, not *how big*.
- **Reagan et al., "The emotional arcs of stories are dominated by six basic shapes" (EPJ Data Science 2016)** — **A−**. 1,327 Project Gutenberg novels, three independent methods (SVD, hierarchical clustering, SOM) converge on six arcs: Rags-to-riches (rise), Tragedy (fall), Man-in-a-hole (fall→rise), Icarus (rise→fall), Cinderella (rise→fall→rise), Oedipus (fall→rise→fall). Fall→rise and rise→fall→rise shapes correlate with popularity (downloads). Justifies classifying a run's intensity curve against a small shape library.
- **Kishōtenketsu scholarship (Wikipedia summary; Novielli 2018; Hinds 1982; Oh, TEXT 2025)** — **B+**. Four parts — *ki* introduction, *shō* development, *ten* twist/turn, *ketsu* reconciliation — with **no structural requirement for conflict**; *ten* is deliberately held to the third quarter and often changes the story's genre/frame. *Ketsu* "reconciles" act-1/2 state with the twist and need not be decisive. Maps perfectly onto our Discovery→consequence events.
- **Save the Cat beat sheet (Snyder; teaching materials)** — **B**. 15 beats at fixed page fractions for a 110-page script: Catalyst ~10–12%, Break into 2 ~25%, Midpoint ~55%, All Is Lost ~75%, Break into 3 ~85%, Finale 85–100%. Pop-craft, not science, but the *climax-placement band* is the most operationally useful pacing claim available.
- **Dwarf Fortress legends / Chronicles interface paper (FDG 2022, ACM 3555858.3555909) + "Characterization and Emergent Narrative in DF"** — **B+**. DF generates history, records it, and players *mine* it; likes/dislikes characterization produced disproportionately more player retellings than wounds or family. The Chronicles work adds the missing piece: designed *entry points* into a recorded event mass, plus a subjective mode limited to what the player-character could know.
- **GMTK "Can we Improve Tutorials for Complex Games?" + George Fan's principle** — **B**. Willingness to learn grows with investment → distribute tutorials through play, trigger them at first relevance, and teach without a tutorial at all when steps are small (Portal chambers). The theoretical backbone for tutorializing arcs.
- **CK3 new-player-experience coverage (Polygon/GameWatcher) + Frostpunk minimal-tutorial discourse** — **C+**. CK3's guided-Murchad scenario + reactive advice + tooltip-encyclopedia is the "manual woven into play" model; Frostpunk's two-objective cold open (gather coal / turn on generator) shows teaching-by-situation can misfire when the situation's implicit model is wrong — a cautionary datapoint for our own teaching arcs.

**Limitations:** Western-structural claims (Save the Cat percentages, Freytag) are craft convention, not validated science — treat their numbers as priors to calibrate, not laws. Soap-braiding thread counts (2–4) come from serial-TV practice; no experimental source found. RimWorld constants are datamined/community-documented, hence reliable, but tuned for a colony-survival sim — our village runs longer and softer, so absolute day-values will need rescaling while the *ratios* (on:off ≈ 1:1.3, min-gap ≈ 0.4× on-phase) should transfer.

## (a) Classical Structures

**Aristotle (Poetics):** unity of action — every event causally belongs; beginning/middle/end defined by what must precede and follow, not by position. Also *peripeteia* (reversal) and *anagnorisis* (recognition) as the highest-value events, and "magnitude": an arc must be long enough for change to occur but short enough to hold in memory. For a sim: an arc is a *causal chain*, so arc detection should link events by shared participants/stakes, not just co-occurrence (current `detect_story` does co-occurrence only — two unrelated deaths in one window currently read as one story).

**Freytag's pyramid (1863, *Die Technik des Dramas*):** five acts —
1. **Exposition** — establish world + stakes.
2. **Inciting incident + rising action** — complications raise tension.
3. **Climax** — the apex; fortune reverses.
4. **Falling action** — consequences propagate.
5. **Denouement** — new equilibrium.

The sim *already generates* textbook Freytag material without authoring: writ → defy → double writ → defy again → knights → hanging → vigil is a five-act arc; summons → ransom-deadline → rescue-or-ladder is a three-act arc with a literal ticking clock. Our detector's job is *recognition* of these shapes — far cheaper than generation.

**Three-act / Save the Cat:** the industrial refinement — 15 beats at fixed page fractions (of a 110-page script):

| Beat | Position | Function |
|---|---|---|
| Catalyst | ~10–12% | inciting incident, point of no return |
| Break into 2 | ~25% | protagonist commits to the new world |
| Midpoint | ~55% | false victory/defeat, stakes raise |
| All Is Lost | ~75% | lowest point, "whiff of death" |
| Break into 3 | ~85% | synthesis, final push |
| Finale | 85–100% | climax + resolution |

The computable residue: inciting incident in the first ~10–12%, a midpoint reversal near 50–55%, a lowest point near 75%, climax+resolution in the final ~15%.

**Hero's Journey (Campbell/Vogler):** departure → initiation → return — structurally Freytag with a named protagonist; less useful for an ensemble village sim, except that *The Wanderer herself* is a natural monomyth carrier (outsider → trials → transformed status → "the work goes on" when her hands stop). Implication: score player-centric arcs and village-centric arcs in separate channels — the player's journey can climax while the village braids on.

## (b) Non-Western / Alternative Structures

**Kishōtenketsu:** ki-shō-ten-ketsu — introduce, develop, *twist*, reconcile. The twist need not be a conflict-escalation; it recontextualizes (e.g., *Parasite*). This is a gift to our event vocabulary: `Discovery` events (the clerk's tallies, the pale stone, the hermit's truth) are *ten* events, and the existing "truth sits unsaid until it ripens → power answers" mechanic is already a ki-shō-ten-ketsu machine. Score a twist-arc as: quiet establishment (ki+shō, low-intensity events sharing participants) → a single Discovery (ten) → consequence events referencing the same entities (ketsu).

**Nested/episodic arcs:** serials nest arcs — scene inside episode inside season inside series. For a long-running village this gives a four-level detection stack:

| Level | Span | Example in this sim | Scoring use |
|---|---|---|---|
| Event | 1 tick | a death, a writ | raw intensity signal |
| Thread | ≤1 season | the summons-ransom affair | `arc_completeness` |
| Season | ~1/4 year | the hard winter: stores thin → share-or-blame → thaw | `tension_curve_shape` at seasonal scale |
| Era | multi-season | "the fever year," "the circus winter" (PLAY.md already names these retroactively) | `novelty`, chronicle UI |

**Soap-opera braiding:** soaps sustain indefinite runtime by keeping **2–4 threads open at staggered phases** — as one resolves, another is mid-escalation, another is seeding. The rules: never zero open threads (dead air), never so many the audience can't track (>5–6 is incoherence), and *stagger phases* — synchronized climaxes are the "everything happens at once" failure mode. This gives `concurrent_thread_count` its justification, and tells the scheduler to *seed* new threads during valleys rather than pile them onto peaks.

## (c) Interactive / Emergent Narrative

**Façade's beat sequencer:** beats carry preconditions + story-state effects; the manager picks the highest-value *applicable* beat, keeping the beat network forward-flowing. Adapted to a systemic sim: the scheduler doesn't author beats, it *biases* which latent event-chains get resources — e.g., during a valley, suppress new Conflict/Illness events but allow Bonding/Recovery; during escalation, the reverse.

**PaSSAGE / delayed authoring:** pick event *content* by predicted player preference; a learned model beats a fixed curve for novices. In a balance tester context this matters less for scheduling than for *reporting*: different personas produce different measurable arc mixes, so score each persona against its own intent.

**Roberts & Isbell desiderata:** the survey's checklist translates directly to acceptance criteria for our storyteller —
1. Monitor player/sim state continuously (we have the event log already).
2. Intervene via the *least-authorial* lever available: bias event probabilities, never teleport outcomes — the player should never be able to prove the manager exists.
3. Keep interventions invisible; a detected puppet-master breaks emergence's core promise.
4. Degrade gracefully: when no narrative goal applies, fall back to un-managed sim rather than forcing a shape.
5. Prefer guidance over control (PaSSAGE's term "informed delayed authoring"): choose *which* content fires, let the sim decide *how* it plays out.

**RimWorld personas — the shipped existence proof.** Three schedulers, same event pool, same threat-point math — personas differ only in *when* events fire, not *how big*:

| Persona | Cycle | On-phase | Off-phase (valley) | Major threats | Min inter-threat gap |
|---|---|---|---|---|---|
| Cassandra Classic | 10.6 days | 4.6 days | 6.0 days | 1–2 per on-phase (~8.5/60-day year) | 1.9 days |
| Phoebe Chillax | 16 days | 8 days | 8 days | exactly 1 per on-phase | n/a (1 per phase) |
| Randy Random | none | n/a | n/a | flat-rate dice | none |

Cassandra's first on-phase starts day 11 — i.e., the game *deliberately grants an opening valley* for onboarding. Threat *size* scales with colony wealth (points budget): the difficulty dial is decoupled from the rhythm dial. Randy exists as a control condition — the proof that the shaped curves do the work. **Design lesson: persona = a small parameter tuple (cycle length, on/off split, threats-per-phase, min gap, threat scaling, stochasticity), not a different system.**

**Dwarf Fortress:** two contributions. (1) History is *recorded then mined* — legends are post-hoc, which means our detected stories also serve a chronicle UI. (2) Likes/dislikes-style small characterization drove more player retelling than epic events — scorecard implication: personal-scale arcs (a courtship, a rivalry) count, not only village-scale ones.

## (d) Narrative Pacing Math

- **Tension curve:** Reagan et al. show real stories project onto ~6 basis shapes; a run's intensity series can be correlated against each shape and classified. A *shapeless* (flat or white-noise) curve is the negative result to flag. The intellectual precursor is Vonnegut's rejected master's thesis on "story shapes" — he graphed fortune-vs-time by hand; the 2016 paper confirmed his hand-drawn archetypes fall out of SVD on a real corpus. Notably, the shapes that correlate with popularity are the *non-monotone* ones (Man-in-a-hole, Cinderella, Oedipus) — pure rise or pure fall underperforms a curve with at least one reversal. Scoring implication: `tension_curve_shape` should reward ≥1 sign change in the smoothed derivative, not merely a high peak.
- **Inter-beat interval:** RimWorld is the empirical anchor: min gap between major threats ≈1.9 days; major beats arrive ≈every 7 days on average under Cassandra (8.5 major threats / 60-day year); off-phases run 6–8 days. The *variance requirement* matters: Randy (uniform random) feels chaotic precisely because gaps are unclustered; a metronome (CV≈0) reads as artificial — players learn the schedule and the drama dies. Target an intermediate CV: variable enough to surprise, correlated enough to feel intentional. Statistically, a Gamma/Poisson-cluster process, not uniform — beats should slightly *attract* (set-up → complication pairs close together) while *repelling* at the climax scale (two peaks never adjacent).
- **Valley requirement:** both RimWorld's off-phases and Freytag's denouement encode recovery. Operationally: after any beat above a climax threshold (`I > 0.8 × running_max`), enforce a cooldown window where intensity must stay below a fraction of the peak (~0.5×). Cassandra's opening 11-day grace period is a special case worth copying: the *first* valley is the onboarding valley, and it should be the longest.
- **Climax placement/density:** Save the Cat puts the decisive beat in the last ~15–25%; Freytag puts the pyramid apex past the midpoint; kishōtenketsu holds the twist to the third quarter. Three independent traditions agree: **the turn lives in [0.55T, 0.95T]**, and the final ~10% shows release (mean intensity < ~60% of peak). A run whose max lands at 20% then meanders is the "climaxed too early" failure; a run still climbing at cutoff is the "no denouement" failure.
- **Climax density caveat:** braiding complicates argmax — with 2–4 threads, peaks interleave, so compute `climax_pos` both globally and per-thread. Per-thread, each thread's own peak should land in its last third; globally, we want a *dominant* late peak (the season's big story) with subordinate peaks elsewhere.

## (e) Teaching Through Story

The "manual you don't read" problem has a known solution family:

1. **Distribute lessons; trigger at first relevance** (GMTK/Fan): willingness to learn grows with investment, so each mechanic gets a teaching *situation*, not a page, fired at the moment it first matters. In the sim: the first hunger event teaches Talk (a villager voices the problem); the first parallel-complaint cluster teaches Organize (three voices → issue surfaces); the first writ teaches collective response (bargain down vs. defy — with the doubled-writ consequence visible in the very next season).
2. **Scaffold by gating** (Portal/CK3): each teaching arc only *completes* when the player attempts the mechanic — the arc pauses at its climax until the action is tried, giving immediate diegetic feedback. Portal's trick worth stealing: the "tutorial" is invisible — the player thinks they're just playing. CK3's version: a guided starter scenario (Petty King Murchad unifying Ireland) plus *reactive advice* that fires contextually for the whole campaign, not a front-loaded dump.
3. **Teach by consequence, honestly** (Frostpunk counter-example): Frostpunk's cold open taught a wrong implicit model ("need 200 coal before the generator" — actually it runs on much less; players froze waiting). The lesson for our teaching arcs: the situation's *affordance must match the real mechanic* — if the arc implies "recruit five before the writ comes" but the mechanic actually rewards three trusted members, the arc teaches a bug. This is a scoring concern, hence `teaching_coverage` below and a red flag on gates that never fire.
4. **The story *is* the syllabus**: sequence the teaching arcs as one ki-shō-ten-ketsu — ki: outsider arrives, talks; shō: learns wants/problems, recruits; ten: first writ/crackdown (recontextualizes "village sim" as "power struggle"); ketsu: first organized response resolves. The player finishes the tutorial arc having experienced the entire game-loop once, in miniature, as a story — which is precisely what a manual can never do, because the manual has no stakes.
5. **The DF/CK social-safety-net pattern:** both games lean on community wikis *because* in-game teaching is thin — their players' advice converges on "let yourself fail, treat each loss as a lesson" (the CK3 roguelike-framing advice). Better: make early failure *diegetically cheap* — the first crackdown should burn a small stake (a shunning, a fine), not end the run, so the teaching arc's climax is survivable. Dwarf Fortress's community motto "losing is fun" only works because losses produce the best legends — our chronicle view is what converts a failed teaching arc into a story worth keeping.

## Game Mechanic Proposals

### Mechanic 1: Storyteller personas as a tension scheduler (RimWorld pattern)
A `Storyteller` config struct consumed by the sim's event-bias layer:
`cycle_len`, `on_frac`, `max_major_per_phase`, `min_gap`, `valley_len`, `threat_scaling` (village wealth/org-power → event magnitude), `stochasticity` (Cassandra≈low, Randy≈1.0). Ship three personas, parameterized from the RimWorld anchors (sim-day values; rescale on calibration):

| Persona | `cycle_len` | `on_frac` | `max_major_per_phase` | `min_gap` | `stochasticity` | Intent |
|---|---|---|---|---|---|---|
| The Chronicler | 10 days | ~0.4 | 2 | 2 days | 0.2 | escalating pressure, enforced valleys; default |
| The Quiet Year | 16 days | ~0.5 | 1 | n/a | 0.15 | builder-friendly; long recovery |
| The Mad Piper | — (uniform) | — | unbounded | none | 1.0 | chaos; **A/B control arm** for scoring |

`threat_scaling` applies on all three: writs, crackdowns and winters should size to the org's accumulated power the way RimWorld sizes raids to colony wealth — a powerless Wanderer gets a shunning, a baron-tier org gets the knights.

### Mechanic 2: Arc detection as a phase-state machine over threads
Upgrade `detect_story`: group events into threads by shared participants + issue key; each thread runs a state machine `setup → escalation → climax → resolution` (plus `ki/shō/ten/ketsu` as an alternative phase vocabulary where climax is a `Discovery`/`Rejection` rather than peak conflict). Variables: `min_events_per_arc` (≥3), `max_thread_span` (events beyond span start a new thread), `climax_kinds` per template, `resolution_kinds` (Recovery, Alliance, Achievement, exile/hanging, issue-cleared).

Detection sketch per thread (replaces the current kind-membership classifier):
1. `setup` — first event(s) in thread while running-mean intensity < thread's own median.
2. `escalation` — intensity trending up: ≥2 consecutive events with non-decreasing weight, or a threat-kind event arriving while thread is open.
3. `climax` — thread's argmax-weight event ∈ `climax_kinds` (Conflict/Death/Betrayal for Freytag threads; Discovery/Rejection for kishōtenketsu threads) and positioned in the thread's last two-thirds.
4. `resolution` — any `resolution_kind` event within `cooldown_span` after climax, or thread closes by participant exit (exile, hanging, leaving with the circus — all already generated events).
A thread that reaches climax but never resolves feeds `resolution_rate`'s denominator (a dangle); one that escalates then goes quiet without climax is an aborted arc — count separately, since abortions are normal (soap threads get dropped) but >40% aborted signals the scheduler starves threads.

### Mechanic 3: Braided-thread scheduler
Track open threads; bias event generation so that median concurrent threads stays 2–4, and stagger phases (don't let two threads climax within `min_gap`). Variables: `target_threads` (2–4), `phase_stagger_min`, `thread_seed_rate` (new threads only during valleys or when count < target).

### Mechanic 4: Tutorializing arc templates
Six teaching arcs (Talk, Recruit, Organize, Invest, Build, Contract), each a small scripted-bias template: `taught_mechanic`, `setup_events`, `gate` (arc climax waits for the player action), `resolution_event`, `fallback` (if player ignores for N ticks, degrade gracefully — the villager solves it worse, teaching by omission). Scored separately (see `teaching_coverage`).

### Mechanic 5: Chronicle / legends view
A post-run (and mid-run) legend browser fed by detected stories: entry points = "the fever year," "the widow's shop," sorted by intensity; optional subjective mode (only threads the Wanderer could plausibly know). Variables: `intensity_rank_top_n`, `subjective_knowledge_filter`, `era_naming_threshold` (a season whose arc set exceeds intensity X earns a generated name — feeding `novelty`/`era` scoring).

## Scoring / KPI / OKR System

All metrics compute over the run's event log (`tick, kind, participants, intensity`) — no new instrumentation needed beyond thread tagging. Let `D` = ticks per sim-day (calibrate once); `T` = run length in ticks; a "weighted event" = `intensity ≥ w_min` (suggest `w_min = 0.3`).

Suggested `EventKind` → base intensity weights (calibrate against logs):

| EventKind | Weight | Role in arc vocab |
|---|---|---|
| Betrayal, Conflict, Death | 1.0 | climax-class / threat-introducing |
| Discovery, Rejection | 0.8 | ten-class twist / reversal |
| Illness, summons/writ-class | 0.7 | escalation / threat-introducing |
| Recruitment, Alliance, Achievement | 0.6 | resolution-class (positive) |
| Recovery, Bonding | 0.5 | resolution / valley content |
| Birth, minor social texture | 0.3 | setup / texture |

### KPI 1 — `arc_completeness`
**Formula:** fraction of runs containing ≥1 thread that passes through all four phases in order (`setup→escalation→climax→resolution` or `ki→shō→ten→ketsu`) within `span ≤ 1 season`. Also report per-run complete-arc count.
**Target:** ≥0.90 of seeded runs have ≥1 complete arc; median runs have ≥2.
**Red flag:** runs with ≥5 started threads and 0 complete — the sim escalates but never lands (all rising action, no denouement).

### KPI 2 — `beat_interval_stats`
**Formula:** gaps `g_i = t_{i+1} − t_i` between consecutive weighted events. Report `mean(g)` in sim-days and `CV = σ/μ`.
**Target (anchored on RimWorld):** `mean(g) ∈ [2.5, 8]` sim-days; `min(g) ≥ ~1` day for major-threat-class events; `CV ∈ [0.3, 0.8]` — variable but clustered, not metronomic (CV<0.15 = artificial rhythm) and not white-noise (CV>1.0 under a *shaped* persona = the scheduler isn't shaping).
**Red flag:** `max(g) > 10` days (dead air — fun-rubric's "100+ ticks no events" scaled up); or ≥3 weighted events inside `min_gap` (pile-up).

### KPI 3 — `tension_curve_shape`
**Formula:** bin per-tick summed intensity into 20 buckets → series `I[1..20]`. (a) `climax_pos = argmax(I)/20` ∈ [0.55, 0.95]; (b) `release = mean(I[last 2 buckets]) < 0.6 × max(I)`; (c) best-match correlation `r` against the six Reagan basis shapes — run "has shape" if `max r ≥ 0.5`.
**Target:** `climax_pos` in band ≥70% of Chronicler runs; `release` holds ≥70%; ≥60% of runs match a basis shape.
**Red flag:** flat curve (max/mean < 1.8) = no drama; `climax_pos < 0.35` = peaked early then drifted; no release = ends mid-climax (run stopped before resolution — also check `resolution_rate`).

### KPI 4 — `concurrent_thread_count`
**Formula:** in each rolling `2×mean_gap` window, count distinct open threads with ≥1 event; report median and max.
**Target:** median ∈ [2, 4]; max ≤ 6.
**Red flag:** median < 1.5 (thin, episodic-only sim); sustained >6 (soap braiding fails — player can't track; correlates with low `resolution_rate` since threads starve).

### KPI 5 — `resolution_rate`
**Formula:** `resolved_threats / introduced_threats`, where introduced = thread containing a `Conflict/Illness/Betrayal/Discovery/summons-class` event and resolved = thread reaches a resolution-kind event within `span` (else it dangles).
**Target:** ≥0.70. Some dangling is correct (kishōtenketsu's open *ketsu*, soap cliffhangers) — flag only below 0.5.
**Red flag:** <0.5 = the sim generates Chekhov's guns it never fires; also flag *instant* resolutions (climax→resolution < `min_gap` in >50% of arcs = no falling action).

### KPI 6 — `novelty`
**Formula:** distinct `StoryTheme`s detected per run + count of twist-arcs (ten-type) + distinct era names.
**Target:** ≥3 distinct themes median; ≥1 twist-arc in ≥40% of runs (Discovery events exist — they should *land*).
**Red flag:** ≥80% of runs produce only `Survival`+`Community` = event generators are monotone (likely illness-dominated).

### KPI 7 — `teaching_coverage` (teaching-mode runs only)
**Formula:** `teaching_arcs_completed / 6`, where complete requires the gated player action.
**Target:** 1.0 by tick `T_teach` (calibrate; suggest first ~2 seasons).
**Red flag:** arc gated but `gate` never fires within `fallback` window in >20% of runs — the situation isn't affording the mechanic (the Frostpunk coal bug pattern).

### Composite — `narrative_score`
**Formula:** weighted aggregate per run: `0.25·arc_completeness + 0.20·shape_score + 0.15·beat_band_score + 0.15·thread_band_score + 0.15·resolution_rate + 0.10·novelty_score` (each sub-score normalized to [0,1] against its band; teaching runs add `teaching_coverage` as a gate — score × 0 if coverage < 1.0 at `T_teach`, since a run that fails to teach has failed its primary purpose regardless of drama).
**Use:** single number for balance-tester regression alarms and persona A/B comparisons; **always report alongside the per-KPI breakdown** — a composite hides *which* narrative property regressed.
**Red flag:** composite >0.85 on Mad Piper (random) runs — means the metrics are measuring event *volume*, not *shape*; recalibrate weights.

### Aggregation & calibration notes
- Score **per-persona against persona intent**: The Quiet Year should be judged on `release` quality and `resolution_rate`, not `climax_pos` sharpness; Mad Piper is the null model — it should *fail* most bands, and the size of the gap vs. The Chronicler is the effect size of shaping.
- `D`, `w_min`, `T_teach`, `span`, and all bands above are research-anchored defaults. First scoring pass on real logs should plot distributions *before* enforcing thresholds — expect to widen bands, not narrow them.
- Thread grouping is the fragile step: start with participants-overlap + issue-key; if the log lacks issue keys, fall back to entity co-occurrence within a rolling window (weaker — Aristotle's unity becomes co-presence, which conflates the wedding and the feud).

### OKRs

- **O1 — Every run tells a story.** KR1: `arc_completeness ≥ 0.90` across 100 seeded Chronicler runs. KR2: median ≥2 complete arcs/run. KR3: Mad Piper (random) control scores measurably lower — proves shaping is causal, not incidental.
- **O2 — Tension has rhythm.** KR1: `CV(beat_intervals) ∈ [0.3,0.8]` in ≥80% of runs. KR2: `climax_pos ∈ [0.55,0.95]` in ≥70%. KR3: zero runs with `max(g) > 10` sim-days.
- **O3 — Threads braid without tangling.** KR1: median `concurrent_thread_count ∈ [2,4]` in ≥75% of runs. KR2: `resolution_rate ≥ 0.70`. KR3: phase-stagger holds — two thread climaxes within `min_gap` in <15% of runs.
- **O4 — The village surprises.** KR1: ≥3 distinct themes median. KR2: ≥1 kishōtenketsu twist-arc in ≥40% of runs. KR3: ≥30% of runs earn an era-nameable season.
- **O5 — The story is the tutorial.** KR1: `teaching_coverage = 1.0` by `T_teach` in ≥90% of novice-agent runs. KR2: every gated mechanic attempted within its arc window ≥80% of the time. KR3: post-teaching runs show no drop in `arc_completeness` (teaching doesn't sterilize emergence).

## Next Steps

- Implement thread tagging in `story.rs` (participants + issue key) and the two phase-state machines (Freytag and kishōtenketsu) as detectors — read-only, no scheduler yet.
- Build the scoring harness: event log → the 7 KPIs; run against current sim across seeds to get a *baseline before* any storyteller exists. The random-persona control arm justifies the scheduler empirically.
- Parameterize the three storyteller personas (Mechanic 1) behind a config; wire to event-rate bias only (least-authorial lever, per Roberts & Isbell).
- Prototype one tutorializing arc (Talk — the first voiced problem) to validate the `gate`/`fallback` pattern before templating all six.
- Calibrate `D`, `w_min`, `T_teach`, and span constants against real run logs; all bands above are research-anchored defaults, not final values.
