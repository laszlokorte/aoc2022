#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]

use std::collections::BTreeMap;

use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::line_ending;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::*;

#[derive(Debug)]
pub enum Operation<'a> {
    Cd(Cd<'a>),
    Ls(Vec<File<'a>>),
}

#[derive(Debug)]
pub enum Cd<'a> {
    Root,
    Up,
    Down(&'a str),
}

#[derive(Debug)]
pub enum File<'a> {
    File { size: u32, name: &'a str },
    Dir { name: &'a str },
}

fn file_name(input: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n")(input)
}

fn file(input: &str) -> IResult<&str, File> {
    let (input, (size, name)) =
        separated_pair(character::complete::u32, tag(" "), file_name)(input)?;

    Ok((input, File::File { size, name }))
}

fn directory(input: &str) -> IResult<&str, File> {
    let (input, _) = tag("dir ")(input)?;
    let (input, name) = file_name(input)?;

    Ok((input, File::Dir { name }))
}

fn ls(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("$ ls")(input)?;
    let (input, _) = line_ending(input)?;
    let (input, files): (&str, Vec<File>) =
        separated_list1(line_ending, alt((file, directory)))(input)?;

    Ok((input, Operation::Ls(files)))
}

fn cd(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("$ cd ")(input)?;
    let (input, dir) = alt((tag(".."), tag("/"), file_name))(input)?;

    let op = match dir {
        "/" => Operation::Cd(Cd::Root),
        ".." => Operation::Cd(Cd::Up),
        name => Operation::Cd(Cd::Down(name)),
    };

    Ok((input, op))
}

fn commands(input: &str) -> IResult<&str, Vec<Operation>> {
    let (input, cmds) = separated_list1(line_ending, alt((ls, cd)))(input)?;

    Ok((input, cmds))
}

fn collect_folder_sizes(operations: Vec<Operation>) -> Vec<(String, u32)> {
    let mut path_stack = Vec::<&str>::new();
    let mut directory_flat_sizes = BTreeMap::<String, u32>::new();

    for cmd in operations {
        match cmd {
            Operation::Cd(Cd::Root) => path_stack.clear(),
            Operation::Cd(Cd::Up) => {
                path_stack.pop();
            }
            Operation::Cd(Cd::Down(name)) => path_stack.push(name),
            Operation::Ls(files) => {
                for file in files {
                    if let File::File { size, .. } = file {
                        for p in 0..=path_stack.len() {
                            let path = path_stack
                                .iter()
                                .take(p)
                                .cloned()
                                .intersperse("/")
                                .collect::<String>();
                            let old = directory_flat_sizes.entry(path).or_insert(0);
                            *old += size;
                        }
                    }
                }
            }
        }
    }
    let mut sorted = directory_flat_sizes
        .clone()
        .into_iter()
        .collect::<Vec<(String, u32)>>();
    sorted.sort_by_key(|(_, size)| *size);

    sorted
}

pub fn process_sum(input: String, threshold: u32) -> Option<u32> {
    let operations = commands(&input).ok()?.1;

    let sorted_sized = collect_folder_sizes(operations);

    Some(
        sorted_sized
            .iter()
            .cloned()
            .map(|p| p.1)
            .filter(|v| *v < threshold)
            .sum(),
    )
}

pub fn process_deletion(input: String, total_space: u32, needed_space: u32) -> Option<u32> {
    let operations = commands(&input).ok()?.1;

    let sorted_sized = collect_folder_sizes(operations);
    let (_, total_size) = sorted_sized.last()?;
    let free_space = total_space - total_size;
    let to_delete = needed_space - free_space;

    let (_, size_to_delete) = *sorted_sized.iter().find(|(_, size)| *size > to_delete)?;

    Some(size_to_delete)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process_sum(COMMANDS.to_string(), 100000), Some(95437));
        assert_eq!(
            process_deletion(COMMANDS.to_string(), 70000000, 30000000),
            Some(24933642)
        );
    }
}
