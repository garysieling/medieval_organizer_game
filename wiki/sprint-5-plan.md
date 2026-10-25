# Sprint 5: "It Looks Like a Game"

**Theme:** Full UI build-out — when you open the game, you see a real isometric medieval village with sprites, text, panels, and atmosphere. Not a prototype. A game.

**Status:** Planned

## Asset Base (already in repo)

- **Isometric Medieval Pack**: buildings (houses ×4 colors, blacksmiths, corn, wheat, mill), castle towers ×4 colors, environment (trees/stones ×3 seasons, desert), rivers ×3 seasons, roads ×3 seasons
- **Minifolks Humans**: KingMan, Queen, NobleMan/Woman, Prince, Princess, Peasant, OldMan/Woman, Worker, Mage, soldiers (Sword/Spear/Halberd/Archer/CrossBow/Cavalier/Horse)
- **Minifolks Villagers**: 9 villager sprites ×2 styles (outline/flat)
- **315 PNG files total**

## Workstream A: Text & Fonts (CRITICAL — UI needs text)

- [ ] A1: PNG decoding — add `png` or `image` crate, decode PNGs to RGBA buffers
- [ ] A2: Bitmap font renderer — bake a 5x7 or 8x8 pixel font into code, `draw_text(buf, x, y, str, color)`
- [ ] A3: Text everywhere — buttons labeled, status readouts, villager names, tick counter

## Workstream B: Sprite Pipeline

- [ ] B1: `Sprite` type — load PNG → RGBA, store as Buffer, `blit()` onto framebuffer
- [ ] B2: Asset manifest — `assets.rs` maps roles → sprite paths (Peasant→MiniPeasant.png, Noble→MiniNobleMan.png...)
- [ ] B3: Villagers render as actual Minifolks sprites (not dots), selected target gets outline variant
- [ ] B4: Buildings render as isometric building sprites on tiles (houses, blacksmith, mill)
- [ ] B5: Environment sprites — trees, stones scattered on grass tiles by season

## Workstream C: Map 2.0

- [ ] C1: Tile-aligned placement — sprites snap to isometric grid correctly (anchor points, depth sort)
- [ ] C2: Buildings on map — village has houses/mill/market at fixed positions from world state
- [ ] C3: Seasonal tile colors — Spring green / Autumn amber / Winter snow-white, swap tile palette
- [ ] C4: Depth sorting — entities + buildings render back-to-front correctly (no overlap bugs)
- [ ] C5: Water animation — water tiles shimmer (2-frame palette cycle)

## Workstream D: Villager Life on Screen

- [ ] D1: Villagers walk — position interpolates smoothly between tiles (lerp, not teleport)
- [ ] D2: Activity indicators — icon over head (food bubble when hungry, Zzz when idle, ! when seeking)
- [ ] D3: Health bar — tiny bar over villagers when injured/sick
- [ ] D4: Death visual — corpse sprite or fade-out when a villager dies
- [ ] D5: Dog sprite on map — pet follows its owner

## Workstream E: Action Panel (the real control UI)

- [ ] E1: Buttons with real text labels, hover highlight, press animation
- [ ] E2: Disabled states — Recruit grayed out until Talk done; Build needs resources
- [ ] E3: Tooltips — hovering a button shows what it does + cost
- [ ] E4: Action feedback — result flashes on panel ("They joined!" / "They declined")
- [ ] E5: Keyboard shortcuts still work (T/R/I/O/B/W)

## Workstream F: Villager Inspector

- [ ] F1: Click a villager → inspector panel opens (name, sprite, stats)
- [ ] F2: Shows hunger/health bars, attachment style badge, current activity
- [ ] F3: Relationship line — how they feel about you (from Talk history)
- [ ] F4: Inspector closes on click-away or Esc

## Workstream G: HUD & Status

- [ ] G1: Top bar — date (season + day), resources, influence
- [ ] G2: Organization panel — member count, member sprites, charter
- [ ] G3: Speed controls — pause / play / 2x / 4x buttons
- [ ] G4: Win condition banner — "Recognized Leader" overlay when reached

## Workstream H: Event Log & Notifications

- [ ] H1: Scrolling text log panel — actual rendered text, last ~8 events
- [ ] H2: Color-coded — actions white, warnings yellow, events cyan, deaths red
- [ ] H3: Event popups — "Merchant Guild formed!" toast in map corner

## Workstream I: Camera & Navigation

- [ ] I1: Pan — arrow keys + edge-of-map margin scroll
- [ ] I2: Zoom — +/- or mouse wheel (scale tiles 0.5x–2x)
- [ ] I3: Minimap — corner panel, colored dots, click to jump

## Workstream J: Atmosphere

- [ ] J1: Weather overlays — rain streaks, snow, dark storm tint (from weather system)
- [ ] J2: Day/night tint — subtle brightness pulse over ticks
- [ ] J3: Screen transitions — fade between seasons/events

## Workstream K: Screens

- [ ] K1: Title screen — "THE WANDERER" + start button
- [ ] K2: Pause menu — resume/quit
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
