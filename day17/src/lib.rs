#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
enum Shape {
    Horizontal,
    Cross,
    Jay,
    Vertical,
    Box,
}

impl Shape {
    fn spawn_at(&self, pos: &Position) -> Stone {
        match self {
            Self::Horizontal => [0, 1, 2, 3]
                .iter()
                .map(|x| Position {
                    x: x + pos.x,
                    y: pos.y,
                })
                .collect(),
            Self::Cross => [(1, 0), (1, 1), (1, 2), (0, 1), (2, 1)]
                .iter()
                .map(|(x, y)| Position {
                    x: pos.x + x,
                    y: pos.y + y,
                })
                .collect(),
            Self::Jay => [(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]
                .iter()
                .map(|(x, y)| Position {
                    x: pos.x + x,
                    y: pos.y + y,
                })
                .collect(),
            Self::Vertical => [0, 1, 2, 3]
                .iter()
                .map(|y| Position {
                    x: pos.x,
                    y: pos.y + y,
                })
                .collect(),
            Self::Box => [(0, 0), (0, 1), (1, 0), (1, 1)]
                .iter()
                .map(|(x, y)| Position {
                    x: pos.x + x,
                    y: pos.y + y,
                })
                .collect(),
        }
    }
}

type Stone = HashSet<Position>;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Position {
    x: i64,
    y: i64,
}

#[derive(Debug)]
enum Direction {
    Left,
    Right,
    Down,
}

impl Position {
    fn step(&self, direction: &Direction) -> Position {
        match direction {
            Direction::Left => Self {
                x: self.x - 1,
                ..*self
            },
            Direction::Right => Self {
                x: self.x + 1,
                ..*self
            },
            Direction::Down => Self {
                y: self.y - 1,
                ..*self
            },
        }
    }
}

struct Cave<const WIDTH: i64> {
    stone: Option<Stone>,
    fields: HashSet<Position>,
}
impl<const WIDTH: i64> std::fmt::Display for Cave<WIDTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let max_height = self
            .fields
            .iter()
            .map(|p| p.y)
            .max()
            .unwrap_or_default()
            .max(
                self.stone
                    .as_ref()
                    .map(|s| s.iter().map(|p| p.y).max().unwrap_or_default())
                    .unwrap_or_default(),
            );
        for r in 0..=max_height {
            let y = max_height - r;
            for x in 0..7 {
                let pos = Position { x, y };
                if self.fields.contains(&pos) {
                    write!(f, "#")?;
                } else if let Some(s) = self.stone.as_ref() && s.contains(&pos) {
                    write!(f, "@")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }

        writeln!(f, "*******")
    }
}
impl<const WIDTH: i64> Cave<WIDTH> {
    fn new() -> Self {
        Self {
            fields: HashSet::default(),
            stone: None,
        }
    }
    fn max_height(&self) -> i64 {
        self.fields
            .iter()
            .map(|p| p.y + 1)
            .max()
            .unwrap_or_default()
    }

    fn is_free(&self, pos: &Position) -> bool {
        (0..WIDTH).contains(&pos.x) && pos.y >= 0 && !self.fields.contains(pos)
    }
    const GRAVITY: Direction = Direction::Down;
    fn step(&mut self, wind: &Direction) -> bool {
        if let Some(stone) = &self.stone {
            let fragments = stone.iter().map(|p| p.step(wind));

            let valid = fragments.clone().all(|p| self.is_free(&p));
            let new_stone = if valid {
                fragments.collect()
            } else {
                stone.clone()
            };
            let fallen = new_stone.iter().map(|p| p.step(&Self::GRAVITY));

            let valid = fallen.clone().all(|p| self.is_free(&p));
            if valid {
                self.stone = Some(fallen.collect());
                true
            } else {
                for p in new_stone {
                    self.fields.insert(p);
                }
                self.stone = None;
                false
            }
        } else {
            false
        }
    }

