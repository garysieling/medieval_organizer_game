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
   cold. Before the first `ask_user_question` call, post a briefing in
   chat covering:
   - Which ticket: `#N - <title>`, plus one or two sentences on what it
     wants in plain terms. No jargon, no "see the issue" - the user may
     not remember which ticket this is.
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
   "Option A / Option B". If the user says "use your judgment", record
   each judgment call and proceed. Don't ask what you can verify
   yourself.
4. **Record** - post one comment on the issue with the briefing summary,
   the Q&A and decisions, so a fresh session understands them without
   this chat:

   ```bash
   gh issue comment <N> -R garysieling/game2 --body-file grill.md
   gh issue edit <N> -R garysieling/game2 --add-label grilled
   ```

5. Next ticket. Stop when the user says so or the queue is empty - say
   how many were grilled and which remain.

## Rules

- One ticket at a time; never batch questions across tickets.
- Questions and briefings stand alone: someone reading the chat (or the
  recorded comment) cold must know which ticket, what the code does
  today, and what the fork is - no "as discussed above" or bare jargon.
- The comment is the deliverable: `/ticket-loop` reads it in Stage 1/2,
  so write answers a stranger could implement from.
- Don't edit the issue body; comments + label only.
- `grilled` means "questions answered", not "approved to build" - scope
  can still be revised in ticket-loop's plan stage.
