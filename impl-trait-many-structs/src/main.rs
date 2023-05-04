struct A {
    x: u32,
}

struct B {
    x: u32,
}

trait T {
    fn double(&self) -> u32;
}

macro_rules! impl_T {
    (for $($t:ty),+) => {
        $(impl T for $t {
            fn double(&self) -> u32 {
                self.x * 2
            }
        })*
    }
}

impl_T!(for A, B);

fn main() {
    let a = A { x: 2 };
    let b = B { x: 7 };
    println!("a: {:}", a.double());
    println!("b: {:}", b.double());
}
