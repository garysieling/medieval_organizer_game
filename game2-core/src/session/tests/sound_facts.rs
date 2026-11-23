//! The UiModel projections the sound layer reads (#1487): the weather
//! event, the calendar and the frame clock - all things the seat
//! already sees on screen, surfaced for a client that cannot watch.
use crate::session::*;
use crate::session::routine::{is_holy, is_market_day, HOLY_EVERY};
use crate::sim::weather::{CurrentWeather, Weather};

#[test]
fn weather_none_when_clear() {
    let mut g = Game::new();
    g.update();
    assert!(g.ui_model().weather.is_none(), "clear sky still projects an event");
}

#[test]
fn weather_storm_projects_kind_and_days_left() {
    let mut g = Game::new();
    // the sim keeps one weather entity, rolled in place at the day's turn
    let set = |g: &mut Game, kind, intensity, duration| {
        if let Some(w) = g.world.get_component_mut::<CurrentWeather>(g.weather) {
            w.kind = kind; w.intensity = intensity; w.duration = duration;
        }
    };
    set(&mut g, Weather::Storm, 0.8, 5);
    let w = g.ui_model().weather.expect("a storm in the sky must project");
    assert_eq!(w.kind, "storm");
    assert_eq!(w.days_left, 5);
    assert!((w.intensity - 0.8).abs() < 0.001, "intensity {}", w.intensity);
    // and a frost reads as its own word
    set(&mut g, Weather::Frost, 0.5, 2);
    let w = g.ui_model().weather.expect("frost projects");
    assert_eq!(w.kind, "frost");
    assert_eq!(w.days_left, 2);
    // and clear weather projects nothing
    set(&mut g, Weather::Clear, 0.0, 1);
    assert!(g.ui_model().weather.is_none(), "a spent event is the clear sky");
}

#[test]
fn calendar_holy_day_every_7() {
    let mut g = Game::new();
    while g.tick < HOLY_EVERY { g.update(); }
    assert!(is_holy(g.tick), "test premise: day {} is holy", g.tick);
    assert!(g.ui_model().calendar.holy_day, "the holy day projects");
    g.tick += 1;
    assert!(!g.ui_model().calendar.holy_day, "the day after isn't holy");
}

#[test]
fn calendar_market_day_matches_is_market_day() {
    let mut g = Game::new();
    g.tick = 19;
    assert!(!g.ui_model().calendar.market_day);
    g.tick = 20;
    assert!(is_market_day(g.tick));
    assert!(g.ui_model().calendar.market_day, "day 20 is the market's");
}

#[test]
fn calendar_days_to_counts_down() {
    let mut g = Game::new();
    g.tick = 5;
    let c = g.ui_model().calendar;
    assert_eq!(c.days_to_holy, 2, "day 5 -> 2 days to the holy day");
    assert_eq!(c.days_to_market, 15, "day 5 -> 15 to the market");
    g.tick = 7; // the holy day itself counts 0
    assert_eq!(g.ui_model().calendar.days_to_holy, 0);
}

#[test]
fn calendar_feast_name_when_feast() {
    let mut g = Game::new();
    g.tick = 81; // May Day on the celestial wheel
    let c = g.ui_model().calendar;
    assert_eq!(c.feast.as_deref(), Some("May Day"));
    g.tick = 82;
    assert!(g.ui_model().calendar.feast.is_none(), "no feast the day after");
}

#[test]
fn clock_frames_per_day_normal_and_turbo() {
    let mut g = Game::new();
    g.update();
    let c = g.ui_model().clock;
    assert_eq!(c.frames_per_day, 21 * 60, "normal speed: a 21s day");
    assert!((c.speed - 1.0).abs() < 0.01, "normal multiplier is 1.0, got {}", c.speed);
    g.speed = 4;
    let c = g.ui_model().clock;
    assert_eq!(c.frames_per_day, 225, "turbo: 15s/4 frames per day");
    assert!(c.speed > 5.0 && c.speed < 6.0, "turbo multiplier ~5.6, got {}", c.speed);
    assert!(c.day_frac >= 0.0 && c.day_frac < 1.0, "day_frac in range");
}

#[test]
fn old_payload_without_new_fields_decodes() {
    let mut g = Game::new();
    g.update();
    let m = g.ui_model();
    let mut v: serde_json::Value = serde_json::to_value(&m).unwrap();
    for k in ["weather", "calendar", "clock"] { v.as_object_mut().unwrap().remove(k); }
    let old: UiModel = serde_json::from_value(v)
        .expect("an older server's frame must still decode");
    assert!(old.weather.is_none());
    assert!(!old.calendar.holy_day && old.calendar.days_to_holy == 0);
    assert_eq!(old.clock.frames_per_day, 0, "0 = unknown");
}
