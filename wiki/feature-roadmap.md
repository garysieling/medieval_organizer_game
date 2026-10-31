---
created: 2026-10-30
status: ordered backlog — work top-down, one feature per PR
---
# Feature Roadmap — Missing Features, Spec'd for Incremental Work

A living list of features the game doesn't have yet, specced so each one
can be implemented as a single incremental PR that leaves the game a bit
better and all tests green. Ordered: earlier items are higher-value,
lower-risk, and lay groundwork later items reuse.

## How to work the list

- **One feature = one PR.** Don't batch; each merge must stand alone.
- **Additive modules.** New behavior goes in a `game/` (or `sim/`) module;
  wire into `game_ui.rs` without disturbing the text game or `jobs.rs`
  users — same rule as `skills-vocations-plan.md`.
- **Check overlap first.** Many `feat/*` remote branches are pending or
  squash-merged under different names (e.g. `feat/apothecary-prepares-remedies`,
  `feature/trail-strength-feedback-loop`, `feature/contagion-spreads-by-proximity`).
  Before starting an item, `git fetch` and grep the pending branches for
  its topic; skip or re-scope what's already landed.
- **Verify in repo style:** unit tests per module; `cargo test --lib`
  green; `cargo run --bin game_ui -- --dump-frame N` evidence BMPs where
  the feature is visible; update `PLAY.md` when controls/cards change.
- **Traceability.** Each item notes the lit review or brainstorm line it
  draws on, so the spec can be deepened on demand.

---

## Wave 1 — The cottage economy (animals, plants, and the people who tend them)



### F14. Perceived-power matrix — power recognizes power

*`sprint-7-organizing-power.md`; `sim/organized_power.rs` exists.*

**What.** Every org holds an estimate of every other's power, updated by
*visible* actions; demands are ignored, negotiated, or repressed based
on the estimate — not raw truth.

**Spec.** `power_estimate: HashMap<(Entity,Entity), f32>`; +on visible
actions (meetings held, sanctions landed, bread distributed), decay when
idle (power atrophies — unlike gold). Knight/lord replies to the player's
org switch on the estimate: ignore → negotiate → concede/repress. The
classic organizer arc becomes playable: you're nobody until you've shown
power in the street. **Size: M.**

---


## Wave 3 — The world outside (roads, travelers, and the far powers)


### F19. The roving pedlar

*Brainstorm "roving pots-and-pans seller"; reuses `traveller_path`
+ `TownRole::Trader` machinery + `sim/gossip.rs`.*

**What.** Every ~60–120 days a pedlar walks the road to the market,
sells imported oddities (ribbons, needles, a lute string, a rumor about
the next village over), trades gossip both ways, and leaves.

**Spec.** `PedlarEvent` like the minstrel: stock of 4–8 goods incl. 1–2
items nothing local supplies; carries `gossip` entries seeded from rival
village state (their famine becomes our rumor). Villagers `Buying`-walk
to his stall. He's the game's newspaper. **Size: M.**

---

### F20. Perfumer — flowers, aqua vitae, and glass

*`sprint-7-perfume-and-smell.md`; consumes F4 flowers, F9 distillery
alcohol, glazier glass; counterpoint to F10 smell.*

**What.** A rare petit-bourgeois trade: the perfumer distills garden
flowers into scent — a luxury good that also mechanically counters
`Smelly` problems and serves status/clout.

**Spec.** `Vocation::Perfumer` (inputs `["flowers","aqua_vitae","glass"]`,
output `"perfume"`, high price ~20g). Wearing perfume sets a positive
`Smell` for days (visible "smells of roses" on the card). Noble/PMC
buyers; a gift of perfume is a trust lever the player can invest in.
Glassblower secret-trade variant (below) pairs naturally. **Size: M.**

---

### F21. Number systems & translators — the dragoman problem

*`sprint-7-number-systems-and-translators.md`.*

**What.** Rival villages keep different accounts: tallies, Roman+board,
positional numerals. Cross-village trade and contracts lose value to
"conversion friction" and open fraud unless a translator/clerk is
involved.

**Spec.** `Numeracy` trait per village/faction; trade events between
mismatched systems apply a friction tax (1–15%, error-seeded) unless a
`Translator` (new TownRole, spawns when two villages trade regularly)
mediates. `AbacusSchool` building spreads positional numeracy. Fraud
cases become `Problem`-worthy grievances — an organizer issue: "the
reckoning that didn't add up." **Size: M-L.**

---

### F22. Sky events — comets, eclipses, solstices

*Brainstorm "astrology/sky events"; pre-loads F23.*

**What.** A celestial calendar: solstices anchor festivals (F15); rare
comets/eclipses are *interpreted* — villagers draw omens matching their
problems (the fearful see doom, the priest sees warning, the herb-wife
sees a gathering-night).

**Spec.** `SkyEvent { kind, tick }` seeded sparse; on trigger, each
villager samples an `Omen` from their wants/problems → one-line bubbles
across the map (a crowd of small opinions, not a modal). Priest can
`counsel` on omens (calm vs inflame — feeds `sim/fundamentalism.rs`).
The night the comet rides over the walls should be a screenshot people
share. **Size: S-M.**

