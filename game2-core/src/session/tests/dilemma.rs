use crate::session::*;

#[test]
fn test_the_choice_comes_home() {
    let mut g = Game::new();
    let victim = g.villagers[0];
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(0, 0);
    assert!(g.dilemma.is_none(), "the knot never answered");
    assert_eq!(g.dilemma_debts.len(), 1, "the debt never wrote");
    assert_eq!(g.dilemma_debts[0].witness, g.player, "the answering pair of hands remembers");
    g.tick = g.dilemma_debts[0].due;
    g.dilemma_tick(g.tick);
    assert!(g.dilemma_debts.is_empty(), "the debt never landed");
    // and there's no virtue stat - the remembering's in the episodes
    assert!(g.log.iter().any(|l| l.contains("someone remembers")));
}

#[test]
fn test_the_remembering_lands_on_who_answered() {
    // the knot's global - one granary, one answer - but whose hands made
    // the pick is the answering seat's, not seat 0's (#962)
    let mut g = Game::new();
    let s1 = g.add_seat();
    let victim = g.villagers[0];
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    g.decide_at(s1, 0);
    assert_eq!(g.dilemma_debts.len(), 1);
    assert_eq!(g.dilemma_debts[0].witness, g.seats[s1].player,
        "seat 0's body would carry a remembering it never made");
    assert_ne!(g.dilemma_debts[0].witness, g.seats[0].player);
}

/// A decision box never lists a lone option (#1627): a pick list of
/// one isn't a choice - every `decides` panel keeps an out even when
/// its own slate runs empty
#[test]
fn test_no_decision_box_lists_a_lone_option() {
    // testimony with no history to tell still offers the plain telling
    // and the held tongue
    let mut g = Game::new();
    g.seats[0].testimony_offer = Some((g.tick, vec![]));
    let panel = g.convo_panel_at(0).expect("the testimony box never showed");
    assert!(panel.options.len() >= 2, "a pick list of one isn't a choice");
    g.decide_at(0, panel.options.len() - 1);
    assert!(g.seats[0].testimony_offer.is_none(), "the declined offer never closed");

    // an agenda night the size can't carry a motion on still offers
    // the bare floor or the adjournment
    let mut g = Game::new();
    g.seats[0].pending_agenda = Some((vec![], g.tick));
    let panel = g.convo_panel_at(0).expect("the agenda box never showed");
    assert!(panel.options.len() >= 2, "a pick list of one isn't a choice");
    g.decide_at(0, panel.options.len() - 1);
    assert!(g.seats[0].pending_agenda.is_none(), "the adjourned night never closed");
    assert!(g.log.iter().any(|l| l.contains("adjourned")),
        "the adjourn logged nothing - or a meeting seated anyway");
}

/// The knot's card shows while it's out (#1628): its rows name the
/// ways to be wrong, and the pick echoes what was carried out
#[test]
fn test_the_knot_shows_its_own_card_and_echoes_the_pick() {
    let mut g = Game::new();
    let victim = g.villagers[0];
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    let panel = g.convo_panel_at(0).expect("the knot's card never showed");
    assert_eq!(panel.title, "THE KNOT");
    assert!(panel.decides && panel.options.len() == 2, "{:?}", panel.options);
    assert!(panel.options.iter().any(|o| o == "the grain"));
    g.decide_at(0, 0);
    assert!(g.dilemma.is_none(), "the knot never answered");
    assert!(g.log.iter().any(|l| l.contains("the grain") && l.contains("carried out")),
        "no resolution echo: {:?}", g.log.first());
}

/// The traveller's ask can't be eaten by a card the seat can't see
/// (#1628): while a knot's out its own card shows instead, and once
/// it's answered the survey box comes back and takes the pick
#[test]
fn test_a_standing_knot_cant_eat_the_travellers_ask() {
    let mut g = Game::new();
    let victim = g.villagers[0];
    g.seats[0].survey = Some(crate::session::survey::SurveyOffer {
        beat: crate::session::survey::SurveyBeat::OrgFounded, asked: g.tick });
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ] });
    // the knot's card outranks the ask - the box that shows is the box
    // that answers
    assert_eq!(g.convo_panel_at(0).map(|p| p.title).as_deref(), Some("THE KNOT"));
    g.decide_at(0, 0);
    assert!(g.dilemma.is_none(), "the knot never answered");
    // the ask's box is back, and this pick lands on it
    let p = g.convo_panel_at(0).expect("the ask never came back");
    assert_eq!(p.title, "THE TRAVELLER ASKS");
    g.decide_at(0, 0);
    assert!(g.seats[0].survey.is_none(), "the ask never answered");
    assert!(g.log.iter().any(|l| l.contains("you tell the pedlar")),
        "no echo for the answer");
}
