//! The repair move (#1251): a wrong word owned. From social-skills
//! training literature (PEERS, Skillstreaming) - a misstep isn't the
//! end, naming it is a learnable move. Both the player's Talk and a
//! passing chat between neighbours roll the same table: who the answer
//! comes from decides whether it lands.

use crate::game::traits::Trait;

/// The answer a wrong word owned gets (#1251): a Forgiving one takes it
/// warm, a Proud one takes it but grudges it, a Grievous one may not
/// take it at all
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Answer {
    /// Accepted warm - "water under the bridge"
    Warm,
    /// Accepted grudgingly - "I'll hear it; mind your tongue"
    Grudging,
    /// Refused - the chill stands
    Refused,
}

/// The chance they refuse the amends outright (#1251): only a
/// grudge-holding temperament refuses at all - a Cruel soul half the
/// time, a hot head sometimes; a proud one never refuses, they just
/// let you know what it cost
pub fn refusal_chance(traits: &[Trait]) -> f32 {
    if traits.contains(&Trait::Cruel) { 0.5 }
    else if traits.contains(&Trait::Hotheaded) { 0.2 }
    else { 0.0 }
}

/// `answer` once the refusal roll's behind them: warm for the kind,
/// grudging for the proud, plain for the rest
pub fn their_answer(traits: &[Trait], refused: bool) -> Answer {
    if refused { return Answer::Refused; }
    if traits.contains(&Trait::Kind) { Answer::Warm }
    else if traits.contains(&Trait::Proud) { Answer::Grudging }
    else { Answer::Warm }
}

/// The line the answer carries
pub fn answer_line(a: Answer) -> &'static str {
    match a {
        Answer::Warm => "Water under the bridge - think no more of it.",
        Answer::Grudging => "I'll hear it. Mind your tongue, mind.",
        Answer::Refused => "You've a nerve, coming with that.",
    }
}

/// The ask itself, whoever's sorry
pub const AMENDS_ASK: &str = "About the other day - I spoke out of turn.";

/// The warmth an accepted amends puts back, `n` already made to the
/// same one (#1251): the first lands near a third of the chill, and
/// repeated sorries stop landing - the fourth's a coin for the plate
pub fn warmth_back(prior: u8) -> f32 {
    0.3 * 0.5f32.powi(prior.min(4) as i32)
}

/// The chance a villager offers the word in a passing chat (#1251):
/// the kindly and the sociable extend the hand more, a cruel one
/// almost never does
pub fn attempt_chance(traits: &[Trait]) -> f32 {
    let mut c: f32 = 0.30;
    if traits.iter().any(|t| matches!(t, Trait::Kind | Trait::Gregarious)) { c += 0.15; }
    if traits.contains(&Trait::Cruel) { c -= 0.15; }
    c.clamp(0.05, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_answer_is_their_temper() {
        // the kind take it warm, the proud take it but let you know,
        // and only a grudge-holding bent refuses at all
        assert_eq!(refusal_chance(&[Trait::Kind]), 0.0);
        assert_eq!(refusal_chance(&[Trait::Proud]), 0.0);
        assert_eq!(refusal_chance(&[]), 0.0);
        assert!(refusal_chance(&[Trait::Cruel]) > 0.0);
        assert!(refusal_chance(&[Trait::Hotheaded]) > 0.0);
        assert!(refusal_chance(&[Trait::Cruel]) > refusal_chance(&[Trait::Hotheaded]));
        assert_eq!(their_answer(&[Trait::Kind], false), Answer::Warm);
        assert_eq!(their_answer(&[Trait::Proud], false), Answer::Grudging);
        assert_eq!(their_answer(&[Trait::Cruel], true), Answer::Refused);
        assert_eq!(their_answer(&[Trait::Cruel], false), Answer::Warm);
    }

    #[test]
    fn test_sorries_stop_landing() {
        let (w0, w1, w4) = (warmth_back(0), warmth_back(1), warmth_back(4));
        assert!(w0 > w1 && w1 > warmth_back(3), "each sorry lands less");
        assert_eq!(warmth_back(4), warmth_back(5), "the decay bottoms out");
        assert!(w4 > 0.0, "it's never nothing - it just stops mattering");
    }
}
