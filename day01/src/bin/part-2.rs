use std::fs;

use day01::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(text) = file {
        if let Ok(result) = process(text, 3) {
            print!("Sum of most three calories: {result}");
        }
    }
}
