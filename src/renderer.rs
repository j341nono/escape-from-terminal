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
        for y in top..bottom {
            frame.set(x, y, '█');
        }
    }
    frame
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize, fill: char) -> Self {
        Self {
            width,
            height,
            cells: vec![fill; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
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
        let mut game = Game::new().expect("game should initialize");
        game.player.position = crate::geom::Vec2::new(3.5, 2.5);
        game.player.angle = 0.0;
        let output = render_world(&game, 20, 10).to_terminal_string();
        assert!(output.contains('█'));
        assert_eq!(output.lines().count(), 10);
    }
}
use crate::{game::Game, raycaster::cast_view};
