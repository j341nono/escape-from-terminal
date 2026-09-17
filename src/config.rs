use std::f32::consts::FRAC_PI_3;

pub const GAME_NAME: &str = "ESCAPE FROM TERMINAL";
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;
pub const MAX_RENDER_WIDTH: usize = 240;
pub const MAX_RENDER_HEIGHT: usize = 80;
pub const TARGET_FPS: u64 = 30;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub fov: f32,
    pub render_distance: f32,
    pub player_speed: f32,
    pub rotation_speed: f32,
    pub player_radius: f32,
    pub monster_wander_speed: f32,
    pub monster_chase_speed: f32,
    pub monster_sight_range: f32,
    pub monster_hearing_range: f32,
    pub monster_fov: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            fov: FRAC_PI_3,
            render_distance: 28.0,
            player_speed: 2.4,
            rotation_speed: 2.25,
            player_radius: 0.22,
            monster_wander_speed: 1.35,
            monster_chase_speed: 2.65,
            monster_sight_range: 12.0,
            monster_hearing_range: 10.0,
            monster_fov: 1.75,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chase_speed_preserves_corner_escape_window() {
        let config = GameConfig::default();
        assert!(config.player_speed < config.monster_chase_speed);
        assert!(config.monster_chase_speed <= config.player_speed * 1.15);
    }
}
