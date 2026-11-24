//! Inoculation (#1246): after someone signs on, the organizer names the
//! lord's likely counter and rehearses the answer with them, so the blow
//! lands on a prepared member. Each shield guards against one kind of
//! counter, shields stack, and the first blow of that kind uses it up.

use crate::engine::ecs::{Entity, World};

/// The counter-moves the lord's side plays against a member
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Counter {
    /// The steward's threat: the de-oathing, hands loosened by fear
    Threat,
    /// The soft plea: the lord's grace, an amnesty for whoever walks away
    Plea,
    /// The denouncement: named from the pulpit or the bench
    Denounce,
}

pub const COUNTERS: [Counter; 3] = [Counter::Threat, Counter::Plea, Counter::Denounce];

/// The counters a member's been readied for
#[derive(Debug, Clone, Default)]
pub struct Inoculated {
    pub against: Vec<Counter>,
}

/// The choice that names it
pub fn label(c: Counter) -> &'static str {
    match c {
        Counter::Threat => "Name the steward's threat",
        Counter::Plea => "Name the lord's soft plea",
        Counter::Denounce => "Name the denouncement",
    }
}

/// Their fear, said aloud when it's named
pub fn fear(c: Counter) -> &'static str {
    match c {
        Counter::Threat => "And when the steward comes to my door and says he'll have my strip?",
        Counter::Plea => "And if the lord says all's forgiven, just walk away from {2p:obj} lot?",
        Counter::Denounce => "And if Father stands up on Sunday and names me for it?",
    }
}

/// The organizer's answer, the rehearsal of it
pub fn answer(c: Counter) -> &'static str {
    match c {
        Counter::Threat => "Then he's afraid - a man sure of his land doesn't need to threaten it. {2:subj} tell him {2:subj} {2:will} speak with the others first.",
        Counter::Plea => "His grace comes the day we're strong, and leaves the day we're not. {2:subj} {2:will} know it for what it is.",
        Counter::Denounce => "They name {2:obj} because {2:subj} matter. {2:subj} {2:will} stand there with us beside {2:obj} - and {2:subj} {2:will} not be alone in that pew.",
    }
}

/// Signed on: a member of the cause, or holding a part at its meeting
pub fn committed(world: &World, org: Entity, who: Entity) -> bool {
    let Some(o) = world.get_component::<crate::sim::community::Organization>(org) else { return false };
    o.members.contains(&who)
        || world.get_component::<crate::game::organizing::MeetingPlan>(org)
            .map(|p| p.holders().contains(&who)).unwrap_or(false)
}

pub fn shielded(world: &World, who: Entity, c: Counter) -> bool {
    world.get_component::<Inoculated>(who).map(|i| i.against.contains(&c)).unwrap_or(false)
}

/// Ready them against `c`; false when they already were
pub fn shield(world: &mut World, who: Entity, c: Counter) -> bool {
    if shielded(world, who, c) { return false; }
    if world.get_component::<Inoculated>(who).is_none() {
        world.add_component(who, Inoculated::default());
    }
    if let Some(i) = world.get_component_mut::<Inoculated>(who) { i.against.push(c); }
    true
}

/// A counter of kind `c` lands on them: true if they were ready for it,
/// which spends the shield
pub fn take(world: &mut World, who: Entity, c: Counter) -> bool {
    let Some(i) = world.get_component_mut::<Inoculated>(who) else { return false };
    let Some(k) = i.against.iter().position(|&x| x == c) else { return false };
    i.against.remove(k);
    true
}
