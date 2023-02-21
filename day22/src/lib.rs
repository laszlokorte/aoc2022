#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]
#![feature(map_try_insert)]
#![feature(map_many_mut)]

use std::collections::{BTreeMap, HashMap, VecDeque};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
        if direction == self.entrance_direction && self.entrance_orientation_at((x, y)).is_some() {
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
            
            Some((
                (projected_x as usize, projected_y as usize),
                self.exit_direction,
            ))
        } else {
            None
        }
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
        writeln!(f)
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

    fn get_rotation(&self) -> Transform {
        match self {
            Direction::Right => Transform::rotate_y(1),
            Direction::Down => Transform::rotate_x(1),
            Direction::Left => Transform::rotate_y(-1),
            Direction::Up => Transform::rotate_x(-1),
        }
    }

    fn get_delta(&self) -> (isize, isize) {
        match self {
            Direction::Right => (-1, 0),
            Direction::Down => (0,1),
            Direction::Left => (1,0),
            Direction::Up => (0,-1),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Transform {
    matrix: [[isize;4];3],
}

impl Transform {
    fn new() -> Self {
        Self {
            matrix: [
                [1,0,0,0],
                [0,1,0,0],
                [0,0,1,0],
            ],
        }
    }

    fn mul_trans(&self, other: Self) -> Self {
        let a = 
        self.matrix[0][0] * other.matrix[0][0]+ 
        self.matrix[0][1] * other.matrix[1][0]+ 
        self.matrix[0][2] * other.matrix[2][0];
        let b = 
        self.matrix[0][0] * other.matrix[0][1]+ 
        self.matrix[0][1] * other.matrix[1][1]+ 
        self.matrix[0][2] * other.matrix[2][1];
        let c = 
        self.matrix[0][0] * other.matrix[0][2]+ 
        self.matrix[0][1] * other.matrix[1][2]+ 
        self.matrix[0][2] * other.matrix[2][2];
        let d = 
        self.matrix[0][0] * other.matrix[0][3]+ 
        self.matrix[0][1] * other.matrix[1][3]+ 
        self.matrix[0][2] * other.matrix[2][3]+ 
        self.matrix[0][3];


        let e = 
        self.matrix[1][0] * other.matrix[0][0]+ 
        self.matrix[1][1] * other.matrix[1][0]+ 
        self.matrix[1][2] * other.matrix[2][0];
        let f = 
        self.matrix[1][0] * other.matrix[0][1]+ 
        self.matrix[1][1] * other.matrix[1][1]+ 
        self.matrix[1][2] * other.matrix[2][1];
        let g = 
        self.matrix[1][0] * other.matrix[0][2]+ 
        self.matrix[1][1] * other.matrix[1][2]+ 
        self.matrix[1][2] * other.matrix[2][2];
        let h = 
        self.matrix[1][0] * other.matrix[0][3]+ 
        self.matrix[1][1] * other.matrix[1][3]+ 
        self.matrix[1][2] * other.matrix[2][3]+ 
        self.matrix[1][3];


        let i = 
        self.matrix[2][0] * other.matrix[0][0]+ 
        self.matrix[2][1] * other.matrix[1][0]+ 
        self.matrix[2][2] * other.matrix[2][0];
        let j = 
        self.matrix[2][0] * other.matrix[0][1]+ 
        self.matrix[2][1] * other.matrix[1][1]+ 
        self.matrix[2][2] * other.matrix[2][1];
        let k = 
        self.matrix[2][0] * other.matrix[0][2]+ 
        self.matrix[2][1] * other.matrix[1][2]+ 
        self.matrix[2][2] * other.matrix[2][2];
        let l = 
        self.matrix[2][0] * other.matrix[0][3]+ 
        self.matrix[2][1] * other.matrix[1][3]+ 
        self.matrix[2][2] * other.matrix[2][3]+ 
        self.matrix[2][3];

        Self {
            matrix: [
                [a,b,c,d],
                [e,f,g,h],
                [i,j,k,l],
            ],
        }
    }

    fn mul_pos(&self, pos: Position) -> Position {
        let x = self.matrix[0][0] * pos.vector[0] + 
        self.matrix[0][1] * pos.vector[1] + 
        self.matrix[0][2] * pos.vector[2] +
        self.matrix[0][3];

        let y = self.matrix[1][0] * pos.vector[0] + 
        self.matrix[1][1] * pos.vector[1] + 
        self.matrix[1][2] * pos.vector[2] +
        self.matrix[1][3];

        let z = self.matrix[2][0] * pos.vector[0] + 
        self.matrix[2][1] * pos.vector[1] + 
        self.matrix[2][2] * pos.vector[2] +
        self.matrix[2][3];

        Position { vector: 
            [
                x,y,z
            ]
        }
    }

    fn rotate_x(i:isize) -> Transform {
        Self {
            matrix: [
                [1, 0,0,0],
                [0, 0,-i,0],
                [0, i,0,0],
            ],
        }
    }

    fn rotate_y(i:isize) -> Transform {
        Self {
            matrix: [
                [0,0,i,0],
                [0,1,0,0],
                [-i,0,0,0],
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    vector: [isize;3],
}

impl Position {
    fn new(x:isize,y:isize,z:isize,) -> Self {
        Self {
            vector: [x,y,z],
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
            Some((*direction, (target_x, target_y)))
        } else {
            None
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
            let _ = write!(f, "{row}");
        }
        write!(f, "")
    }
}
impl<'a> std::fmt::Display for State<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (r, row) in self.puzzle.rows.iter().enumerate() {
            for (c, field) in row.fields.iter().enumerate() {
                if let Some(d) = self.visited.get(&(c, r)) {
                    let _ = write!(f, "{d}");
                } else if self.position == (c, r) {
                    let _ = write!(f, "{}", self.direction);
                } else {
                    let _ = write!(f, "{field}");
                }
            }
            let _ = writeln!(f);
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
            portals: vec![],
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

struct Edge {
    from: (isize, isize),
    to: (isize, isize),
    direction: Direction,
}

struct ExtractedFaces {
    face_set: BTreeMap<(isize, isize), usize>,
    edges: Vec<Edge>,
}

fn extract_faces(puzzle: &Puzzle) -> ExtractedFaces {
    let (w, h) = puzzle.dimensions_2d();
    let side_length = num::integer::gcd(w, h) as isize;
    let sides_x = w as isize / side_length;
    let sides_y = h as isize / side_length;
    
    let mut face_set = BTreeMap::<(isize, isize), usize>::new();
    let mut edges = Vec::new();

    for y in 0..sides_y {
        for row in 0..side_length {
            for x in 0..sides_x {
                for col in 0..side_length {
                    let coord = ((x * side_length + col) as usize, ((y * side_length) + row) as usize);
                    if !puzzle.is_void_at(coord) {
                        let side_count = face_set.len();
                        face_set.entry((x, y)).or_insert(side_count);
                        edges.push(Edge {
                            from: (x, y),
                            to: (x, y + 1),
                            direction: Direction::Down,
                        });
                        edges.push(Edge {
                            from: (x, y + 1),
                            to: (x + 1, y + 1),
                            direction: Direction::Right,
                        });
                        edges.push(Edge {
                            from: (x + 1, y + 1),
                            to: (x + 1, y),
                            direction: Direction::Up,
                        });
                        edges.push(Edge {
                            from: (x + 1, y),
                            to: (x, y),
                            direction: Direction::Left,
                        });
                    }
                }
            }
        }
    }
    ExtractedFaces {
        face_set, edges
    }
}
fn detect_portals_geometrically(puzzle: &Puzzle)  -> Option<Vec<Portal>> {
    let (w, h) = puzzle.dimensions_2d();
    let side_length = num::integer::gcd(w, h) as isize;
    
    let ExtractedFaces{ mut face_set, edges} = extract_faces(puzzle);

    let Some((face_coord, _)) = face_set.pop_first() else {
        return None;
    };

    let mut corner_mapping = HashMap::new();
    let mut queue = VecDeque::new();
    queue.push_back((face_coord, Transform::new()));
    
    while let Some(((face_x,face_y), rotation)) = queue.pop_front() {
        let corners_2d = [
            (face_x, face_y, Position::new(-1,-1,-1)),
            (face_x, face_y + 1, Position::new(-1,1,-1)),
            (face_x + 1, face_y + 1, Position::new(1,1,-1)),
            (face_x + 1, face_y, Position::new(1,-1,-1))
        ];

        for (x,y, c3d) in corners_2d {
            corner_mapping.insert((x,y), rotation.mul_pos(c3d));
        }

        for d in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let (delta_x, delta_y) = d.get_delta();
            let next_face = (face_x + delta_x, face_y + delta_y);
            if face_set.remove(&next_face).is_some() {
                queue.push_back((next_face, rotation.mul_trans(d.get_rotation())));
            }
        }
    }
    let mut portals = Vec::<_>::new();
    let mut colored_edges =
        HashMap::<(&Position, &Position), (Direction, (isize, isize), (isize, isize))>::new();
    
    for edge in edges {
        let from_position = corner_mapping.get(&edge.from)?;
        let to_position = corner_mapping.get(&edge.to)?;
        let edge_position = (from_position, to_position);
        let back_edge = (to_position, from_position);
        if let Some((other_edge_direction, ref other_to, ref other_from)) =
            colored_edges.remove(&back_edge)
        {
            if (other_to, other_from) == (&edge.to, &edge.from) {
                continue;
            }

            let entrance_direction = edge.direction.turn_cw();
            let exit_direction = other_edge_direction.turn_ccw();

            let offset: (isize, isize) = match entrance_direction {
                Direction::Right => (-1, 0),
                Direction::Down => (0, -1),
                Direction::Left => (0, 0),
                Direction::Up => (0, 0),
            };

            let offset_other: (isize, isize) = match exit_direction {
                Direction::Right => (0, 0),
                Direction::Down => (0, 0),
                Direction::Left => (-1, 0),
                Direction::Up => (0, -1),
            };
            
            let (from_x, from_y) = edge.from;
            let (to_x, to_y) = edge.to;
            let (other_from_x, other_from_y) = other_from;
            let (other_to_x, other_to_y) = other_to;
            let (offset_x, offset_y) = offset;
            let (offset_other_x, offset_other_y) = offset_other;

            let start_x_sign = if from_x > to_x { -1 } else { 0 };
            let start_y_sign = if from_y > to_y { -1 } else { 0 };
            let end_x_sign = if to_x > from_x { -1 } else { 0 };
            let end_y_sign = if to_y > from_y { -1 } else { 0 };

            let other_start_x_sign = if other_from_x > other_to_x { -1 } else { 0 };
            let other_start_y_sign = if other_from_y > other_to_y { -1 } else { 0 };
            let other_end_x_sign = if other_to_x > other_from_x { -1 } else { 0 };
            let other_end_y_sign = if other_to_y > other_from_y { -1 } else { 0 };

            let portal = Portal {
                entrance_start: (
                    (start_x_sign + from_x * side_length + offset_x) as usize,
                    (start_y_sign + from_y * side_length + offset_y) as usize,
                ),
                entrance_end: (
                    (end_x_sign + to_x * side_length + offset_x) as usize,
                    (end_y_sign + to_y * side_length + offset_y) as usize,
                ),
                entrance_direction,
                exit_start: (
                    (other_start_x_sign
                        + other_from_x * side_length
                        + offset_other_x) as usize,
                    (other_start_y_sign
                        + other_from_y * side_length
                        + offset_other_y) as usize,
                ),
                exit_end: (
                    (other_end_x_sign
                        + other_to_x * side_length
                        + offset_other_x) as usize,
                    (other_end_y_sign
                        + other_to_y * side_length
                        + offset_other_y) as usize,
                ),
                exit_direction,
            };
            portals.push(portal);
            portals.push(portal.inverse());
        } else {
            colored_edges.insert(edge_position, (edge.direction, edge.from, edge.to));
            colored_edges.insert(back_edge, (edge.direction.opposite(), edge.to, edge.from));
        }
    }
    
    Some(portals)
}

pub fn process_with_portals(input: String) -> Option<usize> {
    let (_, puzzle) = puzzle(&input).ok()?;

    let auto_portals_geo = detect_portals_geometrically(&puzzle)?;

    let puzzle = Puzzle {
        rows: puzzle.rows,
        steps: puzzle.steps,
        portals: auto_portals_geo,
    };
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
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process(COMMANDS.to_string()), Some(6032));
        assert_eq!(process_with_portals(COMMANDS.to_string()), Some(5031));
    }

    #[test]
    fn test_transform() {
        let rot_x = Transform::rotate_x(1);
        assert_eq!(rot_x.mul_pos(Position { vector: [0,0,0] }), Position { vector: [0,0,0] });
        assert_eq!(rot_x.mul_pos(Position { vector: [0,1,0] }), Position { vector: [0,0,1] });
        assert_eq!(rot_x.mul_pos(Position { vector: [0,0,1] }), Position { vector: [0,-1,0] });
        assert_eq!(rot_x.mul_pos(Position { vector: [0,-1,0] }), Position { vector: [0,0,-1] });
        assert_eq!(rot_x.mul_pos(Position { vector: [0,0,-1] }), Position { vector: [0,1,0] });


        let rot_x_neg = Transform::rotate_x(-1);
        assert_eq!(rot_x_neg.mul_pos(Position { vector: [0,0,0] }), Position { vector: [0,0,0] });
        assert_eq!(rot_x_neg.mul_pos(Position { vector: [0,1,0] }), Position { vector: [0,0,-1] });
        assert_eq!(rot_x_neg.mul_pos(Position { vector: [0,0,1] }), Position { vector: [0,1,0] });
        assert_eq!(rot_x_neg.mul_pos(Position { vector: [0,-1,0] }), Position { vector: [0,0,1] });
        assert_eq!(rot_x_neg.mul_pos(Position { vector: [0,0,-1] }), Position { vector: [0,-1,0] });
        

        let rot_y = Transform::rotate_y(1);
        assert_eq!(rot_y.mul_pos(Position { vector: [0,0,0] }), Position { vector: [0,0,0] });
        assert_eq!(rot_y.mul_pos(Position { vector: [1,0,0] }), Position { vector: [0,0,-1] });
        assert_eq!(rot_y.mul_pos(Position { vector: [0,0,1] }), Position { vector: [1,0,0] });
        assert_eq!(rot_y.mul_pos(Position { vector: [-1,0,0] }), Position { vector: [0,0,1] });
        assert_eq!(rot_y.mul_pos(Position { vector: [0,0,-1] }), Position { vector: [-1,0,0] });
    }
}
