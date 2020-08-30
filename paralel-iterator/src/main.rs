extern crate rayon;

use std::fs;
use std::error::Error;
use std::path::Path;

use rayon::prelude::*;

fn main() -> Result<(), Box<dyn Error>> {

    let file_content = fs::read_to_string("../numbers.txt")?;

    let mut numbers = file_content
        .lines()
        .map(|n| n.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()?;

    // cuando usamos unstable para ordenar se refiere a que no se garantiza que
    // los elementos esten en el mismo orden siempre(o sea los mismos elementos)
    numbers.sort_unstable();

    let sum = numbers.iter().take(10).sum::<u32>();
    println!("{:?}", sum);

    Ok(())
}
