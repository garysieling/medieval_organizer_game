//! Staged starts for tests: drop the village into a state a rare event
//! matters in - a famine, a fever, a strong org, a hard winter - then let
//! it run and watch. The events are semi-rare by design, so the tests
//! don't wait for the schedule: `force` puts the moment on now and
//! `run_days` plays it out for weeks of turns.

use serde::{Deserialize, Serialize};
use crate::engine::ecs::Entity;
use crate::game::desires::{self, Problem};
use crate::game::livelihood::Purse;
use crate::game::villager::{self, Hunger};
use crate::game::household::{self, Building};
use crate::sim::community::Organization;
use crate::sim::illness::{self, Disease};
use super::events::EventId;
use super::{Game, MAP_SEED};

/// A village already in a state when the test walks in
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// The default village, nothing amiss
    Ordinary,
    /// A fever's already in three beds
    Fever,
    /// The stores are empty and everyone's hungry
    Famine,
    /// Inside the harsh-winter window, stores thin
    HardWinter,
    /// The org's strong: members, power, the lord's respect
    StrongOrg,
    /// Grievances everywhere: taxed, broke, blamed
    Unrest,
    /// Everyone's purse is full
    Flush,
    /// Three beds new-empty: the village grieves
    Bereft,
}

/// All the stages, for sweeps
pub const STAGES: &[Stage] = &[
    Stage::Ordinary, Stage::Fever, Stage::Famine, Stage::HardWinter,
    Stage::StrongOrg, Stage::Unrest, Stage::Flush, Stage::Bereft,
];

/// A fresh game staged into `s`
pub fn staged(s: Stage) -> Game {
    let mut g = Game::new_seeded(MAP_SEED);
    stage(&mut g, s);
    g
}

/// Put `g` into state `s` - the same moves a scenario's setup makes
pub fn stage(g: &mut Game, s: Stage) {
    match s {
        Stage::Ordinary => {}
        Stage::Fever => {
            for (i, &v) in g.villagers.iter().take(3).enumerate() {
                let d = g.world.create_entity();
                g.world.add_component(d,
                    Disease::new(illness::DiseaseKind::ScarletFever, v, 0.6 - i as f32 * 0.1));
            }
        }
        Stage::Famine => {
            if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) {
                f.servings = 4;
            }
            if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
                let _ = household::take_stock(b, "grain", 999);
                let _ = household::take_stock(b, "bread", 999);
            }
            for &v in &g.villagers.clone() {
                if let Some(h) = g.world.get_component_mut::<Hunger>(v) { h.value = 22.0; }
            }
        }
        Stage::HardWinter => {
            // inside the harsh window (tick % 730 in 400..=545), stores thin
            g.tick = 410;
            g.harsh_winter = true;
            if let Some(b) = g.world.get_component_mut::<Building>(g.market_food) {
                let _ = household::take_stock(b, "grain", b.stock.get("grain").copied().unwrap_or(0) / 2);
            }
        }
        Stage::StrongOrg => {
            let members: Vec<Entity> = g.villagers.iter().take(12).copied().collect();
            if let Some(o) = g.world.get_component_mut::<Organization>(g.org) {
                for m in members { if !o.members.contains(&m) { o.members.push(m); } }
            }
            if let Some(p) = g.world.get_component_mut::<crate::sim::organized_power::OrganizedPower>(g.org) {
                p.capacity = 0.8;
            }
            g.lord_esteem = 0.6;
        }
        Stage::Unrest => {
            for (i, &v) in g.villagers.iter().take(10).enumerate() {
                if villager::is_dead(&g.world, v) { continue; }
                let p = match i % 3 { 0 => Problem::Taxed, 1 => Problem::Broke, _ => Problem::Blamed };
                desires::add_grievance(&mut g.world, v, p, g.tick + 60);
            }
        }
        Stage::Flush => {
            for &v in &g.villagers.clone() {
                if let Some(p) = g.world.get_component_mut::<Purse>(v) { p.gold = 60.0; }
            }
        }
        Stage::Bereft => {
            for &v in g.villagers.iter().take(3) {
                g.world.add_component(v, villager::Dead);
                desires::add_grievance(&mut g.world, v, Problem::Grief, g.tick + 90);
            }
            g.succession_tick(1);
        }
    }
}

