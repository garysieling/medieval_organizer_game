//! Rare events: each family's handlers, the scheduler and the staged harness.
use crate::session::*;
use crate::session::events::*;
use crate::session::stage;
use crate::session::PKind;

/// The whole registry's its hundred and counting, every id once
#[test]
fn test_the_registry_is_a_hundred() {
    assert_eq!(EVENTS.len(), 102);
    for i in 0..EVENTS.len() {
        for j in (i + 1)..EVENTS.len() {
            assert_ne!(EVENTS[i].id, EVENTS[j].id, "duplicate id in the registry");
        }
    }
    for s in EVENTS { assert_eq!(spec(s.id).id, s.id); }
}

/// Every event has somewhere in the schedule
#[test]
fn test_every_event_is_scheduled() {
    let g = Game::new();
    for s in EVENTS {
        assert!(g.next_rare.iter().any(|(id, _)| *id == s.id), "{:?} never comes", s.id);
    }
}

/// The tinker walks in, stands his pitch, mends what needs mending,
/// and moves on - state the backend and the frame both show
#[test]
fn test_the_tinker_stands_and_mends() {
    let mut g = Game::new();
    g.force_event(EventId::Tinker);
    assert!(g.rare.iter().any(|r| r.id == EventId::Tinker), "he's afoot");
    assert!(g.log.iter().any(|l| l.contains("tinker")), "the log names him");
    // walk him in - the pitch takes once he's stood at market
    for _ in 0..2000 {
        if g.rare.iter().any(|r| r.id == EventId::Tinker && r.until.is_some()) { break; }
        g.advance_rare();
    }
    let r = g.rare.iter().find(|r| r.id == EventId::Tinker).unwrap();
    assert!(r.until.is_some(), "he's stood his pitch");
    // the frame shows him: a traveller labelled by his trade
    let model = g.ui_model();
    assert!(model.people.iter().any(|p| matches!(p.kind, PKind::Traveller)
        && p.label.as_deref() == Some("the tinker")), "the frame shows the tinker");
    // his days mend tools and take pennies
    let mended = g.villagers.iter().copied().any(|v|
        desires::is_satisfied(&g.world, v, Want::BetterTools, g.tick));
    stage::run_days(&mut g, 2);
    let mended = mended || g.villagers.iter().copied().any(|v|
        desires::is_satisfied(&g.world, v, Want::BetterTools, g.tick));
    assert!(mended, "someone's tools got sharper");
    // and the road takes him back when his days are done
    stage::run_days(&mut g, 6);
    assert!(g.rare.iter().all(|r| r.id != EventId::Tinker), "the road took him");
    assert!(g.next_rare.iter().any(|(id, d)| *id == EventId::Tinker && *d > g.tick),
        "he's due again one day");
}

/// The hailstorm batters on its day and lifts on the next
#[test]
fn test_the_hailstorm_batters_and_lifts() {
    let mut g = Game::new();
    // a garden worth battering
    if g.gardens.is_empty() {
        let home = g.building_entities[0];
        g.gardens.push((home, (15, 20), 0.8, false));
    }
    let before: Vec<f32> = g.gardens.iter().map(|g| g.2).collect();
    g.force_event(EventId::Hailstorm);
    assert!(g.rare.iter().any(|r| r.id == EventId::Hailstorm));
    stage::run_days(&mut g, 1);
    // the battering fell: growth shredded somewhere (or already low)
    let after: Vec<f32> = g.gardens.iter().map(|g| g.2).collect();
    let battered = before.iter().zip(after.iter()).any(|(b, a)| a < b);
    assert!(battered || before.iter().all(|b| *b < 0.1), "the hail bit");
    stage::run_days(&mut g, 2);
    assert!(g.rare.iter().all(|r| r.id != EventId::Hailstorm), "the storm lifted");
}

/// The knife-grinder puts an edge on the trades' steel - the frame
/// shows him at his wheel, and the road takes him when he's done
#[test]
fn test_the_knifegrinder_puts_an_edge_on() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        g.force_event(EventId::KnifeGrinder);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::KnifeGrinder && r.until.is_some()) { break; }
            g.advance_rare();
        }
        assert!(g.rare.iter().any(|r| r.id == EventId::KnifeGrinder), "he's afoot on {:?}", st);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the knife-grinder")),
            "the frame shows the grinder on {:?}", st);
        // his days sharpen the trades' tools
        stage::run_days(&mut g, 2);
        let sharpened = g.villagers.iter().copied().any(|v|
            g.world.get_component::<crate::game::vocation::Craft>(v).is_some()
                && desires::is_satisfied(&g.world, v, Want::BetterTools, g.tick));
        assert!(sharpened, "a working blade got an edge on {:?}", st);
        // and the road takes him when his days are done
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::KnifeGrinder), "the road took him on {:?}", st);
    }
}

/// The rag man pays the poorest in reach, and leaves the rich street
/// empty-handed - staged poor and flush alike, the frame shows him too
#[test]
fn test_the_ragman_pays_the_poorest() {
    for st in [stage::Stage::Unrest, stage::Stage::Flush] {
        let mut g = stage::staged(st);
        // a rag-seller worth his coin: poor and standing by the market
        let seller = g.villagers[0];
        g.world.get_component_mut::<Purse>(seller).unwrap().gold = 1.0;
        g.force_event(EventId::RagAndBone);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::RagAndBone && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::RagAndBone).and_then(|r| r.at).unwrap();
        // stand her near his pitch
        g.world.get_component_mut::<Pos>(seller).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(seller).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(seller, t); }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the rag-and-bone man")),
            "the frame shows the rag man on {:?}", st);
        stage::run_days(&mut g, 1);
        let gold = g.world.get_component::<Purse>(seller).unwrap().gold;
        // the staged flush was overridden to a penny for her - the
        // point stands either way: her rags earned
        assert!(gold > 1.0, "her rags earned on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::RagAndBone), "he moved on from {:?}", st);
    }
}

/// The sweep's clean flues hold the chimney fire off - the schedule
/// itself shows it, and the street's easier for his visit
#[test]
fn test_the_sweep_holds_the_fire_off() {
    for st in [stage::Stage::Ordinary, stage::Stage::HardWinter] {
        let mut g = stage::staged(st);
        let before = g.next_rare.iter().find(|(i, _)| *i == EventId::ChimneyFire).map(|(_, d)| *d).unwrap();
        g.force_event(EventId::ChimneySweep);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::ChimneySweep && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the sweep")),
            "the frame shows the sweep on {:?}", st);
        // his days are done: the flues are clean, the fire's far off
        stage::run_days(&mut g, 6);
        let after = g.next_rare.iter().find(|(i, _)| *i == EventId::ChimneyFire).map(|(_, d)| *d).unwrap();
        assert!(after > before || after > g.tick + 100,
            "the chimney fire's pushed off ({before} -> {after}, tick {}) on {:?}", g.tick, st);
        assert!(g.log.iter().any(|l| l.contains("flue")), "the log says why");
        assert!(g.rare.iter().all(|r| r.id != EventId::ChimneySweep), "he moved on from {:?}", st);
    }
}

/// The rat-catcher's terrier earns: the store loses nothing while he
/// works, and the plague's pushed off when he's gone
#[test]
fn test_the_ratcatcher_clears_the_larder() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let plague_before = g.next_rare.iter().find(|(i, _)| *i == EventId::RatPlague).map(|(_, d)| *d).unwrap();
        g.force_event(EventId::RatCatcher);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::RatCatcher && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the rat-catcher")),
            "the frame shows the catcher on {:?}", st);
        stage::run_days(&mut g, 2);
        // his sack fills: the day's tally rides in aux (the servings
        // net out against a hungry village, but his work shows here)
        assert!(g.rare.iter().find(|r| r.id == EventId::RatCatcher).map(|r| r.aux).unwrap_or(0) > 0,
            "his terrier earned on {:?}", st);
        stage::run_days(&mut g, 4);
        let after = g.next_rare.iter().find(|(i, _)| *i == EventId::RatPlague).map(|(_, d)| *d).unwrap();
        assert!(after > g.tick + 100 || after > plague_before,
            "the plague's pushed off ({plague_before} -> {after}) on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::RatCatcher), "he moved on from {:?}", st);
    }
}

/// The diviner's stick speaks at the end: water marked, or dry ground
/// all round - and the frame showed him walking it first
#[test]
fn test_the_dowsers_stick_speaks() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        if g.gardens.is_empty() {
            let home = g.building_entities[0];
            g.gardens.push((home, (15, 20), 0.5, false));
        }
        let growth0: f32 = g.gardens.iter().map(|g| g.2).sum();
        g.force_event(EventId::Dowser);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Dowser && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the water-diviner")),
            "the frame shows the diviner on {:?}", st);
        stage::run_days(&mut g, 7);
        let found = g.log.iter().any(|l| l.contains("water sings"));
        let dry = g.log.iter().any(|l| l.contains("never dipped"));
        assert!(found || dry, "the stick said something on {:?}", st);
        // a spring feeds the gardens; the pit's stone yield is untouched
        if found {
            let growth1: f32 = g.gardens.iter().map(|g| g.2).sum();
            assert!(growth1 > growth0, "the gardens drank on {:?}", st);
        }
        assert!(g.land.pit_water <= 0.6, "the pit's not drowned on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::Dowser), "he moved on from {:?}", st);
    }
}

/// The scribe's fair hand earns: letters fly for the illiterate to
/// the ones they think of - the frame shows him at his desk first
#[test]
fn test_the_scribes_fair_hand() {
    for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Scribe);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Scribe && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Scribe).and_then(|r| r.at).unwrap();
        // stand a letter-sender by his desk with a bond to write to -
        // a living one: the bereft's dead write no letters
        let living: Vec<_> = g.villagers.iter().copied()
            .filter(|&v| !villager::is_dead(&g.world, v)).collect();
        let (a, b) = (living[0], living[1]);
        g.world.get_component_mut::<Pos>(a).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(a).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(a, t); }
        let r = g.world.create_entity();
        g.world.add_component(r, crate::game::relationships::Relationship {
            from: a, other: b, kind: crate::game::relationships::RelKind::Friend,
            strength: 0.8, interactions: 5 });
        g.world.get_component_mut::<Purse>(a).unwrap().gold = 10.0;
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the scribe")),
            "the frame shows the scribe on {:?}", st);
        stage::run_days(&mut g, 2);
        assert!(g.letters.iter().any(|&(f, t, _)| f == a && t == b),
            "a letter went out for her on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Scribe), "he moved on from {:?}", st);
    }
}

