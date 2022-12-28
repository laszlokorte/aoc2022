#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
enum Proposition {
    Single((isize, isize)),
    Duplicate,
}

fn neighbourhood((x, y): (isize, isize)) -> impl Iterator<Item = (isize, isize)> {
    [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 1),
        (0, 1),
        (1, 1),
        (-1, 0),
        (1, 0),
    ]
    .into_iter()
    .map(move |(dx, dy)| (x + dx, y + dy))
}

struct Landscape {
    elves: BTreeSet<(isize, isize)>,
}

impl Landscape {
    const TRY_DIRECTIONS: [[(isize, isize); 3]; 4] = [
        [(-1, -1), (0, -1), (1, -1)],
        [(-1, 1), (0, 1), (1, 1)],
        [(-1, -1), (-1, 0), (-1, 1)],
        [(1, -1), (1, 0), (1, 1)],
    ];
    fn step(&mut self, round: usize) -> bool {
        let mut propositions = BTreeMap::<(isize, isize), Proposition>::new();
        for &(x, y) in &self.elves {
            if !neighbourhood((x, y)).any(|n| self.elves.contains(&n)) {
                continue;
            }
            let Some(free_position) = Self::TRY_DIRECTIONS.iter().cycle().skip(round).take(4).find_map(|[a, b, c]| {
                if ![a, b, c]
                    .into_iter()
                    .any(|(dx, dy)| self.elves.contains(&(x + dx, y + dy)))
                {
                    Some((x + b.0, y + b.1))
                } else {
                    None
                }
            }) else {
                continue;
            };

            propositions
                .entry(free_position)
                .and_modify(|p| *p = Proposition::Duplicate)
                .or_insert(Proposition::Single((x, y)));
        }
        if propositions.is_empty() {
            false
        } else {
            for (new_pos, proposition) in &propositions {
                if let Proposition::Single(old_pos) = proposition {
                    self.elves.remove(old_pos);
                    self.elves.insert(*new_pos);
                }
            }
            propositions.clear();
    
            true
        }
    }

    fn count_empty(&self) -> isize {
        let minx = *self.elves.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let maxx = *self.elves.iter().map(|(x, _)| x).max().unwrap_or(&0);
        let miny = *self.elves.iter().map(|(_, y)| y).min().unwrap_or(&0);
        let maxy = *self.elves.iter().map(|(_, y)| y).max().unwrap_or(&0);
        let width = maxx - minx + 1;
        let height = maxy - miny + 1;

        width * height - self.elves.len() as isize
    }
}

impl std::fmt::Display for Landscape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let minx = *self.elves.iter().map(|(x, _)| x).min().unwrap_or(&0);
        let maxx = *self.elves.iter().map(|(x, _)| x).max().unwrap_or(&0);
        let miny = *self.elves.iter().map(|(_, y)| y).min().unwrap_or(&0);
        let maxy = *self.elves.iter().map(|(_, y)| y).max().unwrap_or(&0);
        for y in miny..maxy {
            for x in minx..maxx {
                if self.elves.contains(&(x, y)) {
                    let _ = write!(f, "#");
                } else {
                    let _ = write!(f, ".");
                }
            }
            let _ = writeln!(f);
        }
        writeln!(f)
    }
}
pub fn process(input: String, round_limit: Option<usize>) -> Option<(usize, isize)> {
    let current_positions = input
        .lines()
        .enumerate()
        .flat_map(|(row, l)| {
            l.chars().enumerate().filter_map(move |(col, c)| {
                if c == '#' {
                    Some((col as isize, row as isize))
                } else {
                    None
                }
            })
        })
        .collect::<BTreeSet<(isize, isize)>>();

    let mut landscape = Landscape {
        elves: current_positions,
    };

    let mut round = 0;
    let last_round = loop {
        if let Some(limit) = round_limit && limit <= round {
            break round;
        }
        if !landscape.step(round) {
            break round + 1;
        }
        round += 1;
    };
    println!("{landscape}");
    Some((last_round, landscape.count_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process(COMMANDS.to_string(), Some(10)), Some((10, 110)));
        assert_eq!(process(COMMANDS.to_string(), None), Some((20, 146)));
    }
}
