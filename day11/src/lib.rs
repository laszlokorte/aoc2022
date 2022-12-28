#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::combinator::value;
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::*;
use std::collections::BinaryHeap;
use std::collections::VecDeque;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum Operation {
    Multiply(u64),
    Add(u64),
    Square,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
struct Test {
    divisible: u64,
    true_target: usize,
    false_target: usize,
}

#[derive(Eq, PartialEq, Debug)]
struct Monkey {
    name: u64,
    items: VecDeque<u64>,
    operation: Operation,
    test: Test,
    inspection_count: u64,
}

impl Operation {
    fn apply(&self, number: u64) -> u64 {
        match self {
            Self::Multiply(factor) => number * factor,
            Self::Square => number * number,
            Self::Add(sum) => number + sum,
        }
    }
}

impl Test {
    fn decide(&self, number: u64) -> usize {
        if number % self.divisible == 0 {
            self.true_target
        } else {
            self.false_target
        }
    }
}

fn items(input: &str) -> IResult<&str, VecDeque<u64>> {
    let (input, numbers) = preceded(
        pair(line_ending, tag("  Starting items: ")),
        separated_list1(tag(", "), character::complete::u64),
    )(input)?;

    Ok((input, VecDeque::from(numbers)))
}

fn operation(input: &str) -> IResult<&str, Operation> {
    preceded(
        pair(line_ending, tag("  Operation: new = old ")),
        alt((
            value(Operation::Square, tag("* old")),
            preceded(tag("+ "), character::complete::u64).map(Operation::Add),
            preceded(tag("* "), character::complete::u64).map(Operation::Multiply),
        )),
    )(input)
}

fn test(input: &str) -> IResult<&str, Test> {
    let (input, divisible) = preceded(
        pair(line_ending, tag("  Test: divisible by ")),
        character::complete::u64,
    )(input)?;
    let (input, true_target) = preceded(
        pair(line_ending, tag("    If true: throw to monkey ")),
        character::complete::u64,
    )(input)?;
    let (input, false_target) = preceded(
        pair(line_ending, tag("    If false: throw to monkey ")),
        character::complete::u64,
    )(input)?;

    Ok((
        input,
        Test {
            divisible,
            true_target: true_target as usize,
            false_target: false_target as usize,
        },
    ))
}

fn monkey(input: &str) -> IResult<&str, Monkey> {
    let (input, name) = preceded(tag("Monkey "), character::complete::u64)(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, items) = items(input)?;
    let (input, op) = operation(input)?;
    let (input, test) = test(input)?;

    Ok((
        input,
        Monkey {
            name,
            items,
            operation: op,
            test,
            inspection_count: 0,
        },
    ))
}

fn monkeys(input: &str) -> IResult<&str, Vec<Monkey>> {
    let (input, mks) = separated_list1(pair(line_ending, line_ending), monkey)(input)?;
    Ok((input, mks))
}

pub fn process(input: String, rounds: u64, worried: bool) -> Option<u64> {
    let (_, mut mnks) = monkeys(&input).ok()?;
    let lowest_common_denominator: u64 = mnks.iter().map(|m| m.test.divisible).product();

    for _ in 0..rounds {
        // dbg!(round);
        for monkey_number in 0..mnks.len() {
            // dbg!(&monkey.items);
            while let Some(item) = mnks[monkey_number].items.pop_front() {
                // dbg!(item);
                let changed = mnks[monkey_number].operation.apply(item);
                mnks[monkey_number].inspection_count += 1;
                let calmed = if worried {
                    changed % lowest_common_denominator
                } else {
                    changed / 3
                };

                let new_monkey = mnks[monkey_number].test.decide(calmed);
                mnks[new_monkey].items.push_back(calmed);
            }
        }
    }

    let mut inspection_counts = mnks
        .iter()
        .map(|m| m.inspection_count)
        .collect::<BinaryHeap<_>>();
    Some(inspection_counts.pop()? * inspection_counts.pop()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process(COMMANDS.to_string(), 20, false), Some(10605));
        assert_eq!(process(COMMANDS.to_string(), 10000, true), Some(2713310158));
    }
}
