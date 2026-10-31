//! Livelihood: purses, satchels, the day's work, buying and selling,
//! overseers' cuts, the tax collector's yearly round, and the odd jobs
//! children run. Movement and pathing live in the windowed game (it owns
//! the map); this module holds the state and the rules.

use std::collections::HashMap;
use crate::engine::ecs::{Entity, World};
use crate::game::desires::{self, Problem};
use crate::game::household::{self, Building};
use crate::game::villager::Pos;
use crate::game::vocation::{self, Craft, TownRole};

/// A character's gold
#[derive(Debug)]
pub struct Purse { pub gold: f32 }

/// What a character carries on them
#[derive(Debug, Default)]
pub struct Satchel { pub goods: HashMap<String, u32> }

/// The tax collector's round: everyone due once a year, in a seeded order
#[derive(Debug)]
pub struct TaxRound {
    /// (who, the day they're next due)
    pub due: Vec<(Entity, u64)>,
    pub at: usize,
}

/// An errand a child runs for pay: fetch `good` from `from`, or deliver
/// `good` to `to`
#[derive(Debug)]
pub struct OddJob {
    pub hirer: Entity,
    pub good: String,
    pub qty: u32,
    pub from: Entity,
    pub to: Entity,
    pub pay: f32,
    /// Whether the goods are picked up already (second leg, to the hirer)
    pub loaded: bool,
}

/// Gold everyone starts with, by class (the player rolls one of these too)
pub const WORKING_GOLD: (f32, f32) = (5.0, 20.0);
pub const PMC_GOLD: (f32, f32) = (50.0, 200.0);      // ~10x the working folk
pub const PB_GOLD: (f32, f32) = (30.0, 80.0);
pub const KID_GOLD: (f32, f32) = (1.0, 3.0);          // pocket money

/// The fixed cut the tax collector takes (a tenth of the purse; variable
/// mechanics later)
pub const TAX_CUT: f32 = 0.10;
/// An overseer's boost to his workers' output and his share of their takings
pub const OVERSEER_BOOST: f32 = 1.25;
pub const OVERSEER_CUT: f32 = 0.20;
/// Working with no workshop costs half the day's output
pub const NO_SHOP_RATE: f32 = 0.5;
/// Owning tools speeds the work this much
pub const TOOL_BOOST: f32 = 1.25;
/// A shop keeps at most this many of each good
pub const SHOP_STOCK_CAP: u32 = 20;
/// What a child earns running an odd job
pub const ODD_JOB_MIN: f32 = 2.0;
pub const ODD_JOB_MAX: f32 = 5.0;
/// How much a satchel holds of any one good
pub const SATCHEL_CAP: u32 = 10;
/// A senior's commission on output a tasked junior puts through his shop
pub const TASK_COMMISSION: f32 = 0.25;
/// How many days the trader's stall takes to restock one imported good
pub const TRADER_RESTOCK_DAYS: u64 = 2;
/// The lord's share on the barrel: on collection day the collector takes
/// this cut of whatever a shop's stocked drink is worth
pub const DRINK_EXCISE: f32 = 0.15;

/// What a day's work needs that isn't already in hand (satchel or own
/// shop's stock)
pub fn needs_input(world: &World, e: Entity) -> Option<&'static str> {
    let c = world.get_component::<Craft>(e)?;
    let satchel = world.get_component::<Satchel>(e);
    let shop = c.workplace.and_then(|b| world.get_component::<Building>(b));
    vocation::inputs(c.vocation).iter().copied().find(|&g| {
        let in_bag = satchel.map(|s| s.goods.get(g).copied().unwrap_or(0)).unwrap_or(0);
        let in_shop = shop.map(|b| b.stock.get(g).copied().unwrap_or(0)).unwrap_or(0);
        in_bag + in_shop < 1
    })
}

