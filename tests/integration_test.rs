pub fn add_two(a: u64) -> u64 {
    internal_adder(a, 2)
}

fn internal_adder(left: u64, right: u64) -> u64 {
    left + right
}


#[test]
fn it_adds_two() {
    let result = add_two(2);
    assert_eq!(result, 4);
}