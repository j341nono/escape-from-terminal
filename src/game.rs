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
        if command == Command::Interact && self.state == GameState::Playing {
            self.interact_door();
        }
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

    fn interact_door(&mut self) {
        let origin = crate::geom::Cell::new(
            self.player.position.x as usize,
            self.player.position.y as usize,
        );
        let forward = self.player.forward();
        let mut best = None;
        for (dx, dy) in [(0isize, -1isize), (1, 0), (0, 1), (-1, 0)] {
            let Some(x) = origin.x.checked_add_signed(dx) else {
                continue;
            };
            let Some(y) = origin.y.checked_add_signed(dy) else {
                continue;
            };
            let candidate = crate::geom::Cell::new(x, y);
            let direction = crate::geom::Vec2::new(dx as f32, dy as f32);
            if matches!(
                self.map.tile(candidate),
                crate::map::Tile::DoorClosed | crate::map::Tile::DoorOpen
            ) {
                let score = forward.dot(direction);
                if score > 0.15 && best.is_none_or(|(_, current)| score > current) {
                    best = Some((candidate, score));
                }
            }
        }
        if let Some((door, _)) = best {
            self.map.toggle_door(door);
        }
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

    #[test]
    fn interaction_toggles_door_in_front_of_player() {
        let mut game = Game::new(1);
        game.map = Map::from_ascii(&["#####", "#.D.#", "#####"]).unwrap();
        game.player = Player::new(crate::geom::Vec2::new(1.5, 1.5), 0.0);
        game.state = GameState::Playing;
        game.handle_command(Command::Interact);
        assert_eq!(
            game.map.tile(crate::geom::Cell::new(2, 1)),
            crate::map::Tile::DoorOpen
        );
    }
}
