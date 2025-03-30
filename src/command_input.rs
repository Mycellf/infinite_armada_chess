use macroquad::{
    color::colors,
    input::{self, KeyCode},
    shapes,
    text::{self, TextDimensions, TextParams},
};

use crate::chess_board;

#[derive(Default)]
pub struct CommandInput {
    pub command: String,
}

impl CommandInput {
    pub const MAX_COMMAND_LENGTH: usize = 20;

    pub fn update(&mut self) -> Option<MoveCommand> {
        while let Some(character) = input::get_char_pressed() {
            match character {
                // backspace
                '\x08' => {
                    self.command.pop();
                }
                other => {
                    if (other.is_ascii_graphic() || (other == ' ' && !self.command.is_empty()))
                        && self.command.len() < Self::MAX_COMMAND_LENGTH
                    {
                        self.command.push(other);
                    }
                }
            }
        }

        if input::is_key_pressed(KeyCode::Enter) {
            let result = MoveCommand::from_command(&self.command);
            self.command.clear();
            result
        } else {
            None
        }
    }

    pub fn draw(&self) {
        if self.command.is_empty() {
            return;
        }

        let font_ui_size: f32 = 0.5;

        let (font_size, font_scale, _) = text::camera_font_scale(font_ui_size);

        let box_height: f32 = font_ui_size;
        let cursor_height: f32 = font_ui_size * 3.0 / 4.0;

        let horizontal_offset: f32 = font_ui_size / 4.0;
        let vertical_offset: f32 = font_ui_size / 4.0;

        let TextDimensions { width, .. } =
            text::measure_text(&self.command, None, font_size, font_scale);

        shapes::draw_rectangle(
            0.0,
            -box_height,
            horizontal_offset * 2.0 + width,
            box_height,
            colors::BLACK,
        );

        if self.command.len() < Self::MAX_COMMAND_LENGTH {
            shapes::draw_rectangle(
                horizontal_offset + width,
                -(box_height + cursor_height) / 2.0,
                font_ui_size / 16.0,
                cursor_height,
                colors::WHITE,
            );
        }

        text::draw_text_ex(
            &self.command,
            horizontal_offset,
            -vertical_offset,
            TextParams {
                font_size,
                font_scale,
                color: colors::WHITE,
                ..Default::default()
            },
        );
    }
}

pub enum MoveCommand {
    MovePiece { start: [isize; 2], end: [isize; 2] },
}

impl MoveCommand {
    pub fn from_command(command: &str) -> Option<Self> {
        let mut locations = command.split_whitespace().map(parse_position);

        let Some(Some(start)) = locations.next() else {
            return None;
        };

        let Some(Some(end)) = locations.next() else {
            return None;
        };

        let None = locations.next() else {
            return None;
        };

        Some(Self::MovePiece { start, end })
    }
}

pub fn parse_position(position: &str) -> Option<[isize; 2]> {
    let file = position.chars().next().unwrap();
    let rank = &position[1..];

    if file < 'a' || file >= ('a' as u8 + chess_board::NUM_FILES as u8) as char {
        return None;
    }

    let file = (file as u8 - 'a' as u8) as isize;

    let rank = rank.parse::<isize>().ok()? - 1;

    Some([rank, file])
}
