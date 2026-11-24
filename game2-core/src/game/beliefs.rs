//! The lie a ghost teaches is a hypothesis the campaign tests (E8-2,
//! #1444): each Belief declares which outcomes bear on it (`hears`),
//! and when one lands, `tested_by` moves the ghost's strength -
//! confirmed up, contradicted down. Past `TURN_FLOOR` the ghost turns:
//! the mark is set, the core memory is filed toward the organizer, and
//! `turn_line` gives the words they say back. Whether the turn surfaces
//! as a scheduled beat is E8-3's; the state, the memory and the words
//! are this ticket's.

use crate::engine::ecs::{Entity, World};
use crate::game::persona::{Belief, Persona};

/// A thing the campaign did that a lie can hear (E8-2): the sites
/// report what happened; which lies it bears on is `hears`' table
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tested {
    /// The bailiff rode home empty - the village stood against a writ
    WritDefied,
    /// The village walked a win in and celebrated out loud
    VictoryWalked,
    /// The cause lost one of its own - the gibbet, the questioning
    MemberLost,
    /// The org stood with one of its own and the standing held
    StoodTogether,
    /// The room decided and the resolve went on the record
    Decided,
    /// A promise was kept where the keeper said it would be
    PromiseKept,
}

/// The signed shift an outcome lands on a lie (E8-2): negative weakens
/// it (the outcome contradicts what the ghost taught), positive
/// confirms it. Zero means the outcome doesn't bear - the listener
/// table is per-lie, which is what makes each lie a hypothesis with
/// its own tests rather than a shared morale meter
pub fn hears(lie: Belief, t: Tested) -> f32 {
    use Belief::*;
    use Tested::*;
    match (lie, t) {
        // "the lord's men always win" breaks on every proof they don't
        (LordAlwaysWins, WritDefied) => -0.16,
        (LordAlwaysWins, VictoryWalked) => -0.12,
        (LordAlwaysWins, StoodTogether) => -0.08,
        (LordAlwaysWins, MemberLost) => 0.10,
        // "the village always finds someone to blame" - the cause
        // refusing to give them a face is the counter-proof; losing one
        // to the rope is the confirmation
        (SomeoneToBlame, MemberLost) => 0.14,
        (SomeoneToBlame, StoodTogether) => -0.12,
        // "the lord's wars take our sons" - every lost member is one
        // more the road kept
        (WarsTakeSons, MemberLost) => 0.14,
        (WarsTakeSons, StoodTogether) => -0.06,
        // "the lord takes even from the dead" - the cause standing
        // between the tally and the grave is its counter-proof
        (LordTakesFromDead, MemberLost) => 0.06,
        (LordTakesFromDead, StoodTogether) => -0.10,
        // "God sends what God sends" - hands that came because they
        // were organized, not sent
        (GodSends, VictoryWalked) => -0.10,
        (GodSends, Decided) => -0.08,
        // "the mill owns us" - a win over the powers that weigh the sacks
        (MillOwnsUs, WritDefied) => -0.10,
        (MillOwnsUs, VictoryWalked) => -0.12,
        // "promises are for those who can keep them" - a kept promise
        // is the direct counter-proof
        (PromisesForKeepers, PromiseKept) => -0.14,
        (PromisesForKeepers, MemberLost) => 0.08,
        // "keep your true self hidden" - a member marked and carried
        // through is the counter-proof; one taken proves the rule
        (KeepHidden, StoodTogether) => -0.10,
        (KeepHidden, MemberLost) => 0.12,
        // "never cross the reeve" - a writ defied and lived through is
        // the crossing that cost nothing
        (NeverCrossReeve, WritDefied) => -0.16,
        (NeverCrossReeve, VictoryWalked) => -0.06,
        (NeverCrossReeve, MemberLost) => 0.12,
        // "we were strong together once" - proof it can happen again
        (StrongTogether, StoodTogether) => -0.14,
        (StrongTogether, VictoryWalked) => -0.08,
        (StrongTogether, MemberLost) => 0.10,
        // "the truth comes out if someone says it" - a room that
        // decided on what was said is the proof
        (TruthComesOut, Decided) => -0.12,
        (TruthComesOut, WritDefied) => -0.06,
        _ => 0.0,
    }
}

/// The strength under which a ghost stops holding (E8-2): below it the
/// lie is disproved enough to turn. The mark is one-way - a re-confirmed
/// ghost can climb again, but the turn is a remembered thing
pub const TURN_FLOOR: f32 = 0.25;

