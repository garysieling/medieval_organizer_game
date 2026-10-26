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
    s += &format!("You: {} | influence: {:.2} | standing: {} | gold: {:.0}\n",
        player.name, player.influence,
        if player.influence > 0.8 { "Recognized" } else if player.influence > 0.5 { "Trusted" } else { "Outsider" },
        player.resources);
    s += &format!("Organization: {} members\n\n", org_members);
    s += "Actions:\n";
    s += "  [t] Talk to someone (1-1, learn concerns)\n";
    s += "  [r] Recruit to your cause (needs trust)\n";
    s += "  [i] Invest resources (gold -> influence)\n";
    s += "  [o] Organize a meeting (gather people)\n";
    s += "  [c] Sign a contract (formal agreement)\n";
    s += "  [b] Build something (structure)\n";
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
                if r.success { "They joined!".into() } else { format!("{}.", r.description) }
            } else { "No one to recruit.".into() }
        }
        MenuActionType::Invest => {
            let r = act_invest(world, player, 10.0);
            format!("{}", r.description)
        }
        MenuActionType::Organize => "You call a house meeting. People gather.".into(),
        MenuActionType::Contract => "You draft a contract.".into(),
        MenuActionType::Build => "You commission a structure.".into(),
        MenuActionType::Wait => "You wait and observe.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
