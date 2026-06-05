//! Core game logic for Dungeon Run: world state, the per-tick update, enemy
//! pathfinding and level generation.
//!
//! There is deliberately no rendering or windowing code in here, so the game can
//! run headless which makes it testable and, down the line, drivable by
//! a learning agent instead of a keyboard.

use rand::{RngExt, seq::IndexedRandom};
use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

/// Dungeon size in tiles. The outermost ring is always wall.
pub const GRID_WIDTH: u8 = 20;
pub const GRID_HEIGHT: u8 = 12;
/// Minimum Manhattan distance the player must spawn from every skeleton.
pub const MIN_DISTANCE: u8 = 3;
/// Round length, in seconds.
pub const TIMER: u64 = 120;

const MIN_REGION_SIZE: usize = 2;
const OBSTACLE_CHANCE: f64 = 0.10;

// Scoring and coin layout.
const COIN_VALUE: usize = 10;
const CLEAR_BOARD_BONUS: usize = 50;
const COINS_PER_BOARD: usize = 10;

/// Next-step lookup table: `path_map[from][to]` is the first tile to step onto
/// when walking the shortest path from `from` to `to`. Built once per level so
/// skeleton chasing is just a hashmap lookup instead of a search every tick.
pub type PathMap = HashMap<(u8, u8), HashMap<(u8, u8), (u8, u8)>>;

/// Everything that makes one difficulty harder than another. Passed in at
/// new-game time — difficulty is data, not branching logic.
#[derive(Clone, Copy)]
pub struct DifficultyParameters {
    pub lives: u8,
    pub skeleton_count: u8,
    /// Skeleton moves per second; higher is faster.
    pub skeleton_speed: f32,
    /// How close (Manhattan) the player must be before a skeleton starts chasing.
    pub vision: u8,
}

/// The full world state for one round. `tick` takes this and hands back the next.
pub struct Game {
    pub difficulty: DifficultyParameters,
    pub player_position: (u8, u8),
    pub skeleton_positions: Vec<(u8, u8)>,
    /// Leftover time carried between frames so skeletons step at a fixed rate no
    /// matter the frame rate.
    pub skeleton_move_accumulator: Duration,
    pub coin_positions: Vec<(u8, u8)>,
    pub lives_left: u8,
    pub time_left: Duration,
    pub grid: Vec<Vec<TileType>>,
    pub score: usize,
    pub path_map: PathMap,
}

// Starting difficulty presets. Tuned by playtesting — see the README table.
pub const EASY: DifficultyParameters = DifficultyParameters {
    lives: 5,
    skeleton_count: 4,
    skeleton_speed: 1.0,
    vision: 3,
};

pub const MEDIUM: DifficultyParameters = DifficultyParameters {
    lives: 3,
    skeleton_count: 6,
    skeleton_speed: 1.25,
    vision: 4,
};

pub const HARD: DifficultyParameters = DifficultyParameters {
    lives: 2,
    skeleton_count: 8,
    skeleton_speed: 1.5,
    vision: 5,
};

/// What sits in a grid cell. Walls and obstacles block movement; only floor is walkable.
#[derive(PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Obstacle,
}

/// One player action for a single tick. `None` means stay put.
#[derive(PartialEq, Clone)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    None,
}

/// Build a fresh dungeon: a solid wall border, floor inside, with some interior
/// tiles randomly turned into obstacles.
pub fn generate_grid(rng: &mut impl rand::Rng) -> Vec<Vec<TileType>> {
    let mut grid = Vec::new();
    for y in 0..GRID_HEIGHT {
        let mut row = Vec::new();
        for x in 0..GRID_WIDTH {
            if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                row.push(TileType::Wall);
            } else if rng.random_bool(OBSTACLE_CHANCE) {
                row.push(TileType::Obstacle);
            } else {
                row.push(TileType::Floor);
            }
        }
        grid.push(row);
    }
    grid
}

/// Every walkable interior tile, minus anything in `exclude` — handy for keeping
/// spawns from landing on top of each other.
pub fn get_floor_tiles(grid: &[Vec<TileType>], exclude: &[(u8, u8)]) -> Vec<(u8, u8)> {
    let mut tiles = Vec::new();
    for y in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            if matches!(grid[y as usize][x as usize], TileType::Floor) && !exclude.contains(&(x, y))
            {
                tiles.push((x, y));
            }
        }
    }
    tiles
}

/// Drop the difficulty's number of skeletons onto random floor tiles.
pub fn generate_skeletons(
    grid: &[Vec<TileType>],
    difficulty: DifficultyParameters,
    rng: &mut impl rand::Rng,
) -> Vec<(u8, u8)> {
    let floor_tiles = get_floor_tiles(grid, &[]);
    floor_tiles
        .sample(rng, difficulty.skeleton_count as usize)
        .cloned()
        .collect()
}

