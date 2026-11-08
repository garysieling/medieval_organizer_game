//! Player character: an outsider earning trust, not authority.
//! Actions: talk, recruit, organize, contract, build.

use crate::engine::ecs::{Entity, World};

/// The player's character
#[derive(Debug)]
pub struct PlayerCharacter {
    pub name: String,
    pub resources: f32,   // gold/materials
}

/// Which figure the player wears: a look into the villager pack (the
/// same indices `figure_look` resolves), small if they took a childhood
#[derive(Debug, Clone, Copy)]
pub struct Avatar {
    pub look: usize,
    pub kid: bool,
}

/// Result of a player action
#[derive(Debug)]
pub struct ActionResult {
    pub success: bool,
    pub description: String,
}

/// How well a player knows a villager (0 = stranger, 1 = heard all their
/// concerns) - kept per player, so one organizer's conversations don't hand
/// another organizer the trust needed to recruit someone they've barely met
#[derive(Debug, Clone, Default)]
pub struct Rapport { pub by_player: crate::engine::det::HashMap<Entity, f32> }

impl Rapport {
    /// A Rapport already at `value` for `player` - test/demo shorthand
    pub fn for_player(player: Entity, value: f32) -> Self {
        let mut by_player = crate::engine::det::HashMap::default();
        by_player.insert(player, value);
        Rapport { by_player }
    }
}

/// How many times the player has talked with this villager - uncapped,
/// unlike `Rapport` (which caps at 1.0 for relationship purposes).
/// talk_response uses this so the recurring hunger/sickness concern keeps
/// coming up on schedule even once rapport itself has maxed out. `value`
/// is shared across every player so dialogue variety still advances
/// regardless of who's talking; the same-day cooldown below is per
/// player, since one seat's talk shouldn't block another's bond.
#[derive(Debug, Clone, Default)]
pub struct TalkCount {
    pub value: usize,
    /// the last day each player talked here - that player's same-day
    /// repeat doesn't deepen the bond, but a different player's talk
    /// on the same day still does
    pub last_day_by: crate::engine::det::HashMap<Entity, u64>,
}

pub const TALK_RAPPORT_STEP: f32 = 0.15;
/// How much a talk still counts with someone you already know well: a
/// catch-up that keeps the relationship topped off against its daily decay
pub const TALK_MAINTENANCE: f32 = 0.1;

/// Player takes a talk action (1-1 meeting).
/// Repeat talks with the same person yield less; talking alone can't win.
/// The conversation also builds the person's trust in the player (their
/// relationship), so someone you've heard out is readier to join you at a
/// house meeting; it tapers off as you get to know them, down to
/// TALK_MAINTENANCE - a catch-up that keeps an old friendship from fading.
pub fn act_talk(world: &mut World, player: Entity, target: Entity, day: u64) -> ActionResult {
    use crate::game::relationships::{record_interaction, Interaction, InteractionKind};
    if world.get_component::<crate::game::villager::Dead>(target).is_some() {
        return ActionResult { success: false, description: "You can't talk with the dead.".into() };
    }
    let mut rap = world.get_component::<Rapport>(target).cloned().unwrap_or_default();
    let rapport = rap.by_player.get(&player).copied().unwrap_or(0.0);
    // a bonded pet at heel softens the talk - what a word earns warms a
    // little when the village's beasts trust you
    let pet_charmed = world.query::<crate::game::pets::Pet>().iter().any(|&e| {
        world.get_component::<crate::game::pets::Pet>(e)
            .map(|p| p.owner == Some(player) && p.bond_strength >= crate::game::pets::COMFORT_BOND)
            .unwrap_or(false)
    });
    let charm = if pet_charmed { 1.2 } else { 1.0 };
    rap.by_player.insert(player, (rapport + TALK_RAPPORT_STEP * charm).min(1.0));
    world.add_component(target, rap);
    let mut count = world.get_component::<TalkCount>(target).cloned().unwrap_or_default();
    let last_day = count.last_day_by.get(&player).copied().unwrap_or(u64::MAX);
    count.value += 1;
    count.last_day_by.insert(player, day);
    world.add_component(target, count);
    // you can't talk the same ear off twice in a day: only that
    // player's first real talk that day deepens the bond
    if last_day != day {
        record_interaction(world, player, Interaction {
            other: target, kind: InteractionKind::Conversation,
            intensity: ((1.0 - rapport).max(TALK_MAINTENANCE)) * charm,
        });
    }
    ActionResult {
        success: true,
        description: if rapport >= 1.0 { "Caught up with them".into() }
            else { "Had a conversation, learned their concerns".into() },
    }
}

/// Rapport the player needs with someone before asking them to join a
/// campaign: four real conversations, not a nod in passing
pub const RECRUIT_RAPPORT_NEEDED: f32 = 0.55;

