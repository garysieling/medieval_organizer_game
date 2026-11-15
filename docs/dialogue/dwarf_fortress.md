# Dialogue research: Dwarf Fortress

**Issue:** #1196 · **Sources:** Dwarf Fortress Wiki (Talking, Social skill, Personality value, Adventurer mode, Reputation).

Dwarf Fortress is the deep end: conversation as a *verbs menu over a world model*. Adventure mode offers topic menus (greet, ask directions, bring up an incident, request duties, argue a value); fortress mode runs socialization autonomously in taverns. Under both sits the machinery that matters for us — rumors, identities, values, and skills trained by the moves you pick. Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

- **Topic menus, not trees** — a talk is a stack of menus: greet, bring up an incident/rumor, ask after a person/place, request a duty, propose a role (hearthperson, lieutenant, troupe), argue a value, perform. Each pick opens sub-choices; the talk ends when either side leaves.
- **Incidents as first-class topics** — anything that happened (a kill, a theft, a brawl) is an *event* you can "bring up" — spreading it as rumor is a conversation move.
- **Arguments over values** — a dedicated move: state a value, press it, or respond with flattery/dismissal/joke/passivity. The response type is what gets trained.
- **Fortress mode has no player talk at all** — dwarves socialize in taverns autonomously: needs, friendships, grudges, and lovers all form off-screen from proximity and compatibility.

## What drives variation

- **Values** — each creature holds philosophical values (tradition, cooperation, sacrifice…) on a spectrum; values clash or harmonize between speakers, and — uniquely — **can be changed by a won argument**.
- **Personality facets vs values** — they can conflict inside one creature ("values romance, cannot form romantic bonds — and is bothered by it"), which is a whole characterization engine in one line.
- **Social skills** — Judge of Intent, Persuader, Flatterer, Intimidator, Pacifier, Comedian, Liar: *trained by the move you pick* (argue → Persuader, flatter → Flatterer, dismiss → Intimidator). High skill persuades anyone into anything.
- **Identities** — every creature can carry aliases; reputation accrues *per identity*. NPCs link appearance↔name; asking after an agent's true name in a town where they're undercover gets a correct "don't know them."

## Choices, relationships, consequences

- **Rumor propagation** — incidents spread only through talk and only via witnesses who live to leave; kill every witness before the site offloads and the rumor dies. Reputation is built by bragging up your own incidents or performing them to spectators who re-tell.
- **Reputation scoped per entity** — a new civilizational entity doesn't know your name; a fresh alias resets you.
- **Roles through talk** — become a hearthperson/lieutenant/performer by asking the right leader; leaders reciprocally recruit.
- **Quests through talk** — "request duty" is the quest-giver interface.

## What it remembers / forgets

**Remembers:** every historical event as a queryable incident, per-identity reputations, value shifts from arguments, which witnesses saw what.

**Forgets:** nothing, practically — that's DF's point. The cost is that the "dialogue" is menu labels over the ledger, not lines.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Argue a value one-on-one** — press/flatter/dismiss, a won argument moves the listener's *convictions* (Personality value, Social skill) | **want** | Meetings move positions; nothing lets a private talk argue a villager round on the issue. → **#1234** |
| 2 | **Rumor propagation through talk** — incidents spread via living witnesses; brag your own feats (Reputation) | **have** | Mysteries propagate facts one-per-telling through talk; `reactions` spreads witnessed events through the crowd; notices are the public version |
| 3 | **Kill the witnesses and the rumor dies** (Reputation) | **have, inverted** | Our informers carry names to the law and counter-intel is *finding* them — same lever, opposite chair; the sign-hunt already plays this |
| 4 | **Per-identity reputation + aliases** (Reputation) | **half have** | Informers/the-sign/`counter_intel` model *being known* vs *unknown*; full alias-swapping would mean the player renaming to shed notoriety — a thin fit for a game where the village knows your face |
| 5 | **Social skills trained by move choice** (Social skill) | **reject** | The player has no stat sheet by design — the bond ladder is the progression, and a Persuader level would turn organizing into a grindable number |
| 6 | **Roles through asking** (hearthperson/lieutenant/troupe) | **have** | The asks ladder (#1188) — Introduce → ComeAlong → Follower → CoLeader is exactly DF's "ask the leader for a place" made gradual |
| 7 | **Duty/quest through talk** | **have** | Quests + propositions do this; ours are bonded to the ask ladder, not flat |
| 8 | **Autonomous socialization** — fortress dwarves befriend/lover/grudge off-screen | **have** | Villager chats + the relationship ledger do this; #1230 (typed chat deltas) is the remaining RimWorld-shaped gap, not a DF one |
| 9 | **Values-vs-facets internal conflict** ("values romance, can't love, is bothered by it") | **want-ish, no ticket** | Our traits + InnerLife/grievances gesture at inner conflict, but nothing *states the contradiction*. Fold into #1234's back-and-forth: a villager arguing against their own grain is where this would show |
| 10 | **Performance spreads the tale** — recite the deed to spectators who retell it | **half have** | Feast/meeting speeches address a crowd; the re-telling step is mysteries/notices. A deliberate "tell the story of what we did" performance could ride #1234's argument shape — revisit after |
| 11 | **Menu-of-topics conversation UI** | **reject** | Explicitly the thing we're not building — the talk reads as speech, not a menu of incidents |

## What the pass confirms about our shape

DF is proof that *the ledger is the game*: strip its menu labels away and the rumor/identity/value machinery still generates all the stories. We already sit on most of that ledger (mysteries, informers, reactions, asks). The one move DF has that we lack is the **private argument** — the organizer going door to door, winning one conviction at a time between meetings. That's #1234.
