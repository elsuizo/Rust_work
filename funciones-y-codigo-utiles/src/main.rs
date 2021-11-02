use std::fs::{self, File};
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

// NOTE(elsuizo:2021-11-02): esta funcion lee los archivos que hay en un dado directorio
// y los pone en un Vec de Results
fn read_dir<P: AsRef<Path>>(directory: P) -> IoResult<Vec<PathBuf>> {
    fs::read_dir(directory)?
        .map(|res_entry| res_entry.map(|entry| entry.path()))
        .collect()
}

fn main() {
    println!("Hello, world!");
}
