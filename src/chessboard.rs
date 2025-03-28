use std::collections::VecDeque;

pub const NUM_FILES: usize = 8;

#[derive(Clone, Debug)]
pub struct Chessboard {
    pub pieces: VecDeque<[Option<ChessPiece>; NUM_FILES]>,
    pub first_rank: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct ChessPiece {
    pub kind: PieceKind,
    pub team: PieceTeam,
    pub num_moves: u16,
}

impl ChessPiece {
    pub fn new(kind: PieceKind, team: PieceTeam) -> Self {
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
    pub fn pawn_upgrade_rank(self) -> usize {
        match self {
            PieceTeam::Black => 0,
            PieceTeam::White => 7,
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

const fn invert_moves<const N: usize>(mut moves: [PieceMove; N]) -> [PieceMove; N] {
    let mut i = 0;

    while i < moves.len() {
        moves[i].offset[0] = -moves[i].offset[0];

        i += 1;
    }

    moves
}
