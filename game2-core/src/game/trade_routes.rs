//! Trade routes: villages exchange goods, wealth flows, relationships form.
//! From medieval econ: trade links distant economies, merchants travel routes.

use crate::engine::ecs::{Entity, World};
use crate::sim::economics::Market;

/// A trade route between two settlements
#[derive(Debug)]
pub struct TradeRoute {
    pub from: Entity,          // origin village
    pub to: Entity,            // destination village
    pub goods: Vec<String>,    // what's traded
    pub volume: f32,           // trade intensity
    pub profit: f32,           // wealth generated
    pub danger: f32,           // risk (bandits, weather)
}

/// A merchant traveling a route
#[derive(Debug)]
pub struct Caravan {
    pub route: Entity,
    pub goods_carried: Vec<(String, u32)>,
    pub origin: Entity,
    pub destination: Entity,
    pub progress: f32,  // 0-1 along the route
}

/// Trade happens -> goods flow, profit generated
pub fn trade_tick(route: &mut TradeRoute) -> f32 {
    // Profit proportional to volume - danger
    let effective_volume = route.volume * (1.0 - route.danger);
    let profit = effective_volume * 0.1;
    route.profit += profit;
    profit
}

/// Routes form between villages with complementary goods: either side
/// has something the other lacks (order of the arguments doesn't matter)
pub fn route_should_form(village_a: &[&str], village_b: &[&str]) -> bool {
    village_a.iter().any(|g| !village_b.contains(g))
        || village_b.iter().any(|g| !village_a.contains(g))
}

/// Spawn a trade route
pub fn spawn_route(world: &mut World, from: Entity, to: Entity, goods: Vec<&str>) -> Entity {
    let e = world.create_entity();
    world.add_component(e, TradeRoute {
        from, to,
        goods: goods.iter().map(|g| g.to_string()).collect(),
        volume: 50.0, profit: 0.0, danger: 0.2,
    });
    e
}

/// Spawn a caravan on a route; None if the route doesn't exist
pub fn spawn_caravan(world: &mut World, route: Entity, goods: Vec<(&str, u32)>) -> Option<Entity> {
    let (origin, destination) = world.get_component::<TradeRoute>(route).map(|r| (r.from, r.to))?;
    let e = world.create_entity();
    world.add_component(e, Caravan {
        route,
        goods_carried: goods.iter().map(|(g, q)| (g.to_string(), *q)).collect(),
        origin, destination,
        progress: 0.0,
    });
    Some(e)
}

/// Fraction of the route a caravan covers per tick
pub const CARAVAN_SPEED: f32 = 0.1;

/// System: caravans travel their route. On arrival the route's danger
/// (bandits, weather) has cost that share of each cargo, the rest is sold
/// into the destination's market, and the caravan disbands.
pub struct CaravanSystem;
impl crate::engine::ecs::System for CaravanSystem {
    fn run(&mut self, world: &mut World) {
        for c in world.query::<Caravan>() {
            let arrived = match world.get_component_mut::<Caravan>(c) {
                Some(cv) => {
                    cv.progress = (cv.progress + CARAVAN_SPEED).min(1.0);
                    cv.progress >= 1.0
                }
                None => continue,
            };
            if !arrived { continue; }
            let cv = world.get_component::<Caravan>(c).unwrap();
            let (route, dest, cargo) = (cv.route, cv.destination, cv.goods_carried.clone());
            let danger = world.get_component::<TradeRoute>(route).map(|r| r.danger).unwrap_or(0.0);
            if let Some(mkt) = world.get_component_mut::<Market>(dest) {
                for (good, qty) in cargo {
                    let delivered = (qty as f32 * (1.0 - danger.clamp(0.0, 1.0))).round() as u32;
                    if delivered > 0 { mkt.add_supply(&good, delivered); }
                }
            }
            world.destroy_entity(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;

    #[test]
    fn test_route_formation_is_symmetric() {
        assert!(route_should_form(&["grain"], &["grain", "iron"]));
        assert!(route_should_form(&["grain", "iron"], &["grain"]));
        assert!(!route_should_form(&["grain"], &["grain"]));
    }

    #[test]
    fn test_caravan_on_missing_route_is_refused() {
        let mut w = World::new();
        let not_a_route = w.create_entity();
        assert!(spawn_caravan(&mut w, not_a_route, vec![("salt", 10)]).is_none());
    }

    #[test]
    fn test_caravan_delivers_to_destination_market_minus_losses() {
        let mut w = World::new();
        let a = w.create_entity();
        let b = w.create_entity();
        w.add_component(b, Market::default());
        let r = spawn_route(&mut w, a, b, vec!["salt"]); // danger 0.2
        let c = spawn_caravan(&mut w, r, vec![("salt", 10)]).unwrap();
        for _ in 0..9 { CaravanSystem.run(&mut w); }
        assert!(w.get_component::<Caravan>(c).is_some(), "still on the road");
        assert!(w.get_component::<Market>(b).unwrap().supply.get("salt").is_none());
        CaravanSystem.run(&mut w);
        assert!(w.get_component::<Caravan>(c).is_none(), "caravan disbands on arrival");
        assert_eq!(w.get_component::<Market>(b).unwrap().supply["salt"], 8);
    }
}
