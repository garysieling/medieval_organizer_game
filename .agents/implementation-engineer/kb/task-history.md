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

### Task: Conversation Framework & Dialogue Generator (PR)
**Date:** 2026-11-01
**Type:** feature
**Files Changed:**
- game2-core/src/game/conversation.rs (new: Facts/Criterion specificity cascade, Act enum, Reply with trait bias, Exchange, converse, converse_with for player choice, run_meeting for house meetings)
- game2-core/src/game/dialogue_trees.rs (new: village_library with generic smalltalk tail + sickness/grief/clergy/rivalry overrides, meeting pools, backstory generator)
- game2-core/src/game/mod.rs (pub mod conversation, dialogue_trees)
- game2-core/src/bin/conversation_demo.rs (new: worked examples of 1-1 specificity cascade, rivalry, clergy, interactive player turn, and house meeting)
- game2-core/src/bin/dialogue_gen.rs (new: benchmark generator producing N characters x M conversations = 50 total + AI scoring rubric)
- wiki/literature-reviews/dialogue-systems.md (new: literature survey on Ruskin rule databases, Facade beats/acts, Prom Week CiF social considerations, Versu quip exclusivity)
- wiki/dialogue-scoring-rubric.md (new: 100-point rubric for AI grading)
- evidence/dialogue-gen-sample.md (new: 10 characters x 5 conversations sample benchmark)
- AGENTS.md (new: project instructions and Windows-gnu dlltool toolchain guidance)

**Outcome:** Success - all 695 library tests passing, demo and generator binaries verified
**Notes:** Implemented Ruskin's specificity cascade (most criteria wins, generic tail is the always-on fallback). Integrated CiF-style trait weighting so traits bias reply distribution rather than hard-gating. Added Versu-inspired quip exclusivity so used lines aren't repeated and dialogue gracefully exhausts into small talk or farewell. Added concept-threaded house meeting engine allowing dynamic addressing of organizer questions or prior statements.
**Patterns Used:** Ruskin criteria-based rule database; Façade discourse acts; SplitMix64 seeded RNG without external deps; Builder/declarative static const trees.

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

### Task: Wide/Shallow Dialogue Expansion (1000+ Responses)
**Date:** 2026-11-01
**Type:** feature
**Files Changed:**
- game2-core/src/game/dialogue_trees.rs (expanded to 30 trees across all vocations, circumstances, and social relations + 10 meeting concern pools, totaling 1,330 authentic responses)
- game2-core/src/game/conversation.rs (added test_dialogue_expansion_exceeds_1000_responses_and_is_clean verifying 1000+ responses and safety filtering)
- evidence/dialogue-gen-sample.md (regenerated 50-conversation benchmark with wide variety of vocation and relationship trees)
- scripts/generate_all.py, scripts/emit_rust.py, scripts/trees_data.py, scripts/more_trees_data.py, scripts/trade_data.py, scripts/vocations_data.py, scripts/meeting_data.py (generator toolchain using Ollama gemma3:4b with strict safety filter)

**Outcome:** Success - 1,330 responses generated, 0 sensitive words, all 699 library tests pass
**Notes:** Built wide and shallow dialogue option trees covering 14 trades (smith, miller, baker, weaver, carpenter, mason, shepherd, brewer, beekeeper, herb-wife, merchant, lord, fisher, charcoal burner), life circumstances (fever, care, mourning, condolence, hunger, fuel, revelry, elder memories, newborn), relationships (rivalry, kin, friendship, road strangers), and 10 meeting concern pools. Enforced strict sensitivity check disallowing modern or archaic slurs.
**Patterns Used:** Domain-specific exchange structures, topological static declarations, Ollama gemma3:4b prompt augmentation with safety filter.

---

### Task: Castle walls, towers and keeps take their grand dress up a size
**Date:** 2026-11-07
**Type:** feature
**Files Changed:**
- game2-client/src/draw.rs (draw_standing_h/draw_castle_h -> _wh width+height variants; GRAND_WALL_K 1.18->1.9, GRAND_TOWER_K 1.45->2.2 + GRAND_TOWER_WK 1.5, new KEEP_WK 1.8 / KEEP_K 2.6)
- game2-client/src/view.rs (Piece::Castle carries (wk, hk); keep gets KEEP_*; corner towers get per-tile loftier lift via noise(x,y,37)<0.35 -> x1.3)
- evidence/castle-grand-{before,after}.bmp

