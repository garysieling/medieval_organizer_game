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
    // Fractional deaths carried over from the previous tick, so a small
    // infected population with a sub-1-death-per-tick mortality rate still
    // accumulates deaths over time instead of always rounding to zero.
    pub mortality_carry: f32,
    // Fractional new infections carried over, for the same reason
    pub infection_carry: f32,
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
    // Spread (SIR mass action): each susceptible's chance of catching it
    // scales with the share of the living population that's infected, so
    // with no one sick nothing spreads, and more cases spread it faster
    let count = |world: &World, f: fn(f32) -> bool| pop.iter()
        .filter(|e| world.get_component::<Health>(**e).map(|h| f(h.condition)).unwrap_or(false))
        .count();
    let susceptible_before = count(world, |c| c >= 0.9);
    let infected_before = count(world, |c| c > 0.0 && c < 0.5);
    let alive = count(world, |c| c > 0.0).max(1);
    let raw_infections = susceptible_before as f32 * ep.infection_rate
        * infected_before as f32 / alive as f32 + ep.infection_carry;
    let new_infections = raw_infections.floor() as usize;
    ep.infection_carry = raw_infections - new_infections as f32;
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
    let raw_deaths = still_infected as f32 * ep.mortality + ep.mortality_carry;
    let new_deaths = raw_deaths.floor() as usize;
    ep.mortality_carry = raw_deaths - new_deaths as f32;
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
            None => {} // no Health component means never tracked, not dead
        }
    }
    sir
}

/// Spawn an epidemic
pub fn spawn_epidemic(world: &mut World, name: &str, infection_rate: f32, recovery_rate: f32, mortality: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Epidemic {
        name: name.into(), infection_rate, recovery_rate, mortality, tick: 0, mortality_carry: 0.0, infection_carry: 0.0,
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
        let mut ep = Epidemic { name: "plague".into(), infection_rate: 0.0, recovery_rate: 0.0, mortality: 1.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };

        let sir = step_epidemic(&mut world, &pop, &mut ep);

        assert_eq!(sir.dead, 4, "all infected should die when mortality is 1.0 and nobody recovers");
        assert_eq!(sir.infected, 0);
    }

    #[test]
    fn recovery_prevents_mortality_same_tick() {
        let mut world = World::new();
        let pop = infected_pop(&mut world, 4);
        // recovery pushes condition from 0.3 to 1.0, clearing infection before mortality is applied
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.0, recovery_rate: 0.7, mortality: 1.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };

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
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.0, recovery_rate: 0.7, mortality: 0.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };

        let sir = step_epidemic(&mut world, &pop, &mut ep);

        // Before the fix this would report infected: 4, recovered: 0 (the pre-tick state)
        assert_eq!(sir.infected, 0);
        assert_eq!(sir.susceptible, 4);
    }

    #[test]
    fn sub_threshold_mortality_accumulates_deaths_over_ticks() {
        // Regression test: a small infected population with a mortality rate
        // below 1/population used to round to zero deaths every tick,
        // forever. The fractional remainder should now carry over and
        // eventually produce a death.
        let mut world = World::new();
        let pop = infected_pop(&mut world, 4);
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.0, recovery_rate: 0.0, mortality: 0.1, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };

        let mut sir = step_epidemic(&mut world, &pop, &mut ep);
        for _ in 0..9 {
            sir = step_epidemic(&mut world, &pop, &mut ep);
        }

        assert!(sir.dead > 0, "mortality=0.1 over 10 ticks on 4 infected should kill at least one");
    }

    #[test]
    fn entities_without_health_are_not_counted_as_dead() {
        // Regression test: entities with no Health component (e.g. people
        // spawned without health tracking) used to be miscounted as dead
        // in the final tally instead of being excluded entirely.
        let mut world = World::new();
        let untracked = world.create_entity();
        let pop = vec![untracked];
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.0, recovery_rate: 0.0, mortality: 0.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };

        let sir = step_epidemic(&mut world, &pop, &mut ep);

        assert_eq!(sir.dead, 0);
        assert_eq!(sir.susceptible, 0);
        assert_eq!(sir.infected, 0);
        assert_eq!(sir.recovered, 0);
    }

    fn healthy_pop(world: &mut World, n: usize) -> Vec<Entity> {
        (0..n).map(|_| {
            let e = world.create_entity();
            world.add_component(e, Health { condition: 1.0, immunity: 0.5 });
            e
        }).collect()
    }

    #[test]
    fn no_infected_means_no_spread() {
        let mut world = World::new();
        let pop = healthy_pop(&mut world, 20);
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.9, recovery_rate: 0.0, mortality: 0.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };
        for _ in 0..10 {
            let sir = step_epidemic(&mut world, &pop, &mut ep);
            assert_eq!(sir.infected, 0, "a disease-free village can't catch it from no one");
        }
    }

    #[test]
    fn spread_grows_with_number_infected() {
        let new_cases = |seeded: usize| {
            let mut world = World::new();
            let pop = healthy_pop(&mut world, 20);
            for e in &pop[..seeded] {
                world.get_component_mut::<Health>(*e).unwrap().condition = 0.3;
            }
            let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.5, recovery_rate: 0.0, mortality: 0.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };
            step_epidemic(&mut world, &pop, &mut ep).infected - seeded
        };
        assert!(new_cases(8) > new_cases(2), "more cases should mean faster spread");
    }

    #[test]
    fn single_case_eventually_spreads() {
        let mut world = World::new();
        let pop = healthy_pop(&mut world, 10);
        world.get_component_mut::<Health>(pop[0]).unwrap().condition = 0.3;
        let mut ep = Epidemic { name: "flu".into(), infection_rate: 0.3, recovery_rate: 0.0, mortality: 0.0, tick: 0, mortality_carry: 0.0, infection_carry: 0.0 };
        let mut infected = 1;
        for _ in 0..10 { infected = step_epidemic(&mut world, &pop, &mut ep).infected; }
        assert!(infected > 1, "sub-1 infections per tick should accumulate via the carry");
    }
}
