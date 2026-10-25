//! Fundamentalism emergence: conditions that produce extremist communities.
//! From lit review (Cambridge-INET): fundamentalism emerges from weak peer
//! attachment + high income + low secular substitution + intolerance.
//! Not scripted - emerges when conditions align.

use super::religion::Congregation;

/// Conditions that enable fundamentalism
#[derive(Debug)]
pub struct FundamentalismConditions {
    pub peer_attachment: f32,    // low = weak bonds enable extremism
    pub real_income: f32,        // high = resources for extremism
    pub secular_goods: f32,      // low = fewer alternatives to religion
    pub tolerance: f32,          // low = less countervailing pressure
}

/// Compute fundamentalism risk (0-1, emerges from conditions)
pub fn fundamentalism_risk(c: &FundamentalismConditions) -> f32 {
    // Risk = low attachment * high income * low secular options * low tolerance
    let risk = (1.0 - c.peer_attachment)
        * c.real_income
        * (1.0 - c.secular_goods)
        * (1.0 - c.tolerance);
    risk.min(1.0)
}

/// Check if a congregation is becoming fundamentalist
pub fn is_fundamentalist(c: &Congregation, conditions: &FundamentalismConditions) -> bool {
    fundamentalism_risk(conditions) > 0.5 && c.orthodoxy > 0.6
}

/// Emergent effect: fundamentalist congregations produce cohesion for members
/// but hostility toward outsiders
pub fn fundamentalist_effects(c: &Congregation) -> (f32, f32) {
    let internal_cohesion = c.orthodoxy * 0.3;
    let external_hostility = c.orthodoxy * 0.4;
    (internal_cohesion, external_hostility)
}
