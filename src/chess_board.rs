use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
};

use macroquad::{
    color::{Color, colors},
    shapes,
    text::{self, TextDimensions, TextParams},
    texture::{self, DrawTextureParams},
};

use crate::chess_piece::{ChessPiece, PieceKind, PieceTeam};

pub const NUM_FILES: usize = 8;
pub const NUM_TRADITIONAL_RANKS: usize = 8;

pub type Rank = [Option<ChessPiece>; NUM_FILES];

#[derive(Clone, Debug)]
pub struct ChessBoard {
    pub ranks: VecDeque<Rank>,
    pub ranks_behind_white: usize,
    pub turn: PieceTeam,
}

impl ChessBoard {
    pub fn new() -> Self {
        let mut ranks = VecDeque::with_capacity(NUM_TRADITIONAL_RANKS);

        ranks.push_back(KING_RANK_WHITE);
        ranks.push_back(PAWN_RANK_WHITE);

        for _ in 0..NUM_TRADITIONAL_RANKS - 4 {
            ranks.push_back(EMPTY_RANK);
        }

        ranks.push_back(PAWN_RANK_BLACK);
        ranks.push_back(KING_RANK_BLACK);

        Self {
            ranks,
            ranks_behind_white: 0,
            turn: PieceTeam::White,
        }
    }

    pub fn move_piece(&mut self, from: [isize; 2], to: [isize; 2]) -> Result<(), ()> {
        let turn = self.turn;

        let Some(starting_tile) = self.get_piece(from) else {
            return Err(());
        };

        let Some(starting_piece) = starting_tile else {
            return Err(());
        };

        if starting_piece.team != turn {
            return Err(());
        }

        let moved_tile = Some(starting_piece.moved());

        if !self.check_move(from, to) {
            return Err(());
        }

        let Some(ending_tile) = self.get_piece_expanding(to) else {
            return Err(());
        };

        *ending_tile = moved_tile;

        let Some(starting_tile) = self.get_piece_expanding(from) else {
            return Err(());
        };

        *starting_tile = None;

        self.turn = self.turn.opposite();

        Ok(())
    }

    pub fn check_move(&self, from: [isize; 2], to: [isize; 2]) -> bool {
        let Some(Some(starting_piece)) = self.get_piece(from) else {
            return false;
        };

        let Some(destination_tile) = self.get_piece(to) else {
            return false;
        };

        let offset = [0, 1].map(|i| to[i] - from[i]);

        // find the move being referenced
        let Some(&piece_move) = (starting_piece.moves().iter())
            .filter(|&&piece_move| piece_move.is_offset_valid(offset))
            .next()
        else {
            return false;
        };

        // check that the destination is valid
        if !(destination_tile.is_some() && piece_move.can_capture
            || destination_tile.is_none() && piece_move.can_move)
        {
            return false;
        }

        // if the destination tile is another piece, check that it's not an ally
        if let Some(ChessPiece { team, .. }) = destination_tile {
            if team == starting_piece.team {
                return false;
            }
        }

        if piece_move.repeating {
            // check that any intermediate moves are valid
            let mut tile_index = from;

            loop {
                for i in [0, 1] {
                    tile_index[i] += piece_move.offset()[i];
                }

                if tile_index == to {
                    break;
                }

                if self.get_piece(tile_index).unwrap().is_some() {
                    return false;
                }
            }
        }

        true
    }

    pub fn expand_to_rank(&mut self, rank: isize) {
        if rank < self.first_rank() {
            for _ in rank..self.first_rank() {
                self.ranks.push_front(QUEEN_RANK_WHITE);
            }

            self.ranks_behind_white = -rank as usize;
        } else if rank > self.last_rank() {
            for _ in self.last_rank()..rank {
                self.ranks.push_back(QUEEN_RANK_BLACK);
            }
        }
    }

    pub fn index_of_rank(&self, rank: isize) -> isize {
        rank + self.ranks_behind_white as isize
    }

    pub fn first_rank(&self) -> isize {
        -(self.ranks_behind_white as isize)
    }

    pub fn last_rank(&self) -> isize {
        self.ranks.len() as isize + self.first_rank() - 1
    }

    pub fn invert_rank(&self, rank: isize) -> isize {
        NUM_TRADITIONAL_RANKS as isize - rank - 1
    }

    pub fn invert_file(&self, file: isize) -> isize {
        NUM_FILES as isize - file - 1
    }

    pub fn get_piece(&self, [rank, file]: [isize; 2]) -> Option<Option<ChessPiece>> {
        Some(*self.get_rank(rank).get(usize::try_from(file).ok()?)?)
    }

