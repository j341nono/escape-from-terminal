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
    pub sprint_speed: f32,
    pub rotation_speed: f32,
    pub player_radius: f32,
    pub stamina_seconds: f32,
    pub stamina_recovery: f32,
    pub stamina_resume: f32,
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
            walk_speed: 2.4,
            sprint_speed: 4.0,
            rotation_speed: 2.25,
            player_radius: 0.22,
            stamina_seconds: 5.5,
            stamina_recovery: 0.72,
            stamina_resume: 1.1,
            monster_wander_speed: 1.35,
            monster_chase_speed: 3.25,
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
    fn chase_speed_preserves_escape_window() {
        let config = GameConfig::default();
        assert!(config.walk_speed < config.monster_chase_speed);
        assert!(config.monster_chase_speed < config.sprint_speed);
        assert!(config.stamina_seconds >= 5.0);
        assert!(config.stamina_recovery < 1.0);
    }
}
