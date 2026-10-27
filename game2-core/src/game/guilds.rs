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
        social_capital: 0.0, // set below from membership
        charter: "Mutual protection of trade".into(),
    });
    protect_members(world, g);
    g
}

/// Social capital each member contributes to the guild's enforcement power
pub const CAPITAL_PER_MEMBER: f32 = 0.5;

/// The guild's protection covers its members: they are no longer exposed
/// to confiscation, and its social capital tracks its membership
fn protect_members(world: &mut World, guild: Entity) {
    let members = match world.get_component_mut::<Guild>(guild) {
        Some(g) => {
            g.social_capital = g.members.len() as f32 * CAPITAL_PER_MEMBER;
            g.members.clone()
        }
        None => return,
    };
    for m in members {
        if let Some(mer) = world.get_component_mut::<Merchant>(m) {
            mer.vulnerable = false;
        }
    }
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

/// System: vulnerable merchants spontaneously organize once insecurity is
/// widespread (guild_should_form); afterwards, newly vulnerable merchants
/// seek the existing guild's protection and join it
pub struct GuildFormationSystem;
impl crate::engine::ecs::System for GuildFormationSystem {
    fn run(&mut self, world: &mut World) {
        let merchants: Vec<Entity> = world.query::<Merchant>();
        let vuln: Vec<Entity> = merchants.iter().copied()
            .filter(|m| world.get_component::<Merchant>(*m).map(|x| x.vulnerable).unwrap_or(false))
            .collect();
        match world.query::<Guild>().first().copied() {
            None => {
                let all: Vec<&Merchant> = merchants.iter()
                    .filter_map(|m| world.get_component::<Merchant>(*m))
                    .collect();
                if vuln.len() >= 2 && guild_should_form(&all) {
                    form_guild(world, "Merchants Guild", vuln);
                }
            }
            Some(g) if !vuln.is_empty() => {
                if let Some(guild) = world.get_component_mut::<Guild>(g) {
                    for m in vuln {
                        if !guild.members.contains(&m) { guild.members.push(m); }
                    }
                }
                protect_members(world, g);
            }
            Some(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::System;

    fn merchant(w: &mut World, wealth: f32, vulnerable: bool) -> Entity {
        let m = w.create_entity();
        w.add_component(m, Merchant { name: "m".into(), wealth, trade_routes: vec![], vulnerable });
        m
    }

    #[test]
    fn test_poor_merchants_do_not_form_guild() {
        let mut w = World::new();
        merchant(&mut w, 20.0, true);
        merchant(&mut w, 20.0, true);
        GuildFormationSystem.run(&mut w);
        assert!(w.query::<Guild>().is_empty(), "avg wealth below 50 shouldn't support a guild");
    }

    #[test]
    fn test_guild_protects_members() {
        let mut w = World::new();
        let a = merchant(&mut w, 60.0, true);
        let b = merchant(&mut w, 60.0, true);
        let safe = merchant(&mut w, 60.0, false);
        GuildFormationSystem.run(&mut w);
        let g = w.query::<Guild>()[0];
        let guild = w.get_component::<Guild>(g).unwrap();
        assert_eq!(guild.members.len(), 2);
        assert!(guild.members.contains(&a) && guild.members.contains(&b));
        assert_eq!(guild.social_capital, 1.0);
        assert!(!w.get_component::<Merchant>(a).unwrap().vulnerable);
        assert_eq!(enforce_security(&mut w, g, safe), 0.0);
    }

    #[test]
    fn test_newly_vulnerable_merchant_joins_and_capital_grows() {
        let mut w = World::new();
        merchant(&mut w, 60.0, true);
        merchant(&mut w, 60.0, true);
        let late = merchant(&mut w, 60.0, false);
        GuildFormationSystem.run(&mut w);
        w.get_component_mut::<Merchant>(late).unwrap().vulnerable = true;
        GuildFormationSystem.run(&mut w);
        assert_eq!(w.query::<Guild>().len(), 1);
        let g = w.query::<Guild>()[0];
        let guild = w.get_component::<Guild>(g).unwrap();
        assert!(guild.members.contains(&late));
        assert_eq!(guild.social_capital, 1.5);
        assert!(!w.get_component::<Merchant>(late).unwrap().vulnerable);
    }
}
