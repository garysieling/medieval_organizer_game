//! Livelihood: purses, satchels, the day's work, buying and selling,
//! overseers' cuts, the tax collector's yearly round, and the odd jobs
//! children run. Movement and pathing live in the windowed game (it owns
//! the map); this module holds the state and the rules.

use crate::engine::det::HashMap;
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
        // the slot's short only when nothing that fills it is to hand
        std::iter::once(g).chain(crate::game::recipes::fillers(g).iter().copied()).all(|cand| {
            let in_bag = satchel.map(|s| s.goods.get(cand).copied().unwrap_or(0)).unwrap_or(0);
            let in_shop = shop.map(|b| b.stock.get(cand).copied().unwrap_or(0)).unwrap_or(0);
            in_bag + in_shop < 1
        })
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
    // inputs: only work if the day's raw goods are in hand - a slot
    // fills from whatever's stocked, the named good or its fellows.
    // Pick each slot's source before any of it is spent: a missing
    // second slot mustn't eat the first's goods for nothing
    let mut picks: Vec<(&'static str, bool)> = vec![]; // (good, satchel-held?)
    for &input in vocation::inputs(vocation_) {
        let mut pick = None;
        for cand in std::iter::once(input).chain(crate::game::recipes::fillers(input).iter().copied()) {
            let held = world.get_component::<Satchel>(e)
                .map(|s| s.goods.get(cand).copied().unwrap_or(0)).unwrap_or(0);
            if held >= 1 { pick = Some((cand, true)); break; }
            let stocked = workplace
                .and_then(|b| world.get_component::<Building>(b))
                .map(|b| b.stock.get(cand).copied().unwrap_or(0)).unwrap_or(0);
            if stocked >= 1 { pick = Some((cand, false)); break; }
        }
        let Some(p) = pick else { return (0.0, None) }; // can't work a day without it
        picks.push(p);
    }
    for (g, in_bag) in picks {
        if in_bag {
            let s = world.get_component_mut::<Satchel>(e).unwrap();
            *s.goods.get_mut(g).unwrap() -= 1;
        } else if let Some(b) = workplace.and_then(|w| world.get_component_mut::<Building>(w)) {
            household::take_stock(b, g, 1);
        }
    }
    // the day's rate: workshop or not, overseer boost, tools. Fractions
    // carry over in `pending` - half a rate means a unit every other day
    let mut rate = if workplace.is_some() { 1.0 } else { NO_SHOP_RATE };
    if overseer_boost { rate *= OVERSEER_BOOST; }
    if has_tools { rate *= TOOL_BOOST; }
    // foul weather thins a food harvest: a storm flattens the field, a
    // dry spell stints it, a hard frost kills it outright. It hits what
    // comes out of the ground and the net - the baker's oven takes its
    // wheat as it finds it, rain or shine
    if vocation::inputs(vocation_).is_empty()
        && crate::game::goods::spec(good)
        .map(|s| s.tags.contains(&crate::game::goods::Tag::StapleFood)).unwrap_or(false) {
        if let Some(w) = world.query::<crate::sim::weather::CurrentWeather>().first().copied()
            .and_then(|e| world.get_component::<crate::sim::weather::CurrentWeather>(e)) {
            rate *= crate::sim::weather::food_effect(w);
        }
    }
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
        // a shelf too full to take the day's work leans the good a
        // little cheap - the glut reads on the ledger, not the street
        if leftover > 0.0 {
            crate::game::goods::nudge(world, good, crate::game::goods::GLUT_NUDGE);
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
                    let fit = household::add_stock(bb, good, to_senior as u32);
                    // his shelf too full reads as glut like anyone's
                    if fit < to_senior as u32 {
                        crate::game::goods::nudge(world, good, crate::game::goods::GLUT_NUDGE);
                    }
                }
            }
            let pay = to_senior * crate::game::goods::worth_at(world, good) * 0.5;
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
    let cost = crate::game::goods::worth_at(world, good) * qty as f32;
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

/// The slack a neighbour cuts on a swap - a fifth either way rides as
/// goodwill between folk who'll trade again next week
pub const BARTER_SLACK: f32 = 0.2;
/// How far over the ask an offer can run before it's not an offer but a gift
pub const BARTER_OVERGIVE: f32 = 1.5;

