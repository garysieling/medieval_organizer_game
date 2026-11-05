//! Screen layout shared by the client (drawing) and the server (hit-testing
//! clicks and minimap picks): the window size, the iso tile size, the camera
//! maths and the minimap box. Public, so the client needs nothing private.

use crate::game::tilemap::TileMap;

pub const W: usize = 1440;

pub const H: usize = 960;

pub const MAP_W: usize = 1080;

pub const UI_X: usize = 1100;

pub const TILE_W: i32 = 28;

pub const TILE_H: i32 = 14;

/// Convert world tile coords to screen isometric position
/// Point the zoom scales about: the centre of the map view, so zooming
/// keeps what you're looking at on screen
pub const ZOOM_PIVOT: (f32, f32) = (MAP_W as f32 / 2.0, H as f32 / 2.0);

pub fn iso_to_screen_cam(tx: f32, ty: f32, cam_x: i32, cam_y: i32, zoom: f32) -> (i32, i32) {
    // position at zoom 1, then scaled about the view centre
    let bx = MAP_W as f32 / 2.0 + (tx - ty) * TILE_W as f32 + cam_x as f32;
    let by = -60.0 + (tx + ty) * TILE_H as f32 + cam_y as f32;
    let sx = ZOOM_PIVOT.0 + (bx - ZOOM_PIVOT.0) * zoom;
    let sy = ZOOM_PIVOT.1 + (by - ZOOM_PIVOT.1) * zoom;
    (sx.round() as i32, sy.round() as i32)
}

/// Conservative tile bounds whose ground sprites can reach the viewport.
/// The screen margins match draw_ground's sprite-overhang cull, then one tile
/// is added to absorb rounding at fractional zooms.
pub fn visible_tile_bounds(cam_x: i32, cam_y: i32, zoom: f32, width: usize, height: usize,
    map_width: i32, map_height: i32) -> (i32, i32, i32, i32) {
    let (mx, my) = ((2.0 * TILE_W as f32 * zoom).ceil(), (4.0 * TILE_H as f32 * zoom).ceil());
    let corners = [(-mx, -my), (width as f32 + mx, -my), (-mx, height as f32 + my),
        (width as f32 + mx, height as f32 + my)];
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (sx, sy) in corners {
        let u = ((sx - ZOOM_PIVOT.0) / zoom - cam_x as f32) / TILE_W as f32;
        let v = ((sy - ZOOM_PIVOT.1) / zoom + 60.0 - cam_y as f32) / TILE_H as f32;
        let (tx, ty) = ((u + v) * 0.5, (v - u) * 0.5);
        min_x = min_x.min(tx); min_y = min_y.min(ty);
        max_x = max_x.max(tx); max_y = max_y.max(ty);
    }
    let (x0, y0, x1, y1) = ((min_x.floor() as i32 - 40).max(0),
        (min_y.floor() as i32 - 40).max(0), (max_x.ceil() as i32 + 40).min(map_width - 1),
        (max_y.ceil() as i32 + 40).min(map_height - 1));
    (x0, y0, x1, y1)
}

/// Can nothing standing on screen point (sx, sy) reach the map view? A
/// piece draws up from its tile (tall towers ~260px at zoom 1), a little
/// below it, and at most ~150px to either side
pub fn off_view(sx: i32, sy: i32, zoom: f32) -> bool {
    let z = |v: f32| (v * zoom).round() as i32;
    sx < -z(150.0) || sx > MAP_W as i32 + z(150.0) || sy < -z(40.0) || sy > H as i32 + z(300.0)
}

// Minimap: the same isometric projection as the map, shrunk to fit a box at
// the bottom of the side panel (so up on the minimap is up on screen)
pub const MM_W: i32 = 320;

pub const MM_H: i32 = 160;

pub const MM_LEFT: i32 = UI_X as i32 + 10;

pub const MM_TOP: i32 = 660;

