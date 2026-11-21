## Ticket #1071 - Unclear what the 'The school' button does
Agent: devin | SWE-2 High    Started: 2026-11-11

### Claim
- Label: grilled -> in-progress

### Plan (per the grill comment)
- **Offer gating** (`verbs.rs::update_ctx_actions_at`): push `CtxAct::School` only when (a) the org treasury covers the press - 30g before the schoolhouse stands, 10g after (new `SCHOOL_FOUND_COST`/`SCHOOL_TEACHER_COST` consts shared with `school_at`), and (b) the player is near the site: within `MEETING_REACH` of the market cross before it exists, of the schoolhouse after. Hidden, not greyed, per the grill.
- **The reason line**: `ctx_blocked` both greys AND disables (`ctx_act_at` re-checks), so the description can't ride it. New `ctx_hint: Vec<Option<String>>` on `UiModel` (`#[serde(default)]`, absent from older servers - the established wire-compat pattern), populated from `ctx_hint_at(seat, act)` - School gets "the treasury raises the schoolhouse here - 30 gold" / "send for a teacher for the season - 10 gold" per state. Client shows the hint on hover in the same under-band line, without greying.
- `docs/protocol.md` gains a `ctx_hint` line; `seat.rs:121` doc updated. `school_at`'s failure messages stay for the between-frames race.

### Test plan
- `cargo test -p game2-core --lib school` - new tests in `session/tests/school.rs`: button absent when treasury < cost, absent when player far from the site, present + `ctx_hint` text when near and funded, press still builds + hires.
- `cargo test -p game2-core --lib` - full suite (protocol_doc covers the JSON examples).
- `cargo test -p game2-client --lib` - draw test in `src/tests/ctx.rs`: hovered School button renders its hint and isn't greyed.

### Branch / worktree
- branch: `fix/1071-school-action`
- worktree: F:/projects/game2-1071-school