/// The mushroom flush readies every patch in the wood, adds a few
/// bonus caps of its own for the stay, and is picked through by the
/// end - the schedule gives it back one day
#[test]
fn test_the_mushroom_flush_fills_the_wood() {
    let mut g = Game::new();
    for p in g.mush_patches.iter_mut() { p.next_ready = g.tick + 100; }
    let n_before = g.mush_patches.len();
    g.force_event(EventId::MushroomFlush);
    assert!(g.mush_patches.iter().all(|p| p.next_ready <= g.tick),
        "the flush readies every patch");
    assert!(g.mush_patches.len() > n_before, "bonus caps with the flush");
    assert!(g.log.iter().any(|l| l.contains("full of caps")), "the log says so");
    stage::run_days(&mut g, 12);
    assert!(g.rare.iter().all(|r| r.id != EventId::MushroomFlush), "the flush passes");
    assert!(g.mush_patches.iter().skip(n_before).all(|p| p.next_ready > g.tick + 300),
        "the bonus caps are picked through");
}

/// The worst jaw in reach gets the pull, a penny the tooth - and the
/// frame showed the drawer at his stool first
#[test]
fn test_the_toothdrawer_pulls_the_worst_jaw() {
    for st in [stage::Stage::Ordinary, stage::Stage::Fever] {
        let mut g = stage::staged(st);
        g.force_event(EventId::ToothDrawer);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::ToothDrawer && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::ToothDrawer).and_then(|r| r.at).unwrap();
        // a bad jaw by his stool: a living one with a coin for the pull
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Health>(v).unwrap().condition = 0.4;
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the tooth-drawer")),
            "the frame shows the drawer on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<Health>(v).unwrap().condition > 0.4,
            "the pull eased her on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::ToothDrawer), "he moved on from {:?}", st);
    }
}

/// A bloodletting for a penny: the knife touches whoever's worst off
/// in reach - eased or nicked, the day's work shows
#[test]
fn test_the_barbers_knife_touches() {
    for st in [stage::Stage::Ordinary, stage::Stage::Fever] {
        let mut g = stage::staged(st);
        g.force_event(EventId::BarberSurgeon);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::BarberSurgeon && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::BarberSurgeon).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Health>(v).unwrap().condition = 0.5;
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the barber-surgeon")),
            "the frame shows the barber on {:?}", st);
        stage::run_days(&mut g, 1);
        assert_ne!(g.world.get_component::<Health>(v).unwrap().condition, 0.5,
            "his knife touched her on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::BarberSurgeon), "he moved on from {:?}", st);
    }
}

/// The pilgrim bowl feeds the faith: a penny in, a blessing out -
/// and the frame shows the band passing first
#[test]
fn test_the_pilgrim_bowl_feeds_the_faith() {
    for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
        let mut g = stage::staged(st);
        g.force_event(EventId::PilgrimBand);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::PilgrimBand && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::PilgrimBand).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        // she's one who feels the lack of it
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a band of pilgrims")),
            "the frame shows the band on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick),
            "the bowl fed her faith on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny went in");
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::PilgrimBand), "they passed from {:?}", st);
    }
}

/// The knuckle-bone's blessing holds: two gold for the faithful, and
/// the frame shows his stall first
#[test]
fn test_the_relics_blessing_holds() {
    for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
        let mut g = stage::staged(st);
        g.force_event(EventId::RelicSeller);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::RelicSeller && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::RelicSeller).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the relic-seller")),
            "the frame shows the seller on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick),
            "the blessing took on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold <= 8.0, "two gold went for it");
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::RelicSeller), "he moved on from {:?}", st);
    }
}

/// A grievance off the heart for two gold: the burdened pay, the
/// paper takes the weight - and the frame shows him first
#[test]
fn test_the_pardons_paper_takes_the_weight() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Pardoner);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Pardoner && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Pardoner).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 30);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the pardoner")),
            "the frame shows the pardoner on {:?}", st);
        stage::run_days(&mut g, 1);
        // the weight he was paid for is gone (the day may have laid a
        // fresh one of its own - an empty flour bin isn't his to pardon)
        assert!(!g.world.get_component::<desires::InnerLife>(v).unwrap().grievances.iter()
            .any(|(p, _)| *p == desires::Problem::Taxed),
            "the pardon lifted her on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Pardoner), "he moved on from {:?}", st);
    }
}

/// Three knives and a grin: the joy spreads free, the hat takes a
/// penny from those who can spare it - the frame shows him first
#[test]
fn test_the_jugglers_knives_lift_the_street() {
    for st in [stage::Stage::Ordinary, stage::Stage::Fever] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Juggler);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Juggler && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Juggler).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Joy];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a juggler")),
            "the frame shows the juggler on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
            "the knives lifted her on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::Juggler), "he moved on from {:?}", st);
    }
}

/// The bear dances and the joy spreads - the snap's a maybe, but the
/// crowd's lift is certain, and the frame shows the pair
#[test]
fn test_the_bear_dances_and_the_hat_goes_round() {
    for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
        let mut g = stage::staged(st);
        g.force_event(EventId::BearWard);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::BearWard && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::BearWard).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Joy];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a bear-ward")),
            "the frame shows the pair on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
            "the bear lifted her on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::BearWard), "they moved on from {:?}", st);
    }
}

/// Noah's flood enthralled them: the kids are lifted for free, and
/// the frame shows the booth first
#[test]
fn test_the_puppet_show_enthralls_the_kids() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Puppeteer);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Puppeteer && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Puppeteer).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Joy];
        // she's a kid for the front row: no hat money changes hands
        g.world.add_component(v, crate::game::vocation::Kid);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a puppeteer")),
            "the frame shows the booth on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
            "the show enthralled her on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold == 10.0,
            "the front row was free on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Puppeteer), "he moved on from {:?}", st);
    }
}

/// A penny a broadside: the tune sticks in whoever has the ear for
/// it - and the frame shows the sheets first
#[test]
fn test_the_ballad_sticks_in_the_ear() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        g.force_event(EventId::BalladMonger);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::BalladMonger && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::BalladMonger).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Music];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the ballad-monger")),
            "the frame shows the sheets on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Music, g.tick),
            "the tune stuck on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny went for the sheet");
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::BalladMonger), "he moved on from {:?}", st);
    }
}

/// The news gives them something to talk about - bonds pick up a
/// day's talk in earshot of his bell
#[test]
fn test_the_news_sets_the_street_talking() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        g.force_event(EventId::NewsCrier);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::NewsCrier && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::NewsCrier).and_then(|r| r.at).unwrap();
        // give a listener a bond to take the talk home to
        let a = g.villagers.iter().copied().find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let b = g.villagers.iter().copied().find(|&v| !villager::is_dead(&g.world, v) && v != a).unwrap();
        g.world.get_component_mut::<Pos>(a).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(a).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(a, t); }
        let bond = g.world.create_entity();
        g.world.add_component(bond, crate::game::relationships::Relationship {
            from: a, other: b, kind: crate::game::relationships::RelKind::Friend,
            strength: 0.5, interactions: 0 });
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the news-crier")),
            "the frame shows the crier on {:?}", st);
        stage::run_days(&mut g, 1);
        let talks = g.world.get_component::<crate::game::relationships::Relationship>(bond)
            .map(|r| r.interactions).unwrap_or(0);
        assert!(talks > 0, "the street talked on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::NewsCrier), "he moved on from {:?}", st);
    }
}

/// The levy marches off with a young one if the drum caught one -
/// or the village keeps its young, and the log says which
#[test]
fn test_the_drum_takes_or_the_village_keeps() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Recruiter);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Recruiter && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Recruiter).and_then(|r| r.at).unwrap();
        // a young one near his drum
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.add_component(v, crate::game::vocation::Kid);
        let before = g.villagers.len();
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the recruiting sergeant")),
            "the frame shows the drum on {:?}", st);
        stage::run_days(&mut g, 6);
        let took = g.log.iter().any(|l| l.contains("the levy took"));
        let kept = g.log.iter().any(|l| l.contains("kept its young"));
        assert!(took || kept, "the drum said something on {:?}", st);
        assert_eq!(took, g.villagers.len() < before, "the roll matched the roster on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::Recruiter), "he marched off from {:?}", st);
    }
}

/// Hide him and the writ follows: whoever knows his face bought the
/// debt - and the frame showed him in the green first
#[test]
fn test_the_shelter_buy_the_debt() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        let debt0 = g.repression_debt;
        g.force_event(EventId::Deserter);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Deserter && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Deserter).and_then(|r| r.at).unwrap();
        // someone knows his face
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a deserter")),
            "the frame shows him on {:?}", st);
        stage::run_days(&mut g, 1);
        stage::run_days(&mut g, 5);
        assert!(g.repression_debt > debt0, "the writ's debt landed on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("writ for harboring")), "the riders came on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::Deserter), "he melted off from {:?}", st);
    }
}

/// Fed and rested, they stay or the road takes them - either way
/// the store thinned for them and the log says how it ended
#[test]
fn test_the_family_roots_or_the_road_takes_them() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let before = g.villagers.len();
        let food0 = g.probe().food_servings;
        g.force_event(EventId::RefugeeFamily);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::RefugeeFamily && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a refugee family")),
            "the frame shows them on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.probe().food_servings < food0, "they ate from the store on {:?}", st);
        stage::run_days(&mut g, 6);
        let stayed = g.log.iter().any(|l| l.contains("family stayed"));
        let moved = g.log.iter().any(|l| l.contains("moved on at dawn"));
        assert!(stayed || moved, "their fate was told on {:?}", st);
        assert_eq!(stayed, g.villagers.len() == before + 2, "the roster matches on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::RefugeeFamily), "their camp's gone from {:?}", st);
    }
}

