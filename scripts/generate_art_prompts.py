#!/usr/bin/env python3
"""Build the Retro Diffusion prompt manifest for game2's art pass.

Walks the game's asset taxonomy (seasons, biomes, tilesets, buildings,
people, festivals, cultures, icons, matched sheets, future features) and
emits a flat list
of generation prompts for https://retrodiffusion.ai/, with per-prompt
request settings, cost estimates and stable ids.

Outputs:
  art-prompts/prompts.json   machine manifest, one entry per API call
  art-prompts/prompts.md     human list, grouped by priority then group

De-dup:
  Every entry has a stable id (slug + hash of the request). Pass a state
  file with --state listing ids already generated; those are skipped.
  --record writes the emitted ids into the state file, so the workflow is:

    python scripts/generate_art_prompts.py --limit 100 --record
    # ...generate those 100 on retrodiffusion.ai...
    python scripts/generate_art_prompts.py --limit 100 --record   # next 100

Smoke test first: `--smoke` emits one curated call per rendering mode
(~$3) to validate the pipeline before any real spend.

Stdlib only. Run from the repo root.
"""
import argparse
import hashlib
import json
import os
import sys
from collections import OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(ROOT, "art-prompts")
STATE = os.path.join(OUT_DIR, "generated.json")
CULTURES_TSV = os.path.join(ROOT, "wiki", "creativity-corpus", "sources.tsv")

# ---------------------------------------------------------------------------
# Retro Diffusion reference (api-examples README, v2 API).
# Price per image by style family; animations/tilesets are per call.
PRICE = {
    "rd_fast": 0.03, "rd_plus": 0.06, "rd_mini": 0.06, "rd_pro": 0.18,
    "rd_tile__single_tile": 0.06, "rd_tile__tile_variation": 0.06,
    "rd_tile__tile_object": 0.06, "rd_tile__scene_object": 0.06,
    "rd_tile__tileset": 0.10, "rd_tile__tileset_advanced": 0.10,
    "rd_pro__fps_weapon": 0.25,
    "rd_animation__battle_sprites": 0.07,
    "rd_animation__four_angle_walking": 0.07,
    "rd_animation__four_angle_walking_idle": 0.07,
    "rd_animation__small_sprites": 0.07,
    "rd_animation__vfx": 0.07,
    "rd_animation__any_animation": 0.25,
    "rd_animation__big_animation": 0.25,
    "rd_animation__8_dir_rotation": 0.25,
    "rd_advanced_animation__walking": 0.14,
    "rd_advanced_animation__idle": 0.14,
    "rd_advanced_animation__jump": 0.14,
    "rd_advanced_animation__crouch": 0.14,
    "rd_advanced_animation__attack": 0.14,
    "rd_advanced_animation__destroy": 0.14,
    "rd_advanced_animation__rotate": 0.10,
    "rd_advanced_animation__custom_action": 0.25,
    "rd_advanced_animation__subtle_motion": 0.25,
}


def cost(style, n):
    if style in PRICE:
        return PRICE[style] * n
    return PRICE[style.split("__")[0]] * n


TIER_RANK = {"p0": 0, "p1": 1, "p2": 2, "anim": 3}
TIER_NAMES = {
    "p0": "P0 - needed now (replaces assets the game already draws)",
    "p1": "P1 - fills gaps in shipped features",
    "p2": "P2 - future features and cultures",
    "anim": "ANIMATED - premium tier, all animation styles",
}

ENTRIES = []


def add(group, slug, prompt, style="rd_pro__isometric", w=128, h=128, n=1,
        tier="p0", remove_bg=True, tile_x=False, tile_y=False,
        frames=None, sheet=False, notes=None):
    """One prompt entry -> one POST /v2/inferences call."""
    req = {"prompt_style": style, "width": w, "height": h, "num_images": n}
    if remove_bg:
        req["remove_bg"] = True
    if tile_x:
        req["tile_x"] = True
    if tile_y:
        req["tile_y"] = True
    if frames:
        req["frames_duration"] = frames
    if sheet:
        req["return_spritesheet"] = True
    if style.startswith("rd_animation") or style.startswith("rd_advanced_animation"):
        tier = "anim"
    # stable id: readable slug + hash of what actually gets sent
    digest = hashlib.sha1(
        json.dumps({"group": group, "slug": slug, "prompt": prompt, "req": req},
                   sort_keys=True).encode()).hexdigest()[:8]
    ENTRIES.append(OrderedDict([
        ("id", f"{slug}-{digest}"),
        ("tier", tier),
        ("group", group),
        ("title", slug.replace("-", " ")),
        ("prompt", prompt),
        ("request", req),
        ("est_cost_usd", round(cost(style, n), 3)),
        ("notes", notes or ""),
    ]))


# ---------------------------------------------------------------------------
# Shared phrasing. Prompts describe the SUBJECT only - the prompt_style
# carries the pixel-art look (per RD docs: don't waste tokens on "pixel art").
ISO = "single isometric medieval village asset"
SPRITE = "small chibi medieval character, front view, full body, on empty background"

# Appearance variety: the cast should read as a real mix of people, not one
# face pasted everywhere. Every base villager gets 3 looks spread across the
# list; vocations and town roles cycle through it so the whole set is diverse.
APPEARANCES = [
    ("fair", "fair skin and light brown hair"),
    ("pale", "pale freckled skin and red hair"),
    ("olive", "olive skin and dark wavy hair"),
    ("tan", "warm tan skin and dark hair"),
    ("brown", "brown skin and coiled black hair"),
    ("deep", "deep dark skin and short black hair"),
]

# Scenes and festivals draw crowds - say so, or they come out monochrome.
CROWD = "villagers of varied ages and skin tones"

SEASONS = [
    ("spring", "fresh green spring, small flowers, new leaves"),
    ("summer", "lush deep-green summer, dense foliage"),
    ("autumn", "golden autumn, orange and amber foliage, fallen leaves"),
    ("winter", "snow-covered winter, bare branches, white drifts"),
]

BIOMES = [
    ("temperate", "temperate green meadowland"),
    ("desert", "desert sand and rock, oasis fringe"),
    ("highland", "rugged grey highland, heather and stone"),
    ("mediterranean", "mediterranean coast, dry grass and cypress"),
    ("boreal", "northern pine forest, moss and dark earth"),
]

# Festivals on the solstices/equinoxes (the backlog's astrology/sky events).
FESTIVALS = [
    ("spring-equinox", "spring equinox festival, maypole with ribbons, flower garlands, villagers dancing"),
    ("summer-solstice", "summer solstice festival, great bonfire, flower crowns, longest day celebration"),
    ("autumn-equinox", "autumn harvest festival, feast tables, wheat sheaves, apple baskets, lanterns"),
    ("winter-solstice", "winter solstice festival, yule log fire, evergreen wreaths, candle lanterns, snow"),
]

# ---------------------------------------------------------------------------
# P0: terrain and tilesets - what the map draws every frame.
# The pack's isometric ground blocks, one per Tile in tilemap.rs, per season.

for season, desc in SEASONS:
    for tile, subj in [
        ("grass", f"flat grassy ground block, {desc}"),
        ("shore", f"sandy shoreline ground block where land meets water, {desc}"),
        ("hill", f"raised earthen hill ground block with grass top, {desc}"),
        ("rock", f"bare grey rock ground block, craggy top, {desc}"),
    ]:
        add(f"tiles/ground-{season}", f"ground-{tile}-{season}",
            f"{ISO}: {subj}, square isometric ground block", w=128,
            tile_x=True, tile_y=True,
            notes="ground block like the Isometric Medieval Pack; tile_x+tile_y set so edges meet")
    for water, subj in [
        ("shallow", "shallow clear water ground block, shore edge visible"),
        ("deep", "deep open-sea water ground block, dark blue"),
    ]:
        add(f"tiles/ground-{season}", f"water-{water}-{season}",
            f"{ISO}: {subj}, {desc}, square isometric water block", w=128, tile_x=True, tile_y=True)

