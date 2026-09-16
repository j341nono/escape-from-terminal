use std::f32::consts::PI;

use rand::Rng;

use crate::{
    config::GameConfig,
    geom::{Cell, Vec2},
    map::{Map, Tile},
    pathfinding::{path_distance, shortest_path},
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

#[derive(Debug, Default, Clone, Copy)]
pub struct MonsterReport {
    pub spotted: bool,
    pub caught: bool,
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

    pub fn update<R: Rng>(
        &mut self,
        map: &mut Map,
        player: Vec2,
        noise: f32,
        delta: f32,
        config: &GameConfig,
        rng: &mut R,
    ) -> MonsterReport {
        let mut report = MonsterReport::default();
        self.repath_timer -= delta;
        let player_cell = Cell::new(player.x as usize, player.y as usize);
        if self.can_see(player, map, config) {
            report.spotted = self.state != AiState::Chasing;
            self.state = AiState::Chasing;
            self.last_known = Some(player_cell);
            self.search_timer = 7.0;
        } else if self.state == AiState::Chasing {
            self.state = AiState::Searching;
        } else if self.state != AiState::Chasing
            && self.can_hear(player_cell, player, noise, map, config)
        {
            self.state = AiState::Suspicious;
            self.last_known = Some(player_cell);
        }
        if self.state == AiState::Searching {
            self.search_timer -= delta;
            if self.search_timer <= 0.0 {
                self.state = AiState::Wandering;
                self.last_known = None;
                self.path.clear();
            }
        }
        if self.state == AiState::Wandering
            && (self.path_index >= self.path.len() || self.repath_timer <= 0.0)
        {
            self.last_known = random_floor(map, rng);
        }
        let target = match self.state {
            AiState::Chasing => Some(player_cell),
            AiState::Suspicious | AiState::Searching | AiState::Wandering => self.last_known,
        };
        if let Some(target) = target {
            if self.repath_timer <= 0.0
                || self.path.last().copied() != Some(target)
                || self.path_index >= self.path.len()
            {
                self.path = shortest_path(map, self.cell(), target, true).unwrap_or_default();
                self.path_index = 1.min(self.path.len());
                self.repath_timer = if self.state == AiState::Chasing {
                    0.28
                } else {
                    0.9
                };
            }
            self.follow_path(map, delta, config);
            if self.state == AiState::Suspicious && self.cell() == target {
                self.state = AiState::Searching;
                self.search_timer = 4.0;
            }
        }
        report.caught = self.position.distance(player) < 0.52;
        report
    }

    fn can_hear(
        &self,
        player_cell: Cell,
        player: Vec2,
        noise: f32,
        map: &Map,
        config: &GameConfig,
    ) -> bool {
        let range = config.monster_hearing_range * noise;
        if range <= 0.0 || self.position.distance(player) > range {
            return false;
        }
        path_distance(map, self.cell(), player_cell, true)
            .is_some_and(|distance| distance as f32 <= range)
    }

    fn follow_path(&mut self, map: &mut Map, delta: f32, config: &GameConfig) {
        let Some(next) = self.path.get(self.path_index).copied() else {
            self.door_timer = 0.0;
            return;
        };
        if map.tile(next) == Tile::DoorClosed {
            self.door_timer += delta;
            if self.door_timer >= 1.1 {
                map.toggle_door(next);
                self.door_timer = 0.0;
            } else {
                return;
            }
        }
        self.door_timer = 0.0;
        let target = next.center();
        let offset = target - self.position;
        let speed = if self.state == AiState::Chasing {
            config.monster_chase_speed
        } else {
            config.monster_wander_speed
        };
        let distance = offset.length();
        let step = speed * delta;
        let direction = offset.normalized();
        if distance > f32::EPSILON {
            self.heading = direction.y.atan2(direction.x);
        }
        if distance <= step.max(0.12) {
            if map.is_walkable_position(target, config.player_radius) {
                self.position = target;
                self.path_index += 1;
            }
            return;
        }
        let next_position = self.position + direction * step;
        if map.is_walkable_position(next_position, config.player_radius) {
            self.position = next_position;
        }
    }
}

fn random_floor<R: Rng>(map: &Map, rng: &mut R) -> Option<Cell> {
    for _ in 0..80 {
        let cell = Cell::new(
            rng.random_range(1..map.width() - 1),
            rng.random_range(1..map.height() - 1),
        );
        if map.is_walkable(cell) {
            return Some(cell);
        }
    }
    None
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
    use rand::SeedableRng;
    #[test]
    fn walls_block_monster_line_of_sight() {
        let map = Map::from_ascii(&["#####", "#.#.#", "#####"]).unwrap();
        assert!(!line_of_sight(
            &map,
            Vec2::new(1.5, 1.5),
            Vec2::new(3.5, 1.5)
        ));
    }

    #[test]
    fn visible_player_triggers_chase() {
        let mut map = Map::from_ascii(&["#######", "#.....#", "#######"]).unwrap();
        let mut monster = Monster::new(Cell::new(1, 1));
        monster.heading = 0.0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        let report = monster.update(
            &mut map,
            Vec2::new(4.5, 1.5),
            0.0,
            0.1,
            &GameConfig::default(),
            &mut rng,
        );
        assert!(report.spotted);
        assert_eq!(monster.state, AiState::Chasing);
    }

    #[test]
    fn monster_opens_blocking_door_after_delay() {
        let mut map = Map::from_ascii(&["#######", "#.D...#", "#######"]).unwrap();
        let mut monster = Monster::new(Cell::new(1, 1));
        monster.state = AiState::Chasing;
        monster.last_known = Some(Cell::new(5, 1));
        monster.search_timer = 7.0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(2);
        for _ in 0..14 {
            let _ = monster.update(
                &mut map,
                Vec2::new(5.5, 1.5),
                0.0,
                0.1,
                &GameConfig::default(),
                &mut rng,
            );
        }
        assert_eq!(map.tile(Cell::new(2, 1)), Tile::DoorOpen);
    }

    #[test]
    fn walls_attenuate_quiet_footsteps() {
        let map = Map::from_ascii(&[
            "#########",
            "#...#...#",
            "#...#...#",
            "#.......#",
            "#########",
        ])
        .unwrap();
        let monster = Monster::new(Cell::new(3, 1));
        let player = Vec2::new(5.5, 1.5);
        let config = GameConfig::default();
        assert!(!monster.can_hear(Cell::new(5, 1), player, 0.42, &map, &config));
        assert!(monster.can_hear(Cell::new(5, 1), player, 1.0, &map, &config));
    }

    #[test]
    fn movement_clamps_to_waypoint_without_oscillation() {
        let mut map = Map::from_ascii(&["#####", "#...#", "#####"]).unwrap();
        let mut monster = Monster::new(Cell::new(1, 1));
        monster.state = AiState::Chasing;
        monster.path = vec![Cell::new(1, 1), Cell::new(2, 1)];
        monster.path_index = 1;
        for _ in 0..4 {
            monster.follow_path(&mut map, 0.1, &GameConfig::default());
        }
        assert_eq!(monster.position, Cell::new(2, 1).center());
        assert_eq!(monster.path_index, 2);
    }

    #[test]
    fn lost_chase_preserves_last_visible_position() {
        let mut map = Map::from_ascii(&["#######", "#..#..#", "#..#..#", "#######"]).unwrap();
        let mut monster = Monster::new(Cell::new(2, 1));
        monster.state = AiState::Chasing;
        monster.last_known = Some(Cell::new(2, 2));
        monster.search_timer = 7.0;
        let mut rng = rand::rngs::StdRng::seed_from_u64(5);
        let _ = monster.update(
            &mut map,
            Vec2::new(4.5, 1.5),
            0.0,
            0.1,
            &GameConfig::default(),
            &mut rng,
        );
        assert_eq!(monster.state, AiState::Searching);
        assert_eq!(monster.last_known, Some(Cell::new(2, 2)));
    }

    #[test]
    fn searching_times_out_to_wandering() {
        let mut map = Map::from_ascii(&["#######", "#..#..#", "#..#..#", "#######"]).unwrap();
        let mut monster = Monster::new(Cell::new(2, 1));
        monster.state = AiState::Searching;
        monster.last_known = Some(Cell::new(2, 2));
        monster.search_timer = 0.05;
        let mut rng = rand::rngs::StdRng::seed_from_u64(8);
        let _ = monster.update(
            &mut map,
            Vec2::new(4.5, 1.5),
            0.0,
            0.1,
            &GameConfig::default(),
            &mut rng,
        );
        assert_eq!(monster.state, AiState::Wandering);
    }

    #[test]
    fn unreachable_suspicious_target_is_safe() {
        let mut map = Map::from_ascii(&["#######", "#..#..#", "#..#..#", "#######"]).unwrap();
        let mut monster = Monster::new(Cell::new(1, 1));
        monster.state = AiState::Suspicious;
        monster.last_known = Some(Cell::new(5, 1));
        let mut rng = rand::rngs::StdRng::seed_from_u64(13);
        let _ = monster.update(
            &mut map,
            Vec2::new(5.5, 1.5),
            0.0,
            0.1,
            &GameConfig::default(),
            &mut rng,
        );
        assert!(monster.path.is_empty());
        assert_eq!(monster.state, AiState::Suspicious);
    }
}
