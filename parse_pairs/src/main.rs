use std::str::FromStr;
/// Parse the String `s` as coordinate pair like `"400x600"` o `"1.0,0.5"`
/// Specifically, `s` should have the form <left><sep><right> and <lef> and <right> are
/// both strings that can be parse by `T::from_str`
///
/// if `s` has the proper form, return `Some(<x,y>)`. If doesn't parse correctly return `None`
fn parse_pairs<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1 ..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _              => None
            }
        }
    }
}

fn main() {

}
