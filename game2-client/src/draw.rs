//! Draw helpers for the map view: figures, props, bubbles, labels, colours, the minimap and weather.

use super::*;

/// Draw `sprite` with its top-left at (x, y), scaled by `scale`
pub(crate) fn draw_zoomed(buf: &mut Buffer, sprite: &Sprite, x: i32, y: i32, scale: f32) {
    if (scale - 1.0).abs() < 0.01 {
        sprite.blit(buf, x, y);
    } else if (scale - 2.0).abs() < 0.01 {
        sprite.blit_scaled(buf, x, y, 2);
    } else {
        let (w, h) = ((sprite.width as f32 * scale).round().max(1.0) as usize, (sprite.height as f32 * scale).round().max(1.0) as usize);
        sprite.blit_resized(buf, x, y, w, h);
    }
}

/// Where a floating inspector's grave portrait stands, as offsets from
/// the card's old panel slot: the view's lx/ly shift them to the card
pub(crate) const PORTRAIT_FOOT: (i32, i32) = (38, 384);
/// Villager name colours on the map: within MEETING_REACH of the player
/// (Talk, Recruit and Organize can reach them) or not
pub(crate) const NAME_IN_REACH: (u8, u8, u8) = (255, 255, 255);

pub(crate) const NAME_OUT_OF_REACH: (u8, u8, u8) = (150, 150, 165);

/// The org mate's pip: a gold that keeps clear of the heart red, the
/// in-reach white and the ask mark's yellow
pub(crate) const ORG_GOLD: (u8, u8, u8) = (212, 175, 55);

/// Gravestone colours
pub(crate) const GRAVE_STONE: (u8, u8, u8) = (135, 135, 145);

pub(crate) const GRAVE_EDGE: (u8, u8, u8) = (55, 55, 65);

/// A ready herb patch: a little spray of flowers low on the meadow tile
pub(crate) fn herb_tuft(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    // green sprigs, then petal dots above them
    for (dx, dy) in [(-3, 0), (0, -1), (3, 0), (-1, -2), (1, -2)] {
        buf.rect(sx + k(dx), sy + k(dy), 1.max(k(1)), 1.max(k(1)), 70, 145, 75);
    }
    for (dx, dy, c) in [(-3, -3, (240, 225, 245)), (1, -4, (205, 165, 235)), (3, -2, (245, 240, 200))] {
        buf.rect(sx + k(dx), sy + k(dy), 1.max(k(1)), 1.max(k(1)), c.0, c.1, c.2);
    }
}

/// A wild mushroom: a little cap on a pale stem
pub(crate) fn shroom_tuft(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    buf.rect(sx - k(2), sy - k(4), 1.max(k(5)), 1.max(k(3)), 195, 70, 55);
    buf.rect(sx - k(1), sy - k(1), 1.max(k(3)), 1.max(k(2)), 235, 225, 200);
}

/// A woven skep: a small domed hive with banding
pub(crate) fn draw_skep(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    buf.rect(sx - k(4), sy - k(7), 1.max(k(8)), 1.max(k(6)), 210, 185, 120);
    buf.rect(sx - k(3), sy - k(9), 1.max(k(6)), 1.max(k(2)), 220, 198, 140);
    for row in [4, 6] { buf.rect(sx - k(4), sy - k(row), 1.max(k(8)), 1.max(k(1)), 160, 135, 85); }
}

/// A swarm on the wing: a knot of dark specks
pub(crate) fn draw_swarm(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    for (dx, dy) in [(-3, -8), (0, -10), (3, -8), (-1, -6), (2, -11), (-4, -10)] {
        buf.rect(sx + k(dx), sy + k(dy), 1.max(k(2)), 1.max(k(1)), 45, 35, 20);
    }
}

/// A grazing animal: a wool or hair blob low on four legs
pub(crate) fn draw_beast(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32, sheep: bool) {
    let k = |v: i32| ((v as f32) * zoom).round() as i32;
    let (body, dark) = if sheep { ((235, 230, 212), (205, 198, 180)) } else { ((148, 132, 110), (104, 92, 75)) };
    buf.rect(sx - k(5), sy - k(8), 2.max(k(10)), 1.max(k(5)), body.0, body.1, body.2);
    buf.rect(sx + k(4), sy - k(9), 1.max(k(3)), 1.max(k(4)), dark.0, dark.1, dark.2);
    for lx in [-3, -1, 1, 3] { buf.rect(sx + k(lx), sy - k(3), 1.max(k(1)), 1.max(k(3)), dark.0, dark.1, dark.2); }
}

/// Draw an animal sprite (1px art, so 1x zoom) standing on screen point
/// (sx, sy); returns its height on screen, for a label above it
pub(crate) fn draw_animal(buf: &mut Buffer, sprite: &Sprite, sx: i32, sy: i32, zoom: f32) -> i32 {
    let (w, h) = ((sprite.width as f32 * zoom).round() as i32, (sprite.height as f32 * zoom).round() as i32);
    draw_zoomed(buf, sprite, sx - w / 2, sy - h, zoom);
    h
}

