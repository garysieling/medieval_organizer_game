//! The View: draws a UiModel. It never sees the Game - only what the seat's model carries.

use super::*;

/// The visible ground viewport, cached for a fixed camera, zoom, season and map revision.
/// Scaled ground blocks with their opaque runs, keyed by the source
/// block's address; valid for one zoom and tile set
#[derive(Default)]
pub struct BlockCache {
    zoom: f32,
    set: usize,
    blocks: game2_core::engine::det::HashMap<usize, (Sprite, Vec<(u16, u16, u16)>)>,
}

pub struct GroundCache {
    pub buf: Buffer,
    pub camera: (i32, i32),
    pub zoom: f32,
    pub season: Season, // the season it was drawn in (Spring and Summer share a tile set but not their fallback colours)
    pub rev: u64,       // map.rev() when it was drawn
}

/// A toast's screen life in model frames - about five seconds at 60fps,
/// whatever the watch speed (the sim's own expiry is days, for the log)
pub(crate) const TOAST_LIFE: u32 = 300;
/// The stretch at the end a toast fades over, in frames
pub(crate) const TOAST_FADE: u32 = 45;
/// How far a toast climbs over its life, px - under the pitch so it
/// never quite reaches the banner above
const TOAST_RISE: f32 = 10.0;
/// The feed's first banner top; each older one sits a pitch higher
const TOAST_TOP: i32 = 44;
const TOAST_PITCH: i32 = 30;
/// The LOG panel's wrap width in glyphs ((box width - padding) / glyph
/// width) and its row count - shared by the draw pass and log_hit's
/// row math so a click lands on the line that's drawn there (#645)
const LOG_CHARS: usize = 52;
const LOG_ROWS: usize = 8;
/// The LOG box's screen rect: x, y, w, h - hit-testing reads the same
const LOG_BOX: (i32, i32, i32, i32) = (UI_X as i32 + 10, 512, 320, 110);

/// Everything a frame needs that isn't the game itself: the sprite pack,
/// the camera and UI toggles, and the caches repainting costs too much to
/// redo every frame. One View per client (or per local player).
pub struct View {
    pub art: Art,
    pub cam_x: i32,
    pub cam_y: i32,
    pub zoom: f32,
    pub inspector_open: bool,
    pub help_open: bool,
    /// the village ledger (F3): the public KPI lines the session built
    pub ledger_open: bool,
    /// Where the mouse rests this frame - the flyer's hover reads it
    pub mouse: Option<(i32, i32)>,
    /// The village's tune silenced: M toggles it, the client's own
    /// choice, never the sim's
    pub mute: bool,
    /// The find box under the minimap: what was typed and whether it has
    /// the keys - widget state like the toggles above
    pub search_q: String,
    pub search_focus: bool,
    /// The frame stamp of the last sim camera-ask answered, so the same
    /// ask isn't answered twice
    pub focus_seen: std::cell::Cell<Option<u32>>,
    /// The ground layer repainted only when stale
    pub(crate) ground: std::cell::RefCell<Option<GroundCache>>,
    /// Ground blocks at the current zoom, as opaque runs (draw_ground):
    /// kept across frames so panning doesn't rescale them every pass
    blocks: std::cell::RefCell<BlockCache>,
    /// A finished frame handed back with recycle(), reused by the next
    /// render instead of a fresh 5.5MB buffer
    spare: std::cell::RefCell<Option<Buffer>>,
    last_zoom: std::cell::Cell<f32>,
    last_camera: std::cell::Cell<(i32, i32)>,
    /// The frame being drawn is winter: buildings use their snowy looks
    winter: std::cell::Cell<bool>,
    /// Walk-cycle phase per entity: ground covered, so a stroll and a
    /// sprint stride at their own pace instead of every figure
    /// moonwalking to the same beat. (last x, last y, tiles walked, frame)
    walk: std::cell::RefCell<game2_core::engine::det::HashMap<u32, (f32, f32, f32, u32)>>,
}

/// The mystery panel's display line, composed the way the sim used to
/// send it - a legacy bare-string entry (empty title) draws verbatim
fn mystery_line(mz: &MysteryV) -> String {
    if mz.title.is_empty() {
        mz.asker_name.clone()
    } else {
        format!("Mystery: {} - {}/{} whispers ({} wants to know)",
            mz.title, mz.heard, mz.total, mz.asker_name)
    }
}

/// Where the asker's name lands in the wrapped panel line: (row, first
/// char col, one past the last). The name rides inside the "(name" word,
/// which greedy wrapping places identically whether or not the suffix
/// follows, so wrapping just up to the name finds its row and column
fn mystery_name_col(mz: &MysteryV) -> Option<(usize, usize, usize)> {
    if mz.title.is_empty() || mz.asker_name.is_empty() { return None; }
    let prefix = format!("Mystery: {} - {}/{} whispers (", mz.title, mz.heard, mz.total);
    let lines = wrap_text(&format!("{}{}", prefix, mz.asker_name), 30);
    let (row, last) = lines.iter().enumerate().last()?;
    if row >= 3 { return None; }
    let byte = last.find(&format!("({}", mz.asker_name))?;
    let col = last[..byte].chars().count() + 1;
    Some((row, col, col + mz.asker_name.chars().count()))
}

impl View {
    /// A view of the game drawn with `art`, looking at the market to start
    pub fn new(art: Art) -> View {
        View {
            art,
            cam_x: camera_on(16.0, 16.0).0,
            cam_y: camera_on(16.0, 16.0).1,
            zoom: 1.0,
            inspector_open: true,
            help_open: false,
            ledger_open: false,
            mouse: None,
            mute: false,
            search_q: String::new(),
            search_focus: false,
            focus_seen: std::cell::Cell::new(None),
            ground: std::cell::RefCell::new(None),
            blocks: std::cell::RefCell::new(BlockCache::default()),
            spare: std::cell::RefCell::new(None),
            last_zoom: std::cell::Cell::new(f32::NAN),
            last_camera: std::cell::Cell::new((i32::MIN, i32::MIN)),
            winter: std::cell::Cell::new(false),
            walk: std::cell::RefCell::new(game2_core::engine::det::HashMap::default()),
        }
    }

    /// The `t` a walk sheet animates on, built from ground covered rather
    /// than wall-clock frames: `cycle` frames make one loop, a loop covers
    /// STRIDE_TILES of ground, and `t` counts the sheets' 6-tick steps.
    /// `frame` is the model's sim frame: renders can skip frames (a
    /// recording captures every few), so the teleport check divides the
    /// step by frames elapsed - anything faster than a sprint is a jump,
    /// not a stride, and doesn't spin the legs.
    pub(crate) fn walk_t(&self, e: u32, x: f32, y: f32, cycle: usize, frame: u32) -> u32 {
        const STRIDE_TILES: f32 = 0.5;
        /// tiles per sim frame anyone can legitimately cover (~9 t/s)
        const FASTEST: f32 = 0.15;
        let mut w = self.walk.borrow_mut();
        let st = w.entry(e).or_insert((x, y, (e as f32 * 0.618_033_9) % STRIDE_TILES, frame));
        let elapsed = frame.saturating_sub(st.3).max(1) as f32;
        let d = ((x - st.0).powi(2) + (y - st.1).powi(2)).sqrt();
        st.0 = x;
        st.1 = y;
        st.3 = frame;
        if d / elapsed <= FASTEST {
            st.2 += d;
        }
        (st.2 / STRIDE_TILES * 6.0 * cycle.max(1) as f32) as u32
    }

    /// Where each live toast draws and how faded it is: (index into
    /// `toasts`, x, y, w, fade). The feed runs top-down like a
    /// conveyor - a new toast pops in at the bottom and, as the oldest
    /// expires off the top, the rest move up a place; each also climbs
    /// a little and fades over its last frames. A toast's age is its
    /// own stamp against the model's frame - an older server that sends
    /// none reads as just arrived
    pub(crate) fn toast_layout(&self, toasts: &[ToastV], now: u32)
        -> Vec<(usize, i32, i32, i32, f32)> {
        let live: Vec<(usize, u32)> = toasts.iter().enumerate()
            .filter_map(|(i, t)| {
                let age = t.born.map(|b| now.wrapping_sub(b)).unwrap_or(0);
                (age < TOAST_LIFE).then_some((i, age))
            }).collect();
        live.iter().enumerate().map(|(slot, &(i, age))| {
            let t = &toasts[i];
            let tw = t.text.len() as i32 * 6 + 16;
            let rise = age as f32 / TOAST_LIFE as f32 * TOAST_RISE;
            let fade = ((TOAST_LIFE - age) as f32 / TOAST_FADE as f32).min(1.0);
            (i, MAP_W as i32 / 2 - tw / 2, TOAST_TOP + slot as i32 * TOAST_PITCH - rise as i32, tw, fade)
        }).collect()
    }

    /// Which toast the mouse is over - the same boxes render draws, so
    /// a click on a banner that has `at` can jump the camera there
    pub fn toast_hit(&self, m: &UiModel, mx: i32, my: i32) -> Option<usize> {
        self.toast_layout(&m.toasts, m.frame).iter()
            .find(|&&(_, x, y, w, _)| mx >= x && mx < x + w && my >= y && my < y + 20)
            .map(|&(i, ..)| i)
    }

    /// Which LOG-panel entry the mouse is over, as an index into `m.log` -
    /// only entries that carry a position or a villager answer; plain
    /// lines and the "earlier lines" marker are inert. The row math
    /// mirrors the draw pass: word-wrapped to LOG_CHARS, capped at
    /// log_rows, a click on any wrapped row of an entry counting (#645)
    pub fn log_hit(&self, m: &UiModel, mx: i32, my: i32) -> Option<usize> {
        let (lx, ly, lw, lh) = LOG_BOX;
        if mx < lx || mx >= lx + lw || my < ly + 18 || my >= ly + lh { return None; }
        let log_rows = if m.log_dropped > 0 { LOG_ROWS - 1 } else { LOG_ROWS };
        let mut row = 0usize;
        for (i, entry) in m.log.iter().enumerate() {
            let (day, msg) = log_parts(&entry.text);
            let text = match &day { Some(d) => format!("{} {}", d, msg), None => msg.to_string() };
            for _ in wrap_text(&text, LOG_CHARS) {
                if row == log_rows { return None; }
                let y = 530 + row as i32 * 12;
                if my >= y && my < y + 12 {
                    return (entry.at.is_some() || entry.who.is_some()).then_some(i);
                }
                row += 1;
            }
        }
        None
    }

    /// Which mystery-panel asker the mouse is over, as their villager
    /// index - only the name's own cells answer; the rest of the line is
    /// inert, and a legacy bare-string entry has no asker to click (#646)
    pub fn mystery_hit(&self, m: &UiModel, mx: i32, my: i32) -> Option<usize> {
        let mz = m.panel.mystery.as_ref()?;
        let asker = mz.asker?;
        let (row, c0, c1) = mystery_name_col(mz)?;
        let y = 68 + row as i32 * 11;
        let x0 = UI_X as i32 + 150 + c0 as i32 * GLYPH_W;
        let x1 = UI_X as i32 + 150 + c1 as i32 * GLYPH_W;
        (mx >= x0 && mx < x1 && my >= y && my < y + 11).then_some(asker)
    }

