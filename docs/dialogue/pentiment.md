# Dialogue research: Pentiment

**Issue:** #1200 · **Sources:** Pentiment Wiki (Dialogue system, Persuasion), Neoseeker/GameFAQs walkthroughs, GameRant backgrounds guide.

Pentiment is authored-branching done with unusual honesty: no dice, no hidden RNG — persuasion is **your accumulated history scored against a threshold**, and the game literally tells you when a pick is being filed away. Its other two signature structures are the *meal as a gossip venue* and *chosen backgrounds that lock/unlock options for the whole game*. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Authored trees with memory-scored gates** — branching conversations where prior choices accrue +/− points (shown as blue/red triangles); persuasion succeeds when the total meets a target.
- **"This will be remembered"** — key choices get a flag; you learn later what it bought you.
- **Meals as scenes** — picking who to eat with is a dedicated scene: three courses, conversation between mouthfuls, gossip and leads on the table, favour won or lost.
- **Backgrounds** — five early picks (Wanderjahre place, temperament, education, skills) that permanently open/lock dialogue options and auto-add to persuasion where relevant.
- **Mind Palace** — ethical dilemmas spawn an internal beat where Beatrice/Socrates/Saint Grobian argue Andreas's conflict — inner voices, Disco-lite.
- **Time as the constraint** — the day fits a few scenes; choosing who to eat with chooses who you don't.

## What drives variation

Your background picks, everything you've said that got remembered, who you dined with, and which act's consequences have ripened (Act 1 choices bear fruit in Act 2, years later).

## Choices, relationships, consequences

- **Persuasion as accounting** — no roll; the ledger of your prior behavior *is* the check.
- **Favours won/lost at table** — meals are where opinion moves.
- **Generational consequences** — decisions echo across the time-skips; the town's children grow up inside what you chose.
- **Suspicion has a memory** — accusing wrongly isn't forgotten; the village holds what you said.

## What it remembers / forgets

**Remembers:** every flagged choice, background, who fed you what, act-spanning grudges and debts.

**Forgets:** unflagged chit-chat — like every authored game, memory is a flags-and-points ledger.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **The shared table as gossip venue** — a meal is a scene with its own material and its own clock | **want** | `SharedMeal` is a bond delta and the feast ring a venue, but the player can't sit at a family's table and hear the household's news. → **#1241** |
| 2 | **Persuasion = accumulated history vs threshold** | **have** | The bond ledger + stage ladder IS this — asks test the accumulated record, not a roll (#1188) |
| 3 | **"This will be remembered" flag** | **half have** | Picks are recorded (mood, ledger, transcript) but nothing marks a pick as consequential in the moment — the TalkMood/vibe feedback carries the weight instead; deliberate, not a gap |
| 4 | **Background picks that lock/unlock options** | **reject in form, want in spirit** | No chargen — but #1236 is the same mechanic earned: your line hardens through play rather than being picked at the start. The emergent version is better for us |
| 5 | **Meals move favour** | **have** | SharedMeal +0.20 already does the bond math; #1241 adds the content |
| 6 | **Mind Palace inner voices** | **reject** | Same call as Disco — villagers carry InnerLife; the player's head isn't a cast |
| 7 | **Time pressure: choose who gets your hours** | **have** | Routines + the day clock already force the allocation; talks take real ticks |
| 8 | **Suspicion remembers a wrong accusation** | **have** | Grudges + PushBack chill + mysteries — a wrong word lingers on the ledger |
| 9 | **Generational consequences** | **have** | Heirs, aging, the village outliving decisions — and #1239 (mentorship marks) extends it to traits |

## What the pass confirms about our shape

Pentiment's persuasion-without-dice is the vindication of the bond-ledger design: accumulate real history, test the total. What we lack from it is the *venue* — the meal as the place where a household opens up. That's #1241.
