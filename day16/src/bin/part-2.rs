use std::fs;

use day16::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process::<2>(content, "AA", 26) {
            println!("{result}");
        }
    }
}
