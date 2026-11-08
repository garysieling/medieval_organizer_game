use crate::session::*;

/// A taxed cause founded the way the campaign tests found one
fn taxed_campaign(g: &mut Game) -> Entity {
    for &v in &g.villagers[..3] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members =
        g.villagers[..3].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    crate::game::campaign::campaign_of(&g.world, g.org).unwrap()
}

#[test]
fn test_the_organizer_calls_the_assembly_and_the_agenda_posts() {
    let mut g = Game::new();
    let c = taxed_campaign(&mut g);
    // nothing on the panel's line till the institution's called
    assert!(g.ui_model().panel.campaign.unwrap().assembly.is_empty());
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Assembly),
        "{:?}", g.seats[0].ctx_actions);
    let i = g.seats[0].ctx_actions.iter().position(|&a| a == CtxAct::Assembly).unwrap();
    g.ctx_act_at(0, i);
    assert!(g.log.last().unwrap().contains("accountability assembly"),
        "{:?}", g.log.last());
    let a = g.world.get_component::<crate::game::campaign::Assembly>(c).unwrap();
    assert!(a.announced.is_some() && !a.agenda.is_empty(), "the agenda stands");
    assert!(g.ui_model().panel.campaign.unwrap().assembly.contains("agenda"),
        "{:?}", g.ui_model().panel.campaign.unwrap().assembly);
    // the card won't take a second call the same day - nor offer one
    // while the posted agenda still stands
    assert!(g.ctx_blocker_at(0, CtxAct::Assembly).is_some(), "same-day call refused");
    g.update_ctx_actions_at(0);
    assert!(!g.seats[0].ctx_actions.contains(&CtxAct::Assembly),
        "an agenda's up: {:?}", g.seats[0].ctx_actions);
    // the publish gap passes and the assembly sits on its own
    g.tick += crate::game::campaign::PUBLISH_GAP;
    g.assembly_tick(g.tick);
    let a = g.world.get_component::<crate::game::campaign::Assembly>(c).unwrap();
    assert_eq!(a.held, 1, "the assembly sat");
    assert!(a.announced.is_none() && !a.pledges.is_empty(),
        "the leader's answers are on record");
    g.update_ctx_actions_at(0);
    assert!(g.seats[0].ctx_actions.contains(&CtxAct::Assembly),
        "the card offers the next call: {:?}", g.seats[0].ctx_actions);
}

#[test]
fn test_the_assembly_calls_itself_and_sits_on_its_own_dates() {
    let mut g = Game::new();
    let c = taxed_campaign(&mut g);
    let founded = g.world.get_component::<crate::game::campaign::Campaign>(c)
        .unwrap().founded_day;
    // the institution isn't there the day the cause is - it
    // establishes itself a while into the fight
    g.tick = founded + crate::game::campaign::FIRST_ASSEMBLY_DELAY - 1;
    g.assembly_tick(g.tick);
    assert!(g.world.get_component::<crate::game::campaign::Assembly>(c).is_none(),
        "too early for the institution");
    g.tick += 1;
    g.assembly_tick(g.tick);
    let a = g.world.get_component::<crate::game::campaign::Assembly>(c).unwrap();
    assert_eq!(a.last_held, founded, "the clock runs from the founding");
    // the cadence calls it: the agenda posts, the gap passes, it sits
    g.tick = founded + crate::game::campaign::ASSEMBLY_CADENCE;
    g.assembly_tick(g.tick);
    assert!(g.world.get_component::<crate::game::campaign::Assembly>(c)
        .unwrap().announced.is_some(), "the agenda's posted");
    assert!(g.log.iter().any(|l| l.contains("accountability assembly is called")),
        "{:?}", g.log.last());
    g.tick += crate::game::campaign::PUBLISH_GAP;
    g.assembly_tick(g.tick);
    let a = g.world.get_component::<crate::game::campaign::Assembly>(c).unwrap();
    assert_eq!(a.held, 1);
    assert!(!a.pledges.is_empty(), "the leader answered on the record");
    // the panel carries the standing
    let line = g.ui_model().panel.campaign.unwrap().assembly;
    assert!(!line.is_empty(), "the panel says where the pledges stand");
}

