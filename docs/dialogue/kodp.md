# Dialogue research: King of Dragon Pass / Six Ages

**Issue:** #1198 · **Sources:** Six Ages development blog ("Advice"), Six Ages manual, Six Ages Fandom wiki (Clan Circle), King of Dragon Pass dev blog, Steam leaders guide.

KoDP/Six Ages is the *council* model: you play the clan, not a person, and your seven-member circle is simultaneously the UI, the information layer, and a cast of characters. Their advice is the game's dialogue — every screen and every event is narrated through voices who have skills, families, gods, and agendas. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Advisors as interface** — seven named leaders picked by the player; each screen's advice comes from the relevant face. No dialogue trees — advice is *positioned counsel*: lines attached to roles, topics, and events.
- **Events as the conversation** — a story event presents 5–7 responses; each advisor comments, recommends, or warns about tradeoffs. There is "seldom just one right answer"; some responses delay, some spawn chains.
- **The circle is a persistent choice** — who you seat changes what you're told. An empty seat hurts clan mood *and* a yearly magic point. Advisors age, die, and are replaced by their younger kin.
- **Pick-the-hand** — some choices ask which leader executes; the pick *is* the test ("the leader with the best Bargaining makes the test on behalf of the clan" — Bargaining vs. the other side's Bargaining).

## What drives variation

- **Skill determines who speaks and whether it lands** — "the game favors the best advice": whoever's skill fits the topic gets heard, and the executor's skill is what gets tested.
- **Personality flavors counsel** — a pacifist advises against war and is bad at intimidation; the goat-obsessed one mentions goats. Personality is *hidden at first* — learned from campaign pitches and a hundred meetings, not a tooltip.
- **Agendas inside advice** — advisors have families and gods; "occasionally their advice serves their own agendas better than it serves the clan." Kin back kin in events.
- **Relations tracks** — clan-to-clan standing, favors owed, promises to raid; events consult them.

## Choices, relationships, consequences

- **You decide, someone performs** — the decision and its execution are split: the advisor who carries your answer is tested, and a merciful face is the wrong pick for intimidation.
- **Clan mood** — a shared morale layer events and seats feed.
- **Consequences compound across events** — a delayed problem returns; chains remember earlier picks.
- **Generational churn** — the cast turns over; keeping the circle competent is a standing problem.

## What it remembers / forgets

**Remembers:** promises made to other clans, event chains, relations tracks, advisor histories, clan mood.

**Forgets:** conversational detail — advice is functional counsel, not a transcript.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Skill weights whose counsel lands** ("the game favors the best advice") | **want** | Meeting bids weight concept overlap + thread + bond — not whether the speaker's trade touches the matter. → **#1238** |
| 2 | **Advisors with hidden personalities, learned over time** | **have** | Our traits + reveals do this — a villager's bend is learned through talk and asks, exactly the "get to know them" arc |
| 3 | **Advice that serves the advisor's agenda** (kin back kin) | **have** | Society already runs villager agendas that constrain what they lend their names to; PushBack in meetings is kin-and-grievance-shaped |
| 4 | **Pick-the-hand: decide, then a named member executes and is tested** | **have** | `delegate_role` + campaign seats — the org's roles are carried by named members, and dungeon/unavailability gates the pick |
| 5 | **The council as a picked, persistent composition** (empty seat costs) | **have** | Campaign seats/roles are the circle; CoLeader is the top rung of the ask ladder |
| 6 | **Clan mood layer** | **have** | Alignment (Bridged/Amplified/Transformed) reads member buy-in; village morale runs underneath |
| 7 | **Responses that delay vs resolve vs spawn chains** | **have** | Campaign arc beats chain consequences the same way; deferral is a real option at meetings |
| 8 | **Generational replacement of advisors** | **have** | Heirs/aging already run the village's turnover |
| 9 | **Favors/relation tracks between groups** | **have** | Contracts/debts between people; campaign standing between org and powers |
| 10 | **No ducking an event** — it must be answered | **half have** | Meeting agendas force the decision; some world events are passive weather rather than demands — a deliberate choice, not a gap |
| 11 | **The player as the clan, not a person** | **reject** | The game's premise is the opposite — one organizer inside the crowd, which is what makes the talk matter |

## What the pass confirms about our shape

KoDP's lesson is that **positioned voices can carry an entire game's dialogue** — no conversations needed, just the right person saying the situated thing. Our meetings already work that way; the piece it sharpens for us is KoDP's "favor the best advice": a voice should carry the weight of where it stands. That's #1238.
