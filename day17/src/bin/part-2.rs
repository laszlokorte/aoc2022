use std::fs;

use day17::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process(content, 1000000000000) {
            println!("{result}");
        }
    }
}
