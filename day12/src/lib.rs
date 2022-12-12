#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Marker {
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

struct Grid<'a> {
    fields: &'a Vec<Vec<Field>>,
}

impl<'a> Grid<'a> {
    const moves: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
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
    fn neighbour_positions<'b: 'a>(
        &'a self,
        pos: &'b (usize, usize),
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        Self::moves
            .iter()
            .copied()
            .flat_map(|(h, v)| self.relative_position(pos, (v, h)))
    }

    fn relative_position(&self, pos: &(usize, usize), delta: (i32, i32)) -> Option<(usize, usize)> {
        let new_column = (pos.1 as i32 + delta.1) as usize;
        let new_row = (pos.0 as i32 + delta.0) as usize;

        if new_column >= 0
            && new_row >= 0
            && new_row < self.fields.len()
            && new_column < self.fields[new_row].len()
        {
            Some((new_row, new_column))
        } else {
            None
        }
    }
}
pub fn process(input: String) -> Option<u32> {
    let cells = input
        .lines()
        .map(|l| l.chars().map(Field::from).collect())
        .collect::<Vec<Vec<_>>>();
    let grid = Grid { fields: &cells };
    let start = grid.find_marker(Marker::Start);
    let end = grid.find_marker(Marker::End);
    dbg!(start);
    dbg!(end);

    for ref n in grid.neighbour_positions(&start?) {
        dbg!(n);
    }
    None
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

        assert_eq!(process(COMMANDS.to_string()), Some(42));
    }
}
