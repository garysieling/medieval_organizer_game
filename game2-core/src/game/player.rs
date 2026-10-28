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

/// How well the player knows a villager (0 = stranger, 1 = heard all their concerns)
#[derive(Debug, Clone, Copy, Default)]
pub struct Rapport { pub value: f32 }

pub const TALK_INFLUENCE: f32 = 0.05;
pub const TALK_RAPPORT_STEP: f32 = 0.15;
pub const RECRUIT_INFLUENCE: f32 = 0.1;

/// Player takes a talk action (1-1 meeting).
/// Repeat talks with the same person yield less; talking alone can't win.
pub fn act_talk(world: &mut World, player: Entity, target: Entity) -> ActionResult {
    let rapport = world.get_component::<Rapport>(target).map(|r| r.value).unwrap_or(0.0);
    let gained = TALK_INFLUENCE * (1.0 - rapport);
    world.add_component(target, Rapport { value: (rapport + TALK_RAPPORT_STEP).min(1.0) });
    if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
        p.influence += gained;
    }
    ActionResult {
        success: true,
        influence_change: gained,
        description: if rapport >= 1.0 { "You've already heard all their concerns".into() }
            else { "Had a conversation, learned their concerns".into() },
    }
}

/// Influence at which the village recognizes the player as its leader
pub const LEAD_INFLUENCE: f32 = 2.0;

/// How the village sees the player, by influence: an Outsider until people
/// trust them enough to join (recruit_influence_needed), Respected halfway
/// to leading, and Leader at LEAD_INFLUENCE
pub fn standing(influence: f32) -> &'static str {
    if influence > LEAD_INFLUENCE { "Leader" }
    else if influence >= LEAD_INFLUENCE / 2.0 { "Respected" }
    else if can_recruit(influence) { "Trusted" }
    else { "Outsider" }
}

/// Trust a stranger places in the player before any influence counts
const RECRUIT_BASE_TRUST: f32 = 0.3;
/// Extra trust per point of the player's influence
const RECRUIT_TRUST_PER_INFLUENCE: f32 = 0.5;

/// Trust a villager places in the player when asked to join
pub fn recruit_trust(player_influence: f32) -> f32 {
    player_influence * RECRUIT_TRUST_PER_INFLUENCE + RECRUIT_BASE_TRUST
}

/// The least influence at which a recruit attempt succeeds
pub fn recruit_influence_needed() -> f32 {
    (crate::sim::community::RECRUIT_MIN_TRUST - RECRUIT_BASE_TRUST) / RECRUIT_TRUST_PER_INFLUENCE
}

/// Whether the player has enough influence for a recruit attempt to succeed
pub fn can_recruit(player_influence: f32) -> bool {
    recruit_trust(player_influence) >= crate::sim::community::RECRUIT_MIN_TRUST
}

/// Player attempts to recruit someone
pub fn act_recruit(world: &mut World, player: Entity, org: Entity, target: Entity) -> ActionResult {
    let already = world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.members.contains(&target)).unwrap_or(false);
    if already {
        return ActionResult { success: false, influence_change: 0.0, description: "Already a member".into() };
    }
    let player_infl = world.get_component::<PlayerCharacter>(player).map(|p| p.influence).unwrap_or(0.0);
    let recruited = crate::sim::community::recruit(world, org, target, recruit_trust(player_infl));
    let gained = if recruited { RECRUIT_INFLUENCE } else { 0.0 };
    if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
        p.influence += gained;
    }
    ActionResult {
        success: recruited,
        influence_change: gained,
        description: if recruited { "They joined your cause".into() } else { "They declined".into() },
    }
}

/// Player invests resources
/// All 100 starting gold buys +0.5 influence; investing alone can't win.
pub const INVEST_INFLUENCE_PER_GOLD: f32 = 0.005;

pub fn act_invest(world: &mut World, player: Entity, amount: f32) -> ActionResult {
    // Only a real, positive sum can be invested: a negative amount would pass
    // the affordability check and mint gold, and NaN would poison both stats
    if !(amount.is_finite() && amount > 0.0) {
        return ActionResult { success: false, influence_change: 0.0, description: "Nothing to invest".into() };
    }
    if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
        if p.resources >= amount {
            p.resources -= amount;
            p.influence += amount * INVEST_INFLUENCE_PER_GOLD;
            return ActionResult { success: true, influence_change: amount * INVEST_INFLUENCE_PER_GOLD, description: format!("Invested {:.0} gold", amount) };
        }
    }
    ActionResult { success: false, influence_change: 0.0, description: "Not enough resources".into() }
}

