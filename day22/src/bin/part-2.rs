use std::fs;

use day22::process_3d;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_3d(content) {
            println!("{result}");
        }
    }
}
