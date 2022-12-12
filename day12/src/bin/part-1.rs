use std::fs;

use day12::process;
use day12::{Marker, Policy, SearchTerm};

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let result = process(
            content,
            Policy::UPHILL,
            Marker::Start,
            SearchTerm::Marker(Marker::End),
        );

        println!("Visible Trees: {}", result.unwrap());
    }
}
