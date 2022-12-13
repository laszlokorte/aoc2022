#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list0;
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, separated_pair};
use nom::*;

#[derive(Debug, Eq, PartialEq, Clone)]
enum Signal {
    Value(u32),
    Nested(Vec<Signal>),
}

impl PartialOrd for Signal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Signal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use Signal::*;
        match (self, other) {
            (Value(a), Value(b)) => a.cmp(b),
            (Nested(a), Nested(b)) => a.cmp(b),
            (a, b @ Nested(_)) => Nested(vec![a.clone()]).cmp(b),
            (a @ Nested(_), b) => a.cmp(&Nested(vec![b.clone()])),
        }
    }
}

fn list(input: &str) -> IResult<&str, Signal> {
    delimited(
        tag("["),
        separated_list0(
            tag(","),
            alt((character::complete::u32.map(|v| Signal::Value(v)), list)),
        ),
        tag("]"),
    )(input)
    .map(|(s, l)| (s, Signal::Nested(l)))
}

fn pairs(input: &str) -> IResult<&str, Vec<(Signal, Signal)>> {
    separated_list1(
        pair(line_ending, line_ending),
        separated_pair(list, line_ending, list),
    )(input)
}

pub fn process(input: String) -> Option<usize> {
    let (_, signal_pairs) = pairs(&input).ok()?;
    Some(
        signal_pairs
            .iter()
            .enumerate()
            .filter_map(|(index, (p1, p2))| if p1 <= p2 { Some(index + 1) } else { None })
            .sum(),
    )
}

pub fn process_sort(input: String) -> Option<usize> {
    let (_, signal_pairs) = pairs(&input).ok()?;
    let mut all_signals = signal_pairs
        .iter()
        .flat_map(|(a, b)| [a, b])
        .collect::<Vec<&Signal>>();
    let deviders = [
        Signal::Nested(vec![Signal::Nested(vec![Signal::Value(2)])]),
        Signal::Nested(vec![Signal::Nested(vec![Signal::Value(6)])]),
    ];
    for dev in &deviders {
        all_signals.push(&dev);
    }
    all_signals.sort();

    deviders
        .iter()
        .map(|d| all_signals.iter().position(|s| s == &d).map(|i| i + 1))
        .product::<Option<usize>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "\
        [1,1,3,1,1]\n\
        [1,1,5,1,1]\n\
        \n\
        [[1],[2,3,4]]\n\
        [[1],4]\n\
        \n\
        [9]\n\
        [[8,7,6]]\n\
        \n\
        [[4,4],4,4]\n\
        [[4,4],4,4,4]\n\
        \n\
        [7,7,7,7]\n\
        [7,7,7]\n\
        \n\
        []\n\
        [3]\n\
        \n\
        [[[]]]\n\
        [[]]\n\
        \n\
        [1,[2,[3,[4,[5,6,7]]]],8,9]\n\
        [1,[2,[3,[4,[5,6,0]]]],8,9]";

        assert_eq!(process(COMMANDS.to_string()), Some(13));
        assert_eq!(process_sort(COMMANDS.to_string()), Some(140));
    }
}
