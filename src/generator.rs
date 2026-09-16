use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    geom::Cell,
    map::{Map, Tile},
    pathfinding::path_distance,
};

const WIDTH: usize = 57;
const HEIGHT: usize = 41;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facility {
    pub map: Map,
    pub start: Cell,
    pub objective: Cell,
    pub exit: Cell,
    pub monster_spawn: Cell,
    pub seed: u64,
}

#[derive(Debug, Clone, Copy)]
struct Room {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}
impl Room {
    fn center(self) -> Cell {
        Cell::new(self.x + self.w / 2, self.y + self.h / 2)
    }
    fn overlaps(self, other: Self) -> bool {
        self.x <= other.x + other.w + 1
            && self.x + self.w + 1 >= other.x
            && self.y <= other.y + other.h + 1
            && self.y + self.h + 1 >= other.y
    }
}

pub fn generate(seed: u64) -> Facility {
    for attempt in 0..64 {
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(attempt));
        if let Some(facility) = generate_attempt(seed, &mut rng) {
            return facility;
        }
    }
    unreachable!("facility generation should have a valid attempt")
}

fn generate_attempt(seed: u64, rng: &mut StdRng) -> Option<Facility> {
    let mut map = Map::filled(WIDTH, HEIGHT, Tile::Wall);
    let mut rooms = Vec::new();
    for _ in 0..48 {
        if rooms.len() >= 13 {
            break;
        }
        let room = Room {
            w: rng.random_range(5..=9),
            h: rng.random_range(5..=8),
            x: rng.random_range(1..WIDTH - 10),
            y: rng.random_range(1..HEIGHT - 9),
        };
        if room.x + room.w >= WIDTH - 1
            || room.y + room.h >= HEIGHT - 1
            || rooms.iter().any(|existing| room.overlaps(*existing))
        {
            continue;
        }
        carve_room(&mut map, room);
        rooms.push(room);
    }
    if rooms.len() < 9 {
        return None;
    }
    for index in 1..rooms.len() {
        carve_corridor(
            &mut map,
            rooms[index - 1].center(),
            rooms[index].center(),
            rng.random(),
        );
    }
    for _ in 0..4 {
        let a = rng.random_range(0..rooms.len());
        let mut b = rng.random_range(0..rooms.len());
        if a == b {
            b = (b + 1) % rooms.len();
        }
        carve_corridor(&mut map, rooms[a].center(), rooms[b].center(), rng.random());
    }
    let start = rooms[0].center();
    let objective = farthest(&map, start, &rooms)?;
    let exit = farthest(&map, objective, &rooms)?;
    let monster_spawn = choose_monster_spawn(&map, start, objective, exit, &rooms)?;
    if objective == exit {
        return None;
    }
    let first_leg = path_distance(&map, start, objective, false)?;
    let second_leg = path_distance(&map, objective, exit, false)?;
    if first_leg < 22 || second_leg < 22 {
        return None;
    }
    add_doors(&mut map, &[start, objective, exit, monster_spawn], rng);
    if path_distance(&map, start, objective, true).is_none()
        || path_distance(&map, objective, exit, true).is_none()
    {
        return None;
    }
    Some(Facility {
        map,
        start,
        objective,
        exit,
        monster_spawn,
        seed,
    })
}

fn choose_monster_spawn(
    map: &Map,
    start: Cell,
    objective: Cell,
    exit: Cell,
    rooms: &[Room],
) -> Option<Cell> {
    rooms
        .iter()
        .map(|room| room.center())
        .filter(|cell| ![start, objective, exit].contains(cell))
        .filter_map(|cell| {
            let from_start = path_distance(map, start, cell, false)?;
            let from_objective = path_distance(map, objective, cell, false)?;
            let from_exit = path_distance(map, exit, cell, false)?;
            (from_start >= 18 && from_objective >= 10 && from_exit >= 10)
                .then_some((from_start + 2 * from_objective.min(from_exit), cell))
        })
        .max_by_key(|pair| pair.0)
        .map(|pair| pair.1)
}