/// `player`'s own rapport with `target` (0 = stranger) - only their own
/// conversations count, never another player's
pub fn rapport(world: &World, player: Entity, target: Entity) -> f32 {
    world.get_component::<Rapport>(target).and_then(|r| r.by_player.get(&player).copied()).unwrap_or(0.0)
}

/// A real conversation happened between `speaker` and `target`, outside
/// act_talk's reach - a house meeting, a 1-1. The talkee's warmth for the
/// speaker grows a talk's step, so the seat's bond ledger counts everyone
/// who actually spoke with them, not only who they sought out one by one
pub fn note_conversation(world: &mut World, speaker: Entity, target: Entity) {
    let mut rap = world.get_component::<Rapport>(target).cloned().unwrap_or_default();
    let cur = rap.by_player.get(&speaker).copied().unwrap_or(0.0);
    rap.by_player.insert(speaker, (cur + TALK_RAPPORT_STEP).min(1.0));
    world.add_component(target, rap);
}

/// Whether `target` has been woken to `campaign`'s issue already
pub fn sympathizes(world: &World, campaign: Entity, target: Entity) -> bool {
    world.get_component::<crate::game::campaign::Campaign>(campaign)
        .map(|c| world.get_component::<crate::game::reactions::Sympathized>(target)
            .map(|s| s.issues.iter().any(|(i, _)| *i == c.issue)).unwrap_or(false))
        .unwrap_or(false)
}

