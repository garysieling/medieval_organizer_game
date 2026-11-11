#[test]
fn test_the_vintner_makes_hippocras() {
    let lux = crate::game::vocation::luxuries(crate::game::vocation::Vocation::Vintner);
    assert!(lux.contains(&"hippocras"), "the vintner makes no hippocras");
}

#[test]
fn test_the_glazier_makes_sight_and_glass() {
    let lux = crate::game::vocation::luxuries(crate::game::vocation::Vocation::Glazier);
    assert!(lux.contains(&"spectacles"), "the glazier makes no spectacles");
    assert!(lux.contains(&"mirror"), "the glazier makes no mirror");
}

#[test]
fn test_the_perfumer_makes_the_scent_luxuries() {
    let lux = crate::game::vocation::luxuries(crate::game::vocation::Vocation::Perfumer);
    assert!(lux.contains(&"rosewater"), "the perfumer makes no rosewater");
    assert!(lux.contains(&"pomander"), "the perfumer makes no pomander");
}

#[test]
fn test_the_printmaker_cuts_fine_prints() {
    let lux = crate::game::vocation::luxuries(crate::game::vocation::Vocation::Printmaker);
    assert!(lux.contains(&"prints"), "the printmaker cuts no fine prints");
}
