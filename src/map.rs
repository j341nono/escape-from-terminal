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

    pub fn from_ascii(rows: &[&str]) -> Result<Self, String> {
        let height = rows.len();
        let width = rows.first().map_or(0, |row| row.chars().count());
        if width < 3 || height < 3 {
            return Err("map must be at least 3x3".into());
        }
        if rows.iter().any(|row| row.chars().count() != width) {
            return Err("map rows must have equal width".into());
        }

        let mut map = Self::filled(width, height, Tile::Wall);
        for (y, row) in rows.iter().enumerate() {
            for (x, symbol) in row.chars().enumerate() {
                let tile = match symbol {
                    '#' => Tile::Wall,
                    '.' | ' ' => Tile::Floor,
                    other => return Err(format!("unsupported map symbol: {other}")),
                };
                map.set_tile(Cell::new(x, y), tile);
            }
        }
        map.validate_enclosed()?;
        Ok(map)
    }

    fn validate_enclosed(&self) -> Result<(), String> {
        let horizontal = (0..self.width()).all(|x| {
            self.tile(Cell::new(x, 0)) == Tile::Wall
                && self.tile(Cell::new(x, self.height() - 1)) == Tile::Wall
        });
        let vertical = (0..self.height()).all(|y| {
            self.tile(Cell::new(0, y)) == Tile::Wall
                && self.tile(Cell::new(self.width() - 1, y)) == Tile::Wall
        });
        if horizontal && vertical {
            Ok(())
        } else {
            Err("map perimeter must be enclosed by walls".into())
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

    #[test]
    fn ascii_maps_must_be_rectangular_and_enclosed() {
        assert!(Map::from_ascii(&["###", "#.#", "###"]).is_ok());
        assert!(Map::from_ascii(&["###", "#.", "###"]).is_err());
        assert!(Map::from_ascii(&["###", "#..", "###"]).is_err());
    }
}
