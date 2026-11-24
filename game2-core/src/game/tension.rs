//! Tension between what a soul does and what they hold (E8-4, #1454):
//! the pull of behavior against belief, identity, the issue they
//! carry, or the house that feeds them. A directional act feeds the
//! strain on its axis, a consonant one bleeds it, and time softens
//! what the day didn't pull. Past `FLIP_AT` the pull wins and the
//! softer side gives - the session's sites resolve it as a flip: out
//! of the cause, or into it, or out from under the issue.
//!
//! The shape mirrors the ghost's `tested_by` (E8-2): the same "the
//! thing they hold loses ground until the person turns" mechanism,
//! generalized off the wound to the whole self.

use crate::engine::det::HashMap;
use crate::engine::ecs::{Entity, World};

/// The four strains the day can pull on - the axes of #1454. What
/// they DO can fight what they BELIEVE (the cause's work their ghost
/// calls futile), who they ARE (rebel work under an establishment
/// self), the ISSUE they carry (the cause lifts it while they watch
/// from outside), or their own INTEREST (the org's asks over a bare
/// house). Each accrues its own count; the strongest names the flip
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    Belief,
    Identity,
    Issue,
    Interest,
}

/// Every strain, in a fixed order - the daily ease walks them all
pub const AXES: [Axis; 4] = [Axis::Belief, Axis::Identity, Axis::Issue, Axis::Interest];

/// How far along each strain has pulled - 0 at rest, `FLIP_AT` when
/// it gives. One axis resolves at a time: the loudest gives first
#[derive(Debug, Default, Clone)]
pub struct Tension {
    pub scores: HashMap<Axis, f32>,
}

/// The strength at which the pull wins and the softer side yields
pub const FLIP_AT: f32 = 0.5;

/// The day's softening: a day that didn't pull on a strain bleeds it
/// a little - consonant living wears the edge off
pub const DAILY_EASE: f32 = 0.01;

fn press(world: &mut World, v: Entity, f: impl FnOnce(&mut Tension)) {
    let mut t = world.get_component::<Tension>(v).cloned().unwrap_or_default();
    f(&mut t);
    world.add_component(v, t);
}

/// `v`'s behavior pulled against `axis` by `w` - the strain accrues,
/// clamped so a pile of dissonant days can't overshoot the flip
pub fn strained(world: &mut World, v: Entity, axis: Axis, w: f32) {
    press(world, v, |t| {
        *t.scores.entry(axis).or_insert(0.0) =
            (t.scores.get(&axis).copied().unwrap_or(0.0) + w).min(1.0);
    });
}

/// `v` acted with `axis`, not against it - the strain bleeds off,
/// floored at rest
pub fn eased(world: &mut World, v: Entity, axis: Axis, w: f32) {
    press(world, v, |t| {
        if let Some(s) = t.scores.get_mut(&axis) { *s = (*s - w).max(0.0); }
    });
}

/// The day's bleed: every strain softens by `DAILY_EASE` - what the
/// day didn't pull on, time wears down
pub fn rest(world: &mut World, v: Entity) {
    press(world, v, |t| {
        for s in t.scores.values_mut() { *s = (*s - DAILY_EASE).max(0.0); }
    });
}

/// The strongest strain past `FLIP_AT`, if one has given - the caller
/// resolves it as the axis's flip and then calls `resolved`
pub fn giving(world: &World, v: Entity) -> Option<Axis> {
    world.get_component::<Tension>(v).and_then(|t| {
        AXES.iter().copied()
            .filter(|&a| t.scores.get(&a).copied().unwrap_or(0.0) >= FLIP_AT)
            .max_by(|&a, &b| {
                let (x, y) = (t.scores.get(&a).copied().unwrap_or(0.0),
                              t.scores.get(&b).copied().unwrap_or(0.0));
                x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal)
            })
    })
}

/// The flip landed: the axis rests back at zero - the pull is spent,
/// and building again means living against the self again
pub fn resolved(world: &mut World, v: Entity, axis: Axis) {
    press(world, v, |t| { t.scores.insert(axis, 0.0); });
}

/// The current pull on an axis - for the report and the tests
pub fn strain_of(world: &World, v: Entity, axis: Axis) -> f32 {
    world.get_component::<Tension>(v)
        .and_then(|t| t.scores.get(&axis).copied()).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dissonant_streak_grows_then_gives() {
        let mut w = World::new();
        let v = w.create_entity();
        for _ in 0..6 {
            strained(&mut w, v, Axis::Belief, 0.08);
            assert!(giving(&w, v).is_none());
        }
        strained(&mut w, v, Axis::Belief, 0.08);
        assert_eq!(giving(&w, v), Some(Axis::Belief));
    }

    #[test]
    fn consonant_days_bleed_the_strain() {
        let mut w = World::new();
        let v = w.create_entity();
        strained(&mut w, v, Axis::Interest, 0.10);
        eased(&mut w, v, Axis::Interest, 0.04);
        assert!((strain_of(&w, v, Axis::Interest) - 0.06).abs() < 1e-6);
        eased(&mut w, v, Axis::Interest, 1.0);
        assert_eq!(strain_of(&w, v, Axis::Interest), 0.0, "ease can't go under rest");
    }

    #[test]
    fn rest_softens_what_the_day_didnt_pull() {
        let mut w = World::new();
        let v = w.create_entity();
        strained(&mut w, v, Axis::Identity, 0.05);
        rest(&mut w, v);
        assert!((strain_of(&w, v, Axis::Identity) - 0.04).abs() < 1e-6);
        for _ in 0..10 { rest(&mut w, v); }
        assert_eq!(strain_of(&w, v, Axis::Identity), 0.0, "rest floors at zero");
    }

    #[test]
    fn the_loudest_axis_names_the_flip() {
        let mut w = World::new();
        let v = w.create_entity();
        strained(&mut w, v, Axis::Belief, FLIP_AT);
        strained(&mut w, v, Axis::Issue, FLIP_AT + 0.1);
        assert_eq!(giving(&w, v), Some(Axis::Issue));
        resolved(&mut w, v, Axis::Issue);
        assert_eq!(giving(&w, v), Some(Axis::Belief),
            "the quieter strain waits its turn");
        resolved(&mut w, v, Axis::Belief);
        assert_eq!(giving(&w, v), None);
    }

    #[test]
    fn a_soul_at_rest_carries_no_tension() {
        let mut w = World::new();
        let v = w.create_entity();
        assert_eq!(giving(&w, v), None);
        rest(&mut w, v);
        assert_eq!(strain_of(&w, v, Axis::Belief), 0.0);
    }
}