/// Crusts go out at arm's length: the store thins, the carriers'
/// faith is fed - and the frame showed the band first
#[test]
fn test_the_crusts_and_the_faithful() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let food0 = g.probe().food_servings;
        g.force_event(EventId::LeperBand);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::LeperBand && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::LeperBand).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a leper band")),
            "the frame shows the band on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.probe().food_servings < food0, "the crusts went out on {:?}", st);
        assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick),
            "the charity counted on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::LeperBand), "the bells faded from {:?}", st);
    }
}

/// Confession under the hawthorn: a live weight comes off and the
/// ear learns a want - the frame shows him first
#[test]
fn test_the_hawthorn_lightens_a_heart() {
    for st in [stage::Stage::Ordinary, stage::Stage::Unrest] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Friar);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Friar && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Friar).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Love];
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 30);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a wandering friar")),
            "the frame shows him on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(!g.world.get_component::<desires::InnerLife>(v).unwrap().grievances.iter()
            .any(|(p, _)| *p == desires::Problem::Taxed),
            "the weight came off on {:?}", st);
        assert!(g.world.get_component::<desires::Known>(v)
            .map(|k| k.revealed.contains(&desires::Reveal::Want(Want::Love))).unwrap_or(false),
            "the ear learned the want on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Friar), "he wandered on from {:?}", st);
    }
}

/// The strange doctrine doesn't leave faith where it found it:
/// somebody's comfort moved, one way or the other
#[test]
fn test_the_doctrine_moves_the_streets_faith() {
    for st in [stage::Stage::Ordinary, stage::Stage::Bereft] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Heretic);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Heretic && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Heretic).and_then(|r| r.at).unwrap();
        // two ears for his doctrine, one on each side of the line:
        // an even-indexed soul gets shaken, an odd one gets drawn
        let even = g.villagers.iter().enumerate()
            .find(|(i, v)| i % 2 == 0 && !villager::is_dead(&g.world, **v)).map(|(_, v)| *v).unwrap();
        let odd = g.villagers.iter().enumerate()
            .find(|(i, v)| i % 2 == 1 && !villager::is_dead(&g.world, **v)).map(|(_, v)| *v).unwrap();
        for v in [even, odd] {
            g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
            g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
            // come to see the visitor: that's today's place
            { let t = g.tick; g.plan_visit(v, t); }
            g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Faith];
        }
        desires::satisfy(&mut g.world, even, Want::Faith, g.tick + 60);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a quiet preacher")),
            "the frame shows him on {:?}", st);
        stage::run_days(&mut g, 1);
        let faith_of = |g: &Game, v| g.world.get_component::<desires::InnerLife>(v)
            .and_then(|l| l.satisfied.iter().find(|(w, _)| *w == Want::Faith).map(|(_, u)| *u));
        assert!(faith_of(&g, even).is_none(), "the shaken lost comfort on {:?}", st);
        assert!(faith_of(&g, odd).is_some(), "the drawn were fed on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Heretic), "he preached elsewhere from {:?}", st);
    }
}

/// A head a day off the drove: the fold takes a sheep, the coin
/// changes hands - and the frame shows the drove first
#[test]
fn test_the_drove_fills_the_fold() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        // a fold for the sheep if the stage lacks one
        let has_fold = g.world.query::<household::Building>().iter()
            .any(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
        if !has_fold {
            let f = g.world.create_entity();
            g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
            g.world.add_component(f, household::Building {
                kind: household::BuildingKind::Pasture, name: "the fold".into(),
                occupants: vec![], worker: None, overseer: None,
                stock: Default::default(), capacity: 0 });
        }
        let sheep0 = g.world.query::<crate::game::livestock::Livestock>().len();
        g.force_event(EventId::Drover);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Drover && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Drover).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a drover")),
            "the frame shows the drove on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.query::<crate::game::livestock::Livestock>().len() > sheep0,
            "the fold grew on {:?}", st);
        // and she's on the flock's roll - the frame and the grazing know her
        assert!(g.flock.len() > 0, "she's on the roll on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Drover), "the drove moved on from {:?}", st);
    }
}

/// Salt fish keeps all winter: a penny a fish for those who'd know
/// good food - and the frame shows the cart first
#[test]
fn test_the_salt_fish_sells_to_taste() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::FishHawker);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::FishHawker && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::FishHawker).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::GoodFood];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a fish-hawker")),
            "the frame shows the cart on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::GoodFood, g.tick),
            "the fish eased her on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny went for the fish");
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::FishHawker), "the cart moved on from {:?}", st);
    }
}

/// The quarter's bought for fair weight: the store thins and the
/// market till fattens - the frame shows the factor first
#[test]
fn test_the_quarter_moves_for_fair_weight() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::CornFactor);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::CornFactor && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let food0 = g.probe().food_servings;
        let seller = g.world.get_component::<household::Building>(g.market_food)
            .and_then(|b| b.worker.or(b.overseer))
            .or_else(|| g.villagers.iter().copied()
                .find(|&v| !villager::is_dead(&g.world, v)));
        let gold0 = seller.and_then(|s| g.world.get_component::<Purse>(s)).map(|p| p.gold).unwrap_or(0.0);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the corn-factor")),
            "the frame shows the factor on {:?}", st);
        stage::run_days(&mut g, 1);
        let food1 = g.probe().food_servings;
        let gold1 = seller.and_then(|s| g.world.get_component::<Purse>(s)).map(|p| p.gold).unwrap_or(0.0);
        assert!(food0 - food1 <= 3, "only surplus moved on {:?}", st);
        assert!(food1 < food0 || gold1 > gold0 || g.probe().food_servings <= 4,
            "the quarter moved for coin on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::CornFactor), "he weighed out from {:?}", st);
    }
}

/// Pepper's dear: two gold a packet eases the taste and folk hold
/// a pinch cures what ails - the frame shows the spice-cart first
#[test]
fn test_the_pepper_eases_and_cures() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::SpiceMerchant);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::SpiceMerchant && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::SpiceMerchant).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::GoodFood];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the spice merchant")),
            "the frame shows the spice-cart on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::GoodFood, g.tick),
            "the packet eased her taste on {:?}", st);
        assert_eq!(g.world.get_component::<Purse>(v).unwrap().gold, 8.0, "her two gold went for the pepper");
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::SpiceMerchant), "the cart moved on from {:?}", st);
    }
}

/// White gold stretches the winter: a gold a handful and the salt
/// makes the larder last - the frame shows the salt-cart first
#[test]
fn test_the_salt_stretches_the_store() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::SaltSeller);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::SaltSeller && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::SaltSeller).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        let food0 = g.probe().food_servings;
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the salt-seller")),
            "the frame shows the salt-cart on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.probe().food_servings > food0, "the salt stretched the store on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her gold went for the salt");
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::SaltSeller), "the cart moved on from {:?}", st);
    }
}

/// The clip off the fold pays its shepherd a gold the head - the
/// frame shows the wool-cart first
#[test]
fn test_the_clip_pays_the_shepherd() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        // a fold with a shepherd and a sheep, if the stage lacks one
        let shepherd = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        if g.world.query::<household::Building>().iter()
            .all(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind != household::BuildingKind::Pasture).unwrap_or(false)) {
            let f = g.world.create_entity();
            g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
            g.world.add_component(f, household::Building {
                kind: household::BuildingKind::Pasture, name: "the fold".into(),
                occupants: vec![], worker: Some(shepherd), overseer: None,
                stock: Default::default(), capacity: 0 });
        } else {
            g.world.get_component_mut::<household::Building>(
                g.world.query::<household::Building>().iter().copied()
                    .find(|&b| g.world.get_component::<household::Building>(b)
                        .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap())
                .unwrap().worker = Some(shepherd);
        }
        let fold = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap();
        if g.flock.is_empty() {
            let e = crate::game::livestock::spawn_animal(&mut g.world,
                crate::game::livestock::LivestockKind::Sheep, fold, 21.0, 21.0);
            g.flock.push(e);
        }
        let gold0 = g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0);
        g.force_event(EventId::WoolMerchant);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::WoolMerchant && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the wool merchant")),
            "the frame shows the wool-cart on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0) > gold0,
            "the clip paid him on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::WoolMerchant), "the cart moved on from {:?}", st);
    }
}

/// Words that outlive the winter: the lettered pay dear for a
/// book's long joy - the frame shows the book-cart first
#[test]
fn test_the_book_outlives_the_winter() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Bookseller);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Bookseller && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Bookseller).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        // lettered like the book-buyers are: give her the priest's craft
        if !g.literate(v) {
            g.world.add_component(v, crate::game::vocation::craft(
                crate::game::vocation::Vocation::Priest, 0.5));
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the bookseller")),
            "the frame shows the book-cart on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick),
            "the book's joy holds on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her gold went for the book");
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Bookseller), "the cart moved on from {:?}", st);
    }
}

/// His heap burns warm at the woods edge, and when he breaks it a
/// stock of coal is left - the frame shows his smoke first
#[test]
fn test_the_heap_leaves_coal() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::CharcoalBurner);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::CharcoalBurner && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::CharcoalBurner).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        let cond0 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a charcoal burner")),
            "the frame shows his smoke on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0) > cond0,
            "the heap warmed her on {:?}", st);
        stage::run_days(&mut g, 8);
        let coal: u32 = g.world.query::<household::Building>().iter()
            .filter_map(|&b| g.world.get_component::<household::Building>(b)
                .and_then(|bb| bb.stock.get("coal").copied()))
            .sum();
        assert!(coal > 0, "he left a heap on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::CharcoalBurner), "he packed out from {:?}", st);
    }
}

