//! Weather events: storms, droughts, heat waves affect food/health.
//! Emergent effects - weather + climate produce real consequences.

use crate::engine::ecs::{Entity, World};

/// A weather event (shorter-term than climate)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Weather {
    Clear,
    Storm,      // damages crops, stops travel
    Drought,    // reduces food growth
    HeatWave,   // affects health, speeds spoilage
    Frost,      // kills crops, cold damage
}

/// Active weather
#[derive(Debug)]
pub struct CurrentWeather {
    pub kind: Weather,
    pub intensity: f32,  // 0-1 severity
    pub duration: u32,   // ticks remaining
}

/// Weather effect on food production
pub fn food_effect(w: &CurrentWeather) -> f32 {
    match w.kind {
        Weather::Clear => 1.0,
        Weather::Storm => 1.0 - (w.intensity * 0.5),
        Weather::Drought => 1.0 - (w.intensity * 0.7),
        Weather::HeatWave => 1.0 - (w.intensity * 0.3),
        Weather::Frost => 1.0 - (w.intensity * 0.6),
    }
}

/// Weather effect on health
pub fn health_effect(w: &CurrentWeather) -> f32 {
    match w.kind {
        Weather::HeatWave | Weather::Frost => -(w.intensity * 0.1),
        _ => 0.0,
    }
}

/// Spawn weather event
pub fn spawn_weather(world: &mut World, kind: Weather, intensity: f32, duration: u32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, CurrentWeather { kind, intensity, duration });
    e
}

/// System: weather evolves, duration counts down
pub struct WeatherSystem;
impl crate::engine::ecs::System for WeatherSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<CurrentWeather>() {
            if let Some(w) = world.get_component_mut::<CurrentWeather>(e) {
                w.duration = w.duration.saturating_sub(1);
                if w.duration == 0 {
                    w.kind = Weather::Clear;
                }
            }
        }
    }
}
