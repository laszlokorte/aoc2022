#![feature(iter_array_chunks)]
#![feature(iter_intersperse)]
use take_until::TakeUntilExt;

pub fn process(input: String) -> Option<(u32, usize)> {
    let grid = input.lines()
    .map(str::chars)
    .map(|r| r.map(|c| c  as u32 - 48).collect()).collect::<Vec<Vec<u32>>>();
    let height = grid.len();
    let mut count = 0;
    let mut highest_score = 0;

    for (r, row) in grid.iter().enumerate() {
        let width = row.len();

        for (c, &own_height) in row.iter().enumerate() {
            let visible_left = row[0..c].iter().rev().skip_while(|h| *h < &own_height).count() == 0;
            let visible_right = row[(c+1)..width].iter().skip_while(|h| *h < &own_height).count() == 0;
            let visible_top = grid[0..r].iter().rev().skip_while(|h| h[c] < own_height).count() == 0;
            let visible_bottom = grid[(r+1)..height].iter().skip_while(|h| h[c] < own_height).count() == 0;

            if visible_left || visible_right || visible_top || visible_bottom {
                count += 1;
            }

            let view_distance_left = row[0..c].iter().rev().take_until(|h| *h >= &own_height).count();
            let view_distance_right = row[(c+1)..width].iter().take_until(|h| *h >= &own_height).count();
            let view_distance_top = grid[0..r].iter().rev().take_until(|h| h[c] >= own_height).count();
            let view_distance_bottom = grid[(r+1)..height].iter().take_until(|h| h[c] >= own_height).count();

            let scenic_score = view_distance_left * view_distance_right * view_distance_bottom * view_distance_top;

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
