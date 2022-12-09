use std::fs;

use day09::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(content, 1);

        println!("Visited Fields: {}", result.unwrap());
    }
}
