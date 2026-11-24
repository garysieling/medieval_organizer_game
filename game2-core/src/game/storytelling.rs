//! Telling a story in Labov's parts (E4-8, #1426; design
//! `docs/plans/procedural-characters.md` Appendix B, Part 5): abstract,
//! orientation, complicating action, evaluation, resolution, coda -
//! the order, the evaluation and the omissions chosen per teller. The
//! widow tells the levy as loss, the reeve's cousin as duty, the
//! firebrand as theft: shared events become a cross-check - the facts
//! agree, the meanings don't. The surface is the Talk ask-topic
//! (`Draw::Story`): the player asks, they tell.

use crate::game::persona::{Belief, EventKind, Persona};
use crate::game::traits::Trait;

/// Labov's six parts of a personal narrative
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part { Abstract, Orient, Action, Eval, Resolve, Coda }

/// The record a villager tells from: their deepest prehistory event,
/// the part they played in it, whom else it touched, and the frame
/// their stake sets on it. Seeded on the Speaker at talk open so the
/// walk needs no world to tell it
#[derive(Debug, Clone)]
pub struct StorySeed {
    /// Which of the nine wounds the event was
    pub kind: EventKind,
    /// How far back it sits - the orientation names it
    pub years_ago: u8,
    /// The part the teller played ("buried their own", "took a side")
    pub part: &'static str,
    /// The record's other souls: (name, part) - the facts two tellings
    /// of one event share
    pub others: Vec<(String, &'static str)>,
    /// The evaluation the telling exists to carry, read off the teller
    /// at seed time
    pub frame: Frame,
}

/// How the teller reads what happened - the evaluation the whole
/// telling exists to carry. Their part sets it; a devout mouth or a
/// fate-fed lie bends it toward God's sending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame { Loss, Duty, Wrong, Fight, Warm, Fate }

/// The frame a part sets on the telling - the stake the teller had in
/// the event is what Labov calls the evaluation's home. Called at seed
/// time, where the teller's persona is still in reach
pub fn frame_of(part: &'static str, p: Option<(&Persona, &[Trait])>) -> Frame {
    let mut f = match part {
        "buried their own" | "remembers one the levy took" | "was promised"
        | "burned out" | "watched it burn" => Frame::Loss,
        "paid the death duty" | "served and came home" | "took the strip" => Frame::Duty,
        "blamed" | "put out for arrears" | "owes the miller" | "broke the troth"
        | "was made to spit on them" => Frame::Wrong,
        "took a side" | "taken" => Frame::Fight,
        "feasted" => Frame::Warm,
        "still keeps the stones" => Frame::Fate,
        _ => Frame::Duty,
    };
    if let Some((p, traits)) = p {
        // fate reads first where it's already the lesson or the habit
        let fate = traits.contains(&Trait::Pious)
            || p.ghosts.iter().any(|g| g.lie == Belief::GodSends);
        if fate { f = Frame::Fate; }
    }
    f
}

/// The parts this frame tells, in order - the order is the teller's
/// choice: the widow's loss opens on what it meant and never says how
/// it ended; duty and warmth run the full course
pub fn order_of(f: Frame) -> &'static [Part] {
    use Part::*;
    match f {
        Frame::Loss => &[Eval, Orient, Action, Coda],
        Frame::Wrong => &[Orient, Action, Eval, Coda],
        Frame::Fight => &[Abstract, Action, Eval, Resolve, Coda],
        Frame::Fate => &[Orient, Action, Eval, Coda],
        Frame::Duty | Frame::Warm => &[Abstract, Orient, Action, Eval, Resolve, Coda],
    }
}

/// The fact-clause a record part renders as, in the teller's mouth -
/// the part strings are the record's own (`prehistory::record`); the
/// kind says which "taken" it was
fn clause(kind: EventKind, name: &str, part: &str, own: bool) -> String {
    let who = if own { "I".into() } else { name.to_string() };
    let own_of = |s: &str| if own { s.replace("their", "my") } else { s.to_string() };
    match part {
        "taken" => match kind {
            EventKind::Fever => if own { "the sickness took mine".into() }
                else { format!("the sickness took {name}") },
            _ => format!("{who} went in the levy"),
        },
        "burned out" => format!("{who} lost the roof to the fire"),
        "watched it burn" => format!("{who} stood and watched it burn"),
        "blamed" => format!("{who} took the blame for it"),
        "paid the death duty" => format!("{who} paid the toll on the dead"),
        "died" => format!("{who} died of it"),
        "buried their own" => own_of(&format!("{who} buried their own")),
        "owes the miller" => if own { "I still owe the miller".into() }
            else { format!("{who} still owes the miller") },
        "broke the troth" => format!("{who} broke the troth"),
        "was promised" => format!("{who} was promised once"),
        "took a side" => format!("{who} took a side"),
        "put out for arrears" => format!("{who} was put out for the arrears"),
        "took the strip" => format!("{who} took the strip off the family"),
        "feasted" => format!("{who} danced the ring till the moon quit"),
        "served and came home" => format!("{who} served and came home"),
        "remembers one the levy took" => if own {
            "I remember one the levy took".into() }
            else { format!("{who} remembers one the levy took") },
        "still keeps the stones" => if own { "I still keep the stones".into() }
            else { format!("{who} still keeps the stones") },
        "was made to spit on them" => format!("{who} was made to spit on the stones"),
        _ => format!("{who} was there"),
    }
}

