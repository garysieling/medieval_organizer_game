# Societal Evolution Model — the Generative History Engine

**Category:** Design / Core Systems
**Source:** Targeted research request (anthropology of societal development + religion)
**Date:** 2026-10-25
**Research base:** `wiki/literature-reviews/sprint-5-anthropology-social-evolution.md` + `sprint-5-anthropology-religion.md`

## The thesis

Societies don't get designed, they *develop*. The game's history should run the same causal chain anthropology describes: **ecology → subsistence → violence pressure → protection market → stratification → legitimation → revolt risk**. A medieval-looking world isn't medieval because we drew castles — it's medieval because pastoral raiders made farmers pay for protection, and the protectors calcified into lords.

## The causal chain as a state machine

```
ECOLOGY template (climate, water, arability, borders)
        │
        ▼
SUBSISTENCE mode: grain / mixed / pastoral / horticultural
        │                                    │
        │                    pastoral + horse-tech → MOBILE RAIDERS
        │                                    │
        ▼                                    ▼
   agrarian village  ←── raid pressure ──  steppe confederation
        │                               (trade-or-raid economy)
        ▼
PROTECTION MARKET: villagers can't fight horsemen; someone
who can (fortified + mounted retinue) sells security for dues
        │
        ▼
STRATIFICATION: protector → warlord → hereditary lord
   (military tech cost × threat level → how steep the pyramid)
        │
        ▼
LEGITIMATION: the arrangement becomes cosmology
   (three orders: pray/fight/work — religion as doctrine of rank)
        │
        ▼
EQUILIBRIUM: feudalism — stable while the moral-economy floor holds
        │
   breach: subsistence failure / escape route open /
   succession crisis / new military tech / commutation market
        │
        ▼
RESISTANCE GRADIENT: everyday resistance → organizing → REVOLT
   (pilfering/foot-dragging/gossip → commune mobilization → uprising)
```

## Per-link mechanics

### 1. Ecology → subsistence (Steward's culture core)

Region template (from `world-generation-exploration.md` + `climate-change-simulation.md`):

| Ecology | Feasible subsistence | Generated organization |
|---|---|---|
| Hot/dry steppe, thin soils | **pastoralism** | mobile camps, kin-herds, raiding norms, horse wealth |
| River valley, fertile | **grain agriculture** | settled villages, surplus, storable wealth (taxable!) |
| Hills/marginal | **mixed/swidden** | dispersed homesteads, hard to govern (Scott's Zomia) |
| Forest frontier | **horticulture + forage** | mobile, low-density, escape terrain |
| Coastal/harbor | **maritime + trade** | towns, merchants, alternative to land-power |

Design rule: subsistence is **derived, never chosen** — a villager doesn't pick "pastoralist" from a menu; hot dry land + herd animals *makes* him one. `game/terrain.rs` + climate zones produce the mode.

### 2. Horse + spear → the raider asymmetry (Anthony, Khazanov)

- **Military tech nodes** in the tech constellation: riding → mounted herding → mounted warfare → composite bow / lance → heavy cavalry. Pastoral regions get these *free with their subsistence* (they live on horses); agrarian regions must *buy* them expensively.
- Raid pressure = f(steppe proximity, steppe organization, border state's strength). When the steppe unifies (war-leader), raids become extortion; when the border state is strong, the steppe fragments into traders.
- Agrarian wealth is *storable and taxable* (grain) — pastoral wealth is *mobile and raidable* (herds). The asymmetry is structural: farmers need walls; herders need horses.

### 3. The protection market → stratification (Tilly, Bloch)

- `contracts.rs` gets a **protection contract** type: lord offers security + justice; village owes labor dues + taxes + military service. It starts negotiated (village *asks* for protection) and heredities into custom.
- Stratification depth = f(threat level, military-tech cost, duration). Border villages near active raiders → steep hierarchy fast; safe interior villages → shallow, guild-y, communal.
- The protector's own incentives: a lord wants **legible, stationary, taxable subjects** (Scott's "seeing like a state") — pressure to convert grazing to grain, commons to fields, custom to census.

### 4. Legitimation layer (religion as ideological glue — sprint-5-religion)

- Religion stage tracks polity scale (Bellah): village folk-religion (saints, spirits, ritual cycle) → regional sanctioned cult (priestly class, cathedral parish) → doctrinal orthodoxy for kingdoms (Big God, universal moral code).
- **Imagistic vs doctrinal** (Whitehouse): routine ritual maintains cohesion (mass, feast days = slow regen of community bonds); imagistic events (prophecy, martyrdom, pilgrimage fervor) create intense small-group bonds that can go revolutionary — `fundamentalism.rs` is the imagistic flashpoint mechanic.
- The three-orders ideology (pray/fight/work) = the legibility doctrine for feudalism: the church tells you your rank is cosmic order. When the church's legitimacy breaks (corruption, alternative preachers), the political order loses its glue.

### 5. The feudal equilibrium and its breach conditions

Feudalism persists while: extraction ≤ subsistence floor (Scott's moral economy) AND no viable escape AND no challenger elite AND military tech keeps protection expensive.

Breach triggers (each a real mechanic):
- **Subsistence breach** — famine/tax past the floor → everyday resistance intensifies → organizing (commune exists in `community.rs`) → open revolt when an opportunity lands
- **Escape option** — free land on the frontier; lords respond with serfdom (binding peasants) or incentives
- **Elite crisis** — succession war, elite split → opportunity window (Tilly's mobilization condition)
- **Military-tech shift** — cheap weapons (bows, later gunpowder-equivalent) devalue the mounted knight → protection monopoly breaks → the whole bargain renegotiates
- **Commutation market** — peasants with surplus buying out labor dues → free peasantry → the Brenner path to something post-feudal

### 6. Revolt outcome space

Never scripted — the result of actual balance: grievance × organization × opportunity vs. lord's force + legitimacy + elite cohesion. Outcomes: crushed (martyr → imagistic scar), bargained (charter — new contract template), escape (frontier colony founded by refugees), capture (role reversal — the steward becomes the lord, structure unchanged — the dark historical punchline).

## Design rules

- **No hardcoded ladder** — Graeber & Wengrow's warning: the chain is a *probability surface*, not a tech tree. A village that keeps escape open or wins early doesn't feudalize; a commune can persist. Determinism is in the *mechanics*, not the *outcome*.
- **Regions diverge structurally** — one map can hold a feudal valley, a free-peasant hill zone, a raider steppe, and a trade coast — the *world* is the museum of the whole process at once (synchronic diversity, not diachronic levels).
- **Religion is causal** — not cosmetic: it produces cohesion, legitimates hierarchy, and supplies the revolt's theology. `religion.rs` is load-bearing.

## System couplings

- `world-generation-exploration.md` (ecology templates), `climate-change-simulation.md` (aridification → pastoralist pressure)
- `simple-military-battle-system.md` + `non-rps-combat.md` (raiding, protection as combat asymmetry)
- `legal-framework-contracts.md` (protection/tenure contracts → hereditary custom)
- `sim/religion.rs`, `sim/fundamentalism.rs`, `sim/magic.rs` (legitimation, imagistic events, heterodoxy)
- `sim/community.rs` (organizing infrastructure), `medieval-allegory-power-dynamics.md` (the politics this generates)
- `lifecycle-simulation.md`, `age-based-self-interest-economy.md` (household-level pressure response)
- `dynamic-map-evolution.md` (frontier shrink/growth), `fog-of-war-memory-fade.md` (the ungoverned edges)

## Rating

The deepest system in the project — this is what turns "a medieval game" into "a game that generates medieval societies." Nearly every spec feeds it.
