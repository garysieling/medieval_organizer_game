#[test]
fn test_the_bookbinder_binds_vellum_and_ink() {
    let inputs = crate::game::vocation::inputs(crate::game::vocation::Vocation::Bookbinder);
    let output = crate::game::vocation::output(crate::game::vocation::Vocation::Bookbinder);
    assert!(inputs.contains(&"vellum"), "the bookbinder takes no vellum");
    assert!(inputs.contains(&"ink"), "the bookbinder takes no ink");
    assert_eq!(output, Some("books"), "the bookbinder makes no books");
}

#[test]
fn test_the_bindery_and_the_name() {
    let name = crate::game::vocation::vocation_name(crate::game::vocation::Vocation::Bookbinder);
    assert_eq!(name, "Bookbinder", "the bookbinder's not named: {}", name);
    let shop = crate::game::vocation::trade_name(crate::game::vocation::Vocation::Bookbinder);
    assert_eq!(shop, "the bindery", "the bindery's not housed: {}", shop);
}
