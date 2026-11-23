//! The desktop output bridge (#1494): `CpalSink` opens the default
//! output device and runs `LivePump` in its callback; `open_host` is
//! `game_ui`'s startup path - `--no-audio` or a failed device means a
//! silent host, never a crash. Everything device-shaped stays behind
//! the `audio` feature so `--no-default-features` builds silent.
use game2_audio::host::{AudioHost, AudioSink, HostSettings};
use game2_audio::settings::AudioSettings;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// `game_ui`'s host: a real device when one's open, a silent host for
/// `--no-audio` or a failed open. The sink owns the stream, so the
/// caller keeps it alive (drop it and playback stops).
pub fn open_host(no_audio: bool, settings: &HostSettings,
                 sink: Option<&mut dyn AudioSink>) -> AudioHost {
    if no_audio { return AudioHost::silent(); }
    match sink {
        Some(s) => AudioHost::start(settings, s),
        None => AudioHost::silent(),
    }
}

/// How long after the last change the settings file is written - a
/// slider drag saves once at the end, not on every step.
const DEBOUNCE: Duration = Duration::from_millis(1000);

/// The desktop's persisted `AudioSettings` (#1505):
/// `game2.settings.json` in the working directory, next to
/// `game2.claim`. Changes are written a second after the last one and
/// flushed on quit; a corrupt file is one warning and defaults, not a
/// crash - the next write puts a clean copy back.
pub struct SettingsStore {
    pub settings: AudioSettings,
    path: PathBuf,
    /// When the last change happened; `None` means nothing to write
    changed: Option<Instant>,
}

impl SettingsStore {
    /// Load `path`, or defaults when it doesn't exist or won't parse.
    pub fn load(path: &Path) -> Self {
        let settings = match std::fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("audio: ignoring corrupt {} ({e}) - defaults", path.display());
                    AudioSettings::default()
                }
            },
            Err(_) => AudioSettings::default(),
        };
        SettingsStore { settings, path: path.to_path_buf(), changed: None }
    }

    /// A setting changed: the write lands a second out, and a later
    /// change restarts the clock.
    pub fn touch(&mut self, now: Instant) { self.changed = Some(now); }

    /// Write if the debounce has run out; the caller passes the clock
    /// so tests can inject one. Returns true when a write happened.
    pub fn due(&mut self, now: Instant) -> bool {
        if self.changed.is_some_and(|t| now.duration_since(t) >= DEBOUNCE) {
            self.changed = None;
            self.write();
            true
        } else { false }
    }

    /// On quit: the current settings always land, so the file exists
    /// (and a corrupt one gets rewritten) even when nothing changed.
    pub fn flush(&mut self) {
        self.changed = None;
        self.write();
    }

    fn write(&self) {
        if let Ok(text) = serde_json::to_string_pretty(&self.settings) {
            let _ = std::fs::write(&self.path, text);
        }
    }
}

#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
mod real {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::io;

    /// The default output device over cpal, ~`settings.buffer_ms` deep.
    /// `probe` fails cleanly (no device, odd config) so the caller can
    /// go silent rather than crash.
    pub struct CpalSink {
        device: cpal::Device,
        config: cpal::StreamConfig,
        /// The live stream - the device's heartbeat. Dropped, it stops.
        stream: Option<cpal::Stream>,
    }

    impl CpalSink {
        pub fn probe() -> Result<CpalSink, io::Error> {
            let dev = cpal::default_host().default_output_device()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no output device"))?;
            let supported = dev.default_output_config().map_err(io::Error::other)?;
            if supported.sample_format() != cpal::SampleFormat::F32 {
                return Err(io::Error::new(io::ErrorKind::Other,
                    format!("device speaks {:?}, the engine renders f32", supported.sample_format())));
            }
            // ~100 ms of runway so a slow frame never starves the callback
            let mut config = supported.config();
            config.buffer_size = cpal::BufferSize::Fixed(
                (config.sample_rate.0 as f32 * config.channels as f32 * 0.100) as u32);
            Ok(CpalSink { device: dev, config, stream: None })
        }

    }

    impl AudioSink for CpalSink {
        /// The rate the device actually runs at - the host core must
        /// render at it or the pitch goes sideways.
        fn sample_rate(&self) -> u32 { self.config.sample_rate.0 }

        fn open(&mut self, mut pump: Box<dyn FnMut(&mut [f32]) + Send>) -> Result<(), io::Error> {
            let err = |e| eprintln!("audio: stream error: {e}");
            let stream = self.device
                .build_output_stream(&self.config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| pump(data),
                    err, None)
                .map_err(io::Error::other)?;
            stream.play().map_err(io::Error::other)?;
            self.stream = Some(stream);
            Ok(())
        }
    }
}

#[cfg(all(feature = "audio", not(target_arch = "wasm32")))]
pub use real::CpalSink;
