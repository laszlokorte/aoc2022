use std::fs;

use day04::process;
use day04::Overlap;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(content, Overlap::Partial);

        println!("Sum: {}", result.unwrap());
    }
}
