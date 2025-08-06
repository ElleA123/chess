#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,
    Black
}

pub const NUM_COLORS: usize = 2;

impl Color {
    pub const fn is_white(&self) -> bool {
        match self {
            Color::White => true,
            Color::Black => false
        }
    }

    pub const fn is_black(&self) -> bool {
        match self {
            Color::White => false,
            Color::Black => true
        }
    }
}

impl std::ops::Not for Color {
    type Output = Color;
    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn
}

pub const NUM_PIECE_TYPES: usize = 6;

impl PieceType {
    pub const fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'R' => Some(PieceType::Rook),
            'N' => Some(PieceType::Knight),
            'B' => Some(PieceType::Bishop),
            'Q' => Some(PieceType::Queen),
            'K' => Some(PieceType::King),
            'P' => Some(PieceType::Pawn),
            _ => None
        }
    }

    pub const fn from_ascii_char(c: u8) -> Option<Self> {
        match c.to_ascii_uppercase() {
            b'R' => Some(PieceType::Rook),
            b'N' => Some(PieceType::Knight),
            b'B' => Some(PieceType::Bishop),
            b'Q' => Some(PieceType::Queen),
            b'K' => Some(PieceType::King),
            b'P' => Some(PieceType::Pawn),
            _ => None
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            PieceType::Rook => "R",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Queen => "Q",
            PieceType::King => "K",
            PieceType::Pawn => "P",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color
}

pub const NUM_PIECES: usize = NUM_COLORS * NUM_PIECE_TYPES;

impl Piece {
    pub fn new(c: char) -> Option<Self> {
        Some(Piece {
                piece_type: PieceType::from_char(c)?,
                color: if c.is_ascii_uppercase() { Color::White } else { Color::Black }
            })
    }

    pub fn new_ascii(c: u8) -> Option<Self> {
        Some(Piece {
            piece_type: PieceType::from_ascii_char(c)?,
            color: if c.is_ascii_uppercase() { Color::White } else { Color::Black }
        })
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.color {
            Color::White => self.piece_type.as_str().to_owned(),
            Color::Black => self.piece_type.as_str().to_ascii_lowercase()
        })
    }
}