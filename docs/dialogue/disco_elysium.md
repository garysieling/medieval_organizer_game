# Dialogue research: Disco Elysium

**Issue:** #1197 · **Sources:** Disco Elysium Wiki (Skills, Thought Cabinet, checks), Gamepressure skill-check guide.

Disco Elysium is dialogue as the *entire* game — combat, exploration, and character sheet all live inside the talk box. Its two big ideas: the **player's skills are interlocutors** (inner voices with personalities that argue among themselves) and **checks are content gates with teeth** (white = retriable, red = one-shot, failure usually still gives you a scene). Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Branching trees with embedded checks** — authored trees where active checks appear as special nodes; the check's outcome routes the branch.
- **Skills as speakers** — 24 skills across 4 attributes, each with a personality; above ~4 points a skill starts interjecting in dialogue. Skills banter, disagree, and give the same scene different counsel. The player's own head is a multi-party conversation.
- **Passive checks** — run silently in the background (skill + 6 vs threshold); passing unlocks options, and some lines exist *only on a failed passive* — low skill sees different content, not just less.
- **White vs red** — white checks can be retried after skill growth or changed circumstances; red are one-shot and consequences stand.
- **Thought Cabinet** — 53 acquirable "thoughts"; internalizing takes in-game time (research bonus → permanent bonus/penalty + dialogue unlocks); 12 slots max, some mutually exclusive, forgettable for a skill point.

## What drives variation

Your build (which voices are loud), accumulated modifiers (clothes, drugs, thoughts), whether you passed this check before, and which thoughts you chose to internalize. Two players genuinely hear different scenes.

## Choices, relationships, consequences

- **Fail-forward** — a failed check rarely dead-ends; it gives a different, often better-written scene, sometimes at a cost of Health/Morale (social failure literally wounds).
- **Ideology accumulation** — dialogue choices accrue toward political alignments; enough of a line unlocks the matching thought — the player's *rhetoric* is a persistent, mechanical fact.
- **Retry economics** — white checks reopen only on real change (skill up, new info, new item), so you can't save-scum a conversation.
- **Damage as social consequence** — failed checks can cost Morale; pushing too hard has a price in the same currency as everything else.

## What it remembers / forgets

**Remembers:** every red check's outcome, internalized thoughts, ideology tallies, items worn (as modifiers), and which passives have fired.

**Forgets:** the shape of a finished conversation — like Stardew, memory lives in flags and tallies, not the transcript.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Rhetoric accumulates into convictions** — said enough, a line unlocks a thought that opens/closes options; some mutually exclusive (Thought Cabinet) | **want** | Villagers carry InnerLife/stances and the org's decisions are even contradiction-tracked (`noted_contradiction` beat) — but the *organizer's own* line is tracked nowhere. → **#1236** |
| 2 | **Passive checks incl. failed-passive content** | **have** | The criteria cascade is a passive-check engine (`Has`/`Lacks`); `Lacks` criteria are literally failed-passive lines |
| 3 | **White/red check discipline** — retriable only on real change vs one-shot | **have** | The ask ladder does exactly this: refused asks reopen as the stage rises (#1188); once-per-beat events are the red checks |
| 4 | **Fail-forward** — a blown check still gives a scene | **have** | Vibe-probe miss forks to a different tree (#1187); refused asks answer with a reasoned line, not silence |
| 5 | **Skills as arguing inner voices** | **reject for player; have for villagers** | The player's head isn't a cast by design — but `InnerLife` gives *villagers* the same layered interiority that steers their lines |
| 6 | **Morale/Health damage from social failure** | **half have** | PushBack chills the bond and vibe-mismatch closes depth; there's no player-side wound currency — and arguably shouldn't be (the stakes are the cause, not the psyche) |
| 7 | **Equipment/drugs as situational check modifiers** | **reject** | An inventory-modifier economy doesn't fit — our modifiers are relational (bond, mood, last talk) and stay that way |
| 8 | **Internalize over real time** — research phase before a thought lands | **have-ish** | Mysteries work this shape in miniature (facts accrue over tellings until the payoff); the stage ladder is the slow-internalization arc for bonds |
| 9 | **12-slot cabinet, forgetting costs** | **reject** | A slot economy for the player's beliefs is RPG bookkeeping; the conviction layer (#1236) should just be what the village can see |

## What the pass confirms about our shape

Disco's great trick — the player *is* a conversation — doesn't port: our player has no psyche to voice, by design. But its second trick does: **what you keep saying becomes who you are, mechanically**. Villagers already get that treatment (InnerLife, stances, traits); the organizer's own accumulating line is the one missing layer. Filed as #1236.
