//! THE GAME: playable medieval village sim.
//! You are The Wanderer - an outsider building influence through organizing.
//! Actions each turn: talk, recruit, invest, organize, build, wait.

use std::io::{self, Write};
use game2_core::engine::ecs::{Scheduler, World};
use game2_core::engine::render::Buffer;
use game2_core::game::terrain::Terrain;
use game2_core::game::villager::{self, Hunger, Villager, Activity, Pos};
use game2_core::game::pets::{self, Species, PetRole};
use game2_core::game::player::PlayerCharacter;
use game2_core::game::guilds::{GuildFormationSystem, Merchant, Guild};
use game2_core::game::action_menu::{MenuActionType, show_menu, execute_choice, parse_command};
use game2_core::game::tech_tree::extended_printmaking_tree;
use game2_core::ui::isometric::{render_terrain, render_entity};
use game2_core::sim::climate::{self, SeasonSystem};
use game2_core::sim::illness::Health;
use game2_core::sim::psychology::AttachmentHistory;
use game2_core::sim::community::{Citizen, Organization};
use game2_core::sim::organized_power::{OrganizedPower, PowerAtrophySystem};

fn main() {
    println!("=== THE WANDERER ===\n");
    println!("You arrive in a village with nothing. Build influence,");
    println!("organize the villagers, gain real power.\n");

    let mut world = World::new();
    let mut sched = Scheduler::new();
    sched.add_system(villager::HungerSystem);
    sched.add_system(villager::SeekFoodSystem);
    sched.add_system(pets::PetBondSystem);
    sched.add_system(SeasonSystem);
    sched.add_system(GuildFormationSystem);
    sched.add_system(PowerAtrophySystem); // people power fades if you stop organizing

    let terrain = Terrain::generate(33, 0.7, 42);
    let climate = climate::spawn_climate(&mut world);
    villager::spawn_food_source(&mut world, 8.0, 8.0, 500);

    // Villagers
    let mut names: Vec<String> = ["Aldric","Eliza","Bram","Corvin","Dagny"].iter().map(|n| n.to_string()).collect();
    let mut villagers = vec![];
    for i in 0..names.len() {
        villagers.push(settle(&mut world, i));
    }
    // Pilgrims who may settle here on a pilgrimage, in order of arrival
    let mut pilgrims = vec!["Wystan", "Hild", "Osric", "Maud"].into_iter();

    // Merchants (guild may form)
    for i in 0..3 {
        let m = world.create_entity();
        world.add_component(m, Merchant { name: format!("Merchant{}",i), wealth: 60.0, trade_routes: vec!["north_road".into()], vulnerable: i < 2 });
    }

    // Player + org
    let player = world.create_entity();
    world.add_component(player, PlayerCharacter { name: "The Wanderer".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
    let org = world.create_entity();
    world.add_component(org, Organization { name: "The Harvesters".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 50.0, charter: "Mutual aid".into() });
    world.add_component(org, OrganizedPower::new()); // grows with house meetings
    let dog = pets::spawn_pet(&mut world, "Rex", Species::Dog, PetRole::Companion);
    pets::adopt(&mut world, dog, villagers[0]);

    let _techs = extended_printmaking_tree();
    std::fs::create_dir_all("evidence").ok();
    let mut tick = 0u64;
    let mut target_idx = 0usize; // villager the player is dealing with

    // Game loop
    loop {
        tick += 1;
        sched.tick(&mut world);

        // Show state
        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        let members = world.get_component::<Organization>(org).unwrap().members.len();
        let guilds = world.query::<Guild>().len();
        let power = world.get_component::<OrganizedPower>(org).map(|p| p.capacity).unwrap_or(0.0);
        let cl = world.get_component::<climate::Climate>(climate).unwrap();
        println!("--- Tick {} | {:?} | {} org members | people power {:.2} | {} guilds ---",
            tick, cl.season, members, power, guilds);

        // Villager status (numbered so the player can pick a target; * = current)
        for (i, v) in villagers.iter().enumerate() {
            if let (Some(h), Some(vil)) = (world.get_component::<Hunger>(*v), world.get_component::<Villager>(*v)) {
                let status = match vil.activity { Activity::SeekingFood => "seeking food", _ => "idle" };
                let mark = if i == target_idx { "*" } else { "" };
                print!("{}{}.{}:{}({:.0}) ", mark, i + 1, names[i], status, h.value);
            }
        }
        println!();

        // Render the map each turn (visible to player)
        let mut board = render_terrain(&terrain, 400, 300);
        let colors = [(255,80,80),(80,255,80),(80,80,255),(255,255,80),(255,80,255)];
        for (i, v) in villagers.iter().enumerate() {
            if let Some(p) = world.get_component::<Pos>(*v) {
                let c = colors[i % colors.len()];
                render_entity(&mut board, p.x, p.y, c.0, c.1, c.2);
            }
        }
        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        // Player on map (white)
        render_entity(&mut board, 10.0, 10.0, 255, 255, 255);
        let frame_path = format!("evidence/game-frame-{:03}.bmp", tick);
        board.save_bmp(&frame_path).ok();
        println!("  map: {}", frame_path);

        // Show menu
        let p_ref = world.get_component::<PlayerCharacter>(player).unwrap().clone();
        print!("{}", show_menu(&p_ref, members, tick as u32));
        print!("choose: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap_or(0) == 0 {
            println!("(end of input)");
            break;
        }
        let (choice, new_target) = parse_command(&input, target_idx, villagers.len());
        if choice == 'q' {
            println!("You leave the village.");
            break;
        }
        target_idx = new_target;

        // Execute choice
        let target = villagers[target_idx];
        // Name the villager for actions aimed at one person (Organize invites everyone)
        if "trc".contains(choice) { print!("[{}] ", names[target_idx]); }
        let result = match choice {
            't' => execute_choice(&mut world, player, org, MenuActionType::Talk, Some(target)),
            'r' => execute_choice(&mut world, player, org, MenuActionType::Recruit, Some(target)),
            'i' => execute_choice(&mut world, player, org, MenuActionType::Invest, None),
            'o' => execute_choice(&mut world, player, org, MenuActionType::Organize, Some(target)),
            'c' => execute_choice(&mut world, player, org, MenuActionType::Contract, Some(target)),
            'b' => execute_choice(&mut world, player, org, MenuActionType::Build, None),
            'w' | _ => "You wait and observe.".into(),
        };
        println!("{}\n", result);

        // Random world events
        if tick % 50 == 0 {
            match ["harvest","market day","pilgrimage","storm warning"][tick as usize / 50 % 4] {
                "market day" => {
                    // The Harvesters sell their members' work; the player gets a share
                    let earned = game2_core::game::player::market_day(&mut world, player, org);
                    if earned > 0.0 {
                        println!("EVENT: market day! The Harvesters sell their work and you earn {:.0} gold.", earned);
                    } else {
                        println!("EVENT: market day! With no members, the Harvesters have nothing to sell.");
                    }
                }
                "pilgrimage" => match pilgrims.next() {
                    // A pilgrim stays on: one more villager to hear out and win over
                    Some(name) => {
                        villagers.push(settle(&mut world, villagers.len()));
                        names.push(name.to_string());
                        println!("EVENT: pilgrimage! {} decides to settle in the village (villager {}).", name, villagers.len());
                    }
                    None => println!("EVENT: pilgrims pass through, but none stay."),
                },
                other => println!("EVENT: {} occurs!", other),
            }
        }

        // Win/lose check
        let p = world.get_component::<PlayerCharacter>(player).unwrap();
        if p.influence > game2_core::game::player::LEAD_INFLUENCE {
            println!("\n*** You are now the recognized leader of the village! ***");
            println!("The Wanderer became someone who matters.");
            break;
        }
        if tick > 500 { println!("\n500 turns passed. The game continues..."); break; }
    }
    println!("\n=== GAME OVER ===");
}

/// A villager settles at the i-th homestead along the village row
fn settle(world: &mut World, i: usize) -> game2_core::engine::ecs::Entity {
    let v = villager::spawn_villager(world, ((i * 4) % 30) as f32, (i % 3) as f32);
    world.add_component(v, Health { condition: 0.9, immunity: 0.5 });
    world.add_component(v, AttachmentHistory { anxiety_episodes: (i % 5) as u32, warmth_episodes: (4 - i % 5) as u32, episodes: vec![] });
    world.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
    v
}
