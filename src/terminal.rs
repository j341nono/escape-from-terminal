use std::io::{self, Stdout, Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

pub struct TerminalSession {
    output: Stdout,
    active: bool,
    previous_frame: String,
}

impl TerminalSession {
    pub fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut output = stdout();
        if let Err(error) = execute!(output, EnterAlternateScreen, Hide, Clear(ClearType::All)) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        Ok(Self {
            output,
            active: true,
            previous_frame: String::new(),
        })
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
        let _ = execute!(self.output, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        self.active = false;
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.restore();
    }
}
