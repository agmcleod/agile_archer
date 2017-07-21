use std::cmp::max;

pub fn distance_to_tile(from: &(usize, usize), to: &(usize, usize)) -> usize {
    let y_diff = (from.1 as i32 - to.1 as i32).abs();
    let x_diff = (from.0 as i32 - to.0 as i32).abs();
    max(x_diff as usize, y_diff as usize)
}