# Trees with real variety: species x season (the pack ships 4 spring + 4 winter).
TREES = [
    ("oak", "broad spreading oak tree"),
    ("birch", "slender white-barked birch tree"),
    ("pine", "tall evergreen pine tree"),
    ("fruit-tree", "small gnarled fruit tree in blossom"),
    ("dead-tree", "leafless twisted dead tree"),
]
for season, desc in SEASONS:
    for slug, tree in TREES:
        add(f"tiles/trees-{season}", f"tree-{slug}-{season}",
            f"{ISO}: {tree}, {desc}", style="rd_tile__tile_object", w=96, h=96, n=2,
            notes="num_images=2 gives two variants of the same species")

# Rocks, tufts, flowers, snow lumps - the detail specks on open ground.
for season, desc in SEASONS:
    for slug, prop in [
        ("boulder", "grey boulder, mossy"),
        ("stone-ground", "flat scatter of small stones on the ground"),
        ("tufts", "grass tufts and tiny wildflowers" if season != "winter" else "lumps of snow and frost-covered tufts"),
    ]:
        add(f"tiles/details-{season}", f"detail-{slug}-{season}",
            f"{ISO}: {prop}, {desc}", style="rd_tile__tile_object", w=64, h=64, n=4)

# Roads and rivers are wang tilesets: one call = the full edge-joined set.
for season, desc in SEASONS:
    add(f"tiles/wang-{season}", f"road-wang-{season}",
        f"dirt road tiles for a medieval village, wheel-rutted earth, {desc}",
        style="rd_tile__tileset", w=32, h=32, remove_bg=False,
        notes="wang tileset: 16px or 32px tiles, all edge combinations in one sheet")
    add(f"tiles/wang-{season}", f"river-wang-{season}",
        f"river water tiles with grassy banks, flowing stream, {desc}",
        style="rd_tile__tileset", w=32, h=32, remove_bg=False)

# Bridges across the river, both orientations.
for axis, axis_desc in (
    ("x", "deck running east-west along the x axis"),
    ("y", "deck running north-south along the y axis"),
):
    add("tiles/bridges", f"bridge-{axis}",
        f"{ISO}: wooden plank bridge crossing a river, arched, viewed isometric, {axis_desc}",
        w=128, notes=f"deck running along the {axis} axis")

# Crops at growth stages (wheat and corn are the farmed Tiles).
for crop, stages in [("wheat", ("green shoots", "ripe golden heads")),
                     ("corn", ("young green stalks", "tall ripe ears"))]:
    for i, stage in enumerate(stages, 1):
        add("tiles/crops", f"crop-{crop}-stage{i}",
            f"{ISO}: small field patch of {crop}, {stage}", w=96)

# Castle walls, gates, towers (the pack's 10 wall pieces + 4 tower looks).
add("tiles/walls", "castle-wall-set",
    "isometric medieval castle curtain wall pieces: straight runs along both axes, corner joins, crenellated grey stone",
    w=192, h=192, notes="request one wall set; slice into per-edge pieces like castle_wall(1..10)")
for slug, t in [
    ("gatehouse", "fortified gatehouse with open archway a road passes through"),
    ("postern", "wall section with a small postern door"),
    ("tower-wood", "square timber watchtower, flat top"),
    ("tower-stone", "round stone keep tower with conical roof"),
    ("tower-capped", "stone corner tower with crenellated cap"),
]:
    add("tiles/walls", f"wall-{slug}", f"{ISO}: {t}", w=128, h=128)

# ---------------------------------------------------------------------------
# P0: buildings - every BuildingKind the game places, plus shop per vocation.

add("buildings", "cottages-set",
    "isometric medieval cottages: timber-frame and plastered fronts, thatched and tiled roofs, varied door and gable positions",
    w=192, h=192, n=4, notes="batch of 4 distinct cottage variants; repeat calls for more variety")

SHOPS = [
    ("bakery", "bakery with brick oven chimney and bread sign"),
    ("smithy", "blacksmith smithy with forge, anvil outside, smoke"),
    ("tailor", "tailor's shop with cloth bolts displayed"),
    ("glazier", "glazier's workshop with glass panes in the window"),
    ("printshop", "print shop with a wooden press and posted bills"),
    ("masons-yard", "mason's yard with cut stone blocks and scaffolding"),
    ("luthier", "luthier's workshop, lutes hanging by the door"),
    ("farm", "farmhouse with fenced field and hay cart"),
]
for slug, b in SHOPS:
    add("buildings/shops", f"shop-{slug}", f"{ISO}: {b}", w=160, h=160)

for slug, b in [
    ("church", "stone village church with bell tower and graveyard"),
    ("watch-post", "watch house, small guard post with bell and spear rack"),
    ("mill-tower", "windmill tower with cap, sails separate"),
    ("mill-sails", "windmill sails, four lattice blades, standalone"),
    ("granary", "raised granary barn on staddle stones"),
    ("trader-stall", "open-air market stall with canvas awning and goods"),
    ("guildhall", "grand timber guildhall, two storeys, ornate gables"),
    ("meeting-hall", "village meeting hall with bench rows and hearth"),
    ("market-stands", "market square stands: crates, barrels, awnings"),
    ("well", "stone village well with little wooden roof and bucket"),
    ("smoke", "wispy chimney smoke puffs, three stages rising"),
]:
    add("buildings", slug, f"{ISO}: {b}", w=160, h=160)

# ---------------------------------------------------------------------------
# P0: people. Minifolks-style chibi sprites ~32-48px, front view, remove_bg.
# Roles x (m/f) as villager_sprite() knows them, plus children.

PEOPLE = [
    # (slug, subject) - base villagers
    ("peasant-m", "male peasant villager in rough brown tunic"),
    ("peasant-f", "female peasant villager in homespun dress and kerchief"),
    ("worker-m", "male labourer in rolled sleeves and apron"),
    ("worker-f", "female labourer in work dress and apron"),
    ("elder-m", "elderly man with white beard and walking stick"),
    ("elder-f", "elderly woman with grey bun and shawl"),
    ("child-m", "small boy child in short tunic"),
    ("child-f", "small girl child in short dress"),
    ("youth-m", "teenage boy in a loose tunic"),
    ("youth-f", "teenage girl with a long braid"),
    ("parent-infant", "young parent carrying a swaddled infant"),
    ("noble-m", "male noble in fur-trimmed cloak and fine hose"),
    ("noble-f", "female noble in long gown and headdress"),
    ("princess", "young princess in pale gown with small crown"),
    ("queen", "matriarch queen in rich robes and circlet"),
    ("player", "player character, young organizer in green tunic with satchel"),
]
# Vocations: every Vocation in vocation.rs, male and female.
VOCATIONS = [
    ("farmer", "farmer with straw hat, sickle at belt"),
    ("baker", "baker in flour-dusted apron holding a loaf"),
    ("toolmaker", "blacksmith toolmaker with leather apron and hammer"),
    ("knight", "knight-at-arms in mail with sword and surcoat"),
    ("tailor", "tailor with measuring tape and pins"),
    ("glazier", "glazier holding a leaded glass pane"),
    ("printmaker", "printmaker in ink-stained apron holding a pamphlet"),
    ("mason", "stonemason with mallet and chisel, dusty apron"),
    ("priest", "village priest in plain robes with symbol pendant"),
    ("luthier", "luthier holding an unfinished lute"),
]
VOCATION_PEOPLE = [(f"{slug}-{g}", f"{gword} {subj}")
                   for slug, subj in VOCATIONS for g, gword in (("m", "male"), ("f", "female"))]

# Town roles and special figures (TownRole enum + the minstrel + guards).
TOWN_ROLES = [
    ("overseer", "stern overseer with ledger and short whip, fine coat"),
    ("tax-collector", "tax collector in dark robes with coin purse and ledger"),
    ("trader", "travelling trader in foreign-looking bright robes at a stall"),
    ("minstrel", "travelling minstrel with a lute, colourful patched cloak, mid-performance"),
    ("courier", "road courier on horseback, satchel across chest"),
    ("guard-spear", "town guard with spear and open helmet"),
    ("guard-sword", "town guard with sword and buckler"),
    ("guard-shield", "town guard with large shield"),
    ("guard-halberd", "town guard with halberd"),
    ("archer", "archer with bow and quiver"),
    ("crossbowman", "crossbowman winding his weapon"),
    ("cavalier", "mounted knight on horseback with lance"),
    ("king", "old king in ermine cloak and crown"),
    ("prince", "young prince in doublet"),
    ("mage", "village hedge mage in robes with staff"),
]
def person(group):
    return lambda slug, subj: add(group, f"person-{slug}", f"{SPRITE}: {subj}",
        style="rd_pro__default", w=48, h=48,
        notes="use a saved reference_image of your base villager for look consistency")

