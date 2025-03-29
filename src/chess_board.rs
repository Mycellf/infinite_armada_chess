use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
    sync::LazyLock,
};

use macroquad::{
    color::{Color, colors},
    shapes,
    texture::{self, DrawTextureParams, FilterMode, Image, Texture2D},
};

pub const NUM_FILES: usize = 8;
pub const NUM_TRADITIONAL_RANKS: usize = 8;

type Rank = [Option<ChessPiece>; NUM_FILES];

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

        let (start, end) = if self.turn == PieceTeam::Black {
            (self.invert_rank(highest), self.invert_rank(lowest))
        } else {
            (lowest, highest)
        };

        for rank in start..end + 1 {
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
            let tile_x = {
                let file = if PieceTeam::Black == self.turn {
                    // if file is in the range 0..NUM_FILES, it will remain in that range when flipped
                    self.invert_file(file as isize) as usize
                } else {
                    file
                };

                file as f32 * Self::TILE_SIZE
            };

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
    }

    pub fn height_of_rank(&self, rank: isize) -> f32 {
        let rank = if let PieceTeam::Black = self.turn {
            self.invert_rank(rank)
        } else {
            rank
        };

        rank as f32 * Self::RANK_HEIGHT
    }
}

pub fn format_file_and_rank([rank, file]: [isize; 2]) -> String {
    format!("{}{}", ('a' as u8 + file as u8) as char, rank + 1)
}

#[derive(Clone, Copy, Debug)]
pub struct ChessPiece {
    pub kind: PieceKind,
    pub team: PieceTeam,
    pub num_moves: u16,
}

impl ChessPiece {
    pub const fn new(kind: PieceKind, team: PieceTeam) -> Self {
        Self {
            kind,
            team,
            num_moves: 0,
        }
    }

