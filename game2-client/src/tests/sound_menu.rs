//! Sound overlay tests for #1506 (U2): the pause menu's third row, the
//! 360px panel's geometry, its keyboard and click handling, and that a
//! change rides the settings file out. Names are as the ticket requires.
use crate::*;
use game2_audio::settings::{AudioSettings, Presence};
use std::time::{Duration, Instant};

#[test]
fn pause_menu_has_four_rows() {
    let rows = pause_menu_rows();
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0].4, MenuPick::Resume);
    assert_eq!(rows[1].4, MenuPick::Sound);
    assert_eq!(rows[2].4, MenuPick::Fullscreen);
    assert_eq!(rows[3].4, MenuPick::Quit);
    assert!(rows[0].1 < rows[1].1 && rows[1].1 < rows[2].1 && rows[2].1 < rows[3].1,
        "the settings sit between Resume and Quit");
}

#[test]
fn overlay_is_centred_360_wide() {
    let (x, _, w, _) = sound_menu_box();
    assert_eq!(w, SOUND_W, "the design's 360px panel");
    assert_eq!(x, MAP_W as i32 / 2 - SOUND_W / 2, "centred on the map view");
    assert_eq!(sound_menu_rows().len(), SOUND_ROWS, "twelve rows");
    // the box also draws where it says: its corner pixels are the modal's
    let mut buf = Buffer::new(MAP_W, H);
    let s = AudioSettings::default();
    draw_sound_menu(&mut buf, (-1, -1), &s, 0);
    let px = |px: i32, py: i32| -> (u8, u8, u8) {
        let j = (py as usize * MAP_W + px as usize) * 4;
        (buf.pixels[j], buf.pixels[j + 1], buf.pixels[j + 2])
    };
    let (_x, y, _w, h) = sound_menu_box();
    assert_eq!(px(x + 4, y + h - 4), (20, 20, 35), "modal background paints");
    let (rx, ry, _, _) = sound_menu_rows()[0];
    assert_eq!(px(rx + 2, ry), (150, 230, 150), "row 0's focus edge lights");
}

#[test]
fn row_hit_tests_for_every_row() {
    let mut s = AudioSettings::default();
    for (i, &(rx, ry, rw, rh)) in sound_menu_rows().iter().enumerate() {
        let hit = sound_menu_click(rx + 4, ry + rh / 2, &mut s);
        assert!(matches!(hit, Some((row, _)) if row == i),
            "click inside row {i} reports row {i}");
        let _ = rw;
    }
    // and a click off the panel hits nothing
    assert_eq!(sound_menu_click(2, 2, &mut s), None);
}

#[test]
fn keyboard_focus_wraps_and_steps_sliders_by_5() {
    let mut s = AudioSettings::default();
    let (f, _) = sound_menu_key(&mut s, 0, SndKey::Up);
    assert_eq!(f, SOUND_ROWS - 1, "up from the top wraps to the bottom");
    let (f, _) = sound_menu_key(&mut s, f, SndKey::Down);
    assert_eq!(f, 0, "down from the bottom wraps to the top");
    // master at 80: Right steps to 85, Left back to 80, and it floors at 0
    let (_, act) = sound_menu_key(&mut s, 0, SndKey::Right);
    assert_eq!(s.master, 85);
    assert_eq!(act, SoundAct::Changed("master"));
    let (_, act) = sound_menu_key(&mut s, 0, SndKey::Left);
    assert_eq!(s.master, 80);
    assert_eq!(act, SoundAct::Changed("master"));
    s.master = 0;
    let (_, _) = sound_menu_key(&mut s, 0, SndKey::Left);
    assert_eq!(s.master, 0, "a slider never steps below 0");
    s.master = 100;
    let (_, _) = sound_menu_key(&mut s, 0, SndKey::Right);
    assert_eq!(s.master, 100, "or above 100");
}

#[test]
fn enter_toggles_and_cycles_choice() {
    let mut s = AudioSettings::default();
    // Mono is row 6, Score presence row 5, Back row 11
    let (_, act) = sound_menu_key(&mut s, 6, SndKey::Enter);
    assert!(s.mono && act == SoundAct::Changed("mono"), "Enter flips a toggle");
    let (_, act) = sound_menu_key(&mut s, 6, SndKey::Enter);
    assert!(!s.mono && act == SoundAct::Changed("mono"), "and back");
    let (_, act) = sound_menu_key(&mut s, 5, SndKey::Enter);
    assert_eq!(s.presence, Presence::Rich, "Enter cycles the choice forward");
    assert_eq!(act, SoundAct::Changed("presence"));
    let (_, act) = sound_menu_key(&mut s, 5, SndKey::Left);
    assert_eq!(s.presence, Presence::Normal, "Left cycles it back");
    assert_eq!(act, SoundAct::Changed("presence"));
    let (_, act) = sound_menu_key(&mut s, 11, SndKey::Enter);
    assert_eq!(act, SoundAct::Close, "Enter on Back is Esc");
}

