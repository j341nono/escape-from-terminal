use crate::{
    config::{GAME_NAME, MIN_HEIGHT, MIN_WIDTH},
    game::{Game, GameState},
    raycaster::{WallSide, cast_view},
};

pub fn render_frame(game: &Game, width: usize, height: usize) -> FrameBuffer {
    if width < usize::from(MIN_WIDTH) || height < usize::from(MIN_HEIGHT) {
        let mut frame = FrameBuffer::new(width, height, ' ');
        frame.write_centered(height.saturating_sub(2) / 2, "Terminal too small.");
        frame.write_centered(
            height / 2,
            &format!("Please resize to at least {MIN_WIDTH}x{MIN_HEIGHT}."),
        );
        return frame;
    }
    match game.state {
        GameState::Title => render_title(width, height),
        GameState::Playing => {
            let mut frame = render_world(game, width, height);
            frame.write_at(
                1,
                height.saturating_sub(1),
                "WASD MOVE  ←/→ TURN  ESC PAUSE  Q QUIT",
            );
            frame
        }
        GameState::Paused => {
            let mut frame = render_world(game, width, height);
            let middle = height / 2;
            frame.write_centered(middle.saturating_sub(1), "[ PAUSED ]");
            frame.write_centered(middle + 1, "ESC / ENTER  RESUME");
            frame.write_centered(middle + 2, "Q            QUIT");
            frame
        }
        GameState::Exiting => FrameBuffer::new(width, height, ' '),
    }
}

fn render_title(width: usize, height: usize) -> FrameBuffer {
    let mut frame = FrameBuffer::new(width, height, ' ');
    let logo = [
        " _   _ _   _ _     _       ____  _____ ____ _____ ___  ____  ",
        "| \\ | | | | | |   | |     / ___|| ____/ ___|_   _/ _ \\|  _ \\ ",
        "|  \\| | | | | |   | |     \\___ \\|  _|| |     | || | | | |_) |",
        "| |\\  | |_| | |___| |___   ___) | |__| |___  | || |_| |  _ < ",
        "|_| \\_|\\___/|_____|_____| |____/|_____\\____| |_| \\___/|_| \\_\\",
    ];
    let top = height.saturating_sub(logo.len() + 8) / 2;
    for (line, text) in logo.iter().enumerate() {
        frame.write_centered(top + line, text);
    }
    frame.write_centered(top + logo.len(), GAME_NAME);
    frame.write_centered(top + logo.len() + 2, "NO RECORD OF THIS FACILITY EXISTS.");
    frame.write_centered(top + logo.len() + 5, "ENTER  START");
    frame.write_centered(top + logo.len() + 6, "Q      QUIT");
    frame
}

#[derive(Debug, Clone)]
pub struct FrameBuffer {
    width: usize,
    height: usize,
    cells: Vec<char>,
}

pub fn render_world(game: &Game, width: usize, height: usize) -> FrameBuffer {
    let mut frame = FrameBuffer::new(width, height, ' ');
    if width == 0 || height == 0 {
        return frame;
    }

    render_floor_and_ceiling(&mut frame);

    let rays = cast_view(
        &game.map,
        game.player.position,
        game.player.angle,
        game.config.fov,
        width,
        game.config.render_distance,
    );

    for (x, ray) in rays.into_iter().enumerate() {
        let wall_height = (height as f32 / ray.perpendicular_distance) as usize;
        let top = height.saturating_sub(wall_height) / 2;
        let bottom = (top + wall_height).min(height);
        let shade = wall_shade(ray.perpendicular_distance, ray.side);
        for y in top..bottom {
            frame.set(x, y, shade);
        }
    }
    frame
}

fn render_floor_and_ceiling(frame: &mut FrameBuffer) {
    let horizon = frame.height / 2;
    for y in 0..frame.height {
        let glyph = if y < horizon {
            let depth = (horizon - y) as f32 / horizon.max(1) as f32;
            if depth > 0.78 { '.' } else { ' ' }
        } else {
            let depth = (y - horizon) as f32 / (frame.height - horizon).max(1) as f32;
            match depth {
                value if value > 0.76 => ':',
                value if value > 0.42 => '.',
                _ => ' ',
            }
        };
        for x in 0..frame.width {
            frame.set(x, y, glyph);
        }
    }
}

fn wall_shade(distance: f32, side: WallSide) -> char {
    let distance = distance
        + if side == WallSide::Horizontal {
            0.9
        } else {
            0.0
        };
    match distance {
        value if value < 1.6 => '█',
        value if value < 3.5 => '▓',
        value if value < 7.0 => '▒',
        value if value < 13.0 => '░',
        _ => '.',
    }
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize, fill: char) -> Self {
        Self {
            width,
            height,
            cells: vec![fill; width * height],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, value: char) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = value;
        }
    }

    pub fn write_centered(&mut self, y: usize, text: &str) {
        let length = text.chars().count();
        let start = self.width.saturating_sub(length) / 2;
        for (offset, character) in text.chars().take(self.width).enumerate() {
            self.set(start + offset, y, character);
        }
    }

    pub fn write_at(&mut self, x: usize, y: usize, text: &str) {
        for (offset, character) in text.chars().enumerate() {
            self.set(x + offset, y, character);
        }
    }

    pub fn to_terminal_string(&self) -> String {
        let mut output = String::with_capacity((self.width + 2) * self.height);
        for (row, cells) in self.cells.chunks(self.width).enumerate() {
            output.extend(cells);
            if row + 1 < self.height {
                output.push_str("\r\n");
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_single_buffered_terminal_frame() {
        let mut frame = FrameBuffer::new(5, 2, '.');
        frame.write_centered(0, "HUD");
        assert_eq!(frame.to_terminal_string(), ".HUD.\r\n.....");
    }

    #[test]
    fn projects_wall_columns_into_the_viewport() {
        let mut game = Game::new(1);
        game.player.position = crate::geom::Vec2::new(3.5, 2.5);
        game.player.angle = 0.0;
        let output = render_world(&game, 20, 10).to_terminal_string();
        assert!(
            output
                .chars()
                .any(|cell| matches!(cell, '█' | '▓' | '▒' | '░' | '.'))
        );
        assert_eq!(output.lines().count(), 10);
    }

    #[test]
    fn wall_shading_fades_with_distance() {
        assert_eq!(wall_shade(1.0, WallSide::Vertical), '█');
        assert_eq!(wall_shade(5.0, WallSide::Vertical), '▒');
        assert_eq!(wall_shade(20.0, WallSide::Vertical), '.');
    }

    #[test]
    fn floor_and_ceiling_have_depth_patterns() {
        let mut frame = FrameBuffer::new(8, 12, ' ');
        render_floor_and_ceiling(&mut frame);
        let output = frame.to_terminal_string();
        assert!(output.lines().next().unwrap().contains('.'));
        assert!(output.lines().last().unwrap().contains(':'));
    }

    #[test]
    fn title_uses_canonical_game_name_art() {
        let game = Game::new(1);
        let title = render_frame(&game, 80, 24).to_terminal_string();
        assert!(title.contains("NO RECORD OF THIS FACILITY EXISTS."));
        assert!(title.contains(GAME_NAME));
        assert!(title.contains("ENTER  START"));
    }

    #[test]
    fn small_terminals_receive_resize_instructions() {
        let game = Game::new(1);
        let output = render_frame(&game, 60, 18).to_terminal_string();
        assert!(output.contains("Terminal too small."));
        assert!(output.contains("80x24"));
    }
}
