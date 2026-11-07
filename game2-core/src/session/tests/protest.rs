use crate::session::*;

fn voteless(g: &Game) -> Entity {
    g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap()
}

#[test]
fn test_a_picket_calls_the_neighbours_with_signs() {
    let mut g = Game::new();
    let v = voteless(&g);
    let t = g.tick;
    g.apply_sanction(v, Sanction::Picket, t);
    let p = g.protests.iter().find(|p| p.until == t + 3)
        .expect("the picket is called for three days");
    assert!(!p.folk.contains(&v), "the sanctioned one doesn't picket themselves");
    assert!(p.folk.iter().all(|u| g.villagers.contains(u)), "only villagers carry signs");
    assert!(p.folk.len() <= 6, "a neighbourhood, not a riot");
    assert!(!p.folk.is_empty(), "someone answers the call");
    // the crowd holds the rally: protesting routines facing the middle
    for &u in &p.folk {
        let r = g.world.get_component::<routine::Routine>(u).expect("a plan");
        assert!(matches!(r.mode, routine::Mode::Protesting), "{:?} isn't protesting", r.mode);
        assert_eq!(r.spot, routine::Spot::Rally);
        assert_eq!(r.day, t);
        assert_eq!(r.drift, 0.0, "they hold their slot");
    }
    // and the model says who's carrying which board
    let m = g.ui_model();
    for &u in &p.folk {
        let pv = m.people.iter().find(|p| p.e == u).expect("on the model");
        assert!(pv.sign.is_some(), "a protester carries a sign");
    }
    let other = g.villagers.iter().copied().find(|u| !p.folk.contains(u)).unwrap();
    let pv = m.people.iter().find(|p| p.e == other).unwrap();
    assert!(pv.sign.is_none(), "bystanders carry nothing");
}

#[test]
fn test_the_picket_holds_for_its_days_then_goes_home() {
    let mut g = Game::new();
    let v = voteless(&g);
    g.apply_sanction(v, Sanction::Picket, 10);
    assert_eq!(g.protests.len(), 1);
    // everyone arrived overnight; the dawn call re-seats them for the new day
    g.travels.clear();
    g.protest_dawn(12);
    assert_eq!(g.protests.len(), 1, "still out on the second day");
    for &u in &g.protests[0].folk.clone() {
        let r = g.world.get_component::<routine::Routine>(u).expect("a plan");
        assert!(matches!(r.mode, routine::Mode::Protesting) && r.day == 12,
            "re-seated for day 12");
    }
    g.protest_dawn(13);
    assert!(g.protests.is_empty(), "the last day was the 12th - they go home");
}

#[test]
fn test_a_march_gathers_the_turnout_for_the_day() {
    let mut g = Game::new();
    let rally = g.world.get_component::<Pos>(g.villagers[0]).map(|p| (p.x, p.y)).unwrap();
    let folk: Vec<Entity> = g.villagers.iter().copied().take(4).collect();
    g.call_protest(rally, folk.clone(), 1, g.tick + 1, g.tick);
    let p = g.protests.iter().find(|p| p.sign == 1).expect("the march is called");
    assert_eq!(p.rally, rally);
    assert_eq!(p.until, g.tick + 1, "a march is a day's business");
    let m = g.ui_model();
    for &u in &folk {
        let pv = m.people.iter().find(|p| p.e == u).unwrap();
        assert_eq!(pv.sign, Some(1), "printed placards for this one");
    }
}


