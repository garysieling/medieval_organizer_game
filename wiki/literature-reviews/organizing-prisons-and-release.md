# Lit Review: The Dungeon — Imprisonment, Hunger Strikes, and Release Campaigns

**Survey Date:** 2026-10-31
**Agent:** Literature Survey Agent
**Domain:** Social Movement History / Prison Organizing / Recruitment Sociology
**Series:** Organizing research round 2 (user-requested). Feeds `wiki/campaign-plan.md` piece C6 (the dungeon) and deepens C5 (bystanders). Companion survey: `organizing-internal-structure.md` (sibling PR #188)
**Confidence:** High — the suffragette and Attica records are primary-source rich; Jasper & Poulsen is the standard stranger-recruitment paper

## Purpose

Round-2 surveys established that arrests *outside* the walls are a backfire risk for power (Otpor trained for arrest; Birmingham was planned for cameras). This survey covers the *inside*: what imprisonment does to a movement, what prisoners do back, how release becomes a campaign in itself, and — critically for the game — **how people who were never organized come to own an issue**.

## Finding 1: Arrest Generates Recruitment Through Condensing Symbols

Jasper & Poulsen's *"Recruiting Strangers and Friends"* (*Social Problems*, 1995) is the missing half of the network-recruitment model our relationship system embodies:

- **Two recruitment channels, not one.** Networks recruit people who already know a member; **moral shocks recruit strangers** — people with no tie to the movement at all, reached by broadcast images/accounts that make the issue viscerally personal.
- **Moral shocks ride condensing symbols** — a single image or scene that packs a whole grievance into something instantly legible (their animal-rights data: strangers recruited directly by rhetoric, no network needed).
- **Implication for the game's hidden-strength model:** the org's relationship web recruits *adjacent* people; an arrest/violent backlash recruits *non-adjacent* people — including, per Gary's brief, "people not in your group who take ownership of the issue as part of their self interest." Bystander conversion is a distinct mechanic from member recruitment, not a weaker version of it.

## Finding 2: The Prisoner as Organizer — Status, Demands, the Inside Continues

The record is consistent that imprisonment does not stop organizing; it relocates it:

- **Marion Wallace Dunlop, Holloway, 1909** — a lone suffragette's self-initiated hunger strike (91 hours, released) invented the tactic that 1,300+ imprisoned suffragettes would repeat. Their demand was *status itself*: to be classed as political prisoners, not common criminals. **The prison was an arena with its own demands** — and the demand was recognition.
- **Attica Liberation Faction, 1971** — prisoners wrote a 28-point Manifesto of Demands to Commissioner Oswald before and during the uprising. The demands were "strikingly moderate" (visitation, legal access, dissent rights); the *process* of drafting them was itself organizing — and the manifesto was aimed at publicity outside the walls as much as the administration inside. Oswald accepted most demands; the deal collapsed on **amnesty**, and Rockefeller's assault killed 39. Imprisoned people can negotiate — and the state's refusal point tells you what it actually fears (accountability, not the demands).
- **Jailhouse lawyers** (the Sostre strand at Attica) — legalistic self-representation as a second organizing track inside, parallel to radical action.

**Mechanic insight:** an `Imprisoned` member shouldn't be inert — the dungeon is a venue where organizing continues at a different tempo (demands drafted, strikes called, lawyers worked). The interesting state isn't captured/not-captured; it's *what the prisoner's campaign-in-miniature is doing*.

## Finding 3: Force-Feeding and Cat-and-Mouse — Repression That Keeps Losing

The suffragette record is a masterclass in how not to handle a prisoner:

- **Force-feeding was a propaganda gift.** Nasal-tube feeding of hunger strikers produced exactly the "powerful condensing symbols" Jasper & Poulsen describe — restrained women, tubes, public disgust. Keir Hardie raised it in Parliament repeatedly. The state's attempt to prevent martyrs *manufactured* the moral shock.
- **The Cat and Mouse Act (1913)** — release hunger strikers when weak, re-arrest when recovered — tried to escape the bind. It failed twice over: released prisoners went into hiding (the act created fugitives), and the revolving-door cruelty "was held in distaste by the general public." A state caught cycling release/re-arrest is spending legitimacy every pass.

**Mechanic insight:** every state response to a prisoner should be scored like round-2's decision-tree rule — *does the state have a non-losing response?* Force-feed: loses (moral shock). Release-and-rearrest: loses (looks absurd, creates fugitives). Hold without process: slow legitimacy drain. Amnesty/concede status: the prisoner wins. A dungeon full of organized prisoners is a trap the jailer fell into, not a cost the movement paid.

## Finding 4: The Release Campaign Is a Pressure Accumulator

Amnesty International's founding model (Benenson's 1961 "Forgotten Prisoners" appeal → adoption groups) is the cleanest documented release-campaign architecture:

