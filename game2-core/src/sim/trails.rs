//! Trails: per-tile trail strength that grows with footfall and decays
//! toward a terrain baseline. Stronger trails are cheaper to move through,
//! which makes agents prefer them, reinforcing the trail - the feedback
//! loop from Helbing, Schweitzer, Keltsch & Molnár's Active Walker Model
//! (Phys. Rev. E 56, 1997).
//!
//! See wiki/literature-reviews/sprint-7-roads-and-trails.md for the
//! research and design notes this implements:
//!   dG/dt = (G0 - G)/T  +  I * (1 - G/Gmax) * footfall(x)

/// Per-tile trail state.
#[derive(Debug, Clone, Copy)]
pub struct Trail {
    pub strength: f32, // 0 = wild, ~0.3 trackway, ~0.7 well-worn, 1.0 engineered road
}

impl Trail {
    pub fn wild() -> Self { Trail { strength: 0.0 } }
}

/// Tuning knobs from the ant-trail paper (Edelstein-Keshet et al. 1995):
/// too little decay and everything becomes road; too much and nothing persists.
#[derive(Debug, Clone, Copy)]
pub struct TrailParams {
    pub reinforce_rate: f32, // I: how fast footfall builds the trail
    pub decay_rate: f32,     // 1/T: how fast it decays back to baseline
    pub saturation: f32,     // Gmax: the strongest a trail can get
}

impl Default for TrailParams {
    fn default() -> Self {
        TrailParams { reinforce_rate: 0.08, decay_rate: 0.02, saturation: 1.0 }
    }
}

/// Step one tile's trail strength for one tick.
/// `footfall` is how many agents crossed this tile this tick.
/// `baseline` is the terrain's natural, trail-free state (0 for most
/// terrain; higher for terrain that erodes into paths faster).
pub fn step_trail(trail: &mut Trail, footfall: u32, baseline: f32, params: &TrailParams) {
    let decay = (baseline - trail.strength) * params.decay_rate;
    let reinforce = params.reinforce_rate
        * (1.0 - trail.strength / params.saturation).max(0.0)
        * footfall as f32;
    trail.strength = (trail.strength + decay + reinforce).clamp(0.0, params.saturation);
}

/// Movement cost for crossing a tile with the given trail strength: strong
/// trails are cheaper to cross. `k` controls how much the trail helps
/// (0 = no effect, 1 = a fully-worn trail is free to cross).
pub fn movement_cost(terrain_cost: f32, trail_strength: f32, k: f32) -> f32 {
    terrain_cost * (1.0 - k * trail_strength).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footfall_reinforces_the_trail() {
        let mut trail = Trail::wild();
        let params = TrailParams::default();
        let mut prev = trail.strength;
        for _ in 0..20 {
            step_trail(&mut trail, 5, 0.0, &params);
            assert!(trail.strength >= prev, "strength should not drop while in heavy use");
            prev = trail.strength;
        }
        assert!(trail.strength > 0.0, "repeated footfall should build up a trail");
    }

    #[test]
    fn trail_decays_toward_baseline_without_use() {
        let mut trail = Trail { strength: 0.8 };
        let params = TrailParams::default();
        for _ in 0..150 {
            step_trail(&mut trail, 0, 0.0, &params);
        }
        assert!(trail.strength < 0.1, "an unused trail should fade back toward baseline, got {}", trail.strength);
    }

    #[test]
    fn strength_never_exceeds_saturation() {
        let mut trail = Trail::wild();
        let params = TrailParams::default();
        for _ in 0..1000 {
            step_trail(&mut trail, 50, 0.0, &params);
        }
        assert!(trail.strength <= params.saturation);
    }

    #[test]
    fn stronger_trails_are_cheaper_to_cross() {
        let cost_wild = movement_cost(1.0, 0.0, 0.6);
        let cost_worn = movement_cost(1.0, 1.0, 0.6);
        assert!(cost_worn < cost_wild, "a well-worn trail should be faster than wild terrain");
    }

    #[test]
    fn decay_and_reinforcement_balance_at_equilibrium() {
        // With steady, moderate footfall, strength should settle rather than
        // oscillate or run away (the regime the ant-trail paper calls "order").
        let mut trail = Trail::wild();
        let params = TrailParams::default();
        for _ in 0..500 {
            step_trail(&mut trail, 1, 0.0, &params);
        }
        let settled = trail.strength;
        step_trail(&mut trail, 1, 0.0, &params);
        assert!((trail.strength - settled).abs() < 0.01, "strength should have settled by now");
    }
}