/// Scatter coins on floor tiles the player can actually reach. We filter through
/// the path map so a coin never spawns walled off in an unreachable pocket.
pub fn generate_coins(
    grid: &[Vec<TileType>],
    skeletons: &[(u8, u8)],
    player_pos: &(u8, u8),
    rng: &mut impl rand::Rng,
    path_map: &PathMap,
) -> Vec<(u8, u8)> {
    let exclude = [skeletons, [*player_pos].as_slice()].concat();
    let floor_tiles = get_floor_tiles(grid, &exclude)
        .into_iter()
        .filter(|tile| path_map[player_pos].contains_key(tile))
        .collect::<Vec<_>>();
    floor_tiles
        .sample(rng, COINS_PER_BOARD.min(floor_tiles.len()))
        .cloned()
        .collect()
}

/// Pick a start tile at least `MIN_DISTANCE` away from every skeleton. If the
/// map is too cramped to manage that, fall back to any floor tile.
pub fn spawn_player(
    grid: &[Vec<TileType>],
    skeletons: &[(u8, u8)],
    rng: &mut impl rand::Rng,
) -> (u8, u8) {
    let floor_tiles: Vec<(u8, u8)> = get_floor_tiles(grid, skeletons)
        .into_iter()
        .filter(|&(x, y)| {
            skeletons
                .iter()
                .all(|&(sx, sy)| sx.abs_diff(x) + sy.abs_diff(y) >= MIN_DISTANCE)
        })
        .collect();
    floor_tiles.choose(rng).cloned().unwrap_or_else(|| {
        get_floor_tiles(grid, skeletons)
            .into_iter()
            .next()
            .expect("No valid player spawn points")
    })
}

/// Breadth-first search across floor tiles from `source`, returning a
/// came-from map (each reachable tile points at the tile we arrived from).
fn bfs(grid: &[Vec<TileType>], source: (u8, u8)) -> HashMap<(u8, u8), (u8, u8)> {
    let mut queue = VecDeque::new();
    let mut parents: HashMap<(u8, u8), (u8, u8)> = HashMap::new();
    parents.insert(source, source);
    queue.push_back(source);
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let new_x = (x as i16 + dx) as u8;
            let new_y = (y as i16 + dy) as u8;
            if new_x >= GRID_WIDTH || new_y >= GRID_HEIGHT {
                continue;
            }
            if matches!(grid[new_y as usize][new_x as usize], TileType::Floor)
                && !parents.contains_key(&(new_x, new_y))
            {
                parents.insert((new_x, new_y), (x, y));
                queue.push_back((new_x, new_y));
            }
        }
    }
    parents
}

/// Follow the came-from pointers back from `target` to find the very first step
/// to take when leaving `source`. `None` if `target` isn't reachable.
fn first_step_towards(
    parents: &HashMap<(u8, u8), (u8, u8)>,
    source: (u8, u8),
    target: (u8, u8),
) -> Option<(u8, u8)> {
    if !parents.contains_key(&target) {
        return None;
    }
    let mut current = target;
    while parents[&current] != source {
        current = parents[&current];
    }
    Some(current)
}

/// Run one BFS from every floor tile up front to build the whole [`PathMap`].
/// It's more work at level start, but it turns per-tick chasing into a cheap
/// lookup and lets us confirm the level is solvable before play begins.
fn build_map(grid: &[Vec<TileType>]) -> PathMap {
    let mut path_map = HashMap::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if matches!(grid[y as usize][x as usize], TileType::Floor) {
                let source = (x, y);
                let parents = bfs(grid, source);
                let mut target_map = HashMap::new();
                for target in parents.keys() {
                    if *target != source
                        && let Some(step) = first_step_towards(&parents, source, *target)
                    {
                        target_map.insert(*target, step);
                    }
                }
                path_map.insert(source, target_map);
            }
        }
    }
    path_map
}

/// Move every skeleton one tile. If the player is within vision, chase along the
/// shortest path; otherwise take a random step. A skeleton won't move onto a
/// tile another skeleton already holds.
pub fn move_skeletons(state: &mut Game, player_pos: (u8, u8), rng: &mut impl rand::Rng) {
    let mut skeletons = state.skeleton_positions.clone();
    for skelly in &mut state.skeleton_positions {
        let distance = skelly.0.abs_diff(player_pos.0) + skelly.1.abs_diff(player_pos.1);
        skeletons.retain(|&pos| pos != *skelly);
        if distance <= state.difficulty.vision {
            if let Some(step) = state.path_map[skelly].get(&player_pos)
                && !skeletons.contains(step)
            {
                skelly.0 = step.0;
                skelly.1 = step.1;
            }
        } else {
            let dirs: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            let dir = dirs.choose(rng).unwrap();
            let new_x = (skelly.0 as i16 + dir.0 as i16) as u8;
            let new_y = (skelly.1 as i16 + dir.1 as i16) as u8;
            if matches!(state.grid[new_y as usize][new_x as usize], TileType::Floor)
                && !skeletons.contains(&(new_x, new_y))
            {
                skelly.0 = new_x;
                skelly.1 = new_y;
            }
        }
        skeletons.push(*skelly);
    }
}

