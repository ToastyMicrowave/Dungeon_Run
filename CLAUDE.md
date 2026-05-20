# Dungeon Run

A 2D top-down arcade game built in Rust with Macroquad. The player controls a
treasure hunter navigating a tile-based dungeon, collecting coins for score
while avoiding patrolling skeletons. Rounds are timed, with three difficulty
modes and a limited number of lives.

This is a solo coursework project on a ~3-week timeline. A longer-term goal is
to bolt a reinforcement-learning agent onto the game — this is the reason the
architecture below separates game logic from rendering. **Keep that separation
intact even if it looks like over-engineering for a small game.**

## Architecture

Cargo workspace with two crates:

- `dungeon_core` — all game logic: game state, entity behaviour, collision
  detection, scoring, tick advancement. **No dependency on Macroquad or any
  graphics/windowing library.** Must be runnable and testable headlessly.
- `dungeon_run` — the playable binary. Depends on `dungeon_core` + Macroquad.
  Handles keyboard input, calls into `dungeon_core`, renders state to the
  screen. Contains no game rules.

The boundary is strict: if you find yourself wanting to import Macroquad into
`dungeon_core`, stop — that's the signal something is in the wrong crate.

## Core design

- **Tile-based.** Positions are integer `(x, y)` tile coordinates, never
  floats. The dungeon is a 2D grid of wall/floor tiles.
- **Tick-based loop, fixed rate, decoupled from frame rate.** `dungeon_core`
  exposes a step/tick function that takes a player action, advances the world
  by one tick, and returns the new state. `dungeon_run`'s loop reads input →
  translates it to an action → calls the tick → renders. The RL agent will
  later call the same tick function without rendering.
- **Action set:** `Up | Down | Left | Right | NoOp`.
- **Difficulty is data, not code.** A `DifficultyConfig` struct (lives, time
  limit, skeleton count, skeleton speed) is passed into the game on new-game.
  Do not hardcode difficulty values.
- **Determinism:** randomness (skeleton spawns, patrol directions) goes through
  a seedable RNG so a playthrough can be reproduced for debugging.

## Game rules

- Player collects a coin → score increases.
- Player touches a skeleton → loses a life.
- Round ends when lives reach 0 OR the timer reaches 0.
- Skeletons patrol; spawn positions and patrol paths are randomised per
  playthrough.
- The player starts away from skeletons.

## Difficulty starting values

| Difficulty | Lives | Time | Skeletons | Skeleton speed |
| --- | --- | --- | --- | --- |
| Easy | 5 | 90s | 2 | Slow |
| Normal | 3 | 60s | 4 | Medium |
| Hard | 2 | 45s | 6 | Fast |

These are starting values for balancing, not final numbers — expect to tune
them through playtesting.

## Controls

- Move: WASD or arrow keys
- Pause: ESC

## Assets

Sprites come from a third-party pixel-art pack (16×16 tiles). They are
spritesheets — load each sheet as a single texture and draw individual sprites
via source rectangles rather than splitting them into separate files. Asset
files live with `dungeon_run` (the rendering crate); `dungeon_core` never
touches them.

## Dependencies

- `macroquad` — rendering, input, audio, windowing (`dungeon_run` only)
- `rand` — seedable RNG (`dungeon_core`)

Keep dependencies minimal. Both must be free and open source.

## Build & run

- Run the game: `cargo run -p dungeon_run`
- Test the logic: `cargo test -p dungeon_core`
- Format / lint: `cargo fmt` and `cargo clippy`

## Conventions

- Standard Rust style — run `cargo fmt` and `cargo clippy` before considering
  a change done.
- `dungeon_core` should have unit tests for the game logic (collision,
  scoring, win/lose conditions). It is headless-testable by design — use that.
- Prefer clear, readable code over clever code. This is a learning project;
  explain non-obvious decisions in comments.

## Constraints / notes

- Solo project, ~3-week timeline. Scope tightly; do not add features beyond
  what is described here without flagging it first.
- Kill switch: if there is no playable loop (controllable character + coin
  pickup + skeleton collision) by roughly the 2-week mark, the fallback plan
  is to rebuild in Python/Pygame to hit the deadline. Flag early if progress
  is slipping rather than pushing on silently.
- Audio is not in MVP scope. Only consider basic SFX if core gameplay is
  finished and time remains.
