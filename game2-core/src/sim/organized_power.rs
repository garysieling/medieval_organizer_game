//! Organized power: a "people power" capacity that grows with skillful
//! use and atrophies when idle - the opposite of wealth, which depletes
//! when spent. From the backlog ("power builds as you spend it") and
//! researched in wiki/literature-reviews/sprint-7-organizing-power.md
//! (McAlevey, Alinsky, Ganz, Han):
//!   capacity += growth * successful_actions * development_quality
//!   capacity -= atrophy * idle_time
//!   failed/over-reaching actions cost capacity (burnout, demoralization)

/// An org's organized-power capacity (distinct from money/resources, which
/// depletes when spent). Grows from well-run actions, atrophies when idle.
#[derive(Debug, Clone, Copy)]
pub struct OrganizedPower {
    pub capacity: f32,   // 0+; how much people-power the org can call on
    pub idle_ticks: u32, // ticks since the last action, for atrophy tracking
}

impl OrganizedPower {
    pub fn new() -> Self { OrganizedPower { capacity: 0.0, idle_ticks: 0 } }
}

impl Default for OrganizedPower {
    fn default() -> Self { Self::new() }
}

/// Tuning knobs for the growth/atrophy balance.
#[derive(Debug, Clone, Copy)]
pub struct PowerParams {
    pub growth_rate: f32,     // capacity added by a well-run action
    pub atrophy_rate: f32,    // capacity lost per idle tick
    pub burnout_penalty: f32, // capacity lost for an overreaching/failed action
}

impl Default for PowerParams {
    fn default() -> Self {
        PowerParams { growth_rate: 0.15, atrophy_rate: 0.01, burnout_penalty: 0.2 }
    }
}

/// How an organizing action was run (Han's transactional vs transformational
/// organizing): transformational actions develop people - real roles,
/// reflection, relationship-building - and grow capacity the most.
/// Transactional asks barely develop anyone. An overreach - acting beyond
/// the org's actual strength without a structure test first - fails and
/// burns capacity (burnout, demoralization).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionQuality {
    Transformational,
    Transactional,
    Overreach,
}

/// Run an organizing action against this capacity.
pub fn apply_action(power: &mut OrganizedPower, params: &PowerParams, quality: ActionQuality) {
    power.idle_ticks = 0;
    match quality {
        ActionQuality::Transformational => power.capacity += params.growth_rate,
        ActionQuality::Transactional => power.capacity += params.growth_rate * 0.2,
        ActionQuality::Overreach => {
            power.capacity = (power.capacity - params.burnout_penalty).max(0.0);
        }
    }
}

/// Step one tick of idleness: capacity atrophies without use, like an
/// unused muscle (the same use-it-or-lose-it shape as sim::trails, applied
/// to people power instead of terrain).
pub fn step_idle(power: &mut OrganizedPower, params: &PowerParams) {
    power.idle_ticks += 1;
    power.capacity = (power.capacity - params.atrophy_rate).max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transformational_actions_grow_capacity_more_than_transactional() {
        let params = PowerParams::default();
        let mut transformational = OrganizedPower::new();
        let mut transactional = OrganizedPower::new();
        for _ in 0..10 {
            apply_action(&mut transformational, &params, ActionQuality::Transformational);
            apply_action(&mut transactional, &params, ActionQuality::Transactional);
        }
        assert!(transformational.capacity > transactional.capacity,
            "developing members should build more capacity than one-off asks");
    }

    #[test]
    fn overreach_burns_capacity() {
        let params = PowerParams::default();
        let mut power = OrganizedPower { capacity: 0.5, idle_ticks: 3 };
        apply_action(&mut power, &params, ActionQuality::Overreach);
        assert!(power.capacity < 0.5, "an overreaching action should cost capacity");
        assert_eq!(power.idle_ticks, 0, "taking any action, even a failed one, resets idleness");
    }

    #[test]
    fn idleness_atrophies_capacity_toward_zero() {
        let params = PowerParams::default();
        let mut power = OrganizedPower { capacity: 1.0, idle_ticks: 0 };
        for _ in 0..200 {
            step_idle(&mut power, &params);
        }
        assert!(power.capacity < 0.01, "an org that never acts should lose its organized power, got {}", power.capacity);
    }

    #[test]
    fn capacity_never_goes_negative() {
        let params = PowerParams::default();
        let mut power = OrganizedPower { capacity: 0.0, idle_ticks: 0 };
        for _ in 0..50 {
            step_idle(&mut power, &params);
        }
        assert_eq!(power.capacity, 0.0);
        apply_action(&mut power, &params, ActionQuality::Overreach);
        assert_eq!(power.capacity, 0.0);
    }

    #[test]
    fn sustained_good_organizing_outgrows_atrophy() {
        // The muscle metaphor only holds if regular use beats idle decay.
        let params = PowerParams::default();
        let mut power = OrganizedPower::new();
        for _ in 0..20 {
            apply_action(&mut power, &params, ActionQuality::Transformational);
            step_idle(&mut power, &params);
        }
        assert!(power.capacity > 1.0, "an org that organizes regularly should keep building power, got {}", power.capacity);
    }
}
