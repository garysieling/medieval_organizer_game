//! Idiolect (E4-7, #1416): the ear's landmark. Every villager carries a
//! pet phrase and one structural habit - a proverb bent, an oath, a
//! trick of answering - each unique in the village, each used at most
//! once a talk. Proverbs carry values cheaply: the cautious counsel
//! caution, the firebrand mocks it.
//!
//! The mechanics ride the transcript, not the pick: when the talk
//! settles, each speaker's habit fires at the first slot its context
//! fits (and the pet phrase tags a greeting), so a replayed player talk
//! keeps its prefix - the same line lands in the same place every time.

use crate::game::conversation::{self, Act, Speaker, Transcript};

/// One villager's speech habits: a pet phrase and a structural habit
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Idiolect {
    /// Dropped once a talk into a greeting or parting: " - as the mill
    /// grinds."
    pub phrase: &'static str,
    /// The structural habit - proverbs, an oath, answering a question
    /// with a question - unique in the village
    pub habit: Habit,
}

/// How a habit's line meets the turn it fires on
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum How {
    /// The variant stands in for the line - a proverb IS the answer
    Whole,
    /// It opens the line: "Mark me - it won't hold."
    Preface,
    /// It closes it: "...for my sins."
    Coda,
}

/// The structural speech habits - one per villager, no two shared in a
/// village. Each owns a small pool of whole-line variants; the cast
/// deal (`cast_check::deal_idiolects`) prefers habits whose values the
/// persona's top value speaks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Habit {
    /// Answers a question with a question
    QuestionBack,
    /// Proverbs counseling caution (Safety)
    ProverbCaution,
    /// Proverbs mocking too-careful folk (Freedom)
    ProverbMockery,
    /// Proverbs of patience and seasons (Custom)
    ProverbPatience,
    /// Proverbs of honest work (Advancement)
    ProverbWork,
    /// Swears by the saints
    OathSaints,
    /// Swears by the harvest
    OathHarvest,
    /// Swears by the mill
    OathMill,
    /// Mutters the answer under the breath
    UnderBreath,
    /// Names themselves instead of "I"
    ThirdPerson,
    /// Says the thing back before answering it
    Echo,
    /// Counts the points off like a tally
    CountsOff,
    /// Caps a line with a rhyme
    RhymingCoda,
    /// Dates everything by saints' days
    SaintsDay,
    /// Quotes a father's sayings
    FatherSaid,
    /// Quotes a mother's sayings
    MotherSaid,
    /// Hums a note ahead of the greeting
    Hums,
    /// Opens with a blessing on the hearer
    BlessesHearer,
    /// Reads omens into everyday talk
    SeesSigns,
    /// Addresses the beasts before the people
    TalksToBeasts,
    /// Closes every agreement "God willing"
    GodWilling,
    /// Gives the craft the credit
    CreditsTheCraft,
    /// Ends statements "you know"
    YouKnow,
    /// Prices everything in farthings and groats
    PricesAll,
    /// Weather-lore for every subject
    WeatherLore,
    /// Recites the old rhymes
    OldRhymes,
    /// Warns "mark me" ahead of the point
    MarkMe,
    /// Cites a father's example
    AsFatherBefore,
    /// Greets folk as chick and lamb
    DearOnes,
    /// Prefaces answers "truth to tell"
    TruthToTell,
    /// Closes remarks "for my sins"
    ForMySins,
    /// Measures everything in winters lived
    CountsYears,
    /// Names the dead in ordinary talk
    NamesTheDead,
    /// Comforts with worse elsewhere
    WorseElsewhere,
    /// Works every talk around to food
    EverHungry,
    /// Trusts that luck turns
    LuckTurns,
}

