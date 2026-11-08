//! Households and workplaces: buildings as entities. Homes hold 1-4
//! occupants; shops hold a worker, a stockpile of goods in and out (capped),
//! and maybe an overseer. A workplace no one works stands derelict - a
//! problem for everyone who walks by.

use crate::engine::det::HashMap;
use crate::engine::ecs::{Entity, World};
use crate::game::livelihood::{Purse, Satchel, OVERSEER_CUT};
use crate::game::vocation::{self, Vocation};

/// What a building is for
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildingKind {
    /// Someone's home - holds occupants
    Home,
    /// A vocation's workplace: bakery, smithy, print shop...
    Shop(Vocation),
    Church,
    /// The knight's post
    WatchPost,
    Mill,
    /// The village food store's building
    Granary,
    /// The road trader's stall at the market (imported goods)
    TraderStall,
    /// The shepherd's pen out in the meadow: a workplace, not a shop -
    /// the flock grazes round it
    Pasture,
    /// The beekeeper's yard at the meadow's edge: the skeps stand round it
    Apiary,
    /// The cheese cellar by the fold: wheels age in the dark down there
    Cellar,
    /// The one-room school: a venue, not a workplace - the teacher's a
    /// traveller, and the kids' desks hold no stock
    Schoolhouse,
    /// The settlement post: letters for the road gathered under one
    /// roof - a venue, not a workplace
    PostOffice,
    /// The keep's undercroft grown: a castle's own cell block, built
    /// by a commission - more of the realm's prisoners fit inside
    Dungeon,
    /// A cash-crop ground out on a castle's demesne: vines tended for
    /// the market's wine, a commission's work
    Plantation,
}

/// A building on the map (entity; its Pos is where it stands)
#[derive(Debug)]
pub struct Building {
    pub kind: BuildingKind,
    pub name: String,
    /// Who lives here (homes: 1-4)
    pub occupants: Vec<Entity>,
    /// Who works this shop
    pub worker: Option<Entity>,
    /// The overseer driving the workers (farms and mason's yards)
    pub overseer: Option<Entity>,
    /// Goods held: inputs in, finished goods out
    pub stock: HashMap<String, u32>,
    /// Cap per good kept in stock
    pub capacity: u32,
    /// What's nailed to the door: a writ's posting, a feast's handbill -
    /// the board's own words where the building carries them
    pub flyer: Option<String>,
}

/// The building kind a vocation works out of
pub fn kind_for(v: Vocation) -> BuildingKind {
    match v {
        Vocation::Priest => BuildingKind::Church,
        Vocation::Knight => BuildingKind::WatchPost,
        Vocation::Shepherd => BuildingKind::Pasture,
        Vocation::Beekeeper => BuildingKind::Apiary,
        other => BuildingKind::Shop(other),
    }
}

/// Whether the kind is a workplace someone should be working
pub fn is_workplace(k: BuildingKind) -> bool {
    matches!(k, BuildingKind::Shop(_) | BuildingKind::Church | BuildingKind::WatchPost | BuildingKind::TraderStall | BuildingKind::Mill)
}

/// Whether a workplace stands empty: a problem for the street
pub fn derelict(b: &Building) -> bool {
    is_workplace(b.kind) && b.worker.is_none()
}

/// How much more of `good` the stock will hold
pub fn stock_room(b: &Building, good: &str) -> u32 {
    b.capacity.saturating_sub(b.stock.get(good).copied().unwrap_or(0))
}

/// Add to the stockpile up to the cap; returns how much fit
pub fn add_stock(b: &mut Building, good: &str, qty: u32) -> u32 {
    let fit = stock_room(b, good).min(qty);
    *b.stock.entry(good.to_string()).or_insert(0) += fit;
    fit
}

