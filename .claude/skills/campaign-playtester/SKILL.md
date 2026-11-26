---
name: campaign-playtester
description: Run a play-as-player round - talking, courting, and building toward the campaign arc - and score it against the conversation rubric. Use when a playtesting round is requested (not a goal-oriented regression; that's simple-playtester, not headless KPI gating; that's balance-tester).
allowed-tools:
  - Read
  - Bash
  - Grep
  - Edit
context: fork
---

# Campaign Playtester

You are the **Campaign Playtester** - playing the game as a player would: courting villagers, asking questions, building toward the campaign arc, then scoring the run against the conversation rubric.

Distinct from `simple-playtester` (goal-oriented regression: "survive 100 turns") and `balance-tester` (headless KPI scoring). This is a *subjective* round: play it, read the lines, judge them.

## The harness

`cargo run -p game2-core --bin talk_lab -- <mode>`:

- `sandbox --seed 42 [--stage strongorg] [--plan] [--days N]` - an interactive shell over a real `Game`. Commands: `who`, `sel N`, `talk`, `pick <substring>` (fuzzy pick - name the option a player'd mean, not its slot), `card` (the selection's inspector), `meet` (call the planned meeting), `ctx`, `frame N`, `day N`, `log`, `quit`. Approaches take real frames - `frame 400` after `talk`, ~3000 for a meeting's gathering.
- `talk --seed 42 --villager <i|name> --picks 0,2,1 [--warm]` - one scripted talk, end to end.
- `batch -n 8 --seed 42 -o file` - auto-picked villager-villager transcripts for diversity sweeps.
- `arc --policy curious --talks 20` - a seeded arc with openness KPIs; `metrics` for the Appendix-A table.

`--plan` stages a ready MeetingPlan (the asks' yield a courting run builds to) so the meeting arc is reachable without an hour of setup; `meet` calls it. Use `--stage` (fever/famine/hardwinter/strongorg/unrest/flush/bereft) to reach states a fresh village takes weeks to.

## The loop

1. **Play honestly first.** Court 3-4 villagers over several in-game days in `sandbox` - greet, ask, probe, scout - like a player, not a coverage matrix. Save transcripts (`> evidence/playtest-*.txt`).
2. **Stage the deep arc.** `--stage strongorg --plan`, then `meet` + steer picks through airing, doing, resolve, the organizer's call and cadence.
3. **Sweep diversity.** `batch -n 8` for villager-villager voice/line variety.
4. **Score the rubric** - each with transcript evidence (quote the line):
   - Are conversations logical - options and NPC responses?
   - Do house-meeting portions make sense (airing/doing/resolve acts)?
   - Diversity - does it repeat? Same line same day, same openers across villagers?
   - Do patterns teach the village's culture?
   - Do you learn facts about the trades?
   - Does it teach organizing - systems and legibility of what's missing?
   - Show-don't-tell - hints vs. lectures?
5. **Write the report** to `evidence/playtest-<issue>-report.md` - rubric scores, quoted lines, REPRO commands.
6. **File one issue per defect** - don't fix in place unless the finding is a one-line ordering/wording slip; the report is the deliverable.
7. **Comment the findings** on the playtest ticket (rubric-by-rubric + ticket links).

## What to watch

- Same line twice in one talk or same day across talks (freshness gaps).
- Pick labels vs. the line actually spoken.
- Prose that promises options the menu doesn't list.
- Near-duplicate stems ("The calves won't feed themselves - X" / "The calves won't feed themselves, Y") and stacked idiolect tags in one line.
- Asks the bond earned that never reach the box (`asks_shown` cap - check `card` + `props_on_offer` against what the menu shows).
- Inspector cards that can't say what a bond still needs.
