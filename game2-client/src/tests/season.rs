use crate::*;

fn brightness(c: (u8, u8, u8)) -> u32 { c.0 as u32 + c.1 as u32 + c.2 as u32 }

#[test]
fn test_winter_land_is_snowy_and_water_stays_water() {
    for h in [0.25, 0.45, 0.65] {
        let summer = season_colour(h, Season::Summer);
        let winter = season_colour(h, Season::Winter);
        assert!(brightness(winter) > brightness(summer) + 150, "winter land at {} should be snow-pale: {:?} vs {:?}", h, winter, summer);
    }
    let water = season_colour(0.05, Season::Winter);
    assert!(water.2 > water.0 && water.2 > water.1, "winter water should still read as water: {:?}", water);
}

#[test]
fn test_autumn_grass_turns_gold_and_summer_is_the_base_palette() {
    let (r, g, _) = season_colour(0.25, Season::Autumn);
    assert!(r > 100 && r as i32 > g as i32 - 20, "autumn grass should turn golden: {:?}", (r, g));
    assert_eq!(season_colour(0.25, Season::Summer), terrain_colour(0.25));
}

#[test]
fn test_the_cottages_have_a_slot_for_every_season() {
    // the seasonal pool (#1276): every season's slot resolves - the
    // base look for the green seasons, the snowed set for winter -
    // until a per-cottage seasonal sheet lands (rd/seasons/
    // cottage<i>-<season>.png), when only that slot swaps
    let art = Art::load_fs("../assets", 0);
    for season in [Season::Spring, Season::Summer, Season::Autumn] {
        assert_eq!(art.house_season(0, season).is_some(), art.house(0).is_some(),
            "{:?}'s cottage lost its base look", season);
    }
    assert_eq!(art.house_season(0, Season::Winter).is_some(), art.houses_winter[0].is_some(),
        "winter's cottage lost its snowed look");
    assert!(art.houses_by_season.iter().all(|s| s.len() == art.houses.len()),
        "a season's pool doesn't match the cottage pool");
}

#[test]
fn a_landed_seasonal_variant_swaps_into_its_season() {
    // the landed pool (#1303): cottage1's autumn file carries its own
    // art - the season's slot doesn't share the base sprite
    let art = Art::load_fs("../assets", 0);
    let aut = art.house_season(0, Season::Autumn).as_ref()
        .expect("cottage1's autumn sprite landed");
    let base = art.house(0).as_ref().unwrap();
    assert_ne!(aut.pixels, base.pixels, "autumn shares the base sprite");
    // and the botháns' real winter art still stands over any variant
    assert_eq!(art.house_season(9, Season::Winter).as_ref().map(|s| &s.pixels),
        art.houses_winter[9].as_ref().map(|s| &s.pixels),
        "a bothán's winter isn't the snowed sprite");
}
