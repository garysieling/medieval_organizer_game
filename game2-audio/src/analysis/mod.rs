//! The agent's ears (#1491): every later sound ticket proves itself
//! with numbers and images produced here - loudness, true peak, clicks,
//! silence gaps, pitch classes, and the spectrogram / piano-roll /
//! timeline PNGs an agent reads instead of listening.
//!
//! Loudness is BS.1770 written out in Rust rather than the `ebur128`
//! crate: that binding wraps a C library and would break the wasm32
//! target the engine builds for. The math is the spec's - K-weighting
//! filters, 400 ms gated blocks, -0.691 offset.
use std::f32::consts::PI;
use std::io;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------- loudness

pub struct Loudness {
    pub integrated_lufs: f32,
    pub true_peak_dbtp: f32,
}

/// A biquad as (b0, b1, b2, a1, a2) applied direct-form-I per channel.
struct Biquad { b: [f32; 3], a: [f32; 2] }
impl Biquad {
    fn tick(&self, x: f32, st: &mut [f32; 4]) -> f32 {
        let y = self.b[0] * x + self.b[1] * st[0] + self.b[2] * st[1]
            - self.a[0] * st[2] - self.a[1] * st[3];
        st[1] = st[0]; st[0] = x; st[3] = st[2]; st[2] = y;
        y
    }
}

/// BS.1770 stage-1 filter: the high-shelf "pre-filter". Coefficients
/// from the ITU formulas so any sample rate gets the right shape.
fn shelf(sr: f32) -> Biquad {
    let (f0, g, q) = (1681.9744509555319f32, 3.99984385397f32, 0.7071752369554196f32);
    let k = (PI * f0 / sr).tan();
    let vh = 10f32.powf(g / 20.0);
    let vb = vh.powf(0.499666774155_f32);
    let a0 = 1.0 + k / q + k * k;
    Biquad {
        b: [(vh + vb * k / q + k * k) / a0, 2.0 * (k * k - vh) / a0, (vh - vb * k / q + k * k) / a0],
        a: [2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0],
    }
}
/// BS.1770 stage-2 filter: the RLB high-pass.
fn rlb(sr: f32) -> Biquad {
    let (f0, q) = (38.13547087602444f32, 0.5003270373238773f32);
    let k = (PI * f0 / sr).tan();
    let a0 = 1.0 + k / q + k * k;
    Biquad {
        b: [1.0 / a0, -2.0 / a0, 1.0 / a0],
        a: [2.0 * (k * k - 1.0) / a0, (1.0 - k / q + k * k) / a0],
    }
}

/// Integrated loudness (gated) and true peak of interleaved stereo.
pub fn loudness(buf: &[f32], sample_rate: u32) -> Loudness {
    let sr = sample_rate as f32;
    let (pre, hp) = (shelf(sr), rlb(sr));
    let frames = buf.len() / 2;
    let block = (0.400 * sr) as usize;
    let hop = (0.100 * sr) as usize;
    let mut zs = Vec::new();
    let mut start = 0;
    while start + block <= frames {
        let (mut s1, mut s2) = ([0f32; 4], [0f32; 4]);
        let (mut h1, mut h2) = ([0f32; 4], [0f32; 4]);
        let (mut ml, mut mr) = (0.0f32, 0.0f32);
        for f in start..start + block {
            let l = hp.tick(pre.tick(buf[2 * f], &mut s1), &mut h1);
            let r = hp.tick(pre.tick(buf[2 * f + 1], &mut s2), &mut h2);
            ml += l * l; mr += r * r;
        }
        zs.push((ml + mr) / block as f32);
        start += hop;
    }
    let to_lu = |z: f32| if z > 0.0 { -0.691 + 10.0 * z.log10() } else { f32::NEG_INFINITY };
    let lus: Vec<f32> = zs.iter().map(|&z| to_lu(z)).collect();
    // absolute gate at -70 LUFS, then a relative gate 10 LU under the
    // mean of what passed it
    let pass1: Vec<usize> = (0..lus.len()).filter(|&i| lus[i] > -70.0).collect();
    let integrated = if pass1.is_empty() { f32::NEG_INFINITY } else {
        let mean_z = pass1.iter().map(|&i| zs[i]).sum::<f32>() / pass1.len() as f32;
        let rel = to_lu(mean_z) - 10.0;
        let pass2: Vec<usize> = pass1.iter().cloned().filter(|&i| lus[i] > rel).collect();
        if pass2.is_empty() { f32::NEG_INFINITY } else {
            to_lu(pass2.iter().map(|&i| zs[i]).sum::<f32>() / pass2.len() as f32)
        }
    };
    Loudness { integrated_lufs: integrated, true_peak_dbtp: true_peak(buf, sample_rate) }
}

