# Dialogue research: Stardew Valley

**Issue:** #1195 · **Sources:** Stardew Valley Wiki (Friendship, Modding:Dialogue, Bouquet), Stardew Modding Wiki (dialogue JSON templates, events tutorial).

Stardew sits opposite RimWorld on the spectrum: **fully authored, fully keyed** — every villager carries a `Characters/Dialogue/<NPC>` file where each line is addressed by season, weekday, weather, and heart level. Nothing is generated; variation comes from an enormous key-space plus a friendship meter that decides which slice of the file you're allowed to hear. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Keyed daily lines** — keys like `spring_Mon`, `Rainy_Day_3`, `<season>_<day>_<hearts>`; the game picks the most specific key that exists and falls back to generic. One click = one line; a second click can give a "chat" line.
- **Inline commands** — dialogue strings embed `$q` (question) / `$r` (responses that carry a *friendship delta* and a response-key), `$e` (fallback after first ask), `$c` (chance fork), `$d` (world-state fork: bus repaired, Joja gone), `$b` (page break), pronoun tokens.
- **Heart events** — cutscenes gated by `Friendship <NPC> <points>` preconditions plus time/location; choices inside the scene carry point deltas. These are the game's real "dialogue trees" — tiny authored scenes unlocked by the meter.
- **Marriage pool** — spouses get a whole second file: `funLeave_/jobReturn_` schedule lines, `Rainy_Night`, seasonal 9-heart-plus variants.
- **Festival dialogue** — per-festival lines, often keyed to hearts, so the same feast reads different at 2 vs 8 hearts.

## What drives variation

Who you are to them (hearts), what day/season/weather it is, whether you're married to them, whether a world-flag is set (bus, Joja), and a dice roll (`$c`). The *player's history with that one villager* is the dominant axis — almost everything is per-NPC.

## Choices, relationships, consequences

- **Friendship points** — 250/heart. +20 first talk of the day; further talks same day are free. Gifts by taste (loved +80 base, hated −40), quest deliveries +150, movie invites, heart-event choices ±, slingshot −30 a hit.
- **Decay** — friendship decays daily *if you didn't greet them*; spouses decay harder. Neglect has a cost independent of what you do.
- **Gates on the meter** — marriage candidates cap at 8 hearts until given a **bouquet** (an item that declares intent); the pendant proposal is rejected with *typed reasons* (`Under10Hearts`, `NpcWithSomeoneElse`, `NeedHouseUpgrade`).
- **Weekly gift cap** — two gifts a week (+10 bonus for maxing) — an explicit anti-grind throttle.
- **Witnessed misbehavior** — rummaging a garbage can within 7 tiles of a villager: −25 and they *bark that they saw you*. Private act, public cost.

## What it remembers / forgets

**Remembers:** the point total per villager, which heart events have played, gift tastes you've proven (the Gift Log fills in what you've given), bouquet/dating state, gifts-this-week counter, and which `$q` answers you gave (via the `_old` fallback key).

**Forgets:** the content of any talk — there's no transcript; "memory" is the meter plus fired-event flags.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **First-greeting-per-day counts; spam doesn't** (+20/day, repeats free) | **want** | Our act deltas (Greet/Agree each warm) stack per *talk*, and nothing caps re-opening Talk the same day — the bond can be ground. → **#1232** |
| 2 | **Typed rejection by cause** (`RejectMermaidPendant_Under10Hearts` vs `NpcWithSomeoneElse`) | **have** | #1188's reasoned refusals — the ask names the rung you haven't reached |
| 3 | **Heart-gated scene unlocks** (heart events at point thresholds) | **have** | Stage-gated reveals/asks (#1188) + mysteries do the unlock-by-bond shape — and ours is earned by *content*, not a point total |
| 4 | **`$q`/`$r` inline questions carrying deltas** | **have** | Player picks + per-act deltas (#1189's ask-topics are exactly a `$q` with richer consequences) |
| 5 | **Keyed daily dialogue** (season + weekday + weather + hearts → line) | **have, better** | The criteria cascade generates the same specificity space without a key table — our `Has`/`Lacks` facts are their key-prefixes made composable |
| 6 | **Decay on neglect** (−/day ungreeted, spouses harder) | **have** | The 0.99 ledger decay is the same pressure, continuously applied — arguably kinder (contact resets the clock rather than a daily flag) |
| 7 | **Bouquet: an item that declares the stage** (8-heart cap until given) | **half have** | The signature/pledge at a meeting is our public-declaration beat — private ledger, public moment. A *giveable token* as a distinct object is close enough to the pledge to skip |
| 8 | **Witnessed misbehavior** (garbage rummage −25 + "saw you" bark) | **have, better** | `virtue::witness` (moral acts, co-witness share) + `reactions::witness` (moral-shock recruitment) — our witnesses do more than a bark |
| 9 | **Gift-taste economy** (loved/hated gifts, Gift Log learns tastes, 2/week cap) | **reject** | A buy-friendship economy lets wealth substitute for organizing work — the game's premise is that you *talk* people round. Taste-learning itself is already covered by reveals + ask-topics |
| 10 | **Married pool as a second file** (spouse schedules, Rainy_Night, 14-heart meter) | **have** | `RelKind::Married` + spouse statuses feed the cascade the same way — and our marriage keeps living in the sim rather than swapping to a script file |
| 11 | **`$d` world-flag forks** (bus repaired, Joja gone) | **have** | `any:*`/`pair:*` criteria + world statuses do this declaratively |
| 12 | **`$c` chance forks / `$e` once-only fallback** | **have** | Seeded `TalkRng` picks + spent lines are the same pair of ideas, deterministic |

## What the pass confirms about our shape

Stardew shows the *ceiling of the authored approach*: the writing is charming precisely because a human placed every line — but the machinery under it (a point meter, threshold gates, typed rejections, a daily-greeting rule) is identical in kind to ours, and our generated cascade covers its key-space without a per-NPC file. The one rule worth taking verbatim is the anti-grind: **the day's first greeting carries the warmth** — filed as #1232.
