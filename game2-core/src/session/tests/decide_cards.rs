use crate::session::*;

/// The decide arms' own box (#1663): every standing ask that used to
/// announce in the log and eat another panel's pick now renders its own
/// card where its resolver sits in decide_at - the box that shows is
/// the box that answers
fn card(g: &Game) -> Option<(String, Vec<String>)> {
    g.convo_panel_at(0).filter(|p| p.decides)
        .map(|p| (p.title.clone(), p.options.clone()))
}

#[test]
fn test_every_standing_ask_renders_and_answers_on_its_own_card() {
    // the postings: satisfy with the cause's coin, or let them stand
    // for the day - a waved-off box comes back tomorrow
    let mut g = Game::new();
    g.compliance_burden = 0.5;
    if let Some(o) = g.world.get_component_mut::<crate::sim::community::Organization>(g.org) {
        o.resources = 1.0;
    }
    let (title, options) = card(&g).expect("the postings never showed");
    assert_eq!(title, "THE POSTINGS");
    assert_eq!(options.len(), 2);
    g.decide_at(0, 0);
    assert!(g.compliance_burden < 0.5, "the satisfy never landed");
    let mut g = Game::new();
    g.compliance_burden = 0.5;
    g.decide_at(0, 1); // let them stand
    assert!(g.compliance_burden == 0.5, "a let-stand still spent");
    assert!(card(&g).map(|(t, _)| t != "THE POSTINGS").unwrap_or(true),
        "the stood-down postings kept the box");
    g.tick += 1;
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE POSTINGS"),
        "tomorrow's postings never came back");

    // the stall: convene, or let the silence deepen
    let mut g = Game::new();
    g.council = Some(g.tick);
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE STALL"));
    g.decide_at(0, 1);
    assert!(g.council.is_none(), "the stall never answered");

    // the mystic: canonize, license, or suppress
    let mut g = Game::new();
    g.mystic = Some(g.tick);
    let (t, o) = card(&g).expect("the mystic never showed");
    assert_eq!(t, "THE MYSTIC");
    assert_eq!(o.len(), 3);
    g.decide_at(0, 0);
    assert!(g.mystic.is_none(), "the canonize never landed");

    // the charter: the gold or the edge
    let mut g = Game::new();
    g.incorporation_offer = Some(g.tick);
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE CHARTER"));
    g.decide_at(0, 1);
    assert!(g.incorporation_offer.is_none(), "the refusal never landed");

    // the eyes' hold: free an exposed one, or let the hold stand
    let mut g = Game::new();
    g.inquisition = Some(3);
    g.penetration = 0.5;
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE EYES' HOLD"));
    g.decide_at(0, 0);
    assert!(g.penetration < 0.5, "the release never landed");
    let mut g = Game::new();
    g.inquisition = Some(3);
    g.penetration = 0.5;
    g.decide_at(0, 1); // leave them to the eyes
    assert_eq!(g.penetration, 0.5, "a let-stand still freed someone");
    assert!(card(&g).map(|(t, _)| t != "THE EYES' HOLD").unwrap_or(true));
    g.tick += 1;
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE EYES' HOLD"));

    // the parley: ride, hold the distance, or demand hostages
    let mut g = Game::new();
    g.parley = Some(g.tick);
    let (t, o) = card(&g).expect("the parley never showed");
    assert_eq!(t, "THE PARLEY");
    assert_eq!(o.len(), 3);
    g.decide_at(0, 1); // hold the distance - resolves, keeps the leader
    assert!(g.parley.is_none(), "the held distance never resolved");

    // the unsworn oath: refuse the honor, or keep the custom today
    let mut g = Game::new();
    g.separation_pressure = 0.6;
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE UNSWORN OATH"));
    g.decide_at(0, 0); // the refusal's the first row now
    assert!(g.log.iter().any(|l| l.contains("honor goes unsworn")),
        "the refusal never landed");
    let mut g = Game::new();
    g.separation_pressure = 0.6;
    g.decide_at(0, 1); // the custom kept - the box waves off for the day
    assert!(card(&g).map(|(t, _)| t != "THE UNSWORN OATH").unwrap_or(true),
        "the kept custom kept the box");
    g.tick += 1;
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE UNSWORN OATH"));
}

/// The bug itself (#1663): a standing parley used to swallow a Decide
/// meant for whatever box the player saw - a pick on the founding card
/// silently riding to the parley. Now the parley's own card shows
/// first, so what shows is what answers - and the call keeps
#[test]
fn test_the_box_that_shows_is_the_box_that_answers() {
    let mut g = Game::new();
    g.parley = Some(g.tick);
    g.seats[0].pending_founding = Some(PendingFounding {
        record: 0, vote: None, org_name: "the cause".into(),
        meet_again: vec![g.villagers[0]], street: false, turnout: false, offered_tick: g.tick });
    let (t, _) = card(&g).expect("no card showed at all");
    assert_eq!(t, "THE PARLEY", "the parley stood but a lower box showed");
    g.decide_at(0, 1); // the pick lands on the shown card: hold the distance
    assert!(g.parley.is_none(), "the shown parley never answered");
    // and the founding's untouched - it answers on its own card now
    assert!(g.seats[0].pending_founding.is_some(),
        "the parley's pick ate the founding's box");
    let (t, _) = card(&g).expect("the call never showed after the parley");
    assert!(t.contains("YOUR CALL"), "the waiting call's not the card: {}", t);
}

