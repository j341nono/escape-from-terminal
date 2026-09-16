mod config;
mod game;
mod generator;
mod geom;
mod input;
mod map;
mod monster;
mod pathfinding;
mod player;
mod raycaster;
mod renderer;
mod terminal;

use std::{
    error::Error,
    io, thread,
    time::{Duration, Instant},
};

use crossterm::{event, terminal as crossterm_terminal};

use crate::{
    args::seed_from_args,
    config::{MIN_HEIGHT, MIN_WIDTH, TARGET_FPS},
    game::Game,
    input::{Command, InputState},
    renderer::render_frame,
    terminal::TerminalSession,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("{}: {error}", config::GAME_NAME);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let seed = seed_from_args(std::env::args())
        .map_err(io::Error::other)?
        .unwrap_or_else(rand::random);
    let mut game = Game::new(seed);
    let mut terminal = TerminalSession::enter()?;
    let mut previous_frame = Instant::now();
    let mut input = InputState::new(previous_frame);
    let frame_budget = Duration::from_secs_f64(1.0 / TARGET_FPS as f64);

    while game.is_running() {
        let frame_start = Instant::now();
        let delta_seconds = frame_start.duration_since(previous_frame).as_secs_f32();
        previous_frame = frame_start;

        while event::poll(Duration::ZERO)? {
            if let event::Event::Key(key) = event::read()? {
                let command = input.handle_key(key, frame_start);
                if matches!(
                    command,
                    Command::Confirm | Command::TogglePause | Command::Retry | Command::NewFacility
                ) {
                    input.clear_movement(frame_start);
                }
                game.handle_command(command);
            }
        }

        let (width, height) = crossterm_terminal::size()?;
        if width >= MIN_WIDTH && height >= MIN_HEIGHT {
            game.update(input.movement(frame_start), delta_seconds);
        }
        let frame = render_frame(&game, usize::from(width), usize::from(height));
        terminal.draw(&frame.to_terminal_string())?;

        if let Some(remaining) = frame_budget.checked_sub(frame_start.elapsed()) {
            thread::sleep(remaining);
        }
    }
    Ok(())
}
mod args;