    fn spawn(&mut self, shape: &Shape) {
        if self.stone.is_none() {
            self.stone = Some(shape.spawn_at(&self.spawn_position()));
        }
    }

    fn spawn_position(&self) -> Position {
        Position {
            x: 2,
            y: self.max_height() + 3,
        }
    }

    fn get_top(&self, rows: u64) -> BTreeSet<Position> {
        let top = self.max_height();
        let range = (top - rows as i64)..top;

        self.fields
            .iter()
            .filter(|p| range.contains(&p.y))
            .map(|p| Position {
                x: p.x,
                y: top - p.y,
            })
            .collect()
    }
}
#[derive(Hash, Eq, PartialEq)]
struct CycleIndex {
    tops: BTreeSet<Position>,
    shape_index: usize,
    wind_index: usize,
}

struct CycleEntry {
    height: i64,
    stone_number: u64,
}

#[derive(Default)]
struct CycleDetector {
    cycles: HashMap<CycleIndex, CycleEntry>,
}

struct CycleResult {
    remaining_drops: std::ops::Range<u64>,
    extrapolated_height: i64,
}

impl CycleDetector {
    fn detect(
        &mut self,
        tops: BTreeSet<Position>,
        shape_index: usize,
        wind_index: usize,
        current_height: i64,
        stone_number: u64,
        iterations: u64,
    ) -> Option<CycleResult> {
        if let Some(detected) = &mut self.cycles.insert(
            CycleIndex {
                tops,
                shape_index,
                wind_index,
            },
            CycleEntry {
                stone_number,
                height: current_height,
            },
        ) {
            let delta_height = current_height - detected.height;
            let height_before_cycle = detected.height;
            let delta_drops = stone_number - detected.stone_number;
            let remaining_drops = iterations - detected.stone_number;
            let skippable_cycles = remaining_drops / delta_drops;
            let drops_to_do = remaining_drops % delta_drops;
            let skippable_height = skippable_cycles as i64 * delta_height;
            Some(CycleResult {
                remaining_drops: (iterations - drops_to_do + 1)..iterations,
                extrapolated_height: skippable_height + height_before_cycle - current_height,
            })
        } else {
            None
        }
    }
}

pub fn process(input: String, iterations: u64) -> Option<i64> {
    let wind = input
        .chars()
        .filter_map(|c| match c {
            '>' => Some(Direction::Right),
            '<' => Some(Direction::Left),
            _ => None,
        })
        .collect::<Vec<Direction>>();
    const SHAPES: [Shape; 5] = [
        Shape::Horizontal,
        Shape::Cross,
        Shape::Jay,
        Shape::Vertical,
        Shape::Box,
    ];
    let mut cave = Cave::<7>::new();
    let mut cycle_detector = CycleDetector::default();
    let mut wind_index = 0;
    for stone_number in 0..iterations {
        let shape_index = stone_number as usize % SHAPES.len();
        let current_shape = &SHAPES[shape_index];
        cave.spawn(current_shape);
        while cave.step(&wind[wind_index]) {
            wind_index += 1;
            wind_index %= wind.len();
        }
        wind_index += 1;
        wind_index %= wind.len();
        let tops = cave.get_top(4);
        if let Some(cycle) = cycle_detector.detect(
            tops,
            shape_index,
            wind_index,
            cave.max_height(),
            stone_number,
            iterations,
        ) {
            for stone_number in cycle.remaining_drops {
                let shape_index = stone_number as usize % SHAPES.len();
                let current_shape = &SHAPES[shape_index];
                cave.spawn(current_shape);
                while cave.step(&wind[wind_index]) {
                    wind_index += 1;
                    wind_index %= wind.len();
                }
                wind_index += 1;
                wind_index %= wind.len();
            }

            return Some(cave.max_height() + cycle.extrapolated_height);
        }
    }
    Some(cave.max_height())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

        assert_eq!(process(COMMANDS.to_string(), 2022), Some(3068));
        assert_eq!(
            process(COMMANDS.to_string(), 1000000000000),
            Some(1514285714288)
        );
    }
}
