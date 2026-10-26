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
    s
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
                if r.success { "They joined!".into() } else if r.description.contains("already") { "They're already a member.".into() } else { "They declined.".into() }
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
