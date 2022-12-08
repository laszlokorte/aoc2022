use std::fs;

use day08::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(content);

        println!("Visible Trees: {}", result.unwrap().0);
    }
}