/// Advance the world by `delta`: count down the timer, move skeletons on their
/// own cadence, apply the player's action, then resolve collisions, coin
/// pickups, and a board refresh once every coin is gone. Returns `None` when the
/// round is over (timer hit zero or the last life was lost), otherwise the
/// updated state.
pub fn tick(
    mut state: Game,
    input: Input,
    delta: Duration,
    rng: &mut impl rand::Rng,
) -> Option<Game> {
    state.time_left = state.time_left.saturating_sub(delta);
    if state.time_left.is_zero() {
        return None;
    }

    let mut player_pos = state.player_position;

    state.skeleton_move_accumulator += delta;
    let move_interval = Duration::from_secs_f32(1.0 / state.difficulty.skeleton_speed);
    if state.skeleton_move_accumulator >= move_interval {
        state.skeleton_move_accumulator = Duration::ZERO;
        move_skeletons(&mut state, player_pos, rng);
    }

    match input {
        Input::Up => player_pos.1 = player_pos.1.saturating_sub(1),
        Input::Down => player_pos.1 = player_pos.1.saturating_add(1),
        Input::Left => player_pos.0 = player_pos.0.saturating_sub(1),
        Input::Right => player_pos.0 = player_pos.0.saturating_add(1),
        Input::None => (),
    }
    if !matches!(
        state.grid[player_pos.1 as usize][player_pos.0 as usize],
        TileType::Wall | TileType::Obstacle
    ) {
        state.player_position = player_pos;
    }
    if state.skeleton_positions.contains(&state.player_position) {
        state.lives_left = state.lives_left.saturating_sub(1);
        if state.lives_left == 0 {
            return None;
        }
        state.player_position = loop {
            let new_pos = spawn_player(&state.grid, &state.skeleton_positions, rng);
            if state.path_map[&new_pos].contains_key(&state.coin_positions[0]) {
                break new_pos;
            }
        }
    }
    if state.coin_positions.contains(&state.player_position) {
        state.score += COIN_VALUE;
        state
            .coin_positions
            .retain(|&pos| pos != state.player_position);
    }
    if state.coin_positions.is_empty() {
        state.score += CLEAR_BOARD_BONUS;
        state.coin_positions = generate_coins(
            &state.grid,
            &state.skeleton_positions,
            &state.player_position,
            rng,
            &state.path_map,
        );
    }
    Some(state)
}

/// Start a fresh round for the given difficulty: generate the grid, precompute
/// paths, place the skeletons, then the player (somewhere reachable and clear of
/// skeletons), then the coins.
pub fn new_game(difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Game {
    let grid = generate_grid(rng);
    let path_map = build_map(&grid);
    let skeletons = generate_skeletons(&grid, difficulty, rng);
    let player_position = loop {
        let candidate = spawn_player(&grid, &skeletons, rng);
        if path_map[&candidate].len() >= MIN_REGION_SIZE {
            break candidate;
        }
    };
    let coins = generate_coins(&grid, &skeletons, &player_position, rng, &path_map);
    Game {
        difficulty,
        player_position,
        skeleton_positions: skeletons,
        skeleton_move_accumulator: Duration::ZERO,
        coin_positions: coins,
        lives_left: difficulty.lives,
        time_left: Duration::from_secs(TIMER),
        grid,
        score: 0,
        path_map,
    }
}

// #[test]
// fn test() {
//     let mut rng = rand::rng();
//     let grid = generate_grid(&mut rng);
//     let skeletons = generate_skeletons(&grid, MEDIUM, &mut rng);
//     // let coins = generate_coins(&grid, &skeletons, &mut rng);
//     let player = spawn_player(&grid, &skeletons, &coins, &mut rng);

//     for (y, row) in grid.iter().enumerate() {
//         let line: String = row.iter().enumerate().map(|(x, t)| {
//             let pos = (x as u8, y as u8);
//             if pos == player { 'P' }
//             else if skeletons.contains(&pos) { 'S' }
//             else if coins.contains(&pos) { 'C' }
//             else {
//                 match t {
//                     TileType::Wall => 'W',
//                     TileType::Floor => '.',
//                     TileType::Obstacle => 'O',
//                 }
//             }
//         }).collect();
//         println!("{}", line);
//     }
//     println!("Player: {:?}", player);
//     println!("Skeletons: {:?}", skeletons);
//     println!("Coins: {:?}", coins);
// }
