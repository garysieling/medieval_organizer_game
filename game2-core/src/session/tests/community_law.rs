use crate::session::*;
use crate::game::community_law::*;

fn grieve(g: &mut Game, v: Entity, p: desires::Problem, tick: u64) {
    desires::add_grievance(&mut g.world, v, p, tick + 60);
}

fn villager(g: &Game) -> Entity {
    g.villagers.iter().copied().find(|&v|
        g.world.get_component::<TownRole>(v).is_none()).unwrap()
}

fn aid(a: Entity, b: Entity, g: &mut Game) {
    relationships::record_interaction(&mut g.world, a,
        relationships::Interaction { other: b,
            kind: relationships::InteractionKind::Aid, intensity: 1.0 });
}

fn set_gold(g: &mut Game, v: Entity, gold: f32) {
    if let Some(p) = g.world.get_component_mut::<Purse>(v) { p.gold = gold; }
    else { g.world.add_component(v, Purse { gold }); }
}

#[test]
fn test_repeated_friction_becomes_a_bylaw() {
    let mut g = Game::new();
    let v = villager(&g);
    for p in [desires::Problem::ThinWood, desires::Problem::DryPit,
              desires::Problem::StinkingReach] {
        grieve(&mut g, v, p, 10);
    }
    g.bylaw_tick(10);
    assert!(g.bylaws.iter().any(|l| l.kind == ByeKind::Commons),
        "three incidents of a kind and the village agrees a rule");
    assert!(g.log.iter().any(|l| l.contains("agrees a rule")));
}

#[test]
fn test_a_broken_bylaw_draws_a_fine_and_the_word() {
    let mut g = Game::new();
    g.bylaws.push(ByeLaw { kind: ByeKind::Commons, passed_on: 0, violations: 0 });
    let v = villager(&g);
    set_gold(&mut g, v, 10.0);
    grieve(&mut g, v, desires::Problem::ThinWood, 10);
    g.bylaw_tick(10);
    let gold = g.world.get_component::<Purse>(v).unwrap().gold;
    assert!((gold - 9.0).abs() < 0.01, "a tenth of the purse, got {gold}");
    assert_eq!(g.bylaws[0].violations, 1);
    let blamed = g.world.get_component::<InnerLife>(v).unwrap().grievances.iter()
        .any(|(p, u)| *p == desires::Problem::Blamed && *u > 10);
    assert!(blamed, "the neighbors' word lands as blame, feeding the ladder");
    // the penalty's own blame isn't a fresh incident of the rule broken
    g.bylaw_tick(11);
    assert_eq!(g.bylaws[0].violations, 1, "blame shouldn't retrigger the rule");
    assert!(g.bylaw_friction.get(&ByeKind::Conduct).copied().unwrap_or(0) == 0,
        "nor count toward a conduct rule");
}

#[test]
fn test_kin_stand_surety_for_a_debtor() {
    let mut g = Game::new();
    g.bylaws.push(ByeLaw { kind: ByeKind::Commons, passed_on: 0, violations: 0 });
    let debtor = villager(&g);
    let voucher = g.villagers.iter().copied().find(|&v| v != debtor
        && g.world.get_component::<TownRole>(v).is_none()).unwrap();
    aid(debtor, voucher, &mut g);
    aid(voucher, debtor, &mut g);
    set_gold(&mut g, debtor, 0.01);
    set_gold(&mut g, voucher, 5.0);
    grieve(&mut g, debtor, desires::Problem::ThinWood, 10);
    g.bylaw_tick(10);
    let u = g.vouches.iter().find(|u| u.debtor == debtor && u.voucher == voucher)
        .expect("a bonded neighbor stands surety for the fine");
    assert!(u.bond > 0.0 && u.owed > 0.0);
}

#[test]
fn test_no_surety_where_the_pull_is_gone() {
    let mut g = Game::new();
    g.community_strength = 0.1;
    g.bylaws.push(ByeLaw { kind: ByeKind::Commons, passed_on: 0, violations: 0 });
    let debtor = villager(&g);
    let voucher = g.villagers.iter().copied().find(|&v| v != debtor
        && g.world.get_component::<TownRole>(v).is_none()).unwrap();
    aid(debtor, voucher, &mut g);
    aid(voucher, debtor, &mut g);
    set_gold(&mut g, debtor, 0.01);
    set_gold(&mut g, voucher, 5.0);
    grieve(&mut g, debtor, desires::Problem::ThinWood, 10);
    g.bylaw_tick(10);
    assert!(g.vouches.is_empty(), "no one stands surety for strangers");
}