/// Player attempts to recruit someone into the org's campaign. There must
/// be a campaign to ask them into (one is founded at a house meeting), and
/// the player must have built up rapport with them personally. Then they
/// join if their own bond with the player - deepened by talks, meals and
/// shared work - is trust enough. Someone the cause has already woken
/// skips the persuading.
pub fn act_recruit(world: &mut World, player: Entity, org: Entity, target: Entity) -> ActionResult {
    let already = world.get_component::<crate::sim::community::Organization>(org)
        .map(|o| o.members.contains(&target)).unwrap_or(false);
    if already {
        return ActionResult { success: false, description: "Already a member".into() };
    }
    let Some(campaign) = crate::game::campaign::campaign_of(world, org) else {
        return ActionResult { success: false, description: "No campaign to recruit into".into() };
    };
    // They'll join on their own trust in the player (built by talking
    // with them and at meetings), helped by how well the player knows them
    let personal = crate::game::relationships::find_relationship(world, player, target)
        .and_then(|r| world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    // Someone woken to the cause signs on without being convinced -
    // the issue organized them before you ever knocked (moral shock)
    let sympathetic = sympathizes(world, campaign, target);
    if !sympathetic && rapport(world, player, target) < RECRUIT_RAPPORT_NEEDED {
        return ActionResult { success: false, description: "They hardly know you".into() };
    }
    let trust = if sympathetic { 1.0 } else { (personal + rapport(world, player, target) * 0.4).min(1.0) };
    let recruited = crate::sim::community::recruit(world, org, target, trust);
    ActionResult {
        success: recruited,
        description: if recruited { "They joined your cause".into() } else { "They declined".into() },
    }
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
    use crate::sim::community::Organization;

    fn setup() -> (World, Entity, Entity, Vec<Entity>) {
        let mut w = World::new();
        let p = w.create_entity();
        w.add_component(p, PlayerCharacter { name: "P".into(), resources: 100.0 });
        let o = w.create_entity();
        w.add_component(o, Organization { name: "O".into(), members: vec![], leader: p, influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
        let vs = (0..5).map(|_| w.create_entity()).collect();
        (w, p, o, vs)
    }

    /// Give org `o` a campaign to recruit into
    fn found(w: &mut World, o: Entity) { crate::game::campaign::found_for_test(w, o); }

    #[test]
    fn test_talk_spam_merely_caps_the_bond() {
        let (mut w, p, _, vs) = setup();
        for i in 0..500 { act_talk(&mut w, p, vs[i % 5], i as u64); }
        assert!(rapport(&w, p, vs[0]) <= 1.0, "the bond tops out at full");
    }

    #[test]
    fn test_recruit_needs_their_trust_not_a_score() {
        let (mut w, p, o, vs) = setup();
        found(&mut w, o);
        // known, but no bond: knowing them isn't the same as being trusted
        w.add_component(vs[0], Rapport::for_player(p, RECRUIT_RAPPORT_NEEDED));
        assert!(!act_recruit(&mut w, p, o, vs[0]).success, "a nod's acquaintance won't sign on");
        // a real friendship carries the ask
        let rel = w.create_entity();
        w.add_component(rel, crate::game::relationships::Relationship {
            from: p, other: vs[0], kind: crate::game::relationships::RelKind::Friend,
            strength: 0.6, interactions: 8 });
        assert!(act_recruit(&mut w, p, o, vs[0]).success, "a friend who trusts you joins");
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

    #[test]
    fn test_hearing_someone_out_builds_their_trust_and_tapers() {
        use crate::game::relationships::{find_relationship, Relationship};
        let (mut w, p, _, vs) = setup();
        let trust = |w: &World| find_relationship(w, p, vs[0])
            .and_then(|r| w.get_component::<Relationship>(r)).map(|r| r.strength).unwrap_or(0.0);
        act_talk(&mut w, p, vs[0], 0);
        assert!((trust(&w) - 0.10).abs() < 1e-6);
        for i in 0..20 { act_talk(&mut w, p, vs[0], i + 1); }
        let heard_out = trust(&w);
        // 0.10 x (1 + 0.85 + ... + 0.1), then 14 catch-ups at 0.1 each
        assert!((heard_out - 0.525).abs() < 1e-4, "talking someone through builds real but bounded trust, got {}", heard_out);
        act_talk(&mut w, p, vs[0], 21);
        let topped = trust(&w) - heard_out;
        assert!((topped - 0.10 * TALK_MAINTENANCE).abs() < 1e-6, "once you know them, a talk only tops the relationship off, got +{}", topped);
    }

    #[test]
    fn test_someone_who_trusts_you_joins_on_their_own_bond() {
        use crate::game::relationships::{Relationship, RelKind};
        let (mut w, p, o, vs) = setup();
        found(&mut w, o);
        for v in &vs[..2] { w.add_component(*v, Rapport::for_player(p, RECRUIT_RAPPORT_NEEDED)); }
        assert!(!act_recruit(&mut w, p, o, vs[0]).success, "a stranger won't join on a nod");
        let r = w.create_entity();
        w.add_component(r, Relationship { from: p, other: vs[1], kind: RelKind::Friend, strength: 0.6, interactions: 8 });
        assert!(act_recruit(&mut w, p, o, vs[1]).success, "a friend who trusts you joins anyway");
    }

    #[test]
    fn test_no_one_is_recruited_without_a_campaign() {
        let (mut w, p, o, vs) = setup();
        w.add_component(vs[0], Rapport::for_player(p, 1.0));
        let r = act_recruit(&mut w, p, o, vs[0]);
        assert!(!r.success && r.description == "No campaign to recruit into", "{}", r.description);
        found(&mut w, o);
        // rapport alone won't carry the ask - they need a bond too
        let bond = w.create_entity();
        w.add_component(bond, crate::game::relationships::Relationship {
            from: p, other: vs[0], kind: crate::game::relationships::RelKind::Friend,
            strength: 0.6, interactions: 8 });
        assert!(act_recruit(&mut w, p, o, vs[0]).success, "with a campaign to join they sign on");
    }

    #[test]
    fn test_recruiting_is_earned_by_kept_company() {
        let (mut w, p, o, vs) = setup();
        found(&mut w, o);
        // a nod in passing isn't an ask; kept company is
        for i in 0..3 { act_talk(&mut w, p, vs[0], i); }
        let r = act_recruit(&mut w, p, o, vs[0]);
        assert!(!r.success, "three talks aren't enough: {}", r.description);
        let mut joined = false;
        for i in 3..15 {
            act_talk(&mut w, p, vs[0], i);
            if act_recruit(&mut w, p, o, vs[0]).success { joined = true; break; }
        }
        assert!(joined, "enough kept company and they sign on");
    }

    #[test]
    fn test_another_players_talks_dont_carry_your_recruit() {
        // Two organizers courting the same villager: Ada builds real rapport
        // with Bram, but Bea - who has barely spoken to him - can't ride
        // Ada's conversations into a successful recruit of her own.
        let (mut w, ada, o_ada, vs) = setup();
        found(&mut w, o_ada);
        let bea = w.create_entity();
        w.add_component(bea, PlayerCharacter { name: "Bea".into(), resources: 100.0 });
        let o_bea = w.create_entity();
        w.add_component(o_bea, Organization { name: "O2".into(), members: vec![], leader: bea, influence_pool: 0.0, resources: 0.0, charter: "".into(), successors: vec![] });
        found(&mut w, o_bea);
        let bram = vs[0];
        for i in 0..20 { act_talk(&mut w, ada, bram, i); }
        assert!(rapport(&w, ada, bram) >= RECRUIT_RAPPORT_NEEDED, "Ada earned real rapport");
        assert_eq!(rapport(&w, bea, bram), 0.0, "Bea's own rapport is untouched by Ada's talks");
        assert!(!act_recruit(&mut w, bea, o_bea, bram).success, "Bea hasn't earned Bram's trust herself");
        assert!(act_recruit(&mut w, ada, o_ada, bram).success, "Ada's own kept company still carries her ask");
    }
}
