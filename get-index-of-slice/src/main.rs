/// get
///

use std::cmp::PartialEq;


fn index<T: PartialEq>(slice: &[T], target: &T) -> Option<usize> {
    for (index, element) in slice.iter().enumerate() {
        if element == target {
            return Some(index);
        }
    }
    None
}

fn main() {
    let arr = [1, 2, 3, 4, 10, 37];
    match index(&arr, &37) {
        Some(value) => println!("el valor esta en el index: {:}", value),
        None        => println!("el valor no esta en el slice!!!"),
    };
}