/// One day at the craft: consume inputs (satchel first, then shop stock),
/// produce `rate` output into the workplace's stockpile (or the satchel
/// when working with no workshop, capped), grow the skill, and let the
/// tasking arrangement skim a share for the senior. Returns the units of
/// output made, the level reached if it changed, and where output went.
pub fn work_day(world: &mut World, e: Entity, tick: u64) -> (f32, Option<vocation::Level>) {
    let (vocation_, _has_master, has_tools, workplace, tasked_by, overseer_boost) = {
        let Some(c) = world.get_component::<Craft>(e) else { return (0.0, None) };
        let tools = world.get_component::<Satchel>(e)
            .map(|s| s.goods.get("tools").copied().unwrap_or(0) > 0).unwrap_or(false);
        let boost = c.workplace
            .and_then(|b| world.get_component::<Building>(b))
            .map(|b| b.overseer.is_some()).unwrap_or(false);
        (c.vocation, c.master.is_some(), tools, c.workplace, c.tasked_by, boost)
    };
    let Some(good) = vocation::output(vocation_) else { return (0.0, None) };
    // inputs: only work if the day's raw goods are in hand
    for &input in vocation::inputs(vocation_) {
        let have = world.get_component::<Satchel>(e)
            .map(|s| s.goods.get(input).copied().unwrap_or(0)).unwrap_or(0);
        if have >= 1 {
            let s = world.get_component_mut::<Satchel>(e).unwrap();
            *s.goods.get_mut(input).unwrap() -= 1;
            continue;
        }
        let taken = workplace
            .and_then(|b| world.get_component_mut::<Building>(b))
            .map(|b| household::take_stock(b, input, 1)).unwrap_or(0);
        if taken < 1 { return (0.0, None) } // can't work a day without it
    }
    // the day's rate: workshop or not, overseer boost, tools. Fractions
    // carry over in `pending` - half a rate means a unit every other day
    let mut rate = if workplace.is_some() { 1.0 } else { NO_SHOP_RATE };
    if overseer_boost { rate *= OVERSEER_BOOST; }
    if has_tools { rate *= TOOL_BOOST; }
    let units = {
        let c = world.get_component_mut::<Craft>(e).unwrap();
        c.pending += rate;
        let whole = c.pending.floor();
        c.pending -= whole;
        whole as u32
    };
    let mut made = 0.0f32;
    let mut to_senior = 0.0f32;
    for i in 0..units {
        // tasked juniors put half their output through the senior's shop
        if tasked_by.is_some() && i % 2 == 1 { to_senior += 1.0; } else { made += 1.0; }
    }
    // stockpile it: the worker's own shop first, else the satchel
    let mut leftover = made;
    if let Some(b) = workplace {
        if let Some(bb) = world.get_component_mut::<Building>(b) {
            leftover = made - household::add_stock(bb, good, made as u32) as f32;
        }
    }
    if leftover > 0.0 {
        if let Some(s) = world.get_component_mut::<Satchel>(e) {
            let held = s.goods.get(good).copied().unwrap_or(0);
            let room = SATCHEL_CAP.saturating_sub(held);
            *s.goods.entry(good.to_string()).or_insert(0) += (leftover as u32).min(room);
        }
    }
    // the senior's commission: his shop stocks it and he pays the junior
    // half its street price
    if to_senior > 0.0 {
        if let Some(senior) = tasked_by {
            let senior_shop = world.get_component::<Craft>(senior).and_then(|c| c.workplace);
            if let Some(b) = senior_shop {
                if let Some(bb) = world.get_component_mut::<Building>(b) {
                    household::add_stock(bb, good, to_senior as u32);
                }
            }
            let pay = to_senior * vocation::price(good) * 0.5;
            if let Some(p) = world.get_component_mut::<Purse>(senior) { p.gold -= pay.min(p.gold); }
            if let Some(p) = world.get_component_mut::<Purse>(e) { p.gold += pay; }
        }
    }
    // skill: train_day handles the x10 under a master
    let level_up = world.get_component_mut::<Craft>(e)
        .map(|c| vocation::train_day(c)).unwrap_or(None);
    // master + owns his shop: petit bourgeois now (recomputed in class_of)
    let _ = tick;
    (made + to_senior, level_up)
}

