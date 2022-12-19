use std::fs;

use day19::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process(content, 24) {
            println!("{result}");
        }
    }
}
