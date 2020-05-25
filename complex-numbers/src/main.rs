use num_traits::Float;
use std::fmt;

#[derive(Debug, Copy, Clone)]
struct Complex<T> {
    re: T,
    im: T,
}

impl<T> Complex<T> {
    fn new(re: T, im: T) -> Self {
        Self { re, im }
    }
}

impl<T: Float + std::cmp::PartialOrd + std::fmt::Display> fmt::Display for Complex<T> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let (r, i) = (self.re, self.im);
        if dest.alternate() {
            let abs = T::sqrt(r * r + i * i);
            let angle = T::atan2(i, r).to_degrees();
            write!(dest, "{} ∠ {}", abs, angle)
        } else {
            let i_sign = if i < T::zero() { '-' } else { '+' };
            write!(dest, "{}{}{}i", r, i_sign, T::abs(i))
        }
    }
}

fn main() {
    let c = Complex { re: 2.9, im: -9.9 };
    println!("c: {:#}", c);
    println!("c: {}", c);
}
