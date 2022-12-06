use std::fs;

use day06::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(content, 14);

        println!("Top Containers: {}", result.unwrap());
    }
}
