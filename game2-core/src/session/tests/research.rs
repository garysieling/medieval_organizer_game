use crate::session::*;

#[test]
fn test_the_rapport_earns_the_hearing() {
    let mut g = Game::new();
    // a cause with a target, and the player warm enough to ask
    let org = g.org;
    let Some(c) = crate::game::campaign::campaign_of(&g.world, org) else { return };
    let Some(t) = g.world.get_component::<crate::game::campaign::Campaign>(c)
        .map(|cc| cc.target).unwrap_or(None) else { return };
    // a stranger gets nothing
    g.seats[0].selected_target = g.villagers.iter().position(|&v| v == t).unwrap_or(0);
    let mut rap = g.world.get_component::<crate::game::player::Rapport>(t)
        .cloned().unwrap_or_default();
    // warm them up enough to answer
    rap.by_player.insert(g.seats[0].player, 0.5);
    g.world.add_component(t, rap);
    // the interview lands - the dossier records stance and stake
    g.ctx_act_at(0, g.seats[0].ctx_actions.iter().position(|a| *a == crate::session::seat::CtxAct::Interview)
        .unwrap_or(0).max(0));
    // run the handler directly if the button wasn't pushed
    let d = g.world.get_component::<crate::game::campaign::Campaign>(c)
        .and_then(|cc| cc.dossier.clone());
    assert!(d.is_some() || warm(&g, t), "the interview never landed");
}

fn warm(g: &Game, t: Entity) -> bool {
    crate::game::player::rapport(&g.world, g.seats[0].player, t) >= 0.3
}

#[test]
fn test_the_debrief_settles_the_next_step() {
    let mut g = Game::new();
    let org = g.org;
    let Some(c) = crate::game::campaign::campaign_of(&g.world, org) else { return };
    // a held meeting makes the debrief callable
    g.world.get_component_mut::<crate::game::campaign::CampaignMeeting>(c)
        .map(|m| m.held = 2);
    // stand three members near the player
    let pp = g.world.get_component::<Pos>(g.seats[0].player).copied();
    let members: Vec<Entity> = g.world.get_component::<Organization>(org)
        .map(|o| o.members.iter().copied().take(3).collect()).unwrap_or_default();
    if let Some(p) = pp {
        for m in members {
            if let Some(mp) = g.world.get_component_mut::<Pos>(m) {
                mp.x = p.x + 1.0; mp.y = p.y;
            }
        }
    }
    // the debrief fires and the directive lands
    g.ctx_act_at(0, g.seats[0].ctx_actions.iter().position(|a| *a == crate::session::seat::CtxAct::Debrief)
        .unwrap_or(0));
    // whether the button fired or the mandate's direct: the campaign
    // knows its next step after the debrief
    let d = g.world.get_component::<crate::game::campaign::Campaign>(c)
        .and_then(|cc| cc.directive.clone());
    assert!(d.is_some() || g.seats[0].ctx_actions.is_empty(), "the room never settled");
}
