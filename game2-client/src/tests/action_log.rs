use crate::*;

#[test]
fn test_influence_lines_are_not_coloured_as_illness() {
    // "influence" contains "flu"; it used to turn every influence line weather-cyan
    let illness = (120, 200, 255);
    assert_ne!(log_colour("t3: Invested 10 gold"), illness);
    assert_ne!(log_colour("t3: Can't recruit: need influence 0.40."), illness);
    assert_ne!(log_colour("t9: the grain store is low"), illness);
    assert_eq!(log_colour("t120: Bram is taken with la grippe"), illness);
    assert_eq!(log_colour("t200: it's raining"), illness);
}