/// True peak: 4x-oversampled maximum, in dBTP. A windowed sinc is
/// exact for band-limited material (a sine's real peak sits where the
/// continuous curve does).
pub fn true_peak(buf: &[f32], sample_rate: u32) -> f32 {
    let _ = sample_rate; // the interpolator is rate-free: it works on sample positions
    let frames = buf.len() / 2;
    let chan = |c: usize, f: isize| -> f32 {
        if f >= 0 && (f as usize) < frames { buf[2 * f as usize + c] } else { 0.0 }
    };
    const TAPS: isize = 8; // each side
    let mut peak = 0.0f32;
    for c in 0..2 {
        for f in 0..frames {
            for q in 0..4 {
                // interpolate at frame f + q/4: x(t) = sum x[n] sinc(t-n)
                // over the samples nearest the point, Hann-windowed for
                // a short kernel
                let t = q as f32 / 4.0;
                let mut acc = 0.0;
                for n in -TAPS..=TAPS {
                    let x = t - n as f32;
                    let w = if x.abs() < 1e-9 { 1.0 }
                        else if x.abs() <= TAPS as f32 {
                            let s = (PI * x).sin() / (PI * x);
                            s * 0.5 * (1.0 + (PI * x / TAPS as f32).cos())
                        } else { 0.0 };
                    acc += chan(c, f as isize + n) * w;
                }
                peak = peak.max(acc.abs());
            }
        }
    }
    if peak > 0.0 { 20.0 * peak.log10() } else { f32::NEG_INFINITY }
}

// ---------------------------------------------------------------- defects

/// Sample-to-sample jumps a listener would hear as a click: a delta
/// over half the local peak and over 0.2 absolute, with hits inside
/// 5 ms merged to one timestamp (seconds).
pub fn clicks(buf: &[f32], sample_rate: u32) -> Vec<f32> {
    let sr = sample_rate as usize;
    let frames = buf.len() / 2;
    let win = sr / 100; // 10 ms either side for the local peak
    let merge = sr / 200; // 5 ms
    let mut out: Vec<f32> = vec![];
    let mut last_hit: Option<usize> = None;
    for f in 1..frames {
        let dl = (buf[2 * f] - buf[2 * f - 2]).abs();
        let dr = (buf[2 * f + 1] - buf[2 * f - 1]).abs();
        let d = dl.max(dr);
        if d <= 0.2 { continue; }
        let (lo, hi) = (f.saturating_sub(win), (f + win).min(frames));
        let local = (lo..hi).fold(0.0f32, |m, i| m.max(buf[2 * i].abs()).max(buf[2 * i + 1].abs()));
        if d <= 0.5 * local { continue; }
        if let Some(p) = last_hit {
            if f - p <= merge { continue; }
        }
        last_hit = Some(f);
        out.push(f as f32 / sample_rate as f32);
    }
    out
}

/// The longest run of 20 ms windows whose peak stays under
/// `floor_dbfs`, in seconds.
pub fn longest_silence(buf: &[f32], sample_rate: u32, floor_dbfs: f32) -> f32 {
    let sr = sample_rate as usize;
    let frames = buf.len() / 2;
    let win = sr / 50; // 20 ms
    let floor = 10f32.powf(floor_dbfs / 20.0);
    let (mut best, mut run) = (0usize, 0usize);
    let mut f = 0;
    while f + win <= frames {
        let peak = (f..f + win).fold(0.0f32, |m, i| m.max(buf[2 * i].abs()).max(buf[2 * i + 1].abs()));
        if peak < floor { run += 1; best = best.max(run); } else { run = 0; }
        f += win;
    }
    best as f32 * win as f32 / sample_rate as f32
}

// ---------------------------------------------------------------- pitch