# Every base villager in 3 looks, spread across APPEARANCES so the town can
# mix faces freely; vocations and roles each take one rotating look.
for i, (slug, subj) in enumerate(PEOPLE):
    for v in (i, i + 2, i + 4):
        app_slug, app = APPEARANCES[v % len(APPEARANCES)]
        person("people/villagers")(f"{slug}-{app_slug}", f"{subj}, {app}")
for i, (slug, subj) in enumerate(VOCATION_PEOPLE):
    _, app = APPEARANCES[i % len(APPEARANCES)]
    person("people/vocations")(slug, f"{subj}, {app}")
for i, (slug, subj) in enumerate(TOWN_ROLES):
    _, app = APPEARANCES[(i + 3) % len(APPEARANCES)]
    person("people/roles")(slug, f"{subj}, {app}")

# Pets (Species enum) - small critters on the map.
for slug, pet in [
    ("dog", "shaggy village dog sitting"),
    ("cat", "small tabby cat curled"),
    ("chicken", "brown hen pecking"),
    ("goat", "grey goat, bell on collar"),
    ("sheep", "woolly sheep grazing"),
    ("pig", "pink pig with mud patch"),
    ("cow", "brown dairy cow standing"),
]:
    tier = "p0" if slug in ("dog", "cat", "chicken") else "p2"
    add("people/animals", f"animal-{slug}", f"{ISO}: {pet}",
        style="rd_tile__tile_object", w=64, h=64, tier=tier)

# ---------------------------------------------------------------------------
# P0: icon set replacement. The whole UI is text glyphs today - replace with
# real icons: status marks, action buttons, meters, goods, weather, seasons.

ICONS_STATUS = [
    ("hungry", "empty bowl with spoon, warning feel"),
    ("finding-food", "basket with bread loaf"),
    ("busy", "hammer mid-swing"),
    ("sick", "green face with sweat drop"),
    ("dead", "small skull"),
    ("grieving", "wilted flower"),
    ("happy", "smiling sun face"),
    ("cold", "blue snowflake"),
    ("injured", "bandaged arm"),
]
ICONS_ACTIONS = [
    ("talk", "speech bubble"),
    ("recruit", "clasped handshake"),
    ("invest", "gold coins into a pot"),
    ("organize", "raised fist with banner"),
    ("build", "mallet and wooden beam"),
    ("wait", "hourglass"),
    ("inspect", "magnifying glass over a face"),
]
ICONS_METERS = [
    ("hunger", "bread loaf"),
    ("health", "red heart"),
    ("trust", "two joined hands"),
    ("mood", "sun behind small cloud"),
    ("faith", "lit candle"),
    ("power", "raised banner on pole"),
    ("gold", "gold coin stack"),
]
ICONS_WEATHER = [
    ("rain", "grey cloud with rain streaks"),
    ("heavy-rain", "dark storm cloud with heavy rain"),
    ("snow", "white cloud with snowflakes"),
    ("heavy-snow", "blizzard cloud"),
    ("sun", "bright sun"),
    ("overcast", "flat grey cloud"),
    ("rainbow", "rainbow arc over small cloud"),
    ("storm", "cloud with lightning bolt"),
    ("fog", "low fog bank swallowing a rooftop"),
    ("drizzle", "pale cloud with fine slanting drizzle"),
]
ICONS_SEASONS = [
    ("spring", "sprouting seedling with flower"),
    ("summer", "golden wheat sheaf and sun"),
    ("autumn", "orange maple leaf"),
    ("winter", "bare branch with snow"),
]
for slug, subj in ICONS_STATUS + ICONS_ACTIONS + ICONS_METERS + ICONS_WEATHER + ICONS_SEASONS:
    grp = ("icons/status" if (slug, subj) in ICONS_STATUS else
           "icons/actions" if (slug, subj) in ICONS_ACTIONS else
           "icons/meters" if (slug, subj) in ICONS_METERS else
           "icons/weather" if (slug, subj) in ICONS_WEATHER else "icons/seasons")
    add(grp, f"icon-{slug}", f"small game UI icon of {subj}, simple, readable at tiny size",
        style="rd_mini__skill_icon", w=32, h=32, n=4,
        notes="batch of 4 variants; keep the clearest")

# Goods icons: every tradeable good in vocation.rs, plus gold.
GOODS = [
    ("wheat", "wheat sheaf"), ("bread", "round bread loaf"),
    ("flour", "cloth sack of flour"), ("stone", "cut stone block"),
    ("wood", "stack of logs"), ("blocks", "dressed stone blocks"),
    ("sand", "small pile of sand"), ("paper", "stack of paper sheets"),
    ("ink", "ink pot with quill"), ("faith", "candle flame in holder"),
    ("pamphlets", "folded printed pamphlet"), ("pigment", "pots of coloured pigment"),
    ("cloth", "rolled bolt of cloth"), ("security", "small shield"),
    ("glass", "leaded glass pane"), ("garments", "folded clothes stack"),
    ("tools", "hammer and chisel crossed"), ("lute", "wooden lute"),
    ("gold", "pile of gold coins"),
]
for slug, subj in GOODS:
    add("icons/goods", f"good-{slug}", f"small game item icon of {subj}",
        style="rd_mini__topdown_item", w=32, h=32, n=2)

# One matched sheet per goods group - inventory_items returns an atlas JSON.
add("icons/goods", "goods-sheet",
    "inventory item sheet: wheat sheaf, bread loaf, flour sack, stone block, log stack, sand pile, paper, ink pot, pamphlet, pigment pots, cloth bolt, glass pane, garments, hammer tools, lute, gold coins",
    style="rd_pro__inventory_items", w=256, h=256, remove_bg=False,
    notes="one call draws a matched set; include_downloadable_data returns the item atlas")

# More atlases: each $0.18 call replaces a bought item pack or a dozen
# individual icon calls. 256x256 is the only size this style accepts.
add("icons/goods", "tools-sheet",
    "inventory item sheet: sickle, hoe, hammer, saw, chisel, axe, mallet, needle and thread, quill, trowel, smith's tongs, wood plane",
    style="rd_pro__inventory_items", w=256, h=256, remove_bg=False, tier="p1",
    notes="vocation workshop tools; include_downloadable_data returns the item atlas")
add("icons/goods", "food-sheet",
    "inventory item sheet: bread loaf, cheese wheel, ale mug, honey jar, dried fish, egg basket, milk jug, apples, pottage bowl, turnips, peas, fuel faggot bundle",
    style="rd_pro__inventory_items", w=256, h=256, remove_bg=False, tier="p1",
    notes="kitchen-garden and hearth foods, plus the fuel goods")
add("icons/goods", "materials-sheet",
    "inventory item sheet: log, dressed stone block, sand pile, pigment pot, cloth bolt, glass pane, paper sheet, ink pot, wool ball, tanned hide, candle, clay brick",
    style="rd_pro__inventory_items", w=256, h=256, remove_bg=False, tier="p1",
    notes="raw and crafted materials for the production chains")

# UI chrome: panels and window pieces. ui_panel returns a matched chrome set
# per call, so the kit-level prompts below cover a whole UI for ~$0.18 each.
for slug, subj in [
    ("panel", "carved wooden-framed parchment UI panel, empty centre"),
    ("button", "wooden game button, unpressed"),
    ("villager-card", "character card panel with portrait frame and stat rows"),
    ("log-box", "long thin parchment scroll log box"),
    ("ui-kit", "complete medieval game UI kit: buttons, sliders, panels, knobs, checkboxes, scroll bars, matching carved wood and parchment"),
    ("hud-bar", "game HUD strip: resource counters with icon slots, meter frames, season and date display"),
    ("window-set", "set of matching medieval game windows: title bar, dialog box, tooltip, modal frame"),
    ("inventory-grid", "empty inventory grid panel with item slots and a character paper-doll frame"),
    ("title-keyart", "sweeping isometric medieval valley town at golden hour, river and bridge, walled town, fields - title screen key art"),
    ("end-screen", "quiet dusk view of a medieval village, lamps lit, end-of-game mood"),
]:
    painterly = slug in ("title-keyart", "end-screen")
    add("icons/ui", f"ui-{slug}", subj + ("" if painterly else ", game UI element"),
        style="rd_pro__painterly" if painterly else "rd_pro__ui_panel",
        w=256, h=256, remove_bg=False, n=1,
        notes=None if painterly else "ui_panel draws a matched chrome set; slice per element")

