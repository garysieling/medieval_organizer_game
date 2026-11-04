use crate::session::*;
use crate::client::*;

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
