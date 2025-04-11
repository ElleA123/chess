#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn
}

impl PieceType {
    pub const fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'r' => Some(PieceType::Rook),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            'p' => Some(PieceType::Pawn),
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            &PieceType::Rook => "r",
            &PieceType::Knight => "n",
            &PieceType::Bishop => "b",
            &PieceType::Queen => "q",
            &PieceType::King => "k",
            &PieceType::Pawn => "p",
        }.to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: bool
}

impl Piece {
    pub const fn new(c: char) -> Option<Self> {
        if let Some(piece_type) = PieceType::from_char(c) {
            Some(Piece {
                piece_type,
                color: c.is_ascii_uppercase()
            })
        } else {
            None
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = if self.color {
            self.piece_type.to_string().to_ascii_uppercase()
        } else {
            self.piece_type.to_string()
        };
        write!(f, "{}", s)
    }
}