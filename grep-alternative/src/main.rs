use std::io;
// NOTE(elsuizo:2020-05-27): esto no lo hace solo???
use std::io::prelude::*;

/// Esta funcion solo acepta a stdin como Reader
fn grep(target: &str) -> io::Result<()> {
    let stdin = io::stdin();
    // aca el lock es porque estamos obteniendo un Mutex
    for line_result in stdin.lock().lines() {
        let line = line_result?;
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok(())
}

/// Esta funcion es generica sobre el reader y lo utiliza para tener el metodo del trait BufRead
/// `lines()`
fn grep_generic<R>(target: &str, reader: R) -> Result<()>
where R: BufRead
{
    for line_result in reader.lines() {
        let line = line_result?;
        if line.contains(target) {
            println!("{}", line);
        }
    }
    Ok()
}

fn main() {
    let stdin = io::stdin();
    grep_generic(&target, stdin.lock())?;

    // como File no implementa automaticamente BufRead, solo implementa Read. Sin embargo podemos
    // crear un buffered reader para un file o cualquier otro unbuffered reader con
    // BufReader::new(reader)
    let f = File::open(file_path)?;

    grep_generic(&target, BufReader::new(f))?;
}
