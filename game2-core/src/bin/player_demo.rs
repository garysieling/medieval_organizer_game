//! D1-D4: Player layer - outsider takes actions that change world state.
//! Talk and recruit: kept company earns trust, trust fills the guild.

use game2_core::engine::ecs::World;
use game2_core::game::villager::{self};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, rapport, RECRUIT_RAPPORT_NEEDED};
use game2_core::sim::community::{Citizen, Organization};

fn main() {
    game2_core::infra::crash::install();
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
        influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into(), successors: vec![] });

    // a campaign to recruit into
    let vote = game2_core::game::campaign::IssueVote { winner: "bread".into(), ballots: vec![] };
    game2_core::game::campaign::found_campaign(&mut world, org, &vote,
        game2_core::game::campaign::Framing::SharedGrievance,
        game2_core::game::campaign::DecisionRule::Majority, 1);

    println!("Turn | Action | Result | Rapport | Guild\n");
    // Work through one villager at a time: keep talking (each a distinct
    // day, so every one deepens the bond) until rapport clears the
    // recruiting gate, then try the ask - repeating the ask as talks
    // keep building trust, until they join or we give up on them.
    let mut turn = 0u64;
    for &target in &villagers {
        loop {
            let (action_desc, result) = if rapport(&world, player, target) >= RECRUIT_RAPPORT_NEEDED {
                ("Recruit", act_recruit(&mut world, player, org, target))
            } else {
                ("Talk", act_talk(&mut world, player, target, turn))
            };
            let members = world.get_component::<Organization>(org).unwrap().members.len();
            println!("{} | {} | {} | rapport={:.2} | {} members", turn, action_desc,
                if result.success { "OK" } else { "FAIL" }, rapport(&world, player, target), members);
            let recruited = action_desc == "Recruit" && result.success;
            turn += 1;
            if recruited || turn > 40 { break; }
        }
    }

    let members = world.get_component::<Organization>(org).unwrap().members.len();
    println!("\nHarvesters Guild: {} members", members);
    assert!(members > 0, "kept company should have won over at least one villager");
    println!("=== PLAYER LAYER COMPLETE ===");
}
