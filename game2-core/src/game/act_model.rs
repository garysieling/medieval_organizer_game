//! The act model (E8-1, #1442): whether a villager moves when asked is
//! nine tensions weighed against the ask's cost, not a flat stage
//! number - urgency, anger, hope, solidarity and efficacy push them out
//! the door; inertia, fear, isolation and self-doubt keep them in.
//! Ganz's account: fear can bend anger aside into resentment, so anger
//! counts for less as fear rises. The organizer's moves push the terms -
//! an argued case agitates, a carried demand raises hope, a shared meal
//! cuts isolation, a kept promise builds efficacy, and a counter-move
//! lands harder on an unreadied, nervous soul.
//!
//! One read - `act_chance` - is asked wherever a villager chooses to
//! act: the one-to-one asks (`yes_chance`), the bare word's call to a
//! meeting (`shows_up`), and the step of signing on (`house_meeting`'s
//! recruit). The player's own read of it is what their ledger knows.

use crate::engine::ecs::{Entity, World};
use crate::game::relationships::{bonds_of, find_relationship, Relationship};

/// The pushes a campaign has moved a villager by, one per tension -
/// they ride on top of what the day already says, so a warming bond or
/// a fresh trouble still counts without rewriting the record
#[derive(Debug, Clone, Copy, Default)]
pub struct Pressure {
    /// the troubles pressing them toward now
    pub urgency: f32,
    /// what the wound's built up - bends aside under fear
    pub anger: f32,
    /// that this can be different
    pub hope: f32,
    /// that they're not alone in it
    pub solidarity: f32,
    /// that their part can matter
    pub efficacy: f32,
    /// the drag of another day's work
    pub inertia: f32,
    /// what it could cost them
    pub fear: f32,
    /// that nobody stands beside them
    pub isolation: f32,
    /// that they're not the one to carry it
    pub self_doubt: f32,
}

impl Pressure {
    fn add(&mut self, o: &Pressure) {
        self.urgency += o.urgency;
        self.anger += o.anger;
        self.hope += o.hope;
        self.solidarity += o.solidarity;
        self.efficacy += o.efficacy;
        self.inertia += o.inertia;
        self.fear += o.fear;
        self.isolation += o.isolation;
        self.self_doubt += o.self_doubt;
    }
    fn clamped(&mut self) {
        for v in [&mut self.urgency, &mut self.anger, &mut self.hope,
            &mut self.solidarity, &mut self.efficacy, &mut self.inertia,
            &mut self.fear, &mut self.isolation, &mut self.self_doubt] {
            *v = v.clamp(0.0, 1.0);
        }
    }
}

