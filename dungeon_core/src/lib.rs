use std::{os::macos::raw::stat, time::Duration};
use rand::{seq::IndexedRandom, RngExt};

pub const GRID_WIDTH: u8 = 14;
pub const GRID_HEIGHT: u8 = 8;
pub const MIN_DISTANCE: u8 = 3;

const VISION: u8 = 1;

#[derive(Clone, Copy)]
pub struct DifficultyParameters {
    pub lives: u8,
    pub timer: u8,
    pub skeleton_count: u8,
    pub skeleton_speed: f32,
}

pub struct Game {
    pub difficulty: DifficultyParameters,
    pub player_position: (u8, u8),
    pub skeleton_positions: Vec<(u8, u8)>,
    pub skeleton_move_accumulator: Duration,
    pub coin_positions: Vec<(u8, u8)>,
    pub lives_left: u8,
    pub time_left: Duration,
    pub grid: Vec<Vec<TileType>>,
    pub score: u16,
}

pub const EASY: DifficultyParameters = DifficultyParameters {
    lives: 5,
    timer: 90,
    skeleton_count: 2,
    skeleton_speed: 1.0,
};

pub const MEDIUM: DifficultyParameters = DifficultyParameters {
    lives: 3,
    timer: 60,
    skeleton_count: 4,
    skeleton_speed: 1.5,
};

pub const HARD: DifficultyParameters = DifficultyParameters {
    lives: 2,
    timer: 45,
    skeleton_count: 6,
    skeleton_speed: 2.0,
};

#[derive(PartialEq)]
pub enum TileType {
    Floor,
    Wall,
    Obstacle,
}

#[derive(PartialEq, Clone)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    None,
}

pub fn generate_grid(difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Vec<Vec<TileType>> {
    let mut grid = Vec::new();
    for y in 0..GRID_HEIGHT {
        let mut row = Vec::new();
        for x in 0..GRID_WIDTH {
            if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                row.push(TileType::Wall);
            } else if rng.random_bool(0.15) {
                row.push(TileType::Obstacle);
            } else {
                row.push(TileType::Floor);
            }
        }
        grid.push(row);
    }
    grid
}

pub fn get_floor_tiles(grid: &Vec<Vec<TileType>>, exclude: &[(u8, u8)]) -> Vec<(u8, u8)> {
    let mut tiles = Vec::new();
    for y in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            if matches!(grid[y as usize][x as usize], TileType::Floor)
                && !exclude.contains(&(x, y))
            {
                tiles.push((x, y));
            }
        }
    }
    tiles
}

