use std::fs;

use day22::process_with_portals;

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        if let Some(result) = process_with_portals(content) {
            println!("result: {result}");
        }
    }
}
