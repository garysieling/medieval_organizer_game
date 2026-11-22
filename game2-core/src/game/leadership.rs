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

/// The work of a meeting somebody can watch or hold (#1345): the five
/// promised parts of a house meeting, then the three campaign seats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Job { Snacks, Notes, Opener, Song, Closer, Turnout, Minutes, Chair }

impl Job {
    pub const ALL: [Job; 8] = [Job::Snacks, Job::Notes, Job::Opener, Job::Song,
        Job::Closer, Job::Turnout, Job::Minutes, Job::Chair];
    pub fn ix(self) -> usize { self as usize }
    /// The job's name as the card reads it
    pub fn name(self) -> &'static str {
        ["the table", "the word", "the opener", "the song", "the send-off",
            "the turnout", "the minutes", "the chair"][self.ix()]
    }
}

/// The meeting jobs a member has watched (seen) and held (done) (#1345):
/// the row of yellows turning green on their card - done implies seen
#[derive(Debug, Default)]
pub struct JobMarks {
    pub seen: [u16; 8],
    pub done: [u16; 8],
}

/// `who` was in the room while `job` was worked
pub fn mark_observed(world: &mut World, who: Entity, job: Job) {
    if world.get_component::<JobMarks>(who).is_none() {
        world.add_component(who, JobMarks::default());
    }
    if let Some(m) = world.get_component_mut::<JobMarks>(who) {
        m.seen[job.ix()] = m.seen[job.ix()].saturating_add(1);
    }
}

/// `who` held `job` themselves - holding it implies having seen it
pub fn mark_performed(world: &mut World, who: Entity, job: Job) {
    mark_observed(world, who, job);
    if let Some(m) = world.get_component_mut::<JobMarks>(who) {
        m.done[job.ix()] = m.done[job.ix()].saturating_add(1);
    }
}

/// How much of a leader the marks make them (#1345): a job held weighs
/// double a job only watched. The points explain the ladder, they don't
/// climb it - `Rung` still needs somebody brought in
pub fn job_score(world: &World, who: Entity) -> f32 {
    world.get_component::<JobMarks>(who)
        .map(|m| m.seen.iter().zip(&m.done)
            .map(|(&s, &d)| (s + 2 * d) as f32).sum())
        .unwrap_or(0.0)
}

/// How much the marks steady a leader's own sit-down ask (#1345),
/// per point of job score up to this cap
pub const MARK_REACH_BONUS: f32 = 0.02;
pub const MARK_BONUS_CAP: f32 = 0.2;
/// What the marks add to the tie a reach already rests on
pub fn reach_bonus(world: &World, leader: Entity) -> f32 {
    (job_score(world, leader) * MARK_REACH_BONUS).min(MARK_BONUS_CAP)
}

/// The score at which a member starts holding their own one-on-ones
/// without being sent (#1345)
pub const SELF_START_SCORE: f32 = 8.0;
/// Days a self-starter rests between their own reaches
pub const SELF_START_GAP: u64 = 20;
/// When a self-starter may take their next own reach (#1345)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OwnReach { pub next: u64 }

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
    // the marks steady the ask: a leader who's held the work convinces
    // a little surer than one who's only been sent (#1345)
    let t = tie(world, leader, r.target) + reach_bonus(world, leader);
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
