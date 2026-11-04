use crate::*;
use game2_core::game::desires;
use game2_core::sim::community::Organization;

/// A tax campaign with six members, of whom only the first answers
/// the organizer's call - a thin room
fn thin_tax_campaign() -> Game {
    let mut g = Game::new_seeded(MAP_SEED);
    let pe = g.player;
    for &v in &g.villagers[..6] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 400);
    }
    for _ in 0..3 {
        game2_core::game::relationships::record_interaction(&mut g.world, g.villagers[0],
            game2_core::game::relationships::Interaction {
                other: pe, kind: game2_core::game::relationships::InteractionKind::Aid, intensity: 1.0 });
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
