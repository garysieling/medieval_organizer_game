# AGENTS.md - Game2 Project Instructions & Learned Information

## Build & Test Instructions

### Windows GNU Toolchain Note
On Windows with `x86_64-pc-windows-gnu`, crates with Windows import library generation (like `getrandom` under `rand`) require `dlltool.exe`. If missing from standard PATH, it is located in rustup's self-contained tool directory:
```bash
PATH="$PATH:$USERPROFILE/.rustup/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained"
```

### Running Tests
- Test library modules:
  ```bash
  cargo test -p game2-core --lib game::conversation
  cargo test -p game2-core --lib
  ```
- Build binaries:
  ```bash
  cargo check --bin conversation_demo --bin dialogue_gen
  cargo run --bin conversation_demo
  cargo run --bin dialogue_gen -- -n 10 -m 5 -o evidence/dialogue-gen-sample.md
  ```

## Conversation System Architecture
- **Rules & Specificity Cascade (`game/conversation.rs`)**:
  - Declarative criteria (`Has`, `Lacks`) on world facts (`self:*`, `other:*`, `pair:*`, `any:*`).
  - Ruskin's scoring: most criteria wins.
  - Cascades degrade gracefully from high-specificity trees (sickness, grief, rivalry, clergy) into the generic `village_smalltalk` tail.
- **Social Considerations & Weighting**:
  - Trait multipliers on reply weights bias choices probabilistically (Prom Week / Comme il Faut model) without hard-gating.
- **Discourse Acts & Quip Exclusivity**:
  - `Act` enums (`Greet`, `Remark`, `Question`, `Answer`, `Agree`, `PushBack`, `Farewell`).
  - Lines are spent upon utterance to avoid in-conversation repetition and allow natural material exhaustion.
- **House Meetings**:
  - Looser model where attendees bid on speaking based on concept overlap with the organizer's open question, the previous speaker, or prior statements.
- **In-game wiring (`session/dialogue.rs`)**:
  - Villagers meeting in passing run `converse()`; the transcript goes into `Game::talk_log` (no map bubbles). Kids only talk to kids, on the `children_at_play` tree.
  - The player's Talk opens a `PlayerTalk` that is replayed from its seed plus the picks so far (`converse_with`), so it can wait across frames for a reply (`Command::TalkPick`).
  - House meetings use `run_meeting` lines for attendees who don't reveal an issue, and are recorded as they're said.
  - The TALK panel (under the minimap) shows the picked villager's conversations, or the player's own. The event LOG holds only events.

## Art Pipeline (Retro Diffusion)
- Full recipe: `assets/rd/README.md`. Pick from `art-prompts/prompts.md` -> `scripts/rd_generate.py <id>` (use `--dry-run` first; `--max-usd` guard defaults to $2) -> `scripts/rd_slice.py` -> wire into `Art::load` with a fallback -> verify with `game_ui --dump-frame N --zoom 2 --focus <kind>`.
- API token: `$RETRODIFFUSION_API_KEY` or `~/.config/retrodiffusion/token`. Never commit it.
- `art-prompts/generated.json` is tracked: it's the ledger of paid generations; raw outputs live in `art-prompts/raw/<id>/`.
- Sync API calls reject an `Idempotency-Key` header (`idempotency_async_required`).
- Pack art is drawn 2x; RD sprites are box-resampled to a target height (`name@H`) and drawn at 1x.
- RD sprite pixel size is fixed (~25px objects, ~35px walk figures): request 256 sheets and resample; a smaller canvas only clips. `four_angle_walking` needs 48px (32 fails).
- Ground-block prompts use `ISO_GROUND`, never `ISO` (it draws a house on the block). Tile/building swaps need the whole set (roads/rivers carry their own grass).
- Animation dumps: `GAME2_DUMP_WALK=dx,dy game_ui --dump-frame N --focus player` walks the player 8 frames before the shot.
