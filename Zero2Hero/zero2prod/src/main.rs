use std::error::Error;
use std::net::TcpListener;
use zero2prod::run;

// #[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    let _server = run(TcpListener::bind("127.0.0.1:0")?)?;

    Ok(())
}