/// The assignable habit pool - wide enough that no two villagers need
/// share one (the deal draws without replacement)
pub const HABITS: &[Habit] = &[
    Habit::QuestionBack, Habit::ProverbCaution, Habit::ProverbMockery,
    Habit::ProverbPatience, Habit::ProverbWork, Habit::OathSaints,
    Habit::OathHarvest, Habit::OathMill, Habit::UnderBreath,
    Habit::ThirdPerson, Habit::Echo, Habit::CountsOff, Habit::RhymingCoda,
    Habit::SaintsDay, Habit::FatherSaid, Habit::MotherSaid, Habit::Hums,
    Habit::BlessesHearer, Habit::SeesSigns, Habit::TalksToBeasts,
    Habit::GodWilling, Habit::CreditsTheCraft, Habit::YouKnow,
    Habit::PricesAll, Habit::WeatherLore, Habit::OldRhymes, Habit::MarkMe,
    Habit::AsFatherBefore, Habit::DearOnes, Habit::TruthToTell,
    Habit::ForMySins, Habit::CountsYears, Habit::NamesTheDead,
    Habit::WorseElsewhere, Habit::EverHungry, Habit::LuckTurns,
];

/// The pet phrases - one each, dealt like the habits. They attach to a
/// greeting or parting as a tail, so they read as tics, not lines
pub const PHRASES: &[&'static str] = &[
    "as the mill grinds", "God send it", "saints keep us",
    "mind the stile", "so it goes", "and what of it", "by the rood",
    "softly now", "as my mother taught", "the saints see all",
    "well, well", "as folk do", "mark it well", "pray the weather holds",
    "as the ditch fills", "bless the bread", "come what may",
    "and that's the truth", "if God wills", "as the crow flies",
    "the reeve's own luck", "so they say", "God be good to us",
    "wonders never cease", "by hook and by crook", "come the frost",
    "as sure as sunrise", "the old ways", "peace be on it",
    "a thing said twice", "Lord keep us", "as the well runs",
    "in good time", "small thanks", "the long road home",
    "as grain to bread", "by the stones", "as the bells ring",
    "all in good season", "God send Sunday",
];

