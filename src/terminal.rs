use std::io::{self, Stdout, Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute, queue,
    style::Print,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardMode {
    Enhanced,
    Legacy,
}

impl KeyboardMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Enhanced => "Enhanced",
            Self::Legacy => "Legacy",
        }
    }
}

const KEYBOARD_FLAGS: KeyboardEnhancementFlags =
    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
        .union(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        .union(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES);

pub struct TerminalSession {
    output: Stdout,
    active: bool,
    keyboard_mode: KeyboardMode,
    previous_frame: String,
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        let keyboard_mode = if matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        ) {
            KeyboardMode::Enhanced
        } else {
            KeyboardMode::Legacy
        };
        if std::env::var_os("NULL_SECTOR_INPUT_DEBUG").is_some() {
            eprintln!("NULL SECTOR keyboard mode: {}", keyboard_mode.label());
        }
        enable_raw_mode()?;
        let mut output = stdout();
        if let Err(error) = execute!(output, EnterAlternateScreen, Hide, Clear(ClearType::All)) {
            let _ = execute!(output, Show, LeaveAlternateScreen);
            let _ = disable_raw_mode();
            return Err(error);
        }
        if keyboard_mode == KeyboardMode::Enhanced
            && let Err(error) = execute!(output, PushKeyboardEnhancementFlags(KEYBOARD_FLAGS))
        {
            let _ = execute!(
                output,
                PopKeyboardEnhancementFlags,
                Show,
                LeaveAlternateScreen
            );
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self {
            output,
            active: true,
            keyboard_mode,
            previous_frame: String::new(),
        })
    }

    pub const fn keyboard_mode(&self) -> KeyboardMode {
        self.keyboard_mode
    }

    pub fn draw(&mut self, frame: &str) -> io::Result<()> {
        if self.previous_frame == frame {
            return Ok(());
        }
        queue!(self.output, MoveTo(0, 0), Print(frame))?;
        self.output.flush()?;
        frame.clone_into(&mut self.previous_frame);
        Ok(())
    }

    fn restore(&mut self) {
        if !self.active {
            return;
        }
        if self.keyboard_mode == KeyboardMode::Enhanced {
            let _ = execute!(
                self.output,
                PopKeyboardEnhancementFlags,
                Show,
                LeaveAlternateScreen
            );
        } else {
            let _ = execute!(self.output, Show, LeaveAlternateScreen);
        }
        let _ = disable_raw_mode();
        self.active = false;
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enhanced_mode_requests_release_events_for_plain_keys() {
        assert!(KEYBOARD_FLAGS.contains(KeyboardEnhancementFlags::REPORT_EVENT_TYPES));
        assert!(KEYBOARD_FLAGS.contains(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES));
        assert!(KEYBOARD_FLAGS.contains(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES));
    }
}