/// What their life already says, before the campaign's pushes: read off
/// the bond they hold with the asker, the persona's temperament, the
/// ghosts they carry, the deeds behind them and the troubles pressing.
/// A computed read, so it moves as the village does - and a villager
/// nobody's pushed still has a shape.
fn base_tensions(world: &World, organizer: Entity, v: Entity, tick: u64) -> Pressure {
    use crate::game::traits::Trait::*;
    let [o, c, e, _a, n] = world.get_component::<crate::game::persona::Persona>(v)
        .map(|p| p.temperament).unwrap_or([0.5; 5]);
    let ghost = world.get_component::<crate::game::persona::Persona>(v)
        .map(|p| p.ghosts.iter().map(|g| g.strength).fold(0.0f32, f32::max))
        .unwrap_or(0.0);
    let traits = world.get_component::<crate::game::traits::Personality>(v);
    let has = |t: crate::game::traits::Trait| traits
        .map(|p| p.traits.contains(&t)).unwrap_or(false);
    // the bond the ask rides on: a stranger's voice weighs nothing,
    // and the pair's kept-work history is kept on it too
    let rel = find_relationship(world, organizer, v);
    let bond = rel.and_then(|r| world.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    // who else stands with them: a full bench cuts the alone of it
    let close = bonds_of(world, v).iter().filter(|&&b|
        world.get_component::<Relationship>(b)
            .map(|r| r.strength >= 0.3
                && !crate::game::villager::is_dead(world,
                    if r.from == v { r.other } else { r.from }))
            .unwrap_or(false)).count() as f32;
    let problems = crate::game::desires::problems_of(world, v, tick).len() as f32;
    let deeds = rel.and_then(|r| world.get_component::<crate::game::organizing::Deeds>(r))
        .map(|d| d.n).unwrap_or(0) as f32;
    Pressure {
        urgency: (0.1 + problems * 0.15 + ghost * 0.2).min(0.8),
        anger: (0.1 + ghost * 0.45 + if has(Hotheaded) { 0.25 } else { 0.0 }).min(0.8),
        hope: (0.2 + o * 0.25).min(0.6),
        solidarity: (bond.clamp(0.0, 1.0) * 0.9 + e * 0.1).min(0.9),
        efficacy: (0.1 + deeds * 0.08).min(0.7),
        inertia: (0.28 - c * 0.18
            + if has(Lazy) { 0.3 } else { 0.0 }
            + if has(Stolid) { 0.15 } else { 0.0 }).clamp(0.05, 0.8),
        fear: (0.1 + n * 0.35 + ghost * 0.25
            + if has(Wary) { 0.15 } else { 0.0 }).min(0.85),
        isolation: (0.5 - e * 0.3 - close * 0.12).clamp(0.05, 0.8),
        self_doubt: (0.1 + n * 0.3 + ghost * 0.2).min(0.8),
    }
}

/// Their standing on the nine tensions: what the day gives them plus
/// what the campaign has pushed them to
pub fn tension_of(world: &World, organizer: Entity, v: Entity, tick: u64) -> Pressure {
    let mut t = base_tensions(world, organizer, v, tick);
    if let Some(p) = world.get_component::<Pressure>(v).copied() { t.add(&p); }
    t.clamped();
    t
}

/// Move one of their tensions: the push stacks on the record the day's
/// read adds to. `f` gets the accumulated pushes, not the live standing
pub fn press(world: &mut World, v: Entity, f: impl FnOnce(&mut Pressure)) {
    if world.get_component::<Pressure>(v).is_none() {
        world.add_component(v, Pressure::default());
    }
    if let Some(p) = world.get_component_mut::<Pressure>(v) { f(p); }
}

/// The chance they act on a cost `c` ask: the tensions summed by
/// Ganz's account, sigmoid to a roll-able chance. Callers whose ask
/// grows dearer from a stranger's voice fold `stranger_premium` into
/// the cost themselves - reads that already carry a bond term don't
/// pay it twice
pub fn act_chance(world: &World, organizer: Entity, v: Entity, cost: f32, tick: u64) -> f32 {
    let t = tension_of(world, organizer, v, tick);
    let x = t.urgency + t.anger * (1.0 - t.fear) + t.hope + t.solidarity + t.efficacy
        - t.inertia - t.fear - t.isolation - t.self_doubt - cost;
    sigmoid(3.5 * x + 0.7)
}

/// The surcharge a stranger's voice adds to an ask: nobody's ask costs
/// them nearly half again what a friend's does
pub fn stranger_premium(world: &World, organizer: Entity, v: Entity) -> f32 {
    let bond = find_relationship(world, organizer, v)
        .and_then(|r| world.get_component::<Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    (1.0 - bond.clamp(0.0, 1.0)) * 0.4
}

fn sigmoid(x: f32) -> f32 { 1.0 / (1.0 + (-x).exp()) }

/// An argued case is the agitation work itself: their trouble gains
/// its urgency and its edge, won or lost - and a pressed nerve leaves
/// mostly the edge
pub fn argued(world: &mut World, v: Entity, won: bool) {
    press(world, v, |p| {
        p.urgency += if won { 0.10 } else { 0.06 };
        p.anger += if won { 0.08 } else { 0.10 };
    });
}

/// A carried demand walked through the streets: everyone who saw it
/// believes a little more
pub fn win_witnessed(world: &mut World, v: Entity) {
    press(world, v, |p| p.hope += 0.12);
}

/// Company at table or at meeting cuts the alone of it - the outgoing
/// take more of it than the shut-in, and a shared meal more than bare
/// talk (`weight` scales the keeping)
pub fn company_kept(world: &mut World, v: Entity, weight: f32) {
    let e = world.get_component::<crate::game::persona::Persona>(v)
        .map(|p| p.temperament[2]).unwrap_or(0.5);
    press(world, v, |p| {
        p.isolation -= 0.05 * weight * (0.6 + e);
        p.solidarity += 0.02 * weight;
    });
}

/// A complaint heard in the room that was already theirs: it lands
/// lighter than a meal, but it lands - they are not the only one
pub fn parallel_heard(world: &mut World, v: Entity, speaker: bool) {
    press(world, v, |p| p.isolation -= if speaker { 0.05 } else { 0.03 });
}

/// A kept small ask is proof their part can matter
pub fn ask_kept(world: &mut World, v: Entity) {
    press(world, v, |p| {
        p.efficacy += 0.08;
        p.hope += 0.03;
        p.solidarity += 0.02;
    });
}

/// A counter-move lands on them: readied, it still registers; raw, it
/// lands whole - and the nervous take it deeper either way
pub fn counter_lands(world: &mut World, v: Entity, shielded: bool) {
    let n = world.get_component::<crate::game::persona::Persona>(v)
        .map(|p| p.temperament[4]).unwrap_or(0.5);
    let blow = if shielded { 0.04 } else { 0.12 } * (0.5 + n);
    press(world, v, |p| p.fear += blow);
}

/// Signing on to the cause at a meeting: a quieter pledge than the
/// petition's public name, but still theirs to give
pub const JOIN_COST: f32 = 0.6;

/// What the ask's price reads as to the model: the cost term `act_chance`
/// weighs against the tensions
pub fn ask_cost(p: crate::game::organizing::PropKind) -> f32 {
    use crate::game::organizing::PropKind::*;
    match p {
        ComeAlong => 0.25,
        TakeNotes => 0.30,
        CloseOut => 0.30,
        Introduce => 0.30,
        GiveSong => 0.35,
        BringSnacks => 0.35,
        OpenWithWord => 0.40,
        InviteFriends => 0.50,
        LendVenue => 0.55,
        // the name on paper is the dearest ask there is
        SignPetition => 0.80,
        Argue | Inoculate | Scout | Probe => 0.30,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::organizing::PropKind;

    /// A villager of the given bond to the asker, nothing pushed
    fn world_with(bond: f32) -> (World, Entity, Entity) {
        let mut w = World::new();
        let (org, v) = (w.create_entity(), w.create_entity());
        w.add_component(v, crate::game::traits::Personality { traits: vec![] });
        if bond > 0.0 {
            let r = crate::game::relationships::record_interaction(&mut w, org,
                crate::game::relationships::Interaction {
                    other: v, kind: crate::game::relationships::InteractionKind::Aid,
                    intensity: bond });
            w.get_component_mut::<crate::game::relationships::Relationship>(r)
                .unwrap().strength = bond;
        }
        (w, org, v)
    }

    #[test]
    fn test_the_model_separates_strangers_from_followers() {
        // the done-when's shape: a stranger's ask rarely lands, a known
        // one's often, a follower's near always
        let bands = [(0.0, 0.02, 0.30), (0.35, 0.25, 0.75), (0.8, 0.55, 1.0)];
        for (bond, lo, hi) in bands {
            let (w, org, v) = world_with(bond);
            let p = act_chance(&w, org, v,
                ask_cost(PropKind::ComeAlong) + stranger_premium(&w, org, v), 10);
            assert!(p >= lo && p <= hi,
                "bond {bond}: P(act) {p:.2} outside {lo}..{hi}");
        }
    }

    #[test]
    fn test_fear_bends_anger_aside() {
        // the same anger moves a brave one and stalls a frightened one
        let (w, org, v) = world_with(0.4);
        let calm = act_chance(&w, org, v, 0.3, 10);
        let mut w = w;
        press(&mut w, v, |p| p.fear += 0.4);
        let scared = act_chance(&w, org, v, 0.3, 10);
        assert!(scared < calm - 0.1,
            "fear should stall what anger was starting: {calm:.2} -> {scared:.2}");
    }

    #[test]
    fn test_the_campaigns_pushes_move_the_read() {
        let (mut w, org, v) = world_with(0.4);
        let before = act_chance(&w, org, v, 0.4, 10);
        for _ in 0..3 { argued(&mut w, v, true); }
        for _ in 0..4 { company_kept(&mut w, v, 1.0); }
        win_witnessed(&mut w, v);
        ask_kept(&mut w, v);
        let after = act_chance(&w, org, v, 0.4, 10);
        assert!(after > before + 0.15,
            "agitation and company should move the read: {before:.2} -> {after:.2}");
    }
}