/// What a barter struck: `boot` is the gold that changed hands to square
/// it (positive when the buyer paid, negative when the seller did), and
/// `fairness` is offered-want worth - 1.0 is dead even
#[derive(Debug, Clone, Copy)]
pub struct Deal {
    pub boot: f32,
    pub fairness: f32,
}

/// A swap between neighbours: `want` goods out of the seller's satchel,
/// `offer` goods out of the buyer's, and a few coppers "to boot" when the
/// bundles don't square. Neither side quotes a price - the worths stay
/// under the counter, and a little slack rides as goodwill. Returns the
/// deal struck, or None when the goods aren't there or the gap can't be
/// covered. The dead keep their satchels and make no deals.
pub fn barter_person(
    world: &mut World, buyer: Entity, seller: Entity,
    want: &[(&str, u32)], offer: &[(&str, u32)],
) -> Option<Deal> {
    if crate::game::villager::is_dead(world, buyer) || crate::game::villager::is_dead(world, seller) {
        return None;
    }
    if want.is_empty() || offer.is_empty() { return None; }
    // bundles can name a good twice - fold them or the stock check and
    // the subtraction would read different numbers
    let merge = |bundle: &[(&str, u32)]| -> Vec<(String, u32)> {
        let mut out: Vec<(String, u32)> = vec![];
        for (g, q) in bundle {
            match out.iter_mut().find(|(og, _)| og == g) {
                Some((_, oq)) => *oq += q,
                None => out.push((g.to_string(), *q)),
            }
        }
        out
    };
    let want = merge(want);
    let offer = merge(offer);
    let worth_of = |bundle: &[(String, u32)]| bundle.iter()
        .map(|(g, q)| crate::game::goods::worth_at(world, g) * *q as f32).sum::<f32>();
    let want_worth = worth_of(&want);
    let offer_worth = worth_of(&offer);
    // the goods have to be there to trade
    let held = |who: Entity, bundle: &[(String, u32)]| bundle.iter().all(|(g, q)| {
        world.get_component::<Satchel>(who)
            .map(|s| s.goods.get(g.as_str()).copied().unwrap_or(0) >= *q).unwrap_or(false)
    });
    if !held(seller, &want) || !held(buyer, &offer) { return None; }
    let gap = offer_worth - want_worth;
    let mut boot = 0.0f32;
    if gap > 0.0 {
        // the buyer over-gave: the seller owes the difference to boot
        if gap > want_worth * (BARTER_OVERGIVE - 1.0) { return None; }
        let have = world.get_component::<Purse>(seller).map(|p| p.gold).unwrap_or(0.0);
        if have >= gap { boot = -gap; } else if gap > want_worth * BARTER_SLACK { return None; }
        // else the gap rides as goodwill - small change between neighbours
    } else {
        // the offer falls short: the buyer covers it, else a little
        // slack rides, else there's no deal
        let short = -gap;
        let have = world.get_component::<Purse>(buyer).map(|p| p.gold).unwrap_or(0.0);
        if have >= short { boot = short; } else if short > want_worth * BARTER_SLACK { return None; }
    }
    // goods cross the table
    for (g, q) in &offer {
        if let Some(s) = world.get_component_mut::<Satchel>(buyer) {
            *s.goods.get_mut(g.as_str()).unwrap() -= q;
        }
        if let Some(s) = world.get_component_mut::<Satchel>(seller) {
            *s.goods.entry(g.clone()).or_insert(0) += q;
        }
    }
    for (g, q) in &want {
        if let Some(s) = world.get_component_mut::<Satchel>(seller) {
            *s.goods.get_mut(g.as_str()).unwrap() -= q;
        }
        if let Some(s) = world.get_component_mut::<Satchel>(buyer) {
            *s.goods.entry(g.clone()).or_insert(0) += q;
        }
    }
    if boot != 0.0 {
        if let Some(p) = world.get_component_mut::<Purse>(buyer) { p.gold -= boot; }
        if let Some(p) = world.get_component_mut::<Purse>(seller) { p.gold += boot; }
    }
    // how fair it reads counts the coin too: a swap squared by the boot
    // is an even one, whatever the goods alone would say
    let gave = offer_worth + boot.max(0.0);
    let got = want_worth + (-boot).max(0.0);
    Some(Deal { boot, fairness: if got > 0.0 { gave / got } else { 1.0 } })
}

