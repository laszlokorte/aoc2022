#![feature(iter_array_chunks)]

use std::ops::RangeInclusive;

#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub enum Overlap {
    Partial,
    Fully,
}

fn overlapping(a: &RangeInclusive<u32>, b: &RangeInclusive<u32>) -> Option<Overlap> {
    let left_b_in_a = a.contains(&b.clone().min()?);
    let right_b_in_a = a.contains(&b.clone().max()?);
    let left_a_in_b = b.contains(&a.clone().min()?);
    let right_a_in_b = b.contains(&a.clone().max()?);

    if (left_b_in_a && right_b_in_a) || (left_a_in_b && right_a_in_b) {
        Some(Overlap::Fully)
    } else if left_b_in_a || right_b_in_a || left_a_in_b || right_a_in_b {
        Some(Overlap::Partial)
    } else {
        None
    }
}

fn parse_range(string: &str) -> Option<RangeInclusive<u32>> {
    let (start_str, end_str) = string.split_once('-')?;
    let s = str::parse::<u32>(start_str).ok();
    let e = str::parse::<u32>(end_str).ok();

    Some(s?..=e?)
}

pub fn process(input: String, test_overlap: Overlap) -> Option<u32> {
    input
        .lines()
        .map(|line: &str| -> Option<u32> {
            let (left, right) = line.split_once(',')?;

            let range_a = parse_range(left)?;
            let range_b = parse_range(right)?;

            if overlapping(&range_a, &range_b) >= Some(test_overlap) {
                Some(1)
            } else {
                Some(0)
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVES: &str = include_str!("test.txt");

    #[test]
    fn test_process_fully() {
        assert_eq!(process(MOVES.to_string(), Overlap::Fully), Some(2));
    }

    #[test]
    fn test_process_partially() {
        assert_eq!(process(MOVES.to_string(), Overlap::Partial), Some(4));
    }
}
