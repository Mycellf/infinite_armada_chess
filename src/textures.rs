use std::sync::LazyLock;

use macroquad::texture::{FilterMode, Image, Texture2D};

use crate::chess_piece::{ChessPiece, PieceKind, PieceTeam};

impl ChessPiece {
    pub fn texture(&self) -> &'static Texture2D {
        match self.team {
            PieceTeam::Black => match self.kind {
                PieceKind::Pawn => &BLACK_PAWN_TEXTURE,
                PieceKind::Bishop => &BLACK_BISHOP_TEXTURE,
                PieceKind::Knight => &BLACK_KNIGHT_TEXTURE,
                PieceKind::Rook => &BLACK_ROOK_TEXTURE,
                PieceKind::Queen => &BLACK_QUEEN_TEXTURE,
                PieceKind::King => &BLACK_KING_TEXTURE,
            },
            PieceTeam::White => match self.kind {
                PieceKind::Pawn => &WHITE_PAWN_TEXTURE,
                PieceKind::Bishop => &WHITE_BISHOP_TEXTURE,
                PieceKind::Knight => &WHITE_KNIGHT_TEXTURE,
                PieceKind::Rook => &WHITE_ROOK_TEXTURE,
                PieceKind::Queen => &WHITE_QUEEN_TEXTURE,
                PieceKind::King => &WHITE_KING_TEXTURE,
            },
        }
    }
}

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