# ---------------------------------------------------------------------------
# Matched sprite sheets: rd_pro__spritesheet returns a grid of same-style
# assets or poses on a plain ground. One call = a whole set that would
# otherwise be a bought sprite pack or a run of single-asset calls.

for slug, subj in [
    ("villager-poses", "small chibi medieval peasant villager in nine poses: standing, walking, carrying a sack, kneeling, hammering, hoeing, sitting, waving, sleeping"),
    ("villager-variants", "small chibi medieval peasant villagers, varied tunic colours and hair, front view, full body"),
    ("cottage-construction", "isometric medieval cottage construction stages: foundation stones, timber frame, half-roofed, finished thatch"),
    ("tree-growth", "isometric oak tree growth stages: sapling, young tree, mature, ancient"),
    ("crop-growth", "isometric wheat field patch growth stages: bare soil, green shoots, half-grown, ripe golden heads"),
    ("guard-poses", "small medieval town guard with spear: standing, marching, ready stance, saluting"),
    ("cottage-seasons", "the same isometric medieval cottage in four seasons: spring green, summer lush, autumn gold, winter snow"),
    ("farm-animals", "small medieval farm animals: dog, cat, hen, goat, sheep, pig, cow"),
    ("visitor-pack", "small chibi travelling medieval figures: pedlar with pack, pilgrim with staff, friar in habit, minstrel with lute, beggar with bowl"),
]:
    add("sheets/sprites", f"sheet-{slug}", subj + ", consistent matched set",
        style="rd_pro__spritesheet", w=256, h=256, remove_bg=False, tier="p1",
        notes="one call returns the whole grid; slice per cell")

# ---------------------------------------------------------------------------
# P1: festival modes - the four solstice/equinox occasions.

for slug, f in FESTIVALS:
    add("festivals", f"festival-{slug}", f"{ISO}, wide festive scene: {f}, {CROWD}",
        w=256, h=256, remove_bg=False, tier="p1")
for slug, deco in [
    ("maypole", "maypole with coloured ribbons"),
    ("bonfire", "stacked bonfire, unlit"),
    ("wreath", "evergreen wreath with candles"),
    ("feast-table", "long feast table laden with food"),
    ("lantern-post", "wooden post with hanging lanterns"),
    ("flower-garland", "flower garland strung between posts"),
    ("harvest-display", "harvest display: wheat sheaves, pumpkins, apples"),
    ("yule-log", "decorated yule log with holly"),
]:
    add("festivals/decor", f"decor-{slug}", f"{ISO}: {deco}",
        style="rd_tile__tile_object", w=96, h=96, tier="p1")

# ---------------------------------------------------------------------------
# P1: missing season/biome coverage and weather moods.
# The pack only has Spring + Winter + Desert blocks: fill the rest.

for biome, desc in BIOMES[1:]:  # temperate is covered by the season sets
    for tile, subj in [
        ("ground", f"ground block, {desc}"),
        ("vegetation", f"vegetation clump native to {desc.split(',')[0]}"),
        ("rock", f"rock outcrop, {desc}"),
    ]:
        add(f"biomes/{biome}", f"{biome}-{tile}", f"{ISO}: {subj}",
            w=128, tier="p1", tile_x=(tile == "ground"), tile_y=(tile == "ground"))
    add(f"biomes/{biome}", f"{biome}-village",
        f"{ISO}, wide view: small medieval village in {desc}",
        w=256, h=256, remove_bg=False, tier="p1")

# Signature biome trees/props the temperate set lacks.
for slug, p in [
    ("cactus", "clustered desert cacti"),
    ("cypress", "tall thin mediterranean cypress"),
    ("palm", "oasis date palm"),
    ("dead-pine", "bleached standing dead pine"),
    ("standing-stones", "small ring of weathered standing stones"),
    ("snowdrift", "deep smooth snowdrift"),
    ("ice-floe", "cracked ice floe in dark water"),
]:
    add("biomes/props", f"prop-{slug}", f"{ISO}: {p}",
        style="rd_tile__tile_object", w=96, h=96, tier="p1")

# ---------------------------------------------------------------------------
# P1: Scotland - Edinburgh and Glasgow reads. Real buildings including the
# gloriously weird ones; slate skies and sandstone palettes throughout.

SCOTTISH = [
    # Edinburgh: medieval skyscrapers of the Old Town
    ("edinburgh-land", "tall narrow old town tenement 'land', six storeys, harled stone, crow-stepped gables, dormer windows"),
    ("edinburgh-crown-spire", "gothic kirk with a crown-shaped steeple"),
    ("edinburgh-castle-crag", "castle on a sheer volcanic rock crag above the town"),
    ("scott-monument", "spindly black victorian gothic rocket-spire monument"),
    ("calton-folly", "unfinished greek temple columns alone on a hilltop"),
    ("georgian-crescent", "blonde sandstone georgian terrace, columns and fanlights"),
    ("camera-obscura", "old outlook tower with a domed camera chamber on top"),
    ("cannonball-house", "stone merchant house with a cannonball lodged in its gable wall"),
    ("baronial-turret", "scots baronial corner turret with a conical witch's-hat roof"),
    ("whitehorse-close", "white-harled courtyard inn with outside forestairs"),
    ("painted-terrace", "curved terrace of brightly painted shopfronts"),
    # Glasgow: sandstone, gothic, and one perfect folly
    ("glasgow-red-tenement", "red sandstone tenement block, four storeys, bay windows"),
    ("glasgow-blonde-tenement", "honey blonde sandstone tenement with a corner turret"),
    ("glasgow-cathedral", "dark gothic cathedral, soot-blackened stone"),
    ("necropolis", "hilltop victorian cemetery, monument spires along the skyline"),
    ("templeton-factory", "carpet factory dressed as a venetian gothic palace, polychrome glazed brick"),
    ("tolbooth-steeple", "lone stone steeple standing free at a crossroads"),
    ("bridge-of-sighs", "single-span stone bridge up to a walled cemetery"),
    ("finnieston-crane", "giant grey cantilever crane over a river"),
    ("peoples-palace", "red sandstone museum with a glass winter-gardens dome"),
    ("saracen-bandstand", "ornate victorian cast-iron bandstand"),
    ("cone-statue", "equestrian statue with a traffic cone on its head"),
]
for slug, b in SCOTTISH:
    add("regions/scotland", f"scot-{slug}", f"{ISO}: {b}, grey scottish light",
        w=160, h=160, tier="p1",
        notes="real building - Edinburgh or Glasgow landmark")

for slug, s in [
    ("edinburgh-street", "steep old town street of tall tenements and crow-stepped gables under a low grey sky"),
    ("glasgow-green", "sandstone tenements round a green, a gaudy venetian-gothic factory at one edge"),
]:
    add("regions/scotland", f"scot-scene-{slug}", f"{ISO}, wide view: {s}, {CROWD}",
        w=256, h=256, remove_bg=False, tier="p1")

# ---------------------------------------------------------------------------
# P1: Scottish weather modes - haar, dreich, smirr, stinging showers, and
# the famous four seasons in one day. Scene art; the procedural weather_fx
# system can take these as its look reference.

SCOT_WEATHER = [
    ("haar", "cold sea fog rolling in over rooftops, buildings dissolving into grey"),
    ("dreich", "flat grey drizzle, wet stone, dreary light"),
    ("smirr", "fine misting drizzle settling over fields"),
    ("stinging-shower", "slanted heavy rain squall driving sideways"),
    ("four-seasons", "sunlit showers: bright patches and falling rain in the same sky"),
]
for slug, w in SCOT_WEATHER:
    add("weather", f"weather-{slug}", f"wide isometric view: medieval village under {w}",
        w=256, h=256, remove_bg=False, tier="p1")

