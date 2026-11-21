## Grill briefing + decisions (from grilling pass)

**Current state:** `option_rows` draws "1) text" and "x) Take your leave" (`game2-client/src/talk.rs:30,47`) — keys that don't exist. `read_input` maps 1-8 unconditionally to `Command::Select(n)` (`game_ui.rs:176`); `Command::TalkPick` and `ConvoClick::Decide` are mouse-only. The convo box is also where `Decide(i)` founding-call rows render — same "options you see" surface.

**Decisions:**

1. **Talk wins.** While a convo panel is open, digits 1-8 route to `TalkPick(Some(i))`; `Command::Select` villager-selection is suspended until the talk closes. The drawn numbers are honest.
2. **X and Esc both take leave.** X sends `TalkPick(None)` matching the drawn prefix; Esc also takes your leave while a talk is open — ordering note for #1063: Esc leaves the talk first (its cancel-layer), the pause menu only comes up when no talk is open.
3. **Decide rows too.** Digits drive whichever rows the convo box offers — `Reply(i)` or `Decide(i)` — one rule: the numbers you see are the keys that work.

**Implementation notes:**
- Client-side only: `read_input` in `game_ui.rs` needs the convo-open state (the same `cur_model.convo` the click path reads) to pick digit routing — no protocol/core change.
- Digit rows map 1-indexed display → 0-indexed command (`n-1`), and digits beyond the row count are swallowed (TalkPick already bounds-checks, but an out-of-range digit shouldn't fall through to Select either — swallow, don't misfire).
- Focus precedence stays: search-box focus types first, picker/intro consume their keys before the game loop.
- Esc-in-talk interacts with #1063's pause menu — the talk-leave check must run ahead of opening the menu.
- Tests: `cargo test -p game2-client --lib` — key→command mapping tests for digits-with-talk-open (TalkPick), digits-without (Select), X/Esc → TalkPick(None), decide rows → Decide(i).
