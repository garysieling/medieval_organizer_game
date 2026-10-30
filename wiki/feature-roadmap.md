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

### F1. The Herbologist — one wise woman at the edge of town

*Priority pick. Draws on `game/herbology.rs` (fully built, unwired) +
`sprint-7-medicinal-plants.md` + `sprint-4-tea-herbology.md`.*

**What.** Exactly one herb-wife lives in a hut outside the wall. She
forages herbs that grow as visible patches on meadow/forest tiles,
prepares remedies (`prepare_remedy` already returns a `Treatment`), and
sick villagers walk to her instead of waiting out the `IllnessSystem`.

**Spec.**

- `TownRole::Herbologist` (not a `Vocation` — she is seeded, never rolled:
  exactly one per village, `SocialClass::Working`, no `Craft`; she carries
  `Apothecary { known_herbs, remedies, skill }` from `herbology.rs`).
- `HerbPatch` component on map tiles: a flower-tinted tuft marker;
  regrows ~90 days after being picked. Seed 8–15 patches of the 7 herbs
  in `herb_tree()`, meadow- and forest-biased.
- `BuildingKind::HerbHut` — a dimmed-cottage tint variant placed just
  outside the gate near meadow. Her stock is her hut's `stock["remedies"]`.
- Activity: `Foraging(patch)` — she walks out (walk_path), picks a herb
  she knows into stock; `Preparing` at the hut turns 2 herb units into 1
  remedy good via `prepare_remedy`.
- Sickness loop: when `is_sick(v)` fires, the villager gets `Activity::
  SeekingRemedy`, walks to her hut, pays 2–5g, receives the remedy as a
  `Treatment` applied to their `Disease` (efficacy scales her skill ×
  herb potency). Out of stock or wrong herb → they go home and endure.
- Discovery game (from the lit review): each herb has `true_effect` and
  `believed_effect`; she only *believes* some. Remedies from wrongly-
  believed herbs have hidden near-zero efficacy — sometimes "the cure
  didn't take," which is a story, not a bug.
