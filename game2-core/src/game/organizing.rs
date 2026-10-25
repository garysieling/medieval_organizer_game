//! 1-1 organizing meetings + house meetings.
//! From lit review: relational interview reveals concerns/networks/leadership -> recruit.

use crate::engine::ecs::{Entity, World};
use crate::sim::community::{Citizen, Organization};

/// What a 1-1 reveals about a citizen
#[derive(Debug)]
pub struct OneOnOneResult {
    pub concerns: Vec<String>,       // what they care about
    pub networks: Vec<String>,       // who they know
    pub leadership_potential: f32,   // 0-1
    pub trust_gained: f32,
}

/// Conduct a 1-1 relational meeting (from NEA organizing guide)
pub fn one_on_one(world: &World, organizer: Entity, citizen: Entity) -> OneOnOneResult {
    let c = world.get_component::<Citizen>(citizen);
    OneOnOneResult {
        concerns: vec![format!("worried about {}", c.map(|x| x.ideology.as_str()).unwrap_or("work"))],
        networks: vec!["neighbors".into(), "workmates".into()],
        leadership_potential: c.map(|x| x.influence * 1.5).unwrap_or(0.3).min(1.0),
        trust_gained: 0.2,
    }
}

/// A house meeting: small group, shared agenda emerges
#[derive(Debug)]
pub struct HouseMeetingResult {
    pub shared_concerns: Vec<String>,
    pub formed_committee: bool,
    pub attendees_recruited: u32,
}

/// Run a house meeting with identified citizens
pub fn house_meeting(world: &mut World, org: Entity, attendees: &[Entity]) -> HouseMeetingResult {
    // Shared concerns emerge from overlap
    let shared = vec!["fair wages".into(), "safe harvest".into()];
    let mut recruited = 0;
    for a in attendees {
        if crate::sim::community::recruit(world, org, *a, 0.6) {
            recruited += 1;
        }
    }
    HouseMeetingResult {
        shared_concerns: shared,
        formed_committee: recruited >= 2,
        attendees_recruited: recruited,
    }
}