impl Habit {
    /// The talk slots the habit fires in - its first matching turn
    /// carries it, and it never fires twice
    pub fn acts(self) -> &'static [Act] {
        use Act::*;
        match self {
            Habit::QuestionBack | Habit::Echo | Habit::CountsOff
            | Habit::CreditsTheCraft | Habit::TruthToTell => &[Answer],
            Habit::OathSaints | Habit::OathHarvest | Habit::OathMill => {
                &[Greet, Agree]
            }
            Habit::Hums | Habit::BlessesHearer | Habit::TalksToBeasts
            | Habit::DearOnes => &[Greet],
            Habit::GodWilling => &[Agree, Farewell],
            Habit::YouKnow | Habit::UnderBreath => &[Answer, Agree],
            Habit::MarkMe | Habit::LuckTurns => &[Remark, PushBack],
            Habit::ProverbCaution | Habit::ProverbMockery
            | Habit::ProverbPatience | Habit::ProverbWork
            | Habit::FatherSaid | Habit::MotherSaid
            | Habit::WorseElsewhere | Habit::ThirdPerson => &[Answer, Remark],
            _ => &[Remark],
        }
    }

    /// Whether the variant replaces, opens or closes the line
    pub fn how(self) -> How {
        match self {
            Habit::OathSaints | Habit::OathHarvest | Habit::OathMill
            | Habit::Hums | Habit::MarkMe | Habit::AsFatherBefore
            | Habit::TruthToTell => How::Preface,
            Habit::RhymingCoda | Habit::GodWilling | Habit::YouKnow
            | Habit::ForMySins => How::Coda,
            _ => How::Whole,
        }
    }

    /// The `VALUES` index this habit speaks for, when it has one -
    /// proverbs carry values cheaply, so the deal prefers a match
    pub fn prefers(self) -> Option<usize> {
        match self {
            Habit::ProverbMockery | Habit::LuckTurns => Some(0),
            Habit::EverHungry | Habit::Hums => Some(2),
            Habit::ProverbWork | Habit::PricesAll | Habit::CreditsTheCraft => Some(3),
            Habit::ThirdPerson | Habit::MarkMe => Some(4),
            Habit::ProverbCaution | Habit::WeatherLore | Habit::WorseElsewhere => Some(5),
            Habit::OathSaints | Habit::SaintsDay | Habit::GodWilling => Some(6),
            Habit::ProverbPatience | Habit::FatherSaid
            | Habit::OldRhymes | Habit::AsFatherBefore => Some(8),
            Habit::MotherSaid | Habit::BlessesHearer
            | Habit::DearOnes | Habit::NamesTheDead => Some(9),
            _ => None,
        }
    }

    /// The whole-line variants the habit owns; the turn index picks
    /// one, deterministically, so a replayed talk reads the same
    pub fn lines(self) -> &'static [&'static str] {
        match self {
            Habit::QuestionBack => &[
                "And what {2:do} {2:subj} think I'd say to that, {addr}?",
                "Now that is a question. What {2:think} {2:subj}?",
                "{2:do} {2:subj} ask, or only talk?",
            ],
            Habit::ProverbCaution => &[
                "A counted sheep is a shorn sheep, my father said.",
                "Better the wolf {2:subj} {2:know}, the saying goes.",
                "Softly goes far, {addr}, and that's the whole of it.",
            ],
            Habit::ProverbMockery => &[
                "They that count sheep all day shear none, friend.",
                "A barred door keeps out friends as well as wolves.",
                "Caution's a warm cloak, and a heavy one.",
            ],
            Habit::ProverbPatience => &[
                "The ditch fills one stone at a time, my mother said.",
                "Green wood burns when it's ready and not before.",
                "All things ripen that are let be, {addr}.",
            ],
            Habit::ProverbWork => &[
                "Idle hands feed no one, as they say.",
                "The field answers work, not wishes.",
                "A bent back makes a full barn.",
            ],
            Habit::OathSaints => &[
                "By the saints - ", "Saints keep us - ",
                "By all the saints - ",
            ],
            Habit::OathHarvest => &[
                "By the harvest - ", "Harvest take it - ",
            ],
            Habit::OathMill => &[
                "By the mill - ", "Mill and water - ",
            ],
            Habit::UnderBreath => &[
                "Mm. As {2:subj} {2:say}.", "Mm-mm. If it must be, it must.",
                "...aye. Aye.",
            ],
            Habit::ThirdPerson => &[
                "{self} minds {his:self} own affairs, thanks.",
                "{self} says what {self} says, {addr}.",
                "{self} was not born yesterday, {addr}.",
            ],
            Habit::Echo => &[
                "Aye. Aye - as {2:subj} {2:say}, as {2:subj} {2:say}.",
                "So {2:subj} {2:say}, so it is.",
                "Heard, and heard again - aye.",
            ],
            Habit::CountsOff => &[
                "One: there's work. Two: there's winter. The rest is talk.",
                "One hand does, two hands finish - that's the count of it.",
                "First the roof, then the stew, then talk. In that order, {addr}.",
            ],
            Habit::RhymingCoda => &[
                " - rain makes grain.", " - frost or thaw, same to the plough.",
                " - and that's no boast.",
            ],
            Habit::SaintsDay => &[
                "Since St. Agnes' eve it's been the same, and same it'll stay.",
                "Come Martinmas we'll know more, mark it.",
                "St. Swithin's day comes, and it still rains, {addr}.",
            ],
            Habit::FatherSaid => &[
                "My father said: pay the miller first, friend.",
                "As my father told it: the well outlives the bucket.",
                "My father's word was: lend once, count twice.",
            ],
            Habit::MotherSaid => &[
                "My mother said: a warm hearth wins no wars alone.",
                "As my mother had it: feed first, ask after.",
                "Mother's rule: a kind word costs nothing, {addr}.",
            ],
            Habit::Hums => &[
                "Mm-mm-mm - ", "Hm, hm - ",
            ],
            Habit::BlessesHearer => &[
                "The saints keep {2:obj}, {addr}.",
                "Blessings on {2:obj} and {2:poss} kin, {other}.",
                "God be good to {2:obj} this day, {addr}.",
            ],
            Habit::SeesSigns => &[
                "The magpies said as much this morning, one for sorrow.",
                "A crow on the church roof - it's a sign, mark it.",
                "The smoke bent north today. That means something, {addr}.",
            ],
            Habit::TalksToBeasts => &[
                "Off the heel, Nip - morrow to {2:obj}, friend.",
                "Easy, old dog - a fair day to {2:obj}, {addr}.",
                "Down, down - sorry, friend, the beasts like company.",
            ],
            Habit::GodWilling => &[
                ", God willing.", ", God willing it.",
                ", the saints willing.",
            ],
            Habit::CreditsTheCraft => &[
                "The craft teaches, not I - I only listen.",
                "Ask the craft, {addr}; it knows more than I do.",
                "The trade answers before I can, and truer.",
            ],
            Habit::YouKnow => &[
                ", you know.", ", {2:subj} {2:know}.", ", you mark me?",
            ],
            Habit::PricesAll => &[
                "Two pennies' worth of worry, that is.",
                "It'd cost {2:obj} a groat to hear worse.",
                "Worth a farthing at market, that news.",
            ],
            Habit::WeatherLore => &[
                "Red sky at night - shepherd's delight, as my gran taught.",
                "Rain before seven, dry by eleven, they say.",
                "The haar's a fisher's blanket - keeps the cold off, mind.",
            ],
            Habit::OldRhymes => &[
                "As the rhyme goes: March dust and April sun makes corn white and weeds none.",
                "The old rhyme says: when the ash is out before the oak, we'll surely get a soak.",
                "A fog in the hollow's a fair day to follow.",
            ],
            Habit::MarkMe => &[
                "Mark me - ", "Mark my words - ", "Hear me now - ",
            ],
            Habit::AsFatherBefore => &[
                "As my father before me - ", "As my father always said - ",
            ],
            Habit::DearOnes => &[
                "Morrow, chick - a fine day to {2:obj}.",
                "Well met, little one, well met.",
                "A good day to {2:obj}, lamb - and to {other} beside.",
            ],
            Habit::TruthToTell => &[
                "Truth to tell - ", "To tell {2:obj} true - ",
                "Plainly put - ",
            ],
            Habit::ForMySins => &[
                ", for my sins.", ", for mine own sins.",
                ", and for my sins beside.",
            ],
            Habit::CountsYears => &[
                "Forty year I've watched that ditch, and it wants digging still.",
                "Sixty winters I've kept this patch, {addr}; I know its ways.",
                "In all my years I never saw the like, {addr}.",
            ],
            Habit::NamesTheDead => &[
                "My Tom, rest him, would have said the same.",
                "The dead knew better than I, and I name them often.",
                "My mother, God rest her, taught me the like.",
            ],
            Habit::WorseElsewhere => &[
                "There's worse in the world, {addr} - worse elsewhere by far.",
                "It could rain harder. It has, and will again.",
                "The next village over has it worse, and we have bread.",
            ],
            Habit::EverHungry => &[
                "All this talk's thin fare next to a loaf, mind.",
                "Speaking's hungry work - is there bread after?",
                "My belly says noon, whatever the sky says, {addr}.",
            ],
            Habit::LuckTurns => &[
                "Luck's a wheel, friend - it turns.",
                "The wind shifts; so does fortune, mark it.",
                "Fortune's a ferry - it comes back round, {addr}.",
            ],
        }
    }
}

