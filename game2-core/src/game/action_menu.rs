//! Action menu: text-based player interface.
//! Player chooses actions each turn: talk, recruit, invest, organize, contract, build.

use crate::engine::ecs::{Entity, World};
use super::player::{PlayerCharacter, act_talk, act_recruit, act_invest};

/// Available player actions for the menu
#[derive(Debug)]
pub struct MenuAction {
    pub key: char,
    pub name: String,
    pub description: String,
    pub action: MenuActionType,
}

#[derive(Debug)]
pub enum MenuActionType {
    Talk,
    Recruit,
    Invest,
    Organize,
    Contract,
    Build,
    Wait,
}

/// Display the action menu for a turn
pub fn show_menu(player: &PlayerCharacter, org_members: usize, turn: u32) -> String {
    let mut s = format!("\n=== Turn {} ===\n", turn);
    s += &format!("You: {} | influence: {:.2} / {:.1} to lead | standing: {} | gold: {:.0}\n",
        player.name, player.influence, super::player::LEAD_INFLUENCE,
        super::player::standing(player.influence), player.resources);
    s += &format!("Organization: {} members\n\n", org_members);
    s += "Actions:\n";
    s += "  [t] Talk to someone (1-1, learn concerns)\n";
    s += "  [r] Recruit to your cause (needs trust)\n";
    s += "  [i] Invest resources (gold -> influence)\n";
    s += "  [o] Organize a meeting (gather people)\n";
    s += "  [c] Seal a mutual-aid pact (10 gold of aid; builds their trust)\n";
    s += "  [b] Build a meeting hall (40 gold; meetings become shared meals)\n";
    s += "  [w] Wait (let the world turn)\n";
    s += "  [q] Quit\n";
    s += "Add a number to choose who: t2 = talk to villager 2\n";
    s
}

/// Parse a typed command like "t" or "r3" into (action key, target index).
/// The number is 1-based; without a valid one the current target is kept,
/// so talking to someone and then recruiting them hits the same person.
pub fn parse_command(input: &str, current_target: usize, num_targets: usize) -> (char, usize) {
    let s = input.trim();
    let choice = s.chars().next().map(|c| c.to_ascii_lowercase()).unwrap_or('w');
    let rest: String = s.chars().skip(1).collect();
    let target = rest.trim().parse::<usize>().ok()
        .filter(|n| (1..=num_targets).contains(n))
        .map(|n| n - 1)
        .unwrap_or(current_target);
    (choice, target)
}

