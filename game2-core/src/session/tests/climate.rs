use crate::session::*;
use crate::sim::climate::{self, Climate, Season};

#[test]
fn test_a_lean_winter_thins_the_harvest() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v| g.world.get_component::<Craft>(v)
        .map(|c| c.vocation == Vocation::Farmer).unwrap_or(false)).expect("no farmer");
    let (made_spring, _) = g.bench_work(v, g.tick);
    assert!(made_spring > 0.0);
    g.world.get_component_mut::<Craft>(v).unwrap().pending = 0.0;
    // the year turns to a hard winter
    let c = g.world.get_component_mut::<Climate>(g.climate).unwrap();
    c.season = Season::Winter;
    c.rainfall_drift = climate::wetness(Season::Winter, 5);
    let (made_winter, _) = g.bench_work(v, g.tick);
    assert!(made_winter < made_spring, "winter didn't thin the field: {} vs {}",
        made_winter, made_spring);
}

#[test]
fn test_a_dry_spell_stints_the_field() {
    let mut g = Game::new();
    let v = g.villagers.iter().copied().find(|&v| g.world.get_component::<Craft>(v)
        .map(|c| c.vocation == Vocation::Farmer).unwrap_or(false)).expect("no farmer");
    // a wet season's work, then a dry spell's, several days each so the
    // stint rises above the whole-unit granularity
    g.world.get_component_mut::<Craft>(v).unwrap().pending = 0.0;
    g.world.get_component_mut::<Climate>(g.climate).unwrap().rainfall_drift = 0.8;
    let wet: f32 = (0..6).map(|_| g.bench_work(v, g.tick).0).sum();
    g.world.get_component_mut::<Craft>(v).unwrap().pending = 0.0;
    g.world.get_component_mut::<Climate>(g.climate).unwrap().rainfall_drift = 0.0;
    let dry: f32 = (0..6).map(|_| g.bench_work(v, g.tick).0).sum();
    assert!(dry < wet, "a dry spell didn't stint: {} vs {}", dry, wet);
}
