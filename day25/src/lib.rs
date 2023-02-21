#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

fn char_to_digit(char: char) -> Option<i64> {
    Some(match char {
        '2' => 2,
        '1' => 1,
        '0' => 0,
        '-' => -1,
        '=' => -2,
        _ => return None,
    })
}

fn digit_to_char(digit: i64) -> Option<char> {
    Some(match digit {
        -2 => '=',
        -1 => '-',
        0 => '0',
        1 => '1',
        2 => '2',
        _ => return None,
    })
}

fn parse_number(input: &str) -> i64 {
    input
        .chars()
        .rev()
        .enumerate()
        .filter_map(|(pos, c)| char_to_digit(c).map(|d| d * 5_i64.pow(pos as u32)))
        .sum()
}

fn stringify_number(number: i64) -> String {
    let mut num = number;
    let mut digits = vec![];
    while num > 0 {
        let mut remainder = num.rem_euclid(5);
        num -= remainder;
        num /= 5;
        if remainder > 2 {
            remainder -= 5;
            num += 1;
        }
        if let Some(digit) = digit_to_char(remainder) {
            digits.push(digit.to_string());
        }
    }
    digits.reverse();
    digits.join("")
}

pub fn process(input: String) -> Option<String> {
    Some(stringify_number(dbg!(input
        .lines()
        .map(parse_number)
        .sum())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = include_str!("test.txt");

        assert_eq!(process(COMMANDS.to_string()), Some("2=-1=0".to_owned()));
    }
}