# ---------------------------------------------------------------------------
# P0: palette reference - generate once, then pass as input_palette to pin
# the english countryside colors on every other call.

add("colors", "palette-english-countryside",
    "flat color palette chart, one square swatch per color: sage green, hedgerow green, meadow green, honey limestone, chalk white, barley gold, thatch amber, drystone grey, slate blue-grey, weathered oak brown, muted terracotta, drizzle-grey sky",
    style="rd_pro__simple", w=128, h=128, remove_bg=False, tier="p0",
    notes="use the output as input_palette on other calls to lock the countryside palette")

# English countryside biome dressing, in that palette.
COUNTRYSIDE = "sage greens, honey limestone, drystone grey, drizzle-grey sky"
for slug, p in [
    ("hedgerow", "hedgerow of hawthorn and bramble"),
    ("drystone-wall", "drystone field wall"),
    ("copse", "small copse of beech and ash"),
    ("sheep-fold", "stone sheep fold in a field corner"),
    ("stile", "wooden stile over a field wall"),
    ("hayrick", "conical hayrick with a loose cover"),
]:
    add("biomes/countryside", f"country-{slug}", f"{ISO}: {p}, {COUNTRYSIDE}",
        style="rd_tile__tile_object", w=96, h=96, tier="p1")
for tile, subj in [
    ("meadow", "meadow ground block, long grass"),
    ("chalk-down", "chalk downland ground block, short turf"),
    ("barley-field", "barley field ground block, ripe gold"),
]:
    add("biomes/countryside", f"country-{tile}", f"{ISO}: {subj}, {COUNTRYSIDE}, square isometric ground block",
        w=128, h=128, tile_x=True, tile_y=True, tier="p1")
add("biomes/countryside", "country-village",
    f"{ISO}, wide view: small english countryside village, honey limestone cottages, hedgerows and barley fields, {CROWD}",
    w=256, h=256, remove_bg=False, tier="p1")

# ---------------------------------------------------------------------------
# P1: special features already in the game - minstrel visit, market day,
# weather, pets, the mill, graves, scenario scenes.

SCENES = [
    ("minstrel-visit", "travelling minstrel playing lute at the market crossroads, villagers gathered round, dropped coins"),
    ("market-day", "busy medieval market day, stalls, crowds, goods baskets"),
    ("church-service", "villagers entering a stone church for service"),
    ("harvest-in", "villagers bringing in the harvest together, carts of wheat"),
    ("funeral", "small funeral at the graveyard, villagers heads bowed"),
    ("hard-winter", "village in deep snow, thin smoke from chimneys, bleak hard winter"),
    ("outbreak", "village under quarantine, closed doors, a healer walking alone"),
    ("storm-damage", "storm-torn village, fallen branch, damaged roof"),
    ("guild-meeting", "craftsmen meeting in a timber guildhall"),
    ("tax-day", "tax collector at the market with guards, villagers queueing"),
    ("wedding", "village wedding outside the church, flower petals"),
    ("pilgrims-arrive", "pilgrims with staffs arriving on the road"),
]
for slug, s in SCENES:
    add("scenes", f"scene-{slug}", f"{ISO}, wide lively scene: {s}, {CROWD}",
        w=256, h=256, remove_bg=False, tier="p1",
        notes="cutscene/loading art; also usable as reference for the matching feature")

# ---------------------------------------------------------------------------
# P2: cultures - the 20 traditions in the creativity corpus. Each gets a
# villager pair + a house style + a motif so regions read differently.

# ---------------------------------------------------------------------------
# Rare events (session/events.rs). Every Visitor stands a pitch and cries a
# trade - each needs its own figure; today all draw traveller_s. Moments and
# Spells are map phenomena: an icon for the toast, plus an in-world tile for
# the creatures and props that stand on the ground.

EVENT_VISITORS = [
    ("tinker", "itinerant tinker with a pack of kettles and pots on his back"),
    ("knife-grinder", "knife-grinder at a foot-treadle grindstone, sparks"),
    ("rag-and-bone", "rag-and-bone man with a sack of rags over his shoulder"),
    ("chimney-sweep", "sooty chimney sweep with brush and rods"),
    ("rat-catcher", "rat-catcher with a terrier and a pole of dead rats"),
    ("dowser", "water-diviner with a forked hazel wand held out before him"),
    ("scribe", "itinerant scribe with a writing slope and quill case"),
    ("tooth-drawer", "travelling tooth-drawer holding up iron pincers"),
    ("barber-surgeon", "barber-surgeon with basin and fleam, rolled sleeves"),
    ("pilgrims", "three weary pilgrims with staffs and scallop badges"),
    ("relic-seller", "relic-seller holding a tiny sealed reliquary box"),
    ("pardoner", "pardoner flourishing a sealed parchment indulgence"),
    ("juggler", "juggler tossing three knives, one foot up"),
    ("bear-ward", "bear-ward leading a muzzled brown bear on a rope"),
    ("puppeteer", "puppeteer beside a small striped puppet booth"),
    ("ballad-monger", "ballad-monger holding up printed broadside sheets"),
    ("news-crier", "town crier with a bell and a scroll"),
    ("recruiter", "recruiting sergeant in a sash, coin purse held out"),
    ("deserter", "ragged deserter, half-worn soldier's coat, wary look"),
    ("refugee-family", "refugee family: mother, father, two children, bundled goods"),
    ("leper-band", "two hooded lepers with a clapper and begging bowl"),
    ("friar", "wandering friar in a brown habit, bare feet"),
    ("quiet-preacher", "plain-clothed hedge preacher, hands open, no vestments"),
    ("drover", "drover with a long stick guiding a knot of sheep"),
    ("fish-hawker", "fish-hawker with a creel basket of silver fish"),
    ("salt-seller", "salt-seller with a sack and a cup of white salt"),
    ("wool-merchant", "wool merchant in a good cloak, scale in hand"),
    ("wine-merchant", "wine merchant with a small cask and a tasting cup"),
    ("spice-merchant", "spice merchant with bright packets of pepper and cloves"),
    ("bookseller", "bookseller with a tray of small bound volumes"),
    ("glazier", "glazier carrying a leaded glass pane and a glass-cutter"),
    ("thatcher", "thatcher with a bundle of reed and a leggett tool"),
    ("well-digger", "well-digger with a shovel and rope over his shoulder"),
    ("charcoal-burner", "charcoal-burner in a soot-stained smock, coal bag"),
    ("lime-burner", "lime-burner with a grey dust coat and lime shovel"),
    ("tanner", "tanner with rolled hides on his shoulder, stained apron"),
    ("candle-maker", "candle-maker carrying a dipper frame hung with candles"),
    ("potter", "travelling potter with a crate of crocks and jugs"),
    ("horse-coper", "horse-coper holding a halter, appraising look"),
    ("eggler", "eggler with a basket of eggs and a butter crock"),
    ("cheese-monger", "cheese-monger carrying a whole round cheese"),
    ("fortune-teller", "fortune-teller in bright shawls, palm held out"),
    ("astrologer", "astrologer with a star chart and brass astrolabe"),
    ("alchemist", "alchemist in a stained robe, odd vials at his belt"),
    ("quack", "medicine-show quack holding a small bottled elixir"),
    ("midwife", "midwife with a basket of linen and herbs"),
    ("sowgelder", "pig man with a small knife roll and a patient air"),
    ("farrier", "farrier with a hoof pick and a horseshoe held high"),
    ("corn-factor", "corn-factor with a grain measure and a tally stick"),
    ("messenger", "messenger in a badge, letter held out before him"),
    ("prodigal", "travel-worn returned son, staff and a hopeful look"),
]
for slug, subj in EVENT_VISITORS:
    person("events/visitors")(slug, subj)

