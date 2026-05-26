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

enum Screen {
    MainMenu,
    DifficultySelect,
    HowToPlay,
    Playing,
    Paused,
    GameOver(usize),
}

const SPRITE_FLOOR: SpriteRect = SpriteRect { x: 16.0 * 7.0, y: 0.0 };
const SPRITE_WALL: SpriteRect = SpriteRect { x: 16.0 * 3.0 , y: 0.0 };
const SPRITE_OBSTACLE: SpriteRect = SpriteRect { x: 16.0 * 9.0, y: 16.0 * 4.0 };
const SPRITE_PLAYER: SpriteRect = SpriteRect { x: 16.0 * 4.0, y: 0.0 };
const SPRITE_SKELETON: SpriteRect = SpriteRect { x: 16.0 * 6.0, y: 16.0 * 3.0};
const SPRITE_COIN: SpriteRect = SpriteRect { x: 16.0 * 6.0, y: 16.0 * 8.0 };

const MAIN_MENU_BUTTONS: [&str; 3] = ["Play", "How To Play", "Exit"];
const DIFFICULTY_BUTTONS: [&str; 4] = ["Easy", "Medium", "Hard", "Back"];
const PAUSE_BUTTONS: [&str; 4] = ["Resume", "Restart", "How To Play","Main Menu"];
const GAME_OVER_BUTTONS: [&str; 2] = ["Play Again", "Main Menu"];

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


fn draw_button(text: &str, x: f32, y: f32, width: f32, max_h: f32, max_o: f32, font_size: f32, color: Color) {
    let size = measure_text(text, None, font_size as u16, 1.0);
    let pad_y = 10.0;
    let box_height = max_h + max_o + pad_y * 2.0;
    let box_y = y - max_h - pad_y;

    draw_rectangle(x, box_y, width, box_height, Color::from_rgba(50, 50, 58, 255));
    draw_rectangle_lines(x, box_y, width, box_height, 2.0, color);
    draw_text(text, x + (width - size.width) / 2.0, y, font_size, color);
}


fn generate_menus(title: &str, buttons: &[&str], selected: usize, title_color: Color, title_font_size: f32) {
    let title_size = measure_text(title, None, title_font_size as u16, 1.0);
    let screen_center_x = screen_width() / 2.0;
    let screen_center_y = screen_height() / 2.0 - 60.0;
    let font_size = 40.0;
    let pad_x = 28.0;

    draw_text(title,screen_center_x - title_size.width / 2.0,(GRID_HEIGHT as f32 * TILE_SIZE) / 5.0, title_font_size, title_color);

    // All buttons share the width and height of the widest/tallest one.
    let btn_width = buttons
        .iter()
        .map(|b| measure_text(b, None, font_size as u16, 1.0).width)
        .fold(0.0_f32, f32::max)
        + pad_x * 2.0;
    let max_h = buttons
        .iter()
        .map(|b| measure_text(b, None, font_size as u16, 1.0).height)
        .fold(0.0_f32, f32::max);
    let max_o = buttons
        .iter()
        .map(|b| measure_text(b, None, font_size as u16, 1.0).offset_y)
        .fold(0.0_f32, f32::max);

    for (i, button) in buttons.iter().enumerate() {
        let x = screen_center_x - btn_width / 2.0;
        let y = screen_center_y + i as f32 * (font_size + 40.0);
        let color = if i == selected { GOLD } else { WHITE };
        draw_button(button, x, y, btn_width, max_h, max_o, font_size, color);
    }
}

