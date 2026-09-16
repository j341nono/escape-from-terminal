use std::f32::consts::TAU;

use crate::{geom::Vec2, input::MovementInput};

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
        });
        assert!((direction.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn rotation_wraps_to_positive_radians() {
        let mut player = Player::new(Vec2::new(1.0, 1.0), 0.0);
        player.rotate(-1.0, 2.0, 1.0);
        assert!(player.angle > 0.0 && player.angle < TAU);
    }
}
