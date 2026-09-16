use crate::{
    config::GameConfig,
    input::Command,
    level::training_sector,
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
}

impl Game {
    pub fn new() -> Result<Self, String> {
        let level = training_sector()?;
        Ok(Self {
            state: GameState::Title,
            player: Player::new(level.player_start, level.player_angle),
            map: level.map,
            config: GameConfig::default(),
        })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_between_foundation_states() {
        let mut game = Game::new().expect("game should initialize");
        game.handle_command(Command::Confirm);
        assert_eq!(game.state, GameState::Playing);
        game.handle_command(Command::TogglePause);
        assert_eq!(game.state, GameState::Paused);
        game.handle_command(Command::TogglePause);
        assert_eq!(game.state, GameState::Playing);
        game.handle_command(Command::Quit);
        assert!(!game.is_running());
    }
}