/// Lime for the fields: while he stands the beds grow sweeter -
/// the frame shows the lime-cart first
#[test]
fn test_the_lime_sweetens_the_beds() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::LimeBurner);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::LimeBurner && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let growth0: f32 = g.gardens.iter().map(|g| g.2).sum();
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the lime-burner")),
            "the frame shows the lime-cart on {:?}", st);
        stage::run_days(&mut g, 1);
        let growth1: f32 = g.gardens.iter().map(|g| g.2).sum();
        assert!(growth1 > growth0 || g.gardens.is_empty() && growth1 == growth0,
            "the beds grew on {:?}", st);
        stage::run_days(&mut g, 7);
        assert!(g.rare.iter().all(|r| r.id != EventId::LimeBurner), "his kiln cooled from {:?}", st);
    }
}

/// Mind the smell: the tan-pit's reek hangs on whoever stands near
/// his vats - the frame shows the tanner's trade first
#[test]
fn test_the_reek_hangs_on_the_near() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Tanner);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Tanner && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Tanner).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the tanner")),
            "the frame shows the vats on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<desires::InnerLife>(v)
            .map(|l| l.grievances.iter().any(|(p, u)| *p == desires::Problem::Smelly && g.tick <= *u))
            .unwrap_or(false), "the reek hung on her on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Tanner), "the vats packed out from {:?}", st);
    }
}

/// Glass for the church: one home's panes keep the cold out and
/// let the light in - the frame shows the glazier first
#[test]
fn test_the_panes_keep_the_cold_out() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Glazier);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Glazier && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Glazier).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)
                && household::home_of(&g.world, v).is_some()).unwrap();
        let h = household::home_of(&g.world, v).unwrap();
        g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(h, t); }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the glazier")),
            "the frame shows him on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick), "the light came in on {:?}", st);
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::Glazier), "his ladder came down on {:?}", st);
    }
}

/// Roofs patched before the rain finds you: fresh thatch on the
/// stock roll and the leak's due pushed off - the frame shows
/// his ladder first
#[test]
fn test_the_thatch_holds_the_rain_off() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Thatcher);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Thatcher && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Thatcher).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)
                && household::home_of(&g.world, v).is_some()).unwrap();
        let h = household::home_of(&g.world, v).unwrap();
        g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(h, t); }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the thatcher")),
            "the frame shows his ladder on {:?}", st);
        stage::run_days(&mut g, 1);
        let thatch = g.world.get_component::<household::Building>(h)
            .and_then(|b| b.stock.get("thatch").copied()).unwrap_or(0);
        assert!(thatch > 0, "fresh thatch on the roll on {:?}", st);
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::Thatcher), "his ladder came down on {:?}", st);
    }
}

/// Sweet water, not the pond's: when his shaft comes in the sour
/// gut clears and the beds drink - the frame shows his spade first
#[test]
fn test_the_shaft_strikes_sweet_water() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::WellDigger);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::WellDigger && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        desires::add_grievance(&mut g.world, v, desires::Problem::SourGut, g.tick + 30);
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the well-digger")),
            "the frame shows his spade on {:?}", st);
        stage::run_days(&mut g, 7);
        assert!(g.world.get_component::<desires::InnerLife>(v)
            .map(|l| l.grievances.iter().all(|(p, _)| *p != desires::Problem::SourGut))
            .unwrap_or(false), "the sweet water cleared her gut on {:?}", st);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 > growth0 || g.gardens.is_empty(), "the beds drank on {:?}", st);
        assert!(g.rare.iter().all(|r| r.id != EventId::WellDigger), "he packed his spade from {:?}", st);
    }
}

/// Your palm, love: a penny a palm and the lines surface a want -
/// the frame shows her scarf first
#[test]
fn test_the_lines_dont_lie() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::FortuneTeller);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::FortuneTeller && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::FortuneTeller).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        g.world.get_component_mut::<desires::InnerLife>(v).unwrap().wants = vec![Want::Love];
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the fortune-teller")),
            "the frame shows her scarf on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<desires::Known>(v)
            .map(|k| k.revealed.contains(&desires::Reveal::Want(Want::Love))).unwrap_or(false),
            "the lines surfaced the want on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny bought the palm");
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::FortuneTeller), "her tent came down on {:?}", st);
    }
}

/// He reads the sky over the village once: whatever's written
/// nearest gets named, and near folk feel forearmed - the frame
/// shows his charts first
#[test]
fn test_the_sky_reads_the_village() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Astrologer);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Astrologer && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Astrologer).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the astrologer")),
            "the frame shows his charts on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.log.iter().any(|l| l.contains("astrologer reads the sky")),
            "the reading was told on {:?}", st);
        assert!(desires::is_satisfied(&g.world, v, Want::Security, g.tick),
            "forearmed on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Astrologer), "his charts rolled up on {:?}", st);
    }
}

/// Strange things only: he takes whatever odd's in the satchel for
/// two gold - the frame shows his flasks first
#[test]
fn test_the_strange_thing_sells() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Alchemist);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Alchemist && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Alchemist).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<crate::game::livelihood::Satchel>(v)
            .unwrap().goods.insert("witch's hazel".into(), 1);
        let gold0 = g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the alchemist")),
            "the frame shows his flasks on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<crate::game::livelihood::Satchel>(v)
            .map(|s| s.goods.is_empty()).unwrap_or(false),
            "the strange thing's gone on {:?}", st);
        assert!(g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0) > gold0,
            "he paid for it on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Alchemist), "his flasks packed up on {:?}", st);
    }
}

/// Cures all: a penny a dose and the bottle does what it does -
/// the frame shows his cart first
#[test]
fn test_the_bottle_does_what_it_does() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Quack);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Quack && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Quack).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.world.get_component_mut::<Pos>(v).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(v).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(v, t); }
        g.world.get_component_mut::<Purse>(v).unwrap().gold = 10.0;
        let cond0 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the quack")),
            "the frame shows his cart on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<Purse>(v).unwrap().gold < 10.0, "her penny bought the dose");
        let cond1 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
        assert!(cond1 != cond0, "the bottle did something on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Quack), "his cart rolled on from {:?}", st);
    }
}

/// Babies caught, mothers kept: a home with a small one gets her
/// tending - the frame shows her basket first
#[test]
fn test_the_blessing_on_the_house() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Midwife);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Midwife && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Midwife).and_then(|r| r.at).unwrap();
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)
                && household::home_of(&g.world, v).is_some()).unwrap();
        let h = household::home_of(&g.world, v).unwrap();
        g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(h, t); }
        // a small one in the house for her to tend
        let k = villager::spawn_villager(&mut g.world, at.0 as f32, at.1 as f32);
        g.world.add_component(k, Health { condition: 0.5, immunity: 0.5 });
        g.world.add_component(k, crate::game::vocation::Kid);
        g.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(k);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the midwife")),
            "the frame shows her basket on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<Health>(k).map(|x| x.condition).unwrap_or(0.0) > 0.5,
            "her hands helped the small one on {:?}", st);
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::Midwife), "she moved on from {:?}", st);
    }
}

/// Beasts tended: the doctored fold holds the wolves off, and the
/// shepherd sleeps the easier - the frame shows his crook first
#[test]
fn test_the_doctored_fold_holds_the_wolves_off() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let shepherd = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let fold = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
        let fold = match fold {
            Some(f) => { g.world.get_component_mut::<household::Building>(f).unwrap().worker = Some(shepherd); f }
            None => {
                let f = g.world.create_entity();
                g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
                g.world.add_component(f, household::Building {
                    kind: household::BuildingKind::Pasture, name: "the fold".into(),
                    occupants: vec![], worker: Some(shepherd), overseer: None,
                    stock: Default::default(), capacity: 0 });
                f
            }
        };
        let _ = fold;
        g.force_event(EventId::Sowgelder);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Sowgelder && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let due0 = g.next_rare.iter().find(|(i, _)| *i == EventId::WolfAtFold).map(|(_, d)| *d).unwrap_or(0);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the pig man")),
            "the frame shows his crook on {:?}", st);
        stage::run_days(&mut g, 1);
        let due1 = g.next_rare.iter().find(|(i, _)| *i == EventId::WolfAtFold).map(|(_, d)| *d).unwrap_or(0);
        assert!(due1 > due0, "the wolves' day pushed off on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Sowgelder), "he moved his crook on from {:?}", st);
    }
}

/// Shoes for hooves: the shepherd pays for sound stock, and a shod
/// beast doesn't founder - the frame shows his anvil first
#[test]
fn test_the_ring_of_iron() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let shepherd = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let fold = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false));
        match fold {
            Some(f) => { g.world.get_component_mut::<household::Building>(f).unwrap().worker = Some(shepherd); }
            None => {
                let f = g.world.create_entity();
                g.world.add_component(f, Pos { x: 20.0, y: 20.0 });
                g.world.add_component(f, household::Building {
                    kind: household::BuildingKind::Pasture, name: "the fold".into(),
                    occupants: vec![], worker: Some(shepherd), overseer: None,
                    stock: Default::default(), capacity: 0 });
            }
        }
        g.world.get_component_mut::<Purse>(shepherd).unwrap().gold = 10.0;
        // stock on the fold for him to shoe
        let fold_e = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap();
        if g.flock.is_empty() {
            let e = crate::game::livestock::spawn_animal(&mut g.world,
                crate::game::livestock::LivestockKind::Sheep, fold_e, 21.0, 21.0);
            g.flock.push(e);
        }
        g.force_event(EventId::Farrier);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Farrier && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the farrier")),
            "the frame shows his anvil on {:?}", st);
        stage::run_days(&mut g, 1);
        assert_eq!(g.world.get_component::<Purse>(shepherd).unwrap().gold, 8.0, "his two gold went for the shoes");
        assert!(desires::is_satisfied(&g.world, shepherd, Want::Security, g.tick),
            "sound stock from here out on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Farrier), "his anvil packed up on {:?}", st);
    }
}

/// Sound wind and mostly honest - he buys a head off the fold a
/// day at a dealer's price, and the shepherd takes the coin
#[test]
fn test_the_dealers_price() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let shepherd = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let fold = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Pasture).unwrap_or(false)).unwrap();
        g.world.get_component_mut::<household::Building>(fold).unwrap().worker = Some(shepherd);
        if g.flock.is_empty() {
            let e = crate::game::livestock::spawn_animal(&mut g.world,
                crate::game::livestock::LivestockKind::Sheep, fold, 21.0, 21.0);
            g.flock.push(e);
        }
        let flock0 = g.flock.len();
        let gold0 = g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0);
        g.force_event(EventId::HorseCop);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::HorseCop && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the horse-coper")),
            "the frame shows his string on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.flock.len() < flock0, "a head left the fold on {:?}", st);
        assert!(g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0) > gold0,
            "the dealer's price paid on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::HorseCop), "his string moved on from {:?}", st);
    }
}

