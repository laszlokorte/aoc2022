use std::fs;

use day23::process;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some((last_round, _)) = process(content, None) {
            println!("{last_round}");
        }
    }
}
