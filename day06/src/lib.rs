#![feature(iter_array_chunks)]
use std::collections::HashSet;

pub fn process(input: String, streak: usize) -> Option<usize> {
    let inter = input.chars().collect::<Vec<char>>();
    for (i, chars) in inter.windows(streak).enumerate() {
        if HashSet::<&char>::from_iter(chars).len() == streak {
            return Some(i+streak)
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_package() {
        assert_eq!(process("mjqjpqmgbljsphdztnvjfqwrcgsmlb".to_string(), 4), Some(7));
        assert_eq!(process("bvwbjplbgvbhsrlpgdmjqwftvncz".to_string(), 4), Some(5));
        assert_eq!(process("nppdvjthqldpwncqszvftbrmjlhg".to_string(), 4), Some(6));
        assert_eq!(process("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg".to_string(), 4), Some(10));
        assert_eq!(process("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw".to_string(), 4), Some(11));
    }

    #[test]
    fn test_process_message() {
        assert_eq!(process("mjqjpqmgbljsphdztnvjfqwrcgsmlb".to_string(), 14), Some(19));
        assert_eq!(process("bvwbjplbgvbhsrlpgdmjqwftvncz".to_string(), 14), Some(23));
        assert_eq!(process("nppdvjthqldpwncqszvftbrmjlhg".to_string(), 14), Some(23));
        assert_eq!(process("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg".to_string(), 14), Some(29));
        assert_eq!(process("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw".to_string(), 14), Some(26));
    }
}
