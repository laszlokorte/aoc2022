#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Marker {
    Start,
    End,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Field {
    height: u32,
    marker: Option<Marker>,
}

impl From<char> for Field {
    fn from(value: char) -> Self {
        match value {
            'S' => Self {
                height: 0,
                marker: Some(Marker::Start),
            },
            'E' => Self {
                height: 25,
                marker: Some(Marker::End),
            },
            _ => Self {
                height: value as u32 - 'a' as u32,
                marker: None,
            },
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
struct Step {
    dx: i32,
    dy: i32,
}

impl From<(i32, i32)> for Step {
    fn from((dx, dy): (i32, i32)) -> Self {
        Step { dx, dy }
    }
}

pub struct Policy {
    moves: [Step; 4],
    max_up: Option<i32>,
    min_up: Option<i32>,
}

impl Policy {
    pub const UPHILL: Policy = Policy {
        moves: [
            Step { dx: 0, dy: 1 },
            Step { dx: 0, dy: -1 },
            Step { dx: 1, dy: 0 },
            Step { dx: -1, dy: 0 },
        ],
        max_up: Some(1),
        min_up: None,
    };
    pub const DOWNHILL: Policy = Policy {
        moves: [
            Step { dx: 0, dy: 1 },
            Step { dx: 0, dy: -1 },
            Step { dx: 1, dy: 0 },
            Step { dx: -1, dy: 0 },
        ],
        max_up: None,
        min_up: Some(-1),
    };

    fn allows(&self, from: Field, to: Field) -> bool {
        let height_difference = to.height as i32 - from.height as i32;
        match (self.min_up, self.max_up) {
            (Some(u), Some(d)) => (d..=u).contains(&height_difference),
            (Some(u), None) => (u..).contains(&height_difference),
            (None, Some(d)) => (-100..=d).contains(&height_difference),
            _ => true,
        }
    }
}

pub enum SearchTerm {
    Height(u32),
    Marker(Marker),
}

impl SearchTerm {
    fn matches(&self, field: &Field) -> bool {
        match self {
            Self::Height(h) => h == &field.height,
            Self::Marker(m) => Some(*m) == field.marker,
        }
    }
}

#[derive(Debug)]
struct Grid<'a> {
    fields: &'a Vec<Vec<Field>>,
}

impl<'a> Grid<'a> {
    fn find_marker(&self, marker: Marker) -> Option<(usize, usize)> {
        self.fields.iter().enumerate().find_map(|(r, row)| {
            row.iter().enumerate().find_map(|(c, col)| {
                if col.marker == Some(marker) {
                    Some((r, c))
                } else {
                    None
                }
            })
        })
    }
    fn neighbour_positions(
        &'a self,
        policy: &'a Policy,
        pos: &'a (usize, usize),
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        policy
            .moves
            .iter()
            .flat_map(move |s| self.relative_position(&policy, pos, s))
    }

    fn relative_position(
        &self,
        policy: &Policy,
        pos: &(usize, usize),
        step: &Step,
    ) -> Option<(usize, usize)> {
        let new_column = (pos.1 as i32 + step.dx) as usize;
        let new_row = (pos.0 as i32 + step.dy) as usize;

        if new_row < self.fields.len()
            && new_column < self.fields[new_row].len()
            && policy.allows(self.fields[pos.0][pos.1], self.fields[new_row][new_column])
        {
            Some((new_row, new_column))
        } else {
            None
        }
    }

    fn field_at(&self, (row, col): (usize, usize)) -> Option<&Field> {
        self.fields.get(row)?.get(col)
    }
}

pub fn process(
    input: String,
    policy: Policy,
    start: Marker,
    search_term: SearchTerm,
) -> Option<u32> {
    let cells = input
        .lines()
        .map(|l| l.chars().map(Field::from).collect())
        .collect::<Vec<Vec<_>>>();
    let grid = Grid { fields: &cells };
    let start = grid.find_marker(start)?;
    let mut seen = HashSet::new();
    let mut predecessor = HashMap::<(usize, usize), (usize, usize)>::new();
    let mut queue = VecDeque::<(usize, usize)>::new();
    queue.push_back(start);
    seen.insert(start);
    let goal = 'find: {
        while let Some(field) = queue.pop_front() {
            if search_term.matches(grid.field_at(field)?) {
                break 'find field;
            }
            for n in grid.neighbour_positions(&policy, &field) {
                if seen.insert(n) {
                    predecessor.insert(n, field);
                    queue.push_back(n)
                }
            }
        }
        return None;
    };

    let mut count = 0;
    let mut current = &goal;
    while let Some(pred) = &predecessor.get(&current) {
        count += 1;
        current = *pred;
    }

    Some(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "\
        Sabqponm\n\
        abcryxxl\n\
        accszExk\n\
        acctuvwj\n\
        abdefghi";

        assert_eq!(
            process(
                COMMANDS.to_string(),
                Policy::UPHILL,
                Marker::Start,
                SearchTerm::Marker(Marker::End)
            ),
            Some(31)
        );
        assert_eq!(
            process(
                COMMANDS.to_string(),
                Policy::DOWNHILL,
                Marker::End,
                SearchTerm::Height(0)
            ),
            Some(29)
        );
    }
}