#[test]
fn test_a_broken_pledge_chills_the_room_that_answered() {
    let mut g = Game::new();
    let c = taxed_campaign(&mut g);
    // ties from two members to the leader, so the room answers the call
    let org = g.org;
    let leader = crate::game::organizing::living_leader(&mut g.world, org).unwrap();
    for &v in &g.villagers[1..3] {
        crate::game::relationships::record_interaction(&mut g.world, v,
            crate::game::relationships::Interaction {
                other: leader,
                kind: crate::game::relationships::InteractionKind::Aid,
                intensity: 1.0 });
    }
    crate::game::campaign::announce_assembly(&mut g.world, c, g.tick);
    let held = g.tick + crate::game::campaign::PUBLISH_GAP;
    let out = crate::game::campaign::hold_assembly(&mut g.world, c, held).unwrap();
    assert!(out.attendees.len() >= 2, "the tied members answered: {:?}", out.attendees);
    // make the verdicts knowable: every check reads an action count
    // that can't move, so a yes on the record must break
    {
        let a = g.world.get_component_mut::<crate::game::campaign::Assembly>(c).unwrap();
        for p in &mut a.pledges {
            p.check = crate::game::campaign::PledgeCheck::Action { from: u32::MAX };
            p.answer = true;
            p.deadline = held;
        }
    }
    let bond_before = crate::game::relationships::find_relationship(&g.world,
        g.villagers[1], leader)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    let verdicts = crate::game::campaign::resolve_pledges(&mut g.world, c, held);
    assert_eq!(verdicts.len(), out.pledges.len());
    assert!(verdicts.iter().all(|v| !v.kept), "every yes broke: {:?}", verdicts);
    assert_eq!(crate::game::campaign::broken_pledges(&g.world, c).len(), verdicts.len());
    // the room that answered heard the broken word - the bond cooled
    let bond_after = crate::game::relationships::find_relationship(&g.world,
        g.villagers[1], leader)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    assert!(bond_after < bond_before, "the chill reached the room: {bond_before} -> {bond_after}");
    // and the consent the meetings banked didn't survive it
    assert!(!g.world.get_component::<crate::game::campaign_arc::ArcRecord>(c)
        .map(|r| r.consent).unwrap_or(true), "a broken word spends the consent");
    // the panel counts the break
    let line = g.ui_model().panel.campaign.unwrap().assembly;
    assert!(line.contains("broke"), "{line}");
}

#[test]
fn test_a_kept_word_warms_the_room() {
    let mut g = Game::new();
    let c = taxed_campaign(&mut g);
    let org = g.org;
    let leader = crate::game::organizing::living_leader(&mut g.world, org).unwrap();
    for &v in &g.villagers[1..3] {
        crate::game::relationships::record_interaction(&mut g.world, v,
            crate::game::relationships::Interaction {
                other: leader,
                kind: crate::game::relationships::InteractionKind::Aid,
                intensity: 1.0 });
    }
    crate::game::campaign::announce_assembly(&mut g.world, c, g.tick);
    let held = g.tick + crate::game::campaign::PUBLISH_GAP;
    crate::game::campaign::hold_assembly(&mut g.world, c, held).unwrap();
    // a no on a thing that can't happen is a refusal kept - the word
    // held, and the room that heard it warms
    {
        let a = g.world.get_component_mut::<crate::game::campaign::Assembly>(c).unwrap();
        for p in &mut a.pledges {
            p.check = crate::game::campaign::PledgeCheck::Action { from: u32::MAX };
            p.answer = false;
            p.deadline = held;
        }
    }
    let bond_before = crate::game::relationships::find_relationship(&g.world,
        g.villagers[1], leader)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    let verdicts = crate::game::campaign::resolve_pledges(&mut g.world, c, held);
    assert!(verdicts.iter().all(|v| v.kept), "every refusal held: {:?}", verdicts);
    assert!(crate::game::campaign::broken_pledges(&g.world, c).is_empty());
    let bond_after = crate::game::relationships::find_relationship(&g.world,
        g.villagers[1], leader)
        .and_then(|r| g.world.get_component::<crate::game::relationships::Relationship>(r))
        .map(|r| r.strength).unwrap_or(0.0);
    assert!(bond_after > bond_before, "the word kept warmed the room: {bond_before} -> {bond_after}");
}
