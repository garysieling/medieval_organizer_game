# Test fixtures

`sine440.ogg` - 1 s, 44.1 kHz mono 440 Hz sine, own-synth
(ffmpeg `sine` filter, generated in-repo). It lives under `tests/`,
not `assets/sound/`, so it sits outside the ledger gate on purpose:
the gate checks committed game assets, and this file exists only to
exercise the decoder.
