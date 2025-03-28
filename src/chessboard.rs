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
}

#[derive(Clone, Copy, Debug)]
pub enum PieceTeam {
    Black,
    White,
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
