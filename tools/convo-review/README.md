# Conversation review

A small local web app for reviewing the game's conversations one at a time:
rate them, tag the problems, rewrite or flag individual lines, and turn the
feedback into GitHub issues.

```bash
python3 tools/convo-review/server.py          # http://localhost:8765
```

Standard-library Python only. It builds and runs the generators with cargo
(set `CARGO=/path/to/cargo` if it isn't on PATH; under WSL it falls back to the
Windows `cargo.exe`), and files issues with the `gh` CLI.

## Sources

- **Village** (`talk_dump`, in the client crate for the art): a seeded village run forward N
  days. You get the chats villagers actually had (meetings and meals too, when they happen).
  Each speaker comes with the figure the map draws them as and their full sheet: trade, age,
  traits, temper, faith, kin, feuds, the wants and troubles they've told and the ones they
  still keep secret, and the ideology/status tags the talk engine reads. Lines don't carry
  their dialogue tree, so tickets find each line in the source instead.
- **Generated** (`dialogue_gen --json`): synthetic characters (ideology, traits, status,
  backstory) paired up under random relations. Every line is tagged with its act and tree.

Same seed and settings give the same conversations, so feedback is keyed by
conversation id and carries across batches.

## Reviewing

| Key | |
|---|---|
| `←` `→` / `k` `j` | previous / next |
| `1`-`5` | rating |
| `n` `o` `a` `r` `t` `e` `f` `d` `g` | toggle a problem tag (hover a tag for its meaning) |
| click a line, or `↑` `↓` + `Enter` | rewrite it and/or say what's wrong (`Ctrl+Enter` saves) |
| `/` | notes |
| `c` | show / hide the character cards |
| `s` | fine as it is: mark reviewed, go to the next unreviewed |
| `u` | next unreviewed |
| `[` `]` | previous / next batch |
| `b` | new batch (in the dialog: `1`-`4` picks 10/25/50/100, `Enter` generates, `Esc` cancels) |
| `v` / `i` | Review / Tickets tab |
| `Ctrl+Enter` | on the Tickets tab: Create (press twice) |

Every button shows its key.

Everything saves as you go to `data/` (git-ignored; `--data DIR` puts it elsewhere).

## Tickets

The **Tickets** tab groups feedback that hasn't been ticketed yet:

- one issue per problem tag, with each conversation's transcript, notes and flagged lines
- one issue per dialogue tree (or source block) that has line rewrites: the old line, the
  suggested one, why, and `file:line` where it's written
- one issue for leftover notes on conversations rated 3 or lower

Edit titles and bodies, untick any you don't want, then **Create selected on GitHub**
(click twice to confirm). Issues get the `enhancement` and `conversation-review` labels, and
the conversations are marked as ticketed so they don't get filed twice. Conversations tagged
`good` or rated 5 aren't ticketed.
