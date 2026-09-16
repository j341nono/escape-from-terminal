use crate::geom::{Cell, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
}

impl Tile {
    pub fn blocks_movement(self) -> bool {
        matches!(self, Self::Wall)
    }

    pub fn blocks_sight(self) -> bool {
        self.blocks_movement()
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn filled(width: usize, height: usize, tile: Tile) -> Self {
        Self {
            width,
            height,
            tiles: vec![tile; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn in_bounds(&self, cell: Cell) -> bool {
        cell.x < self.width && cell.y < self.height
    }

    pub fn tile(&self, cell: Cell) -> Tile {
        if self.in_bounds(cell) {
            self.tiles[cell.y * self.width + cell.x]
        } else {
            Tile::Wall
        }
    }

    pub fn set_tile(&mut self, cell: Cell, tile: Tile) {
        if self.in_bounds(cell) {
            self.tiles[cell.y * self.width + cell.x] = tile;
        }
    }

    pub fn is_walkable(&self, cell: Cell) -> bool {
        self.in_bounds(cell) && !self.tile(cell).blocks_movement()
    }

    pub fn is_walkable_position(&self, position: Vec2, radius: f32) -> bool {
        let samples = [
            Vec2::new(position.x - radius, position.y - radius),
            Vec2::new(position.x + radius, position.y - radius),
            Vec2::new(position.x - radius, position.y + radius),
            Vec2::new(position.x + radius, position.y + radius),
        ];
        samples.into_iter().all(|point| {
            point.x >= 0.0
                && point.y >= 0.0
                && self.is_walkable(Cell::new(point.x as usize, point.y as usize))
        })
    }

    pub fn neighbors(&self, cell: Cell) -> Vec<Cell> {
        let mut result = Vec::with_capacity(4);
        for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let Some(x) = cell.x.checked_add_signed(dx) else {
                continue;
            };
            let Some(y) = cell.y.checked_add_signed(dy) else {
                continue;
            };
            let next = Cell::new(x, y);
            if self.in_bounds(next) && !self.tile(next).blocks_movement() {
                result.push(next);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_bounds_is_solid() {
        let map = Map::filled(4, 4, Tile::Floor);
        assert_eq!(map.tile(Cell::new(5, 1)), Tile::Wall);
    }

    #[test]
    fn collision_radius_detects_a_nearby_wall() {
        let mut map = Map::filled(5, 5, Tile::Floor);
        map.set_tile(Cell::new(2, 2), Tile::Wall);
        assert!(!map.is_walkable_position(Vec2::new(1.9, 2.5), 0.2));
        assert!(map.is_walkable_position(Vec2::new(1.5, 2.5), 0.2));
    }
}
