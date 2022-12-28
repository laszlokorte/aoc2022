#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    mem,
};

#[derive(Debug, Eq, PartialEq, Hash)]
enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Eq, PartialEq, Hash)]
struct Blizzard {
    start: (isize, isize),
    direction: Direction,
}

impl Blizzard {
    fn position_at_time(&self, time: isize, width: isize, height: isize) -> (isize, isize) {
        let x = self.start.0;
        let y = self.start.1;
        let (dx, dy) = match self.direction {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        };

        let x_at_time = (x + dx * time).rem_euclid(width);
        let y_at_time = (y + dy * time).rem_euclid(height);

        (x_at_time, y_at_time)
    }
}

struct Valley {
    width: isize,
    height: isize,
    winds: Vec<Blizzard>,
}
impl std::fmt::Display for Valley {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let winds = self.blizzards_at_time(3);

        let _ = write!(f, "# ");
        for _ in 1..self.width {
            let _ = write!(f, "#");
        }

        let _ = writeln!(f, "#");
        for y in 0..self.height {
            let _ = write!(f, "#");
            for x in 0..self.width {
                if winds.contains(&(x, y)) {
                    let _ = write!(f, "w");
                } else {
                    let _ = write!(f, ".");
                }
            }
            let _ = writeln!(f, "#");
        }
        for _ in 0..self.width {
            let _ = write!(f, "#");
        }
        writeln!(f, " #")
    }
}
impl Valley {
    fn blizzards_at_time(&self, time: isize) -> HashSet<(isize, isize)> {
        self.winds
            .iter()
            .map(|b| b.position_at_time(time, self.width, self.height))
            .collect()
    }

    fn possible_moves(
        &self,
        (x, y): (isize, isize),
        current_time: isize,
    ) -> impl Iterator<Item = ((isize, isize), isize)> {
        let width = self.width;
        let height = self.height;
        let next_time = current_time + 1;
        let winds = self.blizzards_at_time(next_time);
        [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)]
            .into_iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(move |&(x, y)| x >= 0 && y >= 0 || (x, y) == (0, -1))
            .filter(move |&(x, y)| x < width && y < height || (x, y) == (width - 1, height))
            .filter(move |p| !winds.contains(p))
            .map(move |p| (p, next_time))
    }
}

fn parse_wind(x: isize, y: isize, char: char) -> Option<Blizzard> {
    Some(Blizzard {
        direction: match char {
            '>' => Direction::East,
            '^' => Direction::North,
            'v' => Direction::South,
            '<' => Direction::West,
            _ => return None,
        },
        start: (x, y),
    })
}

pub fn process(input: String, number_of_passes: usize) -> Option<isize> {
    let lines = input.lines().into_iter().skip(1);
    let height = lines.clone().count() as isize - 1;
    let width = lines.clone().find_map(|line| Some(line.len() - 2))? as isize;
    let blizzards = lines.enumerate().flat_map(|(y, line)| {
        line.chars()
            .skip(1)
            .enumerate()
            .filter_map(move |(x, c)| parse_wind(x as isize, y as isize, c))
    });
    let valley = Valley {
        winds: blizzards.collect(),
        width,
        height,
    };
    let mut total_time = 0;
    let mut start_position: (isize, isize) = (0, -1);
    let mut goal: (isize, isize) = (width - 1, height);
    for _ in 0..number_of_passes {
        let mut visited = HashSet::<((isize, isize), isize)>::new();
        let mut queue = VecDeque::<((isize, isize), isize)>::new();
        queue.push_back((start_position, total_time));
        let time_taken = 'search: loop {
            let Some((current_pos, current_time)) = queue.pop_front() else {
            break 'search None;
        };
            if current_pos == goal {
                break 'search Some(current_time);
            }
            for (next_pos, next_time) in valley.possible_moves(current_pos, current_time) {
                if visited.insert((
                    next_pos,
                    next_time.rem_euclid(num::integer::lcm(width, height)),
                )) {
                    queue.push_back((next_pos, next_time))
                }
            }
        };
        if let Some(t) = time_taken {
            total_time = t;
        } else {
            return None;
        }
        mem::swap(&mut goal, &mut start_position);
    }
    Some(total_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#";

        assert_eq!(process(COMMANDS.to_string(), 1), Some(18));
        assert_eq!(process(COMMANDS.to_string(), 3), Some(54));
    }
}