    /// The sprite a building draws with
    pub(crate) fn sprite_of(&self, b: BSprite) -> &Option<Sprite> {
        match b {
            BSprite::House(i) if self.winter.get() => self.art.house_winter(i),
            BSprite::House(i) => self.art.house(i),
            BSprite::Rowhouse(i) if self.winter.get() => self.art.house_winter(i),
            BSprite::Rowhouse(i) => self.art.house(i),
            BSprite::Mansion if self.winter.get() && self.art.trades_winter[1].is_some() =>
                &self.art.trades_winter[1],
            BSprite::Mansion => &self.art.guildhall_s,
            BSprite::Barracks(i) => {
                let set = if self.winter.get() { &self.art.works_winter[WorkLook::WatchHouse as usize] }
                    else { &self.art.works[WorkLook::WatchHouse as usize] };
                if set.is_empty() { self.sprite_of(BSprite::House(i)) } else { &set[i % set.len()] }
            }
            BSprite::Blacksmith | BSprite::Guildhall | BSprite::RivalHouse | BSprite::RivalWoodhouse
                if self.winter.get() && self.art.trades_winter[trade_index(b)].is_some() =>
                &self.art.trades_winter[trade_index(b)],
            BSprite::Blacksmith => &self.art.blacksmith_s,
            BSprite::Church => &self.art.church_s,
            BSprite::Guildhall => &self.art.guildhall_s,
            BSprite::RivalHouse => &self.art.rival_house,
            BSprite::RivalWoodhouse => &self.art.rival_woodhouse,
            BSprite::CastleKeep => &self.art.tower_s,
            BSprite::Work(w, i) => {
                let set = if self.winter.get() { &self.art.works_winter[w as usize] } else { &self.art.works[w as usize] };
                if set.is_empty() { self.sprite_of(BSprite::House(i)) } else { &set[i % set.len()] }
            }
        }
    }

