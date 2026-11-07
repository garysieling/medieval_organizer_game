use crate::session::*;

#[test]
fn test_the_knights_take_the_loudest_and_the_letters_bring_them_home() {
    let mut g = Game::new();
    for &v in &g.villagers[..4] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..4].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // name villagers[0] the turnout coordinator, bond the rest to him
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Delegate);
    g.ctx_act_at(0, 0);
    for &v in &g.villagers[1..4] {
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, v,
                crate::game::relationships::Interaction {
                    other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
    }
    // a real crowd draws the knights, and they take the loudest
    g.seats[0].ctx_actions.push(CtxAct::March);
    g.ctx_act_at(0, 1);
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(g.log.iter().any(|l| l.contains("knights come out") && l.contains("is taken")), "{:?}", g.log);
    // then the letters bring him home
    g.seats[0].ctx_actions = vec![CtxAct::Petition];
    for _ in 0..4 { g.ctx_act_at(0, 0); }
    assert!(g.log.iter().any(|l| l.contains("walks free")), "{:?}", g.log);
}

#[test]
fn test_the_cell_holds_them_they_dont_live_their_lives() {
    let mut g = Game::new();
    let v = g.villagers[0];
    let at = g.world.get_component::<Pos>(v).copied().unwrap();
    let hunger = g.world.get_component::<Hunger>(v).unwrap().value;
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
    // the held take no errands
    assert!(!g.send(v, Goal::Wander));
    // days pass: they don't stroll off, don't hunger in the cell
    for _ in 0..600 { g.update(); }
    let now = g.world.get_component::<Pos>(v).copied().unwrap();
    assert_eq!((at.x, at.y), (now.x, now.y), "the held stand where the guards left them");
    assert_eq!(g.world.get_component::<Hunger>(v).unwrap().value, hunger,
        "the castle feeds its prisoners - no starving where they stand");
}

#[test]
fn test_the_held_walk_free_when_the_village_never_stops_asking() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
    // no campaign, no petition - just the days passing and the
    // remembering that gathers a little every one of them
    for t in 1..55 { g.marks_tick(g.tick + t); }
    assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
        "fifty days of asking is a sentence served");
    assert!(g.log.iter().any(|l| l.contains("walks free")), "{:?}", g.log);
}

#[test]
fn test_the_taken_get_a_release_campaign_and_their_friends_sign_on() {
    let mut g = Game::new();
    for &v in &g.villagers[..4] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..4].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    // the loudest leads, and a bystander loves him enough to feel the take
    for &v in &g.villagers[1..4] {
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, v,
                crate::game::relationships::Interaction {
                    other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
    }
    let friend = g.villagers[6];
    for _ in 0..3 {
        crate::game::relationships::record_interaction(&mut g.world, friend,
            crate::game::relationships::Interaction {
                other: g.villagers[0], kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
    }
    g.seats[0].ctx_actions.push(CtxAct::March);
    g.ctx_act_at(0, 0);
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    let prisoner = g.villagers.iter().copied()
        .find(|&v| g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_some())
        .expect("the crackdown takes the loudest");
    // the cause becomes a person: a campaign of their own
    let rc = g.world.get_component::<crate::game::dungeon::ReleaseOf>(prisoner)
        .map(|r| r.campaign).expect("a release campaign stands for the taken");
    assert!(g.world.get_component::<crate::game::campaign::Campaign>(rc).is_some());
    // and the ones who loved them sign on without an invite
    assert!(g.world.get_component::<Organization>(g.org).unwrap().members.contains(&friend),
        "the friend signs on for the release");
}

#[test]
fn test_you_cant_talk_a_man_out_of_a_cell() {
    let mut g = Game::new();
    // stand him beside the player, then the knights take him
    let v = g.villagers[0];
    let (px, py) = g.player_pos_at(0);
    *g.world.get_component_mut::<Pos>(v).unwrap() = Pos { x: px + 1.0, y: py };
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
    // he's not around to be reached, and the buttons say why
    assert!(!g.villagers_in_reach().contains(&v));
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    assert_eq!(g.disabled_reason(0).as_deref(), Some("in the dungeon"));
    assert_eq!(g.disabled_reason(1).as_deref(), Some("in the dungeon"));
    g.act(0);
    assert!(g.log.last().unwrap().contains("in the dungeon"), "{:?}", g.log.last());
}

#[test]
fn test_the_knights_ride_home_when_the_tour_is_done() {
    let mut g = Game::new();
    g.world.add_component(g.org, crate::game::reactions::GuardsPosted { knights: 2, since: 10 });
    g.posted.push((g.org, 15.0, 16.0));
    g.posted.push((g.org, 14.0, 16.0));
    // another org's knights still stand their own tour
    let other = g.world.create_entity();
    g.posted.push((other, 20.0, 20.0));
    // mid-tour they stand their ground
    g.guards_tick(20);
    assert!(g.world.get_component::<crate::game::reactions::GuardsPosted>(g.org).is_some());
    assert_eq!(g.posted.len(), 3);
    // the tour done, they ride out - the posting and the street both
    // clear, but only theirs: another org's knights keep their own tour
    g.guards_tick(40);
    assert!(g.world.get_component::<crate::game::reactions::GuardsPosted>(g.org).is_none());
    assert_eq!(g.posted.len(), 1);
    assert_eq!(g.posted[0].0, other);
    // the gate watch, though, stands all year
    assert!(!g.guards.is_empty());
    assert!(g.log.iter().any(|l| l.contains("knights ride home")), "{:?}", g.log);
}

#[test]
fn test_petition_shows_even_when_the_cause_is_spent() {
    let mut g = Game::new();
    // no campaign at all - and someone's inside
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Petition),
        "the way out doesn't depend on a live campaign: {:?}", g.seats[0].ctx_actions);
}

#[test]
fn test_the_cell_opens_when_the_knights_ride_home() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = vec![v];
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: g.tick, charge: "sedition".into() });
    g.world.add_component(g.org, crate::game::reactions::GuardsPosted { knights: 2, since: 0 });
    // petitions unanswered and the tour still ends: the cell opens
    g.guards_tick(30);
    assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
        "no life sentence - the knights gone, the door stands open");
    assert!(g.log.iter().any(|l| l.contains("walks home")), "{:?}", g.log);
}

#[test]
fn test_the_stint_is_the_stint_whatever_the_tours_do() {
    let mut g = Game::new();
    let v = g.villagers[0];
    g.world.add_component(v, crate::game::dungeon::Imprisoned { since: 0, charge: "sedition".into() });
    // a fresh crackdown every week keeps the tour going - the prisoner
    // still walks when their own month is done
    g.world.add_component(g.org, crate::game::reactions::GuardsPosted { knights: 2, since: 25 });
    g.guards_tick(31);
    assert!(g.world.get_component::<crate::game::dungeon::Imprisoned>(v).is_none(),
        "a restamped tour can't stretch the stint past a month");
}