/// Who to buy `good` from near `at`: whoever holds it - a person's satchel
/// for trade goods, the trader's stall for imports - nearest first, since
/// it's a small town and everyone knows who has what. Ink is the one
/// import a shop can also carry: the ink-maker stocks the print shop
/// himself, and a shopless printer knows where his trade's ink lives
pub fn seller_for(world: &World, good: &str, at: Pos) -> Option<Entity> {
    if vocation::imported(good) {
        let stall = world.query::<Building>().into_iter()
            .filter(|&b| {
                world.get_component::<Building>(b)
                    .map(|bb| bb.stock.get(good).copied().unwrap_or(0) > 0
                        && (bb.kind == household::BuildingKind::TraderStall
                            || (good == "ink"
                                && bb.kind == household::BuildingKind::Shop(vocation::Vocation::Printmaker))
                            // the stall's the usual source, but a shop
                            // that makes the good itself sells it too -
                            // the distillery's aqua vitae comes off its
                            // own shelf, not the road's
                            || matches!(bb.kind, household::BuildingKind::Shop(v)
                                if vocation::output(v) == Some(good))))
                    .unwrap_or(false)
            })
            .min_by_key(|&b| {
                world.get_component::<Pos>(b)
                    .map(|p| ((p.x - at.x).abs() + (p.y - at.y).abs()) as i32).unwrap_or(i32::MAX)
            });
        if stall.is_some() { return stall; }
        // and a cask in a distiller's satchel sells like anything else
    }
    world.query::<Satchel>().into_iter()
        .filter(|&s| {
            !crate::game::villager::is_dead(world, s)
                && world.get_component::<Satchel>(s)
                    .map(|t| t.goods.get(good).copied().unwrap_or(0) > 0).unwrap_or(false)
        })
        .min_by_key(|&s| {
            world.get_component::<Pos>(s)
                .map(|p| ((p.x - at.x).abs() + (p.y - at.y).abs()) as i32).unwrap_or(i32::MAX)
        })
}

/// Which shop near `at` stocks `good` for a shopper to buy
pub fn shop_for(world: &World, good: &str, at: Pos) -> Option<Entity> {
    world.query::<Building>().into_iter()
        .filter(|&b| {
            world.get_component::<Building>(b)
                .map(|bb| bb.stock.get(good).copied().unwrap_or(0) > 0
                    && household::is_workplace(bb.kind) && !household::derelict(bb))
                .unwrap_or(false)
        })
        .min_by_key(|&b| {
            world.get_component::<Pos>(b)
                .map(|p| ((p.x - at.x).abs() + (p.y - at.y).abs()) as i32).unwrap_or(i32::MAX)
        })
}

/// A person-to-person sale: `qty` of `good` from seller's satchel to
/// buyer's, gold the other way (the overseer's skim only applies to shop
/// sales). Returns the price paid
pub fn trade_person(world: &mut World, buyer: Entity, seller: Entity, good: &str, qty: u32) -> Option<f32> {
    // the dead keep their satchels and take no coin
    if crate::game::villager::is_dead(world, buyer) || crate::game::villager::is_dead(world, seller) {
        return None;
    }
    let cost = vocation::price(good) * qty as f32;
    let afford = world.get_component::<Purse>(buyer).map(|p| p.gold).unwrap_or(0.0);
    if afford < cost { return None; }
    let have = world.get_component::<Satchel>(seller)
        .map(|s| s.goods.get(good).copied().unwrap_or(0)).unwrap_or(0);
    if have < qty { return None; }
    *world.get_component_mut::<Satchel>(seller).unwrap().goods.get_mut(good).unwrap() -= qty;
    *world.get_component_mut::<Satchel>(buyer).unwrap().goods.entry(good.to_string()).or_insert(0) += qty;
    world.get_component_mut::<Purse>(buyer).unwrap().gold -= cost;
    if let Some(p) = world.get_component_mut::<Purse>(seller) { p.gold += cost; }
    Some(cost)
}