    pub fn get_piece_mut(&mut self, [rank, file]: [isize; 2]) -> Option<&mut Option<ChessPiece>> {
        self.get_rank_mut(rank)?
            .get_mut(usize::try_from(file).ok()?)
    }

    pub fn get_piece_expanding(
        &mut self,
        [rank, file]: [isize; 2],
    ) -> Option<&mut Option<ChessPiece>> {
        self.get_rank_expanding(rank)
            .get_mut(usize::try_from(file).ok()?)
    }

    pub fn get_rank(&self, rank: isize) -> &Rank {
        if let Ok(rank_index) = self.index_of_rank(rank).try_into() {
            self.ranks.get(rank_index).unwrap_or(
                // rank is too high
                &QUEEN_RANK_BLACK,
            )
        } else {
            // rank is too low
            &QUEEN_RANK_WHITE
        }
    }

    pub fn get_rank_mut(&mut self, rank: isize) -> Option<&mut Rank> {
        self.ranks
            .get_mut(self.index_of_rank(rank).try_into().ok()?)
    }

    pub fn get_rank_expanding(&mut self, rank: isize) -> &mut Rank {
        self.expand_to_rank(rank);

        self.get_rank_mut(rank).unwrap()
    }
}

impl Index<isize> for ChessBoard {
    type Output = Rank;

    fn index(&self, index: isize) -> &Self::Output {
        self.get_rank(index)
    }
}

impl IndexMut<isize> for ChessBoard {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        self.get_rank_expanding(index)
    }
}

impl ChessBoard {
    pub const TILE_SIZE: f32 = 1.0;
    pub const RANK_HEIGHT: f32 = Self::TILE_SIZE;
    pub const RANK_WIDTH: f32 = Self::TILE_SIZE * NUM_FILES as f32;

    // colors from the palette used by lichess.org
    pub const DARK_TILE_COLOR: Color = Color::from_hex(0xb58863);
    pub const LIGHT_TILE_COLOR: Color = Color::from_hex(0xf0d9b5);

    pub fn draw_ranks(&self, start: f32, end: f32, highlighted_tile: Option<[isize; 2]>) {
        let lowest = start.floor() as isize;
        let highest = (end - Self::RANK_HEIGHT).ceil() as isize;

        let (start_rank, end_rank) = if self.turn == PieceTeam::Black {
            (self.invert_rank(highest), self.invert_rank(lowest))
        } else {
            (lowest, highest)
        };

        for rank in start_rank..end_rank + 1 {
            let highlights_mask = if let Some(highlighted_tile) = highlighted_tile {
                if rank == highlighted_tile[0] {
                    1 << highlighted_tile[1]
                } else {
                    0
                }
            } else {
                0
            };

            self.draw_rank(rank, highlights_mask);
        }

        shapes::draw_rectangle(
            -2.0,
            start - 0.5,
            Self::RANK_WIDTH + 4.0,
            0.5,
            colors::BLACK,
        );
        shapes::draw_rectangle(-2.0, end, Self::RANK_WIDTH + 4.0, 0.5, colors::BLACK);

        for file in 0..NUM_FILES {
            let tile_x = self.x_position_of_file(file as isize) + Self::TILE_SIZE / 2.0;

            let file_string = (('a' as u8 + file as u8) as char).to_string();

            let foreground = colors::WHITE;
            let background = colors::BLANK;

            #[rustfmt::skip]
            {
                draw_boxed_text(&file_string, tile_x, start, 0.5, [0.5, 1.0], foreground, background);
                draw_boxed_text(&file_string, tile_x, end, 0.5, [0.5, 0.0], foreground, background);
            };
        }
    }

    pub fn tile_at_position(&self, position: [f32; 2]) -> [isize; 2] {
        let rank = (position[1] / Self::RANK_HEIGHT).floor() as isize;
        let file = (position[0] / Self::TILE_SIZE).floor() as isize;

        if self.turn == PieceTeam::Black {
            [self.invert_rank(rank), self.invert_file(file)]
        } else {
            [rank, file]
        }
    }

    pub fn tile_at_position_bounded(&self, position: [f32; 2]) -> Option<[isize; 2]> {
        let [rank, file] = self.tile_at_position(position);

        if file >= 0 && file < NUM_FILES as isize {
            Some([rank, file])
        } else {
            None
        }
    }

