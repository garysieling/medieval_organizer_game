#[test]
fn test_the_cobbler_turns_leather_to_shoes() {
    let inputs = crate::game::vocation::inputs(crate::game::vocation::Vocation::Shoemaker);
    let output = crate::game::vocation::output(crate::game::vocation::Vocation::Shoemaker);
    assert!(inputs.contains(&"leather"), "the cobbler takes no leather");
    assert_eq!(output, Some("shoes"), "the cobbler makes no shoes");
}

#[test]
fn test_the_cobblers_own_place() {
    let name = crate::game::vocation::vocation_name(crate::game::vocation::Vocation::Shoemaker);
    assert_eq!(name, "Cobbler", "the cobbler's not named: {}", name);
    let shop = crate::game::vocation::trade_name(crate::game::vocation::Vocation::Shoemaker);
    assert_eq!(shop, "the cobbler's", "the cobbler's not housed: {}", shop);
}
