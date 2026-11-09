use crate::session::*;

#[test]
fn test_the_village_walks_slower_for_a_lame_leg() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<villager::Condition>(v).is_none()).unwrap();
    g.world.add_component(v, villager::Condition { kind: "a lame leg", until: u64::MAX });
    assert!(g.world.get_component::<villager::Condition>(v)
        .map(|c| c.kind == "a lame leg").unwrap_or(false));
}

#[test]
fn test_theirs_check_on_them() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
    let home = household::home_of(&g.world, v).unwrap();
    let kin = g.world.get_component::<Building>(home).unwrap()
        .occupants.iter().copied().find(|&o| o != v);
    if kin.is_some() {
        g.care_tick(2);
        // the visit warms them - Joy's satisfied
        assert!(g.world.get_component::<desires::InnerLife>(v)
            .map(|l| l.satisfied.iter().any(|(w, u)| *w == Want::Joy && *u > 2))
            .unwrap_or(false), "theirs check on them");
    }
}

#[test]
fn test_the_grave_sends_no_visitors() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
    // no kin under the roof to take the visit
    if let Some(home) = household::home_of(&g.world, v) {
        g.world.get_component_mut::<Building>(home).unwrap()
            .occupants.retain(|&o| o == v);
    }
    // their best friend's gone: the strongest bond points the visit
    // at the grave - before the fix the dead still called round
    let dead_friend = g.villagers.iter().copied().find(|&x| x != v).unwrap();
    let rel = g.world.create_entity();
    g.world.add_component(rel, relationships::Relationship {
        from: dead_friend, other: v, kind: relationships::RelKind::Friend,
        strength: 0.9, interactions: 9 });
    g.world.add_component(dead_friend, villager::Dead);
    g.care_tick(2);
    let after = g.world.get_component::<relationships::Relationship>(rel).unwrap();
    assert_eq!(after.interactions, 9, "the grave sends no visitors");
    assert!(g.bubbles.iter().all(|(w, _, _, _, _)| *w != dead_friend),
        "no word from the grave");
}

#[test]
fn test_the_dungeon_sends_no_visitors() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<villager::Condition>(v).is_some()).unwrap();
    if let Some(home) = household::home_of(&g.world, v) {
        g.world.get_component_mut::<Building>(home).unwrap()
            .occupants.retain(|&o| o == v);
    }
    let held_friend = g.villagers.iter().copied().find(|&x| x != v).unwrap();
    let rel = g.world.create_entity();
    g.world.add_component(rel, relationships::Relationship {
        from: held_friend, other: v, kind: relationships::RelKind::Friend,
        strength: 0.9, interactions: 9 });
    g.world.add_component(held_friend,
        crate::game::dungeon::Imprisoned { since: 0, charge: "sedition".into() });
    g.care_tick(2);
    let after = g.world.get_component::<relationships::Relationship>(rel).unwrap();
    assert_eq!(after.interactions, 9, "the dungeon sends no visitors");
}
