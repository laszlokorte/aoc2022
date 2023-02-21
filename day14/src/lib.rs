#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::*;
use std::fmt;

#[derive(Debug, PartialOrd, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: u32,
    y: u32,
}

impl From<(u32, u32)> for Point {
    fn from((x, y): (u32, u32)) -> Self {
        Point { x, y }
    }
}

#[derive(Debug)]
enum Line {
    Vertical(Point, u32),
    Horizontal(Point, u32),
}

impl Line {
    fn points(&self) -> Vec<Point> {
        match self {
            Self::Vertical(Point { x, y: y0 }, y1) => (u32::min(*y0, *y1)..=u32::max(*y0, *y1))
                .map(|y| Point { x: *x, y })
                .collect(),
            Self::Horizontal(Point { x: x0, y }, x1) => (u32::min(*x0, *x1)..=u32::max(*x0, *x1))
                .map(|x| Point { x, y: *y })
                .collect(),
        }
    }
}

fn connect(p1: &Point, p2: &Point) -> Option<Line> {
    if p1.x == p2.x {
        Some(Line::Vertical(*p1, p2.y))
    } else if p1.y == p2.y {
        Some(Line::Horizontal(*p1, p2.x))
    } else {
        None
    }
}

#[derive(Debug)]
struct Path {
    points: Vec<Point>,
}

impl From<Vec<Point>> for Path {
    fn from(points: Vec<Point>) -> Self {
        Path { points }
    }
}

enum CellContent {
    Stone,
    Sand,
    Source,
}

struct Cave {
    source: Point,
    grid: std::collections::HashMap<Point, CellContent>,
    floor: Option<u32>,
}

fn path(input: &str) -> IResult<&str, Path> {
    map(
        separated_list1(
            tag(" -> "),
            separated_pair(character::complete::u32, tag(","), character::complete::u32)
                .map(Point::from),
        ),
        Path::from,
    )(input)
}
fn paths(input: &str) -> IResult<&str, Vec<Path>> {
    // 498,4 -> 498,6 -> 496,6
    separated_list1(line_ending, path)(input)
}

impl Cave {
    fn insert_floor(&mut self, offset: u32) {
        self.floor = Some(self.grid.keys().map(|p| p.y).max().unwrap_or_default() + offset);
    }

    fn fill(&mut self) -> Option<usize> {
        let mut counter = 0;
        self.grid.insert(self.source, CellContent::Source);
        let max_y: u32 = self.grid.keys().map(|p| p.y).chain(self.floor).max()?;
        let mut sand_position = self.source;
        while sand_position.y <= max_y {
            if let Some(next_pos) = self.tick(sand_position) {
                sand_position = next_pos;
            } else {
                counter += 1;
                if self.grid.insert(sand_position, CellContent::Sand).is_some() {
                    break;
                }
                sand_position = self.source;
            }
        }

        Some(counter)
    }

    fn tick(&self, pos: Point) -> Option<Point> {
        let down = Point {
            x: pos.x,
            y: pos.y + 1,
        };
        let down_left = Point {
            x: pos.x - 1,
            y: pos.y + 1,
        };
        let down_right = Point {
            x: pos.x + 1,
            y: pos.y + 1,
        };
        for p in [down, down_left, down_right] {
            if let Some(g) = self.floor && p.y == g {
                return None;
            }
            if !self.grid.contains_key(&p) {
                return Some(p);
            }
        }
        None
    }
}

impl fmt::Display for Cave {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let min_x = self.grid.keys().map(|p| p.x).min().unwrap_or_default();
        let max_x = self.grid.keys().map(|p| p.x).max().unwrap_or_default();
        let min_y = self.grid.keys().map(|p| p.y).min().unwrap_or_default();
        let max_y = self
            .grid
            .keys()
            .map(|p| p.y)
            .chain(self.floor)
            .max()
            .unwrap_or_default();

        write!(
            f,
            "{}",
            (min_y..=max_y)
                .map(|y| {
                    (min_x..=max_x)
                        .map(|x| {
                            match self.grid.get(&Point { x, y }) {
                            Some(CellContent::Stone) => "#",
                            Some(CellContent::Sand) => "o",
                            Some(CellContent::Source) => "+",
                            None => {
                                if let Some(f) = self.floor && f == y {
                                    "#"
                                } else {
                                    "."
                                }
                            }
                        }
                        })
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

pub fn process(input: String, floor: bool) -> Option<(usize, String)> {
    let (_, ref pts) = paths(&input).ok()?;
    let mut grid = std::collections::HashMap::<Point, CellContent>::new();

    let segments = pts
        .iter()
        .flat_map(|path| path.points.array_windows().map(|[a, b]| connect(a, b)))
        .collect::<Option<Vec<Line>>>()?;

    for segment in segments {
        for point in segment.points() {
            grid.insert(point, CellContent::Stone);
        }
    }
    let mut cave = Cave {
        grid,
        source: Point { x: 500, y: 0 },
        floor: None,
    };

    if floor {
        cave.insert_floor(2);
    }

    Some((cave.fill()?, format!("{cave}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(
            process(COMMANDS.to_string(), false),
            Some((
                24,
                include_str!("assert-1.txt").to_string()
            ))
        );
        assert_eq!(
            process(COMMANDS.to_string(), true),
            Some((
                93,
                include_str!("assert-2.txt").to_string()
            ))
        );
    }
}