pub fn generate_skeletons(grid: &Vec<Vec<TileType>>, difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Vec<(u8, u8)> {
    let floor_tiles = get_floor_tiles(grid, &[]);
    floor_tiles.sample(rng, difficulty.skeleton_count as usize).cloned().collect()
}

pub fn generate_coins(grid: &Vec<Vec<TileType>>, skeletons: &Vec<(u8, u8)>, rng: &mut impl rand::Rng) -> Vec<(u8, u8)> {
    let floor_tiles = get_floor_tiles(grid, skeletons);
    floor_tiles.sample(rng, 10).cloned().collect()
}

pub fn spawn_player(grid: &Vec<Vec<TileType>>, skeletons: &Vec<(u8, u8)>, coins: &Vec<(u8, u8)>, rng: &mut impl rand::Rng) -> (u8, u8) {
    let exclude = [skeletons.as_slice(), coins.as_slice()].concat();
    let floor_tiles: Vec<(u8, u8)> = get_floor_tiles(grid, &exclude)
        .into_iter()
        .filter(|&(x, y)| skeletons.iter().all(|&(sx, sy)| sx.abs_diff(x) + sy.abs_diff(y) >= MIN_DISTANCE))
        .collect();
    floor_tiles.choose(rng).cloned().unwrap_or_else(|| {
        get_floor_tiles(grid, &exclude).into_iter().next().expect("No valid player spawn points")
    })
}

pub fn move_skeletons(state: &mut Game, player_pos: (u8, u8), rng: &mut impl rand::Rng) {
    for skelly in &mut state.skeleton_positions {
        let distance = skelly.0.abs_diff(player_pos.0) + skelly.1.abs_diff(player_pos.1);
        if distance <= VISION {
            let dx = player_pos.0 as i8 - skelly.0 as i8;
            let dy = player_pos.1 as i8 - skelly.1 as i8;
            if dx.abs() >= dy.abs() {
                let new_x = if dx > 0 { skelly.0 + 1 } else if dx < 0 { skelly.0 - 1 } else { skelly.0 };
                if matches!(state.grid[skelly.1 as usize][new_x as usize], TileType::Floor) {
                    skelly.0 = new_x;
                }
            } else {
                let new_y = if dy > 0 { skelly.1 + 1 } else if dy < 0 { skelly.1 - 1 } else { skelly.1 };
                if matches!(state.grid[new_y as usize][skelly.0 as usize], TileType::Floor) {
                    skelly.1 = new_y;
                }
            }
        } else {
            let dirs: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            let dir = dirs.choose(rng).unwrap();
            let new_x = (skelly.0 as i16 + dir.0 as i16) as u8;
            let new_y = (skelly.1 as i16 + dir.1 as i16) as u8;
            if matches!(state.grid[new_y as usize][new_x as usize], TileType::Floor) {
                skelly.0 = new_x;
                skelly.1 = new_y;
            }
        }
    }
}

pub fn tick(mut state: Game, input: Input, delta: Duration, rng: &mut impl rand::Rng) -> Option<Game> {
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
    if !matches!(state.grid[player_pos.1 as usize][player_pos.0 as usize], TileType::Wall | TileType::Obstacle) {
        state.player_position = player_pos;
    }
    if state.skeleton_positions.contains(&state.player_position) {
        state.lives_left = state.lives_left.saturating_sub(1);
        if state.lives_left == 0 {
            return None;
        }
        state.player_position = spawn_player(&state.grid, &state.skeleton_positions, &state.coin_positions, rng);
    }
    if state.coin_positions.contains(&state.player_position) {
        state.score += 10;
        state.coin_positions.retain(|&pos| pos != state.player_position);
    }
    if state.coin_positions.is_empty() {
        state.score += 50;
        state.coin_positions = generate_coins(&state.grid, &state.skeleton_positions, rng);
    }
    Some(state)
}

pub fn new_game(difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Game {
    let grid = generate_grid(difficulty, rng);
    let skeletons = generate_skeletons(&grid, difficulty, rng);
    let coins = generate_coins(&grid, &skeletons, rng);
    let player_position = spawn_player(&grid, &skeletons, &coins, rng);
    Game {
        difficulty,
        player_position,
        skeleton_positions: skeletons,
        skeleton_move_accumulator: Duration::ZERO,
        coin_positions: coins,
        lives_left: difficulty.lives,
        time_left: Duration::from_secs(difficulty.timer as u64),
        grid,
        score: 0,
    }
}

#[test]
fn test() {
    let mut rng = rand::rng();
    let grid = generate_grid(EASY, &mut rng);
    let skeletons = generate_skeletons(&grid, MEDIUM, &mut rng);
    let coins = generate_coins(&grid, &skeletons, &mut rng);
    let player = spawn_player(&grid, &skeletons, &coins, &mut rng);

    for (y, row) in grid.iter().enumerate() {
        let line: String = row.iter().enumerate().map(|(x, t)| {
            let pos = (x as u8, y as u8);
            if pos == player { 'P' }
            else if skeletons.contains(&pos) { 'S' }
            else if coins.contains(&pos) { 'C' }
            else {
                match t {
                    TileType::Wall => 'W',
                    TileType::Floor => '.',
                    TileType::Obstacle => 'O',
                }
            }
        }).collect();
        println!("{}", line);
    }
    println!("Player: {:?}", player);
    println!("Skeletons: {:?}", skeletons);
    println!("Coins: {:?}", coins);
}