/// Tallow taken, light for the dark: a home's tallow turns to
/// candles, a coin for the wax - the frame shows his dipper first
#[test]
fn test_the_tallow_turns_to_light() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::CandleMaker);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::CandleMaker && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::CandleMaker).and_then(|r| r.at).unwrap();
        let h = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).unwrap();
        g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(h, t); }
        g.world.get_component_mut::<household::Building>(h).unwrap().stock.insert("tallow".into(), 1);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the candle-maker")),
            "the frame shows his dipper on {:?}", st);
        stage::run_days(&mut g, 1);
        let b = g.world.get_component::<household::Building>(h).unwrap();
        assert_eq!(*b.stock.get("tallow").unwrap_or(&0), 0, "the tallow's in his dipper");
        assert!(*b.stock.get("candle").unwrap_or(&0) > 0, "light for the dark on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::CandleMaker), "his dipper packed up on {:?}", st);
    }
}

/// A jug that doesn't weep: a penny a crock and the house eats the
/// better for it - the frame shows his wheel first
#[test]
fn test_the_crock_holds_the_stew() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Potter);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Potter && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Potter).and_then(|r| r.at).unwrap();
        let h = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).unwrap();
        g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(h, t); }
        // someone home to buy a crock
        let k = villager::spawn_villager(&mut g.world, at.0 as f32, at.1 as f32);
        g.world.add_component(k, Health { condition: 0.5, immunity: 0.5 });
        g.world.add_component(k, Purse { gold: 5.0 });
        g.world.get_component_mut::<household::Building>(h).unwrap().occupants.push(k);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the potter")),
            "the frame shows his wheel on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(*g.world.get_component::<household::Building>(h).unwrap()
            .stock.get("crock").unwrap_or(&0) > 0, "a jug that doesn't weep on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::Potter), "his wheel moved on from {:?}", st);
    }
}

/// What have your hens done - he buys the dairy's put-by off the
/// shelf, a coin a basket - the frame shows his panniers first
#[test]
fn test_the_hens_day_pays() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Eggler);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Eggler && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let at = g.rare.iter().find(|r| r.id == EventId::Eggler).and_then(|r| r.at).unwrap();
        let h = g.world.query::<household::Building>().iter().copied()
            .find(|&b| g.world.get_component::<household::Building>(b)
                .map(|bb| bb.kind == household::BuildingKind::Home).unwrap_or(false)).unwrap();
        // his panniers fill in order - make her shelf the only
        // dairy in reach so it's hers he buys
        for &o in &g.world.query::<household::Building>().iter().copied().collect::<Vec<_>>() {
            if let Some(b) = g.world.get_component_mut::<household::Building>(o) {
                b.stock.remove("eggs"); b.stock.remove("butter"); b.stock.remove("cheese");
            }
        }
        g.world.get_component_mut::<Pos>(h).unwrap().x = at.0 as f32;
        g.world.get_component_mut::<Pos>(h).unwrap().y = at.1 as f32;
        // come to see the visitor: that's today's place
        { let t = g.tick; g.plan_visit(h, t); }
        g.world.get_component_mut::<household::Building>(h).unwrap().stock.insert("eggs".into(), 2);
        let k = villager::spawn_villager(&mut g.world, at.0 as f32, at.1 as f32);
        g.world.add_component(k, Health { condition: 0.5, immunity: 0.5 });
        g.world.add_component(k, Purse { gold: 0.0 });
        g.world.get_component_mut::<household::Building>(h).unwrap().occupants.insert(0, k);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the eggler")),
            "the frame shows his panniers on {:?}", st);
        // the hen may lay again the same day - the coin's the surer
        // sign, and it's the hand he pays that has to show it: a
        // housemate's day of buying and selling says nothing about
        // whether the eggler's penny came in
        let gold0 = g.world.get_component::<Purse>(k).unwrap().gold;
        stage::run_days(&mut g, 1);
        let b = g.world.get_component::<household::Building>(h).unwrap();
        assert!(*b.stock.get("eggs").unwrap_or(&0) <= 2, "his basket's no deeper on {:?}", st);
        let gold1 = g.world.get_component::<Purse>(k).unwrap().gold;
        assert!(gold1 > gold0, "the coin came back on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::Eggler), "his panniers moved on from {:?}", st);
    }
}

/// Aged wheels bought: two gold a wheel off the shelf, the older
/// the better - the frame shows his rounds first
#[test]
fn test_the_wheel_fetches_two() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::CheeseMonger);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::CheeseMonger && r.until.is_some()) { break; }
            g.advance_rare();
        }
        // a well-aged wheel on the rack for him to covet
        let cellar = g.world.create_entity();
        g.cellar = Some(cellar);
        g.cheeses = vec![g.tick.saturating_sub(100)];
        let shepherd = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.shepherd = Some(shepherd);
        let gold0 = g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0);
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("the cheese-monger")),
            "the frame shows his rounds on {:?}", st);
        stage::run_days(&mut g, 1);
        assert!(g.cheeses.is_empty(), "the oldest wheel went with him on {:?}", st);
        assert!(g.world.get_component::<Purse>(shepherd).map(|p| p.gold).unwrap_or(0.0) > gold0,
            "a monger's price paid on {:?}", st);
        stage::run_days(&mut g, 4);
        assert!(g.rare.iter().all(|r| r.id != EventId::CheeseMonger), "his rounds rolled on from {:?}", st);
    }
}

/// A letter, is there one here called-? news rides in for one of
/// them - the frame shows his horse first
#[test]
fn test_the_letter_rides_in() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::Messenger);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::Messenger && r.until.is_some()) { break; }
            g.advance_rare();
        }
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some("a messenger")),
            "the frame shows his horse on {:?}", st);
        stage::run_days(&mut g, 1);
        let moved = desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.world.get_component::<desires::InnerLife>(v)
                .map(|l| l.grievances.iter().any(|(p, _)| *p == desires::Problem::Grief))
                .unwrap_or(false)
            || g.villagers.iter().any(|&o| o != v
                && g.world.get_component::<desires::InnerLife>(o)
                    .map(|l| l.grievances.iter().any(|(p, _)| *p == desires::Problem::Grief)
                        || l.satisfied.iter().any(|(w, _)| *w == Want::Joy))
                    .unwrap_or(false));
        assert!(moved, "the letter found its man on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::Messenger), "he rode on from {:?}", st);
    }
}

/// A heat wave: the folk droop, the beds ripen, the well's head
/// drops - and the spell lifts when its days are out
#[test]
fn test_the_stones_are_hot() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let cond0 = g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0);
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        let water0 = g.land.pit_water;
        g.force_event(EventId::HeatWave);
        assert!(g.rare.iter().any(|r| r.id == EventId::HeatWave), "the heat settled in");
        stage::run_days(&mut g, 1);
        assert!(g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0) < cond0
            || g.villagers.iter().all(|&w| g.world.get_component::<Health>(w)
                .map(|h| h.condition <= 0.05 + 0.005).unwrap_or(true)),
            "the folk droop on {:?}", st);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 > growth0 || g.gardens.is_empty(), "the beds ripen on {:?}", st);
        assert!(g.land.pit_water < water0, "the well's head dropped on {:?}", st);
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::HeatWave), "the heat lifted on {:?}", st);
    }
}

/// A drought: the well runs low and the beds wilt while the sky
/// forgets rain - and the dry lifts when its fortnight's out
#[test]
fn test_the_sky_forgets_rain() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let water0 = g.land.pit_water;
        g.force_event(EventId::Drought);
        assert!(g.rare.iter().any(|r| r.id == EventId::Drought), "the dry settled in");
        stage::run_days(&mut g, 1);
        assert!(g.land.pit_water < water0, "the well ran low on {:?}", st);
        stage::run_days(&mut g, 15);
        assert!(g.rare.iter().all(|r| r.id != EventId::Drought), "the rain remembered on {:?}", st);
    }
}

/// The river comes up: the pit fills toward drowning and the beds
/// drink deep - and the water falls when its days are out
#[test]
fn test_the_reach_is_full() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let water0 = g.land.pit_water;
        g.force_event(EventId::Flood);
        assert!(g.rare.iter().any(|r| r.id == EventId::Flood), "the reach rose");
        stage::run_days(&mut g, 1);
        assert!(g.land.pit_water > water0 || water0 >= 1.0, "the water came up on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Flood), "the water fell on {:?}", st);
    }
}

/// Lightning strikes: the bolt picks one of them and knocks them
/// flat where it lands
#[test]
fn test_the_bolt_picks_one() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let before: Vec<f32> = g.villagers.iter()
            .map(|&v| g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0)).collect();
        g.force_event(EventId::LightningStrike);
        let after: Vec<f32> = g.villagers.iter()
            .map(|&v| g.world.get_component::<Health>(v).map(|h| h.condition).unwrap_or(0.0)).collect();
        let hit = before.iter().zip(after.iter()).any(|(b, a)| a < b);
        assert!(hit || g.villagers.is_empty(), "the bolt picked one on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("lightning")), "the strike was told on {:?}", st);
    }
}

/// A late frost: the beds are nipped in a night and the patches die
/// back a month
#[test]
fn test_white_on_the_beds() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        for x in g.gardens.iter_mut() { x.2 = x.2.max(1.0); }
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        g.force_event(EventId::LateFrost);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 < growth0 || g.gardens.is_empty(), "the beds were nipped on {:?}", st);
        if !g.mush_patches.is_empty() {
            assert!(g.mush_patches.iter().all(|p| p.next_ready >= g.tick + 29),
                "the patches died back on {:?}", st);
        }
        assert!(g.log.iter().any(|l| l.contains("late frost")), "the frost was told on {:?}", st);
    }
}

