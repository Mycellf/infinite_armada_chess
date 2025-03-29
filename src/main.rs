pub mod chess_board;

use chess_board::ChessBoard;
use macroquad::{
    camera::{self, Camera2D},
    input::{self, KeyCode},
    time, window,
};

#[macroquad::main("Infinite Armada Chess")]
async fn main() {
    const SCREEN_HEIGHT: f32 = 9.5;
    const SCREEN_START_POSITION: f32 = -(SCREEN_HEIGHT - 8.0) / 2.0;

    let mut world_camera = Camera2D {
        zoom: [1.0, -2.0 / SCREEN_HEIGHT].into(),
        offset: [0.0, -1.0].into(),
        target: [ChessBoard::RANK_WIDTH / 2.0, SCREEN_START_POSITION].into(),
        ..Default::default()
    };

    let mut fullscreen = false;

    let mut board = ChessBoard::new();

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            window::set_fullscreen(fullscreen);
        }

        let input = input::is_key_down(KeyCode::Up) as i8 - input::is_key_down(KeyCode::Down) as i8;
        let speed =
            if input::is_key_down(KeyCode::LeftShift) || input::is_key_down(KeyCode::RightShift) {
                16.0
            } else {
                4.0
            };

        world_camera.target.y += input as f32 * speed * time::get_frame_time();

        if input::is_key_pressed(KeyCode::C) {
            world_camera.target.y = SCREEN_START_POSITION;
        }

        if input::is_key_pressed(KeyCode::Space) {
            board.turn = board.turn.opposite();
            world_camera.target.y = -world_camera.target.y + 2.0 * SCREEN_START_POSITION;
        }

        update_camera_aspect_ratio(&mut world_camera);
        camera::set_camera(&world_camera);

        board.draw_ranks(world_camera.target.y, world_camera.target.y + SCREEN_HEIGHT);

        window::next_frame().await;
    }
}

fn update_camera_aspect_ratio(camera: &mut Camera2D) {
    if let Some((_, _, size_x, size_y)) = camera.viewport {
        camera.zoom.x = camera.zoom.y.abs() * size_y as f32 / size_x as f32;
    } else {
        camera.zoom.x = camera.zoom.y.abs() * window::screen_height() / window::screen_width();
    }
}
