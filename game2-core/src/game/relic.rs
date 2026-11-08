use crate::engine::ecs::*;

/// A saint's bones in a box: prestige and legitimacy for whoever holds
/// it, and a story the theft has to carry - furta sacra only moves the
/// holiness if the saint consents
#[derive(Debug, Clone)]
pub struct Relic {
    pub name: String,
    pub prestige: f32,      // the pull it gives the holder
    pub legitimacy: f32,    // the right it lends the holder's rule
    pub revenue: f32,       // what the pilgrims leave behind per visit
    pub exposed: bool,      // a denounced fake carries nothing
}

impl Relic {
    pub fn new(name: &str, prestige: f32, legitimacy: f32) -> Self {
        Relic { name: name.into(), prestige, legitimacy,
            revenue: prestige * 0.2, exposed: false }
    }
}

/// The story a theft has to carry: the relic only moves its holiness
/// when the saint's said to consent (the roll) or the old regime's
/// illegitimate anyway - else it's a sack of bones
pub fn raid_steals(world: &mut World, from: Entity, to: Entity, roll: u32) -> bool {
    let Some(r) = world.get_component::<Relic>(from).cloned() else { return false };
    // one in three the story holds - the saint consents
    let holds = roll % 3 == 0;
    let mut carried = r.clone();
    if !holds { carried.prestige = 0.0; carried.legitimacy = 0.0; carried.exposed = true; }
    world.add_component(to, carried);
    world.remove_component::<Relic>(from);
    holds
}

/// The exposé: a denounced relic is a fake everywhere - the market's
/// faith in all of them crashes at once (the Reformation scenario)
pub fn expose_all(world: &mut World) {
    for e in world.query::<Relic>() {
        if let Some(r) = world.get_component_mut::<Relic>(e) {
            r.exposed = true; r.prestige *= 0.2; r.revenue *= 0.2;
        }
    }
}
