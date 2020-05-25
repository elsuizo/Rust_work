use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn source_statistics(file_path: &str) -> Result<HashMap<char, u32>, std::io::Error> {
    // open and reading the file
    let file = File::open(file_path).expect("error opening the file");
    let reader = BufReader::new(file);
    // TODO(elsuizo:2019-09-07): pero no se si esta bien hacerlo asi porque me parece que se esta
    // evitando los posibles errores
    let lines: Vec<String> = reader.lines().flatten().collect(); // transform the file on Vector of String
    let mut probabilities: HashMap<char, u32> = HashMap::new();
    for line in lines {
        for c in line.chars() {
            *probabilities.entry(c).or_insert(0) += 1; // look if exist the char in the dictionary and add +1 to the count
        }
    }
    Ok(probabilities)
}

fn main() {
    let file_path = "/home/elsuizo/Repos/Rust_work/huffman/Files/castellano.txt";
    let file = BufReader::new(File::open(file_path).unwrap());
    let mut probs: HashMap<char, u32> = HashMap::new();
    if let Ok(probs) = source_statistics(file_path) {
        println!("{:?}", probs);
    }
}
