# Dialogue research: Wildermyth

**Issue:** #1201 · **Sources:** Wildermyth Wiki (Relationship, Event, Hook), Steam personality-stats analysis.

Wildermyth is procedural narrative's poster child: comic-panel events chosen by **targeting criteria** — the game picks who speaks and which event fires off personality stats, hooks, and relationships. Its distinctive contribution for us: relationships as *typed five-level ladders that unlock content*, and **hooks** — named wounds from a generated history that become resolvable personal arcs at deep bond. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Events chosen by targeting criteria** — an event fires only if the party matches its requirements (personality stats present, a hook held, a relationship of the right kind/level). The cast is picked per-event by *highest relevant stat*.
- **Personality picks the speaker AND the line** — the goofball never goes serious; stats decide both who gets the role and the flavor of their lines.
- **Panel-cutscenes, not trees** — linear event scenes with decision points; choices modify state and sometimes the outcome, but events aren't branching conversations.
- **Hook quests ("opportunities")** — at relationship level 3+, a hero's hook can trigger a bespoke mini-arc for them plus one companion; resolving it grants a permanent mark (retirement age, gear, ability). Cooldown + per-chapter caps throttle them.
- **Interludes** — between chapters, quiet relationship scenes develop the bonds.

## What drives variation

Personality stats (bookish, snark, goofball, hothead, coward, leader, healer, poet, loner, greedy), hooks held, relationship kinds/levels, and generated history (each hero is rolled with a past that *caused* the hooks).

## Choices, relationships, consequences

- **Typed relationship ladders** — Friends (Crony→Bloodbond), Lovers (Crush→Soulmate), Rivals: 5 levels each, with mechanical bonuses per level (adjacency block chance, vengeance damage) and different event pools per kind.
- **Bonds form without the player** — relationships arise and shift autonomously from shared battles and tasks.
- **Compatibility from stats** — romance/rivalry compatibility is a sum over personality pairs; only stats over 50 count.
- **Legacy** — heroes carry scars, relationships, and earned bonuses into later campaigns; locked romances recur.

## What it remembers / forgets

**Remembers:** hooks (resolved or not), relationship levels, campaign history, scars, who's fought beside whom, legacy locks.

**Forgets:** the words — events are state, not transcript.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Hooks — named past-wounds that become resolvable arcs at deep bond** | **want** | Traits are static, Wants/Problems are recomputed daily — nothing is *carried from a villager's history* to be laid to rest. → **#1243** |
| 2 | **Typed relationship ladders with per-level names/effects** | **have** | `RelKind` + the `stage_of` ladder (Stranger→CoLeader) is exactly the shape — named rungs, gated content |
| 3 | **Personality picks the speaker and shapes the line** | **have** | Trait multipliers on reply weights + vocation-aware pools — the same mechanism, generated rather than authored |
| 4 | **Event targeting criteria** (party must match to fire) | **have** | The criteria cascade (`Has`/`Lacks`/`pair:*`/`any:*`) is a generalization of their event targeting |
| 5 | **Bonds deepen by shared activity, autonomously** | **have** | `record_interaction` on meetings/meals/aid — the ledger accrues without player involvement |
| 6 | **Compatibility from personality pairs** | **half have** | Traits weight replies; no explicit pair-compatibility — folded into **#1230**'s grill questions (RimWorld's hidden constant is the same mechanic) |
| 7 | **Per-relationship-kind event pools** | **have** | `pair:*` criteria gate pools by relationship kind already |
| 8 | **Rivals as a first-class track with content** | **have** | `RelKind::Rival`, feuds, the `old_scores` tree — rivalry already has its own material |
| 9 | **Interludes — quiet bond-development scenes** | **half have** | Passing chats + feasts fill the role; a dedicated quiet-scene beat isn't separate from everyday talk — and arguably shouldn't be |
| 10 | **Legacy heroes carrying history forward** | **reject** | One village, one timeline — but the heirs/aging machinery gives the same "the cast carries forward" flavor inside the sim |

## What the pass confirms about our shape

Wildermyth is the closest architectural match to our conversation system in the whole survey: **criteria-targeted material where personality chooses the voice** is literally `converse` + `criteria`. What it has that we don't is the *backstory that matters* — our villagers are fully present-tense (traits + today's problems); a named wound from before the game began is the missing depth. That's #1243.
