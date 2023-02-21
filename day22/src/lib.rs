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
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

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

    fn get_translation(&self) -> Transform {
        match self {
            Direction::Right => Transform::translate(2, 0, 0),
            Direction::Down => Transform::translate(0, -2, 0),
            Direction::Left => Transform::translate(-2, 0, 0),
            Direction::Up => Transform::translate(0, 2, 0),
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

#[derive(Debug)]
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

    fn translate(x:isize, y:isize, z:isize) -> Self {
        Self {
            matrix: [
                [1,0,0,x],
                [0,1,0,y],
                [0,0,1,z],
            ],
        }
    }

    fn mul_trans(&self, other: &Self) -> Self {
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

#[derive(Hash, Debug, PartialEq, Eq, Copy, Clone)]
enum CubeColors {
    Gray,
    Red,
    Green,
    Blue,
    White,
    Magenta,
    Cyan,
    Yellow,
    Black,
}

impl std::fmt::Display for CubeColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            CubeColors::Red => write!(f, "red"),
            CubeColors::Green => write!(f, "green"),
            CubeColors::Blue => write!(f, "blue"),
            CubeColors::White => write!(f, "white"),
            CubeColors::Magenta => write!(f, "magenta"),
            CubeColors::Cyan => write!(f, "cyan"),
            CubeColors::Yellow => write!(f, "yellow"),
            CubeColors::Black => write!(f, "black"),
            CubeColors::Gray => write!(f, "gray"),
        }
    }
}

pub struct CubeNet {
    node_degrees: HashMap<(usize, usize), usize>,
    color_degrees: HashMap<CubeColors, usize>,
    corners: BTreeMap<(usize, usize), Option<CubeColors>>,
    edges: HashMap<(usize, usize), BTreeSet<(usize, usize)>>,
    unused_colors: VecDeque<CubeColors>,
}

impl CubeNet {
    fn new() -> Self {
        Self {
            node_degrees: HashMap::<(usize, usize), usize>::new(),
            color_degrees: HashMap::<CubeColors, usize>::new(),
            corners: BTreeMap::new(),
            edges: HashMap::new(),
            unused_colors: [
                CubeColors::Red,
                CubeColors::Green,
                CubeColors::Blue,
                CubeColors::White,
                CubeColors::Cyan,
                CubeColors::Magenta,
                CubeColors::Yellow,
                CubeColors::Black,
            ]
            .into(),
        }
    }

    pub fn to_graphviz(&self) {
        for ((cx, cy), color) in &self.corners {
            println!(
                "\"{cx},{cy}\"[pos=\"{cx},{cy}\",style=filled,fillcolor={}];",
                color.unwrap_or(CubeColors::Gray)
            );
        }
        for ((ax, ay), nei) in &self.edges {
            for (bx, by) in nei {
                println!("\"{ax},{ay}\"->\"{bx},{by}\";");
            }
        }
    }

    fn set_color(&mut self, node: (usize, usize), color: Option<CubeColors>) {
        if let Some(node_color) = self.corners.get_mut(&node) {
            if let Some(old_color) = node_color && let Some(node_degree) = self.node_degrees.get(&node){
                *self.color_degrees.entry(*old_color).or_insert(0) += node_degree;
            }
            *node_color = color;
            if let Some(new_color) = color && let Some(node_degree) =self.node_degrees.get(&node) {
                *self.color_degrees.entry(new_color).or_insert(0) += node_degree;
            }
        }
    }

    fn set_color_same_as(&mut self, node: (usize, usize), other_node: (usize, usize)) {
        if let Some(&color) = self.corners.get(&other_node) {
            self.set_color(node, color);
        }
    }

