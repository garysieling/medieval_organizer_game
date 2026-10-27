//! Economy: resources, production chains, village economy.

use crate::engine::ecs::{Entity, World};

/// Resource stockpile
#[derive(Debug)]
pub struct Stockpile {
    pub goods: std::collections::HashMap<String, u32>,
}

/// A building that produces/consumes resources
#[derive(Debug)]
pub struct Producer {
    pub name: String,
    pub inputs: Vec<(String, u32)>,   // consumes
    pub outputs: Vec<(String, u32)>,  // produces
    pub workers: Vec<Entity>,
    pub efficiency: f32,
    /// Fractions of an output unit made but not yet whole, carried to the
    /// next cycle so efficiency below 1.0 still yields its share over time
    pub partial: std::collections::HashMap<String, f32>,
}

/// System: each producer runs one cycle per tick against the village
/// stockpile (the first Stockpile in the world), converting inputs to
/// outputs when the stock covers them
pub struct ProductionChainSystem;
impl crate::engine::ecs::System for ProductionChainSystem {
    fn run(&mut self, world: &mut World) {
        let Some(store) = world.query::<Stockpile>().first().copied() else { return };
        for p in world.query::<Producer>() {
            // Take the producer's carry-over out so the stockpile can be
            // borrowed mutably alongside it
            let Some(prod) = world.get_component::<Producer>(p) else { continue };
            let (inputs, outputs, eff) = (prod.inputs.clone(), prod.outputs.clone(), prod.efficiency);
            let mut partial = std::mem::take(&mut world.get_component_mut::<Producer>(p).unwrap().partial);
            if let Some(stock) = world.get_component_mut::<Stockpile>(store) {
                run_cycle(stock, &inputs, &outputs, eff, &mut partial);
            }
            world.get_component_mut::<Producer>(p).unwrap().partial = partial;
        }
    }
}

/// Run one production cycle
pub fn produce(stock: &mut Stockpile, p: &mut Producer) -> bool {
    run_cycle(stock, &p.inputs, &p.outputs, p.efficiency, &mut p.partial)
}

fn run_cycle(
    stock: &mut Stockpile,
    inputs: &[(String, u32)],
    outputs: &[(String, u32)],
    efficiency: f32,
    partial: &mut std::collections::HashMap<String, f32>,
) -> bool {
    // Check inputs available
    for (g, qty) in inputs {
        if stock.goods.get(g).copied().unwrap_or(0) < *qty { return false; }
    }
    // Consume inputs
    for (g, qty) in inputs {
        *stock.goods.get_mut(g).unwrap() -= qty;
    }
    // Emit whole output units; keep the fraction for next cycle
    for (g, qty) in outputs {
        let carry = partial.entry(g.clone()).or_insert(0.0);
        *carry += *qty as f32 * efficiency.max(0.0);
        let whole = carry.floor();
        *carry -= whole;
        if whole > 0.0 {
            *stock.goods.entry(g.clone()).or_insert(0) += whole as u32;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_chain() {
        let mut s = Stockpile { goods: Default::default() };
        s.goods.insert("wheat".into(), 10);
        let mut mill = Producer {
            name: "Mill".into(),
            inputs: vec![("wheat".into(), 2)],
            outputs: vec![("flour".into(), 1)],
            workers: vec![], efficiency: 1.0, partial: Default::default(),
        };
        assert!(produce(&mut s, &mut mill));
        assert_eq!(s.goods["wheat"], 8);
        assert_eq!(s.goods["flour"], 1);
    }

    #[test]
    fn test_low_efficiency_still_yields_its_share() {
        let mut s = Stockpile { goods: Default::default() };
        s.goods.insert("wheat".into(), 20);
        let mut mill = Producer {
            name: "Old Mill".into(),
            inputs: vec![("wheat".into(), 2)],
            outputs: vec![("flour".into(), 1)],
            workers: vec![], efficiency: 0.5, partial: Default::default(),
        };
        for _ in 0..10 { assert!(produce(&mut s, &mut mill)); }
        assert_eq!(s.goods["wheat"], 0);
        assert_eq!(s.goods["flour"], 5, "half efficiency should give half the flour, not none");
    }

    #[test]
    fn test_system_runs_chain_from_village_stockpile() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let store = w.create_entity();
        w.add_component(store, Stockpile { goods: [("wheat".to_string(), 8)].into_iter().collect() });
        for (name, input, output) in [("Mill", "wheat", "flour"), ("Bakery", "flour", "bread")] {
            let p = w.create_entity();
            w.add_component(p, Producer {
                name: name.into(), inputs: vec![(input.into(), 2)], outputs: vec![(output.into(), 2)],
                workers: vec![], efficiency: 1.0, partial: Default::default(),
            });
        }
        for _ in 0..10 { ProductionChainSystem.run(&mut w); }
        let goods = &w.get_component::<Stockpile>(store).unwrap().goods;
        assert_eq!(goods["wheat"], 0);
        assert_eq!(goods["bread"], 8, "all the wheat should end up as bread");
    }
}
