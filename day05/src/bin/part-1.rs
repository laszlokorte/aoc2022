use std::fs;

use day05::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(content, false);

        println!("Top Containers: {}", result.unwrap());
    }
}
