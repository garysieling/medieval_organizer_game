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
    pub inventory: crate::engine::det::HashMap<String, u32>,
}

/// Price assumed for a good nobody has registered a base value for
pub const DEFAULT_BASE_PRICE: f32 = 10.0;

/// Market price for a good at a location
#[derive(Debug, Default)]
pub struct Market {
    pub prices: crate::engine::det::HashMap<String, f32>,
    pub supply: crate::engine::det::HashMap<String, u32>,
    /// Each good's resting price, which prices drift back to and are bounded around
    pub base_prices: crate::engine::det::HashMap<String, f32>,
}

impl Market {
    /// The price `good` settles at absent supply/demand pressure
    pub fn base_price(&self, good: &str) -> f32 {
        self.base_prices.get(good).copied().unwrap_or(DEFAULT_BASE_PRICE)
    }

    /// Register a good's base value; seeds its price if it has none yet
    pub fn set_base_price(&mut self, good: &str, base: f32) {
        self.base_prices.insert(good.to_string(), base);
        self.prices.entry(good.to_string()).or_insert(base);
    }

    /// Update price based on supply vs demand pressure
    pub fn update_price(&mut self, good: &str, demand: u32) {
        let supply = self.supply.get(good).copied().unwrap_or(0);
        let base = self.base_price(good);
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
        let price = self.prices.get(good).copied().unwrap_or_else(|| self.base_price(good));
        Some(price * qty as f32)
    }
}

/// System: producers add goods to market supply each tick
pub struct ProductionSystem;
impl crate::engine::ecs::System for ProductionSystem {
    fn run(&mut self, world: &mut World) {
        for e in world.query::<Good>() {
            let good = world.get_component::<Good>(e).unwrap();
            let (name, base_value) = (good.name.clone(), good.base_value);
            if let Some(m) = world.query::<Market>().first().copied() {
                if let Some(mkt) = world.get_component_mut::<Market>(m) {
                    if !mkt.base_prices.contains_key(&name) {
                        mkt.set_base_price(&name, base_value);
                    }
                    mkt.add_supply(&name, 1);
                }
            }
        }
    }
}

/// Spawn a market with a base food price and no stock
pub fn spawn_market(world: &mut World) -> Entity {
    let e = world.create_entity();
    let mut m = Market::default();
    m.set_base_price("food", DEFAULT_BASE_PRICE);
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
        let mut m = Market::default();
        m.add_supply("food", 10);
        m.update_price("food", 50); // demand >> supply
        assert!(m.prices["food"] > 10.0);
    }

    #[test]
    fn test_price_falls_with_surplus() {
        let mut m = Market::default();
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
        let mut m = Market::default();
        m.prices.insert("food".to_string(), 10.0);
        m.update_price("food", 5); // no stock at all
        assert!(m.prices["food"] > 10.0);
    }

    #[test]
    fn test_glut_does_not_crash_price_to_zero() {
        let mut m = Market::default();
        m.add_supply("food", 1000);
        for _ in 0..500 { m.update_price("food", 0); }
        assert!(m.prices["food"] >= 2.0, "price should settle above a floor, got {}", m.prices["food"]);
    }

    #[test]
    fn test_prolonged_shortage_price_is_bounded() {
        let mut m = Market::default();
        for _ in 0..500 { m.update_price("food", 10); }
        assert!(m.prices["food"] <= 50.0, "price should be capped, got {}", m.prices["food"]);
    }

    #[test]
    fn test_buy_reduces_supply() {
        let mut m = Market::default();
        m.add_supply("food", 10);
        m.buy("food", 3).unwrap();
        assert_eq!(m.supply["food"], 7);
    }

    #[test]
    fn test_price_settles_at_goods_own_base_value() {
        use crate::engine::ecs::System;
        let mut w = World::new();
        let m = spawn_market(&mut w);
        spawn_producer(&mut w, "cloth", 40.0);
        ProductionSystem.run(&mut w);
        let mkt = w.get_component_mut::<Market>(m).unwrap();
        assert_eq!(mkt.prices["cloth"], 40.0, "price should start at the good's base value");
        // Balanced market: supply covers demand without a glut
        for _ in 0..500 { mkt.update_price("cloth", 1); }
        let p = mkt.prices["cloth"];
        assert!((p - 40.0).abs() < 1.0, "cloth should rest near 40, not the generic 10; got {}", p);
    }

    #[test]
    fn test_expensive_good_shortage_cap_scales_with_base() {
        let mut m = Market::default();
        m.set_base_price("tea", 40.0);
        for _ in 0..500 { m.update_price("tea", 10); }
        assert!(m.prices["tea"] > 50.0, "cap should be 5x tea's base, not 5x 10; got {}", m.prices["tea"]);
        assert!(m.prices["tea"] <= 200.0);
    }
}
