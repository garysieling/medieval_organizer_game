//! Vocations, classes and skill: who does what for a living, and how good
//! they are at it. Classes run working -> petit bourgeois -> professional
//! middle (knights, priests, overseers, the tax collector) -> the off-map
//! landowner. Working folk hold a vocation and climb
//! journeyman -> apprentice -> master; masters who own their workshop are
//! petit bourgeois.

use crate::engine::ecs::Entity;

/// A character's social class
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SocialClass {
    Working,
    /// A master who owns their workshop
    PetitBourgeois,
    /// Knights, priests, overseers, the tax collector
    ProfessionalMiddle,
    /// The lord - off the map, seen through rents and taxes
    Landowner,
}

/// A working-class trade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vocation {
    Farmer,
    Baker,
    Toolmaker,
    Knight,
    Tailor,
    Glazier,
    Printmaker,
    Mason,
    Priest,
    Luthier,
    /// Keeps the fold out in the meadow: milk from the goats daily, wool
    /// off the sheep at shearing
    Shepherd,
    /// Keeps the skeps at the meadow's edge: honey in season, wax for
    /// the candles and the seals
    Beekeeper,
    /// Works the loom: the shepherd's wool in, village cloth out
    Weaver,
    /// The petit-bourgeois: garden flowers distilled to scent - a
    /// luxury that counters the smell of honest work
    Perfumer,
    /// Mixes pigment and lays it on wood: painted signs, murals, heraldry
    Painter,
    /// Works the river's edge: fish, eels, and the strange things the
    /// water brings up
    Fisher,
}

/// The PMC postings that aren't working-class vocations
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TownRole {
    /// The recluse at the cave - the dark keeps them
    Hermit,
    /// Drives a farm or mason's yard: boosts the work and takes a cut
    Overseer,
    /// Rounds the village on his yearly circuit, taking the lord's share
    TaxCollector,
    /// Sells the imported goods no one makes here from a stall at the market
    Trader,
    /// The wise woman in the herb hut outside the wall: forages, brews,
    /// and treats the sick who walk out to her
    Herbologist,
    /// The heart-helper: keeps an eye on lonely hearts and brings them
    /// together, for a pie and a good word
    Matchmaker,
}

/// How far up the trade someone is: journeyman, then apprentice, then master
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Journeyman,
    Apprentice,
    Master,
}

/// A person's craft: their vocation, their skill at it, the workshop they
/// run (if they have one) and the master they're paired with (if any)
#[derive(Debug)]
pub struct Craft {
    pub vocation: Vocation,
    pub skill: f32, // 0..1
    /// The building entity they work out of; None means no workshop - a
    /// problem, and their work rate is halved
    pub workplace: Option<Entity>,
    /// The master they learn under; journeymen and apprentices without one
    /// grow slowly and carry the NoMaster problem
    pub master: Option<Entity>,
    /// The senior who tasks them (a master tasks a journeyman or apprentice,
    /// an apprentice tasks a journeyman): their output partly goes to the
    /// senior's shop
    pub tasked_by: Option<Entity>,
    /// Part of a day's output made but not yet a whole unit - a half-rate
    /// day still counts toward the next one
    pub pending: f32,
}

/// A child: no craft, no level - they run errands and play
#[derive(Debug)]
pub struct Kid;

/// How a kid is growing: the days they've seen, and who keeps giving
/// them errands (the hirer they run for most often is their likeliest
/// master when they come of age)
#[derive(Debug)]
pub struct KidGrowth {
    pub age_days: u64,
    pub errands: Vec<(Entity, u32)>,
}

/// The day a kid comes of age: a couple of winters' errands makes a hand
pub const COMING_OF_AGE: u64 = 730;

/// The level a skill puts someone at
pub fn level_of(skill: f32) -> Level {
    if skill >= 0.75 { Level::Master } else if skill >= 0.4 { Level::Apprentice } else { Level::Journeyman }
}

/// How many interaction options a level gives (1/2/3 up the tree)
pub fn options_for(level: Level) -> usize {
    match level {
        Level::Journeyman => 1,
        Level::Apprentice => 2,
        Level::Master => 3,
    }
}

