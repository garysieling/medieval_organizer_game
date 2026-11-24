//! The place-based soundscape (#1496, A3): a pure function of the
//! `UiModel` and a `Listener` that turns what the camera can see into
//! positioned emitters - keynote beds, one-shot signals and landmark
//! soundmarks, in the design's three tiers. No audio, no randomness:
//! A4's mixer plays what this returns.
//!
//! The emitter table is fixed by wiki/software-design/sound-system.md's
//! "Place-based soundscape" section; each row's rule is its own block in
//! `scan` below.
#[cfg(test)]
mod tests;
pub mod ambience;

use game2_core::game::tilemap::{Tile, WallPiece};
use game2_core::session::model::{guard_pose, pose};
use game2_core::session::{BSprite, PKind, UiModel, WorkLook, MAP_W, TILE_W};
use game2_core::sim::climate::Season;
use game2_core::sim::precipitation::PrecipKind;

/// Where hearing sits: tile space, with the view's half-width in px so
/// pan can be normalised (`session/layout.rs` screen maths).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Listener {
    pub x: f32,
    pub y: f32,
    /// How far the ear reaches, `clamp(10/zoom, 5, 20)`: beds gather
    /// from 1.5x this, one-shots play only inside it
    pub radius_tiles: f32,
    /// Half the map view's width in px, the pan normaliser
    pub screen_half_w: f32,
}

/// The seat's ear radius at zoom: `clamp(10/zoom, 5, 20)`
pub fn radius_at(zoom: f32) -> f32 { (10.0 / zoom).clamp(5.0, 20.0) }

impl Listener {
    /// The model's own ear. `UiModel` doesn't carry the camera - the
    /// client owns it - so this hears where the camera follows: the
    /// seat's player, at the default zoom. Clients with a real camera
    /// build `Listener::at`; fields are public for that.
    pub fn from_model(m: &UiModel) -> Self {
        let at = m.people.iter().find(|p| p.kind == PKind::Player)
            .map(|p| p.pos)
            .unwrap_or((m.map.width as f32 / 2.0, m.map.height as f32 / 2.0));
        Listener::at(at.0, at.1, 1.0)
    }

    /// The ear at a tile spot and zoom (the client's camera, or a
    /// test's staged view)
    pub fn at(x: f32, y: f32, zoom: f32) -> Self {
        Listener { x, y, radius_tiles: radius_at(zoom),
            screen_half_w: MAP_W as f32 / 2.0 }
    }
}

/// The design's three tiers: looped keynote beds, positioned one-shot
/// signals, and a place's unique soundmarks
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier { Bed, Signal, Soundmark }

/// One variant per row of the design's emitter table
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EmitterKind {
    Songbirds, Insects, Bees, River, Wind, Rain, SnowHush, Haar,
    VillageMurmur, Market, Smithy, Work, ChurchBell, Chant, Tavern, Fold,
    Homesteads, Cart, Gate, Mill, Boats, Fire, Graveyard, StandingStones,
    Cave, Castle, Guards, March,
}

/// One emitter the scan found: what, how loud, where it pans and how
/// often a one-shot may fire (the mixer's Poisson rate and gap)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Emitter {
    pub kind: EmitterKind,
    pub tier: Tier,
    /// Tile space; `None` for an unlocated bed (pan centre, no distance)
    pub pos: Option<(f32, f32)>,
    /// Sort weight 0..1 - how loud a voice it is in its tier
    pub weight: f32,
    pub gain: f32,
    pub pan: f32,
    pub lowpass_hz: f32,
    pub reverb: f32,
    pub rate_hz: f32,
    pub min_gap_s: f32,
}

/// Equal-power pan from a screen-x offset normalised by the half-width
/// (the power law itself is the mixer's; this is its -1..1 position)
pub fn pan(emitter_x_screen: f32, half_w: f32) -> f32 {
    (emitter_x_screen / half_w).clamp(-1.0, 1.0)
}

/// The design's distance curve: `1 / (1 + (d / (radius/3))²)`
pub fn gain(d: f32, radius: f32) -> f32 {
    let r = d / (radius / 3.0);
    1.0 / (1.0 + r * r)
}