    /// The frame: ground, everything standing on it (in depth order),
    /// then the side panel. Draws the model the sim produced - the same
    /// code a remote client runs on the frame it gets over the wire.
    pub fn render(&self, m: &UiModel, mouse: (i32, i32)) -> Buffer {
        let (mx, my) = mouse;
        let art = &self.art;
        let (tick, cam_x, cam_y, zoom) = (m.tick, self.cam_x, self.cam_y, self.zoom);
        self.winter.set(season_tiles(m.season) == 2);
        let mut prof = Prof::new();

        // a recycled frame needs no clearing: the ground (cached or drawn
        // over the background fill), the strip below and the panel cover
        // every pixel
        let mut buf = self.spare.borrow_mut().take().filter(|b| b.width == W && b.height == H).unwrap_or_else(|| Buffer::new(W, H));
        // the background: the map area gets it only when the ground is drawn
        // straight in (the cached ground carries its own); the gap between
        // the map and the panel always
        buf.rect(MAP_W as i32, 0, UI_X as i32 - MAP_W as i32, H as i32, 25, 25, 38);

        // terrain rendered in the SAME projection as entities (unified map).
        // The ground is thousands of tile blits, so it's painted once into a
        // world-space layer and each frame just blits the window the camera
        // shows; it's redrawn only when the zoom, the season or the map
        // itself has changed since the last pass.
        let season = m.season;
        let camera = (cam_x, cam_y);
        if self.last_zoom.get() != zoom || self.last_camera.get() != camera {
            // Pan and zoom input can change every frame. Draw only the visible
            // viewport while moving; rebuild its compact cache when settled.
            buf.rect(0, 0, MAP_W as i32, H as i32, 25, 25, 38);
            self.draw_ground(m, &mut buf, cam_x, cam_y, zoom);
        } else {
            let stale = match *self.ground.borrow() {
                Some(ref g) => g.zoom != zoom || g.camera != camera || g.season != season || g.rev != m.map.rev(),
                None => true,
            };
            if stale { *self.ground.borrow_mut() = Some(self.render_ground(m, zoom, camera)); }
            let g = self.ground.borrow();
            let g = g.as_ref().unwrap();
            buf.blit_from(&g.buf, 0, 0, 0, 0, MAP_W as i32, H as i32);
        }
        self.last_zoom.set(zoom);
        self.last_camera.set(camera);
        prof.mark("render.ground");

        // everything standing on the map, people included, drawn back to
        // front in one painter's-order pass: whatever is nearer covers what
        // is behind it (people used to be painted over the lot and the
        // occluders re-drawn on top, which flipped nearer pieces back behind
        // farther ones - glitched seams, e.g. on the town wall)
        #[derive(Clone, Copy)]
        enum Piece<'a> {
            Building(Option<&'a Sprite>, f32), // a house, and how much of it stands
            Cutaway(Option<&'a Sprite>, f32),  // a shop shown open: the building, then its front's inside
            Prop(Option<&'a Sprite>, f32),     // a tree or boulder in the middle of its tile, and its zoom-1 scale
            Castle(Option<&'a Sprite>, f32, f32),   // a town-wall block: wall, gatehouse or tower; (width, height) stretch - the castles' grand dress
            Herb,                              // a wild herb tuft, flat on the meadow
            Shroom,                            // a wild mushroom in the shade
            Cache,                             // a cache the woods' rebels left behind
            Skep,                              // a woven beehive in the apiary's yard
            Swarm,                             // a swarm of bees on the wing
            Stand(&'a Sprite),                 // a generated prop at its own size (draw_standing)
            Gaol,                             // a cell block or a barred door: dark timber, drawn
        }
        #[derive(Clone, Copy)]
        enum Who<'a> { P(&'a PersonV), Mill, Smoke(usize) }

        // a workplace nobody works stands dimmed - derelict
        let tint_cache: game2_core::engine::det::HashMap<usize, Option<Sprite>> = m.buildings.iter().enumerate()
            .filter(|(_, b)| b.derelict)
            .map(|(i, b)| (i, self.sprite_of(b.sprite).as_ref().map(|sp| tint(sp, (40, 40, 55), 0.6))))
            .collect();
        let mut standing: Vec<(f32, f32, Piece)> = vec![];
        for (i, b) in m.buildings.iter().enumerate() {
            let s = self.sprite_of(b.sprite);
            let sprite = if b.derelict { tint_cache[&i].as_ref().or(s.as_ref()) } else { s.as_ref() };
            standing.push((b.x, b.y, if b.cutaway { Piece::Cutaway(sprite, b.risen) } else { Piece::Building(sprite, b.risen) }));
        }
        standing.extend(m.settlement_houses.iter().map(|&(x, y, sprite)|
            (x, y, Piece::Building(self.sprite_of(sprite).as_ref(), 1.0))));
        // (the keep stands over the wall's towers, the tallest in the
        // country; on the generated path it's its own sprite at 1x, the
        // pack fallback keeps the stretch it always had)
        let keep = art.keep_s.as_ref().or(art.tower_s.as_ref());
        let (kwk, kkh) = if art.castle_rd { (1.0, 1.0) } else { (KEEP_WK, KEEP_K) };
        standing.extend(m.castle_keeps.iter().map(|&(x, y)|
            (x, y, Piece::Castle(keep, kwk, kkh))));
        // a wood is a mix of the pack's trees, stable per tile
        standing.extend(m.map.trees.iter().map(|&(x, y)| (x as f32, y as f32, Piece::Prop(pick(&art.tiles[season_tiles(season)].trees, x, y, 4), art.tiles[season_tiles(season)].trees_k))));
        // boulders and flat rocks, a different one each time
        let rock_prop = |x: i32, y: i32| if noise(x, y, 14) < 0.5 {
            pick(&art.stones, x, y, 16) } else { pick(&art.stone_grounds, x, y, 16) };
        standing.extend(m.stones.iter().map(|&(x, y)| (x as f32, y as f32, Piece::Prop(rock_prop(x, y), art.stones_k))));
        // wild herb tufts, ready for the picking
        // the small things: generated when there's art, else the old blocks
        fn small(set: &[Sprite], x: f32, y: f32, salt: u32) -> Option<&Sprite> { pick(set, (x * 7.0) as i32, (y * 7.0) as i32, salt) }
        standing.extend(m.herbs.iter().map(|&(x, y)| (x, y, small(&art.herbs, x, y, 51).map_or(Piece::Herb, Piece::Stand))));
        standing.extend(m.mush.iter().map(|&(x, y)| (x, y, small(&art.shrooms, x, y, 52).map_or(Piece::Shroom, Piece::Stand))));
        standing.extend(m.stolen.iter().map(|&(x, y)| (x, y, Piece::Cache)));
        standing.extend(m.skeps.iter().map(|&(x, y)| (x, y, small(&art.skeps, x, y, 53).map_or(Piece::Skep, Piece::Stand))));
        // the boats ride the river: a hull at the moored tile
        for &(bx, by) in &m.boats {
            if let Some(s) = art.boat.as_ref() {
                standing.push((bx as f32 + 0.5, by as f32 + 0.5, Piece::Stand(s)));
            }
        }
        // the board's own post: a placard at the market's edge - the
        // notices stand on it, the player reads them at a click
        if let Some((x, y)) = m.market {
            standing.push((x + 1.5, y + 1.0, Piece::Gaol));
        }
        // the market cross, or the maypole on a feast day
        if let Some((x, y)) = m.market {
            let s = if m.feast { art.maypole.as_ref().or(art.market_cross.as_ref()) } else { art.market_cross.as_ref() };
            if let Some(s) = s { standing.push((x, y, Piece::Stand(s))); }
        }
        if let Some((x, y)) = m.stocks {
            if let Some(s) = art.stocks.as_ref() { standing.push((x as f32, y as f32, Piece::Stand(s))); }
        }
        // the lychgate over the churchyard path, when the graveyard's
        // planning found its spot
        if let Some((x, y)) = m.lychgate {
            if let Some(s) = pick(&art.lychgate, x, y, 55) {
                standing.push((x as f32, y as f32, Piece::Stand(s)));
            }
        }
        for (x, y) in [m.village_gaol, m.castle_gaol].iter().flatten() {
            standing.push((*x as f32, *y as f32, Piece::Gaol));
        }
        // today's stalls round the square, and the pedlar's booth
        for &(x, y, look) in &m.stalls {
            let s = if look == routine::STALL_TRADER { art.trader_stall.as_ref() }
                else if art.stalls.is_empty() { None } else { Some(&art.stalls[look as usize % art.stalls.len()]) };
            if let Some(s) = s { standing.push((x, y, Piece::Stand(s))); }
        }
        // the footbridges: deck across the river, so along whichever axis
        // the water doesn't run
        for (x, y) in m.map.bridges() {
            let wet = |dx: i32, dy: i32| matches!(m.map.get(x + dx, y + dy), Some(Tile::River | Tile::Water));
            let along_y = wet(1, 0) || wet(-1, 0);
            if let Some(s) = art.bridges[along_y as usize].as_ref() { standing.push((x as f32, y as f32, Piece::Stand(s))); }
        }
        // what's alight: flames flickering between frames
        if !art.fires.is_empty() {
            for &(x, y) in &m.burning {
                let f = (m.frame as usize / 4 + (noise(x as i32, y as i32, 54) * 7.0) as usize) % art.fires.len();
                standing.push((x, y, Piece::Stand(&art.fires[f])));
            }
        }
        if let Some((wx, wy)) = m.swarm { standing.push((wx, wy, Piece::Swarm)); }
        // the land's score shows: stumps where the canopy went, the foul
        // reach below town, the pit filled to a pond
        for &(tx, ty) in &m.stumps {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(1.0), sy - k(4.0), k(2.0), k(4.0), 95, 75, 50);
            buf.rect(sx - k(2.0), sy - k(5.0), k(4.0), k(1.0), 120, 95, 65);
        }
        for &(tx, ty) in &m.murk {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 85, 70, 45);
        }
        if let Some((tx, ty)) = m.pond {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 60, 90, 120);
        }
        // something riding over the walls: a bright head and a fading
        // tail across the top of the frame
        if let Some(kind) = m.sky.as_deref() {
            let (hx, hy) = (buf.width as i32 * 2 / 3, buf.height as i32 / 8);
            let head = if kind == "comet" { (255, 245, 200) } else { (180, 60, 50) };
            for t in 0..14 {
                let (x, y) = (hx - t * 6, hy - t * 3);
                let f = 1.0 - t as f32 / 14.0;
                let (r, g, b) = ((head.0 as f32 * f) as u8, (head.1 as f32 * f) as u8, (head.2 as f32 * f) as u8);
                if x >= 0 && y >= 0 { buf.rect(x - 2, y - 1, 4, 3, r, g, b); }
            }
            buf.rect(hx - 3, hy - 3, 6, 6, 255, 250, 230);
        }
        // the cave mouth: a dark hollow at the map's edge, desaturated
        if let Some((cx, cy)) = m.cave {
            let (sx, sy) = iso_to_screen_cam(cx, cy, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(6.0), sy - k(6.0), k(12.0), k(7.0), 18, 18, 22);
            buf.rect(sx - k(4.0), sy - k(4.0), k(8.0), k(5.0), 8, 8, 10);
            buf.rect(sx - k(7.0), sy - k(7.0), k(14.0), k(1.0), 40, 40, 45);
        }
        // the martyr's plot: a grey cairn where they hanged him
        // the monuments stand at their spots: a pale column that names
        // the deed it keeps
        for &(mx, my, kind, ref _words) in &m.monuments {
            let (sx, sy) = iso_to_screen_cam(mx as f32, my as f32, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            if kind == 1 {
                // the org's ring: standing stones in a circle
                for i in 0..6 {
                    let a = (i as f32 * 1.0472).sin();
                    let b = (i as f32 * 1.0472).cos();
                    buf.rect(sx + (a * k(6.0) as f32) as i32 - k(1.0),
                        sy + (b * k(3.0) as f32) as i32 - k(4.0), k(2.0), k(4.0), 195, 190, 180);
                }
            } else {
                buf.rect(sx - k(2.0), sy - k(9.0), k(4.0), k(8.0), 210, 205, 195);
                buf.rect(sx - k(3.0), sy - k(2.0), k(6.0), k(2.0), 170, 165, 155);
            }
        }
        if let Some((gx, gy)) = m.grave {
            let (sx, sy) = iso_to_screen_cam(gx, gy, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(5.0), sy - k(4.0), k(10.0), k(4.0), 70, 70, 75);
            buf.rect(sx - k(3.0), sy - k(7.0), k(6.0), k(4.0), 96, 96, 102);
            buf.rect(sx - k(1.0), sy - k(9.0), k(2.0), k(4.0), 130, 130, 135);
        }
        // the painted halls: a bright trim above the door
        for &(tx, ty) in &m.painted {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(9.0), k(8.0), k(2.0), 200, 160, 40);
            buf.rect(sx - k(4.0), sy - k(9.0), k(2.0), k(5.0), 160, 60, 140);
            buf.rect(sx + k(2.0), sy - k(9.0), k(2.0), k(5.0), 60, 100, 160);
        }
        // the garden beds: soil strip, greener as it fills in
        for &(tx, ty, g) in &m.gardens {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(3.0), sy - k(1.0), k(6.0), k(3.0), 110, 85, 55);
            if g > 0.4 {
                buf.rect(sx - k(2.0), sy - k(2.0), k(4.0), k(1.0), 80, (120.0 + 60.0 * g) as u8, 60);
            }
            if g >= 1.0 {
                buf.rect(sx - k(1.0), sy - k(3.0), k(2.0), k(1.0), 220, 180, 80);
            }
        }
        // the water's cells: a depth-tinted tile where the land lies
        // under - shallow reads a sheen, deep a darker sink
        for &(tx, ty, band) in &m.flood {
            let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            if band >= 2 {
                buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 20, 35, 90);
            } else {
                buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 70, 100, 140);
            }
        }
        // the scorched ground: dark scars where the fire's been
        for &(tx, ty) in &m.scorched {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            if let Some(b) = pick(&art.burnt, tx as i32, ty as i32, 55) { draw_standing(&mut buf, b, sx, sy, zoom); continue; }
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(4.0), sy - k(3.0), k(8.0), k(6.0), 30, 26, 24);
        }
        // and what's alight: a flame's worth of colour over the tile (the
        // generated flames stand with everything else)
        for &(tx, ty) in m.burning.iter().filter(|_| art.fires.is_empty()) {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            let lick = (m.frame % 9 < 5) as i32; // it gutters
            buf.rect(sx - k(4.0), sy - k(2.0 + lick as f32), k(8.0), k(5.0 + lick as f32), 160, 45, 20);
            buf.rect(sx - k(2.0), sy - k(3.0 + lick as f32), k(4.0), k(4.0), 235, 150, 40);
        }
        // the desire-lines: where feet keep wearing, the ground shows it
        for &(tx, ty, s) in &m.trails {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            let shade = (30.0 * s) as u8;
            buf.rect(sx - k(3.5), sy - k(1.5), k(7.0), k(3.0),
                105u8.saturating_sub(shade), 86u8.saturating_sub(shade), 64u8.saturating_sub(shade));
        }
        let rocks = (0..m.map.height as i32).flat_map(|y| (0..m.map.width as i32).map(move |x| (x, y)))
            .filter(|&(x, y)| m.map.get(x, y) == Some(Tile::Rock));
        standing.extend(rocks.map(|(x, y)| (x as f32, y as f32, Piece::Prop(rock_prop(x, y), art.stones_k))));
        // the castles' curtains stand grand: the grand set's own pieces
        // at true proportions on the generated path, the pack's stretch
        // on the fallback one
        let grand: std::collections::HashSet<(i32, i32)> = m.map.grand_walls.iter().copied().collect();
        standing.extend(m.map.walls.iter().map(|&((x, y), piece)| {
            let is_grand = grand.contains(&(x, y));
            // the odd stretch of wall has a postern door in it; a castle's
            // takes the grand set's door, then its plain wall, then the
            // town wall's - the ensemble never mixes scales
            let postern = noise(x, y, 15) < 0.15;
            let g = if is_grand { art.castle_grand.as_ref() } else { None };
            let wall_at = |gi: usize, post: bool| -> Option<&Sprite> {
                let gw = g.and_then(|g| g.walls.get(gi).and_then(|s| s.as_ref()));
                let bw = art.walls.get(gi).and_then(|s| s.as_ref());
                if !post { return gw.or(bw); }
                g.and_then(|g| g.posts.get(gi).and_then(|s| s.as_ref()))
                    .or(gw)
                    .or(art.wall_posts.get(gi).and_then(|s| s.as_ref()))
                    .or(bw)
            };
            let sprite = match piece {
                WallPiece::AlongX => wall_at(0, postern),
                WallPiece::AlongY => wall_at(1, postern),
                WallPiece::GateAlongX => wall_at(2, false),
                WallPiece::GateAlongY => wall_at(3, false),
                WallPiece::Tower => g.and_then(|g| g.tower.as_ref()).or(art.tower_s.as_ref()),
            };
            let (wk, hk) = if !art.castle_rd && is_grand {
                if matches!(piece, WallPiece::Tower) {
                    // the odd tower stands loftier than its brothers
                    let lift = if noise(x, y, 37) < 0.35 { 1.3 } else { 1.0 };
                    (GRAND_TOWER_WK, GRAND_TOWER_K * lift)
                } else { (1.0, GRAND_WALL_K) }
            } else { (1.0, 1.0) };
            (x as f32, y as f32, Piece::Castle(sprite, wk, hk))
        }));
        // only what can reach the view goes into the sort (the whole map's
        // trees and rocks are in the list)
        standing.retain(|&(x, y, _)| { let (sx, sy) = iso_to_screen_cam(x, y, cam_x, cam_y, zoom); !off_view(sx, sy, zoom) });
        standing.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());
        // the press inside an open shop wears the era the research reached
        let press_era = game2_core::game::tech_tree::press_era(&m.tech);
        let draw_piece = |buf: &mut Buffer, &(bx, by, piece): &(f32, f32, Piece)| {
            let (sx, sy) = iso_to_screen_cam(bx, by, cam_x, cam_y, zoom);
            // the whole map's trees, rocks and walls are in this list: skip
            // what can't reach the view (most of it when zoomed in)
            if off_view(sx, sy, zoom) { return; }
            match piece {
                Piece::Prop(sprite, k) => if let Some(s) = sprite { draw_prop(buf, s, sx, sy, zoom, k); },
                Piece::Herb => herb_tuft(buf, sx, sy, zoom),
                Piece::Shroom => shroom_tuft(buf, sx, sy, zoom),
                Piece::Cache => draw_cache(buf, sx, sy, zoom),
                Piece::Skep => draw_skep(buf, sx, sy, zoom),
                Piece::Swarm => draw_swarm(buf, sx, sy, zoom),
                Piece::Stand(s) => draw_standing(buf, s, sx, sy, zoom),
                Piece::Gaol => {
                    // a barred door: a dark lintel over iron bars
                    let k = |d: i32| 1.max((d as f32 * zoom) as i32);
                    buf.rect(sx - k(5), sy - k(12), k(10), k(2), 44, 36, 28);
                    for x in [-4, -1, 2] { buf.rect(sx + k(x), sy - k(10), k(1), k(9), 58, 48, 38); }
                },
                Piece::Castle(sprite, wk, hk) => if let Some(s) = sprite {
                    // the generated set draws every piece at 1x; only the
                    // pack fallback still takes the grand stretch
                    if wk == 1.0 && hk == 1.0 {
                        if art.castle_rd { draw_standing(buf, s, sx, sy, zoom); } else { draw_castle(buf, s, sx, sy, zoom); }
                    } else { draw_castle_wh(buf, s, sx, sy, zoom, wk, hk); }
                },
                Piece::Building(sprite, up) | Piece::Cutaway(sprite, up) => if let Some(s) = sprite {
                    if up >= 1.0 {
                        if matches!(piece, Piece::Cutaway(_, _)) { draw_cutaway(buf, s, sx, sy, zoom, press_era); }
                        else { draw_on_tile(buf, s, sx, sy, zoom); }
                    } else if up > 0.0 {
                        // going up: the bottom of the house, from the ground up
                        let rows = ((s.height as f32 * up).round() as usize).max(1);
                        draw_on_tile(buf, &s.crop(0, s.height - rows, s.width, rows), sx, sy, zoom);
                    }
                },
            }
        };

        prof.mark("render.scene-setup");
        // people share the pass, along with the windmill and the chimney
        // smoke, which stand where their tower and houses stand
        let mut people: Vec<(f32, f32, Who)> = vec![];
        if art.mill_tower.is_some() {
            if let Some((mx_, my_)) = m.mill { people.push((mx_, my_, Who::Mill)); }
        }
        if art.house(0).is_some() {
            for (i, b) in m.buildings.iter().enumerate().filter(|(_, b)| b.smoke).take(3) {
                people.push((b.x, b.y, Who::Smoke(i)));
            }
        }
        people.extend(m.people.iter().filter(|p| p.kind != PKind::Gull).map(|p| (p.pos.0, p.pos.1, Who::P(p))));
        people.sort_by(|a, b| (a.0 + a.1).partial_cmp(&(b.0 + b.1)).unwrap());

        // market food stock: shown in the HUD (top-left), not on the market
        // tile, where villagers always gather to eat and covered it up
        let food_stock = m.panel.food;


        // name labels are drawn after every sprite (so no sprite covers one)
        // and de-overlapped: (text, x, y, colour, hungry, selected, tgt,
        // in_reach, hearts - the seat's warmth, floated over the name,
        // member - the org pip trails a warmed member's tag)
        let mut labels: Vec<(String, i32, i32, (u8, u8, u8), bool, bool, bool, bool, f32, bool)> = vec![];
        // the grave under the mouse, for its epitaph card
        let mut grave_tip: Option<(&str, &str)> = None;
        // a figure's bond hearts, 0 for a stranger - the map only sends
        // bonds with warmth worth showing
        let bond_hearts = |e: Entity| m.bonds.iter().find(|b| b.0 == e).map(|b| b.1).unwrap_or(0.0);
        // the player's dart waits for the label pass: a displaced name
        // mustn't paint over the one mark that says "this one is you"
        let mut player_marker: Option<(i32, i32, f32, u32)> = None;
        let mut standing = standing.iter().peekable();
        for &(px, py, who) in &people {
            while let Some(st) = standing.next_if(|st| st.0 + st.1 <= px + py) { draw_piece(&mut buf, st); }
            {
                let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                if off_view(sx, sy, zoom) { continue; }
            }
            match who {
                // windmill: the tower on its tile, its sails turning on the
                // tower's face (frame cycles every 10 render frames)
                Who::Mill => {
                    if let (Some(tower), true) = (&art.mill_tower, art.mill_rd) {
                        let (msx, msy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                        draw_standing(&mut buf, tower, msx, msy, zoom);
                        if let Some(sails) = art.mill_frames.get(((m.frame / 10) % 4) as usize) {
                            let (hx, hy) = mill_hub_rd(tower, msx, msy, zoom);
                            let (w, h) = ((sails.width as f32 * zoom).round() as i32, (sails.height as f32 * zoom).round() as i32);
                            draw_zoomed(&mut buf, sails, hx - w / 2, hy - h / 2, zoom);
                        }
                    } else if let Some(tower) = &art.mill_tower {
                        let (msx, msy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                        draw_on_tile(&mut buf, tower, msx, msy, zoom);
                        if let Some(sails) = art.mill_frames.get(((m.frame / 10) % 4) as usize) {
                            let (hx, hy) = mill_hub(tower, msx, msy, zoom);
                            let k = MILL_SAILS * 2.0 * TILE_W as f32 * zoom / tower.width as f32;
                            let (w, h) = ((sails.width as f32 * k).round() as i32, (sails.height as f32 * k).round() as i32);
                            draw_zoomed(&mut buf, sails, hx - w / 2, hy - h / 2, k);
                        }
                    }
                },
                // chimney smoke: a few small puffs rising off each home's
                // roof, drifting with the wind and thinning out (the pack's
                // smoke sprite is a cluster of balls, which just read as
                // circles on the roof)
                Who::Smoke(i) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    for (x, y, size, grey) in smoke_puffs(m.frame + i as u32 * 37, sx, sy, zoom) {
                        buf.rect(x, y, size, size, grey, grey, grey + 6);
                    }
                },
                Who::P(pv) if let PKind::Villager(i) = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    // the dead get a gravestone where they lie: the
                    // churchyard's buried wear the tombstone, the
                    // lying-in keep the flat mark
                    if pv.dead {
                        let stones = if pv.buried && !art.tombstones.is_empty() { &art.tombstones } else { &art.graves };
                        let top = match variant(stones, pv.e) {
                            Some(g) => {
                                draw_standing(&mut buf, g, sx, sy, zoom);
                                sy + ((g.width as f32 / 4.0).min(TILE_H as f32) * zoom).round() as i32 - (g.height as f32 * zoom).round() as i32
                            }
                            None => draw_grave(&mut buf, sx, sy, zoom),
                        };
                        let (lx, _) = name_label_pos(sx, sy, vill_name(i));
                        labels.push((vill_name(i).into(), lx, top - GLYPH_H - LABEL_GAP, (160, 160, 170), false, pv.selected, false, false, 0.0, false));
                        // the grave answers the mouse with who it keeps
                        if let Some(epi) = pv.epitaph.as_deref() {
                            let reach = 14.0 * zoom;
                            if (mx - sx).abs() as f32 <= reach && (my - sy) > -(26.0 * zoom) as i32 && (my - sy) < (8.0 * zoom) as i32 {
                                grave_tip = Some((vill_name(i), epi));
                            }
                        }
                    } else {
                        // a knight on the long roads rides: one
                        // horse-and-rider figure, mirrored leftward
                        let mounted_fig = if pv.mounted { art.mounted.as_ref() } else { None };
                        let anchor = if let Some(ms) = mounted_fig {
                            let facing = m.poses.iter().find(|p| p.0 == pv.e).map(|p| p.1).unwrap_or(2);
                            let s = if facing == 3 { art.mounted_flip.as_ref().unwrap_or(ms) } else { ms };
                            let h = draw_animal(&mut buf, s, sx, sy, zoom);
                            sy - h + 24
                        } else {
                            let look = figure_look(pv.kid, i, art.villager_sprites.len());
                            let fig = &art.villager_sprites[look];
                            if let Some(Some(ws)) = art.villager_walks.get(look) {
                                // a generated walk cycle: facing and striding as they move
                                let (facing, walking) = m.poses.iter().find(|p| p.0 == pv.e).map(|p| (p.1, p.2)).unwrap_or((2, false));
                                let scale = if pv.kid { 0.6 * zoom } else { zoom };
                                let cycle = ws.frames.get(facing as usize).map_or(4, |r| r.len());
                                ws.draw(&mut buf, facing, walking, self.walk_t(pv.e, px, py, cycle, m.frame), sx, sy, scale) - SPRITE_HEAD
                            } else {
                                // children draw small and flagged: the pack has no
                                // figure for them, so they're a scaled villager
                                if pv.kid {
                                    draw_kid(&mut buf, fig, sx, sy, zoom);
                                } else {
                                    draw_character(&mut buf, fig, sx, sy, zoom);
                                }
                                sprite_label_anchor(fig, sy, zoom)
                            }
                        };
                        // a protester carries their board
                        if let Some(d) = pv.sign { draw_picket_sign(&mut buf, sx, sy, zoom, d); }
                        let lbl = pv.label.clone().unwrap_or_else(|| vill_name(i).into());
                        let (lx, ly) = name_label_pos(sx, anchor, &lbl);
                        // bright when close enough to talk, recruit or meet; dim otherwise
                        let rgb = if pv.in_reach { NAME_IN_REACH } else { NAME_OUT_OF_REACH };
                        // the one an ask would land on rides the label
                        // pass: the picked villager, or the one in
                        // reach when nobody's picked
                        let t = if m.sel.picked { m.sel.target } else { m.reach_target };
                        let tgt = i == t && !pv.dead;
                        labels.push((lbl, lx, ly, rgb, pv.hungry, pv.selected, tgt, pv.in_reach, bond_hearts(pv.e),
                            m.org_members.contains(&pv.e)));
                        // sick: a green cross on the other side of the head
                        if pv.sick {
                            let (mk, (r, g, b), _) = MARK_SICK;
                            draw_text_outlined(&mut buf, sx - 16, head_y(sy, zoom), mk, r, g, b);
                        }
                        // activity marker beside the head
                        if pv.act_mark > 0 {
                            let (mk, (r, g, b), _) = if pv.act_mark == 1 { MARK_FOOD } else { MARK_BUSY };
                            draw_text_outlined(&mut buf, sx + 10, head_y(sy, zoom), mk, r, g, b);
                        }
                    }
                },
                Who::P(pv) if matches!(pv.kind, PKind::Sheep | PKind::Goat) => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    let set = if pv.kind == PKind::Sheep { &art.sheep } else { &art.goats };
                    match variant(set, pv.e) {
                        Some(s) => { draw_animal(&mut buf, s, sx, sy, zoom); }
                        None => draw_beast(&mut buf, sx, sy, zoom, pv.kind == PKind::Sheep),
                    }
                },
                Who::P(pv) if let PKind::Pet(species) = pv.kind => {
                    let sprite = match species { Species::Dog => &self.art.dog, Species::Cat => &self.art.cat, _ => &NO_SPRITE };
                    let (dx, dy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    // standing on its spot, drawn at the villagers' scale
                    let h = match sprite {
                        Some(s) => draw_animal(&mut buf, s, dx, dy, zoom),
                        None => 0,
                    };
                    if pv.bond {
                        labels.push(("<3".into(), dx - 6, dy - h - GLYPH_H - LABEL_GAP, (255, 120, 120), false, false, false, false, 0.0, false)); // placed with the names
                    }
                },
                // the watch: soldiers from the humans pack at the gates
                Who::P(pv) if let PKind::Guard(i) = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(Some(ws)) = art.guard_walks.get(i % art.guard_walks.len().max(1)) {
                        ws.draw(&mut buf, 2, false, 0, sx, sy, zoom);
                    } else if !art.guards.is_empty() {
                        draw_character(&mut buf, &art.guards[i % art.guards.len()], sx, sy, zoom);
                    }
                },
                Who::P(pv) if let PKind::Priest = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(ws) = &art.priest_walk { ws.draw(&mut buf, 2, false, 0, sx, sy, zoom); }
                    else if let Some(s) = &art.priest_s { draw_character(&mut buf, s, sx, sy, zoom); }
                },
                Who::P(pv) if let PKind::Traveller = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(ws) = &art.traveller_walk {
                        let (facing, walking) = m.poses.iter().find(|p| p.0 == pv.e).map(|p| (p.1, p.2)).unwrap_or((2, false));
                        let cycle = ws.frames.get(facing as usize).map_or(4, |r| r.len());
                        ws.draw(&mut buf, facing, walking, self.walk_t(pv.e, px, py, cycle, m.frame), sx, sy, zoom);
                    } else if let Some(s) = &art.traveller_s { draw_character(&mut buf, s, sx, sy, zoom); }
                },
                Who::P(pv) if let PKind::Chicken = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    if let Some(c) = variant(&art.hens, pv.e) {
                        draw_animal(&mut buf, c, sx, sy, zoom);
                    }
                },
                // rival villagers (foreign settlement): the outlined figures
                Who::P(pv) if let PKind::Rival(i) = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    let anchor = match (art.rival_walks.get(i), art.rival_sprites.get(i)) {
                        (Some(Some(ws)), _) => {
                            let (facing, walking) = m.poses.iter().find(|p| p.0 == pv.e).map(|p| (p.1, p.2)).unwrap_or((2, false));
                            let cycle = ws.frames.get(facing as usize).map_or(4, |r| r.len());
                            ws.draw(&mut buf, facing, walking, self.walk_t(pv.e, px, py, cycle, m.frame), sx, sy, zoom) - SPRITE_HEAD
                        }
                        (_, Some(rs)) => { draw_character(&mut buf, rs, sx, sy, zoom); sprite_label_anchor(rs, sy, zoom) }
                        _ => label_anchor(sy, zoom),
                    };
                    let (lx, ly) = name_label_pos(sx, anchor, RIVAL_NAMES[i]);
                    labels.push((RIVAL_NAMES[i].into(), lx, ly, (255, 140, 140), false, false, false, false, bond_hearts(pv.e),
                        m.org_members.contains(&pv.e)));
                },
                // player sprite at their actual position
                Who::P(pv) if let PKind::Player = pv.kind => {
                    let (sx, sy) = iso_to_screen_cam(px, py, cam_x, cam_y, zoom);
                    let (facing, walking) = m.player_pose;
                    // the picked look: a villager figure, small for a
                    // childhood pick - the dedicated player sprite only
                    // when the pack has no figures at all
                    let look = figure_look(pv.kid, pv.look as usize, art.villager_sprites.len());
                    let vws = art.villager_walks.get(look).and_then(|o| o.as_ref());
                    let vfig = art.villager_sprites.get(look);
                    let scale = if pv.kid { 0.6 * zoom } else { zoom };
                    let mut anchor = None;
                    // the gold ring marks you whatever you wear
                    buf.rect(sx - 12, sy - 4, 24, 3, 255, 215, 0);
                    if let Some(ws) = vws {
                        let cycle = ws.frames.get(facing as usize).map_or(4, |r| r.len());
                        anchor = Some(ws.draw(&mut buf, facing, walking,
                            self.walk_t(pv.e, px, py, cycle, m.frame), sx, sy, scale) - SPRITE_HEAD);
                    } else if let Some(fig) = vfig {
                        if pv.kid { draw_kid(&mut buf, fig, sx, sy, zoom); }
                        else { draw_character(&mut buf, fig, sx, sy, zoom); }
                        anchor = Some(sprite_label_anchor(fig, sy, zoom));
                    } else if let Some(ws) = &art.player_walk {
                        let cycle = ws.frames.get(facing as usize).map_or(4, |r| r.len());
                        anchor = Some(ws.draw(&mut buf, facing, walking,
                            self.walk_t(pv.e, px, py, cycle, m.frame), sx, sy, zoom) - SPRITE_HEAD);
                    } else if let Some(ps) = &art.player_s {
                        draw_character(&mut buf, ps, sx, sy, zoom);
                        anchor = Some(sprite_label_anchor(ps, sy, zoom));
                    } else {
                        draw_rect(&mut buf, sx - 4, sy - 8, 8, 8, 255, 255, 255);
                    }
                    let anchor = anchor.unwrap_or_else(|| label_anchor(sy, zoom));
                    let (lx, ly) = name_label_pos(sx, anchor, "YOU");
                    player_marker = Some((sx, ly - 2, zoom, m.frame));
                    labels.push(("YOU".into(), lx, ly, (255, 255, 0), false, false, false, false, 0.0, false));
                },
                Who::P(_) => {},
            }
        }
        for st in standing { draw_piece(&mut buf, st); }
        prof.mark("render.sprites");

        // snowballs in the air, over the sprites and under the names
        for &(bx, by) in &m.snowballs {
            let (sx, sy) = iso_to_screen_cam(bx, by, cam_x, cam_y, zoom);
            if off_view(sx, sy, zoom) { continue; }
            let s = (2.0 * zoom).max(1.0) as i32;
            buf.rect(sx - s / 2, sy - s / 2 - (4.0 * zoom) as i32, s, s, 240, 245, 255);
        }

        // freight on the roads: a wagon at the feet of whoever's driving
        // - the cart, the levy, the tribute and the gift trains all pull
        // one; drawn after the sprites so it sits on the lane, underfoot
        for &(tx, ty) in &m.trains {
            let (sx, sy) = iso_to_screen_cam(tx, ty, cam_x, cam_y, zoom);
            if off_view(sx, sy, zoom) { continue; }
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            let wy = sy + k(3.0);
            // the bed, its load, and the two wheels
            buf.rect(sx - k(4.0), wy - k(3.0), k(8.0), k(2.0), 122, 84, 48);
            buf.rect(sx - k(2.5), wy - k(4.0), k(5.0), k(1.0), 150, 110, 60);
            buf.rect(sx - k(4.5), wy - k(1.0), k(1.0), k(1.0), 52, 40, 30);
            buf.rect(sx + k(3.5), wy - k(1.0), k(1.0), k(1.0), 52, 40, 30);
        }

        // the cause's flyers, nailed up on doors and the market cross
        for &(fx, fy) in &m.flyers {
            let (sx, sy) = iso_to_screen_cam(fx, fy, cam_x, cam_y, zoom);
            if off_view(sx, sy, zoom) { continue; }
            let k = |v: f32| ((v * zoom).round() as i32).max(1);
            buf.rect(sx - k(3.0), sy - k(14.0), k(6.0), k(5.0), 232, 224, 200);
            buf.rect(sx - k(1.5), sy - k(13.0), k(3.0), k(1.0), 120, 100, 80);
            buf.rect(sx - k(0.5), sy - k(9.0), k(1.0), k(9.0), 70, 55, 40);
        }

        // gulls wheel over the open sea
        if let Some(gull) = &art.gull {
            for (i, pv) in m.people.iter().filter(|p| p.kind == PKind::Gull).enumerate() {
                let (gx, gy) = pv.pos;
                let t = m.frame as f32 * 0.05 + i as f32 * 2.1;
                let (sx, sy) = iso_to_screen_cam(gx + t.cos() * 0.6, gy + t.sin() * 0.3, cam_x, cam_y, zoom);
                draw_zoomed(&mut buf, gull, sx - gull.width as i32, sy - (14.0 * zoom) as i32, 2.0 * zoom);
            }
        }

        // all name labels, on top of every sprite, nudged apart where they'd overlap
        // a name's tag is its text plus, for a friend, the heart strip
        // floated over it: the collision box covers the whole tag - strip
        // width included - so a neighbour can't ride over either half
        let heart_zone = |hearts: f32| if hearts > 0.0 { HEART_H + 1 } else { 0 };
        let strip_w = 5 * HEART_PITCH - (HEART_PITCH - HEART_W);
        let boxes: Vec<(i32, i32, i32, i32)> = labels.iter()
            // the ask's target mark sits a glyph left of the name - count
            // it in the collision box so a neighbour can't ride over it
            .map(|(t, x, y, _, hungry, _, tgt, _, hearts, member)| {
                let zone = heart_zone(*hearts);
                let text_w = t.len() as i32 * GLYPH_W
                    + if *hungry { GLYPH_W + 2 } else { 0 }
                    + if *member && *hearts > 0.0 { GLYPH_W + 2 } else { 0 }
                    + if *tgt { GLYPH_W + 2 } else { 0 };
                let x0 = if *tgt { *x - GLYPH_W - 2 } else { *x };
                let (bx, bw) = if zone > 0 {
                    // the strip centres on the name; widen the box to it
                    let hx = *x + (t.len() as i32 * GLYPH_W - strip_w) / 2;
                    (x0.min(hx), (x0 + text_w).max(hx + strip_w) - x0.min(hx))
                } else { (x0, text_w) };
                (bx, *y - zone, bw, GLYPH_H + 3 + zone)
            })
            .collect();
        let placed = place_labels(&boxes);
        // where each name finally sits, so a speech bubble can clear it
        let label_ys: Vec<(&String, i32)> = labels.iter().zip(&placed)
            .map(|(l, y)| (&l.0, *y + heart_zone(l.8))).collect();
        for ((text, x, _, (r, g, b), hungry, selected, tgt, in_reach, hearts, member), y) in labels.iter().zip(placed) {
            let ty = y + heart_zone(*hearts);
            // warmth floats over the name: the five-heart strip, centred
            // on it - a stranger's name carries none
            if *hearts > 0.0 {
                draw_hearts(&mut buf, x + (text.len() as i32 * GLYPH_W - strip_w) / 2, y, *hearts, 5);
            }
            // the ask's target leads its name: bright when they'd hear it
            // now, dim when the picked one's a walk away
            if *tgt {
                let (r2, g2, b2) = if *in_reach { (240, 220, 130) } else { (160, 150, 120) };
                draw_text_outlined(&mut buf, x - GLYPH_W - 2, ty, "\u{203a}", r2, g2, b2);
            }
            draw_text_outlined(&mut buf, *x, ty, text, *r, *g, *b);
            // hunger indicator, right after the name
            if *hungry { let (m, (r, g, b), _) = MARK_HUNGRY; draw_text_outlined(&mut buf, x + text.len() as i32 * GLYPH_W + 2, ty, m, r, g, b); }
            // an org mate's gold pip trails the name, where the warmth
            // that shows it rides over the head
            if *member && *hearts > 0.0 {
                let mx = x + text.len() as i32 * GLYPH_W + 2
                    + if *hungry { GLYPH_W + 2 } else { 0 };
                let (r2, g2, b2) = ORG_GOLD;
                draw_rect(&mut buf, mx + 1, ty + 2, 1, 1, r2, g2, b2);
                draw_rect(&mut buf, mx, ty + 3, 3, 1, r2, g2, b2);
                draw_rect(&mut buf, mx + 1, ty + 4, 1, 1, r2, g2, b2);
            }
            // selection: underline the selected villager's name
            if *selected { draw_rect(&mut buf, x - 1, ty + GLYPH_H + 1, text.len() as i32 * GLYPH_W + 1, 2, 255, 255, 0); }
        }
        // a grave's hover card: the name and how they went, beside the cursor
        if let Some((name, epi)) = grave_tip {
            let line = format!("{} - {}", name, epi);
            let w = (text_width(&line, 1) + 14).min(MAP_W as i32 - 8);
            let bx = (mx + 16).min(MAP_W as i32 - w - 4);
            let by = (my - GLYPH_H - 14).max(4);
            draw_rect(&mut buf, bx, by, w, GLYPH_H + 12, 24, 24, 34);
            draw_text(&mut buf, bx + 7, by + 6, &line, 225, 218, 200);
        }
        if let Some((cx, top, z, f)) = player_marker {
            draw_player_marker(&mut buf, cx, top, z, f);
        }

        prof.mark("render.gulls+labels");
        // --- UI PANEL ---
        draw_rect(&mut buf, UI_X as i32, 0, W as i32 - UI_X as i32, H as i32, 32, 32, 48);
        draw_text_scaled(&mut buf, UI_X as i32 + 10, 8, "THE WANDERER", 2, 255, 220, 130);

        // status: members, season, tick
        let p = &m.panel;
        let guild = Some(m.panel.guilds).filter(|g| *g > 0);
        draw_text(&mut buf, UI_X as i32 + 10, 30, &status_line(&p.org, p.leader.as_deref(), p.members.len(), p.gold, guild), 200, 200, 200);
        // the odd job under way, if the Wanderer's carrying one
        if let Some(q) = &m.panel.quest {
            draw_text(&mut buf, UI_X as i32 + 10, 42, q, 240, 220, 140);
        }
        // the mystery doing the rounds, if the seat's piecing one
        // together - right of the action buttons, where nothing else sits;
        // the asker's name is a button (mystery_hit knows the same rows)
        if let Some(mz) = &m.panel.mystery {
            let line = mystery_line(mz);
            let name_col = mystery_name_col(mz);
            for (n, l) in wrap_text(&line, 30).iter().take(3).enumerate() {
                let y = 68 + n as i32 * 11;
                // the name's cells draw bright - and brighter still under
                // the mouse, the way the log's clickable lines do
                let span = name_col.and_then(|(row, c0, c1)| (row == n).then_some((c0, c1)));
                let hovered = span.map(|(c0, c1)| {
                    mx >= UI_X as i32 + 150 + c0 as i32 * GLYPH_W
                        && mx < UI_X as i32 + 150 + c1 as i32 * GLYPH_W
                        && my >= y && my < y + 11
                }).unwrap_or(false);
                draw_text(&mut buf, UI_X as i32 + 150, y, l, 200, 180, 235);
                if let Some((c0, c1)) = span {
                    let chars: Vec<char> = l.chars().collect();
                    let name: String = chars[c0..c1.min(chars.len())].iter().collect();
                    let x = UI_X as i32 + 150 + c0 as i32 * GLYPH_W;
                    let (r, g, b) = if hovered { (255, 240, 160) } else { (255, 220, 120) };
                    draw_text(&mut buf, x, y, &name, r, g, b);
                }
            }
        }
        // the powers' standing words on the org - one line, so a
        // stack of them can't run under the buttons
        if !m.panel.org_status.is_empty() {
            draw_text(&mut buf, UI_X as i32 + 10, 54, &org_status_line(&m.panel.org_status), 200, 150, 150);
        }


        // buttons
        for (i, b) in action_buttons().iter().enumerate() {
            if !action_shown(m, i) { continue; }
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            let reason = m.blocked[i].as_deref();
            draw_button(&mut buf, b, hover, reason.is_some());
            // a greyed-out button always says why; others explain themselves on hover
            if let Some(r) = &reason {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, r, 200, 140, 120);
            } else if hover {
                draw_text(&mut buf, b.x + b.w + 6, b.y + 12, TOOLTIPS[i], 220, 220, 160);
            }
        }

        // the house-meeting plan filling in: who's said yes, by name -
        // it sits right of the BONDS block, clear of the ctx band below
        if let Some(mp) = &m.panel.meeting {
            draw_text(&mut buf, UI_X as i32 + 190, 190, "MEETING", 255, 215, 0);
            let lines: Vec<String> = meeting_lines(mp).iter()
                .flat_map(|l| wrap_text(l, 23)).collect();
            for (n, line) in lines.iter().take(3).enumerate() {
                draw_text(&mut buf, UI_X as i32 + 190, 201 + n as i32 * 11, line, 220, 220, 180);
            }
        }

        // inspector: a villager's properties page, or a building's card
        // when one's clicked - it floats on the map's bottom-left now
        // that the panel's middle band belongs to the context buttons.
        // The card's lines carry offsets from its old panel slot; the
        // plate grows to meet the deepest row, so a full page has the
        // room a bare name never asked for
        const INSP_X: i32 = 16;
        const INSP_W: i32 = 364;
        if self.inspector_open && m.inspector.is_some() {
            let insp = m.inspector.as_ref().unwrap();
            let deep = insp.lines.iter().map(|l| l.y).max().unwrap_or(460) + 14;
            let card_h = (deep - 316).clamp(120, 560);
            let top = 948 - card_h;
            let lx = |k: i32| INSP_X + k - 10;
            let ly = |k: i32| top + k - 316;
            draw_rect(&mut buf, lx(10), ly(316), INSP_W, card_h, 45, 45, 65);
            if insp.dead {
                draw_grave(&mut buf, lx(PORTRAIT_FOOT.0), ly(PORTRAIT_FOOT.1), 2.0);
            } else if let Some(fig_i) = insp.fig {
                let look = figure_look(insp.kid, fig_i, art.villager_sprites.len());
                let fig = &art.villager_sprites[look];
                fig.blit_scaled(&mut buf, lx(6), ly(322), 2);
            }
            for line in &insp.lines {
                if line.text.starts_with('\u{1}') {
                    // "METER:name:frac:note" sentinel from the sim
                    let mut it = line.text[1..].split('\u{2}');
                    let (name, rest) = (it.next().unwrap_or(""), it.next().unwrap_or(""));
                    let (frac, note) = rest.split_once(':')
                        .map(|(a, b)| (a.parse::<f32>().unwrap_or(0.0), b))
                        .unwrap_or((0.0, ""));
                    draw_meter(&mut buf, lx(line.x), ly(line.y), name, frac, note);
                } else {
                    draw_text(&mut buf, lx(line.x), ly(line.y), &line.text, line.rgb.0, line.rgb.1, line.rgb.2);
                }
            }
            // the seat's hearts for them, top-right of the card -
            // five pixels-hearts in half steps, clear of the name row
            if let Some(h) = insp.hearts {
                draw_hearts(&mut buf, lx(10) + INSP_W - 8 - 5 * HEART_PITCH, ly(320), h, 5);
            }
        }

        // contextual actions: one layout for the draw and the click -
        // ctx_buttons stacks them in list order across the panel's band,
        // so no two rects ever meet. Greyed reasons still show on hover
        // for the rate-limited ones (a used-up snowball fight, a walked
        // cat); the unavailable never reach the list at all
        let ctx_btns: Vec<(Button, Option<&String>)> = ctx_buttons(&m.ctx).into_iter()
            .zip(m.ctx_blocked.iter()).map(|(b, r)| (b, r.as_ref())).collect();
        for (b, reason) in &ctx_btns {
            let hover = mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h;
            draw_button(&mut buf, b, hover, reason.is_some());
        }
        // a greyed reason reads on one line under the band - beside its
        // button it would land on the neighbouring column
        if let Some(r) = ctx_btns.iter()
            .find(|(b, _)| mx >= b.x && mx < b.x + b.w && my >= b.y && my < b.y + b.h)
            .and_then(|(_, r)| r.as_deref()) {
            let bottom = ctx_btns.iter().map(|(b, _)| b.y + b.h).max().unwrap_or(CTX_BAND_Y);
            draw_text(&mut buf, UI_X as i32 + 18, bottom + 4, r, 255, 180, 140);
        }

        // the bond ledger: the reward for keeping company - who the
        // seat has spoken to and how far each bond has climbed. It sits
        // between the Build button (ends 184) and the Gather ctx button
        // (starts 236), so the block can't outgrow three lines
        let s = &m.panel.social;
        draw_text(&mut buf, UI_X as i32 + 18, 190, "BONDS", 255, 215, 0);
        for (n, line) in bond_lines(s).iter().enumerate() {
            draw_text(&mut buf, UI_X as i32 + 18, 202 + n as i32 * 12, line, 215, 215, 195);
        }

        // the org's own muscle - a word for its strength and whether it's
        // been used; the inspector card floats on the map now, so the row
        // always has the panel to itself
        if let Some(pw) = &m.panel.power {
            draw_text(&mut buf, UI_X as i32 + 18, 452, pw, 170, 200, 170);
        }

        // organization: who has joined, by name
        let member_names: Vec<&str> = m.panel.members.iter().map(|s| s.as_str()).collect();
        draw_text(&mut buf, UI_X as i32 + 18, 464, MEMBERS_LABEL, 255, 215, 0);
        draw_text(&mut buf, UI_X as i32 + 18 + (MEMBERS_LABEL.len() as i32 + 1) * GLYPH_W, 464, &members_line(&member_names), 220, 220, 180);

        // the issues the organizer has pieced together
        draw_text(&mut buf, UI_X as i32 + 18, 476, "ISSUES", 255, 215, 0);
        let issue_line = if m.panel.issues.is_empty() { "none heard yet".to_string() } else { m.panel.issues.join("; ") };
        for (n, line) in wrap_text(&issue_line, 42).iter().take(if m.panel.campaign.is_some() { 1 } else { 2 }).enumerate() {
            draw_text(&mut buf, UI_X as i32 + 18 + (7 * GLYPH_W), 476 + n as i32 * 11, line, 220, 190, 140);
        }
        // the campaign the org fights for, once it's taken its vote - the
        // Meet button (a ctx action) sits to the right of the issue line
        if let Some(c) = &m.panel.campaign {
            // the pull behind the seats: the member whose ties run the
            // room's empty chairs
            let pull = c.shadow.as_ref().map(|s| format!(" - {} pulls", s)).unwrap_or_default();
            draw_text(&mut buf, UI_X as i32 + 18, 488,
                &format!("FIGHT {}{}", c.issue, pull), 255, 180, 120);
            let due = if c.due { "meet due" } else { "meet to come" };
            // the cause's posture rides the cadence: won through, the
            // room's split, the cause's weary
            let state = if c.state.is_empty() { String::new() } else { format!(" - {}", c.state) };
            // the assembly's standing rides the same line when it's
            // got something to say - the agenda posted, pledges counted
            let assembly = if c.assembly.is_empty() { String::new() } else { format!(" - {}", c.assembly) };
            draw_text(&mut buf, UI_X as i32 + 18, 500,
                &format!("every {}d - {}{}{}{}", c.cadence_days, due, state, assembly,
                    if c.seats.join(" ").is_empty() { String::new() } else { format!(" - {}", c.seats.join(" ")) }),
                120, 120, 140);
        } else {
            draw_text(&mut buf, UI_X as i32 + 18, 500, PANEL_HINT, 120, 120, 140);
        }

        // event log
        let (lx, ly, lw, lh) = LOG_BOX;
        draw_rect(&mut buf, lx, ly, lw, lh, 25, 25, 40);
        draw_text(&mut buf, lx + 4, ly + 4, "LOG", 150, 150, 180);
        // newest first, word-wrapped to the box instead of cut off mid-word;
        // an entry that knows where it happened or who it's about brightens
        // under the mouse - it's a button (log_hit knows the same rows)
        // the last row marks what rolled off the top, when it did
        let log_rows = if m.log_dropped > 0 { LOG_ROWS - 1 } else { LOG_ROWS };
        let mut row = 0;
        'log: for entry in &m.log {
            let clickable = entry.at.is_some() || entry.who.is_some();
            let (r, g, b) = log_colour(&entry.text);
            // "day 120" in dim grey, then the message in its colour
            let (day, msg) = log_parts(&entry.text);
            let text = match &day { Some(d) => format!("{} {}", d, msg), None => msg.to_string() };
            for (n, line) in wrap_text(&text, LOG_CHARS).iter().enumerate() {
                if row == log_rows { break 'log; }
                let y = 530 + row as i32 * 12;
                let hovered = clickable && mx >= lx && mx < lx + lw && my >= y && my < y + 12;
                let (r, g, b) = if hovered {
                    let lift = |c: u8| (c as u16 + (255 - c as u16) / 2) as u8;
                    (lift(r), lift(g), lift(b))
                } else { (r, g, b) };
                match &day {
                    Some(d) if n == 0 && line.starts_with(d.as_str()) => {
                        draw_text(&mut buf, lx + 4, y, d, 120, 120, 145);
                        draw_text(&mut buf, lx + 4 + (d.len() as i32 + 1) * GLYPH_W, y, line[d.len()..].trim_start(), r, g, b);
                    }
                    _ => { draw_text(&mut buf, lx + 4, y, line, r, g, b); }
                }
                row += 1;
            }
        }
        // the reserved row holds the count of what's already gone
        if m.log_dropped > 0 {
            draw_text(&mut buf, UI_X as i32 + 14, 530 + row as i32 * 12,
                &format!("· · · {} earlier lines", m.log_dropped), 100, 100, 125);
        }

        // the TALK panel under the find box: conversations, apart from the log
        talk::draw_talk_panel(&mut buf, &m.talk, (mx, my));

        // the find box under the minimap
        draw_search_box(&mut buf, &self.search_q,
            search_people(&m.people, &self.search_q).len(), self.search_focus);

        // the conversation box, top-right of the map: a Talk under way or
        // a house meeting's lines as they're said - and the call one leaves
        if let Some(c) = &m.convo { talk::draw_convo_panel(&mut buf, c, (mx, my)); }

        // what the marks over villagers' heads mean, under the log
        let mut mx = UI_X as i32 + 14;
        for (m, (r, g, b), word) in MAP_MARKS {
            draw_text(&mut buf, mx, LEGEND_Y, m, r, g, b);
            mx += (m.len() as i32 + 1) * GLYPH_W;
            draw_text(&mut buf, mx, LEGEND_Y, word, 150, 150, 170);
            mx += (word.len() as i32 + 2) * GLYPH_W;
        }

        // season change: subtle banner notification instead of flash
        if tick > 10 && tick % 90 < 3 {
            let sname = match season { Season::Spring => "Spring", Season::Summer => "Summer", Season::Autumn => "Autumn", Season::Winter => "Winter" };
            let banner = format!("{} arrives", sname);
            let bw = banner.len() as i32 * GLYPH_W + 16;
            buf.rect(MAP_W as i32 / 2 - bw / 2, 56, bw, 16, 20, 20, 35);
            draw_text(&mut buf, MAP_W as i32 / 2 - bw / 2 + 8, 60, &banner, 255, 220, 130);
        }


        prof.mark("render.ui-panel");
        // weather over the map: overcast, falling rain or snow, the rainbow after rain
        draw_weather(&mut buf, m.precip, m.frame as u64, m.rainbow_from.map(|f| (m.frame - f) as u64));
        prof.mark("render.weather");

        // minimap (top-right of map area): the map in miniature, same projection
        // and colours, with villagers, the player and the visible area marked
        buf.rect(MM_LEFT - 2, MM_TOP - 2, MM_W + 4, MM_H + 4, 20, 20, 30);
        let (w_t, h_t) = (m.map.width as f32, m.map.height as f32);
        let k = mm_k(w_t, h_t);
        // the woods as a flat grid for the per-pixel lookup below (a
        // contains() on the Vec walked the whole woods for every pixel)
        let mut tree_at = vec![false; m.map.width * m.map.height];
        for &(tx, ty) in &m.map.trees { tree_at[ty as usize * m.map.width + tx as usize] = true; }
        for my in 0..MM_H { for mx in 0..MM_W {
            if let Some((tx, ty)) = minimap_tile((MM_LEFT + mx) as f32 + 0.5, (MM_TOP + my) as f32 + 0.5, w_t, h_t) {
                let (r, g, b) = m.map.get(tx as i32, ty as i32).map(|t| tile_colour(t, season)).unwrap_or((20, 20, 30));
                // the woods show darker
                let (r, g, b) = if tree_at[ty as usize * m.map.width + tx as usize] { blend((r, g, b), (20, 70, 35), 0.5) } else { (r, g, b) };
                buf.set(MM_LEFT + mx, MM_TOP + my, r, g, b);
            }
        }}
        let mm_dot = |buf: &mut Buffer, tx: f32, ty: f32, r: u8, g: u8, b: u8| {
            let (x, y) = tile_to_minimap(tx, ty, k);
            buf.rect(x as i32 - 1, y as i32 - 1, 2, 2, r, g, b);
        };
        // the country's sites: a dot in the kind's colour, the name
        // under it - the keeper's, if the keep's bed holds one
        for sv in &m.sites {
            let (x, y) = tile_to_minimap(sv.x, sv.y, k);
            let (r, g, b) = match sv.kind {
                country::SiteKind::Castle => (200, 170, 90),
                country::SiteKind::Town => (170, 120, 70),
                country::SiteKind::Farm => (110, 160, 80),
                country::SiteKind::Monastery => (220, 220, 210),
                country::SiteKind::Mine => (140, 115, 90),
            };
            buf.rect(x as i32 - 2, y as i32 - 2, 4, 4, r, g, b);
            // the head's style leads the name ("Earl Godric of
            // Ravensholt"), the treasury's gold behind it
            let label = match &sv.head {
                Some(h) => {
                    let gold = if sv.treasury >= 1.0 { format!(" ({:.0}g)", sv.treasury) } else { String::new() };
                    match &sv.keeper {
                        Some(k) if !h.contains(k.as_str()) => format!("{} of {} ({}){}", h, sv.name, k, gold),
                        _ => format!("{} of {}{}", h, sv.name, gold),
                    }
                }
                None => match &sv.keeper {
                    Some(k) => format!("{} ({})", sv.name, k),
                    None => sv.name.clone(),
                },
            };
            draw_text(&mut buf, x as i32 + 4, y as i32 - 3, &label, r, g, b);
        }
        // graves go down last: a crowd at the graveside mustn't hide one
        let (living, graves): (Vec<&PersonV>, Vec<&PersonV>) = m.people.iter()
            .partition(|pv| !(matches!(pv.kind, PKind::Villager(_)) && pv.dead));
        for pv in living.into_iter().chain(graves) {
            match pv.kind {
                PKind::Villager(_) => {
                    // the dead are graves now: grey, not a living red villager
                    let (r, g, b) = if !pv.dead { MM_VILLAGER } else { MM_GRAVE };
                    mm_dot(&mut buf, pv.pos.0, pv.pos.1, r, g, b);
                }
                PKind::Rival(_) => mm_dot(&mut buf, pv.pos.0, pv.pos.1, 255, 160, 60),
                PKind::Player => mm_dot(&mut buf, pv.pos.0, pv.pos.1, 255, 255, 0),
                _ => {}
            }
        }
        // outline of what the map view is showing, clipped to the minimap box
        let (vx0, vy0) = screen_to_minimap(0.0, 0.0, cam_x, cam_y, zoom, w_t, h_t);
        let (vx1, vy1) = screen_to_minimap(MAP_W as f32, H as f32, cam_x, cam_y, zoom, w_t, h_t);
        let (x0, x1) = ((vx0 as i32).max(MM_LEFT), (vx1 as i32).min(MM_LEFT + MM_W - 1));
        let (y0, y1) = ((vy0 as i32).max(MM_TOP), (vy1 as i32).min(MM_TOP + MM_H - 1));
        if x0 < x1 && y0 < y1 {
            buf.rect(x0, y0, x1 - x0 + 1, 1, 255, 255, 255);
            buf.rect(x0, y1, x1 - x0 + 1, 1, 255, 255, 255);
            buf.rect(x0, y0, 1, y1 - y0 + 1, 255, 255, 255);
            buf.rect(x1, y0, 1, y1 - y0 + 1, 255, 255, 255);
        }

        prof.mark("render.minimap");
        // speech bubbles over whoever is speaking
        for bv in &m.bubbles {
            let (text, thought) = (bv.text.as_str(), bv.thought);
            {
                {
                    let (sx, sy) = iso_to_screen_cam(bv.pos.0, bv.pos.1, cam_x, cam_y, zoom);
                    // off the map view (e.g. under the panel): the log already
                    // has what they said, so don't float a bubble over the panel
                    // the drawn label carries the marks - the plain name
                    // is its prefix, not the whole string
                    let label_y = bv.name.as_deref().and_then(|n| label_ys.iter().find(|(t, _)| t.starts_with(n)).map(|(_, y)| *y));
                    if let Some((bx, by, bw)) = bubble_rect(sx, sy, zoom, text, label_y) {
                        let tail_x = (sx - 4).clamp(bx + 2, bx + bw - 10);
                        if thought {
                            // a thought: pale blue, its corners rounded off, a
                            // trail of shrinking puffs down to the thinker
                            let c = (225, 235, 250);
                            buf.rect(bx + 2, by, bw - 4, BUBBLE_H, c.0, c.1, c.2);
                            buf.rect(bx, by + 2, bw, BUBBLE_H - 4, c.0, c.1, c.2);
                            buf.rect(tail_x + 2, by + BUBBLE_H + 1, 4, 4, c.0, c.1, c.2);
                            buf.rect(tail_x + 4, by + BUBBLE_H + 6, 2, 2, c.0, c.1, c.2);
                        } else {
                            buf.rect(bx, by, bw, BUBBLE_H, 240, 240, 220);        // cream bubble
                            buf.rect(tail_x, by + BUBBLE_H - 2, 8, 6, 240, 240, 220); // tail, under the speaker
                        }
                        draw_text(&mut buf, bx + 6, by + 5, text, 40, 30, 20);
                    }
                }
            }
        }

        // toasts (big moments); one that happened somewhere is a button -
        // a click jumps the camera there (toast_hit knows the same boxes)
        for (i, bx, by, bw, fade) in self.toast_layout(&m.toasts, m.frame) {
            let t = &m.toasts[i];
            let hovered = t.at.is_some() && mx >= bx && mx < bx + bw && my >= by && my < by + 20;
            // the fade runs every colour toward the banner's own dark
            let mix = |(r, g, b): (i32, i32, i32)| -> (u8, u8, u8) {
                ((20.0 + (r as f32 - 20.0) * fade) as u8,
                 (20.0 + (g as f32 - 20.0) * fade) as u8,
                 (35.0 + (b as f32 - 35.0) * fade) as u8)
            };
            buf.rect(bx, by, bw, 20, 20, 20, 35);
            let (br, bg, bb) = mix((255, 215, 0));
            buf.rect(bx, by, bw, 1, br, bg, bb);
            let (r, g, b) = if t.at.is_some() { mix((255, 232, 160)) } else { mix((255, 220, 130)) };
            draw_text(&mut buf, bx + 8, by + 6, &t.text, r, g, b);
            if hovered {
                let (hr, hg, hb) = mix((255, 240, 170));
                buf.rect(bx, by + 19, bw, 1, hr, hg, hb);
            }
        }

        // season banner
        let hud = hud_text(season, tick, food_stock, m.precip.map(|k| k.label()), m.paused, m.speed, m.food_yield, m.harsh_winter);
        // on a dark strip so it reads over any terrain the camera shows
        buf.rect(4, 4, text_width(&hud, 1) + 12, GLYPH_H + 8, 20, 20, 32);
        draw_text_outlined(&mut buf, 10, 8, &hud, 255, 255, 255);
        // the seat's own bag and belly ride under it
        let strip = satchel_strip(m);
        if !strip.is_empty() {
            buf.rect(4, 20, text_width(&strip, 1) + 12, GLYPH_H + 8, 20, 20, 32);
            draw_text_outlined(&mut buf, 10, 24, &strip, 200, 190, 170);
        }

        // help overlay
        if self.help_open { draw_help(&mut buf); }

        // the board's postings: the notices panel, read at the market
        if m.notice_panel && !m.notices.is_empty() {
            let lines: Vec<String> = m.notices.iter()
                .map(|(k, t)| format!("[{}] {}", k, t)).collect();
            let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
            draw_overlay(&mut buf, 480, "the board's postings", (255, 240, 200), &refs, 120, None);
        }
        // a flyer's hover: mouse near a posted door and its words show
        if let Some((mx, my)) = self.mouse {
            for &(fx, fy, ref f) in &m.postings {
                let (sx, sy) = iso_to_screen_cam(fx as f32, fy as f32, cam_x, cam_y, zoom);
                if (mx - sx).abs() < 20 && (my - sy).abs() < 20 {
                    draw_overlay(&mut buf, 300, f, (220, 200, 160), &[f.as_str()], 60, None);
                }
            }
        }
        // the ledger overlay: the public KPIs the model carried (F3)
        if self.ledger_open && !m.debug_lines.is_empty() {
            let rows = m.debug_lines.len();
            draw_rect(&mut buf, 10, 60, 200, rows as i32 * 12 + 22, 20, 22, 34);
            draw_text(&mut buf, 14, 64, "LEDGER", 150, 150, 180);
            for (i, line) in m.debug_lines.iter().take(14).enumerate() {
                draw_text(&mut buf, 14, 76 + i as i32 * 12, line, 190, 190, 210);
            }
        }

        // pause menu overlay
        if m.paused {
            draw_overlay(&mut buf, PAUSE_W, "PAUSED", (255, 255, 255), &PAUSE_LINES, PAUSE_ROW, None);
        }

        buf
    }
    /// The ground pass: every tile's block (or flat diamond where there's no
    /// art) drawn back to front so nearer blocks cover farther ones, plus the
    /// odd tuft or flower on open grass. Drawn into `buf` at camera
    /// (cam_x, cam_y) - called for the world-space cache layer, not per frame.
    /// The ground pass: every tile's block (or flat diamond where there's no
    /// art) drawn back to front so nearer blocks cover farther ones, plus the
    /// odd tuft or flower on open grass. Drawn into `buf` at camera
    /// (cam_x, cam_y) - called for the world-space cache layer, not per frame.
    /// The ground pass: every tile's block (or flat diamond where there's no
    /// art) drawn back to front so nearer blocks cover farther ones, plus the
    /// odd tuft or flower on open grass. Drawn into `buf` at camera
    /// (cam_x, cam_y) - called for the world-space cache layer, not per frame.
    pub fn draw_ground(&self, m: &UiModel, buf: &mut Buffer, cam_x: i32, cam_y: i32, zoom: f32) {
        let season = m.season;
        let tw = (TILE_W as f32 * zoom) as i32;
        let th = (TILE_H as f32 * zoom) as i32;
        // the sea is level with the land, or the shore blocks' sides showed
        // over it like a strip of grass. Blocks resize as they blit (no
        // scaled copies).
        let set = &self.art.tiles[season_tiles(season)];
        let faces = &self.art.faces[season_tiles(season)];
        // the churchyard's plots, for the worn-earth ground (#673)
        let graveyard: std::collections::HashSet<(i32, i32)> = m.graveyard.iter().copied().collect();
        let mut cache = self.blocks.borrow_mut();
        if cache.zoom != zoom || cache.set != season_tiles(season) {
            *cache = BlockCache { zoom, set: season_tiles(season), blocks: Default::default() };
        }
        let (w_t, h_t) = (m.map.width as i32, m.map.height as i32);
        let (x0, y0, x1, y1) = visible_tile_bounds(cam_x, cam_y, zoom, buf.width, buf.height, w_t, h_t);
        if x0 > x1 || y0 > y1 { return; }
        for d in x0 + y0..=x1 + y1 {
            for tx in (d - y1).max(x0)..=d.min(x1).min(d - y0) {
                let ty = d - tx;
                let Some(tile) = m.map.get(tx, ty) else { continue };
                let (sx, sy) = iso_to_screen_cam(tx as f32, ty as f32, cam_x, cam_y, zoom);
                if sx < -2 * tw || sx > buf.width as i32 + 2 * tw || sy < -4 * th || sy > buf.height as i32 + 4 * th { continue; }
                // the churchyard's plots are worn earth, not grass
                if graveyard.contains(&(tx, ty)) {
                    let (r, g, b) = graveyard_dirt(season);
                    buf.diamond(sx, sy, tw, th, r, g, b);
                    buf.diamond(sx, sy + th, tw, 1, r / 2, g / 2, b / 2);
                    continue;
                }
                // inland the blocks in front cover a tile's sides: just its face
                let inland = tx + 1 < w_t && ty + 1 < h_t;
                let set = if inland { faces } else { set };
                match set.block(&m.map, tx, ty) {
                    Some(block) => {
                        let (w, h, x, y) = block_rect(block, sx, sy, zoom);
                        let (scaled, runs) = cache.blocks.entry(block as *const Sprite as usize).or_insert_with(|| {
                            let mut s = block.scaled(w, h);
                            let runs = s.opaque_runs();
                            (s, runs)
                        });
                        scaled.blit_runs(runs, buf, x, y);
                    }
                    None => {
                        let (r, g, b) = tile_colour(tile, season);
                        buf.diamond(sx, sy, tw, th, r, g, b);
                        buf.diamond(sx, sy + th, tw, 1, r / 2, g / 2, b / 2);
                    }
                }
                // the odd tuft or flower on open grass outside the walls
                let in_town = (m.town_wall.0..=m.town_wall.2).contains(&tx)
                    && (m.town_wall.1..=m.town_wall.3).contains(&ty);
                if tile == Tile::Grass && !in_town && !m.map.is_occupied(tx, ty) && noise(tx, ty, 7) < 0.10 {
                    if let Some(d) = pick(&self.art.details[season_tiles(season)], tx, ty, 8) {
                        let k = self.art.details_k * zoom;
                        let (dw, dh) = ((d.width as f32 * k).round() as i32, (d.height as f32 * k).round().max(1.0) as i32);
                        let (jx, jy) = (noise(tx, ty, 10) - 0.5, noise(tx, ty, 11) - 0.5);
                        draw_zoomed(buf, d,
                            sx + (jx * TILE_W as f32 * zoom) as i32 - dw / 2,
                            sy + ((jy - 0.6) * TILE_H as f32 * zoom) as i32 - dh, k);
                    }
                }
            }
        }
    }

    /// Hand a frame from render() back once it's been shown, so the next
    /// render can draw into it
    pub fn recycle(&self, buf: Buffer) { *self.spare.borrow_mut() = Some(buf); }

    /// Render just the visible ground viewport. Cache size is independent of
    /// total map area, so enlarging the country cannot allocate a giant layer.
    pub fn render_ground(&self, m: &UiModel, zoom: f32, camera: (i32, i32)) -> GroundCache {
        let mut buf = Buffer::new(MAP_W, H);
        buf.rect(0, 0, MAP_W as i32, H as i32, 25, 25, 38);
        self.draw_ground(m, &mut buf, camera.0, camera.1, zoom);
        GroundCache { buf, camera, zoom, season: m.season, rev: m.map.rev() }
    }

}

