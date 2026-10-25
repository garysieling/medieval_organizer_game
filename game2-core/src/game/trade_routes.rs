//! Trade routes: villages exchange goods, wealth flows, relationships form.
//! From medieval econ: trade links distant economies, merchants travel routes.

use crate::engine::ecs::{Entity, World};

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

/// Routes form between villages with complementary goods
pub fn route_should_form(village_a: &[&str], village_b: &[&str]) -> bool {
    // Routes form when villages have different goods (complementary)
    !village_a.iter().all(|g| village_b.contains(g))
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

/// Spawn a caravan on a route
pub fn spawn_caravan(world: &mut World, route: Entity, goods: Vec<(&str, u32)>) -> Entity {
    let e = world.create_entity();
    let r = world.get_component::<TradeRoute>(route);
    world.add_component(e, Caravan {
        goods_carried: goods.iter().map(|(g, q)| (g.to_string(), *q)).collect(),
        origin: r.map(|x| x.from).unwrap_or(0),
        destination: r.map(|x| x.to).unwrap_or(0),
        progress: 0.0,
    });
    e
}
