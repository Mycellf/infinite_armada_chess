pub mod chess_board;
pub mod chess_piece;

use chess_board::ChessBoard;
use macroquad::{
    camera::{self, Camera2D},
    input::{self, KeyCode, MouseButton},
    time, window,
};

#[macroquad::main("Infinite Armada Chess")]
async fn main() {
    const SCREEN_HEIGHT: f32 = 9.5;
    const SCREEN_START_POSITION: f32 =
        chess_board::NUM_TRADITIONAL_RANKS as f32 / 2.0 * ChessBoard::RANK_HEIGHT;

    let mut world_camera = Camera2D {
        zoom: [1.0, -2.0 / SCREEN_HEIGHT].into(),
        target: [ChessBoard::RANK_WIDTH / 2.0, SCREEN_START_POSITION].into(),
        ..Default::default()
    };

    let mut fullscreen = false;

    let mut board = ChessBoard::new();
    let mut selected_tile = None;

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            window::set_fullscreen(fullscreen);
        }

        update_camera_aspect_ratio(&mut world_camera);

        let input = input::is_key_down(KeyCode::Up) as i8 - input::is_key_down(KeyCode::Down) as i8;
        let pan_speed =
            if input::is_key_down(KeyCode::LeftShift) || input::is_key_down(KeyCode::RightShift) {
                16.0
            } else {
                4.0
            };

        let scroll_speed = 0.5;

        world_camera.target.y += input::mouse_wheel().1 * scroll_speed
            + input as f32 * pan_speed * time::get_frame_time();

        if input::is_key_pressed(KeyCode::C) {
            world_camera.target.y = SCREEN_START_POSITION;
        }

        // move selection
        'outer: {
            if !input::is_mouse_button_pressed(MouseButton::Left) {
                break 'outer;
            }

            let mouse_position = world_camera.screen_to_world(input::mouse_position().into());
            let clicked_tile = board.tile_at_position_bounded(mouse_position.into());

            let Some(end_tile) = clicked_tile else {
                selected_tile = None;
                break 'outer;
            };

            let Some(start_tile) = selected_tile else {
                if let Some(Some(selected_piece)) = board.get_piece(end_tile) {
                    if selected_piece.team == board.turn {
                        selected_tile = Some(end_tile);
                    }
                }

                break 'outer;
            };

            if let Ok(()) = board.move_piece(start_tile, end_tile) {
                world_camera.target.y = -world_camera.target.y + 2.0 * SCREEN_START_POSITION;
            }

            selected_tile = None;
        }

        camera::set_camera(&world_camera);

        board.draw_ranks(
            world_camera.target.y - SCREEN_HEIGHT / 2.0,
            world_camera.target.y + SCREEN_HEIGHT / 2.0,
            selected_tile,
        );

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
