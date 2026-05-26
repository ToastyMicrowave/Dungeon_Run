use std::{collections::{HashMap, VecDeque}, time::Duration};
use rand::{seq::IndexedRandom, RngExt};

pub const GRID_WIDTH: u8 = 14;
pub const GRID_HEIGHT: u8 = 8;
pub const MIN_DISTANCE: u8 = 3;
pub const VISION: u8 = 4;

const MIN_REGION_SIZE: usize = 2;
const OBSTACLE_CHANCE: f64 = 0.10;


pub type PathMap = HashMap<(u8, u8), HashMap<(u8, u8), (u8, u8)>>;

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
    pub score: usize,
    pub path_map: PathMap,
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
    skeleton_speed: 1.25,
};

pub const HARD: DifficultyParameters = DifficultyParameters {
    lives: 2,
    timer: 45,
    skeleton_count: 5,
    skeleton_speed: 1.5,
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

pub fn get_floor_tiles(grid: &[Vec<TileType>], exclude: &[(u8, u8)]) -> Vec<(u8, u8)> {
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

pub fn generate_skeletons(grid: &[Vec<TileType>], difficulty: DifficultyParameters, rng: &mut impl rand::Rng) -> Vec<(u8, u8)> {
    let floor_tiles = get_floor_tiles(grid, &[]);
    floor_tiles.sample(rng, difficulty.skeleton_count as usize).cloned().collect()
}

pub fn generate_coins(grid: &[Vec<TileType>], skeletons: &[(u8, u8)], player_pos: &(u8, u8), rng: &mut impl rand::Rng, path_map: &PathMap) -> Vec<(u8, u8)> {
    let exclude = [skeletons, [*player_pos].as_slice()].concat();
    let floor_tiles = get_floor_tiles(grid, &exclude).into_iter().filter(|tile| {path_map[player_pos].contains_key(tile) }).collect::<Vec<_>>();
    floor_tiles.sample(rng, 10.min(floor_tiles.len())).cloned().collect()
}

pub fn spawn_player(grid: &[Vec<TileType>], skeletons: &[(u8, u8)], rng: &mut impl rand::Rng) -> (u8, u8) {
    let floor_tiles: Vec<(u8, u8)> = get_floor_tiles(grid, skeletons)
        .into_iter()
        .filter(|&(x, y)| skeletons.iter().all(|&(sx, sy)| sx.abs_diff(x) + sy.abs_diff(y) >= MIN_DISTANCE))
        .collect();
    floor_tiles.choose(rng).cloned().unwrap_or_else(|| {
        get_floor_tiles(grid, skeletons).into_iter().next().expect("No valid player spawn points")
    })
}

fn bfs(grid: &[Vec<TileType>], source: (u8, u8)) -> HashMap<(u8, u8), (u8, u8)>  {
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

fn first_step_towards(parents: &HashMap<(u8, u8), (u8, u8)>, source: (u8, u8), target: (u8, u8)) -> Option<(u8, u8)> {
    if !parents.contains_key(&target) {
        return None;
    }
    let mut current = target;
    while parents[&current] != source {
        current = parents[&current];
    }
    Some(current)
}

fn build_map(grid: &[Vec<TileType>]) -> PathMap {
    let mut path_map = HashMap::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if matches!(grid[y as usize][x as usize], TileType::Floor) {
                let source = (x, y);
                let parents = bfs(grid, source);
                let mut target_map = HashMap::new();
                for target in parents.keys() {
                    if *target != source && let Some(step) = first_step_towards(&parents, source, *target) {
                        target_map.insert(*target, step);
                    }
                }
                path_map.insert(source, target_map);
            }
        }
    }
    path_map
}



pub fn move_skeletons(state: &mut Game, player_pos: (u8, u8), rng: &mut impl rand::Rng) {
    let mut skeletons = state.skeleton_positions.clone();
    for skelly in &mut state.skeleton_positions {
        let distance = skelly.0.abs_diff(player_pos.0) + skelly.1.abs_diff(player_pos.1);
        skeletons.retain(|&pos| pos != *skelly);
        if distance <= VISION {
            if let Some(step) = state.path_map[skelly].get(&player_pos) && !skeletons.contains(step) {
                    skelly.0 = step.0;
                    skelly.1 = step.1;
            }
        } else {
            let dirs: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
            let dir = dirs.choose(rng).unwrap();
            let new_x = (skelly.0 as i16 + dir.0 as i16) as u8;
            let new_y = (skelly.1 as i16 + dir.1 as i16) as u8;
            if matches!(state.grid[new_y as usize][new_x as usize], TileType::Floor) && !skeletons.contains(&(new_x, new_y)) {
                skelly.0 = new_x;
                skelly.1 = new_y;
            }
        }
        skeletons.push(*skelly);
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
        state.player_position = loop {
            let new_pos = spawn_player(&state.grid, &state.skeleton_positions, rng);
            if state.path_map[&new_pos].contains_key(&state.coin_positions[0]) {
                break new_pos;
            }
        }
            
    }
    if state.coin_positions.contains(&state.player_position) {
        state.score += 10;
        state.coin_positions.retain(|&pos| pos != state.player_position);
    }
    if state.coin_positions.is_empty() {
        state.score += 50;
        state.coin_positions = generate_coins(&state.grid, &state.skeleton_positions, &state.player_position, rng, &state.path_map);
    }
    Some(state)
}

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
        time_left: Duration::from_secs(difficulty.timer as u64),
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