/// Run the sim `days` days forward, full frames - the same code path a
/// player sits through, not a shortcut
pub fn run_days(g: &mut Game, days: u64) {
    let target = g.tick + days;
    // a day is ticks_per_frame frames; the guard catches a stalled sim
    let guard = days.saturating_mul(g.ticks_per_frame() as u64).saturating_mul(2) + 60;
    let mut frames = 0u64;
    while g.tick < target && frames < guard {
        g.update();
        frames += 1;
    }
}

/// Fire the rare event now: the schedule can wait for its due, the test can't
pub fn force(g: &mut Game, id: EventId) {
    g.force_event(id);
}

/// The stages an event sweep plays each event against: the default plus a
/// rotating pair, so across the hundred every stage gets exercised and the
/// sweep stays quick
pub fn sweep_stages(i: usize) -> [Stage; 3] {
    let rest: Vec<Stage> = STAGES.iter().copied().filter(|s| *s != Stage::Ordinary).collect();
    [Stage::Ordinary, rest[i % rest.len()], rest[(i + 3) % rest.len()]]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each stage puts the village in a different place - the staging works
    #[test]
    fn test_the_stages_differ() {
        let ordinary = staged(Stage::Ordinary).probe();
        let famine = staged(Stage::Famine).probe();
        assert!(famine.food_servings < ordinary.food_servings, "the famine's stores are thin");
        let hungry_famine = famine.inner.iter().filter(|i| i.hunger < 30.0).count();
        let hungry_ordinary = ordinary.inner.iter().filter(|i| i.hunger < 30.0).count();
        assert!(hungry_famine > hungry_ordinary, "the famine's bellies are empty");

        let fever = staged(Stage::Fever).probe();
        assert!(fever.diseases.len() >= 3, "the scarlatina's in the beds");

        let winter = staged(Stage::HardWinter).probe();
        assert!(winter.harsh_winter, "the winter's running long");

        let strong = staged(Stage::StrongOrg).probe();
        assert!(strong.org_power > ordinary.org_power, "the org's strong");

        let unrest = staged(Stage::Unrest).probe();
        let grieved = unrest.inner.iter().filter(|i| !i.grievances.is_empty()).count();
        assert!(grieved >= 8, "the grumbles are abroad");

        let flush = staged(Stage::Flush).probe();
        assert!(flush.gold_total > ordinary.gold_total, "the purses are fuller");
        assert!(flush.inner.iter().filter(|i| !i.dead).all(|i| i.gold >= 60.0),
            "every purse's full");

        let bereft = staged(Stage::Bereft).probe();
        assert_eq!(bereft.inner.iter().filter(|i| i.dead).count(), 3, "three beds empty");
    }

    /// run_days moves the clock, not just the frames
    #[test]
    fn test_run_days_runs_days() {
        let mut g = staged(Stage::Ordinary);
        let t0 = g.tick;
        run_days(&mut g, 5);
        assert!(g.tick >= t0 + 5, "five days passed (got {} from {})", g.tick, t0);
    }

    /// force puts a semi-rare event on now, whatever the schedule says
    #[test]
    fn test_forcing_an_event() {
        let mut g = staged(Stage::Ordinary);
        force(&mut g, EventId::Tinker);
        let pv = g.probe();
        assert!(pv.rare_active.iter().any(|(id, _, _)| id == "Tinker"), "the tinker's afoot");
        assert!(pv.rare_fired.iter().any(|(id, _)| id == "Tinker"), "the firing's on record");
    }
}
