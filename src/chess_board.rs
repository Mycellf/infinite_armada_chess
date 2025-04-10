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

use crate::chess_piece::{self, ChessPiece, PieceKind, PieceMove, PieceTeam};

pub const NUM_FILES: usize = 8;
pub const NUM_TRADITIONAL_RANKS: usize = 8;

pub type Rank = [Option<ChessPiece>; NUM_FILES];

#[derive(Clone, Debug)]
pub struct ChessBoard {
    pub ranks: VecDeque<Rank>,
    pub ranks_behind_white: usize,
    pub turn: PieceTeam,
    pub king_positions: [[isize; 2]; 2],
    pub opportunity_location: Option<[isize; 2]>,
    pub selection_mode: SelectionMode,
}

#[derive(Clone, Copy, Debug)]
pub enum SelectionMode {
    MovePiece,
    PromotePiece([isize; 2]),
}

impl Default for ChessBoard {
    fn default() -> Self {
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
            king_positions: [[7, 4], [0, 4]],
            opportunity_location: None,
            selection_mode: SelectionMode::MovePiece,
        }
    }
}

impl ChessBoard {
    // Returns true if the camera should be flipped
    #[must_use]
    pub fn move_piece(&mut self, from: [isize; 2], to: [isize; 2]) -> Option<bool> {
        let SelectionMode::MovePiece = self.selection_mode else {
            return None;
        };

        let turn = self.turn;

        let starting_piece = self.get_piece(from)??;

        if starting_piece.team != turn {
            return None;
        }

        let piece_move = self.check_move(from, to)?;

        if self.king_is_in_check_with_move(from, to, Some(piece_move)) {
            return None;
        }

        if !piece_move.allowed_in_check && self.king_is_in_check() {
            return None;
        }

        if let Some(destination) = piece_move.apply_captured_piece_offset_to_origin(from) {
            let captured_tile = self.get_piece(to)?;

            let ending_tile = self.get_piece_expanding(destination)?;

            let captured_piece = captured_tile?;

            *ending_tile = Some(captured_piece.moved());
        }

        if piece_move.forced_motion_offset().is_some() {
            let captured_tile = self.get_piece_expanding(to)?;

            *captured_tile = None;
        }

        let destination = piece_move
            .apply_additional_motion_offset_to_move(from, to)
            .unwrap();

        let ending_tile = self.get_piece_expanding(destination)?;

        *ending_tile = Some(starting_piece.moved());
        if let PieceKind::King = starting_piece.kind {
            *self.get_king_position_mut() = destination;
        }

        let starting_tile = self
            .get_piece_expanding(from)
            .expect("Starting tile should already checked to be valid");

        *starting_tile = None;

        if piece_move.provokes_opportunity {
            self.opportunity_location = Some(destination);
        } else {
            self.opportunity_location = None;
        }

        if Some(destination[0]) == starting_piece.upgrade_rank() {
            self.selection_mode = SelectionMode::PromotePiece(destination);
            Some(false)
        } else {
            self.turn = self.turn.opposite();
            Some(true)
        }
    }

    #[must_use]
    pub fn select_promotion(&mut self, index: usize) -> Option<()> {
        let SelectionMode::PromotePiece(location) = self.selection_mode else {
            return None;
        };

        let selected_piece = self.get_piece_mut(location)?.as_mut()?;

        let upgrade_kinds = (selected_piece.upgrade_kinds())
            .expect("The piece being promoted should have a valid set of upgrades.");

        if index >= upgrade_kinds.len() {
            return None;
        }

        selected_piece.kind = upgrade_kinds[index];

        self.turn = self.turn.opposite();
        self.selection_mode = SelectionMode::MovePiece;
        Some(())
    }