fn carve_room(map: &mut Map, room: Room) {
    for y in room.y..room.y + room.h {
        for x in room.x..room.x + room.w {
            map.set_tile(Cell::new(x, y), Tile::Floor);
        }
    }
}

fn carve_corridor(map: &mut Map, from: Cell, to: Cell, horizontal_first: bool) {
    let (corner_x, corner_y) = if horizontal_first {
        (to.x, from.y)
    } else {
        (from.x, to.y)
    };
    carve_line(map, from, Cell::new(corner_x, corner_y));
    carve_line(map, Cell::new(corner_x, corner_y), to);
}

fn carve_line(map: &mut Map, from: Cell, to: Cell) {
    let (min_x, max_x) = (from.x.min(to.x), from.x.max(to.x));
    let (min_y, max_y) = (from.y.min(to.y), from.y.max(to.y));
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            map.set_tile(Cell::new(x, y), Tile::Floor);
        }
    }
}

fn farthest(map: &Map, source: Cell, rooms: &[Room]) -> Option<Cell> {
    rooms
        .iter()
        .map(|room| room.center())
        .filter_map(|cell| path_distance(map, source, cell, false).map(|distance| (distance, cell)))
        .max_by_key(|pair| pair.0)
        .map(|pair| pair.1)
}

fn add_doors(map: &mut Map, protected: &[Cell], rng: &mut StdRng) {
    let mut candidates = Vec::new();
    for y in 2..map.height() - 2 {
        for x in 2..map.width() - 2 {
            let cell = Cell::new(x, y);
            if map.tile(cell) != Tile::Floor
                || protected
                    .iter()
                    .any(|point| point.x.abs_diff(x) + point.y.abs_diff(y) < 4)
            {
                continue;
            }
            let horizontal = map.is_walkable(Cell::new(x - 1, y))
                && map.is_walkable(Cell::new(x + 1, y))
                && !map.is_walkable(Cell::new(x, y - 1))
                && !map.is_walkable(Cell::new(x, y + 1));
            let vertical = map.is_walkable(Cell::new(x, y - 1))
                && map.is_walkable(Cell::new(x, y + 1))
                && !map.is_walkable(Cell::new(x - 1, y))
                && !map.is_walkable(Cell::new(x + 1, y));
            if horizontal || vertical {
                candidates.push(cell);
            }
        }
    }
    for _ in 0..3.min(candidates.len()) {
        let index = rng.random_range(0..candidates.len());
        map.set_tile(candidates.swap_remove(index), Tile::DoorClosed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn facility_seed_is_deterministic() {
        assert_eq!(generate(1234), generate(1234));
    }
    #[test]
    fn generated_facilities_satisfy_seed_corpus_invariants() {
        for seed in 0..256 {
            let facility = generate(seed);
            assert!(facility.map.is_walkable(facility.start), "seed {seed}");
            assert!(facility.map.is_walkable(facility.objective), "seed {seed}");
            assert!(facility.map.is_walkable(facility.exit), "seed {seed}");
            assert!(
                facility.map.is_walkable(facility.monster_spawn),
                "seed {seed}"
            );
            assert!(
                path_distance(&facility.map, facility.start, facility.objective, true)
                    .is_some_and(|distance| distance >= 22),
                "seed {seed}"
            );
            assert!(
                path_distance(&facility.map, facility.objective, facility.exit, true)
                    .is_some_and(|distance| distance >= 22),
                "seed {seed}"
            );
            assert!(
                path_distance(&facility.map, facility.start, facility.monster_spawn, true)
                    .is_some_and(|distance| distance >= 18),
                "seed {seed}"
            );
            assert!(
                ![facility.start, facility.objective, facility.exit]
                    .contains(&facility.monster_spawn),
                "seed {seed}"
            );
        }
    }
}
