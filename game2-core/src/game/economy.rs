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
}

/// System: producers convert inputs to outputs each tick
pub struct ProductionChainSystem;
impl crate::engine::ecs::System for ProductionChainSystem {
    fn run(&mut self, world: &mut World) {
        for p in world.query::<Producer>() {
            // Would consume inputs from a stockpile, emit outputs
            let _ = p;
        }
    }
}

/// Run one production cycle
pub fn produce(stock: &mut Stockpile, p: &Producer) -> bool {
    // Check inputs available
    for (g, qty) in &p.inputs {
        if stock.goods.get(g).copied().unwrap_or(0) < *qty { return false; }
    }
    // Consume inputs
    for (g, qty) in &p.inputs {
        *stock.goods.get_mut(g).unwrap() -= qty;
    }
    // Emit outputs
    for (g, qty) in &p.outputs {
        *stock.goods.entry(g.clone()).or_insert(0) += (*qty as f32 * p.efficiency) as u32;
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
        let mill = Producer {
            name: "Mill".into(),
            inputs: vec![("wheat".into(), 2)],
            outputs: vec![("flour".into(), 1)],
            workers: vec![], efficiency: 1.0,
        };
        assert!(produce(&mut s, &mill));
        assert_eq!(s.goods["wheat"], 8);
        assert_eq!(s.goods["flour"], 1);
    }
}
