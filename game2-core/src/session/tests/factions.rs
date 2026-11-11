use crate::session::*;
use crate::sim::factions::*;
use crate::sim::religion::Congregation;

fn faction_named(g: &Game, want: &str) -> Entity {
    g.factions.iter().copied().find(|&e|
        g.world.get_component::<Organization>(e)
            .map(|o| o.name.contains(want)).unwrap_or(false))
        .unwrap_or_else(|| panic!("no faction named {want}"))
}

#[test]
fn test_the_parish_and_the_market_stand_at_the_start() {
    let g = Game::new();
    let church = faction_named(&g, "parish");
    let market = faction_named(&g, "market");
    for e in [church, market] {
        assert!(g.world.get_component::<Faction>(e).is_some(),
            "a seeded power center carries the tag");
        let o = g.world.get_component::<Organization>(e).unwrap();
        assert_eq!(o.leader, e, "no soul speaks for the interest");
    }
}

#[test]
fn test_a_factions_people_are_read_not_stored() {
    let g = Game::new();
    let church = faction_named(&g, "parish");
    let f = g.world.get_component::<Faction>(church).unwrap();
    let members = members_of(&g.world, f);
    let cong = g.world.get_component::<Congregation>(g.congregation).unwrap();
    let living = cong.members.iter().filter(|&&v| !villager::is_dead(&g.world, v)).count();
    assert_eq!(members.len(), living, "the parish's people are the congregation's");
    assert!(g.world.get_component::<Organization>(church).unwrap().members.is_empty(),
        "and the org's own list stays empty - membership is derived");
}

#[test]
fn test_a_trade_grown_thick_takes_form() {
    let mut g = Game::new();
    // three worked shops under one trade - enough to act as one
    let worked: Vec<(Entity, Vocation)> = g.building_entities.iter().copied()
        .filter_map(|b| g.world.get_component::<Building>(b).and_then(|x|
            match x.kind {
                BuildingKind::Shop(v) if x.worker.is_some() => Some((b, v)),
                _ => None,
            })).collect();
    assert!(worked.len() >= 3, "the village keeps at least three shops");
    let trade = worked[0].1;
    for &(b, _) in worked.iter().take(3) {
        g.world.get_component_mut::<Building>(b).unwrap().kind = BuildingKind::Shop(trade);
    }
    g.faction_tick(5);
    let fellowship = g.factions.iter().copied().find(|&e|
        g.world.get_component::<Faction>(e)
            .map(|f| f.kind == FactionKind::Guild { trade }).unwrap_or(false))
        .expect("the trade takes form");
    let f = g.world.get_component::<Faction>(fellowship).unwrap();
    assert_eq!(members_of(&g.world, f).len(), 3, "its people are the shops' holders");
    assert!(g.log.iter().any(|l| l.contains("fellowship")));
    // and it doesn't spawn twice
    let n = g.factions.len();
    g.faction_tick(12);
    assert_eq!(g.factions.len(), n, "a faction keeps once it forms");
}

#[test]
fn test_a_household_grown_wide_takes_form() {
    let mut g = Game::new();
    let home = g.building_entities.iter().copied().find(|&b|
        g.world.get_component::<Building>(b)
            .map(|x| x.kind == BuildingKind::Home).unwrap_or(false)).unwrap();
    // the household grows past the count - add souls under the roof
    while g.world.get_component::<Building>(home).unwrap().occupants.len() < FAMILY_SOULS {
        let extra = g.villagers.iter().copied().find(|&v|
            !g.world.get_component::<Building>(home).unwrap().occupants.contains(&v)).unwrap();
        g.world.get_component_mut::<Building>(home).unwrap().occupants.push(extra);
    }
    g.faction_tick(5);
    let house = g.factions.iter().copied().find(|&e|
        g.world.get_component::<Faction>(e)
            .map(|f| f.kind == FactionKind::Family { home }).unwrap_or(false))
        .expect("the house answers to its own name");
    let o = g.world.get_component::<Organization>(house).unwrap();
    assert!(o.name.starts_with("the house of"));
}

#[test]
fn test_a_factions_pull_is_its_peoples() {
    let mut g = Game::new();
    g.faction_tick(5);
    let church = faction_named(&g, "parish");
    let f = g.world.get_component::<Faction>(church).unwrap();
    let n = members_of(&g.world, f).len();
    let pool = g.world.get_component::<Organization>(church).unwrap().influence_pool;
    assert!(pool > 0.0, "a village of souls has pull");
    assert!((pool - n as f32 * 0.2).abs() < 0.01,
        "the pull is the sum of its people's influence");
    // and the member-roll accounting leaves it alone - a faction keeps
    // no rolls, so a member-sum would read as nothing
    g.update();
    let pool = g.world.get_component::<Organization>(church).unwrap().influence_pool;
    assert!(pool > 0.0, "the interest's pull isn't its empty rolls'");
}

#[test]
fn test_the_talk_can_name_what_pulls_on_them() {
    let g = Game::new();
    let v = g.villagers.iter().copied().find(|&v|
        g.world.get_component::<Craft>(v).is_some()).unwrap();
    let s = crate::session::dialogue::speaker_of(&g.world, v, "test", 0);
    assert!(s.status.contains(&"parish_faction"),
        "a villager of the parish carries its fact into talk");
    let agent_org_leak = g.agent_obs(0).unwrap().orgs.iter()
        .any(|o| o.contains("parish") || o.contains("market"));
    assert!(!agent_org_leak, "a faction is a power center, not a banner");
}
