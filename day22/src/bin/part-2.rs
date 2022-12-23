use std::fs;

use day22::process_with_portals;
use day22::{Direction, Portal};

fn main() {
    let file = fs::read_to_string("input.txt");

    if let Ok(content) = file {
        let portals = vec![
            Portal {
                entrance_start: (50, 0),
                entrance_end: (99, 0),
                entrance_direction: Direction::Up,
                exit_start: (0, 150),
                exit_end: (0, 199),
                exit_direction: Direction::Right,
            },
            Portal {
                entrance_start: (100, 0),
                entrance_end: (149, 0),
                entrance_direction: Direction::Up,
                exit_start: (0, 199),
                exit_end: (49, 199),
                exit_direction: Direction::Up,
            },
            Portal {
                entrance_start: (149, 0),
                entrance_end: (149, 49),
                entrance_direction: Direction::Right,
                exit_start: (99, 149),
                exit_end: (99, 100),
                exit_direction: Direction::Left,
            },
            Portal {
                entrance_start: (149, 49),
                entrance_end: (100, 49),
                entrance_direction: Direction::Down,
                exit_start: (99, 99),
                exit_end: (99, 50),
                exit_direction: Direction::Left,
            },
            Portal {
                entrance_start: (50, 50),
                entrance_end: (50, 99),
                entrance_direction: Direction::Left,
                exit_start: (0, 100),
                exit_end: (49, 100),
                exit_direction: Direction::Down,
            },
            Portal {
                entrance_start: (50, 0),
                entrance_end: (50, 50),
                entrance_direction: Direction::Left,
                exit_start: (0, 149),
                exit_end: (0, 100),
                exit_direction: Direction::Right,
            },
            Portal {
                entrance_start: (50, 149),
                entrance_end: (99, 149),
                entrance_direction: Direction::Down,
                exit_start: (49, 150),
                exit_end: (49, 199),
                exit_direction: Direction::Left,
            },
        ]
        .into_iter()
        .flat_map(|p| [p, p.inverse()])
        .collect();

        if let Some(result) = process_with_portals(content, portals) {
            println!("{result}");
        }
    }
}
