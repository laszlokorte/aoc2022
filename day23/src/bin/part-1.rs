use std::fs;

use day23::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some((_, result)) = process(content, Some(10)) {
            println!("{result}");
        }
    }
}
