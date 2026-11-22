//! Leaders out of followers (#1268): a leader is a member who has made
//! another. Every join is credited to whoever brought them in, and a
//! member who's brought someone can go out and organize on their own.

use crate::engine::ecs::{Entity, World};
use crate::sim::community::Organization;

/// A tie this warm to the recruit, from a member, is what brought them in
pub const BROUGHT_TIE: f32 = 0.4;
/// Days a leader sent out takes to hold their one-on-one
pub const REACH_DAYS: u64 = 3;

/// Who developed this member toward the cause
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Developed { pub org: Entity, pub by: Entity }

/// A leader sent out to hold a one-on-one without the organizer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reach { pub org: Entity, pub target: Entity, pub due: u64 }

/// The ladder a follower climbs: helpless -> member -> leader
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rung { Helpless, Member, Leader }

pub fn tie(world: &World, from: Entity, to: Entity) -> f32 {
    crate::game::relationships::find_relationship(world, from, to)
        .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0)
}

/// Credit a new member to whoever brought them: the living member closest
/// to them, if any's close enough, else the org's leader. A member already
/// credited keeps their first developer.
pub fn credit(world: &mut World, org: Entity, recruit: Entity) {
    if world.get_component::<Developed>(recruit).is_some() { return; }
    let Some(o) = world.get_component::<Organization>(org) else { return };
    let mut by = o.leader;
    let mut best = BROUGHT_TIE;
    for &m in &o.members {
        if m == recruit || crate::game::villager::is_dead(world, m) { continue; }
        let t = tie(world, m, recruit);
        if t >= best { best = t; by = m; }
    }
    if by != recruit { world.add_component(recruit, Developed { org, by }); }
}

/// The living members `who` brought into `org`
pub fn brought(world: &World, org: Entity, who: Entity) -> usize {
    world.get_component::<Organization>(org).map(|o| o.members.iter()
        .filter(|&&m| m != who && !crate::game::villager::is_dead(world, m)
            && world.get_component::<Developed>(m).map(|d| d.org == org && d.by == who).unwrap_or(false))
        .count()).unwrap_or(0)
}

pub fn rung(world: &World, org: Entity, who: Entity) -> Rung {
    let member = world.get_component::<Organization>(org).map(|o| o.members.contains(&who)).unwrap_or(false);
    if !member { Rung::Helpless }
    else if brought(world, org, who) > 0 { Rung::Leader }
    else { Rung::Member }
}

/// Members grown into leaders: the organizer's own count doesn't figure
pub fn leaders_grown(world: &World, org: Entity) -> usize {
    let Some(o) = world.get_component::<Organization>(org) else { return 0 };
    o.members.iter().filter(|&&m| m != o.leader && !crate::game::villager::is_dead(world, m)
        && rung(world, org, m) == Rung::Leader).count()
}

/// "helpless -> member -> LEADER", the rung reached in capitals
pub fn ladder_line(r: Rung) -> String {
    [(Rung::Helpless, "helpless"), (Rung::Member, "member"), (Rung::Leader, "leader")].iter()
        .map(|&(k, w)| if k == r { w.to_uppercase() } else { w.to_string() })
        .collect::<Vec<_>>().join(" -> ")
}

/// The leader's own ask: the non-member among `candidates` they're closest to
pub fn reach_target(world: &World, org: Entity, leader: Entity, candidates: &[Entity]) -> Option<Entity> {
    let o = world.get_component::<Organization>(org)?;
    let mut best: Option<(Entity, f32)> = None;
    for &c in candidates {
        if c == leader || o.members.contains(&c) || c == o.leader
            || crate::game::dungeon::unavailable(world, c)
            || world.get_component::<crate::game::vocation::Kid>(c).is_some()
            || world.get_component::<crate::session::country::Outlander>(c).is_some()
            || world.get_component::<crate::game::prophetic::Follower>(c).is_some() { continue; }
        let t = tie(world, leader, c);
        if best.map(|(_, b)| t > b).unwrap_or(true) { best = Some((c, t)); }
    }
    best.map(|(c, _)| c)
}

/// What came of a leader's one-on-one
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReachOutcome { Joined, Heard }

/// Hold a due one-on-one: the leader talks with their target, and a
/// warm enough tie brings them in, credited to the leader.
pub fn run_reach(world: &mut World, leader: Entity, day: u64) -> Option<(Entity, ReachOutcome)> {
    let r = *world.get_component::<Reach>(leader)?;
    if r.due > day { return None; }
    world.remove_component::<Reach>(leader);
    // dead, jailed or held at the castle: the sit-down never happens
    if crate::game::dungeon::unavailable(world, leader) || crate::game::dungeon::unavailable(world, r.target) { return None; }
    crate::game::organizing::one_on_one(world, leader, r.target, day);
    let t = tie(world, leader, r.target);
    // a first developer, in any org, keeps the credit
    let credited = world.get_component::<Developed>(r.target).is_some();
    if crate::sim::community::recruit(world, r.org, r.target, t) {
        if !credited { world.add_component(r.target, Developed { org: r.org, by: leader }); }
        Some((r.target, ReachOutcome::Joined))
    } else { Some((r.target, ReachOutcome::Heard)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_line_caps_the_rung() {
        assert_eq!(ladder_line(Rung::Member), "helpless -> MEMBER -> leader");
    }
}
