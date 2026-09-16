use std::f32::consts::FRAC_PI_3;

pub const GAME_NAME: &str = "NULL SECTOR";
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;
pub const TARGET_FPS: u64 = 30;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub fov: f32,
    pub render_distance: f32,
    pub walk_speed: f32,
    pub rotation_speed: f32,
    pub player_radius: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            fov: FRAC_PI_3,
            render_distance: 28.0,
            walk_speed: 2.4,
            rotation_speed: 2.25,
            player_radius: 0.22,
        }
    }
}
