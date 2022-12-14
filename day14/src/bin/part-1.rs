use std::fs;

use day14::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some((result, display)) = process(content, false) {
            println!("Steps: {}", result);
            println!("{display}");
        }
    }
}
