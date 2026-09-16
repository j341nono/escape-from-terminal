use crate::geom::Vec2;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facing_vectors_are_perpendicular() {
        let player = Player::new(Vec2::new(1.0, 1.0), 0.7);
        assert!(player.forward().dot(player.right()).abs() < 0.0001);
    }
}
