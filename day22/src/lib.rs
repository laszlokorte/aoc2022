#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::line_ending,
    combinator::{map, value},
    multi::{many1, separated_list1},
    sequence::{pair, separated_pair},
    *,
};
#[derive(Clone, Debug)]
enum Move {
    TurnLeft,
    TurnRight,
    Forward(usize),
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum Field {
    Void,
    Free,
    Stone,
}

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    pub entrance_start: (usize, usize),
    pub entrance_end: (usize, usize),
    pub entrance_direction: Direction,
    pub exit_start: (usize, usize),
    pub exit_end: (usize, usize),
    pub exit_direction: Direction,
}

impl Portal {
    fn teleport(
        &self,
        (x, y): (usize, usize),
        direction: Direction,
    ) -> Option<((usize, usize), Direction)> {
        if direction == self.entrance_direction {
            if self.entrance_orientation_at((x, y)).is_some() {
                let (start_x, start_y) = self.entrance_start;
                let (end_x, end_y) = self.entrance_end;
                let mask_x = (end_x as i32 - start_x as i32).signum();
                let mask_y = (end_y as i32 - start_y as i32).signum();
                let delta_x = x as i32 - start_x as i32;
                let delta_y = y as i32 - start_y as i32;
                let projected_length = delta_x * mask_x + delta_y * mask_y;

                let (exit_start_x, exit_start_y) = self.exit_start;
                let (exit_end_x, exit_end_y) = self.exit_end;
                let exit_mask_x = (exit_end_x as i32 - exit_start_x as i32).signum();
                let exit_mask_y = (exit_end_y as i32 - exit_start_y as i32).signum();

                let projected_x = exit_start_x as i32 + projected_length * exit_mask_x;
                let projected_y = exit_start_y as i32 + projected_length * exit_mask_y;
                // dbg!(exit_mask_x, exit_mask_y);
                // dbg!(self.entrance_start);
                // dbg!(self.exit_start);
                // dbg!(x, y);
                // dbg!(delta_x, delta_y);
                // dbg!(projected_x, projected_y);
                // dbg!(self.exit_direction);
                return Some((
                    (projected_x as usize, projected_y as usize),
                    self.exit_direction,
                ));
            }
        }

        return None;
    }

    pub fn inverse(&self) -> Self {
        Self {
            entrance_start: self.exit_start,
            entrance_end: self.exit_end,
            exit_start: self.entrance_start,
            exit_end: self.entrance_end,
            entrance_direction: self.exit_direction.opposite(),
            exit_direction: self.entrance_direction.opposite(),
        }
    }

    fn entrance_orientation_at(&self, (x, y): (usize, usize)) -> Option<Direction> {
        Self::orientation_at(self.entrance_start, self.entrance_end, (x, y))
    }

    fn orientation_at(
        (start_x, start_y): (usize, usize),
        (end_x, end_y): (usize, usize),
        (x, y): (usize, usize),
    ) -> Option<Direction> {
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);

