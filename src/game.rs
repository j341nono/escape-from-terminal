use crate::{
    config::GameConfig,
    generator::generate,
    input::{Command, MovementInput},
    map::Map,
    player::Player,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    Playing,
    Paused,
    Exiting,
}

pub struct Game {
    pub state: GameState,
    pub map: Map,
    pub player: Player,
    pub config: GameConfig,
    pub seed: u64,
    pub objective_cell: crate::geom::Cell,
    pub exit_cell: crate::geom::Cell,
    pub monster_spawn: crate::geom::Cell,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let facility = generate(seed);
        Self {
            state: GameState::Title,
            player: Player::new(facility.start.center(), 0.0),
            map: facility.map,
            config: GameConfig::default(),
            seed,
            objective_cell: facility.objective,
            exit_cell: facility.exit,
            monster_spawn: facility.monster_spawn,
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        self.state = match (self.state, command) {
            (_, Command::Quit) => GameState::Exiting,
            (GameState::Title, Command::Confirm) => GameState::Playing,
            (GameState::Playing, Command::TogglePause) => GameState::Paused,
            (GameState::Paused, Command::TogglePause | Command::Confirm) => GameState::Playing,
            (state, _) => state,
        };
    }

    pub fn is_running(&self) -> bool {
        self.state != GameState::Exiting
    }

    pub fn update(&mut self, input: MovementInput, delta_seconds: f32) {
        if self.state != GameState::Playing {
            return;
        }

        let delta_seconds = delta_seconds.clamp(0.0, 0.1);
        self.player
            .rotate(input.turn, self.config.rotation_speed, delta_seconds);
        let displacement =
            self.player.movement_direction(input) * (self.config.walk_speed * delta_seconds);
        self.player
            .move_with_collision(&self.map, displacement, self.config.player_radius);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_between_foundation_states() {
        let mut game = Game::new(1);
        game.handle_command(Command::Confirm);
        assert_eq!(game.state, GameState::Playing);
        game.handle_command(Command::TogglePause);
        assert_eq!(game.state, GameState::Paused);
        game.handle_command(Command::TogglePause);
        assert_eq!(game.state, GameState::Playing);
        game.handle_command(Command::Quit);
        assert!(!game.is_running());
    }

    #[test]
    fn movement_updates_only_while_playing() {
        let mut game = Game::new(1);
        let start = game.player.position;
        let input = MovementInput {
            forward: 1.0,
            ..MovementInput::default()
        };
        game.update(input, 0.1);
        assert_eq!(game.player.position, start);
        game.handle_command(Command::Confirm);
        game.update(input, 0.1);
        assert!(game.player.position.x > start.x);
    }
}
