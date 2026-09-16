# NULL SECTOR

An ASCII first-person survival horror game for the terminal.

No record of this underground research wing exists. Restore emergency power, find the emergency exit, and avoid **SPECIMEN-NULL**. There are no weapons and no way to kill it.

```text
.STAMINA [#########-----].......................................................
.OBJECTIVE: RESTORE EMERGENCY POWER.............................................
................................................................................
                         ░░░░░░░░░░░░░░░░░░░░░░
                    ▒▒▒▒▒▒▒▒▒          ▒▒▒▒▒▒▒▒▒
               ▓▓▓▓▓▓▓▓                  ▓▓▓▓▓▓▓▓
            ▓▓▓▓▓               !             ▓▓▓▓▓
         ███████                              ███████
..............................................................
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
```

## Requirements

- A recent stable Rust toolchain
- An ANSI-compatible terminal at least 80 columns by 24 rows
- macOS Terminal, iTerm2, and Ghostty are the primary targets

The implementation uses portable terminal APIs and should also work on many Linux terminals, but macOS is the currently supported platform.

## Build and run

```bash
git clone <repository-url> null-sector
cd null-sector
cargo run --release
```

To reproduce a facility, provide an unsigned 64-bit seed:

```bash
cargo run --release -- --seed 1337
```

The current seed is shown on the pause screen.

## Controls

| Key | Action |
| --- | --- |
| `W` / `Up` | Move forward |
| `S` / `Down` | Move backward |
| `A` | Strafe left |
| `D` | Strafe right |
| `Left` / `Right` | Turn |
| `Space` | Sprint while moving |
| `E` | Operate a door, power control, or exit |
| `Esc` | Pause or resume |
| `Enter` | Start, continue, or resume |
| `Q` | Quit |
| `R` | Retry the same facility after capture or escape |
| `N` | Generate a new facility after capture or escape |

## Objective

1. Explore the facility and locate the `!` emergency-power beacon.
2. Stand beside it and press `E` to restore power.
3. Locate the `>` emergency-exit beacon.
4. Stand beside it and press `E` to escape.

The HUD always shows the current objective. Trying the exit before restoring power displays a locked-exit message. Closed doors can be opened and closed with `E`; a closed door can buy time, but SPECIMEN-NULL will eventually open it.

Walking produces limited noise. Sprinting is faster and can create distance during a chase, but it is louder and consumes stamina. When stamina is exhausted, it must partially recover before sprinting becomes available again.

## Facility generation

Each seed deterministically creates a research facility from rooms and orthogonal corridors. Extra room connections create loops that can be used to break pursuit. Generation selects separate start, power, exit, and monster-spawn cells, rejects short critical routes, and validates reachability before play begins.

The same seed reconstructs the same original facility, including door and spawn placement. Retrying resets all mutable run state; choosing a new facility creates a new seed.

## Rendering

NULL SECTOR uses grid-based DDA ray casting, with one ray per terminal column. Perpendicular distance correction prevents fisheye distortion. Wall height and the `█▓▒░` character ramp provide depth, while separate floor and ceiling patterns establish the horizon.

SPECIMEN-NULL is projected as a distance-scaled ASCII sprite. Each sprite column is compared with wall-ray depth, so walls and closed doors occlude it. Subtle edge corruption appears only when the creature is nearby.

The complete frame is assembled in memory and written in one terminal update. Unchanged frames are not written again.

## Monster AI

SPECIMEN-NULL uses four states:

- **Wandering** — chooses reachable destinations around the facility.
- **Suspicious** — investigates nearby footsteps, sprinting, and interactions.
- **Chasing** — follows a visible player using periodically refreshed BFS paths.
- **Searching** — continues toward the last visible position before returning to wandering.

Detection uses a vision cone, range checks, and wall/door line of sight. Hearing uses traversable path distance, so nearby walls and long detours attenuate quiet movement. The monster cannot walk through walls or closed doors and must spend time opening a blocking door.

## Architecture

The game is intentionally engine-free and divided by responsibility:

- `game.rs` — state transitions, progression, stamina, and run statistics
- `generator.rs` — deterministic facility generation and placement
- `map.rs` — tiles, doors, and collision queries
- `player.rs` / `input.rs` — movement and terminal-key state
- `raycaster.rs` / `renderer.rs` — DDA visibility and ASCII projection
- `monster.rs` / `pathfinding.rs` — perception, state machine, and BFS navigation
- `terminal.rs` — raw mode, alternate screen, drawing, and cleanup
- `config.rs` — gameplay and rendering parameters

## Terminal notes

The game enters raw mode, hides the cursor, and uses the alternate screen. A drop guard restores raw mode, the cursor, and the original screen on normal exit, returned errors, and unwinding panics.

If the terminal becomes smaller than 80x24, gameplay freezes and a resize message is shown. It resumes safely after the terminal is enlarged. `Ctrl-C` is not a game command in raw mode; use `Q` to quit normally.

## Development checks

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```