/// What the buyer's satchel could offer toward `target` worth: cheapest
/// goods first, never the good being asked for. Only what the purse
/// can't cover needs to come in goods - coppers to boot do the rest.
/// Takes whole units while the pile falls short, then trims back what
/// only overshoots.
pub fn pick_offer(world: &World, buyer: Entity, target: f32, not: &str) -> Vec<(String, u32)> {
    let purse = world.get_component::<Purse>(buyer).map(|p| p.gold).unwrap_or(0.0);
    let target = (target - purse).max(0.0);
    if target <= 0.0 { return vec![]; }
    let mut priced: Vec<(String, u32, f32)> = world.get_component::<Satchel>(buyer)
        .map(|s| s.goods.iter()
            .filter(|(g, q)| g.as_str() != not && **q > 0)
            .map(|(g, q)| (g.clone(), *q, crate::game::goods::worth_at(world, g)))
            .collect())
        .unwrap_or_default();
    priced.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    let mut offer: Vec<(String, u32, f32)> = vec![];
    let mut total = 0.0f32;
    'goods: for (g, qty, unit) in priced {
        for _ in 0..qty {
            if total >= target { break 'goods; }
            total += unit;
            offer.last_mut().filter(|(og, _, _)| *og == g)
                .map(|(_, oq, _)| *oq += 1)
                .unwrap_or_else(|| offer.push((g.clone(), 1, unit)));
        }
    }
    // then trim back: drop the cheapest units that only overshoot the
    // ask - three roots and a loaf when the loaf alone squares it
    loop {
        let Some((_, _, unit)) = offer.first() else { break };
        if total - unit < target { break; }
        total -= unit;
        if offer[0].1 <= 1 { offer.remove(0); } else { offer[0].1 -= 1; }
    }
    offer.into_iter().map(|(g, q, _)| (g, q)).collect()
}

