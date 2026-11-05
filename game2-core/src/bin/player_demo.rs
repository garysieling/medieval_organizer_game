//! D1-D4: Player layer - outsider takes actions that change world state.
//! Talk and recruit: kept company earns trust, trust fills the guild.

use game2_core::engine::ecs::World;
use game2_core::game::villager::{self};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, rapport};
use game2_core::sim::community::{Citizen, Organization};

fn main() {
    println!("=== PLAYER LAYER: outsider earns trust ===\n");
    let mut world = World::new();

    // The player: an outsider with no authority
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter {
        name: "The Wanderer".into(), resources: 100.0,
    });

    // Villagers to interact with
    let names = ["Aldric","Eliza","Bram","Corvin"];
    let mut villagers = vec![];
    for (i, _n) in names.iter().enumerate() {
        let v = villager::spawn_villager(&mut world, (i*4) as f32, 0.0);
        world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        villagers.push(v);
    }

    // Player's organization
    let org = world.create_entity();
    world.add_component(org, Organization {
        name: "The Harvesters".into(), members: vec![], leader: player,
        influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into(),
    });

    // a campaign to recruit into
    let vote = game2_core::game::campaign::IssueVote { winner: "bread".into(), ballots: vec![] };
    game2_core::game::campaign::found_campaign(&mut world, org, &vote,
        game2_core::game::campaign::Framing::SharedGrievance,
        game2_core::game::campaign::DecisionRule::Majority, 1);

    println!("Turn | Action | Result | Rapport | Guild\n");
    for turn in 0..12 {
        let target = villagers[turn % 4];
        let (action_desc, result) = match turn % 3 {
            0 => ("Talk", act_talk(&mut world, player, target, turn as u64)),
            1 => ("Talk", act_talk(&mut world, player, target, turn as u64)),
            _ => ("Recruit", act_recruit(&mut world, player, org, target)),
        };
        let members = world.get_component::<Organization>(org).unwrap().members.len();
        println!("{} | {} | {} | rapport={:.2} | {} members", turn, action_desc,
            if result.success { "OK" } else { "FAIL" }, rapport(&world, target), members);
    }

    let members = world.get_component::<Organization>(org).unwrap().members.len();
    println!("\nHarvesters Guild: {} members", members);
    println!("=== PLAYER LAYER COMPLETE ===");
}
