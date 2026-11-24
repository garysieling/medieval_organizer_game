//! Soundscape tests for #1496 (A3): real `UiModel`s off
//! `Game::new_seeded(MAP_SEED)`, fields staged on the model itself.
//! The wrapping module keeps the crate's layering lint happy - a
//! `#[cfg(test)]` run is stripped before it greps for `Game`.
#[cfg(test)]
mod t {
    use crate::soundscape::*;
    use game2_core::game::tilemap::WallPiece;
    use game2_core::session::{BSprite, ConvoPanelV, EventFocusV, Game, MAP_SEED};
    use game2_core::sim::climate::Season;
    use game2_core::sim::precipitation::PrecipKind;

    fn model() -> UiModel { Game::new_seeded(MAP_SEED).ui_model() }

    /// The village's own heart: the listener at the player, zoom 1
    fn at_village(m: &UiModel) -> Listener { Listener::from_model(m) }

    #[test]
    fn camera_at_skeps_bees_top_emitter() {
        let mut m = model();
        m.season = Season::Summer;
        let skep = *m.skeps.first().expect("MAP_SEED keeps an apiary");
        let l = Listener::at(skep.0, skep.1, 1.0);
        let out = scan(&m, &l);
        assert_eq!(out.first().map(|e| e.kind), Some(EmitterKind::Bees),
            "the bees' bed tops the order at the skeps: {:?}",
            out.iter().map(|e| e.kind).collect::<Vec<_>>());
        assert_eq!(out[0].pos, Some(skep));
    }

    #[test]
    fn church_on_holy_day_has_bell_and_chant() {
        let mut m = model();
        m.calendar.holy_day = true;
        let church = m.buildings.iter().find(|b| b.sprite == BSprite::Church)
            .map(|b| (b.x, b.y)).expect("the village has a church");
        let l = Listener::at(church.0, church.1, 1.0); // within half radius
        let out = scan(&m, &l);
        let kinds: Vec<EmitterKind> = out.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&EmitterKind::ChurchBell), "the bell sounds: {kinds:?}");
        assert!(kinds.contains(&EmitterKind::Chant), "the chant carries: {kinds:?}");
        assert_eq!(out.iter().find(|e| e.kind == EmitterKind::ChurchBell).unwrap().tier,
            Tier::Soundmark, "the bell is the village's soundmark");
    }

    #[test]
    fn gate_with_train_has_wheels() {
        let mut m = model();
        let gate = m.map.walls.iter()
            .find(|(_, p)| matches!(p, WallPiece::GateAlongX | WallPiece::GateAlongY))
            .map(|(t, _)| *t).expect("the wall has a gate");
        m.trains.push((gate.0 as f32 + 0.5, gate.1 as f32 + 0.5));
        let l = Listener::at(gate.0 as f32, gate.1 as f32, 1.0);
        let out = scan(&m, &l);
        let kinds: Vec<EmitterKind> = out.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&EmitterKind::Gate), "the gate creaks: {kinds:?}");
        assert!(kinds.contains(&EmitterKind::Cart), "the wheels roll: {kinds:?}");
    }

    #[test]
    fn winter_has_no_insects() {
        let mut m = model();
        m.season = Season::Winter;
        let l = at_village(&m);
        let out = scan(&m, &l);
        assert!(!out.iter().any(|e| e.kind == EmitterKind::Insects),
            "winter is silent of insects");
        // and summer has them back
        m.season = Season::Summer;
        let out = scan(&m, &l);
        assert!(out.iter().any(|e| e.kind == EmitterKind::Insects),
            "summer's grass hums");
    }

    #[test]
    fn heavy_snow_adds_hush_and_drops_birds() {
        let mut m = model();
        m.season = Season::Spring;
        m.precip = Some(PrecipKind::HeavySnow);
        let l = at_village(&m);
        let out = scan(&m, &l);
        let hush = out.iter().find(|e| e.kind == EmitterKind::SnowHush)
            .expect("heavy snow brings the hush");
        assert_eq!(hush.lowpass_hz, 5000.0, "the master low-pass sits at 5 kHz");
        assert!(!out.iter().any(|e| e.kind == EmitterKind::Songbirds),
            "no birds sing in heavy snow");
    }

    #[test]
    fn zoomed_out_has_no_one_shots() {
        let m = model();
        // zoom far out: the radius clamps to 20 and every signal drops
        let l = Listener::at(m.map.width as f32 / 2.0, m.map.height as f32 / 2.0, 0.3);
        assert_eq!(l.radius_tiles, 20.0);
        let out = scan(&m, &l);
        assert!(!out.iter().any(|e| e.tier == Tier::Signal),
            "zoomed out, the village is a murmur - no one-shots: {:?}",
            out.iter().map(|e| e.kind).collect::<Vec<_>>());
    }

    #[test]
    fn radius_clamps_5_to_20() {
        assert_eq!(Listener::at(0.0, 0.0, 3.0).radius_tiles, 5.0, "zoomed in floors at 5");
        assert_eq!(Listener::at(0.0, 0.0, 1.0).radius_tiles, 10.0);
        assert_eq!(Listener::at(0.0, 0.0, 0.1).radius_tiles, 20.0, "zoomed out caps at 20");
    }

    #[test]
    fn scan_is_deterministic_and_ordered() {
        let m = model();
        let l = at_village(&m);
        let a = scan(&m, &l);
        let b = scan(&m, &l);
        assert_eq!(a, b, "same model, same listener, same emitters");
        // tier ascends, and within a tier weight never rises
        for w in a.windows(2) {
            assert!(w[0].tier <= w[1].tier, "tiers don't reorder");
            if w[0].tier == w[1].tier {
                assert!(w[0].weight >= w[1].weight,
                    "weight sorts desc in a tier: {:?} then {:?}",
                    w[0].kind, w[1].kind);
            }
        }
    }

    #[test]
    fn pan_equal_power_and_gain_curve_and_lowpass_curve() {
        let half = 540.0;
        assert_eq!(pan(0.0, half), 0.0);
        assert_eq!(pan(half, half), 1.0);
        assert_eq!(pan(-half, half), -1.0);
        assert_eq!(pan(half * 3.0, half), 1.0, "pan clamps past the edge");
        assert_eq!(gain(0.0, 10.0), 1.0, "at the ear it's full");
        let edge = gain(10.0, 10.0);
        assert!(edge > 0.0 && edge < 0.15, "at the radius it's faded: {edge}");
        assert!(gain(5.0, 10.0) > gain(9.0, 10.0), "gain falls with distance");
        assert_eq!(lowpass_hz(0.0, 10.0), 16000.0);
        assert_eq!(lowpass_hz(10.0, 10.0), 2000.0);
        assert_eq!(lowpass_hz(5.0, 10.0), 9000.0, "linear between");
    }

    #[test]
    fn conversation_does_not_change_scan() {
        let mut m = model();
        let l = at_village(&m);
        let quiet = scan(&m, &l);
        m.convo = Some(ConvoPanelV { decides: true, ..Default::default() });
        let talk = scan(&m, &l);
        assert_eq!(quiet, talk, "ducking is the director's job, not the scan's");
    }

    #[test]
    fn march_bed_follows_the_event() {
        let mut m = model();
        m.player_event = Some(EventFocusV { epoch: 1, kind: 0,
            pos: (m.map.width as f32 / 2.0, m.map.height as f32 / 2.0),
            extent: 20.0, moving: true });
        let l = at_village(&m);
        let out = scan(&m, &l);
        assert!(out.iter().any(|e| e.kind == EmitterKind::March
            && e.tier == Tier::Bed), "the march marches");
    }
}