/// Who the find box's query fits, as indexes into `people`: a folded
/// substring match on each person's `find` text (name and posting)
pub fn search_people(people: &[PersonV], query: &str) -> Vec<usize> {
    let q = find_key(query);
    if q.is_empty() { return vec![]; }
    people.iter().enumerate()
        .filter(|(_, p)| !p.find.is_empty() && p.find.contains(&q))
        .map(|(i, _)| i).collect()
}

/// How far past the map view's rim the character's feet may sit before
/// the camera counts them as lost - about a sprite's worth
const LOST_MARGIN: i32 = 48;

/// The camera follows a character who walks clean off the map view:
/// while they move - or the zoom shifts them out of sight - and their
/// feet are out of sight, look back to have them in the middle again.
/// `seen` is where they stood and the zoom last frame: a deliberate
/// pan or jump with them standing still isn't fought. Returns the
/// camera to use, when it should move.
pub fn camera_follow(m: &UiModel, cam: (i32, i32), zoom: f32, seen: &mut Option<((f32, f32), f32)>) -> Option<(i32, i32)> {
    let p = m.people.iter().find(|p| p.kind == PKind::Player)?;
    let pos = p.pos;
    let moved = seen.is_some_and(|(q, z)| (q.0 - pos.0).abs() > 0.01 || (q.1 - pos.1).abs() > 0.01 || (z - zoom).abs() > 0.001);
    *seen = Some((pos, zoom));
    let (sx, sy) = iso_to_screen_cam(pos.0, pos.1, cam.0, cam.1, zoom);
    let lost = sx < -LOST_MARGIN || sx > MAP_W as i32 + LOST_MARGIN
        || sy < -LOST_MARGIN || sy > H as i32 + LOST_MARGIN;
    (moved && lost).then(|| camera_on(pos.0, pos.1))
}

