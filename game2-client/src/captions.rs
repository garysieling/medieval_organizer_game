//! Sound captions (U4/#1508): the soundscape's signals named on screen
//! for the player who can't hear them - accessibility, drawn only when
//! `AudioSettings.captions` is on. The scan's non-bed emitters inside
//! the listener radius caption once each for three seconds, at most the
//! nearest three, stacked at the map view's bottom-left, fading out.
//! A signal re-arms only after it leaves the ear entirely.
use super::*;
use game2_audio::soundscape::{Emitter, EmitterKind, Listener, Tier};

/// A caption's screen life in model frames: 3 s at 60 fps
const LIFE: u32 = 180;
/// The last stretch fades toward the chip's dark, in frames
const FADE: u32 = 30;
/// At most the nearest three signals caption at once (the design's cap)
const MAX: usize = 3;

/// The words a signal gets on screen, in the HUD's register. `None`
/// means the kind stays off the captions too - beds, weather shrouds
/// and modifiers aren't a moment worth naming.
fn caption_text(kind: EmitterKind) -> Option<&'static str> {
    use EmitterKind as K;
    Some(match kind {
        K::ChurchBell => "Alarm bell - church",
        K::Smithy => "Hammering",
        K::Work => "Hewing and hauling",
        K::Market => "A crier calls",
        K::Songbirds => "Birdsong",
        K::Chant => "Chanting",
        K::Tavern => "A lute at the tavern",
        K::Fold => "Bleating",
        K::Homesteads => "Hens and hounds",
        K::Cart => "Cart wheels",
        K::Gate => "The gate creaks",
        K::Boats => "Lapping at the boats",
        K::Fire => "A fire!",
        K::Graveyard => "Crows at the graves",
        K::Cave => "Drips in the cave",
        K::Guards => "Armour on the march",
        K::Mill => "The mill wheel",
        K::StandingStones => "The stones hum",
        K::Castle => "Banners at the keep",
        _ => return None,
    })
}

/// The caption feed's memory and this frame's output. `View` owns one;
/// the client calls `update` once per frame with the same scan the
/// audio hears, and `render` draws `rows`.
#[derive(Default)]
pub struct Captions {
    /// kind -> frame it first captioned. Stays while the signal stays
    /// in earshot so it can't re-arm until it leaves.
    seen: Vec<(EmitterKind, u32)>,
    /// What draws this frame: caption text and its 0..1 brightness
    rows: Vec<(String, f32)>,
}

impl Captions {
    /// One frame's worth: a signal in earshot captions on first sight,
    /// holds for `LIFE` frames (fading over the last `FADE`), and can't
    /// re-arm until it leaves the ear. Off means empty - and nothing
    /// tracked, so flipping captions on names what's sounding now.
    pub fn update(&mut self, emitters: &[Emitter], l: &Listener,
                  frame: u32, on: bool) {
        self.rows.clear();
        if !on { self.seen.clear(); return; }
        // audible, located, captioned kinds - nearest first
        let mut near: Vec<(EmitterKind, f32)> = emitters.iter()
            .filter(|e| e.tier != Tier::Bed)
            .filter_map(|e| e.pos.map(|p| (e.kind, p)))
            .filter(|(k, _)| caption_text(*k).is_some())
            .map(|(k, (x, y))| (k, ((x - l.x).powi(2) + (y - l.y).powi(2)).sqrt()))
            .filter(|(_, d)| *d <= l.radius_tiles)
            .collect();
        near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        // one chip per kind - two smithies both say "Hammering", and
        // identical stacked words add nothing; the nearest stands in
        let mut kinds = Vec::new();
        near.retain(|(k, _)| !kinds.contains(k) && { kinds.push(*k); true });
        // signals gone silent re-arm: drop their memory
        self.seen.retain(|(k, _)| near.iter().any(|(nk, _)| nk == k));
        for (k, _) in near.iter().take(MAX) {
            let born = match self.seen.iter().find(|(sk, _)| sk == k) {
                Some((_, b)) => *b,
                None => { self.seen.push((*k, frame)); frame }
            };
            let age = frame.saturating_sub(born);
            if age >= LIFE { continue; }
            let fade = ((LIFE - age) as f32 / FADE as f32).min(1.0);
            self.rows.push((caption_text(*k).unwrap().to_string(), fade));
        }
    }

    /// What draws this frame: caption text and its brightness.
    pub fn rows(&self) -> &[(String, f32)] { &self.rows }
}

/// The chips' home: the map view's bottom-left, stacked upward. When
/// the inspector card is open the feed anchors just above it - the
/// render passes the card's top so the two never overlap.
const BOX_X: i32 = 10;
const ROW: i32 = 22;

/// The captions, in the HUD's dark-chip style: a gold left edge, dim
/// book-hand text, each row eased toward the chip's dark by its fade.
/// `bottom` is the lowest chip's top edge.
pub(crate) fn draw_captions(buf: &mut Buffer, caps: &Captions, bottom: i32) {
    for (i, (text, fade)) in caps.rows().iter().enumerate() {
        let y = bottom - i as i32 * ROW;
        let w = measure(text) + 18;
        let mix = |c: i32| (30.0 + (c as f32 - 30.0) * fade) as u8;
        buf.rect(BOX_X, y, w, 18, 30, 30, 38);
        buf.rect(BOX_X, y, 2, 18, mix(255), mix(215), mix(0));
        draw_text(buf, BOX_X + 9, y + 5, text, mix(230), mix(230), mix(240));
    }
}