/// The facts a telling must cover - the record's own entries, teller
/// first: two tellings of one event agree on these because they render
/// the same touched list, in the same words
pub fn facts_of(seed: &StorySeed) -> Vec<String> {
    let mut v = vec![clause(seed.kind, "", seed.part, true)];
    for (n, part) in &seed.others { v.push(clause(seed.kind, n, part, false)); }
    v
}

/// The orientation line per wound - when and where, the scene set
fn orient_line(seed: &StorySeed) -> &'static str {
    match seed.kind {
        EventKind::Levy => "It was the year the call went out at the gate.",
        EventKind::Fire => "It was a dry thatch year, and the wind had a mind.",
        EventKind::DeathDuty => "It was the burying season, when the tally men ride.",
        EventKind::Fever => "It was the year the sickness went house to house.",
        EventKind::MillerDebt => "It was the year the miller weighed the sacks twice.",
        EventKind::BrokenBetrothal => "It was the year the troth was spoken and unspoken.",
        EventKind::OldFaith => "It was the year they went to the stones still.",
        EventKind::Eviction => "It was the year the rents were called in.",
        EventKind::HarvestFeast => "It was the year the barns stood full.",
    }
}

/// The evaluation each frame hangs on the story - the meaning line,
/// where the person shows
fn eval_line(f: Frame) -> &'static str {
    match f {
        Frame::Loss => "It took what it took, and I've not stopped counting it.",
        Frame::Duty => "You did what was asked of you - that's all any of us could do.",
        Frame::Wrong => "It was never a fair reckoning, and I've not forgot it.",
        Frame::Fight => "That's where I learned it's not just taken - it can be fought.",
        Frame::Warm => "One good year - I've held it like a coin I won't spend.",
        Frame::Fate => "God sends what God sends - who am I to call it else?",
    }
}

/// The resolution and coda lines - how it came out, the door it closes on
fn resolve_line(f: Frame) -> &'static str {
    match f {
        Frame::Duty => "It ended as it had to, and we went on.",
        Frame::Fight => "It ended with us still standing - remember that.",
        Frame::Warm => "It ended round the fire with the music up.",
        _ => "It ended how it ended.",
    }
}

fn coda_line(f: Frame) -> &'static str {
    match f {
        Frame::Loss => "And that's all I'll say of it.",
        Frame::Wrong => "So don't ask me to think better of it now.",
        Frame::Fight => "And I'd stand that way again.",
        Frame::Warm => "And it keeps the hard years off, remembering it.",
        Frame::Fate => "And it's sent or it isn't - that's the whole of it.",
        Frame::Duty => "And that's the shape of it, as I recall.",
    }
}