/// The find box under the minimap: "find:" and what was typed, with the
/// match count (or the notice that no one fits) at the right
pub fn draw_search_box(buf: &mut Buffer, query: &str, hits: usize, focused: bool) {
    buf.rect(SRCH_X, SRCH_Y, SRCH_W, SRCH_H, 30, 30, 48);
    buf.rect(SRCH_X, SRCH_Y, SRCH_W, 1, 60, 60, 80);
    buf.rect(SRCH_X, SRCH_Y + SRCH_H - 1, SRCH_W, 1, 60, 60, 80);
    buf.rect(SRCH_X, SRCH_Y, 1, SRCH_H, 60, 60, 80);
    buf.rect(SRCH_X + SRCH_W - 1, SRCH_Y, 1, SRCH_H, 60, 60, 80);
    draw_text(buf, SRCH_X + 6, SRCH_Y + 8, "find:", 140, 140, 160);
    let q = if focused && query.is_empty() { "type a name".to_string() } else { query.to_string() };
    let (qr, qg, qb) = if query.is_empty() { (110, 110, 130) } else { (235, 235, 245) };
    draw_text(buf, SRCH_X + 40, SRCH_Y + 8, &q, qr, qg, qb);
    let status = if query.is_empty() { String::new() }
        else if hits == 0 { "no one by that name".into() }
        else { format!("{} found", hits) };
    let sw = status.len() as i32 * GLYPH_W;
    draw_text(buf, SRCH_X + SRCH_W - sw - 6, SRCH_Y + 8, &status, 150, 150, 170);
    if focused {
        // a caret after the typed text so it's plain the box has the keys
        let cx = SRCH_X + 40 + query.len() as i32 * GLYPH_W + 1;
        buf.rect(cx, SRCH_Y + 6, 1, 10, 220, 220, 235);
    }
}