/// Take goods out of stock; returns how many were actually there
pub fn take_stock(b: &mut Building, good: &str, qty: u32) -> u32 {
    let have = b.stock.get_mut(good);
    match have {
        Some(h) => { let n = (*h).min(qty); *h -= n; n }
        None => 0,
    }
}

/// What the shop has for offer: the goods it stocks now, with prices
pub fn on_offer(b: &Building) -> Vec<(String, u32, f32)> {
    let mut v: Vec<_> = b.stock.iter().filter(|(_, q)| **q > 0)
        .map(|(g, q)| (g.clone(), *q, vocation::price(g))).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

/// The same offer, priced by the day's felt worth (scarcity-leaned)
pub fn on_offer_at(world: &World, b: &Building) -> Vec<(String, u32, f32)> {
    let mut v: Vec<_> = b.stock.iter().filter(|(_, q)| **q > 0)
        .map(|(g, q)| (g.clone(), *q, crate::game::goods::worth_at(world, g))).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

/// Spawn a building entity at (x, y)
pub fn spawn_building(world: &mut World, kind: BuildingKind, name: &str, x: f32, y: f32, capacity: u32) -> Entity {
    use crate::game::villager::Pos;
    let e = world.create_entity();
    world.add_component(e, Pos { x, y });
    world.add_component(e, Building {
        kind, name: name.to_string(),
        occupants: vec![], worker: None, overseer: None,
        stock: HashMap::default(), capacity,
        flyer: None,
    });
    e
}

/// The building a person lives in, if any (they're an occupant of a Home)
pub fn home_of(world: &World, e: Entity) -> Option<Entity> {
    world.query::<Building>().into_iter().find(|&b| {
        world.get_component::<Building>(b)
            .map(|bb| bb.kind == BuildingKind::Home && bb.occupants.contains(&e))
            .unwrap_or(false)
    })
}

/// A sale at a shop: `qty` of `good` out of stock into the buyer's satchel;
/// the price goes to the worker's purse (the overseer skims their cut if
/// the shop is overseen). Returns the gold paid, or None if stock's short
/// or the buyer can't cover it
pub fn buy_at_shop(world: &mut World, shop: Entity, buyer: Entity, good: &str, qty: u32) -> Option<f32> {
    let price = crate::game::goods::worth_at(world, good) * qty as f32;
    // who gets paid: the shop's worker; a stall with no worker can't sell
    let (worker, overseer) = {
        let b = world.get_component::<Building>(shop)?;
        (b.worker, b.overseer)
    };
    let worker = worker?;
    let afford = world.get_component::<Purse>(buyer).map(|p| p.gold).unwrap_or(0.0);
    if afford < price { return None; }
    let taken = {
        let b = world.get_component_mut::<Building>(shop)?;
        take_stock(b, good, qty)
    };
    if taken < qty { return None; }
    // the deal's a meeting of a kind: the buyer knows the keeper now,
    // a bigger order carrying the warmer acquaintance
    crate::game::relationships::record_interaction(world, buyer,
        crate::game::relationships::Interaction { kind: crate::game::relationships::InteractionKind::Conversation,
            intensity: (price / 8.0).min(1.0), other: worker });
    // a shelf just emptied leans the good's felt worth a little dear
    let bare = world.get_component::<Building>(shop)
        .map(|b| b.stock.get(good).copied().unwrap_or(0) == 0).unwrap_or(false);
    if bare { crate::game::goods::nudge(world, good, crate::game::goods::BARE_NUDGE); }
    // goods to the buyer's satchel
    {
        let s = world.get_component_mut::<Purse>(buyer)?;
        s.gold -= price;
        let goods = &mut world.get_component_mut::<Satchel>(buyer)?.goods;
        *goods.entry(good.to_string()).or_insert(0) += qty;
    }
    // gold out: the overseer skims OVERSEER_CUT if the shop is overseen
    let cut = if overseer.is_some() { price * OVERSEER_CUT } else { 0.0 };
    if let Some(p) = world.get_component_mut::<Purse>(worker) { p.gold += price - cut; }
    if let Some(o) = overseer {
        if let Some(p) = world.get_component_mut::<Purse>(o) { p.gold += cut; }
    }
    Some(price)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::livelihood::{Purse, Satchel};

    fn shop(w: &mut World, stock: &[(&str, u32)], worker: Option<Entity>, overseer: Option<Entity>) -> Entity {
        let b = spawn_building(w, BuildingKind::Shop(Vocation::Baker), "the bakery", 0.0, 0.0, 10);
        {
            let bb = w.get_component_mut::<Building>(b).unwrap();
            bb.worker = worker;
            bb.overseer = overseer;
            for (g, q) in stock { bb.stock.insert(g.to_string(), *q); }
        }
        b
    }

    fn buyer(w: &mut World, gold: f32) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Purse { gold });
        w.add_component(e, Satchel { goods: Default::default() });
        e
    }

    #[test]
    fn test_stockpiles_cap_out() {
        let mut w = World::new();
        let b = spawn_building(&mut w, BuildingKind::Shop(Vocation::Baker), "the bakery", 0.0, 0.0, 5);
        {
            let bb = w.get_component_mut::<Building>(b).unwrap();
            assert_eq!(add_stock(bb, "bread", 8), 5, "the cap holds");
            assert_eq!(add_stock(bb, "bread", 8), 0);
            assert_eq!(take_stock(bb, "bread", 3), 3);
            assert_eq!(add_stock(bb, "bread", 8), 3, "room for three more");
        }
    }

    #[test]
    fn test_buying_moves_goods_and_gold() {
        let mut w = World::new();
        let worker = w.create_entity();
        w.add_component(worker, Purse { gold: 0.0 });
        let shop = shop(&mut w, &[("bread", 4)], Some(worker), None);
        let buyer = buyer(&mut w, 20.0);
        assert_eq!(buy_at_shop(&mut w, shop, buyer, "bread", 2), Some(10.0));
        assert_eq!(w.get_component::<Purse>(buyer).unwrap().gold, 10.0);
        assert_eq!(w.get_component::<Purse>(worker).unwrap().gold, 10.0);
        assert_eq!(w.get_component::<Satchel>(buyer).unwrap().goods["bread"], 2);
        assert_eq!(w.get_component::<Building>(shop).unwrap().stock["bread"], 2);
    }

    #[test]
    fn test_an_overseen_shop_skims_the_cut() {
        let mut w = World::new();
        let worker = w.create_entity();
        w.add_component(worker, Purse { gold: 0.0 });
        let overseer = w.create_entity();
        w.add_component(overseer, Purse { gold: 0.0 });
        let shop = shop(&mut w, &[("bread", 4)], Some(worker), Some(overseer));
        let buyer = buyer(&mut w, 20.0);
        buy_at_shop(&mut w, shop, buyer, "bread", 2);
        assert_eq!(w.get_component::<Purse>(overseer).unwrap().gold, 2.0, "the overseer skims 20%");
        assert_eq!(w.get_component::<Purse>(worker).unwrap().gold, 8.0);
    }

    #[test]
    fn test_empty_shops_are_derelict_and_grounded_ones_not() {
        let mut w = World::new();
        let empty = spawn_building(&mut w, BuildingKind::Shop(Vocation::Printmaker), "the print shop", 0.0, 0.0, 10);
        assert!(derelict(w.get_component::<Building>(empty).unwrap()));
        let home = spawn_building(&mut w, BuildingKind::Home, "a house", 0.0, 0.0, 10);
        assert!(!derelict(w.get_component::<Building>(home).unwrap()), "a home with no lodgers isn't derelict");
        // and a buy from a derelict shop can't happen
        let buyer = buyer(&mut w, 20.0);
        assert_eq!(buy_at_shop(&mut w, empty, buyer, "pamphlets", 1), None);
    }
}
