---
name: tickets
description: File GitHub issues for requested features/bugs — grounded in the codebase, labeled, with testing steps
---

File GitHub issues for the game2 repo (`garysieling/game2`) from a list of feature/bug requests. Each ticket must be grounded in the actual code and include concrete testing steps — not generic advice.

## Workflow

1. **Verify tooling**: `gh auth status` and `git remote -v`. Repo is `garysieling/game2`.
2. **Dedupe**: `gh issue list --state all --limit 100` — don't file tickets for things already tracked.
3. **Ground each ticket in the code** before writing it. For every item, spend a few greps/reads finding the real subsystems, files, and existing scaffolding it touches (e.g. `game/fire.rs`, `session/journal.rs`, `sim/religion.rs`). Mention concrete paths/symbols in the ticket so whoever picks it up doesn't start from zero. If something may already exist, say so in the ticket (audit-first framing).
4. **Labels**: ensure these exist (`gh label create <name> --color <hex> --description "..."`, ignore "already exists" errors):
   - `enhancement` (#0e8a16) — default for features
   - `bug` (#d73a4a) — broken behavior
   - `ui` (#1d76db) — client/UI work
   - `art` (#fbca04) — needs new sprites/art
   - `audio` (#6f42c1) — sound/music
   Apply 1–2 per issue.
5. **Write bodies** in this shape:
   - `## Summary` — what the user wants, in their words tightened up.
   - `## Details` or `## Proposal` — where it touches (file paths, existing systems to build on, design questions/constraints like determinism or the art pipeline).
   - `## How to test` — REQUIRED. Concrete steps using the project's harnesses (below). Every ticket needs this section.
6. **Create them via a script**, not one-off commands: write a temp bash script (e.g. `.create_issues.sh`) with one block per issue:

   ```bash
   gh issue create --title "..." --label "enhancement,ui" --body "$(cat <<'EOF'
   ...markdown body...
   EOF
   )"
   ```

   Run it, confirm the URLs, then `rm` the script. Bodies contain backticks and quotes — the quoted `<<'EOF'` heredoc keeps them literal.
7. **Report back** a table mapping issue number → title, plus a short note on any audit findings or cross-references between tickets.

## Project testing cheat-sheet (put relevant commands in each ticket's How to test)

- Session/sim tests: `cargo test -p game2-core --lib [filter]` — tests live in `game2-core/src/session/tests/<subsystem>.rs`, start with `use crate::session::*;`, register new files in `tests/mod.rs`.
- Client draw tests: `cargo test -p game2-client --lib` — tests in `game2-client/src/tests/<area>.rs`; build games with `Game::new_seeded(MAP_SEED)`.
- Manual/visual: from repo root, `cargo run -p game2-client --bin game_ui -- --dump-frame N [--zoom Z --focus <kind>]`; env vars `GAME2_DUMP_WALK=dx,dy`, `GAME2_BENCH=N`, `GAME2_BENCH_PAN=1`, `GAME2_BENCH_ZOOM=1`, `GAME2_BENCH_JSON=path`.
- Multiplayer/protocol: `cargo test -p game2-core --test multiplayer`; `docs/protocol.md` JSON examples are tested — protocol changes must update the doc.
- Determinism (sim code): use `crate::engine::det::{HashMap, HashSet}` (build `::default()`), never std's; `tests/layering.rs` and `tests/determinism.rs` enforce it. New sim work wrapped in `crate::timed!("sim.<name>", ...)`.
- Art: pipeline in `assets/rd/README.md` — `art-prompts/prompts.md` → `scripts/rd_generate.py <id>` (`--dry-run` first, `--max-usd` guard) → `scripts/rd_slice.py` → `Art::load` with fallback → verify via `game_ui --dump-frame --zoom 2 --focus <kind>`.

See `AGENTS.md` for the full project conventions — read it before writing tickets if anything above seems stale.
