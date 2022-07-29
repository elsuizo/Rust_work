// Esto lo saque del siguiente posteo del foro:
// https://users.rust-lang.org/t/is-it-possible-to-imply-copy-for-option-none/78892/3
// osea que vamos a poder inicializar estructuras de datos en tiempo de compilacion lo que se
// traduce en mejor performance
#![feature(inline_const)]
pub fn demo() -> [Option<String>; 100] {
    [const { None }; 100]
}

fn main() {
    let mut d = demo();
    d[1] = Some("piola".to_string());
    dbg!(d);
}
