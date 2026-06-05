# Dungeon Run

A 2D top-down arcade game written in **Rust**. You play a treasure hunter
navigating a procedurally generated dungeon — grab coins to rack up score while
dodging skeletons that hunt you down. Rounds are timed, with three difficulty
modes, limited lives, and a persistent high score.

> Built as a learning project to pick up Rust from scratch. The interesting
> part isn't the game — it's the architecture: all game logic lives in a
> graphics-free, headless-testable core crate, deliberately decoupled from
> rendering so the same tick loop can later be driven by a reinforcement-learning
> agent instead of a keyboard.

<!-- TODO: drop a gameplay screenshot or GIF here — it sells the project instantly.
     e.g. ![Dungeon Run gameplay](docs/gameplay.gif) -->

## Highlights

- **Clean logic/rendering split.** A Cargo workspace with two crates: a pure
  `dungeon_core` (no graphics dependency, fully runnable headless) and a thin
  `dungeon_run` binary that only handles input, rendering, and audio.
- **Guaranteed-solvable levels.** Every generated dungeon is validated with
  breadth-first search so the player can always reach the coins — no soft-locks.
- **Pathfinding enemy AI.** Skeletons chase the player via BFS when they're in
  vision range, and wander randomly otherwise.
- **Data-driven difficulty.** Difficulty is a config struct (lives, enemy count,
  enemy speed, vision range), not branching code — easy to tune and extend.
- **Deterministic by design.** All randomness flows through a single seedable
  RNG, so any playthrough can be reproduced for debugging.

## Tech stack

| Concern | Choice |
| --- | --- |
| Language | Rust (2024 edition) |
| Rendering / input / windowing | [`macroquad`](https://macroquad.rs) |
| RNG | [`rand`](https://crates.io/crates/rand) (seedable) |
| Project layout | Cargo workspace, two crates |

Dependencies are kept deliberately minimal and are all free / open source.

## Architecture

The whole project is organized around one rule: **game rules never touch the
screen, and the screen never decides game rules.**

```
Dungeon_Run/
├── dungeon_core/      # all game logic — NO graphics dependency
│   └── src/lib.rs     # state, tick(), entity behaviour, pathfinding, scoring
├── dungeon_run/       # the playable binary
│   └── src/main.rs    # input → action → tick → render loop (no game rules)
└── assets/            # 16×16 pixel-art spritesheets (drawn via source rects)
```

The core exposes a single `tick(state, action, delta, rng) -> state` function:

```
read input ──▶ translate to Action ──▶ tick() ──▶ render
                                          ▲
                       (an RL agent will call this same tick
                        with no rendering attached)
```

This separation is the reason the project is interesting from an engineering
standpoint rather than just a game standpoint. Because `dungeon_core` has no
dependency on Macroquad, the entire game can run thousands of ticks per second
with no window — which is exactly what a future training loop needs.

### Notable design decisions

- **All-pairs reachability via BFS.** On level generation, the core runs BFS
  from every floor tile and caches a path map. This is used both to validate
  that levels are solvable and to drive skeleton chase behaviour, trading a
  little upfront compute for guaranteed-fair levels and cheap per-tick pathing.
- **Tile-based, integer coordinates.** Positions are integer `(x, y)` tile
  coordinates — never floats — which keeps collision and pathfinding exact.
- **Spritesheets, not loose files.** Each sheet is loaded once as a single
  texture; individual sprites are drawn via source rectangles.

## Gameplay

- Collect coins → **+10** each.
- Clear every coin on the board → **+50** bonus, and a fresh set spawns.
- Touch a skeleton → lose a life and respawn somewhere safe.
- The round ends when lives hit **0** or the timer runs out.
- Beat your best — the high score is saved between sessions.

### Difficulty

| Difficulty | Lives | Skeletons | Skeleton speed | Vision range |
| --- | --- | --- | --- | --- |
| Easy | 5 | 4 | Slow (1.0×) | 3 |
| Medium | 3 | 6 | Medium (1.25×) | 4 |
| Hard | 2 | 8 | Fast (1.5×) | 5 |

All modes use a 120-second timer on a 20×12 grid. (Values are tuned for balance
and live in `dungeon_core` as plain data — see `EASY` / `MEDIUM` / `HARD`.)

## Controls

| Action | Keys |
| --- | --- |
| Move | `W` `A` `S` `D` or arrow keys |
| Navigate menus | Up / Down |
| Select | `Enter` |
| Pause | `Esc` |

## Build & run

Requires a [Rust toolchain](https://rustup.rs) (stable).

```sh
# Play the game
cargo run -p dungeon_run --release

# Run the headless logic tests
cargo test -p dungeon_core

# Format and lint
cargo fmt
cargo clippy
```

## Roadmap

- [ ] Unit tests covering collision, scoring, and win/lose conditions in
      `dungeon_core` (the crate is headless-testable by design — this is the
      next priority).
- [ ] Expose the seed via a CLI flag for fully reproducible runs.
- [ ] A reinforcement-learning agent that drives `tick()` directly, training
      headlessly against the same game logic a human plays.
- [ ] Sound effects (out of MVP scope; only if core gameplay is locked).

## License

Solo coursework / learning project. Sprites are from a third-party pixel-art
pack; see the asset pack's own license for reuse terms.
