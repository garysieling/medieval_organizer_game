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