/// An Indian summer: the beds get one more ripening and the warm
/// stretch sits easy on the folk - and the year turns after
#[test]
fn test_the_year_gives_back() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        g.force_event(EventId::IndianSummer);
        assert!(g.rare.iter().any(|r| r.id == EventId::IndianSummer), "the warm stretch settled in");
        stage::run_days(&mut g, 1);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 > growth0 || g.gardens.is_empty(), "the beds ripened on {:?}", st);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the warm stretch sat easy on {:?}", st);
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::IndianSummer), "the year turned on {:?}", st);
    }
}

/// The first snow: white on the roofs, the kids out in it early,
/// and the melt feeds the well a little
#[test]
fn test_the_kids_out_early() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let kid = g.villagers.iter().copied()
            .find(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_some()
                && !villager::is_dead(&g.world, v));
        g.force_event(EventId::FirstSnow);
        if let Some(k) = kid {
            assert!(desires::is_satisfied(&g.world, k, Want::Joy, g.tick),
                "the wonder of it on {:?}", st);
        }
        assert!(g.log.iter().any(|l| l.contains("first snow")), "the turning was told on {:?}", st);
    }
}

/// The thaw: the roofs drip, the ground's soft for the working,
/// and the melt refills the well short of drowning it
#[test]
fn test_the_ground_works_again() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.land.pit_water = 0.1;
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::Thaw);
        assert!(g.rare.iter().any(|r| r.id == EventId::Thaw), "the roofs started dripping");
        stage::run_days(&mut g, 1);
        assert!(g.land.pit_water > 0.1 && g.land.pit_water <= 0.6,
            "the melt refilled without drowning on {:?}", st);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the folk felt spring on {:?}", st);
        stage::run_days(&mut g, 5);
        assert!(g.rare.iter().all(|r| r.id != EventId::Thaw), "the melt was done on {:?}", st);
    }
}

/// The aurora: the sky burns and the praying comes easy while it
/// lasts - two nights, then the sky's the sky again
#[test]
fn test_the_sky_burns() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::Aurora);
        assert!(g.rare.iter().any(|r| r.id == EventId::Aurora), "the sky caught fire");
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Faith, g.tick)),
            "the sign was taken on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::Aurora), "the sky's the sky again on {:?}", st);
    }
}

/// A meteor shower: the sky rains stars and nobody's eyes are
/// anywhere else - the wonder of it sits easy after
#[test]
fn test_the_sky_rains_stars() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::MeteorShower);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the wonder of it on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("meteor")), "the shower was told on {:?}", st);
    }
}

/// A blood moon: the prayers go up and the sleep's the worse for
/// the dread - two nights, then the moon's the moon again
#[test]
fn test_the_moon_hangs_red() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::BloodMoon);
        assert!(g.rare.iter().any(|r| r.id == EventId::BloodMoon), "the moon turned red");
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Faith, g.tick)),
            "the prayers went up on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::BloodMoon), "the moon's the moon again on {:?}", st);
    }
}

/// A dark noon: the sun goes out at midday, the prayers go up and
/// the beds get no sun - one day, then the light's back
#[test]
fn test_the_sun_goes_out() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::DarkNoon);
        assert!(g.rare.iter().any(|r| r.id == EventId::DarkNoon), "the day went dark");
        stage::run_days(&mut g, 1);
        assert!(desires::is_satisfied(&g.world, v, Want::Faith, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Faith, g.tick)),
            "the prayers went up on {:?}", st);
        stage::run_days(&mut g, 2);
        assert!(g.rare.iter().all(|r| r.id != EventId::DarkNoon), "the light came back on {:?}", st);
    }
}

/// A windstorm: the beds flatten and the thatch tears - two days
/// of it, then the wind lies down
#[test]
fn test_the_thatch_tears() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        for x in g.gardens.iter_mut() { x.2 = x.2.max(1.0); }
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        g.force_event(EventId::WindStorm);
        assert!(g.rare.iter().any(|r| r.id == EventId::WindStorm), "the wind came up");
        stage::run_days(&mut g, 1);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 < growth0 || g.gardens.is_empty(), "the beds flattened on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::WindStorm), "the wind lay down on {:?}", st);
    }
}

/// A mirage: a city of towers hangs on the horizon and melts when
/// you look twice - a marvel to talk of
#[test]
fn test_the_city_that_isnt() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::Mirage);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the marvel of it on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("mirage")), "the towers were told on {:?}", st);
    }
}

/// A wolf at the fold: the flock scatters and some are lost - the
/// same way the night-run does it
#[test]
fn test_the_wolfs_in_the_fold() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let flock0 = g.flock.len();
        g.force_event(EventId::WolfAtFold);
        assert!(g.log.iter().any(|l| l.contains("wolf")), "the wolf was told on {:?}", st);
        // the wolf's take is a roll of 0..2, so a night he gets
        // nothing is a night he gets nothing; what the test holds
        // the fold to is that the tally and the pen agree
        let lost: usize = g.log.iter().rev()
            .find(|l| l.contains("wolf got into the fold"))
            .and_then(|l| l.split("- ").nth(1))
            .and_then(|t| t.split_whitespace().next())
            .and_then(|n| n.parse().ok())
            .unwrap_or_else(|| panic!("the night's tally was told on {:?}", st));
        assert_eq!(g.flock.len() + lost, flock0,
            "the fold is short exactly what the wolf took on {:?}", st);
        assert!(lost <= flock0, "he can't take more than was penned on {:?}", st);
    }
}

/// A fox in the yard: a hen's gone and the house grieves her -
/// the same way the fox night does it
#[test]
fn test_the_fox_takes_her() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let hens0 = g.hens.len();
        g.force_event(EventId::FoxInYard);
        assert!(g.hens.len() <= hens0, "the yard's lighter on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("fox")), "the fox was told on {:?}", st);
    }
}

/// A hawk stoops: a shadow falls on the green and a hen's gone
/// with it - or the yard's empty and it wheels off
#[test]
fn test_the_shadow_falls() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let hens0 = g.hens.len();
        g.force_event(EventId::HawkStoops);
        assert!(g.hens.len() <= hens0, "the yard's lighter on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("hawk")), "the stoop was told on {:?}", st);
    }
}

/// The hawk's grief lands on a hearth that can feel it - the held
/// and the dead see nothing fall
#[test]
fn test_the_shadow_skips_the_held() {
    let mut g = stage::staged(stage::Stage::Ordinary);
    let Some(&h) = g.hen_houses.first() else { panic!("no hen house") };
    let Some(&occ) = g.world.get_component::<household::Building>(h)
        .and_then(|b| b.occupants.first()) else { panic!("empty hearth") };
    g.world.add_component(occ, crate::game::dungeon::CastleHeld { since: 0 });
    let bubbles0 = g.bubbles.len();
    g.force_event(EventId::HawkStoops);
    assert!(g.bubbles.iter().skip(bubbles0).all(|b| b.0 != occ),
        "the held see nothing fall");
    assert!(g.world.get_component::<desires::InnerLife>(occ)
        .map(|l| l.grievances.iter().all(|(p, _)| *p != desires::Problem::Loss))
        .unwrap_or(true), "no grief lands on the held");
}

/// The stray cat hangs about the market, then makes her choice -
/// the loneliest hearth takes her in
#[test]
fn test_the_cat_makes_her_choice() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::StrayCat);
        for _ in 0..2000 {
            if g.rare.iter().any(|r| r.id == EventId::StrayCat && r.until.is_some()) { break; }
            g.advance_rare();
        }
        assert!(g.rare.iter().any(|r| r.id == EventId::StrayCat), "she's about");
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| matches!(p.kind, PKind::Pet(Species::Cat))),
            "the frame shows the cat on {:?}", st);
        stage::run_days(&mut g, 8);
        assert!(g.rare.iter().all(|r| r.id != EventId::StrayCat), "she chose on {:?}", st);
        assert!(g.pets.iter().any(|&(_, sp, _)| sp == Species::Cat)
            || g.log.iter().any(|l| l.contains("slips off")), "she stayed or slipped on {:?}", st);
    }
}

/// A plague of rats: every larder pays them where no cat walks,
/// the tally grows, and the swarm's gone when its ten days are out
#[test]
fn test_the_rats_own_ten_days() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::RatPlague);
        assert!(g.rare.iter().any(|r| r.id == EventId::RatPlague), "the swarm's in");
        stage::run_days(&mut g, 3);
        let toll = g.rare.iter().find(|r| r.id == EventId::RatPlague).map(|r| r.aux).unwrap_or(0);
        assert!(toll > 0 || g.building_entities.iter().all(|&be| g.cat_watches(be)),
            "the larders paid on {:?}", st);
        stage::run_days(&mut g, 12);
        assert!(g.rare.iter().all(|r| r.id != EventId::RatPlague), "the swarm's gone on {:?}", st);
    }
}

/// Locusts: the cloud comes down and strips the beds near bare,
/// and the patches won't fruit for two months
#[test]
fn test_nothing_green_where_they_pass() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        for x in g.gardens.iter_mut() { x.2 = x.2.max(1.0); }
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        g.force_event(EventId::Locusts);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 < growth0 * 0.5 || g.gardens.is_empty(),
            "the beds were stripped on {:?}", st);
        if !g.mush_patches.is_empty() {
            assert!(g.mush_patches.iter().all(|p| p.next_ready >= g.tick + 59),
                "the patches were eaten back on {:?}", st);
        }
        assert!(g.log.iter().any(|l| l.contains("locust")), "the cloud was told on {:?}", st);
    }
}

/// A caterpillar year: the leaves are lacework while the creep
/// holds, the kids have the sport of it, and it passes
#[test]
fn test_the_leaves_are_lacework() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        for x in g.gardens.iter_mut() { x.2 = x.2.max(1.0); }
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        let kid = g.villagers.iter().copied()
            .find(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_some()
                && !villager::is_dead(&g.world, v));
        g.force_event(EventId::CaterpillarYear);
        assert!(g.rare.iter().any(|r| r.id == EventId::CaterpillarYear), "the creep's in");
        stage::run_days(&mut g, 2);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 < growth0 || g.gardens.is_empty(), "the beds were eaten on {:?}", st);
        if let Some(k) = kid {
            assert!(desires::is_satisfied(&g.world, k, Want::Joy, g.tick),
                "the kids had the sport on {:?}", st);
        }
        stage::run_days(&mut g, 11);
        assert!(g.rare.iter().all(|r| r.id != EventId::CaterpillarYear), "the creep passed on {:?}", st);
    }
}

