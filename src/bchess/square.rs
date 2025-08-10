use crate::bchess::color::Color;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum File { A, B, C, D, E, F, G, H }

pub const NUM_FILES: usize = 8;
pub const FILES: [File; NUM_FILES] = [
    File::A, File::B, File::C, File::D, File::E, File::F, File::G, File::H, 
];

impl File {
    pub const fn from_u8(n: u8) -> Self {
        assert!(n < 8);
        match n {
            0 => File::A,
            1 => File::B,
            2 => File::C,
            3 => File::D,
            4 => File::E,
            5 => File::F,
            6 => File::G,
            7 => File::H,
            _ => unreachable!()
        }
    }

    pub const fn from_ascii(b: u8) -> Self {
        assert!(b >= b'a' && b <= b'h');
        Self::from_u8(b - b'a')
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rank { One, Two, Three, Four, Five, Six, Seven, Eight }

pub const NUM_RANKS: usize = 8;
pub const RANKS: [Rank; NUM_RANKS] = [
    Rank::One, Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven, Rank::Eight
];

impl Rank {
    pub const fn from_u8(n: u8) -> Self {
        assert!(n < 8);
        match n {
            0 => Rank::One,
            1 => Rank::Two,
            2 => Rank::Three,
            3 => Rank::Four,
            4 => Rank::Five,
            5 => Rank::Six,
            6 => Rank::Seven,
            7 => Rank::Eight,
            _ => unreachable!()
        }
    }

    pub const fn from_ascii(b: u8) -> Self {
        assert!(b >= b'1' && b <= b'8');
        Self::from_u8(b - b'1')
    }

    pub const fn up(&self) -> Option<Self> {
        match *self {
            Rank::One => Some(Rank::Two),
            Rank::Two => Some(Rank::Three),
            Rank::Three => Some(Rank::Four),
            Rank::Four => Some(Rank::Five),
            Rank::Five => Some(Rank::Six),
            Rank::Six => Some(Rank::Seven),
            Rank::Seven => Some(Rank::Eight),
            Rank::Eight => None
        }
    }

    pub const fn down(&self) -> Option<Self> {
        match *self {
            Rank::One => None,
            Rank::Two => Some(Rank::One),
            Rank::Three => Some(Rank::Two),
            Rank::Four => Some(Rank::Three),
            Rank::Five => Some(Rank::Four),
            Rank::Six => Some(Rank::Five),
            Rank::Seven => Some(Rank::Six),
            Rank::Eight => Some(Rank::Seven),
        }
    }

    pub const fn forward(&self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.up(),
            Color::Black => self.down()
        }
    }

    pub const fn backward(&self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.down(),
            Color::Black => self.up()
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Square(u8);

impl Square {
    pub const fn new(square: u8) -> Self {
        assert!(square < 64);
        Self(square)
    }

    pub const fn from_coords(file: File, rank: Rank) -> Self {
        assert!((file as u8) < 8 && (rank as u8) < 8);
        Self(8 * rank as u8 + file as u8)
    }

    pub fn from_san(san: &str) -> Option<Self> {
        let bytes = san.as_bytes();
        if !san.is_ascii() || bytes.len() != 2
        || bytes[0] < b'a' || bytes[0] > b'h'
        || bytes[1] < b'1' || bytes[1] > b'8' {
            return None;
        }

        Some(Self::from_coords(File::from_ascii(bytes[0]), Rank::from_ascii(bytes[1])))
    }

    pub const fn rank(&self) -> Rank {
        Rank::from_u8(self.0 >> 3)
    }

    pub const fn file(&self) -> File {
        File::from_u8(self.0 & 7)
    }

    pub const fn idx(&self) -> usize {
        self.0 as usize
    }

    pub const fn backward(&self, color: Color) -> Option<Self> {
        match self.rank().backward(color) {
            Some(rank) => Some(Self::from_coords(self.file(), rank)),
            None => None
        }
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}",
        (self.file() as u8 + b'a') as char,
        (self.rank() as u8 + b'1') as char)
    }
}