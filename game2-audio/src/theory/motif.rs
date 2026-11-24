//! Authored motifs and the pure transforms the composer applies to
//! them. A motif is scale degrees (so it transposes for free when the
//! mode moves) plus a rhythm in sixteenth notes. The composer states a
//! motif then varies it - it never invents a main theme.

use super::Degree;

/// A melodic figure: `degrees[i]` is the scale degree (1-based, as
/// `Degree`) of the i-th note and `rhythm[i]` its length in 16ths.
/// The vecs stay the same length; a rhythm of 0 is invalid input and
/// callers compose none.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Motif {
    pub degrees: Vec<i8>,
    /// Note lengths in sixteenth notes, aligned 1:1 with `degrees`.
    pub rhythm: Vec<u8>,
}

impl Motif {
    pub fn new(degrees: &[i8], rhythm: &[u8]) -> Motif {
        debug_assert_eq!(degrees.len(), rhythm.len());
        Motif {
            degrees: degrees.to_vec(),
            rhythm: rhythm.to_vec(),
        }
    }

    /// Shift every note by `steps` scale degrees (diatonic - the mode
    /// supplies the accidentals when it's realised).
    pub fn transpose(&self, steps: i8) -> Motif {
        Motif {
            degrees: self.degrees.iter().map(|d| d + steps).collect(),
            rhythm: self.rhythm.clone(),
        }
    }

    /// A sequence up the scale: identical to `transpose` - named for
    /// the build's "motif sequenced up a step each phrase".
    pub fn sequence_up(&self, steps: i8) -> Motif {
        self.transpose(steps)
    }

    /// Melodic inversion about the first note: every interval flips
    /// direction. Applied twice it returns the original.
    pub fn invert(&self) -> Motif {
        let d0 = self.degrees.first().copied().unwrap_or(1);
        Motif {
            degrees: self.degrees.iter().map(|d| 2 * d0 - d).collect(),
            rhythm: self.rhythm.clone(),
        }
    }

    /// Rhythmic augmentation: every note length times `factor`.
    pub fn augment(&self, factor: u8) -> Motif {
        Motif {
            degrees: self.degrees.clone(),
            rhythm: self.rhythm.iter().map(|r| r * factor).collect(),
        }
    }

    /// The first `n` notes - how a fragment of the cause's cell stands
    /// in when the arc is unresolved.
    pub fn fragment(&self, n: usize) -> Motif {
        Motif {
            degrees: self.degrees.iter().take(n).copied().collect(),
            rhythm: self.rhythm.iter().take(n).copied().collect(),
        }
    }

    /// Realise the motif as midi notes over a tonic and mode.
    pub fn notes(&self, tonic: u8, mode: super::Mode) -> Vec<u8> {
        self.degrees
            .iter()
            .map(|&d| super::midi(tonic, mode, Degree(d)))
            .collect()
    }
}

/// Bjorklund's algorithm: `k` hits spread as evenly as possible over
/// `n` steps, as `true`/`false` pulses. The composer uses these for the
/// Pulse voice (E(3,8), E(5,8) by density). Rotation is canonical -
/// `euclid(3,8)` is `x..x..x.` and `euclid(5,8)` is `x.xx.xx.` - so the
/// same seed always draws the same pattern.
pub fn euclid(k: usize, n: usize) -> Vec<bool> {
    if k == 0 {
        return vec![false; n];
    }
    if k >= n {
        return vec![true; n];
    }
    // Toussaint's formulation of Bjorklund's algorithm: count the
    // distribution rounds, then rebuild the pattern recursively.
    let mut counts = Vec::new();
    let mut remainders = vec![k];
    let mut divisor = n - k;
    loop {
        let level = remainders.len() - 1;
        counts.push(divisor / remainders[level]);
        remainders.push(divisor % remainders[level]);
        divisor = remainders[level];
        if *remainders.last().unwrap() <= 1 {
            break;
        }
    }
    counts.push(divisor);
    let mut pattern = Vec::new();
    fn build(counts: &[usize], remainders: &[usize], level: i32, out: &mut Vec<bool>) {
        if level == -1 {
            out.push(false);
        } else if level == -2 {
            out.push(true);
        } else {
            let lvl = level as usize;
            for _ in 0..counts[lvl] {
                build(counts, remainders, level - 1, out);
            }
            if remainders[lvl] != 0 {
                build(counts, remainders, level - 2, out);
            }
        }
    }
    build(&counts, &remainders, counts.len() as i32 - 1, &mut pattern);
    let start = pattern.iter().position(|&b| b).unwrap_or(0);
    pattern.rotate_left(start);
    pattern
}
