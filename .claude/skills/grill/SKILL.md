---
name: grill
description: Grill the user with questions about open GitHub issues, one ticket at a time, recording answers on the issue and tagging it `grilled` when done. Use when asked to "grill me about open tickets", "grill tickets", "pre-chew issues", or prep tickets before implementation.
allowed-tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
context: fork
model: sonnet
---

# Grill Tickets

One session = a grilling pass over open issues. For each ticket: read it,
census the code it touches, brief the user with real context, grill them
with real questions, record the answers on the issue, tag it `grilled`.
Tickets tagged `grilled` are the pool `/ticket-loop` picks from - don't
grill tickets that are already tagged, claimed, or closed.

Repo: `github.com/garysieling/game2`. Pass `-R garysieling/game2` to
every `gh` call - you may be standing in a worktree.

## Setup

```bash
gh label create grilled -R garysieling/game2 --color fbca04 \
  --description "Questions answered - ready to implement" --force
gh issue list -R garysieling/game2 --state open --limit 100
```

Work oldest-first (lowest issue number) unless the user names a ticket
or says otherwise. Skip issues that already have `grilled`,
`in-progress`, or `agent-ready`, an assignee, or an open PR referencing
them. If the user says "skip #N" or "pass on this one", move on without
tagging.

## Per ticket

1. **Read** the issue and every comment:
   `gh issue view <N> -R garysieling/game2 --comments`
2. **Census the code** it touches - same drill as ticket-loop Stage 1:
   grep/read the files and subsystems the ticket names; check whether
   the capability already exists unwired (`docs/architecture.md` is
   law; `docs/bug-audit.md` and `docs/unwired-backlog.md` are the
   autopsy style).
3. **Brief, then grill** - the user should never answer a question
   cold. A grilling pass flits from ticket to ticket, so every ticket's
   briefing must re-ground them completely. Before the first
   `ask_user_question` call, post a briefing in chat covering:
   - Which ticket: `#N - <title>`, plus one or two sentences on what it
     wants in plain terms. No jargon, no "see the issue" - the user may
     not remember which ticket this is.
   - The impact: what changes for the player or the game if this lands
     (or what stays broken if it's a bug). One concrete sentence -
     "the TALK panel would show X", "marches would keep failing when Y" -
     so they know why the question matters before they're asked it.
   - What the code does today, with file paths - e.g. "`session/x.rs`
     already has `foo()` but nothing schedules it". This is what makes
     the question answerable.
   - The 1-3 real decisions the ticket leaves open.

   Then grill with `ask_user_question`, 1-4 questions per call. Grill
   means surfacing what the issue *doesn't* say:
   - Ambiguous acceptance criteria: what does "done" look like concretely?
   - Scope edges: which of several plausible wirings is intended?
   - Probe truths: may the player see the real number, or only a
     rumour/sign? (Say what "probe truth" means - data only tests can
     see.)
   - New art needed? Which `assets/rd` prompt / fallback?
   - Anything in the ticket's test steps you can't actually run - say so.

   Every question must stand alone: name the ticket in the question
   text and state the concrete fork ("Ticket #N: should X, or Y?").
   Option labels and descriptions carry the trade-offs - never bare
   "Option A / Option B". Tag each question's kind so the user knows
   what judgment they're lending:
   - `[design]` - what the game should do: mechanics, feel, scope,
     consequences. Most questions are these.
   - `[arch]` - how it should be built: where code lives, data shapes,
     wire changes, layering. Say which file/system the choice touches
     ("should this live in `session/verbs.rs` or a new subsystem?") so
     it doesn't read as a design question in disguise.
   If the user says "use your judgment", record each judgment call and
   proceed. Don't ask what you can verify yourself.

   Question quality (user feedback, #1225): a question the user can't
   parse is a failed question. Before asking, translate the mechanic
   into plain language - name the real-world analogue when one exists
   ("is this bribery?"), define any coined term in the question text
   itself, and spell out what each option concretely allows or forbids
   ("can you bribe a total stranger who never told you anything, or
   must they have shared at least one thing first?"). If a draft
   question leans on a gate/stage/flag name, rewrite it as the player
   experience it produces. When the user answers "Other" with
   "I don't understand", reframe and re-ask - don't record a guess.
4. **Record** - post one comment on the issue with the briefing summary,
   the Q&A and decisions, so a fresh session understands them without
   this chat:

   ```bash
   gh issue comment <N> -R garysieling/game2 --body-file grill.md
   gh issue edit <N> -R garysieling/game2 --add-label grilled
   ```

5. Next ticket. Stop when the user says so or the queue is empty - say
   how many were grilled and which remain.

## Judgment calls

Grilling is for forks only the user can price. Before spending a
question, spend your own judgment - the defaults below are the user's
recorded preferences (#1455):

- **Dependency over question** - if a fork can be resolved by adding a
  `Needs` dependency between tickets, prefer the dependency and record
  it on both issues instead of asking.
- **Auto-choose consistent** - pick the option consistent with the
  user's prior answers on earlier tickets; only ask when prior answers
  genuinely conflict or the choice changes gameplay/behavior they
  haven't priced yet.
- **Gameplay-impact wording** - options must state the behavior
  consequence explicitly, not just the mechanism ("the TALK panel
  gains a X row", not "wires X through Y").
- **Extend over add** - prefer extending an existing enum/concept to
  creating a new one, unless the spec demands a separate axis.

Every judgment call made this way is recorded on the issue like an
answer would be - a fresh session must see what was decided, not just
that it was.

## Rules

- One ticket at a time; never batch questions across tickets.
- Questions and briefings stand alone: someone reading the chat (or the
  recorded comment) cold must know which ticket, what it changes, what
  the code does today, and what the fork is - no "as discussed above"
  or bare jargon. `[design]` asks land the game decision; `[arch]`
  asks land the build decision.
- The comment is the deliverable: `/ticket-loop` reads it in Stage 1/2,
  so write answers a stranger could implement from.
- Don't edit the issue body; comments + label only.
- `grilled` means "questions answered", not "approved to build" - scope
  can still be revised in ticket-loop's plan stage.
