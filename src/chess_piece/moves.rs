use std::ptr;

use super::{ChessPiece, PieceKind, PieceMove, PieceTeam};

impl ChessPiece {
    pub fn moves(&self) -> &'static [PieceMove] {
        match self.kind {
            #[rustfmt::skip]
            PieceKind::Pawn => match self.team {
                PieceTeam::Black => if self.moves == 0 { &PAWN_MOVES_BLACK_NEW } else { &PAWN_MOVES_BLACK }
                PieceTeam::White => if self.moves == 0 { &PAWN_MOVES_WHITE_NEW } else { &PAWN_MOVES_WHITE }
            },
            PieceKind::Bishop => &BISHOP_MOVES,
            PieceKind::Knight => &KNIGHT_MOVES,
            PieceKind::Rook => &ROOK_MOVES,
            PieceKind::Queen => &QUEEN_MOVES,
            PieceKind::King => &KING_MOVES,
        }
    }

    /// WARN: This does not compare any underlying data, only whether or not it is a reference to
    /// the same static as self.moves() would return.
    pub fn is_moveset_from_same_reference(&self, moves: &[PieceMove]) -> bool {
        ptr::eq(self.moves(), moves)
    }
}

pub static ALL_MOVES: [&[PieceMove]; 9] = [
    &PAWN_MOVES_BLACK,
    &PAWN_MOVES_BLACK_NEW,
    &PAWN_MOVES_WHITE,
    &PAWN_MOVES_WHITE_NEW,
    &BISHOP_MOVES,
    &KNIGHT_MOVES,
    &ROOK_MOVES,
    &QUEEN_MOVES,
    &KING_MOVES,
];

#[rustfmt::skip]
static PAWN_MOVES_BLACK: [PieceMove; 3] = [
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true,  can_move: false, provokes_opportunity: false },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: false, can_move: true,  provokes_opportunity: false },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true,  can_move: false, provokes_opportunity: false },
];

#[rustfmt::skip]
static PAWN_MOVES_BLACK_NEW: [PieceMove; 4] = [
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true,  can_move: false, provokes_opportunity: false },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: false, can_move: true,  provokes_opportunity: false },
    PieceMove { offset: [-2, 0],  repeating: false, can_capture: false, can_move: true,  provokes_opportunity: true },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true,  can_move: false, provokes_opportunity: false },
];

static PAWN_MOVES_WHITE: [PieceMove; 3] = invert_moves(PAWN_MOVES_BLACK);

static PAWN_MOVES_WHITE_NEW: [PieceMove; 4] = invert_moves(PAWN_MOVES_BLACK_NEW);

#[rustfmt::skip]
static BISHOP_MOVES: [PieceMove; 4] = [
    PieceMove { offset: [1, 1],   repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [1, -1],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 1],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, -1], repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
];

#[rustfmt::skip]
static KNIGHT_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [1, 2],   repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [2, 1],   repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [1, -2],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [2, -1],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 2],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-2, 1],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, -2], repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-2, -1], repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
];

#[rustfmt::skip]
static ROOK_MOVES: [PieceMove; 4] = [
    PieceMove { offset: [1, 0],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [0, 1],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 0], repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [0, -1], repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
];

#[rustfmt::skip]
static QUEEN_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [1, 0],   repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [1, 1],   repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [0, 1],   repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 1],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 0],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, -1], repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [0, -1],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [1, -1],  repeating: true, can_capture: true, can_move: true, provokes_opportunity: false },
];

#[rustfmt::skip]
static KING_MOVES: [PieceMove; 8] = [
    PieceMove { offset: [1, 0],   repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [1, 1],   repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [0, 1],   repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 1],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, 0],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [-1, -1], repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [0, -1],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
    PieceMove { offset: [1, -1],  repeating: false, can_capture: true, can_move: true, provokes_opportunity: false },
];

const fn invert_moves<const N: usize>(mut moves: [PieceMove; N]) -> [PieceMove; N] {
    let mut i = 0;

    while i < moves.len() {
        moves[i].offset[0] = -moves[i].offset[0];

        i += 1;
    }

    moves
}
