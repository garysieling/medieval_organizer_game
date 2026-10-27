//! Climate change threat system: seasonal cycles + long-term drift.
//! From backlog: climate change as a "you versus the world" threat.
//! Emergent: seasonal pressure + slow climate drift affect resources.

use crate::engine::ecs::{Entity, World};

/// Current climate state
#[derive(Debug)]
pub struct Climate {
    pub season: Season,
    pub year: u32,
    pub day: u32,               // day of the year (tick counter)
    pub temperature_drift: f32, // long-term warming/cooling
    pub rainfall_drift: f32,    // how wet the weather is right now, 0-1; above RAIN_THRESHOLD it's raining
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

/// System: advance one day per tick (90 days per season, 360 per year)
pub struct SeasonSystem;
impl crate::engine::ecs::System for SeasonSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Climate>() {
            if let Some(c) = world.get_component_mut::<Climate>(e) {
                c.day += 1;
                // season changes every 90 days (ticks)
                if c.day % 90 == 0 {
                    c.season = match c.season {
                        Season::Spring => Season::Summer,
                        Season::Summer => Season::Autumn,
                        Season::Autumn => Season::Winter,
                        Season::Winter => { c.year += 1; Season::Spring }
                    };
                }
                c.rainfall_drift = wetness(c.season, c.day);
                // Climate drifts over time (the threat)
                c.temperature_drift += 0.001;
                c.severity = (c.temperature_drift * 0.1).min(1.0);
            }
        }
    }
}

/// Wetness above which it's raining
pub const RAIN_THRESHOLD: f32 = 0.5;

/// Typical wetness of each season (spring and autumn wet, summer dry)
fn season_wetness(s: Season) -> f32 {
    match s {
        Season::Spring => 0.45,
        Season::Summer => 0.25,
        Season::Autumn => 0.4,
        Season::Winter => 0.3,
    }
}

/// Today's wetness: the season's typical level plus wet and dry spells
/// lasting several days (two slow overlapping cycles, so spells vary in
/// length but the same day always has the same weather)
pub fn wetness(season: Season, day: u32) -> f32 {
    use std::f32::consts::TAU;
    let d = day as f32;
    let spells = 0.2 * (TAU * d / 11.0).sin() + 0.1 * (TAU * d / 4.3).sin();
    (season_wetness(season) + spells).clamp(0.0, 1.0)
}

/// Whether it's raining today
pub fn is_raining(c: &Climate) -> bool {
    c.rainfall_drift > RAIN_THRESHOLD
}

/// Climate affects resource availability (emergent scarcity)
pub fn climate_multiplier(c: &Climate, resource: &str) -> f32 {
    let base = match c.season {
        Season::Spring => 1.1,
        Season::Summer => 1.0,
        Season::Autumn => 0.9,
        Season::Winter => 0.7,
    };
    let weather = if resource == "food" { rain_effect(c) } else { 1.0 };
    base * weather * (1.0 - c.severity * 0.5)
}

/// How much a wet or dry spell changes crop growth per unit of wetness
/// away from the season's norm
pub const RAIN_SENSITIVITY: f32 = 0.5;

/// Crops grow faster in a wet spell and slower in a dry one. Measured
/// against the season's typical wetness, so it averages out over a season:
/// the seasonal base above already covers which seasons are lean
pub fn rain_effect(c: &Climate) -> f32 {
    1.0 + RAIN_SENSITIVITY * (c.rainfall_drift - season_wetness(c.season))
}

/// Spawn climate entity
pub fn spawn_climate(world: &mut World) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Climate {
        season: Season::Spring,
        day: 0,
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
        for _ in 0..89 { s.tick(&mut w); }
        assert_eq!(w.get_component::<Climate>(c).unwrap().season, Season::Spring);
        s.tick(&mut w);
        assert_eq!(w.get_component::<Climate>(c).unwrap().season, Season::Summer);
        for _ in 0..270 { s.tick(&mut w); }
        assert_eq!(w.get_component::<Climate>(c).unwrap().season, Season::Spring);
        assert_eq!(w.get_component::<Climate>(c).unwrap().year, 2); // year advanced
    }

    #[test]
    fn test_climate_scarcity() {
        let c = Climate { season: Season::Winter, year: 1, day: 0, temperature_drift: 0.5, rainfall_drift: 0.0, severity: 0.8 };
        assert!(climate_multiplier(&c, "food") < 0.5); // winter + severe = scarce
    }

    #[test]
    fn test_it_rains_on_some_days_and_not_others() {
        let mut w = World::new();
        let c = spawn_climate(&mut w);
        let mut s = crate::engine::ecs::Scheduler::new();
        s.add_system(SeasonSystem);
        let mut rainy = [0u32; 4]; // per season
        for _ in 0..360 {
            s.tick(&mut w);
            let cl = w.get_component::<Climate>(c).unwrap();
            if is_raining(cl) { rainy[cl.season as usize] += 1; }
        }
        let total: u32 = rainy.iter().sum();
        assert!(total > 20 && total < 180, "rain should be occasional, got {} rainy days", total);
        assert!(rainy[Season::Spring as usize] > rainy[Season::Summer as usize], "spring wetter than summer: {:?}", rainy);
    }

    #[test]
    fn test_rain_helps_crops_and_drought_hurts() {
        let mut c = Climate { season: Season::Spring, year: 1, day: 0, temperature_drift: 0.0, rainfall_drift: 0.45, severity: 0.0 };
        let normal = climate_multiplier(&c, "food");
        c.rainfall_drift = 0.75;
        let wet = climate_multiplier(&c, "food");
        c.rainfall_drift = 0.15;
        let dry = climate_multiplier(&c, "food");
        assert!(wet > normal && normal > dry, "wet {} normal {} dry {}", wet, normal, dry);
        assert_eq!(climate_multiplier(&c, "timber"), 1.1, "rain only affects food");
    }

    #[test]
    fn test_rain_effect_averages_out_over_a_season() {
        for (season, start) in [(Season::Spring, 0), (Season::Summer, 90), (Season::Autumn, 180), (Season::Winter, 270)] {
            let mean: f32 = (start..start + 90).map(|d| {
                let c = Climate { season, year: 1, day: d, temperature_drift: 0.0, rainfall_drift: wetness(season, d), severity: 0.0 };
                rain_effect(&c)
            }).sum::<f32>() / 90.0;
            assert!((mean - 1.0).abs() < 0.03, "{:?} mean rain effect {}", season, mean);
        }
    }
}
