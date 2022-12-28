#![feature(binary_heap_into_iter_sorted)]

use std::{collections::BinaryHeap, num::ParseIntError};

/// Expects the text to be composed of paragraphs that are each composed of lines containing a single
/// integer value.
/// returns the sum of the top_num paragraphs with the largest numeric sum.
pub fn process(text: String, top_num: usize) -> Result<u32, ParseIntError> {
    let chunks = text.split("\n\n");

    chunks
        .map(|chunk| chunk.lines().map(str::parse::<u32>).sum())
        .collect::<Result<BinaryHeap<u32>, _>>()
        .map(|h| h.into_iter_sorted().take(top_num).sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = include_str!("test.txt");
    
    #[test]
    fn test() {
        assert_eq!(process(INPUT.to_owned(), 1), Ok(24000))
    }
}
