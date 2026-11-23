# assets/sound

Every file under this directory is redistributed raw with the repo, so
the license policy is strict (wiki/software-design/sound-system.md,
decisions 6 and 7):

- **Allowed:** `CC0-1.0`, `unrestricted` (with the grant's own words in
  `grant_quote`), `own-synth` (rendered by our code). Nothing else.
- **`LEDGER.json`** is a JSON array with one entry per file:
  `id`, `path` (under this dir, forward slashes), `source_url`,
  `author`, `license`, `license_evidence_url`, `sha256` (of the
  committed, processed file), `processing` (the ffmpeg recipe it got),
  and `grant_quote` when the license is `unrestricted`.
- **`_incoming/`** is gitignored staging: fetched files waiting to be
  processed never get committed.
- Total budget: **20 MB** of OGG.

`cargo test -p game2-audio ledger` gates all of it - an unledgered
file, an off-list license, a missing grant, a hash mismatch or an
over-budget tree fails the build.

## Fetching candidates

`scripts/sound_fetch.py` searches Freesound's CC0 corpus and downloads
HQ OGG previews into `_incoming/` with a JSON sidecar each:

```sh
export FREESOUND_API_KEY=...   # never committed, never printed
python3 scripts/sound_fetch.py 'church bell' --max 5
python3 scripts/sound_fetch.py 'church bell' --dry-run   # list only
```

## Processing into assets

`scripts/sound_process.py` runs the ffmpeg recipe and appends the
ledger entry (sha256 computed on the processed file):

```sh
python3 scripts/sound_process.py 12345 --name church-bell --kind oneshot
python3 scripts/sound_process.py 12345 --name river-bed --kind bed --dry-run
```

- beds: trim, `loudnorm` to -20 LUFS, crossfade the tail into the head
  for a seamless loop
- one-shots: trim, peak-normalise to -3 dBFS, mono (point sources)
- instruments: mono, keep notes at 4-semitone intervals (the sampler
  pitch-shifts ±3)
- everything: 44.1 kHz Ogg Vorbis `-q:a 5`, written under
  `beds/`, `oneshots/` or `instruments/`

Linux uses the system `ffmpeg`; on Windows run
`scripts/get-ffmpeg.ps1` first and put it on PATH.
