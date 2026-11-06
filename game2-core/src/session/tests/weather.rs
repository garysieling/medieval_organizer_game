use crate::session::*;
use crate::sim::weather::{self, Weather, CurrentWeather};
use crate::engine::ecs::System;

#[test]
fn test_a_drought_stints_the_harvest() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v| g.world.get_component::<Craft>(v)
        .map(|c| c.vocation == Vocation::Farmer).unwrap_or(false)).expect("no farmer");
    let (made_fair, _) = g.bench_work(v, g.tick);
    assert!(made_fair > 0.0);
    // a hard drought settles over the fields
    let w = g.world.get_component_mut::<CurrentWeather>(g.weather).unwrap();
    w.kind = Weather::Drought;
    w.intensity = 0.8;
    w.duration = 3;
    // the pending fraction from the fair day skews the next call, so zero
    // it out for a like-for-like read
    g.world.get_component_mut::<Craft>(v).unwrap().pending = 0.0;
    let (made_dry, _) = g.bench_work(v, g.tick);
    assert!(made_dry < made_fair, "a drought didn't stint the harvest: {} vs {}",
        made_dry, made_fair);
}

#[test]
fn test_a_heat_wave_wears_on_the_village() {
    let mut g = Game::new();
    let v = g.villagers[0];
    let c0 = g.world.get_component::<Health>(v).unwrap().condition;
    let w = g.world.get_component_mut::<CurrentWeather>(g.weather).unwrap();
    w.kind = Weather::HeatWave;
    w.intensity = 0.9;
    w.duration = 3;
    g.weather_tick(99); // day 99: no event roll determinism to lean on
    let c1 = g.world.get_component::<Health>(v).unwrap().condition;
    assert!(c1 < c0, "the heat spared the bones: {} -> {}", c0, c1);
    // and a long enough dry spell ends itself
    let w = g.world.get_component_mut::<CurrentWeather>(g.weather).unwrap();
    w.kind = Weather::Drought;
    w.duration = 1;
    weather::WeatherSystem.run(&mut g.world);
    assert_eq!(g.world.get_component::<CurrentWeather>(g.weather).unwrap().kind,
        Weather::Clear);
}