/// The next person due on the collector's round (skipping the dead);
/// their due day advances a year when they're collected
pub fn tax_due(world: &World, collector: Entity, tick: u64) -> Option<Entity> {
    let round = world.get_component::<TaxRound>(collector)?;
    round.due.iter().find(|(_, day)| tick >= *day)
        .map(|(e, _)| *e)
        .filter(|&e| !crate::game::villager::is_dead(world, e))
}

/// The collection: TAX_CUT of the target's purse to the collector, a year's
/// grievance on the target, the roster entry pushed out a year. Returns
/// the gold taken
pub fn collect_tax(world: &mut World, collector: Entity, target: Entity, tick: u64) -> f32 {
    let taken = {
        let p = world.get_component_mut::<Purse>(target);
        let gold = p.map(|p| p.gold).unwrap_or(0.0);
        let cut = (gold * TAX_CUT).round();
        if let Some(p) = world.get_component_mut::<Purse>(target) { p.gold -= cut; }
        cut
    };
    if let Some(p) = world.get_component_mut::<Purse>(collector) { p.gold += taken; }
    desires::add_grievance(world, target, Problem::Taxed, tick + desires::YEAR_DAYS);
    if let Some(r) = world.get_component_mut::<TaxRound>(collector) {
        for d in r.due.iter_mut() { if d.0 == target { d.1 = tick + desires::YEAR_DAYS; } }
    }
    taken
}

/// Set up the collector's round: everyone due over the coming year,
/// staggered in `order`
pub fn tax_round(world: &mut World, collector: Entity, roster: &[Entity], start_day: u64, gap_days: u64) {
    let due = roster.iter().enumerate()
        .map(|(i, &e)| (e, start_day + i as u64 * gap_days)).collect();
    world.add_component(collector, TaxRound { due, at: 0 });
}

/// The trader's stall restocks one imported good a turn
pub fn restock_trader(world: &mut World, stall: Entity, seed: u64) {
    const EXOTICS: [&str; 8] = ["cloth", "paper", "ink", "pigment",
        "hops", "grapes", "wine", "spirits"];
    // the seed's a tick and the calls come every RESTOCK days: index on
    // the count of restocks, not the day, or half the cart never lands
    let good = EXOTICS[((seed / TRADER_RESTOCK_DAYS) as usize) % EXOTICS.len()];
    if let Some(b) = world.get_component_mut::<Building>(stall) {
        household::add_stock(b, good, 1);
    }
}

/// Gold to seed a character with, by class (roll is 0..1)
pub fn seed_gold(class: vocation::SocialClass, is_kid: bool, roll: f32) -> f32 {
    let (lo, hi) = if is_kid { KID_GOLD } else {
        match class {
            vocation::SocialClass::Working => WORKING_GOLD,
            vocation::SocialClass::PetitBourgeois => PB_GOLD,
            vocation::SocialClass::ProfessionalMiddle => PMC_GOLD,
            vocation::SocialClass::Landowner => (10000.0, 10000.0),
        }
    };
    lo + (hi - lo) * roll
}

