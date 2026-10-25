//! Epidemics: disease spreads through population, peaks, burns out.
//! SIR-style: Susceptible -> Infected -> Recovered.

use crate::engine::ecs::{Entity, World};
use super::illness::Health;

/// Epidemic state for a population
#[derive(Debug)]
pub struct Epidemic {
    pub name: String,
    pub infection_rate: f32,    // how fast it spreads
    pub recovery_rate: f32,
    pub mortality: f32,
    pub tick: u64,
}

/// SIR compartment counts for the population
#[derive(Debug)]
pub struct SirState {
    pub susceptible: usize,
    pub infected: usize,
    pub recovered: usize,
    pub dead: usize,
}

/// Step the epidemic: spread disease to susceptibles, recover infected
pub fn step_epidemic(world: &mut World, pop: &[Entity], ep: &mut Epidemic) -> SirState {
    let mut sir = SirState { susceptible: 0, infected: 0, recovered: 0, dead: 0 };
    // Count current states
    for e in pop {
        if let Some(h) = world.get_component::<Health>(*e) {
            if h.condition < 0.5 { sir.infected += 1; }
            else if h.condition < 0.9 { sir.recovered += 1; }
            else { sir.susceptible += 1; }
        }
    }
    // Spread: susceptibles get infected proportional to rate
    let new_infections = (sir.susceptible as f32 * ep.infection_rate * 0.3) as usize;
    let mut infected_count = 0;
    for e in pop {
        if infected_count >= new_infections { break; }
        if let Some(h) = world.get_component_mut::<Health>(*e) {
            if h.condition >= 0.9 {
                h.condition = 0.4; // becomes infected
                infected_count += 1;
            }
        }
    }
    // Recovery: infected recover proportional to rate
    let mut recovered_count = 0;
    for e in pop {
        if let Some(h) = world.get_component_mut::<Health>(*e) {
            if h.condition < 0.5 {
                h.condition = (h.condition + ep.recovery_rate).min(1.0);
                if h.condition >= 0.9 { recovered_count += 1; }
            }
        }
    }
    ep.tick += 1;
    sir
}

/// Spawn an epidemic
pub fn spawn_epidemic(world: &mut World, name: &str, infection_rate: f32, recovery_rate: f32, mortality: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Epidemic {
        name: name.into(), infection_rate, recovery_rate, mortality, tick: 0,
    });
    e
}
