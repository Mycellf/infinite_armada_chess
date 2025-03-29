use std::{
    collections::VecDeque,
    ops::{Index, IndexMut},
};

pub const NUM_FILES: usize = 8;
pub const NUM_TRADITIONAL_RANKS: usize = 8;

type Rank = [Option<ChessPiece>; NUM_FILES];

#[derive(Clone, Debug)]
pub struct ChessBoard {
    pub ranks: VecDeque<Rank>,
    pub ranks_behind_white: usize,
}

impl ChessBoard {
    pub fn new() -> Self {
        let mut ranks = VecDeque::with_capacity(NUM_TRADITIONAL_RANKS + 4);

        ranks.push_front(QUEEN_RANK_WHITE);
        ranks.push_front(QUEEN_RANK_WHITE);

        ranks.push_front(KING_RANK_WHITE);
        ranks.push_front(PAWN_RANK_WHITE);

        for _ in 0..NUM_TRADITIONAL_RANKS - 2 {
            ranks.push_front(EMPTY_RANK);
        }

        ranks.push_front(PAWN_RANK_BLACK);
        ranks.push_front(KING_RANK_BLACK);

        ranks.push_front(QUEEN_RANK_BLACK);
        ranks.push_front(QUEEN_RANK_BLACK);

        Self {
            ranks,
            ranks_behind_white: 2,
        }
    }

    pub fn expand_to_rank(&mut self, rank: isize) {
        let target = self.index_of_rank(rank);

        if target < self.first_rank() {
            for _ in target..self.first_rank() {
                self.ranks.push_back(QUEEN_RANK_WHITE);
            }

            self.ranks_behind_white = -target as usize;
        } else if target > self.last_rank() {
            for _ in self.last_rank()..target {
                self.ranks.push_front(QUEEN_RANK_BLACK);
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
        self.ranks.len() as isize + self.first_rank()
    }

    pub fn get_piece(&self, [rank, file]: [isize; 2]) -> Option<Option<ChessPiece>> {
        Some(*self.get_rank(rank)?.get(usize::try_from(file).ok()?)?)
    }

    pub fn get_piece_mut(&mut self, [rank, file]: [isize; 2]) -> Option<&mut Option<ChessPiece>> {
        self.get_rank_mut(rank)?
            .get_mut(usize::try_from(file).ok()?)
    }

    pub fn get_rank(&self, rank: isize) -> Option<&Rank> {
        self.ranks.get(self.index_of_rank(rank).try_into().ok()?)
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
        self.get_rank(index).unwrap()
    }
}

impl IndexMut<isize> for ChessBoard {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        self.get_rank_mut(index).unwrap()
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

    pub fn increment_moves(&mut self) {
        self.num_moves = self.num_moves.saturating_add(1);
    }

    pub fn moved(self) -> Self {
        Self {
            num_moves: self.num_moves.saturating_add(1),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum PieceKind {
    Pawn,
    Bishop,
    Knight,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug)]
pub struct PieceMove {
    pub offset: [i8; 2],
    pub repeating: bool,
    pub can_capture: bool,
    pub can_move: bool,
}

impl ChessPiece {
    pub fn moves(&self) -> &[PieceMove] {
        match self.kind {
            PieceKind::Pawn => match self.team {
                PieceTeam::Black => &PAWN_MOVES_BLACK,
                PieceTeam::White => &PAWN_MOVES_WHITE,
            },
            PieceKind::Bishop => &BISHOP_MOVES,
            PieceKind::Knight => &KNIGHT_MOVES,
            PieceKind::Rook => &ROOK_MOVES,
            PieceKind::Queen => &QUEEN_MOVES,
            PieceKind::King => &KING_MOVES,
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
    Some(ChessPiece::new(PieceKind::King,   PieceTeam::Black)),
    Some(ChessPiece::new(PieceKind::Queen,  PieceTeam::Black)),
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

#[rustfmt::skip]
static PAWN_MOVES_BLACK: [PieceMove; 3] = [
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true,  can_move: false },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: false, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true,  can_move: false },
];

static PAWN_MOVES_WHITE: [PieceMove; 3] = invert_moves(PAWN_MOVES_BLACK);

#[rustfmt::skip]
static BISHOP_MOVES: [PieceMove; 4] = [
    PieceMove { offset: [1, 1],   repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [1, -1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, 1],  repeating: true, can_capture: true, can_move: true },
    PieceMove { offset: [-1, -1], repeating: true, can_capture: true, can_move: true },
];

#[rustfmt::skip]
static KNIGHT_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [2, 3],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [3, 2],   repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [2, -3],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [3, -2],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-2, 3],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-3, 2],  repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-2, -3], repeating: false, can_capture: true, can_move: true },
    PieceMove { offset: [-3, -2], repeating: false, can_capture: true, can_move: true },
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

const fn invert_teams<const N: usize>(pieces: [Option<ChessPiece>; N]) -> [Option<ChessPiece>; N] {
    let mut i = 0;

    while i < pieces.len() {
        if let Some(mut piece) = pieces[i] {
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
