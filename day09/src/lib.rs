#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]
#![feature(array_windows)]

use std::collections::BTreeSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::newline;
use nom::error::Error;
use nom::error::ErrorKind;
use nom::multi::separated_list1;
use nom::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Movement {
    direction: Direction,
    distance: u32,
}

impl Movement {
    fn to_steps(self) -> Vec<Direction> {
        vec![self.direction; self.distance as usize]
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    fn moved_in_direction(self, dir: Direction) -> Self {
        match dir {
            Direction::Up => Self {
                y: self.y - 1,
                ..self
            },
            Direction::Down => Self {
                y: self.y + 1,
                ..self
            },
            Direction::Left => Self {
                x: self.x - 1,
                ..self
            },
            Direction::Right => Self {
                x: self.x + 1,
                ..self
            },
        }
    }
    fn follow(self, head: Position) -> Option<Position> {
        let delta_x = head.x - self.x;
        let delta_y = head.y - self.y;
        let max = i32::max(delta_x.abs(), delta_y.abs());

        if max < 2 {
            Some(self)
        } else {
            match (delta_x.abs(), delta_y.abs()) {
                (_, 0) => Some(Position {
                    x: self.x + delta_x.signum(),
                    ..self
                }),
                (0, _) => Some(Position {
                    y: self.y + delta_y.signum(),
                    ..self
                }),
                (l, r) if l + r <= 4 => Some(Position {
                    x: self.x + delta_x.signum(),
                    y: self.y + delta_y.signum(),
                }),
                _ => None,
            }
        }
    }
}

struct Rope {
    head: Position,
    segments: Vec<Position>,
}

impl Rope {
    fn new(length: usize) -> Self {
        Rope {
            head: Position::new(),
            segments: vec![Position::new(); length],
        }
    }

    fn move_head(self, dir: Direction) -> Option<Rope> {
        let new_head = self.head.moved_in_direction(dir);

        Some(Rope {
            head: new_head,
            segments: self
                .segments
                .iter()
                .scan(Some(new_head), |a, b| {
                    *a = b.follow((*a)?);
                    Some(*a)
                })
                .collect::<Option<Vec<_>>>()?,
        })
    }

    fn tail(&self) -> Option<Position> {
        Some(*self.segments.last()?)
    }
}

fn direction(input: &str) -> IResult<&str, Direction> {
    let (input, dir) = alt((tag("U"), tag("D"), tag("L"), tag("R")))(input)?;

    Ok((
        input,
        match dir {
            "U" => Direction::Up,
            "D" => Direction::Down,
            "L" => Direction::Left,
            "R" => Direction::Right,
            _ => {
                return Err(nom::Err::Error(Error {
                    input,
                    code: ErrorKind::Tag,
                }));
            }
        },
    ))
}

fn movement(input: &str) -> IResult<&str, Movement> {
    let (input, direction) = direction(input)?;
    let (input, _) = tag(" ")(input)?;
    let (input, distance) = character::complete::u32(input)?;

    Ok((
        input,
        Movement {
            direction,
            distance,
        },
    ))
}

fn moves(input: &str) -> IResult<&str, Vec<Movement>> {
    let (input, cmds) = separated_list1(newline, movement)(input)?;

    Ok((input, cmds))
}

pub fn process(input: String, length: usize) -> Option<usize> {
    let (_, mvs) = moves(&input).ok()?;

    let mut rope = Rope::new(length);

    let mut visited = BTreeSet::<Position>::new();

    for dir in mvs.iter().copied().flat_map(Movement::to_steps) {
        rope = rope.move_head(dir)?;

        visited.insert(rope.tail()?);
    }

    Some(visited.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "R 4\n\
        U 4\n\
        L 3\n\
        D 1\n\
        R 4\n\
        D 1\n\
        L 5\n\
        R 2";

        assert_eq!(process(COMMANDS.to_string(), 1), Some(13));
        assert_eq!(process(COMMANDS.to_string(), 9), Some(1));
    }

    #[test]
    fn test_process_longer() {
        const COMMANDS: &str = "R 5\n\
        U 8\n\
        L 8\n\
        D 3\n\
        R 17\n\
        D 10\n\
        L 25\n\
        U 20";

        assert_eq!(process(COMMANDS.to_string(), 9), Some(36));
    }
}
