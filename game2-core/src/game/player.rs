//! Player character: an outsider building influence, not authority.
//! Actions: talk, recruit, invest, organize, contract, build.

use crate::engine::ecs::{Entity, World};

/// The player's character
#[derive(Debug)]
pub struct PlayerCharacter {
    pub name: String,
    pub influence: f32,   // soft power - relationships, reputation
    pub authority: f32,   // formal power - comes later via recognition
    pub resources: f32,   // gold/materials
    pub standing: f32,    // how the village views them (outsider->trusted->leader)
}

/// Actions the player can take
#[derive(Debug)]
pub enum PlayerAction {
    TalkTo(Entity),           // 1-1 meeting, build relationship
    Recruit(Entity, f32),     // try to recruit to org (trust)
    Invest(Entity, f32),      // spend resources on something/someone
    OrganizeMeeting(Vec<Entity>), // gather identified people
    SignContract(Entity),     // formal agreement
    BuildEntity(String),      // construct a building/structure
}

/// Result of a player action
#[derive(Debug)]
pub struct ActionResult {
    pub success: bool,
    pub influence_change: f32,
    pub description: String,
}

/// Player takes a talk action (1-1 meeting)
pub fn act_talk(world: &mut World, player: Entity, _target: Entity) -> ActionResult {
    // Talking builds standing with that person
    let gained = 0.05;
    if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
        p.influence += gained;
    }
    ActionResult {
        success: true,
        influence_change: gained,
        description: "Had a conversation, learned their concerns".into(),
    }
}

/// Player attempts to recruit someone
pub fn act_recruit(world: &mut World, player: Entity, org: Entity, target: Entity) -> ActionResult {
    let already = world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.members.contains(&target)).unwrap_or(false);
    if already {
        return ActionResult { success: false, influence_change: 0.0, description: "Already a member".into() };
    }
    let player_infl = world.get_component::<PlayerCharacter>(player).map(|p| p.influence).unwrap_or(0.0);
    let trust = player_infl * 0.5 + 0.3;
    let recruited = crate::sim::community::recruit(world, org, target, trust);
    ActionResult {
        success: recruited,
        influence_change: if recruited { 0.1 } else { 0.0 },
        description: if recruited { "They joined your cause".into() } else { "They declined".into() },
    }
}

/// Player invests resources
pub fn act_invest(world: &mut World, player: Entity, amount: f32) -> ActionResult {
    if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
        if p.resources >= amount {
            p.resources -= amount;
            p.influence += amount * 0.05;
            return ActionResult { success: true, influence_change: amount * 0.05, description: format!("Invested {:.0} gold", amount) };
        }
    }
    ActionResult { success: false, influence_change: 0.0, description: "Not enough resources".into() }
}
