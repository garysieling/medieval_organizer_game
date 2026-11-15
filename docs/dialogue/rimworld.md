# Dialogue research: RimWorld

**Issue:** #1194 · **Sources:** RimWorld Wiki (Social, Mental break, Thoughts/Memory Social), Steam community dev-mode analysis (compatibility values), game knowledge.

RimWorld is the control case: **it has no dialogue at all** — zero authored lines, no conversation UI between pawns. What it has instead is a *social ledger*: every interaction is a typed event carrying an opinion delta, and the emergent story comes from the ledger, not the words. That's exactly the layer under our generated talk, so it reads as an audit of our `relationships.rs` machinery. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How "dialogue" is structured

Two pawns near each other roll a **random social interaction** from a fixed type table — no trees, no lines. The type carries everything: a name for the log, an opinion delta, a mood thought, a selection weight, and trait restrictions:

| Interaction | Opinion | Weight | Restriction |
|---|---|---|---|
| Chitchat | +0.66 | 1.0 | not psychopath/inhumanized |
| Deep talk | +15 | 0.075 | same |
| Kind words | +15 | 0.01 | initiator must be Kind |
| Slighted | −5 | 0.02 | initiator not Kind |
| Insulted | −15 | 0.007 | initiator not Kind |
| Strange chat | −3 | exclusive | initiator Disturbing/Void-touched |

The shape that matters: **mostly mild positives, rare big positives, rarer negatives gated by traits.** A Kind pawn can only ever make things warmer; everyone else occasionally makes things worse. The village's social texture is *entirely* this table rolling all day.

## What drives variation

- **Hidden compatibility** — a per-pair constant (pseudorandom from the two pawn IDs, penalized by age gap; *not* traits, per dev-mode analysis) that biases whether a pair's rolls land as deep talks or insults. Two pawns with low compatibility feud forever and nobody can fix it.
- **Traits gate the type pool** — Kind unlocks kind words and locks out slight/insult; Abrasive/ugly-beauty traits shift the baseline opinion independent of events.
- **Mood breaks cascade** — a mental break can be an *insulting spree* (targeted or random): the breaking pawn follows a victim stacking up to −33 mood in insults, which can break the victim in turn. Insults escalate to **social fights** — actual violence with injury — and the fight resolves the spree.

## Choices, relationships, consequences

- **Opinion ledger** — per-pair −100..+100, shaped by traits/beauty plus the typed rolls; gates romance (min opinion to attempt), fuels feuds → fights below ~−50.
- **Thoughts vs opinion** — two separate memories: the *opinion* on the person (long, ~20-day events) and *thoughts* as mood items (Insulted −5 for 2 days, stacks ×10 at 0.9 decay). A slight chills the *relationship* for weeks and the *mood* for days — different clocks.
- **Romance arc on the ledger** — opinion threshold → attempt → Rebuffed/proposal thoughts (−18, 25 days) or marriage → breakup thoughts (−20, 30 days). The relationship drama is all thoughts, no lines.
- **Warden→prisoner rapport** — a warden chats with a prisoner; rapport builds on their social skill and eventually *converts* (recruit). Social contact as the whole recruitment mechanism.
- **Ritual speech** (Ideology/Royalty DLC) — a gather-the-colony speech rolls morale/inspiration on the speaker's skill.

## What it remembers / what it forgets

**Remembers:** the pair opinion, the thought stack (with per-item expiry), romance states, social-fight outcomes as health.

**Forgets:** anything said — there's nothing to remember; and the *interaction event itself* only persists as its deltas. Compatibility is fixed at worldgen.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Typed social interactions** — named event + opinion delta + weight + trait gate (Social) | **half have** | The ledger's the same (`record_interaction`, Conversation/Conflict/Aid deltas, kind inference, 0.99 decay) and the *player's* talk types its acts (PushBack chills, Agree warms, vibe-safe warms faster). But villager↔villager meetings log a flat `Conversation` whatever the chat held — a spat warms like a friendship. → **#1230** |
| 2 | **Hidden pair compatibility** — fixed per-pair constant biasing roll outcomes (dev-mode data) | **want** | Our traits weight replies but no pair-constant exists; rolled-from-entities compatibility would make some pairs *read* quarrelsome before anything's said. Folded into **#1230**'s grill questions |
| 3 | **Thoughts vs opinion split** — fast mood items over a slow bond (Thoughts/Memory Social) | **have** | `TalkMemory` (20-day mood toward the player) + the bond ledger is the same two-clock shape — and ours colors the next talk's reveals/gist, which RimWorld's mood never does |
| 4 | **Social fight as feud resolution** — low opinion + insults → violence with injury (Mental break) | **have** | Feuds (`feuds.rs`) and grudges already escalate with real consequences; ours resolves through the community (law, gibbet) rather than a scuffle, which fits better |
| 5 | **Insulting-spree cascade** — a break targets social damage that can break the victim (Mental break) | **want-ish, no ticket** | A "takes it out on the village" cascade is real, but our break analogues are grief/illness and a targeted-insult campaign needs #1230's typed chats first. Revisit after #1230 |
| 6 | **Warden rapport → recruit** — repeated contact converts (Social) | **have** | The whole talk→meeting→ask→Follower ladder is this idea made playable (#1188); ours is richer — the player *earns* the conversion, it isn't a stat that ticks over |
| 7 | **Romance on the ledger** — opinion thresholds, rebuffed/proposal/breakup thoughts (Social) | **have** | `RelKind::Lover`, `Married` forms (annulled/deserted), romance routines — and our courtship is witnessed, not just ledgered |
| 8 | **Ritual speech** — gather-all morale/inspiration roll on speaker skill (Ideology) | **have** | Meeting rites (#1185) + agenda motions do the gather-them beat; the prayer/benediction already lands the register |
| 9 | **No transcript at all** — the game never shows what pawns say | **reject** | RimWorld's famous lesson is that the ledger alone *carries* a colony sim; but this game's whole premise is the words — the village talks. Take the ledger mechanics, keep our lines |

## What the pass confirms about our shape

RimWorld proves the thesis this game already runs on: **a social ledger, typed and decaying, generates story better than authored lines do.** The pleasant surprise is how much of the table we already sit on — `record_interaction` is the same machinery down to the decay tick, and our player-talk path already types acts the way RimWorld types interactions.

The one clean gap is the *feed*: RimWorld's villager-villager interactions are where the typing lives, and ours log a flat delta regardless of content — the transcript knows whether that chat was a spat or a kinship, but the ledger never hears it. That's #1230.
