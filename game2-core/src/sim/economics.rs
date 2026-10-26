//! Economics simulation: markets, supply/demand, prices, trade.
//!
//! Simple Machinations-style resource flow model.
//! Producers make goods, consumers buy them, prices adjust to supply.

use crate::engine::ecs::{Entity, World};

/// A tradeable good (food, wood, cloth, tea, etc.)
#[derive(Debug, Clone, PartialEq)]
pub struct Good {
    pub name: String,
    pub base_value: f32, // base price
}

/// An agent's wallet + inventory of goods
#[derive(Debug)]
pub struct Trader {
    pub gold: f32,
    pub inventory: std::collections::HashMap<String, u32>,
}

/// Market price for a good at a location
#[derive(Debug)]
pub struct Market {
    pub prices: std::collections::HashMap<String, f32>,
    pub supply: std::collections::HashMap<String, u32>,
}

impl Market {
    /// Update price based on supply vs demand pressure
    pub fn update_price(&mut self, good: &str, demand: u32) {
        let supply = self.supply.get(good).copied().unwrap_or(0);
        let base = 10.0; // base price
        let mut price = self.prices.get(good).copied().unwrap_or(base);
        if demand > supply {
            // Empty stalls are the worst shortage, so treat zero supply as one
            // unit rather than skipping the rise; cap the per-tick jump.
            let shortage = (demand - supply) as f32 / supply.max(1) as f32;
            price *= 1.0 + (shortage * 0.1).min(0.5);
        } else if supply > demand * 2 {
            price *= 0.95; // surplus drops price
        }
        // Drift back toward base so a glut or scare doesn't pin the price forever
        price += (base - price) * 0.02;
        price = price.clamp(base * 0.2, base * 5.0);
        self.prices.insert(good.to_string(), price);
    }

    /// Record units available for sale
    pub fn add_supply(&mut self, good: &str, qty: u32) {
        *self.supply.entry(good.to_string()).or_insert(0) += qty;
    }

    /// Execute a buy: reduces supply, returns cost
    pub fn buy(&mut self, good: &str, qty: u32) -> Option<f32> {
        let supply = self.supply.get_mut(good)?;
        if *supply < qty { return None; }
        *supply -= qty;
        let price = self.prices.get(good).copied().unwrap_or(10.0);
        Some(price * qty as f32)
    }
}

/// System: producers add goods to market supply each tick
pub struct ProductionSystem;
impl crate::engine::ecs::System for ProductionSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Good>() {
            let name = world.get_component::<Good>(e).unwrap().name.clone();
            if let Some(m) = world.query::<Market>().first().copied() {
                if let Some(mkt) = world.get_component_mut::<Market>(m) {
                    mkt.add_supply(&name, 1);
                }
            }
        }
    }
}

/// Spawn a market with a base food price and no stock
pub fn spawn_market(world: &mut World) -> Entity {
    let e = world.create_entity();
    let mut m = Market {
        prices: Default::default(),
        supply: Default::default(),
    };
    m.prices.insert("food".to_string(), 10.0);
    world.add_component(e, m);
    e
}

/// Spawn a producer that adds one unit of `name` to the market each tick
pub fn spawn_producer(world: &mut World, name: &str, base_value: f32) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Good { name: name.to_string(), base_value });
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_rises_with_demand() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        m.add_supply("food", 10);
        m.update_price("food", 50); // demand >> supply
        assert!(m.prices["food"] > 10.0);
    }

    #[test]
    fn test_price_falls_with_surplus() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        m.prices.insert("food".to_string(), 10.0);
        m.add_supply("food", 100);
        m.update_price("food", 10); // supply >> demand
        assert!(m.prices["food"] < 10.0);
    }

    #[test]
    fn test_spawn_market_attaches_market() {
        let mut w = World::new();
        let m = spawn_market(&mut w);
        assert_eq!(w.get_component::<Market>(m).unwrap().prices["food"], 10.0);
    }

    #[test]
    fn test_production_fills_spawned_market() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let m = spawn_market(&mut w);
        spawn_producer(&mut w, "food", 10.0);
        let mut sys = ProductionSystem;
        for _ in 0..5 { sys.run(&mut w); }
        assert_eq!(w.get_component::<Market>(m).unwrap().supply["food"], 5);
    }

    #[test]
    fn test_price_rises_when_sold_out() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        m.prices.insert("food".to_string(), 10.0);
        m.update_price("food", 5); // no stock at all
        assert!(m.prices["food"] > 10.0);
    }

    #[test]
    fn test_glut_does_not_crash_price_to_zero() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        m.add_supply("food", 1000);
        for _ in 0..500 { m.update_price("food", 0); }
        assert!(m.prices["food"] >= 2.0, "price should settle above a floor, got {}", m.prices["food"]);
    }

    #[test]
    fn test_prolonged_shortage_price_is_bounded() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        for _ in 0..500 { m.update_price("food", 10); }
        assert!(m.prices["food"] <= 50.0, "price should be capped, got {}", m.prices["food"]);
    }

    #[test]
    fn test_buy_reduces_supply() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        m.add_supply("food", 10);
        m.buy("food", 3).unwrap();
        assert_eq!(m.supply["food"], 7);
    }
}
