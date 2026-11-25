use crate::session::*;
use crate::session::routine::HOLY_EVERY;

#[test]
fn test_the_soundscape_raises_its_cues() {
    let mut g = Game::new();
    // the ambient bed (rain, anvil, festival) no longer rides sfx_tick:
    // the soundscape plays it by place (A4) - only the moments remain
    g.precip = Some(crate::sim::precipitation::PrecipKind::Rain);
    for tick in [10, 20, 45, 90] {
        g.sfx.clear();
        g.sfx_tick(tick);
        assert!(g.sfx.iter().all(|c| !matches!(c.kind,
            "rain" | "anvil" | "festival")),
            "ambient cues moved to the soundscape: {:?}",
            g.sfx.iter().map(|c| &c.kind).collect::<Vec<_>>());
    }
}

#[test]
fn test_the_holy_day_is_seen_and_heard() {
    let mut g = Game::new();
    // run through the first holy day's dawn frame
    while g.tick < HOLY_EVERY { g.update(); }
    // the bell rings over the square that morning
    assert!(g.sfx.iter().any(|c| c.kind == "bell"),
        "the bell should ring on the holy day: {:?}", g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
    // and somebody says the word over their head
    let said = g.bubbles.iter().any(|b| {
        ["gather", "faith", "Holy", "Pray"].iter().any(|k| b.1.contains(k))
    });
    assert!(said, "somebody should speak at the gathering: {:?}",
        g.bubbles.iter().map(|b| &b.1).collect::<Vec<_>>());
}

#[test]
fn test_cues_reach_the_scene() {
    let mut g = Game::new();
    g.emit_sfx("wonder", 20.0, 5.0, 0.7);
    let m = g.ui_model();
    assert!(m.sfx.iter().any(|k| k == "wonder"), "the cue's on the scene for the audio layer");
    // and a frame later it's gone - cues live exactly one read
    g.update();
    assert!(g.sfx.is_empty(), "the queue resets");
}

// A5's scene cues (#1501): each beat of village life raises the cue the
// audio layer turns into a voice.

fn sfx_has(g: &Game, kind: &str) -> bool {
    g.sfx.iter().any(|c| c.kind == kind)
}

/// A found cause with `n` members, all aggrieved on the taxes - the
/// standing recipe the campaign tests cook with
fn found_tax_campaign(g: &mut Game, n: usize) {
    for &v in &g.villagers[..n] {
        desires::add_grievance(&mut g.world, v, desires::Problem::Taxed, g.tick + 200);
    }
    g.world.get_component_mut::<crate::sim::community::Organization>(g.org).unwrap().members =
        g.villagers[..n].to_vec();
    g.world.get_component_mut::<desires::Organizer>(g.seats[0].player).unwrap()
        .issues.push("the taxes".into());
    g.maybe_found_campaign(0).unwrap();
}

#[test]
fn alarm_on_fire_start() {
    let mut g = Game::new();
    crate::session::stage::force(&mut g, crate::session::events::EventId::ChimneyFire);
    assert!(sfx_has(&g, "alarm"), "a house catching raises the alarm: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn alarm_on_crackdown() {
    let mut g = Game::new();
    found_tax_campaign(&mut g, 6);
    // under a ban, any crowd over the cap is broken up on sight - a
    // march of one still meets the knights
    g.world.add_component(g.org, crate::game::reactions::AssemblyBan { cap: 0 });
    g.march_at(0);
    for _ in 0..12000 { g.update(); if g.formation.is_none() { break; } }
    assert!(sfx_has(&g, "alarm"), "the knights riding out raises the alarm: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn toll_on_death() {
    let mut g = Game::new();
    crate::session::stage::stage(&mut g, crate::session::stage::Stage::Bereft);
    g.grief_tick(g.tick);
    assert!(sfx_has(&g, "toll"), "the passing bell rings for the newly dead: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn cheer_on_townhall_win() {
    let mut g = Game::new();
    found_tax_campaign(&mut g, 8);
    // a coordinator named, and the cause bonded to them - the room fills
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Delegate);
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    for i in 1..8 {
        for _ in 0..3 {
            crate::game::relationships::record_interaction(&mut g.world, g.villagers[i],
                crate::game::relationships::Interaction {
                    other: g.villagers[0],
                    kind: crate::game::relationships::InteractionKind::Aid, intensity: 1.0 });
        }
    }
    // and the man they face has his own dog in the fight - a demand he
    // shares gets carried, not refused
    let collector = g.collector.unwrap();
    desires::add_grievance(&mut g.world, collector, desires::Problem::Taxed, g.tick + 200);
    g.seats[0].ctx_actions.push(CtxAct::Townhall);
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    assert!(sfx_has(&g, "cheer"), "a carried demand sends the street up: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn chant_once_per_day_while_march_live() {
    use crate::session::formation::{Formation, FormationKind};
    let mut g = Game::new();
    let who = g.villagers[0];
    let (x, y) = g.world.get_component::<Pos>(who).map(|p| (p.x, p.y)).unwrap();
    g.formation = Some(Formation {
        kind: FormationKind::March, seat: 0, born: g.tick,
        members: vec![who], route: vec![(x + 30.0, y)], at: 0,
        stuck: 0, waited: 0, leg0: -1.0,
        cheered: Default::default(), dest: None,
    });
    g.formation_step();
    assert!(sfx_has(&g, "chant"), "the column raises its voice");
    g.sfx.clear();
    g.formation_step();
    assert!(!sfx_has(&g, "chant"), "once a day, not every step");
    g.tick += 1;
    g.formation_step();
    assert!(sfx_has(&g, "chant"), "a new day, the chant again");
}

#[test]
fn coins_on_purchase() {
    let mut g = Game::new();
    let (buyer, seller) = (g.villagers[0], g.villagers[1]);
    g.world.get_component_mut::<crate::game::livelihood::Purse>(buyer).unwrap().gold = 50.0;
    g.world.get_component_mut::<crate::game::livelihood::Satchel>(seller).unwrap()
        .goods.insert("bread".into(), 5);
    g.arrive(buyer, Goal::BuyFrom(seller, "bread", 1));
    assert!(sfx_has(&g, "coins"), "the till rings on a sale: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn quill_on_petition() {
    let mut g = Game::new();
    found_tax_campaign(&mut g, 4);
    g.seats[0].selected_target = 0;
    g.seats[0].picked = true;
    g.seats[0].ctx_actions.push(CtxAct::Petition);
    g.ctx_act_at(0, g.seats[0].ctx_actions.len() - 1);
    assert!(sfx_has(&g, "quill"), "every letter is a scratch of the quill: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn door_on_visit() {
    let mut g = Game::new();
    found_tax_campaign(&mut g, 4);
    let c = crate::game::campaign::campaign_of(&g.world, g.org).unwrap();
    let canvasser = g.villagers[0];
    // the canvasser knows the doors they knock - the round reaches
    // whoever has a tie to them (grown folk, not the members' own)
    let members: Vec<Entity> = g.world.get_component::<crate::sim::community::Organization>(g.org)
        .unwrap().members.clone();
    let outside: Vec<Entity> = g.villagers.iter().copied()
        .filter(|&v| g.world.get_component::<crate::game::vocation::Kid>(v).is_none()
            && !members.contains(&v))
        .take(6).collect();
    for &v in &outside {
        let r = g.world.create_entity();
        g.world.add_component(r, crate::game::relationships::Relationship {
            from: v, other: canvasser,
            kind: crate::game::relationships::RelKind::Friend,
            strength: 0.5, interactions: 1, ..Default::default() });
    }
    // let the morning settle first: at spawn the canvasser stands
    // inside a house no path can reach, so the round waits a few
    // frames for folk to step out to their routines
    for _ in 0..1200 { g.update(); }
    g.org_survey = Some(crate::game::campaign::OrgSurvey {
        campaign: c, issue: "the taxes".into(),
        canvassers: vec![canvasser], days_left: 3,
        knocked: Default::default(),
    });
    g.sfx.clear();
    g.survey_tick(g.tick);
    assert!(sfx_has(&g, "door"), "a knock sounds at the door: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn cart_arrive_on_delivery() {
    let mut g = Game::new();
    let who = g.villagers[0];
    g.arrive(who, Goal::Cart(0));
    assert!(sfx_has(&g, "cart_arrive"), "the wheels' clatter lands at the stop: {:?}",
        g.sfx.iter().map(|c| c.kind).collect::<Vec<_>>());
}

#[test]
fn gate_edge_triggered_once() {
    let mut g = Game::new();
    let &(gx, gy) = g.map.gates().first().expect("the town has its gates");
    // a passable neighbour on each side of the gap to walk between
    let nbrs: Vec<(i32, i32)> = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter()
        .map(|(dx, dy)| (gx + dx, gy + dy))
        .filter(|&(x, y)| g.map.passable(x, y)).collect();
    assert!(nbrs.len() >= 2, "a gate stands between two open tiles");
    let who = g.villagers[0];
    if let Some(p) = g.world.get_component_mut::<Pos>(who) {
        p.x = nbrs[0].0 as f32 + 0.5; p.y = nbrs[0].1 as f32 + 0.5;
    }
    g.travels.push(Travel { who, path: vec![(gx, gy), nbrs[1]],
        goal: Goal::Herd(who) });
    // the walker's own pace decides how many frames the crossing takes;
    // the cue should still sound exactly once for the whole pass
    let mut heard = 0;
    for _ in 0..600 {
        g.advance_travels();
        heard += g.sfx.iter().filter(|c| c.kind == "gate").count();
        g.sfx.retain(|c| c.kind != "gate");
    }
    assert_eq!(heard, 1, "the crossing creaks once, then not again");
}

/// The sky turning wet cues the first drops where the village stands.
#[test]
fn rain_cues_the_wet_onset() {
    use crate::sim::{climate, precipitation::PrecipKind};
    let mut g = Game::new();
    g.precip = Some(PrecipKind::Drizzle);
    g.update(); // drain any cues the birth frames raised
    g.sfx.clear();
    g.precip = None;
    {
        let c = g.world.get_component_mut::<climate::Climate>(g.climate).unwrap();
        c.season = climate::Season::Summer;
        c.rainfall_drift = 0.9; // HeavyRain, well past the threshold
    }
    g.update();
    let cue = g.sfx.iter().find(|c| c.kind == "rain")
        .expect("the onset should cue the first drops");
    assert!(cue.intensity > 0.5, "a heavy onset lands loud: {}", cue.intensity);
}

/// Snow and the haar cue the wind instead of drops.
#[test]
fn gale_cues_the_cold_onsets() {
    use crate::sim::climate;
    let mut g = Game::new();
    g.precip = None;
    g.update();
    g.sfx.clear();
    {
        let c = g.world.get_component_mut::<climate::Climate>(g.climate).unwrap();
        c.season = climate::Season::Winter;
        c.rainfall_drift = 0.9; // HeavySnow
    }
    g.update();
    assert!(g.sfx.iter().any(|c| c.kind == "gale"),
        "snow should cue the wind: {:?}", g.sfx.iter().map(|c| &c.kind).collect::<Vec<_>>());
    // and the haar - a damp spring day just short of rain - sounds it too
    g.sfx.clear();
    g.precip = None;
    {
        let c = g.world.get_component_mut::<climate::Climate>(g.climate).unwrap();
        c.season = climate::Season::Spring;
        c.rainfall_drift = 0.48; // just under RAIN_THRESHOLD, inside the haar band
    }
    g.update();
    assert!(g.sfx.iter().any(|c| c.kind == "gale"),
        "the haar's fog rolls in on a gust too");
}

/// A celebration's ring sounds where the neighbours gather.
#[test]
fn festival_rings_the_celebration() {
    let mut g = Game::new();
    let at = (g.village.0 as f32 + 0.5, g.village.1 as f32 + 0.5);
    g.local_celebration(1, at, None, "a coming of age");
    assert!(g.sfx.iter().any(|c| c.kind == "festival"),
        "the gathered ring should ring: {:?}", g.sfx.iter().map(|c| &c.kind).collect::<Vec<_>>());
}

/// The floor's answer to the settled word is a murmur, not silence.
#[test]
fn murmur_answers_the_settled_word() {
    use crate::game::campaign::{self, DecisionRule, Framing, IssueVote};
    let mut g = Game::new();
    let org = g.seats[0].org;
    let vote = IssueVote { winner: "the taxes".into(), ballots: vec![] };
    campaign::found_campaign(&mut g.world, org, &vote,
        Framing::CommonFoe, DecisionRule::Majority, 0);
    let present: Vec<Entity> = g.villagers.iter().copied().take(4).collect();
    g.debrief_at(0, &present);
    assert!(g.sfx.iter().any(|c| c.kind == "murmur"),
        "the debrief's settled word should get its murmur");
}

/// The wedding's peal rings the couple out of the rite.
#[test]
fn bells_peal_the_wedding() {
    let mut g = Game::new();
    let (a, b, by) = (g.villagers[0], g.villagers[1], g.villagers[2]);
    let site = (10, 10);
    for e in [a, b, by] {
        g.world.get_component_mut::<Pos>(e).unwrap().x = 10.0;
        g.world.get_component_mut::<Pos>(e).unwrap().y = 10.0;
    }
    for (me, them) in [(a, b), (b, a)] {
        g.world.add_component(me, villager::Betrothed {
            partner: them, site, by, old: false, since: 0 });
    }
    g.rites_tick(1);
    assert!(g.sfx.iter().any(|c| c.kind == "bells"),
        "the wedding should ring a peal: {:?}",
        g.sfx.iter().map(|c| &c.kind).collect::<Vec<_>>());
}

/// A wolf in the fold bays where it struck.
#[test]
fn baying_off_the_fold_on_wolf_night() {
    let mut g = Game::new();
    assert!(g.fold.is_some(), "a fresh village keeps a fold");
    g.wolf_night(1);
    assert!(g.sfx.iter().any(|c| c.kind == "baying"),
        "the wolf's taking should howl");
}

/// A fox in the yard scatters the hens with a squawk.
#[test]
fn hens_scatter_on_fox_night() {
    let mut g = Game::new();
    assert!(!g.hen_houses.is_empty(), "a fresh village keeps hens");
    g.fox_night(1);
    assert!(g.sfx.iter().any(|c| c.kind == "hens"),
        "the yard should scatter squawking");
}
