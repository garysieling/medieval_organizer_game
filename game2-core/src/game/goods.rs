//! Goods: the registry of what's traded, what tier it lives on, what it
//! can fill in a recipe slot, and what it's worth when nobody's counting
//! coin. The worth stays hidden - the village speaks in swaps ("two
//! roots for three eggs"), the registry only keeps the ledger fair.

/// Which tier a good sits on: staples everyone needs, manufactured
/// craft goods, and luxuries that answer a want rather than a need
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// What every household wants weekly - food, fuel, fibre, light-makings
    Staple,
    /// What a craft turns staples into - drink, cloth, tools, candles
    Crafted,
    /// What answers a want - scent, silk, spectacles, the water of life
    Luxury,
}

/// A slot a good fills - the tags let a recipe ask "any grain?" instead of
/// "wheat?", so the web holds when one good runs out
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    /// Wheat, rye, barley - the mill's and the oven's slot
    Grain,
    /// Wool, flax - what the loom takes
    Fibre,
    /// Faggots, turf, dung, charcoal - what the hearth takes
    Fuel,
    /// Rushlights, candles, tapers - what fights the dark
    Light,
    /// Ale, beer, cider, mead, wine, spirits - what pours
    Drink,
    /// Fills the belly: bread, pottage, roots, peas, eggs, cheese, fish...
    StapleFood,
    /// Answers Want::Finery - garments, ribbon, silk
    Finery,
    /// Paper or vellum - the printer's slot
    Writing,
    /// Apples and the like - the press's and the table's slot
    Fruit,
    /// Honey and sugar-work - the sweet slot
    Sweet,
    /// Frankincense, herbs for the censer - the church's smoke
    Incense,
    /// A service, not a thing: faith, security, remedies
    Service,
    /// Goes off: bread sours, ale turns, milk won't keep
    Perishable,
}

/// The registry entry for one good: its tier, its hidden worth in gold
/// when the season's ordinary, and the slots it can fill
#[derive(Debug, Clone, Copy)]
pub struct Spec {
    pub tier: Tier,
    /// The everyday worth the ledger uses to balance a swap - never shown
    pub worth: f32,
    pub tags: &'static [Tag],
}

const fn s(tier: Tier, worth: f32, tags: &'static [Tag]) -> Spec {
    Spec { tier, worth, tags }
}

use Tag::*;
use Tier::*;

