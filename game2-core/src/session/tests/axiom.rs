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