- **A prisoner is *adopted*, not just petitioned for.** Each group takes responsibility for 2–3 specific prisoners — name, face, story. The prisoner becomes the campaign's *charter*, not its topic.
- **Pressure = volume × time, not a single act.** Amnesty's own handbook is blunt: "One letter to a Minister of Justice is not pressure. A second letter is. Two letters from one person every single week is a little more pressure." Release mechanics should accumulate sustained small actions, not resolve on one big ask.
- **Elite voices are weighted pressure.** "One statement by an Archbishop can have a greater effect than a petition signed by several thousand individuals" — pressure has a prestige multiplier, so recruiting a priest/noble to the release campaign is worth a thousand letters.
- **The Urgent Action network (1973–)** institutionalized speed: a prisoner's fate could move fast, so the network had to answer fast — response latency is a campaign stat.

**Mechanic insight:** a release campaign has a `pressure` meter fed by (letters/meetings × participants × time) + (elite endorsements × prestige) — and the jailer's resistance is its own resource. Nothing dramatic is required: the model that freed prisoners of conscience for 60 years is *boring persistence*, which is exactly what a game can make legible.

## The Synthesis

For C6, the dungeon is not a punishment event — it's a **venue transfer** with four game-relevant properties:

1. Prisoners keep organizing (status demands, manifestos, strikes) — the campaign shrinks to a cell but doesn't stop
2. State responses to prisoners are all-losing when the movement has `frame readiness` (countertactics doc) — every handling choice feeds moral shocks or fugitives
3. Arrests broadcast to *strangers* via condensing symbols — bystander recruitment runs on moral shock, not relationship ties
4. Release is a campaign: an adopted prisoner, accumulated pressure, elite multipliers, and a jailer whose every response costs something

## Game Mechanic Proposals

### Mechanic 1: `Imprisoned` as an active state *(C6)*
`struct Imprisoned { since, charge, status_demand }` — the prisoner can `draft_demands` (a manifesto the outside campaign can publish — org members outside gain an agenda item), `hunger_strike` (burns own health, generates moral shocks per day, forces a state response each tick), or `petition` (jailhouse-lawyer track: slow, legalistic, can win early release on a roll).

### Mechanic 2: Moral shocks recruit strangers *(C6, deepens C5)*
A `MoralShock` event (arrest witnessed, force-feeding leaked, crackdown on sympathetic actors) broadcasts to non-members within sight/hearing: each bystander whose live stake intersects the shock's issue tag gains the campaign's issue as a grievance — "the issue organizes them" without a single 1-1. Weighted by condensing-symbol power (a knight dragging a child > a writ served). Network-tied bystanders (friends/family of the jailed) convert harder — the two channels from the paper, literally.

### Mechanic 3: The jailer's dilemma *(C5/C6)*
Every state response to a prisoner is a losing-branch evaluation: `ForceFeed` (ends the strike, emits a high-power MoralShock), `ReleaseAndRearrest` (cat-and-mouse: emits repeated moderate shocks + fugitive risk), `HoldWithoutProcess` (low-drama legitimacy drain × days held), `ConcedeStatus`/`Release` (prisoner wins the round; the campaign banks the win as capacity). Counter-AI picks its least-bad option by the movement's `frame readiness` — an unready movement just gets quietly held.

### Mechanic 4: Release campaign as pressure accumulator *(C6)*
`struct ReleaseCampaign { prisoner, pressure, endorsers }` — pressure += (sustained small actions) each day + prestige × elite endorsements; jailer resistance -= its own legitimacy drain. Release rolls when pressure > resistance. Amnesty's insight baked in: persistence beats spectacle — five villagers writing weekly outlast one dramatic protest, unless the protest *is* the moral shock.

### Mechanic 5: Adoption as charter *(C6)*
`found_release_campaign(prisoner)` creates a Campaign whose issue is the prisoner — dissenters are impossible (the issue is a person), framing defaults to MoralDuty, and joining requires no org membership: bystanders converted by Mechanic 2 can take seats directly. This is the cleanest implementation of "people build a campaign just around your release."

## Key Sources

- **Jasper & Poulsen, "Recruiting Strangers and Friends," *Social Problems* 42(4), 1995** — A+. The stranger-recruitment paper; the two-channel model and condensing symbols are its core.
- **Suffragette prison record** (UK Parliament heritage pages; London Museum collection; OpenLearn) — A/A+. Dunlop's 1909 strike, force-feeding practice, Cat and Mouse Act texts and failure.
- **Attica Liberation Faction Manifesto + historiography** (Freedom Archives primary PDF; *Race & Class* 2011 reprint; Harvard CR-CL on the demands' moderation; NYS Archives timeline) — A. Inside-organizing, demands-as-process, the amnesty refusal point.
- **Amnesty International archive** (AI@50 timeline; 1981 membership handbook "what is pressure"; Urgent Action records) — A. The adoption-group architecture and sustained-pressure doctrine, primary.

## Next Steps

- Feeds `campaign-plan.md` C6 directly: `Imprisoned`, release campaigns, and the bystander channel are now specced; C5's reaction table gets the jailer's-dilemma branch evaluation.
- The "prestige multiplier" on endorsements connects C6 to the existing `Citizen.authority`/`influence` fields — a priest's letter is mechanically different from a mason's.
- Open question: should a prisoner's own organizing skills (C3) still apply inside? The record says yes — Dunlop's strike was self-taught, Attica's manifesto was collectively drafted — so `Imprisoned` members keep their `Organizing` skills but apply them to a smaller room.