/// A chromagram: the spectrum's energy folded into the 12 pitch
/// classes over C2..C7 (MIDI 36..96), normalised to sum 1. Index 0 is C.
pub fn pitch_classes(buf: &[f32], sample_rate: u32) -> [f32; 12] {
    let sr = sample_rate as f32;
    let frames = buf.len() / 2;
    let mono: Vec<f32> = (0..frames).map(|f| (buf[2 * f] + buf[2 * f + 1]) * 0.5).collect();
    let mut out = [0.0f32; 12];
    for midi in 36..=96u8 {
        let freq = 440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0);
        // Goertzel at this note's frequency
        let w = 2.0 * PI * freq / sr;
        let coeff = 2.0 * w.cos();
        let (mut s1, mut s2) = (0.0f32, 0.0f32);
        for &x in &mono {
            let s0 = x + coeff * s1 - s2;
            s2 = s1; s1 = s0;
        }
        let power = s1 * s1 + s2 * s2 - coeff * s1 * s2;
        out[(midi % 12) as usize] += power.max(0.0);
    }
    let sum: f32 = out.iter().sum();
    if sum > 0.0 { for v in &mut out { *v /= sum; } }
    out
}

// ---------------------------------------------------------------- metrics

/// The metrics.json keys, exactly. `voices_peak` and `render_ms` are
/// the caller's to fill - they default to 0.
#[derive(Clone, Debug, serde::Serialize)]
pub struct Metrics {
    pub lufs_integrated: f32,
    pub true_peak_dbtp: f32,
    pub clicks: usize,
    pub longest_silence_s: f32,
    pub voices_peak: usize,
    pub render_ms: f32,
}

// ---------------------------------------------------------------- PNGs

fn write_rgb_png(path: &str, w: u32, h: u32, rgb: &[u8]) -> io::Result<()> {
    let f = std::fs::File::create(path)?;
    let wref = &mut std::io::BufWriter::new(f);
    let mut enc = png::Encoder::new(wref, w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    writer.write_image_data(rgb).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// In-place radix-2 FFT on complex pairs packed (re, im). n must be a
/// power of two.
fn fft(v: &mut [(f32, f32)]) {
    let n = v.len();
    let (mut i, mut j) = (1usize, 0usize);
    while i < n {
        let bit = n >> 1;
        while j & bit != 0 { j ^= bit; }
        j |= bit;
        if i < j { v.swap(i, j); }
        i += 1;
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        for i in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (ar, ai) = (v[i + k].0, v[i + k].1);
                let (br, bi) = (v[i + k + len / 2].0 * cr - v[i + k + len / 2].1 * ci,
                                v[i + k + len / 2].0 * ci + v[i + k + len / 2].1 * cr);
                v[i + k] = (ar + br, ai + bi);
                v[i + k + len / 2] = (ar - br, ai - bi);
                let (nr, ni) = (cr * wr - ci * wi, cr * wi + ci * wr);
                cr = nr; ci = ni;
            }
        }
        len *= 2;
    }
}

/// A dB-of-magnitude colour ramp: near-black through blue and amber to
/// white at 0 dB (range -90..0 dB).
fn db_color(db: f32) -> [u8; 3] {
    let t = ((db + 90.0) / 90.0).clamp(0.0, 1.0);
    let ramp = |a: f32, b: f32, c: f32, d: f32| -> (f32, f32, f32) {
        if t < a { (0.0, 0.0, 0.0) }
        else if t < b { let u = (t - a) / (b - a); (u * 90.0, u * 40.0, u * 160.0) }
        else if t < c { let u = (t - b) / (c - b); (90.0 + u * 120.0, 40.0 + u * 140.0, 160.0 - u * 80.0) }
        else { let u = (t - c) / (d - c); (210.0 + u * 45.0, 180.0 + u * 75.0, 80.0 + u * 175.0) }
    };
    let (r, g, b) = ramp(0.05, 0.45, 0.75, 1.0);
    [r as u8, g as u8, b as u8]
}

