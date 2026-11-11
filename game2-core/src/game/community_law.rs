//! The village's own law, beside `laws.rs`: that book is the crown's and
//! the lord's word - this one is the neighbors'. When the same wrong
//! keeps coming back - the commons cropped too close, light measure at
//! market, fists and blame in the street - the village agrees a bye-law
//! and starts fining. Kin and near neighbors stand surety for a debtor,
//! partners swear to shared work, and the whole of it waxes and wanes as
//! the community's strength over the generations.

use crate::engine::ecs::Entity;

/// The kind of wrong a bye-law answers: the village writes no rule for a
/// one-off, only for the friction that keeps returning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ByeKind {
    /// The commons cropped too close: thin woods, the dry pit, the fouled reach
    Commons,
    /// Light purses and short measure: what's taken at market
    Trade,
    /// Fists and blame: conduct the neighbors won't have
    Conduct,
}

/// Which friction a grievance counts toward, if any
pub fn bye_kind(p: crate::game::desires::Problem) -> Option<ByeKind> {
    use crate::game::desires::Problem::*;
    match p {
        ThinWood | DryPit | StinkingReach => Some(ByeKind::Commons),
        ShortChanged | Robbed => Some(ByeKind::Trade),
        Rowdy | Blamed => Some(ByeKind::Conduct),
        _ => None,
    }
}

/// What a rule is called when the village agrees it
pub fn bye_name(kind: ByeKind) -> &'static str {
    match kind {
        ByeKind::Commons => "stint the commons",
        ByeKind::Trade => "fair measure at market",
        ByeKind::Conduct => "keep the peace",
    }
}

/// Incidents of a kind before the village agrees a rule - the same
/// three voices that found an issue found a bye-law
pub const ADOPT_AT: u32 = 3;

/// A rule the village agreed, and how many times it's been broken since
#[derive(Debug, Clone)]
pub struct ByeLaw {
    pub kind: ByeKind,
    pub passed_on: u64,
    pub violations: u32,
}

/// The fine a broken bye-law draws: a share of the purse - community
/// justice bites shallower than the lord's
pub const BYE_FINE_SHARE: f32 = 0.1;
/// The least a fine can ask, so even a thin purse feels the rule
pub const BYE_FINE_MIN: f32 = 0.05;

/// Where a partnership stands in its life
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartnerStage {
    /// Fresh-sworn, still proving
    Forming,
    /// Doing well by each other
    Peak,
    /// The bond's cooled - the shared work falls away
    Decline,
    /// Broken - and what bad feeling it left behind
    Dissolved,
}

/// Neighbors bound to shared work: sworn on a strong bond over a common
/// grievance, it waxes while the bond holds and wanes as it sours; a
/// sour ending leaves rancor for the parish to settle
#[derive(Debug, Clone)]
pub struct Partnership {
    pub members: Vec<Entity>,
    pub formed_on: u64,
    /// 0..1 - tracks the bond between the members
    pub strength: f32,
    pub stage: PartnerStage,
    /// The bad feeling a sour break leaves (0 for a clean parting)
    pub rancor: f32,
    /// The dispute's been put to rest by the parish
    pub settled: bool,
}

/// The bond a partnership is sworn on
pub const PARTNER_MIN_BOND: f32 = 0.5;
/// Strength that counts as the partnership's prime
pub const PEAK_AT: f32 = 0.8;
/// Below this the compact's broken
pub const DISSOLVE_AT: f32 = 0.1;
/// The most partnerships a village will hold
pub const PARTNER_CAP: usize = 8;

/// A surety: `voucher` posted `bond` gold that `debtor` would make good
/// `owed`. The debtor pays it back in weekly installments; a fresh
/// offense while the word stands forfeits the bond to the debt, and
/// whatever's still owing lands on the voucher's own good name
#[derive(Debug, Clone)]
pub struct Vouch {
    pub voucher: Entity,
    pub debtor: Entity,
    /// The gold the voucher put up, held against the word
    pub bond: f32,
    /// What the debtor still owes the voucher
    pub owed: f32,
    pub made_on: u64,
}

/// Only kin and close friends can stand for someone
pub const VOUCH_MIN_BOND: f32 = 0.4;
/// Posted over the advance - the standing behind the gold
pub const BOND_SHARE: f32 = 0.25;
/// The weekly installment toward what's owed
pub const REPAY_SHARE: f32 = 0.2;
/// The most a debtor parts with in a week while repaying
pub const REPAY_CAP: f32 = 0.5;

/// The village's pull over the generations. Bands: below LOW the place
/// is only houses; at HIGH it speaks with one voice
pub const COMMUNITY_LOW: f32 = 0.25;
pub const COMMUNITY_HIGH: f32 = 0.7;
/// A surety only holds where the community's pull is real
pub const VOUCH_MIN_STRENGTH: f32 = COMMUNITY_LOW;
/// What the strength drifts back toward absent any pull
pub const COMMUNITY_BASELINE: f32 = 0.3;
/// The weekly pull toward the baseline
pub const COMMUNITY_DECAY: f32 = 0.02;