/// The low-pass falls from 16 kHz at the listener to 2 kHz at the radius
pub fn lowpass_hz(d: f32, radius: f32) -> f32 {
    16000.0 - (14000.0 * (d / radius).clamp(0.0, 1.0))
}

/// Zoomed out past a 20-tile radius the one-shots drop out and the
/// village blurs into its murmur (the design's "fully zoomed out" rule)
const ONE_SHOT_MAX_RADIUS: f32 = 20.0;

fn dist(x: f32, y: f32, l: &Listener) -> f32 {
    ((x - l.x).powi(2) + (y - l.y).powi(2)).sqrt()
}

/// The screen-x offset of a tile spot, using `layout`'s isometric x
/// (px at zoom 1 - pan normalises it away)
fn screen_x(x: f32, y: f32, l: &Listener) -> f32 {
    ((x - y) - (l.x - l.y)) * TILE_W as f32
}

/// Spatial terms for a positioned emitter: distance shapes gain,
/// low-pass and the reverb send
fn spatial(e: &mut Emitter, l: &Listener) {
    if let Some((x, y)) = e.pos {
        let d = dist(x, y, l);
        e.gain *= gain(d, l.radius_tiles);
        e.pan = pan(screen_x(x, y, l), l.screen_half_w);
        e.lowpass_hz = lowpass_hz(d, l.radius_tiles);
        e.reverb = (d / l.radius_tiles).clamp(0.0, 1.0);
    }
}

fn emit(out: &mut Vec<Emitter>, l: &Listener, kind: EmitterKind, tier: Tier,
        pos: Option<(f32, f32)>, weight: f32, gain: f32, rate_hz: f32, min_gap_s: f32) {
    let mut e = Emitter { kind, tier, pos, weight, gain,
        pan: 0.0, lowpass_hz: 16000.0, reverb: 0.0, rate_hz, min_gap_s };
    spatial(&mut e, l);
    out.push(e);
}

fn in_r(x: f32, y: f32, l: &Listener, r: f32) -> bool { dist(x, y, l) <= r }

/// Share of the tiles in radius that are `kinds` (0..1): the "share of
/// Grass and Wheat tiles" / "share of Hill and Rock" rules
fn tile_share(m: &UiModel, l: &Listener, r: f32, kinds: &[Tile]) -> (f32, usize) {
    let (x0, y0) = ((l.x - r).floor() as i32, (l.y - r).floor() as i32);
    let (x1, y1) = ((l.x + r).ceil() as i32, (l.y + r).ceil() as i32);
    let (mut hit, mut seen) = (0usize, 0usize);
    for ty in y0..=y1 {
        for tx in x0..=x1 {
            if !in_r(tx as f32 + 0.5, ty as f32 + 0.5, l, r) { continue; }
            if let Some(t) = m.map.get(tx, ty) {
                seen += 1;
                if kinds.contains(&t) { hit += 1; }
            }
        }
    }
    (if seen > 0 { hit as f32 / seen as f32 } else { 0.0 }, hit)
}

fn nearest<'a, I: Iterator<Item = &'a (f32, f32)>>(l: &Listener, it: I, r: f32)
        -> Option<(f32, f32)> {
    it.filter(|p| in_r(p.0, p.1, l, r))
        .min_by(|a, b| dist(a.0, a.1, l).partial_cmp(&dist(b.0, b.1, l)).unwrap())
        .copied()
}

