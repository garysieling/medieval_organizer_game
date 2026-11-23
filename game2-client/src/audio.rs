//! The desktop output bridge (#1494): `CpalSink` opens the default
//! output device and runs `LivePump` in its callback; `open_host` is
//! `game_ui`'s startup path - `--no-audio` or a failed device means a
//! silent host, never a crash. Everything device-shaped stays behind
//! the `audio` feature so `--no-default-features` builds silent.
use game2_audio::host::{AudioHost, AudioSink, HostSettings};

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
