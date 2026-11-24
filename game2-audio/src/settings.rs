//! `AudioSettings` (#1505): the one settings struct both clients show -
//! the desktop's Sound overlay (U2) and the web's dialog (U3) bind to
//! the same rows. Settings never reach the sim or the server.
//!
//! Persistence (desktop): `game2.settings.json` next to `game2.claim`,
//! written a second after the last change and on quit; unknown fields
//! are ignored, missing ones take the defaults, and a corrupt file is
//! a warning plus defaults, never a crash.
use serde::{Deserialize, Serialize};

/// How much of the timeline the score may fill - the director's
/// silence budget. Off hears no score at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Presence { Off, Sparse, Normal, Rich }

impl Default for Presence {
    fn default() -> Self { Presence::Normal }
}

impl Presence {
    /// The share of the timeline the score may fill (the design's
    /// director budget: 0 / 35 / 60 / 85%)
    pub fn budget(self) -> f32 {
        match self {
            Presence::Off => 0.0,
            Presence::Sparse => 0.35,
            Presence::Normal => 0.60,
            Presence::Rich => 0.85,
        }
    }
}

/// The persisted file's schema version.
pub const SETTINGS_VERSION: u32 = 1;

/// The audio rows, in the design's order. Sliders are 0..=100 in steps
/// of 5 (see `clamp`); `serde(default)` means an older or hand-trimmed
/// file still loads, each missing row taking its design default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    pub version: u32,
    pub master: u8,
    pub music: u8,
    pub ambience: u8,
    pub effects: u8,
    pub interface: u8,
    pub presence: Presence,
    pub mono: bool,
    pub night_mode: bool,
    pub mute_in_background: bool,
    pub captions: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings {
            version: SETTINGS_VERSION,
            master: 80, music: 70, ambience: 70, effects: 80, interface: 60,
            presence: Presence::Normal,
            mono: false, night_mode: false, mute_in_background: true, captions: false,
        }
    }
}

/// Round to the nearest step of 5, capped at 100: 150 -> 100, 73 -> 75.
fn snap5(v: u8) -> u8 { (v.min(100) + 2) / 5 * 5 }

impl AudioSettings {
    /// Snap every slider to its 0..=100 step-of-5 grid.
    pub fn clamp(&mut self) {
        self.master = snap5(self.master);
        self.music = snap5(self.music);
        self.ambience = snap5(self.ambience);
        self.effects = snap5(self.effects);
        self.interface = snap5(self.interface);
    }

    /// A slider's volume as a bus gain: the design's square-law taper,
    /// (v/100)^2, so the middle of the track reads as quarter power
    /// rather than half loudness. 0 is exact silence.
    pub fn bus_amp(v: u8) -> f32 {
        let x = v.min(100) as f32 / 100.0;
        x * x
    }

    /// Set one row by name - U3's `set_audio_setting` lands here. A bad
    /// key or a value of the wrong type is an `Err` the caller can show;
    /// a good set snaps the sliders.
    pub fn apply_setting(&mut self, key: &str, value: serde_json::Value) -> Result<(), String> {
        let as_u8 = |v: &serde_json::Value| -> Result<u8, String> {
            v.as_u64().map(|n| n.min(100) as u8)
                .ok_or_else(|| format!("{key}: wants a number 0..=100"))
        };
        let as_bool = |v: &serde_json::Value| -> Result<bool, String> {
            v.as_bool().ok_or_else(|| format!("{key}: wants true or false"))
        };
        match key {
            "master" => self.master = as_u8(&value)?,
            "music" => self.music = as_u8(&value)?,
            "ambience" => self.ambience = as_u8(&value)?,
            "effects" => self.effects = as_u8(&value)?,
            "interface" => self.interface = as_u8(&value)?,
            "presence" => {
                self.presence = match value.as_str() {
                    Some("off") => Presence::Off,
                    Some("sparse") => Presence::Sparse,
                    Some("normal") => Presence::Normal,
                    Some("rich") => Presence::Rich,
                    _ => return Err("presence: wants off, sparse, normal or rich".into()),
                };
            }
            "mono" => self.mono = as_bool(&value)?,
            "night_mode" => self.night_mode = as_bool(&value)?,
            "mute_in_background" => self.mute_in_background = as_bool(&value)?,
            "captions" => self.captions = as_bool(&value)?,
            _ => return Err(format!("{key}: no such setting")),
        }
        self.clamp();
        Ok(())
    }
}