    pub fn moved(self) -> Self {
        Self {
            kind: self.kind.moved(),
            num_moves: self.num_moves.saturating_add(1),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceTeam {
    Black,
    White,
}

impl PieceTeam {
    pub const fn pawn_upgrade_rank(self) -> usize {
        match self {
            PieceTeam::Black => 0,
            PieceTeam::White => NUM_TRADITIONAL_RANKS - 1,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            PieceTeam::Black => PieceTeam::White,
            PieceTeam::White => PieceTeam::Black,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceKind {
    Pawn { new: bool },
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub fn moved(self) -> Self {
        match self {
            Self::Pawn { new: _ } => Self::Pawn { new: false },
            _ => self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PieceMove {
    pub offset: [i8; 2],
    pub repeating: bool,
    pub can_capture: bool,
    pub can_move: bool,
}

impl PieceMove {
    pub fn offset(self) -> [isize; 2] {
        self.offset.map(|x| x as isize)
    }

    pub fn is_offset_valid(self, offset: [isize; 2]) -> bool {
        if self.repeating {
            for i in [0, 1] {
                if !(self.offset()[i] == 0 || offset[i] % self.offset()[i] == 0)
                    || offset[i].signum() == self.offset()[i].signum()
                {
                    return false;
                }
            }

            if self.offset()[0] == 0 || self.offset()[1] == 0 {
                true
            } else {
                offset[0] / self.offset()[0] == offset[1] / self.offset()[1]
            }
        } else {
            self.offset() == offset
        }
    }
}

impl ChessPiece {
    pub fn moves(&self) -> &[PieceMove] {
        match self.kind {
            #[rustfmt::skip]
            PieceKind::Pawn { new } => match self.team {
                PieceTeam::Black => if new { &PAWN_MOVES_BLACK_NEW } else { &PAWN_MOVES_BLACK }
                PieceTeam::White => if new { &PAWN_MOVES_WHITE_NEW } else { &PAWN_MOVES_WHITE }
            },
            PieceKind::Bishop => &BISHOP_MOVES,
            PieceKind::Knight => &KNIGHT_MOVES,
            PieceKind::Rook => &ROOK_MOVES,
            PieceKind::Queen => &QUEEN_MOVES,
            PieceKind::King => &KING_MOVES,
        }
    }

    pub fn texture(&self) -> &Texture2D {
        match self.team {
            PieceTeam::Black => match self.kind {
                PieceKind::Pawn { new: _ } => &BLACK_PAWN_TEXTURE,
                PieceKind::Bishop => &BLACK_BISHOP_TEXTURE,
                PieceKind::Knight => &BLACK_KNIGHT_TEXTURE,
                PieceKind::Rook => &BLACK_ROOK_TEXTURE,
                PieceKind::Queen => &BLACK_QUEEN_TEXTURE,
                PieceKind::King => &BLACK_KING_TEXTURE,
            },
            PieceTeam::White => match self.kind {
                PieceKind::Pawn { new: _ } => &WHITE_PAWN_TEXTURE,
                PieceKind::Bishop => &WHITE_BISHOP_TEXTURE,
                PieceKind::Knight => &WHITE_KNIGHT_TEXTURE,
                PieceKind::Rook => &WHITE_ROOK_TEXTURE,
                PieceKind::Queen => &WHITE_QUEEN_TEXTURE,
                PieceKind::King => &WHITE_KING_TEXTURE,
            },
        }
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

#[rustfmt::skip]
static PAWN_RANK_BLACK: Rank =
    [Some(ChessPiece::new(PieceKind::Pawn { new: true }, PieceTeam::Black)); NUM_FILES];

static EMPTY_RANK: Rank = [None; NUM_FILES];

static PAWN_RANK_WHITE: Rank = invert_teams(PAWN_RANK_BLACK);
static KING_RANK_WHITE: Rank = invert_teams(KING_RANK_BLACK);
static QUEEN_RANK_WHITE: Rank = invert_teams(QUEEN_RANK_BLACK);

#[rustfmt::skip]
static PAWN_MOVES_BLACK: [PieceMove; 3] = [
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true,  can_move: false },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: false, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true,  can_move: false },
];

#[rustfmt::skip]
static PAWN_MOVES_BLACK_NEW: [PieceMove; 4] = [
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true,  can_move: false },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: false, can_move: true },
    PieceMove { offset: [-2, 0],  repeating: false, can_capture: false, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true,  can_move: false },
];

static PAWN_MOVES_WHITE: [PieceMove; 3] = invert_moves(PAWN_MOVES_BLACK);

static PAWN_MOVES_WHITE_NEW: [PieceMove; 4] = invert_moves(PAWN_MOVES_BLACK_NEW);

#[rustfmt::skip]
static BISHOP_MOVES: [PieceMove; 4] = [
    PieceMove { offset: [1, 1],   repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [1, -1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, -1], repeating: true, can_capture: true, can_move: true },
];

#[rustfmt::skip]
static KNIGHT_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [1, 2],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [2, 1],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [1, -2],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [2, -1],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 2],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-2, 1],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-1, -2], repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-2, -1], repeating: false, can_capture: true, can_move: true },
];

#[rustfmt::skip]
static ROOK_MOVES: [PieceMove; 4] = [
    PieceMove { offset: [1, 0],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [0, 1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 0], repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [0, -1], repeating: true, can_capture: true, can_move: true },
];

#[rustfmt::skip]
static QUEEN_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [1, 0],   repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [1, 1],   repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [0, 1],   repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 0],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, -1], repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [0, -1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [1, -1],  repeating: true, can_capture: true, can_move: true },
];

#[rustfmt::skip]
static KING_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [1, 0],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [1, 1],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [0, 1],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [0, -1],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [1, -1],  repeating: false, can_capture: true, can_move: true },
];

static BLACK_PAWN_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/black_pawn.png")));
static WHITE_PAWN_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/white_pawn.png")));

static BLACK_BISHOP_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/black_bishop.png")));
static WHITE_BISHOP_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/white_bishop.png")));

static BLACK_KNIGHT_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/black_knight.png")));
static WHITE_KNIGHT_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/white_knight.png")));

static BLACK_ROOK_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/black_rook.png")));
static WHITE_ROOK_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/white_rook.png")));

static BLACK_QUEEN_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/black_queen.png")));
static WHITE_QUEEN_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/white_queen.png")));

static BLACK_KING_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/black_king.png")));
static WHITE_KING_TEXTURE: LazyLock<Texture2D> =
    LazyLock::new(|| texture_from_bytes(include_bytes!("../textures/pieces/white_king.png")));

fn texture_from_bytes(bytes: &[u8]) -> Texture2D {
    let texture = Texture2D::from_image(&Image::from_file_with_format(bytes, None).unwrap());

    texture.set_filter(FilterMode::Nearest);

    texture
}

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

const fn invert_moves<const N: usize>(mut moves: [PieceMove; N]) -> [PieceMove; N] {
    let mut i = 0;

    while i < moves.len() {
        moves[i].offset[0] = -moves[i].offset[0];

        i += 1;
    }

    moves
}