    fn add_face(&mut self, [a, b, c, d]: [(usize, usize); 4]) {
        self.corners.insert(a, None);
        self.corners.insert(b, None);
        self.corners.insert(c, None);
        self.corners.insert(d, None);
        *self.node_degrees.entry(a).or_insert(0) += 1;
        *self.node_degrees.entry(b).or_insert(0) += 1;
        *self.node_degrees.entry(c).or_insert(0) += 1;
        *self.node_degrees.entry(d).or_insert(0) += 1;

        let _ = self.edges.try_insert(a, BTreeSet::new());
        let _ = self.edges.try_insert(b, BTreeSet::new());
        let _ = self.edges.try_insert(c, BTreeSet::new());
        let _ = self.edges.try_insert(d, BTreeSet::new());
        let [edges_a, edges_b, edges_c, edges_d] =
            self.edges.get_many_mut([&a, &b, &c, &d]).unwrap();

        edges_a.insert(b);
        edges_b.insert(a);
        edges_b.insert(c);
        edges_c.insert(b);
        edges_c.insert(d);
        edges_d.insert(c);
        edges_d.insert(a);
        edges_a.insert(d);
    }

    fn assign_new_color(&mut self, node: (usize, usize)) {
        let color = self.unused_colors.pop_front();
        self.set_color(node, color);
    }

    fn set_initial_colors(&mut self) {
        let degrees = self.node_degrees.clone();
        for (&node, deg) in &degrees {
            match deg {
                3 | 2 => {
                    let color = self.unused_colors.pop_front();
                    self.set_color(node, color);
                }
                1 => {
                    continue;
                }
                _ => panic!("unexpected node degree"),
            }
        }
    }

    fn set_diagonal_colors(&mut self) {
        let sides_y = self.corners.keys().map(|c| c.1).max().unwrap_or(0);
        let sides_x = self.corners.keys().map(|c| c.0).max().unwrap_or(0);
        'try_extend_color: loop {
            let mut double_uncolored_corners = HashMap::<(usize, usize), usize>::new();
            for y in 0..sides_y {
                for x in 0..sides_x {
                    let local_corners = [(x, y), (x, y + 1), (x + 1, y + 1), (x + 1, y)];
                    let void_corners = local_corners
                        .iter()
                        .filter(|c| !self.corners.contains_key(c))
                        .count();
                    let uncolored_corners = local_corners.iter().filter(|c| {
                        self.corners
                            .get(c)
                            .map(|col| col.is_none())
                            .unwrap_or(false)
                    });
                    let colored_l_corner = local_corners
                        .iter()
                        .filter(|c| self.corners.get(c).map(|c| c.is_some()).unwrap_or(true))
                        .find(|c| self.edges.get(c).map(|e| e.len() != 4).unwrap_or(false));

                    if void_corners == 1 && let Some(&uncolored) = uncolored_corners.clone().next() && let Some(&colored) = colored_l_corner {
                    self.set_color_same_as(uncolored, colored);
                } else if void_corners == 1 && colored_l_corner.is_none()  && uncolored_corners.clone().count() == 2 {
                    for &dbl in uncolored_corners {
                        *double_uncolored_corners.entry(dbl).or_insert(0) += 1;
                    }
                }
                }
            }

            if let Some(&double_gray) =
                double_uncolored_corners
                    .iter()
                    .find_map(|(c, &count)| if count > 0 { Some(c) } else { None })
            {
                self.assign_new_color(double_gray);
                continue 'try_extend_color;
            }
            break;
        }
    }

    fn set_distant_colors(&mut self) {
        loop {
            let Some((corn, new_color)) = ('find_color: {
                let mut count_gray = 0;
                for (corner, color) in &self.corners {
                    if color.is_none() {
                        let neighbours = self.edges.get(corner).unwrap();
                        let neigbour_colors = neighbours.iter().flat_map(|n| self.corners.get(n));
                        let nodes_of_color = neigbour_colors.flat_map(|&c| {
                            self.corners
                                .iter()
                                .filter_map(move |(n, &nc)| if c.is_some() && nc == c { Some(n) } else { None })
                        });
                        let nodes_touching_color: BTreeSet<_> = nodes_of_color
                            .clone()
                            .flat_map(|n| {
                                self.edges
                                    .get(n)
                                    .unwrap()
                                    .iter()
                                    .filter(|c| self.corners.get(c).is_some())
                            })
                            .filter(|n| {
                                if *n == corner {
                                    return false;
                                }
                                let Some(col) = self.corners.get(n).unwrap() else {
                                    return false;
                                };
    
                                let total_degree = self.color_degrees.get(col).unwrap_or(&0);
                                total_degree < &3
                            })
                            .collect();
    
                        if nodes_touching_color.len() == 1 {
                            let touching_node = *nodes_touching_color.iter().next().unwrap();
                            let new_color = *self.corners.get(touching_node).unwrap();
                            break 'find_color Some((corner, new_color));
                        }
                        count_gray += 1;
                    }
                }

                if count_gray == 1 {
                    let gray_corner = *self.corners.iter().find_map(|(corn, col)| if col.is_none() {Some(corn)} else {None}).unwrap();
                    let color_left = self.color_degrees.iter().find_map(|(col,deg)| if deg != &3 {Some(col)} else {None}).cloned();
                    self.set_color(gray_corner, color_left);
                }

                return;
            }) else {
                continue;
            };
    
            self.set_color(*corn, new_color);
        }
        
    }

    fn set_leftover_colors(&mut self) {
        if self.unused_colors.len() == 1 {
            let corners_cloned = self.corners.clone();
            let grays = corners_cloned
                .iter()
                .filter_map(|(corn, col)| if col.is_none() { Some(corn) } else { None })
                .collect::<Vec<_>>();
            if grays.len() == 3 {
                let remaining_color = self.unused_colors.pop_front();
                for node in grays {
                    self.corners.insert(*node, remaining_color);
                }
            }
        }
    }
}

