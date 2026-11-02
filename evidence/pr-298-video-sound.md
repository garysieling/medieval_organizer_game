# PR #298: Give the picture a voice (soundtracks + speed presets)
**Link:** https://github.com/garysieling/game2/pull/298
**Status:** merged 2026-10-31 (squash), branch deleted

## What changed
- `--speed realtime|1x|2x|5x|10x|60x` presets (--speedup for arbitrary)
- `--format avi`: MJPEG+PCM AVI written in pure Rust (new jpeg-encoder dep)
- `--format mp4 --audio`: raw frames to ffmpeg + wav temp + aac mux pass
- sfx cues (rain/anvil/festival/wonder) synthesised to stereo PCM on the
  watch timeline, equal-power pan by map x
- scripts/get-ffmpeg.ps1: npm ffmpeg-static into .tools/ (gitignored)

## Quality gates
- 8 record tests pass (presets, wav, synth placement, AVI RIFF layout)
- ffmpeg decode-validated: avi reads mjpeg+pcm_s16le, mp4 reads h264+aac
- Independent review: idx1 off-by-4 (ffmpeg convention), temp-file leaks,
  envelope clicks - all fixed pre-merge; Devin Review pass

## Demo artifacts (gitignored, in evidence/videos/)
play-10s.mp4 (realtime+scripted), play-1m.mp4 (2x), play-5m.mp4 (5x),
play-10m.mp4 (10x), play-30m.mp4 (10x = full in-game year in 3min),
demo-30s.avi (pure-Rust path)
