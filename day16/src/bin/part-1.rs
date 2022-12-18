use std::fs;

use day16::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process::<1>(content, "AA", 30) {
            println!("{result}");
        }
    }
}
