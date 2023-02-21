use std::fs;

use day01::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(text) = file {
        if let Ok(result) = process(text, 1) {
            print!("Most calories: {result}");
        }
    }
}
