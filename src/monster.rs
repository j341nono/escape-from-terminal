use std::f32::consts::PI;

use crate::{
    config::GameConfig,
    geom::{Cell, Vec2},
    map::Map,
};

pub const MONSTER_NAME: &str = "SPECIMEN-NULL";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiState {
    Wandering,
    Suspicious,
    Chasing,
    Searching,
}

#[derive(Debug, Clone)]
pub struct Monster {
    pub position: Vec2,
    pub heading: f32,
    pub state: AiState,
    pub last_known: Option<Cell>,
    pub path: Vec<Cell>,
    pub path_index: usize,
    pub repath_timer: f32,
    pub search_timer: f32,
    pub door_timer: f32,
}

impl Monster {
    pub fn new(spawn: Cell) -> Self {
        Self {
            position: spawn.center(),
            heading: PI,
            state: AiState::Wandering,
            last_known: None,
            path: Vec::new(),
            path_index: 0,
            repath_timer: 0.0,
            search_timer: 0.0,
            door_timer: 0.0,
        }
    }

    pub fn cell(&self) -> Cell {
        Cell::new(self.position.x as usize, self.position.y as usize)
    }

    pub fn can_see(&self, player: Vec2, map: &Map, config: &GameConfig) -> bool {
        let offset = player - self.position;
        let distance = offset.length();
        if distance > config.monster_sight_range || distance < 0.01 {
            return distance < 0.01;
        }
        let direction = offset.normalized();
        if Vec2::from_angle(self.heading).dot(direction) < (config.monster_fov * 0.5).cos() {
            return false;
        }
        line_of_sight(map, self.position, player)
    }
}

pub fn line_of_sight(map: &Map, from: Vec2, to: Vec2) -> bool {
    let direction = to - from;
    let distance = direction.length();
    let steps = (distance / 0.12).ceil() as usize;
    for step in 1..steps {
        let point = from + direction * (step as f32 / steps as f32);
        if map
            .tile(Cell::new(point.x as usize, point.y as usize))
            .blocks_sight()
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn walls_block_monster_line_of_sight() {
        let map = Map::from_ascii(&["#####", "#.#.#", "#####"]).unwrap();
        assert!(!line_of_sight(
            &map,
            Vec2::new(1.5, 1.5),
            Vec2::new(3.5, 1.5)
        ));
    }
}
