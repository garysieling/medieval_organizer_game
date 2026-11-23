//! The sampler (A2/#1498): decodes committed OGG (lewton) or 16-bit
//! WAV bytes into a `SampleBank` of mono PCM, organises samples into
//! `Patch`es of key `Zone`s, and hands the engine per-voice playback.
//! The desktop `load_dir` reads `assets/sound/**/*.ogg`; the web
//! inserts fetched bytes via `insert_bytes`. Samples every 4th
//! semitone cover the keyboard - a zone never shifts past ±3.
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

/// Index of a decoded buffer inside a `SampleBank`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SampleId(pub u32);

/// Index of a patch inside a `SampleBank` - what `Source::Sample` and
/// `Source::Bed` name (a `Copy` handle, so voices stay `Copy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatchId(pub u32);

/// One decoded buffer: mono f32 PCM at `sr`.
#[derive(Clone, Debug)]
pub struct Sample {
    pub pcm: Vec<f32>,
    pub sr: u32,
}

/// A key range of a patch: notes `key_lo..=key_hi` play `sample`
/// pitch-shifted from `root` by `2^((midi-root)/12)`; `loop_start`/
/// `loop_end` give a sustain loop in sample frames when present.
#[derive(Clone, Debug)]
pub struct Zone {
    pub key_lo: u8,
    pub key_hi: u8,
    pub root: u8,
    pub loop_start: Option<u32>,
    pub loop_end: Option<u32>,
    pub sample: SampleId,
}

/// A playable instrument: a set of zones the sampler picks between.
#[derive(Clone, Debug)]
pub struct Patch {
    pub zones: Vec<Zone>,
}

impl Patch {
    /// The zone for a note: zones covering `midi`, nearest root wins.
    /// `None` when no zone's range covers the note.
    pub fn zone_for(&self, midi: u8) -> Option<&Zone> {
        self.zones.iter()
            .filter(|z| z.key_lo <= midi && midi <= z.key_hi)
            .min_by_key(|z| (z.root as i16 - midi as i16).unsigned_abs())
    }
}

/// What went wrong decoding a file.
#[derive(Debug)]
pub enum DecodeError {
    /// Not OGG and not RIFF/WAVE
    UnknownFormat,
    /// OGG that lewton rejected
    Ogg(String),
    /// WAV that isn't plain PCM
    Wav(String),
}

/// Every decoded buffer plus the patches naming them.
#[derive(Default)]
pub struct SampleBank {
    pub samples: Vec<Sample>,
    /// `(name, patch)` - the name is the ledger id / file stem callers
    /// use to find it again (`patch_id`).
    pub patches: Vec<(String, Patch)>,
}

impl SampleBank {
    pub fn new() -> Self { Self::default() }

    /// A bank holding one single-zone patch from `bytes` (OGG or WAV),
    /// covering the whole keyboard, rooted at middle C (60).
    pub fn from_bytes(id: &str, bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut bank = SampleBank::new();
        bank.insert_bytes(id, bytes)?;
        Ok(bank)
    }

    /// Decode `bytes` (OGG or 16-bit WAV) and add a single-zone patch
    /// named `id`. The web loader calls this once per manifest entry.
    pub fn insert_bytes(&mut self, id: &str, bytes: &[u8]) -> Result<PatchId, DecodeError> {
        let sample = decode(bytes)?;
        let sample_id = SampleId(self.samples.len() as u32);
        self.samples.push(sample);
        let patch = Patch { zones: vec![Zone {
            key_lo: 0, key_hi: 127, root: 60,
            loop_start: None, loop_end: None, sample: sample_id,
        }] };
        let pid = PatchId(self.patches.len() as u32);
        self.patches.push((id.to_string(), patch));
        Ok(pid)
    }

    pub fn patch_id(&self, name: &str) -> Option<PatchId> {
        self.patches.iter().position(|(n, _)| n == name).map(|i| PatchId(i as u32))
    }

    pub fn patch(&self, id: PatchId) -> Option<&Patch> {
        self.patches.get(id.0 as usize).map(|(_, p)| p)
    }

    pub fn sample(&self, id: SampleId) -> Option<&Sample> {
        self.samples.get(id.0 as usize)
    }
}