# In-world tiles: creatures and props that stand on the map during the event.
EVENT_TILES = [
    ("wolf", "grey wolf at a sheep pen, crouched"),
    ("fox", "red fox slinking near a hen run"),
    ("hawk", "stooping hawk with wings tucked, mid-dive"),
    ("rat-swarm", "a mob of rats pouring across the ground"),
    ("locust-cloud", "a dense cloud of locusts over a field"),
    ("caterpillar", "fat green caterpillar on a leaf"),
    ("starling-flock", "a wheeling murmuration of starlings"),
    ("bee-swarm", "a swarm of bees around a hollow tree trunk"),
    ("deer", "red deer doe grazing in a crop field"),
    ("wild-boar", "wild boar rooting in a furrow"),
    ("barn-owl", "white-faced barn owl perched on a roof beam"),
    ("crow-mob", "a mob of crows settling on a field"),
    ("leaping-fish", "silver fish leaping in the shallows"),
    ("truffle-pig", "pig nosing the ground under an oak"),
    ("bear", "large muzzled dancing bear on a rope"),
]
for slug, subj in EVENT_TILES:
    add("events/creatures", f"creature-{slug}", f"{ISO}: {subj}",
        style="rd_tile__tile_object", w=64, h=64, tier="p1")

EVENT_PROPS = [
    ("bee-tree", "hollow oak with bees at a knothole, honeycomb edge"),
    ("honey-tree", "tree hung with dripping wild honeycomb"),
    ("fallen-oak", "giant old oak fallen across a lane, split trunk"),
    ("old-stones", "ancient dressed stones half-buried in a furrow"),
    ("spring", "fresh spring bubbling up between rocks, clear water"),
    ("truffle", "black truffle in leaf litter, a hand reaching"),
    ("mushroom-cluster", "cluster of wild mushrooms under a tree"),
    ("berry-bush", "hedge bush heavy with dark berries"),
    ("herb-clump", "clump of flowering medicinal herbs by a hedge"),
    ("coin-hoard", "half-buried clay pot spilling old silver coins"),
    ("cart-overturned", "overturned handcart spilling sacks on a lane"),
    ("fouled-well", "village well with green scum on the water"),
    ("granary-leak", "grain store with wet sacks and drips from the eaves"),
    ("chimney-fire", "cottage chimney with smoke and a lick of flame"),
    ("drowned-pond", "village pond with a small ring of ripples and a dropped basket"),
    ("lost-child", "small child's dropped toy at a wood's edge"),
    ("foundling", "wrapped infant in a basket on a church doorstep"),
    ("quarrel", "two villagers face to face, fists up, small crowd"),
    ("driftwood", "driftwood, a broken barrel and rope on a river strand"),
    ("storm-leavings", "scattered planks and a staved barrel after a storm"),
]
for slug, subj in EVENT_PROPS:
    add("events/props", f"prop-{slug}", f"{ISO}: {subj}",
        style="rd_tile__tile_object", w=64, h=64, tier="p1")

# Toast icons for moments and spells (the sky/weather/social events with no
# standing figure or tile of their own).
EVENT_ICONS = [
    ("hailstorm", "white hailstones bouncing off a roof"),
    ("heat-wave", "shimmering heat lines over dry crops"),
    ("drought", "cracked dry earth under a pale sun"),
    ("flood", "brown river water risen over a bank"),
    ("lightning", "a single lightning bolt striking a field"),
    ("late-frost", "white frost on a green sprout"),
    ("indian-summer", "golden warm haze over autumn fields"),
    ("first-snow", "first thin snowflakes over a cottage"),
    ("thaw", "dripping icicle melting into a puddle"),
    ("aurora", "green aurora ribbons over a village at night"),
    ("meteor-shower", "streaking meteors in a night sky"),
    ("blood-moon", "large red moon over rooftops"),
    ("dark-noon", "black disc of an eclipse over a pale sky"),
    ("windstorm", "bent trees and flying thatch in wind"),
    ("mirage", "shimmering false city on the horizon"),
    ("alehouse-brawl", "two tankards clashing, spilled ale"),
    ("assize", "weighing scales with a loaf and a tankard"),
    ("cellar-thief", "dark figure slipping out of a cellar door with a sack"),
    ("lost-child-icn", "small empty shoe beside a path"),
    ("foundling-icn", "basket with a swaddled bundle"),
    ("quarrel-icn", "two raised fists face to face"),
    ("thief-icn", "shadowed hand lifting a jar"),
]
for slug, subj in EVENT_ICONS:
    add("events/icons", f"event-{slug}", f"small game UI icon of {subj}, simple, readable at tiny size",
        style="rd_mini__skill_icon", w=32, h=32, n=4,
        notes="batch of 4 variants; keep the clearest")


def cultures():
    try:
        with open(CULTURES_TSV, encoding="utf-8") as f:
            seen = []
            for line in f:
                if line.startswith("#") or "\t" not in line:
                    continue
                parts = line.rstrip("\n").split("\t")
                if len(parts) > 1 and parts[1] not in seen and parts[1] != "tradition":
                    seen.append(parts[1])
            return seen
    except OSError:
        return ["norse", "celtic", "anglo-saxon", "medieval", "byzantine"]

CULTURE_HINTS = {
    "african": "west african savanna", "anglo-saxon": "early medieval anglo-saxon",
    "arthurian": "arthurian high-medieval", "celtic": "celtic iron-age",
    "classical": "classical greco-roman", "east-asian": "east asian",
    "egyptian": "ancient egyptian", "finnish": "finnish forest",
    "frankish": "frankish carolingian", "germanic": "germanic tribal",
    "hindu": "classical indian", "medieval": "western medieval",
    "mesoamerican": "mesoamerican", "mesopotamian": "mesopotamian mudbrick",
    "norse": "norse viking", "persian": "persian",
    "polynesian": "polynesian island", "scripture": "ancient near-eastern biblical",
    "spanish": "spanish colonial", "tolkien-roots": "rustic shire-like fantasy",
}
for c in cultures():
    hint = CULTURE_HINTS.get(c, c.replace("-", " "))
    add(f"cultures/{c}", f"culture-{c}-villagers",
        f"small chibi character pair, front view, full body, on empty background: man and woman in traditional {hint} dress",
        style="rd_pro__default", w=64, h=64, n=2, tier="p2")
    add(f"cultures/{c}", f"culture-{c}-house",
        f"single isometric building asset: {hint} dwelling house, traditional materials and roof",
        w=160, h=160, tier="p2")
    add(f"cultures/{c}", f"culture-{c}-motif",
        f"small decorative emblem motif of {hint} art, banner/badge",
        style="rd_mini__skill_icon", w=32, h=32, n=2, tier="p2")

# ---------------------------------------------------------------------------
# P2: future features from wiki/backlog.md - pull ahead so the art exists
# when the feature lands.

FUTURE_VOCATIONS = [
    ("shepherd", "shepherd with crook and dog"), ("goatherd", "goatherd with goats"),
    ("beekeeper", "beekeeper in veil by straw skeps"), ("brewer", "brewer by ale barrels"),
    ("vintner", "vintner pruning grapevines"), ("distiller", "distiller by copper still"),
    ("cheesemaker", "cheesemaker with round cheese wheels"),
    ("pasta-maker", "pasta maker hanging noodles to dry"),
    ("perfumer", "perfumer with flower petals and glass bottles"),
    ("fletcher", "fletcher binding arrow shafts"), ("cooper", "cooper hooping a barrel"),
    ("shoemaker", "cobbler at a shoe last"), ("tentmaker", "tentmaker stitching canvas"),
    ("pigment-collector", "pigment collector grinding minerals"),
    ("peddler", "roving pots-and-pans peddler, wares on back"),
    ("translator", "translator with scrolls of different scripts"),
    ("herbalist", "herbalist witch by drying herbs"),
    ("fisherman", "fisherman with net and basket"),
    ("potter", "potter at a kick wheel"), ("weaver", "weaver at a loom"),
    ("matchmaker", "village matchmaker, older woman in worn finery carrying "
                   "a ledger of eligible names and letters"),
]
for slug, subj in FUTURE_VOCATIONS:
    add("future/vocations", f"future-{slug}", f"{SPRITE}: {subj}",
        style="rd_pro__default", w=48, h=48, tier="p2")