/// Gold each member of the player's organization earns them on market day
pub const MARKET_PAY_PER_MEMBER: f32 = 5.0;

/// Market day: the organization sells its living members' work, and the
/// player's share comes back as gold - so organizing people also funds the
/// halls, pacts and investments that cost gold. Returns the gold earned.
pub fn market_day(world: &mut World, player: Entity, org: Entity) -> f32 {
    let workers = world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.members.clone()).unwrap_or_default()
        .into_iter().filter(|m| !crate::game::villager::is_dead(world, *m)).count();
    let earned = workers as f32 * MARKET_PAY_PER_MEMBER;
    if let Some(p) = world.get_component_mut::<PlayerCharacter>(player) {
        p.resources += earned;
    }
    earned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recruit_influence_needed_is_the_threshold() {
        let needed = recruit_influence_needed();
        assert!(can_recruit(needed + 1e-4));
        assert!(!can_recruit(needed - 0.01));
    }
    use crate::sim::community::Organization;

    fn setup() -> (World, Entity, Entity, Vec<Entity>) {
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, PlayerCharacter { name: "P".into(), influence: 0.1, authority: 0.0, resources: 100.0, standing: 0.0 });
        let o = w.create_entity();
        w.add_component(o, Organization { name: "O".into(), members: vec![], leader: p, influence_pool: 0.0, resources: 0.0, charter: "".into() });
        let vs = (0..5).map(|_| w.create_entity()).collect();
        (w, p, o, vs)
    }

    #[test]
    fn test_talk_spam_cannot_win() {
        let (mut w, p, _, vs) = setup();
        for i in 0..500 { act_talk(&mut w, p, vs[i % 5]); }
        let infl = w.get_component::<PlayerCharacter>(p).unwrap().influence;
        assert!(infl < 1.5, "talking alone reached {}", infl);
    }

    #[test]
    fn test_recruit_applies_influence_and_matches_gate() {
        let (mut w, p, o, vs) = setup();
        assert!(!can_recruit(0.1));
        assert!(!act_recruit(&mut w, p, o, vs[0]).success);
        w.get_component_mut::<PlayerCharacter>(p).unwrap().influence = 0.4;
        assert!(can_recruit(0.4));
        let r = act_recruit(&mut w, p, o, vs[0]);
        assert!(r.success);
        assert!((w.get_component::<PlayerCharacter>(p).unwrap().influence - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_full_campaign_can_win() {
        let (mut w, p, o, vs) = setup();
        for i in 0..50 { act_talk(&mut w, p, vs[i % 5]); }
        for _ in 0..10 { act_invest(&mut w, p, 10.0); }
        for v in &vs { act_recruit(&mut w, p, o, *v); }
        assert!(w.get_component::<PlayerCharacter>(p).unwrap().influence > 2.0);
    }

    #[test]
    fn test_invest_rejects_negative_and_nan_amounts() {
        let (mut w, p, _, _) = setup();
        for bad in [-50.0, 0.0, f32::NAN, f32::INFINITY] {
            let r = act_invest(&mut w, p, bad);
            assert!(!r.success, "investing {} should fail", bad);
        }
        let pc = w.get_component::<PlayerCharacter>(p).unwrap();
        assert_eq!(pc.resources, 100.0, "no gold minted or lost");
        assert_eq!(pc.influence, 0.1);
    }

    #[test]
    fn test_market_day_pays_for_living_members() {
        use crate::sim::illness::Health;
        let (mut w, p, o, vs) = setup();
        assert_eq!(market_day(&mut w, p, o), 0.0, "no members, nothing to sell");
        w.get_component_mut::<Organization>(o).unwrap().members = vs[..3].to_vec();
        w.add_component(vs[2], Health { condition: 0.0, immunity: 0.5 });
        assert_eq!(market_day(&mut w, p, o), 10.0, "two living members");
        assert_eq!(w.get_component::<PlayerCharacter>(p).unwrap().resources, 110.0);
    }
}
