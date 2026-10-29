//! Precipitation: day-to-day rain and snow on top of the seasonal climate.
//! Spells start and stop on their own; winter turns rain into snow.

use crate::engine::ecs::{Entity, World};
use crate::sim::climate::{Climate, Season};

/// What is falling from the sky
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            PrecipKind::Drizzle => "Drizzle",
            PrecipKind::LightRain => "Light rain",
            PrecipKind::Showers => "Showers",
            PrecipKind::Rain => "Rain",
            PrecipKind::HeavyRain => "Heavy rain",
            PrecipKind::Flurries => "Snow flurries",
            PrecipKind::Snow => "Snow",
            PrecipKind::HeavySnow => "Heavy snow",
        }
    }
}

/// Current precipitation state (one per world, beside the Climate)
#[derive(Debug)]
pub struct Precipitation {
    pub current: Option<PrecipKind>,
    pub days_left: u32,
    pub seed: u64,
}

impl Precipitation {
    pub fn is_raining(&self) -> bool {
        self.current.map(|k| !k.is_snow()).unwrap_or(false)
    }
}

fn hash(mut x: u64) -> u64 {
    // splitmix64
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn unit(h: u64) -> f32 { (h % 10_000) as f32 / 10_000.0 }

/// Daily chance a dry day turns wet
fn start_chance(season: Season, rainfall_drift: f32) -> f32 {
    let base = match season {
        Season::Spring => 0.20,
        Season::Summer => 0.10,
        Season::Autumn => 0.16,
        Season::Winter => 0.14,
    };
    (base + rainfall_drift * 0.2).clamp(0.0, 0.9)
}

/// Pick a kind and duration (days) for a new spell
fn roll_spell(season: Season, roll: f32, dur_roll: u64) -> (PrecipKind, u32) {
    let kind = if season == Season::Winter {
        if roll < 0.4 { PrecipKind::Flurries } else if roll < 0.8 { PrecipKind::Snow } else { PrecipKind::HeavySnow }
    } else if season == Season::Summer {
        // summer: mostly showers and short downpours
        if roll < 0.15 { PrecipKind::Drizzle } else if roll < 0.6 { PrecipKind::Showers }
        else if roll < 0.8 { PrecipKind::Rain } else { PrecipKind::HeavyRain }
    } else if roll < 0.25 { PrecipKind::Drizzle } else if roll < 0.5 { PrecipKind::LightRain }
        else if roll < 0.7 { PrecipKind::Showers } else if roll < 0.9 { PrecipKind::Rain } else { PrecipKind::HeavyRain };
    let (lo, span) = match kind {
        PrecipKind::Showers | PrecipKind::HeavyRain | PrecipKind::HeavySnow => (1, 2),
        PrecipKind::Drizzle | PrecipKind::Flurries => (1, 3),
        _ => (2, 3),
    };
    (kind, lo + (dur_roll % span) as u32)
}

/// Spawn precipitation state (starts dry)
pub fn spawn_precipitation(world: &mut World, seed: u64) -> Entity {
    let e = world.create_entity();
    world.add_component(e, Precipitation { current: None, days_left: 0, seed });
    e
}

/// System: one roll per day. Run after SeasonSystem so it sees today's season.
pub struct PrecipitationSystem;
impl crate::engine::ecs::System for PrecipitationSystem {
    fn run(&mut self, world: &mut World) {
        let Some((season, day, year, drift)) = world.query::<Climate>().first()
            .and_then(|e| world.get_component::<Climate>(*e))
            .map(|c| (c.season, c.day, c.year, c.rainfall_drift)) else { return };
        for e in world.query::<Precipitation>() {
            if let Some(p) = world.get_component_mut::<Precipitation>(e) {
                if let Some(k) = p.current {
                    p.days_left = p.days_left.saturating_sub(1);
                    // a season change ends the spell (no snow lingering into spring)
                    if p.days_left == 0 || k.is_snow() != (season == Season::Winter) { p.current = None; }
                    continue;
                }
                let h = hash(p.seed ^ ((year as u64) << 32 | day as u64));
                if unit(h) < start_chance(season, drift) {
                    let (kind, days) = roll_spell(season, unit(hash(h)), hash(h ^ 0xABCD));
                    p.current = Some(kind);
                    p.days_left = days;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ecs::Scheduler;
    use crate::sim::climate::{spawn_climate, SeasonSystem};

    fn run_year() -> Vec<(Season, Option<PrecipKind>)> {
        let mut w = World::new();
        let c = spawn_climate(&mut w);
        let p = spawn_precipitation(&mut w, 7);
        let mut s = Scheduler::new();
        s.add_system(SeasonSystem);
        s.add_system(PrecipitationSystem);
        (0..360).map(|_| {
            s.tick(&mut w);
            (w.get_component::<Climate>(c).unwrap().season, w.get_component::<Precipitation>(p).unwrap().current)
        }).collect()
    }

    #[test]
    fn test_it_rains_and_clears() {
        let days = run_year();
        let wet = days.iter().filter(|(_, k)| k.is_some()).count();
        assert!(wet > 20 && wet < 250, "wet days {}", wet);
        // spells end: there is a wet day followed by a dry day
        assert!(days.windows(2).any(|d| d[0].1.is_some() && d[1].1.is_none()));
    }

    #[test]
    fn test_snow_only_in_winter() {
        for (season, kind) in run_year() {
            if let Some(k) = kind {
                if k.is_snow() { assert_eq!(season, Season::Winter); }
            }
        }
    }

    #[test]
    fn test_spell_lengths_bounded() {
        for season in [Season::Spring, Season::Summer, Season::Autumn, Season::Winter] {
            for i in 0..200u64 {
                let (_, days) = roll_spell(season, (i % 100) as f32 / 100.0, hash(i));
                assert!((1..=4).contains(&days));
            }
        }
    }
}