/// Execute a chosen action
pub fn execute_choice(world: &mut World, player: Entity, org: Entity, choice: MenuActionType, target: Option<Entity>) -> String {
    match choice {
        MenuActionType::Talk => {
            if let Some(t) = target {
                let r = act_talk(world, player, t);
                format!("You talk. {}", r.description)
            } else { "No one to talk to.".into() }
        }
        MenuActionType::Recruit => {
            if let Some(t) = target {
                let r = act_recruit(world, player, org, t);
                let infl = world.get_component::<PlayerCharacter>(player).map(|p| p.influence).unwrap_or(0.0);
                if r.success { "They joined!".into() }
                else if r.description == "They declined" && !super::player::can_recruit(infl) {
                    // Say what's missing, as game_ui's disabled button does
                    format!("They declined - they don't trust you enough yet (you need influence {:.2}, you have {:.2}).",
                        super::player::recruit_influence_needed(), infl)
                }
                else { format!("{}.", r.description) }
            } else { "No one to recruit.".into() }
        }
        MenuActionType::Invest => {
            let r = act_invest(world, player, 10.0);
            format!("{}", r.description)
        }
        MenuActionType::Organize => {
            // Every villager is invited; those who already trust you join
            let invited = world.query::<super::villager::Villager>();
            if invited.is_empty() { return "No one comes to your house meeting.".into(); }
            let members = world.get_component::<crate::sim::community::Organization>(org)
                .map(|o| o.members.clone()).unwrap_or_default();
            let outsiders = invited.iter().filter(|v| !members.contains(v)).count();
            let r = super::organizing::house_meeting(world, org, &invited);
            // Each member won at a meeting counts toward leading the village
            // just like one won with Recruit (as in game_ui, #74) - otherwise
            // organizing the whole village could never win the game
            if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
                p.influence += super::player::RECRUIT_INFLUENCE * r.attendees_recruited as f32;
            }
            let joined = match r.attendees_recruited {
                0 if outsiders == 0 => "Everyone here is already with you; the meeting strengthens your ties.".to_string(),
                0 => "No one joins yet, but they'll remember the conversation.".to_string(),
                1 => "1 joins your cause.".to_string(),
                n => format!("{} join your cause{}.", n, if r.formed_committee { " and form a committee" } else { "" }),
            };
            format!("You call a house meeting. {} come, and talk about {}. {}",
                invited.len(), r.shared_concerns.join(" and "), joined)
        }
        MenuActionType::Contract => {
            use super::contracts::{has_pact, seal_aid_pact, PACT_AID};
            let Some(t) = target else { return "No one to make a pact with.".into() };
            if has_pact(world, player, t) { return "You already have a pact with them.".into(); }
            let mut gold = world.get_component::<PlayerCharacter>(player).map(|p| p.resources).unwrap_or(0.0);
            if seal_aid_pact(world, player, t, &mut gold).is_none() {
                return format!("A pact means giving {:.0} gold of aid; you have {:.0}.", PACT_AID, gold);
            }
            if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) { p.resources = gold; }
            format!("You seal a mutual-aid pact and give {:.0} gold of help. They won't forget it.", PACT_AID)
        }
        MenuActionType::Build => {
            use super::organizing::{MeetingHall, HALL_COST};
            if world.get_component::<MeetingHall>(org).is_some() {
                return "Your meeting hall already stands.".into();
            }
            let gold = world.get_component::<PlayerCharacter>(player).map(|p| p.resources).unwrap_or(0.0);
            if gold < HALL_COST {
                return format!("A meeting hall costs {:.0} gold; you have {:.0}.", HALL_COST, gold);
            }
            if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) { p.resources -= HALL_COST; }
            world.add_component(org, MeetingHall);
            format!("You build a meeting hall ({:.0} gold). From now on your house meetings share a meal.", HALL_COST)
        }
        MenuActionType::Wait => "You wait and observe.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declined_recruit_says_what_is_missing() {
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let v = w.create_entity();
        let msg = execute_choice(&mut w, player, org, MenuActionType::Recruit, Some(v));
        assert!(msg.contains("need influence 0.40") && msg.contains("you have 0.10"), "{}", msg);
    }

    #[test]
    fn test_parse_command_keeps_target_without_number() {
        assert_eq!(parse_command("t\n", 2, 5), ('t', 2));
        assert_eq!(parse_command("R", 2, 5), ('r', 2));
    }

    #[test]
    fn test_parse_command_picks_numbered_target() {
        assert_eq!(parse_command("t4", 0, 5), ('t', 3));
        assert_eq!(parse_command("r 1", 3, 5), ('r', 0));
        // out-of-range or junk numbers keep the current target
        assert_eq!(parse_command("t9", 1, 5), ('t', 1));
        assert_eq!(parse_command("t0", 1, 5), ('t', 1));
        assert_eq!(parse_command("tx", 1, 5), ('t', 1));
        assert_eq!(parse_command("", 1, 5), ('w', 1));
    }

    #[test]
    fn test_organize_holds_a_real_house_meeting() {
        use crate::game::relationships::{find_relationship, Relationship, RelKind};
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let friend = crate::game::villager::spawn_villager(&mut w, 0.0, 0.0);
        let stranger = crate::game::villager::spawn_villager(&mut w, 1.0, 0.0);
        let rel = w.create_entity();
        w.add_component(rel, Relationship { from: player, other: friend, kind: RelKind::Friend, strength: 0.7, interactions: 6 });

        let msg = execute_choice(&mut w, player, org, MenuActionType::Organize, None);

        let members = &w.get_component::<Organization>(org).unwrap().members;
        assert_eq!(members, &vec![friend], "the trusted friend joins, the stranger doesn't");
        assert!(find_relationship(&w, player, stranger).is_some(), "the meeting still starts a relationship with the stranger");
        assert!(msg.contains("1 joins"), "{}", msg);
    }

    #[test]
    fn test_the_text_game_can_be_won_by_organizing() {
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let villagers: Vec<Entity> = (0..5).map(|i| crate::game::villager::spawn_villager(&mut w, i as f32, 0.0)).collect();
        // Hear everyone out, organize until the village has joined, invest the gold
        for &v in &villagers {
            for _ in 0..8 { execute_choice(&mut w, player, org, MenuActionType::Talk, Some(v)); }
        }
        for _ in 0..20 { execute_choice(&mut w, player, org, MenuActionType::Organize, None); }
        assert_eq!(w.get_component::<Organization>(org).unwrap().members.len(), 5);
        for _ in 0..10 { execute_choice(&mut w, player, org, MenuActionType::Invest, None); }
        let infl = w.get_component::<PlayerCharacter>(player).unwrap().influence;
        assert!(infl > 2.0, "organized the whole village and invested everything, yet influence is {:.2} (win at 2.0)", infl);
    }

    #[test]
    fn test_menu_shows_the_goal_and_a_standing_that_matches_the_rules() {
        let p = |influence: f32| PlayerCharacter { name: "P".into(), influence, authority: 0.0, resources: 100.0, standing: 0.0 };
        assert!(show_menu(&p(0.1), 0, 1).contains("influence: 0.10 / 2.0 to lead | standing: Outsider"));
        // Trusted exactly when recruiting starts to work
        let needed = crate::game::player::recruit_influence_needed();
        assert!(show_menu(&p(needed + 1e-3), 0, 1).contains("standing: Trusted"));
        assert!(show_menu(&p(needed - 0.01), 0, 1).contains("standing: Outsider"));
        assert!(show_menu(&p(1.2), 0, 1).contains("standing: Respected"));
    }

    #[test]
    fn test_build_buys_one_meeting_hall() {
        use crate::game::organizing::MeetingHall;
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let msg = execute_choice(&mut w, player, org, MenuActionType::Build, None);
        assert!(msg.contains("build a meeting hall"), "{}", msg);
        assert!(w.get_component::<MeetingHall>(org).is_some());
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 60.0);
        // only one hall, and no charge for trying again
        assert!(execute_choice(&mut w, player, org, MenuActionType::Build, None).contains("already stands"));
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 60.0);
    }

    #[test]
    fn test_build_needs_the_gold() {
        use crate::game::organizing::MeetingHall;
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 30.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        assert!(execute_choice(&mut w, player, org, MenuActionType::Build, None).contains("costs 40 gold; you have 30"));
        assert!(w.get_component::<MeetingHall>(org).is_none());
    }

    #[test]
    fn test_contract_seals_an_aid_pact_with_the_target() {
        use crate::game::relationships::{find_relationship, Relationship};
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let v = w.create_entity();
        let msg = execute_choice(&mut w, player, org, MenuActionType::Contract, Some(v));
        assert!(msg.contains("seal a mutual-aid pact"), "{}", msg);
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 90.0);
        assert!(find_relationship(&w, player, v).and_then(|r| w.get_component::<Relationship>(r)).unwrap().strength > 0.1);
        assert!(execute_choice(&mut w, player, org, MenuActionType::Contract, Some(v)).contains("already have a pact"));
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 90.0);
    }
}
