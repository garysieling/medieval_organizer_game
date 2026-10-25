//! C4: Guild formation live - vulnerable merchants spontaneously organize.
//! Greif/Milgrom/Weingast model: guilds emerge to secure trade.

use game2_core::engine::ecs::{Scheduler, World};
use game2_core::game::guilds::{Merchant, Guild, GuildFormationSystem, enforce_security};

fn main() {
    println!("=== GUILD FORMATION (emergent) ===\n");
    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(GuildFormationSystem);

    // Spawn merchants - some vulnerable, some not
    let names = ["Wool Trader", "Spice Merchant", "Grain Dealer", "Cloth Seller"];
    for (i, n) in names.iter().enumerate() {
        let m = world.create_entity();
        world.add_component(m, Merchant {
            name: n.to_string(),
            wealth: 60.0 + i as f32 * 10.0,
            trade_routes: vec!["north_road".into()],
            vulnerable: i < 3, // first 3 are vulnerable to confiscation
        });
    }

    println!("3 vulnerable merchants, 1 safe. Running GuildFormationSystem...\n");
    for tick in 0..3 {
        sched.tick(&mut world);
        let guilds = world.query::<Guild>();
        println!("tick {}: {} guild(s) exist", tick, guilds.len());
        if let Some(g) = guilds.first() {
            let (name, members, capital, first_member) = {
                let guild = world.get_component::<Guild>(*g).unwrap();
                (guild.name.clone(), guild.members.len(), guild.social_capital, guild.members[0])
            };
            println!("  '{}' formed: {} members, social_capital={:.1}", name, members, capital);
            let protection = enforce_security(&mut world, *g, first_member);
            println!("  member protection: {:.1}", protection);
        }
    }
    println!("\n=== GUILD EMERGED (not scripted) ===");
}
