use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::terminal::KeyboardMode;

const LEGACY_KEY_LATCH: Duration = Duration::from_millis(140);
const LEGACY_LOOK_TOGGLE_DEBOUNCE: Duration = Duration::from_millis(800);
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
    mode: KeyboardMode,
    held: HeldKeys,
    legacy: LegacyKeys,
    command_until: Instant,
}

#[derive(Debug, Default)]
struct HeldKeys {
    forward: bool,
    backward: bool,
    strafe_left: bool,
    strafe_right: bool,
    turn_left: bool,
    turn_right: bool,
    look_back: bool,
}

#[derive(Debug)]
struct LegacyKeys {
    forward_until: Instant,
    backward_until: Instant,
    strafe_left_until: Instant,
    strafe_right_until: Instant,
    turn_left_until: Instant,
    turn_right_until: Instant,
    look_back: bool,
    next_look_toggle: Instant,
}

impl InputState {
    pub fn new(mode: KeyboardMode, now: Instant) -> Self {
        Self {
            mode,
            held: HeldKeys::default(),
            legacy: LegacyKeys::new(now),
            command_until: now,
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent, now: Instant) -> Command {
        match self.mode {
            KeyboardMode::Enhanced => self.held.update(event),
            KeyboardMode::Legacy => self.legacy.update(event, now),
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
        match self.mode {
            KeyboardMode::Enhanced => self.held.movement(),
            KeyboardMode::Legacy => self.legacy.movement(now),
        }
    }

    pub fn clear_movement(&mut self, now: Instant) {
        self.held = HeldKeys::default();
        self.legacy.clear(now);
    }
}

impl HeldKeys {
    fn update(&mut self, event: KeyEvent) {
        let active = event.kind != KeyEventKind::Release;
        match event.code {
            KeyCode::Char('w' | 'W') | KeyCode::Up => self.forward = active,
            KeyCode::Char('s' | 'S') | KeyCode::Down => self.backward = active,
            KeyCode::Char('a' | 'A') => self.strafe_left = active,
            KeyCode::Char('d' | 'D') => self.strafe_right = active,
            KeyCode::Left => self.turn_left = active,
            KeyCode::Right => self.turn_right = active,
            KeyCode::Char('f' | 'F') => self.look_back = active,
            _ => {}
        }
    }

    fn movement(&self) -> MovementInput {
        MovementInput {
            forward: bool_axis(self.forward, self.backward),
            strafe: bool_axis(self.strafe_right, self.strafe_left),
            turn: bool_axis(self.turn_right, self.turn_left),
            look_back: self.look_back,
        }
    }
}

impl LegacyKeys {
    fn new(now: Instant) -> Self {
        Self {
            forward_until: now,
            backward_until: now,
            strafe_left_until: now,
            strafe_right_until: now,
            turn_left_until: now,
            turn_right_until: now,
            look_back: false,
            next_look_toggle: now,
        }
    }

    fn update(&mut self, event: KeyEvent, now: Instant) {
        if matches!(event.code, KeyCode::Char('f' | 'F')) {
            if event.kind == KeyEventKind::Press && now >= self.next_look_toggle {
                self.look_back = !self.look_back;
                self.next_look_toggle = now + LEGACY_LOOK_TOGGLE_DEBOUNCE;
            }
            return;
        }
        let active = event.kind != KeyEventKind::Release;
        let deadline = if active { now + LEGACY_KEY_LATCH } else { now };
        match event.code {
            KeyCode::Char('w' | 'W') | KeyCode::Up => self.forward_until = deadline,
            KeyCode::Char('s' | 'S') | KeyCode::Down => self.backward_until = deadline,
            KeyCode::Char('a' | 'A') => self.strafe_left_until = deadline,
            KeyCode::Char('d' | 'D') => self.strafe_right_until = deadline,
            KeyCode::Left => self.turn_left_until = deadline,
            KeyCode::Right => self.turn_right_until = deadline,
            _ => {}
        }
    }

    fn movement(&self, now: Instant) -> MovementInput {
        MovementInput {
            forward: deadline_axis(self.forward_until, self.backward_until, now),
            strafe: deadline_axis(self.strafe_right_until, self.strafe_left_until, now),
            turn: deadline_axis(self.turn_right_until, self.turn_left_until, now),
            look_back: self.look_back,
        }
    }

    fn clear(&mut self, now: Instant) {
        *self = Self::new(now);
    }
}

fn bool_axis(positive: bool, negative: bool) -> f32 {
    f32::from(positive) - f32::from(negative)
}

fn deadline_axis(positive_until: Instant, negative_until: Instant, now: Instant) -> f32 {
    bool_axis(positive_until > now, negative_until > now)
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
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::NONE, kind)
    }

