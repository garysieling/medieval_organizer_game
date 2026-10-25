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
        if demand > supply && supply > 0 {
            price *= 1.0 + (demand - supply) as f32 / supply as f32 * 0.1;
        } else if supply > demand * 2 {
            price *= 0.95; // surplus drops price
        }
        price = price.max(0.1);
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

/// Spawn a good + a market
pub fn spawn_market(world: &mut World) -> Entity {
    let e = world.create_entity();
    let mut m = Market {
        prices: Default::default(),
        supply: Default::default(),
    };
    m.prices.insert("food".to_string(), 10.0);
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
    fn test_buy_reduces_supply() {
        let mut m = Market { prices: Default::default(), supply: Default::default() };
        m.add_supply("food", 10);
        m.buy("food", 3).unwrap();
        assert_eq!(m.supply["food"], 7);
    }
}
