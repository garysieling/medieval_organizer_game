use crate::session::*;

#[test]
fn test_the_player_stands_in_the_gathering() {
    let mut g = Game::new();
    // a holy day: the gathering's at hand and the button should fire
    let holy = (0..40).find(|d| routine::is_holy(*d)).unwrap_or(7);
    g.tick = holy;
    let had_faith = g.world.get_component::<desires::InnerLife>(g.seats[0].player)
        .map(|l| l.wants.contains(&desires::Want::Faith)).unwrap_or(false);
    if !g.seats[0].ctx_actions.contains(&crate::session::seat::CtxAct::Attend) {
        g.seats[0].ctx_actions.push(crate::session::seat::CtxAct::Attend);
    }
    let i = g.seats[0].ctx_actions.iter().position(|a| *a == crate::session::seat::CtxAct::Attend).unwrap();
    g.ctx_act_at(0, i);
    // the day's rite counts once: attended_days holds the player
    assert!(g.attended_days.iter().any(|&(p, d)| p == g.seats[0].player && d == holy),
        "the attend never landed");
    // and the faith want found its satisfaction where the rite touched it
    if had_faith {
        assert!(!g.world.get_component::<desires::InnerLife>(g.seats[0].player)
            .map(|l| l.wants.contains(&desires::Want::Faith)).unwrap_or(true),
            "the rite didn't quiet the want it answers");
    }
}

#[test]
fn test_a_birthday_lands_where_the_means_can_take_it() {
    let mut g = Game::new();
    // force a celebrant: the first adult villager's day is today
    let v = g.villagers[3];
    g.world.get_component_mut::<crate::game::lifecycle::Age>(v)
        .map(|a| a.born_day = (g.tick % 360) as u16);
    let before = g.log.len();
    g.birthday_tick(g.tick);
    assert!(g.log.len() > before, "no observance fired");
    assert!(g.log.last().unwrap().contains("birthday"), "the log didn't name the day");
}
