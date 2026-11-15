# Dialogue research: Skyrim

**Issue:** #1193 · **Sources:** UESP wiki (Speech, Generic Dialogue, Disposition, A Few Words with You), Beyond Skyrim Arcane University (Dialogue Systems for Writers, Quest Implementation), S.T.F.U.'s dialogue-subtype docs, Creation Kit conventions.

Skyrim's dialogue is studied as a working system: what it runs on, what varies, what it remembers, and how the pieces map onto ours (`game/conversation.rs`, `session/dialogue.rs`, `run_meeting`). Verdicts: **have**, **want** (with a gap ticket), **reject** (with why).

## How dialogue is structured

Three separate pools, layered:

1. **Top-Level Branches** — the player-initiated menu. Each option is its own branch, pooled from every source that targets that NPC (generic lines, quests, factions, DLC). The NPC's tree isn't authored as one flow; it's a *union of candidate topics* gated per-condition.
2. **Greetings / Hellos** — a separate stack that fires *before* the menu and on walk-by. Written in a fixed order with reset timers (~0.5 in-game hours): the unique, important greetings run first in sequence, then the stack exhausts to generic "Yes?" / "Need something?" until the timer resets.
3. **Scenes** — NPC-NPC scripted conversations. Two kinds: *major establishers* (once, near entrances, during quests) and **random conversations** — ~3 hand-authored exchanges per characterized pair, triggered on proximity, on reset timers, replayable (Nazeem's three scenes with Carlotta).

Ambient **barks** (idles, combat taunts, crime/guard reactions) run under the same condition system but never open the menu.

## What drives variation

Every line carries **conditions**: speaker faction (mechanical factions like `AllowWorldInteractions`, not just story ones), voice type, race, disposition value, quest stage, player skill level, bounty held, armor worn, game-hour. The engine picks the most conditioned match — the same "most specific wins" cascade ours runs.

Notable condition-driven sets:

- **Skill-aware greetings** — "you're a sneaky sort" fires when Sneak is high; merchants compliment Speech.
- **Crime/bounty layers** — guards open with a bounty confrontation (pay/resist), generic NPCs get "Wait... I know you" variants.
- **Disposition-tinted greetings** — the UESP generic table marks red lines (disposition −1 or lower) and green lines (+1 or higher) in the *same* greeting pool.
- **Faction armor** — a guard reads your armor as allegiance.

## Choices, relationships, consequences

- **Speech challenges** — where a check gates the branch, *three* verbs sit on the same lock: **Persuade** (Speech skill roll), **Intimidate** (your computed "scariness" vs theirs), **Bribe** (a formula-priced gold cost — shows the price up front, always succeeds if you can pay). Plus **brawling**: a fists-only duel that resolves the ask. Three currencies — nerve, threat, coin — one gate.
- **Disposition** — a per-NPC scalar (−2..+2). Favors raise it (+1 per favor quest), crimes and refusals lower it. It colors greetings, gates marriage/followers, and feeds macro systems: "Help the People" favor *counts* across a hold progress the thane track.
- **Radiant quests through dialogue** — "A Few Words with You" (Favor013) is a whole favor quest delivered through speech challenges: a quest-giver names a target NPC, you persuade/bribe/intimidate/brawl them, and the giver's disposition rises. A high *target* disposition can skip the check entirely — friendship pre-pays the ask.
- **Relationship-threshold dialogue swaps** — marriage and follower status swap the whole dialogue pool; a spouse stops speaking like a stranger.

## What it remembers / what it forgets

**Remembers:** per-NPC disposition, per-hold bounty, faction memberships, quest stage flags, favor counts, greeting/scene reset timers.

**Forgets:** almost everything said. No transcript memory, no "you asked me that already," no learned facts about the player beyond mechanical flags. The greeting reset timer is the *only* talk-state memory — and it's a freshness tool, not a recollection.

## The mechanics, mapped

| # | Mechanic (source) | Verdict | Notes / ticket |
|---|-------------------|---------|----------------|
| 1 | **Greeting stack with reset timers** — unique hellos in fixed order, exhaust to generics, reset ~0.5h (UESP Generic Dialogue; AU) | **partly have** | Our greeting exchange leads the cascade and talks close themselves when material's spent (#1179). What we lack is *cross-talk* exhaustion: two talks in a row reset clean. → **#1226** |
| 2 | **Three keys per gate: persuade/intimidate/bribe** — skill roll, scariness formula, priced gold (UESP Speech) | **want** | Ours has one key: the RelStage ladder (#1188). Coin and nerve as alternative ask currencies → **#1225** |
| 3 | **Random conversations** — authored 2-NPC scenes on timers for characterized pairs (AU Dialogue Systems) | **want** | Our chats are all generated; nothing authored, nothing recurs. Signature scenes for story pairs → **#1227** |
| 4 | **Disposition scalar** — favors ±1, colors greetings, gates thresholds (UESP Disposition) | **have** | `RelStage` (Stranger→Acquaintance→Known→Follower) + bond + `TalkMemory` mood (#1183) cover it, and ours is richer (mood is shaped by *how the last talk ended*, not just favors) |
| 5 | **Skill/faction/bounty-conditional lines** — one pool, most-conditioned wins (UESP Generic; AU) | **have** | Our `Facts` criteria (`self:`, `other:`, `pair:`, `any:`) + specificity cascade is the same mechanism — and ranks/ideology registers land through it (#1180) |
| 6 | **Radiant favor quests through speech** — favor chain + disposition payout + macro count (Favor013; thane favors) | **have, weaker** | Parcel quests bond you to the giver (`quest_bond`). The *macro count* (favors → village-level unlock like thane) is thinner: the campaign is our macro, and member count is the count. Close enough; no gap |
| 7 | **Rumors as quest hooks** — innkeeper "heard any rumors?" starts quests (S.T.F.U. subtypes) | **have** | `ASK_TOPICS` mystery/hearsay draws pull live mystery facts (#1189); mysteries spread through gossip already |
| 8 | **Relationship-threshold pool swap** — spouse/follower gets a new dialogue set (UESP Marriage/Followers) | **want** | Lines gate by status but the arc doesn't shift at Follower — a sworn villager still talks weather. → **#1228** |
| 9 | **Command-mode follower dialogue** — "I need you to do something" opens a command shell (UESP) | **reject** | Wrong game: we organize through meetings, roles and asks, not a companion command shell. The *feeling* it buys (a follower taking orders) lives in our org agenda verbs already |
| 10 | **Brawling as a speech resolution** — fists duel resolves the ask (UESP A Few Words) | **reject** | A duel-minigame inside a conversation is a hero fantasy, not an organizer one. If intimidation lands at all, it lands through #1225's nerve currency, not fists |
| 11 | **Walk-by forced dialogue** — NPCs open on you unprompted ("Get up here!") | **reject** | Pulls the camera and the player's agency for a bark; our rare-event visitors approaching on their own already carry the same energy without hijacking input |

## What the pass confirms about our shape

The encouraging read: our cascade architecture is *the same family* as theirs — conditioned pools, most-specific-wins, layered greeting → exchange → close — and several systems we built this sprint (vibe probes #1187, ask ladder #1188, mood memory #1183, meeting rites #1185) are mechanics Skyrim simply doesn't have (it never probes, never remembers a slight, never lets the player ask a question back — its gaps were our whole #1177–#1189 batch).

The sharpest gap: **Skyrim treats talk-state as something that persists *between* conversations** (greeting timers, disposition, threshold swaps) while ours mostly persists *across* a campaign (bond, memory, mystery knowledge) but resets cleanly *within a day*. Talk fatigue (#1226) and the follower register (#1228) are both versions of "the villager should know where in the story we are."