/// Whether a vocation is professional-middle rather than working class
pub fn is_pmc(v: Vocation) -> bool {
    matches!(v, Vocation::Knight | Vocation::Priest)
}

/// Raw goods a vocation needs in hand to work a day (empty for those who
/// work the land or give a service)
pub fn inputs(v: Vocation) -> &'static [&'static str] {
    match v {
        Vocation::Baker => &["wheat"],
        Vocation::Toolmaker => &["wood", "stone"],
        Vocation::Tailor => &["cloth"],
        Vocation::Weaver => &["wool"],
        Vocation::Perfumer => &["marigold", "lavender"],
        Vocation::Painter => &["pigment", "wood"],
        Vocation::Fisher => &[],
        Vocation::Glazier => &["sand", "pigment"],
        Vocation::Printmaker => &["paper", "ink"],
        Vocation::Mason => &["stone"],
        Vocation::Luthier => &["wood"],
        _ => &[],
    }
}

/// The good a vocation makes for stock and sale. Knights and priests give
/// services instead, which stock their posts the same way
pub fn output(v: Vocation) -> Option<&'static str> {
    match v {
        Vocation::Farmer => Some("wheat"),
        Vocation::Baker => Some("bread"),
        Vocation::Toolmaker => Some("tools"),
        Vocation::Knight => Some("security"),
        Vocation::Tailor => Some("garments"),
        Vocation::Glazier => Some("glass"),
        Vocation::Printmaker => Some("pamphlets"),
        Vocation::Mason => Some("blocks"),
        Vocation::Priest => Some("faith"),
        Vocation::Luthier => Some("lute"),
        Vocation::Shepherd => Some("milk"),
        Vocation::Beekeeper => Some("honey"),
        Vocation::Weaver => Some("cloth"),
        Vocation::Perfumer => Some("perfume"),
        Vocation::Painter => Some("paint"),
        Vocation::Fisher => Some("fish"),
    }
}

/// Whether the vocation's product is a service, not a stackable good
pub fn is_service(v: Vocation) -> bool {
    matches!(v, Vocation::Knight | Vocation::Priest)
}

/// Whether the good can be gathered from the land itself rather than
/// bought (wood from the trees, stone from the rocks, sand off the shore)
pub fn gatherable(good: &str) -> bool {
    matches!(good, "wood" | "stone" | "sand")
}

/// Whether the good only comes in on the road (the trader's stock)
pub fn imported(good: &str) -> bool {
    matches!(good, "cloth" | "paper" | "ink" | "pigment")
}

/// Everyday price of a good or service, in gold
pub fn price(good: &str) -> f32 {
    match good {
        "sand" => 1.0,
        "wheat" => 2.0,
        "stone" => 2.0,
        "wood" => 3.0,
        "blocks" => 4.0,
        "paper" => 4.0,
        "ink" => 4.0,
        "faith" => 4.0,
        "pamphlets" => 3.0,
        "pigment" => 5.0,
        "bread" => 5.0,
        "cloth" => 6.0,
        "perfume" => 20.0,
        "paint" => 8.0,
        "fish" => 3.0,
        "marigold" | "lavender" => 1.0,
        "security" => 8.0,
        "glass" => 10.0,
        "garments" => 14.0,
        "tools" => 15.0,
        "lute" => 25.0,
        _ => 5.0,
    }
}

/// The sign over a vocation's workplace
pub fn trade_name(v: Vocation) -> &'static str {
    match v {
        Vocation::Farmer => "the farm",
        Vocation::Baker => "the bakery",
        Vocation::Toolmaker => "the smithy",
        Vocation::Knight => "the watch house",
        Vocation::Tailor => "the tailor's",
        Vocation::Glazier => "the glazier's",
        Vocation::Printmaker => "the print shop",
        Vocation::Mason => "the mason's yard",
        Vocation::Priest => "the church",
        Vocation::Luthier => "the luthier's",
        Vocation::Shepherd => "the fold",
        Vocation::Beekeeper => "the apiary",
        Vocation::Weaver => "the loom house",
        Vocation::Perfumer => "the perfumery",
        Vocation::Painter => "the paint shop",
        Vocation::Fisher => "the fish-smoke house",
    }
}