/// Every named good, its tier, hidden worth and slots. Worths keep the
/// numbers the old fixed-price table carried, so a swap prices the same
/// until scarcity moves it. New goods fill out the three tiers per the
/// goods-market-economy design doc.
pub fn spec(good: &str) -> Option<Spec> {
    Some(match good {
        // --- Tier 1: staples ------------------------------------------------
        "sand" => s(Staple, 1.0, &[]),
        "marigold" | "lavender" => s(Staple, 1.0, &[Incense]),
        "feverfew" => s(Staple, 5.0, &[Incense]),
        "wheat" | "rye" | "barley" => s(Staple, 2.0, &[Grain, StapleFood]),
        "peas" => s(Staple, 2.0, &[StapleFood]),
        "roots" | "potherbs" => s(Staple, 1.0, &[StapleFood]),
        // goods that rode the old table's 5.0 catch-all keep it for now -
        // the design's real staple worths (eggs ~2, milk ~2) land with the
        // repricing tranche, not smuggled through the plumbing
        "mushrooms" => s(Staple, 5.0, &[StapleFood, Perishable]),
        "chanterelle" => s(Staple, 5.0, &[StapleFood, Perishable]),
        "eggs" => s(Staple, 5.0, &[StapleFood, Perishable]),
        "milk" => s(Staple, 5.0, &[StapleFood, Perishable]),
        "butter" => s(Staple, 5.0, &[StapleFood, Perishable]),
        "cheese" => s(Staple, 5.0, &[StapleFood]),
        "fish" => s(Staple, 3.0, &[StapleFood, Perishable]),
        "apples" => s(Staple, 2.0, &[Fruit, StapleFood]),
        "honey" => s(Staple, 5.0, &[Sweet, StapleFood]),
        "wool" => s(Staple, 5.0, &[Fibre]),
        "flax" => s(Staple, 2.0, &[Fibre]),
        "hides" => s(Staple, 2.0, &[]),
        "tallow" => s(Staple, 5.0, &[]),
        "wax" => s(Staple, 5.0, &[]),
        "hay" | "straw" | "rushes" => s(Staple, 1.0, &[]),
        "faggots" => s(Staple, 1.0, &[Fuel]),
        "turf" => s(Staple, 1.0, &[Fuel]),
        "dung" => s(Staple, 1.0, &[Fuel]),
        "salt" => s(Staple, 3.0, &[]),
        "timber" | "wood" => s(Staple, 3.0, &[]),
        "driftwood" => s(Staple, 5.0, &[Fuel]),
        "fieldstone" | "stone" => s(Staple, 2.0, &[]),

        // --- Tier 2: manufactured -------------------------------------------
        "bread" => s(Crafted, 5.0, &[StapleFood, Perishable]),
        "pottage" => s(Crafted, 3.0, &[StapleFood, Perishable]),
        "sausages" => s(Crafted, 4.0, &[StapleFood]),
        "ale" => s(Crafted, 4.0, &[Drink, Perishable]),
        "beer" => s(Crafted, 7.0, &[Drink]),
        "cider" => s(Crafted, 5.0, &[Drink]),
        "mead" => s(Crafted, 8.0, &[Drink]),
        "wine" => s(Crafted, 12.0, &[Drink]),
        "vinegar" => s(Crafted, 2.0, &[]),
        "cloth" => s(Crafted, 6.0, &[]),
        "linen" => s(Crafted, 5.0, &[]),
        "leather" => s(Crafted, 4.0, &[]),
        "garments" => s(Crafted, 14.0, &[Finery]),
        "blocks" => s(Crafted, 4.0, &[]),
        "glass" => s(Crafted, 10.0, &[]),
        "paint" => s(Crafted, 8.0, &[]),
        "pigment" => s(Crafted, 5.0, &[]),
        "pamphlets" => s(Crafted, 3.0, &[]),
        "playing-cards" => s(Crafted, 4.0, &[]),
        "paper" => s(Crafted, 4.0, &[Writing]),
        "vellum" => s(Crafted, 5.0, &[Writing]),
        "ink" => s(Crafted, 4.0, &[]),
        "tools" => s(Crafted, 15.0, &[]),
        "casks" => s(Crafted, 8.0, &[]),
        "soap" => s(Crafted, 2.0, &[]),
        "baskets" => s(Crafted, 2.0, &[]),
        "treen" => s(Crafted, 2.0, &[]),
        "crocks" => s(Crafted, 2.0, &[]),
        "rope" => s(Crafted, 3.0, &[]),
        "rushlights" => s(Crafted, 1.0, &[Light]),
        "candles" => s(Crafted, 3.0, &[Light]),
        "tapers" => s(Crafted, 5.0, &[Light]),
        "charcoal" => s(Crafted, 3.0, &[Fuel]),
        "hops" => s(Crafted, 4.0, &[]),
        "grapes" => s(Crafted, 5.0, &[Fruit]),
        "needle" => s(Crafted, 5.0, &[]),
        "remedies" => s(Crafted, 5.0, &[Service]),
        "security" => s(Crafted, 8.0, &[Service]),
        "faith" => s(Crafted, 4.0, &[Service]),

        // --- Tier 3: luxuries -------------------------------------------------
        "ribbon" => s(Luxury, 5.0, &[Finery]),
        "perfume" => s(Luxury, 20.0, &[]),
        "rosewater" => s(Luxury, 8.0, &[]),
        "pomander" => s(Luxury, 10.0, &[]),
        "incense" => s(Luxury, 12.0, &[Incense]),
        "spice" => s(Luxury, 15.0, &[]),
        "marchpane" => s(Luxury, 10.0, &[Sweet]),
        "hippocras" => s(Luxury, 18.0, &[Drink]),
        "spirits" => s(Luxury, 25.0, &[Drink]),
        "silk" => s(Luxury, 20.0, &[Finery]),
        "lute" => s(Luxury, 25.0, &[]),
        "lute_string" => s(Luxury, 5.0, &[]),
        "spectacles" => s(Luxury, 20.0, &[]),
        "mirrors" => s(Luxury, 18.0, &[]),
        "fox-pelts" => s(Luxury, 8.0, &[]),
        "amber" => s(Luxury, 15.0, &[]),
        "whirligigs" => s(Luxury, 4.0, &[]),
        "pilgrim-badges" => s(Luxury, 4.0, &[]),
        "aged-cheese" => s(Luxury, 12.0, &[StapleFood]),

        _ => return None,
    })
}

/// What the ledger says a good is worth on an ordinary day. Registered
/// goods use the registry; anything else falls back to the trade's old
/// price table so nothing reprices by accident.
pub fn worth(good: &str) -> f32 {
    spec(good).map(|sp| sp.worth).unwrap_or_else(|| crate::game::vocation::price(good))
}

/// The village's felt scarcity: a drift on each good's worth that leans
/// with what the shelves show - a bare shelf dear, a full one cheap. It
/// never gets quoted; the street says "dear this year" and the ledger
/// just leans. Bounded and damped: this is systems weather, not a
/// trading sim.
#[derive(Debug, Default)]
pub struct Worths {
    /// good -> multiplier on its registry worth (1.0 = ordinary day)
    pub mult: std::collections::HashMap<String, f32>,
}

