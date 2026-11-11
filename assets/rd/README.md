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
- **Memorials** (`monuments/`, #703): wayside cross, picture stone,
  obelisk, trophy and the Scott column (`rd_tile__tile_object` 96px for
  the ground pieces, `rd_pro__isometric` 96px for the column), sliced
  tight - the exports carried no ground slab to strip; the cairns are composed offline from
  `stones/stone*.png`, not generated. Registered as `Art.memorial_*`
  options - #702 picks which form a monument wears. The lychgate
  (`bits/lychgate1..2.png`, #674) stands at the churchyard's entrance,
  picked per tile like the other props.
- **Tombstones** (`bits/tomb1..3.png`, #673): upright grave markers for
  the churchyard's buried, resampled to ~20px via `rd_slice.py name@H`;
  the flat `graves` marks still serve the lying-in.
- **Botháns** (`buildings/bothan1..3.png` + `bothan-winter1..3.png`,
  #708): squat Irish thatch cottages appended to the cottage rotation.
  One 4-image call for summer picks, then a winter call per pick with
  the summer sprite as `reference_images`, so each snowy version keeps
  its design; `Art::load` overrides the recoloured winter look with the
  generated one where it exists.
- **Ground:** wheat and corn fields (soil + wheat + green-crop blocks) and
  the whole winter set (snow + ice blocks; roads and rivers reuse dirt and
  water) composed like the green set. Blocks with crops or snow above the
  face are measured at their widest row and taken as 2:1.

Still pack art after this pass: windmill, church tower, walls,
guildhall/blacksmith, rival town, traveller, trees, stones.

## Props and the town wall (2026-11-03, $4.32)

All `rd_pro__isometric` 64px with `buildings/cottage1.png` as
`reference_images` (the `props` group in generate_art_prompts.py), except
the stones sheet. Drawn at 1x like the cottages and figures, not at the
pack's block scale (`TileSet.trees_k`, `Art.stones_k`, `Art.castle_rd`).

- **Trees** (`trees/tree1..8`: 4 broadleaf, 4 fir; `trees/winter/tree1..4`:
  2 bare, 2 snowy fir). Autumn is the green set recoloured, as before. The
  firs came on a little dirt patch: `rd_slice.py --strip-base` takes it off.
- **Stones** (`stones/stone1..12` boulders and piles, `ground1..6` flat
  scatters) from one `rd_pro__spritesheet` ($0.18). The heavily mossy
  boulders in the sheet (rows 2-3) are left out: they read as bushes on
  rock and snow. The first export missed the grass slab under each piece
  (visible on snow and rock); `rd_slice.py --strip-fringe` takes it off
  without `--strip-slab`'s bottom-only, one-piece crop, which would eat
  most of a flat scatter's separate stones.
- **Town wall** (`castle/`): one wall block, gatehouse and postern
  (the wall call's second image has a door), exported again with `--mirror`
  for the other axis (`_x` / `_y`), and 4 towers (one per map, like the
  pack's). `--flush-base --strip-slab` takes off their grass slab so they
  stand on snow and rock without a green fringe; they draw at their own
  size centred on the tile (`draw_standing`), so a run of blocks joins up.
  The separate `castle-postern` call wasn't used (a different stone).
- **Church** (`buildings/church1..2`): drawn like the cottages
  (`draw_on_tile`, slab kept).

Screenshots: `evidence/rd-props/` (before | after). `--focus` also takes
`tree`, `stone`, `wall`, `gate` and `church` now.

Still pack art after this pass: windmill, guildhall/blacksmith, rival town, traveller.

## The rest of the pack (2026-11-03, $3.59)

Same recipe as the props (`props` and `anim/walk` groups, round 4).

- **Ground tufts** (`tufts/tuft1..14`, `tufts/winter/tuft1..12`): two
  `rd_pro__spritesheet` calls (green tufts/flowers/ferns, snowy tufts and
  dead reeds), resampled `@14` and drawn at 1x (`Art.details_k`); autumn is
  the green set gilded, as before.
- **Blacksmith, guildhall** (`buildings/smithy1..2`, `guildhall1..2`) and
  **the rival village** (`buildings/rival-house1..2`, `rival-woodhouse1..2`):
  64px isometric with the cottage as reference, `--flush-base`, drawn like
  the cottages. Each gets a snowed-over winter look (`Art.trades_winter`).
- **Windmill** (`windmill/tower.png` + `sails1..4`): a 96px tower with no sails (at 64px it came out cottage-sized)
  (`--flush-base`, drawn at its own size by `draw_standing`) and one cross
  of sails (`rd_pro__default`), rotated 0/22.5/45/67.5 degrees and box
  resampled to 58px for the turning frames; they turn on `MILL_HUB_RD`.
  `windmill/tower2.png` is the spare tower.
- **Rivals and travellers** (`rivals/{man,worker,queen}.png`,
  `traveller/courier.png`): raw four_angle_walking sheets. They face and
  stride as they move (`villager_poses` tracks them too). The walking
  style only draws people: "courier riding a horse" (and a second,
  horse-first prompt) came back as a hooded walker, so the courier and the
  other travellers now go on foot.

`--focus` also takes `smithy`, `guildhall`, `mill` and `rivals`.
Screenshots: `evidence/rd-round4/`.

Still pack art: no sprite on the map (the ground blocks keep the pack's geometry). Its figures (`player_s`,
`priest_s`, `traveller_s`, rival stills) and buildings remain only as
fallbacks when `assets/rd` is missing.

## Workplaces, market, bridge and the small things (2026-11-03, $3.42)

Round 5 (`props` group, notes "round 5"): every 64px piece uses
`buildings/cottage1.png` as reference.

- **Workplaces** (`buildings/{tavern,bakery,granary,watch-house,fold,
  apiary,cheese-cellar,workshop}{1,2}.png`, `--flush-base`): shops used
  to wear a random cottage. `WorkLook::of(BuildingKind)` now picks the
  look at setup (`BSprite::Work(look, i)`): taverner/alewife/brewer get a
  tavern, the baker a bakery, the shepherd's fold, the apiary, the cheese
  cellar and the watch post their own, any other shop the open-fronted
  workshop. The farmhouse and the smithy keep theirs. Each look snows
  over in winter like the cottages. With no art it falls back to the cottage.
- **Small things** (`bits/`): one `rd_pro__spritesheet` sliced into
  `skep1..4@14`, `herb1..4@10`, `shroom1..4@9`, `grave1..6@16`,
  `fire1..4@17` (the flames flicker between them) and `burnt1..2@8`. They
  replace the coloured-rectangle skeps, herb tufts, mushrooms, gravestones,
  flames and scorch marks; drawn at 1x by `draw_standing`.
- **Market** (`bits/market-cross.png@30`, `maypole.png@46` with
  `--flush-base --strip-slab`, `stall1..2@34`, `trader-stall@38`): the
  cross with its notice board stands on the market tile, swapped for the
  maypole on a feast day (`UiModel.feast`). On market day each seller who
  reaches their pitch gets a stall between them and the square
  (`Game::stalls_standing`, `UiModel.stalls`); the pedlar's pitch gets
  the striped booth. Use `--gap 8` when slicing: the cross and its board are
  separate blobs.
- **Bridge** (`bits/bridge_y.png`, `bridge_x.png` = `--mirror`,
  `@62x53`): bridge tiles used to draw as plain river. The generated
  deck runs along y (bottom-left to top-right on screen); the view picks
  the axis that crosses the water.

`--focus` also takes `market`, `bridge` and a workplace look (`tavern`,
`fold`, `apiary`, `cheese`, `watch`, `workshop`...). `GAME2_DUMP_FRAMES=N`
plays N frames into the dumped day, so stalls and congregations show
(e.g. `--dump-frame 110 --seed 7` + 100 frames: two stalls).
Screenshots: `evidence/rd-round5/`.

## P0 buildings and shops (2026-11-08, $1.80 fresh + reuse)

Issue #688: the P0 catalog's 20 prompts (`generate_art_prompts.py`
buildings/buildings-shops/colors groups). Ten subjects already had paid
output under older ids - reused, not regenerated (old raw dir -> catalog
id): `church-keep` -> `church`, `watch-house` -> `watch-post`,
`windmill-tall`+`windmill-sails` -> `mill-tower`+`mill-sails`,
`granary` -> `granary`, `trader-stall` -> `trader-stall`,
`guildhall` -> `guildhall`, `market-stall` -> `market-stands`,
`bakery` -> `shop-bakery`, `smithy` -> `shop-smithy`. All ten catalog ids
are marked done in `generated.json`; the reuse map is in the PR.

- **Palette** (`raw/palette-english-countryside-860d3b3d`, rd_pro__simple
  128px): the countryside swatch chart, kept as the reference for later
  passes; the pipeline has no `input_palette` field, so fresh calls ran
  plain `rd_pro__isometric`.
- **New shops** (`buildings/{tailor,glazier,printshop,masons-yard,
  luthier,farmhouse}1.png`, `@58 --flush-base`): `WorkLook` grew six
  variants and `WorkLook::of` maps Tailor/Glazier/Printmaker/Mason/
  Luthier/Farmer shops to them (the smithy keeps `BSprite::Blacksmith`,
  other trades keep the workshop). One variant each; winter snow comes
  from `works_winter` like the rest.
- **Commons** (`bits/well.png@40`, `bits/meeting-hall.png@54`): the
  village well and meeting hall stand on the first clear grass tiles
  beside the market square (`Game::well_tile`/`hall_tile`, picked at
  settle like the stocks'), drawn as `UiModel.well`/`hall` props.
- **Smoke** (`bits/smoke1.png@20`): the generated wisp cycles through
  the same three-slot rise the drawn puffs used; no file, drawn puffs.

`--focus` also takes the new look names (`tailor`, `glazier`, ...).
Screenshots: `evidence/rd-p0-buildings/`.
