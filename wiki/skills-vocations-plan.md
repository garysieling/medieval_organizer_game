---
agent: devin-local
session: dapper-blarney
created: 2026-10-30T15:18:27Z
---
# Skills, Class & Vocations in the Windowed Game

Build the class/vocation/skill model (~30 characters incl. a few children, secret wants/problems, homes, workplaces, trade, a tax collector) and surface all of it in the windowed UI: building inspectors, villager cards, issue discovery, errands on the map, and music.

## Context

The windowed game (`game2-core/src/bin/game_ui.rs`, minifb) currently has 8 named villagers who wander and eat, and buildings as bare `(x, y, sprite)` tuples — not clickable, no entities behind them. Backend pieces exist but are unwired or shaped differently: `jobs.rs` (a different Job enum, used only by `game_loop` bin), `economics.rs` Market/Trader, `recipes.rs`, a different 6-tier `Class` in `stratification.rs`, `lifecycle.rs` Age/LifeStage (unwired), `RelKind::Mentor/Apprentice` (ready-made for master pairing), and the house-meeting machinery (`organizing.rs` + `Game::advance_meeting`). This work builds the new model in fresh `game/` modules and wires it into `game_ui.rs` without disturbing the text game or `jobs.rs` users.

**Decisions from Gary's answers:**
- ~30 characters (expects a perf pass), random class+vocation+gold for the player (never landowner), petit bourgeois = masters who own their workplace, landowner is an off-map presence, hybrid trade (person-to-person for inputs, shop stockpiles for finished goods), overseers take a cut AND boost work, master pairing is seeded + emergent + organizer-brokered, secrets surface in both the per-person inspector and a persistent issues list.
- Luthier added as a 10th vocation; a rare travelling minstrel; villagers can own instruments and play for fun.
- **Children:** a small share of the cast (~10%, ~3 of 30). Very little gold (~1–3g). No vocation/level — they run **odd jobs**: fetch/deliver items for 2–5 gold. No child sprites in the packs → drawn ~0.6× scaled with "(child)" on the label + "Child — runs odd jobs" in the inspector.
- **Tax collector:** one guaranteed PMC character who wanders the village collecting for the (off-map) lord. Works a roster schedule — each character is visited once per year (360 days), so roughly one collection every ~12 days. For now a **fixed cut** (~10% of the target's purse, flat-rate flag for future variable mechanics). A collection seeds the `TaxCut` problem on the target → shared grievance feeding issue discovery. The player is on the roster too.

**Confirmed in spec (will flag, not re-ask):** level order is Journeyman → Apprentice → Master (journeyman is lowest — inverted from history, but stated twice consistently). One "year" = 360 days (the `tick % 90` season cadence gives 4×90-day seasons).

## Data Model — new `game2-core/src/game/` modules

### `vocation.rs`
- `enum SocialClass { Working, PetitBourgeois, ProfessionalMiddle, Landowner }` — Landowner exists only as an off-map reference in wants/problems/log text.
- `enum Vocation { Farmer, Baker, Toolmaker, Knight, Tailor, Glazier, Printmaker, Mason, Priest, Luthier }` + PMC roles `{ Overseer, TaxCollector }`. Knight, Priest, Overseer, TaxCollector are `ProfessionalMiddle`.
- `struct Craft { vocation: Vocation, skill: f32 /*0..1*/, home_shop: Option<Entity>, master: Option<Entity> }` — level derived from skill: <0.4 Journeyman, 0.4–0.75 Apprentice, ≥0.75 Master. Interaction options = 1/2/3 by level.
- `struct Kid;` marker: no Craft, no skill growth; runs `OddJob` errands instead. Children get wants/problems like anyone.
- Per-vocation tables: `building_kind()`, `inputs()` (e.g. Baker ← wheat; Glazier ← sand/pigments; Luthier ← lumber; Printmaker ← paper+ink; Mason ← stone; Tailor ← cloth; Toolmaker ← lumber+ore; Farmer/Knight/Priest none), `outputs()`, `actions(level)` per-vocation flavored list (e.g. Farmer: sell crops / hire a hand / lease land from the lord).
- Skill growth: `+0.005/tick` while working; **×10** when `master.is_some()`; output rate **×0.5** and a `NoWorkplace` problem when `home_shop.is_none()`. Reaching Master while owning a workplace → class graduates `Working → PetitBourgeois`.
- Task delegation: a Master can task Apprentice+Journeyman, an Apprentice can task a Journeyman — and anyone can hire a `Kid` for a fetch/deliver (`TaskOrder { from, to, errand, pay }`, pay 2–5g on completion).

### `desires.rs` — secret inner lives
- `enum Want { GoodFood, Love, Faith, Shelter, Security, Esteem, Music, Instrument, BetterTools, DifferentTrade, Joy }` — Maslow-flavored; ~2 sampled per person, vocation/class-weighted (e.g. Glazier → `DifferentTrade` "a glazier who'd rather be a luthier"; Priest→Faith; Journeymen→Esteem/mastery; kids→Joy/play). `Instrument` satisfied by buying a lute.
- `enum Problem { NoHome, NoWorkplace, NoMaster, NoInput(good), DerelictNearby(building), OverseerCut, TaxCut, Broke, ... }` — evaluated/seeded dynamically; dynamic ones recompute each day.
- Reveal mechanics: Talk → ~15% roll to reveal one unrevealed want/problem; house-meeting turns → same roll, **+10% per parallel complaint already voiced that meeting** (same issue-category). Meeting success = ≥3 attendees voice the same category.
- `Heard { entries: Vec<(Entity, IssueKind, u64)> }` on the organizer (the player): every revealed item is journaled; entries **expire after 360 days**; ≥3 live entries in one category → `Issue` discovered.
- `Issue { kind, name }` — discovery notification is exactly "Organizer discovers issue X" (log line + same-text toast). X is a named pattern, e.g. "the bread shortage", "masons with no master", "the tax collector's cut".
- `meeting_remark`/`talk_response` draws lines from a speaker's wants/problems (keeps the per-person voice table; kids get child-flavored lines).

### `household.rs` — buildings as entities
- `struct Building { kind: BuildingKind, occupants: Vec<Entity>, worker: Option<Entity>, overseer: Option<Entity>, stock: HashMap<String, u32>, capacity }`.
- `enum BuildingKind { Home, Workshop(Vocation), Church, Mill, Granary }`. Homes hold 1–4 occupants (children count); unhomed → `NoHome`. Workshop(Vocation) with `worker: None` is derelict → `DerelictNearby` problem accrues to passersby. Missing workshop for a vocation → `NoWorkplace` for those workers.
- Stockpiles: inputs in, finished goods out, capped (e.g. 20/good). Shop stock is what buyers purchase; a building's "for offer" list is its non-zero stock + prices.

### `livelihood.rs` — work, trade, errands, overseers, taxes, minstrel
- `struct Purse { gold: f32 }` on every character (incl. player). Seed gold: Working ~5–20g, PM ~50–200g (10×), landed-PB in between, kids ~1–3g; off-map lord nominally 1000× (rents referenced in text only).
- Daily agenda (new `Activity` variants: `Working, Fetching(e), Buying(e), Selling, Errand(e), Collecting(e), Playing`): each day a working character works (skill↑, output → own stock or shop stock); when an input runs dry → `Fetching`: pick a producer (omniscient, distance-biased via `walk_path`), walk over, buy person-to-person (gold↔good); finished goods → deposited to workplace stock (a child may be hired to run the delivery for 2–5g instead); consumers with unmet `GoodFood`/`Instrument`-type wants → `Buying` walk to the shop, buy from stock.
- `OddJob` errands: a worker/master posts a fetch-or-deliver; an idle `Kid` nearby takes it, walks the route, earns 2–5g from the hirer. Children otherwise stay near home and play.
- Overseers (PMC, farm/mason only): `overseer` on the building; workers' output rate boosted (~+25%) and the overseer takes a cut (~20% of sale value) → seeds `OverseerCut` problems (a shared grievance).
- **Tax collector schedule:** `struct TaxRoute { roster: Vec<(Entity, u64 /*next due*/)>, at: usize }`. Each character (children included, flat-rate or exempt — flag) is due once per 360 days; the collector walks to the next due person, takes the fixed cut (10% of purse, min 0 — can't take what they don't have; a `Broke` target stays broke), speech bubble on collection, `TaxCut` problem added to the target. Rounds the roster forever; the player is in it.
- Emergent pairing: a Master with no junior and a nearby same-vocation unpaired junior → `RelKind::Apprentice/Mentor` forms.
- Minstrel: rare traveller event (~every 250–450 days) — a figure walks the road to the market, plays for a day or two (music-note marker, villagers gather), satisfies `Music`/`Joy` wants, then leaves. Reuses the `traveller_path` machinery; sprite = a humans-pack figure (e.g. MiniPrinceMan/MiniCavalierMan).

## Seeding — `Game::new_seeded` rework (probabilities, not constraints)

- NAMES grows to ~30 medieval names (+ a few child names); ~26 adults + ~3 children + 1 tax collector; `villager_sprites` picks look by class/vocation (worker look for laborers, noble for PMC/PB incl. the collector, scaled-down for kids); number-key selection 1-9 keeps working, click-select is primary.
- Class roll ≈ 80% Working, ~12% PM (Knights/Priests/Overseers + the guaranteed collector), rest become PetitBourgeois by starting as Master+shop; lord off-map. Vocation weights: Farmer .22, Baker .12, Mason .10, Tailor .09, Toolmaker .08, Luthier .06, Printmaker .05, Glazier .05, Priest .08, Knight .08; overseers derived from farmer/mason counts.
- Levels ≈ 55% Journeyman, 30% Apprentice, 15% Master. Seeded master-pairing for ~half of juniors.
- Homes: ~12–16 houses placed around/inside the town wall + outskirts; 1–4 occupants each (kids live with an adult occupant), randomized overflow → `NoHome`.
- Workplaces: one per vocation present with ~85% coverage (some vocations deliberately lack a building); a possible extra derelict shop. Sprites: existing houses/mill/church/smith reused + `tint()` variants (bakery, tailor, printshop, glazier, luthier, mason's yard, barracks).
- Player: rolls random non-owner class + vocation + gold + wants/problems — "just another character"; panel header shows the roll (e.g. "The Wanderer — Journeyman Mason, 14g"); the player is on the tax roster.

## UI — `game_ui.rs`

- **Buildings clickable:** `click_map` picks nearest building too; inspector swaps between villager card and building card. Building card: name/purpose, worker/overseer/occupants by name, stock levels, "for offer" goods, derelict tag. Selected building gets a marker/underline like selected villagers.
- **Villager card additions:** class + vocation/role + level (e.g. "Apprentice Baker · Working", "Tax Collector · PMC"), skill meter, revealed wants ("wants good bread", "bored with the trade"), revealed problems ("no master", "homeless", "the tax collector's cut"), home + workplace names, master's name. Children: "Child — runs odd jobs" + their wants. Secret items never shown until revealed.
- **Issues list:** "KNOWN ISSUES" card under the members line — discovered issues persist there.
- **Contextual action row** under the main buttons: `Work` (when at own workplace — does the player's level-appropriate vocation action incl. tasking), `Pair up` (when the selected junior has no master and a same-vocation master exists — the organizer-brokered apprenticeship), `Send errand` (hire a selected kid to fetch/deliver, 2–5g).
- **Map life:** villagers visibly walk errands (existing walk-path style); the tax collector walks door to door and demands the cut with a speech bubble ("The lord's share"); children drawn ~0.6× with "(child)" on the label; `MARK_*` gains `♪` (playing), an icon for errands/collecting; derelict buildings drawn dimmed/no chimney smoke; minstrel walks the road with a `♪`.
- **Meetings:** speech lines now come from wants/problems; parallel complaints visibly build (later speakers echo the theme); discovery line at resolution.
- **Rivals** stay decorative (no class model) this round.

## Files

- **New:** `game2-core/src/game/vocation.rs`, `desires.rs`, `household.rs`, `livelihood.rs` (+ `mod.rs` entries)
- **Modified:** `game2-core/src/bin/game_ui.rs` (seeding, buildings as entities, inspector cards, issues card, kid rendering, tax collector, activities/markers, context actions, minstrel event), `game/organizing.rs` (remarks/reveal hooks — additive, keep existing tests' API intact), `game/villager.rs` (Activity variants), `PLAY.md` (controls/features)

## Suggested phases (separate PRs)

1. **Model + read surfaces:** vocation/desires/household components, seeding (incl. children + tax collector), ~30 villagers, clickable building inspector, villager card, empty issues card.
2. **Livelihood:** work/errands/walking, trade + stockpiles, skill growth, overseer cut+boost, tax collection schedule, task delegation + child odd jobs, emergent + brokered pairing, player `Work`.
3. **Secrets:** wants/problems, reveal rolls, parallel-complaint boost, heard ledger w/ 360-day expiry, issue discovery.
4. **Music:** instrument want/trade, `Playing` activity + `♪` marker, rare minstrel.
5. **Perf + polish:** ~30-char perf pass, sprite tint variety, evidence screenshots.

## Verification

- Unit tests per module in repo style: seeded distributions (ratios within probability bands across seeds, ~10% children, exactly 1 collector), homes 1–4 + homeless path, skill ×10 mentored / ×0.5 unworshopped, PB graduation, stockpile cap, trade moves gold+goods, child errand pays 2–5g on delivery, overseer cut accrues, tax roster visits each person once/360d + takes fixed cut + adds `TaxCut`, reveal probability bounds, parallel-boost ordering, 360-day expiry, ≥3-pattern discovery, tasking legality by level.
- `game_ui` tests in existing style: building click selects + inspector draws, villager card shows vocation/skill, child labels flagged, collector walks to a due target and the cut shows in a bubble, issues card lists a discovered issue, derelict building dims, minstrel sprite path, no starvation regressions.
- `cargo test --lib`, `cargo run --bin game_ui -- --dump-frame N` evidence BMPs at several days/seeds/weather.

## Risks / assumptions

- Level naming is deliberately Journeyman→Apprentice→Master per spec (inverted vs. real guilds).
- Tax cut is a fixed ~10% of purse (flagged for future variable mechanics); children are taxed the same fixed cut of their small purses — cheap to special-case later.
- ~30 villagers + more buildings is the biggest visual/perf change; the plan includes a perf pass (O(n²) separation pass, scaled-sprite cache, errand path recomputation only on retarget/block).
- No child sprites exist — scaled-down adult figure + explicit "(child)" flag, per Gary's direction.
- Children don't age up this round (`lifecycle.rs` aging stays unwired — a year is 360 ticks=days and the sim doesn't run years yet).
- The mill stays a landmark (bakers convert wheat→bread directly); rivals stay un-classed.
- Talk/meeting lines migrate from `village_concerns` to wants/problems — kept additive so existing tests still hold (hunger/sickness remain as `Broke`-like dynamic problems).