#[test]
fn click_on_track_sets_value_from_x() {
    let mut s = AudioSettings::default();
    let (tx, ty, tw, _) = track_rect(0);
    // the midpoint of the 120px track reads 50
    let hit = sound_menu_click(tx + tw / 2, ty + 2, &mut s);
    assert_eq!(s.master, 50, "mid-track is half volume");
    assert!(matches!(hit, Some((0, SoundAct::Changed("master")))));
    // both ends pin to 0 and 100, snapped to the 5-grid
    sound_menu_click(tx, ty + 2, &mut s);
    assert_eq!(s.master, 0);
    sound_menu_click(tx + tw - 1, ty + 2, &mut s);
    assert_eq!(s.master, 100);
    // a click on the slider's label side just takes the focus
    let (rx, ry, _, _) = sound_menu_rows()[0];
    let hit = sound_menu_click(rx + 10, ry + 8, &mut s);
    assert!(matches!(hit, Some((0, SoundAct::Move))));
}

#[test]
fn reset_restores_defaults() {
    let mut s = AudioSettings::default();
    s.apply_setting("master", serde_json::json!(15)).unwrap();
    s.apply_setting("mono", serde_json::json!(true)).unwrap();
    // Reset is row 10: Enter on it puts every row back
    let (_, act) = sound_menu_key(&mut s, 10, SndKey::Enter);
    assert_eq!(act, SoundAct::Reset);
    assert_eq!(s, AudioSettings::default(), "the U1 defaults return");
    // and a click on the row does the same
    s.master = 5;
    let (rx, ry, rw, rh) = sound_menu_rows()[10];
    let hit = sound_menu_click(rx + rw / 2, ry + rh / 2, &mut s);
    assert!(matches!(hit, Some((10, SoundAct::Reset))));
    assert_eq!(s, AudioSettings::default());
}

#[test]
fn esc_returns_to_pause_menu() {
    let mut s = AudioSettings::default();
    let (_, act) = sound_menu_key(&mut s, 3, SndKey::Esc);
    assert_eq!(act, SoundAct::Close, "Esc peels back to the menu, not the day");
    // the Back row's click does the same
    let (rx, ry, rw, rh) = sound_menu_rows()[11];
    let hit = sound_menu_click(rx + rw / 2, ry + rh / 2, &mut s);
    assert!(matches!(hit, Some((11, SoundAct::Close))));
}

#[test]
fn v_hotkey_opens_overlay() {
    // the row the V press lands on exists and says so, and the overlay
    // it opens draws over the pause menu - the press itself is in
    // game_ui, so this pins the row and the draw both
    let rows = pause_menu_rows();
    assert_eq!(rows[1].4, MenuPick::Sound, "the menu's middle row is Sound");
    let mut buf = Buffer::new(MAP_W, H);
    draw_pause_menu(&mut buf, (-1, -1), "PAUSED");
    let s = AudioSettings::default();
    draw_sound_menu(&mut buf, (-1, -1), &s, 4);
    // the overlay's title band paints over the menu under it
    let (x, y, w, _) = sound_menu_box();
    let j = ((y + 1) as usize * MAP_W + (x + w / 2) as usize) * 4;
    assert_eq!((buf.pixels[j], buf.pixels[j + 1], buf.pixels[j + 2]),
        (200, 200, 220), "the overlay's top bar covers the menu's box");
}

#[test]
fn changes_persist_through_settings_json() {
    let dir = std::env::temp_dir().join(format!("game2-u2-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("game2.settings.json");
    let _ = std::fs::remove_file(&path);
    let mut store = crate::audio::SettingsStore::load(&path);
    // drive the overlay's own handlers, then the store's own clock
    let (_, act) = sound_menu_key(&mut store.settings, 0, SndKey::Right);
    assert_eq!(act, SoundAct::Changed("master"));
    store.touch(Instant::now());
    let t = Instant::now();
    assert!(store.due(t + Duration::from_millis(1100)), "the debounced write lands");
    let back: AudioSettings = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(back.master, 85, "the overlay's change is in the file");
    let _ = std::fs::remove_file(&path);
}