/// Minimap px per tile step along the x/y diagonal: as small as the box
/// needs so the whole map fits whatever its size
pub fn mm_k(w_t: f32, h_t: f32) -> f32 {
    let span = (w_t + h_t - 2.0).max(1.0);
    ((MM_W - 24) as f32 / span).min((MM_H - 16) as f32 * 2.0 / span)
}

/// Minimap position of a world tile
pub fn tile_to_minimap(tx: f32, ty: f32, k: f32) -> (f32, f32) {
    (MM_LEFT as f32 + MM_W as f32 / 2.0 + (tx - ty) * k, MM_TOP as f32 + 2.0 + (tx + ty) * k / 2.0)
}

/// The find box under the minimap: type a name or a posting ("heart
/// helper") and Enter walks the camera to who fits
pub const SRCH_X: i32 = MM_LEFT;
pub const SRCH_Y: i32 = MM_TOP + MM_H + 4;
pub const SRCH_W: i32 = MM_W;
pub const SRCH_H: i32 = 20;

/// Is the point on the find box?
pub fn search_box_at(mx: i32, my: i32) -> bool {
    mx >= SRCH_X && mx < SRCH_X + SRCH_W && my >= SRCH_Y && my < SRCH_Y + SRCH_H
}

/// The tile under minimap pixel (mx, my), if it's on the minimap's tiles
/// (inverse of tile_to_minimap)
/// Where a minimap click centres the camera, for any map (sim's or a model's)
pub fn minimap_look_map(map: &TileMap, mx: i32, my: i32) -> Option<(i32, i32)> {
    minimap_tile(mx as f32, my as f32, map.width as f32, map.height as f32)
        .map(|(tx, ty)| camera_on(tx, ty))
}

pub fn minimap_tile(mx: f32, my: f32, w_t: f32, h_t: f32) -> Option<(f32, f32)> {
    let k = mm_k(w_t, h_t);
    let (cx, top) = (MM_LEFT as f32 + MM_W as f32 / 2.0, MM_TOP as f32 + 2.0);
    let diff = mx - cx;
    let sum = (my - top) * 2.0;
    let (tx, ty) = ((sum / k + diff / k) / 2.0, (sum / k - diff / k) / 2.0);
    let inside = mx >= MM_LEFT as f32 && mx < (MM_LEFT + MM_W) as f32 && my >= MM_TOP as f32 && my < (MM_TOP + MM_H) as f32;
    (inside && tx >= 0.0 && ty >= 0.0 && tx < w_t && ty < h_t).then_some((tx, ty))
}

/// The camera offset that puts tile (tx, ty) in the middle of the map view
/// (at any zoom: the zoom pivots about the view's centre)
pub fn camera_on(tx: f32, ty: f32) -> (i32, i32) {
    (-((tx - ty) * TILE_W as f32).round() as i32, (ZOOM_PIVOT.1 + 60.0 - (tx + ty) * TILE_H as f32).round() as i32)
}

/// Font glyph width and height (ui::font)
pub const GLYPH_W: i32 = 6;

pub const GLYPH_H: i32 = 7;

/// Word-wrap `text` into lines of at most `max` characters, breaking at
/// spaces (a word longer than a whole line is split)
pub fn wrap_text(text: &str, max: usize) -> Vec<String> {
    let mut lines = vec![];
    let mut line = String::new();
    for word in text.split_whitespace() {
        let mut word = word.to_string();
        while word.chars().count() > max {
            if !line.is_empty() { lines.push(std::mem::take(&mut line)); }
            let head: String = word.chars().take(max).collect();
            word = word.chars().skip(max).collect();
            lines.push(head);
        }
        if line.is_empty() {
            line = word;
        } else if line.chars().count() + 1 + word.chars().count() <= max {
            line.push(' ');
            line.push_str(&word);
        } else {
            lines.push(std::mem::replace(&mut line, word));
        }
    }
    if !line.is_empty() { lines.push(line); }
    lines
}
