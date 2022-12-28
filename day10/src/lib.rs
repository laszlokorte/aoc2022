#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::sequence::preceded;
use nom::*;

#[derive(Debug)]
enum Command {
    Noop,
    Addx(i32),
}

impl Command {
    fn cycles_needed(&self) -> u32 {
        match self {
            Command::Noop => 1,
            Command::Addx(_) => 2,
        }
    }

    fn next_state(&self, state: i32, cycle: u32) -> i32 {
        match self {
            Self::Noop => state,
            Self::Addx(incr) => state + if cycle == 1 { incr } else { &0 },
        }
    }
}

fn commands(input: &str) -> IResult<&str, Vec<Command>> {
    let (input, cmds) = separated_list1(
        line_ending,
        alt((
            tag("noop").map(|_| Command::Noop),
            preceded(tag("addx "), character::complete::i32).map(Command::Addx),
        )),
    )(input)?;
    Ok((input, cmds))
}

pub fn process_crt(input: String) -> Option<(i32, String)> {
    let (_, cmds) = commands(&input).ok()?;
    const SCREEN_WIDTH: usize = 40;
    const SCREEN_HEIGHT: usize = 6;
    let mut crt: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT] = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];

    let mut counter: i32 = 1;
    let mut cycle: usize = 0;
    let mut sum_strength = 0;

    for cmd in &cmds {
        for c in 0..cmd.cycles_needed() {
            let crt_column = cycle % SCREEN_WIDTH;
            let crt_row = cycle / SCREEN_WIDTH;
            let pixel_lit = ((counter - 1)..=(counter + 1)).contains(&(crt_column as i32));
            crt[crt_row][crt_column] = pixel_lit;

            cycle += 1;
            if (cycle + 20) % 40 == 0 {
                let strength = cycle as i32 * counter;
                sum_strength += strength;
            }
            counter = cmd.next_state(counter, c);
        }
    }
    Some((
        sum_strength,
        crt.map(|row| row.map(|p| if p { "#" } else { "." }).join(""))
            .join("\n"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(
            process_crt(COMMANDS.to_string()),
            Some((
                13140,
                include_str!("assert.txt").to_string()
            ))
        );
    }
}
