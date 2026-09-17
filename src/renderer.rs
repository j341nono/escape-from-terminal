use crate::{
    config::{GAME_NAME, MAX_RENDER_HEIGHT, MAX_RENDER_WIDTH, MIN_HEIGHT, MIN_WIDTH},
    game::{Game, GameState},
    monster::MONSTER_NAME,
    raycaster::{WallSide, cast_view},
};
use std::f32::consts::PI;

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
    let width = width.min(MAX_RENDER_WIDTH);
    let height = height.min(MAX_RENDER_HEIGHT);
    match game.state {
        GameState::Title => render_title(width, height),
        GameState::Intro => render_intro(width, height),
        GameState::Playing => {
            let mut frame = render_world(game, width, height);
            render_hud(&mut frame, game);
            frame.write_at(
                1,
                height.saturating_sub(1),
                "WASD MOVE  SPACE RUN  ←/→ TURN  E USE  ESC PAUSE  Q QUIT",
            );
            frame
        }
        GameState::Paused => {
            let mut frame = render_world(game, width, height);
            let middle = height / 2;
            frame.write_centered(middle.saturating_sub(1), "[ PAUSED ]");
            frame.write_centered(middle + 1, "ESC / ENTER  RESUME");
            frame.write_centered(middle + 2, "Q            QUIT");
            frame.write_centered(middle + 4, &format!("SEED: {}", game.seed));
            frame
        }
        GameState::Caught => render_caught(width, height),
        GameState::Escaped => render_escaped(game, width, height),
        GameState::Exiting => FrameBuffer::new(width, height, ' '),
    }
}

fn render_hud(frame: &mut FrameBuffer, game: &Game) {
    let filled = ((game.stamina / game.config.stamina_seconds) * 14.0).round() as usize;
    let bar = format!("{}{}", "#".repeat(filled), "-".repeat(14 - filled));
    frame.write_at(1, 0, &format!("STAMINA [{bar}]"));
    frame.write_at(1, 1, &format!("OBJECTIVE: {}", game.objective_text()));
    if let Some(message) = game.status_message.or_else(|| game.interaction_hint()) {
        frame.write_centered(3, message);
    }
    if game.caught_flash > 0.0 {
        frame.write_centered(frame.height / 2, "IT SAW YOU");
    }
}

fn render_intro(width: usize, height: usize) -> FrameBuffer {
    let mut frame = FrameBuffer::new(width, height, ' ');
    let middle = height / 2;
    frame.write_centered(middle.saturating_sub(3), "02:17 AM");
    frame.write_centered(middle.saturating_sub(1), "SECTOR STATUS: NULL");
    frame.write_centered(middle + 1, "Emergency power is offline.");
    frame.write_centered(middle + 2, "Something is moving in the facility.");
    frame.write_centered(middle + 5, "ENTER  CONTINUE");
    frame
}

fn render_caught(width: usize, height: usize) -> FrameBuffer {
    let mut frame = FrameBuffer::new(width, height, ' ');
    let middle = height / 2;
    for (offset, line) in [
        "      /\\___/\\",
        "     (  o o  )",
        "     /   ^   \\",
        "    /|  ---  |\\",
    ]
    .iter()
    .enumerate()
    {
        frame.write_centered(middle.saturating_sub(6) + offset, line);
    }
    frame.write_centered(middle, "YOU WERE FOUND");
    frame.write_centered(middle + 1, &format!("SUBJECT: {MONSTER_NAME}"));
    frame.write_centered(middle + 3, "R  RETRY SAME FACILITY");
    frame.write_centered(middle + 4, "N  NEW FACILITY");
    frame.write_centered(middle + 5, "Q  QUIT");
    frame
}

