#![feature(iter_array_chunks)]
#![feature(let_chains)]
#![feature(array_windows)]
#![feature(array_chunks)]
#![feature(iterator_try_reduce)]
#![feature(iter_intersperse)]
#![feature(step_trait)]
use nom::{
    bytes::complete::tag, character::complete::line_ending, combinator::map,
    multi::separated_list0, sequence::tuple, *,
};
use std::collections::{HashSet, VecDeque};
fn cube(input: &str) -> IResult<&str, (i32, i32, i32)> {
    use character::complete::i32 as dig;
    map(
        tuple((dig, tag(","), dig, tag(","), dig)),
        |(a, _, b, _, c)| (a, b, c),
    )(input)
}
fn cubes(input: &str) -> IResult<&str, HashSet<(i32, i32, i32)>> {
    map(separated_list0(line_ending, cube), |l| {
        HashSet::from_iter(l.iter().cloned())
    })(input)
}

const DIRECTIONS: [(i32, i32, i32); 6] = [
    (0, 0, 1),
    (0, 0, -1),
    (0, 1, 0),
    (0, -1, 0),
    (1, 0, 0),
    (-1, 0, 0),
];
fn find_exterrior(cubes: &HashSet<(i32, i32, i32)>) -> Option<HashSet<(i32, i32, i32)>> {
    let min = &cubes
        .iter()
        .cloned()
        .reduce(|(ma, mb, mc), (a, b, c)| (ma.min(a), mb.min(b), mc.min(c)))?;
    let max = &cubes
        .iter()
        .cloned()
        .reduce(|(ma, mb, mc), (a, b, c)| (ma.max(a), mb.max(b), mc.max(c)))?;
    let xrange = (min.0 - 1)..=(max.0 + 1);
    let yrange = (min.1 - 1)..=(max.1 + 1);
    let zrange = (min.2 - 1)..=(max.2 + 1);
    let start_position = (min.0 - 1, min.1, min.2);
    let mut result = HashSet::new();
    let mut seen = HashSet::<(i32, i32, i32)>::new();
    let mut queue = VecDeque::new();
    queue.push_back(start_position);
    seen.insert(start_position);
    while let Some(current) = queue.pop_front() {
        for (dx, dy, dz) in DIRECTIONS {
            let next = (current.0 + dx, current.1 + dy, current.2 + dz);
            if cubes.contains(&next) {
                result.insert(current.clone());
                continue;
            }
            if !xrange.contains(&next.0) || !yrange.contains(&next.1) || !zrange.contains(&next.2) {
                continue;
            }
            if seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    Some(result)
}
pub fn process(input: String, exclude_bubbles: bool) -> Option<usize> {
    let (_, cubes) = cubes(&input).ok()?;
    if exclude_bubbles {
        let exterrior = find_exterrior(&cubes)?;

        let neighbours_to_check = exterrior
            .iter()
            .flat_map(|(x, y, z)| DIRECTIONS.map(|(dx, dy, dz)| (x + dx, y + dy, z + dz)));
        let open_faces = neighbours_to_check.filter(|n| cubes.contains(n)).count();

        Some(open_faces)
    } else {
        let neighbours_to_check = cubes
            .iter()
            .flat_map(|(x, y, z)| DIRECTIONS.map(|(dx, dy, dz)| (x + dx, y + dy, z + dz)));
        let open_faces = neighbours_to_check.filter(|n| !cubes.contains(n)).count();

        Some(open_faces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS: &str = "2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";

        assert_eq!(process(COMMANDS.to_string(), false), Some(64));
        assert_eq!(process(COMMANDS.to_string(), true), Some(58));
    }
}
