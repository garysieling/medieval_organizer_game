//! The desktop Sound overlay (#1506): a modal off the pause menu that
//! edits the shared `AudioSettings` live. Rows, geometry, key and click
//! handling are pure functions here so `tests/sound_menu.rs` can drive
//! them without a window; `game_ui` owns the open flag and the audio
//! host the changes land on.

use super::*;
use crate::hud::{draw_centered_at, overlay_rect, text_width, MENU_ROW};
use game2_audio::settings::{AudioSettings, Presence};

/// The overlay's width - the design's 360px panel
pub const SOUND_W: i32 = 360;

/// Twelve rows, in the design's order
pub const SOUND_ROWS: usize = 12;

/// Row pitch: MENU_ROW plus its 2px gap, matching the pause menu's
const SOUND_PITCH: i32 = MENU_ROW + 2;

/// The slider track, fixed at the design's 120px; value text sits right
pub const TRACK_W: i32 = 120;

/// What a row controls when an arrow or Enter lands on it
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SoundCtl {
    /// A 0..=100 slider by its `AudioSettings` key
    Slider(&'static str),
    /// The score-presence choice: Off / Sparse / Normal / Rich
    Presence,
    /// An on/off row by its `AudioSettings` key
    Toggle(&'static str),
    /// Restore every row to the U1 defaults
    Reset,
    /// Back to the pause menu (Esc does the same)
    Back,
}

/// The design's row table: label plus control
pub const ROW_DEFS: [(&str, SoundCtl); SOUND_ROWS] = [
    ("Master", SoundCtl::Slider("master")),
    ("Music", SoundCtl::Slider("music")),
    ("Ambience", SoundCtl::Slider("ambience")),
    ("Effects", SoundCtl::Slider("effects")),
    ("Interface", SoundCtl::Slider("interface")),
    ("Score presence", SoundCtl::Presence),
    ("Mono", SoundCtl::Toggle("mono")),
    ("Night mode", SoundCtl::Toggle("night_mode")),
    ("Mute in background", SoundCtl::Toggle("mute_in_background")),
    ("Sound captions", SoundCtl::Toggle("captions")),
    ("Reset to defaults", SoundCtl::Reset),
    ("Back  (Esc)", SoundCtl::Back),
];

/// The overlay's box: centred on the map view, 360px wide, with the
/// footer line the hints sit on
pub fn sound_menu_box() -> (i32, i32, i32, i32) {
    overlay_rect(SOUND_W, SOUND_ROWS, SOUND_PITCH, true)
}

/// Each row's rect, in draw order - the same pitch the box was sized for
pub fn sound_menu_rows() -> [(i32, i32, i32, i32); SOUND_ROWS] {
    let (x, y, w, _) = sound_menu_box();
    let mut rows = [(0, 0, 0, 0); SOUND_ROWS];
    for (i, r) in rows.iter_mut().enumerate() {
        *r = (x + 8, y + 52 + i as i32 * SOUND_PITCH, w - 16, MENU_ROW);
    }
    rows
}

/// The slider's track rect inside row `i` - clicks on it set from x
pub fn track_rect(i: usize) -> (i32, i32, i32, i32) {
    let (rx, ry, rw, rh) = sound_menu_rows()[i];
    (rx + rw - 170, ry + rh / 2 - 3, TRACK_W, 6)
}

/// A slider row's current value by key
fn slider_of(s: &AudioSettings, key: &str) -> u8 {
    match key {
        "master" => s.master,
        "music" => s.music,
        "ambience" => s.ambience,
        "effects" => s.effects,
        _ => s.interface,
    }
}

/// A toggle row's current state by key
fn toggle_of(s: &AudioSettings, key: &str) -> bool {
    match key {
        "mono" => s.mono,
        "night_mode" => s.night_mode,
        "mute_in_background" => s.mute_in_background,
        _ => s.captions,
    }
}

/// The presence choices, in cycle order - index 0 is Off
const PRESENCES: [Presence; 4] =
    [Presence::Off, Presence::Sparse, Presence::Normal, Presence::Rich];

fn presence_ix(s: &AudioSettings) -> usize {
    PRESENCES.iter().position(|p| *p == s.presence).unwrap_or(2)
}

fn presence_name(s: &AudioSettings) -> &'static str {
    match s.presence {
        Presence::Off => "Off",
        Presence::Sparse => "Sparse",
        Presence::Normal => "Normal",
        Presence::Rich => "Rich",
    }
}

fn set_presence(s: &mut AudioSettings, ix: usize) {
    s.presence = PRESENCES[ix % PRESENCES.len()];
}

/// One key's worth of input, mapped in the bin so this stays windowless
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SndKey { Up, Down, Left, Right, Enter, Esc }

/// What the key or click did - the bin applies the setting to the host,
/// marks the store dirty, and plays the Effects preview on `Changed`
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SoundAct {
    /// Focus moved, or the key did nothing
    Move,
    /// The named setting changed (its `AudioSettings` key)
    Changed(&'static str),
    /// Reset landed: `s` already holds the defaults
    Reset,
    /// Esc or Back: return to the pause menu, game still paused
    Close,
}

fn step_slider(s: &mut AudioSettings, key: &'static str, dir: i32) -> SoundAct {
    let v = slider_of(s, key) as i32 + dir * 5;
    let _ = s.apply_setting(key, serde_json::Value::from(v.clamp(0, 100) as u64));
    SoundAct::Changed(key)
}

fn cycle_presence(s: &mut AudioSettings, dir: i32) -> SoundAct {
    let ix = (presence_ix(s) as i32 + dir).rem_euclid(PRESENCES.len() as i32);
    set_presence(s, ix as usize);
    SoundAct::Changed("presence")
}

fn flip_toggle(s: &mut AudioSettings, key: &'static str) -> SoundAct {
    let _ = s.apply_setting(key, serde_json::Value::from(!toggle_of(s, key)));
    SoundAct::Changed(key)
}

/// Keyboard input: arrows or WASD move and adjust, Enter toggles or
/// activates, Esc backs out to the pause menu. Returns the new focus
/// row plus what happened, so the bin can apply and persist the change.
pub fn sound_menu_key(s: &mut AudioSettings, focus: usize, key: SndKey) -> (usize, SoundAct) {
    let (_, ctl) = ROW_DEFS[focus.min(SOUND_ROWS - 1)];
    match key {
        SndKey::Up => ((focus + SOUND_ROWS - 1) % SOUND_ROWS, SoundAct::Move),
        SndKey::Down => ((focus + 1) % SOUND_ROWS, SoundAct::Move),
        SndKey::Esc => (focus, SoundAct::Close),
        SndKey::Left | SndKey::Right => {
            let dir = if key == SndKey::Left { -1 } else { 1 };
            let act = match ctl {
                SoundCtl::Slider(k) => step_slider(s, k, dir),
                SoundCtl::Presence => cycle_presence(s, dir),
                SoundCtl::Toggle(k) => flip_toggle(s, k),
                _ => SoundAct::Move,
            };
            (focus, act)
        }
        SndKey::Enter => {
            let act = match ctl {
                SoundCtl::Presence => cycle_presence(s, 1),
                SoundCtl::Toggle(k) => flip_toggle(s, k),
                SoundCtl::Reset => { *s = AudioSettings::default(); SoundAct::Reset }
                SoundCtl::Back => SoundAct::Close,
                _ => SoundAct::Move,
            };
            (focus, act)
        }
    }
}

/// A click: which row it landed on (the new focus) and what it did.
/// On a slider's track the value comes from x snapped to the 5-grid;
/// left half of a control zone steps back, right half steps forward,
/// and a toggle flips wherever on the control it lands.
pub fn sound_menu_click(mx: i32, my: i32, s: &mut AudioSettings) -> Option<(usize, SoundAct)> {
    for (i, &(rx, ry, rw, rh)) in sound_menu_rows().iter().enumerate() {
        if my < ry || my >= ry + rh || mx < rx || mx >= rx + rw { continue; }
        let (_, ctl) = ROW_DEFS[i];
        let act = match ctl {
            SoundCtl::Slider(k) => {
                let (tx, ty, tw, th) = track_rect(i);
                if mx >= tx && mx < tx + tw && my >= ty - 4 && my < ty + th + 4 {
                    let raw = ((mx - tx) * 100 + tw / 2) / tw;
                    let v = (raw + 2) / 5 * 5;
                    let _ = s.apply_setting(k, serde_json::Value::from(v.clamp(0, 100) as u64));
                    SoundAct::Changed(k)
                } else { SoundAct::Move }
            }
            SoundCtl::Presence => {
                let (tx, _, tw, _) = track_rect(i);
                cycle_presence(s, if mx < tx + tw / 2 { -1 } else { 1 })
            }
            SoundCtl::Toggle(k) => {
                let (tx, _, tw, _) = track_rect(i);
                if mx >= tx && mx < tx + tw { flip_toggle(s, k) } else { SoundAct::Move }
            }
            SoundCtl::Reset => { *s = AudioSettings::default(); SoundAct::Reset }
            SoundCtl::Back => SoundAct::Close,
        };
        return Some((i, act));
    }
    None
}

/// The overlay, drawn over the pause menu it sits on: bordered rows
/// like the menu's, the hover or keyboard focus lighting its row
pub fn draw_sound_menu(buf: &mut Buffer, mouse: (i32, i32), s: &AudioSettings, focus: usize) {
    let (x, y, w, h) = sound_menu_box();
    buf.rect(x, y, w, h, 20, 20, 35);
    buf.rect(x, y, w, 2, 200, 200, 220);
    draw_centered_at(buf, x + w / 2, y + 15, "SOUND", 2, (255, 220, 130));
    for (i, &(rx, ry, rw, rh)) in sound_menu_rows().iter().enumerate() {
        let hot = i == focus
            || (mouse.0 >= rx && mouse.0 < rx + rw && mouse.1 >= ry && mouse.1 < ry + rh);
        let (r, g, b) = if hot { (52, 64, 54) } else { (36, 48, 38) };
        buf.rect(rx, ry, rw, rh, r, g, b);
        let edge = if hot { (150, 230, 150) } else { (70, 110, 70) };
        buf.rect(rx, ry, rw, 1, edge.0, edge.1, edge.2);
        buf.rect(rx, ry + rh - 1, rw, 1, edge.0, edge.1, edge.2);
        buf.rect(rx, ry, 1, rh, edge.0, edge.1, edge.2);
        buf.rect(rx + rw - 1, ry, 1, rh, edge.0, edge.1, edge.2);
        let (label, ctl) = ROW_DEFS[i];
        match ctl {
            SoundCtl::Slider(k) => {
                draw_text(buf, rx + 10, ry + 8, label, 210, 230, 210);
                let (tx, ty, tw, th) = track_rect(i);
                buf.rect(tx, ty, tw, th, 24, 26, 34);
                let v = slider_of(s, k);
                let fill = tw * v as i32 / 100;
                if fill > 0 { buf.rect(tx, ty, fill, th, 210, 180, 100); }
                let val = format!("{}%", v);
                draw_text(buf, rx + rw - 10 - text_width(&val, 1), ry + 8, &val, 210, 210, 190);
            }
            SoundCtl::Presence => {
                draw_text(buf, rx + 10, ry + 8, label, 210, 230, 210);
                let (tx, _, _, _) = track_rect(i);
                let word = format!("< {} >", presence_name(s));
                draw_text(buf, tx, ry + 8, &word, 215, 200, 150);
            }
            SoundCtl::Toggle(k) => {
                draw_text(buf, rx + 10, ry + 8, label, 210, 230, 210);
                let (tx, _, _, _) = track_rect(i);
                let word = if toggle_of(s, k) { "< On >" } else { "< Off >" };
                draw_text(buf, tx, ry + 8, word, 215, 200, 150);
            }
            SoundCtl::Reset =>
                draw_centered_at(buf, rx + rw / 2, ry + 8, label, 1, (215, 200, 150)),
            SoundCtl::Back =>
                draw_centered_at(buf, rx + rw / 2, ry + 8, label, 1, (170, 170, 190)),
        }
    }
    draw_centered_at(buf, x + w / 2, y + h - 22,
        "W/S move - A/D adjust - Enter toggles", 1, (150, 150, 170));
}