/// What the trade is called on a character card
pub fn vocation_name(v: Vocation) -> &'static str {
    match v {
        Vocation::Farmer => "Farmer",
        Vocation::Baker => "Baker",
        Vocation::Toolmaker => "Toolmaker",
        Vocation::Knight => "Knight",
        Vocation::Tailor => "Tailor",
        Vocation::Glazier => "Glazier",
        Vocation::Printmaker => "Printmaker",
        Vocation::Mason => "Mason",
        Vocation::Priest => "Priest",
        Vocation::Luthier => "Luthier",
        Vocation::Shepherd => "Shepherd",
        Vocation::Beekeeper => "Beekeeper",
        Vocation::Weaver => "Weaver",
        Vocation::Perfumer => "Perfumer",
        Vocation::Painter => "Painter",
        Vocation::Fisher => "Fisher",
    }
}

pub fn level_name(l: Level) -> &'static str {
    match l {
        Level::Journeyman => "Journeyman",
        Level::Apprentice => "Apprentice",
        Level::Master => "Master",
    }
}

pub fn class_name(c: SocialClass) -> &'static str {
    match c {
        SocialClass::Working => "working",
        SocialClass::PetitBourgeois => "petit bourgeois",
        SocialClass::ProfessionalMiddle => "PMC",
        SocialClass::Landowner => "landowner",
    }
}

/// One thing someone at a level can do, from the level's option list
#[derive(Debug, Clone, Copy)]
pub struct VocationAction {
    pub name: &'static str,
    pub does: &'static str,
}

/// The actions a vocation opens up, level by level (options_for() of them
/// are live at each level - the higher levels add the rest)
const fn a(name: &'static str, does: &'static str) -> VocationAction {
    VocationAction { name, does }
}

pub fn actions(v: Vocation) -> Vec<VocationAction> {
    match v {
        Vocation::Farmer => vec![
            a("sell crops", "sell wheat to whoever's buying"),
            a("hire a hand", "task a journeyman to work the rows"),
            a("lease land", "rent another field off the lord"),
        ],
        Vocation::Baker => vec![
            a("bake bread", "turn wheat into loaves"),
            a("send for grain", "task a journeyman to fetch wheat"),
            a("festival loaves", "a big run for a feast"),
        ],
        Vocation::Toolmaker => vec![
            a("forge tools", "make tools folk work faster with"),
            a("mend tools", "repair what the village wears out"),
            a("fine set", "a masterwork set for a patron"),
        ],
        Vocation::Knight => vec![
            a("walk the watch", "patrol and sell protection"),
            a("escort", "see someone safe on the road"),
            a("drill", "train a journeyman-at-arms"),
        ],
        Vocation::Tailor => vec![
            a("mend clothes", "repair garments"),
            a("cut garments", "make new clothes to order"),
            a("fine cloth", "a festival outfit"),
        ],
        Vocation::Glazier => vec![
            a("cut glass", "make panes and pieces"),
            a("lead a window", "a church commission"),
            a("masterpiece", "a window to be remembered by"),
        ],
        Vocation::Printmaker => vec![
            a("print", "run pamphlets and bills"),
            a("a run", "a big printing for a cause"),
            a("the press", "set the type himself"),
        ],
        Vocation::Mason => vec![
            a("cut stone", "dress blocks for building"),
            a("lay a wall", "build and repair"),
            a("grand work", "raise something that lasts"),
        ],
        Vocation::Priest => vec![
            a("hold service", "keep the faith for the village"),
            a("counsel", "hear people out"),
            a("consecrate", "bless a great work"),
        ],
        Vocation::Luthier => vec![
            a("build lutes", "make instruments"),
            a("repair", "fix a worn instrument"),
            a("masterwork", "an instrument to be treasured"),
        ],
        Vocation::Shepherd => vec![
            a("walk the flock", "drive the animals to fresh grazing"),
            a("milk the goats", "a pail a day from the fold"),
            a("shear the flock", "wool off their backs at the season"),
        ],
        Vocation::Beekeeper => vec![
            a("tend the skeps", "smoke the hives and check the queens"),
            a("draw honey", "a comb a season from each skep"),
            a("render wax", "for the candles and the seals"),
        ],
        Vocation::Weaver => vec![
            a("work the loom", "wool in, cloth out"),
            a("buy wool at the fold", "the shepherd keeps them supplied"),
            a("sell cloth", "a bolt for the tailor when it's cut"),
        ],
        Vocation::Perfumer => vec![
            a("distill the flowers", "marigold and lavender under the still"),
            a("sell scent", "a luxury that counters honest work's smell"),
            a("buy garden stock", "a bunch of marigold for the still"),
        ],
        Vocation::Painter => vec![
            a("mix pigment", "madder red, woad blue, a ground of chalk"),
            a("lay it on wood", "a painted sign is work that shows"),
            a("take commissions", "the lord's hall wants its colors"),
        ],
        Vocation::Fisher => vec![
            a("set the eel-pots", "the river runs full of dark water"),
            a("smoke the catch", "the day's take hung over embers"),
            a("mend the nets", "the river takes its share of line"),
        ],
    }
}

