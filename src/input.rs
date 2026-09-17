use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

const KEY_LATCH: Duration = Duration::from_millis(140);
const COMMAND_DEBOUNCE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    None,
    Confirm,
    TogglePause,
    Interact,
    Retry,
    NewFacility,
    Quit,
}

#[derive(Debug)]
pub struct InputState {
    forward_until: Instant,
    backward_until: Instant,
    left_until: Instant,
    right_until: Instant,
    turn_left_until: Instant,
    turn_right_until: Instant,
    look_back_until: Instant,
    command_until: Instant,
}

impl InputState {
    pub fn new(now: Instant) -> Self {
        Self {
            forward_until: now,
            backward_until: now,
            left_until: now,
            right_until: now,
            turn_left_until: now,
            turn_right_until: now,
            look_back_until: now,
            command_until: now,
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent, now: Instant) -> Command {
        let active = matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat);
        let deadline = if active { now + KEY_LATCH } else { now };
        match event.code {
            KeyCode::Char('w' | 'W') | KeyCode::Up => self.forward_until = deadline,
            KeyCode::Char('s' | 'S') | KeyCode::Down => self.backward_until = deadline,
            KeyCode::Char('a' | 'A') => self.left_until = deadline,
            KeyCode::Char('d' | 'D') => self.right_until = deadline,
            KeyCode::Left => self.turn_left_until = deadline,
            KeyCode::Right => self.turn_right_until = deadline,
            KeyCode::Char('f' | 'F') => self.look_back_until = deadline,
            _ => {}
        }
        if event.kind != KeyEventKind::Press || now < self.command_until {
            return Command::None;
        }
        let command = match event.code {
            KeyCode::Enter => Command::Confirm,
            KeyCode::Esc => Command::TogglePause,
            KeyCode::Char('e' | 'E') => Command::Interact,
            KeyCode::Char('r' | 'R') => Command::Retry,
            KeyCode::Char('n' | 'N') => Command::NewFacility,
            KeyCode::Char('q' | 'Q') => Command::Quit,
            _ => Command::None,
        };
        if command != Command::None {
            self.command_until = now + COMMAND_DEBOUNCE;
        }
        command
    }

    pub fn movement(&self, now: Instant) -> MovementInput {
        MovementInput {
            forward: axis(self.forward_until, self.backward_until, now),
            strafe: axis(self.right_until, self.left_until, now),
            turn: axis(self.turn_right_until, self.turn_left_until, now),
            look_back: self.look_back_until > now,
        }
    }

    pub fn clear_movement(&mut self, now: Instant) {
        self.forward_until = now;
        self.backward_until = now;
        self.left_until = now;
        self.right_until = now;
        self.turn_left_until = now;
        self.turn_right_until = now;
        self.look_back_until = now;
    }
}

fn axis(positive_until: Instant, negative_until: Instant, now: Instant) -> f32 {
    f32::from(positive_until > now) - f32::from(negative_until > now)
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MovementInput {
    pub forward: f32,
    pub strafe: f32,
    pub turn: f32,
    pub look_back: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_movement_and_rotation_keys() {
        let now = Instant::now();
        let mut input = InputState::new(now);
        input.handle_key(
            KeyEvent::new(KeyCode::Char('w'), crossterm::event::KeyModifiers::NONE),
            now,
        );
        input.handle_key(
            KeyEvent::new(KeyCode::Left, crossterm::event::KeyModifiers::NONE),
            now,
        );
        let movement = input.movement(now);
        assert_eq!(movement.forward, 1.0);
        assert_eq!(movement.turn, -1.0);
    }

    #[test]
    fn latches_movement_rotation_and_look_back_independently() {
        let now = Instant::now();
        let mut input = InputState::new(now);
        for key in [
            KeyCode::Char('w'),
            KeyCode::Char('a'),
            KeyCode::Left,
            KeyCode::Char('f'),
        ] {
            input.handle_key(
                KeyEvent::new(key, crossterm::event::KeyModifiers::NONE),
                now,
            );
        }
        assert_eq!(
            input.movement(now),
            MovementInput {
                forward: 1.0,
                strafe: -1.0,
                turn: -1.0,
                look_back: true,
            }
        );
    }

    #[test]
    fn opposite_keys_cancel_each_other() {
        let now = Instant::now();
        let mut input = InputState::new(now);
        input.handle_key(
            KeyEvent::new(KeyCode::Char('a'), crossterm::event::KeyModifiers::NONE),
            now,
        );
        input.handle_key(
            KeyEvent::new(KeyCode::Char('d'), crossterm::event::KeyModifiers::NONE),
            now,
        );
        assert_eq!(input.movement(now).strafe, 0.0);
    }

    #[test]
    fn modal_transition_clears_latched_keys() {
        let now = Instant::now();
        let mut input = InputState::new(now);
        input.handle_key(
            KeyEvent::new(KeyCode::Char('w'), crossterm::event::KeyModifiers::NONE),
            now,
        );
        input.handle_key(
            KeyEvent::new(KeyCode::Char(' '), crossterm::event::KeyModifiers::NONE),
            now,
        );
        input.clear_movement(now);
        assert_eq!(input.movement(now), MovementInput::default());
    }

    #[test]
    fn one_shot_commands_are_debounced() {
        let now = Instant::now();
        let mut input = InputState::new(now);
        let enter = KeyEvent::new(KeyCode::Enter, crossterm::event::KeyModifiers::NONE);
        assert_eq!(input.handle_key(enter, now), Command::Confirm);
        assert_eq!(input.handle_key(enter, now), Command::None);
        assert_eq!(
            input.handle_key(enter, now + COMMAND_DEBOUNCE),
            Command::Confirm
        );
    }
}