/// A knot still outranks the standing asks (#1663): while its card's
/// out, none of them show and none of them answer - the pick resolves
/// the knot
#[test]
fn test_a_knot_outranks_the_standing_asks() {
    let mut g = Game::new();
    let victim = g.villagers[0];
    g.parley = Some(g.tick);
    g.separation_pressure = 0.6;
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the grain or the children?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch { text: "the grain".into(), debt: 0.6, victim },
            crate::session::events::DilemmaBranch { text: "the children".into(), debt: 0.3, victim },
        ],
            offered: g.tick
         });
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE KNOT"),
        "the knot's card didn't outrank the standing asks");
    g.decide_at(0, 0);
    assert!(g.dilemma.is_none(), "the knot never answered");
    assert!(g.parley.is_some(), "the knot's pick ate the parley");
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE PARLEY"),
        "the parley never showed once the knot closed");
}

/// The fold yields to the louder boxes (#1663): its own card's gated
/// off while the emergency, the violent word or the lord's offer
/// stands - and now its resolver's gated the same, so a shown card's
/// pick can't land on a hidden fold
#[test]
fn test_the_fold_cant_eat_a_louder_boxs_pick() {
    let mut g = Game::new();
    if let Some(c) = g.world.get_component_mut::<crate::sim::religion::Congregation>(g.congregation) {
        c.cultish = true;
    }
    g.emergency = Some((g.tick, g.tick + 20));
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE LORD'S EMERGENCY"),
        "the fold's card showed over the emergency's");
    g.decide_at(0, 0); // gathers a witness - must not shelter a soul out
    assert_eq!(g.witnesses, 1, "the emergency's own pick never landed");
    assert!(!g.log.iter().any(|l| l.contains("walks free of the fold")),
        "the hidden fold ate the emergency's pick");
    // and when the louder boxes are gone, the fold's card's back
    g.emergency = None;
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE TIGHTENED FOLD"),
        "the fold never showed once the louder box closed");
}

/// The arc's early contest rides the eyes' card (#1663): while the
/// watching's young it heads the play rows, and picking it contests -
/// not feeds
#[test]
fn test_the_arcs_contest_rides_the_eyes_card() {
    let mut g = Game::new();
    g.counter_intel = 2;
    g.suppression_arc = 1;
    let (t, o) = card(&g).expect("the eyes never showed");
    assert_eq!(t, "THE ORG'S OWN EYES");
    assert_eq!(o.len(), 3, "the contest never headed the plays: {:?}", o);
    assert!(o[0].contains("Contest the word"), "the first row's not the contest: {}", o[0]);
    g.decide_at(0, 0);
    assert!(g.log.iter().any(|l| l.contains("contested early")),
        "the contest never landed");
    assert!(!g.log.iter().any(|l| l.contains("false word")),
        "the contest's pick fed the watcher instead");
}

/// A standing Decide can't squat the box (#1688): a meeting under way
/// renders ahead of every standing card, and an offer the moment's
/// passed answers itself unanswered - the class, not one card
#[test]
fn test_a_live_floor_outranks_a_standing_card_and_the_card_lapses() {
    // the knot the road brings stands on the box
    let mut g = Game::new();
    g.dilemma = Some(crate::session::events::MoralKnot {
        text: "the granary's caught - which do you carry out first?".into(),
        branches: vec![
            crate::session::events::DilemmaBranch {
                text: "the grain - the village starves else".into(),
                debt: 0.6, victim: g.villagers[0] },
            crate::session::events::DilemmaBranch {
                text: "the children - let the grain burn".into(),
                debt: 0.3, victim: g.villagers[1] },
        ],
        offered: g.tick,
    });
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE KNOT"));

    // a meeting convening underneath takes the panel - the stale
    // dilemma can't cover the room the world's actually in
    g.meeting = Some(Meeting {
        label: "Bram's house".into(), venue: (10, 10), born: g.tick,
        invited: vec![g.villagers[0], g.villagers[1]],
        spots: vec![(10, 10); 2], paths: vec![None; 2],
        frames: 0, talk_from: None,
        lines: vec![(g.villagers[0], "Friends - you came.".into())],
        talk_id: 0, seat: 0,
        roles: (false, false, false, false, false),
        role_people: [None; 5],
        bought: vec![], probe: false, testimony: None,
        agenda: None, steer_at: None, steer_wait: 0, steered: false,
        promised: vec![], marks: vec![], impromptu: false, pledged: vec![],
    });
    let p = g.convo_panel_at(0).expect("the floor never rendered");
    assert!(p.title.starts_with("HOUSE MEETING"),
        "a standing card covered the live floor: {}", p.title);
    // a pick while the floor shows can't be eaten by the hidden card
    g.decide_at(0, 0);
    assert!(g.dilemma.is_some(), "a card not showing took the floor's pick");
    g.meeting = None;

    // the card comes back when the floor's done - the offer still stands
    assert_eq!(card(&g).map(|(t, _)| t).as_deref(), Some("THE KNOT"),
        "the knot never came back once the floor cleared");

    // but the moment passes: the week out, an unanswered offer lapses
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS - 1);
    assert!(g.dilemma.is_some(), "the knot lapsed before its week was out");
    g.livelihood_tick(g.tick + FOUNDING_OFFER_EXPIRY_DAYS);
    assert!(g.dilemma.is_none(), "the knot squatted the box past its week");
}