fn render_escaped(game: &Game, width: usize, height: usize) -> FrameBuffer {
    let mut frame = FrameBuffer::new(width, height, ' ');
    let middle = height / 2;
    frame.write_centered(middle.saturating_sub(3), "///  SIGNAL RESTORED  ///");
    frame.write_centered(middle, "YOU ESCAPED");
    frame.write_centered(
        middle + 2,
        &format!(
            "TIME: {:02}:{:02}",
            (game.elapsed_seconds / 60.0) as u32,
            game.elapsed_seconds as u32 % 60
        ),
    );
    frame.write_centered(middle + 3, &format!("CHASES: {}", game.chase_count));
    frame.write_centered(
        middle + 6,
        "R  RETRY SAME FACILITY   N  NEW FACILITY   Q  QUIT",
    );
    frame
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

    for (x, ray) in rays.iter().enumerate() {
        let wall_height = (height as f32 / ray.perpendicular_distance) as usize;
        let top = height.saturating_sub(wall_height) / 2;
        let bottom = (top + wall_height).min(height);
        let shade = wall_shade(ray.perpendicular_distance, ray.side);
        for y in top..bottom {
            frame.set(x, y, shade);
        }
    }
    render_monster(&mut frame, game, &rays);
    let marker = if game.power_restored { '>' } else { '!' };
    let marker_cell = if game.power_restored {
        game.exit_cell
    } else {
        game.objective_cell
    };
    render_marker(&mut frame, game, &rays, marker_cell.center(), marker);
    apply_proximity_glitch(&mut frame, game);
    frame
}

fn render_marker(
    frame: &mut FrameBuffer,
    game: &Game,
    wall_depth: &[crate::raycaster::ViewRay],
    position: crate::geom::Vec2,
    glyph: char,
) {
    let relative = position - game.player.position;
    let raw_angle = relative.y.atan2(relative.x) - game.player.angle;
    let angle = (raw_angle + PI).rem_euclid(2.0 * PI) - PI;
    if angle.abs() > game.config.fov * 0.52 {
        return;
    }
    let depth = relative.length() * angle.cos();
    let x = ((angle / game.config.fov + 0.5) * frame.width as f32) as isize;
    if x < 0 || x >= frame.width as isize || depth >= wall_depth[x as usize].perpendicular_distance
    {
        return;
    }
    let height = ((frame.height as f32 / depth.max(1.0)) * 0.3).clamp(1.0, 6.0) as isize;
    for y in 0..height {
        frame.set(x as usize, frame.height / 2 - y as usize, glyph);
    }
}

fn apply_proximity_glitch(frame: &mut FrameBuffer, game: &Game) {
    let distance = game.player.position.distance(game.monster.position);
    if distance >= 8.0 {
        return;
    }
    let intensity = ((8.0 - distance) * 2.0) as usize;
    let phase = (game.elapsed_seconds * 19.0) as usize;
    for index in 0..intensity.min(frame.height / 2) {
        let y = (phase + index * 3) % frame.height;
        frame.set(0, y, if index % 2 == 0 { '%' } else { '?' });
        frame.set(frame.width.saturating_sub(1), (y + 5) % frame.height, '#');
    }
    if distance < 4.0 && phase.is_multiple_of(359) {
        frame.write_centered(2, "SIGNAL: NULL");
    }
}

