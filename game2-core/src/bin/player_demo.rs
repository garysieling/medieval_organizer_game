//! D1-D4: Player layer - outsider takes actions that change world state.
//! Talk, recruit, invest -> influence grows -> standing changes.

use game2_core::engine::ecs::World;
use game2_core::game::villager::{self};
use game2_core::game::player::{PlayerCharacter, act_talk, act_recruit, act_invest};
use game2_core::sim::community::{Citizen, Organization};

fn main() {
    println!("=== PLAYER LAYER: outsider builds influence ===\n");
    let mut world = World::new();

    // The player: an outsider with no authority
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter {
        name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0,
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

    println!("Turn | Action | Result | Influence | Standing\n");
    for turn in 0..10 {
        // Cycle through actions
        let target = villagers[turn % 4];
        let (action_desc, result) = match turn % 4 {
            0 => {
                let r = act_talk(&mut world, player, target, 0);
                ("Talk", r)
            }
            1 => {
                let r = act_recruit(&mut world, player, org, target);
                ("Recruit", r)
            }
            2 => {
                let r = act_invest(&mut world, player, 15.0);
                ("Invest", r)
            }
            _ => {
                let r = act_talk(&mut world, player, target, 0);
                ("Talk", r)
            }
        };
        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        println!("{} | {} | {} | infl={:.2} stand={:.2}", turn, action_desc,
            if result.success { "OK" } else { "FAIL" }, p.influence, p.standing);
    }

    // Standing progression: influence -> standing milestones
    let p = world.get_component::<PlayerCharacter>(player).unwrap();
    let standing = if p.influence > 0.8 { "Recognized Leader" }
        else if p.influence > 0.5 { "Trusted Ally" }
        else if p.influence > 0.2 { "Known Face" }
        else { "Outsider" };
    println!("\nPlayer achieved: {} (influence {:.2})", standing, p.influence);
    let org_size = world.get_component::<Organization>(org).unwrap().members.len();
    println!("Harvesters Guild: {} members", org_size);
    println!("=== PLAYER LAYER COMPLETE ===");
}
