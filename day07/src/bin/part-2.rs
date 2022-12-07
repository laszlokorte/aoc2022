use std::fs;

use day07::process_deletion;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process_deletion(content, 70000000, 30000000);

        println!("Folder size to delete: {}", result.unwrap());
    }
}
