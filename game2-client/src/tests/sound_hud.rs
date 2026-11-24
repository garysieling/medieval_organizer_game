//! Sound HUD tests for #1505 (U1): M's toast, the muted glyph, and the
//! settings file's debounced persistence. Names are as the ticket
//! requires.
use crate::*;
use game2_audio::settings::AudioSettings;
use std::time::{Duration, Instant};

#[test]
fn m_toggles_mute_and_toasts() {
    let mut view = View::new(Art::load_fs("../assets", 0));
    view.toggle_mute(120);
    assert!(view.mute, "M flips the mute on");
    let note = view.note.as_ref().expect("the press announces itself");
    assert_eq!(note.text, "Sound off");
    assert_eq!(note.born, Some(120), "born on the model frame the key landed");
    assert_eq!(note.at, None, "client notices aren't camera jumps");
    view.toggle_mute(200);
    assert!(!view.mute);
    assert_eq!(view.note.as_ref().unwrap().text, "Sound on");
    assert_eq!(view.note.as_ref().unwrap().born, Some(200));
}

#[test]
fn muted_glyph_drawn_only_when_muted() {
    let mut game = Game::new_seeded(MAP_SEED);
    while game.tick < 30 { game.update(); }
    let m = game.ui_model();
    let view = View::new(Art::load_fs("../assets", 0));
    // the same line and icon slots render lays out: the glyph's box
    // starts where mute_glyph_x puts it, and its slash is the strip's
    // only red
    let hud = hud_text(m.season, m.tick, m.panel.food,
        m.precip.map(|k| k.label()), m.paused, m.food_yield, m.harsh_winter);
    let iw = view.art.icon(&format!("icon-{}", format!("{:?}", m.season).to_lowercase()))
        .map_or(0, |_| 16)
        + view.art.icon(m.precip.map(precip_icon)
            .unwrap_or(if m.rainbow_from.is_some() { "icon-rainbow" } else { "icon-sun" }))
        .map_or(0, |_| 16);
    let gx = mute_glyph_x(&hud, iw);
    let slash_at = |buf: &Buffer| -> bool {
        (0..12).any(|i| {
            let j = (((5 + 11 - i) as usize) * W + (gx + i) as usize) * 4;
            buf.pixels[j] == 220 && buf.pixels[j + 1] == 90 && buf.pixels[j + 2] == 70
        })
    };
    let buf = view.render(&m, (-1, -1));
    assert!(!slash_at(&buf), "unmuted draws no crossed-out horn");
    let mut view = View::new(Art::load_fs("../assets", 0));
    view.mute = true;
    let buf = view.render(&m, (-1, -1));
    assert!(slash_at(&buf), "muted draws the crossed-out horn in the HUD strip");
}

#[test]
fn settings_file_written_after_debounce() {
    let dir = std::env::temp_dir().join(format!("game2-u1-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("game2.settings.json");
    let _ = std::fs::remove_file(&path);
    let mut store = crate::audio::SettingsStore::load(&path);
    let t0 = Instant::now();
    store.settings.master = 55;
    store.touch(t0);
    assert!(!store.due(t0 + Duration::from_millis(500)),
        "half a second in, nothing's written yet");
    assert!(!path.exists());
    assert!(store.due(t0 + Duration::from_millis(1100)),
        "a second after the last change the file lands");
    let back: AudioSettings = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(back.master, 55);
    // a later change restarts the clock
    store.settings.music = 20;
    store.touch(t0 + Duration::from_secs(2));
    assert!(!store.due(t0 + Duration::from_millis(2900)));
    assert!(store.due(t0 + Duration::from_secs(3)));
    let back: AudioSettings = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(back.music, 20);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn corrupt_settings_file_falls_back_to_defaults() {
    let dir = std::env::temp_dir().join(format!("game2-u1-corrupt-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("game2.settings.json");
    std::fs::write(&path, "{ not json at all !!!").unwrap();
    let store = crate::audio::SettingsStore::load(&path);
    assert_eq!(store.settings, AudioSettings::default(),
        "a corrupt file is defaults, not a crash");
    // and a flush writes a clean copy back over it
    let mut store = store;
    store.flush();
    let back: AudioSettings = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(back, AudioSettings::default());
    let _ = std::fs::remove_file(&path);
}
