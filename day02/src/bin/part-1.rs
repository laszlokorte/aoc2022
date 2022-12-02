use std::fs;

use day02::process_move;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_move(content);

        println!("Score: {}", result);
    }
}