        if x == start_x && x == end_x {
            if (min_y..=max_y).contains(&y) {
                Some(if start_y > end_y {
                    Direction::Up
                } else {
                    Direction::Down
                })
            } else {
                None
            }
        } else if y == start_y && y == end_y {
            if (min_x..=max_x).contains(&x) {
                Some(if start_x > end_x {
                    Direction::Left
                } else {
                    Direction::Right
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    // fn exit_orientation_at(&self, (x, y): (usize, usize)) -> Option<Direction> {
    //     Self::orientation_at(self.exit_start, self.exit_end, (x, y))
    // }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Free => write!(f, "."),
            Self::Stone => write!(f, "#"),
            Self::Void => write!(f, " "),
        }
    }
}

#[derive(Debug)]
struct Row {
    fields: Vec<Field>,
}

impl Row {
    fn is_void_at(&self, x: usize) -> bool {
        self.fields
            .get(x)
            .map(|f| f == &Field::Void)
            .unwrap_or(true)
    }
    fn can_walk_on(&self, x: usize) -> bool {
        self.fields
            .get(x)
            .map(|f| f == &Field::Free)
            .unwrap_or(false)
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for field in &self.fields {
            let _ = write!(f, "{field}");
        }
        writeln!(f, "")
    }
}

#[derive(Debug)]
struct Puzzle {
    rows: Vec<Row>,
    steps: Vec<Move>,
    portals: Vec<Portal>,
}

impl Puzzle {
    fn dimensions_2d(&self) -> (usize, usize) {
        (
            self.rows
                .iter()
                .map(|row| row.fields.len())
                .max()
                .unwrap_or(0),
            self.rows.len(),
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Direction {
    Right,
    Down,
    Left,
    Up,
}

impl Direction {
    pub fn opposite(&self) -> Direction {
        match &self {
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Up => Self::Down,
        }
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Right => write!(f, ">"),
            Self::Left => write!(f, "<"),
            Self::Down => write!(f, "v"),
            Self::Up => write!(f, "^"),
        }
    }
}

impl Direction {
    fn number(&self) -> usize {
        match self {
            Self::Right => 0,
            Self::Down => 1,
            Self::Left => 2,
            Self::Up => 3,
        }
    }
    fn turn_cw(&self) -> Self {
        match self {
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
            Self::Up => Self::Right,
        }
    }
    fn turn_ccw(&self) -> Self {
        match self {
            Self::Down => Self::Right,
            Self::Left => Self::Down,
            Self::Up => Self::Left,
            Self::Right => Self::Up,
        }
    }
}

struct State<'a> {
    position: (usize, usize),
    step: usize,
    direction: Direction,
    puzzle: &'a Puzzle,
    visited: HashMap<(usize, usize), Direction>,
}

impl<'a> State<'a> {
    fn step(&mut self) {
        let movement = &self.puzzle.steps[self.step];
        match movement {
            Move::TurnLeft => {
                self.direction = self.direction.turn_ccw();
                self.visited.insert(self.position, self.direction);
            }
            Move::TurnRight => {
                self.direction = self.direction.turn_cw();
                self.visited.insert(self.position, self.direction);
            }
            Move::Forward(distance) => {
                for _ in 0..*distance {
                    if let Some((new_direction, new_position)) =
                        &self.puzzle.go_from(self.position, &self.direction)
                    {
                        self.position = *new_position;
                        self.direction = *new_direction;
                        self.visited.insert(*new_position, self.direction);
                    }
                }
            }
        }
        self.step += 1;
    }
}

impl Puzzle {
    fn go_from(
        &self,
        (mut x, mut y): (usize, usize),
        direction: &Direction,
    ) -> Option<(Direction, (usize, usize))> {
        if let Some(((ported_x, ported_y), ported_direction)) = self
            .portals
            .iter()
            .find_map(|p| p.teleport((x, y), *direction))
        {
            if self.can_walk_on((ported_x, ported_y)) {
                return Some((ported_direction, (ported_x, ported_y)));
            } else {
                return None;
            }
        }
        let (max_x, max_y) = self.dimensions_2d();
        let (target_x, target_y) = loop {
            (x, y) = match direction {
                Direction::Right => ((x + 1).rem_euclid(max_x), y),
                Direction::Left => ((x + max_x - 1).rem_euclid(max_x), y),
                Direction::Down => (x, (y + 1).rem_euclid(max_y)),
                Direction::Up => (x, (y + max_y - 1).rem_euclid(max_y)),
            };

            if !&self.is_void_at((x, y)) {
                break (x, y);
            }
        };

        if self.can_walk_on((target_x, target_y)) {
            return Some((*direction, (target_x, target_y)));
        } else {
            return None;
        }
    }

    fn is_void_at(&self, (x, y): (usize, usize)) -> bool {
        self.rows[y].is_void_at(x)
    }
    fn can_walk_on(&self, (x, y): (usize, usize)) -> bool {
        self.rows[y].can_walk_on(x)
    }
}

impl std::fmt::Display for Puzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            let _ = write!(f, "{}", row);
        }
        write!(f, "")
    }
}
impl<'a> std::fmt::Display for State<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (r, row) in self.puzzle.rows.iter().enumerate() {
            for (c, field) in row.fields.iter().enumerate() {
                if let Some(d) = self.visited.get(&(c, r as usize)) {
                    let _ = write!(f, "{}", d);
                } else if self.position == (c, r as usize) {
                    let _ = write!(f, "{}", self.direction);
                } else {
                    let _ = write!(f, "{}", field);
                }
            }
            let _ = writeln!(f, "");
        }
        write!(f, "")
    }
}

fn steps(input: &str) -> IResult<&str, Vec<Move>> {
    many1(alt((
        value(Move::TurnLeft, tag("L")),
        value(Move::TurnRight, tag("R")),
        map(character::complete::u32, |c| Move::Forward(c as usize)),
    )))(input)
}

fn row(input: &str) -> IResult<&str, Row> {
    let (input, fields) = many1(alt((
        value(Field::Free, tag(".")),
        value(Field::Void, tag(" ")),
        value(Field::Stone, tag("#")),
    )))(input)?;

    Ok((input, Row { fields }))
}

fn rows(input: &str) -> IResult<&str, Vec<Row>> {
    separated_list1(line_ending, row)(input)
}

