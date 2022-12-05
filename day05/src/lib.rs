#![feature(iter_array_chunks)]

#[derive(Debug)]
struct Movement {
    count: usize,
    source: usize,
    target: usize,
}

fn setup_stacks(desc: &str) -> Option<(Vec<char>, Vec<Vec<char>>)> {
    let mut lines = desc.lines().rev();
    let last_line = lines.next()?;

    let stack_names = last_line.split_whitespace().map(|c|c.chars().next()).collect::<Option<Vec<char>>>()?;
    let number_of_stacks = stack_names.len();

    let mut stacks = vec![Vec::new(); number_of_stacks];

    for line in lines {

        for c in 0..number_of_stacks {
            let container_name = line.chars().nth(c*4+1);
            match container_name {
                Some(' ') => {},
                Some(name) => stacks[c].push(name),
                None => {
                    println!("{:?}, {}", line.chars().count(), c);
                    return None;
                },
            }
        }
    }


    Some((stack_names, stacks))
}

fn setup_commands(stack_names: &Vec<char>, desc: &str) -> Option<Vec<Movement>> {
    desc.lines().map(|line| {
        let parts = line.split_whitespace().array_chunks::<6>().next()?;
        let source_name = parts[3].chars().next()?;
        let target_name = parts[5].chars().next()?;
        let source_index = stack_names.iter().position(|&x| x == source_name)?;
        let target_index = stack_names.iter().position(|&x| x == target_name)?;
        
        Some(Movement {
            count: str::parse(parts[1]).ok()?,
            source: source_index,
            target: target_index,
        })
    }).collect()
}

fn apply_commands(stacks: &mut Vec<Vec<char>>, commands: Vec<Movement>) {
    for cmd in commands {    
        for _ in 0..cmd.count {
            let e = stacks[cmd.source].pop().unwrap();
            stacks[cmd.target].push(e)
        }
    }
}

fn apply_commands_multiple(stacks: &mut Vec<Vec<char>>, commands: Vec<Movement>) {
    let mut crane = Vec::new();
    for cmd in commands {
        for _ in 0..cmd.count {
            let e = stacks[cmd.source].pop().unwrap();
            crane.push(e)
        }
        while let Some(c) = crane.pop() {
            stacks[cmd.target].push(c);
        }
    }
}

pub fn process(input: String, move_multiple: bool) -> Option<String> {
    let (stack_description, command_description) = input.split_once("\n\n")?;
    let (stack_names, mut stacks) = setup_stacks(stack_description)?;
    let commands = setup_commands(&stack_names, command_description)?;

    if move_multiple {
        apply_commands_multiple(&mut stacks, commands);
    } else {
        apply_commands(&mut stacks, commands);
    }

    Some(stacks.iter().filter_map(|s| s.last()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOVES: &str = "    [D]    
[N] [C]    
[Z] [M] [P]
 1   2   3 

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

    #[test]
    fn test_process() {
        assert_eq!(process(MOVES.to_string(), false), Some("CMZ".to_string()));
    }

    #[test]
    fn test_process_multiple() {
        assert_eq!(process(MOVES.to_string(), true), Some("MCD".to_string()));
    }
}
