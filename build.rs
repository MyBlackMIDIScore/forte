use std::fs::File;
use std::io::prelude::*;

// Increase build number

fn main() {
    let current = include!("build.number");
    let mut file = File::create("build.number").unwrap();
    file.write_all(format!("{}", current + 1).as_bytes())
        .unwrap();
}