/// STFT spectrogram to a PNG: 2048 window / 512 hop, log-frequency
/// 30 Hz-16 kHz on the y axis, a -90..0 dB colour map, and tick marks
/// along the bottom every 5 s.
pub fn spectrogram_png(buf: &[f32], sample_rate: u32, path: &str) -> io::Result<()> {
    let sr = sample_rate as usize;
    let frames = buf.len() / 2;
    const WIN: usize = 2048;
    const HOP: usize = 512;
    const H: usize = 400;
    let n_frames = (frames.saturating_sub(WIN)) / HOP;
    let w = n_frames.max(1) as u32;
    let mut rgb = vec![0u8; w as usize * H * 3];
    // Hann window once
    let hann: Vec<f32> = (0..WIN).map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / WIN as f32).cos())).collect();
    // per column: STFT magnitudes, then log-frequency resample
    for col in 0..n_frames {
        let off = col * HOP;
        let mut v: Vec<(f32, f32)> = (0..WIN)
            .map(|i| ((buf[2 * (off + i)] + buf[2 * (off + i) + 1]) * 0.5 * hann[i], 0.0))
            .collect();
        fft(&mut v);
        let mags: Vec<f32> = v[..WIN / 2].iter().map(|(r, i)| (r * r + i * i).sqrt()).collect();
        let norm = 2.0 / WIN as f32;
        // each row's log-frequency span to the next row's, so the pixel
        // reads the loudest bin it covers - nearest-bin aliasing moirés
        let freq_of = |row: usize| 30.0 * (16000.0f32 / 30.0).powf(1.0 - row as f32 / H as f32);
        for row in 0..H {
            let (f_hi, f_lo) = (freq_of(row), freq_of(row + 1));
            let b0 = (f_lo * WIN as f32 / sr as f32).floor() as usize;
            let b1 = ((f_hi * WIN as f32 / sr as f32).ceil() as usize).max(b0 + 1);
            let mag = (b0..b1.min(mags.len())).map(|b| mags[b]).fold(0.0f32, f32::max) * norm;
            let db = if mag > 1e-9 { 20.0 * mag.log10() } else { -120.0 };
            let c = db_color(db.max(-90.0));
            let p = (row * w as usize + col) * 3;
            rgb[p..p + 3].copy_from_slice(&c);
        }
    }
    // a tick every 5 s along the bottom edge
    let tick = (5 * sr / HOP) as usize;
    for col in 0..n_frames {
        if col % tick == 0 {
            for row in (H - 8)..H {
                let p = (row * w as usize + col) * 3;
                rgb[p..p + 3].copy_from_slice(&[200, 200, 200]);
            }
        }
    }
    write_rgb_png(path, w, H as u32, &rgb)
}

// ---------------------------------------------------------------- roll

pub struct RollNote {
    pub start_s: f32,
    pub dur_s: f32,
    pub midi: u8,
    pub voice: u8,
    pub in_mode: bool,
}
pub struct RollInput {
    pub notes: Vec<RollNote>,
    pub mode_pitch_classes: [bool; 12],
}

/// A piano roll: light background rows mark the mode's pitch classes,
/// in-mode notes draw dark, out-of-mode notes draw red.
pub fn piano_roll_png(input: &RollInput, path: &str) -> io::Result<()> {
    const ROWS_LO: u8 = 36; // C2
    const ROWS_HI: u8 = 96; // C7
    const ROW_H: usize = 6;
    const W: usize = 800;
    let rows = (ROWS_HI - ROWS_LO + 1) as usize;
    let h = rows * ROW_H;
    let mut rgb = vec![0u8; W * h * 3];
    let end = input.notes.iter().map(|n| n.start_s + n.dur_s).fold(4.0f32, f32::max);
    // background: mode rows a pale wash, others white (high notes on
    // top, same as the notes below)
    for midi in ROWS_LO..=ROWS_HI {
        let on = input.mode_pitch_classes[(midi % 12) as usize];
        let c = if on { [232, 240, 252] } else { [255, 255, 255] };
        for y in ((ROWS_HI - midi) as usize * ROW_H)..((ROWS_HI - midi + 1) as usize * ROW_H) {
            for x in 0..W {
                let p = (y * W + x) * 3;
                rgb[p..p + 3].copy_from_slice(&c);
            }
        }
    }
    for n in &input.notes {
        if n.midi < ROWS_LO || n.midi > ROWS_HI { continue; }
        let row = (ROWS_HI - n.midi) as usize; // high notes on top
        let (x0, x1) = (((n.start_s / end) * W as f32) as usize,
                        (((n.start_s + n.dur_s) / end) * W as f32) as usize);
        let c = if n.in_mode { [40, 60, 120] } else { [200, 40, 40] };
        for y in (row * ROW_H + 1)..((row + 1) * ROW_H).min(h) {
            for x in x0..x1.min(W) {
                let p = (y * W + x) * 3;
                rgb[p..p + 3].copy_from_slice(&c);
            }
        }
    }
    write_rgb_png(path, W as u32, h as u32, &rgb)
}

