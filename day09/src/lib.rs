#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]
#![feature(array_windows)]

use std::collections::BTreeSet;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone)]
struct Rope {
    head: Position,
    segments: Vec<Position>,
}

impl Movement {
    fn to_steps(self) -> Vec<Direction> {
        vec![self.direction; self.distance as usize]
    }
}

impl Direction {
    fn dx(&self) -> i32 {
        match self {
            Self::Left => 1,
            Self::Right => -1,
            _ => 0,
        }
    }

    fn dy(&self) -> i32 {
        match self {
            Self::Up => -1,
            Self::Down => 1,
            _ => 0,
        }
    }
}

impl Position {
    fn new() -> Self {
        Self { x: 0, y: 0 }
    }

    fn moved_in_direction(self, dir: Direction) -> Self {
        Self {
            x: self.x + dir.dx(),
            y: self.y + dir.dy(),
        }
    }

    fn follow(self, head: Position) -> Option<Position> {
        let delta_x = head.x - self.x;
        let delta_y = head.y - self.y;
        let (x_abs, x_sgn) = (delta_x.abs(), delta_x.signum());
        let (y_abs, y_sgn) = (delta_y.abs(), delta_y.signum());
        let chebyshev_distance = i32::max(x_abs, y_abs);

        if chebyshev_distance < 2 {
            Some(self)
        } else {
            match (x_abs, y_abs) {
                (_, 0) => Some(Position {
                    x: self.x + x_sgn,
                    ..self
                }),
                (0, _) => Some(Position {
                    y: self.y + y_sgn,
                    ..self
                }),
                (l, r) if l + r <= 4 => Some(Position {
                    x: self.x + x_sgn,
                    y: self.y + y_sgn,
                }),
                _ => None,
            }
        }
    }
}

impl Rope {
    fn new(length: usize) -> Self {
        Rope {
            head: Position::new(),
            segments: vec![Position::new(); length],
        }
    }

    fn move_head(&self, dir: Direction) -> Option<Rope> {
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

fn movement(input: &str) -> IResult<&str, Movement> {
    let (input, direction) = alt((
        tag("U").map(|_| Direction::Up),
        tag("D").map(|_| Direction::Down),
        tag("L").map(|_| Direction::Left),
        tag("R").map(|_| Direction::Right),
    ))(input)?;
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
    let (input, cmds) = separated_list1(line_ending, movement)(input)?;

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

pub fn process_alternative(input: String, length: usize) -> Option<usize> {
    let (_, mvs) = moves(&input).ok()?;

    Some(
        mvs.iter()
            .copied()
            .flat_map(Movement::to_steps)
            .scan(Some(Rope::new(length)), |r, dir| {
                if let Some(ref rr) = r {
                    let rrr = rr.move_head(dir)?;
                    *r = Some(rrr.clone());
                    Some(rrr.tail())
                } else {
                    None
                }
            })
            .collect::<Option<BTreeSet<Position>>>()?
            .len(),
    )
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

        assert_eq!(process_alternative(COMMANDS.to_string(), 1), Some(13));
        assert_eq!(process_alternative(COMMANDS.to_string(), 9), Some(1));
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
        assert_eq!(process_alternative(COMMANDS.to_string(), 9), Some(36));
    }
}
