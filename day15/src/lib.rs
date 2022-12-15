#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::combinator::map;
use nom::multi::separated_list0;
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::*;
use std::collections::HashSet;

use rayon::prelude::*;
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn manhatten(&self, other: &Self) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct SensorReading {
    own_position: Position,
    nearest_bacon: Position,
}

impl From<(i32, i32)> for Position {
    fn from(value: (i32, i32)) -> Self {
        Position {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<(Position, Position)> for SensorReading {
    fn from(value: (Position, Position)) -> Self {
        SensorReading {
            own_position: value.0,
            nearest_bacon: value.1,
        }
    }
}

impl SensorReading {
    fn reachable_range_at_y(&self, y: i32) -> Option<std::ops::RangeInclusive<i32>> {
        let beacon_distance = Position::manhatten(&self.own_position, &self.nearest_bacon);
        let vertical_distance = (self.own_position.y - y).abs();
        let horizontal_rest = beacon_distance - vertical_distance;
        if horizontal_rest > 0 {
            let min_x = self.own_position.x - horizontal_rest;
            let max_x = self.own_position.x + horizontal_rest;

            Some(min_x..=max_x)
        } else {
            None
        }
    }
}

fn reading(input: &str) -> IResult<&str, SensorReading> {
    map(
        pair(
            preceded(
                tag("Sensor at x="),
                separated_pair(
                    character::complete::i32,
                    tag(", y="),
                    character::complete::i32,
                )
                .map(Position::from),
            ),
            preceded(
                tag(": closest beacon is at x="),
                separated_pair(
                    character::complete::i32,
                    tag(", y="),
                    character::complete::i32,
                )
                .map(Position::from),
            ),
        ),
        SensorReading::from,
    )(input)
}

fn sensor_readings(input: &str) -> IResult<&str, Vec<SensorReading>> {
    // Sensor at x=2, y=18: closest beacon is at x=-2, y=15
    separated_list0(line_ending, reading)(input)
}

pub fn process(input: String, line: i32) -> Option<usize> {
    let (_, readings) = sensor_readings(&input).ok()?;
    let ruled_out = HashSet::<i32>::from_iter(
        readings
            .iter()
            .flat_map(|r| r.reachable_range_at_y(line))
            .flatten(),
    );
    Some(ruled_out.len() - 1)
}

fn merge_range(
    a: std::ops::RangeInclusive<i32>,
    b: std::ops::RangeInclusive<i32>,
) -> Result<std::ops::RangeInclusive<i32>, std::ops::Range<i32>> {
    if a.contains(b.start()) || a.contains(b.end()) {
        Ok(*a.start().min(b.start())..=*(a.end().max(b.end())))
    } else {
        Err((*a.end().min(b.end()) + 1)..*(a.start().max(b.start())))
    }
}

pub fn process_search(input: String, limit: i32) -> Option<u64> {
    let (_, readings) = sensor_readings(&input).ok()?;
    let rng: Vec<i32> = (0..=limit).collect();
    rng.par_iter().find_map_any(|&line| {
        let mut ranges = readings
            .iter()
            .flat_map(|r| r.reachable_range_at_y(line))
            .collect::<Vec<_>>();
        ranges.sort_by_key(|r| *r.start());

        if let Err(e) = ranges.iter().cloned().try_reduce(merge_range) {
            return Some(e.start as u64 * 4000000 + line as u64);
        }

        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";

        assert_eq!(process(COMMANDS.to_string(), 10), Some(26));
        assert_eq!(process_search(COMMANDS.to_string(), 20), Some(56000011));
    }
}
