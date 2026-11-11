//! Art loading: the sprite pack, the generated (RD) sprites, walk sheets and ground tile sets.

use super::*;

/// The dog's coat
pub(crate) const DOG: (u8, u8, u8) = (139, 90, 43);

/// A little dog, facing left, as pixel art: o outline, b coat, n eye/nose,
/// . transparent. Drawn 2x like the villagers.
pub(crate) const DOG_ART: [&str; 7] = [
    "..oo.......",
    ".obbo....o.",
    "obnbboooobo",
    ".obbbbbbbo.",
    "..obbbbbbo.",
    "..ob.o.obo.",
    "..oo....oo.",
];

pub(crate) fn dog_sprite() -> Sprite {
    let (w, h) = (DOG_ART[0].len(), DOG_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in DOG_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c { 'o' => (45, 28, 16), 'b' => DOG, 'n' => (15, 10, 8), _ => continue };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// The cat's coat (a grey tabby)
pub(crate) const CAT: (u8, u8, u8) = (95, 95, 105);

/// A little cat, facing right, sitting with its tail curled round: o
/// outline, b coat, n eye/nose, . transparent. Drawn 2x like the dog.
pub(crate) const CAT_ART: [&str; 7] = [
    "..o.....o..",
    ".obo...obo.",
    ".obbbnbbo..",
    "oobbbbbbo..",
    "obbbbbbbo..",
    "ob..bb..o..",
    ".o..oo.....",
];

pub(crate) fn cat_sprite() -> Sprite {
    let (w, h) = (CAT_ART[0].len(), CAT_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in CAT_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c { 'o' => (40, 38, 45), 'b' => CAT, 'n' => (15, 10, 8), _ => continue };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// A hen scratching round the farmyard, facing left: o outline, w white
/// feathers, r comb, n eye/beak, l legs, . transparent
pub(crate) const CHICKEN_ART: [&str; 7] = [
    "....oo.....",
    "...orro....",
    "..owwno....",
    ".owwwwwo...",
    ".owwwwoo...",
    "..l..l.....",
    "..l..l.....",
];

pub(crate) fn chicken_sprite() -> Sprite {
    let (w, h) = (CHICKEN_ART[0].len(), CHICKEN_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in CHICKEN_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c {
                'o' => (60, 55, 50), 'w' => (235, 230, 220), 'r' => (200, 60, 50),
                'n' => (15, 10, 8), 'l' => (200, 150, 60), _ => continue,
            };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// A gull on the wing: just two white strokes and a grey under-shadow
pub(crate) const GULL_ART: [&str; 3] = [
    "w.....w",
    ".w...w.",
    "..g.g..",
];

pub(crate) fn gull_sprite() -> Sprite {
    let (w, h) = (GULL_ART[0].len(), GULL_ART.len());
    let mut pixels = vec![0u8; w * h * 4];
    for (y, row) in GULL_ART.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let rgb = match c { 'w' => (245, 248, 250), 'g' => (170, 180, 195), _ => continue };
            pixels[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[rgb.0, rgb.1, rgb.2, 255]);
        }
    }
    Sprite { width: w, height: h, pixels }
}

/// The castles' grand set: the same slots as the town wall's pieces
/// (curtain along x, along y, the two gatehouses; posterns along x, y;
/// one tower look per map), at fortress scale
pub struct CastleGrand {
    pub walls: [Option<Sprite>; 4],
    pub posts: [Option<Sprite>; 2],
    pub tower: Option<Sprite>,
}

/// Every sprite the game draws, loaded once up front
#[derive(Default)]
pub struct Art {
    pub villager_sprites: Vec<Sprite>,
    /// The rivals' figures: the same pack's outlined look, so they read foreign
    pub rival_sprites: Vec<Sprite>,
    /// The cottages a home can be: colour, timber or plaster, and the door
    /// and gable in different spots (the pack's numbered variants)
    pub houses: Vec<Option<Sprite>>,
    /// The houses with their greens snowed over, for winter
    pub houses_winter: Vec<Option<Sprite>>,
    pub mill_frames: Vec<Sprite>, // the sails only: they turn on mill_tower
    pub mill_tower: Option<Sprite>,
    /// The windmill is generated (assets/rd/windmill): the tower stands at
    /// its own size (draw_standing) and the sails turn at 1x on MILL_HUB_RD
    pub mill_rd: bool,
    pub blacksmith_s: Option<Sprite>,
    /// The trade halls and the rival village's homes with their grass
    /// snowed over (blacksmith, guildhall, rival house, rival woodhouse);
    /// None for the pack's, which keep their summer look
    pub trades_winter: [Option<Sprite>; 4],
    pub stones: Vec<Sprite>,        // boulders, a few shapes
    pub stone_grounds: Vec<Sprite>, // flat rocks
    /// The rocks' size at zoom 1 (PACK_PROP_K for the pack's, 1 for assets/rd/stones)
    pub stones_k: f32,
    pub player_s: Option<Sprite>,
    /// The player's walk cycle (assets/rd/player/walk.png). None: the
    /// pack's still figure (player_s) stands in.
    pub player_walk: Option<WalkSheet>,
    /// Walk cycles for the villager looks, by the same index as
    /// villager_sprites (None: that look is still the pack's figure)
    pub villager_walks: Vec<Option<WalkSheet>>,
    pub tower_s: Option<Sprite>,
    /// The town wall, its towers and gatehouses are generated
    /// (assets/rd/castle): drawn one tile wide like the cottages instead of
    /// on the pack's block scale
    pub castle_rd: bool,
    /// Town wall blocks: running along x, along y, and the two gatehouses
    pub walls: [Option<Sprite>; 4],
    /// The odd wall stretch with a postern door in it: along x, along y
    pub wall_posts: [Option<Sprite>; 2],
    /// The castles' own set (assets/rd/castle/grand_*): fortress-scale
    /// curtains, gates, posterns and towers at true proportions, where the
    /// village wall's pieces stand at fence height. All-or-nothing with
    /// keep_s: a partial set would mix scales inside one castle.
    pub castle_grand: Option<CastleGrand>,
    /// The keep in each castle's courtyard (assets/rd/castle/keep*): the
    /// tallest thing the country raises. None: tower_s stands in.
    pub keep_s: Option<Sprite>,
    pub church_s: Option<Sprite>,
    pub guildhall_s: Option<Sprite>,
    pub rival_house: Option<Sprite>,
    pub rival_woodhouse: Option<Sprite>,
    /// The watch at the town gates, from the humans pack
    pub guards: Vec<Sprite>,
    /// Generated looks for the watch (rd/guards), by the same index as
    /// guards; drawn standing, facing out
    pub guard_walks: Vec<Option<WalkSheet>>,
    /// Generated people stills (assets/rd/people), by file name - the
    /// postings and trades the pack has no figure for. Drawn standing at
    /// 1x by draw_animal; a missing name leaves the pack figure in use
    pub people: Vec<(String, Sprite)>,
    pub priest_s: Option<Sprite>,
    pub priest_walk: Option<WalkSheet>,
    pub traveller_s: Option<Sprite>, // a courier riding the road between villages
    /// Generated walk cycles: the rivals' (by RIVAL_NAMES index) and the
    /// travellers' (courier, minstrel, pedlar, the rare visitors)
    pub rival_walks: Vec<Option<WalkSheet>>,
    pub traveller_walk: Option<WalkSheet>,
    /// A knight's mount (assets/rd/mounted): horse and rider as one
    /// figure for `PersonV.mounted`; `mounted_flip` mirrors it for the
    /// leftward ride. None: the walker figure stands in
    pub mounted: Option<Sprite>,
    pub mounted_flip: Option<Sprite>,
    /// The animals: Retro Diffusion art (assets/rd/farm-animals, sliced
    /// from one generated sheet - see assets/rd/README.md), drawn at 1x
    /// zoom by draw_animal. If those files are missing the old hand-drawn
    /// pixel art stands in, pre-doubled to the same scale.
    pub dog: Option<Sprite>,
    pub cat: Option<Sprite>,
    /// Hens about the village (variants: hen, rooster)
    pub hens: Vec<Sprite>,
    /// The fold: a couple of looks each, picked per animal
    pub sheep: Vec<Sprite>,
    pub goats: Vec<Sprite>,
    pub gull: Option<Sprite>,
    /// Ground specks scattered on open grass, per season set
    pub details: [Vec<Sprite>; 3],
    /// Their size at zoom 1 (PACK_PROP_K for the pack's, 1 for assets/rd/tufts)
    pub details_k: f32,
    /// The workplaces' own looks (assets/rd/buildings), by WorkLook, and
    /// the same snowed over; an empty set wears a cottage instead
    pub works: [Vec<Option<Sprite>>; WorkLook::N],
    pub works_winter: [Vec<Option<Sprite>>; WorkLook::N],
    /// The market cross with its notice board, and the maypole that
    /// stands there on a feast day
    /// The stocks by the market: two posts, a foot bar, a seat - drawn,
    /// not generated, a punishment the village feeds
    pub stocks: Option<Sprite>,
    pub market_cross: Option<Sprite>,
    /// A moored boat's hull - drawn small, the river's own craft
    pub boat: Option<Sprite>,
    pub maypole: Option<Sprite>,
    /// Market-day stalls (sellers' looks), and the pedlar's striped booth
    pub stalls: Vec<Sprite>,
    pub trader_stall: Option<Sprite>,
    /// The village well and meeting hall on the grass by the market
    /// square (assets/rd/bits); None: they go undrawn
    pub well_s: Option<Sprite>,
    pub hall_s: Option<Sprite>,
    /// Chimney-smoke frames (assets/rd/bits/smoke*): wispy puffs rising;
    /// empty keeps the drawn puffs
    pub smoke: Vec<Sprite>,
    /// The footbridge over the river: deck along x, along y
    pub bridges: [Option<Sprite>; 2],
    /// The memorial pieces (assets/rd/monuments, #703): wayside cross,
    /// picture stone, obelisk, trophy and the Scott column, and the
    /// composed cairns; None/empty keeps the caller's placeholder draw
    pub memorial_cross: Option<Sprite>,
    pub memorial_picture: Option<Sprite>,
    pub memorial_obelisk: Option<Sprite>,
    pub memorial_trophy: Option<Sprite>,
    pub memorial_column: Option<Sprite>,
    pub memorial_cairns: Vec<Sprite>,
    /// Small generated props (assets/rd/bits); empty: the old blocks stand in
    pub skeps: Vec<Sprite>,
    pub graves: Vec<Sprite>,
    /// Upright tombstones for the churchyard's buried (assets/rd/bits/tomb*);
    /// empty: the flat grave marks serve both
    pub tombstones: Vec<Sprite>,
    /// The lychgate over the churchyard path (assets/rd/bits/lychgate*);
    /// empty: the gate goes undrawn
    pub lychgate: Vec<Sprite>,
    pub herbs: Vec<Sprite>,
    pub shrooms: Vec<Sprite>,
    /// Flames (frames to flicker between) and the scorched patch left behind
    pub fires: Vec<Sprite>,
    pub burnt: Vec<Sprite>,
    /// Ground blocks for the green seasons, autumn and winter (season_tiles),
    /// and the same as top faces only, for inland tiles
    pub tiles: [TileSet; 3],
    pub faces: [TileSet; 3],
}

/// A generated 4-direction walk cycle: a row per facing (0 up, 1 right,
/// 2 down, 3 left on screen), 4 frames each, 1x art (rd_animation
/// four_angle_walking, 48px frames, figures ~35px tall)
pub struct WalkSheet {
    pub frames: Vec<Vec<Sprite>>,
    /// The row the feet stand on and the top of the head, over every frame
    /// (so anchors don't jitter between frames)
    pub feet: i32,
    pub head: i32,
}

impl WalkSheet {
    /// Cut a 4x4 sheet into frames and find its feet and head rows
    pub fn from_sheet(sheet: &Sprite) -> WalkSheet {
        let f = sheet.width / 4;
        let frames: Vec<Vec<Sprite>> = (0..4).map(|r| (0..4).map(|c| sheet.crop(c * f, r * f, f, f)).collect()).collect();
        let rows = |s: &Sprite| -> Vec<usize> {
            (0..s.height).filter(|&y| (0..s.width).any(|x| s.pixels[(y * s.width + x) * 4 + 3] > 0)).collect()
        };
        let feet = frames.iter().flatten().filter_map(|s| rows(s).last().copied()).max().unwrap_or(0) as i32 + 1;
        let head = frames.iter().flatten().filter_map(|s| rows(s).first().copied()).min().unwrap_or(0) as i32;
        WalkSheet { frames, feet, head }
    }

    /// The frame for a pose at frame count `t`: striding through the row
    /// while walking, the standing frame otherwise
    pub fn frame(&self, facing: u8, walking: bool, t: u32) -> Option<&Sprite> {
        let row = self.frames.get(facing as usize)?;
        row.get(if walking { (t / 6) as usize % row.len().max(1) } else { 1 })
    }

    /// Draw a pose standing on screen point (sx, sy) at `scale` (1.0 a
    /// grown-up at the current zoom); returns the y of the top of the head
    pub fn draw(&self, buf: &mut Buffer, facing: u8, walking: bool, t: u32, sx: i32, sy: i32, scale: f32) -> i32 {
        let feet = (self.feet as f32 * scale).round() as i32;
        if let Some(f) = self.frame(facing, walking, t) {
            let w = (f.width as f32 * scale).round() as i32;
            draw_zoomed(buf, f, sx - w / 2, sy - feet, scale);
        }
        sy - feet + (self.head as f32 * scale).round() as i32
    }
}

/// The pack's ground blocks for one season's look: one per kind of map
/// tile, and every numbered road and river piece, in the pack's order
#[derive(Default)]
pub struct TileSet {
    pub grass: Option<Sprite>,
    /// Sandy grass where the sea meets the land, and drier bare looks up
    /// on the hills and the bare tops
    pub shore: Option<Sprite>,
    pub hill: Option<Sprite>,
    pub rock: Option<Sprite>,
    pub water: Option<Sprite>,
    /// Pale water round the shore, darker out at sea
    pub shallow: Option<Sprite>,
    pub deep: Option<Sprite>,
    pub wheat: Vec<Sprite>,
    pub corn: Vec<Sprite>,
    pub roads: Vec<Sprite>,
    pub rivers: Vec<Sprite>,
    /// Winter only: iced-over river stretches
    pub rivers_ice: Vec<Sprite>,
    pub trees: Vec<Sprite>,
    /// How big the trees draw at zoom 1: the pack's are drawn on its block
    /// scale (PACK_PROP_K), the generated ones (assets/rd/trees) at 1x
    pub trees_k: f32,
}

/// Which TileSet a season draws with
pub(crate) fn season_tiles(season: Season) -> usize {
    match season { Season::Autumn => 1, Season::Winter => 2, _ => 0 }
}

/// `block` with its greens recoloured by `f` (grass to water, or autumn gold)
pub(crate) fn recolour_greens(block: &Sprite, f: impl Fn((u8, u8, u8)) -> (u8, u8, u8)) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    for p in out.pixels.chunks_mut(4) {
        if p[3] > 0 && p[1] > p[0] && p[1] > p[2] {
            let (r, g, b) = f((p[0], p[1], p[2]));
            p[0] = r; p[1] = g; p[2] = b;
        }
    }
    out
}

/// The top face of the pack's 94x97 ground blocks: a diamond centred on
/// (47, 26.5), 47 across and 26.5 down from its centre to its corners
pub(crate) const FACE_CENTRE: (f32, f32, f32, f32) = (47.0, 26.5, 47.0, 26.5);

/// How far the top face's outline runs either side of its edge, as a
/// fraction of the way to the centre (measured on the grass block: 0.76
/// to 1.12 - it's thickest at the corners)
pub(crate) const FACE_OUTLINE: (f32, f32) = (0.25, 0.15);

/// `block` with the dark outline round its top face (and its soft,
/// half-dark edge) painted over in the colour just inside it
pub(crate) fn strip_face_outline(block: &Sprite) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    let (cx, cy, rx, ry) = FACE_CENTRE;
    let px = |x: i32, y: i32| -> [u8; 4] {
        let i = (y as usize * block.width + x as usize) * 4;
        [block.pixels[i], block.pixels[i + 1], block.pixels[i + 2], block.pixels[i + 3]]
    };
    let lum = |p: [u8; 4]| p[0] as u32 + p[1] as u32 + p[2] as u32;
    for y in 0..block.height.min(60) as i32 {
        for x in 0..block.width as i32 {
            let d = ((x as f32 + 0.5 - cx) / rx).abs() + ((y as f32 + 0.5 - cy) / ry).abs();
            let here = px(x, y);
            if d < 1.0 - FACE_OUTLINE.0 || d > 1.0 + FACE_OUTLINE.1 || here[3] == 0 { continue; }
            // the face's own colour, just inside the outline on the way to
            // the centre (the distance shrinks evenly along that line)
            let t = 1.0 - (1.0 - FACE_OUTLINE.0 - 0.04) / d;
            let (ix, iy) = ((x as f32 + (cx - x as f32) * t) as i32, (y as f32 + (cy - y as f32) * t) as i32);
            let fill = px(ix, iy);
            if fill[3] > 0 && lum(here) * 100 < lum(fill) * 85 {
                let i = (y as usize * block.width + x as usize) * 4;
                out.pixels[i..i + 4].copy_from_slice(&fill);
            }
        }
    }
    out
}

/// Just the top face of a ground block, its sides cut away: inland, where
/// the blocks in front cover the sides anyway, their edges only peeked
/// between the tiles as faint lines
pub(crate) fn face_only(block: &Sprite) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    let (cx, cy, rx, ry) = FACE_CENTRE;
    for y in 0..block.height {
        for x in 0..block.width {
            let d = ((x as f32 + 0.5 - cx) / rx).abs() + ((y as f32 + 0.5 - cy) / ry).abs();
            if d > 1.0 + FACE_OUTLINE.1 / 2.0 { out.pixels[(y * block.width + x) * 4 + 3] = 0; }
        }
    }
    out
}

/// The sea: the pack has no open-water block, so it's the grass block with
/// its grass turned to water, colder in winter. `shade` scales the blue
/// per channel: shallow water is paler, the open sea darker.
pub(crate) fn water_block(grass: &Sprite, winter: bool, shade: (f32, f32, f32)) -> Sprite {
    let k = if winter { 0.8 } else { 1.0 };
    recolour_greens(grass, |(_, g, _)| {
        let l = g as f32 * k;
        ((l * shade.0) as u8, (l * shade.1) as u8, (l * shade.2).min(255.0) as u8)
    })
}

/// Blend every drawn pixel of `block` toward `colour` by `t` (where
/// recolour_greens only touches the greens - snow has none to take)
pub(crate) fn tint(block: &Sprite, colour: (u8, u8, u8), t: f32) -> Sprite {
    let mut out = Sprite { width: block.width, height: block.height, pixels: block.pixels.clone() };
    for p in out.pixels.chunks_mut(4) {
        if p[3] == 0 { continue; }
        let (r, g, b) = blend((p[0], p[1], p[2]), colour, t);
        p[0] = r; p[1] = g; p[2] = b;
    }
    out
}

/// A per-tile pick from a sprite list, stable for the tile (`salt` keeps
/// different lists from lining up on the same tiles)
pub(crate) fn pick(v: &[Sprite], x: i32, y: i32, salt: u32) -> Option<&Sprite> {
    if v.is_empty() { None } else { v.get((noise(x, y, salt) * v.len() as f32) as usize % v.len()) }
}

impl TileSet {
    /// Load the blocks for the green seasons (0), autumn (1: the green set
    /// gone gold) or winter (2)
    pub(crate) fn load(at: &dyn Fn(&str) -> Option<Sprite>, season: usize) -> TileSet {
        let load = |rel: &str| at(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/{}", rel));
        let winter = season == 2;
        let gold = |s: Sprite| if season == 1 { recolour_greens(&s, |c| blend(c, (185, 125, 45), 0.45)) } else { s };
        let pieces = |dir: &str, name: &str, n: usize| -> Vec<Sprite> {
            (1..=n).filter_map(|i| load(&format!("{}/{}({}).png", dir, name, i))).map(gold).collect()
        };
        // the generated block (assets/rd/tiles, exported onto the pack's
        // block geometry) if present; water, shore, hills and rock are all
        // recoloured from it
        let grass = at("rd/tiles/grass.png").or_else(|| load("Enviroument/Spring/grass.png"));
        // the generated road/river pieces (scripts/rd_compose_tiles.py: the
        // pack's shapes repainted from RD blocks, same geometry and order),
        // all of a set or none so a road never mixes looks
        let rd_pieces = |name: &str, n: usize| -> Option<Vec<Sprite>> {
            let v: Vec<Sprite> = (1..=n).filter_map(|i| at(&format!("rd/tiles/{}({}).png", name, i))).map(&gold).collect();
            (v.len() == n).then_some(v)
        };
        // fields lie under the snow in winter
        let snow = || at("rd/tiles/winter/snow.png").or_else(|| load("Enviroument/Winter/snow.png"));
        let water = |shade: (f32, f32, f32)| grass.as_ref().map(|g| water_block(g, winter, shade));
        let (water, shallow, deep) = (water((0.55, 0.8, 1.2)), water((0.8, 1.0, 1.35)), water((0.35, 0.55, 1.0)));
        let ground = if winter { snow() } else { grass.map(gold) };
        // the look of bare land this season: a sandy rim where the sea
        // meets it, parched hills and grey tops (snow has no greens to
        // recolour, so winter tints the whole block instead)
        let (shore, hill, rock) = if winter {
            (ground.as_ref().map(|s| tint(s, (200, 210, 230), 0.3)),
             ground.as_ref().map(|s| tint(s, (215, 195, 165), 0.3)),
             ground.as_ref().map(|s| tint(s, (155, 160, 175), 0.35)))
        } else {
            (ground.as_ref().map(|s| recolour_greens(s, |c| blend(c, (224, 204, 150), 0.55))),
             ground.as_ref().map(|s| recolour_greens(s, |c| blend(c, (170, 140, 85), 0.45))),
             ground.as_ref().map(|s| recolour_greens(s, |c| blend(c, (150, 145, 140), 0.55))))
        };
        // both growth stages of each crop, so a field isn't one plant repeated
        let crop = |name: &str| -> Vec<Sprite> {
            rd_pieces(name, 2).unwrap_or_else(||
                (1..=2).filter_map(|i| load(&format!("Buildings/{}/{}({}).png", name, name, i))).map(&gold).collect())
        };
        // the generated woods (assets/rd/trees: broadleaf and fir, winter's
        // bare and snowed-on) if there, else the pack's four
        let rd_trees: Vec<Sprite> = (1..=8)
            .filter_map(|i| at(&format!("rd/trees/{}tree{}.png", if winter { "winter/" } else { "" }, i))).map(&gold).collect();
        let rd_trees_k = !rd_trees.is_empty();
        let pack_trees: Vec<Sprite> = (1..=4).filter_map(|i|
            load(&format!("Enviroument/{}/trees/tree({}).png", if winter { "Winter" } else { "Spring" }, i)).map(&gold)).collect();
        let set = TileSet {
            shore, hill, rock, water, shallow, deep,
            grass: ground,
            trees: if rd_trees.is_empty() { pack_trees } else { rd_trees },
            trees_k: if rd_trees_k { 1.0 } else { PACK_PROP_K },
            wheat: if winter { snow().into_iter().collect() } else { crop("wheat") },
            corn: if winter { snow().into_iter().collect() } else { crop("corn") },
            roads: if winter {
                // the winter set numbers its two straight roads the other way round
                let mut r = rd_pieces("winter/road_winter", 15).unwrap_or_else(|| pieces("Roads/Winter", "road_winter", 15));
                if r.len() > 2 { r.swap(1, 2); }
                r
            } else { rd_pieces("road", 15).unwrap_or_else(|| pieces("Roads/Spring", "road", 15)) },
            rivers: if winter { rd_pieces("winter/river_winter", 17).unwrap_or_else(|| pieces("Rivers/Winter", "river_winter", 17)) }
                else { rd_pieces("river", 17).unwrap_or_else(|| pieces("Rivers/Spring", "river", 17)) },
            rivers_ice: if winter {
                rd_pieces("winter/river_ice_winter", 17).unwrap_or_else(|| pieces("Rivers/Winter/Ice", "river_ice_winter", 17))
            } else { vec![] },
        };
        // without the dark outline round their top faces, so the ground
        // reads as one surface rather than a grid of tiles (lines inside a
        // tile, like a road's or a riverbank's edges, stay)
        set.map_blocks(strip_face_outline)
    }

    /// Every ground block put through `f` (the trees stay as they are)
    pub(crate) fn map_blocks(&self, f: impl Fn(&Sprite) -> Sprite) -> TileSet {
        TileSet {
            grass: self.grass.as_ref().map(&f),
            shore: self.shore.as_ref().map(&f),
            hill: self.hill.as_ref().map(&f),
            rock: self.rock.as_ref().map(&f),
            water: self.water.as_ref().map(&f),
            shallow: self.shallow.as_ref().map(&f),
            deep: self.deep.as_ref().map(&f),
            wheat: self.wheat.iter().map(&f).collect(),
            corn: self.corn.iter().map(&f).collect(),
            roads: self.roads.iter().map(&f).collect(),
            rivers: self.rivers.iter().map(&f).collect(),
            rivers_ice: self.rivers_ice.iter().map(&f).collect(),
            trees: self.trees.clone(),
            trees_k: self.trees_k,
        }
    }

    /// The block for map tile (x, y), joined up with its neighbours
    pub(crate) fn block(&self, map: &TileMap, x: i32, y: i32) -> Option<&Sprite> {
        let sea = |dx: i32, dy: i32| map.get(x + dx, y + dy) == Some(Tile::Water);
        let by_sea = || NEIGHBOURS.iter().any(|&(dx, dy)| sea(dx, dy));
        let by_land = || NEIGHBOURS.iter().any(|&(dx, dy)| !sea(dx, dy) && map.get(x + dx, y + dy) != Some(Tile::River));
        match map.get(x, y)? {
            // the shore wears sand, the hills go parched, the tops bare (the
            // pack's dirt block is orange and read as a stray bit of road)
            Tile::Grass if by_sea() => self.shore.as_ref().or(self.grass.as_ref()),
            Tile::Grass => self.grass.as_ref(),
            Tile::Hill => self.hill.as_ref().or(self.grass.as_ref()),
            Tile::Rock => self.rock.as_ref().or(self.hill.as_ref()).or(self.grass.as_ref()),
            Tile::Water if by_land() => self.shallow.as_ref().or(self.water.as_ref()),
            Tile::Water if noise(x, y, 9) < 0.35 => self.deep.as_ref().or(self.water.as_ref()),
            Tile::Water => self.water.as_ref(),
            Tile::Wheat => pick(&self.wheat, x, y, 5),
            Tile::Corn => pick(&self.corn, x, y, 6),
            Tile::Road => self.roads.get(map.piece(x, y)? as usize - 1),
            Tile::River => {
                let p = map.piece(x, y)? as usize - 1;
                // in winter stretches of the river freeze right over
                if noise(x, y, 13) < 0.45 { self.rivers_ice.get(p).or(self.rivers.get(p)) } else { self.rivers.get(p) }
            }
            Tile::Bridge => self.rivers.get(map.piece(x, y)? as usize - 1),
            // a wadeable stretch draws as the shallows it is (#1074)
            Tile::Ford => self.shallow.as_ref().or(self.water.as_ref()),
        }
    }
}

/// Every generated people still the loader looks for
/// (assets/rd/people/<name>.png): the postings first, then the ten
/// covered trades in -m/-f pairs. The sex suffix is a placeholder until
/// the sim carries a villager's Sex (#681); unmapped roles (archer,
/// king, mage, ...) load dormant for whoever wires them later.
pub(crate) const PEOPLE_SPRITES: [&str; 36] = [
    "archer", "cavalier", "courier", "crossbowman",
    "guard-halberd", "guard-shield", "guard-spear", "guard-sword",
    "king", "mage", "minstrel", "overseer", "plague-doctor", "prince", "tax-collector", "trader",
    "baker-f", "baker-m", "farmer-f", "farmer-m",
    "glazier-f", "glazier-m", "knight-f", "knight-m",
    "luthier-f", "luthier-m", "mason-f", "mason-m",
    "priest-f", "priest-m", "printmaker-f", "printmaker-m",
    "tailor-f", "tailor-m", "toolmaker-f", "toolmaker-m",
];

/// A villager sprite cropped to its standing frame (each sheet is a grid
/// of 32x32 cells)
pub(crate) fn figure(at: &dyn Fn(&str) -> Option<Sprite>, rel: &str) -> Sprite {
    match at(rel) {
        Some(s) => s.crop(0, 0, 32, 32),
        None => { println!("sprite load {}", rel); Sprite { width: 1, height: 1, pixels: vec![255, 0, 255, 255] } }
    }
}

/// The sprite for `houses[i]`, or none where nothing loaded
pub(crate) static NO_SPRITE: Option<Sprite> = None;

impl Art {
    /// The generated still for a person key: a PersonV.person token
    /// ("baker", "tax-collector"), or the token with a -m/-f suffix for
    /// the sexed trades. None: the caller draws the pack figure.
    pub fn person(&self, name: &str) -> Option<&Sprite> {
        self.people.iter().find(|(n, _)| n == name).map(|(_, s)| s)
    }

    /// Load every sprite through `fetch` (rel path -> png bytes): the file
    /// system on desktop, a bundled pack on the web.
    /// Load from an asset folder on disk ("assets" from the repo root).
    pub fn load_fs(root: &str, seed: u64) -> Art {
        let root = std::path::PathBuf::from(root);
        Self::load(&move |rel: &str| std::fs::read(root.join(rel)).ok(), seed)
    }

    /// The cottage `i` wraps round to (the house list cycles)
    pub(crate) fn house(&self, i: usize) -> &Option<Sprite> {
        self.houses.get(i % self.houses.len().max(1)).unwrap_or(&NO_SPRITE)
    }

    /// The same cottage in winter: the generated ones stand on a grass
    /// base, which snows over (and moss on a thatch roof gets a dusting)
    pub(crate) fn house_winter(&self, i: usize) -> &Option<Sprite> {
        self.houses_winter.get(i % self.houses_winter.len().max(1)).unwrap_or(&NO_SPRITE)
    }

    /// Load every sprite, `seed` settling which variant each single-choice
    /// building wears this map (the mill's colour, the corner towers, the
    /// church's keep) - the same map seed looks the same every run
    pub fn load(fetch: &dyn Fn(&str) -> Option<Vec<u8>>, seed: u64) -> Art {
        let at = |rel: &str| fetch(rel).and_then(|b| Sprite::decode(&b).ok());
        let assets = Assets::new("");
        let pack = |rel: &str| at(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/{}", rel));

        // one variant out of several, chosen by the seed (None if it won't
        // load, or the first that did)
        let chosen = |v: Vec<Option<Sprite>>| -> Option<Sprite> {
            if v.is_empty() { return None; }
            v.get(seed as usize % v.len()).and_then(|s| s.clone())
                .or_else(|| v.into_iter().find_map(|s| s))
        };

        // Load villager sprites (every look the villagers pack has, one
        // each): role and gender as villager_sprite knows them
        let vpeople: [(&str, &str); 8] = [
            ("peasant", "m"), ("peasant", "f"), ("elder", "m"), ("noble", "m"),
            ("princess", "f"), ("worker", "m"), ("elder", "f"), ("noble", "f"),
        ];
        let villager_sprites: Vec<Sprite> = vpeople.iter()
            .map(|&(role, gender)| figure(&at, &assets.villager_sprite(role, gender))).collect();
        // the rivals: same figures, the outlined look, and their matriarch
        // is the pack's queen - every villager sprite ends up in use
        let rival_sprites: Vec<Sprite> = ["MiniVillagerMan", "MiniWorker", "MiniQueen"].iter()
            .map(|n| figure(&at, &format!("MinifolksVillagers/MinifolksVillagers/Outline/{}.png", n))).collect();
        // the watch at the gates: soldiers from the humans pack
        let guards: Vec<Sprite> = ["MiniSpearMan", "MiniSwordMan", "MiniShieldMan", "MiniHalberdMan"].iter()
            .map(|n| figure(&at, &format!("MinifolksHumans/Without Outline/{}.png", n))).collect();

        // the cottages: plaster and timber fronts in every colour, with the
        // door and gable in different spots, so no two homes look alike
        let houses: Vec<Option<Sprite>> = [
            "Buildings/house/White houses/Blue house/white_blue_house(1).png",
            "Buildings/house/Wood houses/Blue house/wood_blue_house(1).png",
            "Buildings/house/White houses/Green house/white_green_house(3).png",
            "Buildings/house/Wood houses/Red house/wood_red_house(2).png",
            "Buildings/house/White houses/Wood house/white_wood_house(4).png",
            "Buildings/house/Wood houses/Green house/wood_green_house(2).png",
            "Buildings/house/White houses/Red house/white_red_house(3).png",
            "Buildings/house/Wood houses/Wood house/wood_house(4).png",
            "Buildings/house/Wood houses/Wood house/wood_house(2).png",
            "Buildings/house/White houses/Blue house/white_blue_house(4).png",
        ].iter().map(|rel| pack(rel)).collect();
        // the generated cottages (assets/rd/buildings) replace the pack's
        // looks when present: a matched street, not a mix
        let mut rd_houses: Vec<Option<Sprite>> = (1..=9).filter_map(|i| at(&format!("rd/buildings/cottage{}.png", i))).map(Some).collect();
        // the botháns - squat thatch cottages - join the street behind the
        // cottages; (file number, slot) pairs so winter sprites land right
        let mut bothan_slots = Vec::new();
        if !rd_houses.is_empty() {
            for i in 1..=3 {
                if let Some(s) = at(&format!("rd/buildings/bothan{}.png", i)) {
                    bothan_slots.push((i, rd_houses.len()));
                    rd_houses.push(Some(s));
                }
            }
        }
        let houses = if rd_houses.is_empty() { houses } else { rd_houses };
        let mut houses_winter: Vec<Option<Sprite>> = houses.iter()
            .map(|h| h.as_ref().map(|s| recolour_greens(s, |c| blend(c, (236, 240, 246), 0.85)))).collect();
        // a bothán's winter sprite replaces its recoloured summer self:
        // snow loads the thatch differently than a green tint can show
        for (i, slot) in bothan_slots {
            if let Some(s) = at(&format!("rd/buildings/bothan-winter{}.png", i)) {
                houses_winter[slot] = Some(s);
            }
        }

        // walk cycles: the player's, and the villager looks that have one
        // (same order as vpeople: the peasant man and woman so far)
        let walk = |rel: &str| at(rel).map(|s| WalkSheet::from_sheet(&s));
        let player_walk = walk("rd/player/walk.png");
        let villager_walks: Vec<Option<WalkSheet>> = vpeople.iter()
            .map(|&(role, gender)| walk(&format!("rd/villagers/{}-{}.png", role, gender))).collect();
        // the knight's mount: one horse-and-rider figure, mirrored for
        // the leftward ride
        let mounted = at("rd/mounted/knight.png");
        let mounted_flip = mounted.as_ref().map(|s| {
            let (w, h) = (s.width as usize, s.height as usize);
            let mut px = s.pixels.clone();
            for y in 0..h {
                for x in 0..w / 2 {
                    let a = (y * w + x) * 4;
                    let b = (y * w + (w - 1 - x)) * 4;
                    let t = [px[a], px[a + 1], px[a + 2], px[a + 3]];
                    let u = [px[b], px[b + 1], px[b + 2], px[b + 3]];
                    px[a..a + 4].copy_from_slice(&u);
                    px[b..b + 4].copy_from_slice(&t);
                }
            }
            Sprite { width: s.width, height: s.height, pixels: px }
        });

        // the animals: generated art if it's there, else the hand-drawn
        // fallback doubled so both draw at the same 1x scale
        let doubled = |s: Sprite| s.scaled(s.width * 2, s.height * 2);
        let rd = |name: &str| at(&format!("rd/farm-animals/{}.png", name));
        let rd_set = |names: &[&str]| -> Vec<Sprite> { names.iter().filter_map(|n| rd(n)).collect() };
        let hens = rd_set(&["hen", "hen", "rooster"]);
        let hens = if hens.is_empty() { vec![doubled(chicken_sprite())] } else { hens };

        // ground specks for open grass: tufts and flowers, snow lumps in
        // winter, the green set gone gold in autumn
        let gold = |s: Sprite| recolour_greens(&s, |c| blend(c, (185, 125, 45), 0.45));
        let detail_set = |dir: &str, name: &str, gild: bool| -> Vec<Sprite> {
            (1..=9).filter_map(|i| pack(&format!("Enviroument/{}/{}({}).png", dir, name, i)).map(|s| if gild { gold(s) } else { s })).collect()
        };
        // generated tufts and flowers (assets/rd/tufts, a winter set
        // beside them): both sets or neither, so the seasons match
        let rd_tufts: Vec<Sprite> = (1..=14).filter_map(|i| at(&format!("rd/tufts/tuft{}.png", i))).collect();
        let rd_tufts_w: Vec<Sprite> = (1..=12).filter_map(|i| at(&format!("rd/tufts/winter/tuft{}.png", i))).collect();
        let (details, details_k) = if rd_tufts.is_empty() || rd_tufts_w.is_empty() {
            ([
                detail_set("Spring", "grass_detail", false),
                detail_set("Spring", "grass_detail", true),
                detail_set("Winter", "snow_detail", false),
            ], PACK_PROP_K)
        } else {
            let autumn = rd_tufts.iter().map(|s| gold(s.clone())).collect();
            ([rd_tufts, autumn, rd_tufts_w], 1.0)
        };
        // the generated windmill: a tower and its sails' four turning frames
        let rd_mill_tower = at("rd/windmill/tower.png");
        let rd_sails: Vec<Sprite> = (1..=4).filter_map(|i| at(&format!("rd/windmill/sails{}.png", i))).collect();
        let mill_rd = rd_mill_tower.is_some() && rd_sails.len() == 4;
        // the trade halls and the rival village: generated when present
        let rd_b = |name: &str| chosen((1..=2).map(|i| at(&format!("rd/buildings/{}{}.png", name, i))).collect());
        let trades = [rd_b("smithy"), rd_b("guildhall"), rd_b("rival-house"), rd_b("rival-woodhouse")];
        let trades_winter = trades.clone().map(|t| t.map(|s| recolour_greens(&s, |c| blend(c, (236, 240, 246), 0.85))));
        let [smithy, guildhall, rival_house, rival_woodhouse] = trades;
        let rival_walks: Vec<Option<WalkSheet>> = ["man", "worker", "queen"].iter()
            .map(|n| walk(&format!("rd/rivals/{}.png", n))).collect();

        // boulders and flat stone scatters from one generated sheet
        // (assets/rd/stones), drawn at 1x like the rest of the generated art
        let rd_stones: Vec<Sprite> = (1..=12).filter_map(|i| at(&format!("rd/stones/stone{}.png", i))).collect();
        let rd_grounds: Vec<Sprite> = (1..=6).filter_map(|i| at(&format!("rd/stones/ground{}.png", i))).collect();
        // the generated town wall: one wall block, gatehouse and postern
        // block, mirrored for the other axis (the files say which), and a
        // few towers; all of it or none, so a wall never mixes looks
        struct RdCastle { walls: [Option<Sprite>; 4], posts: [Option<Sprite>; 2], towers: Option<Sprite> }
        let c = |n: &str| at(&format!("rd/castle/{}.png", n));
        let rd_castle = Some(RdCastle {
            walls: [c("wall_x"), c("wall_y"), c("gate_x"), c("gate_y")],
            posts: [c("postern_x"), c("postern_y")],
            towers: chosen((1..=4).map(|i| c(&format!("tower{}", i))).collect()),
        }).filter(|r| r.walls.iter().chain(&r.posts).all(|s| s.is_some()) && r.towers.is_some());
        // the castles' grand set and keep (#643): fortress-scale pieces
        // at true proportions, drawn 1x where the stretch used to be -
        // only alongside the town wall's set, and all-or-nothing like it:
        // a partial set would mix scales inside one castle
        let (castle_grand, keep_s) = if rd_castle.is_some() {
            let grand = CastleGrand {
                walls: [c("grand_wall_x"), c("grand_wall_y"), c("grand_gate_x"), c("grand_gate_y")],
                posts: [c("grand_postern_x"), c("grand_postern_y")],
                tower: chosen((1..=2).map(|i| c(&format!("grand_tower{}", i))).collect()),
            };
            let keep = chosen((1..=2).map(|i| c(&format!("keep{}", i))).collect());
            let whole = grand.walls.iter().chain(&grand.posts).all(Option::is_some)
                && grand.tower.is_some() && keep.is_some();
            if whole { (Some(grand), keep) } else { (None, None) }
        } else { (None, None) };
        let (rd_stones, rd_grounds) = if rd_stones.is_empty() || rd_grounds.is_empty() { (vec![], vec![]) } else { (rd_stones, rd_grounds) };

        // the workplaces' own looks, one or two each, snowed over in winter
        let snowed = |s: &Sprite| recolour_greens(s, |c| blend(c, (236, 240, 246), 0.85));
        let works = WorkLook::ALL.map(|w| {
            let own: Vec<Sprite> = (1..=2).filter_map(|i| at(&format!("rd/buildings/{}{}.png", w.file(), i))).collect();
            // the factory borrows the workshop's art until its own set lands
            let set = if own.is_empty() && w == WorkLook::Factory {
                (1..=2).filter_map(|i| at(&format!("rd/buildings/workshop{}.png", i))).collect()
            } else { own };
            set.into_iter().map(Some).collect::<Vec<_>>()
        });
        let works_winter = works.clone().map(|set| set.iter().map(|s| s.as_ref().map(snowed)).collect());
        // the village's small things (assets/rd/bits), numbered from 1
        let bits = |name: &str| -> Vec<Sprite> { (1..=6).filter_map(|i| at(&format!("rd/bits/{}{}.png", name, i))).collect() };

        Art {
            villager_sprites,
            rival_sprites,
            houses,
            houses_winter,
            tiles: [TileSet::load(&at, 0), TileSet::load(&at, 1), TileSet::load(&at, 2)],
            faces: [0, 1, 2].map(|season| TileSet::load(&at, season).map_blocks(face_only)),
            mill_frames: if mill_rd { rd_sails } else { (1..=4).filter_map(|i|
                at(&format!("Isometric Medieval Pack/Isometric Medieval Pack/Sprites/Buildings/mill/mill({}).png", i))
            ).collect() },
            mill_tower: if mill_rd { rd_mill_tower } else { chosen(["wood", "blue", "green", "red"].iter()
                .map(|c| pack(&format!("Buildings/mill/mill_{}.png", c))).collect()) },
            mill_rd,
            blacksmith_s: smithy.or_else(|| at(&assets.building_sprite("blacksmith", "green"))),
            trades_winter,
            // stone(1) is a tiny pebble that's nearly all dark outline - on
            // the map it read as a black hole, so the boulders stand instead
            stones_k: if rd_stones.is_empty() { PACK_PROP_K } else { 1.0 },
            stones: if rd_stones.is_empty() {
                (2..=4).filter_map(|i| pack(&format!("Enviroument/Spring/stones/stone({}).png", i))).collect()
            } else { rd_stones },
            stone_grounds: if rd_grounds.is_empty() {
                (1..=4).filter_map(|i| pack(&format!("Enviroument/Spring/stones/stone_ground ({}).png", i))).collect()
            } else { rd_grounds },
            player_walk,
            villager_walks,
            player_s: at("MinifolksVillagers/MinifolksVillagers/Without Outline/MiniPeasant.png").map(|s| s.crop(0, 0, 32, 32)),
            // the pack's corner towers: flat-top, capped or keeps - the
            // whole watch matches within a map, but varies between runs
            castle_rd: rd_castle.is_some(),
            tower_s: if let Some(c) = &rd_castle { c.towers.clone() } else { chosen([("Wood towers", "castle_tower_wood", 1), ("Wood towers", "castle_tower_wood", 4),
                             ("Wood towers", "castle_tower_wood", 5), ("Blue towers", "castle_tower_blue", 4)]
                .iter().map(|(d, n, i)| pack(&format!("Towers/{}/{}({}).png", d, n, i))).collect()) },
            // the pack's castle_wall(2) runs along x, (1) along y; (5) and (6) are gatehouses
            walls: match &rd_castle { Some(c) => c.walls.clone(),
                None => [2, 1, 5, 6].map(|n| pack(&format!("Castle walls/castle_wall({}).png", n))) },
            // wall(4) and wall(3): the same runs with a postern door in them
            wall_posts: match &rd_castle { Some(c) => c.posts.clone(),
                None => [4, 3].map(|n| pack(&format!("Castle walls/castle_wall({}).png", n))) },
            castle_grand,
            keep_s,
            // a generated stone church, else one of the pack's tall keeps
            church_s: chosen((1..=2).map(|i| at(&format!("rd/buildings/church{}.png", i))).collect())
                .or_else(|| chosen([7, 8, 9].iter().map(|i| pack(&format!("Towers/Blue towers/castle_tower_blue({}).png", i))).collect())),
            guildhall_s: guildhall.or_else(|| at(&assets.building_sprite("blacksmith", "wood"))),
            rival_house: rival_house.or_else(|| at(&assets.building_sprite("house", "red"))),
            rival_woodhouse: rival_woodhouse.or_else(|| at(&assets.building_sprite("woodhouse", "red"))),
            priest_s: Some(figure(&at, "MinifolksHumans/Without Outline/MiniMage.png")),
            priest_walk: walk("rd/villagers/priest.png"),
            guard_walks: ["spear", "sword", "shield", "halberd"].iter().map(|g| walk(&format!("rd/guards/{}.png", g))).collect(),
            // the postings' and trades' own figures (assets/rd/people):
            // 48px stills resampled to villager height at export
            people: PEOPLE_SPRITES.iter()
                .filter_map(|&n| at(&format!("rd/people/{}.png", n)).map(|s| (n.to_string(), s)))
                .collect(),
            traveller_s: Some(figure(&at, "MinifolksHumans/Without Outline/MiniHorseMan.png")),
            rival_walks,
            traveller_walk: walk("rd/traveller/courier.png"),
            mounted,
            mounted_flip,
            guards,
            dog: rd("dog").or_else(|| Some(doubled(dog_sprite()))),
            cat: rd("cat").or_else(|| Some(doubled(cat_sprite()))),
            hens,
            // no fallback sprite: draw_beast's blocks stand in
            sheep: rd_set(&["sheep", "sheep2"]),
            goats: rd_set(&["goat", "goat2"]),
            gull: Some(gull_sprite()),
            details,
            details_k,
            works,
            works_winter,
            stocks: Some({
                // the stocks: two posts, a foot bar between them, a low
                // seat - brown timber on transparent ground
                let (w, h) = (12usize, 9usize);
                let mut px = vec![0u8; w * h * 4];
                let wood = |px: &mut Vec<u8>, x, y| {
                    let i = (y * w + x) * 4;
                    px[i] = 84; px[i + 1] = 60; px[i + 2] = 40; px[i + 3] = 255;
                };
                for y in 0..8 { wood(&mut px, 2, y); wood(&mut px, 9, y); }
                for x in 2..10 { wood(&mut px, x, 7); wood(&mut px, x, 8); }
                for x in 4..8 { wood(&mut px, x, 5); }
                Sprite { width: w, height: h, pixels: px }
            }),
            market_cross: at("rd/bits/market-cross.png"),
            boat: at("rd/bits/boat.png").or_else(|| {
                // a little hull: dark below, a thwart across, a lighter
                // gunwale - the drawn boat stands where the pack's silent
                let mut s = Sprite { width: 14, height: 8, pixels: vec![0; 14*8*4] };
                for x in 0..14 { for y in 2..8 {
                    let c = if y > 6 { (60, 45, 30, 255) } else if y < 4 { (100, 80, 55, 255) } else { (80, 60, 40, 255) };
                    s.pixels[(y*14+x)*4] = c.0; s.pixels[(y*14+x)*4+1] = c.1;
                    s.pixels[(y*14+x)*4+2] = c.2; s.pixels[(y*14+x)*4+3] = c.3;
                }}
                Some(s)
            }),
            maypole: at("rd/bits/maypole.png"),
            memorial_cross: at("rd/monuments/wayside-cross.png"),
            memorial_picture: at("rd/monuments/picture-stone.png"),
            memorial_obelisk: at("rd/monuments/obelisk.png"),
            memorial_trophy: at("rd/monuments/trophy.png"),
            memorial_column: at("rd/monuments/column.png"),
            memorial_cairns: (1..=2).filter_map(|i| at(&format!("rd/monuments/cairn{}.png", i))).collect(),
            stalls: bits("stall"),
            trader_stall: at("rd/bits/trader-stall.png"),
            well_s: at("rd/bits/well.png"),
            hall_s: at("rd/bits/meeting-hall.png"),
            smoke: bits("smoke"),
            bridges: [at("rd/bits/bridge_x.png"), at("rd/bits/bridge_y.png")],
            skeps: bits("skep"),
            graves: bits("grave"),
            tombstones: bits("tomb"),
            lychgate: bits("lychgate"),
            herbs: bits("herb"),
            shrooms: bits("shroom"),
            fires: bits("fire"),
            burnt: bits("burnt"),
        }
    }
}