    pub fn check_move(&self, from: [isize; 2], to: [isize; 2]) -> Option<PieceMove> {
        let starting_piece = self.get_piece(from)??;

        let destination_tile = self.get_piece(to)?;

        // HACK: There is an overflow if the player attempts to make a move with magnitude more
        // than isize can fit. If such an overflow happens, the game would need to be big enough
        // to fit at least 2^31 ranks (at least 34 gigabytes of data, which 32 bit machines
        // cannot index). In this case, the function gives up and returns false.
        let [Some(rank_offset), Some(file_offset)] = [0, 1].map(|i| to[i].checked_sub(from[i]))
        else {
            return None;
        };

        let offset = [rank_offset, file_offset];

        // find the move being referenced
        let piece_move = *(starting_piece.moves().iter())
            .find(|&&piece_move| piece_move.is_offset_valid(offset))?;

        if piece_move.requires_opportunity && self.opportunity_location != Some(to) {
            return None;
        }

        // check that the destination is valid
        if !(destination_tile.is_some() && piece_move.can_capture
            || destination_tile.is_none() && piece_move.can_move)
        {
            return None;
        }

        if piece_move.pieces_must_be_new && starting_piece.moves != 0 {
            return None;
        }

        // check if the destination tile's contents are valid
        if let Some(ChessPiece { kind, team, moves }) = destination_tile {
            if team == starting_piece.team && !piece_move.can_capture_ally {
                return None;
            }

            if let Some(capture_kind) = piece_move.forced_capture_kind {
                if kind != capture_kind {
                    return None;
                }
            }

            if piece_move.pieces_must_be_new && moves != 0 {
                return None;
            }
        }

        if piece_move.forced_motion_offset().is_some() {
            // HACK: See above note about overflows
            let destination = piece_move.apply_additional_motion_offset_to_move(from, to)?;

            let Some(None) = self.get_piece(destination) else {
                return None;
            };
        }

        if piece_move.captured_piece_offset().is_some() {
            // HACK: See above note about overflows
            let captured_piece_destination =
                piece_move.apply_captured_piece_offset_to_origin(from)?;

            let Some(None) = self.get_piece(captured_piece_destination) else {
                return None;
            };
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
                    return None;
                }
            }
        }

        Some(piece_move)
    }

    pub fn king_is_in_check(&self) -> bool {
        self.king_is_in_check_with_move([0, 0], [0, 0], None)
    }

    pub fn king_is_in_check_with_move(
        &self,
        from: [isize; 2],
        to: [isize; 2],
        piece_move: Option<PieceMove>,
    ) -> bool {
        let destination = if let Some(piece_move) = piece_move {
            piece_move
                .apply_additional_motion_offset_to_move(from, to)
                .unwrap()
        } else {
            to
        };

        let captured_destination =
            piece_move.map(|m| m.apply_captured_piece_offset_to_origin(from));

        let map_tile = |tile| {
            if tile == destination {
                Some(from)
            } else if Some(Some(tile)) == captured_destination {
                Some(to)
            } else if tile == from || tile == to {
                None
            } else {
                Some(tile)
            }
        };

        let get_piece = |tile| {
            if let Some(tile) = map_tile(tile) {
                self.get_piece(tile)
            } else {
                Some(None)
            }
        };

        let king_position = self.get_king_position();

        #[rustfmt::skip]
        let king_position = if king_position == from { destination } else { king_position };

        for move_kind in chess_piece::moves::ALL_MOVES {
            for potential_move in move_kind
                .iter()
                .filter(|potential_move| potential_move.can_capture)
            {
                let mut move_position = king_position;
                let offset = potential_move.offset();

                'outer: loop {
                    for i in [0, 1] {
                        if let Some(result) = move_position[i].checked_sub(offset[i]) {
                            move_position[i] = result;
                        } else {
                            break 'outer;
                        }
                    }

                    if let Some(tile) = get_piece(move_position) {
                        if let Some(piece) = tile {
                            if piece.team == self.turn.opposite()
                                && piece.is_moveset_from_same_reference(move_kind)
                            {
                                return true;
                            }

                            break;
                        }
                    } else {
                        break;
                    }

                    if !potential_move.repeating {
                        break;
                    }
                }
            }
        }

        false
    }
}

impl ChessBoard {
    pub const TILE_SIZE: f32 = 1.0;
    pub const RANK_HEIGHT: f32 = Self::TILE_SIZE;
    pub const RANK_WIDTH: f32 = Self::TILE_SIZE * NUM_FILES as f32;

    // CREDIT: colors from the palette used by lichess.org
    pub const DARK_TILE_COLOR: Color = Color::from_hex(0xb58863);
    pub const LIGHT_TILE_COLOR: Color = Color::from_hex(0xf0d9b5);

