// NOTE(elsuizo:2020-05-27): la idea de esta funcion es que sea generica para cualquier "reader" y
// cualquier "writer" y lo que hace es copiar todos lo bytes de uno al otro

use std::io::{self, Read, Write, ErrorKind};

const DEFAULT_BUFFER_SIZE: usize = 8 * 1024;

fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where R: Read, W: Write
{
    let mut buffer = [0; DEFAULT_BUFFER_SIZE];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buffer) {
            Ok(0)      => return Ok(written),
            Ok(len)    => len,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) return Err(e),
        };
        writer.write_all(&buffer[..len])?;
        written += len as u64;
    }
}

fn main() {
    println!("Hello, world!");
}
