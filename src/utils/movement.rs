/**
 * General movement utility functions, used across more than 1 system
*/

use std::cmp::max;

pub fn distance_to_tile(from: &(usize, usize), to: &(usize, usize), total: bool) -> usize {
    let y_diff = (from.1 as i32 - to.1 as i32).abs() as usize;
    let x_diff = (from.0 as i32 - to.0 as i32).abs() as usize;
    if total {
        return x_diff + y_diff
    }
    max(x_diff as usize, y_diff as usize)
}