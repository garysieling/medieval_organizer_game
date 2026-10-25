# Decision Rubric for Autonomous Runs

This rubric governs what decisions Devin can make alone vs. what needs Gary's input. Goal: enable hour-long autonomous runs without compromising on decisions that actually matter.

## Decision Tiers

### Tier 1 — Just Do It (no approval needed)
Basic engineering and game design decisions with clear right answers:

- Code style, naming, file organization
- Which standard pattern to use (e.g., line format vs JSON, HashMap vs Vec)
- Implementation details within an approved design spec
- Test structure and coverage choices
- Doc comment / README wording
- Refactoring within a module
- Adding files/modules that follow existing conventions
- Fixing typos, warnings, obvious bugs
- Choosing between std-lib alternatives when no requirement exists
- Simple data structures for new systems
- Sprint task ordering when priorities aren't specified
- Small additions to agent specs/KBs following the established template
- Git commit messages, branching within the project

### Tier 2 — Decide and Flag (reversible, report later)
Decisions that are reversible or low-cost to change. I make the call, record it in `wiki/pending-decisions.md`, and flag it for your next review:

- Choosing a specific algorithm when several fit (e.g., midpoint displacement vs Perlin for terrain)
- Adopting a crate/dependency (reversible: can remove later)
- Game balance numbers (resource costs, hunger rates, tick speeds)
- KB file layout changes within an agent's KB
- New fields on data structures (additive, non-breaking)
- Which model to use for a given agent loop (flint vs gemma3, etc.)
- Screenshot/doc format choices
- Renaming a module/file (git tracks history)
- Task priority changes within an approved sprint
- Adding a new backlog item

**Flag format:** append to `wiki/pending-decisions.md` — `DECISION | date | what I chose | alternatives | why | reversal cost`

### Tier 3 — Must Ask First (needs Gary's approval)
Irreversible, expensive, or direction-setting decisions:

- Removing/deleting files, data, or committed history
- Changing core architecture (ECS vs OOP, engine choice, language)
- Anything touching security, credentials, secrets
- Decisions that contradict an approved spec or user-stated preference
- Spending money/credits (paid API calls beyond local Ollama)
- Changing the project vision, theme, or core conflict (community organizers vs fascists allegory)
- Adding a major system not on the backlog
- Publishing, pushing to remotes, or sharing anything externally
- Changing git history (force push, rebase of shared commits)
- Anything that would take >1 day to undo if wrong

## Mid-Sprint Blockers

When a task blocks mid-run:

1. **Log the blocker** in `wiki/blockers.md` — `task | blocker | what I tried | what would unblock`
2. **Don't stall** — pull the next ready task from the sprint list or backlog
3. **Revisit** the blocked task on the next run or when context allows
4. If a blocker is a Tier 3 decision, park it and note it as pending-decision instead

## Run Length Target

- **Target: ~1 hour of work per run**
- Plan sprint tasks assuming multiple parallel workstreams so a block on one doesn't end the run
- Keep a "quick wins" pile (KB updates, doc tweaks, small refactors) for filler when a big task is mid-flight

## Pending Decisions Log

Track Tier 2 decisions for your review in `wiki/pending-decisions.md`. Each entry gets: what was decided, alternatives considered, rationale, and reversal cost. You can review and override at any time — cheap to change.

## Examples

| Situation | Tier | Action |
|-----------|------|--------|
| Line-format vs JSON for task queue | 1 | Chose line-format (no deps) |
| Which model for agent loops | 2 | Choose gemma3:4b for speed, flag in pending-decisions |
| Hunger tick rate (1.0 vs 0.5 per turn) | 2 | Pick 1.0, flag for playtest tuning |
| ECS vs OOP for game core | 3 | Already decided (ECS) — would need approval to revisit |
| Delete old task file format | 3 | Ask first (data loss potential) |
| Add Machinations to backlog | 2 | Added, flagged in pending-decisions |
| Push to GitHub remote | 3 | Always ask |

## Status
Active — governs all autonomous runs from Sprint 2 onward.
