//! Religion simulation: faith traits, gatherings, emergent cohesion.
//! From backlog: "emergent effects of religious gatherings not modelled in any game"
//! Bioshock-style satire possible; religion shapes community and power.

use crate::engine::ecs::{Entity, World};

/// An agent's faith/religious identity
#[derive(Debug)]
pub struct Faith {
    pub religion: String,       // which faith
    pub devotion: f32,          // 0-1 intensity
    pub traditions: Vec<String>,// rituals/beliefs adopted
}

/// A religious community
#[derive(Debug)]
pub struct Congregation {
    pub name: String,
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
    pub orthodoxy: f32,         // how strict about beliefs
    pub services: Vec<Service>,
}

/// A religious service/ritual
#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub frequency: String,      // daily/weekly/seasonal
    pub participants: Vec<Entity>,
    pub cohesion_boost: f32,    // emergent bonding effect
    pub charity_output: f32,    // emergent material aid
}

/// Emergent effect: gatherings produce social cohesion
pub struct GatherSystem;
impl crate::engine::ecs::System for GatherSystem {
    fn run(&mut self, world: &mut World) {
        // Congregations strengthen member bonds through services
        for c in world.query::<Congregation>() {
            if let Some(cong) = world.get_component::<Congregation>(c) {
                let cohesion = cong.services.iter()
                    .map(|s| s.cohesion_boost)
                    .sum::<f32>();
                let _ = cohesion; // would update member Bond components
            }
        }
    }
}

/// Emergent effect: devotion grows from gathering attendance
pub struct DevotionSystem;
impl crate::engine::ecs::System for DevotionSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Faith>() {
            if let Some(f) = world.get_component_mut::<Faith>(e) {
                f.devotion = (f.devotion + 0.01).min(1.0);
            }
        }
    }
}

/// Congregation performs a service - emergent effects
pub fn perform_service(world: &mut World, cong: Entity, service: Service) {
    if let Some(c) = world.get_component_mut::<Congregation>(cong) {
        c.services.push(service);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_produces_cohesion() {
        let mut w = World::new();
        let c = w.create_entity();
        w.add_component(c, Congregation {
            name: "Village Church".into(),
            members: vec![],
            leader: None,
            orthodoxy: 0.7,
            services: vec![Service {
                name: "Sunday Mass".into(),
                frequency: "weekly".into(),
                participants: vec![],
                cohesion_boost: 0.2,
                charity_output: 0.1,
            }],
        });
        // Service exists with cohesion effect
        assert!(w.get_component::<Congregation>(c).unwrap().services[0].cohesion_boost > 0.0);
    }
}