/// The telling itself: the plan's parts rendered in order. Facts come
/// off the shared record - a lost or fate-fed teller trims the list
/// when it runs long (the omission is the persona's, not the record's)
pub fn tell(seed: &StorySeed) -> (Frame, String) {
    let f = seed.frame;
    let order = order_of(f);
    let mut facts = facts_of(seed);
    // omissions: a long record gets trimmed by the teller who'd rather
    // not carry it all - the wound keeps the count honest
    if facts.len() > 3 && matches!(f, Frame::Loss | Frame::Fate) {
        facts.truncate(3);
    }
    let abstract_line = match seed.kind {
        EventKind::Levy => "The levy year - you've heard tell of it.",
        EventKind::Fire => "The year the barn went - that one.",
        EventKind::HarvestFeast => "The full-barn year - the one good one.",
        _ => "There's one I keep - the worst year, or near it.",
    };
    let mut parts: Vec<String> = vec![];
    for part in order {
        parts.push(match part {
            Part::Abstract => abstract_line.into(),
            Part::Orient => format!("{} {} years back now.", orient_line(seed), seed.years_ago),
            Part::Action => format!("{}.", facts.join("; ")),
            Part::Eval => eval_line(f).into(),
            Part::Resolve => resolve_line(f).into(),
            Part::Coda => coda_line(f).into(),
        });
    }
    (f, parts.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::persona::*;
    use crate::game::traits::Personality;

    fn shared_seed(part: &'static str) -> StorySeed {
        StorySeed { kind: EventKind::Levy, years_ago: 12, part,
            others: vec![("Hal".into(), "taken"), ("Maud".into(), "remembers one the levy took")],
            frame: frame_of(part, None) }
    }

    /// Done-when: two tellings of one event agree on the facts and
    /// differ in evaluation
    #[test]
    fn two_tellings_share_the_facts_and_split_the_meaning() {
        let (widow, kin) = (shared_seed("buried their own"), shared_seed("served and came home"));
        let (fw, tw) = tell(&widow);
        let (fk, tk) = tell(&kin);
        assert_ne!(fw, fk);
        for fact in ["Hal went in the levy", "Maud remembers one the levy took"] {
            assert!(tw.contains(fact) && tk.contains(fact), "{fact}");
        }
        assert!(tw.contains("It took what it took"));
        assert!(tk.contains("what was asked of you"));
    }

    /// The widow's loss never says how it ended - no resolution part
    #[test]
    fn the_loss_telling_drops_the_resolution() {
        assert!(!order_of(Frame::Loss).contains(&Part::Resolve));
        assert!(order_of(Frame::Duty).contains(&Part::Resolve));
    }

    /// The ordering is the teller's: loss leads on the meaning, duty
    /// walks the record in order
    #[test]
    fn the_order_is_the_tellers() {
        assert_eq!(order_of(Frame::Loss)[0], Part::Eval);
        assert_eq!(order_of(Frame::Duty)[0], Part::Abstract);
        assert_eq!(order_of(Frame::Fight)[1], Part::Action);
    }

    /// A devout mouth bends the same part toward God's sending
    #[test]
    fn the_pious_hear_it_as_sent() {
        let seed = shared_seed("buried their own");
        let p = Personality { traits: vec![Trait::Pious] };
        let (f, _) = tell(&seed);
        assert_eq!(f, Frame::Loss);
        let pious = frame_of("buried their own", Some((&Persona {
            seed: 1, sex: crate::game::villager::Sex::Male, archetype: Archetype::OldGuard,
            temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
            story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
            hidden: Landmark::Skill("y"), contradiction: None,
            idiolect: crate::game::idiolect::Idiolect {
                phrase: "x", habit: crate::game::idiolect::Habit::QuestionBack },
        }, &p.traits)));
        assert_eq!(pious, Frame::Fate);
    }

    /// The omission is the teller's: a long record keeps its count only
    /// for those who can carry it
    #[test]
    fn a_long_record_gets_trimmed_by_who_cant_carry_it() {
        let mut seed = shared_seed("feasted");
        seed.others.push(("Odo".into(), "feasted"));
        let (fw, t) = tell(&seed);
        assert_eq!(fw, Frame::Warm);
        assert!(t.contains("Odo"), "warmth keeps the whole table");
        seed.part = "buried their own";
        seed.frame = frame_of("buried their own", None);
        let (fl, t) = tell(&seed);
        assert_eq!(fl, Frame::Loss);
        assert!(!t.contains("Odo"), "loss trims what it can't carry");
    }

    /// The part string is the record's own word, read under the kind:
    /// a fever "taken" is not a levy "taken" (review finding)
    #[test]
    fn a_fever_taken_is_no_levy_taken() {
        let seed = StorySeed { kind: EventKind::Fever, years_ago: 9,
            part: "buried their own",
            others: vec![("the little one".into(), "taken")],
            frame: frame_of("buried their own", None) };
        let (_, t) = tell(&seed);
        assert!(t.contains("the sickness took the little one"), "{}", t);
        assert!(!t.contains("levy"), "{}", t);
    }

    /// Old-faith souls keep or lose the stones - the telling names
    /// which, in the record's own words
    #[test]
    fn the_stones_are_kept_or_spat_on() {
        let kept = StorySeed { kind: EventKind::OldFaith, years_ago: 22,
            part: "still keeps the stones", others: vec![],
            frame: frame_of("still keeps the stones", None) };
        let (fk, t) = tell(&kept);
        assert_eq!(fk, Frame::Fate);
        assert!(t.contains("I still keep the stones"), "{}", t);
        let spat = StorySeed { kind: EventKind::OldFaith, years_ago: 22,
            part: "was made to spit on them", others: vec![],
            frame: frame_of("was made to spit on them", None) };
        let (fs, t) = tell(&spat);
        assert_eq!(fs, Frame::Wrong);
        assert!(t.contains("I was made to spit on the stones"), "{}", t);
    }

    /// The teller's own part speaks first person - the record's
    /// "their" becomes "my" in their mouth
    #[test]
    fn the_tellers_own_part_speaks_first_person() {
        let seed = shared_seed("buried their own");
        let (_, t) = tell(&seed);
        assert!(t.contains("I buried my own"), "{}", t);
    }
}
