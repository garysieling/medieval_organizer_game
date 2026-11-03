use crate::session::*;

/// A tax campaign with six members, of whom only the first answers
/// the organizer's call - a thin room
fn thin_tax_campaign() -> Game {
    let mut g = Game::new();
    let pe = g.player;
    for &v in &g.villagers[..6] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 400);
    }
    for _ in 0..3 {
        crate::game::relationships::record_interaction(&mut g.world, g.villagers[0],
            crate::game::relationships::Interaction {
                other: pe, kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
    }
    g.world.get_component_mut::<Organization>(g.org).unwrap().members = g.villagers[..6].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.player).unwrap().issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
    g
}

/// For eyes: the TALK panel after a townhall, rendered as --dump-frame
/// would. `CAMPAIGN_FRAME=<path.bmp> cargo test -p game2-core --lib
/// campaign_talk_frame -- --ignored`
#[test]
#[ignore]
fn campaign_talk_frame() {
    let Ok(path) = std::env::var("CAMPAIGN_FRAME") else { return };
    let mut g = thin_tax_campaign();
    let reeve = g.collector.unwrap();
    desires::add_grievance(&mut g.world, reeve, desires::Problem::Taxed, g.tick + 400);
    g.seats[0].selected_target = COLLECTOR_IDX;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Townhall);
    g.ctx_act_at(0, 0);
    g.seats[0].ctx_actions.clear();
    g.seats[0].ctx_actions.push(CtxAct::March);
    g.ctx_act_at(0, 0);
    g.seats[0].picked = false; // your own talk on the panel
    let view = View::new(Art::load_fs("../assets", 0));
    let buf = view.render(&g.ui_model(), (-1, -1));
    buf.save_bmp(&path).unwrap();
}

#[test]
fn test_pressing_again_doesnt_stretch_a_grant() {
    let mut g = thin_tax_campaign();
    let reeve = g.collector.unwrap();
    desires::add_grievance(&mut g.world, reeve, desires::Problem::Taxed, g.tick + 400);
    g.seats[0].selected_target = COLLECTOR_IDX;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Townhall);
    g.ctx_act_at(0, 0);
    let until = g.world.get_component::<crate::game::reactions::LordGrant>(g.org).unwrap().until;
    // a part-grant can be pressed, but a second thin room gets nothing new
    assert!(g.ctx_blocker_at(0, CtxAct::Townhall).is_none());
    g.tick += 30;
    g.ctx_act_at(0, 0);
    assert!(g.log.iter().any(|l| l.contains("no more than he already has")), "{:?}", g.log.last());
    assert_eq!(g.world.get_component::<crate::game::reactions::LordGrant>(g.org).unwrap().until, until);
    // a full grant, honoured, closes the Reeve's door to the same demands
    let issue = "the taxes";
    g.world.add_component(g.org, crate::game::reactions::LordGrant::full(issue, g.tick, g.tick + 100));
    let why = g.ctx_blocker_at(0, CtxAct::Townhall).expect("he's answered");
    assert!(why.contains("grant stands"), "{}", why);
    // hollowed, it can be pressed again
    g.world.get_component_mut::<crate::game::reactions::LordGrant>(g.org).unwrap().hollow = true;
    assert!(g.ctx_blocker_at(0, CtxAct::Townhall).is_none());
}

#[test]
fn test_a_thin_room_wins_part_and_an_unwatched_grant_goes_hollow() {
    let mut g = thin_tax_campaign();
    let reeve = g.collector.unwrap();
    desires::add_grievance(&mut g.world, reeve, desires::Problem::Taxed, g.tick + 400);
    g.seats[0].selected_target = COLLECTOR_IDX;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Townhall);
    g.ctx_act_at(0, 0);
    assert!(g.log.iter().any(|l| l.contains("grants a part of it")), "{:?}", g.log.last());
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    let f = crate::game::campaign_arc::arc_facts(&g.world, c, g.tick);
    assert!(f.has("arc:partial") && !f.has("arc:won"), "a part-grant reads as partial");

    // the tax round honours a part: a quarter back, the grievance stands
    let v = g.villagers[0];
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
    g.arrive(reeve, Goal::CollectTax(v));
    assert!(g.log.iter().any(|l| l.contains("part-grant covers")), "{:?}", g.log.last());
    assert!(desires::problems_of(&g.world, v, g.tick).contains(&desires::Problem::Taxed),
        "a part-grant doesn't answer the grievance");

    // the campaign stops meeting: at the next round the grant stays
    // on paper but the collector takes the full cut
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c).unwrap().last_held = 0;
    g.tick += 60;
    assert!(crate::game::campaign::lapsed(&g.world, c, g.tick));
    let covered_before = g.log.iter().filter(|l| l.contains("covers")).count();
    g.world.get_component_mut::<Purse>(v).unwrap().gold = 50.0;
    g.arrive(reeve, Goal::CollectTax(v));
    assert!(g.log.iter().any(|l| l.contains("still stands on paper") && l.contains("Meet again")), "{:?}", g.log.last());
    assert_eq!(g.log.iter().filter(|l| l.contains("covers")).count(), covered_before, "no relief once hollow");
    let f = crate::game::campaign_arc::arc_facts(&g.world, c, g.tick);
    assert!(f.has("arc:diluted") && !f.has("arc:partial"), "granted in name only");
}
