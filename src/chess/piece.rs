#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    #[inline]
    pub const fn from_idx(idx: usize) -> Self {
        match idx {
            0 => Piece::Rook,
            1 => Piece::Knight,
            2 => Piece::Bishop,
            3 => Piece::Queen,
            4 => Piece::King,
            5 => Piece::Pawn,
            _ => panic!("invalid idx")
        }
    }

    #[inline]
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

    #[inline]
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
            Piece::Rook => "r",
            Piece::Knight => "n",
            Piece::Bishop => "b",
            Piece::Queen => "q",
            Piece::King => "k",
            Piece::Pawn => "p",
        })
    }
}