use std::fs;

use day16::process_double;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_double(content, "AA", 26) {
            println!("{result}");
        }
    }
}
