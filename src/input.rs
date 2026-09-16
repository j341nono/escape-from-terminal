use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

const KEY_LATCH: Duration = Duration::from_millis(140);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    None,
    Confirm,
    TogglePause,
    Interact,
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
    sprint_until: Instant,
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
            sprint_until: now,
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
            KeyCode::Char(' ') => self.sprint_until = deadline,
            KeyCode::Enter if active => return Command::Confirm,
            KeyCode::Esc if active => return Command::TogglePause,
            KeyCode::Char('e' | 'E') if active => return Command::Interact,
            KeyCode::Char('q' | 'Q') if active => return Command::Quit,
            _ => {}
        }
        Command::None
    }

    pub fn movement(&self, now: Instant) -> MovementInput {
        MovementInput {
            forward: axis(self.forward_until, self.backward_until, now),
            strafe: axis(self.right_until, self.left_until, now),
            turn: axis(self.turn_right_until, self.turn_left_until, now),
            sprint: self.sprint_until > now,
        }
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
    pub sprint: bool,
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
}
