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
use std::collections::HashMap;

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

impl Blueprint {
    fn usefulness(&self, resource: &Resource) -> u32 {
        self.robots
            .iter()
            .map(|(_, cost)| cost.ingredients.get(resource).unwrap_or(&0))
            .max()
            .cloned()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
struct State<'blueprint> {
    blueprint: &'blueprint Blueprint,
    time_left: u32,
    resources: HashMap<Resource, u32>,
    robots: HashMap<Resource, u32>,
}

impl<'blueprint> State<'blueprint> {
    fn new(blueprint: &'blueprint Blueprint, time_left: u32) -> Self {
        let mut robots = HashMap::new();
        robots.insert(Resource::Ore, 1);
        State {
            blueprint,
            time_left,
            resources: HashMap::new(),
            robots,
        }
    }

    fn step(&self) -> Option<Self> {
        if self.time_left < 1 {
            return None;
        }
        let mut next_state = self.clone();
        next_state.time_left -= 1;

        for (res, amount) in self.robots.iter() {
            *next_state.resources.entry(*res).or_insert(0) += amount;
        }

        Some(next_state)
    }

    fn try_build_robot(&self, robot: &Resource) -> Option<Self> {
        if let Some(cost) = self.blueprint.robots.get(&robot) {
            if cost
                .ingredients
                .iter()
                .all(|(res, amount)| &self.resources.get(res).unwrap_or(&0) >= &amount)
            {
                let mut next_state = self.clone();

                for (res, amount) in cost.ingredients.iter() {
                    *next_state.resources.entry(*res).or_insert(0) -= amount;
                }

                *next_state.robots.entry(*robot).or_insert(0) += 1;
                return Some(next_state);
            }
        }

        None
    }
    const ALL_RESOURCES: [Resource; 3] = [
        // Resource::Ore,
        Resource::Clay,
        Resource::Obsidian,
        Resource::Geode,
    ];
    fn highest_resource(&self, resource: &Resource) -> u32 {
        let usefuls = Self::ALL_RESOURCES.iter().filter(|r| {
            &resource != r
                && &self.blueprint.usefulness(r) > self.robots.get(resource).unwrap_or(&0)
        });
        let alternative_builds = [resource]
            .into_iter()
            .chain(usefuls)
            .chain([&Resource::Ore].into_iter());

        alternative_builds
            .clone()
            .find_map(|r| self.try_build_robot(r))
            .iter()
            .chain(
                [Some(self), self.try_build_robot(&Resource::Ore).as_ref()]
                    .into_iter()
                    .filter_map(|s| s),
            )
            .filter_map(|s| s.step())
            .map(|s| s.highest_resource(&resource))
            .max()
            .or(self.resources.get(&resource).cloned())
            .unwrap_or(0)
    }
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
pub fn process(input: String) -> Option<u32> {
    let (_, blues) = blueprints(&input).ok()?;

    Some(
        blues
            .par_iter()
            .map(|bp| bp.id * State::new(bp, 24).highest_resource(&Resource::Geode))
            .sum(),
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

        assert_eq!(process(COMMANDS.to_string()), Some(33));
    }
}
