# Sprint 5 Summary - "It Looks Like a Game"

**Date:** 2026-10-25
**Status:** Complete
**Commits:** ~20 UI-focused commits on devin/creativity-generator

## What Was Built

A real, playable, windowed game. `cargo run --bin game_ui` opens a window with a living isometric medieval village — not a text sim, a game.

### Core UI Architecture
- **minifb window** (960x640): isometric map (left 640px) + action panel (right 300px)
- **Bitmap font** (`ui/font.rs`): 5x7 pixel font, `draw_text` + `draw_text_scaled`, no deps
- **Sprite pipeline** (`engine/sprite.rs`): PNG decode (png crate), `blit`, `blit_center`, `blit_scaled`, `crop` for spritesheet frames
- **Assets registry** (`Assets`): maps roles → sprite paths (peasant/worker/elder/noble, buildings by type+color)

### The Village (map)
- **Isometric continent**: water edges, green land center (fixed from all-water bug)
- **Buildings**: houses (blue/wood), animated windmill (4 frames), blacksmith, church (blue tower), guild hall (red house), castle towers + walls (the ruler overlooks)
- **Farms**: wheat fields + corn crops visible on the land
- **Roads** connecting the village center, market plaza
- **River** along the east edge, stones scattered
- **Trees** (seasonal variants), chimney smoke rising from houses

### Villagers (living)
- **Minifolks sprites** at 2x scale, name labels above heads
- **Activity icons**: `!` hungry, `o` seeking food, `*` working
- **Wander** when idle — the village moves
- **Death** → gravestone marker + RIP
- **Dog** follows owner, heart when bonded

### Player
- Sprite (highlighted peasant with gold ring), "YOU" label
- Builds visible houses via Build action

### Action Panel (control UI)
- 6 labeled buttons: Talk, Recruit, Invest, Organize, Build, Wait
- Key hints (T/R/I/O/B/W), hover highlight, tooltips, disabled states (Recruit grays until influence>=0.3)
- Click or keyboard shortcuts

### Panels
- **Inspector**: selected villager's sprite + stats (hunger, health, attachment style, activity) + Trust bar (rapport tracking)
- **Org panel**: member portrait sprites show who's joined
- **Event log**: color-coded (deaths red, guilds gold, joins green, weather cyan), scrolling
- **HUD**: title, influence bar, org/guild/gold, season banner

### Dynamics
- **Camera**: pan (arrows), zoom (+/-), minimap thumbnail + click-to-jump
- **Atmosphere**: day/night tint, rain overlay, season transition flash, water shimmer, ambient particles (snow in winter, dust otherwise)
- **Screens**: title screen, pause menu, win banner, game over summary
- **Toasts**: gold-bordered notifications for big events (guild formation)
- **Dialog bubbles**: talking to a villager shows their concern in a speech bubble
- **Food stock**: market shows remaining servings bar

### Controls
- Click buttons / keys T R I O B W for actions
- Click villager or 1-5 to select
- Arrows pan, +/- zoom, Space pause, 0 turbo 4x, H help, F12 screenshot, Esc quit

## Evidence
- `evidence/ui-preview.bmp` — full UI render
- `evidence/ui-winter.bmp` — winter scene with snow
- `evidence/sprite-test.bmp`, `sprite-crop.bmp` — sprite pipeline
- `screenshots/game-ui-live.png`, `game-ui-final.png` — live window captures

## Sprint 5 Tasks: 40/40 done
