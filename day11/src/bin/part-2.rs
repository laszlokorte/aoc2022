use std::fs;

use day11::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(content, 10000, true);

        println!("{}", result.unwrap());
    }
}
