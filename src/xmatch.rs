use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufRead;

use crate::parse;

pub fn load_xmatch(file: &str) -> HashMap<i64, i32> {
    let mut map = HashMap::new();

    // Read plain text file
    let f = File::open(file).expect("Error: file not found");
    let mut total = 0;

    for line in io::BufReader::new(f).lines() {
        let l = line.expect("Error reading line");
        let tokens: Vec<&str> = l.split(',').collect();

        let source_id = parse::parse_i64(tokens.get(0));
        let hip = parse::parse_i32(tokens.get(1));

        map.insert(source_id, hip);

        total += 1;
    }
    println!("{} records loaded from {}", total, file);

    map
}
