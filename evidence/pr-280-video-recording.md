# PR #280: A moving picture of the village: headless video recording
**Link:** https://github.com/garysieling/game2/pull/280
**Status:** merged 2026-10-31 (squash d1f7df9), branch deleted

## What changed
- `record_video` bin: plays the sim headless at CPU speed, writes the
  windowed render to video. `--play-time/--speedup/--fps/--scale/--format/
  --script/--ffmpeg-path/--game-speed/--zoom/--cam/--seed/--out`
- Sinks: GIF (new `gif` dep), APNG (existing `png` dep), PNG frames,
  raw-RGBA pipe to an ffmpeg binary (mp4/webm; rtmp later)
- `Buffer::scaled` downscale; script ops = `Command`s + `look` camera op
- Latent fix: zero/NaN `Walk` no longer NaNs the player's Pos

## Quality gates
- cargo build clean (warnings denied), 516 lib tests pass
- New tests: capture_every math, script parse/sort, gif+apng end-to-end,
  zero-walk regression
- Devin Review check: pass; independent diff review: 4 real findings fixed
  (camera_on for `look`, act bounds, walk double-step, frame-0/late ops)

## Demo artifacts (gitignored)
`evidence/videos/` in F:\projects\game2: play-10s.gif (scripted walkabout),
play-1m.gif, play-5m.gif, play-10m.gif, play-30m.gif (a full year, 60x)