    pub fn draw_rank(&self, rank: isize, mut hightlights_mask: u8) {
        let height = self.height_of_rank(rank);
        let mut tile_parity = rank % 2 == 0;

        let rank_contents = self.get_rank(rank);

        for file in 0..NUM_FILES {
            let tile_x = self.x_position_of_file(file as isize);

            let tile_color = if hightlights_mask & 1 == 1 {
                colors::WHITE
            } else if tile_parity {
                Self::DARK_TILE_COLOR
            } else {
                Self::LIGHT_TILE_COLOR
            };

            shapes::draw_rectangle(tile_x, height, Self::TILE_SIZE, Self::TILE_SIZE, tile_color);

            if let Some(piece) = rank_contents[file] {
                texture::draw_texture_ex(
                    piece.texture(),
                    tile_x,
                    height,
                    colors::WHITE,
                    DrawTextureParams {
                        dest_size: Some([Self::TILE_SIZE; 2].into()),
                        flip_y: true,
                        ..Default::default()
                    },
                );
            }

            tile_parity ^= true;
            hightlights_mask >>= 1;
        }

        let rank_string = (rank + 1).to_string();
        let size = 0.4;

        let center_y = height + Self::RANK_HEIGHT / 2.0;

        let foreground_color = colors::GRAY;
        let background_color = colors::BLANK;

        #[rustfmt::skip]
        {
            draw_boxed_text(&rank_string, 0.0, center_y, size, [1.0, 0.5], foreground_color, background_color);
            draw_boxed_text(&rank_string, Self::RANK_WIDTH, center_y, size, [0.0, 0.5], foreground_color, background_color);
        };
    }

    pub fn height_of_rank(&self, rank: isize) -> f32 {
        let rank = if let PieceTeam::Black = self.turn {
            self.invert_rank(rank)
        } else {
            rank
        };

        rank as f32 * Self::RANK_HEIGHT
    }

    pub fn x_position_of_file(&self, file: isize) -> f32 {
        let file = if PieceTeam::Black == self.turn {
            // if file is in the range 0..NUM_FILES, it will remain in that range when flipped
            self.invert_file(file)
        } else {
            file
        };

        file as f32 * Self::TILE_SIZE
    }
}

// Align of 0.0 means left align, align of 1.0 means right align
fn draw_boxed_text(
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    align: [f32; 2],
    foreground_color: Color,
    background_color: Color,
) {
    let (font_size, font_scale, _) = text::camera_font_scale(size);

    let horizontal_offset: f32 = size / 4.0;
    let vertical_offset: f32 = size / 4.0;

    let TextDimensions { width, .. } = text::measure_text(text, None, font_size, font_scale);

    let box_width: f32 = horizontal_offset * 2.0 + width;
    let box_height: f32 = size;

    let x = x - box_width * align[0];
    let y = y - box_height * align[1];

    if background_color != colors::BLANK {
        shapes::draw_rectangle(x, y, box_width, box_height, background_color);
    }

    text::draw_text_ex(
        &text,
        x + horizontal_offset,
        y + vertical_offset,
        TextParams {
            font_size,
            font_scale: -font_scale,
            font_scale_aspect: -1.0,
            color: foreground_color,
            ..Default::default()
        },
    );
}

static QUEEN_RANK_BLACK: Rank =
    [Some(ChessPiece::new(PieceKind::Queen, PieceTeam::Black)); NUM_FILES];

#[rustfmt::skip]
static KING_RANK_BLACK: Rank = [
    Some(ChessPiece::new(PieceKind::Rook,   PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Knight, PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Bishop, PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Queen,  PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::King,   PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Bishop, PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Knight, PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Rook,   PieceTeam::Black)),
];

#[rustfmt::skip]
static PAWN_RANK_BLACK: Rank =
    [Some(ChessPiece::new(PieceKind::Pawn { new: true }, PieceTeam::Black)); NUM_FILES];

static EMPTY_RANK: Rank = [None; NUM_FILES];

static PAWN_RANK_WHITE: Rank = invert_teams(PAWN_RANK_BLACK);
static KING_RANK_WHITE: Rank = invert_teams(KING_RANK_BLACK);
static QUEEN_RANK_WHITE: Rank = invert_teams(QUEEN_RANK_BLACK);

const fn invert_teams<const N: usize>(
    mut pieces: [Option<ChessPiece>; N],
) -> [Option<ChessPiece>; N] {
    let mut i = 0;

    while i < pieces.len() {
        if let Some(piece) = &mut pieces[i] {
            piece.team = piece.team.opposite();
        }

        i += 1;
    }

    pieces
}
