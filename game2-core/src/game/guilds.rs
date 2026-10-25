//! Guild formation: emerges from trade risk (Greif/Milgrom/Weingast model).
//! From lit review: guilds emerge to let rulers commit to merchant security.
//! Guilds provide social capital + credible commitment → enable trade.

use crate::engine::ecs::{Entity, World};

/// A merchant seeking security for trade
#[derive(Debug)]
pub struct Merchant {
    pub name: String,
    pub wealth: f32,
    pub trade_routes: Vec<String>,
    pub vulnerable: bool, // exposed to confiscation/robbery
}

/// A guild that emerges to make trade safe
#[derive(Debug)]
pub struct Guild {
    pub name: String,
    pub members: Vec<Entity>,
    pub treasury: f32,
    pub social_capital: f32, // enforcement power from relationships
    pub charter: String,
}

/// Condition for guild formation: merchants face insecurity
pub fn guild_should_form(merchants: &[&Merchant]) -> bool {
    // Guilds emerge when merchants are vulnerable to ruler confiscation
    let vulnerable_count = merchants.iter().filter(|m| m.vulnerable).count();
    let avg_wealth = merchants.iter().map(|m| m.wealth).sum::<f32>() / merchants.len().max(1) as f32;
    vulnerable_count > merchants.len() / 2 && avg_wealth > 50.0
}

/// Merchants form a guild (emergent organization)
pub fn form_guild(world: &mut World, name: &str, merchants: Vec<Entity>) -> Entity {
    let g = world.create_entity();
    world.add_component(g, Guild {
        name: name.into(),
        members: merchants,
        treasury: 0.0,
        social_capital: 1.0, // grows with membership + shared risk
        charter: "Mutual protection of trade".into(),
    });
    g
}

/// Guild enforces member security (social capital → enforcement)
pub fn enforce_security(world: &mut World, guild: Entity, member: Entity) -> f32 {
    if let Some(g) = world.get_component::<Guild>(guild) {
        if g.members.contains(&member) {
            return g.social_capital; // enforcement power
        }
    }
    0.0
}

/// System: vulnerable merchants spontaneously organize
pub struct GuildFormationSystem;
impl crate::engine::ecs::System for GuildFormationSystem {
    fn run(&mut self, world: &mut World) {
        let merchants: Vec<Entity> = world.query::<Merchant>();
        let vuln: Vec<Entity> = merchants.into_iter()
            .filter(|m| world.get_component::<Merchant>(*m).map(|x| x.vulnerable).unwrap_or(false))
            .collect();
        // If enough vulnerable merchants and no guild exists, one forms
        if vuln.len() >= 2 && world.query::<Guild>().is_empty() {
            form_guild(world, "Merchants Guild", vuln);
        }
    }
}
