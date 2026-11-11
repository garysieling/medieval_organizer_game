use crate::session::*;
use crate::game::desires::{InnerLife, Problem};
use crate::game::household;
use crate::game::lifecycle::{Age, LifeStage};
use crate::sim::illness::{LeicesterInstitution, SmallpoxVaccine};

fn make_infant(g: &mut Game, i: usize) {
    let v = g.villagers[i];
    let a = g.world.get_component_mut::<Age>(v).unwrap();
    a.stage = LifeStage::Infant;
    a.years = 0.0;
}

fn taxed(g: &Game, v: Entity) -> bool {
    g.world.get_component::<InnerLife>(v)
        .map(|l| l.grievances.iter().any(|(p, _)| *p == Problem::Taxed))
        .unwrap_or(false)
}

#[test]
fn test_the_officer_offer_waits_on_the_call() {
    let mut g = Game::new();
    g.vax_offer = Some(crate::session::events::VaxOffer { authority: g.player });
    let panel = g.convo_panel_at(0).expect("the offer never reached the box");
    assert_eq!(panel.title, "THE VACCINATION ACT");
    assert!(panel.decides && panel.options.len() == 3, "three ways to answer");
}

#[test]
fn test_comply_marks_the_infants_and_remembers_the_compulsion() {
    let mut g = Game::new();
    make_infant(&mut g, 0);
    g.vax_offer = Some(crate::session::events::VaxOffer { authority: g.player });
    g.decide_at(0, 0);
    assert!(g.vax_offer.is_none() && g.vax_done, "the offer's spent once answered");
    let baby = g.villagers[0];
    assert!(g.world.get_component::<SmallpoxVaccine>(baby).is_some(),
        "the infant never took the scratch");
    assert!(g.villagers.iter().skip(1).any(|&v| taxed(&g, v)),
        "a principled few should remember the compulsion");
}

#[test]
fn test_resist_fines_the_refusers_and_the_league_forms() {
    let mut g = Game::new();
    make_infant(&mut g, 0);
    let baby = g.villagers[0];
    let house = household::home_of(&g.world, baby).expect("the infant has no household");
    let members: Vec<Entity> = g.world.get_component::<household::Building>(house)
        .unwrap().occupants.clone();
    let before: Vec<f32> = members.iter()
        .map(|&m| g.world.get_component::<crate::game::livelihood::Purse>(m).map(|p| p.gold).unwrap_or(0.0))
        .collect();
    g.vax_offer = Some(crate::session::events::VaxOffer { authority: g.player });
    g.decide_at(0, 1);
    assert!(g.vax_done);
    assert!(g.world.get_component::<SmallpoxVaccine>(baby).is_none(),
        "the refusers' infant never took the scratch");
    let mut fined = false;
    for (i, &m) in members.iter().enumerate() {
        let after = g.world.get_component::<crate::game::livelihood::Purse>(m).map(|p| p.gold).unwrap_or(0.0);
        if after < before[i] { fined = true; }
        assert!(taxed(&g, m), "every refusing householder should remember the fine");
    }
    assert!(fined, "the fine never came out of a purse");
}

#[test]
fn test_the_leicester_way_is_backed_then_built_by_the_village() {
    let mut g = Game::new();
    g.vax_offer = Some(crate::session::events::VaxOffer { authority: g.player });
    g.decide_at(0, 2);
    assert!(g.leicester_backing.is_some(), "the backing never registered");
    assert!(g.world.query::<LeicesterInstitution>().is_empty(),
        "the player built it directly - the village builds it in response");
    g.leicester_tick(g.tick + 10);
    let e = g.world.query::<LeicesterInstitution>().first().copied()
        .expect("the counter-institution never rose");
    let l = g.world.get_component::<LeicesterInstitution>(e).unwrap();
    assert_eq!(l.phase, 0);
    assert_eq!(e, g.market_food, "the village's own institution, not the officer's");
}

#[test]
fn test_the_method_phases_in_over_the_season() {
    let mut g = Game::new();
    g.world.add_component(g.market_food, LeicesterInstitution { since: g.tick, phase: 0 });
    g.leicester_tick(g.tick + 45);
    let l = g.world.get_component::<LeicesterInstitution>(g.market_food).unwrap();
    assert_eq!(l.phase, 2, "a season and a half should see quarantine and disinfection");
    g.leicester_tick(g.tick + 70);
    let l = g.world.get_component::<LeicesterInstitution>(g.market_food).unwrap();
    assert_eq!(l.phase, 3, "the method holds once its work's done");
}

#[test]
fn test_the_act_answers_once_a_save() {
    let mut g = Game::new();
    g.rare.push(crate::session::events::RareEvent {
        id: events::EventId::VaccinationAct, e: Some(g.player), path: vec![],
        until: None, at: None, leaving: false, aux: 0,
        petters: crate::engine::det::HashMap::default() });
    g.event_arrived(events::EventId::VaccinationAct);
    assert!(g.vax_offer.is_some(), "the officer's offer never landed");
    g.decide_at(0, 2);
    g.event_arrived(events::EventId::VaccinationAct);
    assert!(g.vax_offer.is_none() && g.vax_done,
        "the Act came round a second time - once a save, the answer stands");
}
