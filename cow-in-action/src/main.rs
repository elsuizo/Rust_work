use std::path::PathBuf;
use std::borrow::Cow;

// NOTE(elsuizo:2020-11-22): de la definicion de `Cow`:
// enum Cow<'a, B: ?Sized + 'a>
//  where B: ToOwned
//  {
//      Borrowed(&'a B),
//      Owned(<B as ToOwned>::Owned),
//  }

// Vemos que le pasamos como parametro el lifetime de la referencia
//
enum Error {
    OutOfMemory,
    StackOverFlow,
    MachineOnFire,
    Unfathomable,
    FileNotFound(PathBuf)
}

fn describe(error: &Error) -> Cow<'static, str> {
    match *error {
        Error::OutOfMemory => "out of memory".into(),
        Error::StackOverFlow => "stack overflow".into(),
        Error::MachineOnFire => "machine on fire".into(),
        Error::Unfathomable => "machine bewildered".into(),
        Error::FileNotFound(ref path) => {
            format!("file not found: {}", path.display()).into()
        }
    }
}
fn main() {
    println!("Hello, world!");
}
