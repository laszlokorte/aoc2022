use std::fs;

use day16::process_single;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_single(content, "AA", 30) {
            println!("{result}");
        }
    }
}
