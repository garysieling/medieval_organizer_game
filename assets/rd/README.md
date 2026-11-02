# Retro Diffusion art

Sprites generated with https://retrodiffusion.ai/ for game2. Each folder
is sliced from one paid generation kept under `art-prompts/raw/<id>/`
(the raw sheet + `meta.json` with prompt, request and cost), so any
folder can be re-sliced or extended without paying again.

## Pipeline (one asset, end to end)

1. **Pick** an entry: `art-prompts/prompts.md` (built by
   `scripts/generate_art_prompts.py`). Sheets (`rd_pro__spritesheet`,
   `rd_pro__inventory_items`) are the best value: one $0.18 call gives
   a 7x7 grid.
2. **Generate**: `python scripts/rd_generate.py <id-or-prefix> --dry-run`
   (free price check), then without `--dry-run`. Token from
   `$RETRODIFFUSION_API_KEY` or `~/.config/retrodiffusion/token`.
   Writes `art-prompts/raw/<id>/` and records the id in
   `art-prompts/generated.json` (tracked: the shared "already paid" ledger).
3. **Slice**: `python scripts/rd_slice.py art-prompts/raw/<id>/0.png
   --preview <scratch>/p.png` to see cell names, then export with
   `--out assets/rd/<set> rRcC=name@H ...` (`@H` = target height in px).
4. **Wire** into `Art::load` (session.rs) with the old art as fallback.
5. **Check**: `game_ui --dump-frame 300 --seed 7 --zoom 2 --focus <kind>
   --out x.bmp` before and after (`--focus` centres on dog, cat, chicken,
   sheep, goat, guard, priest, ...).

### Scale

Pack art (Minifolks, Isometric Medieval Pack) is drawn 2x; a villager is
~24px tall on screen at zoom 1. RD sheet sprites come out ~25px on a 1px
grid. Halving them (`--half`) loses too much detail; box-resampling to a
target height (`@H`) and drawing at 1x reads best. Animals: 14-20px.

## farm-animals

From `sheet-farm-animals-a628136d` (rd_pro__spritesheet 256x256, $0.18,
2026-11-02). Exported with:

    python scripts/rd_slice.py art-prompts/raw/sheet-farm-animals-a628136d/0.png \
        --out assets/rd/farm-animals r0c0=dog@17 r1c0=cat@15 r2c0=hen@14 \
        r2c1=rooster@17 r4c0=sheep@18 r4c1=sheep2@18 r3c0=goat@20 r3c1=goat2@20

Unused cells in the same sheet: more dogs/cats, chick, nesting hen, kid
goats, lamb, pigs (row 5), cows (row 6), doghouse, coop, cart, feed
trough, sacks, hay, fence, milk pail.

## Scale tests (2026-11-02)

One call per replacement scenario, each requested near the size the game
draws it (`scale-tests` group in generate_art_prompts.py, ~$0.86 incl. one
retry). Screenshots: `evidence/rd-scale-tests/`.

| Scenario | Request | Result | Lesson |
|---|---|---|---|
| Sheet at native size | spritesheet 128x128 | Same ~25px sprites, fewer of them, clipped at the edges | RD's pixel size is fixed: canvas size = how many fit, not how big. Always ask 256; resample to size. Slicer now drops clipped cells. |
| Ground tile | isometric 64x64 | 1st try drew a cottage on the block; with `ISO_GROUND` prompt a clean block, face exactly 48x24 (2:1) | "village asset" in the prompt beats the subject. Exported onto the pack's block geometry (`grass@95x99`) the game draws it unchanged. Roads/rivers still carry the pack's grass, so tiles must be replaced as one set. |
| Building | isometric 64x64 | 56x59 cottage, same as the pack houses, sits right on its tile | Fits the footprint as-is, but brings its own grass base and much finer detail than the flat pack houses: replace buildings as a set. |
| Player + animation | four_angle_walking 48 | 192x192 sheet: rows up/right/down/left, 4 frames; transparent; figure ~35px | Wired as the player's walk cycle (`player_pose` in UiModel). 32px requests fail (`inference_failed`, not charged; check_cost doesn't validate sizes). Figure is ~35px vs ~24px villagers. |

Open decisions (proportions):
- Characters ~35px (RD walk native) means villagers grow to match, or the
  player is resampled down (~0.7x) and loses detail.
- Ground blocks: draw RD blocks at native geometry (change BLOCK_FACE /
  FACE_CENTRE) once roads and rivers are RD too, instead of mapping onto
  the pack's.

## Ground set + first villagers (2026-11-02)

**Ground (green seasons + autumn):** `tiles/` is built, not sliced:

    python scripts/rd_compose_tiles.py \
        --grass art-prompts/raw/scale-tile-grass-0b26af48/0.png \
        --dirt  art-prompts/raw/ground-set-dirt-0ff45e1d/0.png \
        --water art-prompts/raw/ground-set-water-dc517000/0.png \
        --out assets/rd/tiles

Three RD blocks (grass, plus dirt and water generated with the grass as
`reference_images`) repaint the pack's 1 grass + 15 road + 17 river
pieces: each pack pixel is classified by its flat palette colour and
filled from the matching RD block's top-face texture (diamond-to-diamond
mapping) or sides. The pieces keep the pack geometry, so tilemap.rs
connections and BLOCK_FACE / FACE_CENTRE are unchanged. The reference
image matched colours, not proportions, so the compositor measures each
block's face itself. 

**Villagers:** `villagers/peasant-m.png`, `peasant-f.png` are raw
four_angle_walking 48px sheets (rows up/right/down/left), the walk cycle
for villager looks 0 and 1 (about a quarter of villagers); they face and
stride as they move (`UiModel.poses`). Prompts for this style need
positive appearance details: "male peasant villager" gave a helmeted
soldier, "bareheaded ... no helmet" a bald man.

## Full village pass (2026-11-02, $3.11)

- **Characters:** all 8 villager looks (`villagers/{role}-{m|f}.png`,
  matched to `vpeople` in session.rs), the priest and the 4 gate guards
  (`guards/`) are walk sheets. Guards and priest stand facing out.
- **Cottages:** `buildings/cottage1..9.png` (the first test cottage plus
  two 4-image calls using it as `reference_images`) replace the pack's
  10 house looks. Their grass bases are recoloured to snow in winter
  (`Art.houses_winter`).
- **Ground:** wheat and corn fields (soil + wheat + green-crop blocks) and
  the whole winter set (snow + ice blocks; roads and rivers reuse dirt and
  water) composed like the green set. Blocks with crops or snow above the
  face are measured at their widest row and taken as 2:1.

Still pack art: windmill, church tower, walls, guildhall/blacksmith,
rival town, traveller, trees, stones.
