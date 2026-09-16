use crate::{
    config::GameConfig,
    generator::generate,
    input::{Command, MovementInput},
    map::Map,
    monster::Monster,
    player::Player,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    Intro,
    Playing,
    Paused,
    Caught,
    Escaped,
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
    pub monster: Monster,
    pub stamina: f32,
    pub sprint_exhausted: bool,
    pub power_restored: bool,
    pub elapsed_seconds: f32,
    pub chase_count: u32,
    pub caught_flash: f32,
    pub status_message: Option<&'static str>,
    pub status_timer: f32,
    interaction_noise: f32,
    rng: StdRng,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let facility = generate(seed);
        let config = GameConfig::default();
        let stamina = config.stamina_seconds;
        Self {
            state: GameState::Title,
            player: Player::new(facility.start.center(), 0.0),
            map: facility.map,
            config,
            seed,
            objective_cell: facility.objective,
            exit_cell: facility.exit,
            monster: Monster::new(facility.monster_spawn),
            stamina,
            sprint_exhausted: false,
            power_restored: false,
            elapsed_seconds: 0.0,
            chase_count: 0,
            caught_flash: 0.0,
            status_message: None,
            status_timer: 0.0,
            interaction_noise: 0.0,
            rng: StdRng::seed_from_u64(seed ^ 0x4e55_4c4c),
        }
    }

    pub fn handle_command(&mut self, command: Command) {
        if command == Command::Retry && matches!(self.state, GameState::Caught | GameState::Escaped)
        {
            *self = Self::new(self.seed);
            self.state = GameState::Playing;
            return;
        }
        if command == Command::NewFacility
            && matches!(self.state, GameState::Caught | GameState::Escaped)
        {
            let seed = self.rng.random();
            *self = Self::new(seed);
            self.state = GameState::Playing;
            return;
        }
        self.state = match (self.state, command) {
            (_, Command::Quit) => GameState::Exiting,
            (GameState::Title, Command::Confirm) => GameState::Intro,
            (GameState::Intro, Command::Confirm) => GameState::Playing,
            (GameState::Playing, Command::TogglePause) => GameState::Paused,
            (GameState::Paused, Command::TogglePause | Command::Confirm) => GameState::Playing,
            (state, _) => state,
        };
        if command == Command::Interact && self.state == GameState::Playing {
            self.interact();
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
        self.elapsed_seconds += delta_seconds;
        self.player
            .rotate(input.turn, self.config.rotation_speed, delta_seconds);
        let moving = input.forward != 0.0 || input.strafe != 0.0;
        if self.sprint_exhausted && self.stamina >= self.config.stamina_resume {
            self.sprint_exhausted = false;
        }
        let sprinting = input.sprint && moving && !self.sprint_exhausted;
        if sprinting {
            self.stamina = (self.stamina - delta_seconds).max(0.0);
            if self.stamina == 0.0 {
                self.sprint_exhausted = true;
            }
        } else {
            self.stamina = (self.stamina + self.config.stamina_recovery * delta_seconds)
                .min(self.config.stamina_seconds);
        }
        let displacement = self.player.movement_direction(input)
            * ((if sprinting {
                self.config.sprint_speed
            } else {
                self.config.walk_speed
            }) * delta_seconds);
        self.player
            .move_with_collision(&self.map, displacement, self.config.player_radius);
        let movement_noise: f32 = if sprinting {
            1.0
        } else if moving {
            0.42
        } else {
            0.0
        };
        let noise = movement_noise.max(self.interaction_noise);
        self.interaction_noise = 0.0;
        let report = self.monster.update(
            &mut self.map,
            self.player.position,
            noise,
            delta_seconds,
            &self.config,
            &mut self.rng,
        );
        if report.spotted {
            self.chase_count += 1;
            self.caught_flash = 0.9;
        }
        self.caught_flash = (self.caught_flash - delta_seconds).max(0.0);
        self.status_timer = (self.status_timer - delta_seconds).max(0.0);
        if self.status_timer == 0.0 {
            self.status_message = None;
        }
        if report.caught {
            self.state = GameState::Caught;
        }
    }

    fn interact(&mut self) {
        let cell = crate::geom::Cell::new(
            self.player.position.x as usize,
            self.player.position.y as usize,
        );
        if Self::near(cell, self.objective_cell) {
            if self.power_restored {
                self.set_status("EMERGENCY POWER: ONLINE");
            } else {
                self.power_restored = true;
                self.interaction_noise = 0.8;
                self.set_status("EMERGENCY POWER RESTORED");
            }
            return;
        }
        if Self::near(cell, self.exit_cell) {
            if self.power_restored {
                self.state = GameState::Escaped;
            } else {
                self.set_status("EXIT LOCKED - RESTORE EMERGENCY POWER");
            }
            return;
        }
        self.interact_door();
    }

    fn interact_door(&mut self) {
        let Some(door) = self.door_in_front() else {
            return;
        };
        if self.map.tile(door) == crate::map::Tile::DoorOpen && self.monster.cell() == door {
            self.set_status("DOOR BLOCKED");
            return;
        }
        let was_closed = self.map.tile(door) == crate::map::Tile::DoorClosed;
        if self.map.toggle_door(door) {
            self.interaction_noise = 0.65;
            self.set_status(if was_closed {
                "DOOR OPEN"
            } else {
                "DOOR CLOSED"
            });
        }
    }

    fn door_in_front(&self) -> Option<crate::geom::Cell> {
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
        best.map(|(door, _)| door)
    }

    fn near(left: crate::geom::Cell, right: crate::geom::Cell) -> bool {
        left.x.abs_diff(right.x) + left.y.abs_diff(right.y) <= 1
    }

    fn set_status(&mut self, message: &'static str) {
        self.status_message = Some(message);
        self.status_timer = 2.4;
    }

    pub fn objective_text(&self) -> &'static str {
        if self.power_restored {
            "REACH EMERGENCY EXIT"
        } else {
            "RESTORE EMERGENCY POWER"
        }
    }

    pub fn interaction_hint(&self) -> Option<&'static str> {
        let cell = crate::geom::Cell::new(
            self.player.position.x as usize,
            self.player.position.y as usize,
        );
        if Self::near(cell, self.objective_cell) {
            return Some(if self.power_restored {
                "POWER IS ONLINE"
            } else {
                "[E] RESTORE EMERGENCY POWER"
            });
        }
        if Self::near(cell, self.exit_cell) {
            return Some(if self.power_restored {
                "[E] OPEN EMERGENCY EXIT"
            } else {
                "EXIT LOCKED - RESTORE POWER"
            });
        }
        self.door_in_front().map(|door| {
            if self.map.tile(door) == crate::map::Tile::DoorClosed {
                "[E] OPEN DOOR"
            } else {
                "[E] CLOSE DOOR"
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_between_foundation_states() {
        let mut game = Game::new(1);
        game.handle_command(Command::Confirm);
        assert_eq!(game.state, GameState::Intro);
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
        assert!(game.interaction_noise > 0.0);
    }

    #[test]
    fn power_then_exit_completes_run() {
        let mut game = Game::new(7);
        game.state = GameState::Playing;
        game.player.position = game.objective_cell.center();
        game.handle_command(Command::Interact);
        assert!(game.power_restored);
        game.player.position = game.exit_cell.center();
        game.handle_command(Command::Interact);
        assert_eq!(game.state, GameState::Escaped);
    }

    #[test]
    fn sprinting_consumes_and_resting_recovers_stamina() {
        let mut game = Game::new(11);
        game.state = GameState::Playing;
        let sprint = MovementInput {
            forward: 1.0,
            sprint: true,
            ..MovementInput::default()
        };
        game.update(sprint, 0.1);
        let spent = game.stamina;
        assert!(spent < game.config.stamina_seconds);
        game.update(MovementInput::default(), 0.1);
        assert!(game.stamina > spent);
    }

    #[test]
    fn capture_enters_retryable_state() {
        let mut game = Game::new(19);
        game.state = GameState::Playing;
        game.monster.position = game.player.position;
        game.update(MovementInput::default(), 0.01);
        assert_eq!(game.state, GameState::Caught);
        game.handle_command(Command::Retry);
        assert_eq!(game.state, GameState::Playing);
        assert_eq!(game.seed, 19);
        assert_eq!(game.stamina, game.config.stamina_seconds);
        assert!(!game.power_restored);
        assert_eq!(game.chase_count, 0);
        assert!(!game.sprint_exhausted);
    }

    #[test]
    fn locked_exit_provides_feedback() {
        let mut game = Game::new(23);
        game.state = GameState::Playing;
        game.player.position = game.exit_cell.center();
        game.handle_command(Command::Interact);
        assert_eq!(game.state, GameState::Playing);
        assert_eq!(
            game.status_message,
            Some("EXIT LOCKED - RESTORE EMERGENCY POWER")
        );
    }

    #[test]
    fn exhausted_sprint_requires_partial_recovery() {
        let mut game = Game::new(29);
        game.state = GameState::Playing;
        game.stamina = 0.01;
        let sprint = MovementInput {
            forward: 1.0,
            sprint: true,
            ..MovementInput::default()
        };
        game.update(sprint, 0.1);
        assert!(game.sprint_exhausted);
        game.update(sprint, 0.1);
        assert!(game.stamina > 0.0);
        assert!(game.sprint_exhausted);
    }

    #[test]
    fn retry_reconstructs_same_facility_without_stale_state() {
        let mut game = Game::new(31);
        let original_map = game.map.clone();
        let original_spawn = game.monster.position;
        game.state = GameState::Caught;
        game.power_restored = true;
        game.stamina = 0.0;
        game.chase_count = 5;
        game.monster.path.push(game.exit_cell);
        game.handle_command(Command::Retry);
        assert_eq!(game.map, original_map);
        assert_eq!(game.monster.position, original_spawn);
        assert!(game.monster.path.is_empty());
        assert_eq!(game.stamina, game.config.stamina_seconds);
        assert_eq!(game.chase_count, 0);
        assert!(!game.power_restored);
    }

    #[test]
    fn new_facility_changes_seed() {
        let mut game = Game::new(37);
        game.state = GameState::Caught;
        game.handle_command(Command::NewFacility);
        assert_ne!(game.seed, 37);
        assert_eq!(game.state, GameState::Playing);
    }

    #[test]
    fn door_cannot_close_on_monster() {
        let mut game = Game::new(41);
        game.map = Map::from_ascii(&["#####", "#.d.#", "#####"]).unwrap();
        game.player = Player::new(crate::geom::Vec2::new(1.5, 1.5), 0.0);
        game.monster.position = crate::geom::Vec2::new(2.5, 1.5);
        game.state = GameState::Playing;
        game.handle_command(Command::Interact);
        assert_eq!(
            game.map.tile(crate::geom::Cell::new(2, 1)),
            crate::map::Tile::DoorOpen
        );
        assert_eq!(game.status_message, Some("DOOR BLOCKED"));
    }
}
