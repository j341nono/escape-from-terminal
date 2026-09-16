#[derive(Debug, Clone)]
pub struct FrameBuffer {
    width: usize,
    height: usize,
    cells: Vec<char>,
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
}
