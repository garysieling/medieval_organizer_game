# Screenshot Policy (Mandatory)

Every sprint task must produce screenshot evidence. This is non-negotiable.

## How to Capture

```powershell
powershell -ExecutionPolicy Bypass -File scripts/screenshot.ps1 -Name "task-N-<thing>"
```

Saves to `screenshots/<name>-<timestamp>.png` automatically.

## Rules

- **Every task gets ≥1 screenshot** before being marked complete
- **In-progress screenshots** count — not just final state
- **Show the work**: terminal output, file contents, git log, running code
- **Name format**: `sprint-N-task-M-<what>` (e.g., `sprint-3-task-B1-economics`)
- **Commit screenshots** with the task commit — they're part of the deliverable
- **Sprint end**: one video or compiled screenshot doc covering all work

## Enforcement

Task acceptance criteria are not met without screenshot evidence. If a task has no screenshot, it's not done.
