//! The sanction ladder: shame before violence. When a villager's
//! grievances pile up - blamed for the bread, suspected of the cure that
//! failed - the village escalates: a word, then a cold shoulder, then
//! their trade dries up, then the stocks, then the road out. The crowd's
//! answer decides whether a rung lands or turns back on the accusers.


use crate::engine::ecs::Entity;

/// The reckoning's mark (#1255): who stood beside the condemned when
/// the rest of the crowd melted. A loyalty fact the lord's informers
/// can read later - standing in the open costs the stander their cover
#[derive(Debug, Clone, Copy)]
pub struct StoodBy {
    /// Whose walk they stood at
    pub of: Entity,
    /// The day the crowd watched it
    pub at: u64,
}

/// The rungs of the ladder, mildest to worst
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sanction {
    Gossip,
    Shun,
    Boycott,
    Picket,
    Stocks,
    Exile,
}

/// Every rung the ladder has, in order
pub const LADDER: [Sanction; 6] = [
    Sanction::Gossip, Sanction::Shun, Sanction::Boycott,
    Sanction::Picket, Sanction::Stocks, Sanction::Exile,
];

/// The rung a grievance count earns: blame doesn't go straight to the
/// stocks - the ladder is climbed
pub fn rung_for(grievances: usize) -> Option<Sanction> {
    match grievances {
        0..=2 => None,
        3..=4 => Some(Sanction::Gossip),
        5..=6 => Some(Sanction::Shun),
        7..=8 => Some(Sanction::Boycott),
        9..=11 => Some(Sanction::Picket),
        12..=14 => Some(Sanction::Stocks),
        _ => Some(Sanction::Exile),
    }
}

/// What a landed sanction is called in the log
pub fn label(s: Sanction) -> &'static str {
    match s {
        Sanction::Gossip => "the talk turns",
        Sanction::Shun => "the cold shoulder",
        Sanction::Boycott => "the boycott",
        Sanction::Picket => "a crowd at the door",
        Sanction::Stocks => "the stocks",
        Sanction::Exile => "driven out",
    }
}
