# Systems-Zoo Audit of the game2-core Loops

**Category:** Design / Internal Modelling Discipline
**Source:** Ticket #953 — audit the sim's loops against Meadows' "system zoo"
(*Thinking in Systems*, ch.5) and leave a checklist for future designs.
**Date:** 2026-11-11

## How to read this

Each entry names the loop's zoo structure: the **stocks** (what
accumulates), the **flows** in and out, the **loops** (B = balancing /
goal-seeking, R = reinforcing / self-feeding), and the **delays** that
decide whether it oscillates. The reference zoo: thermostat (damped B),
bathtub (two flows, no loop), population (renewable stock, births vs
deaths), capital vs nonrenewable (drain-only), capital vs renewable
(drain with regrowth), delayed-loop oscillation. Where a structure is
missing a member the zoo says it needs, the gap is called out and a
fix ticket is filed.

The literature background stays in
`wiki/literature-reviews/systems-thinking-dynamics.md`; this document
maps the structure onto *this* sim, it doesn't re-teach the theory.

## The audit

### Price / worth (`sim/economics.rs`, `game/goods.rs`)
- **Stock:** the good's market price per stall. **Flows:** demand pulls
  it up, supply and the hidden base worth pull it back.
- **Loop:** B — a damped return to each good's base value
  (`test_price_settles_at_goods_own_base_value`). Shortage spikes are
  bounded (`test_prolonged_shortage_price_is_bounded`).
- **Delay:** short — price responds within the market tick; no
  oscillation observed.
- **Verdict:** well-formed thermostat. The one thing it can't express
  is *why* the goal is where it is — base worth is a constant, not a
  stock. Fine as designed; worths are meant to be the quiet anchor.

### The larder (food store → hunger)
- **Stock:** village food store. **Inflow:** harvests, gathering,
  imports. **Outflows:** eating, spoilage, vermin (gated by
  `cat_watches`).
- **Loop:** bathtub, not a loop — nothing in the store's own level
  feeds back into production. The *signal* (hunger) does feed back:
  hungry bellies become `Problem::Hungry`, which feeds the grievance
  chain below.
- **Gap (filed):** births don't feel the larder — see population.

### Grievance → protest → campaign (`game/desires.rs`, `game/campaign.rs`)
- **Stock:** `Organizer.heard` / villager `InnerLife.grievances` —
  grievances accrue from lived problems, expire after a year.
- **Loops:** B on the venting side — grievances surface as issues,
  issues found campaigns, campaign wins relieve the underlying problems
  (grants, law changes), which drains the stock. R on the spiral side —
  failed/repressed campaigns radicalize (`edict` radicalization,
  hanging witnesses), which *adds* grievance.
- **Delay:** explicit — grievances must be heard (3 voices to found an
  issue), meetings take days to call, campaign yields land per phase.
  This is the game's core thermostat-with-lag.
- **Archetype note:** this is Senge's *shifting the burden* — the
  venting loop (protest relieves pressure) can crowd out the
  grievance-removal loop if wins come too easy. Worth watching in
  balance runs, not a missing member.

### Organized power (`sim/organized_power.rs`)
- **Stock:** `OrganizedPower.capacity`. **Inflow:** successful
  organizing actions (transformational > transactional).
  **Outflows:** atrophy on idleness, burnout on overreach.
- **Loops:** B — atrophy decays capacity toward zero when idle
  (`idleness_atrophies_capacity`). R — sustained successful actions
  grow it (`sustained_good_organizing_outgrows_atrophy`).
- **Gap (filed):** the reinforcing loop has **no ceiling**. Capacity is
  floored at 0 but never capped; nothing in the model says a stronger
  cause is *harder* to hold together (coordination cost, lord's
  attention, internal factionalism). The zoo says a reinforcing stock
  needs its balancing constraint or it runs away.

### Spoilage → hunger → sour gut → grievance
- **Stocks:** food store (above), health (`illness`), grievances.
- **Structure:** a *causal chain with delays*, not a loop — spoiled
  stock today → `SourGut` tomorrow → grievance the day after. This is
  how the larder bathtub couples into the organizing thermostat.
- **Verdict:** the delay elements are the point; they keep cause and
  effect readable in the log. Well-formed.

### The cult / tithe growth (`session/subsystems/society.rs`, `clandestine`)
- **Stock:** the fold (followers) + the cult's purse (tithes).
- **Loops:** R — tithes fund appeal, appeal recruits, recruits tithe
  more. B — `cult_collapse_tick` is the brake: exposure, schism, and
  the fold's own overreach drain it.
- **Archetype note:** *success to the successful* — the tithe loop
  rewards whoever already has the flock; the collapse brake is what
  keeps it contestable. Verdict: properly paired loops.

### Bonds (`game/relationships.rs`, `sim/psychology.rs`)
- **Stock:** `Relationship.strength` (-1..1). **Inflow:** warm
  interactions, SharedBits retellings. **Outflow:** ~1%/tick decay.