/// Every emitter the seat can hear this frame: the design's table, one
/// block per row, then a stable sort (tier, weight desc, kind, spot) so
/// a deterministic order comes out of any model
pub fn scan(m: &UiModel, l: &Listener) -> Vec<Emitter> {
    let mut out = vec![];
    let r = l.radius_tiles;
    let bed_r = r * 1.5;
    let one_shots = r < ONE_SHOT_MAX_RADIUS;
    let cal = &m.calendar;
    let holy = cal.holy_day;
    let feast = m.feast || cal.feast.is_some();
    let snowing = matches!(m.precip, Some(PrecipKind::Snow | PrecipKind::HeavySnow));
    let heavy_rain = m.precip == Some(PrecipKind::HeavyRain);

    // Songbirds: 5+ trees in radius, chirps placed at trees. Strong in
    // spring/summer, light in autumn, winter keeps only crows and a
    // robin; off in heavy rain or snow.
    let trees: Vec<(i32, i32)> = m.map.trees.iter().copied()
        .filter(|&(x, y)| in_r(x as f32 + 0.5, y as f32 + 0.5, l, r)).collect();
    if trees.len() >= 5 && !heavy_rain && !snowing {
        let season_w = match m.season {
            Season::Spring | Season::Summer => 1.0,
            Season::Autumn => 0.5, Season::Winter => 0.15,
        };
        emit(&mut out, l, EmitterKind::Songbirds, Tier::Bed, None,
            0.5 * season_w, season_w, 0.0, 0.0);
        if one_shots {
            let mut near: Vec<(i32, i32)> = trees.clone();
            near.sort_by(|a, b| dist(a.0 as f32, a.1 as f32, l)
                .partial_cmp(&dist(b.0 as f32, b.1 as f32, l)).unwrap());
            for &(x, y) in near.iter().take(4) {
                emit(&mut out, l, EmitterKind::Songbirds, Tier::Signal,
                    Some((x as f32 + 0.5, y as f32 + 0.5)), 0.4 * season_w,
                    0.8 * season_w, 0.4, 2.0);
            }
        }
    }

    // Insects: share of Grass and Wheat tiles, summer only, louder in a
    // heat wave
    if m.season == Season::Summer {
        let (share, _) = tile_share(m, l, bed_r, &[Tile::Grass, Tile::Wheat]);
        if share > 0.05 {
            let heat = if m.weather.as_ref().is_some_and(|w| w.kind == "heat") { 1.4 } else { 1.0 };
            emit(&mut out, l, EmitterKind::Insects, Tier::Bed, None,
                share, share * heat, 0.0, 0.0);
        }
    }

    // Bees: skeps within 6 tiles, off in winter, louder at a swarm
    if m.season != Season::Winter {
        if let Some((x, y)) = nearest(l, m.skeps.iter(), 6.0) {
            let swarm = m.swarm.filter(|p| in_r(p.0, p.1, l, bed_r));
            let (pos, g) = swarm.map_or((Some((x, y)), 0.8), |p| (Some(p), 1.2));
            // the point bed at the skeps is the loudest thing in the
            // radius - the design's check is "bees are the top emitter"
            emit(&mut out, l, EmitterKind::Bees, Tier::Bed, pos, 1.0, g, 0.0, 0.0);
        }
    }

    // River: water tiles in radius weighted by count; flood rushes
    let (_, water) = tile_share(m, l, bed_r, &[Tile::River, Tile::Ford]);
    let bridges = m.map.bridges()
        .filter(|&(x, y)| in_r(x as f32 + 0.5, y as f32 + 0.5, l, bed_r)).count();
    if water + bridges > 0 {
        let w = ((water + bridges) as f32 / 20.0).min(1.0);
        let g = if !m.flood.is_empty() { 1.5 } else { 1.0 };
        // pan on the nearest water under it, if one's in earshot
        let pos = m.map.bridges().map(|(x, y)| (x as f32 + 0.5, y as f32 + 0.5))
            .filter(|p| in_r(p.0, p.1, l, bed_r)).next();
        emit(&mut out, l, EmitterKind::River, Tier::Bed, pos, w, w * g, 0.0, 0.0);
    }

    // Wind: share of Hill and Rock; stronger in winter, gusts in a storm
    let (share, _) = tile_share(m, l, bed_r, &[Tile::Hill, Tile::Rock]);
    if share > 0.05 {
        let winter = if m.season == Season::Winter { 1.4 } else { 1.0 };
        let storm = m.weather.as_ref()
            .map(|w| if w.kind == "storm" { 1.0 + 0.5 * w.intensity } else { 1.0 })
            .unwrap_or(1.0);
        emit(&mut out, l, EmitterKind::Wind, Tier::Bed, None,
            (share * winter).min(1.0), share * winter * storm, 0.0, 0.0);
    }

    // Rain: the precip bed, level by kind (Showers come and go)
    let rain_w = match m.precip {
        Some(PrecipKind::Drizzle) => 0.4, Some(PrecipKind::LightRain) => 0.55,
        Some(PrecipKind::Showers) => 0.65, Some(PrecipKind::Rain) => 0.8,
        Some(PrecipKind::HeavyRain) => 1.0, _ => 0.0,
    };
    if rain_w > 0.0 {
        emit(&mut out, l, EmitterKind::Rain, Tier::Bed, None, rain_w, rain_w, 0.0, 0.0);
    }

    // Snow hush: the master low-pass at 5 kHz - the muffled world itself
    if snowing {
        let mut e = Emitter { kind: EmitterKind::SnowHush, tier: Tier::Bed,
            pos: None, weight: 0.6, gain: 0.7, pan: 0.0,
            lowpass_hz: 5000.0, reverb: 0.0, rate_hz: 0.0, min_gap_s: 0.0 };
        spatial(&mut e, l);
        out.push(e);
    }

    // Haar: low-pass at 4 kHz, more reverb (the distant bell rides the
    // row's own bed rather than a separate emitter)
    if m.precip == Some(PrecipKind::Haar) {
        let mut e = Emitter { kind: EmitterKind::Haar, tier: Tier::Bed,
            pos: None, weight: 0.7, gain: 0.8, pan: 0.0,
            lowpass_hz: 4000.0, reverb: 0.8, rate_hz: 0.0, min_gap_s: 0.0 };
        spatial(&mut e, l);
        out.push(e);
    }

    // Village murmur: the crowd in radius scaled by count, quieter
    // while a service runs
    let folk = m.people.iter()
        .filter(|p| !p.dead
            && !matches!(p.kind, PKind::Pet(_) | PKind::Chicken | PKind::Gull | PKind::Sheep | PKind::Goat)
            && in_r(p.pos.0, p.pos.1, l, bed_r)).count();
    if folk >= 3 {
        let service = if holy { 0.7 } else { 1.0 };
        emit(&mut out, l, EmitterKind::VillageMurmur, Tier::Bed, None,
            (folk as f32 / 15.0).min(1.0) * 0.8, (folk as f32 / 15.0).min(1.0) * service,
            0.0, 0.0);
    }

    // Market: the crowd bed and the criers' calls on a market day
    if cal.market_day && !m.stalls.is_empty() {
        let pos = m.market.or(m.stalls.first().map(|s| (s.0, s.1)));
        emit(&mut out, l, EmitterKind::Market, Tier::Bed, pos, 0.85, 1.0, 0.0, 0.0);
        if one_shots {
            emit(&mut out, l, EmitterKind::Market, Tier::Signal, pos, 0.7, 0.9, 0.2, 4.0);
        }
    }

    // Smithy: a hammer on the anvil's rhythm - not on a holy day
    // (also feeds the generic work row below)
    if !holy && one_shots {
        for b in m.buildings.iter().filter(|b| b.sprite == BSprite::Blacksmith) {
            let (bx, by) = (b.x, b.y);
            if !in_r(bx, by, l, r) { continue; }
            if m.people.iter().any(|p| p.pose == pose::HAMMERING
                    && (p.pos.0 - bx).abs() + (p.pos.1 - by).abs() <= 4.0) {
                emit(&mut out, l, EmitterKind::Smithy, Tier::Signal,
                    Some((bx, by)), 0.7, 1.0, 0.8, 0.8);
            }
        }
    }

    // Work sounds: a one-shot at each hammering or hoeing figure
    if one_shots {
        for p in m.people.iter()
            .filter(|p| matches!(p.pose, pose::HAMMERING | pose::HOEING)
                && in_r(p.pos.0, p.pos.1, l, r)) {
            emit(&mut out, l, EmitterKind::Work, Tier::Signal, Some(p.pos),
                0.5, 0.8, 0.5, 3.0);
        }
    }

    // Church bell (soundmark) and chant (diegetic loop) on a holy day
    if let Some(church) = m.buildings.iter().find(|b| b.sprite == BSprite::Church) {
        let at = Some((church.x, church.y));
        if holy {
            if in_r(church.x, church.y, l, r) {
                emit(&mut out, l, EmitterKind::ChurchBell, Tier::Soundmark,
                    at, 1.0, 1.0, 0.1, 8.0);
            }
            if in_r(church.x, church.y, l, r / 2.0) {
                emit(&mut out, l, EmitterKind::Chant, Tier::Signal, at, 0.8, 0.9, 0.0, 0.0);
            }
        }
    }

    // Tavern: muffled lute and laughter within 8 tiles, louder on feast
    // days
    if let Some(t) = m.buildings.iter()
        .find(|b| matches!(b.sprite, BSprite::Work(WorkLook::Tavern, _))
            && in_r(b.x, b.y, l, 8.0)) {
        let g = if feast { 1.3 } else { 0.7 };
        emit(&mut out, l, EmitterKind::Tavern, Tier::Signal,
            Some((t.x, t.y)), 0.6, g, 0.15, 3.0);
    }

    // Fold: bleats and sheep bells at the fold
    if let Some(f) = m.buildings.iter()
        .find(|b| matches!(b.sprite, BSprite::Work(WorkLook::Fold, _))
            && in_r(b.x, b.y, l, r)) {
        emit(&mut out, l, EmitterKind::Fold, Tier::Signal, Some((f.x, f.y)),
            0.55, 0.8, 0.25, 2.0);
    }

    // Homesteads: density of houses - chickens, a dog, a door
    let homes: Vec<(f32, f32)> = m.buildings.iter()
        .filter(|b| matches!(b.sprite, BSprite::House(_) | BSprite::Rowhouse(_)
            | BSprite::Mansion | BSprite::Burgh(_))
            && in_r(b.x, b.y, l, r)).map(|b| (b.x, b.y)).collect();
    if homes.len() >= 3 && one_shots {
        let cx = homes.iter().map(|p| p.0).sum::<f32>() / homes.len() as f32;
        let cy = homes.iter().map(|p| p.1).sum::<f32>() / homes.len() as f32;
        emit(&mut out, l, EmitterKind::Homesteads, Tier::Signal, Some((cx, cy)),
            (homes.len() as f32 / 10.0).min(1.0) * 0.5, 0.6, 0.2, 3.0);
    }

    // Carts and riders: wheels and hooves at each train and mounted
    // figure
    if one_shots {
        for &(x, y) in m.trains.iter().filter(|p| in_r(p.0, p.1, l, r)) {
            emit(&mut out, l, EmitterKind::Cart, Tier::Signal, Some((x, y)),
                0.6, 0.9, 0.3, 1.5);
        }
        for p in m.people.iter()
            .filter(|p| p.mounted && in_r(p.pos.0, p.pos.1, l, r)) {
            emit(&mut out, l, EmitterKind::Cart, Tier::Signal, Some(p.pos),
                0.5, 0.7, 0.3, 1.5);
        }
    }

    // Gate: a train or figure on a gate tile - creak and clatter
    if one_shots {
        for (gx, gy) in m.map.walls.iter()
            .filter(|(_, p)| matches!(p, WallPiece::GateAlongX | WallPiece::GateAlongY))
            .map(|(t, _)| *t) {
            if !in_r(gx as f32 + 0.5, gy as f32 + 0.5, l, r) { continue; }
            let on = m.trains.iter().chain(m.people.iter().map(|p| &p.pos))
                .any(|p| ((p.0 - gx as f32 - 0.5).abs() + (p.1 - gy as f32 - 0.5).abs()) <= 1.5);
            if on {
                emit(&mut out, l, EmitterKind::Gate, Tier::Signal,
                    Some((gx as f32 + 0.5, gy as f32 + 0.5)), 0.7, 1.0, 0.5, 1.0);
            }
        }
    }

    // Mill: the wheel-creak loop, stopped in a hard frost
    if !m.harsh_winter {
        if let Some(at) = m.mill.filter(|p| in_r(p.0, p.1, l, r)) {
            emit(&mut out, l, EmitterKind::Mill, Tier::Soundmark, Some(at),
                0.9, 0.9, 0.0, 0.0);
        }
    }

    // Boats: lapping water and rope creak at the moorings
    if one_shots {
        for &(x, y) in m.boats.iter().take(4)
            .filter(|p| in_r(p.0 as f32 + 0.5, p.1 as f32 + 0.5, l, r)) {
            emit(&mut out, l, EmitterKind::Boats, Tier::Signal,
                Some((x as f32 + 0.5, y as f32 + 0.5)), 0.4, 0.6, 0.2, 4.0);
        }
    }

    // Fire: the crackle bed, and the alarm bell as a signal
    if let Some(&first) = m.burning.first() {
        let pos = Some(first);
        emit(&mut out, l, EmitterKind::Fire, Tier::Bed, pos, 0.95, 1.0, 0.0, 0.0);
        if one_shots {
            emit(&mut out, l, EmitterKind::Fire, Tier::Signal, pos, 1.0, 1.0, 0.4, 1.0);
        }
    }

    // Graveyard: crows, the ambience 3 dB down
    let graves = m.graveyard.iter().map(|&(x, y)| (x as f32 + 0.5, y as f32 + 0.5))
        .chain(m.grave).find(|p| in_r(p.0, p.1, l, r));
    if let (Some(at), true) = (graves, one_shots) {
        emit(&mut out, l, EmitterKind::Graveyard, Tier::Signal, Some(at),
            0.4, 0.7, 0.15, 5.0);
    }

    // Standing stones: the wind-hum soundmark, stronger in wind
    if !m.stones.is_empty() {
        let at = m.stones.iter().map(|&(x, y)| (x as f32 + 0.5, y as f32 + 0.5))
            .find(|p| in_r(p.0, p.1, l, r));
        if let Some(at) = at {
            let (windy, _) = tile_share(m, l, r, &[Tile::Hill, Tile::Rock]);
            emit(&mut out, l, EmitterKind::StandingStones, Tier::Soundmark,
                Some(at), 0.7, 0.5 + windy, 0.0, 0.0);
        }
    }

    // Cave: drips, reverb up
    let cave = m.cave.filter(|p| in_r(p.0, p.1, l, r))
        .or_else(|| m.cave_zone.map(|(a, b)| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0))
            .filter(|p| in_r(p.0, p.1, l, r)));
    if let (Some(at), true) = (cave, one_shots) {
        let mut e = Emitter { kind: EmitterKind::Cave, tier: Tier::Signal,
            pos: Some(at), weight: 0.4, gain: 0.6, pan: 0.0,
            lowpass_hz: 0.0, reverb: 0.9, rate_hz: 0.3, min_gap_s: 3.0 };
        spatial(&mut e, l);
        e.reverb = (e.reverb + 0.5).min(1.0);
        out.push(e);
    }

    // Castle: banners flapping on the keep, armour while guards post
    if let Some(at) = m.castle_keeps.iter().find(|p| in_r(p.0, p.1, l, r)) {
        emit(&mut out, l, EmitterKind::Castle, Tier::Soundmark, Some(*at),
            0.75, 0.7, 0.0, 0.0);
    }

    // Guards: the watch's armour and footsteps while they stand ready
    if one_shots {
        for p in m.people.iter()
            .filter(|p| matches!(p.kind, PKind::Guard(_)) && p.pose == guard_pose::READY
                && in_r(p.pos.0, p.pos.1, l, r)) {
            emit(&mut out, l, EmitterKind::Guards, Tier::Signal, Some(p.pos),
                0.5, 0.7, 0.3, 2.0);
        }
    }

    // March or protest: the moving bed of footsteps and chant the
    // camera's already tracking
    if let Some(ev) = m.player_event {
        if ev.kind == 0 {
            emit(&mut out, l, EmitterKind::March, Tier::Bed, Some(ev.pos),
                0.9, 1.0, 0.0, 0.0);
        }
    }

    // stable order: tier, then weight desc, then kind, then the spot
    let spot = |e: &Emitter| e.pos.unwrap_or((0.0, 0.0));
    out.sort_by(|a, b| a.tier.cmp(&b.tier)
        .then(b.weight.partial_cmp(&a.weight).unwrap())
        .then(a.kind.cmp(&b.kind))
        .then(spot(a).0.partial_cmp(&spot(b).0).unwrap())
        .then(spot(a).1.partial_cmp(&spot(b).1).unwrap()));
    out
}