/// Decode either container to mono f32.
pub fn decode(bytes: &[u8]) -> Result<Sample, DecodeError> {
    if bytes.len() >= 4 && &bytes[..4] == b"OggS" { return decode_ogg(bytes); }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        return decode_wav(bytes);
    }
    Err(DecodeError::UnknownFormat)
}

fn decode_ogg(bytes: &[u8]) -> Result<Sample, DecodeError> {
    let mut rdr = lewton::inside_ogg::OggStreamReader::new(std::io::Cursor::new(bytes))
        .map_err(|e| DecodeError::Ogg(format!("{e:?}")))?;
    let sr = rdr.ident_hdr.audio_sample_rate;
    let mut pcm = vec![];
    while let Some(pck) = rdr.read_dec_packet().map_err(|e| DecodeError::Ogg(format!("{e:?}")))? {
        // fold every channel to mono: point sources are mono anyway
        let n = pck.len();
        if n == 0 { continue; }
        let frames = pck[0].len();
        for i in 0..frames {
            let mut acc = 0f32;
            for ch in &pck { acc += ch[i] as f32 / 32768.0; }
            pcm.push(acc / n as f32);
        }
    }
    Ok(Sample { pcm, sr })
}

fn decode_wav(bytes: &[u8]) -> Result<Sample, DecodeError> {
    // walk the RIFF chunks for fmt + data; plain 16-bit PCM only
    let mut at = 12;
    let (mut channels, mut sr, mut bits) = (0u16, 0u32, 0u16);
    let mut data: Option<&[u8]> = None;
    while at + 8 <= bytes.len() {
        let tag = &bytes[at..at + 4];
        let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
        let body = &bytes[at + 8..(at + 8 + len).min(bytes.len())];
        match tag {
            b"fmt " => {
                if body.len() < 16 { return Err(DecodeError::Wav("short fmt".into())); }
                if u16::from_le_bytes(body[0..2].try_into().unwrap()) != 1 {
                    return Err(DecodeError::Wav("not PCM".into()));
                }
                channels = u16::from_le_bytes(body[2..4].try_into().unwrap());
                sr = u32::from_le_bytes(body[4..8].try_into().unwrap());
                bits = u16::from_le_bytes(body[14..16].try_into().unwrap());
            }
            b"data" => data = Some(body),
            _ => {}
        }
        at += 8 + len + (len & 1); // RIFF chunks are word-aligned
    }
    if bits != 16 || channels == 0 {
        return Err(DecodeError::Wav(format!("{bits}-bit x {channels} ch")));
    }
    let data = data.ok_or_else(|| DecodeError::Wav("no data chunk".into()))?;
    let ch = channels as usize;
    let mut pcm = Vec::with_capacity(data.len() / 2 / ch);
    for frame in data.chunks_exact(ch * 2) {
        let mut acc = 0f32;
        for c in 0..ch {
            let s = i16::from_le_bytes(frame[c * 2..c * 2 + 2].try_into().unwrap());
            acc += s as f32 / 32768.0;
        }
        pcm.push(acc / ch as f32);
    }
    Ok(Sample { pcm, sr })
}

/// Desktop loader: every OGG/WAV under `root` becomes a single-zone
/// patch named by its file stem (`beds/river.ogg` -> `river`). The web
/// build can't read a filesystem - it fetches and calls
/// `insert_bytes` instead.
#[cfg(not(target_arch = "wasm32"))]
pub fn load_dir(root: &Path) -> SampleBank {
    let mut bank = SampleBank::new();
    let mut files = vec![];
    collect(root, &mut files);
    files.sort();
    for f in files {
        let Some(stem) = f.file_stem().and_then(|s| s.to_str()).map(str::to_string) else { continue };
        let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "ogg" && ext != "wav" { continue; }
        if let Ok(bytes) = std::fs::read(&f) {
            let _ = bank.insert_bytes(&stem, &bytes); // undecodable: skipped, the gate owns loudness
        }
    }
    bank
}

#[cfg(not(target_arch = "wasm32"))]
fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            // staging holds raw fetches, not game assets
            if p.file_name().is_some_and(|n| n == "_incoming") { continue; }
            collect(&p, out);
        } else { out.push(p); }
    }
}
