use std::f32::consts::FRAC_PI_3;

pub const GAME_NAME: &str = "NULL SECTOR";
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;
pub const TARGET_FPS: u64 = 30;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub map_width: usize,
    pub map_height: usize,
    pub fov: f32,
    pub render_distance: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub rotation_speed: f32,
    pub player_radius: f32,
    pub stamina_seconds: f32,
    pub stamina_recovery: f32,
    pub monster_wander_speed: f32,
    pub monster_chase_speed: f32,
    pub monster_sight_range: f32,
    pub monster_fov: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            map_width: 55,
            map_height: 39,
            fov: FRAC_PI_3,
            render_distance: 28.0,
            walk_speed: 2.4,
            sprint_speed: 4.2,
            rotation_speed: 2.25,
            player_radius: 0.22,
            stamina_seconds: 5.5,
            stamina_recovery: 0.65,
            monster_wander_speed: 1.55,
            monster_chase_speed: 3.45,
            monster_sight_range: 13.0,
            monster_fov: 1.65,
        }
    }
}