FUTURE_FEATURES = [
    ("circus-wagon", "travelling circus wagon, painted and gaudy"),
    ("circus-tent", "striped circus tent, small"),
    ("castle", "fortified stone castle with keep and dungeon tower"),
    ("stocks", "village punishment stocks"),
    ("guillotine", "wooden guillotine scaffold"),
    ("bee-skep", "straw bee skeps in a garden row"),
    ("herb-garden", "walled herb garden with labelled beds"),
    ("orchard", "small orchard of fruit trees"),
    ("mushroom-ring", "ring of mushrooms in forest shade"),
    ("boat-row", "small wooden rowboat at a jetty"),
    ("cog", "medieval trading cog at anchor"),
    ("harbour", "tiny stone harbour with jetty and crates"),
    ("cave-mouth", "dark cave entrance in a rock face"),
    ("shrine", "roadside pilgrimage shrine with offerings"),
    ("monastery", "small stone monastery with cloister garden"),
    ("tavern", "village tavern with hanging sign and benches"),
    ("dungeon", "gloomy castle dungeon cell, torchlit"),
    ("sea-monster", "sea monster coils breaking the surface"),
    ("fish", "silver fish hanging in a pair"),
    ("lobster", "red lobster on a plate"),
    ("squid", "pale squid"),
    ("comet", "bright comet with tail in a night sky"),
    ("eclipse", "solar eclipse corona in dark sky"),
    ("tomato-throw", "villagers throwing rotten tomatoes at a man in the stocks"),
    ("labor-strike", "villagers blocking a bridge with banners, labor action"),
    ("dog-herding", "sheepdog herding sheep"),
]
for slug, subj in FUTURE_FEATURES:
    add("future/features", f"future-{slug}", f"{ISO}: {subj}",
        w=160, h=160, tier="p2")

# Future goods/items for the production chains the backlog plans.
FUTURE_GOODS = [
    ("cheese", "cheese wheel"), ("wine", "wine bottle and cup"),
    ("ale", "foaming ale mug"), ("honey", "honey jar with dipper"),
    ("perfume", "ornate perfume bottle"), ("mushrooms", "basket of mushrooms"),
    ("herbs", "bundle of dried herbs"), ("fish-dried", "dried fish on a line"),
    ("pasta", "dried pasta noodles"), ("wool", "ball of wool"),
    ("milk", "clay jug of milk"), ("eggs", "basket of eggs"),
    ("mead", "mead bottle"), ("cider", "jug of cider"),
]
for slug, subj in FUTURE_GOODS:
    add("future/goods", f"good-{slug}", f"small game item icon of {subj}",
        style="rd_mini__topdown_item", w=32, h=32, n=2, tier="p2")

# The same list as one matched atlas - far cheaper per item than the calls above.
add("future/goods", "future-goods-sheet",
    "inventory item sheet: cheese wheel, wine bottle, ale mug, honey jar, perfume bottle, mushroom basket, dried herb bundle, dried fish, pasta noodles, wool ball, milk jug, egg basket, mead bottle, cider jug",
    style="rd_pro__inventory_items", w=256, h=256, remove_bg=False, tier="p2",
    notes="atlas alternative to the good-* calls above; include_downloadable_data returns the item atlas")

# ---------------------------------------------------------------------------
# ANIMATED tier: premium. Prompt-driven animations from text alone, plus the
# advanced-animation styles that animate an input_image (two-stage: first
# generate the still in P0, then animate it here).

# 4-angle walk cycles for the sprites that move (48px prompt-driven).
for slug, subj in [
    ("peasant-m", "male peasant villager"), ("peasant-f", "female peasant villager"),
    ("worker-m", "male labourer"), ("child-m", "small boy child"),
    ("elder-m", "elderly man with stick"), ("noble-f", "noblewoman in gown"),
    ("priest", "village priest in robes"), ("guard-spear", "guard with spear"),
    ("minstrel", "minstrel carrying a lute"), ("tax-collector", "tax collector in dark robes"),
]:
    add("anim/walk", f"walk4-{slug}", subj + ", medieval",
        style="rd_animation__four_angle_walking", w=48, h=48,
        notes="48px 4-direction walk cycle, transparent GIF")

# 8-direction rotation for hero characters (3x3 sheet option).
for slug, subj in [
    ("player", "young organizer in green tunic"),
    ("minstrel", "minstrel with lute"),
    ("knight", "knight in mail"),
]:
    add("anim/rotate8", f"rot8-{slug}", subj + ", medieval",
        style="rd_animation__8_dir_rotation", w=64, h=64, sheet=True,
        notes="8 directional views; return_spritesheet gives the 3x3 sheet")

# Battle sprites for armed figures.
for slug, subj in [("knight", "knight in mail with sword"),
                   ("guard-halberd", "guard with halberd")]:
    add("anim/battle", f"battle-{slug}", subj,
        style="rd_animation__battle_sprites", w=64, h=64,
        notes="idle/walk/jump/attack set")

# VFX: weather and festival effects as loops.
for slug, subj in [
    ("bonfire", "large bonfire burning, festival night"),
    ("chimney-smoke", "wispy chimney smoke rising"),
    ("leaves", "autumn leaves drifting down"),
    ("snowfall", "snowflakes falling"),
    ("spark", "spark burst, small magic effect"),
    ("haar", "low sea fog drifting across the ground"),
    ("drizzle", "fine grey drizzle falling"),
]:
    add("anim/vfx", f"vfx-{slug}", subj,
        style="rd_animation__vfx", w=64, h=64)

# Longer ambient animations (64px any_animation, 128px big_animation).
for slug, subj, style in [
    ("mill-sails", "windmill sails slowly turning", "rd_animation__any_animation"),
    ("banner-wave", "village banner waving on a pole", "rd_animation__any_animation"),
    ("river-flow", "river water flowing past reeds", "rd_animation__any_animation"),
    ("sea-waves", "waves lapping a sandy shore", "rd_animation__any_animation"),
    ("festival-dance", "villagers dancing in a ring at festival", "rd_animation__big_animation"),
    ("market-ambient", "market square ambient bustle, flags and awnings swaying", "rd_animation__big_animation"),
]:
    add("anim/ambient", f"amb-{slug}", "isometric medieval: " + subj,
        style=style, w=64 if "any" in style else 128, h=64 if "any" in style else 128)

# Two-stage advanced animations: these need input_image from a P0 still.
TWO_STAGE = [
    ("walking", "rd_advanced_animation__walking", "slow heavy steps",
     "input_image = the person-* P0 sprite, native res, padded to 64px"),
    ("idle", "rd_advanced_animation__idle", "gentle breathing idle",
     "input_image = the person-* P0 sprite"),
    ("custom-action", "rd_advanced_animation__custom_action", "playing a lute, fingers plucking",
     "input_image = the person-minstrel P0 sprite"),
    ("subtle-motion", "rd_advanced_animation__subtle_motion", "ambient crowd sway and blinking",
     "input_image = a scene-* P1 festival or market image"),
    ("rotate", "rd_advanced_animation__rotate", "",
     "input_image = any P0 building/prop still; returns the 8 views"),
]
for slug, style, motion, how in TWO_STAGE:
    add("anim/two-stage", f"adv-{slug}", motion or "the uploaded subject",
        style=style, w=64, h=64, frames=8,
        notes="two-stage: " + how + ". Send native-res frame, pad to 64px, retry once on provider_busy")

# ---------------------------------------------------------------------------
# Quality-floor comparison: the manifest leans on rd_pro; a couple of cheap
# drafts tell you whether Fast/Plus would do for the filler assets.

add("compare/tiers", "compare-cottage-fast",
    "isometric medieval cottage, timber-frame and thatch",
    style="rd_fast__game_asset", w=64, h=64, tier="p1",
    notes="quality floor check: if Fast looks fine, filler assets get much cheaper")
add("compare/tiers", "compare-cottage-plus",
    "isometric medieval cottage, timber-frame and thatch",
    style="rd_plus__default", w=128, h=128, tier="p1",
    notes="middle tier, same subject as the rd_pro cottages-set")

# ---------------------------------------------------------------------------
# Smoke batch: one representative call per rendering mode / style family.
# ~$3 total - run this first to validate prompts, sizes and outputs before
# any real spend. ids get a -smoke suffix so they don't collide with the
# full-batch entries in the state file.

