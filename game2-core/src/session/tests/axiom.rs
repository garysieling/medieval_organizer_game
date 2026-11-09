// the axiom substrate lives in game/ - no session items reach it

#[test]
fn test_the_named_vectors_stand_coherent() {
    // the folk substrate, the parish church, the old faith - each a
    // coherent vector under the model
    assert!(crate::game::faith_axiom::AxiomVector::folk_default().coherent());
    assert!(crate::game::faith_axiom::AxiomVector::church().coherent());
    assert!(crate::game::faith_axiom::AxiomVector::old_faith().coherent());
}

#[test]
fn test_the_impossible_combos_dont_stand() {
    // a total law can't sit on no purity - the coherence rule trims it
    let mut v = crate::game::faith_axiom::AxiomVector::church();
    v.law_scope = 2;
    v.purity_system = 0;
    assert!(!v.coherent(), "a total law stood on no purity");
    // no afterlife means no salvation to speak of
    let mut v2 = crate::game::faith_axiom::AxiomVector::church();
    v2.afterlife = 3;
    v2.salvation_mechanism = 1;
    assert!(!v2.coherent(), "a salvation stood on no afterlife");
}

#[test]
fn test_the_folk_defaults_territorial_and_orthopraxic() {
    let folk = crate::game::faith_axiom::AxiomVector::folk_default();
    assert_eq!(folk.membership_mode, 0, "the folk's not born-to-the-ground");
    assert_eq!(folk.truth_stance, 0, "the folk's not parallel-valid");
    assert_eq!(folk.orthopraxy_orthodoxy, 0, "the folk's not practice-first");
    assert_eq!(folk.afterlife, 0, "the folk's dead don't stay near");
}

#[test]
fn test_two_exclusive_faiths_meet_over_the_shrine() {
    // two one-true-ways over the same ground: the contested shrine
    let a = crate::game::faith_axiom::AxiomVector::church();
    let b = crate::game::faith_axiom::AxiomVector::church();
    assert_eq!(a.collision(&b), Some(crate::game::faith_axiom::CollisionKind::ContestedShrine),
        "exclusive x exclusive never read the shrine");
    // the parish reads the orthopraxic old faith for a heresy - the
    // belief's own law reading the dissenter
    assert_eq!(a.collision(&crate::game::faith_axiom::AxiomVector::old_faith()),
        Some(crate::game::faith_axiom::CollisionKind::Heresy),
        "the orthodoxic church never read the dissenter");
}

#[test]
fn test_the_lean_reads_the_violations_shape() {
    // the orthopraxic breach cleanses; the orthodoxic one answers for
    let orthoprax = crate::game::faith_axiom::AxiomVector::old_faith();
    let orthodox = crate::game::faith_axiom::AxiomVector::church();
    assert_eq!(orthoprax.resolution(), crate::game::faith_axiom::Resolution::Cleansing,
        "the practice-first faith read a heresy");
    assert_eq!(orthodox.resolution(), crate::game::faith_axiom::Resolution::Recantation,
        "the belief-first faith read a pollution");
}

#[test]
fn test_the_prophets_mint_stands_coherent() {
    // a charismatic event calls a faith out: chosen, exclusive, the
    // spirit's own authority - and it stands coherent
    let mint = crate::game::faith_axiom::AxiomVector::mint_prophet();
    assert!(mint.coherent(), "the prophet's mint didn't stand");
    assert_eq!(mint.membership_mode, 2, "the prophet's fold isn't chosen");
    assert_eq!(mint.authority_structure, 3, "the spirit's not the authority");
}

#[test]
fn test_a_schism_differs_on_the_contested_axiom() {
    // the child shares the parent's shape except the one it split over
    let parent = crate::game::faith_axiom::AxiomVector::church();
    let child = parent.schism("authority");
    assert_eq!(child.authority_structure, 3,
        "the schism never contested the authority");
    assert_eq!(child.truth_stance, parent.truth_stance,
        "the schism moved a second axiom");
    assert_eq!(child.law_scope, parent.law_scope,
        "the schism moved the law's reach too");
}

#[test]
fn test_the_accommodative_faith_softens_to_carry_the_cult() {
    // a faith folds a local cult in: the deity becomes a saint, the
    // stance softens to carry the fold
    let sync = crate::game::faith_axiom::AxiomVector::church().syncretize();
    assert_eq!(sync.truth_stance, 2, "the fold never softened");
}
