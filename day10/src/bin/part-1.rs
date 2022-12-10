use std::fs;

use day10::process_crt;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_crt(content);

        println!("Signal Strength: {}", result.unwrap().0);
    }
}