/// A murmuration: ten thousand wings writing on the sky, and the
/// wonder of it sits easy after
#[test]
fn test_the_sky_writes_itself() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::Starlings);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the wonder of it on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("murmuration")), "the wings were told on {:?}", st);
    }
}

/// A bee tree found: the combs come back dripping to a larder,
/// stings and all
#[test]
fn test_the_combs_come_back() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let honey0: u32 = g.building_entities.iter().map(|&be|
            g.world.get_component::<household::Building>(be)
                .and_then(|b| b.stock.get("honey").copied()).unwrap_or(0)).sum();
        g.force_event(EventId::BeeTree);
        let honey1: u32 = g.building_entities.iter().map(|&be|
            g.world.get_component::<household::Building>(be)
                .and_then(|b| b.stock.get("honey").copied()).unwrap_or(0)).sum();
        assert!(honey1 > honey0 || g.building_entities.is_empty(), "the combs came back on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("bee tree")), "the find was told on {:?}", st);
    }
}

/// A deer in the fields: she grazes a bed down and the folk watch
/// from the hedge all the same
#[test]
fn test_the_doe_at_dawn() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::DeerInFields);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the folk watched from the hedge on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("deer")), "the doe was told on {:?}", st);
    }
}

/// A wild boar: the sow roots the beds a little a day while she
/// holds, and she moves off when her five days are out
#[test]
fn test_the_sow_roots_about() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        for x in g.gardens.iter_mut() { x.2 = x.2.max(1.0); }
        let growth0: f32 = g.gardens.iter().map(|x| x.2).sum();
        g.force_event(EventId::WildBoar);
        assert!(g.rare.iter().any(|r| r.id == EventId::WildBoar), "the sow's about");
        stage::run_days(&mut g, 2);
        let growth1: f32 = g.gardens.iter().map(|x| x.2).sum();
        assert!(growth1 < growth0 || g.gardens.is_empty(), "she rooted the beds on {:?}", st);
        stage::run_days(&mut g, 6);
        assert!(g.rare.iter().all(|r| r.id != EventId::WildBoar), "she moved off on {:?}", st);
    }
}

/// An owl in the tower: her watch keeps the vermin off the whole
/// village while she holds - thirty days, then she's gone
#[test]
fn test_the_owl_keeps_the_mice_off() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::OwlInTower);
        assert!(g.rare.iter().any(|r| r.id == EventId::OwlInTower), "she's in the tower");
        stage::run_days(&mut g, 3);
        // the larders hold where her watch stands: the rat-toll
        // machinery counts no toll while she hunts
        let watched = g.building_entities.iter().all(|&be| g.cat_watches(be));
        assert!(watched || g.building_entities.is_empty(), "the mice learned quiet on {:?}", st);
        stage::run_days(&mut g, 32);
        assert!(g.rare.iter().all(|r| r.id != EventId::OwlInTower), "she's gone on {:?}", st);
    }
}

/// A mob of crows: they drop on the larders or the beds, and rise
/// laughing
#[test]
fn test_the_black_storm() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::CrowsMob);
        assert!(g.log.iter().any(|l| l.contains("crows")), "the mob was told on {:?}", st);
    }
}

/// A fish run: the reach is alive with them and the baskets fill
/// while it lasts - six days, then the river's quiet again
#[test]
fn test_the_reach_is_alive() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::FishRun);
        assert!(g.rare.iter().any(|r| r.id == EventId::FishRun), "the run's on");
        stage::run_days(&mut g, 3);
        let catch = g.rare.iter().find(|r| r.id == EventId::FishRun).map(|r| r.aux).unwrap_or(0);
        assert!(catch > 0, "the catch went on the baskets on {:?}", st);
        stage::run_days(&mut g, 7);
        assert!(g.rare.iter().all(|r| r.id != EventId::FishRun), "the river's quiet on {:?}", st);
    }
}

/// A chimney fire: smoke gone wrong at a rolled home, the fire
/// lit for real, and the house grieves it
#[test]
fn test_smoke_gone_wrong() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::ChimneyFire);
        assert!(g.fire.active() || g.building_entities.is_empty(),
            "the fire's lit on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("chimney fire")), "the fire was told on {:?}", st);
    }
}

/// The hearth-grief lands on a house that can feel it - the held
/// see no smoke
#[test]
fn test_the_smoke_skips_the_held() {
    let mut g = stage::staged(stage::Stage::Ordinary);
    let held: Vec<Entity> = g.building_entities.iter().copied()
        .flat_map(|be| g.world.get_component::<household::Building>(be)
            .map(|b| b.occupants.clone()).unwrap_or_default())
        .collect();
    assert!(!held.is_empty(), "the staged village has hearth-folk");
    for &o in &held {
        g.world.add_component(o, crate::game::dungeon::CastleHeld { since: 0 });
    }
    g.force_event(EventId::ChimneyFire);
    assert!(held.iter().all(|&o| g.world.get_component::<desires::InnerLife>(o)
        .map(|l| l.grievances.iter().all(|(p, _)| *p != desires::Problem::NoHome))
        .unwrap_or(true)), "no hearth-grief lands on the held");
}

/// A fouled well: the water's off and folk get the sour gut while
/// it holds - a week, then the well runs clean again
#[test]
fn test_the_water_runs_sour() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::FouledWell);
        assert!(g.rare.iter().any(|r| r.id == EventId::FouledWell), "the well's off");
        stage::run_days(&mut g, 2);
        let sick = g.villagers.iter().any(|&v| g.world.get_component::<desires::InnerLife>(v)
            .map(|l| l.grievances.iter().any(|(p, u)| *p == desires::Problem::SourGut && g.tick <= *u))
            .unwrap_or(false));
        assert!(sick, "the sour gut spread on {:?}", st);
        stage::run_days(&mut g, 8);
        assert!(g.rare.iter().all(|r| r.id != EventId::FouledWell), "the well ran clean on {:?}", st);
    }
}

/// A granary leak: the rain finds a seam and a share of the stock
/// goes to rot
#[test]
fn test_the_rain_finds_a_seam() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        if let Some(f) = g.world.get_component_mut::<villager::FoodSource>(g.market_food) {
            f.servings = f.servings.max(50);
        }
        let food0 = g.world.get_component::<villager::FoodSource>(g.market_food)
            .map(|f| f.servings).unwrap_or(0);
        g.force_event(EventId::GranaryLeak);
        let food1 = g.world.get_component::<villager::FoodSource>(g.market_food)
            .map(|f| f.servings).unwrap_or(0);
        assert!(food1 < food0, "the stock spoiled on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("granary leak")), "the leak was told on {:?}", st);
    }
}

/// A cart overturns: the wheel finds the rut, the load's in the
/// mud, and the driver takes the loss
#[test]
fn test_the_wheel_finds_the_rut() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::CartOverturns);
        assert!(g.log.iter().any(|l| l.contains("cart overturns")), "the spill was told on {:?}", st);
    }
}

/// Drowned in the pond: a rolled soul doesn't come back from the
/// shallows, and her house grieves her
#[test]
fn test_the_shallows_took_her() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let alive0 = g.villagers.iter().filter(|&&v| !villager::is_dead(&g.world, v)).count();
        g.force_event(EventId::DrownedInPond);
        let alive1 = g.villagers.iter().filter(|&&v| !villager::is_dead(&g.world, v)).count();
        assert!(alive1 < alive0 || alive0 == 0, "the pond took one on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("drowned") || l.contains("pond")), "the drowning was told on {:?}", st);
    }
}

/// A lost child: the house wears the worry while she's out, and
/// the homecoming lifts it when the wood lets her go
#[test]
fn test_the_wood_lets_her_go() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::LostChild);
        assert!(g.rare.iter().any(|r| r.id == EventId::LostChild), "she's out");
        stage::run_days(&mut g, 1);
        let worried = g.villagers.iter().any(|&v| g.world.get_component::<desires::InnerLife>(v)
            .map(|l| l.grievances.iter().any(|(p, u)| *p == desires::Problem::Grief && g.tick <= *u))
            .unwrap_or(false));
        assert!(worried || g.villagers.iter().all(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none()),
            "the worry wore on {:?}", st);
        stage::run_days(&mut g, 3);
        assert!(g.rare.iter().all(|r| r.id != EventId::LostChild), "she's home on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("wood") || l.contains("lost child")), "it was told on {:?}", st);
    }
}

/// The wood takes no captive: a kid behind the castle's door is
/// not the one the roll can send out in the dark
#[test]
fn test_the_wood_takes_no_captive() {
    let mut g = stage::staged(stage::Stage::Ordinary);
    let kids: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_some())
        .collect();
    assert!(!kids.is_empty(), "the staged village has kids");
    for &k in &kids {
        g.world.add_component(k, crate::game::dungeon::CastleHeld { since: 0 });
    }
    g.force_event(EventId::LostChild);
    assert!(g.log.iter().any(|l| l.contains("late for supper")),
        "no kid was sent to the wood");
    assert!(g.rare.iter().find(|r| r.id == EventId::LostChild)
        .map(|r| r.aux).unwrap_or(0) == 0, "no captive rides the aux");
}

/// A foundling: a babe in a basket on a rolled home's step - one
/// more mouth under that roof, and the street talks
#[test]
fn test_the_babe_in_the_basket() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let kids0 = g.villagers.iter().filter(|&&v|
            g.world.get_component::<crate::game::vocation::Kid>(v).is_some()).count();
        g.force_event(EventId::Foundling);
        let kids1 = g.villagers.iter().filter(|&&v|
            g.world.get_component::<crate::game::vocation::Kid>(v).is_some()).count();
        assert!(kids1 > kids0 || g.building_entities.is_empty(), "the basket's inside on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("foundling")), "the basket was told on {:?}", st);
    }
}

