use macroquad::prelude::*;

#[macroquad::main("Dungeon Run")]
async fn main() {
    loop {
        clear_background(BLACK);
        draw_text("Dungeon Run", 20.0, 20.0, 30.0, WHITE);
        next_frame().await;
    }
}