/// A campaign event a villager's ghosts just heard (E8-2): each lie
/// that listens takes the shift, clamped 0..=1; the first to fall under
/// `TURN_FLOOR` turns - `turned` is set, a `Turned` core memory is
/// filed toward `pe` when one's in scope, and the lie is handed back so
/// the site can voice `turn_line`. At most one turn per event: the
/// loudest admission is the one they say
pub fn tested_by(world: &mut World, v: Entity, t: Tested, pe: Option<Entity>, day: u64) -> Option<Belief> {
    let mut p = world.get_component::<Persona>(v)?.clone();
    let mut turned = None;
    for g in p.ghosts.iter_mut() {
        let d = hears(g.lie, t);
        if d == 0.0 { continue; }
        g.strength = (g.strength + d).clamp(0.0, 1.0);
        if !g.turned && g.strength < TURN_FLOOR && turned.is_none() {
            g.turned = true;
            turned = Some(g.lie);
        }
    }
    world.add_component(v, p);
    if let (Some(lie), Some(pe)) = (turned, pe) {
        // the memory is seat-scoped: filing over another player's slot
        // would erase their history with this villager - the turn and
        // the spoken line still land, only the record waits
        let foreign = world.get_component::<crate::game::episodes::PairMemory>(v)
            .map(|m| m.pe != pe).unwrap_or(false);
        if !foreign {
            crate::game::episodes::file(world, v, pe, crate::game::episodes::MemKind::Turned,
                day, &["turned", lie.tag()]);
        }
    }
    turned
}

