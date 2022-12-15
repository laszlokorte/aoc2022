use std::fs;

use day15::process_search;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_search(content, 4000000) {
            println!("Tuning frequency: {result}");
        }
    }
}
