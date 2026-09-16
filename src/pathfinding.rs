use std::collections::{HashMap, VecDeque};

use crate::{
    geom::Cell,
    map::{Map, Tile},
};

pub fn shortest_path(
    map: &Map,
    start: Cell,
    goal: Cell,
    allow_closed_doors: bool,
) -> Option<Vec<Cell>> {
    if start == goal {
        return Some(vec![start]);
    }
    let mut queue = VecDeque::from([start]);
    let mut previous = HashMap::from([(start, start)]);
    while let Some(cell) = queue.pop_front() {
        for next in neighbors(map, cell, allow_closed_doors) {
            if previous.contains_key(&next) {
                continue;
            }
            previous.insert(next, cell);
            if next == goal {
                let mut path = vec![goal];
                let mut current = goal;
                while current != start {
                    current = previous[&current];
                    path.push(current);
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(next);
        }
    }
    None
}

pub fn path_distance(
    map: &Map,
    start: Cell,
    goal: Cell,
    allow_closed_doors: bool,
) -> Option<usize> {
    shortest_path(map, start, goal, allow_closed_doors).map(|path| path.len().saturating_sub(1))
}

pub fn neighbors(map: &Map, cell: Cell, allow_closed_doors: bool) -> Vec<Cell> {
    let mut output = Vec::with_capacity(4);
    for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
        let Some(x) = cell.x.checked_add_signed(dx) else {
            continue;
        };
        let Some(y) = cell.y.checked_add_signed(dy) else {
            continue;
        };
        let next = Cell::new(x, y);
        let tile = map.tile(next);
        if map.in_bounds(next)
            && (!tile.blocks_movement() || (allow_closed_doors && tile == Tile::DoorClosed))
        {
            output.push(next);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_finds_a_route_around_walls() {
        let map =
            Map::from_ascii(&["#######", "#.#...#", "#.#.#.#", "#...#.#", "#######"]).unwrap();
        assert_eq!(
            path_distance(&map, Cell::new(1, 1), Cell::new(5, 3), false),
            Some(10)
        );
    }

    #[test]
    fn closed_doors_are_optional_path_edges() {
        let map = Map::from_ascii(&["#####", "#.D.#", "#####"]).unwrap();
        assert!(shortest_path(&map, Cell::new(1, 1), Cell::new(3, 1), false).is_none());
        assert!(shortest_path(&map, Cell::new(1, 1), Cell::new(3, 1), true).is_some());
    }
}
