// NOTE(elsuizo:2020-05-13): como siempre me olvido como mierda se lee desde stdin aca dejo un
// template para que siempre me quede como recuerdo
// running: `cargo run --release < file_name.txt`
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    println!("txt read: {}", buffer);

    Ok(())
}
