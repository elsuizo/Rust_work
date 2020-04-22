/// Esto es un ejemplo del libro Programming Rust, que adapte para que sea generico
#[derive(Copy, Clone, Debug)]
struct Extrema<T> {
    greatest: T,
    least:     T,
}

/// funcion generica que encuentra los valores extremos de un slice cualquiera del type T que impl PartialOrd
/// y Copy
fn find_extrema<T: std::cmp::PartialOrd + Copy>(slice: &[T]) -> Extrema<T> {
    let mut greatest = &slice[0];
    let mut least    = &slice[0];

    for index in 1..slice.len() {
        if slice[index] < *least { least = &slice[index];}
        if slice[index] > *greatest { greatest = &slice[index];}
    }
    Extrema{greatest: *greatest, least: *least}
}

fn main() {
    let v = vec!["piola", "kasdjflak", "alsda"];
    let e = find_extrema(&v);

    println!("extremos: {:?}", e);
}
