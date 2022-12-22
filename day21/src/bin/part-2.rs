use std::fs;

use day21::process_solve;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_solve(content) {
            println!("human value: {result}");
        }
    }
}
