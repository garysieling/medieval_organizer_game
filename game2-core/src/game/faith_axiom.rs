//! A faith's own shape: not a name but a vector over the axiom
//! dimensions - membership's mode, truth's stance, the orthodoxy-vs-
//! orthopraxy lean, the law's scope, the purity system, time's own
//! structure, the afterlife's shape, salvation's mechanism, the
//! authority that reads it, and how the faith stands to the state.
//! A few dims, a few values each - many religions from the one
//! substrate. The vector sits under sim/religion.rs - the faith's own
//! bones, not a replacement for the flesh of services and devotion.
//!
//! (wiki/software-design/faith-axiom-system.md)

/// A religion read as a vector over its axiom dimensions. Each field
/// holds a small index into that dimension's own scale - a few values
/// each, many faiths from the one shape. Coherence rules trim the
/// impossible combinations (a total law can't sit on no purity).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxiomVector {
    /// How one joins: 0 territorial (born to the ground's own faith),
    /// 1 initiated (rite or vow brings you in), 2 chosen (a soul's
    /// own profession)
    pub membership_mode: u8,
    /// Truth's own stance: 0 parallel_valid (many truths run
    /// together), 1 exclusive (one truth, the rest are error), 2
    /// syncretic (the others' truths fold in)
    pub truth_stance: u8,
    /// The lean: 0 orthopraxic (the practice makes the soul), 1
    /// orthodoxic (the belief makes the soul)
    pub orthopraxy_orthodoxy: u8,
    /// The law's own reach: 0 none (custom, not canon), 1 sacramental
    /// (the rites alone), 2 total (the law reaches every hour)
    pub law_scope: u8,
    /// The purity system's own strength: 0 none, 1 ritual (the washes
    /// and the fasts), 2 total (purity reads the body, the table, the
    /// marriage bed)
    pub purity_system: u8,
    /// Time's own structure: 0 cyclical (the wheel of the year), 1
    /// linear (a once-told story to an end), 2 eschatological (the
    /// end's own pull on every day)
    pub time_structure: u8,
    /// What waits past the grave: 0 ancestor_continuation (the dead
    /// stay near), 1 heaven_hell (the sorting), 2 reincarnation (the
    /// wheel), 3 dissolution (nothing waits)
    pub afterlife: u8,
    /// Salvation's own mechanism: 0 works (the doing saves), 1 grace
    /// (the gift saves), 2 knowledge (the knowing saves), 3 none (no
    /// saving needed)
    pub salvation_mechanism: u8,
    /// Who reads the law: 0 folk (the elder, the hearth), 1 clergy
    /// (the ordained), 2 scripture (the written word), 3 charismatic
    /// (the one the spirit marks)
    pub authority_structure: u8,
    /// The faith's own stance to the state: 0 separate, 1 subordinate
    /// (the faith serves), 2 established (the faith IS the state's)
    pub religion_state_relation: u8,
}

/// The fight's own signature, read off two vectors' collision -
/// emergent from the shapes, never a scripted event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionKind {
    /// exclusive x exclusive: two one-true-ways over the same ground -
    /// the contested shrine, and the war it builds
    ContestedShrine,
    /// orthodoxic vs a dissenter: the belief's own law reads the
    /// other shape for a heresy
    Heresy,
    /// a total law under a foreign state: the standing daily-rule
    /// conflict
    RuleConflict,
    /// a closed fold under contact: the boundary hardens to
    /// expulsion's edge
    BoundaryHardening,
}

/// How the faith reads a violation of its own law: the breach's own
/// shape by the faith's own lean
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// The orthopraxic read: the breach's a pollution - a cleansing
    /// rite, exclusion until the wash
    Cleansing,
    /// The orthodoxic read: the breach's a heresy - a recantation,
    /// a penance, a schism if the shape's structural
    Recantation,
}

impl AxiomVector {
    /// The coherence rules: the impossible combinations that can't
    /// stand as one faith. A total law can't sit on no purity; a faith
    /// that holds all truths can't burn the others for theirs.
    pub fn coherent(&self) -> bool {
        // a total law needs a purity to police
        if self.law_scope == 2 && self.purity_system == 0 { return false; }
        // a parallel-valid faith can't carry an exclusive claim's
        // own total reach
        if self.truth_stance == 0 && self.law_scope == 2 { return false; }
        // no afterlife means no salvation mechanism to speak of
        if self.afterlife == 3 && self.salvation_mechanism != 3 { return false; }
        // a folk authority can't enforce a total law
        if self.authority_structure == 0 && self.law_scope == 2 { return false; }
        true
    }

