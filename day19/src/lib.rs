#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]
use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    character::complete::multispace1,
    combinator::value,
    multi::separated_list1,
    sequence::separated_pair,
    sequence::{delimited, pair, terminated},
    *,
};
use rayon::prelude::*;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

#[derive(Clone, Debug)]
struct Cost {
    ingredients: HashMap<Resource, u32>,
}

#[derive(Debug)]
struct Blueprint {
    id: u32,
    robots: HashMap<Resource, Cost>,
}

fn resource(input: &str) -> IResult<&str, Resource> {
    nom::branch::alt((
        value(Resource::Ore, tag("ore")),
        value(Resource::Clay, tag("clay")),
        value(Resource::Obsidian, tag("obsidian")),
        value(Resource::Geode, tag("geode")),
    ))(input)
}

fn recipe(input: &str) -> IResult<&str, (Resource, Cost)> {
    let (input, (robot, ingrediens)) = pair(
        delimited(tag("Each "), resource, tag(" robot costs ")),
        terminated(
            separated_list1(
                tag(" and "),
                separated_pair(character::complete::u32, tag(" "), resource),
            ),
            tag("."),
        ),
    )(input)?;

    Ok((
        input,
        (
            robot,
            Cost {
                ingredients: ingrediens.into_iter().map(|(num, r)| (r, num)).collect(),
            },
        ),
    ))
}

fn blueprint(input: &str) -> IResult<&str, Blueprint> {
    let (input, (id, recipes)) = separated_pair(
        delimited(tag("Blueprint "), character::complete::u32, tag(":")),
        multispace0,
        separated_list1(multispace1, recipe),
    )(input)?;

    Ok((
        input,
        Blueprint {
            id,
            robots: recipes.into_iter().collect(),
        },
    ))
}

fn blueprints(input: &str) -> IResult<&str, Vec<Blueprint>> {
    separated_list1(multispace1, blueprint)(input)
}

#[derive(Debug)]
struct Rule<const SIZE: usize> {
    target: usize,
    costs: [[u32; SIZE]; SIZE],
    max_utility: [u32; SIZE],
}

impl<'bp> std::convert::From<&'bp Blueprint> for Rule<4> {
    fn from(value: &'bp Blueprint) -> Self {
        let mapping = [
            Resource::Ore,
            Resource::Clay,
            Resource::Obsidian,
            Resource::Geode,
        ];
        Rule {
            target: 3,
            costs: mapping.map(|r| {
                mapping.map(|c| {
                    value.robots
                        .get(&r)
                        .and_then(|cc| cc.ingredients.get(&c))
                        .cloned()
                        .unwrap_or_default()
                })
            }),
            max_utility: mapping.map(|r| {
                value.robots
                    .values()
                    .map(|costs| costs.ingredients.get(&r).unwrap_or(&0))
                    .max()
                    .cloned()
                    .unwrap_or(0)
            }),
        }
    }
}

impl<const SIZE: usize> Rule<SIZE> {
    fn can_buy(&self, state: &State<SIZE>, resource_index: usize) -> bool {
        (0..SIZE).all(|r| state.resources[r] >= self.costs[resource_index][r])
    }

    fn buying_to_late(&self, state: &State<SIZE>, resource_index: usize) -> bool {
        state.rejected_to_buy[resource_index]
    }

    fn is_useful(&self, state: &State<SIZE>, robot_index: usize) -> bool {
        state.robots[robot_index] < self.max_utility[robot_index]
    }

    fn do_nothing(&self, state: &State<SIZE>) -> State<SIZE> {
        let mut new_state = *state;
        new_state.time_left -= 1;

        for r in 0..SIZE {
            new_state.rejected_to_buy[r] = self.can_buy(state, r);
            new_state.resources[r] += new_state.robots[r];
        }
        new_state
    }

    fn buy_robot(&self, state: &State<SIZE>, robot_index: usize) -> State<SIZE> {
        let mut new_state = *state;
        new_state.time_left -= 1;
        new_state.rejected_to_buy = [false; SIZE];
        for r in 0..SIZE {
            new_state.resources[r] -= self.costs[robot_index][r];
            new_state.resources[r] += new_state.robots[r];
        }
        new_state.robots[robot_index] += 1;
        new_state
    }
}
#[derive(Copy, Clone)]
struct State<const SIZE: usize> {
    resources: [u32; SIZE],
    robots: [u32; SIZE],
    rejected_to_buy: [bool; SIZE],
    time_left: u32,
}
impl<const SIZE: usize> State<SIZE> {
    fn new(time_left: u32) -> Self {
        Self {
            resources: [0; SIZE],
            robots: [0; SIZE],
            rejected_to_buy: [false; SIZE],
            time_left,
        }
    }
}

fn optimize<const SIZE: usize>(rules: &Rule<SIZE>, mut initial: State<SIZE>) -> u32 {
    let mut queue = VecDeque::new();
    initial.robots[0] = 1;
    queue.push_back(initial);
    let mut best = 0;
    while let Some(current) = queue.pop_front() {
        if current.time_left == 0 {
            best = best.max(current.resources[rules.target]);
            continue;
        }
        if rules.can_buy(&current, rules.target) {
            queue.push_back(rules.buy_robot(&current, rules.target));
        } else {
            for r in 0..SIZE {
                if rules.is_useful(&current, r)
                    && rules.can_buy(&current, r)
                    && !rules.buying_to_late(&current, r)
                {
                    queue.push_back(rules.buy_robot(&current, r));
                }
            }
            queue.push_back(rules.do_nothing(&current));
        }
    }
    best
}
pub fn process(input: String, minutes: u32) -> Option<u32> {
    let (_, blues) = blueprints(&input).ok()?;

    Some(
        blues
            .par_iter()
            .map(|bp| bp.id * optimize(&bp.into(), State::new(minutes)))
            .sum(),
    )
}

pub fn process_part2(input: String, minutes: u32) -> Option<u32> {
    let (_, blues) = blueprints(&input).ok()?;

    Some(
        blues
            .par_iter()
            .take(3)
            .map(|bp| optimize(&bp.into(), State::new(minutes)))
            .product(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process(COMMANDS.to_string(), 24), Some(33));
        // assert_eq!(process_part2(COMMANDS.to_string(), 32), Some(3472));
    }
}