/// The character picker: the seed's roster as cards - a figure, a name,
/// who they are - two to a row, the selection marked. Up/down (or W/S)
/// chooses, Enter takes them, Escape keeps the default wanderer.
pub fn render_picker(art: &Art, cards: &[roster::RosterCard], sel: usize) -> Buffer {
    let mut buf = Buffer::new(W, H);
    buf.rect(0, 0, W as i32, H as i32, 15, 15, 25);
    draw_centered(&mut buf, 70, "THE WANDERER", 4, (255, 220, 130));
    draw_centered(&mut buf, 140, "choose who you are", 2, (200, 200, 200));
    let cols = 2usize;
    let per_col = (cards.len() + cols - 1) / cols;
    let (cw, ch) = (580i32, 118i32);
    let (x0, y0) = (130i32, 180i32);
    for (i, c) in cards.iter().enumerate() {
        let (cx, cy) = (x0 + (i / per_col) as i32 * cw, y0 + (i % per_col) as i32 * ch);
        let on = i == sel;
        // card plate
        let (pr, pg, pb): (u8, u8, u8) = if on { (44, 44, 66) } else { (30, 30, 46) };
        buf.rect(cx, cy, cw - 20, ch - 10, pr, pg, pb);
        if on {
            buf.rect(cx, cy, cw - 20, 1, 255, 220, 130);
            buf.rect(cx, cy + ch - 11, cw - 20, 1, 255, 220, 130);
            buf.rect(cx, cy, 1, ch - 10, 255, 220, 130);
            buf.rect(cx + cw - 21, cy, 1, ch - 10, 255, 220, 130);
        }
        // the figure: the pack's look, small for a child
        if !art.villager_sprites.is_empty() {
            let look = figure_look(c.kid, c.look, art.villager_sprites.len());
            let s = &art.villager_sprites[look];
            let scale = if c.kid { 1.1 } else { 1.5 };
            let fw = (s.width as f32 * scale) as i32;
            let fh = (s.height as f32 * scale) as i32;
            draw_zoomed(&mut buf, s, cx + 24 - fw / 2, cy + 60 - fh, scale);
        }
        let tx = cx + 96;
        let (nr, ng, nb): (u8, u8, u8) = if on { (255, 220, 130) } else { (225, 225, 235) };
        draw_text_scaled(&mut buf, tx, cy + 12, &c.name, 2, nr, ng, nb);
        draw_text(&mut buf, tx, cy + 38, &c.work, 140, 140, 160);
        // the backstory wraps to the card's width
        let mut line = String::new();
        let mut ly = cy + 60;
        for word in c.backstory.split(' ') {
            if line.len() + word.len() + 1 > 62 {
                draw_text(&mut buf, tx, ly, &line, 190, 190, 205);
                ly += 14;
                line.clear();
                if ly > cy + ch - 22 { break; }
            }
            if !line.is_empty() { line.push(' '); }
            line.push_str(word);
        }
        if ly <= cy + ch - 22 { draw_text(&mut buf, tx, ly, &line, 190, 190, 205); }
        if on { draw_text(&mut buf, cx + 8, cy + 8, ">", 255, 220, 130); }
    }
    draw_centered(&mut buf, y0 + ((cards.len() + 1) / 2) as i32 * ch + 30,
        "up/down choose    ENTER take them    ESC keep the wanderer", 1, (150, 150, 170));
    buf
}