/// The basket's not laid at a door nobody can answer - a house of
/// captives takes no foundling in
#[test]
fn test_the_basket_needs_a_door_open() {
    let mut g = stage::staged(stage::Stage::Ordinary);
    let occ: Vec<Entity> = g.building_entities.iter().copied()
        .flat_map(|be| g.world.get_component::<household::Building>(be)
            .map(|b| b.occupants.clone()).unwrap_or_default())
        .collect();
    assert!(!occ.is_empty(), "the staged village has hearth-folk");
    for &o in &occ {
        g.world.add_component(o, crate::game::dungeon::CastleHeld { since: 0 });
    }
    let kids0 = g.villagers.len();
    g.force_event(EventId::Foundling);
    assert_eq!(g.villagers.len(), kids0, "no foundling landed at a captive door");
    assert!(g.log.iter().any(|l| l.contains("no door to lay her")),
        "the basket found no door");
}

/// A quarrel in the square: two tempers meet, the street watches,
/// and neither side walks away sweet
#[test]
fn test_the_street_turned_out() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let conflicts0 = g.world.query::<crate::game::relationships::Relationship>().iter()
            .filter(|&&e| g.world.get_component::<crate::game::relationships::Relationship>(e)
                .map(|r| r.strength < 0.0).unwrap_or(false)).count();
        g.force_event(EventId::Quarrel);
        let conflicts1 = g.world.query::<crate::game::relationships::Relationship>().iter()
            .filter(|&&e| g.world.get_component::<crate::game::relationships::Relationship>(e)
                .map(|r| r.strength < 0.0).unwrap_or(false)).count();
        assert!(conflicts1 > conflicts0 || g.villagers.len() < 2, "tempers met on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("quarrel")), "the scene was told on {:?}", st);
    }
}

/// The old oak falls: a crack like thunder, the landmark's gone,
/// and the wood splits up fair to every hearth
#[test]
fn test_the_landmark_comes_down() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let wood0: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .and_then(|s| s.goods.get("wood").copied()).unwrap_or(0)).sum();
        g.force_event(EventId::OldOakFalls);
        let wood1: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .and_then(|s| s.goods.get("wood").copied()).unwrap_or(0)).sum();
        assert!(wood1 > wood0 || g.building_entities.is_empty(), "the wood split up on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("old oak")), "the fall was told on {:?}", st);
    }
}

/// A coin hoard: the spade rings on a pot of old silver, and a
/// rolled finder's purse fills
#[test]
fn test_the_spade_rings() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let gold0: f32 = g.villagers.iter().map(|&v|
            g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0)).sum();
        g.force_event(EventId::CoinHoard);
        let gold1: f32 = g.villagers.iter().map(|&v|
            g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0)).sum();
        assert!(gold1 > gold0 || g.villagers.is_empty(), "the pot filled a purse on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("coin hoard") || l.contains("silver")), "the find was told on {:?}", st);
    }
}

/// Old stones: the plow turns up someone's old work - stone for
/// the robbing and a marvel to touch
#[test]
fn test_who_laid_them() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let stone0: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .and_then(|s| s.goods.get("stone").copied()).unwrap_or(0)).sum();
        g.force_event(EventId::OldStones);
        let stone1: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .and_then(|s| s.goods.get("stone").copied()).unwrap_or(0)).sum();
        assert!(stone1 > stone0 || g.villagers.is_empty(), "the stones were robbed on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("old stones")), "the find was told on {:?}", st);
    }
}

/// Old stones: the plow turns up someone's old work - a marvel to
/// touch, and the stock of it lands somewhere safe
#[test]
fn test_somebody_built_here() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let v = g.villagers.iter().copied()
            .find(|&v| !villager::is_dead(&g.world, v)).unwrap();
        g.force_event(EventId::OldStones);
        assert!(desires::is_satisfied(&g.world, v, Want::Joy, g.tick)
            || g.villagers.iter().any(|&o| desires::is_satisfied(&g.world, o, Want::Joy, g.tick)),
            "the marvel of it on {:?}", st);
        let stone: u32 = g.villagers.iter().map(|&o|
            g.world.get_component::<crate::game::livelihood::Satchel>(o)
                .and_then(|s| s.goods.get("stone").copied()).unwrap_or(0)).sum();
        assert!(stone > 0 || g.villagers.is_empty(), "the stone went into hands on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("old stones")), "the find was told on {:?}", st);
    }
}

/// A spring bubbles up: cold sweet water out of the ground - the
/// pit's head rises where it's low, and the beds drink
#[test]
fn test_cold_sweet_water() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.land.pit_water = 0.3;
        g.force_event(EventId::SpringBubbles);
        assert!(g.land.pit_water > 0.3, "the spring fed the water on {:?}", st);
        assert!(g.land.pit_water <= 0.5, "the spring doesn't flood on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("spring")), "the water was told on {:?}", st);
    }
}

/// A honey tree: a hive in a hollow trunk - the combs come back
/// in hands that can trade them, stings and all
#[test]
fn test_the_combs_came_back() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let honey0: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .and_then(|s| s.goods.get("honey").copied()).unwrap_or(0)).sum();
        g.force_event(EventId::HoneyTree);
        let honey1: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .and_then(|s| s.goods.get("honey").copied()).unwrap_or(0)).sum();
        assert!(honey1 > honey0 || g.villagers.is_empty(), "the combs came back on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("honey")), "the tree was told on {:?}", st);
    }
}

/// A truffle: the sow noses it out in the oak-leaves, and the
/// apothecary pays dear for it
#[test]
fn test_the_sow_noses_it_out() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let gold0: f32 = g.villagers.iter().map(|&v|
            g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0)).sum();
        g.force_event(EventId::Truffle);
        let gold1: f32 = g.villagers.iter().map(|&v|
            g.world.get_component::<Purse>(v).map(|p| p.gold).unwrap_or(0.0)).sum();
        assert!(gold1 > gold0 || g.villagers.is_empty(), "the apothecary paid on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("truffle") || l.contains("oak-leaves")), "the find was told on {:?}", st);
    }
}

/// A flush of herbs: the hedge is thick with simples, and the
/// satchels fill while it lasts - ten days, then it's gone
#[test]
fn test_the_hedge_is_thick() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::HerbFlush);
        assert!(g.rare.iter().any(|r| r.id == EventId::HerbFlush), "the flush's on");
        stage::run_days(&mut g, 3);
        let stocked: u32 = g.herb_hut.map(|h|
            g.world.get_component::<household::Building>(h)
                .map(|b| b.stock.values().sum()).unwrap_or(0)).unwrap_or(0);
        assert!(stocked > 0 || g.herb_hut.is_none(), "the hut's shelf filled on {:?}", st);
        stage::run_days(&mut g, 11);
        assert!(g.rare.iter().all(|r| r.id != EventId::HerbFlush), "the flush's gone on {:?}", st);
    }
}

/// A berry year: the hedges are black with them and the baskets
/// come to the market - a fortnight, then the picking's done
#[test]
fn test_the_hedges_are_black() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::BerryYear);
        assert!(g.rare.iter().any(|r| r.id == EventId::BerryYear), "the year's on");
        stage::run_days(&mut g, 3);
        let picked = g.rare.iter().find(|r| r.id == EventId::BerryYear).map(|r| r.aux).unwrap_or(0);
        assert!(picked > 0, "the baskets came in on {:?}", st);
        stage::run_days(&mut g, 15);
        assert!(g.rare.iter().all(|r| r.id != EventId::BerryYear), "the picking's done on {:?}", st);
    }
}

/// A storm's leavings: the reach gives back - driftwood and a
/// barrel on the strand, picked clean by noon
#[test]
fn test_the_reach_gives_back() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        let goods0: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .map(|s| s.goods.values().sum()).unwrap_or(0)).sum();
        g.force_event(EventId::StormWashUp);
        let goods1: u32 = g.villagers.iter().map(|&v|
            g.world.get_component::<crate::game::livelihood::Satchel>(v)
                .map(|s| s.goods.values().sum()).unwrap_or(0)).sum();
        assert!(goods1 > goods0 || g.villagers.is_empty(), "the strand was picked on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("leavings")), "the strand was told on {:?}", st);
    }
}

/// The prodigal returns: the door that raised him feasts him
/// home, and the road glitters in his tales while he stays
#[test]
fn test_the_road_glittered() {
    for st in [stage::Stage::Ordinary, stage::Stage::Famine] {
        let mut g = stage::staged(st);
        g.force_event(EventId::ProdigalSon);
        for _ in 0..40 { g.advance_rare(); if g.rare.iter().any(|r| r.id == EventId::ProdigalSon && r.until.is_some()) { break; } }
        stage::run_days(&mut g, 2);
        let lifted = g.villagers.iter().any(|&v| desires::is_satisfied(&g.world, v, Want::Joy, g.tick));
        assert!(lifted || g.villagers.is_empty(), "the tales lifted them on {:?}", st);
        assert!(g.log.iter().any(|l| l.contains("prodigal")), "the return was told on {:?}", st);
    }
}

/// The stray dog picks the loneliest heart and stays
#[test]
fn test_the_stray_dog_picks_a_heart() {
    let mut g = Game::new();
    let pets_before = g.pets.len();
    g.force_event(EventId::StrayDog);
    for _ in 0..2000 {
        if g.rare.iter().any(|r| r.id == EventId::StrayDog && r.until.is_some()) { break; }
        g.advance_rare();
    }
    assert!(g.rare.iter().any(|r| r.id == EventId::StrayDog), "he's about");
    // the frame draws him as a dog, not a man
    let model = g.ui_model();
    assert!(model.people.iter().any(|p| matches!(p.kind, PKind::Pet(Species::Dog))),
        "the frame shows a dog");
    // his stay ends: someone takes him in
    stage::run_days(&mut g, 7);
    assert!(g.rare.iter().all(|r| r.id != EventId::StrayDog));
    assert_eq!(g.pets.len(), pets_before + 1, "the stray stayed - somebody has a dog");
    assert!(g.log.iter().any(|l| l.contains("has a dog now") || l.contains("drifts on")),
        "the log tells how it ended");
}
