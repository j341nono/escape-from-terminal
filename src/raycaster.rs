use crate::{
    geom::{Cell, Vec2},
    map::Map,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallSide {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub distance: f32,
    pub side: WallSide,
}

#[derive(Debug, Clone, Copy)]
pub struct ViewRay {
    pub perpendicular_distance: f32,
    pub side: WallSide,
}

pub fn cast_view(
    map: &Map,
    origin: Vec2,
    angle: f32,
    fov: f32,
    columns: usize,
    max_distance: f32,
) -> Vec<ViewRay> {
    (0..columns)
        .map(|column| {
            let camera = (column as f32 + 0.5) / columns as f32;
            let angle_offset = (camera - 0.5) * fov;
            let hit = cast_ray(
                map,
                origin,
                Vec2::from_angle(angle + angle_offset),
                max_distance,
            );
            ViewRay {
                perpendicular_distance: (hit.distance * angle_offset.cos()).max(0.0001),
                side: hit.side,
            }
        })
        .collect()
}

pub fn cast_ray(map: &Map, origin: Vec2, direction: Vec2, max_distance: f32) -> RayHit {
    let direction = direction.normalized();
    let mut map_x = origin.x.floor() as isize;
    let mut map_y = origin.y.floor() as isize;

    let delta_x = if direction.x.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        (1.0 / direction.x).abs()
    };
    let delta_y = if direction.y.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        (1.0 / direction.y).abs()
    };

    let (step_x, mut side_x) = if direction.x < 0.0 {
        (-1, (origin.x - map_x as f32) * delta_x)
    } else {
        (1, (map_x as f32 + 1.0 - origin.x) * delta_x)
    };
    let (step_y, mut side_y) = if direction.y < 0.0 {
        (-1, (origin.y - map_y as f32) * delta_y)
    } else {
        (1, (map_y as f32 + 1.0 - origin.y) * delta_y)
    };

    loop {
        let (distance, side) = if side_x < side_y {
            map_x += step_x;
            let distance = side_x;
            side_x += delta_x;
            (distance, WallSide::Vertical)
        } else {
            map_y += step_y;
            let distance = side_y;
            side_y += delta_y;
            (distance, WallSide::Horizontal)
        };

        if distance >= max_distance || map_x < 0 || map_y < 0 {
            return RayHit {
                distance: max_distance,
                side,
            };
        }

        if map.tile(Cell::new(map_x as usize, map_y as usize)).blocks_sight() {
            return RayHit { distance, side };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_map() -> Map {
        Map::from_ascii(&["#####", "#...#", "#...#", "#...#", "#####"]).unwrap()
    }

    #[test]
    fn dda_hits_vertical_wall_at_expected_distance() {
        let hit = cast_ray(
            &box_map(),
            Vec2::new(2.5, 2.5),
            Vec2::new(1.0, 0.0),
            20.0,
        );
        assert!((hit.distance - 1.5).abs() < 0.0001);
        assert_eq!(hit.side, WallSide::Vertical);
    }

    #[test]
    fn dda_hits_horizontal_wall_at_expected_distance() {
        let hit = cast_ray(
            &box_map(),
            Vec2::new(2.5, 2.5),
            Vec2::new(0.0, -1.0),
            20.0,
        );
        assert!((hit.distance - 1.5).abs() < 0.0001);
        assert_eq!(hit.side, WallSide::Horizontal);
    }


    #[test]
    fn view_distances_are_fisheye_corrected() {
        let rays = cast_view(
            &box_map(),
            Vec2::new(2.5, 2.5),
            0.0,
            std::f32::consts::FRAC_PI_3,
            9,
            20.0,
        );
        for ray in rays {
            assert!((ray.perpendicular_distance - 1.5).abs() < 0.001);
        }
    }
}
