use std::fs;

use day03::process_groups;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_groups(content);

        println!("Sum: {}", result.unwrap());
    }
}
