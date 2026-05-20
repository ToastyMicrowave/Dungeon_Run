use std::time::{Duration, Instant};
use rand::seq::IndexedRandom;

const GRID_WIDTH: u8 = 14;
const GRID_HEIGHT: u8 = 8;
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
    lives_left: u8,
    time_left: Duration,
    grid: Vec<Vec<u8>>,
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

enum TileType {
    Floor,
    Wall,
}

fn generate_grid(difficulty: DifficultyParameters) -> Vec<Vec<TileType>> {
    let mut grid = Vec::new();
    for y in 0..GRID_HEIGHT {
        let mut row = Vec::new();
        for x in 0..GRID_WIDTH {
            if x == 0 || x == GRID_WIDTH - 1 || y == 0 || y == GRID_HEIGHT - 1 {
                row.push(TileType::Wall);
            } else {
                row.push(TileType::Floor);
            }
        }
        grid.push(row);
    }
    grid
}

fn generate_skeletons(difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Vec<(u8, u8)> {
    let mut floor_tiles = Vec::new();
    for y in 1..GRID_HEIGHT - 1 {
        for x in 1..GRID_WIDTH - 1 {
            floor_tiles.push((x, y));
        }
    }
    let skeletons: Vec<(u8, u8)> = floor_tiles.choose_multiple(rng, difficulty.skeleton_count as usize).cloned().collect();
    skeletons
    
}

fn test() {
    let grid = generate_grid(EASY);
    // print grid — W = wall, . = floor
    for row in &grid {
        let line: String = row.iter().map(|t| match t {
            TileType::Wall => 'W',
            TileType::Floor => '.',
        }).collect();
        println!("{}", line);
    }

    let mut rng = rand::rng();
    let skeletons = generate_skeletons(MEDIUM, &mut rng);
    println!("Skeletons: {:?}", skeletons);

    // verify no duplicates
    let unique: std::collections::HashSet<_> = skeletons.iter().collect();
    assert_eq!(unique.len(), skeletons.len(), "duplicate skeleton positions!");
    println!("No duplicates. Done.");
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

