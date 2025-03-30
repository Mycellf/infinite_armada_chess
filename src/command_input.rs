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
        const FONT_UI_SIZE: f32 = 0.5;
        const FONT_PIXEL_SIZE: u16 = 8;
        const FONT_SCALE: f32 = FONT_UI_SIZE / FONT_PIXEL_SIZE as f32;

        const BOX_HEIGHT: f32 = FONT_UI_SIZE;
        const CURSOR_HEIGHT: f32 = FONT_UI_SIZE * 3.0 / 4.0;

        const HORIZONTAL_OFFSET: f32 = FONT_UI_SIZE / 4.0;
        const VERTICAL_OFFSET: f32 = FONT_UI_SIZE / 4.0;

        if self.command.is_empty() {
            return;
        }

        let TextDimensions { width, .. } = text::draw_text_ex(
            &self.command,
            HORIZONTAL_OFFSET,
            -VERTICAL_OFFSET,
            TextParams {
                font_size: FONT_PIXEL_SIZE,
                font_scale: FONT_SCALE,
                color: colors::BLANK,
                ..Default::default()
            },
        );

        shapes::draw_rectangle(
            0.0,
            -BOX_HEIGHT,
            HORIZONTAL_OFFSET * 2.0 + width,
            BOX_HEIGHT,
            colors::BLACK,
        );

        if self.command.len() < Self::MAX_COMMAND_LENGTH {
            shapes::draw_rectangle(
                HORIZONTAL_OFFSET + width,
                -(BOX_HEIGHT + CURSOR_HEIGHT) / 2.0,
                FONT_UI_SIZE / 16.0,
                CURSOR_HEIGHT,
                colors::WHITE,
            );
        }

        text::draw_text_ex(
            &self.command,
            HORIZONTAL_OFFSET,
            -VERTICAL_OFFSET,
            TextParams {
                font_size: FONT_PIXEL_SIZE,
                font_scale: FONT_SCALE,
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
