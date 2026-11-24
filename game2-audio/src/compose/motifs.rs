//! The motif library (M4): eight hand-authored leitmotifs, one per
//! thing the village knows how to sing about. Each is a `Motif` -
//! scale degrees plus a rhythm in sixteenths - so it transposes and
//! inverts for free when the mode or the treatment moves. The
//! composer states a motif, then varies it; it never invents a main
//! theme (the design's leitmotif table).

use crate::instruments::PatchId;
use crate::theory::Motif;

/// Which leitmotif a phrase plays.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MotifId {
    /// Stepwise, pastoral: the village's own tune, 8 notes.
    Village,
    /// The cause: a 4-note cell, rising 4th then rising 5th, then 7.
    Cause,
    /// The lord: Phrygian-flavoured, dotted, heavy.
    Lord,
    /// The church: an organum cell, open sonorities.
    Church,
    /// The old faith: a Mixolydian circling figure.
    OldFaith,
    /// The road: a dance tune for visitors and carts.
    Road,
    /// Grief: Aeolian, falling, slow.
    Grief,
    /// The feast: a bright dance that wants 3/4 or 6/8.
    Feast,
}

impl MotifId {
    pub const ALL: [MotifId; 8] = [
        MotifId::Village,
        MotifId::Cause,
        MotifId::Lord,
        MotifId::Church,
        MotifId::OldFaith,
        MotifId::Road,
        MotifId::Grief,
        MotifId::Feast,
    ];

    /// The authored figure, as scale degrees + sixteenths.
    pub fn motif(self) -> Motif {
        match self {
            // 8 notes, mostly stepwise, arching up and easing home.
            MotifId::Village => Motif::new(&[1, 2, 3, 4, 5, 4, 3, 2],
                                          &[2, 2, 2, 2, 2, 2, 2, 2]),
            // 1->4 is a rising 4th, 4->8 a rising 5th, then the 7
            // that hangs just under the octave: a question asked.
            MotifId::Cause => Motif::new(&[1, 4, 8, 7],
                                        &[4, 4, 4, 4]),
            // Heavy and dotted: a long tread, the flat 2's scowl; the
            // 5th-leap steps back down (a leap resolves by a step).
            MotifId::Lord => Motif::new(&[1, 2, 1, 5, 4],
                                       &[6, 2, 4, 4, 8]),
            // Open fifths walking: organum's parallel frame.
            MotifId::Church => Motif::new(&[1, 5, 4, 5, 1],
                                         &[4, 4, 4, 4, 8]),
            // Circling the tonic through the flat 7 BELOW it (degree 0),
            // like a ring dance - never more than a fifth between notes.
            MotifId::OldFaith => Motif::new(&[1, 0, 2, 1, 5, 4, 2, 1],
                                           &[2, 2, 2, 4, 2, 2, 2, 4]),
            // A skipping dance: triadic hops over a quick step.
            MotifId::Road => Motif::new(&[1, 3, 5, 4, 3, 2, 1],
                                       &[2, 2, 2, 2, 2, 2, 4]),
            // Falls a full octave, slow: the grief motif descends.
            MotifId::Grief => Motif::new(&[8, 6, 5, 4, 3, 2, 1],
                                        &[4, 4, 4, 4, 4, 4, 8]),
            // Bright triple-time fling: up the triad and back down.
            MotifId::Feast => Motif::new(&[1, 3, 5, 8, 7, 5, 3, 1],
                                        &[2, 2, 2, 2, 2, 2, 2, 2]),
        }
    }

    /// The patch that leads the motif's statements (the leitmotif
    /// table's instrument).
    pub fn lead_patch(self) -> PatchId {
        match self {
            MotifId::Village => PatchId::RecorderAlto,
            MotifId::Cause => PatchId::Fife,
            MotifId::Lord => PatchId::Shawm,
            MotifId::Church => PatchId::Organ,
            MotifId::OldFaith => PatchId::Psaltery,
            MotifId::Road => PatchId::HurdyGurdy,
            MotifId::Grief => PatchId::Vielle,
            MotifId::Feast => PatchId::BagpipeChanter,
        }
    }
}