- Reveal: Talk with her can teach the *player* a herb's believed identity
  (journaled in `Heard`); her card shows a "herb lore" line. She gets
  her own wants/problems — seed ~30% of villagers with latent
  `SuspectHer` (fires when an illness kills anyway: "her cures did
  nothing") — the seed of witch-trial emergence.
- UI: hut clickable like other buildings ("the herb hut", stock of
  remedies, for sale list); her card reads "Herb-wife · Working";
  `Foraging`/`Preparing` get a `✿`/mortar marker; sick walkers show the
  existing `+` sick mark.

**Verify.** Exactly one herbologist across seeds; sick walker reaches hut
and recovers faster with remedy vs without; unknown herb ⇒ no remedy;
believed-vs-true mismatch produces low efficacy; patches regrow; hut
inspector draws. **Size: M.**

---

### F2. Shepherds, sheep & goats

*Draws on brainstorm "Shepherds/goatherds — implies farming; gives dogs
more to do" + `sprint-7-perfume-and-smell.md` (smell).*

**What.** A shepherd vocation keeps a flock of sheep (and a few goats)
that graze the meadow outside the walls. Wool and milk enter the economy;
the dog finally earns its keep.

**Spec.**

- `struct Livestock { kind: LivestockKind (Sheep|Goat), flock: Entity,
  fed: f32 }` — individual animals as entities (8–14 per flock) so they
  can scatter, sicken, and die. Simple `Graze` wander biased to flock
  anchor; sprites = scaled-down quadruped blob (four-pixel legs, wool
  tint) — no pack art needed at first.
- `Vocation::Shepherd` (Working): inputs none; outputs `"wool"`
  (seasonal, ~2 units/sheep on shearing day) + `"milk"` (daily from
  goats, food-priced ~2g). Trade name "the fold"; workplace = a fenced
  pasture `BuildingKind::Pasture` outside the wall.
- Activity `Herding`: the shepherd walks to the flock most days, drifts
  it toward fresh meadow (reuses walk_path). A `PetRole::Herding` dog
  bonded to the shepherd halves drift scatter radius — dogs visibly
  orbit the flock.
- `Problem::Smelly` on shepherds (low social cost on Talk: −10% reveal
  roll) — hooks the smell system from the perfume review, pays off at F20.
- Small crisis: rare `WolfNight` event — a dark shape crosses the map at
  night, flock scatters, 0–2 sheep lost, shepherd gains `Problem::Loss`
  for the season. Stories, not stats.
- Wool feeds the tailor/weaver chain (F8); milk feeds F9 cheese.

**Verify.** Flock stays clustered near anchor; shepherd present → milk
produced daily; shearing yields wool on the season tick; herding dog
reduces scatter; wolf event scatters and can kill within bounds; smelly
flag dampens talk reveals. **Size: M.**

---

### F3. Chickens & eggs

*Quick win after F2 reuses Livestock; brainstorm-adjacent.*

**What.** Hens scratch around 3–5 of the homes; children can be sent to
gather eggs as an odd job.

**Spec.** `LivestockKind::Hen` attached to a `Home` building instead of a
flock — wanders a few tiles of the yard. Produces `"eggs"` into the
house stock daily (cap 6). A `Kid` can be hired (existing `Send errand`
machinery) to collect → house gains eggs, kid gains 1–2g. Rare fox event
carries off a hen — the household mourns it in gossip. **Size: S.**

---

### F4. Physic garden & cottage gardens

*Brainstorm "house gardens and community gardens"; feeds F1 herbs and
F20 perfume flowers.*

**What.** Some homes get a garden bed (soil-tinted tile strip beside the
house). Occupants tend it; it yields kitchen herbs (food want variety),
feverfew/marigold patches the herb-wife may harvest with permission, and
flowers.

**Spec.** `struct Garden { beds: Vec<GardenBed { plant, growth } }` on
`BuildingKind::Home`. Tending is a short `Activity::Tending` idle-break;
growth fills like crops. Harvest adds `food` or named `flower`/`herb`
goods to house stock. The herb-wife's forage search includes gardens
only if an occupant's trust > threshold (otherwise they're "her flowers,
not the common's"). `Want::Garden`-flavored `Joy` satisfaction when
tending. **Size: S-M.**

---

### F5. Foraging & mushrooms — lookalikes, hidden poison, ergot

*`sprint-7-mushrooms.md` — the review calls this CRITICAL for mystery.*

**What.** Mushroom patches spawn in forest/shade after rain. Anyone
forages them; ID skill decides what you actually picked. Some lookalikes
are poisonous with delayed onset.

**Spec.**

- `MushroomPatch { species, visible_traits }`; `species` hidden, traits
  shown ("pale cap, ringed stem"). Forager with skill compares traits →
  believed identity (wrong on low skill).
- `HiddenPoison` illness seeded on eating a bad mushroom: no symptoms
  for ~2–4 days, then severe; the herb-wife (F1) can purge it if reached
  in time — first real "walk to the healer or die" story.
- `ErgotEvent`: wet year + rye-heavy bread stock → village-wide
  St.-Anthony's-fire outbreak; who gets blamed (the baker? the
  herb-wife? a rival village's grain?) is emergent — feeds F13 sanctions.
- Foraging as minor vocation `TownRole::Forager` or a fallback activity
  for the `Broke`: "a poor soul gathering in the wood."

**Verify.** Poison delay holds; skill→ID accuracy monotone; ergot only
fires on wet+rye; remedy path saves within window. **Size: M.**

---

### F6. Beekeeping — honey, wax, and why the church is dark

*`sprint-7-beekeeping.md`.*

**What.** A beekeeper tends skeps at the meadow edge. Honey is a good;
wax supplies church candles and contract seals.

**Spec.** `Vocation::Beekeeper` or a second `TownRole`; 3–6 `Hive`
entities (`Skep|Log|Box` kinds as regional variants). Honey seasonally;
wax per harvest. Two consumptions make it matter: (a) the church burns
`"candle"` stock — at zero, services draw fewer attendees (piety dip);
(b) `contracts.rs` signings consume `"wax"` — no wax, no seals, contracts
stall. Skep harvest = kill-the-bees yield ×2 / sustainable = ×1 (the
short-term/long-term tension from the review). Bees swarm event: a
swarm follows a villager for a day (harmless, funny, memorable). **Size: M.**

---

### F7. Vermin, spoilage & the granary cat

*Small systemic pressure; pets get a second job.*

**What.** Stockpiles quietly spoil; grain draws mice; cats earn their
keep.

**Spec.** Per-good `half_life` (bread days, wheat weeks, wool months);
house/shop stock decays by it. `Granary` buildings gain a `vermin` rate
∝ nearby grain; a bonded `PetRole::Hunting` cat on the owner's home
suppresses it. Broke villagers eat spoiled stock → `Problem::SourGut`
(minor illness). Cheap, makes the store feel alive. **Size: S.**

---

### F8. Wool → cloth: the weaver and the tailor's local supply

*`sprint-4-textiles.md`; completes F2's chain.*

**What.** A weaver vocation turns the shepherd's wool into `"cloth"` at a
loom house; the tailor buys local wool-cloth instead of only imported
cloth.

**Spec.** `Vocation::Weaver` (inputs `["wool"]`, output `"cloth"`,
workplace "the loom house"). Tailor inputs become `wool→cloth` preferred,
`imported cloth` fallback (flexible-recipe style from
`flexible-recipe-system.md`: same output, substitutable inputs at
different prices). Trader's imported cloth price rises → the village
making its own cloth is a visible economic event worth an issues-list
line. **Size: S-M.**

---

### F9. Milk → cheese: cellars and the maturity clock

*`sprint-7-fermentation-trades.md`; completes F2's second chain.*

**What.** A cheesemaker (or the shepherd's household) turns goat/cow milk
into cheese that ages in a cellar — the same recipe in different cellars
yields different named cheeses.

**Spec.** `struct AgingGood { base, cellar_traits (temp, humidity, mold),
  start_tick }` — item-level, on building stock. `"cheese"` price grows
with age; `Cellar` is a `BuildingKind` with seeded traits so each
village's cheese is "the stone-cellar cheese" vs "the damp cheese."
Rare noble-buyer event pays triple for a well-aged wheel. F3-style
children can be sent to "turn the cheeses." **Size: M.**

---

### F10. Smell as a system

*`sprint-7-perfume-and-smell.md`; unifies F2 `Smelly`, F7 rot, F20 perfume.*

**What.** A `smell` field per entity/zone (stables, tanner, herb-hut,
person after work); it shows in talk ("she smells of the fold") and
damps/boosts social rolls; washed off at rain/well; perfume counters it.

**Spec.** `Smell { intensity, kind }` recomputed daily from vocation +
conditions. Talk/meeting lines render it; reveal-roll modifiers. Kept
deliberately thin — it's texture that other features hook, not a sim.
**Size: S.**

---

## Wave 2 — The social economy (people, power, and time)

### F11. Children grow up — errands become apprenticeships

*Extends `Kid`; uses `lifecycle.rs` (unwired aging) + RelKind::Apprentice.*

**What.** At age ~12 a `Kid` graduates: if a master paired them (repeated
errand-givers count), they take that `Craft` at Journeyman; else a
random open vocation. The village visibly replenishes itself.

**Spec.** `Kid` gains `age_days`; on threshold, remove `Kid`, add `Craft`
(preferred master = most-frequent `tasked_by` within trust band). Log
line "Tam takes up the awl." Without this, every death is net loss —
with it, the roster breathes. Requires wiring `lifecycle.rs` age ticks.
**Size: M.** *Depends on F12 for what happens when there's no open shop.*

---

### F12. Shop succession & inheritance

*`game/succession.rs` exists (unwired in game_ui); brainstorm "death &
continuation" groundwork; `feat/hard-winter-shows-succession` pending —
check overlap.*

**What.** When a master dies, the workshop passes to their senior
apprentice, else their eldest kin, else it stands derelict — and a
journeyman of the trade may claim it (a promotion story).

**Spec.** On `Dead`: find heir by (apprentice at shop → kin in household
→ same-vocation journeyman with no workplace). Transfer `Craft.workplace`
+ building `worker`. Kin also inherit `Purse` ×0.8 (the lord's death-duty
is the other 0.2 — a quiet `Problem::DeathDuty` grievance). Unclaimed
shops become `DerelictNearby` sources — the street visibly declines and
can recover. **Size: M.**

---

### F13. The sanction ladder — shame before violence

*`sprint-7-non-violent-deposition.md`; uses `sim/reputation_qsp.rs`,
`game/organizing.rs` votes, `game/feuds.rs`.*

**What.** Orgs (and the lord) can impose escalating non-violent sanctions
on a person: gossip → shun → boycott → picket → stocks → exile. Crowd
reaction decides whether it lands or backfires.

**Spec.** `enum Sanction { Gossip, Shun, Boycott, Picket, Stocks, Exile }`
with the review's ladder table. Org vote (existing meeting machinery,
success = ≥3 voices same category) picks a target + level; `reputation_qsp`
runs the crowd reaction — a failed stocks attempt *raises* the target's
standing. Exile walks them off the map edge. This is the mechanic F5's
ergot-blame and F1's `SuspectHer` escalate into — the witch-trial path
is emergent, not scripted. **Size: L.**

---

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

### F15. Festivals & feast days

*Reuses "festival loaves", `Playing`, minstrel machinery, `sprint-4-tea-herbology` ritual bonding.*

**What.** ~4 feast days a year (seeded saints'-day names). The village
gathers at the market: special goods consumed (festival loaves, cheese,
ale), `Music`/`Joy` wants satisfied en masse, bonds strengthen between
co-attendees, grievances surface in drink — and the tax collector is
never invited but always shows.

**Spec.** `FestivalEvent` on the calendar (tick % 90 season boundary);
villagers divert to market; consume stock; `relationships` +bond among
pairs present; one toast speech-bubble per attendee drawn from wants/
problems (the reveal machinery on a party line). Org can sponsor a feast
(spend stock → +perceived power, F14). **Size: M.**

---

### F41. The Heart Helper — a matchmaker

*Side-feedback request; numbered F41 to keep earlier references stable —
position it early, it mostly reuses existing machinery. Pairs with F1 as
another "important single character".*

**What.** One heart-helper (the village matchmaker — an elder who knows
everyone's business) watches for lonely hearts and engineers meetings
between them. The player can do it too: brokering a match is the same
organizer-brokered pattern as `Pair up`, only for hearts, and it's a
real trust/power lever.

**Spec.**

- `TownRole::Matchmaker` — exactly one, seeded like the herb-wife
  (`SocialClass::Working`, no `Craft`). An elder sprite, a bench or
  market spot she works from ("the heart-helper").
- `Lonely heart` detection: villagers with an unmet `Want::Love`
  (machinery exists in `desires.rs`). She keeps a private
  `MatchNotes { Vec<(Entity, Compatibility)> }` — compatibility sampled
  from shared wants, complementary problems, attachment style
  (`sim/psychology.rs`), and *not* being close kin or a `Kid`.
- `Activity::Matchmaking`: she walks to one heart, then fetches the
  other ("come, I want you to meet someone") — both are pulled to a
  meeting spot for a Talk-like exchange. Chemistry roll →
  `RelKind::Sweetheart` bond (new RelKind between Friend and Spouse) or
  a polite miss (`Problem::Rejected`, ~30-day cooldown before she tries
  that pair again).
- She takes a small fee or gift on success ("a pie from the grateful
  groom") — 2–4g into her `Purse`, and her own `Joy`/`Esteem` tick up.
- Player action `Matchmake` (contextual row): enabled when the player's
  `Heard` ledger holds a `Love` want for *two* different people — you
  have to know hearts to mend them. Success gives both parties a trust
  bump with the player (the organizer who mended them) and counts as a
  visible action for F14 perceived power.
- Cross-feud pairs (from `feuds.rs`) get flagged "a match the families
  won't like" — drama, a bigger payoff if it lands, a feud-stoking miss
  if it doesn't.
- Her own secret: "the heart-helper who never made a match for herself"
  — she can carry an unmet `Love` want like anyone.
- Sweetheart pairs become F16 wedding candidates when that lands; until
  then a sweetheart bond satisfies `Love`, deepens bonds, and shows on
  the card ("walking out with Tam").
- UI: her card reads "Heart-helper · Working"; `Matchmaking` marker
  `♥`; the `Matchmake` action sits on the contextual row next to
  `Pair up` with its own disabled-reason line ("you only know one
  heart that's looking").

**Verify.** Exactly one matchmaker across seeds; no Kid/kin pairings;
bond forms only on the chemistry pass; rejected pairs respect cooldown;
player `Matchmake` gated on two heard `Love` wants; fee and trust bump
land; cross-feud flag sets. **Size: S-M.**

---

### F16. Marriage & households

*Brainstorm "marriage/divorce — TBD".*

**What.** Pairs with `RelKind::Spouse` form from courtship (Love want met
repeatedly with the same person); households merge; kids can arrive.

**Spec.** Courtship = repeated mutual Love-satisfying interactions →
`Spouse` bond → one moves house (`household.rs` occupant swap) → a
wedding mini-festival. Children spawn rarely into households with a
spare bed (feeds F11). Bereavement grief hook (pending branch
`villagers-grieve-the-dead` — check overlap). Divorce only on feud-level
relationship collapse. **Size: L.**

---

### F17. Inside jokes & shared stories

*Brainstorm "inside jokes"; cheap, high charm.*

**What.** Pairs that share events (both present at a festival, both
sanctioned, both survived the wolf night) accrue a `shared_bits` list;
their Talk lines reference it ("still thinking of the goose at the
fair?"), giving dialogue memory instead of per-line amnesia.

**Spec.** `struct SharedBits { Vec<(Entity, BitKind, tick)> }`; Talk
draws from it before the generic line table. Purely additive flavor —
but it's the difference between a cast and *these people*. **Size: S.**

---

### F43. Issues belong to trades — common ground is work

*Side-feedback request; numbered F43 for stable refs. The organizing
game gets its real friction: a baker's grievance isn't a mason's, and a
room full of different trades doesn't converge.*

**What.** Issues carry a vocation scope. A complaint only "counts"
toward discovery among people of the same trade — so a mixed meeting
splinters by trade unless it finds a *bridge issue* everyone shares.

**Spec.**

- `IssueKind` gains `vocation: Option<Vocation>` (None = the shared
  grievances — `Taxed`, `Broke`, the stinking reach from F42 — that
  count for everyone). Meeting convergence (the ≥3-same-category rule)
  requires an exact `(kind, vocation)` match; a mason's `NoMaster`
  doesn't join the shepherd's `WolfAtFold`.
- Each vocation gets 1–2 signature problems in `desires.rs` on top of the
  commons: farmer → `RentOnTheField`; baker → `GrainShort`;
  mason/toolmaker → `WornTools`; shepherd → `WolfAtFold`; tailor →
  `ClothPrice`; luthier → `GreenWood` (unseasoned); kids stay on
  `Joy`/chores.
- A meeting that fails to converge now logs why: "the talk splinters by
  trade" — teaching that homogeneous meetings converge easily, while
  cross-trade organizing needs the rare common issues. Discovery names
  carry the scope: "masons with no master" vs "the tax collector's cut"
  (everybody's).
- Design intent, per Gary: **bridge issues are the organizer's real
  game.** You can farm one trade for quick wins, but village-scale power
  (F14) needs the issues that cross trades — tax, stink, bread. The
  environmental pressures from F42 are deliberately seeded as common
  issues for this reason.
- UI: issues list shows the tag ("the masons' …", "everyone's …");
  meeting log names which trade's voices echoed.

**Verify.** Mixed-trade meeting with only signature complaints never
converges; same-trade meeting does; a common issue converges in a mixed
room; discovered issue names carry the vocation tag. **Size: S-M.**

---

## Wave 3 — The world outside (roads, travelers, and the far powers)

### F18. Footfall trails — the map learns where people walk

*`sprint-7-roads-and-trails.md`; `sim/trails.rs` exists (Active Walker
Model), unwired; `feature/trail-strength-feedback-loop` pending — check.*

**What.** Per-tile `trail_strength` reinforced by walking, decaying to
terrain baseline; stronger trails are cheaper to path through → real
desire-lines emerge and are rendered.

**Spec.** Add `Trail` layer to the game_ui tilemap; feed footfall on
every walk_path step; movement cost ÷ (1 + strength). Render as a
darkening/widening overlay (exists in the module's intent). Emergent:
the path to the herb-hut wears in; a derelict street greens over.
Tuning knobs per the ant paper; expose in debug overlay. **Size: M.**

---

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

### F42. The land keeps score — slow environmental pressure

*Side-feedback request; numbered F42 for stable refs. Three creeping
degradations that generate issues through the existing Heard→Issue
machinery: the point is organizing content, not scenery.*

**What.** The map accumulates damage from what the village does: the
wood thins and isn't replanted, the worked-out pit floods, and the river
carries the town's leavings downstream. Each pressure crosses thresholds
that seed problems → complaints → discoverable issues.

**Spec.**

- New `sim/environment.rs` (or `game/` — sim is the better home since
  bins reuse it): per-tile counters with daily update:
  - `tree_cover: f32` — each wood gather takes ~0.1; regrowth +0.005/day
    on undisturbed forest only (pasture, trail, and built tiles don't
    recover — trampling kills regrowth). Below 0.3 the tile draws
    stumps; wood yields scale with remaining cover → toolmaker/luthier
    `NoInput("wood")` spreads → issue **"the shrinking wood"**.
  - `pit_water: f32` — a `Quarry` zone feeds stone/ore gathering (mason,
    toolmaker); each gather adds water (digging below the water table).
    Yield drops at 0.4/0.7, flooded dead at 1.0 — it renders as a pond
    where the pit was → mason `Problem::DryPit` → issue **"the drowned
    pit"**. (F32's cave becomes the alternative source later — one
    pressure creating a demand for another feature.)
  - `effluent` on river tiles — tanner/works/houses dump a little daily
    upstream; flow propagates a `stink` field *downstream only*: fisher
    catch −, `Problem::Stink` on homes near the reach, small illness
    uptick within it → issue **"the stinking reach"**. Feeds F10's
    smell field and F31's fisher; upstream stays clean so direction
    is legible on the map.
- All three are *slow* — tuned so a casual 360-day year barely shows
  them, but sustained play visibly wears the land (thinned treeline,
  flooded pit, a brown reach below town). Counters are seeded with a
  little head start so mid-length sessions see the first threshold.
- Issue plumbing is the point: when a problem's *cause* is
  environmental, its Talk/meeting line says so ("the wood's thin this
  side of the hill"), and `Heard` entries key to the issue names above.
  All three are `vocation: None` common issues (F43) — the land's
  grievances are the ones everybody shares.
- Rendering: stump/dimmed-tree variants, murky-brown tint on downstream
  water tiles, water-fill in the pit. No new sprites — tint and terrain
  swaps only.
- Later hooks (not this PR): a woodward/coppicing answer, the lord's
  forest law (F25 edict forbidding cutting — a pressure that makes an
  *illegal* economy).

**Verify.** Gathering depletes cover and regrowth needs undisturbed
tiles; the pit floods at cumulative gather count, not calendar; stink
field never propagates upstream; each pressure produces its issue via
≥3 heard entries; mid-length seeds show threshold 1 within ~a year.
**Size: M.**

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

1. F1 Herbologist — the asked-for single character; wires a whole
   finished module into the living game.
2. F2 Shepherds/flocks — asked-for; livestock substrate for F3/F8/F9.
3. F3 Chickens — cheap after F2, proves Livestock generality.
4. F41 Heart Helper — side-feedback priority; second "important single
   character", mostly reuses want/reveal/pairing machinery already built.
5. F5 Foraging/mushrooms — deepens the herb-wife's world; first real
   hidden-poison story.
6. F43 Issues belong to trades — small, and it makes every later
   grievance richer; do it before the roster of scoped problems grows.
7. F7 Vermin/cats — S, makes stock feel alive; dog already had a job,
   now cats do too.
8. F12 Shop succession — deaths stop being pure loss; inheritance feeds
   F28 heirlooms and F33 continuation.
9. F11 Children grow up — the roster breathes; pairs with F12.
10. F18 Footfall trails — visual payoff for all this walking; module
    already written.
11. F42 The land keeps score — the land's grievances are the bridge
    issues F43 makes matter; needs a few features' worth of economy to
    bite.
12. F13 Sanction ladder — the emergent-stories engine; witch-trial and
    ergot paths converge here.
13. F15 Festivals — ties music, food, bonds, and the calendar together;
    the first thing that feels like a *village*, not a sim.
