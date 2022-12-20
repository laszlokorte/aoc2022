#![feature(iter_array_chunks)]
#![feature(linked_list_cursors)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]

struct Permutation {
    original_to_now: Vec<usize>,
    now_to_original: Vec<usize>,
}

impl Permutation {
    fn new(size: usize) -> Self {
        Self {
            original_to_now: (0..size).into_iter().collect(),
            now_to_original: (0..size).into_iter().collect(),
        }
    }

    fn swap(&mut self, a: usize, b: usize) {
        // println!("swap pos {a} with {b}");
        let a_original = self.now_to_original[a];
        let b_original = self.now_to_original[b];
        let a_after_swap = b;
        let b_after_swap = a;

        self.original_to_now[a_original] = a_after_swap;
        self.original_to_now[b_original] = b_after_swap;
        self.now_to_original[a_after_swap] = a_original;
        self.now_to_original[b_after_swap] = b_original;
    }

    fn get_now(&self, origin: usize) -> usize {
        self.original_to_now[origin]
    }
}

pub fn process(input: String, multiplier: i64, repetitions: u64) -> Option<i64> {
    let numbers: Vec<i64> = input
        .lines()
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
        .ok()?
        .into_iter()
        .map(|n: i64| n * multiplier)
        .collect();
    let size = numbers.len();
    // println!("{size}");

    let mut permutation = Permutation::new(size);
    for _ in 0..repetitions {
        for i in 0..size {
            let movement = numbers[i];
            let mut distance = movement.abs() as usize % (size - 1);
            let mut negative = movement.is_negative();

            let start_position = permutation.get_now(i);
            if !(1..size as i64).contains(&(start_position as i64 + movement as i64)) {
                distance = size - distance - 1;
                negative = !negative;
            }
            let number_of_swaps = distance;
            let mut current_position = start_position;
            for _ in 0..number_of_swaps {
                let target_position = if negative {
                    current_position + size - 1
                } else {
                    current_position + size + 1
                } % size;

                permutation.swap(current_position, target_position);

                current_position = target_position;
            }
        }
    }
    let original_zero = numbers.iter().position(|n| n == &0)?;
    let zero_pos_now = permutation.original_to_now[original_zero];
    Some(
        [1000, 2000, 3000]
            .into_iter()
            .map(|o| numbers[permutation.now_to_original[(zero_pos_now + o) % size]])
            .sum(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "1
2
-3
3
-2
0
4";

        assert_eq!(process(COMMANDS.to_string(), 1, 1), Some(3));
        assert_eq!(
            process(COMMANDS.to_string(), 811589153, 10),
            Some(1623178306)
        );
    }
}
