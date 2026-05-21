use macroquad::prelude::*;
use std::time::Duration;
use ::rand;
use dungeon_core::*;

const TILE_SIZE: f32 = 64.0;
const HUD_HEIGHT: f32 = 40.0;

struct SpriteRect {
    x: f32,
    y: f32,
}

const SPRITE_FLOOR: SpriteRect = SpriteRect { x: 16.0 * 7.0, y: 0.0 };
const SPRITE_WALL: SpriteRect = SpriteRect { x: 16.0 * 3.0 , y: 0.0 };
const SPRITE_OBSTACLE: SpriteRect = SpriteRect { x: 16.0 * 9.0, y: 16.0 * 4.0 };
const SPRITE_PLAYER: SpriteRect = SpriteRect { x: 16.0 * 4.0, y: 0.0 };
const SPRITE_SKELETON: SpriteRect = SpriteRect { x: 16.0 * 6.0, y: 16.0 * 3.0};
const SPRITE_COIN: SpriteRect = SpriteRect { x: 16.0 * 6.0, y: 16.0 * 8.0 };

fn window_conf() -> Conf {
    Conf {
        window_title: "Dungeon Run".to_string(),
        window_width: GRID_WIDTH as i32 * TILE_SIZE as i32,
        window_height: GRID_HEIGHT as i32 * TILE_SIZE as i32 + HUD_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

fn sprite_params(sprite: SpriteRect) -> DrawTextureParams {
    DrawTextureParams {
        dest_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
        source: Some(Rect::new(sprite.x, sprite.y, 16.0, 16.0)),
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let tileset = load_texture("assets/Dungeon_Tileset.png").await.unwrap();
    let charset = load_texture("assets/Dungeon_Character.png").await.unwrap();

    tileset.set_filter(FilterMode::Nearest); // Prevent blurring when scaling
    charset.set_filter(FilterMode::Nearest);
    let mut rng = rand::rng();
    let mut state: Game = new_game(MEDIUM, &mut rng);
    
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
        draw_text(&format!("Score: {}", state.score), 10.0, 25.0, 30.0, WHITE);
        draw_text(&format!("Lives: {}", state.lives_left), (GRID_WIDTH as f32 * TILE_SIZE as f32) as f32 / 2.3, 25.0, 30.0, WHITE);
        draw_text(&format!("Time: {}s", state.time_left.as_secs()), (GRID_WIDTH as f32 * TILE_SIZE as f32) - 150.0, 25.0, 30.0, WHITE);

        for (y, row) in state.grid.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE + HUD_HEIGHT);
                let (sprite, color) = match tile {
                    TileType::Wall => (SPRITE_WALL, PURPLE),
                    TileType::Floor => (SPRITE_FLOOR, PURPLE),
                    TileType::Obstacle => (SPRITE_OBSTACLE, PURPLE),
                };
                if matches!(tile, TileType::Obstacle) {
                    draw_texture_ex(&tileset, pos.0, pos.1, color, sprite_params(SPRITE_FLOOR));
                }
                draw_texture_ex(&tileset, pos.0, pos.1, color, sprite_params(sprite));

                if state.player_position == (x as u8, y as u8) {
                    draw_texture_ex(&charset, pos.0, pos.1, WHITE, sprite_params(SPRITE_PLAYER));
                } else if state.skeleton_positions.contains(&(x as u8, y as u8)) {
                    draw_texture_ex(&charset, pos.0, pos.1, WHITE, sprite_params(SPRITE_SKELETON));
                } else if state.coin_positions.contains(&(x as u8, y as u8)) {
                    draw_texture_ex(&tileset, pos.0, pos.1, WHITE, sprite_params(SPRITE_COIN));
                }
            }
        }
        let score = state.score;
        let delta = Duration::from_secs_f32(get_frame_time());
        if let Some(new_state) = tick(state, input, delta, &mut rng) {
            state = new_state;
        } else {
            loop {
                clear_background(BLACK);
                draw_text("Game Over!", (GRID_WIDTH as f32 * TILE_SIZE as f32) / 2.5, (GRID_HEIGHT as f32 * TILE_SIZE as f32) / 2.0, 50.0, RED);
                draw_text(&format!("Final Score: {}", score), (GRID_WIDTH as f32 * TILE_SIZE as f32) / 2.8, (GRID_HEIGHT as f32 * TILE_SIZE as f32) / 1.8, 30.0, WHITE);
                if is_key_pressed(KeyCode::Enter) {
                    state = new_game(MEDIUM, &mut rng);
                    break;
                }
                next_frame().await;
            }
        }

        next_frame().await;
    }
}
