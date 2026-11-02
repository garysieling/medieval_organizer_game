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