    /// The folk substrate: the spirit_world default every region
    /// starts on - territorial, parallel_valid, orthopraxic, ancestor
    /// continuation, a folk authority, no law to speak of
    pub fn folk_default() -> Self {
        Self {
            membership_mode: 0, truth_stance: 0, orthopraxy_orthodoxy: 0,
            law_scope: 0, purity_system: 1, time_structure: 0,
            afterlife: 0, salvation_mechanism: 3, authority_structure: 0,
            religion_state_relation: 0,
        }
    }

    /// The parish church's own vector: initiated, exclusive,
    /// orthodoxic, sacramental law, ritual purity, linear time, the
    /// sorting of heaven and hell, grace, the ordained authority,
    /// established to the state's own faith
    pub fn church() -> Self {
        Self {
            membership_mode: 1, truth_stance: 1, orthopraxy_orthodoxy: 1,
            law_scope: 1, purity_system: 1, time_structure: 1,
            afterlife: 1, salvation_mechanism: 1, authority_structure: 1,
            religion_state_relation: 2,
        }
    }

    /// The collision signatures: two faiths' own shapes meet and the
    /// fight's already read off them - not a scripted event, the
    /// vectors' own collision
    pub fn collision(&self, other: &Self) -> Option<CollisionKind> {
        // exclusive x exclusive: two one-true-ways over the same ground
        if self.truth_stance == 1 && other.truth_stance == 1 {
            return Some(CollisionKind::ContestedShrine);
        }
        // orthodoxic vs a different shape: the belief's own law
        // reads the dissenter for a heresy
        if self.orthopraxy_orthodoxy == 1 && other.orthopraxy_orthodoxy != self.orthopraxy_orthodoxy {
            return Some(CollisionKind::Heresy);
        }
        // a total law under a foreign state: the standing daily-rule
        // conflict - the faith's own law against the state's
        if self.law_scope == 2 && other.religion_state_relation == 2 {
            return Some(CollisionKind::RuleConflict);
        }
        // a closed fold (initiated/chosen, exclusive claim) under
        // contact: the boundary hardens
        if self.membership_mode > 0 && self.truth_stance == 1
            && other.truth_stance != 1 {
            return Some(CollisionKind::BoundaryHardening);
        }
        None
    }

    /// How a violation resolves, read off the faith's own lean: an
    /// orthopraxic faith reads the breach as a pollution to cleanse;
    /// an orthodoxic one reads it as a heresy to answer for
    pub fn resolution(&self) -> Resolution {
        if self.orthopraxy_orthodoxy == 1 { Resolution::Recantation }
        else { Resolution::Cleansing }
    }

    /// A prophet's own minting: a charismatic event in a high-tension
    /// ground calls a new faith out - chosen, exclusive, the spirit's
    /// own authority on one soul (the imagistic mode: the vision, not
    /// the canon)
    pub fn mint_prophet() -> Self {
        Self {
            membership_mode: 2, truth_stance: 1, orthopraxy_orthodoxy: 0,
            law_scope: 0, purity_system: 1, time_structure: 2,
            afterlife: 1, salvation_mechanism: 0, authority_structure: 3,
            religion_state_relation: 0,
        }
    }

    /// A schism's own shape: the child shares the parent's axioms
    /// except the one the split ran over - the schism IS the contested
    /// axiom (the authority's read, the law's reach, the truth's claim)
    pub fn schism(&self, contested: &str) -> Self {
        let mut child = *self;
        match contested {
            "authority" => child.authority_structure = 3, // the spirit over the ordained
            "law" => child.law_scope = (self.law_scope + 1).min(2), // the law tightens
            "truth" => child.truth_stance = (self.truth_stance + 1).min(2),
            _ => {}
        }
        child
    }

    /// The syncretic move: an accommodative faith folds a local cult
    /// in - the deity becomes a saint, a mask of the one god. The
    /// vector softens to carry the fold
    pub fn syncretize(&self) -> Self {
        let mut s = *self;
        s.truth_stance = 2;
        s
    }

    /// The overlay: a dominant faith imposed over the folk substrate
    /// doesn't erase it - the villagers run both, the church's rites
    /// over the elder's hearth-side ones. Returns the two faiths as
    /// they stand together
    pub fn overlay(&self, folk: &Self) -> (Self, Self) {
        (*self, *folk)
    }

    /// The old faith at the stones: territorial, syncretic, orthopraxic,
    /// no law but custom, ritual purity, the wheel of the year, the
    /// ancestors near, no saving needed, the elder's own authority,
    /// separate from the state
    pub fn old_faith() -> Self {
        Self {
            membership_mode: 0, truth_stance: 2, orthopraxy_orthodoxy: 0,
            law_scope: 0, purity_system: 1, time_structure: 0,
            afterlife: 0, salvation_mechanism: 3, authority_structure: 0,
            religion_state_relation: 0,
        }
    }
}