    #[test]
    fn enhanced_events_hold_movement_while_another_key_repeats() {
        let now = Instant::now();
        let mut input = InputState::new(KeyboardMode::Enhanced, now);
        input.handle_key(key(KeyCode::Char('w'), KeyEventKind::Press), now);
        input.handle_key(key(KeyCode::Left, KeyEventKind::Press), now);
        input.handle_key(key(KeyCode::Left, KeyEventKind::Repeat), now);
        input.handle_key(key(KeyCode::Left, KeyEventKind::Repeat), now);

        assert_eq!(
            input.movement(now + Duration::from_secs(10)),
            MovementInput {
                forward: 1.0,
                turn: -1.0,
                ..MovementInput::default()
            }
        );

        input.handle_key(key(KeyCode::Left, KeyEventKind::Release), now);
        assert_eq!(input.movement(now).forward, 1.0);
        assert_eq!(input.movement(now).turn, 0.0);

        input.handle_key(key(KeyCode::Char('w'), KeyEventKind::Release), now);
        assert_eq!(input.movement(now), MovementInput::default());
    }

    #[test]
    fn enhanced_keys_track_movement_rotation_and_look_back_independently() {
        let now = Instant::now();
        let mut input = InputState::new(KeyboardMode::Enhanced, now);
        for key in [
            KeyCode::Char('w'),
            KeyCode::Char('a'),
            KeyCode::Left,
            KeyCode::Char('f'),
        ] {
            input.handle_key(KeyEvent::new(key, KeyModifiers::NONE), now);
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
    fn enhanced_movement_and_turn_states_overlap_for_each_direction() {
        let now = Instant::now();
        for (move_key, turn_key, forward, strafe, turn) in [
            (KeyCode::Char('w'), KeyCode::Left, 1.0, 0.0, -1.0),
            (KeyCode::Char('w'), KeyCode::Right, 1.0, 0.0, 1.0),
            (KeyCode::Char('s'), KeyCode::Left, -1.0, 0.0, -1.0),
            (KeyCode::Char('s'), KeyCode::Right, -1.0, 0.0, 1.0),
            (KeyCode::Char('a'), KeyCode::Left, 0.0, -1.0, -1.0),
            (KeyCode::Char('d'), KeyCode::Right, 0.0, 1.0, 1.0),
        ] {
            let mut input = InputState::new(KeyboardMode::Enhanced, now);
            for key in [move_key, turn_key] {
                input.handle_key(KeyEvent::new(key, KeyModifiers::NONE), now);
            }
            assert_eq!(
                input.movement(now),
                MovementInput {
                    forward,
                    strafe,
                    turn,
                    look_back: false,
                }
            );
        }
    }

    #[test]
    fn enhanced_look_back_uses_press_and_release() {
        let now = Instant::now();
        let mut input = InputState::new(KeyboardMode::Enhanced, now);
        input.handle_key(key(KeyCode::Char('f'), KeyEventKind::Press), now);
        assert!(input.movement(now).look_back);
        input.handle_key(key(KeyCode::Char('f'), KeyEventKind::Repeat), now);
        assert!(input.movement(now).look_back);
        input.handle_key(key(KeyCode::Char('f'), KeyEventKind::Release), now);
        assert!(!input.movement(now).look_back);
    }

    #[test]
    fn legacy_movement_uses_latches_but_look_back_toggles() {
        let now = Instant::now();
        let mut input = InputState::new(KeyboardMode::Legacy, now);
        input.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE), now);
        input.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE), now);
        assert_eq!(input.movement(now).forward, 1.0);
        assert!(input.movement(now).look_back);
        assert_eq!(
            input
                .movement(now + LEGACY_KEY_LATCH + Duration::from_millis(1))
                .forward,
            0.0
        );
        input.handle_key(
            KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE),
            now + LEGACY_LOOK_TOGGLE_DEBOUNCE,
        );
        assert!(!input.movement(now).look_back);
    }

    #[test]
    fn modal_transition_clears_all_key_modes() {
        let now = Instant::now();
        for mode in [KeyboardMode::Enhanced, KeyboardMode::Legacy] {
            let mut input = InputState::new(mode, now);
            input.handle_key(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE), now);
            input.handle_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE), now);
            input.clear_movement(now);
            assert_eq!(input.movement(now), MovementInput::default());
        }
    }

    #[test]
    fn one_shot_commands_are_debounced() {
        let now = Instant::now();
        let mut input = InputState::new(KeyboardMode::Enhanced, now);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(input.handle_key(enter, now), Command::Confirm);
        assert_eq!(input.handle_key(enter, now), Command::None);
        assert_eq!(
            input.handle_key(enter, now + COMMAND_DEBOUNCE),
            Command::Confirm
        );
    }
}
