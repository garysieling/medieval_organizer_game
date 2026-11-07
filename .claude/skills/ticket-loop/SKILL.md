---
name: ticket-loop
description: Work one GitHub issue end to end - claim an agent-ready ticket, plan how to test it, grill the user with questions, branch + worktree, implement using the repo's existing verification tools, survive the AI PR review, merge, then take the next ticket. Use when asked to "take a ticket", "work an issue", "pick up issue #N", "do the next ticket", or run the ticket loop.
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

# Ticket Loop

One ticket at a time, from claim to merged PR. **Never start a second
ticket while one is open.** The loop only moves forward through the stages
below, in order.

Repo: `github.com/garysieling/game2`. Pass `-R garysieling/game2` to every
`gh` call - you may be standing in a worktree.

## Stage 0 - Find a safe ticket

Only issues labeled **`agent-ready`** are safe to take. That label means a
human filled in the ticket form: acceptance criteria, a verify plan, and
the UI surface. Everything else is not yours.

```bash
gh issue list -R garysieling/game2 --label agent-ready --state open
gh pr list  -R garysieling/game2 --state open --search "<N>"   # already claimed?
gh issue view <N> -R garysieling/game2 --comments
```

- Skip issues with an open PR referencing them, an assignee, or the
  `in-progress` label.
- If no `agent-ready` issues exist, say so and stop. Do not promote your
  own issue to `agent-ready`.
- Claim it: `gh issue edit <N> -R garysieling/game2 --add-label in-progress`
  and comment `Taking this - <agent name> (<model>)`.

## Stage 1 - Pull the issue apart

Read the issue and every comment. Then census the code it touches before
planning - the repo's recurring failure is work that lands *unwired*
(see `docs/bug-audit.md` and `docs/unwired-backlog.md` for the autopsy
style):

- Does the capability already exist in `game/`, `sim/`, `session/` but
  never get called, scheduled, or projected? Wiring beats rebuilding.
- Which layer does the change belong to? `docs/architecture.md` is
  law: sim stays deterministic and draws nothing; clients read only
  `UiModel`/`AgentObs`; probe truth stays in tests.
- If it's player-facing, trace the whole path now (Stage 5 checklist) so
  the plan covers every hop, not just the sim.

## Stage 2 - Plan + test plan (fill in the form)

Write the plan into the Ticket Form below. The **test plan must name real
commands from Stage 5** - "it works" is not a test plan. Post the plan +
test plan as the first comment on the issue so the user can veto before
you build:

```bash
gh issue comment <N> -R garysieling/game2 --body-file plan.md
```

## Stage 3 - Grill the user

Ask real questions before writing code - use the platform's question UI
(`ask_user_question` / AskUserQuestion), 1-4 focused questions per call.
Grill means surfacing what the issue *doesn't* say:

- Ambiguous acceptance criteria: what does "done" look like concretely?
- Scope edges: which of several plausible wirings is intended?
- Probe truths: may the player see the real number, or only a rumour/sign?
- New art needed? Which `assets/rd` prompt / fallback?
- Anything in the issue's verify steps you can't actually run - say so.

If the user says "use your judgment", record each judgment call in the
form and proceed. Don't ask what you can verify yourself.

## Stage 4 - Branch + worktree

One worktree per ticket, sibling of the main checkout (the repo already
has ~40 of these). Find the main checkout from wherever you are - never
hardcode a path:

```bash
MAIN=$(git worktree list --porcelain | sed -n '1s/^worktree //p')
WT="$(dirname "$MAIN")/game2-<slug>"
git -C "$MAIN" fetch origin main
git -C "$MAIN" worktree add "$WT" -b <area>/<N>-<slug> origin/main
cd "$WT"
```

- Record `$MAIN` and `$WT` in the Ticket Form - Stage 8 needs them.
- `git -C` does not change your shell's directory; the `cd "$WT"` is
  what puts you in the worktree.

- Branch prefix matches repo convention: `ui/`, `fb/`, `feat/`, `fix/`,
  `art/`, `docs/` - pick the area, keep the issue number: `ui/675-research-meetings`.
- Do ALL work inside the worktree. The main checkout belongs to whoever
  is standing in it.
- Cargo builds per-worktree are slow; reuse a target dir if the machine
  already does (`CARGO_TARGET_DIR=../game2/target-<you>` is the existing
  pattern - check what neighboring worktrees do first).

## Stage 5 - Implement + verify with the tools we already have

This is the section that keeps tickets from landing half-wired. Use the
repo's existing verification - do not invent new harnesses.

### Tests (run from the worktree root)

```bash
cargo test -p game2-core --lib <filter>     # iterate: session unit tests
cargo test -p game2-core --lib              # all of them, incl. layering + determinism guards
cargo test -p game2-client --lib            # draw tests - REQUIRED if anything is drawn
cargo test -p game2-core                    # full: multiplayer sockets, protocol_doc, mortality_chain
cargo test -p game2-core --test multiplayer # only when protocol/server loop changed
```

- New session tests go in `game2-core/src/session/tests/<subsystem>.rs`
  (see AGENTS.md - never back in `session.rs`).
- Anything the player sees gets a draw test in
  `game2-client/src/tests/<area>.rs`; those tests see only `session`'s
  public API and build games with `Game::new_seeded(MAP_SEED)`.
- Wire-protocol change -> update `docs/protocol.md`; `tests/protocol_doc.rs`
  tests its JSON examples and will fail otherwise.
- Windows GNU toolchain: if `getrandom`/`rand` fails needing `dlltool`,
  prepend the self-contained dir to PATH (exact line in AGENTS.md).

### Visual verification (the part agents skip - don't skip it)

```bash
cargo run -p game2-client --bin game_ui -- --dump-frame <tick> --zoom 2 --focus <kind> --out evidence/<name>.bmp
```