/// One of a set of looks, the same one for the same animal every frame
pub(crate) fn variant(set: &[Sprite], key: Entity) -> Option<&Sprite> {
    if set.is_empty() { None } else { set.get(key as usize % set.len()) }
}

/// Draw a gravestone standing on screen point (sx, sy) at the current zoom;
/// returns the y of its top, for the name label
pub(crate) fn draw_grave(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32) -> i32 {
    let z = |v: f32| (v * zoom).round() as i32;
    let (w, h) = (z(12.0).max(4), z(16.0).max(5));
    let (x, top) = (sx - w / 2, sy - h);
    // outline, then the stone with a rounded head, then a carved cross
    buf.rect(x - 1, top + 2, w + 2, h - 1, GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    buf.rect(x + 1, top, w - 2, 3, GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    buf.rect(x, top + 2, w, h - 2, GRAVE_STONE.0, GRAVE_STONE.1, GRAVE_STONE.2);
    buf.rect(x + 2, top + 1, w - 4, 2, GRAVE_STONE.0, GRAVE_STONE.1, GRAVE_STONE.2);
    buf.rect(sx - 1, top + z(3.0), 2, z(8.0).max(3), GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    buf.rect(sx - z(3.0), top + z(5.0), z(6.0).max(2), 2, GRAVE_EDGE.0, GRAVE_EDGE.1, GRAVE_EDGE.2);
    top
}

/// Draw a character sprite (a 32px frame drawn 2x at zoom 1) standing on
/// screen point (sx, sy), at the current zoom
pub(crate) fn draw_character(buf: &mut Buffer, sprite: &Sprite, sx: i32, sy: i32, zoom: f32) {
    draw_zoomed(buf, sprite, sx - (32.0 * zoom).round() as i32, sy - (60.0 * zoom).round() as i32, 2.0 * zoom);
}

/// Screen y of a character's head at the current zoom
pub(crate) fn head_y(sy: i32, zoom: f32) -> i32 { sy + (SPRITE_HEAD as f32 * zoom).round() as i32 }

/// A picket sign held beside the figure: the flat-rect stand-in for sign
/// art until generated boards land. `design` 0 = wooden hand-lettered,
/// 1 = printed placard. (sx, sy) is the figure's feet anchor.
pub(crate) fn draw_picket_sign(buf: &mut Buffer, sx: i32, sy: i32, zoom: f32, design: u8) {
    let z = |v: f32| (v * zoom).round() as i32;
    let bx = sx + z(11.0);
    let by = sy - z(34.0);
    let (bw, bh) = (z(11.0).max(4), z(9.0).max(3));
    // the stick in the fist, then the board: rim, face, a mark of writing
    buf.rect(bx + bw / 2 - 1, by + bh - 1, 2, z(13.0).max(4), 105, 70, 40);
    let (edge, face) = if design == 1 { ((70, 70, 95), (228, 218, 196)) }
        else { ((95, 60, 30), (165, 115, 65)) };
    buf.rect(bx - 1, by - 1, bw + 2, bh + 2, edge.0, edge.1, edge.2);
    buf.rect(bx, by, bw, bh, face.0, face.1, face.2);
    let ink = if design == 1 { (80, 100, 150) } else { (60, 35, 15) };
    buf.rect(bx + 2, by + 2, (bw - 4).max(2), 1, ink.0, ink.1, ink.2);
    buf.rect(bx + 2, by + 4, (bw - 6).max(1), 1, ink.0, ink.1, ink.2);
}

/// The point to hand name_label_pos so a label sits just above a zoomed head
pub(crate) fn label_anchor(sy: i32, zoom: f32) -> i32 { head_y(sy, zoom) - SPRITE_HEAD }

/// How far below SPRITE_HEAD this sprite's head actually starts, in screen
/// pixels at zoom 1: some figures (the old man) have blank rows above
/// row 18, which left their name floating
pub(crate) fn head_drop(sprite: &Sprite) -> i32 {
    let w = sprite.width as usize;
    let top = (0..sprite.height as usize)
        .find(|&y| (0..w).any(|x| sprite.pixels[(y * w + x) * 4 + 3] > 0))
        .unwrap_or(18) as i32;
    (top - 18).max(0) * 2
}

/// label_anchor for a particular sprite, so its name sits on its own head
pub(crate) fn sprite_label_anchor(sprite: &Sprite, sy: i32, zoom: f32) -> i32 {
    label_anchor(sy, zoom) + (head_drop(sprite) as f32 * zoom).round() as i32
}

/// Height of a speech bubble
pub(crate) const BUBBLE_H: i32 = 18;

/// Where a speech bubble goes for a speaker standing on screen point
/// (sx, sy): just above their (zoomed) name label, kept inside the map view.
/// None when the speaker isn't on the map view at all.
/// `label_y` is where the speaker's name label actually ended up: place_labels
/// nudges crowded labels upward, and the bubble must clear the nudged one
pub(crate) fn bubble_rect(sx: i32, sy: i32, zoom: f32, text: &str, label_y: Option<i32>) -> Option<(i32, i32, i32)> {
    if sx < 0 || sx >= MAP_W as i32 || sy < 0 || sy >= H as i32 + 60 { return None; }
    let bw = (text.len() as i32 * GLYPH_W + 12).max(60).min(MAP_W as i32 - 8);
    let bx = (sx - bw / 2).clamp(4, MAP_W as i32 - 4 - bw);
    let label_top = (head_y(sy, zoom) - GLYPH_H - LABEL_GAP).min(label_y.map(|y| y - 4).unwrap_or(i32::MAX));
    let by = (label_top - BUBBLE_H - 8).max(4);
    Some((bx, by, bw))
}

/// Top of a character's head relative to its ground point. Characters are
/// 32px frames drawn 2x from (sx-32, sy-60), but the figure only fills the
/// frame's bottom rows (18-31), so the head is at about sy-24.
pub(crate) const SPRITE_HEAD: i32 = -24;


/// Space between a name label and the head below it (the text outline
/// adds one more pixel)
pub(crate) const LABEL_GAP: i32 = 1;

pub(crate) fn name_label_pos(sx: i32, sy: i32, text: &str) -> (i32, i32) {
    (sx - text.len() as i32 * GLYPH_W / 2, sy + SPRITE_HEAD - GLYPH_H - LABEL_GAP)
}

/// The bobbing gold dart over the player's head - the one mark that says
/// "this one is you" even when the crowd presses the name away. `top` is
/// the y just above where the name label sits
pub(crate) fn draw_player_marker(buf: &mut Buffer, cx: i32, top: i32, zoom: f32, frame: u32) {
    let bob = ((frame as f32 * 0.12).sin() * 2.0 * zoom).round() as i32;
    let cy = top - (9.0 * zoom).round() as i32 - bob;
    let (hw, hh) = ((5.0 * zoom).round().max(2.0) as i32, (7.0 * zoom).round().max(3.0) as i32);
    buf.diamond(cx, cy, hw + 1, hh + 1, 30, 22, 4);
    buf.diamond(cx, cy, hw, hh, 255, 215, 0);
}


/// Base colour of a terrain tile by height - shared by the map and minimap
/// so they agree on what is water, grass, fields, hills and peaks
pub(crate) fn terrain_colour(h: f32) -> (u8, u8, u8) {
    if h < 0.15 { (30, 60, 180) }
    else if h < 0.35 { (60, 140, 60) }
    else if h < 0.55 { (140, 110, 50) }
    else if h < 0.75 { (110, 90, 60) }
    else { (220, 220, 220) }
}

/// A pixel under rain: dimmed and cooled, like light through rain cloud
pub(crate) fn rain_wash(c: (u8, u8, u8)) -> (u8, u8, u8) {
    blend(c, (40, 55, 80), 0.25)
}

/// Mix colour `a` toward `b` by `t` (0 = all a, 1 = all b)
pub(crate) fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

/// A tile's colour in the current season: snow lies on the land in winter
/// and the water turns cold, grass goes gold in autumn and fresh in spring,
/// summer is the base palette. Shared by the map and the minimap.
pub(crate) fn season_colour(h: f32, season: Season) -> (u8, u8, u8) {
    let base = terrain_colour(h);
    let water = h < 0.15;
    let peak = h >= 0.75;
    match season {
        Season::Summer => base,
        Season::Spring if water || peak => base,
        Season::Spring => blend(base, (70, 170, 70), 0.25),
        Season::Autumn if water || peak => base,
        Season::Autumn => blend(base, (185, 125, 45), 0.45),
        Season::Winter if water => blend(base, (60, 80, 120), 0.5),
        Season::Winter => blend(base, (228, 232, 240), 0.65),
    }
}

/// A map tile's colour in a season: the minimap's, and the map's own when
/// a tile's block art is missing. Open ground follows season_colour.
pub(crate) fn tile_colour(tile: Tile, season: Season) -> (u8, u8, u8) {
    let land = |c: (u8, u8, u8)| match season {
        Season::Winter => blend(c, (228, 232, 240), 0.65),
        _ => c,
    };
    match tile {
        Tile::Water | Tile::River => season_colour(0.05, season),
        Tile::Grass => season_colour(0.25, season),
        Tile::Hill => season_colour(0.45, season),
        Tile::Rock => season_colour(0.65, season),
        Tile::Wheat => land((220, 185, 80)),
        Tile::Corn => land((200, 160, 60)),
        Tile::Road => land((229, 143, 77)),
        Tile::Bridge => (229, 143, 77),
    }
}

/// Minimap dot colours: living villagers, and the graves of the dead
pub(crate) const MM_VILLAGER: (u8, u8, u8) = (255, 80, 80);

pub(crate) const MM_GRAVE: (u8, u8, u8) = (150, 150, 160);

/// Minimap position of a point on screen (inverse of iso_to_screen_cam,
/// then tile_to_minimap) - used to outline what the screen is showing
pub(crate) fn screen_to_minimap(sx: f32, sy: f32, cam_x: i32, cam_y: i32, zoom: f32, w_t: f32, h_t: f32) -> (f32, f32) {
    // undo the zoom about the view centre, then the zoom-1 projection
    let bx = ZOOM_PIVOT.0 + (sx - ZOOM_PIVOT.0) / zoom;
    let by = ZOOM_PIVOT.1 + (sy - ZOOM_PIVOT.1) / zoom;
    let diff = (bx - MAP_W as f32 / 2.0 - cam_x as f32) / TILE_W as f32; // tx - ty
    let sum = (by + 60.0 - cam_y as f32) / TILE_H as f32;                 // tx + ty
    tile_to_minimap((sum + diff) / 2.0, (sum - diff) / 2.0, mm_k(w_t, h_t))
}

/// Nudge name labels so none overlap: each label (x, y, width, height)
/// keeps its x and, if it would sit on a label already placed, moves up
/// by its own height at a time (at most a few moves) until it's clear.
/// `y` is the tag's top - for a name with hearts over it, the hearts'
/// top, so the whole tag collides as one. Labels nearer the bottom of
/// the screen (characters in front) are placed first and keep their spot.
pub(crate) fn place_labels(labels: &[(i32, i32, i32, i32)]) -> Vec<i32> {
    let mut order: Vec<usize> = (0..labels.len()).collect();
    order.sort_by_key(|&i| -(labels[i].1 + labels[i].3));
    let mut placed: Vec<(i32, i32, i32, i32)> = vec![];
    let mut ys = vec![0; labels.len()];
    for i in order {
        let (x, mut y, w, h) = labels[i];
        let hits = |y: i32, placed: &[(i32, i32, i32, i32)]| placed.iter().any(|&(px, py, pw, ph)| {
            x < px + pw + 2 && px < x + w + 2 && y < py + ph && py < y + h
        });
        for _ in 0..4 {
            if !hits(y, &placed) { break; }
            y -= h;
        }
        placed.push((x, y, w, h));
        ys[i] = y;
    }
    ys
}

/// Marks drawn by villagers on the map: (glyph, colour, meaning). The
/// panel's legend lists them, so the map and the key can't drift apart.
pub(crate) const MARK_HUNGRY: (&str, (u8, u8, u8), &str) = ("!", (255, 100, 100), "hungry");

pub(crate) const MARK_FOOD: (&str, (u8, u8, u8), &str) = ("o", (255, 200, 100), "finding food");

pub(crate) const MARK_BUSY: (&str, (u8, u8, u8), &str) = ("*", (180, 180, 255), "busy");

pub(crate) const MARK_SICK: (&str, (u8, u8, u8), &str) = ("+", (140, 230, 110), "sick");

pub(crate) const MAP_MARKS: [(&str, (u8, u8, u8), &str); 4] = [MARK_HUNGRY, MARK_FOOD, MARK_BUSY, MARK_SICK];

/// The legend's row, between the log box and the bottom of the window
pub(crate) const LEGEND_Y: i32 = 627;

/// Green when high, amber in the middle, red when low (0..=1)
pub(crate) fn meter_colour(frac: f32) -> (u8, u8, u8) {
    if frac >= 0.6 { (90, 200, 90) } else if frac >= 0.3 { (230, 180, 60) } else { (230, 80, 70) }
}

/// One heart, 7x6, filled left-to-right by `fill` (0..=1): the dark
/// silhouette always shows, so an empty heart still reads as a heart
const HEART_ROWS: [u8; 6] = [0b0110110, 0b1111111, 0b1111111, 0b0111110, 0b0011100, 0b0001000];

/// A heart strip's metrics: each heart is HEART_W wide on an 8px pitch
pub(crate) const HEART_W: i32 = 7;
pub(crate) const HEART_H: i32 = HEART_ROWS.len() as i32;
pub(crate) const HEART_PITCH: i32 = 8;

pub(crate) fn draw_heart(buf: &mut Buffer, x: i32, y: i32, fill: f32) {
    let lit = (fill.clamp(0.0, 1.0) * 7.0).round() as i32;
    for (row, bits) in HEART_ROWS.iter().enumerate() {
        for col in 0..7 {
            if bits & (1 << (6 - col)) == 0 { continue; }
            let (r, g, b) = if col < lit { (232, 92, 112) } else { (72, 52, 62) };
            buf.rect(x + col, y + row as i32, 1, 1, r, g, b);
        }
    }
}

/// A strip of `MAX` hearts filled to `hearts` (0..=MAX, half steps)
pub(crate) fn draw_hearts(buf: &mut Buffer, x: i32, y: i32, hearts: f32, max: usize) {
    for i in 0..max {
        draw_heart(buf, x + i as i32 * HEART_PITCH, y, hearts - i as f32);
    }
}

/// A small labelled meter: label, bar filled to `frac`, then a note
pub(crate) fn draw_meter(buf: &mut Buffer, x: i32, y: i32, label: &str, frac: f32, note: &str) {
    const BAR_X: i32 = 44;
    const BAR_W: i32 = 70;
    draw_text(buf, x, y, label, 200, 200, 200);
    buf.rect(x + BAR_X, y, BAR_W, 6, 30, 30, 45);
    let (r, g, b) = meter_colour(frac);
    buf.rect(x + BAR_X, y, (frac.clamp(0.0, 1.0) * BAR_W as f32) as i32, 6, r, g, b);
    draw_text(buf, x + BAR_X + BAR_W + 6, y, note, r, g, b);
}

pub(crate) fn draw_button(buf: &mut Buffer, b: &Button, hover: bool, disabled: bool) {
    let (mut r, mut g, mut bl) = b.color;
    if disabled { r = 50; g = 50; bl = 55; }
    else if hover { r = r.saturating_add(40); g = g.saturating_add(40); bl = bl.saturating_add(40); }
    draw_rect(buf, b.x, b.y, b.w, b.h, r, g, bl);
    draw_rect(buf, b.x, b.y, b.w, 1, 255, 255, 255);
    draw_rect(buf, b.x, b.y + b.h - 1, b.w, 1, 20, 20, 20);
    // label centered
    let tw = b.label.len() as i32 * 6;
    draw_text(buf, b.x + (b.w - tw) / 2, b.y + (b.h - 7) / 2, b.label, 255, 255, 255);
    // key hint
    let key = format!("{}", b.key);
    draw_text(buf, b.x + b.w - 10, b.y + 2, &key, 180, 180, 200);
}

/// The pack's ground blocks are 94px wide; the top face inside its outline
/// is about 91x50, starting 1.5px down. A block is scaled so that face is
/// exactly one map tile (neighbours' outlines overlap, as in the pack).
pub(crate) const BLOCK_FACE: (f32, f32, f32) = (91.0, 50.0, 1.5);

/// Size and top-left of the block drawn for the tile centred on (sx, sy)
pub(crate) fn block_rect(block: &Sprite, sx: i32, sy: i32, zoom: f32) -> (usize, usize, i32, i32) {
    let kx = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
    let ky = 2.0 * TILE_H as f32 * zoom / BLOCK_FACE.1;
    let (w, h) = ((block.width as f32 * kx).round() as usize, (block.height as f32 * ky).round() as usize);
    (w, h, sx - w as i32 / 2, sy - (TILE_H as f32 * zoom + BLOCK_FACE.2 * ky).round() as i32)
}

/// The scale the pack's props (trees, rocks) draw at on zoom 1: its block scale
pub(crate) const PACK_PROP_K: f32 = 2.0 * TILE_W as f32 / BLOCK_FACE.0;

/// How far past one tile a building's base reaches: at a single tile wide
/// the cottages read smaller than the folk walking in front of them
pub(crate) const BUILDING_SCALE: f32 = 2.0;

/// Draw a building standing on the tile centred on (sx, sy): scaled so its
/// base spans BUILDING_SCALE tiles, its front corner on the tile's front corner
pub(crate) fn draw_on_tile(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32) {
    let w = (2.0 * TILE_W as f32 * zoom * BUILDING_SCALE).round() as usize;
    let h = (s.height as f32 * w as f32 / s.width as f32).round().max(1.0) as usize;
    s.blit_resized(buf, sx - w as i32 / 2, sy + (TILE_H as f32 * zoom).round() as i32 - h as i32, w, h);
}

/// A shop shown open: the building, then its front's inside. The press
/// wears the era the village's research has reached - the hand bench,
/// Gutenberg's screw press, Senefelder's litho stone, the iron machines
/// (game2_core::game::tech_tree::press_era on the tree in the model)
pub(crate) fn draw_cutaway(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32,
    era: game2_core::game::tech_tree::PressEra) {
    draw_on_tile(buf, s, sx, sy, zoom);
    let w = (2.0 * TILE_W as f32 * zoom * BUILDING_SCALE).round() as i32;
    let h = (s.height as f32 * w as f32 / s.width as f32).round().max(1.0) as i32;
    let bottom = sy + (TILE_H as f32 * zoom).round() as i32;
    let (x0, y0) = (sx - w / 3, bottom - h * 3 / 5);
    let (rw, rh) = (2 * w / 3, h * 3 / 5 - h / 12);
    buf.rect(x0, y0, rw, rh, 36, 29, 22);                        // the room behind the front
    use game2_core::game::tech_tree::PressEra::*;
    match era {
        Hand => {
            // the hand worker's bench: an ink slab, the brayer, the sheet
            buf.rect(x0 + rw / 8, y0 + rh / 3, rw / 3, rh / 8, 30, 26, 22);
            buf.rect(x0 + rw / 8 + rw / 24, y0 + rh / 3 - rh / 16, rw / 8, rh / 16, 96, 70, 44);
            buf.rect(x0 + rw * 5 / 8, y0 + rh / 4, rw / 4, rh / 10, 232, 226, 204);
        }
        Screw => {
            // the type case: a dark tray of sorted type
            buf.rect(x0 + rw / 12, y0 + rh / 8, rw / 4, rh / 5, 54, 44, 34);
            for (i, c) in [(172, 62, 54), (60, 98, 170), (78, 150, 84)].iter().enumerate() {
                buf.rect(x0 + rw / 9 + i as i32 * rw / 7, y0 + rh / 6,
                    2.max(rw / 24), 2.max(rh / 50), c.0, c.1, c.2);
            }
            // the press: two posts, a crossbar, the bed and its platen
            let (px, py, pw, ph) = (x0 + rw * 2 / 5, y0 + rh / 10, rw / 4, rh / 2);
            buf.rect(px, py, rw / 16, ph, 98, 72, 46);
            buf.rect(px + pw, py, rw / 16, ph, 98, 72, 46);
            buf.rect(px - rw / 32, py, pw + rw / 8, rh / 14, 110, 84, 56);
            buf.rect(px - rw / 24, py + ph, pw + rw / 12, rh / 16, 70, 64, 66);
            buf.rect(px + rw / 32, py + ph - rh / 5, rw / 8, rh / 18, 52, 50, 56);
        }
        Litho => {
            // the litho stone: a flat slab with the drawing, the scraper, a water pot
            buf.rect(x0 + rw / 6, y0 + rh / 4, rw / 2, rh / 6, 168, 164, 156);
            buf.rect(x0 + rw / 6 + rw / 8, y0 + rh / 4 + rh / 24, rw / 3, rh / 24, 120, 116, 110);
            buf.rect(x0 + rw * 5 / 6 - rw / 10, y0 + rh / 3, rw / 10, rh / 8, 60, 90, 140);
            buf.rect(x0 + rw * 3 / 4, y0 + rh / 8, rw / 5, rh / 14, 96, 70, 44);
        }
        Machine => {
            // the iron press: a dark frame, the flywheel, the bed
            buf.rect(x0 + rw / 8, y0 + rh / 5, rw / 10, rh / 2, 52, 56, 66);
            buf.rect(x0 + rw * 5 / 6 - rw / 10, y0 + rh / 5, rw / 10, rh / 2, 52, 56, 66);
            buf.rect(x0 + rw / 8, y0 + rh / 5, rw * 3 / 4, rh / 12, 66, 70, 82);
            let fs = rh / 3;
            let (fx, fy) = (x0 + rw / 3, y0 + rh / 3);
            buf.rect(fx, fy, fs, fs, 40, 44, 54);
            buf.rect(fx + fs / 4, fy + fs / 4, fs / 2, fs / 2, 96, 100, 112);
            buf.rect(fx + fs / 3, fy + fs / 3, fs / 3, fs / 3, 40, 44, 54);
            buf.rect(x0 + rw / 6, y0 + rh * 3 / 4, rw * 2 / 3, rh / 12, 80, 76, 70);
        }
    }
    // a pile of fresh sheets
    buf.rect(x0 + rw * 5 / 6, y0 + rh / 5, rw / 6, rh / 10, 232, 226, 204);
    buf.rect(x0 + rw * 5 / 6 + rw / 40, y0 + rh / 5 - rh / 30, rw / 8, rh / 12, 240, 234, 214);
}

/// Draw a generated piece standing on the tile centred on (sx, sy) at its
/// own 1x size (no slab, so it's narrower than a tile and isn't stretched to
/// one): a footprint w wide is a w/4-deep diamond's front half, so its
/// bottom sits w/4 below the tile centre (at most the tile's front corner)
pub(crate) fn draw_standing(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32) {
    let (w, h) = ((s.width as f32 * zoom).round() as i32, (s.height as f32 * zoom).round() as i32);
    let bottom = sy + ((s.width as f32 / 4.0).min(TILE_H as f32) * zoom).round() as i32;
    draw_zoomed(buf, s, sx - w / 2, bottom - h, zoom);
}

/// Draw a piece of the town wall standing on the tile centred on (sx, sy):
/// scaled like the ground blocks (the pack draws its castle pieces for the
/// same 91x50 block face), so the piece's footprint is exactly the tile face
/// and its base lands on the tile's edges instead of overhanging them
pub(crate) fn draw_castle(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32) {
    let kx = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0;
    let ky = 2.0 * TILE_H as f32 * zoom / BLOCK_FACE.1;
    let (w, h) = ((s.width as f32 * kx).round() as usize, (s.height as f32 * ky).round().max(1.0) as usize);
    s.blit_resized(buf, sx - w as i32 / 2, sy + (TILE_H as f32 * zoom).round() as i32 - h as i32, w, h);
}

/// draw_castle stretched `wk` wide and `hk` tall: the pack castle set's
/// grander pieces (a wider one reaches past its tile's face)
pub(crate) fn draw_castle_wh(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32, wk: f32, hk: f32) {
    let kx = 2.0 * TILE_W as f32 * zoom / BLOCK_FACE.0 * wk;
    let ky = 2.0 * TILE_H as f32 * zoom / BLOCK_FACE.1 * hk;
    let (w, h) = ((s.width as f32 * kx).round() as usize, (s.height as f32 * ky).round().max(1.0) as usize);
    s.blit_resized(buf, sx - w as i32 / 2, sy + (TILE_H as f32 * zoom).round() as i32 - h as i32, w, h);
}

/// The castle dress against the village wall's fence-height pieces, on
/// the pack fallback path only (the generated set has fortress-scale
/// pieces of its own): the curtains and gatehouses stand twice as tall
/// (their width stays the tile's, so a run still joins up)
pub(crate) const GRAND_WALL_K: f32 = 1.9;
/// The corner towers take both dimensions - lone pieces, free to spread -
/// and the odd one stands loftier still (view.rs picks by tile)
pub(crate) const GRAND_TOWER_WK: f32 = 1.5;
pub(crate) const GRAND_TOWER_K: f32 = 2.2;
/// The keep over the walls: the tallest thing the country raises - kept
/// above the loftiest corner tower's GRAND_TOWER_K * 1.3 lift
pub(crate) const KEEP_WK: f32 = 1.8;
pub(crate) const KEEP_K: f32 = 3.1;

/// Where a windmill's sails turn: on the front of its tower, just below the
/// roof (MILL_HUB, in the tower sprite's pixels), for a tower drawn with
/// draw_on_tile on the tile centred on (sx, sy)
/// The generated windmill's hub on screen, its tower standing on (sx, sy)
/// as draw_standing puts it
pub(crate) fn mill_hub_rd(tower: &Sprite, sx: i32, sy: i32, zoom: f32) -> (i32, i32) {
    let (w, h) = ((tower.width as f32 * zoom).round() as i32, (tower.height as f32 * zoom).round() as i32);
    let bottom = sy + ((tower.width as f32 / 4.0).min(TILE_H as f32) * zoom).round() as i32;
    (sx - w / 2 + (MILL_HUB_RD.0 * zoom).round() as i32, bottom - h + (MILL_HUB_RD.1 * zoom).round() as i32)
}

pub(crate) fn mill_hub(tower: &Sprite, sx: i32, sy: i32, zoom: f32) -> (i32, i32) {
    let k = 2.0 * TILE_W as f32 * zoom / tower.width as f32;
    let (w, h) = ((2.0 * TILE_W as f32 * zoom).round() as i32, (tower.height as f32 * k).round() as i32);
    let top = sy + (TILE_H as f32 * zoom).round() as i32 - h;
    (sx - w / 2 + (MILL_HUB.0 * k).round() as i32, top + (MILL_HUB.1 * k).round() as i32)
}

pub(crate) const MILL_HUB: (f32, f32) = (24.5, 44.0);

/// Where the generated tower's sails turn, in its own pixels (the front of
/// the cap, assets/rd/windmill/tower.png)
pub(crate) const MILL_HUB_RD: (f32, f32) = (19.5, 31.0);

/// Index into Art::trades_winter
pub(crate) fn trade_index(b: BSprite) -> usize {
    match b { BSprite::Blacksmith => 0, BSprite::Guildhall => 1, BSprite::RivalHouse => 2, _ => 3 }
}

/// The sails' size against the tower's (the pack draws them about as tall)
pub(crate) const MILL_SAILS: f32 = 0.7;

/// The puffs of smoke over a home on the tile centred on (sx, sy), at frame
/// `frame`: (x, y, size, grey) for each, rising from the roof over
/// SMOKE_SECS, drifting right and shrinking as they go
pub(crate) fn smoke_puffs(frame: u32, sx: i32, sy: i32, zoom: f32) -> Vec<(i32, i32, i32, u8)> {
    let period = SMOKE_SECS * FPS;
    (0..3).map(|k| {
        let t = ((frame + k * period / 3) % period) as f32 / period as f32; // 0 at the chimney, 1 gone
        let x = sx as f32 + (10.0 + 8.0 * t) * zoom;
        let y = sy as f32 - (34.0 + 26.0 * t) * zoom;
        let size = ((4.0 - 2.5 * t) * zoom).round().max(1.0) as i32;
        (x as i32, y as i32, size, (150.0 + 70.0 * t) as u8)
    }).collect()
}

pub(crate) const SMOKE_SECS: u32 = 2;

/// Draw a tree or boulder standing in the middle of the tile centred on
/// (sx, sy), k times its size at zoom 1 (PACK_PROP_K for the pack's, 1 for
/// generated art)
pub(crate) fn draw_prop(buf: &mut Buffer, s: &Sprite, sx: i32, sy: i32, zoom: f32, k: f32) {
    let k = k * zoom;
    let (w, h) = ((s.width as f32 * k).round() as i32, (s.height as f32 * k).round() as i32);
    draw_zoomed(buf, s, sx - w / 2, sy + (3.0 * zoom) as i32 - h, k);
}

/// Which villager figure a person wears: grown folk cycle the whole
/// pack; children take only its two youngest - a peasant lad or lass -
/// so no elder's beard or noble's poise lands on a kid
pub(crate) fn figure_look(kid: bool, i: usize, n: usize) -> usize {
    let n = n.max(1);
    if kid { i % n.min(2) } else { i % n }
}

/// Draw a child's figure - the pack has none, so they're a scaled-down
/// villager flagged "(child)" on the label
pub(crate) fn draw_kid(buf: &mut Buffer, sprite: &Sprite, sx: i32, sy: i32, zoom: f32) {
    draw_zoomed(buf, sprite, sx - (19.0 * zoom).round() as i32, sy - (36.0 * zoom).round() as i32, 1.2 * zoom);
}

/// Overcast, falling rain/snow and the post-rain rainbow over the map view.
/// `since_clear` is frames since rain stopped (drives the rainbow fade).
/// The terrain runs right up to the panel, so this covers to UI_X, not MAP_W.
pub(crate) fn draw_weather(buf: &mut Buffer, kind: Option<PrecipKind>, frame: u64, since_clear: Option<u64>) {
    // a clear sky draws nothing: don't scan a million pixels for a mask
    // only rain, snow and the rainbow ever use
    if kind.is_none() && since_clear.is_none() { return; }
    let mask = weather_fx::background_mask(buf, UI_X, (25, 25, 38));
    match kind {
        Some(k) => {
            if k.is_snow() {
                weather_fx::overcast(buf, UI_X, k);
            } else {
                // rain: an even, cool dimming, deeper the harder it falls.
                // Each channel maps on its own and the mapping is fixed for the
                // frame, so it's three 256-entry tables (the per-pixel float
                // blends with round() cost ~30ms a frame)
                let t = 0.4 + 0.6 * k.intensity();
                let lut: [[u8; 256]; 3] = std::array::from_fn(|ch| std::array::from_fn(|v| {
                    let v = v as u8;
                    let c = [(v, 0, 0), (0, v, 0), (0, 0, v)][ch];
                    let (r, g, b) = blend(c, rain_wash(c), t);
                    [r, g, b][ch]
                }));
                for y in 0..H {
                    let row = &mut buf.pixels[y * W * 4..(y * W + UI_X) * 4];
                    for px in row.chunks_exact_mut(4) {
                        px[0] = lut[0][px[0] as usize]; px[1] = lut[1][px[1] as usize]; px[2] = lut[2][px[2] as usize];
                    }
                }
            }
            weather_fx::draw_precip(buf, UI_X, &mask, k, frame);
        }
        None => if let Some(f) = since_clear {
            weather_fx::draw_rainbow(buf, UI_X, &mask, weather_fx::rainbow_alpha(f));
        },
    }
}


/// Split a log entry stored as "t120: message" into the day the player
/// sees ("day 120", matching the HUD) and the message
pub(crate) fn log_parts(entry: &str) -> (Option<String>, &str) {
    if let Some((stamp, msg)) = entry.split_once(": ") {
        if let Some(day) = stamp.strip_prefix('t').filter(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit())) {
            return (Some(format!("day {}", day)), msg);
        }
    }
    (None, entry)
}

/// Colour for a log entry: deaths red, guild events gold, joins green,
/// weather/illness cyan, idle gossip dimmed so real events stand out
pub(crate) fn log_colour(entry: &str) -> (u8, u8, u8) {
    if entry.contains("died") || entry.contains("dead") { (255, 100, 100) }
    else if is_gossip(entry) { (140, 140, 160) } // gossip, even when it mentions the guild
    else if entry.contains("Guild") || entry.contains("guild") { (255, 215, 0) }
    else if entry.contains("joined") { (120, 255, 120) }
    // whole words only: "influence" contains "flu" and "grain" contains "rain"
    else if entry.split(|c: char| !c.is_alphabetic()).any(|w| matches!(w, "storm" | "rain" | "raining" | "drizzle" | "showers" | "snow" | "rainbow" | "grippe" | "scarlatina" | "gripes" | "sweats" | "consumption" | "fever" | "cholera" | "ague")) { (120, 200, 255) }
    else { (210, 210, 210) }
}

pub(crate) fn draw_rect(buf: &mut Buffer, x: i32, y: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
    buf.rect(x, y, w, h, r, g, b);
}

pub(crate) const TOOLTIPS: [&str; 3] = [
    "talk to someone nearby",
    "recruit villager to your org",
    "commission a new house",
];
