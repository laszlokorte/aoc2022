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
struct Rule {
    target: usize,
    costs: [[u32; 4]; 4],
    max_utility: [u32; 4],
}

impl From<&Blueprint> for Rule {
    fn from(value: &Blueprint) -> Self {
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
                    value
                        .robots
                        .get(&r)
                        .and_then(|cc| cc.ingredients.get(&c))
                        .cloned()
                        .unwrap_or_default()
                })
            }),
            max_utility: mapping.map(|r| {
                value
                    .robots
                    .values()
                    .map(|costs| costs.ingredients.get(&r).unwrap_or(&0))
                    .max()
                    .cloned()
                    .unwrap_or(0)
            }),
        }
    }
}

impl Rule {
    fn can_buy(&self, state: &State, resource_index: usize) -> bool {
        state.resources[0] >= self.costs[resource_index][0]
            && state.resources[1] >= self.costs[resource_index][1]
            && state.resources[2] >= self.costs[resource_index][2]
            && state.resources[3] >= self.costs[resource_index][3]
    }

    fn buying_to_late(&self, state: &State, resource_index: usize) -> bool {
        state.rejected_to_buy[resource_index]
    }

    fn is_useful(&self, state: &State, robot_index: usize) -> bool {
        state.robots[robot_index] < self.max_utility[robot_index]
    }

    fn do_nothing(&self, state: &State) -> State {
        let mut new_state = *state;
        new_state.time_left -= 1;
        new_state.rejected_to_buy[0] = self.can_buy(&new_state, 0);
        new_state.rejected_to_buy[1] = self.can_buy(&new_state, 1);
        new_state.rejected_to_buy[2] = self.can_buy(&new_state, 2);
        new_state.rejected_to_buy[3] = self.can_buy(&new_state, 3);

        new_state.resources[0] += new_state.robots[0];
        new_state.resources[1] += new_state.robots[1];
        new_state.resources[2] += new_state.robots[2];
        new_state.resources[3] += new_state.robots[3];
        new_state
    }

    fn buy_robot(&self, state: &State, robot_index: usize) -> State {
        let mut new_state = *state;
        new_state.time_left -= 1;
        new_state.rejected_to_buy = [false; 4];
        new_state.resources[0] -= self.costs[robot_index][0];
        new_state.resources[1] -= self.costs[robot_index][1];
        new_state.resources[2] -= self.costs[robot_index][2];
        new_state.resources[3] -= self.costs[robot_index][3];
        new_state.resources[0] += new_state.robots[0];
        new_state.resources[1] += new_state.robots[1];
        new_state.resources[2] += new_state.robots[2];
        new_state.resources[3] += new_state.robots[3];
        new_state.robots[robot_index] += 1;
        new_state
    }
}
#[derive(Copy, Clone)]
struct State {
    resources: [u32; 4],
    robots: [u32; 4],
    rejected_to_buy: [bool; 4],
    time_left: u32,
}
impl State {
    fn new(time_left: u32) -> Self {
        Self {
            resources: [0; 4],
            robots: [1, 0, 0, 0],
            rejected_to_buy: [false; 4],
            time_left,
        }
    }
}

fn optimize(rules: &Rule, initial: &State) -> u32 {
    let mut queue = VecDeque::new();
    queue.push_back(*initial);
    let mut best = 0;
    while let Some(current) = queue.pop_front() {
        if current.time_left == 0 {
            best = best.max(current.resources[rules.target]);
            continue;
        }
        if rules.can_buy(&current, rules.target) {
            queue.push_back(rules.buy_robot(&current, rules.target));
        } else {
            if rules.is_useful(&current, 0)
                && rules.can_buy(&current, 0)
                && !rules.buying_to_late(&current, 0)
            {
                queue.push_back(rules.buy_robot(&current, 0));
            }
            if rules.is_useful(&current, 1)
                && rules.can_buy(&current, 1)
                && !rules.buying_to_late(&current, 1)
            {
                queue.push_back(rules.buy_robot(&current, 1));
            }
            if rules.is_useful(&current, 2)
                && rules.can_buy(&current, 2)
                && !rules.buying_to_late(&current, 2)
            {
                queue.push_back(rules.buy_robot(&current, 2));
            }
            if rules.is_useful(&current, 3)
                && rules.can_buy(&current, 3)
                && !rules.buying_to_late(&current, 3)
            {
                queue.push_back(rules.buy_robot(&current, 3));
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
            .map(|bp| bp.id * optimize(&Rule::from(bp), &State::new(minutes)))
            .sum(),
    )
}

pub fn process_part2(input: String, minutes: u32) -> Option<u32> {
    let (_, blues) = blueprints(&input).ok()?;

    Some(
        blues
            .par_iter()
            .take(3)
            .map(|bp| optimize(&Rule::from(bp), &State::new(minutes)))
            .product(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "Blueprint 1:
  Each ore robot costs 4 ore.
  Each clay robot costs 2 ore.
  Each obsidian robot costs 3 ore and 14 clay.
  Each geode robot costs 2 ore and 7 obsidian.

Blueprint 2:
  Each ore robot costs 2 ore.
  Each clay robot costs 3 ore.
  Each obsidian robot costs 3 ore and 8 clay.
  Each geode robot costs 3 ore and 12 obsidian.";

        assert_eq!(process(COMMANDS.to_string(), 24), Some(33));
        assert_eq!(process_part2(COMMANDS.to_string(), 32), Some(3472));
    }
}
