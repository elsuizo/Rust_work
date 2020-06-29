// NOTE(elsuizo:2020-06-28): este ejemplo lo saque del foro de Rust:
// https://users.rust-lang.org/t/check-symmetry-in-list/45077/2
// y es super cool y no sabia que se podia hacer algo asi
fn is_symmetric(list: &[u32]) -> bool {
    match list {
        [] | [_] => true,
        [x, inside @ .., y] if x == y => is_symmetric(inside),
        _ => false
    }
}

fn main() {
    let sym = &[0, 1, 4, 2, 4, 1, 0];
    assert!(is_symmetric(sym));

    let not_sym = &[0, 1, 7, 2, 4, 1, 0];
    assert!(!is_symmetric(not_sym));
}
