# Sprint 5: "It Looks Like a Game"

**Theme:** Full UI build-out — when you open the game, you see a real isometric medieval village with sprites, text, panels, and atmosphere. Not a prototype. A game.

**Status:** In Progress - most workstreams done, F3/G2/J3/K3/C5/D4 remain

## Asset Base (already in repo)

- **Isometric Medieval Pack**: buildings (houses ×4 colors, blacksmiths, corn, wheat, mill), castle towers ×4 colors, environment (trees/stones ×3 seasons, desert), rivers ×3 seasons, roads ×3 seasons
- **Minifolks Humans**: KingMan, Queen, NobleMan/Woman, Prince, Princess, Peasant, OldMan/Woman, Worker, Mage, soldiers (Sword/Spear/Halberd/Archer/CrossBow/Cavalier/Horse)
- **Minifolks Villagers**: 9 villager sprites ×2 styles (outline/flat)
- **315 PNG files total**

## Workstream A: Text & Fonts (CRITICAL — UI needs text)

- [x] A1: PNG decoding — add `png` or `image` crate, decode PNGs to RGBA buffers
- [x] A2: Bitmap font renderer — bake a 5x7 or 8x8 pixel font into code, `draw_text(buf, x, y, str, color)`
- [x] A3: Text everywhere — buttons labeled, status readouts, villager names, tick counter

## Workstream B: Sprite Pipeline

- [x] B1: `Sprite` type — load PNG → RGBA, store as Buffer, `blit()` onto framebuffer
- [x] B2: Asset manifest — `assets.rs` maps roles → sprite paths (Peasant→MiniPeasant.png, Noble→MiniNobleMan.png...)
- [x] B3: Villagers render as actual Minifolks sprites (not dots), selected target gets outline variant
- [x] B4: Buildings render as isometric building sprites on tiles (houses, blacksmith, mill)
- [x] B5: Environment sprites — trees, stones scattered on grass tiles by season

## Workstream C: Map 2.0

- [ ] C1: Tile-aligned placement — sprites snap to isometric grid correctly (anchor points, depth sort)
- [x] C2: Buildings on map — village has houses/mill/market at fixed positions from world state
- [x] C3: Seasonal tile colors — Spring green / Autumn amber / Winter snow-white, swap tile palette
- [x] C4: Depth sorting — entities + buildings render back-to-front correctly (no overlap bugs)
- [ ] C5: Water animation — water tiles shimmer (2-frame palette cycle)

## Workstream D: Villager Life on Screen

- [x] D1: Villagers walk — position interpolates smoothly between tiles (lerp, not teleport)
- [x] D2: Activity indicators — icon over head (food bubble when hungry, Zzz when idle, ! when seeking)
- [ ] D3: Health bar — tiny bar over villagers when injured/sick
- [ ] D4: Death visual — corpse sprite or fade-out when a villager dies
- [x] D5: Dog sprite on map — pet follows its owner

## Workstream E: Action Panel (the real control UI)

- [x] E1: Buttons with real text labels, hover highlight, press animation
- [x] E2: Disabled states — Recruit grayed out until Talk done; Build needs resources
- [x] E3: Tooltips — hovering a button shows what it does + cost
- [x] E4: Action feedback — result flashes on panel ("They joined!" / "They declined")
- [x] E5: Keyboard shortcuts still work (T/R/I/O/B/W)

## Workstream F: Villager Inspector

- [x] F1: Click a villager → inspector panel opens (name, sprite, stats)
- [x] F2: Shows hunger/health bars, attachment style badge, current activity
- [ ] F3: Relationship line — how they feel about you (from Talk history)
- [x] F4: Inspector closes on click-away or Esc

## Workstream G: HUD & Status

- [x] G1: Top bar — date (season + day), resources, influence
- [ ] G2: Organization panel — member count, member sprites, charter
- [x] G3: Speed controls — pause / play / 2x / 4x buttons
- [x] G4: Win condition banner — "Recognized Leader" overlay when reached

## Workstream H: Event Log & Notifications

- [x] H1: Scrolling text log panel — actual rendered text, last ~8 events
- [x] H2: Color-coded — actions white, warnings yellow, events cyan, deaths red
- [x] H3: Event popups — "Merchant Guild formed!" toast in map corner

## Workstream I: Camera & Navigation

- [x] I1: Pan — arrow keys + edge-of-map margin scroll
- [x] I2: Zoom — +/- or mouse wheel (scale tiles 0.5x–2x)
- [x] I3: Minimap — corner panel, colored dots, click to jump

## Workstream J: Atmosphere

- [x] J1: Weather overlays — rain streaks, snow, dark storm tint (from weather system)
- [x] J2: Day/night tint — subtle brightness pulse over ticks
- [ ] J3: Screen transitions — fade between seasons/events

## Workstream K: Screens

- [x] K1: Title screen — "THE WANDERER" + start button
- [x] K2: Pause menu — resume/quit
- [ ] K3: Game over / win screen — summary of what you built

## Workstream L: Evidence & Integration

- [ ] L1: Frame dumps — UI saves BMP screenshots on keypress (F12)
- [ ] L2: Update PLAY.md — controls, screenshots
- [ ] L3: Record full session → frame sequence evidence

## Exit Criteria

1. Open `cargo run --bin game_ui` → see a real village: sprite villagers walking on an isometric continent, buildings, trees, panel with labeled buttons
2. Click Talk on a villager → inspector shows their state, log updates with text
3. Seasons change the map colors; weather overlays appear
4. Camera pans/zooms; minimap works
5. A full play session looks like a game, feels like Caesar/settlement sim

## Sprint 5 Task Count: ~40 tasks

**Priority order:** A (text) → B (sprites) → C (map) → E (actions) → F (inspector) → G (HUD) → D (villager life) → H (log) → I (camera) → J (atmosphere) → K (screens) → L (evidence)