fn render_monster(frame: &mut FrameBuffer, game: &Game, wall_depth: &[crate::raycaster::ViewRay]) {
    let relative = game.monster.position - game.player.position;
    let raw_angle = relative.y.atan2(relative.x) - game.player.angle;
    let angle = (raw_angle + PI).rem_euclid(2.0 * PI) - PI;
    if angle.abs() > game.config.fov * 0.57 {
        return;
    }
    let distance = relative.length();
    let depth = distance * angle.cos();
    let center_x = ((angle / game.config.fov + 0.5) * frame.width as f32) as isize;
    let size = ((frame.height as f32 / depth.max(0.8)) * 0.42).clamp(2.0, frame.height as f32 * 0.8)
        as isize;
    let center_y = frame.height as isize / 2;
    for y in -size..=size {
        for x in -size..=size {
            let screen_x = center_x + x;
            let screen_y = center_y + y;
            if screen_x < 0
                || screen_y < 0
                || screen_x >= frame.width as isize
                || screen_y >= frame.height as isize
            {
                continue;
            }
            let column = screen_x as usize;
            if depth >= wall_depth[column].perpendicular_distance {
                continue;
            }
            let character = if y.abs() < size / 3 && x.abs() < size / 3 {
                '@'
            } else if y > 0 && x.abs() < size / 5 {
                '|'
            } else if y > size / 3 && x.abs() > size / 3 {
                '/'
            } else {
                continue;
            };
            frame.set(column, screen_y as usize, character);
        }
    }
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
        if self.width == 0 || self.height == 0 {
            return String::new();
        }
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

    #[test]
    fn monster_sprite_is_drawn_in_clear_view() {
        let mut game = Game::new(3);
        game.map =
            crate::map::Map::from_ascii(&["#######", "#.....#", "#.....#", "#######"]).unwrap();
        game.player.position = crate::geom::Vec2::new(2.5, 2.5);
        game.player.angle = 0.0;
        game.monster.position = crate::geom::Vec2::new(4.5, 2.5);
        assert!(
            render_world(&game, 80, 24)
                .to_terminal_string()
                .contains('@')
        );
    }

    #[test]
    fn wall_fully_occludes_monster_sprite() {
        let mut game = Game::new(3);
        game.map =
            crate::map::Map::from_ascii(&["#######", "#..#..#", "#..#..#", "#######"]).unwrap();
        game.player.position = crate::geom::Vec2::new(2.5, 1.5);
        game.player.angle = 0.0;
        game.monster.position = crate::geom::Vec2::new(4.5, 1.5);
        assert!(
            !render_world(&game, 80, 24)
                .to_terminal_string()
                .contains('@')
        );
    }

    #[test]
    fn monster_behind_player_is_not_rendered() {
        let mut game = Game::new(3);
        game.map =
            crate::map::Map::from_ascii(&["#######", "#.....#", "#.....#", "#######"]).unwrap();
        game.player.position = crate::geom::Vec2::new(3.5, 1.5);
        game.player.angle = 0.0;
        game.monster.position = crate::geom::Vec2::new(1.5, 1.5);
        assert!(
            !render_world(&game, 80, 24)
                .to_terminal_string()
                .contains('@')
        );
    }

    #[test]
    fn very_close_monster_and_fov_edge_are_bounds_safe() {
        let mut game = Game::new(3);
        game.map =
            crate::map::Map::from_ascii(&["#######", "#.....#", "#.....#", "#######"]).unwrap();
        game.player.position = crate::geom::Vec2::new(3.5, 1.5);
        game.player.angle = 0.0;
        game.monster.position = crate::geom::Vec2::new(3.51, 1.5);
        assert_eq!(
            render_world(&game, 80, 24)
                .to_terminal_string()
                .lines()
                .count(),
            24
        );
        game.monster.position =
            game.player.position + crate::geom::Vec2::from_angle(game.config.fov * 0.55) * 3.0;
        assert_eq!(
            render_world(&game, 80, 24)
                .to_terminal_string()
                .lines()
                .count(),
            24
        );
    }

    #[test]
    fn extreme_terminal_dimensions_are_capped() {
        let game = Game::new(3);
        let output = render_frame(&game, 10_000, 10_000).to_terminal_string();
        assert_eq!(output.lines().count(), MAX_RENDER_HEIGHT);
        assert!(
            output
                .lines()
                .all(|line| line.chars().count() == MAX_RENDER_WIDTH)
        );
    }

    #[test]
    fn zero_sized_terminal_frame_is_safe() {
        let game = Game::new(3);
        assert!(render_frame(&game, 0, 0).to_terminal_string().is_empty());
        assert!(render_frame(&game, 0, 24).to_terminal_string().is_empty());
    }

    #[test]
    fn outcome_screens_show_required_actions_and_statistics() {
        let mut game = Game::new(3);
        game.state = GameState::Caught;
        let caught = render_frame(&game, 80, 24).to_terminal_string();
        assert!(caught.contains("YOU WERE FOUND"));
        assert!(caught.contains("R  RETRY SAME FACILITY"));
        assert!(caught.contains("N  NEW FACILITY"));

        game.state = GameState::Escaped;
        game.elapsed_seconds = 125.0;
        game.chase_count = 2;
        let escaped = render_frame(&game, 80, 24).to_terminal_string();
        assert!(escaped.contains("YOU ESCAPED"));
        assert!(escaped.contains("TIME: 02:05"));
        assert!(escaped.contains("CHASES: 2"));
    }
}