// ---------------------------------------------------------------- timeline

pub struct Section { pub start_s: f32, pub end_s: f32, pub label: String }
pub struct Lane { pub name: String, pub density: Vec<f32> }
pub struct Marker { pub t_s: f32, pub label: String }
pub struct TimelineInput {
    /// One tension value per second.
    pub tension: Vec<f32>,
    pub sections: Vec<Section>,
    /// Mode changes: (start_s, label).
    pub mode_lane: Vec<(f32, String)>,
    /// One layer per lane; density per bar in 0..1.
    pub layers: Vec<Lane>,
    pub markers: Vec<Marker>,
}

/// The score's shape at a glance: tension curve over section bands, a
/// mode strip, per-layer density lanes and event markers. Plain data
/// in - the composer and director don't exist yet (S2 stands alone).
pub fn timeline_png(input: &TimelineInput, path: &str) -> io::Result<()> {
    const W: usize = 800;
    const H: usize = 360;
    const TENS_H: usize = 180;
    const LANE_H: usize = 18;
    let mut rgb = vec![255u8; W * H * 3];
    let span = input.tension.len().max(1) as f32;
    let x_of = |t: f32| ((t / span) * (W - 1) as f32) as usize;
    let put = |rgb: &mut [u8], x: usize, y: usize, c: [u8; 3]| {
        if x < W && y < H { let p = (y * W + x) * 3; rgb[p..p + 3].copy_from_slice(&c); }
    };
    // section bands behind the tension curve; the label hashes to a hue
    for s in &input.sections {
        let hue = s.label.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32)) % 3;
        let c = [[235, 228, 240], [228, 240, 228], [240, 236, 224]][hue as usize];
        for y in 0..TENS_H { for x in x_of(s.start_s)..x_of(s.end_s).min(W) { put(&mut rgb, x, y, c); } }
    }
    // the tension curve, black
    let ty = |t: f32| TENS_H - 1 - (t.clamp(0.0, 1.0) * (TENS_H - 20) as f32) as usize;
    for i in 0..input.tension.len().saturating_sub(1) {
        let (x0, x1) = (x_of(i as f32), x_of(i as f32 + 1.0));
        let (y0, y1) = (ty(input.tension[i]), ty(input.tension[i + 1]));
        for x in x0..=x1 {
            let u = if x1 > x0 { (x - x0) as f32 / (x1 - x0) as f32 } else { 0.0 };
            let y = y0 as f32 + (y1 as f32 - y0 as f32) * u;
            put(&mut rgb, x, y as usize, [20, 20, 20]);
        }
    }
    // the mode strip: each change starts a coloured span
    let mut y = TENS_H + 8;
    for (i, (t, label)) in input.mode_lane.iter().enumerate() {
        let hue = label.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32)) % 3;
        let c = [[120, 90, 160], [90, 140, 110], [160, 130, 80]][hue as usize];
        let next = input.mode_lane.get(i + 1).map(|(t, _)| *t).unwrap_or(span);
        for yy in y..y + LANE_H { for x in x_of(*t)..x_of(next).min(W) { put(&mut rgb, x, yy, c); } }
    }
    y += LANE_H + 8;
    // layer lanes: darkness reads density per bar
    for l in &input.layers {
        let bars = l.density.len().max(1) as f32;
        for x in 0..W {
            let d = l.density[((x as f32 / W as f32) * bars) as usize].clamp(0.0, 1.0);
            let g = (230.0 - 170.0 * d) as u8;
            for yy in y..y + LANE_H - 4 { put(&mut rgb, x, yy, [g, g, (g as f32 * 0.9) as u8]); }
        }
        y += LANE_H;
    }
    // event markers: a dark red hairline top to bottom
    for m in &input.markers {
        let x = x_of(m.t_s);
        for yy in 0..H { put(&mut rgb, x, yy, [150, 30, 30]); }
    }
    write_rgb_png(path, W as u32, H as u32, &rgb)
}