fn render_game(state: &Game, tileset: &Texture2D, charset: &Texture2D) {
    draw_text(format!("Score: {}", state.score), 10.0, 25.0, 30.0, GOLD);
    draw_text(format!("Lives: {}", state.lives_left), (GRID_WIDTH as f32 * TILE_SIZE) / 2.3, 25.0, 30.0, RED);
    draw_text(format!("Time: {}s", state.time_left.as_secs()), (GRID_WIDTH as f32 * TILE_SIZE) - 150.0, 25.0, 30.0, WHITE);

    // Phase 1: tiles
    for (y, row) in state.grid.iter().enumerate() {
        for (x, tile) in row.iter().enumerate() {
            let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE + HUD_HEIGHT);
            let (sprite, color) = match tile {
                TileType::Wall => (SPRITE_WALL, PURPLE),
                TileType::Floor => (SPRITE_FLOOR, PURPLE),
                TileType::Obstacle => (SPRITE_OBSTACLE, PURPLE),
            };
            if matches!(tile, TileType::Obstacle) {
                draw_texture_ex(tileset, pos.0, pos.1, color, sprite_params(SPRITE_FLOOR));
            }
            draw_texture_ex(tileset, pos.0, pos.1, color, sprite_params(sprite));
        }
    }

    // Phase 2: skeleton vision overlay, drawn over tiles but under entities. Overlapping skeletons stack.
    for &(sx, sy) in &state.skeleton_positions {
        for dy in -(VISION as i16)..=(VISION as i16) {
            for dx in -(VISION as i16)..=(VISION as i16) {
                if dx.abs() + dy.abs() > VISION as i16 { continue; }
                let tx = sx as i16 + dx;
                let ty = sy as i16 + dy;
                if tx < 0 || ty < 0 || tx >= GRID_WIDTH as i16 || ty >= GRID_HEIGHT as i16 || matches!(state.grid[ty as usize][tx as usize], TileType::Wall) { continue; }
                let px = tx as f32 * TILE_SIZE;
                let py = ty as f32 * TILE_SIZE + HUD_HEIGHT;
                draw_rectangle(px, py, TILE_SIZE, TILE_SIZE, Color::from_rgba(255, 50, 50, 10));
            }
        }
    }

    // Phase 3: entities
    for (y, row) in state.grid.iter().enumerate() {
        for (x, _tile) in row.iter().enumerate() {
            let pos = (x as f32 * TILE_SIZE, y as f32 * TILE_SIZE + HUD_HEIGHT);
            if state.player_position == (x as u8, y as u8) {
                draw_texture_ex(charset, pos.0, pos.1, WHITE, sprite_params(SPRITE_PLAYER));
            } else if state.skeleton_positions.contains(&(x as u8, y as u8)) {
                draw_texture_ex(charset, pos.0, pos.1, WHITE, sprite_params(SPRITE_SKELETON));
            } else if state.coin_positions.contains(&(x as u8, y as u8)) {
                draw_texture_ex(tileset, pos.0, pos.1, WHITE, sprite_params(SPRITE_COIN));
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let tileset = load_texture("assets/Dungeon_Tileset.png").await.unwrap();
    let charset = load_texture("assets/Dungeon_Character.png").await.unwrap();

    tileset.set_filter(FilterMode::Nearest);
    charset.set_filter(FilterMode::Nearest);

    let mut rng = rand::rng();
    let mut screen = Screen::MainMenu;
    let mut selected: usize = 0;
    let mut how_to_play_from_pause = false;
    let mut state: Game = new_game(MEDIUM, &mut rng); // overwritten when player picks difficulty
    let mut difficulty: DifficultyParameters = MEDIUM;

    loop {
        clear_background(Color::from_rgba(28, 28, 35, 255));

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

        match screen {
            Screen::MainMenu => {
                generate_menus("Dungeon Run", &MAIN_MENU_BUTTONS, selected, GOLD, 150.0);

                if matches!(input, Input::Up) && selected > 0 { selected -= 1; }
                if matches!(input, Input::Down) && selected < MAIN_MENU_BUTTONS.len() - 1 { selected += 1; }

                if is_key_pressed(KeyCode::Enter) {
                    match selected {
                        0 => screen = Screen::DifficultySelect,
                        1 => {screen = Screen::HowToPlay; how_to_play_from_pause = false},
                        _ => break,
                    }
                    selected = 0;
                }
            }

            Screen::DifficultySelect => {
                generate_menus("Select Difficulty", &DIFFICULTY_BUTTONS, selected, WHITE, 100.0);

                if matches!(input, Input::Up) && selected > 0 { selected -= 1; }
                if matches!(input, Input::Down) && selected < DIFFICULTY_BUTTONS.len() - 1 { selected += 1; }

                if is_key_pressed(KeyCode::Enter) {
                    difficulty = match selected {
                        0 => EASY,
                        1 => MEDIUM,
                        2 => HARD,
                        _ => {
                            screen = Screen::MainMenu;
                            selected = 0;
                            next_frame().await;
                            continue;
                        }
                    };
                    state = new_game(difficulty, &mut rng);
                    selected = 0;
                    screen = Screen::Playing;
                }
            }

            Screen::HowToPlay => {
                let screen_center_x = screen_width() / 2.0;

                // Title at top
                let title = "How To Play";
                let title_size = measure_text(title, None, 100, 1.0);
                draw_text(title, screen_center_x - title_size.width / 2.0, (GRID_HEIGHT as f32 * TILE_SIZE) / 5.0, 100.0, WHITE);

                // Instructions in the middle, multi-line
                let lines = [
                    "Move with WASD or arrow keys.",
                    "Pick up coins for points.",
                    "Avoid skeletons or lose a life.",
                    "Score as much as you can!",
                ];
                for (i, line) in lines.iter().enumerate() {
                    let sz = measure_text(line, None, 30, 1.0);
                    let y = 220.0 + i as f32 * 40.0;
                    draw_text(line, screen_center_x - sz.width / 2.0, y, 30.0, WHITE);
                }

                // Back button at the bottom
                let font_size = 40.0;
                let pad_x = 28.0;
                let back_dims = measure_text("Back", None, font_size as u16, 1.0);
                let btn_width = back_dims.width + pad_x * 2.0;
                draw_button("Back", screen_center_x - btn_width / 2.0, 450.0, btn_width, back_dims.height, back_dims.offset_y, font_size, GOLD);

                if is_key_pressed(KeyCode::Enter) {
                    screen = if how_to_play_from_pause { Screen::Paused } else { Screen::MainMenu };
                    selected = 0;
                }
            }

            Screen::Playing => {
                render_game(&state, &tileset, &charset);
                if is_key_pressed(KeyCode::Escape) {
                    screen = Screen::Paused;
                } else {
                    let score = state.score;
                    let delta = Duration::from_secs_f32(get_frame_time());
                    if let Some(new_state) = tick(state, input, delta, &mut rng) {
                        state = new_state;
                    } else {
                        screen = Screen::GameOver(score);
                        state = new_game(difficulty, &mut rng);
                    }
                }
            }

            Screen::Paused => {
                render_game(&state, &tileset, &charset);
                draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));
                generate_menus("Paused", &PAUSE_BUTTONS, selected, WHITE, 100.0);

                if matches!(input, Input::Up) && selected > 0 { selected -= 1; }
                if matches!(input, Input::Down) && selected < PAUSE_BUTTONS.len() - 1 { selected += 1; }

                if is_key_pressed(KeyCode::Enter) {
                    match selected {
                        0 => screen = Screen::Playing,
                        1 => {
                            state = new_game(difficulty, &mut rng);
                            screen = Screen::Playing;
                        },
                        2 => {screen = Screen::HowToPlay; how_to_play_from_pause = true;},
                        _ => screen = Screen::MainMenu,
                    }
                    selected = 0;
                }
            }

            Screen::GameOver(score) => {
                generate_menus("Game Over!", &GAME_OVER_BUTTONS, selected, RED, 100.0);

                // Final score, drawn between title and buttons
                let screen_center_x = screen_width() / 2.0;
                let score_str = format!("Final Score: {}", score);
                let score_dims = measure_text(&score_str, None, 40, 1.0);
                draw_text(&score_str, screen_center_x - score_dims.width / 2.0, 160.0, 40.0, WHITE);

                if matches!(input, Input::Up) && selected > 0 { selected -= 1; }
                if matches!(input, Input::Down) && selected < GAME_OVER_BUTTONS.len() - 1 { selected += 1; }

                if is_key_pressed(KeyCode::Enter) {
                    match selected {
                        0 => screen = Screen::Playing,
                        _ => screen = Screen::MainMenu,
                    }
                    selected = 0;
                }
            }
        }
        next_frame().await;
    }
}