/// The words a turned ghost finally says (E8-2): the admission the lie
/// kept back, in their own mouth. E8-3 schedules the beat it lands in;
/// the sites log it when the turn happens in public
pub fn turn_line(lie: Belief) -> &'static str {
    use Belief::*;
    match lie {
        LordAlwaysWins =>
            "The lord's men always win - I was raised on it. Then I watched them ride home empty.",
        SomeoneToBlame =>
            "I always said they'd find a face for it. They didn't - they stood by the one they could have blamed.",
        WarsTakeSons =>
            "I counted the road's takings for years. This time the counting went the other way.",
        LordTakesFromDead =>
            "I buried what's dear so deep I forgot the burying. Maybe not everything needs the hole.",
        GodSends =>
            "I waited on God to send. It turns out hands can be gathered, not just sent.",
        MillOwnsUs =>
            "The mill owns us - I weighed it true for years. The weights can move; I've seen it now.",
        PromisesForKeepers =>
            "I said promises were for keepers. I watched one kept where the keeping cost something.",
        KeepHidden =>
            "I kept myself hidden so long I forgot why. Someone stood in the open and lived.",
        NeverCrossReeve =>
            "Never cross the reeve - I've said it fifty years. The reeve was crossed, and the sky held.",
        StrongTogether =>
            "We were strong together once, and I let the once do all the work. It's not only once.",
        TruthComesOut =>
            "I always swore the truth comes out. This time it came out and something moved for it.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::persona::*;

    fn ghosted(lie: Belief, strength: f32) -> Persona {
        let mut p = Persona {
            seed: 1, sex: crate::game::villager::Sex::Male, archetype: Archetype::OldGuard,
            temperament: [0.5; 5], trust_propensity: 0.5, values: [0.5; 10],
            story: vec![], ghosts: vec![], signature: Signature::Habit("x"),
            hidden: Landmark::Skill("y"), contradiction: None,
            idiolect: crate::game::idiolect::Idiolect {
                phrase: "x", habit: crate::game::idiolect::Habit::Echo },
        };
        p.ghosts.push(Ghost { event: EventId(0), kind: EventKind::Levy, hook: None,
            lie, strength, fear: Fear::BailiffKnowsMyFace, need: Need::ToBeSafe,
            turned: false });
        p
    }

    fn world_with(p: Persona) -> (World, Entity, Entity) {
        let mut w = World::new();
        let (pe, v) = (w.create_entity(), w.create_entity());
        w.add_component(v, p);
        (w, pe, v)
    }

    fn strength_of(w: &World, v: Entity) -> f32 {
        w.get_component::<Persona>(v).unwrap().ghosts[0].strength
    }

    /// A listening lie sheds the same slice per proof it hears
    #[test]
    fn a_listening_lie_weakens_on_each_proof() {
        let (mut w, pe, v) = world_with(ghosted(Belief::LordAlwaysWins, 0.8));
        assert_eq!(tested_by(&mut w, v, Tested::WritDefied, Some(pe), 10), None);
        assert!((strength_of(&w, v) - 0.64).abs() < 0.001);
        tested_by(&mut w, v, Tested::WritDefied, Some(pe), 11);
        assert!((strength_of(&w, v) - 0.48).abs() < 0.001);
    }

    /// An event the lie doesn't declare leaves it untouched
    #[test]
    fn a_lie_that_doesnt_listen_stays_put() {
        let (mut w, pe, v) = world_with(ghosted(Belief::GodSends, 0.8));
        assert_eq!(tested_by(&mut w, v, Tested::WritDefied, Some(pe), 10), None);
        assert_eq!(strength_of(&w, v), 0.8);
        assert!(tested_by(&mut w, v, Tested::Decided, Some(pe), 11).is_none());
        assert!((strength_of(&w, v) - 0.72).abs() < 0.001);
    }

    /// Done-when: under the floor the ghost turns once, the core memory
    /// is filed toward the organizer, and the words exist to say back
    #[test]
    fn a_disproved_lie_turns_once_and_the_villager_says_so() {
        let (mut w, pe, v) = world_with(ghosted(Belief::LordAlwaysWins, 0.4));
        let lie = tested_by(&mut w, v, Tested::WritDefied, Some(pe), 10);
        assert_eq!(lie, Some(Belief::LordAlwaysWins));
        tested_by(&mut w, v, Tested::WritDefied, Some(pe), 11);
        let p = w.get_component::<Persona>(v).unwrap();
        assert!(p.ghosts[0].turned);
        assert!(p.ghosts[0].strength < TURN_FLOOR);
        let line = turn_line(Belief::LordAlwaysWins);
        assert!(line.contains("ride home empty"));
        let mem = w.get_component::<crate::game::episodes::PairMemory>(v).unwrap();
        assert_eq!(mem.pe, pe);
        assert_eq!(mem.eps.len(), 1);
        assert_eq!(mem.eps[0].kind, crate::game::episodes::MemKind::Turned);
        assert!(mem.eps[0].concepts.contains(&"lord_always_wins"));
        // one-way: further proofs don't re-fire the turn
        assert_eq!(tested_by(&mut w, v, Tested::WritDefied, Some(pe), 12), None);
        assert_eq!(tested_by(&mut w, v, Tested::VictoryWalked, Some(pe), 13), None);
        assert_eq!(w.get_component::<crate::game::episodes::PairMemory>(v)
            .unwrap().eps.len(), 1);
    }

    /// A lost member confirms the lies that predicted exactly that end
    #[test]
    fn a_lost_member_hardens_the_counter_lies() {
        let (mut w, pe, v) = world_with(ghosted(Belief::WarsTakeSons, 0.5));
        tested_by(&mut w, v, Tested::MemberLost, Some(pe), 10);
        assert!((strength_of(&w, v) - 0.64).abs() < 0.001);
        tested_by(&mut w, v, Tested::StoodTogether, Some(pe), 11);
        assert!((strength_of(&w, v) - 0.58).abs() < 0.001);
        assert_eq!(tested_by(&mut w, v, Tested::PromiseKept, Some(pe), 12), None);
        assert!((strength_of(&w, v) - 0.58).abs() < 0.001);
    }

    /// The turn sheds the lie's drag and its tell; the wound's habit
    /// stays on
    #[test]
    fn a_turned_ghost_stops_wearing_the_lie() {
        let (mut w, pe, v) = world_with(ghosted(Belief::LordAlwaysWins, 0.4));
        let tells = crate::game::fact_content::tells_of(
            w.get_component::<Persona>(v).unwrap());
        assert!(tells.contains(&"makes_way") && tells.contains(&"shuns_the_gate"));
        tested_by(&mut w, v, Tested::WritDefied, Some(pe), 10);
        tested_by(&mut w, v, Tested::WritDefied, Some(pe), 11);
        let p = w.get_component::<Persona>(v).unwrap().clone();
        assert!(p.ghosts[0].turned);
        let tells = crate::game::fact_content::tells_of(&p);
        assert!(!tells.contains(&"makes_way") && tells.contains(&"shuns_the_gate"));
        // lie row's showing factor drops; the wound's stays
        let f = crate::game::fact_content::showing_factor(&w, v);
        assert!((f - 0.55).abs() < 0.001, "f={f}");
    }

    /// The memory slot is seat-scoped: a turn earned by one seat's
    /// campaign doesn't erase another player's record - the ghost still
    /// turns and still speaks, only the filing waits
    #[test]
    fn a_foreign_seats_memory_isnt_clobbered() {
        let (mut w, pe, v) = world_with(ghosted(Belief::LordAlwaysWins, 0.4));
        let other = w.create_entity();
        crate::game::episodes::file(&mut w, v, other,
            crate::game::episodes::MemKind::KeptPromise, 1, &["promise"]);
        assert_eq!(tested_by(&mut w, v, Tested::WritDefied, Some(pe), 10),
            Some(Belief::LordAlwaysWins));
        let mem = w.get_component::<crate::game::episodes::PairMemory>(v).unwrap();
        assert_eq!(mem.pe, other, "the other seat's record stands");
        assert_eq!(mem.eps.len(), 1);
        // the same seat's own win files the turn like always
        let (mut w2, pe2, v2) = world_with(ghosted(Belief::LordAlwaysWins, 0.4));
        crate::game::episodes::file(&mut w2, v2, pe2,
            crate::game::episodes::MemKind::SharedMeal, 1, &["meal"]);
        tested_by(&mut w2, v2, Tested::WritDefied, Some(pe2), 10);
        assert_eq!(w2.get_component::<crate::game::episodes::PairMemory>(v2)
            .unwrap().eps.len(), 2);
    }

    /// Every lie owns an authored admission and at least one listener
    #[test]
    fn every_lie_has_a_turn_line_and_ears() {
        for b in BELIEFS {
            assert!(!turn_line(b).is_empty(), "{b:?} no turn line");
            assert!([Tested::WritDefied, Tested::VictoryWalked, Tested::MemberLost,
                Tested::StoodTogether, Tested::Decided, Tested::PromiseKept]
                .iter().any(|&t| hears(b, t) != 0.0), "{b:?} listens to nothing");
        }
    }
}
