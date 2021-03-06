use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, BinaryHeap};
use types::TileMapping;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TileType {
    Unpassable, Open
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Location {
    position: (usize, usize),
    cost: usize,
}

impl Ord for Location {
    fn cmp(&self, other: &Location) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Location) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn distance_to_target(location: &(usize, usize), target: &(usize, usize)) -> usize {
    let mut x_diff = location.1 as i16 - target.1 as i16;
    let mut y_diff = location.0 as i16 - target.0 as i16;
    if x_diff < 0 {
        x_diff *= -1;
    }
    if y_diff < 0 {
        y_diff *= -1;
    }

    x_diff as usize + y_diff as usize
}

fn get_neighbours(pos: &(usize, usize), tiles: &Vec<Vec<TileType>>) -> Vec<(usize, usize)> {
    let mut neighbours: Vec<(usize, usize)> = Vec::with_capacity(4);

    if pos.0 > 0 {
        let tile_type = &tiles[pos.1][pos.0 - 1];
        if *tile_type == TileType::Open {
            neighbours.push((pos.0 - 1, pos.1));
        }
    }

    if pos.0 < tiles[0].len() - 1 {
        let tile_type = &tiles[pos.1][pos.0 + 1];
        if *tile_type == TileType::Open {
            neighbours.push((pos.0 + 1, pos.1));
        }
    }

    if pos.1 > 0 {
        let tile_type = &tiles[pos.1 - 1][pos.0];
        if *tile_type == TileType::Open {
            neighbours.push((pos.0, pos.1 - 1));
        }
    }

    if pos.1 < tiles.len() - 1 {
        let tile_type = &tiles[pos.1 + 1][pos.0];
        if *tile_type == TileType::Open {
            neighbours.push((pos.0, pos.1 + 1));
        }
    }

    neighbours
}

pub fn build_grid_for_map(closed_tiles: &TileMapping<usize>, tiles_wide: usize, tiles_high: usize) -> Vec<Vec<TileType>> {
    let mut tiles: Vec<Vec<TileType>> = Vec::with_capacity(tiles_high);
    for y in 0..tiles_high {
        let mut row = Vec::with_capacity(tiles_wide);
        for x in 0..tiles_wide {
            if let Some(x_vals) = closed_tiles.get(&y) {
                if x_vals.contains(&x) {
                    row.push(TileType::Unpassable);
                } else {
                    row.push(TileType::Open);
                }
            } else {
                row.push(TileType::Open);
            }
        }
        tiles.push(row);
    }

    tiles
}

pub fn find_path(tiles: &Vec<Vec<TileType>>, start_pos: (usize, usize), target: (usize, usize)) -> Vec<(usize, usize)> {
    let mut closed: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut costs: HashMap<(usize, usize), usize> = HashMap::new();
    costs.insert(start_pos, 0);

    let mut heap = BinaryHeap::new();
    heap.push(Location{ position: start_pos, cost: 0 });

    let mut tracked_positions: Vec<(usize, usize)> = Vec::new();

    while let Some(location) = heap.pop() {
        if location.position.0 == target.0 && location.position.1 == target.1 {
            let mut pos = closed.get(&location.position).unwrap();
            tracked_positions.push(location.position);
            loop {
                if let Some(p) = closed.get(&pos) {
                    tracked_positions.push(*p);
                    pos = p;
                } else {
                    break
                }
            }
            break
        }
        let neighbours = get_neighbours(&location.position, &tiles);
        for neighbour in neighbours {
            let new_cost = costs.get(&location.position).unwrap() + 1;
            if !costs.contains_key(&neighbour) || new_cost < *costs.get(&neighbour).unwrap() {
                heap.push(Location{ position: neighbour, cost: new_cost + distance_to_target(&neighbour, &target) });
                closed.insert(neighbour, location.position);
                costs.insert(neighbour, new_cost);
            }
        }
    }

    tracked_positions.reverse();
    tracked_positions
}
