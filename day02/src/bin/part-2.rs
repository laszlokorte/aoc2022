use std::fs;

use day02::process_goal;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_goal(content);

        println!("Score: {}", result);
    }
}