/// Whether an entity holds a PMC posting
pub fn has_role(world: &World, e: Entity, role: TownRole) -> bool {
    world.get_component::<TownRole>(e).map(|r| *r == role).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::household::BuildingKind;
    use crate::game::vocation::craft;

    fn worker(w: &mut World, v: vocation::Vocation, skill: f32, gold: f32) -> Entity {
        let e = w.create_entity();
        w.add_component(e, Pos { x: 0.0, y: 0.0 });
        w.add_component(e, craft(v, skill));
        w.add_component(e, Purse { gold });
        w.add_component(e, Satchel::default());
        e
    }

    #[test]
    fn test_a_baker_turns_wheat_into_bread_at_his_shop() {
        let mut w = World::new();
        let baker = worker(&mut w, vocation::Vocation::Baker, 0.5, 10.0);
        let shop = household::spawn_building(&mut w, BuildingKind::Shop(vocation::Vocation::Baker), "the bakery", 0.0, 0.0, 20);
        w.get_component_mut::<Building>(shop).unwrap().worker = Some(baker);
        w.get_component_mut::<Craft>(baker).unwrap().workplace = Some(shop);
        w.get_component_mut::<Satchel>(baker).unwrap().goods.insert("wheat".into(), 2);
        let (made, _) = work_day(&mut w, baker, 0);
        assert_eq!(made, 1.0);
        assert_eq!(w.get_component::<Building>(shop).unwrap().stock["bread"], 1);
        assert_eq!(w.get_component::<Satchel>(baker).unwrap().goods["wheat"], 1);
        // and without wheat, no work happens
        let (made2, _) = work_day(&mut w, baker, 0);
        w.get_component_mut::<Satchel>(baker).unwrap().goods.insert("wheat".into(), 0);
        let (made3, _) = work_day(&mut w, baker, 0);
        assert_eq!(made3, 0.0, "no input, no bread");
        let _ = made2;
    }

    #[test]
    fn test_no_workshop_halves_output_but_a_master_still_works() {
        let mut w = World::new();
        let mason = worker(&mut w, vocation::Vocation::Mason, 0.5, 10.0);
        w.get_component_mut::<Satchel>(mason).unwrap().goods.insert("stone".into(), 5);
        assert_eq!(work_day(&mut w, mason, 0).0, 0.0, "half a day's work makes nothing yet");
        assert_eq!(work_day(&mut w, mason, 0).0, 1.0, "the second half-day finishes the block");
        assert_eq!(w.get_component::<Satchel>(mason).unwrap().goods["blocks"], 1);
    }

    #[test]
    fn test_trade_person_moves_gold_and_goods() {
        let mut w = World::new();
        let farmer = worker(&mut w, vocation::Vocation::Farmer, 0.5, 0.0);
        w.get_component_mut::<Satchel>(farmer).unwrap().goods.insert("wheat".into(), 3);
        let baker = worker(&mut w, vocation::Vocation::Baker, 0.5, 10.0);
        assert_eq!(trade_person(&mut w, baker, farmer, "wheat", 2), Some(4.0));
        assert_eq!(w.get_component::<Purse>(farmer).unwrap().gold, 4.0);
        assert_eq!(w.get_component::<Purse>(baker).unwrap().gold, 6.0);
        assert_eq!(w.get_component::<Satchel>(baker).unwrap().goods["wheat"], 2);
        // can't buy what isn't there or can't afford
        assert_eq!(trade_person(&mut w, baker, farmer, "wheat", 3), None);
        assert_eq!(trade_person(&mut w, baker, farmer, "wheat", 1), Some(2.0));
    }

    #[test]
    fn test_the_dead_keep_their_satchels_and_take_no_coin() {
        let mut w = World::new();
        let farmer = worker(&mut w, vocation::Vocation::Farmer, 0.5, 0.0);
        w.get_component_mut::<Satchel>(farmer).unwrap().goods.insert("wheat".into(), 3);
        let baker = worker(&mut w, vocation::Vocation::Baker, 0.5, 10.0);
        w.add_component(farmer, crate::game::villager::Dead);
        assert_eq!(trade_person(&mut w, baker, farmer, "wheat", 1), None,
            "no coin changes hands with a corpse");
        assert_eq!(w.get_component::<Satchel>(farmer).unwrap().goods["wheat"], 3);
        assert_eq!(w.get_component::<Purse>(baker).unwrap().gold, 10.0);
    }

    #[test]
    fn test_the_collector_takes_his_cut_once_a_year() {
        let mut w = World::new();
        let collector = w.create_entity();
        w.add_component(collector, Pos { x: 0.0, y: 0.0 });
        w.add_component(collector, TownRole::TaxCollector);
        w.add_component(collector, Purse { gold: 100.0 });
        let (a, b) = (worker(&mut w, vocation::Vocation::Mason, 0.5, 50.0),
                      worker(&mut w, vocation::Vocation::Baker, 0.5, 30.0));
        tax_round(&mut w, collector, &[a, b], 30, 12);
        assert_eq!(tax_due(&w, collector, 29), None, "nobody due yet");
        assert_eq!(tax_due(&w, collector, 30), Some(a));
        assert_eq!(collect_tax(&mut w, collector, a, 30), 5.0);
        assert_eq!(w.get_component::<Purse>(a).unwrap().gold, 45.0);
        assert_eq!(w.get_component::<Purse>(collector).unwrap().gold, 105.0);
        // he's a grievance now, and not due again for a year
        assert_eq!(tax_due(&w, collector, 42), Some(b));
        assert_eq!(tax_due(&w, collector, 389), Some(b));
        assert_eq!(tax_due(&w, collector, 390), Some(a));
    }

    #[test]
    fn test_the_road_goods_sell_off_the_still_too() {
        let mut w = World::new();
        let at = Pos { x: 0.0, y: 0.0 };
        // the stall stocks the road's spirits as usual
        let stall = household::spawn_building(&mut w, BuildingKind::TraderStall,
            "the trader's stall", 30.0, 0.0, 20);
        w.get_component_mut::<Building>(stall).unwrap().stock.insert("spirits".into(), 2);
        // but the distillery sells its own water of life closer in
        let still = household::spawn_building(&mut w,
            BuildingKind::Shop(vocation::Vocation::Distiller), "the distillery", 5.0, 0.0, 20);
        w.get_component_mut::<Building>(still).unwrap().stock.insert("spirits".into(), 3);
        assert_eq!(seller_for(&w, "spirits", at), Some(still), "nearest first");
        // and when no shelf stocks it, a cask in a satchel sells
        w.get_component_mut::<Building>(still).unwrap().stock.remove("spirits");
        w.get_component_mut::<Building>(stall).unwrap().stock.remove("spirits");
        let pedlar = worker(&mut w, vocation::Vocation::Distiller, 0.5, 0.0);
        w.get_component_mut::<Satchel>(pedlar).unwrap().goods.insert("spirits".into(), 1);
        assert_eq!(seller_for(&w, "spirits", at), Some(pedlar));
    }

    #[test]
    fn test_the_print_shop_sells_the_inkmakers_ink() {
        let mut w = World::new();
        let at = Pos { x: 0.0, y: 0.0 };
        let shop = household::spawn_building(&mut w, BuildingKind::Shop(vocation::Vocation::Printmaker),
            "the print shop", 5.0, 0.0, 20);
        w.get_component_mut::<Building>(shop).unwrap().stock.insert("ink".into(), 6);
        assert_eq!(seller_for(&w, "ink", at), Some(shop),
            "the trade knows where its ink lives");
        // but the shop doesn't deal the road's other exotics
        w.get_component_mut::<Building>(shop).unwrap().stock.insert("paper".into(), 4);
        assert_eq!(seller_for(&w, "paper", at), None,
            "paper still comes off the trader's cart");
        // and the trader's stall stays a seller too
        let stall = household::spawn_building(&mut w, BuildingKind::TraderStall,
            "the trader's stall", 30.0, 0.0, 20);
        w.get_component_mut::<Building>(stall).unwrap().stock.insert("ink".into(), 3);
        assert_eq!(seller_for(&w, "ink", at), Some(shop), "nearest first");
        assert_eq!(seller_for(&w, "ink", Pos { x: 40.0, y: 0.0 }), Some(stall));
    }
}