SMOKE = [
    # (slug prefix, what the call validates)
    ("ground-grass-spring", "isometric ground block + tile_x/tile_y seamless flags"),
    ("road-wang-spring", "wang tileset mode - the whole road set from one call"),
    ("tree-oak-spring", "tile_object prop mode"),
    ("shop-bakery", "rd_pro isometric building"),
    ("person-minstrel", "rd_pro character sprite - keep the output, it feeds the two-stage animations"),
    ("icon-talk", "mini icon readability at tiny size"),
    ("good-bread", "mini item icon"),
    ("goods-sheet", "inventory_items atlas mode + include_downloadable_data"),
    ("sheet-villager-poses", "spritesheet mode - matched pose grid in one call"),
    ("ui-panel", "ui_panel chrome"),
    ("festival-summer-solstice", "wide isometric scene"),
    ("walk4-peasant-m", "prompt-driven 4-angle walk GIF"),
    ("battle-knight", "battle sprites set (idle/walk/jump/attack)"),
    ("vfx-bonfire", "vfx animation loop"),
    ("amb-mill-sails", "any_animation ambient loop"),
    ("amb-festival-dance", "big_animation 128px scene loop"),
    ("rot8-player", "8-direction rotation sheet"),
    ("adv-walking", "two-stage advanced animation - needs a person-* output as input_image"),
    ("adv-rotate", "advanced rotate - 8 views of an uploaded still"),
    ("ui-title-keyart", "painterly key art"),
    ("palette-english-countryside", "flat palette swatch (rd_pro__simple)"),
    ("compare-cottage-fast", "rd_fast quality floor"),
    ("compare-cottage-plus", "rd_plus middle tier"),
]


def smoke_entries():
    """The curated smoke batch: the real entries, num_images clamped to 1."""
    out = []
    for prefix, why in SMOKE:
        e = next((x for x in ENTRIES if x["id"].startswith(prefix + "-")), None)
        if e is None:
            sys.exit(f"smoke slug not found: {prefix}")
        e = json.loads(json.dumps(e))  # deep copy
        e["request"]["num_images"] = 1
        e["est_cost_usd"] = round(cost(e["request"]["prompt_style"], 1), 3)
        e["id"] += "-smoke"
        e["notes"] = f"validates: {why}" + (f" | {e['notes']}" if e["notes"] else "")
        out.append(e)
    return out


# ---------------------------------------------------------------------------
# Assembly: sort, dedup, filter, emit.

def sort_key(e):
    return (TIER_RANK[e["tier"]], e["group"], e["id"])


def load_state(path):
    """Ids already generated. Accepts a JSON list or {"done": [...]} file."""
    try:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        if isinstance(data, dict):
            data = data.get("done", [])
        return set(data)
    except (OSError, json.JSONDecodeError):
        return set()


def save_state(path, done):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump({"done": sorted(done)}, f, indent=1)


def pick(entries, args, done):
    """Apply the selection flags in order: done-state, --tier, --group,
    --limit, --budget. Returns (picked, est spend)."""
    tiers = set(args.tier.split(",")) if args.tier else None
    picked, spend = [], 0.0
    for e in entries:
        if e["id"] in done:
            continue
        if tiers and e["tier"] not in tiers:
            continue
        if args.group and args.group not in e["group"]:
            continue
        if args.limit is not None and len(picked) >= args.limit:
            break
        if args.budget is not None and spend + e["est_cost_usd"] > args.budget:
            continue
        picked.append(e)
        spend += e["est_cost_usd"]
    return picked, spend


def emit(entries, out):
    os.makedirs(out, exist_ok=True)
    total = round(sum(e["est_cost_usd"] for e in entries), 2)
    manifest = {
        "_meta": {
            "service": "https://api.retrodiffusion.ai/v2/inferences",
            "auth": "header X-RD-Token: rdpk-... ; run via scripts/rd_generate.py (sync calls must NOT send Idempotency-Key)",
            "usage": [
                "prompt describes the SUBJECT only; prompt_style carries the pixel look",
                "check_cost:true is a free dry run - verify a batch's price first",
                "two-stage entries need input_image from the earlier still",
                "reference_images (rd_pro) keeps characters consistent - reuse your favourite output",
            ],
            "count": len(entries),
            "est_total_usd": total,
        },
        "prompts": entries,
    }
    with open(os.path.join(out, "prompts.json"), "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=1)

    lines = ["# Retro Diffusion art prompts", "",
             f"{len(entries)} prompts, est. ${total} USD at list prices.",
             "Ids are stable: re-run with `--state art-prompts/generated.json` to skip done work.", ""]
    cur = None
    for e in entries:
        head = (e["tier"], e["group"])
        if head != cur:
            cur = head
            lines += ["", f"## {TIER_NAMES[e['tier']]} / {e['group']}", "",
                      "| id | prompt | style | size | n | est $ | notes |",
                      "|---|---|---|---|---|---|---|"]
        r = e["request"]
        size = f"{r['width']}x{r['height']}"
        lines.append("| {} | {} | {} | {} | {} | {:.2f} | {} |".format(
            e["id"], e["prompt"].replace("|", "\\|"), r["prompt_style"],
            size, r["num_images"], e["est_cost_usd"],
            e["notes"].replace("|", "\\|")))
    with open(os.path.join(out, "prompts.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", default=OUT_DIR, help="output dir (default art-prompts/)")
    ap.add_argument("--tier", help="comma list: p0,p1,p2,anim")
    ap.add_argument("--group", help="substring filter on group, e.g. tiles or cultures")
    ap.add_argument("--limit", type=int, help="emit only the first N (priority order)")
    ap.add_argument("--budget", type=float, help="emit prompts until this USD cap")
    ap.add_argument("--state", default=STATE,
                    help="JSON file of ids already generated (default %(default)s)")
    ap.add_argument("--record", action="store_true",
                    help="record emitted ids into the state file as pending-generation "
                         "(use --mark-done once the API calls actually succeed)")
    ap.add_argument("--include-done", action="store_true",
                    help="don't filter by state (list everything)")
    ap.add_argument("--mark-done", metavar="IDS_OR_FILE",
                    help="mark ids as successfully generated, without emitting a batch. "
                         "Accepts a comma-separated list of ids, or a path to a JSON file "
                         "holding a list of ids (or {\"done\": [...]})")
    ap.add_argument("--smoke", action="store_true",
                    help="emit the curated smoke batch: one call per rendering "
                         "mode, ~$3 total (writes to <out>/smoke unless --out is set)")
    args = ap.parse_args()

    if args.mark_done:
        if os.path.isfile(args.mark_done):
            newly_done = load_state(args.mark_done)
        else:
            newly_done = {i.strip() for i in args.mark_done.split(",") if i.strip()}
        existing = load_state(args.state)
        save_state(args.state, existing | newly_done)
        print(f"marked {len(newly_done)} ids done in {args.state} "
              f"({len(existing | newly_done)} total done)")
        return

    if args.smoke:
        out = args.out if args.out != OUT_DIR else os.path.join(OUT_DIR, "smoke")
        done = set() if args.include_done else load_state(args.state)
        picked, spend = pick(smoke_entries(), args, done)
        emit(picked, out)
        if args.record:
            save_state(args.state, load_state(args.state) | {e["id"] for e in picked})
        print(f"wrote {len(picked)} smoke prompts (${spend:.2f} est) to {out}")
        for e in picked:
            print(f"  {e['id']:42s} {e['request']['prompt_style']:42s} ${e['est_cost_usd']:.2f}")
        return

    ids = [e["id"] for e in ENTRIES]
    dupes = {i for i in ids if ids.count(i) > 1}
    if dupes:
        sys.exit(f"internal duplicate ids: {sorted(dupes)}")

    existing_done = load_state(args.state)
    done = set() if args.include_done else existing_done
    entries = sorted(ENTRIES, key=sort_key)
    picked, spend = pick(entries, args, done)

    emit(picked, args.out)
    if args.record:
        save_state(args.state, existing_done | {e["id"] for e in picked})

    by_tier = {}
    for e in picked:
        t = by_tier.setdefault(e["tier"], [0, 0.0])
        t[0] += 1
        t[1] += e["est_cost_usd"]
    print(f"wrote {len(picked)} prompts (${spend:.2f} est) to {args.out}")
    for t in sorted(by_tier, key=TIER_RANK.get):
        n, c = by_tier[t]
        print(f"  {t:5s} {n:4d} prompts  ${c:7.2f}")
    skipped = len(entries) - len(picked)
    if done:
        print(f"  ({len(done)} already generated, {skipped} skipped by filters/limit)")


if __name__ == "__main__":
    main()