fn detect_portals_geometrically(puzzle: &Puzzle)  -> Option<Vec<Portal>> {
    let (w, h) = puzzle.dimensions_2d();
    let side_length = num::integer::gcd(w, h);
    let sides_x = w / side_length;
    let sides_y = h / side_length;
    let mut side_set = BTreeMap::<(isize, isize), usize>::new();
    let mut edges = Vec::new();

    for y in 0..sides_y {
        for row in 0..side_length {
            for x in 0..sides_x {
                for col in 0..side_length {
                    let coord = (x * side_length + col, (y * side_length) + row);
                    if !puzzle.is_void_at(coord) {
                        let side_count = side_set.len();
                        side_set.entry((x as isize, y as isize)).or_insert(side_count);
                        edges.push(((x as isize, y as isize), (x as isize, y as isize + 1), Direction::Down));
                        edges.push(((x as isize, y as isize + 1), (x as isize + 1, y as isize + 1), Direction::Right));
                        edges.push(((x as isize + 1, y as isize + 1), (x as isize + 1, y as isize), Direction::Up));
                        edges.push(((x as isize + 1, y as isize), (x as isize, y as isize), Direction::Left));
                    }
                }
            }
        }
    }

    let Some((face_coord, _)) = side_set.pop_first() else {
        return None;
    };

    let mut queue = VecDeque::new();
    let (init_x, init_y) = face_coord;

    queue.push_back((face_coord, Transform::translate(-(init_x)*2-1, -(init_y)*2-1, 0), Transform::new()));
    
    let mut corner_mapping = HashMap::new();

    while let Some(((face_x,face_y), translation, rotation)) = queue.pop_front() {
        let corners_2d = [
            (face_x, face_y),
            (face_x, face_y + 1),
            (face_x + 1, face_y + 1),
            (face_x + 1, face_y)
        ];

        for (x,y) in corners_2d {
            let c3d = Position::new(2*x,2*y,-1);
            let pos3d = rotation.mul_pos(translation.mul_pos(c3d));
            corner_mapping.insert((x,y), pos3d);
        }

        for d in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            let (delta_x, delta_y) = d.get_delta();
            let next_face = (face_x + delta_x, face_y + delta_y);
            if side_set.remove(&next_face).is_some() {
                queue.push_back((next_face, translation.mul_trans(&d.get_translation()), rotation.mul_trans(&d.get_rotation())));
            }
        }
    }
    let mut portals = Vec::<_>::new();
    let mut colored_edges =
        HashMap::<(Position, Position), (Direction, (isize, isize), (isize, isize))>::new();
    
    for (from, to, edge_direction) in edges {
        let from_color = *corner_mapping.get(&from)?;
        let to_color = *corner_mapping.get(&to)?;
        let edge_color = (from_color, to_color);
        let back_edge = (edge_color.1, edge_color.0);
        if let Some((other_edge_direction, other_to, other_from)) =
            colored_edges.remove(&back_edge)
        {
            if (other_to, other_from) == (to, from) {
                continue;
            }

            let entrance_direction = edge_direction.turn_cw();
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
            let start_x_sign = if from.0 > to.0 { -1 } else { 0 };
            let start_y_sign = if from.1 > to.1 { -1 } else { 0 };
            let end_x_sign = if to.0 > from.0 { -1 } else { 0 };
            let end_y_sign = if to.1 > from.1 { -1 } else { 0 };

            let other_start_x_sign = if other_from.0 > other_to.0 { -1 } else { 0 };
            let other_start_y_sign = if other_from.1 > other_to.1 { -1 } else { 0 };
            let other_end_x_sign = if other_to.0 > other_from.0 { -1 } else { 0 };
            let other_end_y_sign = if other_to.1 > other_from.1 { -1 } else { 0 };

            let portal = Portal {
                entrance_start: (
                    (start_x_sign + from.0 * side_length as isize + offset.0) as usize,
                    (start_y_sign + from.1 * side_length as isize + offset.1) as usize,
                ),
                entrance_end: (
                    (end_x_sign + to.0 * side_length as isize + offset.0) as usize,
                    (end_y_sign + to.1 * side_length as isize + offset.1) as usize,
                ),
                entrance_direction,
                exit_start: (
                    (other_start_x_sign
                        + other_from.0 * side_length as isize
                        + offset_other.0) as usize,
                    (other_start_y_sign
                        + other_from.1 * side_length as isize
                        + offset_other.1) as usize,
                ),
                exit_end: (
                    (other_end_x_sign
                        + other_to.0 * side_length as isize
                        + offset_other.0) as usize,
                    (other_end_y_sign
                        + other_to.1 * side_length as isize
                        + offset_other.1) as usize,
                ),
                exit_direction,
            };
            portals.push(portal);
            portals.push(portal.inverse());
        } else {
            colored_edges.insert(edge_color, (edge_direction, from, to));
            colored_edges.insert(back_edge, (edge_direction.opposite(), to, from));
        }
    }
    
    Some(portals)
}

