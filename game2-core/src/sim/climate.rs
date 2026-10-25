//! Climate change threat system: seasonal cycles + long-term drift.
//! From backlog: climate change as a "you versus the world" threat.
//! Emergent: seasonal pressure + slow climate drift affect resources.

use crate::engine::ecs::{Entity, World};

/// Current climate state
#[derive(Debug)]
pub struct Climate {
    pub season: Season,
    pub year: u32,
    pub temperature_drift: f32, // long-term warming/cooling
    pub rainfall_drift: f32,    // long-term wet/dry trend
    pub severity: f32,          // 0-1, how harsh the climate is
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Climate's effect on a resource (food, water, etc.)
#[derive(Debug)]
pub struct ClimateEffect {
    pub resource: String,
    pub multiplier: f32, // <1 = scarcer, >1 = abundant
}

/// Weather event (storms, droughts, freezes)
#[derive(Debug)]
pub struct WeatherEvent {
    pub name: String,
    pub severity: f32,
    pub duration: u32,
    pub affected: String, // resource or region
}

/// System: advance seasons each tick (4 ticks = 1 year)
pub struct SeasonSystem;
impl crate::engine::ecs::System for SeasonSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Climate>() {
            if let Some(c) = world.get_component_mut::<Climate>(e) {
                c.season = match c.season {
                    Season::Spring => Season::Summer,
                    Season::Summer => Season::Autumn,
                    Season::Autumn => Season::Winter,
                    Season::Winter => { c.year += 1; Season::Spring }
                };
                // Climate drifts over time (the threat)
                c.temperature_drift += 0.001;
                c.severity = (c.temperature_drift * 0.1).min(1.0);
            }
        }
    }
}

/// Climate affects resource availability (emergent scarcity)
pub fn climate_multiplier(c: &Climate, resource: &str) -> f32 {
    let base = match c.season {
        Season::Spring => 1.1,
        Season::Summer => 1.0,
        Season::Autumn => 0.9,
        Season::Winter => 0.7,
    };
    base * (1.0 - c.severity * 0.5)
}

/// Spawn climate entity
pub fn spawn_climate(world: &mut World) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Climate {
        season: Season::Spring,
        year: 1,
        temperature_drift: 0.0,
        rainfall_drift: 0.0,
        severity: 0.0,
    });
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seasons_cycle() {
        let mut w = World::new();
        let c = spawn_climate(&mut w);
        let mut s = crate::engine::ecs::Scheduler::new();
        s.add_system(SeasonSystem);
        s.tick(&mut w);
        assert_eq!(w.get_component::<Climate>(c).unwrap().season, Season::Summer);
        s.tick(&mut w); s.tick(&mut w); s.tick(&mut w);
        assert_eq!(w.get_component::<Climate>(c).unwrap().year, 2); // year advanced
    }

    #[test]
    fn test_climate_scarcity() {
        let c = Climate { season: Season::Winter, year: 1, temperature_drift: 0.5, rainfall_drift: 0.0, severity: 0.8 };
        assert!(climate_multiplier(&c, "food") < 0.5); // winter + severe = scarce
    }
}