    pub fn draw_ranks(
        &self,
        start: f32,
        end: f32,
        offset: isize,
        highlighted_tile: Option<[isize; 2]>,
    ) {
        let lowest = start.floor() as isize;
        let highest = (end - Self::RANK_HEIGHT).ceil() as isize;

        let (start_rank, end_rank) = if self.turn == PieceTeam::Black {
            (self.invert_rank(highest), self.invert_rank(lowest))
        } else {
            (lowest, highest)
        };

        for rank in start_rank..end_rank + 1 {
            let highlighted_file = if let Some(highlighted_tile) = highlighted_tile {
                if rank.checked_add(offset) == Some(highlighted_tile[0]) {
                    Some(highlighted_tile[1])
                } else {
                    None
                }
            } else {
                None
            };

            self.draw_rank(rank, offset, highlighted_file);
        }

        #[rustfmt::skip]
        {
            shapes::draw_rectangle(-5.0, start - 1.0, Self::RANK_WIDTH + 10.0, 1.0, colors::BLACK);
            shapes::draw_rectangle(-5.0, end, Self::RANK_WIDTH + 10.0, 1.0, colors::BLACK);
        };

        if let Some(highlighted_tile) = highlighted_tile {
            let rank = highlighted_tile[0];
            let file = highlighted_tile[1];

            let file_x = self.x_position_of_file(file);

            let white_side = rank < start_rank.saturating_add(offset);
            let black_side = rank > end_rank.saturating_add(offset);

            let (above, below) = if self.turn == PieceTeam::Black {
                (white_side, black_side)
            } else {
                (black_side, white_side)
            };

            if below {
                shapes::draw_rectangle(file_x, start - 0.5, Self::TILE_SIZE, 0.5, colors::WHITE);
            } else if above {
                shapes::draw_rectangle(file_x, end, Self::TILE_SIZE, 0.5, colors::WHITE);
            }
        }

        for file in 0..NUM_FILES {
            let tile_x = self.x_position_of_file(file as isize) + Self::TILE_SIZE / 2.0;

            let file_string = ((b'a' + file as u8) as char).to_string();

            let foreground = colors::GRAY;
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

    pub fn draw_rank(&self, rank: isize, offset: isize, highlighted_file: Option<isize>) {
        let height = self.height_of_rank(rank);

        let Some(rank) = rank.checked_add(offset) else {
            return;
        };

        let mut tile_parity = rank % 2 == 0;

        let rank_contents = self.get_rank(rank);

        for (file, tile) in rank_contents.iter().enumerate() {
            let tile_x = self.x_position_of_file(file as isize);

            let tile_color = if highlighted_file == Some(file as isize) {
                colors::WHITE
            } else if tile_parity {
                Self::DARK_TILE_COLOR
            } else {
                Self::LIGHT_TILE_COLOR
            };

            shapes::draw_rectangle(tile_x, height, Self::TILE_SIZE, Self::TILE_SIZE, tile_color);

            if let Some(piece) = tile {
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
        }

        let rank_string = (rank as i128 + 1).to_string();
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

    pub fn draw_piece_selection(&self, offset: isize) {
        let SelectionMode::PromotePiece(location) = self.selection_mode else {
            return;
        };

        let selected_piece = self.get_piece(location).unwrap().unwrap();
        let upgrade_kinds = (selected_piece.upgrade_kinds())
            .expect("The piece being promoted should have a valid set of upgrades.");

        let visual_file = if PieceTeam::Black == self.turn {
            self.invert_file(location[1])
        } else {
            location[1]
        };

        let width = Self::TILE_SIZE;
        let height = upgrade_kinds.len() as f32 * Self::RANK_HEIGHT;

        let x_corner = visual_file as f32 * Self::TILE_SIZE;
        let y_corner = self.height_of_rank(location[0] - offset);

        shapes::draw_rectangle(x_corner, y_corner - height, width, height, colors::WHITE);

        for (i, &piece_kind) in upgrade_kinds.iter().enumerate() {
            let texture = ChessPiece::new(piece_kind, self.turn).texture();

            texture::draw_texture_ex(
                texture,
                x_corner,
                y_corner - Self::RANK_HEIGHT * (i + 1) as f32,
                colors::WHITE,
                DrawTextureParams {
                    dest_size: Some([Self::TILE_SIZE; 2].into()),
                    flip_y: true,
                    ..Default::default()
                },
            )
        }
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
        text,
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

impl ChessBoard {
    pub fn expand_to_rank(&mut self, rank: isize) {
        if rank < self.first_rank() {
            let additional_ranks = (self.first_rank() - rank).try_into().unwrap();
            self.ranks.reserve(additional_ranks);

            for _ in 0..additional_ranks {
                self.ranks.push_front(QUEEN_RANK_WHITE);
            }

            self.ranks_behind_white = -rank as usize;
        } else if rank > self.last_rank() {
            let additional_ranks = (rank - self.last_rank()).try_into().unwrap();
            self.ranks.reserve(additional_ranks);

            for _ in 0..additional_ranks {
                self.ranks.push_back(QUEEN_RANK_BLACK);
            }
        }
    }

    pub fn index_of_rank(&self, rank: isize) -> isize {
        // HACK: If rank is greater than isize::MAX - self.ranks_behind_white, this returns the
        // index of the last rank in stead. Under normal circumstances a game will not last long
        // enough for this to matter (32 bit isize would require a game to last at minimum 68 years
        // at a rate of one move per second)
        rank.saturating_add(self.ranks_behind_white as isize)
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

    pub fn get_king_position(&self) -> [isize; 2] {
        match self.turn {
            PieceTeam::Black => self.king_positions[0],
            PieceTeam::White => self.king_positions[1],
        }
    }

    pub fn get_king_position_mut(&mut self) -> &mut [isize; 2] {
        match self.turn {
            PieceTeam::Black => &mut self.king_positions[0],
            PieceTeam::White => &mut self.king_positions[1],
        }
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

static PAWN_RANK_BLACK: Rank =
    [Some(ChessPiece::new(PieceKind::Pawn, PieceTeam::Black)); NUM_FILES];

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
