pub mod moves;
pub mod textures;

use crate::chess_board;

#[derive(Clone, Copy, Debug)]
pub struct ChessPiece {
    pub kind: PieceKind,
    pub team: PieceTeam,
}

impl ChessPiece {
    pub const fn new(kind: PieceKind, team: PieceTeam) -> Self {
        Self { kind, team }
    }

    pub fn moved(self) -> Self {
        Self {
            kind: self.kind.moved(),
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
            PieceTeam::White => chess_board::NUM_TRADITIONAL_RANKS - 1,
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
                // check if the given offset's axis could have possibly come from this move's offset
                // by not being a multiple of this move's offset or being in the wrong direction
                if !(self.offset()[i] == 0 || offset[i] % self.offset()[i] == 0)
                    || offset[i].signum() != self.offset()[i].signum()
                {
                    return false;
                }
            }

            // if this move is vertical or horizontal, the above is enough, otherwise check that both
            // axis are the same multiple of this move's offset
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