/// How far either way the felt worth can lean - muted on purpose
pub const WORTH_LO: f32 = 0.6;
pub const WORTH_HI: f32 = 1.8;
/// A day's mean-reversion: a pressure fades about this much each day
pub const WORTH_DRIFT: f32 = 0.04;
/// The lean a shelf just emptied puts on a good
pub const BARE_NUDGE: f32 = 0.05;
/// The lean a shelf too full to take more puts on a good
pub const GLUT_NUDGE: f32 = -0.03;

/// The one worths ledger (a singleton component, like the climate)
pub fn spawn_worths(world: &mut crate::engine::ecs::World) -> crate::engine::ecs::Entity {
    let e = world.create_entity();
    world.add_component(e, Worths::default());
    e
}

fn worths_entity(world: &crate::engine::ecs::World) -> Option<crate::engine::ecs::Entity> {
    world.query::<Worths>().first().copied()
}

/// What a good's worth today: registry worth leaned by the felt scarcity
pub fn worth_at(world: &crate::engine::ecs::World, good: &str) -> f32 {
    let m = worths_entity(world)
        .and_then(|e| world.get_component::<Worths>(e))
        .and_then(|w| w.mult.get(good).copied())
        .unwrap_or(1.0);
    worth(good) * m
}

/// Lean a good's felt worth by `delta` (clamped to the muted band)
pub fn nudge(world: &mut crate::engine::ecs::World, good: &str, delta: f32) {
    let Some(e) = worths_entity(world) else { return };
    if let Some(w) = world.get_component_mut::<Worths>(e) {
        let m = (w.mult.get(good).copied().unwrap_or(1.0) + delta).clamp(WORTH_LO, WORTH_HI);
        w.mult.insert(good.to_string(), m);
    }
}

/// A day passes: every lean slides a little back toward ordinary
pub fn drift(world: &mut crate::engine::ecs::World) {
    let Some(e) = worths_entity(world) else { return };
    if let Some(w) = world.get_component_mut::<Worths>(e) {
        for m in w.mult.values_mut() {
            *m += (1.0 - *m) * WORTH_DRIFT;
        }
    }
}

/// Which tier a good lives on (unknown goods count as craft goods)
pub fn tier(good: &str) -> Tier {
    spec(good).map(|sp| sp.tier).unwrap_or(Tier::Crafted)
}

/// Whether a good fills a slot ("does rye count as grain?")
pub fn fills(good: &str, slot: Tag) -> bool {
    spec(good).map(|sp| sp.tags.contains(&slot)).unwrap_or(false)
}

/// The goods a slot accepts - for offers, wants and recipe matching
pub fn slot_goods(slot: Tag) -> Vec<&'static str> {
    ALL.iter().copied().filter(|&g| spec(g).map(|sp| sp.tags.contains(&slot)).unwrap_or(false)).collect()
}

