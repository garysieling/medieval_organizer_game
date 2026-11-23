//! Village mysteries: the gossip a seat can piece together a clue at a
//! time. Each is a question somebody in the village wants answered and
//! a small story told in tagged clues - who did it, what was done, when,
//! where, and why - that folk pass on in talk; hear enough of the right
//! directions and the truth's yours to carry back to whoever asked.

use crate::game::vocation::Vocation;

/// The five directions a clue can point (#1350)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dim { Who, What, When, Where, Why }

/// All five, in the order the panel reads them
pub const DIMS: [Dim; 5] = [Dim::Who, Dim::What, Dim::When, Dim::Where, Dim::Why];

impl Dim {
    /// The panel's word for it
    pub fn word(&self) -> &'static str {
        match self {
            Dim::Who => "who", Dim::What => "what", Dim::When => "when",
            Dim::Where => "where", Dim::Why => "why",
        }
    }
}

/// A fact of the story, tagged with the question it answers
#[derive(Debug)]
pub struct Clue {
    pub dim: Dim,
    pub text: &'static str,
}

/// How hard a mystery runs (#1350): which band of the pool a seat draws
/// it from - the early asks are short, the late ones run deep
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Band { Easy, Medium, Hard }

/// One mystery doing the rounds: the question, the tagged gossip, what
/// it takes to solve, and the payoff when it's told to whoever wanted
/// to know
#[derive(Debug)]
pub struct Mystery {
    /// What the asker wonders, in a line ("who's been leaving gifts on doorsteps")
    pub title: &'static str,
    /// The gossip pieces, one learned per telling - more than `need`, so
    /// there's always a draw left
    pub clues: &'static [Clue],
    /// Distinct clues that must be heard to call it solved
    pub need: usize,
    /// The directions the hearing must cover, at least: (dim, how many
    /// of its clues). A hard story can ask two of one - you know it was
    /// a man, then which man
    pub dims: &'static [(Dim, usize)],
    /// The trade most likely to be asking, when one fits - else any folk
    pub asker: Option<Vocation>,
    /// What they say when the truth's told
    pub wonders: &'static str,
    /// And their thanks, once it's delivered
    pub thanks: &'static str,
    /// What the telling's worth to the seat, beyond the friendship
    pub payoff: Payoff,
}

impl Mystery {
    /// Which band of the pool it sits in, off the ask's size - a short
    /// ask's an early story; the long ones come late
    pub fn band(&self) -> Band {
        if self.need <= 3 { Band::Easy }
        else if self.need <= 5 { Band::Medium }
        else { Band::Hard }
    }
}

/// What a told mystery gives back: every telling makes a friend of the
/// asker and opens them up; this is the rest of it, the asker's own
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Payoff {
    /// They press coin on you for the trouble
    Gift(f32),
    /// They ask you to their table
    Table,
    /// They've a job that follows from it - the next thing to run down
    Lead,
    /// They tell it about: every grown soul of the village warms to you
    Word,
}

/// The mysteries written for the village. A seat meets one at a time,
/// drawn off this table - the early rounds pull the easy stories, and
/// the deeper ones come round as solves rack up
pub const MYSTERIES: &[Mystery] = crate::game::mystery_stories::STORIES;
