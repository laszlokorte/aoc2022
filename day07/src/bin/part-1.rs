use std::fs;

use day07::process_sum;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_sum(content, 100000);

        println!("Sum of folder sizes: {}", result.unwrap());
    }
}
