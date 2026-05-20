use std::time::{Duration, Instant};
use rand::{seq::IndexedRandom, RngExt};

const GRID_WIDTH: u8 = 14;
const GRID_HEIGHT: u8 = 8;
const MIN_DISTANCE: u8 = 3; // Minimum distance between player and skeletons
const VISION: u8 = 3; // How far the skeletons chase player from
struct DifficultyParameters {
    lives: u8,
    timer: u8,
    skeleton_count: u8,
    skeleton_speed: f32,
}

struct Game {
    difficulty: DifficultyParameters,
    player_position: (u8, u8),
    skeleton_positions: Vec<(u8, u8)>,
    coin_positions: Vec<(u8, u8)>,
    lives_left: u8,
    time_left: Duration,
    grid: Vec<Vec<TileType>>,
    score: u16,
}

const EASY: DifficultyParameters = DifficultyParameters {
    lives: 5,
    timer: 90,
    skeleton_count: 2,
    skeleton_speed: 1.0,
};

const MEDIUM : DifficultyParameters = DifficultyParameters {
    lives: 3,
    timer: 60,
    skeleton_count: 4,
    skeleton_speed: 1.5,
};

const HARD : DifficultyParameters = DifficultyParameters {
    lives: 1,
    timer: 45,
    skeleton_count: 6,
    skeleton_speed: 2.0,
};

#[derive(PartialEq)]
enum TileType {
    Floor,
    Wall,
    Obstacle,
}

#[derive(PartialEq, Clone)]
enum Input {
    Up,
    Down,
    Left,
    Right,
    None,
}

fn generate_grid(difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Vec<Vec<TileType>> {
    let mut grid = Vec::new();

    for y in 0..GRID_HEIGHT {
        let mut row = Vec::new();
        for x in 0..GRID_WIDTH {
            if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                row.push(TileType::Wall);
            } else {
                if rng.random_bool(0.25) { // 5% chance of obstacle
                    row.push(TileType::Obstacle);
                } else {
                    row.push(TileType::Floor);
                }
            }
        }
        grid.push(row);
    }
    grid
}

fn get_floor_tiles(grid: &Vec<Vec<TileType>>, exclude: &[(u8, u8)]) -> Vec<(u8, u8)> {
    let mut tiles = Vec::new();
    for y in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            if matches!(grid[y as usize][x as usize], TileType::Floor) 
                && !exclude.contains(&(x, y)) {
                tiles.push((x, y));
            }
        }
    }
    tiles
}

fn generate_skeletons(grid: &Vec<Vec<TileType>>, difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Vec<(u8, u8)> {
    let floor_tiles = get_floor_tiles(grid, &[]);
    let skeletons: Vec<(u8, u8)> = floor_tiles.sample(rng, difficulty.skeleton_count as usize).cloned().collect();
    skeletons
    
}

fn generate_coins(grid: &Vec<Vec<TileType>>, skeletons: &Vec<(u8, u8)>, rng: &mut impl rand::Rng) -> Vec<(u8, u8)> {
    let floor_tiles = get_floor_tiles(grid, skeletons);
    let coins: Vec<(u8, u8)> = floor_tiles.sample(rng, 10).cloned().collect(); // Place 10 coins
    coins
}

fn spawn_player(grid: &Vec<Vec<TileType>>, skeletons: &Vec<(u8, u8)>, coins: &Vec<(u8, u8)>,rng: &mut impl rand::Rng) -> (u8, u8) {
    let exclude = [skeletons.as_slice(), coins.as_slice()].concat();
    let floor_tiles: Vec<(u8, u8)> = get_floor_tiles(grid, &exclude)
                    .into_iter()
                    .filter(|&(x, y)| skeletons.iter()
                    .all(|&(sx, sy)| sx.abs_diff(x) + sy.abs_diff(y) >= MIN_DISTANCE))
                    .collect();
    floor_tiles.choose(rng).cloned().unwrap_or_else(|| {
        get_floor_tiles(grid, &exclude).into_iter().next().expect("No valid player spawn points available")
    })
}

fn move_skeletons(state: &mut Game, player_pos: (u8, u8), rng: &mut impl rand::Rng) {
    for mut skelly in &mut state.skeleton_positions {
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
        }
        else {
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

fn tick(mut state: Game, input: Input, rng: &mut impl rand::Rng) -> Option<Game> {
    let mut player_pos = state.player_position;
    move_skeletons(&mut state, player_pos, rng);
    match input {
        Input::Up => player_pos.1 = player_pos.1.saturating_sub(1),
        Input::Down => player_pos.1 = player_pos.1.saturating_add(1),
        Input::Left => player_pos.0 = player_pos.0.saturating_sub(1),
        Input::Right => player_pos.0 = player_pos.0.saturating_add(1),
        Input::None => (),
    }
    if !matches!(state.grid[player_pos.1 as usize][player_pos.0 as usize], TileType::Wall | TileType::Obstacle) {
        state.player_position = player_pos; // only update pos if not wall or obstacle
    }  
    if state.skeleton_positions.contains(&state.player_position) {
        state.lives_left = state.lives_left.saturating_sub(1);
        if state.lives_left == 0 {
            return None; // Game over
        }
        state.player_position = spawn_player(&state.grid, &state.skeleton_positions, &state.coin_positions, rng);
    }
    if state.coin_positions.contains(&state.player_position) {
        state.score += 10;
        state.coin_positions.retain(|&pos| pos != state.player_position);
    }
    Some(state)
}

fn test() {
    let mut rng = rand::rng();
    let grid = generate_grid(EASY, &mut rng);
    let skeletons = generate_skeletons(&grid, MEDIUM, &mut rng);
    let coins = generate_coins(&grid, &skeletons, &mut rng);
    let player = spawn_player(&grid, &skeletons, &coins, &mut rng);

    for (y, row) in grid.iter().enumerate() {
        let line: String = row.iter().enumerate().map(|(x, t)| {
            let pos = (x as u8, y as u8);
            if pos == player {
                'P'
            } else if skeletons.contains(&pos) {
                'S'
            } else if coins.contains(&pos) {
                'C'
            } else {
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
pub fn main() {
    // const TICKS_PER_SECOND: u64 = 64
    // const MS_PER_TICK: u128 = 1000 / TICKS_PER_SECOND as u128;
    // const SECONDS_PER_TICK: f64 = 1.0 / TICKS_PER_SECOND as f64;

    // let mut previous_time = Instant::now();
    // let mut lag = Duration::new(0, 0);
    
    // let mut game_state; // Placeholder for game state

    // 'game_loop: loop {
    //     let current_time = Instant::now();
    //     let elapsed = current_time.duration_since(previous_time);
    //     previous_time = current_time;
    //     lag += elapsed;

    //     // Handle input here 

    //     // Update game logic at fixed intervals

    //     while lag >= Duration::from_millis(MS_PER_TICK as u64) {
    //         // Update game logic here
    //         println!("Tick! Lag: {} ms", lag.as_millis());
    //         lag -= Duration::from_millis(MS_PER_TICK as u64);
    //     }

    //     // Render the game here 

    //     std::thread::sleep(Duration::from_millis(1)); // Sleep to prevent CPU hogging
    // }
    test();
}

