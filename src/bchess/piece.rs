#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Piece {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn
}

pub const NUM_PIECES: usize = 6;
pub const PIECES: [Piece; NUM_PIECES] = [
    Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen, Piece::King, Piece::Pawn
];

impl Piece {
    pub const fn idx(self) -> usize {
        self as usize
    }

    // pub const fn from_char(c: char) -> Option<Self> {
    //     match c.to_ascii_uppercase() {
    //         'R' => Some(Piece::Rook),
    //         'N' => Some(Piece::Knight),
    //         'B' => Some(Piece::Bishop),
    //         'Q' => Some(Piece::Queen),
    //         'K' => Some(Piece::King),
    //         'P' => Some(Piece::Pawn),
    //         _ => None
    //     }
    // }

    pub const fn from_ascii(b: u8) -> Option<Self> {
        match b.to_ascii_uppercase() {
            b'R' => Some(Piece::Rook),
            b'N' => Some(Piece::Knight),
            b'B' => Some(Piece::Bishop),
            b'Q' => Some(Piece::Queen),
            b'K' => Some(Piece::King),
            b'P' => Some(Piece::Pawn),
            _ => None
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Piece::Rook => "R",
            Piece::Knight => "N",
            Piece::Bishop => "B",
            Piece::Queen => "Q",
            Piece::King => "K",
            Piece::Pawn => "P",
        })
    }
}