fn puzzle(input: &str, portals: Vec<Portal>) -> IResult<&str, Puzzle> {
    let (input, (the_rows, the_steps)) =
        separated_pair(rows, pair(line_ending, line_ending), steps)(input)?;

    Ok((
        input,
        Puzzle {
            rows: the_rows,
            steps: the_steps,
            portals,
        },
    ))
}
pub fn process(input: String) -> Option<usize> {
    let (_, puzzle) = puzzle(&input, vec![]).ok()?;
    let mut state = State {
        direction: Direction::Right,
        position: (
            puzzle.rows[0]
                .fields
                .iter()
                .position(|f| f == &Field::Free)?,
            0,
        ),
        puzzle: &puzzle,
        step: 0,
        visited: HashMap::new(),
    };
    for _ in 0..puzzle.steps.len() {
        state.step();
    }
    Some(1000 * (1 + state.position.1) + 4 * (1 + state.position.0) + state.direction.number())
}

pub fn process_with_portals(input: String, portals: Vec<Portal>) -> Option<usize> {
    let (_, puzzle) = puzzle(&input, portals).ok()?;
    // let (w, h) = puzzle.dimensions_2d();
    // let side_length = num::integer::gcd(w, h);
    // let sides_x = w / side_length;
    // let sides_y = h / side_length;
    // let mut side_set = BTreeMap::<(usize, usize), usize>::new();
    // for y in 0..sides_y {
    //     for row in 0..side_length {
    //         for x in 0..sides_x {
    //             for col in 0..side_length {
    //                 let coord = (x * side_length + col, (y * side_length) + row);
    //                 if let Some(portal_orientation) = puzzle.portals.iter().find_map(|p| {
    //                     p.entrance_orientation_at(coord)
    //                         .map(|_| p.entrance_direction)
    //                 }) {
    //                     print!("{portal_orientation}");
    //                 } else if puzzle.is_void_at(coord) {
    //                     print!(" ")
    //                 } else {
    //                     let side_count = side_set.len();
    //                     let side_number = side_set.entry((x, y)).or_insert(side_count);
    //                     print!("{}", side_number);
    //                 }
    //             }
    //         }
    //         println!("");
    //     }
    // }

    let mut state = State {
        direction: Direction::Right,
        position: (
            puzzle.rows[0]
                .fields
                .iter()
                .position(|f| f == &Field::Free)?,
            0,
        ),
        puzzle: &puzzle,
        step: 0,
        visited: HashMap::new(),
    };
    for _ in 0..puzzle.steps.len() {
        state.step();
    }
    Some(1000 * (1 + state.position.1) + 4 * (1 + state.position.0) + state.direction.number())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.

10R5L5R10L4R5L5";

        let portals = vec![
            Portal {
                entrance_start: (8, 3),
                entrance_end: (8, 0),
                entrance_direction: Direction::Left,
                exit_start: (7, 4),
                exit_end: (4, 4),
                exit_direction: Direction::Down,
            },
            Portal {
                entrance_start: (8, 0),
                entrance_end: (11, 0),
                entrance_direction: Direction::Up,
                exit_start: (3, 4),
                exit_end: (0, 4),
                exit_direction: Direction::Down,
            },
            Portal {
                entrance_start: (11, 0),
                entrance_end: (11, 3),
                entrance_direction: Direction::Right,
                exit_start: (15, 11),
                exit_end: (15, 8),
                exit_direction: Direction::Left,
            },
            Portal {
                entrance_start: (11, 4),
                entrance_end: (11, 7),
                entrance_direction: Direction::Right,
                exit_start: (15, 8),
                exit_end: (12, 8),
                exit_direction: Direction::Down,
            },
            Portal {
                entrance_start: (0, 4),
                entrance_end: (0, 7),
                entrance_direction: Direction::Left,
                exit_start: (15, 11),
                exit_end: (12, 11),
                exit_direction: Direction::Up,
            },
            Portal {
                entrance_start: (0, 7),
                entrance_end: (4, 7),
                entrance_direction: Direction::Down,
                exit_start: (11, 11),
                exit_end: (8, 11),
                exit_direction: Direction::Up,
            },
            Portal {
                entrance_start: (4, 7),
                entrance_end: (7, 7),
                entrance_direction: Direction::Down,
                exit_start: (8, 11),
                exit_end: (8, 8),
                exit_direction: Direction::Right,
            },
        ]
        .into_iter()
        .flat_map(|p| [p.inverse(), p])
        .collect();

        // assert_eq!(process(COMMANDS.to_string()), Some(6032));
        assert_eq!(
            process_with_portals(COMMANDS.to_string(), portals),
            Some(5031)
        );
    }
}