/// The phrase's own slot: a greeting or parting it can tail
fn phrase_slot(act: Act) -> bool {
    matches!(act, Act::Greet | Act::Farewell | Act::Agree)
}

/// The ear's landmark, spent once per talk: each speaker's pet phrase
/// tails their first greeting, agreement or parting; their structural
/// habit fires on the first turn whose act its context covers. The
/// player's own words are left alone - their choices are already how
/// they sound. Deterministic per turn, so a replayed talk keeps every
/// line where it landed
pub fn apply(tr: &mut Transcript, pair: &[Speaker; 2], player: usize) {
    let mut phrase_done = [false; 2];
    let mut habit_done = [false; 2];
    for (n, t) in tr.turns.iter_mut().enumerate() {
        if t.speaker == player { continue; }
        let Some(id) = pair.get(t.speaker).and_then(|s| s.idiolect) else { continue };
        if !habit_done[t.speaker] && id.habit.acts().contains(&t.act) {
            let lines = id.habit.lines();
            let v = lines[n % lines.len()];
            let mut rng = conversation::Rng::new(n as u64 + 1);
            let other = &pair[1 - t.speaker];
            match id.habit.how() {
                How::Whole => {
                    t.text = conversation::fill(v, &pair[t.speaker], other, &mut rng);
                }
                How::Preface => {
                    let pre = conversation::fill(v, &pair[t.speaker], other, &mut rng);
                    t.text = format!("{}{}", pre, t.text);
                }
                How::Coda => {
                    let coda = conversation::fill(v, &pair[t.speaker], other, &mut rng);
                    t.text = format!("{}{}", t.text.trim_end_matches('.'), coda);
                }
            }
            habit_done[t.speaker] = true;
        }
        if !phrase_done[t.speaker] && phrase_slot(t.act) {
            t.text = format!("{} - {}.",
                t.text.trim_end_matches(['.', '?', '!']),
                id.phrase.trim_end_matches('.'));
            phrase_done[t.speaker] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::conversation::Turn;

    #[test]
    fn the_pool_is_wide_enough_for_a_village() {
        // the deal draws without replacement - the founding cast of 31
        // must fit under the pool, with country cousins to spare
        assert!(HABITS.len() >= 36);
        assert!(PHRASES.len() >= 36);
    }

    #[test]
    fn every_habit_has_a_slot_and_something_to_say() {
        for h in HABITS {
            assert!(!h.acts().is_empty(), "{h:?} never fires");
            assert!(h.lines().iter().all(|l| !l.is_empty()), "{h:?} has an empty line");
        }
    }

    #[test]
    fn the_habit_fires_once_and_the_phrase_tags_a_greeting() {
        let mut a = Speaker::new("Bram");
        a.idiolect = Some(Idiolect {
            phrase: "as the mill grinds", habit: Habit::TruthToTell });
        let b = Speaker::new("Tilda");
        let mut tr = Transcript {
            speakers: ["Bram".into(), "Tilda".into()], trees_walked: vec![],
            turns: vec![
                Turn { speaker: 0, act: Act::Greet, respond: None,
                    text: "Morrow, Tilda.".into(), concepts: vec![],
                    tree: "t", line: Some("x"),
                    channel: crate::game::setting::Channel::Public },
                Turn { speaker: 1, act: Act::Greet, respond: None,
                    text: "Good day.".into(), concepts: vec![],
                    tree: "t", line: Some("x"),
                    channel: crate::game::setting::Channel::Public },
                Turn { speaker: 0, act: Act::Answer, respond: None,
                    text: "The ditch wants digging.".into(),
                    concepts: vec![], tree: "t", line: Some("x"),
                    channel: crate::game::setting::Channel::Public },
                Turn { speaker: 0, act: Act::Answer, respond: None,
                    text: "And the roof wants thatch.".into(),
                    concepts: vec![], tree: "t", line: Some("x"),
                    channel: crate::game::setting::Channel::Public },
            ],
            ending: None, vibe: None, asked: vec![], forward: 0,
            shared: 0, overshot: false,
            setting: crate::game::setting::STREET,
        };
        apply(&mut tr, &[a, b], usize::MAX);
        assert_eq!(tr.turns[0].text, "Morrow, Tilda - as the mill grinds.");
        assert!(tr.turns[2].text.ends_with("The ditch wants digging.")
            && tr.turns[2].text != "The ditch wants digging.",
            "{:?}", tr.turns[2].text);
        // once a talk, not every line
        assert_eq!(tr.turns[3].text, "And the roof wants thatch.");
    }
}
