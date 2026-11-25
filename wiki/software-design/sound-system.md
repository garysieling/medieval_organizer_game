# The Wanderer — Sound & Music System Design Report

2026-11-23 · Gary Sieling

## Summary and decisions to approve

Build one deterministic, pure-Rust audio crate that turns each frame's public `UiModel` into a generated medieval score plus a place-based soundscape, delivered as 29 tickets in 6 milestones. Approved by Gary on 2026-11-23; tickets are filed from the table in "Ticket plan".

- **The music is generated, not looped.** A director reads public game state. A composer writes 8-bar modal phrases for period instruments: drone, tabor, lute, recorder/fife, bagpipe, psaltery, bells and organ.
- **Tension builds toward planned events** (an assembly sitting, a campaign meeting coming due, a march, a feast) and crescendos when one arrives. The outcome then shifts the mode over a fixed drone, so the key signature really changes: D Lydian has 3 sharps, D Phrygian has 2 flats.
- **The soundscape comes from what the camera sees**: tiles, buildings, people, carts, weather and season. Birds in the woods, bees at the skeps, wheels at the gate, bells at the church.
- **Everything renders offline.** Tests, golden traces, and WAV, spectrogram and piano-roll files let an agent see audio bugs without hearing them.

| # | Decision | My default | Why | Alternative |
|---|---|---|---|---|
| 1 | Audio engine | Custom deterministic mixer/synth in a new `game2-audio` crate (std, plus `lewton` to decode OGG) | Bit-exact offline renders for tests and `record_video`; the same code runs in wasm; fits the repo's "no engine magic" stance | `kira`: tweens and a mixer for free, but limited wasm support (no file loading, no streaming) and no offline render path |
| 2 | Desktop output | `cpal` (already a dependency behind `dev-voice`), turned on by a default `audio` feature in `game2-client` | minifb has no audio, so the desktop is silent today | `rodio` |
| 3 | Web output | The wasm engine plays through cpal's WebAudio backend; `sound.js` is deleted once the wasm path matches it | One engine, one sound, on both clients | Keep the JS SoundBoard for effects only |
| 4 | Where the director runs | In the client, reading `UiModel` only, never `Game` or the probe | Music is an information channel. Scoring hidden state (say, the lord's plant) would leak it | Run it in the sim and project a mood |
| 5 | Existing sim-side mood | Retire `game/music.rs` evolution and `UiModel.music` | Its prosperity term averages every villager's purse, which the player can't see | Keep it as a debug line |
| 6 | Asset license policy | CC0 or our own synthesis only; a ledger plus a license-gate test | The repo is public, so every committed file is redistributed raw. Sonniss and Pixabay forbid standalone redistribution, and BBC is non-commercial | Allow CC-BY with a `CREDITS.md` |
| 7 | Audio size budget | 20 MB of OGG at most | Assets are 24 MB today, and the web build downloads them all | Higher budget, or lazy loading |
| 8 | Tonic | One fixed drone pitch per village (D by default; the seed may pick D, G or A) | Drones (bagpipe, hurdy-gurdy, organ pedal) stay valid; outcomes change the mode, not the tonic | Modulate by fifths |
| 9 | Musical time | Phrases run on real seconds, not sim days; turbo is a time-lapse in which the music holds sections and never speeds up | A day is 21 s at normal speed but 3.8 s at turbo | Scale tempo with speed |
| 10 | Settings UI | Desktop: a "Sound" overlay off the pause menu. Web: an HTML panel bound to the same settings API | The web build has no pause menu | Rust overlay on both |
| 11 | Freesound access | API key in `$FREESOUND_API_KEY` (like the Retro Diffusion token); agents fetch HQ OGG previews | Original files need an OAuth2 login | You download originals by hand |
| 12 | Lute and hurdy-gurdy | Synthesized: Karplus–Strong pluck and a bowed drone model | I found no CC0 sampled lute or hurdy-gurdy | A paid library (licence conflicts with #6) |

## What exists today

The repo has the start of a design (a sim-side mood with techno-style layers, a cue queue, a WebAudio voice per cue) but no audio at all on desktop, and nothing plays the mood. Checked at `main` c5e1d60 (2026-11-23).

| Piece | Where | What it does | Keep? |
|---|---|---|---|
| Mood + layers | `game2-core/src/game/music.rs`, fed in `session/subsystems/day.rs` | 6 moods, 5 layers (drone, pluck, choir, drum, bell) eased toward targets once per sim day, with an 8-tick hold. Inputs: kids, burning feuds, recent graves, average purse, tech | Retire (decision 5); reuse the hold idea |
| Mood on the wire | `UiModel.music` | Mood name plus layer weights; nothing reads it | Remove after the director lands |
| Cue queue | `Game.sfx: Vec<SfxCue>` (`session.rs`); `emit_sfx` in `subsystems/travel.rs` | 8 kinds: rain, anvil, festival, wonder, bell, snowballs, splash, mood. Cleared every frame, projected as `sfx_v` | Keep; grow the palette |
| Cue cadence | `subsystems/animals.rs::sfx_tick` | Rain every 20 days, anvil every 40, festival every 90. At 21 s a day the festival cue sounds every ~31 minutes | Replace with the soundscape |
| Web sound | `game2-web/sound.js` | One WebAudio oscillator or noise voice per cue kind, panned; resumes on first click or key | Replace (decision 3) |
| Video sound | `game2-client/src/record.rs` | `synth_track` renders the same voices offline into AVI/ffmpeg output; `write_wav` exists | Port to the new engine |
| Desktop | `game2-client/src/bin/game_ui.rs` (minifb) | No audio output. `View.mute` exists but nothing reads it, and no key sets it | Wire up |
| Settings | `game2-client/src/hud.rs` | Pause menu has only Resume and Quit | Add a Sound row |
| Design notes | `wiki/software-design/evolving-music-system.md`, `wiki/artistic-references/sfx-ambient.md` | Layered stems, an ambient bed, cues from the sim | This report supersedes both |

The repo already has what the testing plan needs. The sim is deterministic per seed. `session/stage.rs` drops a village into Fever, Famine, HardWinter, StrongOrg, Unrest, Flush or Bereft. `force_event` and `force_road_march` trigger moments on demand. `game_server` streams scenes over WebSocket and has a separate probe socket. `GAME2_BENCH` times sections, and agents already verify features by dumping a frame and reading the image.

Time runs at 60 FPS. A day lasts 21 s at normal speed, about 3.8 s at turbo, and twice as long during a house meeting. Holy days come every 7 days, market days every 30, and assemblies every 14, with the agenda posted 2 days ahead.

## Free sample sources and license policy

Use CC0 sources only, because the repo is public and committing a file redistributes it as-is. That rules out the big "royalty-free" bundles, but CC0 still covers most of a medieval palette: Versilian's VCSL and VSCO 2 CE for instruments, FreePats for a bagpipe, Freesound's CC0 filter for field recordings, and Kenney for UI and foley. The gaps (lute, hurdy-gurdy, chant) get filled by synthesis.

| Source | License | What we'd take | Verdict |
|---|---|---|---|
| VCSL | CC0 1.0 | Baroque recorders (soprano, alto, tenor, bass), Renaissance and pipe organ, folk harp, strumstick, psaltery (bowed and plucked), frame drum, rope-tension snare (as tabor), hand bells, tubular bells, finger cymbals, sleigh bells, tambourine, anvil, ratchet, woodblock, timpani | **Primary instrument source** |
| VSCO 2 CE | CC0 1.0 | Violin and viola (for vielle/rebec), flute and piccolo (for fife), oboe (for shawm), low brass for the lord's motif | Secondary |
| FreePats bagpipe | CC0 1.0 | Bagpipe in G, F4–G6, with 2 drones (SFZ + FLAC) | **Use** for the drone and chanter. Check each FreePats page: their GM sets are not CC0 |
| Karoryfer via sfzinstruments | CC0 for the freebies | Horse Pulse (bass tagelharpa, pizzicato); Etherealwinds Harp II CE | Optional |
| Univ. of Iowa MIS | "Without restriction" (a statement, not formal CC0) | Clean anechoic orchestral notes | Optional; quote the statement in the ledger |
| Freesound | Per sound: CC0, CC-BY, CC-BY-NC, Sampling+ | Birds, bees, rivers, rain, wind, crowds, carts, hooves, church bells, sheep, chickens, fire | **Use CC0 only**, API filter `license:"Creative Commons 0"` |
| Kenney audio | CC0 | UI clicks; RPG foley (cloth, coins, creaks, footsteps) | **Use** for UI and foley |
| OpenGameArt | Mixed, per asset | Whatever is tagged CC0 | CC0 entries only |
| Sonniss GDC bundles | Royalty-free EULA; no republishing the files unchanged | Great field recordings | **Do not commit** |
| Pixabay | Pixabay licence; bars standalone redistribution | — | **Do not commit** |
| BBC Sound Effects | RemArc: personal, educational or research use only | — | **Never** |
| Sonokinetic Hurdy Gurdy | Free, but a proprietary Kontakt library | — | **Never** (synthesize instead) |

Rules for the asset tickets:

1. **Allowed in the repo:** `CC0-1.0`, `unrestricted` (with the grant quoted), or `own-synth` (rendered by our code). Nothing else.
2. **Ledger:** every file under `assets/sound/` has an entry in `assets/sound/LEDGER.json` with id, path, source URL, author, licence, licence-evidence URL, the committed file's sha256, and processing notes. This mirrors `art-prompts/generated.json`.
3. **License gate:** the test `ledger_covers_every_sound` fails when a file has no entry, its licence isn't on the allowlist, or its hash doesn't match.
4. **Processing** (ffmpeg, via `scripts/get-ffmpeg.ps1`): trim; normalise beds to −20 LUFS and one-shots to a −3 dBFS peak; crossfade beds into seamless loops; mono for point sources; 44.1 kHz OGG at q5. Keep only the notes we play: the sampler pitch-shifts up to ±3 semitones, so one sample every 4th semitone is enough.
5. **Budget:** 20 MB in total (decision 7).
6. **Freesound access:** search and HQ OGG previews need only an API key, which goes in `$FREESOUND_API_KEY` and never in the repo. Original files need an OAuth2 login. CC0 covers the preview as much as the original.

## Research foundations

The design borrows from five traditions: from techno, structure; from film and contemporary classical, drama; from medieval music, material; from game audio, mechanics; and from soundscape studies, place. Each row below says what we take and where it lands.

| Idea | Source | What we take |
|---|---|---|
| **The 8-bar rule**: every 8 bars something is added, removed or varied | Techno arrangement rules; Beat Kitchen | The arranger changes at most one layer per 8-bar phrase, so the music evolves without lurching |
| **Build, breakdown, drop**: builds add layers; breakdowns strip the kick and bass; the drop brings everything back | Beat Kitchen; layering as form | Our sections. A build heads toward a planned event, and the arrival is the drop. Afterwards comes a breakdown, the release |
| **Leitmotif network**: themes for characters and places, with cultures set apart by instrument (hobbits get fiddle and tin whistle, elves get a cappella voices, orcs get percussion and horns) | Britannica on Shore's LOTR scores; LibreTexts | A motif plus a fixed instrument for each faction: village, the cause, the lord, the church, the old faith, the road |
| **How tension is built**: short repeated figures, tremolo, more instruments entering, a figure rising in pitch each time it repeats; ostinatos for anticipation; sudden silence | The Conversation; Fiveable | The build toolkit: tabor ostinato, hurdy-gurdy tremolo, the motif sequenced up a step each phrase, layers entering, a one-bar rest before the arrival |
| **Music as a gradual process**: a process you can hear, unfolding very slowly | Reich, 1968 | Gain and density move over phrases, not seconds, and the player can hear the village change |
| **Tintinnabuli**: a stepwise melodic voice (M) plus a voice confined to one triad (T), set by a strict rule. An algorithm reproduces Pärt's *Summa* at 93% of notes | arXiv 2603.26989 | A second voice that is always consonant, for free: for each melody note, take the nearest note of the tonic triad |
| **Game data as performers**: each metro line is a performer whose part changes slowly with play | Disasterpeace on Mini Metro | The org, the market and the weather each drive one musical parameter |
| **Estampie**: 4–7 *puncta*, each played twice, the first time with an open (*ouvert*) ending and the second with a closed (*clos*) one; usually in triple time | Wikipedia: Estampie; SCA early dance | Phrase grammar. While a campaign's fight is unresolved, phrases end *ouvert* (on degree 2 or 5); once it resolves, they end *clos* on the final |
| **Church modes and their ethos**: 8 modes; 11th-century sources call Dorian serious or noble, Hypodorian mournful, Phrygian excited | SAGE: Mode; Britannica | A modal vocabulary instead of major/minor. Outcomes move the mode along a brightness ladder |
| **Vertical layering, horizontal resequencing, stingers**: stingers stay in key and land on a beat or at a phrase's end | Berklee; White Rose thesis | Vertical layers over generated phrases; stingers quantized to the beat and written in the current mode |
| **iMUSE**: instruments come in at markers when it makes musical sense | Giant Bomb; Wikipedia | Changes land only on bar or phrase boundaries, never mid-bar |
| **Kingdom Come: Deliverance**: music is either responsive or anticipatory, driven by game-state variables. Valta: you need reasons for when music matters, not for silence | Sporka's talk; PC Games interview | Builds anticipate planned events. A silence budget, set by the "score presence" setting, keeps music from playing wall-to-wall |
| **No Man's Sky, Pulse**: a library of textures, loops and melodies, layered at random and more or less menacing with the player's state | UAL analysis | Authored motifs and rhythms, chosen and combined by the composer |
| **Left 4 Dead's AI Director**: pacing through build-up, sustained peak, peak fade and relax | Valve slides; Valve wiki | The director's section state machine, with minimum hold times |
| **Keynote, signal, soundmark**: background sounds heard without attention; foreground signals; sounds unique to a place | OpenLearn on Schafer; Canadian Encyclopedia | The three tiers of the soundscape: looped beds, positioned one-shots, and landmark sounds (the church bell, the mill, the stones) |
| **Village bells**: bells marked time, called people to pray or assemble, warned of danger, and stirred feelings of belonging | Corbin, *Village Bells*; JHU review | A bell vocabulary: the call on holy days, the toll at funerals, the alarm rung off schedule for fires and crackdowns |
| **Ambience practice**: beds plus one-shot sweeteners; zones that crossfade; distance takes away detail, not just volume | Eon Altar; CryEngine | The ambience mixer: beds weighted by the camera's surroundings; one-shots scattered around the listener; low-pass with distance |
| **Loudness**: Sony's ASWG-R001 sets −24 LKFS for home platforms and −18 for portable, with a −1 dBTP true peak | Audiokinetic; ITU BS.2434 | Test target: −20 ±3 LUFS integrated at default settings (laptop speakers and a browser sit between the two), true peak ≤ −1 dBTP |

## Architecture

A new workspace crate, `game2-audio`, sits beside `game2-client`. It reads only `UiModel` and does all the listening, composing and rendering, deterministically. `game2-client` (desktop window, `record_video`, the new `sound_dump`) and `game2-web` call one façade, `AudioHost`.

*(Diagram omitted in this markdown copy.)*

The same `AudioFrame` path feeds the speakers in play and the files in tests, so what the tests check is what the player hears.

**Layer rules** (added to `docs/architecture.md`):

1. `game2-audio` depends on `game2-core` and uses only `session`'s public API: `UiModel`, its view types and `layout.rs` camera maths. It never names `Game`, `World` or `probe` outside `#[cfg(test)]`, and a layering test greps for this. `game2-core` never depends on it.
2. The sim's only audio job is publishing facts: cues, `DramaV`, weather, calendar. It picks no sounds and no moods.
3. No clocks and no OS randomness. Every random choice comes from an xorshift seeded by `UiModel.seed` plus the bar index, so one scene stream always renders the same audio offline.

**Modules in ****`game2-audio`****:**

| Module | Job | Key types |
|---|---|---|
| `engine` | Voice pool (cap 48), buses (Master, Music, Ambience, Sfx, Ui), filters, a reverb send per bus, soft limiter at −1 dBFS; `render(&mut [f32])` writes interleaved stereo | `Engine`, `Bus`, `VoiceId` |
| `sampler` | Loads OGG (via `lewton`) or WAV from bytes; zones with key range, root note and loop points; linear interpolation | `SampleBank`, `Patch` |
| `synth` | Karplus–Strong pluck (lute, gittern), bowed drone (hurdy-gurdy), formant choir pad, inharmonic bell, wind and rain noise, the 8 legacy cue voices | `Synth` |
| `theory` | Modes, degrees, key signatures, triads, tintinnabuli, motif transforms, Euclidean rhythms, instrument ranges | `Mode`, `Degree`, `Motif` |
| `compose` | Clock (tempo, meter, bars, phrases), phrase grammar, voice parts producing note events | `Clock`, `Composer`, `NoteEvent` |
| `director` | `UiModel` stream to musical intent: mood vector, section, mode, layers, stingers | `Director`, `Intent` |
| `soundscape` | `UiModel` plus listener to positioned emitters; the ambience mixer for beds and one-shots | `Emitter`, `Ambience` |
| `settings` | Volumes, presence, mono, night mode; serde | `AudioSettings` |
| `trace` | Per-bar records of what was decided and played, as JSON | `BarTrace` |
| `analysis` (feature) | Loudness, true peak, clicks, silence gaps, pitch classes; spectrogram, piano-roll and timeline PNGs | used by tests and `sound_dump` |

**Threads.** Each frame, the main thread runs the scan and the director, which are cheap, and sends a small `AudioFrame` (intent, emitters, cues, listener) down a channel. The audio callback runs the composer, the ambience mixer and the engine against the latest frame. Offline, the same code runs in lockstep, one frame and then that frame's samples, which is what makes renders bit-exact. The web build is single-threaded, and the split works the same way there.

**Rates and budgets:** the engine runs at the device rate, and offline renders use 48 kHz, the rate BS.1770's true-peak filter is specified for. Audio may take at most 1 ms of CPU per 10 ms block on a desktop release build, shown as `audio.*` sections in the `GAME2_BENCH` table. On web it may take at most 2 ms of each frame.

## Game measures that drive the sound

`UiModel` already carries most of what the music and the soundscape need. Three small projections fill the gaps: the weather event plus a calendar and clock, tags on log lines, and a structured `DramaV` for the campaign arc. Every one is built from facts the seat can already see.

**Already public**

| Signal (`UiModel` field) | Drives |
|---|---|
| `season`, `harsh_winter`, `food_yield` | Which beds play (spring birds, summer insects, winter wind); baseline tempo; palette (winter thins the winds) |
| `precip` (Drizzle through HeavySnow, Haar) | Rain and snow beds; a low-pass "hush" in snow; more reverb and a duller sound in haar |
| `sky`, `rainbow_from` | A wonder layer (bells, a high recorder) and a one-step lift up the modal ladder while it lasts |
| `feast`, `feast_name`, `stalls` | Festive section (an estampie in triple time, tambourine); the market crowd bed |
| `people` (position, kind, `pose`, `sick`, `hungry`, `dead`, `mounted`) | Crowd density; work sounds (hammering, hoeing); hooves; guards in the READY pose raise tension |
| `buildings` (`sprite`, `smoke`, `derelict`), `mill`, `skeps`, `swarm`, `boats`, `trains`, `stones`, `graveyard`, `cave` | Point emitters: church, smithy, tavern, apiary, fold, mill wheel, moored boats, cart wheels, soundmarks |
| `map` tiles (River, Ford, Water, Hill, Rock, Wheat), `map.trees`, walls and gates | Beds: river, wind on high ground, birds weighted by nearby trees, wheels and creaking at gates |
| `burning`, `flood` | Fire crackle; an alarm bell rung off schedule; rushing water |
| `panel.campaign` (state, dissenters, due, assembly, leaders), `panel.org_status`, `panel.members` | Tension, how fully the cause's motif is stated, mode drift |
| `player_event` (march, house meeting, rite; moving) | Live-event section: march drums, procession |
| `convo`, `talk.in_talk` | Ducks the score by 9 dB and drops melody, keeping drone and harp |
| `toasts`, `sfx_v` | Stingers and one-shots (toasts stand in until log tags land) |
| `public_kpis()` (graves, marked hungry and sick, fires, food) | The "village fortune" baseline that sets the resting mode |
| `bonds` | A death plays the grief motif only if the seat knew the person |
| `paused` | Music holds at bed level, −6 dB |

**New projections** (each lands with a `docs/protocol.md` update, since `tests/protocol_doc.rs` checks the examples)

- **P1 — weather, calendar, clock.**
  - `weather: Option<WeatherV { kind, intensity, days_left }>` exposes `CurrentWeather` (storm, drought, heat, frost), which isn't projected today.
  - `calendar: CalendarV { holy_day, market_day, feast: Option<String>, days_to_holy, days_to_market }` comes from `HOLY_EVERY = 7` and `is_market_day`.
  - `clock: ClockV { day_frac, frames_per_day, speed }` lets the client turn days into seconds.
- **P2 — log tags.** `LogEntryV.tag: Option<TagV { thread, valence, sev }>` publishes the `EvTag` the log already stores. A Major Negative line triggers a stinger.
- **P3 — ****`DramaV`** under `panel`:
  - `phase`: the `CampaignPhase` (listening, issue cut, research, action, evaluate).
  - `next: Option<PlannedV { kind, in_days, at }>`: the soonest public planned event. An assembly sits `announced + PUBLISH_GAP`; a campaign meeting is due at `last_held + cadence_days`; a pledge comes due at its `deadline`; feasts and holy days come from the calendar.
  - `live: Option<LiveV { kind, progress }>`: what's happening now. For a march, progress is the route position over its length; it also covers house meetings, campaign meetings, assemblies, townhalls, strike votes, funerals and feasts.
  - `last: Option<OutcomeV { kind, verdict, day }>`, with the verdicts mapped in the music section below.
  - `pressure: { guards_posted, assembly_ban, members_held, leader_taken }`: today's `org_status` strings, made structured.
- **No-leak rule for P3**: no field may read the plant, a seat offer's `binding` flag, true organized power, or anything under `probe`. The test builds two staged worlds that differ only in that hidden state and asserts their `DramaV` is identical.

## Music design

The score is a slow techno-style arrangement of medieval material. A director turns game state into a mood and a section. A composer writes 8-bar estampie-shaped phrases over a fixed drone. Campaign outcomes move the mode up or down a brightness ladder.

### Director: mood vector and sections

Each frame, the director eases a mood vector of seven values between 0 and 1: **tension, valence, energy, sacred, grief, wonder, festivity**. Section changes take effect only at phrase boundaries, and a section holds for at least 2 phrases.

- **Tension** rises with the campaign phase (listening 0.1, issue cut 0.25, research 0.4, action 0.6), with `pressure` (+0.15 each), with guards in the READY pose, with fires, and as `drama.next` gets closer. A live march sets it to 0.9.
- **Valence** comes from the village-fortune baseline (public KPIs) and the last verdict.
- **Sacred, grief, wonder and festivity** come from place and calendar: a holy day near the church, a recent death of someone the seat knew, a `sky` event, a feast.

| Section | When | Sounds like |
|---|---|---|
| Silence | The presence budget is spent, or the player is in a conversation with presence below Normal | Ambience only |
| Ambient | Low tension and nothing planned | Drone and sparse harp; the melody enters every other phrase |
| Theme | Default once the village has a story | The village motif, then variations; full voices at moderate density |
| Build | `drama.next` is within the build window (4 phrases, about 60–85 s) | One layer added per phrase, +2 bpm per phrase (capped at +12), the motif sequenced up a step, *ouvert* endings, a one-bar rest just before the arrival |
| Peak | `drama.live` is set (march, assembly, townhall) | Full band in the event's own style: a march is 2/4 with tabor, fife and the cause motif, its intensity following `progress` |
| Release | Right after a verdict | A stinger on the next downbeat, then a *clos* cadence; the next phrase takes the new mode and strips back like a breakdown |
| Festive / Sacred / Lament | Feast; holy day near the church; a known villager's death | Estampie in 3/4 or saltarello in 6/8; organum and a chant pad; slow solo vielle and harp |

The build is **anticipatory**. Seconds until the event equal `in_days × frames_per_day / 60`. The director counts the phrases that fit and starts building early enough that the arrival lands on a phrase downbeat. If the event moves (a postponed meeting), the build holds on its current layer and never drops abruptly.

*(Diagram omitted in this markdown copy.)*

The build adds a layer and lifts the motif each phrase, rests one bar, and lands the peak on the event. The verdict's stinger then sets the mode that the release and the following phrases play in.

### Outcome to mode: the "key signature"

The drone stays on the village's tonic (D by default), and outcomes move the mode along a brightness ladder. On D, each step adds or removes one accidental, so the key signature changes in the literal sense:

**Lydian (F♯ C♯ G♯) → Ionian (F♯ C♯) → Mixolydian (F♯) → Dorian (none) → Aeolian (B♭) → Phrygian (B♭ E♭)**

An ordinary village rests in **Dorian**. After a verdict the mode moves the given number of steps. Then it drifts back toward the fortune baseline by one step every 8 phrases, about 3 minutes.

| Event (from `drama.last`) | Verdict | Mode steps | Stinger |
|---|---|---|---|
| Townhall: Carries; Lord's grant | win | +2 | Bells peal; the cause motif stated in full, *clos* |
| Townhall: Concedes; strike called; pledge kept | partial / win | +1 | The cause motif's first 3 notes, *clos* |
| Townhall: Deflects; march ignored | stall | 0 (endings stay *ouvert*) | An unresolved half-cadence; the march fades into a breakdown |
| Townhall: Refuses; march watched; strike failed; pledge broken; grant diluted, expired or broken | loss | −1 | Low drum; the cause motif inverted |
| March: Crackdown; leader taken | crackdown | −2, and the pressure layer comes on | Alarm bell off schedule; shawm and drums play the lord's motif |

A `sky` wonder lends one step of brightness while it lasts. Grief plays in Aeolian at −15% tempo. Sacred music uses organum in Dorian.

### Leitmotifs

| Motif | Instrument | Character | How it develops |
|---|---|---|---|
| Village | Recorder + lute | Stepwise, 8 notes, pastoral | Season changes its colour: a bright recorder in spring, a low recorder and bowed psaltery in winter |
| The cause | Fife, then fife + tabor, then the full band | A 4-note cell rising a 4th, then a 5th | Notes stated = min(4, 1 + leaders); fragmented when there are 2 or more dissenters; inverted after a loss; stated in full on a win |
| The lord / castle | Shawm + low drum + bass | Phrygian, dotted, heavy | Enters when guards are posted, on a march to the castle, at a townhall with the lord |
| The church | Organ + chant pad | Organum in parallel 4ths and 5ths | Holy day, funerals |
| The old faith | Frame drum + bagpipe drone + bowed psaltery | Mixolydian, a circling figure | Near the stones or the grove; old-faith rites |
| The road | Hurdy-gurdy | A dance tune | Visitors, the pedlar, carts arriving |
| Grief | Solo vielle + harp | Aeolian, falling | A known villager's death |
| Feast | Bagpipe, recorder, tambourine | Ionian or Mixolydian, 3/4 or 6/8 | Feast days; a win at the market cross |

All motifs are hand-authored as scale degrees plus rhythm, in `compose/motifs.rs`. The composer varies them and never invents a main theme.

### Composer: phrase grammar and voices

- **Phrase = 8 bars = one estampie punctus played twice**: 3 bars of body plus 1 bar of ending, then the body restated a sequence step on with the second ending. The first ending is *ouvert* (it lands on degree 2 or 5). The second is *clos* (degree 1), unless the arc is unresolved, in which case both stay *ouvert*.
- **Tempo bands:** ambient 60–72 bpm, theme 84–96, build and march 96–112, feast 108–126.
- **Voices:** Drone (tonic and 5th), Pulse (Euclidean patterns such as E(3,8) and E(5,8), scaled by density), Ground (bass on beats 1 and 3), Harmony (lute open 5ths or organum), Melody, Counter (the tintinnabuli T-voice, taken from the tonic triad), Color (bells and finger cymbals on phrase starts), Choir (sacred only).
- **Melody rules:** state the motif first, then vary it by transposing within the mode, inverting, augmenting or fragmenting. Each punctus's body follows a contour: head notes climb (or, for falling motifs and laments, sink) a step per bar toward the ending, and the restatement shifts the strain by a sequence step so the phrase's apex sits in the repeat. Free notes walk toward the next bar's head — or toward the ending degree on a punctus's last bar — by step 70% of the time, by a 3rd 15%, and may settle into a held note on arrival. A leap resolves by a step the other way. Notes stay in the instrument's range, and no bar repeats twice running.
- **Arranger:** each voice gets a density from 0 to 1, and at most one voice changes per phrase. That is the 8-bar rule.
- **Presence (silence budget):** Off 0%, Sparse ~35%, Normal ~60%, Rich ~85% of time with music. Music sections end on a *clos* cadence and leave 1–3 phrases of ambience.

### Instruments

| Role | Instruments | Source |
|---|---|---|
| Drone | Bagpipe drones; hurdy-gurdy; organ pedal | FreePats; synth; VCSL Renaissance organ |
| Melody | Recorders (S/A/T/B); fife; shawm; vielle; bagpipe chanter | VCSL; VSCO 2 piccolo; VSCO 2 oboe, filtered; VSCO 2 viola and violin; FreePats |
| Harmony | Lute and gittern; psaltery (plucked); folk harp; strumstick | Karplus–Strong synth; VCSL |
| Pulse | Frame drum; tabor; timpani and bass drum (the lord); tambourine; finger cymbals | VCSL |
| Color | Hand bells; tubular bells (stand-in for the church bell); sleigh bells | VCSL; a real church bell from Freesound CC0 if one turns up |
| Choir | Formant "ooh" pad | Synth (I found no CC0 chant; see open questions) |

## Place-based soundscape

The soundscape is a pure function of what the camera can see. `soundscape::scan(&UiModel, &Listener)` returns positioned emitters in three tiers: keynote beds, signals and soundmarks. The ambience mixer plays them with distance, pan and season applied.

**Listener.** Hearing sits at the camera's centre, converted to tile space through `session/layout.rs`. The radius follows zoom: `radius_tiles = clamp(10 / zoom, 5, 20)`. Beds gather from 1.5× the radius; one-shots play only inside it. Fully zoomed out, the village blurs into a murmur and individual work sounds drop out.

**Spatial rules.** Pan is equal-power from the emitter's screen-x offset, normalised by half the view width. Gain is `1 / (1 + (d / (radius/3))²)`. A low-pass falls from 16 kHz at the listener to 2 kHz at the radius, and the reverb send rises with distance, so distant sounds lose detail and not just volume.

**One-shots.** Each emitter fires on a Poisson rate with a minimum gap. Variants play round-robin without immediate repeats, with ±3% pitch and ±2 dB gain, and at most 2 voices per kind at once.

**Mix.** Signals duck beds by 2 dB. A music Peak ducks ambience by 3 dB. A conversation ducks music by 9 dB.

| Emitter | Rule (from `UiModel`) | Tier | Season and weather |
|---|---|---|---|
| Songbirds | 5 or more `map.trees` within the radius; chirps placed at trees | Bed + one-shots | Strong in spring and summer, light in autumn; winter has only crows and a robin; off in heavy rain or snow |
| Insects | Share of Grass and Wheat tiles | Bed | Summer only; louder in a heat wave |
| Bees | `skeps` within 6 tiles; `swarm` position | Point bed | Off in winter; louder at a swarm |
| River | River and Ford tiles within the radius; Bridge adds water under planks | Bed, weighted by tile count | Rushing water when `flood` is set |
| Wind | Share of Hill and Rock tiles | Bed | Stronger in winter; gusts with storm intensity |
| Rain | `precip` from Drizzle to HeavyRain (Showers come and go) | Bed, level by kind; patter on roofs near buildings | — |
| Snow hush | `precip` Snow or HeavySnow | Master low-pass at 5 kHz, birds off | — |
| Haar | `precip` Haar | Low-pass at 4 kHz, more reverb, a distant bell | — |
| Village murmur | Villagers within the radius | Bed scaled by count | Quieter while a service runs |
| Market | `calendar.market_day` and `stalls` | Crowd bed + criers' calls | — |
| Smithy | A Blacksmith building with someone in the HAMMERING pose nearby; not on a holy day | One-shot rhythm | — |
| Work sounds | People in the HAMMERING or HOEING pose | One-shots at each person | — |
| Church bell | Church building: the holy-day call (existing `bell` cue), funeral toll, alarm | Soundmark; the bell's pitch is seeded per village | — |
| Chant / organ | Holy day with the church within half the radius | Diegetic loop | — |
| Tavern | A Tavern building within 8 tiles | Muffled lute and laughter | Louder on feast days |
| Fold | A Fold building | Bleats, sheep bells | — |
| Homesteads | Density of House buildings | Chickens, a dog, a door | — |
| Carts and riders | `trains` positions; `mounted` people | Moving point: wheels, hooves | — |
| Gate | A train or person on a gate tile | Creak and clatter | — |
| Mill | `mill` position | Wheel-creak loop + water | Stops in a hard frost |
| Boats | `boats` tiles | Lapping water, rope creak | — |
| Fire | `burning` tiles | Crackle bed + alarm bell (a signal) | — |
| Graveyard | `graveyard` or `grave` in range | Crows; ambience −3 dB | — |
| Standing stones | `stones` in range | A wind-hum soundmark (synth) | Stronger in wind |
| Cave | `cave_zone` | Drips; reverb up | — |
| Castle | `castle_keeps` in range | Banners flapping; armour clanks while guards are posted | — |
| Guards | Guards in the READY pose in range | Armour, footsteps | — |
| March / protest | `player_event` march; protest crowd | Moving bed of footsteps and chant | — |

**New sim cues** (the sim emits them; the palette maps them to sounds): `alarm` (a fire starts; a crackdown), `toll` (a death; a funeral procession), `cheer` (a march comes home to a good reaction; a win), `chant` (march under way), `coins` (a purchase), `quill` (a petition or letter), `door` (a visit or canvass), `cart_arrive`, `gate`. UI sounds (click, hover, panel open and close, toast) come from the client, not the sim.

## Settings UI

One client-side `AudioSettings` struct is shown two ways: a Rust-drawn "Sound" overlay on desktop, opened from the pause menu, and an HTML dialog on web. Settings never reach the sim or the server. M mutes sound on both clients.

**Desktop entry.** The pause menu gets a middle row, **Sound… (****V****)**, between Resume and Quit. `MenuPick::Sound` is added, and `pause_menu_rows()` returns 3 rows. The overlay uses the pause menu's style: `overlay_rect`, `MENU_ROW = 24`, bordered rows, a hover highlight. It is centred on the map view and 360 px wide.

V is the hotkey because W/S may move focus in menus and M is mute.

| Row | Control | Range / options | Default | Effect |
|---|---|---|---|---|
| Master | Slider | 0–100, steps of 5 | 80 | Master bus |
| Music | Slider | 0–100 | 70 | Music bus |
| Ambience | Slider | 0–100 | 70 | Ambience bus |
| Effects | Slider | 0–100 | 80 | Sfx bus (sim cues) |
| Interface | Slider | 0–100 | 60 | Ui bus |
| Score presence | Choice | Off / Sparse / Normal / Rich | Normal | The director's silence budget (0 / 35 / 60 / 85%) |
| Mono | Toggle | Off / On | Off | Sums to mono |
| Night mode | Toggle | Off / On | Off | Master compressor (−24 dBFS threshold, 3:1) to tame loud moments |
| Mute in background | Toggle | On / Off | On | Fades out over 300 ms when the window loses focus (`window.is_active()`; `document.hidden` on web) |
| Sound captions | Toggle | Off / On | Off | Small captions for signals ("Alarm bell — church"); separate, optional ticket |
| Reset to defaults | Button | — | — | Restores the defaults above |
| Back (Esc) | Button | — | — | Returns to the pause menu without resuming |

**Drawing.** Each slider row shows its label on the left, a 120 px track with a filled portion, and the value on the right ("70%"). Choice rows show `< Normal >`.

**Input.** Up/Down (or W/S) move focus. Left/Right (or A/D) step a slider by 5 or cycle a choice. Enter toggles or activates. A click on a track sets the value from x, and arrows and toggles click. Every change applies at once; while Effects is being adjusted, a sample cue plays.

**Volume curve.** Amplitude is (v/100)², a square-law taper; 0 means silent.

**Mute.** M during play (unless the find box has focus) toggles `View.mute`. A toast says "Sound off" or "Sound on", and a small muted-speaker glyph sits in the HUD corner while muted.

**Persistence.**

- Desktop: `game2.settings.json` in the working directory, next to `game2.claim`. It's written 1 s after the last change and again on quit. It carries a `version` field; unknown fields are ignored and missing ones take defaults.
- Web: `localStorage["game2.audio"]`.

**Web panel.** A gear button in `index.html` opens a `<dialog>` with the same rows as native range inputs, selects and checkboxes. They call `g.set_audio_setting(key, value)` and read `g.audio_settings()` (JSON). It persists to localStorage and needs no change to the canvas click routing.

## Testing strategy

Almost all of this can be tested automatically. The audio crate renders offline and reads only `UiModel`, so a test can drive a real headless game and check the decisions, the notes and the rendered sound. Agents catch "obvious" bugs by reading images and metrics instead of listening. You do one listening pass at the end.

| Layer | What it proves | How | Command |
|---|---|---|---|
| Theory | Modes, key signatures, triads, transforms and Euclidean rhythms are correct | Unit tests (for example, D Phrygian has exactly B♭ and E♭; every T-voice note is in the triad) | `cargo test -p game2-audio theory` |
| Composer | Generated music obeys the rules | 1,000 seeded phrases: 100% of notes in the mode; leaps ≤ a 5th and resolved; *ouvert* endings on degree 2 or 5, *clos* on 1; notes in range; no bar repeated twice running; density monotonic; same seed gives the same notes | `cargo test -p game2-audio compose` |
| Director | State turns into the right intent | Scripted `UiModel` sequences from real games. Assert: a build starts ≥ 3 phrases before an assembly sits and the peak lands within 1 bar of it; a Refuses verdict moves the mode −1 at the next phrase; tension hovering at a threshold never flips the section (as the existing `test_flickering_metrics_do_not_flip_the_mood` does); conversation ducks; pause holds; turbo never changes tempo | `cargo test -p game2-audio director` |
| Soundscape | The right sounds for the place | Camera at the skeps: bees are the top emitter. Church on a holy day: bell + chant. Gate with a train: wheels. Winter: no insects. Heavy snow: the hush filter. Zoomed out: no one-shots | `cargo test -p game2-audio soundscape` |
| Engine (DSP) | The mixer is sound | Sine RMS within 0.1 dB; no NaN or Inf; limiter keeps peaks ≤ −1 dBFS; bit-identical across two renders; voice cap of 48 holds; no clicks at voice steal or crossfade; silent when muted or at Master 0; mono sums correctly | `cargo test -p game2-audio engine` |
| Mix | A whole scene is listenable | 60 s offline renders of the scenarios: −20 ±3 LUFS integrated, true peak ≤ −1 dBTP, no clicks, no music gap beyond the presence budget, ambience never fully silent unless muted; 60 s renders in under 3 s on release | `cargo test -p game2-audio --release mix` |
| Golden traces | Nothing changed by accident | Per-bar JSON (section, mode, tempo, meter, layers and density, motif ids, note hash, top 5 emitters, stingers) for 10 scenarios in `game2-audio/tests/golden/`. `GAME2_GOLDEN=update` rewrites them, and the PR explains the diff. PCM hashes are compared only within one run, since floating point differs across machines | `cargo test -p game2-audio golden` |
| Projections | P1–P3 report the truth and leak nothing | `session/tests/sound_facts.rs`, plus the two-world no-leak test; `protocol_doc` | `cargo test -p game2-core --lib sound_facts` and `cargo test -p game2-core --test protocol_doc` |
| Settings UI | Menu, keys, persistence | Draw tests in `game2-client/src/tests/sound_menu.rs`: centred box, row hit-tests, keyboard focus and steps, settings JSON round-trip, M toggles mute | `cargo test -p game2-client --lib sound_menu` |
| Soak | Long play stays healthy | `GAME2_SOUND_SOAK_DAYS=400`, a headless run at speed 16: no NaN or clipping, never more than 48 voices, loudness drift ≤ 4 LU between the first and last 10 minutes, flat memory | `cargo test -p game2-audio --release soak -- --ignored` |
| Listening | It sounds good | A 5-minute scenario reel; you note issues against a checklist | Manual (ticket I5) |

### Hooks into the simulation backend

- **`sound_dump`**, a new bin in `game2-client`, is the audio twin of `--dump-frame`. It builds a game (`--seed`, `--stage`, `--scenario`), applies `record_video`'s frame-keyed script ops (walk, camera, `force_event`, `force_road_march`, talk), and runs N frames headless through `ui_model()`. Then it writes `<out>.wav`, `.trace.json`, `.metrics.json`, `.spectrogram.png`, `.roll.png` and `.timeline.png`. Example: `cargo run -p game2-client --release --bin sound_dump -- --scenario assembly_build --frames 7200 --out evidence/sound/assembly`
- **Scene capture and replay.** `game_ui --record-scenes path.jsonl` saves each frame's `UiModel` (minus the map, which goes once). `sound_dump --scenes path.jsonl` re-renders it offline, so a real playtest becomes a regression test without the window.
- **Live server.** `sound_dump --connect ws://host:port` joins a `game_server` as a human seat and records its scene stream. That tests the multiplayer path, where scenes arrive at irregular intervals.
- **Scenario library** (`game2-audio/src/scenarios.rs`, built with `stage`, `force_event` and `force_road_march`, which are harness-only uses the architecture allows in tests): `quiet_village`, `holy_day_church`, `market_square`, `apiary_spring`, `winter_gate`, `fire_alarm`, `fever_lament`, `assembly_build`, `march_crackdown`, `feast_day`.

### How an agent sees audio

After each change the agent runs `sound_dump` on the 3 nearest scenarios and opens the PNGs with the Read tool, as AGENTS.md already says to do with BMP dumps:

- **Spectrogram:** clipping shows as full-band smears, clicks as thin vertical lines, dropouts as dark bands, a hard cut as a vertical edge.
- **Piano roll:** mode lines are drawn in the background and out-of-mode notes in red. Stuck notes show as long bars; bad registers show as notes outside the instrument's band.
- **Timeline:** the tension curve, section bands, a mode lane, layer lanes and `DramaV` event markers, so "the build peaked late" or "the mode never moved" is visible at a glance.
- **`metrics.json`****:** LUFS, true peak, click count, longest silence, voices at peak, render ms. The ticket states the pass thresholds.

## Ticket plan

The plan has 29 tickets in 6 milestones, ordered so the engine and its test harness come first. Six can start on day one: S0, S1, P1, P2, P3 and M1. The critical path runs S1 → A1 → A2 → M2 → M4 → M5 → I1 → I5.

*(Diagram omitted in this markdown copy.)*

Coloured tickets are the critical path. Public facts, M1 and S0 have no dependencies, so they can run in parallel with the engine from the start.

| ID | Ticket | Depends on | Done when | Verify |
|---|---|---|---|---|
| S0 | Land this design as `wiki/software-design/sound-system.md`; point the two old notes at it; add the layer rules to `docs/architecture.md` | — | The wiki page exists and the old notes link to it | Review |
| S1 | `game2-audio` crate: engine core (voices, buses, filters, limiter, seeded RNG); move `write_wav` from `record.rs`; layering test | — | A sine renders to spec; bit-identical twice; peaks ≤ −1 dBFS; layering test passes | `cargo test -p game2-audio engine` |
| S2 | Analysis toolkit: LUFS (`ebur128` crate), true peak, clicks, silence, pitch classes; spectrogram, roll and timeline PNGs | S1 | The analysis of known test signals matches expected values ±0.5 | `cargo test -p game2-audio analysis` |
| S3 | `sound_dump` bin plus scene capture and replay (`--record-scenes`, `--scenes`, `--connect`); port the 8 legacy cue voices into `synth` | S1, S2 | A dump at frame 3600 writes all 6 outputs; replaying a capture gives identical audio | `sound_dump --frames 3600 --out evidence/sound/s3` and Read the PNGs |
| S4 | Desktop output: `cpal` behind a default `audio` feature; `AudioHost` bridge; `--no-audio`; silent fallback with no device; M mutes | S1 | `game_ui` plays the cue voices; no device means a warning, no crash | Draw/bridge tests; run `game_ui` |
| S5 | Web output through cpal's WebAudio backend; delete `sound.js` once the wasm path matches it | S4 | The web build plays the same cues; first gesture resumes audio | `wasm.yml` build; manual browser check |
| P1 | Project `weather`, `calendar`, `clock` | — | Fields are correct on staged days; protocol doc updated | `cargo test -p game2-core --lib sound_facts` |
| P2 | Project `LogEntryV.tag` | — | Every log line carries its `EvTag`; older payloads still decode | Same, plus `protocol_doc` |
| P3 | Project `panel.drama: DramaV`; two-world no-leak test | — | Phase, next, live, last and pressure are correct in staged arcs; hidden state never changes `DramaV` | `cargo test -p game2-core --lib drama` |
| A1 | Asset pipeline: `assets/sound/`, `LEDGER.json`, `scripts/sound_fetch.py` (Freesound CC0 search + HQ preview), `scripts/sound_process.py` (ffmpeg); license-gate test | S1 | The gate fails on an unledgered or non-CC0 file | `cargo test -p game2-audio ledger` |
| A2 | Sample loading: `lewton` OGG decode; bank from the filesystem (desktop) or fetched bytes (web); missing-asset fallback | A1 | Loads a test OGG on both; a missing file logs once and plays silence | `cargo test -p game2-audio sampler` |
| A3 | `soundscape::scan`, a pure function (listener, radius, emitter rules table) | P1 | The soundscape rows in the testing table pass | `cargo test -p game2-audio soundscape` |
| A4 | Ambience mixer plus the first asset batch (7 beds, 10 one-shots) | A2, A3 | Mix tests pass for the 4 place scenarios; ledger complete | `sound_dump --scenario apiary_spring` and Read the spectrogram |
| A5 | New sim cues (`alarm`, `toll`, `cheer`, `chant`, `coins`, `quill`, `door`, `cart_arrive`, `gate`) and palette mapping | A2 | Each cue fires in its subsystem; one test per cue in `session/tests/sfx.rs` | `cargo test -p game2-core --lib sfx` |
| M1 | Theory core: modes, key signatures, triads, tintinnabuli, motifs and transforms, Euclidean rhythms, instrument ranges | — | Theory tests pass | `cargo test -p game2-audio theory` |
| M2 | Instruments: sampled patches (VCSL, VSCO 2, FreePats) and synths (Karplus–Strong lute, hurdy-gurdy, choir pad, bells) | A2, M1 | Each patch renders across its range without NaN or clipping; pitch within ±10 cents | `cargo test -p game2-audio instruments` |
| M3 | Clock and sequencer: tempo, meter, bar, beat, phrase; sample-accurate events; quantized change requests | S1 | Event times are exact to the sample; changes wait for the boundary | `cargo test -p game2-audio clock` |
| M4 | Composer: phrase grammar, 8 voices, motif library, arranger (8-bar rule) | M1, M2, M3 | Composer property tests pass | `cargo test -p game2-audio compose` |
| M5 | Director: mood vector, sections, anticipatory build, verdict-to-mode, stingers, presence, ducking | M4, P1, P2, P3 | Director scenario tests pass; the timeline PNG shows the build peaking on the event | `cargo test -p game2-audio director`; `sound_dump --scenario assembly_build` |
| M6 | Retire the sim-side `music.rs` evolution and `UiModel.music` | M5 | The old tests are replaced; protocol doc updated | `cargo test -p game2-core` |
| U1 | `AudioSettings` plus persistence; M toast and muted glyph | S4 | Round-trip and clamp tests; mute renders zeros | `cargo test -p game2-audio settings` |
| U2 | Desktop Sound overlay off the pause menu | U1 | Draw and hit tests pass; `GAME2_DUMP_MENU=sound` dump looks right | `cargo test -p game2-client --lib sound_menu`; dump-frame and Read |
| U3 | Web Sound dialog | U1, S5 | Sliders change the live mix; values survive a reload | Manual browser check |
| U4 | Sound captions (optional) | A4, U1 | Captions appear for signals within the radius | Draw test |
| I1 | Scenario library, golden traces, mix tests | M5, A4, S3 | 10 golden traces committed; mix tests green | `cargo test -p game2-audio --release golden`, then the same with `mix` |
| I2 | 400-day soak | I1 | Soak thresholds met | `cargo test -p game2-audio --release soak -- --ignored` |
| I3 | `record_video` uses the new engine | M5, A4 | An AVI of the walkabout carries the score and ambience | `record_video --format avi --audio` |
| I4 | Perf budget in `GAME2_BENCH` (`audio.*` sections) | S4, S5, M5 | ≤ 1 ms per 10 ms block on desktop; ≤ 2 ms per frame on web | `GAME2_BENCH=600 game_ui --dump-frame 600` |
| I5 | Listening pass (you) | I1 | Checklist done; issues filed | 5-minute reel from `sound_dump --reel` |

**What every ticket body will carry**, in the repo's ticket form, so an agent can run it without grilling:

- **Summary:** what changes and why, plus a "Decided — don't ask" line citing the decision numbers above.
- **Acceptance criteria:** exact files, type and function signatures, constants, and the test names to add.
- **How to verify:** the exact commands above, plus which PNG to open and what it should show.
- **Player / agent surface:** "sim only", "client only", or the new control.
- **Depends on:** the IDs above, as issue numbers.

Each dependency chain shares one `queue-N` label, as the ticket loop expects: S1–S3–I1–I2; S4–S5–I4; A1–A2–A4–A5; M2–M4–M5–M6–I3; U1–U4. P1, P2, P3, A3, M1, M3 and S0 each get their own queue.

## Filed tickets

Filed 2026-11-23 on GitHub, label `audio`; each body carries its own acceptance criteria and verify steps.

| Plan ID | Issue |
|---|---|
| S0 | #1485 |
| S1 | #1486 |
| S2 | #1491 |
| S3 | #1493 |
| S4 | #1494 |
| S5 | #1497 |
| P1 | #1487 |
| P2 | #1488 |
| P3 | #1489 |
| A1 | #1495 |
| A2 | #1498 |
| A3 | #1496 |
| A4 | #1500 |
| A5 | #1501 |
| M1 | #1490 |
| M2 | #1499 |
| M3 | #1492 |
| M4 | #1502 |
| M5 | #1503 |
| M6 | #1504 |
| U1 | #1505 |
| U2 | #1506 |
| U3 | #1507 |
| U4 | #1508 |
| I1 | #1509 |
| I2 | #1510 |
| I3 | #1511 |
| I4 | #1512 |
| I5 | #1513 |

## Risks and open questions

The biggest risk is generated melody that sounds aimless. The plan counters it with authored motifs, a strict phrase grammar, a silence budget, and your listening pass (I5) before any polish work.

| Risk | Mitigation | Fallback |
|---|---|---|
| Generated melody sounds aimless or samey | Authored motifs; estampie grammar; one change per phrase; presence silences; an anti-repeat memory like `DialogueMemory` | Hand-written 4-bar cells per section, resequenced horizontally |
| The synth lute or hurdy-gurdy sounds cheap | Body-resonance filters on Karplus–Strong; listening check in I5 | Record your own and dedicate it CC0, or find a CC0 sampler |
| Web audio crackles on the main thread | 100 ms buffer (fine for music); render budget I4 | cpal's AudioWorklet backend, which needs atomics and cross-origin-isolation headers from the k8s server |
| Floating point differs across machines | Golden traces, not PCM hashes, are the cross-machine check | — |
| Web download size | OGG q5, 20 MB cap, ledger totals checked in CI | Lazy-load beds after the first frame |
| Agents can't hear | Spectrogram, piano roll, timeline and metrics | Your listening pass is mandatory |
| Music leaks hidden state | Rule 1 (`UiModel` only) and the P3 no-leak test | — |

**Open questions** (Gary approved the plan without changing these, so each default stands until he says otherwise):

1. Approve or veto decisions 1–12 in the summary table.
2. Which repo do tickets go on? Resolved: `garysieling/medieval_organizer_game` (the ticket-loop skill's `garysieling/game2` is the pre-rename name).
3. Chant: is a synthesized "ooh" pad acceptable, or will you record a few sung drones and dedicate them CC0? Default: the synth pad.
4. Should the score react to the villager you have selected (their motif, their mood)? Default: no, except the grief motif for people the seat knew.
5. Tonic picked by seed from D, G or A? Default: yes.
6. Will you create a Freesound API key and set `$FREESOUND_API_KEY` where the agents run? Without one, A1 ships the pipeline and you fetch the files.
7. Is it fine for `DramaV` to show the campaign phase to players? It isn't in the panel today. Default: yes, and the panel shows it too.
8. Is the 20 MB audio budget right? Default: yes.

## Sources

The repo was read at `main` c5e1d60 on 2026-11-23. I opened the FreePats and VCSL pages in full and listed VCSL's folders from its git tree. The other sources were read as search excerpts, so re-check any licence page before committing a file from it.

- **Sample licences:** VCSL · VSCO 2 CE · FreePats bagpipe · sfzinstruments: folk · Univ. of Iowa MIS · Kenney UI Audio · Freesound API overview · Freesound authentication · Freesound CC0 filter syntax · Sonniss licence discussion · Pixabay terms · BBC RemArc (TechRadar)
- **Rust audio:** kira · cpal · rustysynth · ebur128 · Karplus–Strong (Stanford)
- **Techno structure:** Techno arrangement rules · Beat Kitchen: arrangement and energy · Layering as form
- **Film and contemporary classical:** Britannica: Shore's LOTR scores · The Conversation: tension in film scores · Reich, Music as a Gradual Process · Tintinnabuli formalized (arXiv)
- **Medieval music:** Estampie · Early medieval dance music · Mode and ethos (SAGE)
- **Adaptive game music:** Berklee on adaptive layering and stingers · iMUSE · Adaptive music in Kingdom Come: Deliverance · Mini Metro's programmed music · No Man's Sky music analysis · The AI systems of Left 4 Dead
- **Soundscape:** Schafer's terms (OpenLearn) · Corbin, Village Bells · Eon Altar ambience system
- **Loudness:** Audiokinetic: mastering a game · ITU-R BS.2434 · EBU R 128