---

### F23. The disclosure — a Galileo-style showdown

*`sprint-7-galileo-showdown.md`; climax of F21/F22/printmaker threads.*

**What.** Rarely, a character makes a discovery (an omen pattern true, a
remedy that works against doctrine, a number that exposes the tax
reckoning). They may disclose it; threatened orgs react.

**Spec.** `Discovery { kind, threatens: Vec<OrgKind> }`; disclosure
choice (whisper, pamphlet, pulpit) → each threatened org responds from
F14 perceived-power: ignore / denounce / sanction (F13) / adopt. The
player can be the discoverer or the protector. The review's ally-buffer
model: patrons and guilds absorb backlash. This is a capstone — it pays
off a dozen earlier systems at once. **Size: L.**

---

### F24. The travelling circus

*Brainstorm "travelling circus — joining means you leave with them."*

**What.** A rare multi-week event: a troupe camps outside the walls —
music daily, wonder, petty theft, and a real temptation: a villager
(maybe a bored kid-grown-junior, maybe the player's recruit) can run off
with the circus.

**Spec.** `CircusEvent`: troupe entities camp on meadow; `Music`/`Joy`
mass-satisfaction + `Problem::Tempted` on those with `DifferentTrade`/
unmet `Joy`; nightly small `Purse` drains on attendees (roguery).
Departure roll: tempted + low bonds → they leave with it (visible walk
off the road). Org membership boosts resist — belonging is the
counterweight. **Size: M.**

---

### F25. The lord's arm — castle writ, dungeon, hostage rent

*Brainstorm "castles & dungeons"; `game/military.rs`, `factions.rs` exist.*

**What.** The off-map lord stops being pure text: a bailiff rides in with
writs — a levy (X grain), a fee (X gold), or a summons (one person for
"questioning"). Orgs can comply, bargain (F14 power), or defy (F13
counter-sanction: the levy wagon ambushed, tithe-barn picketed).

**Spec.** `LordEdict` events on a seasonal cadence + crisis triggers;
compliance costs goods/gold; defiance seeds `Repression` (bailiff
returns with knights — stocks/dungeon for the ringleader, who must be
ransomed or rescued by org action). The village's relationship with the
castle becomes a legible meter made of incidents, not a stat. **Size: L.**

---

### F26. Hard winter as a seasonal scenario inside game_ui

*`bin/hard_winter.rs` exists standalone; `feat/hard-winter-scenario`
pending — check overlap.*

**What.** Roughly every few years (seeded), winter runs long: crops
stunted longer, stock decays bite, wolves press nearer, snow visual
persists, and the village either pulls together (festival-out-of-season,
org food distribution = perceived power) or pulls apart (theft,
scapegoating → F13 targets: the herb-wife, the pedlar's people).

**Spec.** Extend `sim/weather.rs`/`climate.rs` with a `HarshWinter`
multi-year flag; scale existing stunt/starve mechanics rather than new
ones. It's a tuning event that lets existing systems *sing* under
stress. **Size: S-M.**

---

---

## Wave 4 — Craft, art & the strange

### F27. Pigments & painters — color as clout

*Brainstorm "pigment collector / paint maker"; `sprint-4-printmaking`,
art-history review.*

**What.** A pigment-gatherer (shades of ore, madder, woad from the
garden) supplies the glazier and a painter trade; orgs and the wealthy
commission painted signs, murals, and heraldry — visible clout.

**Spec.** `Vocation::Painter` (inputs `["pigment","wood"]`, output =
applied `Painted` tag on buildings — a persistent bright tint + name).
Buildings wear off over seasons (re-commission = repeat business). An
org's painted hall is a visible power claim (F14 sees it). **Size: M.**

---

### F28. Heirloom objects — things with history

*`sprint-7-luthiery-and-minstrels.md` (instruments as items w/ maker,
wood, age); glazier/luthier "masterwork" actions already exist.*

**What.** Masterwork outputs (a lute, a window, a fine tool set) become
named `Heirloom { maker, owner-lineage, quality, story-tags }` items that
persist through deaths (F12 inheritance), accrue stories (played at the
feast of X), and can be lost/sold/bequeathed — grief over an object.

**Spec.** Item entity + `Heirloom` component; inspector shows lineage
("made by old Berta, played at the winter feast"); bequest picks kin.
Makes "a window to be remembered by" literally true. **Size: M.**

---

### F29. Letters & the written word

*`writing-materials-bookmaking.md` review; printmaker synergy.*

**What.** Literate characters (priest, printmaker, clerk, some PB) can
send letters — durable, non-proximity communication that seeds bonds,
warnings (a comet seen elsewhere), and evidence (a contract breach in
writing is actionable).

**Spec.** `Letter { from, to, kind, payload }` travels by errand/trade
routes (F19 pedlar carries post). Literacy is a trait; the illiterate
pay the priest a penny to read — a service economy line and a cozy card
detail. **Size: M.**

---

### F30. Disabilities & care

*Brainstorm "disabilities — consider modelling some".*

**What.** A few villagers seeded with a difference (a lame leg —
slower walk; failing sight; a mind that needs quiet). Buildings can be
"good for them" (near the market vs far); kin/neighbors checking on them
is a modeled care relationship that builds attachment — community made
visible.

**Spec.** `Condition { kind }` on ~8% of villagers; walk-speed/path-cost
modifier, a matching `Want`/caregiver `RelKind`. Kept humane and
non-mockable — texture, not punchline. **Size: S-M.**

---

### F31. The fisher and the water's edge

*Brainstorm "sea creatures/fishable things" — map already has a river.*

**What.** A fisher works the river: fish as a GoodFood source, eels,
occasional river-strange finds (a pale stone, a thing that glows —
vision/magic hooks). Winter ice changes it; the river floods rare years.

**Spec.** `Vocation::Fisher` (no inputs, output `"fish"`, workplace a
hut by the water); catch rate scales with season/flood state; flood
event damages riverside stock (storm-damage machinery pending). Optional
`SeaMonsterRumor` gossip line the pedlar (F19) sells — true or not is a
world parameter. **Size: M.**

---

### F32. Caves — the monochrome under

*Brainstorm "caves — underground is monochromatic."*

**What.** One cave mouth on the map edge: a desaturated mini-layer for
mushrooms (F5's best patch), a hermit (a `TownRole` recluse with rare
lore), ore the toolmaker's inputs could one day come from. Deliberately
small: a place, not a system.

**Spec.** `Cave` building/zone; entities inside drawn desaturated;
hermit gives one big want/problem ("the dark keeps me") and one piece
of true herb lore. Cheap mystery. **Size: S-M.**

---

### F33. Death & continuation — the player is the org

*Brainstorm "death & continuation — on death, a cutscene, then continue
as another character."*

**What.** When the player's character dies, the game doesn't end: a
short grave-scene, then control passes to the org member with highest
trust (or a chosen heir) — "the work goes on."

**Spec.** On player `Dead`: pick successor (trust×bond ranking, org-first
then kin); transfer `PlayerCharacter` + `Heard` ledger (the organizer's
knowledge is institutional, not personal — the organizing thesis made
mechanical). If no heir/org: true end screen as today. Changes the whole
game's emotional contract. **Size: M.**

---

### F34. A voice with Shakespeare in it

*Brainstorm "Shakespearean language — insults, jokes, compliments";
creativity corpus has the texts.*

**What.** Talk/meeting/bubble lines get a register pass: flavored
insults (sanction gossip), compliments (courtship), oaths — drawn from a
line-bank built off the corpus, keeping per-vocation voice tables.

**Spec.** `linebank.rs` data module (seeded tables per register +
vocation); Talk lines sample register by relationship. Zero new
mechanics — pure voice. Do it after F17 so jokes can *reference* things.
**Size: S.**

---

### F35. The dark cap: the gibbet (optional)

*Brainstorm "Guillotine — maybe; the violent end of the spectrum."*

**What.** The top of F13's ladder, gated and rare: execution exists only
for the lord/occupying power (the village never votes it) — a permanent
death, mass grief, and a martyr whose grave becomes a rallying site
(graves already render). A defeat-condition texture, not a player tool.

**Spec.** Only via `LordEdict` repression at max escalation (F25).
Deliberately grim and rare; flag to disable. **Size: S.**

---

## Wave 5 — Meta & quality (work in when convenient)

- **F36. Perf/batch pass** — flock + villagers > 40 entities: batch
  separation pass, sprite cache, path memo (planned in
  skills-vocations-plan's phase 5).
- **F37. Debug/telemetry overlay** — trails heatmap, power matrix,
  economy prices over time; the tuning panel lit reviews ask for.
- **F38. Save-scenario pack** — "the fever year", "the circus winter",
  "the widow's shop" as seeded scenario presets (reuses `infra/save_load.rs`
  checkpoints).
- **F39. WASM build pass** — per `wasm-compilation.md`, for the sister
  build.
- **F40. Sound** — ambient loop + event chimes; last per backlog priority.

---

## Suggested opening run — in order

1. ~~F1 Herbologist~~ — **done, PR #141.**
2. ~~F2 Shepherds/flocks~~ — **done, PR #142.**
3. ~~F3 Chickens~~ — **done, PR #143.**
4. ~~F41 Heart Helper~~ — **done, PR #144.**
5. ~~F5 Foraging/mushrooms~~ — **done, PR #145.**
6. ~~F43 Issues belong to trades~~ — **done, PR #148.**
   grievance richer; do it before the roster of scoped problems grows.
7. ~~F7 Vermin/cats~~ — **done, PR #147.**
   now cats do too.
8. ~~F12 Shop succession~~ — **done, PR #149.**
9. ~~F11 Children grow up~~ — **done, PR #150.**
10. ~~F18 Footfall trails~~ — **done, PR #151.**
    already written.
11. ~~F42 The land keeps score~~ — **done, PR #152.**
    issues F43 makes matter; needs a few features' worth of economy to
    bite.
12. ~~F13 Sanction ladder~~ — **done, PR #153.**
    ergot paths converge here.
13. ~~F15 Festivals~~ — **done, PR #154.**
    the first thing that feels like a *village*, not a sim.
