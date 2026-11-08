
#[test]
fn test_the_windows_keep_their_own_days() {
    // ague unchanged: the autumn marsh window's still its own
    assert!(crate::sim::illness::DiseaseKind::Ague.in_season(270));
    assert!(!crate::sim::illness::DiseaseKind::Ague.in_season(120));
    // winter wraps the year: consumption's damp rides both sides
    assert!(crate::sim::illness::DiseaseKind::Consumption.in_season(330));
    assert!(crate::sim::illness::DiseaseKind::Consumption.in_season(30));
    assert!(!crate::sim::illness::DiseaseKind::Consumption.in_season(150));
    // the foul-water fevers keep summer and autumn
    assert!(crate::sim::illness::DiseaseKind::Cholera.in_season(200));
    assert!(!crate::sim::illness::DiseaseKind::Cholera.in_season(100));
    assert!(crate::sim::illness::DiseaseKind::Typhoid.in_season(270));
    // the sweats only come in their late-summer heat
    assert!(crate::sim::illness::DiseaseKind::Sweats.in_season(250));
    assert!(!crate::sim::illness::DiseaseKind::Sweats.in_season(50));
    // the poisonings hold no season - they come whenever they come
    assert!(crate::sim::illness::DiseaseKind::Gripes.in_season(150));
    assert!(crate::sim::illness::DiseaseKind::Ergotism.in_season(300));
}
