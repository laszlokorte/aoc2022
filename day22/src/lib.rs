#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use std::collections::{BTreeMap, BTreeSet, HashMap};

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
enum Direction {
    Right,
    Down,
    Left,
    Up,
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
                    if let Some(new_position) = &self.puzzle.go_from(self.position, &self.direction)
                    {
                        self.position = *new_position;
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
    ) -> Option<(usize, usize)> {
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
            return Some((target_x, target_y));
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

fn puzzle(input: &str) -> IResult<&str, Puzzle> {
    let (input, (the_rows, the_steps)) =
        separated_pair(rows, pair(line_ending, line_ending), steps)(input)?;

    Ok((
        input,
        Puzzle {
            rows: the_rows,
            steps: the_steps,
        },
    ))
}
pub fn process(input: String) -> Option<usize> {
    let (_, puzzle) = puzzle(&input).ok()?;
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

pub fn process_3d(input: String) -> Option<usize> {
    let (_, puzzle) = puzzle(&input).ok()?;
    let (w, h) = puzzle.dimensions_2d();
    let side_length = num::integer::gcd(w, h);
    let sides_x = w / side_length;
    let sides_y = h / side_length;

    let mut side_set = BTreeMap::<(usize, usize), usize>::new();
    for y in 0..sides_y {
        for row in 0..side_length {
            for x in 0..sides_x {
                for col in 0..side_length {
                    if puzzle.is_void_at((x * side_length + col, (y * side_length) + row)) {
                        print!(" ")
                    } else {
                        let side_count = side_set.len();
                        let side_number = side_set.entry((x, y)).or_insert(side_count);
                        print!("{}", side_number);
                    }
                }
            }
            println!("");
        }
    }
    None
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

        assert_eq!(process(COMMANDS.to_string()), Some(6032));
        assert_eq!(process_3d(COMMANDS.to_string()), Some(6032));
    }
}
