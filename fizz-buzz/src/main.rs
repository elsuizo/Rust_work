// NOTE(elsuizo:2020-05-13): imprime el famoso ejemplo de fizzbuzz pero con iteradores

use std::iter::{once, repeat};

fn main() {
    let fizzes = repeat("").take(2).chain(once("fizz")).cycle();
    let buzzes = repeat("").take(4).chain(once("buzz")).cycle();

    let fizzes_buzzes = fizzes.zip(buzzes);

    let fizz_buzz = (1..100).zip(fizzes_buzzes).map(|tuple|
        match tuple {
            (i, ("", "")) => i.to_string(),
            (_, (fizz, buzz)) => format!("{}{}", fizz, buzz)
        }
    );

    for line in fizz_buzz {
        println!("{}", line);
    }
}