- Run from the worktree root (art loads from `assets/`). Then **open the
  BMP and look at it** - the read tool renders images. A dump you didn't
  eyeball verifies nothing.
- Useful flags: `--cam x,y`, `--seed N`, `--weather <name>`,
  `--focus <kind>` (villager/building kind), `GAME2_DUMP_WALK=dx,dy`
  env to walk the player 8 frames before the shot.
- Perf-sensitive paths: `GAME2_BENCH=<tick> cargo run ... --dump-frame <tick>`
  prints update/model/render ms and the slow-section table; new sim work
  gets a `crate::timed!("sim.<name>", ...)` scope.
- Dialogue work: `cargo run --bin conversation_demo`,
  `cargo run --bin dialogue_gen -- -n 10 -m 5`.

### The UI-wiring checklist (player-facing tickets)

"Built but unreachable" is this repo's signature bug. For every feature
the player should see or use, verify ALL hops - a miss at any hop is an
unwired feature:

1. **Sim produces it** - the system is actually scheduled (an unscheduled
   `*_tick` is the classic W-0xx bug), the `Game` field exists and updates.
2. **Projected** - `session/view_model.rs` puts it on `UiModel` (or
   `AgentObs` for bots). It respects the public-view rules: no probe truth
   leaks; secrets show as rumour/sign/guess.
3. **Drawn** - `game2-client` renders it (View/hud/panel/inspector) with a
   draw test proving it appears.
4. **Reachable** - the player can trigger it: a `Command`/`CtxAct` variant
   produced by real input, or a clickable/HUD element. A new `CtxAct`
   needs all of: offered in `verbs.rs` (`ctx_actions.push` under the
   right conditions), labelled in `ctx_button` (`session.rs`), an arm in
   the `ctx_act_at` match, and a `ctx_blocked` state. A new `Command`
   needs a producer in `game_ui.rs::read_input`/click handling plus a
   `command_at` arm.
5. **Agent-reachable** - if bots should use it, `AgentCmd`/`AgentObs`
   carry it too (bots can't click).
6. **Docs** - `PLAY.md` controls table if there's a new key/button;
   `docs/protocol.md` if the wire changed.
7. **Census** - grep your new `pub` items for callers; an uncalled `pub fn`
   on a player-facing path is a red flag, not a library seam. If it IS an
   intentional seam, say so in the PR.

## Stage 6 - PR

```bash
git add -A && git commit   # match repo style: short imperative subject
git push -u origin <branch>
gh pr create -R garysieling/game2 --title "..." --body-file pr.md
```

The PR body carries the completed Ticket Form plus evidence: test
commands run and pass, dump-frame BMP paths, bench numbers if relevant.
End the body with `Closes #<N>`.

## Stage 7 - Survive the AI review

Devin Review (`devin-ai-integration[bot]`) auto-reviews every PR: a
review summary plus inline comments on `pulls/<PR>/comments`, with
`✅ Resolved` replies as findings get fixed.

```bash
gh api repos/garysieling/game2/pulls/<PR>/reviews    --jq '.[].body[0:200]'
gh api repos/garysieling/game2/pulls/<PR>/comments   --jq '.[] | {path, body: .body[0:160]}'
gh pr checks <PR> -R garysieling/game2               # if CI exists
```

Loop until clean:

1. Wait for the review round (typically a few minutes after push; poll
   every ~2 min, give up after ~20 and ask the user).
2. Fix each unresolved finding, or reply with a justification if it's
   wrong. Push.
3. The next round re-reviews. Repeat.
4. **Done = a round with no new unresolved findings** and checks green.

## Stage 8 - Merge, then next ticket

Shell variables don't survive between agent shells - restore both from
the Ticket Form before running this (or re-derive `MAIN` the same way):

```bash
MAIN=<main checkout abs path from the Ticket Form>
WT=<worktree abs path from the Ticket Form>
cd "$MAIN"        # the main checkout recorded in Stage 4
gh pr merge <PR> -R garysieling/game2 --merge        # repo convention: merge commits
git fetch origin --prune
git worktree remove "$WT"
git branch -d <branch>          # -d, only if merged
git push origin --delete <branch>  # if the remote branch remains
gh issue edit <N> -R garysieling/game2 --remove-label in-progress
```

Merge before cleanup is fully done? The remote branch may already be
deleted by the merge - check `git ls-remote` before pushing a delete.
Only after the issue is closed and the worktree gone do you go back to
Stage 0 for the next `agent-ready` ticket.

---

## Ticket Form

Fill this in as you go; it becomes the PR body. Keep it honest - an
empty section means the stage didn't happen.

```markdown
## Ticket #___ - <title>
Agent: devin | claude    Started: <date>    PR: #___

### Claim
- Label: agent-ready -> in-progress at <date>

### Plan
- <what changes, in which layer, which files>

### Test plan (commands actually run)
- cargo test ...: <pass/fail summary>
- dump-frame: evidence/<name>.bmp at tick <N> - <what it shows>

### Questions asked / answers
- Q: ... A: ... (or "user deferred to judgment" + what was decided)

### UI wiring (player-facing only)
- [ ] sim produces it (system scheduled)
- [ ] projected on UiModel/AgentObs
- [ ] drawn + draw test added
- [ ] reachable via Command/CtxAct/input
- [ ] agent-reachable if bots should use it
- [ ] PLAY.md / protocol.md updated if needed

### Branch / worktree
- branch: <area>/<N>-<slug>
- main checkout: <abs path>    worktree: <abs path>

### AI review rounds
- round 1 (<date>): <N> findings - <fixed/replied each>
- final round: clean

### Merge
- merged <sha> <date>; worktree removed; issue closed
```
