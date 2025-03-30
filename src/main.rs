pub mod chess_board;
pub mod chess_piece;
pub mod command_input;

use chess_board::ChessBoard;
use chess_piece::PieceTeam;
use command_input::{CommandInput, MoveCommand};
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

    let mut ui_camera = Camera2D {
        zoom: [1.0, 2.0 / 10.0].into(),
        offset: [-1.0, -1.0].into(),
        ..Default::default()
    };

    let mut fullscreen = false;

    let mut board = ChessBoard::new();
    let mut selected_tile = None;

    let mut command_input = CommandInput::default();

    loop {
        if input::is_key_pressed(KeyCode::F11) {
            fullscreen ^= true;
            window::set_fullscreen(fullscreen);
        }

        update_camera_aspect_ratio(&mut world_camera);
        update_camera_aspect_ratio(&mut ui_camera);

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

        if input::is_key_pressed(KeyCode::Home) {
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
                let mut end_tile = end_tile;

                #[rustfmt::skip]
                let offset = if board.turn == PieceTeam::Black { 1 } else { -1 };

                loop {
                    if let Some(selected_piece) = board.get_piece(end_tile).unwrap() {
                        if selected_piece.team == board.turn {
                            selected_tile = Some(end_tile);
                        }

                        break 'outer;
                    } else {
                        end_tile[0] += offset;
                    }
                }
            };

            if let Ok(()) = board.move_piece(start_tile, end_tile) {
                world_camera.target.y = -world_camera.target.y + 2.0 * SCREEN_START_POSITION;
            }

            selected_tile = None;
        }

        if let Some(command) = command_input.update() {
            match command {
                MoveCommand::MovePiece { start, end } => {
                    if let Ok(()) = board.move_piece(start, end) {
                        world_camera.target.y =
                            -world_camera.target.y + 2.0 * SCREEN_START_POSITION;
                        command_input.command.clear();
                    }
                }
                MoveCommand::MoveView { rank } => {
                    world_camera.target.y = (rank as f32 + 0.5) * ChessBoard::RANK_HEIGHT;
                    command_input.command.clear();
                }
                MoveCommand::Home => {
                    world_camera.target.y = SCREEN_START_POSITION;
                    command_input.command.clear();
                }
            }
        }

        camera::set_camera(&world_camera);

        board.draw_ranks(
            world_camera.target.y - SCREEN_HEIGHT / 2.0,
            world_camera.target.y + SCREEN_HEIGHT / 2.0,
            selected_tile,
        );

        camera::set_camera(&ui_camera);

        command_input.draw();

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
