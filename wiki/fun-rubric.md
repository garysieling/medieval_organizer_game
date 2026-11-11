# Fun Rubric - Fun Inspector's Scorecard

What "fun" means for this game: a village busy enough to watch, legible
enough to read, and a player whose reach is earned and visibly growing.
Fun here isn't smoothness - the game is meant hard; it's that the
hardness always has a face.

## The Six Scores (1-5 each)

Score each per playtest; `FunScores` in
`game2-core/src/agents/fun_inspector.rs` mirrors these fields.

### 1. Legibility
Could the player tell *why* things happened? Blows arrived with heralds,
verbs said what they do and cost, failures still read on the map. A
consequence that can't be attributed to anything seen is a defect, not a
mystery.
- 5: every consequence traceable; the log reads like cause and effect
- 1: the game just does things to you

### 2. Presence
Did events visibly happen on the map - the crowd gathered, the
procession walked, the fire spread - or did the world change only in a
transcript line? (#1076's "reads on the map" rule.)
- 5: the big beats have bodies and places
- 1: the log scrolls; the map stands still

### 3. Earned Progress
Did the player's reach grow - new verbs, an earned meeting, deepening
bonds, a cause gaining capacity? Reach built through relationships and
world-state, not unlocked by time or interface privilege.
- 5: the session ends with reach the beginning didn't have
- 1: same verbs, same reach, whatever was pressed

### 4. Rhythm
Buildup → climax → breather, not a metronome and not a void. Beats
arrive; quiet stretches feel like valleys, not dead air.
- 5: tension gathers and pays off; rests feel earned
- 1: nothing happens, or everything happens at once

### 5. Attachment
Did the player care - names remembered, a death that lands, a favorite
whose fortunes matter? Emotional stimulation: stakes felt through
people.
- 5: specific villagers' outcomes mattered by name
- 1: the village is an ant farm

### 6. Stakes
Did decisions matter and carry a price - something to lose, choices that
close doors? Agency through the verbs the player actually had.
- 5: a decision's consequence is felt later
- 1: all paths wash out; nothing to lose

## Verdict Bands
- avg >= 4.0: Fun - keep it up
- avg >= 3.0: Decent - needs work
- avg >= 2.0: Meh - grind detected
- below 2.0: Not fun - major issues

## Signals of Fun
- "I knew that was coming and it still hurt" - heralds work
- The player watches the map for a face, not the numbers for a value
- A verb opens and the player knows what earned it
- Quiet days feel like breathers, not stalls
- "Tell me more about that" moments outnumber "I'd alt-tab here" ones

## Signals of Grind / Hollow
- Consequences arrive unattributable - the game's punishing blind
- Big beats live only in the log; the map never shows them
- Reach never changes - the same five verbs all session
- Even spacing: events on a schedule the player can set a watch to
- Waiting without anything worth watching

## Red Flags (auto-flag for review)
- Dead air: no tagged event for a stretch the `dead_air` KPI would flag
- Any `t{tick}:` log line without an `EvTag` - invisible to the scorer,
  likely invisible in spirit too
- A Major negative with no same-thread warning in the prior days
  (`punishment_warn` territory)
- The player's action set identical at session start and end
- Flat affect: the run's `valence_ratio` collapses to one side

## Notes for Playtesters
- Record boredom AND excitement with tick numbers
- For each score below 3, name the specific moment that dragged it down
- "Would you voluntarily play 10 more minutes?" is the sanity check
  under the scores, not one of them