fn detect_portals_topological(puzzle: &Puzzle) -> Option<Vec<Portal>> {
    let (w, h) = puzzle.dimensions_2d();
    let side_length = num::integer::gcd(w, h);
    let sides_x = w / side_length;
    let sides_y = h / side_length;
    let mut side_set = BTreeMap::<(usize, usize), usize>::new();
    let mut net = CubeNet::new();


    for y in 0..sides_y {
        for row in 0..side_length {
            for x in 0..sides_x {
                for col in 0..side_length {
                    let coord = (x * side_length + col, (y * side_length) + row);
                    if !puzzle.is_void_at(coord) {
                        let side_count = side_set.len();
                        side_set.entry((x, y)).or_insert(side_count);
                    }
                }
            }
        }
    }

    for &(x, y) in side_set.keys() {
        let a = (x, y);
        let b = (x, y + 1);
        let c = (x + 1, y + 1);
        let d = (x + 1, y);
        net.add_face([a, b, c, d]);
    }

    net.set_initial_colors();
    net.set_diagonal_colors();
    net.set_distant_colors();
    net.set_leftover_colors();

    let mut portals = Vec::<_>::new();
    let mut colored_edges =
        HashMap::<(CubeColors, CubeColors), (Direction, (usize, usize), (usize, usize))>::new();
    for &(x, y) in side_set.keys() {
        let a = (x, y);
        let b = (x, y + 1);
        let c = (x + 1, y + 1);
        let d = (x + 1, y);

        let ccw_edges = [
            (a, b, Direction::Down),
            (b, c, Direction::Right),
            (c, d, Direction::Up),
            (d, a, Direction::Left),
        ];

        for (from, to, edge_direction) in ccw_edges {
            let from_color = *net.corners.get(&from)?;
            let to_color = *net.corners.get(&to)?;
            let edge_color = (from_color?, to_color?);
            let back_edge = (edge_color.1, edge_color.0);
            if let Some((other_edge_direction, other_to, other_from)) =
                colored_edges.remove(&back_edge)
            {
                if (other_to, other_from) == (to, from) {
                    continue;
                }

                let entrance_direction = edge_direction.turn_cw();
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
                let start_x_sign = if from.0 > to.0 { -1 } else { 0 };
                let start_y_sign = if from.1 > to.1 { -1 } else { 0 };
                let end_x_sign = if to.0 > from.0 { -1 } else { 0 };
                let end_y_sign = if to.1 > from.1 { -1 } else { 0 };

                let other_start_x_sign = if other_from.0 > other_to.0 { -1 } else { 0 };
                let other_start_y_sign = if other_from.1 > other_to.1 { -1 } else { 0 };
                let other_end_x_sign = if other_to.0 > other_from.0 { -1 } else { 0 };
                let other_end_y_sign = if other_to.1 > other_from.1 { -1 } else { 0 };

                let portal = Portal {
                    entrance_start: (
                        (start_x_sign + from.0 as isize * side_length as isize + offset.0) as usize,
                        (start_y_sign + from.1 as isize * side_length as isize + offset.1) as usize,
                    ),
                    entrance_end: (
                        (end_x_sign + to.0 as isize * side_length as isize + offset.0) as usize,
                        (end_y_sign + to.1 as isize * side_length as isize + offset.1) as usize,
                    ),
                    entrance_direction,
                    exit_start: (
                        (other_start_x_sign
                            + other_from.0 as isize * side_length as isize
                            + offset_other.0) as usize,
                        (other_start_y_sign
                            + other_from.1 as isize * side_length as isize
                            + offset_other.1) as usize,
                    ),
                    exit_end: (
                        (other_end_x_sign
                            + other_to.0 as isize * side_length as isize
                            + offset_other.0) as usize,
                        (other_end_y_sign
                            + other_to.1 as isize * side_length as isize
                            + offset_other.1) as usize,
                    ),
                    exit_direction,
                };
                portals.push(portal);
                portals.push(portal.inverse());
            } else {
                colored_edges.insert(edge_color, (edge_direction, from, to));
                colored_edges.insert(back_edge, (edge_direction.opposite(), to, from));
            }
        }
    }

    Some(portals)
}

pub fn process_with_portals(input: String, geo:bool) -> Option<usize> {
    let (_, puzzle) = puzzle(&input).ok()?;

    let auto_portals_geo = detect_portals_geometrically(&puzzle)?;
    let auto_portals_topo = detect_portals_topological(&puzzle)?;

    let puzzle = Puzzle {
        rows: puzzle.rows,
        steps: puzzle.steps,
        portals: if geo { auto_portals_geo } else { auto_portals_topo },
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
        assert_eq!(process_with_portals(COMMANDS.to_string(), true), Some(5031));
        assert_eq!(process_with_portals(COMMANDS.to_string(), false), Some(5031));
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
