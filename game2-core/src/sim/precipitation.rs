//! Precipitation: what is falling from the sky today, read from the climate's
//! wetness (so what the player sees is what the crops feel). Wetter days fall
//! harder; in winter it falls as snow.

use crate::sim::climate::{Climate, Season, RAIN_THRESHOLD};

/// What is falling from the sky
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PrecipKind {
    Drizzle,
    LightRain,
    Showers,   // bursty: comes and goes within the day
    Rain,
    HeavyRain,
    Flurries,
    Snow,
    HeavySnow,
}

impl PrecipKind {
    pub fn is_snow(self) -> bool {
        matches!(self, PrecipKind::Flurries | PrecipKind::Snow | PrecipKind::HeavySnow)
    }

    /// 0-1 how heavy it is (drives particle density and overcast)
    pub fn intensity(self) -> f32 {
        match self {
            PrecipKind::Drizzle => 0.2,
            PrecipKind::Flurries => 0.25,
            PrecipKind::LightRain => 0.4,
            PrecipKind::Showers => 0.55,
            PrecipKind::Snow => 0.6,
            PrecipKind::Rain => 0.7,
            PrecipKind::HeavyRain => 1.0,
            PrecipKind::HeavySnow => 1.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PrecipKind::Drizzle => "drizzle",
            PrecipKind::LightRain => "light rain",
            PrecipKind::Showers => "showers",
            PrecipKind::Rain => "rain",
            PrecipKind::HeavyRain => "heavy rain",
            PrecipKind::Flurries => "snow flurries",
            PrecipKind::Snow => "snow",
            PrecipKind::HeavySnow => "heavy snow",
        }
    }
}

/// Winter wetness above which it snows (winter runs drier than the rain
/// threshold, so snow starts lower or it would hardly ever snow)
pub const SNOW_THRESHOLD: f32 = 0.35;

/// What is falling today, if anything
pub fn precip_kind(c: &Climate) -> Option<PrecipKind> {
    let w = c.rainfall_drift;
    if c.season == Season::Winter {
        if w <= SNOW_THRESHOLD { None }
        else if w < SNOW_THRESHOLD + 0.08 { Some(PrecipKind::Flurries) }
        else if w < SNOW_THRESHOLD + 0.18 { Some(PrecipKind::Snow) }
        else { Some(PrecipKind::HeavySnow) }
    } else {
        // how far past the rain threshold today is
        let over = w - RAIN_THRESHOLD;
        if over <= 0.0 { None }
        else if over < 0.03 { Some(PrecipKind::Drizzle) }
        else if over < 0.06 { Some(PrecipKind::LightRain) }
        // summer's wet days come as showers rather than steady rain
        else if over < 0.1 || c.season == Season::Summer { Some(PrecipKind::Showers) }
        else if over < 0.16 { Some(PrecipKind::Rain) }
        else { Some(PrecipKind::HeavyRain) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::climate::{is_raining, wetness};

    fn climate(season: Season, wet: f32) -> Climate {
        Climate { season, year: 1, day: 0, temperature_drift: 0.0, rainfall_drift: wet, severity: 0.0 }
    }

    #[test]
    fn test_rain_falls_exactly_when_the_climate_says_raining() {
        for season in [Season::Spring, Season::Summer, Season::Autumn] {
            for i in 0..=100 {
                let c = climate(season, i as f32 / 100.0);
                assert_eq!(precip_kind(&c).is_some(), is_raining(&c), "{:?} wetness {}", season, c.rainfall_drift);
            }
        }
    }

    #[test]
    fn test_wetter_days_fall_harder() {
        let light = precip_kind(&climate(Season::Spring, 0.51)).unwrap();
        let heavy = precip_kind(&climate(Season::Spring, 0.8)).unwrap();
        assert_eq!(light, PrecipKind::Drizzle);
        assert_eq!(heavy, PrecipKind::HeavyRain);
        assert!(light.intensity() < heavy.intensity());
    }

    #[test]
    fn test_snow_only_in_winter_and_it_does_snow() {
        let mut snow_days = 0;
        for season in [Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
            for d in 0..90 {
                let c = climate(season, wetness(season, d));
                if let Some(k) = precip_kind(&c) {
                    assert_eq!(k.is_snow(), season == Season::Winter, "{:?} day {}", season, d);
                    if k.is_snow() { snow_days += 1; }
                }
            }
        }
        assert!(snow_days > 10, "winter should see some snow, got {} days", snow_days);
    }

    #[test]
    fn test_every_rain_kind_happens_in_a_year() {
        let mut seen = vec![];
        for season in [Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
            for d in 0..90 {
                if let Some(k) = precip_kind(&climate(season, wetness(season, d))) {
                    if !seen.contains(&k) { seen.push(k); }
                }
            }
        }
        for k in [PrecipKind::Drizzle, PrecipKind::LightRain, PrecipKind::Showers, PrecipKind::Rain, PrecipKind::Flurries, PrecipKind::Snow] {
            assert!(seen.contains(&k), "{:?} never happens: saw {:?}", k, seen);
        }
    }
}
