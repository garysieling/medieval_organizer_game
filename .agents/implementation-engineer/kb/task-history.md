# Task History

**Agent:** Implementation Engineer
**Last Updated:** 2026-10-25
**Version:** 0.1.0

## Summary
History of tasks completed by Implementation Engineer with outcomes and lessons learned.

## Entries

### Task: Demonstration Task (task_1)
**Date Completed:** 2026-10-25
**Assigned By:** Manager Bot
**Approach:** Used TaskQueue to add and track task
**What Worked:** Simple line-based format works well for file storage
**What Didn't Work:** Initial attempt with serde/JSON failed due to Windows Application Control
**Time Taken:** [TBD]
**Peer Review Feedback:** [TBD]
**Lessons:** Avoid external dependencies when possible; simple formats are better for prototype
**Outcome:** Success
**Confidence:** High
**Tags:** infrastructure, task-queue, prototype

### Task: Build ECS Component System (task_2)
**Status:** Pending
**Assigned By:** Manager Bot
**Notes:** Auto-assigned by Manager Bot prototype

### Task: Implement Entity Manager (task_3)
**Status:** Pending
**Assigned By:** Manager Bot
**Notes:** Auto-assigned by Manager Bot prototype

### Task: Write Integration Tests (task_4)
**Status:** Pending
**Assigned By:** Manager Bot
**Notes:** Auto-assigned by Manager Bot prototype
---
**Date:** 1792950811
**Agent:** implementation-engineer

Completed task_1: Demonstration task
---
**Date:** 1792951562
**Agent:** implementation-engineer

Completed task_1: Demonstration task
---
**Date:** 1792951563
**Agent:** implementation-engineer

Completed task task_11: Add a comment to main.rs
---
**Date:** 1792951653
**Agent:** implementation-engineer

Completed task_1: Demonstration task
---
**Date:** 1792951654
**Agent:** implementation-engineer

Completed task task_16: Add a comment to main.rs

### Task: The Oil-Boiler's Visit (ink-making event + map fire)
**Date:** 2026-10-31
**Type:** feature
**Files Changed:**
- wiki/literature-reviews/ink-oil-boiling.md, wiki/software-design/ink-boiler-event.md (research/design)
- game2-core/src/game/fire.rs (new: CA fire spread on TileMap)
- game2-core/src/game/tilemap.rs (build_at records constructed buildings)
- game2-core/src/game/livelihood.rs (print shops sell ink)
- game2-core/src/game/desires.rs (Smelly wording source-agnostic)
- game2-core/src/session.rs, session/model.rs (InkBoiler event, fire state, UiModel overlays)

**Outcome:** Success - 6 PRs merged (#187, #191/#195, #194, #199, #200, #203)
**Notes:** Periodic visitor on the circus/pedlar pattern camps a cauldron outside the south wall (historical zoning for flame-trades). Recipe phases ride the day tick: boil (fire risk + acrolein Smelly), crusts-bearer beat via Goal::TalkTo hook, slate-drip varnish line, lampblack -> ink stocked at the print shop (first local source of an import). Fire = Session.fire CA: burning/scorched tiles render via UiModel, buildings lose stock while alight, tree_cover zeroes, villagers flee.
**Patterns Used:** visitor spawn/advance/tick pattern; injectable roll for deterministic tests; build_at vs occupy distinction (buildings vs walls/stones)

---

### Task: Headless video recording (record_video)
**Date:** 2026-10-31
**Type:** feature
**Files Changed:**
- game2-core/src/session/record.rs (new: run_recording, RecordOpts, Format, parse_script, Sink trait)
- game2-core/src/bin/record_video.rs (new: CLI front-end)
- game2-core/src/engine/render.rs (Buffer::scaled nearest-neighbour downscale)
- game2-core/src/session.rs (pub mod record; walk_at non-finite/zero guard + regression test)
- game2-core/Cargo.toml (+gif 0.13), scripts/record-walkabout.txt, PLAY.md, .gitignore

**Outcome:** Success - PR #280 merged (squash d1f7df9)
**Notes:** `Game::update()` is one 60fps game frame; run it headless and capture every Nth `View::render` per `capture_every(speedup, fps)` - play-time compresses to watch-time. APNG comes free from the existing png dep (`set_animated` + per-frame `set_frame_delay` + one `write_image_data` per frame); gif crate quantizes RGBA in place via `Frame::from_rgba_speed`. Ffmpeg sink pipes raw RGBA to a child's stdin - same shape can point at rtmp for real streaming later. Script ops reuse `Command`; `look` is a View op via `camera_on` (tile -> camera offset, not raw pixels). Windows-gnu gotcha: getrandom's build needs dlltool; it's in rustup's self-contained dir, add to PATH. Zero-length `Walk` used to NaN the player's Pos into a ui_model sort panic - `walk_at` now rejects non-finite/zero steps.
**Patterns Used:** Sink trait over encoders (gif/apng/frames/ffmpeg); probe frame to size sinks pre-run; BTreeMap script schedule; mirror --dump-frame CLI conventions

---

### Task: Video sound + speed presets (PR #298)
**Date:** 2026-10-31
**Type:** feature
**Files Changed:**
- game2-core/src/session/record.rs (AudioEvent, synth_track, write_wav, speed_preset, AviSink, FfmpegSink audio mux, Sink::finish(audio))
- game2-core/src/bin/record_video.rs (--speed presets, --audio, format aliases)
- game2-core/Cargo.toml (+jpeg-encoder 0.6), scripts/get-ffmpeg.ps1, PLAY.md, .gitignore

**Outcome:** Success - PR #298 merged (squash), 8 record tests green
**Notes:** game.sfx drains per update() - cues live one frame, collect post-update. Cues map game-frame -> watch-time at f/(every*fps). AVI is hand-rolled RIFF: LIST sizes patched at finish (flush before seek!), idx1 offsets relative to the 'movi' fourcc (ffmpeg convention, +4 vs movi data start), audio strh uses scale=blockAlign rate=byteRate. ffmpeg can't take two piped inputs on Windows - audio goes via temp wav + a second mux pass (-c:v copy -c:a aac). ffmpeg binary via npm ffmpeg-static into .tools/ (no admin). Voice envelopes need attack+release taper or they click.
**Patterns Used:** placeholder+patch for RIFF headers; two-pass mux when a child can't see the future; named presets over raw params for situational use

---

### Task: RD set-styles in the art prompt generator
**Date:** 2026-11-01
**Type:** feature
**Files Changed:**
- scripts/generate_art_prompts.py (sheets/sprites group + more ui_panel + more inventory_items + smoke entry)
- art-prompts/prompts.{json,md}, art-prompts/smoke/prompts.{json,md} (regenerated)

**Outcome:** Success - PR pending
**Notes:** RD's three set-styles turn one $0.18 call into a whole matched batch: `rd_pro__spritesheet` (64-256px, grid of same-style assets/poses), `rd_pro__ui_panel` and `rd_pro__inventory_items` (both 256x256 ONLY - other sizes fail). `cost()` falls back to `PRICE[style.split("__")[0]]` so unlisted rd_pro__* styles price at $0.18 automatically. Stable ids hash group+slug+prompt+req only - changing `notes` does not break the done-state dedup.
**Patterns Used:** one call -> matched grid over many single-asset calls; tier sheets under their group's neighbors

---
