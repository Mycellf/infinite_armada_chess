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

    pub fn upgrade_rank(self) -> Option<isize> {
        match self.kind {
            PieceKind::Pawn => Some(self.team.pawn_upgrade_rank()),
            _ => None,
        }
    }

    pub fn upgrade_kinds(self) -> Option<&'static [PieceKind]> {
        match self.kind {
            PieceKind::Pawn => Some(&PAWN_UPGRADES),
            _ => None,
        }
    }
}

static PAWN_UPGRADES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieceTeam {
    Black,
    White,
}

impl PieceTeam {
    pub const fn pawn_upgrade_rank(self) -> isize {
        match self {
            PieceTeam::Black => 0,
            PieceTeam::White => (chess_board::NUM_TRADITIONAL_RANKS - 1) as isize,
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
    pub forced_motion_offset: Option<[i8; 2]>,
    pub captured_piece_offset: Option<[i8; 2]>,
    pub repeating: bool,
    pub can_capture: bool,
    pub can_capture_ally: bool,
    pub can_move: bool,
    pub provokes_opportunity: bool,
    pub requires_opportunity: bool,
    pub allowed_in_check: bool,
    pub forced_capture_kind: Option<PieceKind>,
    pub pieces_must_be_new: bool,
}

impl PieceMove {
    pub const DEFAULT: Self = Self {
        offset: [0; 2],
        forced_motion_offset: None,
        captured_piece_offset: None,
        repeating: false,
        can_capture: true,
        can_capture_ally: false,
        can_move: true,
        provokes_opportunity: false,
        requires_opportunity: false,
        allowed_in_check: true,
        forced_capture_kind: None,
        pieces_must_be_new: false,
    };

    pub fn offset(self) -> [isize; 2] {
        self.offset.map(|x| x as isize)
    }

    pub fn forced_motion_offset(self) -> Option<[isize; 2]> {
        self.forced_motion_offset.map(|a| a.map(|x| x as isize))
    }

    pub fn captured_piece_offset(self) -> Option<[isize; 2]> {
        self.captured_piece_offset.map(|a| a.map(|x| x as isize))
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
    pub fn apply_additional_motion_offset_to_move(
        self,
        from: [isize; 2],
        to: [isize; 2],
    ) -> Option<[isize; 2]> {
        if let Some(offset) = self.forced_motion_offset() {
            let [Some(rank), Some(file)] = [0, 1].map(|i| from[i].checked_add(offset[i])) else {
                return None;
            };

            Some([rank, file])
        } else {
            Some(to)
        }
    }

    /// Returns None if there is an overflow or there is no offset selected
    pub fn apply_captured_piece_offset_to_origin(self, from: [isize; 2]) -> Option<[isize; 2]> {
        if let Some(offset) = self.captured_piece_offset() {
            let [Some(rank), Some(file)] = [0, 1].map(|i| from[i].checked_add(offset[i])) else {
                return None;
            };

            Some([rank, file])
        } else {
            None
        }
    }
}

impl Default for PieceMove {
    fn default() -> Self {
        Self::DEFAULT
    }
}