- **Loops:** B — strength decays toward zero without reinforcement;
  SharedBits adds an R edge (retelling warms the pair, which invites
  more telling). Warmth episodes feed `infer_style`, which is how the
  bond stock shapes later behavior (a state variable, like good
  modelling wants).
- **Verdict:** well-formed; decay-to-zero is the modeled "ties fade".

### Trails (`sim/trails.rs`)
- **Stock:** trail strength per tile. **Inflow:** footfall.
  **Outflow:** decay.
- **Loops:** R — worn paths are cheaper to walk, so they draw more
  feet, so they wear further. Capped: `strength_never_exceeds_saturation`
  — the saturation limit IS the balancing member (a hard ceiling, not a
  soft goal).
- **Verdict:** textbook "reinforcing with a limit". Well-formed.

### Population / lifecycle (`subsystems/people.rs`, `lifecycle`)
- **Stock:** villagers. **Inflows:** `maybe_birth` (needs a Home with
  ≥2 resident adults under the 4-occupant cap), `maybe_migrant`.
  **Outflows:** age/illness/hunger/violence deaths, exile.
- **Loop:** population zoo member — beds gate births, so housing stock
  is the (soft) carrying capacity.
- **Delay:** `KidGrowth` aging is a pure delay — kids become workers
  on a fixed clock regardless of anything.
- **Gap (filed):** births are gated by *beds*, not *bread*. Nothing in
  `maybe_birth` consults the larder — the village reproduces at the
  same clip in a famine. The Limits-to-Growth archetype wants the food
  stock in the birth loop: hungry years should thin births (or raise
  infant mortality) before they thin adults.
- **Gap (filed):** newborns and arriving migrants spawn without
  `Virtues` (#950 seeded the component on setup villagers and country
  folk only) — the soul's stock never fills for anyone born into the
  sim after day zero.

### Reputation (`sim/reputation_qsp.rs`)
- **Stock:** reputation score. **Inflow:** deeds (steps, not rates).
  **Outflow:** drift toward neutral — "eventually forgiven".
- **Loop:** B — a leaky stock returning to baseline; betrayals are
  impulses, not flows. Well-formed thermostat with a long time
  constant.

### The lord's read (`lord_esteem`, `society.rs`)
- **Stock:** `lord_esteem` — the lord's *estimate* of the org's power.
- **Loop:** B — it drifts toward `OrganizedPower.capacity` at ~8%/tick:
  a *state-tracked* goal, so the loop's target moves as the cause
  grows. Elegant: the balancing loop itself is the story (the lord is
  always a season behind).
- **Verdict:** well-formed; the moving target is a feature, not a bug.

## Checklist for future system designs

Answer all of these before a new system goes in. If an answer is
"there isn't one", that absence should be deliberate, documented, and
usually a filed ticket.

1. **What's the stock?** Name the thing that accumulates — if nothing
   does, the system has no memory and probably doesn't need a tick.
2. **What fills and drains it?** Both flows must exist somewhere;
   drain-only stocks are countdowns, and fill-only stocks are bombs.
3. **Which zoo member is this?** Bathtub (no feedback), thermostat
   (B to a goal), population (renewable), reinforcing-with-limit
   (growth + cap), or delayed oscillator. If you can't place it, you
   haven't found the loop yet.
4. **Where's the balancing loop?** Every reinforcing loop needs its
   constraint named — a cap, an atrophy, a rising cost, an opponent.
   "It probably won't run away" is not a constraint.
5. **Where's the delay?** Between action and response — births→workers,
   spoils→sick→grievance. Delays make systems oscillate or make them
   legible; either way they should be deliberate and roughly calibrated
   to the day length.
6. **What sets the goal?** Balancing loops need a target. Is it a
   constant (base worth), another stock (`lord_esteem` tracks
   capacity), or a threshold (`straight_river`)? State-tracked goals
   are the interesting ones.
7. **Who can read the stock's signal?** If no villager, player, or
   system reads the level, it isn't doing work — stocks earn their
   keep through what they inform, not what they hold.
8. **Does the inflow respect other stocks?** Cross-stock coupling
   (births ignoring the larder) is where incoherence breeds — check
   the new flow against the stocks that *should* constrain it.

## Gaps filed from this audit

- **Births don't feel the larder** — population's inflow is
  bed-gated, never bread-gated; the Limits-to-Growth balancing loop
  is missing (#1119).
- **The cause's muscle grows without a cap** — `OrganizedPower.capacity`
  is floored but never bounded; the reinforcing loop needs its
  constraint (#1120).
- **Newborns and migrants carry no soul** — `spawn_kid`/`maybe_migrant`
  never seed `Virtues`, so the #950 accretion layer can't reach anyone
  born into the sim (#1121).