**Outcome:** Success - 85 client tests pass, frames dumped for review
**Notes:** Houses went to BUILDING_SCALE=2 (draw_on_tile) but wall pieces still drew at 1x, so castle curtains read as farm fences. Walls can only stretch vertically - a wider run-piece would swallow its neighbours in the painter's-order join - so curtains/gatehouses got hk~1.9 at wk=1.0; lone pieces (corner towers, keep) take both dimensions. Village wall stays at (1,1): the fence look stays by design. Pack-art fallback path got the same treatment via draw_castle_wh.
**Patterns Used:** existing grand_walls marker + per-piece scale constants; deterministic per-tile noise() for organic variety without sim state

---

### Task: Correct the building ordinal in the member log (issue #732)
**Date:** 2026-11-07
**Type:** bug fix
**Files Changed:** `game2-core/src/session/verbs.rs`, `game2-core/src/session/tests/member_line.rs`
**Outcome:** Focused member-line and ordinal suffix tests pass.
**Notes:** Apply the teen exception before checking the final digit; remove parenthetical wording from the member-facing line.

---

### Task: Show live crop plots as worked ground (issue #733)
**Date:** 2026-11-07
**Type:** bug fix
**Files Changed:** `game2-core/src/session/country.rs`, `game2-core/src/session/view_model.rs`, `game2-core/src/session/tests/country.rs`
**Outcome:** Crop-field tile test passes across five seeds; 99 client tests pass. The core suite has the five unrelated failures documented in #700; seed 1 frame captured and inspected (`evidence/issue-733-crop-fields.bmp`).
**Notes:** Reuse existing wheat/corn map tiles for the live fields, avoiding a UiModel or wire-shape change; keep crop plots clear of tree sprites. Add the missing courier `sign: None` initializer required to compile the test target.

---

### Task: Add visual evidence guidance to PR instructions (issue #725)
**Date:** 2026-11-07
**Type:** documentation
**Files Changed:** `wiki/agents/implementation-engineer.md`, `wiki/agents/pr-agent.md`
**Outcome:** Both contributor workflow and PR review checklist now request representative media for visible changes.
**Notes:** Prefer one focused screenshot or a short playthrough; explain when a change has no player-visible effect.

---

### Task: Add playthrough clips to the README (issue #724)
**Date:** 2026-11-07
**Type:** documentation and media
**Files Changed:** `README.md`, `evidence/videos/walkabout.gif`, `evidence/videos/village-day.gif`
**Outcome:** README now embeds two generated, deterministic village playthrough clips.
**Notes:** Use the built-in headless recorder with a fixed seed, a brief player script for one clip, and a separate winter seed for the other.

---

### Task: Funeral procession carrying the dead to the churchyard (issue #674)
**Date:** 2026-11-08
**Type:** feature
**Files Changed:** `game2-core/src/session/formation.rs`, `game2-core/src/session/subsystems/procession.rs` (new), `game2-core/src/session/subsystems/graveyard.rs`, `game2-core/src/session/subsystems/day.rs`, `game2-core/src/game/villager.rs`, `game2-core/src/session/model.rs`, `game2-core/src/session/view_model.rs`, `game2-client/src/{art,view}.rs`, `assets/rd/bits/lychgate*.png`, `docs/protocol.md`, tests
**Outcome:** 7 funeral + 5 graveyard core tests pass; 109 client tests pass; determinism/layering/protocol_doc pass. Full core suite: only the four pre-existing main failures (#831). Evidence: `evidence/funeral-march.bmp`, `evidence/funeral-aftermath.bmp`.
**Notes:** Reuse `Formation` for the column - add `FormationKind::Funeral` plus a `villager::Funeral` cargo component; the body stays at the death tile (`Funeral.carried` latches on when a bearer is within 3 tiles after the head passes route[0]) then rides the lead bearers' midpoint. `Formation.stuck` counts moving-frames only; `waited` counts the gather grace separately, and `leg0` (leg length at waypoint entry) scales the stall budget (`leg0/pace*2+30`) so long approach legs aren't mistaken for walls - the same instant-skip-after-grace bug existed for parades and is fixed for all kinds. Pick mourners/carriers by bond strength + `routine::mix` hashes, never `self.rng`, so the shared roll stream doesn't shift. Lychgate art via the RD pipeline (`memorial-lychgate-c606b4b8`), drawn as `Piece::Stand` at `UiModel.lychgate` (documented in protocol.md).
