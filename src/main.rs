pub mod chess_board;
pub mod chess_piece;
pub mod command_input;
pub mod textures;

use chess_board::{ChessBoard, SelectionMode};
use chess_piece::PieceTeam;
use command_input::{CommandInput, MoveCommand};
use macroquad::{
    camera::{self, Camera2D},
    input::{self, KeyCode, MouseButton},
    time, window,
};

#[macroquad::main("Infinite Armada Chess")]
async fn main() {
    const SCREEN_HEIGHT_INCREMENT: f32 = 10.0;

    const SCREEN_START_POSITION: f32 =
        chess_board::NUM_TRADITIONAL_RANKS as f32 / 2.0 * ChessBoard::RANK_HEIGHT;
    let mut rank_offset = 0;

    let mut screen_height = SCREEN_HEIGHT_INCREMENT;
    let mut zoom_level = 1.0;

    let mut world_camera = Camera2D {
        zoom: [1.0, -2.0 / screen_height].into(),
        target: [ChessBoard::RANK_WIDTH / 2.0, SCREEN_START_POSITION].into(),
        ..Default::default()
    };

    fn flip_camera(camera: &mut Camera2D) {
        camera.target.y = -camera.target.y + 2.0 * SCREEN_START_POSITION;
    }

    let mut ui_camera = Camera2D {
        zoom: [1.0, 2.0 / 10.0].into(),
        offset: [-1.0, -1.0].into(),
        ..Default::default()
    };

    let mut fullscreen = false;

    let mut board = ChessBoard::default();
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
                32.0
            } else {
                8.0
            };

        let scroll_speed = 0.5;

        let input_motion = input::mouse_wheel().1 * scroll_speed
            + input as f32 * pan_speed * time::get_frame_time();

        world_camera.target.y += input_motion * zoom_level;

        // move selection
        'outer: {
            if !input::is_mouse_button_pressed(MouseButton::Left) {
                break 'outer;
            }

            let mouse_position = world_camera.screen_to_world(input::mouse_position().into());

            if let SelectionMode::PromotePiece(location) = board.selection_mode {
                let clicked_tile = board.tile_at_position(mouse_position.into());

                let Some(rank) = clicked_tile[0].checked_add(rank_offset) else {
                    break 'outer;
                };

                let clicked_tile = [rank, clicked_tile[1]];

                if clicked_tile[1] != location[1] {
                    break 'outer;
                }

                let mut selected_index = location[0] - clicked_tile[0];

                if PieceTeam::Black == board.turn {
                    selected_index = -selected_index;
                };

                selected_index -= 1;

                if selected_index < 0 {
                    break 'outer;
                }

                if board.select_promotion(selected_index as usize).is_some() {
                    flip_camera(&mut world_camera);
                }

                break 'outer;
            }

            let clicked_tile = board.tile_at_position_bounded(mouse_position.into());

            let Some(end_tile) = clicked_tile else {
                selected_tile = None;
                break 'outer;
            };

            let Some(end_rank) = end_tile[0].checked_add(rank_offset) else {
                selected_tile = None;
                break 'outer;
            };
            let end_tile = [end_rank, end_tile[1]];

            let Some(start_tile) = selected_tile else {
                let mut end_tile = end_tile;

                let seek_down = input::is_key_down(KeyCode::LeftShift)
                    || input::is_key_down(KeyCode::RightShift);

                #[rustfmt::skip]
                let offset = if board.turn == PieceTeam::Black { 1 } else { -1 };

                loop {
                    if let Some(selected_piece) = board.get_piece(end_tile).unwrap() {
                        if selected_piece.team == board.turn {
                            selected_tile = Some(end_tile);
                        }

                        break 'outer;
                    } else if seek_down {
                        end_tile[0] += offset;
                    } else {
                        break 'outer;
                    }
                }
            };

            if let Some(true) = board.move_piece(start_tile, end_tile) {
                flip_camera(&mut world_camera);
            }

            selected_tile = None;
        }

        if let Some(command) = command_input.update() {
            match command {
                MoveCommand::MovePiece { start, end } => {
                    if let Some(flip_camera_a) = board.move_piece(start, end) {
                        if flip_camera_a {
                            flip_camera(&mut world_camera);
                        }
                        command_input.command.clear();
                    }
                }
                MoveCommand::MoveView { rank } => {
                    if let SelectionMode::PromotePiece(..) = board.selection_mode {
                        if rank < 0 {
                        } else if board.select_promotion(rank as usize).is_some() {
                            flip_camera(&mut world_camera);
                            command_input.command.clear();
                        }
                    } else {
                        world_camera.target.y = 0.5 * ChessBoard::RANK_HEIGHT;
                        if board.turn == PieceTeam::Black {
                            rank_offset = rank
                                .saturating_sub((chess_board::NUM_TRADITIONAL_RANKS - 1) as isize);
                        } else {
                            rank_offset = rank;
                        }
                        command_input.command.clear();
                    }
                }
                MoveCommand::Home => {
                    world_camera.target.y = SCREEN_START_POSITION;
                    rank_offset = 0;
                    command_input.command.clear();
                }
            }
        }

        'outer: {
            if command_input.command.is_empty() {
                zoom_level = if input::is_key_pressed(KeyCode::Key1) {
                    1.0
                } else if input::is_key_pressed(KeyCode::Key2) {
                    2.0
                } else if input::is_key_pressed(KeyCode::Key3) {
                    4.0
                } else {
                    break 'outer;
                };

                screen_height = SCREEN_HEIGHT_INCREMENT * zoom_level;

                world_camera.zoom.y = -2.0 / screen_height;
                update_camera_aspect_ratio(&mut world_camera);
            }
        }

        let camera_nudge = world_camera.target.y.round() as isize;
        world_camera.target.y -= camera_nudge as f32;

        let offset_nudge = if board.turn == PieceTeam::Black {
            -camera_nudge
        } else {
            camera_nudge
        };
        rank_offset = rank_offset.saturating_add(offset_nudge);

        camera::set_camera(&world_camera);

        board.draw_ranks(
            world_camera.target.y - screen_height / 2.0 + 0.5,
            world_camera.target.y + screen_height / 2.0 - 0.5,
            rank_offset,
            if let SelectionMode::PromotePiece(location) = board.selection_mode {
                Some(location)
            } else {
                selected_tile
            },
        );

        board.draw_piece_selection(rank_offset);

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
