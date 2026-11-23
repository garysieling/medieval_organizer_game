//! WAV output for offline renders and recordings.

/// The rate recordings are written at. Offline renders in the engine
/// take whatever rate the caller gives; this is the container's rate.
pub const SAMPLE_RATE: u32 = 44100;

/// Write `pcm` (interleaved, `channels` wide, 16-bit) as a PCM WAV.
/// Moved out of `game2-client/src/record.rs` (#1486) so the audio crate
/// owns the container; the signature is unchanged.
pub fn write_wav(path: &str, pcm: &[i16], channels: u16) -> std::io::Result<()> {
    let data = (pcm.len() * 2) as u32;
    let byte_rate = SAMPLE_RATE * channels as u32 * 2;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF"); out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt "); out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes()); // block align
    out.extend_from_slice(&16u16.to_le_bytes()); // bits
    out.extend_from_slice(b"data"); out.extend_from_slice(&data.to_le_bytes());
    for s in pcm { out.extend_from_slice(&s.to_le_bytes()); }
    std::fs::write(path, out)
}
