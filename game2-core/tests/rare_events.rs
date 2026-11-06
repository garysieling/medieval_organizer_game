// Integration sweep: every rare event, forced onto a few staged villages
// and played out for weeks of turns. The events are semi-rare on purpose,
// so the tests don't wait for the schedule - `force` puts the moment on,
// `run_days` plays it through the same update() a player sits under.
//
// Both surfaces get checked: the backend through the probe (the secret
// API tests see and players don't), and the frontend through the UiModel
// the frame would draw.

use game2_core::session::{self, ClientKind};
use game2_core::session::events::{self, EventId, EventKind};
use game2_core::session::stage::{self, Stage};
use game2_core::game::mycology::MushKind;

/// Every event in the registry, fired on three staged villages and run
/// out a month: the village still works, both surfaces still serve.
#[test]
fn every_event_plays_out_in_every_kind_of_village() {
    // ~300 seeded villages a month deep each - farm the combos out over
    // every core; each is its own game, nothing is shared between them
    let work: Vec<(usize, Stage)> = events::EVENTS.iter().enumerate()
        .flat_map(|(i, _)| stage::sweep_stages(i).into_iter().map(move |st| (i, st)))
        .collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(16);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if i >= work.len() { break; }
                let (ei, st) = work[i];
                let s = &events::EVENTS[ei];
                let mut g = stage::staged(st);
                let t0 = g.tick;
                stage::force(&mut g, s.id);
                // it fired: the record says so, whatever its shape
                assert!(g.probe().rare_fired.iter().any(|(id, _)| *id == format!("{:?}", s.id)),
                    "{:?} didn't record its firing on {:?}", s.id, st);
                // thirty days on: the same update() a player sits under
                stage::run_days(&mut g, 30);
                assert!(g.tick >= t0 + 30, "{:?}/{:?}: the clock didn't run", s.id, st);
                // the backend's secret view still answers
                let pv = g.probe();
                assert_eq!(pv.tick, g.tick);
                assert_eq!(pv.inner.len(), g.villagers.len());
                // and the frame still draws: a serializable model with a village in it
                let model = g.ui_model();
                assert!(!model.people.is_empty(), "{:?}/{:?}: nobody on the map", s.id, st);
                let json = serde_json::to_string(&model).unwrap();
                assert!(!json.is_empty());
            });
        }
    });
}

/// A visitor walks the road in, stands the pitch labelled by its errand,
/// and walks out - the frame shows all of it
#[test]
fn visitors_walk_in_stand_and_walk_out() {
    for s in events::EVENTS.iter().filter(|s| s.kind == EventKind::Visitor).take(6) {
        let mut g = stage::staged(Stage::Ordinary);
        stage::force(&mut g, s.id);
        assert!(g.rare.iter().any(|r| r.id == s.id && r.e.is_some()),
            "{:?}: no figure on the road", s.id);
        // walk him in: frames, not days
        for _ in 0..4000 {
            if g.rare.iter().any(|r| r.id == s.id && r.until.is_some()) { break; }
            g.advance_rare();
        }
        assert!(g.rare.iter().any(|r| r.id == s.id && r.until.is_some()),
            "{:?}: never reached its pitch", s.id);
        // the frame shows a labelled figure while he stands
        let model = g.ui_model();
        assert!(model.people.iter().any(|p| p.label.as_deref() == Some(s.name)),
            "{:?}: '{}' not on the map", s.id, s.name);
        // and the road takes him home when his days are done
        stage::run_days(&mut g, s.stay + 8);
        assert!(g.rare.iter().all(|r| r.id != s.id), "{:?}: never left", s.id);
        assert!(g.next_rare.iter().any(|(id, d)| *id == s.id && *d > g.tick),
            "{:?}: not due again", s.id);
    }
}

/// The probe is a separate API: the same game serves the player's frame,
/// the agent's observation and the test's secret view, and only the last
/// carries the secrets
#[test]
fn the_secret_view_is_separate_from_what_players_and_agents_see() {
    let mut g = stage::staged(Stage::Ordinary);
    // secrets to hide: an unsaid truth and a death-cap
    g.pending_discovery = Some((g.villagers[0], "the tithe is a sham", g.tick));
    if let Some(p) = g.mush_patches.first_mut() { p.kind = MushKind::DeathCap; }

    let probe_json = serde_json::to_string(&g.probe()).unwrap();
    assert!(probe_json.contains("the tithe is a sham"), "the probe sees the unsaid");

    let ui_json = serde_json::to_string(&g.ui_model()).unwrap();
    assert!(!ui_json.contains("the tithe is a sham"), "the player's frame hides it");
    assert!(!ui_json.contains("poison"), "the player's frame hides the death-cap");
    assert!(!ui_json.contains("rare_due"), "the player's frame hides the schedule");

    // an agent's observation carries none of it either
    let seat = g.add_agent_seat(g.villagers[0]);
    let obs_json = serde_json::to_string(&g.agent_obs(seat).unwrap()).unwrap();
    assert!(!obs_json.contains("the tithe is a sham"), "agents see what players see");
    assert!(!obs_json.contains("poison"));
    assert!(!obs_json.contains("grievances"), "an agent sees no one's inner life");
}

/// Staging + forcing works through the wire types too: ClientKind::Agent
/// and the serialized surfaces a remote test would read
#[test]
fn the_surfaces_serialize_for_remote_tests() {
    let mut g = stage::staged(Stage::Famine);
    stage::force(&mut g, EventId::NewsCrier);
    stage::run_days(&mut g, 3);
    // UiModel round-trips (it's what the ws ServerMsg::Scene carries)
    let model = g.ui_model();
    let msg = session::ServerMsg::Scene(Box::new(model));
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("\"Scene\""));
    // and the probe's JSON parses back into the documented shape
    let pv: session::probe::ProbeView =
        serde_json::from_str(&serde_json::to_string(&g.probe()).unwrap()).unwrap();
    assert!(pv.rare_fired.iter().any(|(id, _)| id == "NewsCrier"));
    assert!(pv.inner.iter().all(|i| i.hunger >= 0.0));
    let _ = ClientKind::Agent; // the agent surface is covered by the obs test above
}
