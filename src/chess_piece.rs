pub mod moves;

use crate::chess_board;

#[derive(Clone, Copy, Debug)]
pub struct ChessPiece {
    pub kind: PieceKind,
    pub team: PieceTeam,
    pub moves: u8,
}

impl ChessPiece {
    pub const fn new(kind: PieceKind, team: PieceTeam) -> Self {
        Self {
            kind,
            team,
            moves: 0,
        }
    }

    pub fn moved(self) -> Self {
        Self {
            moves: self.moves.saturating_add(1),
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
    pub additional_motion_offset: Option<[i8; 2]>,
    pub repeating: bool,
    pub can_capture: bool,
    pub can_move: bool,
    pub provokes_opportunity: bool,
    pub requires_opportunity: bool,
}

impl PieceMove {
    pub const DEFAULT: Self = Self {
        offset: [0; 2],
        additional_motion_offset: None,
        repeating: false,
        can_capture: true,
        can_move: true,
        provokes_opportunity: false,
        requires_opportunity: false,
    };

    pub fn offset(self) -> [isize; 2] {
        self.offset.map(|x| x as isize)
    }

    pub fn additional_motion_offset(self) -> Option<[isize; 2]> {
        self.additional_motion_offset.map(|a| a.map(|x| x as isize))
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

    /// Returns None if there is an overflow
    pub fn apply_additional_motion_offset_to(self, position: [isize; 2]) -> Option<[isize; 2]> {
        if let Some(offset) = self.additional_motion_offset() {
            let [Some(rank), Some(file)] = [0, 1].map(|i| position[i].checked_add(offset[i]))
            else {
                return None;
            };

            Some([rank, file])
        } else {
            Some(position)
        }
    }
}

impl Default for PieceMove {
    fn default() -> Self {
        Self::DEFAULT
    }
}
