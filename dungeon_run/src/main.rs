use macroquad::prelude::*;
use std::time::Duration;
use ::rand;
use dungeon_core::*;

const TILE_SIZE: f32 = 64.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Dungeon Run".to_string(),
        window_width: GRID_WIDTH as i32 * TILE_SIZE as i32,
        window_height: GRID_HEIGHT as i32 * TILE_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut rng = rand::rng();
    let mut state: Game = new_game(HARD, &mut rng);
    loop {
        clear_background(BLACK);

        let input = if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
            Input::Up
        } else if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
            Input::Down
        } else if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            Input::Left
        } else if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            Input::Right
        } else {
            Input::None
        };

        for (y, row) in state.grid.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                match tile {
                    TileType::Wall => draw_rectangle(pos.0, pos.1, TILE_SIZE, TILE_SIZE, PURPLE),
                    TileType::Floor => draw_rectangle(pos.0, pos.1, TILE_SIZE, TILE_SIZE, PINK),
                    TileType::Obstacle => draw_rectangle(pos.0, pos.1, TILE_SIZE, TILE_SIZE, BLACK),
                }
                if state.player_position == (x as u8, y as u8) {
                    draw_rectangle(pos.0 + 16.0, pos.1 + 16.0, TILE_SIZE - 32.0, TILE_SIZE - 32.0, GREEN);
                } else if state.skeleton_positions.contains(&(x as u8, y as u8)) {
                    draw_rectangle(pos.0 + 16.0, pos.1 + 16.0, TILE_SIZE - 32.0, TILE_SIZE - 32.0, WHITE);
                } else if state.coin_positions.contains(&(x as u8, y as u8)) {
                    draw_rectangle(pos.0 + 16.0, pos.1 + 16.0, TILE_SIZE - 32.0, TILE_SIZE - 32.0, GOLD);
                }
            }
        }

        let delta = Duration::from_secs_f32(get_frame_time());
        if let Some(new_state) = tick(state, input, delta, &mut rng) {
            state = new_state;
        } else {
            state = new_game(HARD, &mut rng);
        }

        next_frame().await;
    }
}