/// Every good the registry knows, in table order
pub const ALL: &[&str] = &[
    // staples
    "sand", "marigold", "lavender", "feverfew", "wheat", "rye", "barley",
    "peas", "roots", "potherbs", "mushrooms", "chanterelle", "eggs", "milk",
    "butter", "cheese", "fish", "apples", "honey", "wool", "flax", "hides",
    "tallow", "wax", "hay", "straw", "rushes", "faggots", "turf", "dung",
    "salt", "timber", "driftwood", "fieldstone",
    // manufactured
    "bread", "pottage", "sausages", "ale", "beer", "cider", "mead", "wine",
    "vinegar", "cloth", "linen", "leather", "garments", "blocks", "glass",
    "paint", "pigment", "pamphlets", "playing-cards", "paper", "vellum",
    "ink", "tools", "casks", "soap", "baskets", "treen", "crocks", "rope",
    "rushlights", "candles", "tapers", "charcoal", "hops", "grapes",
    "needle", "remedies", "security", "faith",
    // luxuries
    "ribbon", "perfume", "rosewater", "pomander", "incense", "spice",
    "marchpane", "hippocras", "spirits", "silk", "lute", "lute_string",
    "spectacles", "mirrors", "fox-pelts", "amber", "whirligigs",
    "pilgrim-badges", "aged-cheese",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_registered_good_has_a_spec() {
        for g in ALL {
            let sp = spec(g).unwrap_or_else(|| panic!("{} has no spec", g));
            assert!(sp.worth > 0.0, "{} has no worth", g);
        }
    }

    #[test]
    fn test_unchanged_goods_keep_their_old_price() {
        // every good the old table priced by name keeps its number; goods
        // that rode the 5.0 catch-all (eggs, milk, honey...) get their own
        // worths now - that's the registry doing its job
        for (g, old) in [("sand", 1.0), ("wheat", 2.0), ("stone", 2.0), ("wood", 3.0),
                         ("blocks", 4.0), ("paper", 4.0), ("ink", 4.0), ("faith", 4.0),
                         ("pamphlets", 3.0), ("pigment", 5.0), ("bread", 5.0),
                         ("cloth", 6.0), ("perfume", 20.0), ("paint", 8.0), ("fish", 3.0),
                         ("marigold", 1.0), ("lavender", 1.0), ("security", 8.0),
                         ("glass", 10.0), ("garments", 14.0), ("tools", 15.0),
                         ("lute", 25.0), ("ale", 4.0), ("beer", 7.0), ("wine", 12.0),
                         ("spirits", 25.0), ("vinegar", 2.0), ("hops", 4.0),
                         ("grapes", 5.0), ("casks", 8.0)] {
            assert_eq!(worth(g), old, "{} repriced", g);
            assert_eq!(worth(g), crate::game::vocation::price(g));
        }
    }

    #[test]
    fn test_the_tiers_read_right() {
        assert_eq!(tier("roots"), Tier::Staple);
        assert_eq!(tier("bread"), Tier::Crafted);
        assert_eq!(tier("perfume"), Tier::Luxury);
        assert_eq!(tier("something-unnamed"), Tier::Crafted);
    }

    #[test]
    fn test_slots_take_their_alternates() {
        // any grain feeds the oven - the web, not the chain
        assert!(fills("rye", Tag::Grain));
        assert!(fills("barley", Tag::Grain));
        assert!(fills("wheat", Tag::Grain));
        assert!(!fills("peas", Tag::Grain));
        // the hearth burns what's to hand
        assert!(fills("faggots", Tag::Fuel));
        assert!(fills("turf", Tag::Fuel));
        assert!(fills("dung", Tag::Fuel));
        assert!(fills("charcoal", Tag::Fuel));
        assert!(!fills("timber", Tag::Fuel), "craft-wood isn't fuel-wood");
        // the printer takes either leaf
        assert!(fills("paper", Tag::Writing));
        assert!(fills("vellum", Tag::Writing));
        assert!(!fills("parchment", Tag::Writing));
        // every slot has more than one filler, or it's a chain not a web
        for slot in [Tag::Grain, Tag::Fibre, Tag::Fuel, Tag::Light, Tag::Drink, Tag::Writing] {
            assert!(slot_goods(slot).len() >= 2, "slot {:?} has one road in", slot);
        }
    }

    #[test]
    fn test_staples_stay_cheap_and_luxuries_stay_dear() {
        for g in ALL {
            let sp = spec(g).unwrap();
            match sp.tier {
                // 5.0 is the legacy catch-all some staples still carry;
                // the repricing tranche pulls them under 4
                Tier::Staple => assert!(sp.worth <= 5.0, "staple {} too dear", g),
                Tier::Luxury => assert!(sp.worth >= 4.0, "luxury {} too cheap", g),
                Tier::Crafted => {}
            }
        }
    }

    #[test]
    fn test_the_felt_worth_starts_ordinary() {
        let mut w = crate::engine::ecs::World::new();
        spawn_worths(&mut w);
        assert_eq!(worth_at(&w, "bread"), worth("bread"));
    }

    #[test]
    fn test_a_nudge_leans_and_the_days_slide_it_back() {
        let mut w = crate::engine::ecs::World::new();
        spawn_worths(&mut w);
        nudge(&mut w, "bread", 0.3);
        assert!((worth_at(&w, "bread") - worth("bread") * 1.3).abs() < 0.01);
        for _ in 0..150 { drift(&mut w); }
        assert!((worth_at(&w, "bread") - worth("bread")).abs() < 0.05,
            "a season and a half on, the lean is all but forgotten");
    }

    #[test]
    fn test_the_lean_stays_muted() {
        let mut w = crate::engine::ecs::World::new();
        spawn_worths(&mut w);
        for _ in 0..100 { nudge(&mut w, "bread", 0.5); }
        assert_eq!(worth_at(&w, "bread"), worth("bread") * WORTH_HI);
        for _ in 0..100 { nudge(&mut w, "bread", -0.5); }
        assert_eq!(worth_at(&w, "bread"), worth("bread") * WORTH_LO);
    }

    #[test]
    fn test_unlisted_goods_fall_back_to_the_old_table() {
        assert_eq!(worth("quill"), crate::game::vocation::price("quill"));
        assert!(!fills("quill", Tag::Grain));
    }
}
