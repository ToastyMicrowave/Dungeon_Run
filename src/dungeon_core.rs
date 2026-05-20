use std::time::{Duration, Instant};

const GRID_WIDTH: u8 = 14;
const GRID_HEIGHT: u8 = 8;
struct DifficultyParameters {
    lives: u8,
    timer: Duration,
    skeleton_count: u8,
    skeleton_speed: f32,
}

struct Game {
    difficulty: DifficultyParameters,
    player_position: (u32, u32),
    skeleton_positions: Vec<(u32, u32)>,
    lives_left: u8,
    time_left: Duration,
    grid: Vec<Vec<u8>>,
}

const EASY: DifficultyParameters = DifficultyParameters {
    lives: 5,
    timer: Duration::from_secs(90),
    skeleton_count: 2,
    skeleton_speed: 1.0,
};

const MEDIUM : DifficultyParameters = DifficultyParameters {
    lives: 3,
    timer: Duration::from_secs(60),
    skeleton_count: 4,
    skeleton_speed: 1.5,
};

const HARD : DifficultyParameters = DifficultyParameters {
    lives: 1,
    timer: Duration::from_secs(45),
    skeleton_count: 6,
    skeleton_speed: 2.0,
};

pub fn main() {
    const TICKS_PER_SECOND: u64 = 64
    const MS_PER_TICK: u128 = 1000 / TICKS_PER_SECOND as u128;
    const SECONDS_PER_TICK: f64 = 1.0 / TICKS_PER_SECOND as f64;

    let mut previous_time = Instant::now();
    let mut lag = Duration::new(0, 0);
    
    let mut game_state = GameState {x: 0.0}; // Placeholder for game state

    'game_loop: loop {
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(previous_time);
        previous_time = current_time;
        lag += elapsed;

        // Handle input here 

        // Update game logic at fixed intervals

        while lag >= Duration::from_millis(MS_PER_TICK as u64) {
            // Update game logic here
            println!("Tick! Lag: {} ms", lag.as_millis());
            lag -= Duration::from_millis(MS_PER_TICK as u64);
        }

        // Render the game here 

        std::thread::sleep(Duration::from_millis(1)); // Sleep to prevent CPU hogging
    }
}

