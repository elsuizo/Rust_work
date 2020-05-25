// NOTE(elsuizo:2020-05-12): este es un ejemplo loco que esta en el libro: Programming Rust: pag:
// 530
use std::iter::Peekable;

fn parse_number<I>(tokens: &mut Peekable<I>) -> u32
where I: Iterator<Item=char>
{
    let mut n = 0;
    loop {
        match tokens.peek() {
            Some(r) if r.is_digit(10) => {
                n = n * 10 + r.to_digit(10).unwrap();
            }
            _ => return n
        }
        tokens.next();
    }

}

fn main() {
    let mut chars = "73737337,23123123".chars().peekable();
    let result = parse_number(&mut chars);
    println!("result: {:}", result);
    chars.next();
    let result = parse_number(&mut chars);
    println!("result: {:}", result);
}
