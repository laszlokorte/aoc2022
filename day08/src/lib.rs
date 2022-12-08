#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]
use take_until::TakeUntilExt;

fn walk_tree_cast<I:Iterator<Item=u32>+Clone>(height: u32, ray: I) -> (usize, bool)  {
    (
        ray.clone().take_until(|h| h >= &height).count(),
        ray.skip_while(|h| h < &height).count() == 0
    )
}

pub fn process(input: String) -> Option<(u32, usize)> {
    let grid = input.lines()
    .map(str::chars)
    .map(|r| r.map(|c| c as u32 - 48).collect()).collect::<Vec<Vec<u32>>>();
    
    let height = grid.len();
    let width = grid.get(0)?.len();

    let mut count = 0;
    let mut highest_score = 0;

    for (r, row) in grid.iter().enumerate() {
        for (c, &own_height) in row.iter().enumerate() {
            let rays = [
                walk_tree_cast(own_height, row[0..c].iter().rev().cloned()),
                walk_tree_cast(own_height, row[(c+1)..width].iter().cloned()),
                walk_tree_cast(own_height, grid[0..r].iter().rev().map(|h| h[c])),
                walk_tree_cast(own_height, grid[(r+1)..height].iter().map(|h| h[c])),
            ];

            let is_visible = rays.iter().any(|r| r.1);
            let scenic_score = rays.iter().map(|r| r.0).product();

            if is_visible {
                count += 1;
            }

            if scenic_score > highest_score {
                highest_score = scenic_score;
            }
        }
    }

    Some((count, highest_score))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process() {
        const COMMANDS : &str = "30373\n\
        25512\n\
        65332\n\
        33549\n\
        35390";

        assert_eq!(process(COMMANDS.to_string()), Some((21, 8)));
    }

}