#[test]
fn test_a_debt_repaid_brings_the_bond_home() {
    let mut g = Game::new();
    let debtor = villager(&g);
    let voucher = g.villagers.iter().copied().find(|&v| v != debtor
        && g.world.get_component::<TownRole>(v).is_none()).unwrap();
    set_gold(&mut g, debtor, 2.0);
    set_gold(&mut g, voucher, 5.0);
    g.vouches.push(Vouch { voucher, debtor, bond: 0.5, owed: 0.0005, made_on: 0 });
    g.pledge_tick(3);
    assert!(g.vouches.is_empty(), "made good, the surety's done");
    assert!((g.world.get_component::<Purse>(voucher).unwrap().gold - 5.5).abs() < 0.001,
        "the bond comes home");
}

#[test]
fn test_a_second_offense_spends_the_surety() {
    let mut g = Game::new();
    g.bylaws.push(ByeLaw { kind: ByeKind::Commons, passed_on: 0, violations: 0 });
    let debtor = villager(&g);
    let voucher = g.villagers.iter().copied().find(|&v| v != debtor
        && g.world.get_component::<TownRole>(v).is_none()).unwrap();
    set_gold(&mut g, debtor, 10.0);
    g.vouches.push(Vouch { voucher, debtor, bond: 0.01, owed: 1.0, made_on: 0 });
    grieve(&mut g, debtor, desires::Problem::ThinWood, 10);
    g.bylaw_tick(10);
    assert!(g.vouches.is_empty(), "the broken word ends the surety");
    assert!(g.sanctioned.iter().any(|(e, s, _)| *e == voucher && *s == Sanction::Gossip),
        "what the bond can't cover lands on the voucher's name");
}

#[test]
fn test_a_partnership_forms_primes_and_breaks() {
    let mut g = Game::new();
    let a = villager(&g);
    let b = g.villagers.iter().copied().find(|&v| v != a
        && g.world.get_component::<TownRole>(v).is_none()).unwrap();
    // a strong bond over a shared wrong
    aid(a, b, &mut g);
    aid(b, a, &mut g);
    grieve(&mut g, a, desires::Problem::Hungry, 2);
    grieve(&mut g, b, desires::Problem::Hungry, 2);
    g.partnership_tick(2);
    assert_eq!(g.partnerships.len(), 1, "sworn to shared work");
    assert_eq!(g.partnerships[0].stage, PartnerStage::Forming);
    // the bond holds - the compact primes
    g.partnership_tick(9);
    g.partnership_tick(16);
    assert_eq!(g.partnerships[0].stage, PartnerStage::Peak);
    // sour it and the shared work falls away
    for _ in 0..3 {
        relationships::record_interaction(&mut g.world, a,
            relationships::Interaction { other: b,
                kind: relationships::InteractionKind::Betrayal, intensity: 1.0 });
    }
    for t in [23u64, 30, 37] { g.partnership_tick(t); }
    let p = &g.partnerships[0];
    assert_eq!(p.stage, PartnerStage::Dissolved, "the compact breaks");
    assert!(p.rancor > 0.0 && !p.settled, "hard words remain for the parish");
}

#[test]
fn test_the_priest_settles_what_a_break_leaves() {
    let mut g = Game::new();
    let a = villager(&g);
    let b = g.villagers.iter().copied().find(|&v| v != a
        && g.world.get_component::<TownRole>(v).is_none()).unwrap();
    g.partnerships.push(Partnership { members: vec![a, b], formed_on: 0,
        strength: 0.0, stage: PartnerStage::Dissolved, rancor: 0.2, settled: false });
    g.partnership_tick(7);
    let p = &g.partnerships[0];
    assert!(p.settled, "the priest's word puts the dispute to rest");
    assert!(g.log.iter().any(|l| l.contains("makes peace")));
}

#[test]
fn test_the_communitys_pull_is_told_once() {
    let mut g = Game::new();
    g.community_strength = 0.8;
    g.community_tick(6);
    assert_eq!(g.community_band, 2);
    assert_eq!(g.log.iter().filter(|l| l.contains("one voice")).count(), 1);
    g.community_tick(13);
    assert_eq!(g.log.iter().filter(|l| l.contains("one voice")).count(), 1,
        "the band's told once each, not every week");
}