/// Skill gained per day worked: x10 under a master
pub const SKILL_TICK: f32 = 0.005;
pub const MENTOR_MULT: f32 = 10.0;

/// A day's skill growth at the craft
pub fn skill_gain(has_master: bool) -> f32 {
    SKILL_TICK * if has_master { MENTOR_MULT } else { 1.0 }
}

/// A day's work at the craft: skill climbs (fast under a master), and the
/// level comes off the new skill. Returns the level if it changed
pub fn train_day(craft: &mut Craft) -> Option<Level> {
    let before = level_of(craft.skill);
    craft.skill = (craft.skill + skill_gain(craft.master.is_some())).min(1.0);
    let after = level_of(craft.skill);
    if after != before { Some(after) } else { None }
}

/// Seed the craft for a character
pub fn craft(vocation: Vocation, skill: f32) -> Craft {
    Craft { vocation, skill, workplace: None, master: None, tasked_by: None, pending: 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levels_climb_the_tree() {
        assert_eq!(level_of(0.2), Level::Journeyman);
        assert_eq!(level_of(0.5), Level::Apprentice);
        assert_eq!(level_of(0.9), Level::Master);
        assert_eq!(options_for(Level::Journeyman), 1);
        assert_eq!(options_for(Level::Apprentice), 2);
        assert_eq!(options_for(Level::Master), 3);
    }

    #[test]
    fn test_a_master_grows_you_ten_times_faster() {
        let mut lone = craft(Vocation::Baker, 0.3);
        let mut taught = craft(Vocation::Baker, 0.3);
        taught.master = Some(Entity::default());
        train_day(&mut lone);
        train_day(&mut taught);
        assert!((taught.skill - lone.skill - SKILL_TICK * 9.0).abs() < 1e-6);
    }

    #[test]
    fn test_every_vocation_has_a_sign_and_a_product() {
        for v in [Vocation::Farmer, Vocation::Baker, Vocation::Toolmaker, Vocation::Knight,
                  Vocation::Tailor, Vocation::Glazier, Vocation::Printmaker, Vocation::Mason,
                  Vocation::Priest, Vocation::Luthier] {
            assert!(!trade_name(v).is_empty());
            assert!(output(v).is_some(), "{:?} makes nothing", v);
            assert_eq!(actions(v).len(), 3, "{:?} should have three actions", v);
            assert!(price(output(v).unwrap()) > 0.0);
        }
        // goods vocations' inputs are buyable, gatherable or imported
        for v in [Vocation::Baker, Vocation::Toolmaker, Vocation::Tailor, Vocation::Glazier,
                  Vocation::Printmaker, Vocation::Mason, Vocation::Luthier] {
            for i in inputs(v) {
                assert!(gatherable(i) || imported(i) || *i == "wheat",
                    "{:?} needs {} which no one makes or brings", v, i);
            }
        }
    }
}
