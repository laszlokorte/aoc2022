use std::fs;

use day12::process;
use day12::{Marker, Policy, SearchTerm};

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(
            content,
            Policy::DOWNHILL,
            Marker::End,
            SearchTerm::Height(0),
        );

        println!("Visible Trees: {}", result.unwrap());
    }
}