/// How a swap reads in the log - goods both ways, coppers to boot, and
/// never a price named
pub fn barter_terms(offer: &[(String, u32)], want: &[(String, u32)], boot: f32) -> String {
    let bundle = |b: &[(String, u32)]| b.iter()
        .map(|(g, q)| if *q == 1 { g.clone() } else { format!("{} {}", q, g) })
        .collect::<Vec<_>>().join(" and ");
    let mut s = format!("{} for {}", bundle(offer), bundle(want));
    if boot > 0.0 { s.push_str(", and a few coppers to boot"); }
    else if boot < 0.0 { s.push_str(", a few coppers to boot the other way"); }
    s
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
    const EXOTICS: [&str; 12] = ["cloth", "paper", "ink", "pigment",
        "hops", "grapes", "wine", "spirits", "charcoal", "salt", "spice", "silk"];
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
    fn test_a_swap_moves_goods_both_ways() {
        let mut w = World::new();
        crate::game::goods::spawn_worths(&mut w);
        // two roots (worth 2) for three eggs (worth 15): the eggs are
        // dearer, so the boot has to cover it or the deal's off
        let eggler = worker(&mut w, vocation::Vocation::Farmer, 0.5, 0.0);
        w.get_component_mut::<Satchel>(eggler).unwrap().goods.insert("eggs".into(), 3);
        let gardner = worker(&mut w, vocation::Vocation::Baker, 0.5, 0.0);
        w.get_component_mut::<Satchel>(gardner).unwrap().goods.insert("roots".into(), 6);
        // six roots are worth 6 - still short of 15 and neither has coin
        assert!(barter_person(&mut w, gardner, eggler, &[("eggs", 3)], &[("roots", 6)]).is_none(),
            "no deal when the gap can't be covered");
        // but give the buyer coppers and the deal squares itself to boot
        w.get_component_mut::<Purse>(gardner).unwrap().gold = 10.0;
        let deal = barter_person(&mut w, gardner, eggler, &[("eggs", 3)], &[("roots", 6)]).unwrap();
        assert_eq!(deal.boot, 9.0, "15 worth of eggs less 6 of roots");
        assert_eq!(w.get_component::<Satchel>(gardner).unwrap().goods["eggs"], 3);
        assert_eq!(w.get_component::<Satchel>(gardner).unwrap().goods["roots"], 0);
        assert_eq!(w.get_component::<Satchel>(eggler).unwrap().goods["roots"], 6);
        assert_eq!(w.get_component::<Satchel>(eggler).unwrap().goods["eggs"], 0);
        assert_eq!(w.get_component::<Purse>(eggler).unwrap().gold, 9.0);
        // squared by the boot, the deal reads dead even
        assert!((deal.fairness - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_a_little_slack_rides_between_neighbours() {
        let mut w = World::new();
        crate::game::goods::spawn_worths(&mut w);
        // offer worth 1 short of the ask: inside the slack, no coin needed
        let a = worker(&mut w, vocation::Vocation::Farmer, 0.5, 0.0);
        w.get_component_mut::<Satchel>(a).unwrap().goods.insert("eggs".into(), 1);
        let b = worker(&mut w, vocation::Vocation::Baker, 0.5, 0.0);
        w.get_component_mut::<Satchel>(b).unwrap().goods.insert("vinegar".into(), 2);
        // eggs worth 5, two vinegar worth 4 - a tenth under, it rides
        let deal = barter_person(&mut w, b, a, &[("eggs", 1)], &[("vinegar", 2)]).unwrap();
        assert_eq!(deal.boot, 0.0, "a little short rides as goodwill");
        assert_eq!(w.get_component::<Satchel>(b).unwrap().goods["eggs"], 1);
    }

    #[test]
    fn test_an_overgenerous_offer_is_no_offer() {
        let mut w = World::new();
        crate::game::goods::spawn_worths(&mut w);
        let a = worker(&mut w, vocation::Vocation::Farmer, 0.5, 0.0);
        w.get_component_mut::<Satchel>(a).unwrap().goods.insert("eggs".into(), 1);
        let b = worker(&mut w, vocation::Vocation::Luthier, 0.5, 0.0);
        w.get_component_mut::<Satchel>(b).unwrap().goods.insert("lute".into(), 1);
        // a lute (25) for an egg (5) is a gift, not a swap
        assert!(barter_person(&mut w, b, a, &[("eggs", 1)], &[("lute", 1)]).is_none());
    }

    #[test]
    fn test_pick_offer_takes_cheap_goods_first() {
        let mut w = World::new();
        crate::game::goods::spawn_worths(&mut w);
        let b = worker(&mut w, vocation::Vocation::Baker, 0.5, 0.0);
        {
            let s = w.get_component_mut::<Satchel>(b).unwrap();
            s.goods.insert("roots".into(), 3);   // 1 each
            s.goods.insert("bread".into(), 2);   // 5 each
            s.goods.insert("tools".into(), 1);   // 15
        }
        // toward 5 worth: the loaf alone squares it - the roots get
        // trimmed back as overshoot
        let offer = pick_offer(&w, b, 5.0, "eggs");
        assert_eq!(offer, vec![("bread".to_string(), 1)]);
        // and it never offers the good being asked for
        let offer2 = pick_offer(&w, b, 5.0, "bread");
        assert!(!offer2.iter().any(|(g, _)| g == "bread"));
    }

    #[test]
    fn test_the_dead_make_no_deals() {
        let mut w = World::new();
        let a = worker(&mut w, vocation::Vocation::Farmer, 0.5, 0.0);
        w.get_component_mut::<Satchel>(a).unwrap().goods.insert("eggs".into(), 1);
        let b = worker(&mut w, vocation::Vocation::Baker, 0.5, 0.0);
        w.get_component_mut::<Satchel>(b).unwrap().goods.insert("bread".into(), 1);
        w.add_component(b, crate::game::villager::Dead);
        assert!(barter_person(&mut w, a, b, &[("bread", 1)], &[("eggs", 1)]).is_none());
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
