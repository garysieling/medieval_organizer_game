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
    Meet,
    March,
    Delegate,
    Petition,
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
    s += "  [m] Meet on the campaign (founds it on the first go)\n";
    s += "  [s] Take the campaign to the street\n";
    s += "  [d] Name a member a campaign seat\n";
    s += "  [p] Petition for a prisoner's release\n";
    s += "  [w] Wait (let the world turn)\n";
    s += "  [q] Quit\n";
    s += "Add a number to choose who: t2 = talk to villager 2\n";
    s
}

/// How the game went, shown however it ends (won, quit, or out of turns)
pub fn end_summary(player: &PlayerCharacter, members: usize, villagers: usize, turns: u64) -> String {
    format!("\nAfter {} turns: influence {:.2} / {:.1} ({}), {} of {} villagers with the Harvesters, {:.0} gold left.",
        turns, player.influence, super::player::LEAD_INFLUENCE, super::player::standing(player.influence),
        members, villagers, player.resources)
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
pub fn execute_choice(world: &mut World, player: Entity, org: Entity, choice: MenuActionType, target: Option<Entity>, tick: u64) -> String {
    match choice {
        MenuActionType::Talk => {
            if let Some(t) = target {
                // What they actually say: the next thing on their mind.
                // Uses the uncapped talk count, not Rapport (which caps at
                // 1.0 and would freeze the concern/small-talk index).
                let talks_before = world.get_component::<super::player::TalkCount>(t)
                    .map(|c| c.value).unwrap_or(0);
                let (did, said) = super::organizing::talk_response(world, t, "They", talks_before);
                act_talk(world, player, t);
                format!("{}: \"{}\"", did, said)
            } else { "No one to talk to.".into() }
        }
        MenuActionType::Recruit => {
            if let Some(t) = target {
                let r = act_recruit(world, player, org, t);
                let infl = world.get_component::<PlayerCharacter>(player).map(|p| p.influence).unwrap_or(0.0);
                if r.success { "They joined!".into() }
                else if r.description == "They declined" && !super::player::can_recruit(infl) {
                    // Say what's missing, as game_ui's disabled button does
                    format!("They declined - they don't trust you enough yet (you need influence {:.2}, you have {:.2}; or win them over by talking with them and at meetings).",
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
            // Aid to a villager is an investment in the village, like the hall:
            // it earns the influence the same gold would have through Invest
            if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
                p.resources = gold;
                p.influence += PACT_AID * super::player::INVEST_INFLUENCE_PER_GOLD;
            }
            format!("You seal a mutual-aid pact and give {:.0} gold of help. They won't forget it.", PACT_AID)
        }
        MenuActionType::Build => {
            use super::organizing::{HallDamaged, MeetingHall, HALL_COST, HALL_REPAIR_COST};
            if world.get_component::<MeetingHall>(org).is_some() {
                if world.get_component::<HallDamaged>(org).is_none() {
                    return "Your meeting hall already stands.".into();
                }
                // Repairing storm damage is also gold put into the village
                let gold = world.get_component::<PlayerCharacter>(player).map(|p| p.resources).unwrap_or(0.0);
                if gold < HALL_REPAIR_COST {
                    return format!("Repairing the storm-damaged hall costs {:.0} gold; you have {:.0}.", HALL_REPAIR_COST, gold);
                }
                if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
                    p.resources -= HALL_REPAIR_COST;
                    p.influence += HALL_REPAIR_COST * super::player::INVEST_INFLUENCE_PER_GOLD;
                }
                world.remove_component::<HallDamaged>(org);
                return format!("You repair the meeting hall ({:.0} gold). Meetings there share a meal again.", HALL_REPAIR_COST);
            }
            let gold = world.get_component::<PlayerCharacter>(player).map(|p| p.resources).unwrap_or(0.0);
            if gold < HALL_COST {
                return format!("A meeting hall costs {:.0} gold; you have {:.0}.", HALL_COST, gold);
            }
            // The hall is an investment in the village: it earns the influence
            // the same gold would have through Invest, so building it never
            // puts the win (which needs every coin's worth) out of reach
            if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
                p.resources -= HALL_COST;
                p.influence += HALL_COST * super::player::INVEST_INFLUENCE_PER_GOLD;
            }
            world.add_component(org, MeetingHall);
            format!("You build a meeting hall ({:.0} gold). From now on your house meetings share a meal.", HALL_COST)
        }
        MenuActionType::Wait => "You wait and observe.".into(),
        MenuActionType::Meet => {
            use super::campaign;
            let c = match campaign::campaign_of(world, org) {
                Some(c) => c,
                // the first meeting is the founding: they take a vote on
                // the spot, once you know what they'd fight for
                None => {
                    let members = world.get_component::<crate::sim::community::Organization>(org)
                        .map(|o| o.members.clone()).unwrap_or_default();
                    let mut issues = world.get_component::<super::desires::Organizer>(player)
                        .map(|o| o.issues.clone()).unwrap_or_default();
                    if issues.is_empty() {
                        // no organizer's notebook in the text game: the
                        // members' own troubles name the cause - the most
                        // common grievances lead the ballot
                        let mut tally: Vec<(String, usize)> = vec![];
                        let mut roofless = 0usize;
                        for &m in &members {
                            for p in super::desires::problems_of(world, m, tick) {
                                // NoHome is ambient - everyone under a roof
                                // counts it, so it trails the real ballot
                                if p == super::desires::Problem::NoHome { roofless += 1; continue; }
                                let tag = super::desires::issue_tag(world, m,
                                    &super::desires::Reveal::Problem(p));
                                match tally.iter_mut().find(|(t, _)| *t == tag) {
                                    Some(e) => e.1 += 1,
                                    None => tally.push((tag, 1)),
                                }
                            }
                        }
                        tally.sort_by(|a, b| b.1.cmp(&a.1));
                        issues = tally.into_iter().take(3).map(|(t, _)| t).collect();
                        // a roof is still a fight worth having - it just
                        // never leads the ballot when a grievance stands
                        if roofless > 0 && issues.len() < 3 {
                            if let Some(&m) = members.first() {
                                issues.push(super::desires::issue_tag(world, m,
                                    &super::desires::Reveal::Problem(super::desires::Problem::NoHome)));
                            }
                        }
                    }
                    if issues.is_empty() {
                        return "Nobody's told you what they'd fight for - talk to people first.".into();
                    }
                    let living = members.iter().filter(|&&m| !super::villager::is_dead(world, m)).count();
                    if living < 3 { return "Three members make a cause worth a vote.".into(); }
                    match campaign::hold_issue_vote(world, org, &issues, campaign::Framing::SharedGrievance, tick) {
                        Some(vote) => {
                            let c = campaign::found_campaign(world, org, &vote,
                                campaign::Framing::SharedGrievance, campaign::DecisionRule::Majority, tick);
                            campaign::set_cadence(world, c, 3, tick);
                            let oname = world.get_component::<crate::sim::community::Organization>(org)
                                .map(|o| o.name.clone()).unwrap_or_default();
                            return format!("{} take a vote - they'll fight for {}.", oname, vote.winner);
                        }
                        None => return "No common cause emerges.".into(),
                    }
                }
            };
            match campaign::hold_campaign_meeting(world, c, tick) {
                Some(out) => format!("{} meet about the campaign; consensus banks as it does.", out.attendees.len()),
                None => "The campaign isn't due to meet yet.".into(),
            }
        }
        MenuActionType::March => {
            use super::campaign;
            let Some(c) = campaign::campaign_of(world, org) else {
                return "There's no campaign to march for - meet first.".into();
            };
            match super::actions::demonstrate(world, c, player, tick) {
                Some(out) => {
                    let r = super::reactions::react(world, c, &out.turnout, tick);
                    // everyone who saw it and felt the cause wake in them
                    let issue = world.get_component::<campaign::Campaign>(c).map(|c| c.issue.clone()).unwrap_or_default();
                    let mut woke = 0usize;
                    for v in world.query::<super::villager::Villager>() {
                        if out.turnout.contains(&v) { continue; }
                        woke += super::reactions::witness(world, v, &issue, &out.turnout, tick) as usize;
                    }
                    let mut msg = format!("{} march in the street.", out.turnout.len());
                    if woke > 0 { msg += &format!(" {} watching feel the cause wake.", woke); }
                    if r == Some(super::reactions::Reaction::Crackdown) {
                        msg += " The knights come out";
                        if let Some(taken) = super::dungeon::arrest(world, c, tick) {
                            super::dungeon::found_release_campaign(world, org, taken, tick);
                            msg += " - they take the loudest. Letters will have to bring them home."
                        } else { msg += "." }
                    }
                    msg
                }
                None => "You call a march - and nobody comes. The street stays empty.".into(),
            }
        }
        MenuActionType::Delegate => {
            use super::campaign::{self, Role};
            let (Some(c), Some(t)) = (campaign::campaign_of(world, org), target) else {
                return if target.is_none() { "Pick someone first.".into() }
                    else { "There's no campaign with seats to fill.".into() };
            };
            // a seated member doesn't get re-seated silently - their
            // old seat would empty behind the new one
            let role_name = |r: Role| match r {
                Role::TurnoutCoordinator => "turnout coordinator",
                Role::NoteTaker => "note taker", Role::Chair => "chair" };
            let already = world.get_component::<campaign::Campaign>(c)
                .and_then(|c| c.seats.iter().find(|s| s.holder == t)).map(|s| s.role);
            if let Some(r) = already {
                return format!("They already hold the {} seat.", role_name(r));
            }
            let open = [Role::TurnoutCoordinator, Role::NoteTaker, Role::Chair].iter().copied()
                .find(|&r| campaign::seat_holder(world, c, r).is_none());
            match open {
                Some(r) if campaign::delegate_role(world, c, r, t) =>
                    format!("You name them to the {} seat.", match r {
                        Role::TurnoutCoordinator => "turnout coordinator",
                        Role::NoteTaker => "note taker", Role::Chair => "chair" }),
                Some(_) => "They can't hold a seat - living members only.".into(),
                None => "Every seat is spoken for.".into(),
            }
        }
        MenuActionType::Petition => {
            let Some(t) = target else { return "No one inside to petition for.".into() };
            if world.get_component::<super::dungeon::Imprisoned>(t).is_none() {
                return "They aren't inside.".into();
            }
            super::dungeon::press(world, t, 0.3);
            if super::dungeon::release_check(world, t, 1.0) {
                "The letters don't stop - and the cell opens. They walk free.".into()
            } else {
                let p = world.get_component::<super::dungeon::ReleaseCampaign>(t)
                    .map(|r| r.pressure).unwrap_or(0.0);
                format!("Letters, vigils, a line out the door ({:.0}% there).", p * 100.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_talk_names_what_is_on_their_mind() {
        use crate::game::villager::Hunger;
        use crate::sim::community::{Citizen, Organization};
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let v = w.create_entity();
        w.add_component(v, Citizen { influence: 0.2, authority: 0.0, ideology: "laborer".into(), allegiance: "none".into() });
        w.add_component(v, Hunger { value: 10.0 });
        let msg = execute_choice(&mut w, player, org, MenuActionType::Talk, Some(v), 5);
        assert_eq!(msg, "They complained the store is running low: \"There's hardly anything left in the store.\"");
        let msg = execute_choice(&mut w, player, org, MenuActionType::Talk, Some(v), 5);
        assert!(msg.starts_with("They complained about wages"), "{}", msg);
    }

    #[test]
    fn test_declined_recruit_says_what_is_missing() {
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let v = w.create_entity();
        let msg = execute_choice(&mut w, player, org, MenuActionType::Recruit, Some(v), 5);
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

        let msg = execute_choice(&mut w, player, org, MenuActionType::Organize, None, 5);

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
            for _ in 0..8 { execute_choice(&mut w, player, org, MenuActionType::Talk, Some(v), 5); }
        }
        for _ in 0..20 { execute_choice(&mut w, player, org, MenuActionType::Organize, None, 5); }
        assert_eq!(w.get_component::<Organization>(org).unwrap().members.len(), 5);
        for _ in 0..10 { execute_choice(&mut w, player, org, MenuActionType::Invest, None, 5); }
        let infl = w.get_component::<PlayerCharacter>(player).unwrap().influence;
        assert!(infl > 2.0, "organized the whole village and invested everything, yet influence is {:.2} (win at 2.0)", infl);
    }

    #[test]
    fn test_building_the_hall_doesnt_put_the_win_out_of_reach() {
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let villagers: Vec<Entity> = (0..5).map(|i| crate::game::villager::spawn_villager(&mut w, i as f32, 0.0)).collect();
        // Build the hall first, then the same route as above
        execute_choice(&mut w, player, org, MenuActionType::Build, None, 5);
        for &v in &villagers {
            for _ in 0..8 { execute_choice(&mut w, player, org, MenuActionType::Talk, Some(v), 5); }
        }
        for _ in 0..20 { execute_choice(&mut w, player, org, MenuActionType::Organize, None, 5); }
        for _ in 0..10 { execute_choice(&mut w, player, org, MenuActionType::Invest, None, 5); }
        let infl = w.get_component::<PlayerCharacter>(player).unwrap().influence;
        assert!(infl > 2.0, "built the hall, organized everyone and invested the rest, yet influence is {:.2} (win at 2.0)", infl);
    }

    #[test]
    fn test_pacts_dont_put_the_win_out_of_reach() {
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let villagers: Vec<Entity> = (0..5).map(|i| crate::game::villager::spawn_villager(&mut w, i as f32, 0.0)).collect();
        // Spend gold every way there is - the hall and a pact with everyone - then win
        execute_choice(&mut w, player, org, MenuActionType::Build, None, 5);
        for &v in &villagers {
            execute_choice(&mut w, player, org, MenuActionType::Contract, Some(v), 5);
            for _ in 0..8 { execute_choice(&mut w, player, org, MenuActionType::Talk, Some(v), 5); }
        }
        for _ in 0..20 { execute_choice(&mut w, player, org, MenuActionType::Organize, None, 5); }
        for _ in 0..10 { execute_choice(&mut w, player, org, MenuActionType::Invest, None, 5); }
        let infl = w.get_component::<PlayerCharacter>(player).unwrap().influence;
        assert!(infl > 2.0, "spent gold on the hall and pacts, yet influence is {:.2} (win at 2.0)", infl);
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
        let msg = execute_choice(&mut w, player, org, MenuActionType::Build, None, 5);
        assert!(msg.contains("build a meeting hall"), "{}", msg);
        assert!(w.get_component::<MeetingHall>(org).is_some());
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 60.0);
        // only one hall, and no charge for trying again
        assert!(execute_choice(&mut w, player, org, MenuActionType::Build, None, 5).contains("already stands"));
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
        assert!(execute_choice(&mut w, player, org, MenuActionType::Build, None, 5).contains("costs 40 gold; you have 30"));
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
        let msg = execute_choice(&mut w, player, org, MenuActionType::Contract, Some(v), 5);
        assert!(msg.contains("seal a mutual-aid pact"), "{}", msg);
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 90.0);
        assert!(find_relationship(&w, player, v).and_then(|r| w.get_component::<Relationship>(r)).unwrap().strength > 0.1);
        assert!(execute_choice(&mut w, player, org, MenuActionType::Contract, Some(v), 5).contains("already have a pact"));
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 90.0);
    }

    #[test]
    fn test_build_repairs_a_storm_damaged_hall() {
        use crate::game::organizing::{storm_strikes_hall, HallDamaged, MeetingHall};
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 50.0, standing: 0.0 });
        let org = w.create_entity();
        w.add_component(org, Organization { name: "O".into(), members: vec![], leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        w.add_component(org, MeetingHall);
        storm_strikes_hall(&mut w, org);
        let msg = execute_choice(&mut w, player, org, MenuActionType::Build, None, 5);
        assert!(msg.contains("repair the meeting hall"), "{}", msg);
        assert!(w.get_component::<HallDamaged>(org).is_none());
        assert_eq!(w.get_component::<PlayerCharacter>(player).unwrap().resources, 35.0);
        assert!(execute_choice(&mut w, player, org, MenuActionType::Build, None, 5).contains("already stands"));
    }

    #[test]
    fn test_end_summary_reports_how_it_went() {
        let p = PlayerCharacter { name: "P".into(), influence: 1.25, authority: 0.0, resources: 35.0, standing: 0.0 };
        let s = end_summary(&p, 4, 6, 501);
        assert!(s.contains("After 501 turns: influence 1.25 / 2.0 (Respected), 4 of 6 villagers with the Harvesters, 35 gold left."), "{}", s);
    }

    #[test]
    fn test_the_text_game_fights_a_cause_too() {
        use crate::game::desires::{self, InnerLife, Organizer};
        use crate::sim::community::Organization;
        let mut w = World::new();
        let player = w.create_entity();
        w.add_component(player, PlayerCharacter { name: "P".into(), influence: 0.5, authority: 0.0, resources: 100.0, standing: 0.0 });
        w.add_component(player, Organizer { issues: vec!["the taxes".into()], heard: vec![] });
        let org = w.create_entity();
        let members: Vec<Entity> = (0..4).map(|_| {
            let m = w.create_entity();
            w.add_component(m, InnerLife { wants: vec![], grievances: vec![], satisfied: vec![] });
            desires::add_grievance(&mut w, m, desires::Problem::Taxed, 200);
            m
        }).collect();
        w.add_component(org, Organization { name: "the Harvesters".into(), members: members.clone(), leader: player, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        // the first Meet is the founding vote
        let msg = execute_choice(&mut w, player, org, MenuActionType::Meet, None, 5);
        assert!(msg.contains("take a vote"), "{}", msg);
        // a member takes a seat
        let msg = execute_choice(&mut w, player, org, MenuActionType::Delegate, Some(members[0]), 6);
        assert!(msg.contains("seat"), "{}", msg);
        // the street answers (or stays empty - either way the verb lives)
        let msg = execute_choice(&mut w, player, org, MenuActionType::March, None, 8);
        assert!(msg.contains("street"), "{}", msg);
    }
}
