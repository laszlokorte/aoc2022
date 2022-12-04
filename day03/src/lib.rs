#![feature(iter_array_chunks)]

use std::collections::HashSet;

fn char_to_number(char: char) -> u32 {
    if char.is_lowercase() {
        char as u32 - 'a' as u32 + 1
    } else if char.is_uppercase() {
        char as u32 - 'A' as u32 + 1 + 26
    } else {
        0
    }
}

pub fn process(input: String) -> Option<u32> {
    let lines = input.lines();

    lines
        .into_iter()
        .map(|l| {
            let (left, right) = l.split_at(l.len() / 2);
            let right_set: HashSet<char> = HashSet::from_iter(right.chars());
            let left_set: HashSet<char> = HashSet::from_iter(left.chars());

            let intersection = left_set.intersection(&right_set).copied();

            let duplication = intersection.into_iter().next();

            duplication.map(char_to_number)
        })
        .sum()
}

pub fn process_groups(input: String) -> Option<u32> {
    let lines = input.lines();

    lines
        .into_iter()
        .array_chunks::<3>()
        .map(|chunks| {
            chunks
                .iter()
                .map(|l| HashSet::from_iter(l.chars()))
                .reduce(|l: std::collections::HashSet<char>, r| {
                    l.intersection(&r).copied().collect()
                })
                .and_then(|h| h.into_iter().next())
                .map(char_to_number)
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVES: &str = "\
        vJrwpWtwJgWrhcsFMMfFFhFp\n\
        jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL\n\
        PmmdzqPrVvPwwTWBwg\n\
        wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn\n\
        ttgJtRGJQctTZtZT\n\
        CrZsJsPPZsGzwwsLwLmpwMDw
        ";

    #[test]
    fn test_process() {
        assert_eq!(process(MOVES.to_string()), Some(157));
    }

    #[test]
    fn test_badges() {
        assert_eq!(process_groups(MOVES.to_string()), Some(70));
    }
}
