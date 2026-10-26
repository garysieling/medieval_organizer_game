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

/// Step the epidemic: spread disease to susceptibles, recover infected,
/// and apply mortality to those who don't recover. Returns the SIR tally
/// reflecting the state *after* this tick's changes.
pub fn step_epidemic(world: &mut World, pop: &[Entity], ep: &mut Epidemic) -> SirState {
    // Spread: susceptibles get infected proportional to rate
    let susceptible_before = pop.iter()
        .filter(|e| world.get_component::<Health>(**e).map(|h| h.condition >= 0.9).unwrap_or(false))
        .count();
    let new_infections = (susceptible_before as f32 * ep.infection_rate * 0.3) as usize;
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
    for e in pop {
        if let Some(h) = world.get_component_mut::<Health>(*e) {
            if h.condition > 0.0 && h.condition < 0.5 {
                h.condition = (h.condition + ep.recovery_rate).min(1.0);
            }
        }
    }
    // Mortality: a share of those who are still infected after recovery succumb
    let still_infected = pop.iter()
        .filter(|e| world.get_component::<Health>(**e).map(|h| h.condition > 0.0 && h.condition < 0.5).unwrap_or(false))
        .count();
    let new_deaths = (still_infected as f32 * ep.mortality).round() as usize;
    let mut death_count = 0;
    for e in pop {
        if death_count >= new_deaths { break; }
        if let Some(h) = world.get_component_mut::<Health>(*e) {
            if h.condition > 0.0 && h.condition < 0.5 {
                h.condition = 0.0;
                death_count += 1;
            }
        }
    }
    ep.tick += 1;

    // Tally the compartment counts after this tick's changes
    let mut sir = SirState { susceptible: 0, infected: 0, recovered: 0, dead: 0 };
    for e in pop {
        match world.get_component::<Health>(*e) {
            Some(h) if h.condition <= 0.0 => sir.dead += 1,
            Some(h) if h.condition < 0.5 => sir.infected += 1,
            Some(h) if h.condition < 0.9 => sir.recovered += 1,
            Some(_) => sir.susceptible += 1,
            None => sir.dead += 1,
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn infected_pop(world: &mut World, n: usize) -> Vec<Entity> {
        (0..n).map(|_| {
            let e = world.create_entity();
            world.add_component(e, Health { condition: 0.3, immunity: 0.5 });
            e
        }).collect()
    }

    #[test]
    fn mortality_kills_infected_who_dont_recover() {
        let mut world = World::new();
        let pop = infected_pop(&mut world, 4);
        let mut ep = Epidemic { name: "plague".into(), infection_rate: 0.0, recovery_rate: 0.0, mortality: 1.0, tick: 0 };

        let sir = step_epidemic(&mut world, &pop, &mut ep);

        assert_eq!(sir.dead, 4, "all infected should die when mortality is 1.0 and nobody recovers");
        assert_eq!(sir.infected, 0);
    }

    #[test]
    fn recovery_prevents_mortality_same_tick() {
        let mut world = World::new();
        let pop = infected_pop(&mut world, 4);
        // recovery pushes condition from 0.3 to 1.0, clearing infection before mortality is applied
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.0, recovery_rate: 0.7, mortality: 1.0, tick: 0 };

        let sir = step_epidemic(&mut world, &pop, &mut ep);

        assert_eq!(sir.dead, 0, "recovered villagers should not also be counted as dead");
        assert_eq!(sir.susceptible, 4);
    }

    #[test]
    fn returned_tally_reflects_this_ticks_changes() {
        // Regression test: step_epidemic used to return a SIR snapshot taken
        // BEFORE this tick's infection/recovery/mortality were applied.
        let mut world = World::new();
        let pop = infected_pop(&mut world, 4);
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.0, recovery_rate: 0.7, mortality: 0.0, tick: 0 };

        let sir = step_epidemic(&mut world, &pop, &mut ep);

        // Before the fix this would report infected: 4, recovered: 0 (the pre-tick state)
        assert_eq!(sir.infected, 0);
        assert_eq!(sir.susceptible, 4);
    }
}
