use std::path::PathBuf;
use std::borrow::Cow;
use std::env;
use std::error::Error;
// NOTE(elsuizo:2020-11-22): de la definicion de `Cow`:
// enum Cow<'a, B: ?Sized + 'a>
//  where B: ToOwned
//  {
//      Borrowed(&'a B),
//      Owned(<B as ToOwned>::Owned),
//  }

// Vemos que le pasamos como parametro el lifetime de la referencia
//
enum FatalError {
    OutOfMemory,
    StackOverFlow,
    MachineOnFire,
    Unfathomable,
    FileNotFound(PathBuf)
}

fn describe(error: &FatalError) -> Cow<'static, str> {
    match *error {
        FatalError::OutOfMemory => "out of memory".into(),
        FatalError::StackOverFlow => "stack overflow".into(),
        FatalError::MachineOnFire => "machine on fire".into(),
        FatalError::Unfathomable => "machine bewildered".into(),
        FatalError::FileNotFound(ref path) => {
            format!("file not found: {}", path.display()).into()
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = env::current_dir()?;
    let e = FatalError::FileNotFound(path);
    println!("disaster has struck: {:}", describe(&e));
    Ok(())
}
