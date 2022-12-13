use std::fs;

use day13::process_sort;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_sort(content);

        println!("{}", result.unwrap());
    }
}
