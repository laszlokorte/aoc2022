use std::fs;

use day19::process_part2;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_part2(content, 32) {
            println!("{result}");
        }
    }
}
