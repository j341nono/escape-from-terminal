use std::f32::consts::TAU;

use crate::{geom::Vec2, input::MovementInput, map::Map};

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Vec2,
    pub angle: f32,
}

impl Player {
    pub fn new(position: Vec2, angle: f32) -> Self {
        Self { position, angle }
    }

    pub fn forward(&self) -> Vec2 {
        Vec2::from_angle(self.angle)
    }

    pub fn right(&self) -> Vec2 {
        Vec2::new(-self.angle.sin(), self.angle.cos())
    }

    pub fn rotate(&mut self, turn: f32, speed: f32, delta_seconds: f32) {
        self.angle = (self.angle + turn * speed * delta_seconds).rem_euclid(TAU);
    }

    pub fn movement_direction(&self, input: MovementInput) -> Vec2 {
        (self.forward() * input.forward + self.right() * input.strafe).normalized()
    }

    pub fn move_with_collision(&mut self, map: &Map, displacement: Vec2, radius: f32) {
        let next_x = Vec2::new(self.position.x + displacement.x, self.position.y);
        if map.is_walkable_position(next_x, radius) {
            self.position.x = next_x.x;
        }

        let next_y = Vec2::new(self.position.x, self.position.y + displacement.y);
        if map.is_walkable_position(next_y, radius) {
            self.position.y = next_y.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_vectors_are_perpendicular() {
        let player = Player::new(Vec2::new(1.0, 1.0), 0.7);
        assert!(player.forward().dot(player.right()).abs() < 0.0001);
    }

    #[test]
    fn diagonal_movement_is_normalized() {
        let player = Player::new(Vec2::new(1.0, 1.0), 0.0);
        let direction = player.movement_direction(MovementInput {
            forward: 1.0,
            strafe: 1.0,
            turn: 0.0,
            ..MovementInput::default()
        });
        assert!((direction.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn rotation_wraps_to_positive_radians() {
        let mut player = Player::new(Vec2::new(1.0, 1.0), 0.0);
        player.rotate(-1.0, 2.0, 1.0);
        assert!(player.angle > 0.0 && player.angle < TAU);
    }

    #[test]
    fn collision_prevents_wall_entry_and_allows_sliding() {
        let map =
            Map::from_ascii(&["#####", "#.#.#", "#...#", "#...#", "#####"]).expect("valid map");
        let mut player = Player::new(Vec2::new(1.5, 1.5), 0.0);
        player.move_with_collision(&map, Vec2::new(0.8, 0.6), 0.2);
        assert_eq!(player.position.x, 1.5);
        assert!(player.position.y > 1.5);
    }
}
